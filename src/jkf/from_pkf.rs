use crate::error::PkfConvertError;
use crate::{jkf, pkf};
use protobuf::EnumOrUnknown;

impl TryFrom<EnumOrUnknown<pkf::Color>> for jkf::Color {
    type Error = PkfConvertError;

    fn try_from(color: EnumOrUnknown<pkf::Color>) -> Result<Self, Self::Error> {
        match color.enum_value() {
            Ok(c) => match c {
                pkf::Color::BLACK => Ok(jkf::Color::Black),
                pkf::Color::WHITE => Ok(jkf::Color::White),
                _ => Err(PkfConvertError::ColorRequired),
            },
            Err(value) => Err(PkfConvertError::UnknownEnumValue {
                name: "color",
                value,
            }),
        }
    }
}

impl TryFrom<EnumOrUnknown<pkf::PieceKind>> for jkf::Kind {
    type Error = PkfConvertError;

    fn try_from(piece_kind: EnumOrUnknown<pkf::PieceKind>) -> Result<Self, Self::Error> {
        match piece_kind.enum_value() {
            Ok(pk) => match pk {
                pkf::PieceKind::FU => Ok(jkf::Kind::FU),
                pkf::PieceKind::KY => Ok(jkf::Kind::KY),
                pkf::PieceKind::KE => Ok(jkf::Kind::KE),
                pkf::PieceKind::GI => Ok(jkf::Kind::GI),
                pkf::PieceKind::KI => Ok(jkf::Kind::KI),
                pkf::PieceKind::KA => Ok(jkf::Kind::KA),
                pkf::PieceKind::HI => Ok(jkf::Kind::HI),
                pkf::PieceKind::OU => Ok(jkf::Kind::OU),
                pkf::PieceKind::TO => Ok(jkf::Kind::TO),
                pkf::PieceKind::NY => Ok(jkf::Kind::NY),
                pkf::PieceKind::NK => Ok(jkf::Kind::NK),
                pkf::PieceKind::NG => Ok(jkf::Kind::NG),
                pkf::PieceKind::UM => Ok(jkf::Kind::UM),
                pkf::PieceKind::RY => Ok(jkf::Kind::RY),
                _ => Err(PkfConvertError::PieceKindRequired),
            },
            Err(value) => Err(PkfConvertError::UnknownEnumValue {
                name: "piece_kind",
                value,
            }),
        }
    }
}

impl TryFrom<Option<&pkf::Square>> for jkf::PlaceFormat {
    type Error = PkfConvertError;

    fn try_from(square: Option<&pkf::Square>) -> Result<Self, Self::Error> {
        if let Some(sq) = square {
            Ok(jkf::PlaceFormat {
                x: u8::try_from(sq.file)?,
                y: u8::try_from(sq.rank)?,
            })
        } else {
            Err(PkfConvertError::MissingField("square"))
        }
    }
}

impl From<&pkf::initial::Preset> for jkf::Preset {
    fn from(preset: &pkf::initial::Preset) -> Self {
        use jkf::Preset::*;
        use pkf::initial::Preset::*;
        match preset {
            PRESET_HIRATE => PresetHirate,
            PRESET_KY => PresetKY,
            PRESET_KY_R => PresetKYR,
            PRESET_KA => PresetKA,
            PRESET_HI => PresetHI,
            PRESET_HIKY => PresetHIKY,
            PRESET_2 => Preset2,
            PRESET_4 => Preset4,
            PRESET_6 => Preset6,
            PRESET_8 => Preset8,
            PRESET_10 => Preset10,
        }
    }
}

impl TryFrom<&pkf::initial::state::Hand> for jkf::Hand {
    type Error = PkfConvertError;

    fn try_from(hand: &pkf::initial::state::Hand) -> Result<Self, Self::Error> {
        Ok(jkf::Hand {
            FU: hand.fu.try_into()?,
            KY: hand.ky.try_into()?,
            KE: hand.ke.try_into()?,
            GI: hand.gi.try_into()?,
            KI: hand.ki.try_into()?,
            KA: hand.ka.try_into()?,
            HI: hand.hi.try_into()?,
        })
    }
}

impl TryFrom<&pkf::initial::State> for jkf::StateFormat {
    type Error = PkfConvertError;

