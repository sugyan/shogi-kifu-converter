use crate::error::{ConvertError, NormalizeError};
use crate::jkf;
use shogi_core::{Color, Hand, Move, PartialPosition, Piece, PieceKind, Position, Square};
use std::collections::HashMap;

impl From<Color> for jkf::Color {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => jkf::Color::Black,
            Color::White => jkf::Color::White,
        }
    }
}

impl From<PieceKind> for jkf::Kind {
    fn from(pk: PieceKind) -> Self {
        match pk {
            PieceKind::Pawn => jkf::Kind::FU,
            PieceKind::Lance => jkf::Kind::KY,
            PieceKind::Knight => jkf::Kind::KE,
            PieceKind::Silver => jkf::Kind::GI,
            PieceKind::Gold => jkf::Kind::KI,
            PieceKind::Bishop => jkf::Kind::KA,
            PieceKind::Rook => jkf::Kind::HI,
            PieceKind::King => jkf::Kind::OU,
            PieceKind::ProPawn => jkf::Kind::TO,
            PieceKind::ProLance => jkf::Kind::NY,
            PieceKind::ProKnight => jkf::Kind::NK,
            PieceKind::ProSilver => jkf::Kind::NG,
            PieceKind::ProBishop => jkf::Kind::UM,
            PieceKind::ProRook => jkf::Kind::RY,
        }
    }
}

impl From<&Piece> for jkf::Piece {
    fn from(p: &Piece) -> Self {
        jkf::Piece {
            color: Some(p.color().into()),
            kind: Some(p.piece_kind().into()),
        }
    }
}

impl From<&Square> for jkf::PlaceFormat {
    fn from(s: &Square) -> Self {
        jkf::PlaceFormat {
            x: s.file(),
            y: s.rank(),
        }
    }
}

impl From<&Hand> for jkf::Hand {
    fn from(h: &Hand) -> Self {
        jkf::Hand {
            FU: h.count(PieceKind::Pawn).unwrap(),
            KY: h.count(PieceKind::Lance).unwrap(),
            KE: h.count(PieceKind::Knight).unwrap(),
            GI: h.count(PieceKind::Silver).unwrap(),
            KI: h.count(PieceKind::Gold).unwrap(),
            KA: h.count(PieceKind::Bishop).unwrap(),
            HI: h.count(PieceKind::Rook).unwrap(),
        }
    }
}

impl From<&PartialPosition> for jkf::Initial {
    fn from(pos: &PartialPosition) -> Self {
        let mut board = [[jkf::Piece::default(); 9]; 9];
        for sq in Square::all() {
            if let Some(p) = pos.piece_at(sq) {
                board[sq.file() as usize - 1][sq.rank() as usize - 1] = (&p).into();
            }
        }
        let hands = [
            (&pos.hand_of_a_player(Color::Black)).into(),
            (&pos.hand_of_a_player(Color::White)).into(),
        ];
        jkf::Initial {
            preset: jkf::Preset::PresetOther,
            data: Some(jkf::StateFormat {
                color: pos.side_to_move().into(),
                board,
                hands,
            }),
        }
    }
}

impl TryFrom<&Position> for jkf::JsonKifuFormat {
    type Error = ConvertError;

    fn try_from(pos: &Position) -> Result<Self, Self::Error> {
        let mut moves = vec![jkf::MoveFormat::default()];
        let mut pp = pos.initial_position().clone();
        for &mv in pos.moves() {
            let mmf = match mv {
                Move::Normal { from, to, promote } => {
                    let piece = pp.piece_at(from).ok_or_else(|| {
                        ConvertError::Normalize(NormalizeError::NoPieceAt(from).to_string())
                    })?;
                    jkf::MoveMoveFormat {
                        color: pp.side_to_move().into(),
                        from: Some((&from).into()),
                        to: (&to).into(),
                        piece: piece.piece_kind().into(),
                        same: None,
                        promote: Some(promote),
                        capture: None,
                        relative: None,
                    }
                }
                // To disambiguate `Normal` move or `Drop` move, `from` is converted to `Some(PlaceFormat { x: 0, y: 0 })`
                Move::Drop { piece, to } => jkf::MoveMoveFormat {
                    color: pp.side_to_move().into(),
                    from: Some(jkf::PlaceFormat { x: 0, y: 0 }),
                    to: (&to).into(),
                    piece: piece.piece_kind().into(),
                    same: None,
                    promote: None,
                    capture: None,
                    relative: None,
                },
            };
            moves.push(jkf::MoveFormat {
                move_: Some(mmf),
                ..Default::default()
            });
            pp.make_move(mv).ok_or_else(|| {
                ConvertError::Normalize(NormalizeError::MakeMoveFailed(mv).to_string())
            })?;
        }
        let mut ret = jkf::JsonKifuFormat {
            header: HashMap::new(),
            initial: Some(pos.initial_position().into()),
            moves,
        };
        match ret.normalize() {
            Ok(()) => Ok(ret),
            Err(err) => Err(ConvertError::Normalize(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_default() {
        let pos = Position::default();
        assert_eq!(
            Ok(jkf::JsonKifuFormat {
                initial: Some(jkf::Initial {
                    preset: jkf::Preset::PresetHirate,
                    data: None,
                }),
                ..Default::default()
            }),
            jkf::JsonKifuFormat::try_from(&pos)
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
            Ok(jkf::JsonKifuFormat {
                header: HashMap::new(),
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
                ]
            }),
            jkf::JsonKifuFormat::try_from(&pos)
        );
    }
}
