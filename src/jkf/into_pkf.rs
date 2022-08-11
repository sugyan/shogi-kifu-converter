use crate::error::PkfConvertError;
use crate::{jkf, pkf};
use protobuf::MessageField;

impl From<jkf::Color> for pkf::Color {
    fn from(color: jkf::Color) -> Self {
        match color {
            jkf::Color::Black => pkf::Color::BLACK,
            jkf::Color::White => pkf::Color::WHITE,
        }
    }
}

impl From<jkf::Kind> for pkf::PieceKind {
    fn from(kind: jkf::Kind) -> Self {
        match kind {
            jkf::Kind::FU => pkf::PieceKind::FU,
            jkf::Kind::KY => pkf::PieceKind::KY,
            jkf::Kind::KE => pkf::PieceKind::KE,
            jkf::Kind::GI => pkf::PieceKind::GI,
            jkf::Kind::KI => pkf::PieceKind::KI,
            jkf::Kind::KA => pkf::PieceKind::KA,
            jkf::Kind::HI => pkf::PieceKind::HI,
            jkf::Kind::OU => pkf::PieceKind::OU,
            jkf::Kind::TO => pkf::PieceKind::TO,
            jkf::Kind::NY => pkf::PieceKind::NY,
            jkf::Kind::NK => pkf::PieceKind::NK,
            jkf::Kind::NG => pkf::PieceKind::NG,
            jkf::Kind::UM => pkf::PieceKind::UM,
            jkf::Kind::RY => pkf::PieceKind::RY,
        }
    }
}

impl TryFrom<jkf::Preset> for pkf::initial::Preset {
    type Error = PkfConvertError;

    fn try_from(preset: jkf::Preset) -> Result<Self, Self::Error> {
        use jkf::Preset::*;
        use pkf::initial::Preset::*;
        match preset {
            PresetHirate => Ok(PRESET_HIRATE),
            PresetKY => Ok(PRESET_KY),
            PresetKYR => Ok(PRESET_KY_R),
            PresetKA => Ok(PRESET_KA),
            PresetHI => Ok(PRESET_HI),
            PresetHIKY => Ok(PRESET_HIKY),
            Preset2 => Ok(PRESET_2),
            Preset4 => Ok(PRESET_4),
            Preset6 => Ok(PRESET_6),
            Preset8 => Ok(PRESET_8),
            Preset10 => Ok(PRESET_10),
            _ => Err(PkfConvertError::UnsupportedPreset(preset)),
        }
    }
}

impl From<jkf::MoveSpecial> for pkf::move_::Special {
    fn from(special: jkf::MoveSpecial) -> Self {
        use jkf::MoveSpecial::*;
        use pkf::move_::Special::*;
        match special {
            SpecialToryo => TORYO,
            SpecialChudan => CHUDAN,
            SpecialSennichite => SENNICHITE,
            SpecialTimeUp => TIME_UP,
            SpecialIllegalMove => ILLEGAL_MOVE,
            SpecialIllegalActionBlack => ILLEGAL_ACTION_BLACK,
            SpecialIllegalActionWhite => ILLEGAL_ACTION_WHITE,
            SpecialJishogi => JISHOGI,
            SpecialKachi => KACHI,
            SpecialHikiwake => HIKIWAKE,
            SpecialMatta => MATTA,
            SpecialTsumi => TSUMI,
            SpecialFuzumi => FUZUMI,
            SpecialError => ERROR,
        }
    }
}