    fn try_from(state: &pkf::initial::State) -> Result<Self, Self::Error> {
        let color = state.color.try_into()?;
        let mut board = [[jkf::Piece::default(); 9]; 9];
        for (i, row) in board.iter_mut().enumerate() {
            for (j, piece) in row.iter_mut().enumerate() {
                if let Some(p) = get_board_piece(&state.board, (i, j)) {
                    *piece = jkf::Piece {
                        color: Some(p.color.try_into()?),
                        kind: Some(p.kind.try_into()?),
                    }
                }
            }
        }
        let mut hands = [jkf::Hand::default(); 2];
        if let Some(hand) = state.hands.black.as_ref() {
            hands[0] = hand.try_into()?;
        }
        if let Some(hand) = state.hands.white.as_ref() {
            hands[1] = hand.try_into()?;
        }
        Ok(jkf::StateFormat {
            color,
            board,
            hands,
        })
    }
}

impl TryFrom<EnumOrUnknown<pkf::move_::Special>> for jkf::MoveSpecial {
    type Error = PkfConvertError;

    fn try_from(specieal: EnumOrUnknown<pkf::move_::Special>) -> Result<Self, Self::Error> {
        use jkf::MoveSpecial::*;
        use pkf::move_::Special::*;
        match specieal.enum_value() {
            Ok(sp) => match sp {
                TORYO => Ok(SpecialToryo),
                CHUDAN => Ok(SpecialChudan),
                SENNICHITE => Ok(SpecialSennichite),
                TIME_UP => Ok(SpecialTimeUp),
                ILLEGAL_MOVE => Ok(SpecialIllegalMove),
                ILLEGAL_ACTION_BLACK => Ok(SpecialIllegalActionBlack),
                ILLEGAL_ACTION_WHITE => Ok(SpecialIllegalActionWhite),
                JISHOGI => Ok(SpecialJishogi),
                KACHI => Ok(SpecialKachi),
                HIKIWAKE => Ok(SpecialHikiwake),
                MATTA => Ok(SpecialMatta),
                TSUMI => Ok(SpecialTsumi),
                FUZUMI => Ok(SpecialFuzumi),
                ERROR => Ok(SpecialError),
                _ => Err(PkfConvertError::MoveSpecialRequired),
            },
            Err(value) => Err(PkfConvertError::UnknownEnumValue {
                name: "move.special",
                value,
            }),
        }
    }
}

impl TryFrom<EnumOrUnknown<pkf::move_::Relative>> for jkf::Relative {
    type Error = PkfConvertError;

    fn try_from(relative: EnumOrUnknown<pkf::move_::Relative>) -> Result<Self, Self::Error> {
        match relative.enum_value() {
            Ok(rel) => match rel {
                pkf::move_::Relative::L => Ok(jkf::Relative::L),
                pkf::move_::Relative::C => Ok(jkf::Relative::C),
                pkf::move_::Relative::R => Ok(jkf::Relative::R),
                pkf::move_::Relative::U => Ok(jkf::Relative::U),
                pkf::move_::Relative::M => Ok(jkf::Relative::M),
                pkf::move_::Relative::D => Ok(jkf::Relative::D),
                pkf::move_::Relative::LU => Ok(jkf::Relative::LU),
                pkf::move_::Relative::LM => Ok(jkf::Relative::LM),
                pkf::move_::Relative::LD => Ok(jkf::Relative::LD),
                pkf::move_::Relative::RU => Ok(jkf::Relative::RU),
                pkf::move_::Relative::RM => Ok(jkf::Relative::RM),
                pkf::move_::Relative::RD => Ok(jkf::Relative::RD),
                pkf::move_::Relative::H => Ok(jkf::Relative::H),
                _ => Err(PkfConvertError::MoveRelativeRequired),
            },
            Err(value) => Err(PkfConvertError::UnknownEnumValue {
                name: "move.relative",
                value,
            }),
        }
    }
}

impl TryFrom<&pkf::move_::Time> for jkf::Time {
    type Error = PkfConvertError;

    fn try_from(time: &pkf::move_::Time) -> Result<Self, Self::Error> {
        let tf = |s, some_h| -> Result<jkf::TimeFormat, Self::Error> {
            let m = (s / 60) % 60;
            let h = s / 3600;
            Ok(jkf::TimeFormat {
                h: if some_h || h > 0 {
                    Some(u8::try_from(h)?)
                } else {
                    None
                },
                m: m as u8,
                s: (s % 60) as u8,
            })
        };
        Ok(jkf::Time {
            now: tf(time.now, false)?,
            total: tf(time.total, true)?,
        })
    }
}

impl TryFrom<&pkf::Move> for jkf::MoveFormat {
    type Error = PkfConvertError;

