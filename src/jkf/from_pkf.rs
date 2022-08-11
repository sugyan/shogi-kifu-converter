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
                pkf::PieceKind::PIECE_KIND_NONE => Err(PkfConvertError::PieceKindRequired),
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
            Err(PkfConvertError::MissingField { name: "square" })
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
                        promote: None,
                        capture: None,
                        relative: None,
                    })
                }
                pkf::move_::Action::Drop(drop) => {}
                pkf::move_::Action::Special(special) => {}
            }
        }
        if !mv.comments.is_empty() {
            ret.comments = Some(mv.comments.clone());
        }
        Ok(ret)
    }
}

impl TryFrom<&pkf::Kifu> for jkf::JsonKifuFormat {
    type Error = PkfConvertError;

    fn try_from(kifu: &pkf::Kifu) -> Result<Self, Self::Error> {
        let header = kifu.header.clone();
        let initial = kifu.initial.as_ref().map(|initial| {
            if let Some(preset_or_state) = &initial.position {
                match preset_or_state {
                    pkf::initial::Position::Preset(preset) => jkf::Initial {
                        preset: (&preset.unwrap()).into(),
                        data: None,
                    },
                    pkf::initial::Position::State(_) => {
                        todo!()
                    }
                }
            } else {
                jkf::Initial {
                    preset: jkf::Preset::PresetHirate,
                    data: None,
                }
            }
        });
        let mut moves = Vec::new();
        for mv in &kifu.moves {
            moves.push(mv.try_into()?);
        }
        Ok(jkf::JsonKifuFormat {
            header,
            initial,
            moves,
        })
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
