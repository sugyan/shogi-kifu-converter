use crate::error::PkfConvertError;
use crate::{jkf, pkf};

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
                    ..Default::default()
                }))
            } else {
                todo!()
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
        if let Some(data) = initial.data {
            // TODO
        } else {
            ret.position = Some(pkf::initial::Position::Preset(
                pkf::initial::Preset::try_from(initial.preset)?.into(),
            ))
        }
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
