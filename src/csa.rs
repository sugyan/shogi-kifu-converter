use crate::error::ParseError;
use crate::jkf::*;
use crate::normalizer::{hms, HIRATE_BOARD};
use csa::{GameRecord, Position};
use std::collections::HashMap;
use std::time::Duration;

/// `csa::Square` is built from two bare digits with no range check, so a malformed
/// record can name a square off the board. `(0, 0)` is the legal encoding for a piece
/// in hand, and for the from-square of a drop; anything else outside 1..=9 is rejected
/// here rather than indexing a board or being taken for a drop.
fn validate_squares(record: &GameRecord) -> Result<(), ParseError> {
    let on_board = |sq: &csa::Square| (1..=9).contains(&sq.file) && (1..=9).contains(&sq.rank);
    let in_hand = |sq: &csa::Square| sq.file == 0 && sq.rank == 0;
    let pos = &record.start_pos;
    if pos
        .add_pieces
        .iter()
        .any(|(_, sq, _)| !on_board(sq) && !in_hand(sq))
        || pos.drop_pieces.iter().any(|(sq, _)| !on_board(sq))
        || record.moves.iter().any(|m| match &m.action {
            csa::Action::Move(_, from, to, _) => {
                (!on_board(from) && !in_hand(from)) || !on_board(to)
            }
            _ => false,
        })
    {
        return Err(ParseError::CsaConvert("square is off the board"));
    }
    Ok(())
}

/// A piece in hand is one of FU to HI, or `AL` for the rest of them. A king or a promoted
/// piece at `00` is rejected here; `From<Position> for Initial` leaves one out.
fn validate_hands(record: &GameRecord) -> Result<(), ParseError> {
    use csa::PieceType::*;
    if record.start_pos.add_pieces.iter().any(|&(_, sq, pt)| {
        sq.file == 0
            && sq.rank == 0
            && !matches!(
                pt,
                Pawn | Lance | Knight | Silver | Gold | Bishop | Rook | All
            )
    }) {
        return Err(ParseError::CsaConvert(
            "a king or a promoted piece cannot be in hand",
        ));
    }
    Ok(())
}

impl TryFrom<GameRecord> for JsonKifuFormat {
    type Error = ParseError;

    fn try_from(record: GameRecord) -> Result<Self, Self::Error> {
        validate_squares(&record)?;
        validate_hands(&record)?;
        // Header
        let mut header = HashMap::new();
        if let Some(s) = record.black_player {
            header.insert(String::from("先手"), s);
        }
        if let Some(s) = record.white_player {
            header.insert(String::from("後手"), s);
        }
        if let Some(s) = record.event {
            header.insert(String::from("棋戦"), s);
        }
        if let Some(s) = record.site {
            header.insert(String::from("場所"), s);
        }
        if let Some(t) = record.start_time {
            header.insert(String::from("開始日時"), t.to_string());
        }
        if let Some(t) = record.end_time {
            header.insert(String::from("終了日時"), t.to_string());
        }
        if let Some(t) = record.time_limit {
            header.insert(String::from("持ち時間"), t.to_string());
        }
        if let Some(s) = record.opening {
            header.insert(String::from("戦型"), s);
        }
        // Initial
        let initial = Some(record.start_pos.into());
        // Moves
        let mut moves = vec![MoveFormat::default()];
        for m in record.moves {
            moves.push(m.try_into()?);
        }
        Ok(Self {
            header,
            initial,
            moves,
        })
    }
}

/// Off-board squares are rejected by [`validate_squares`] before this module builds a
/// board, but this impl is reachable on its own, so it must not index out of range.
fn square_mut(board: &mut [[Piece; 9]; 9], sq: csa::Square) -> Option<&mut Piece> {
    let (file, rank) = (usize::from(sq.file), usize::from(sq.rank));
    board
        .get_mut(file.checked_sub(1)?)?
        .get_mut(rank.checked_sub(1)?)
}

impl From<Position> for Initial {
    fn from(mut pos: Position) -> Self {
        let mut all_pieces = Hand {
            FU: 18,
            KY: 4,
            KE: 4,
            GI: 4,
            KI: 4,
            KA: 2,
            HI: 2,
        };
        // split to hands' and board's
        let mut hand_pieces = Vec::new();
        pos.add_pieces.retain(|&(c, sq, pt)| {
            if sq.file == 0 && sq.rank == 0 {
                hand_pieces.push((c, pt));
                false
            } else {
                true
            }
        });
        // Color
        let color = pos.side_to_move.into();
        // Board
        let board = if let Some(grid) = pos.bulk {
            // 一括表現
            let mut b = [[Piece::empty(); 9]; 9];
            for (i, row) in grid.iter().enumerate() {
                for (j, &col) in row.iter().enumerate() {
                    b[8 - j][i] = if let Some((c, pt)) = col {
                        Piece::from((c, pt))
                    } else {
                        Piece::empty()
                    };
                }
            }
            b
        } else if pos.add_pieces.is_empty() {
            // 平手初期配置と駒落ち
            let mut b = HIRATE_BOARD;
            for &(sq, _) in &pos.drop_pieces {
                if let Some(p) = square_mut(&mut b, sq) {
                    *p = Piece::empty();
                }
            }
            b
        } else {
            // 駒別単独表現
            let mut b = [[Piece::empty(); 9]; 9];
            for &(c, sq, pt) in &pos.add_pieces {
                if let Some(p) = square_mut(&mut b, sq) {
                    *p = Piece::from((c, pt));
                }
            }
            b
        };
        for row in &board {
            for col in row {
                if let Some(unpromoted) = col.kind.map(Kind::unpromoted) {
                    if unpromoted != Kind::OU {
                        all_pieces.decrement(unpromoted);
                    }
                }
            }
        }
        // Hands
        let mut hands = [Hand::default(); 2];
        for &(c, pt) in &hand_pieces {
            let index = Into::<Color>::into(c) as usize;
            match Kind::try_from(pt) {
                // A king or a promoted piece cannot be in hand. `parse_csa_str` rejects one,
                // and this conversion, which cannot fail, leaves it out.
                Ok(kind) if kind == Kind::OU || kind.unpromoted() != kind => {}
                Ok(kind) => hands[index].increment(kind),
                // In case PieceType::All
                Err(_) => hands[index] = all_pieces,
            }
        }
        Self {
            preset: Preset::PresetOther,
            data: Some(StateFormat {
                color,
                board,
                hands,
            }),
        }
    }
}

