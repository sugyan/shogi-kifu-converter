use crate::error::PkfError;
use crate::jkf;
use crate::pkf;

impl TryFrom<&pkf::Color> for jkf::Color {
    type Error = PkfError;

    fn try_from(c: &pkf::Color) -> Result<Self, Self::Error> {
        match c {
            pkf::Color::COLOR_NONE => Err(PkfError::ColorRequired),
            pkf::Color::BLACK => Ok(jkf::Color::Black),
            pkf::Color::WHITE => Ok(jkf::Color::White),
        }
    }
}

impl TryFrom<&pkf::PieceKind> for jkf::Kind {
    type Error = PkfError;

    fn try_from(pk: &pkf::PieceKind) -> Result<Self, Self::Error> {
        match pk {
            pkf::PieceKind::PIECE_KIND_NONE => Err(PkfError::PieceKindRequired),
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
        }
    }
}

impl TryFrom<Option<&pkf::Square>> for jkf::PlaceFormat {
    type Error = PkfError;

    fn try_from(opt: Option<&pkf::Square>) -> Result<Self, Self::Error> {
        if let Some(sq) = opt {
            Ok(jkf::PlaceFormat {
                x: u8::try_from(sq.file)?,
                y: u8::try_from(sq.rank)?,
            })
        } else {
            Err(PkfError::SquareRequired)
        }
    }
}

impl From<&pkf::initial::Preset> for jkf::Preset {
    fn from(preset: &pkf::initial::Preset) -> Self {
        match preset {
            pkf::initial::Preset::PRESET_HIRATE => jkf::Preset::PresetHirate,
            pkf::initial::Preset::PRESET_KY => jkf::Preset::PresetKY,
            pkf::initial::Preset::PRESET_KY_R => jkf::Preset::PresetKYR,
            pkf::initial::Preset::PRESET_KA => jkf::Preset::PresetKA,
            pkf::initial::Preset::PRESET_HI => jkf::Preset::PresetHI,
            pkf::initial::Preset::PRESET_HIKY => jkf::Preset::PresetHIKY,
            pkf::initial::Preset::PRESET_2 => jkf::Preset::Preset2,
            pkf::initial::Preset::PRESET_4 => jkf::Preset::Preset4,
            pkf::initial::Preset::PRESET_6 => jkf::Preset::Preset6,
            pkf::initial::Preset::PRESET_8 => jkf::Preset::Preset8,
            pkf::initial::Preset::PRESET_10 => jkf::Preset::Preset10,
        }
    }
}

impl TryFrom<&pkf::Move> for jkf::MoveFormat {
    type Error = PkfError;

    fn try_from(mv: &pkf::Move) -> Result<Self, Self::Error> {
        let mut ret = jkf::MoveFormat::default();
        if let Some(move_or_special) = &mv.move_or_special {
            match move_or_special {
                pkf::move_::Move_or_special::Normal(normal) => {
                    ret.move_ = Some(jkf::MoveMoveFormat {
                        color: (&normal.color.unwrap()).try_into()?,
                        from: Some(normal.from.as_ref().try_into()?),
                        to: normal.to.as_ref().try_into()?,
                        piece: (&normal.piece_kind.unwrap()).try_into()?,
                        same: None,
                        promote: None,
                        capture: None,
                        relative: None,
                    })
                }
                pkf::move_::Move_or_special::Drop(drop) => {}
                pkf::move_::Move_or_special::Special(special) => {}
            }
        }
        if !mv.comments.is_empty() {
            ret.comments = Some(mv.comments.clone());
        }
        Ok(ret)
    }
}

impl TryFrom<&pkf::Kifu> for jkf::JsonKifuFormat {
    type Error = PkfError;

    fn try_from(kifu: &pkf::Kifu) -> Result<Self, Self::Error> {
        let header = kifu.header.clone();
        let initial = kifu.initial.as_ref().map(|initial| {
            if let Some(preset_or_state) = initial.preset_or_state.as_ref() {
                match preset_or_state {
                    pkf::initial::Preset_or_state::Preset(preset) => jkf::Initial {
                        preset: (&preset.unwrap()).into(),
                        data: None,
                    },
                    pkf::initial::Preset_or_state::State(_) => {
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
            moves.push(jkf::MoveFormat::try_from(mv)?);
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
        let ret = jkf::JsonKifuFormat::try_from(&pkf::Kifu {
            moves: vec![pkf::Move::default()],
            ..Default::default()
        });
        assert_eq!(Ok(jkf::JsonKifuFormat::default()), ret);
    }

    #[test]
    fn from_arbitrary() {
        let ret = jkf::JsonKifuFormat::try_from(&pkf::Kifu {
            header: [(String::from("key"), String::from("value"))]
                .into_iter()
                .collect(),
            initial: Some(pkf::Initial {
                preset_or_state: Some(pkf::initial::Preset_or_state::Preset(
                    pkf::initial::Preset::PRESET_KY.into(),
                )),
                ..Default::default()
            })
            .into(),
            moves: vec![
                pkf::Move::default(),
                pkf::Move {
                    move_or_special: Some(pkf::move_::Move_or_special::Normal(
                        pkf::move_::Normal {
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
                        },
                    )),
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
