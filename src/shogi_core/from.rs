use crate::error::{ConvertError, NormalizeError};
use crate::jkf;
use crate::jkf::{Color::*, Kind::*, Preset::*};
use shogi_core::{Color, Move, PartialPosition, Piece, PieceKind, Position, Square};

impl From<jkf::Color> for Color {
    fn from(c: jkf::Color) -> Self {
        match c {
            Black => Color::Black,
            White => Color::White,
        }
    }
}

impl From<jkf::Kind> for shogi_core::PieceKind {
    fn from(piece: jkf::Kind) -> Self {
        match piece {
            FU => PieceKind::Pawn,
            KY => PieceKind::Lance,
            KE => PieceKind::Knight,
            GI => PieceKind::Silver,
            KI => PieceKind::Gold,
            KA => PieceKind::Bishop,
            HI => PieceKind::Rook,
            OU => PieceKind::King,
            TO => PieceKind::ProPawn,
            NY => PieceKind::ProLance,
            NK => PieceKind::ProKnight,
            NG => PieceKind::ProSilver,
            UM => PieceKind::ProBishop,
            RY => PieceKind::ProRook,
        }
    }
}

impl TryFrom<&jkf::PlaceFormat> for Square {
    type Error = ConvertError;

    fn try_from(pf: &jkf::PlaceFormat) -> Result<Self, Self::Error> {
        Square::new(pf.x, pf.y).ok_or(ConvertError::InvalidSquare((pf.x, pf.y)))
    }
}

impl TryFrom<&jkf::MoveMoveFormat> for Move {
    type Error = ConvertError;

    fn try_from(mmf: &jkf::MoveMoveFormat) -> Result<Self, Self::Error> {
        if let Some(from) = &mmf.from {
            Ok(shogi_core::Move::Normal {
                from: from.try_into()?,
                to: (&mmf.to).try_into()?,
                promote: mmf.promote.unwrap_or_default(),
            })
        } else {
            Ok(shogi_core::Move::Drop {
                piece: shogi_core::Piece::new(mmf.piece.into(), mmf.color.into()),
                to: (&mmf.to).try_into()?,
            })
        }
    }
}

impl TryFrom<&jkf::Initial> for PartialPosition {
    type Error = ConvertError;

    fn try_from(initial: &jkf::Initial) -> Result<Self, Self::Error> {
        match initial.preset {
            PresetHirate => Ok(PartialPosition::startpos()),
            PresetOther => {
                let data = initial
                    .data
                    .ok_or(ConvertError::InitialBoardNoDataWithPresetOTHER)?;
                let mut pos = PartialPosition::empty();
                // Board
                for (i, v) in data.board.iter().enumerate() {
                    for (j, p) in v.iter().enumerate() {
                        let sq = (&jkf::PlaceFormat {
                            x: i as u8 + 1,
                            y: j as u8 + 1,
                        })
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
                            *h = h.added(pk).ok_or(ConvertError::InvalidHandPiece(pk))?;
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
                #[rustfmt::skip]
                let drops = match initial.preset {
                    PresetKY   => vec![Square::SQ_1A],
                    PresetKYR  => vec![Square::SQ_9A],
                    PresetKA   => vec![Square::SQ_2B],
                    PresetHI   => vec![Square::SQ_8B],
                    PresetHIKY => vec![Square::SQ_8B, Square::SQ_1A],
                    Preset2    => vec![Square::SQ_8B, Square::SQ_2B],
                    Preset4    => vec![Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A],
                    Preset6    => vec![Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A],
                    Preset8    => vec![Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A, Square::SQ_7A, Square::SQ_3A],
                    Preset10   => vec![Square::SQ_8B, Square::SQ_2B, Square::SQ_9A, Square::SQ_1A, Square::SQ_8A, Square::SQ_2A, Square::SQ_7A, Square::SQ_3A, Square::SQ_6A, Square::SQ_4A],
                    // Preset3, Preset5, Preset7...?
                    _ => unimplemented!(),
                };
                for sq in drops {
                    pos.piece_set(sq, None);
                }
                Ok(pos)
            }
        }
    }
}

impl TryFrom<&jkf::JsonKifuFormat> for Position {
    type Error = ConvertError;

    fn try_from(jkf: &jkf::JsonKifuFormat) -> Result<Self, Self::Error> {
        let mut pos = if let Some(initial) = &jkf.initial {
            Position::arbitrary_position(initial.try_into()?)
        } else {
            Position::startpos()
        };
        for mf in jkf.moves.iter() {
            if let Some(mv) = &mf.move_ {
                let mv = mv.try_into()?;
                pos.make_move(mv).ok_or_else(|| {
                    ConvertError::Normalize(NormalizeError::MakeMoveFailed(mv).to_string())
                })?;
            }
        }
        Ok(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_default() {
        assert_eq!(
            Ok(Position::default()),
            Position::try_from(&jkf::JsonKifuFormat::default())
        );
    }

    #[test]
    fn from_arbitrary() {
        let mut pp = PartialPosition::startpos();
        pp.piece_set(Square::SQ_1A, None);
        pp.side_to_move_set(Color::White);
        let mut pos = Position::arbitrary_position(pp);
        pos.make_move(Move::Normal {
            from: Square::SQ_7C,
            to: Square::SQ_7D,
            promote: false,
        })
        .expect("failed to make move");
        assert_eq!(
            Ok(pos),
            Position::try_from(&jkf::JsonKifuFormat {
                header: [(String::from("key"), String::from("value"))]
                    .into_iter()
                    .collect(),
                initial: Some(jkf::Initial {
                    preset: jkf::Preset::PresetKY,
                    data: None,
                }),
                moves: vec![
                    jkf::MoveFormat::default(),
                    jkf::MoveFormat {
                        move_: Some(jkf::MoveMoveFormat {
                            color: jkf::Color::White,
                            from: Some(jkf::PlaceFormat { x: 7, y: 3 }),
                            to: jkf::PlaceFormat { x: 7, y: 4 },
                            piece: jkf::Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    }
                ],
            })
        );
    }
}