impl From<&jkf::Piece> for Option<pkf::Piece> {
    fn from(piece: &jkf::Piece) -> Self {
        if let (Some(c), Some(kind)) = (piece.color, piece.kind) {
            Some(pkf::Piece {
                color: pkf::Color::from(c).into(),
                kind: pkf::PieceKind::from(kind).into(),
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl From<&jkf::Hand> for pkf::initial::state::Hand {
    fn from(hand: &jkf::Hand) -> Self {
        pkf::initial::state::Hand {
            fu: hand.FU.into(),
            ky: hand.KY.into(),
            ke: hand.KE.into(),
            gi: hand.GI.into(),
            ki: hand.KI.into(),
            ka: hand.KA.into(),
            hi: hand.HI.into(),
            ..Default::default()
        }
    }
}

impl TryFrom<&jkf::StateFormat> for pkf::initial::State {
    type Error = PkfConvertError;

    fn try_from(state: &jkf::StateFormat) -> Result<Self, Self::Error> {
        let mut ret = pkf::initial::State::new();
        ret.color = pkf::Color::from(state.color).into();
        let mut board = pkf::initial::state::Board::new();
        for (i, row) in state.board.iter().enumerate() {
            for (j, piece) in row.iter().enumerate() {
                set_board_piece(&mut board, (i, j), Option::<pkf::Piece>::from(piece).into());
            }
        }
        ret.board = Some(board).into();
        let mut hands = pkf::initial::state::Hands::new();
        hands.black = Some((&state.hands[0]).into()).into();
        hands.white = Some((&state.hands[1]).into()).into();
        ret.hands = Some(hands).into();
        Ok(ret)
    }
}

impl From<&jkf::PlaceFormat> for pkf::Square {
    fn from(place: &jkf::PlaceFormat) -> Self {
        pkf::Square {
            file: place.x.into(),
            rank: place.y.into(),
            ..Default::default()
        }
    }
}

impl TryFrom<&jkf::MoveFormat> for pkf::Move {
    type Error = PkfConvertError;

    fn try_from(mv: &jkf::MoveFormat) -> Result<Self, Self::Error> {
        let mut ret = pkf::Move::new();
        ret.action = if let Some(mmf) = &mv.move_ {
            if let Some(from) = &mmf.from {
                Some(pkf::move_::Action::Normal(pkf::move_::Normal {
                    color: pkf::Color::from(mmf.color).into(),
                    from: Some(from.into()).into(),
                    to: Some((&mmf.to).into()).into(),
                    piece_kind: pkf::PieceKind::from(mmf.piece).into(),
                    promote: mmf.promote,
                    ..Default::default()
                }))
            } else {
                Some(pkf::move_::Action::Drop(pkf::move_::Drop {
                    color: pkf::Color::from(mmf.color).into(),
                    to: Some((&mmf.to).into()).into(),
                    piece_kind: pkf::PieceKind::from(mmf.piece).into(),
                    ..Default::default()
                }))
            }
        } else {
            mv.special.map(|special| {
                pkf::move_::Action::Special(pkf::move_::Special::from(special).into())
            })
        };
        Ok(ret)
    }
}

impl TryFrom<&jkf::Initial> for pkf::Initial {
    type Error = PkfConvertError;

    fn try_from(initial: &jkf::Initial) -> Result<Self, Self::Error> {
        let mut ret = pkf::Initial::new();
        ret.position = if let Some(data) = &initial.data {
            Some(pkf::initial::Position::State(
                pkf::initial::State::try_from(data)?,
            ))
        } else {
            Some(pkf::initial::Position::Preset(
                pkf::initial::Preset::try_from(initial.preset)?.into(),
            ))
        };
        Ok(ret)
    }
}

impl TryFrom<&jkf::JsonKifuFormat> for pkf::Kifu {
    type Error = PkfConvertError;

    fn try_from(jkf: &jkf::JsonKifuFormat) -> Result<Self, Self::Error> {
        let mut ret = pkf::Kifu::new();
        ret.header = jkf.header.clone();
        ret.initial = jkf
            .initial
            .as_ref()
            .map(pkf::Initial::try_from)
            .transpose()?
            .into();
        for mv in &jkf.moves {
            ret.moves.push(mv.try_into()?);
        }
        Ok(ret)
    }
}

fn set_board_piece(
    board: &mut pkf::initial::state::Board,
    (i, j): (usize, usize),
    piece: MessageField<pkf::Piece>,
) {
    match (i, j) {
        (0, 0) => board.sq11 = piece,
        (0, 1) => board.sq12 = piece,
        (0, 2) => board.sq13 = piece,
        (0, 3) => board.sq14 = piece,
        (0, 4) => board.sq15 = piece,
        (0, 5) => board.sq16 = piece,
        (0, 6) => board.sq17 = piece,
        (0, 7) => board.sq18 = piece,
        (0, 8) => board.sq19 = piece,
        (1, 0) => board.sq21 = piece,
        (1, 1) => board.sq22 = piece,
        (1, 2) => board.sq23 = piece,
        (1, 3) => board.sq24 = piece,
        (1, 4) => board.sq25 = piece,
        (1, 5) => board.sq26 = piece,
        (1, 6) => board.sq27 = piece,
        (1, 7) => board.sq28 = piece,
        (1, 8) => board.sq29 = piece,
        (2, 0) => board.sq31 = piece,
        (2, 1) => board.sq32 = piece,
        (2, 2) => board.sq33 = piece,
        (2, 3) => board.sq34 = piece,
        (2, 4) => board.sq35 = piece,
        (2, 5) => board.sq36 = piece,
        (2, 6) => board.sq37 = piece,
        (2, 7) => board.sq38 = piece,
        (2, 8) => board.sq39 = piece,
        (3, 0) => board.sq41 = piece,
        (3, 1) => board.sq42 = piece,
        (3, 2) => board.sq43 = piece,
        (3, 3) => board.sq44 = piece,
        (3, 4) => board.sq45 = piece,
        (3, 5) => board.sq46 = piece,
        (3, 6) => board.sq47 = piece,
        (3, 7) => board.sq48 = piece,
        (3, 8) => board.sq49 = piece,
        (4, 0) => board.sq51 = piece,
        (4, 1) => board.sq52 = piece,
        (4, 2) => board.sq53 = piece,
        (4, 3) => board.sq54 = piece,
        (4, 4) => board.sq55 = piece,
        (4, 5) => board.sq56 = piece,
        (4, 6) => board.sq57 = piece,
        (4, 7) => board.sq58 = piece,
        (4, 8) => board.sq59 = piece,
        (5, 0) => board.sq61 = piece,
        (5, 1) => board.sq62 = piece,
        (5, 2) => board.sq63 = piece,
        (5, 3) => board.sq64 = piece,
        (5, 4) => board.sq65 = piece,
        (5, 5) => board.sq66 = piece,
        (5, 6) => board.sq67 = piece,
        (5, 7) => board.sq68 = piece,
        (5, 8) => board.sq69 = piece,
        (6, 0) => board.sq71 = piece,
        (6, 1) => board.sq72 = piece,
        (6, 2) => board.sq73 = piece,
        (6, 3) => board.sq74 = piece,
        (6, 4) => board.sq75 = piece,
        (6, 5) => board.sq76 = piece,
        (6, 6) => board.sq77 = piece,
        (6, 7) => board.sq78 = piece,
        (6, 8) => board.sq79 = piece,
        (7, 0) => board.sq81 = piece,
        (7, 1) => board.sq82 = piece,
        (7, 2) => board.sq83 = piece,
        (7, 3) => board.sq84 = piece,
        (7, 4) => board.sq85 = piece,
        (7, 5) => board.sq86 = piece,
        (7, 6) => board.sq87 = piece,
        (7, 7) => board.sq88 = piece,
        (7, 8) => board.sq89 = piece,
        (8, 0) => board.sq91 = piece,
        (8, 1) => board.sq92 = piece,
        (8, 2) => board.sq93 = piece,
        (8, 3) => board.sq94 = piece,
        (8, 4) => board.sq95 = piece,
        (8, 5) => board.sq96 = piece,
        (8, 6) => board.sq97 = piece,
        (8, 7) => board.sq98 = piece,
        (8, 8) => board.sq99 = piece,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_default() {
        assert_eq!(
            Ok(pkf::Kifu {
                moves: vec![pkf::Move::default()],
                ..Default::default()
            }),
            pkf::Kifu::try_from(&jkf::JsonKifuFormat::default())
        );
    }

    #[test]
    fn from_arbitrary() {
        let ret = pkf::Kifu::try_from(&jkf::JsonKifuFormat {
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
                },
            ],
        });
        assert_eq!(
            Ok(pkf::Kifu {
                header: [(String::from("key"), String::from("value"))]
                    .into_iter()
                    .collect(),
                initial: Some(pkf::Initial {
                    position: Some(pkf::initial::Position::Preset(
                        pkf::initial::Preset::PRESET_KY.into(),
                    )),
                    ..Default::default()
                })
                .into(),
                moves: vec![
                    pkf::Move::default(),
                    pkf::Move {
                        action: Some(pkf::move_::Action::Normal(pkf::move_::Normal {
                            color: pkf::Color::WHITE.into(),
                            from: Some(pkf::Square {
                                file: 7,
                                rank: 3,
                                ..Default::default()
                            })
                            .into(),
                            to: Some(pkf::Square {
                                file: 7,
                                rank: 4,
                                ..Default::default()
                            })
                            .into(),
                            piece_kind: pkf::PieceKind::FU.into(),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            ret
        );
    }
}
