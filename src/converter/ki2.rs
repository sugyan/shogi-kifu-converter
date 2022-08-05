use super::kakinoki::{write_header, write_initial, write_kansuji, write_sanyou_suji};
use crate::jkf::*;
use std::fmt::{Result, Write};

/// A type that is convertible to KI2 format.
pub trait ToKi2 {
    /// Write `self` in KI2 format.
    ///
    /// This function returns Err(core::fmt::Error)
    /// if and only if it fails to write to `sink`.
    fn to_ki2<W: Write>(&self, sink: &mut W) -> Result;

    /// Returns `self`'s string representation.
    fn to_ki2_owned(&self) -> String {
        let mut s = String::new();
        // guaranteed to be Ok(())
        let result = self.to_ki2(&mut s);
        debug_assert_eq!(result, Ok(()));
        s
    }
}

fn write_move_kind<W: Write>(kind: Kind, sink: &mut W) -> Result {
    match kind {
        Kind::FU => sink.write_str("歩"),
        Kind::KY => sink.write_str("香"),
        Kind::KE => sink.write_str("桂"),
        Kind::GI => sink.write_str("銀"),
        Kind::KI => sink.write_str("金"),
        Kind::KA => sink.write_str("角"),
        Kind::HI => sink.write_str("飛"),
        Kind::OU => sink.write_str("玉"),
        Kind::TO => sink.write_str("と"),
        Kind::NY => sink.write_str("成香"),
        Kind::NK => sink.write_str("成桂"),
        Kind::NG => sink.write_str("成銀"),
        Kind::UM => sink.write_str("馬"),
        Kind::RY => sink.write_str("龍"),
    }
}

fn write_moves<W: Write>(moves: &[MoveFormat], sink: &mut W) -> Result {
    if let Some(comments) = &moves[0].comments {
        for comment in comments {
            if !comment.starts_with('&') {
                sink.write_char('*')?;
            }
            sink.write_str(comment)?;
            sink.write_char('\n')?;
        }
    }
    let mut it = moves[1..].iter().peekable();
    while let Some(mf) = it.next() {
        if let Some(mv) = &mf.move_ {
            match mv.color {
                Color::Black => sink.write_char('▲')?,
                Color::White => sink.write_char('△')?,
            }
            if mv.same.is_some() {
                sink.write_str("同")?;
            } else {
                write_sanyou_suji(mv.to.x, sink)?;
                write_kansuji(mv.to.y, sink)?;
            }
            write_move_kind(mv.piece, sink)?;
            if let Some(relative) = mv.relative {
                match relative {
                    Relative::L => sink.write_str("左")?,
                    Relative::C => sink.write_str("直")?,
                    Relative::R => sink.write_str("右")?,
                    Relative::U => sink.write_str("上")?,
                    Relative::M => sink.write_str("寄")?,
                    Relative::D => sink.write_str("引")?,
                    Relative::LU => sink.write_str("左上")?,
                    Relative::LM => sink.write_str("左寄")?,
                    Relative::LD => sink.write_str("左引")?,
                    Relative::RU => sink.write_str("右上")?,
                    Relative::RM => sink.write_str("右寄")?,
                    Relative::RD => sink.write_str("右引")?,
                    Relative::H => sink.write_str("打")?,
                }
            }
            if let Some(promote) = mv.promote {
                if promote {
                    sink.write_str("成")?;
                } else {
                    sink.write_str("不成")?;
                }
            }
        }
        if let Some(comments) = &mf.comments {
            sink.write_char('\n')?;
            for comment in comments {
                if !comment.starts_with('&') {
                    sink.write_char('*')?;
                }
                sink.write_str(comment)?;
                sink.write_char('\n')?;
            }
        } else if it.peek().is_some() {
            sink.write_char(' ')?;
        }
    }
    sink.write_char('\n')?;
    Ok(())
}

impl ToKi2 for JsonKifuFormat {
    fn to_ki2<W: Write>(&self, sink: &mut W) -> Result {
        write_header(&self.header, sink)?;
        write_initial(&self.initial, true, sink)?;
        write_moves(&self.moves, sink)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_ki2_default() {
        assert_eq!("\n", JsonKifuFormat::default().to_ki2_owned());
    }

    #[test]
    fn to_ki2_moves() {
        assert_eq!(
            "▲２六歩 △８四歩 ▲２五歩\n",
            JsonKifuFormat {
                moves: vec![
                    MoveFormat::default(),
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 2, y: 7 }),
                            to: PlaceFormat { x: 2, y: 6 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::White,
                            from: Some(PlaceFormat { x: 8, y: 3 }),
                            to: PlaceFormat { x: 8, y: 4 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    },
                    MoveFormat {
                        move_: Some(MoveMoveFormat {
                            color: Color::Black,
                            from: Some(PlaceFormat { x: 2, y: 6 }),
                            to: PlaceFormat { x: 2, y: 5 },
                            piece: Kind::FU,
                            same: None,
                            promote: None,
                            capture: None,
                            relative: None,
                        }),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
            .to_ki2_owned()
        );
    }
}