    fn try_from(mv: &pkf::Move) -> Result<Self, Self::Error> {
        let mut ret = jkf::MoveFormat::default();
        if let Some(move_or_special) = &mv.action {
            match move_or_special {
                pkf::move_::Action::Normal(normal) => {
                    ret.move_ = Some(jkf::MoveMoveFormat {
                        color: normal.color.try_into()?,
                        from: Some(normal.from.as_ref().try_into()?),
                        to: normal.to.as_ref().try_into()?,
                        piece: normal.piece_kind.try_into()?,
                        same: None,
                        promote: normal.promote,
                        capture: match normal.capture.try_into() {
                            Ok(cap) => Some(cap),
                            Err(PkfConvertError::PieceKindRequired) => None,
                            Err(err) => return Err(err),
                        },
                        relative: match normal.relative.try_into() {
                            Ok(rel) => Some(rel),
                            Err(PkfConvertError::MoveRelativeRequired) => None,
                            Err(err) => return Err(err),
                        },
                    });
                }
                pkf::move_::Action::Drop(drop) => {
                    ret.move_ = Some(jkf::MoveMoveFormat {
                        color: drop.color.try_into()?,
                        from: None,
                        to: drop.to.as_ref().try_into()?,
                        piece: drop.piece_kind.try_into()?,
                        same: None,
                        promote: None,
                        capture: None,
                        relative: match drop.relative.try_into() {
                            Ok(rel) => Some(rel),
                            Err(PkfConvertError::MoveRelativeRequired) => None,
                            Err(err) => return Err(err),
                        },
                    });
                }
                pkf::move_::Action::Special(special) => {
                    ret.special = Some((*special).try_into()?);
                }
            }
        }
        if !mv.comments.is_empty() {
            ret.comments = Some(mv.comments.clone());
        }
        if let Some(time) = mv.time.as_ref() {
            ret.time = Some(time.try_into()?)
        }
        if !mv.forks.is_empty() {
            let mut forks = Vec::new();
            for fork in &mv.forks {
                let mut f = Vec::new();
                for mv in &fork.fork {
                    f.push(mv.try_into()?);
                }
                forks.push(f);
            }
            ret.forks = Some(forks);
        }
        Ok(ret)
    }
}

impl TryFrom<&pkf::Kifu> for jkf::JsonKifuFormat {
    type Error = PkfConvertError;

    fn try_from(kifu: &pkf::Kifu) -> Result<Self, Self::Error> {
        let header = kifu.header.clone();
        let initial = if let Some(initial) = kifu.initial.as_ref() {
            if let Some(preset_or_state) = &initial.position {
                match preset_or_state {
                    pkf::initial::Position::Preset(preset) => Some(jkf::Initial {
                        preset: (&preset.unwrap()).into(),
                        data: None,
                    }),
                    pkf::initial::Position::State(state) => Some(jkf::Initial {
                        preset: jkf::Preset::PresetOther,
                        data: Some(state.try_into()?),
                    }),
                }
            } else {
                Some(jkf::Initial {
                    preset: jkf::Preset::PresetHirate,
                    data: None,
                })
            }
        } else {
            None
        };
        let mut moves = Vec::new();
        for mv in &kifu.moves {
            moves.push(mv.try_into()?);
        }
        set_same(&mut moves, None);
        Ok(jkf::JsonKifuFormat {
            header,
            initial,
            moves,
        })
    }
}

fn set_same(moves: &mut [jkf::MoveFormat], mut prev: Option<jkf::PlaceFormat>) {
    for mv in moves.iter_mut() {
        if let Some(mmf) = mv.move_.as_mut() {
            if Some(mmf.to) == prev {
                mmf.same = Some(true);
            }
        }
        if let Some(forks) = mv.forks.as_mut() {
            for fork in forks.iter_mut() {
                set_same(fork, prev);
            }
        }
        prev = mv.move_.map(|mmf| mmf.to);
    }
}

