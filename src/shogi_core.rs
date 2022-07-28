use crate::jkf;
use shogi_core::{Color, PartialPosition, Piece, PieceKind, Square};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreConvertError {
    #[error("Invalid initial board: no data with preset `OTHER`")]
    InitialBoardNoDataWithPresetOTHER,
    #[error("Invalid initial hands: {0:?}")]
    InitialHands(jkf::Kind),
    #[error("Invalid Place: {0:?}")]
    InvalidPlace((u8, u8)),
}

impl From<jkf::Color> for Color {
    fn from(c: jkf::Color) -> Self {
        match c {
            jkf::Color::Black => Color::Black,
            jkf::Color::White => Color::White,
        }
    }
}

impl From<jkf::Kind> for shogi_core::PieceKind {
    fn from(piece: jkf::Kind) -> Self {
        match piece {
            jkf::Kind::FU => PieceKind::Pawn,
            jkf::Kind::KY => PieceKind::Lance,
            jkf::Kind::KE => PieceKind::Knight,
            jkf::Kind::GI => PieceKind::Silver,
            jkf::Kind::KI => PieceKind::Gold,
            jkf::Kind::KA => PieceKind::Bishop,
            jkf::Kind::HI => PieceKind::Rook,
            jkf::Kind::OU => PieceKind::King,
            jkf::Kind::TO => PieceKind::ProPawn,
            jkf::Kind::NY => PieceKind::ProLance,
            jkf::Kind::NK => PieceKind::ProKnight,
            jkf::Kind::NG => PieceKind::ProSilver,
            jkf::Kind::UM => PieceKind::ProBishop,
            jkf::Kind::RY => PieceKind::ProRook,
        }
    }
}

impl From<jkf::MoveMoveFormat> for Piece {
    fn from(mmf: jkf::MoveMoveFormat) -> Self {
        Piece::new(mmf.piece.into(), mmf.color.into())
    }
}

impl TryFrom<jkf::PlaceFormat> for Square {
    type Error = CoreConvertError;

    fn try_from(pf: jkf::PlaceFormat) -> Result<Self, Self::Error> {
        Square::new(pf.x, pf.y).ok_or(CoreConvertError::InvalidPlace((pf.x, pf.y)))
    }
}

impl TryFrom<jkf::Initial> for PartialPosition {
    type Error = CoreConvertError;

    fn try_from(initial: jkf::Initial) -> Result<Self, Self::Error> {
        match initial.preset {
            jkf::Preset::PresetHirate => Ok(PartialPosition::startpos()),
            jkf::Preset::PresetOther => {
                let data = initial
                    .data
                    .ok_or(CoreConvertError::InitialBoardNoDataWithPresetOTHER)?;
                let mut pos = PartialPosition::empty();
                // Board
                for (i, v) in data.board.iter().enumerate() {
                    for (j, p) in v.iter().enumerate() {
                        let sq = jkf::PlaceFormat {
                            x: i as u8 + 1,
                            y: j as u8 + 1,
                        }
                        .try_into()?;
                        if let (Some(kind), Some(color)) = (p.kind, p.color) {
                            pos.piece_set(sq, Some(Piece::new(kind.into(), color.into())));
                        }
                    }
                }
                // Hands
                for (hand, c) in data.hands.iter().zip(Color::all()) {
                    let h = pos.hand_of_a_player_mut(c);
                    for (num, pk) in [
                        (hand.FU, PieceKind::Pawn),
                        (hand.KY, PieceKind::Lance),
                        (hand.KE, PieceKind::Knight),
                        (hand.GI, PieceKind::Silver),
                        (hand.KI, PieceKind::Gold),
                        (hand.KA, PieceKind::Bishop),
                        (hand.HI, PieceKind::Rook),
                    ] {
                        for _ in 0..num {
                            *h = h
                                .added(pk)
                                .ok_or_else(|| CoreConvertError::InitialHands(pk.into()))?;
                        }
                    }
                }
                // Color
                pos.side_to_move_set(data.color.into());
                Ok(pos)
            }
            _ => {
                let mut pos = PartialPosition::startpos();
                pos.side_to_move_set(Color::White);
                // TODO
                Ok(pos)
            }
        }
    }
}
