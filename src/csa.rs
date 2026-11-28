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

/// What a CSA position cannot hold, rejected here. `From<Position> for Initial` cannot
/// fail, and would leave such an entry out or read it as something else:
/// - a piece in hand is one of FU to HI, or `AL` for the rest of them;
/// - `AL` stands only in hand, and only once;
/// - a `PI` square names the piece the 平手 layout has there.
///
/// The squares are checked by [`validate_squares`] first.
fn validate_position(pos: &Position) -> Result<(), ParseError> {
    use csa::PieceType::*;
    let in_hand = |sq: csa::Square| sq.file == 0 && sq.rank == 0;
    for &(_, sq, pt) in &pos.add_pieces {
        if in_hand(sq)
            && !matches!(
                pt,
                Pawn | Lance | Knight | Silver | Gold | Bishop | Rook | All
            )
        {
            return Err(ParseError::CsaConvert(
                "a king or a promoted piece cannot be in hand",
            ));
        }
    }
    let on_board = pos.add_pieces.iter().filter(|&&(_, sq, _)| !in_hand(sq));
    if on_board.map(|&(_, _, pt)| pt).any(|pt| matches!(pt, All))
        || pos
            .bulk
            .iter()
            .flatten()
            .flatten()
            .any(|cell| matches!(cell, Some((_, All))))
    {
        return Err(ParseError::CsaConvert(
            "`AL` stands only for pieces in hand",
        ));
    }
    if pos
        .add_pieces
        .iter()
        .filter(|&&(_, _, pt)| matches!(pt, All))
        .count()
        > 1
    {
        return Err(ParseError::CsaConvert(
            "`AL` gives the rest of the pieces to one side only",
        ));
    }
    for &(sq, pt) in &pos.drop_pieces {
        let there = HIRATE_BOARD
            .get(usize::from(sq.file).wrapping_sub(1))
            .and_then(|file| file.get(usize::from(sq.rank).wrapping_sub(1)))
            .and_then(|p| p.kind);
        if there.is_none() || Kind::try_from(pt).ok() != there {
            return Err(ParseError::CsaConvert(
                "a `PI` square names a piece the 平手 layout does not have there",
            ));
        }
    }
    Ok(())
}

impl TryFrom<GameRecord> for JsonKifuFormat {
    type Error = ParseError;

    fn try_from(record: GameRecord) -> Result<Self, Self::Error> {
        validate_squares(&record)?;
        validate_position(&record.start_pos)?;
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

/// The board is the `P1`..`P9` grid if there is one, else the 平手 layout less the `PI`
/// squares, with every `P+`/`P-` piece put on top. A position given only by `P+`/`P-`
/// lines starts from an empty board, which a bare `PI` line cannot be told apart from.
/// `AL` gives its side the pieces that are neither on the board nor in either hand.
///
/// This conversion cannot fail, so it leaves out what a CSA position cannot hold, which
/// [`parse_csa_str`](crate::parser::parse_csa_str) rejects: a king or a promoted piece in
/// hand, a second `AL`, and an `AL` on the board, which comes out as a square with a
/// colour and no piece.
impl From<Position> for Initial {
    fn from(pos: Position) -> Self {
        let (hand_pieces, placements): (Vec<_>, Vec<_>) = pos
            .add_pieces
            .into_iter()
            .partition(|&(_, sq, _)| sq.file == 0 && sq.rank == 0);
        // Color
        let color = pos.side_to_move.into();
        // Board
        let mut board = if let Some(grid) = pos.bulk {
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
        } else if !pos.drop_pieces.is_empty() || placements.is_empty() {
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
            [[Piece::empty(); 9]; 9]
        };
        for &(c, sq, pt) in &placements {
            if let Some(p) = square_mut(&mut board, sq) {
                *p = Piece::from((c, pt));
            }
        }
        // Hands: the pieces listed first, then `AL` gives one side what is left.
        let mut rest = Hand {
            FU: 18,
            KY: 4,
            KE: 4,
            GI: 4,
            KI: 4,
            KA: 2,
            HI: 2,
        };
        for row in &board {
            for col in row {
                if let Some(unpromoted) = col.kind.map(Kind::unpromoted) {
                    if unpromoted != Kind::OU {
                        rest.decrement(unpromoted);
                    }
                }
            }
        }
        let mut hands = [Hand::default(); 2];
        let mut al = None;
        for &(c, _, pt) in &hand_pieces {
            let index = Into::<Color>::into(c) as usize;
            match Kind::try_from(pt) {
                Ok(kind) if kind == Kind::OU || kind.unpromoted() != kind => {}
                Ok(kind) => {
                    hands[index].increment(kind);
                    rest.decrement(kind);
                }
                // In case PieceType::All
                Err(_) => {
                    al.get_or_insert(index);
                }
            }
        }
        if let Some(index) = al {
            hands[index].add(&rest);
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