fn get_board_piece(
    board: &pkf::initial::state::Board,
    (i, j): (usize, usize),
) -> Option<&pkf::Piece> {
    match (i, j) {
        (0, 0) => board.sq11.as_ref(),
        (0, 1) => board.sq12.as_ref(),
        (0, 2) => board.sq13.as_ref(),
        (0, 3) => board.sq14.as_ref(),
        (0, 4) => board.sq15.as_ref(),
        (0, 5) => board.sq16.as_ref(),
        (0, 6) => board.sq17.as_ref(),
        (0, 7) => board.sq18.as_ref(),
        (0, 8) => board.sq19.as_ref(),
        (1, 0) => board.sq21.as_ref(),
        (1, 1) => board.sq22.as_ref(),
        (1, 2) => board.sq23.as_ref(),
        (1, 3) => board.sq24.as_ref(),
        (1, 4) => board.sq25.as_ref(),
        (1, 5) => board.sq26.as_ref(),
        (1, 6) => board.sq27.as_ref(),
        (1, 7) => board.sq28.as_ref(),
        (1, 8) => board.sq29.as_ref(),
        (2, 0) => board.sq31.as_ref(),
        (2, 1) => board.sq32.as_ref(),
        (2, 2) => board.sq33.as_ref(),
        (2, 3) => board.sq34.as_ref(),
        (2, 4) => board.sq35.as_ref(),
        (2, 5) => board.sq36.as_ref(),
        (2, 6) => board.sq37.as_ref(),
        (2, 7) => board.sq38.as_ref(),
        (2, 8) => board.sq39.as_ref(),
        (3, 0) => board.sq41.as_ref(),
        (3, 1) => board.sq42.as_ref(),
        (3, 2) => board.sq43.as_ref(),
        (3, 3) => board.sq44.as_ref(),
        (3, 4) => board.sq45.as_ref(),
        (3, 5) => board.sq46.as_ref(),
        (3, 6) => board.sq47.as_ref(),
        (3, 7) => board.sq48.as_ref(),
        (3, 8) => board.sq49.as_ref(),
        (4, 0) => board.sq51.as_ref(),
        (4, 1) => board.sq52.as_ref(),
        (4, 2) => board.sq53.as_ref(),
        (4, 3) => board.sq54.as_ref(),
        (4, 4) => board.sq55.as_ref(),
        (4, 5) => board.sq56.as_ref(),
        (4, 6) => board.sq57.as_ref(),
        (4, 7) => board.sq58.as_ref(),
        (4, 8) => board.sq59.as_ref(),
        (5, 0) => board.sq61.as_ref(),
        (5, 1) => board.sq62.as_ref(),
        (5, 2) => board.sq63.as_ref(),
        (5, 3) => board.sq64.as_ref(),
        (5, 4) => board.sq65.as_ref(),
        (5, 5) => board.sq66.as_ref(),
        (5, 6) => board.sq67.as_ref(),
        (5, 7) => board.sq68.as_ref(),
        (5, 8) => board.sq69.as_ref(),
        (6, 0) => board.sq71.as_ref(),
        (6, 1) => board.sq72.as_ref(),
        (6, 2) => board.sq73.as_ref(),
        (6, 3) => board.sq74.as_ref(),
        (6, 4) => board.sq75.as_ref(),
        (6, 5) => board.sq76.as_ref(),
        (6, 6) => board.sq77.as_ref(),
        (6, 7) => board.sq78.as_ref(),
        (6, 8) => board.sq79.as_ref(),
        (7, 0) => board.sq81.as_ref(),
        (7, 1) => board.sq82.as_ref(),
        (7, 2) => board.sq83.as_ref(),
        (7, 3) => board.sq84.as_ref(),
        (7, 4) => board.sq85.as_ref(),
        (7, 5) => board.sq86.as_ref(),
        (7, 6) => board.sq87.as_ref(),
        (7, 7) => board.sq88.as_ref(),
        (7, 8) => board.sq89.as_ref(),
        (8, 0) => board.sq91.as_ref(),
        (8, 1) => board.sq92.as_ref(),
        (8, 2) => board.sq93.as_ref(),
        (8, 3) => board.sq94.as_ref(),
        (8, 4) => board.sq95.as_ref(),
        (8, 5) => board.sq96.as_ref(),
        (8, 6) => board.sq97.as_ref(),
        (8, 7) => board.sq98.as_ref(),
        (8, 8) => board.sq99.as_ref(),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_default() {
        assert_eq!(
            Ok(jkf::JsonKifuFormat::default()),
            jkf::JsonKifuFormat::try_from(&pkf::Kifu {
                moves: vec![pkf::Move::default()],
                ..Default::default()
            })
        );
    }

    #[test]
    fn from_arbitrary() {
        let ret = jkf::JsonKifuFormat::try_from(&pkf::Kifu {
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
        });
        assert_eq!(
            Ok(jkf::JsonKifuFormat {
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
            }),
            ret
        );
    }
}