impl TryFrom<csa::MoveRecord> for MoveFormat {
    type Error = ParseError;

    fn try_from(m: csa::MoveRecord) -> Result<Self, Self::Error> {
        let time = m.time.map(|d| Time {
            now: d.into(),
            total: TimeFormat::default(),
        });
        match m.action {
            csa::Action::Move(c, from, to, pt) => Ok(MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: c.into(),
                    from: Some(from.into()),
                    to: to.into(),
                    piece: pt.try_into()?,
                    same: None,
                    promote: None,
                    capture: None,
                    relative: None,
                }),
                time,
                ..Default::default()
            }),
            csa::Action::Toryo => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialToryo),
                ..Default::default()
            }),
            csa::Action::Chudan => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialChudan),
                ..Default::default()
            }),
            csa::Action::Sennichite => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialSennichite),
                ..Default::default()
            }),
            csa::Action::TimeUp => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialTimeUp),
                ..Default::default()
            }),
            csa::Action::IllegalMove => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialIllegalMove),
                ..Default::default()
            }),
            csa::Action::IllegalAction(csa::Color::Black) => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialIllegalActionBlack),
                ..Default::default()
            }),
            csa::Action::IllegalAction(csa::Color::White) => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialIllegalActionWhite),
                ..Default::default()
            }),
            csa::Action::Jishogi => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialJishogi),
                ..Default::default()
            }),
            csa::Action::Kachi => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialKachi),
                ..Default::default()
            }),
            csa::Action::Hikiwake => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialHikiwake),
                ..Default::default()
            }),
            csa::Action::Matta => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialMatta),
                ..Default::default()
            }),
            csa::Action::Tsumi => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialTsumi),
                ..Default::default()
            }),
            csa::Action::Fuzumi => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialFuzumi),
                ..Default::default()
            }),
            csa::Action::Error => Ok(MoveFormat {
                time,
                special: Some(MoveSpecial::SpecialError),
                ..Default::default()
            }),
        }
    }
}

impl From<(csa::Color, csa::PieceType)> for Piece {
    fn from((c, pt): (csa::Color, csa::PieceType)) -> Self {
        Piece {
            color: Some(c.into()),
            kind: pt.try_into().ok(),
        }
    }
}

impl From<csa::Color> for Color {
    fn from(c: csa::Color) -> Self {
        match c {
            csa::Color::Black => Color::Black,
            csa::Color::White => Color::White,
        }
    }
}

impl From<Duration> for TimeFormat {
    fn from(d: Duration) -> Self {
        let (h, m, s) = hms(d.as_secs());
        TimeFormat {
            h: (h > 0).then_some(h),
            m,
            s,
        }
    }
}

impl From<csa::Square> for PlaceFormat {
    fn from(sq: csa::Square) -> Self {
        PlaceFormat {
            x: sq.file,
            y: sq.rank,
        }
    }
}

impl TryFrom<csa::PieceType> for Kind {
    type Error = ParseError;

    fn try_from(pt: csa::PieceType) -> Result<Self, Self::Error> {
        match pt {
            csa::PieceType::Pawn => Ok(Kind::FU),
            csa::PieceType::Lance => Ok(Kind::KY),
            csa::PieceType::Knight => Ok(Kind::KE),
            csa::PieceType::Silver => Ok(Kind::GI),
            csa::PieceType::Gold => Ok(Kind::KI),
            csa::PieceType::Bishop => Ok(Kind::KA),
            csa::PieceType::Rook => Ok(Kind::HI),
            csa::PieceType::King => Ok(Kind::OU),
            csa::PieceType::ProPawn => Ok(Kind::TO),
            csa::PieceType::ProLance => Ok(Kind::NY),
            csa::PieceType::ProKnight => Ok(Kind::NK),
            csa::PieceType::ProSilver => Ok(Kind::NG),
            csa::PieceType::Horse => Ok(Kind::UM),
            csa::PieceType::Dragon => Ok(Kind::RY),
            csa::PieceType::All => {
                Err(ParseError::CsaConvert("`AL` cannot be converted to `Kind`"))
            }
        }
    }
}
