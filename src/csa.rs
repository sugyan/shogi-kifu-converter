use crate::error::ConvertError;
use crate::jkf::*;
use crate::normalizer::HIRATE_BOARD;
use csa::{GameRecord, Position};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsaConvertError {
    #[error("Zero Square")]
    SquareZero,
    #[error("AL PieceType")]
    PieceTypeAll,
}

impl TryFrom<GameRecord> for JsonKifuFormat {
    type Error = ConvertError;

    fn try_from(record: GameRecord) -> Result<Self, Self::Error> {
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

impl From<Position> for Initial {
    fn from(mut pos: Position) -> Self {
        let mut pieces = [
            (Kind::FU, 18),
            (Kind::KY, 4),
            (Kind::KE, 4),
            (Kind::GI, 4),
            (Kind::KI, 4),
            (Kind::KA, 2),
            (Kind::HI, 2),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
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
                b[sq.file as usize - 1][sq.rank as usize - 1] = Piece::empty()
            }
            b
        } else {
            // 駒別単独表現
            let mut b = [[Piece::empty(); 9]; 9];
            for &(c, sq, pt) in &pos.add_pieces {
                b[sq.file as usize - 1][sq.rank as usize - 1] = Piece::from((c, pt));
            }
            b
        };
        for row in &board {
            for col in row {
                if let Some(unpromoted) = col.kind.map(Kind::unpromoted) {
                    if unpromoted != Kind::OU {
                        *pieces.get_mut(&unpromoted).unwrap() -= 1;
                    }
                }
            }
        }
        // Hands
        let mut hands = [Hand::default(); 2];
        for &(c, pt) in &hand_pieces {
            let index = Into::<Color>::into(c) as usize;
            if let Ok(kind) = pt.try_into() {
                hands[index].add(kind);
            } else {
                for (&kind, &num) in &pieces {
                    (0..num).for_each(|_| hands[index].add(kind));
                }
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
    type Error = CsaConvertError;

    fn try_from(m: csa::MoveRecord) -> Result<Self, Self::Error> {
        let time = m.time.map(|d| Time {
            now: d.into(),
            total: TimeFormat::default(),
        });
        match m.action {
            csa::Action::Move(c, from, to, pt) => Ok(MoveFormat {
                move_: Some(MoveMoveFormat {
                    color: c.into(),
                    piece: pt.try_into()?,
                    from: from.try_into().ok(),
                    to: to.try_into()?,
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
        let s = d.as_secs();
        let m = (s / 60) % 60;
        let h = s / 3600;
        TimeFormat {
            h: if h > 0 { Some(h as u8) } else { None },
            m: m as u8,
            s: (s % 60) as u8,
        }
    }
}

impl TryFrom<csa::Square> for PlaceFormat {
    type Error = CsaConvertError;

    fn try_from(sq: csa::Square) -> Result<Self, Self::Error> {
        if sq.file == 0 && sq.rank == 0 {
            Err(CsaConvertError::SquareZero)
        } else {
            Ok(PlaceFormat {
                x: sq.file,
                y: sq.rank,
            })
        }
    }
}

impl TryFrom<csa::PieceType> for Kind {
    type Error = CsaConvertError;

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
            csa::PieceType::All => Err(CsaConvertError::PieceTypeAll),
        }
    }
}
