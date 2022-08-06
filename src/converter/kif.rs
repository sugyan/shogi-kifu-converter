use super::kakinoki::{write_header, write_initial, write_kansuji, write_sanyou_suji};
use crate::jkf::*;
use std::fmt::{Result, Write};

/// A type that is convertible to KIF format.
pub trait ToKif {
    /// Write `self` in KIF format.
    ///
    /// This function returns Err(core::fmt::Error)
    /// if and only if it fails to write to `sink`.
    fn to_kif<W: Write>(&self, sink: &mut W) -> Result;

    /// Returns `self`'s string representation.
    fn to_kif_owned(&self) -> String {
        let mut s = String::new();
        // guaranteed to be Ok(())
        let result = self.to_kif(&mut s);
        debug_assert_eq!(result, Ok(()));
        s
    }
}

impl ToKif for JsonKifuFormat {
    fn to_kif<W: Write>(&self, sink: &mut W) -> Result {
        write_header(&self.header, sink)?;
        write_initial(&self.initial, false, sink)?;
        write_moves(&self.moves, sink)?;
        Ok(())
    }
}

fn write_move_kind<W: Write>(kind: Kind, sink: &mut W, offset: &mut usize) -> Result {
    match kind {
        Kind::FU => sink.write_str("歩")?,
        Kind::KY => sink.write_str("香")?,
        Kind::KE => sink.write_str("桂")?,
        Kind::GI => sink.write_str("銀")?,
        Kind::KI => sink.write_str("金")?,
        Kind::KA => sink.write_str("角")?,
        Kind::HI => sink.write_str("飛")?,
        Kind::OU => sink.write_str("玉")?,
        Kind::TO => sink.write_str("と")?,
        Kind::NY => {
            sink.write_str("成香")?;
            *offset += 2;
        }
        Kind::NK => {
            sink.write_str("成桂")?;
            *offset += 2;
        }
        Kind::NG => {
            sink.write_str("成銀")?;
            *offset += 2;
        }
        Kind::UM => sink.write_str("馬")?,
        Kind::RY => sink.write_str("龍")?,
    }
    *offset += 2;
    Ok(())
}

fn write_move_lines<W: Write>(moves: &[MoveFormat], index: usize, sink: &mut W) -> Result {
    let mut forks_stack = Vec::new();
    for (i, mf) in (index..).zip(moves) {
        sink.write_fmt(format_args!("{:4} ", i))?;
        let mut offset = 0;
        if let Some(mv) = &mf.move_ {
            if mv.same.is_some() {
                sink.write_str("同　")?;
            } else {
                write_sanyou_suji(mv.to.x, sink)?;
                write_kansuji(mv.to.y, sink)?;
            }
            offset += 4;
            write_move_kind(mv.piece, sink, &mut offset)?;
            if mv.promote.unwrap_or_default() {
                sink.write_char('成')?;
                offset += 2;
            }
            if let Some(from) = mv.from {
                sink.write_fmt(format_args!("({}{})", from.x, from.y))?;
                offset += 4;
            } else {
                sink.write_char('打')?;
                offset += 2;
            }
        } else if let Some(special) = &mf.special {
            match special {
                MoveSpecial::SpecialToryo => {
                    sink.write_str("投了")?;
                    offset += 4;
                }
                MoveSpecial::SpecialSennichite => {
                    sink.write_str("千日手")?;
                    offset += 6;
                }
                MoveSpecial::SpecialTimeUp => {
                    sink.write_str("切れ負け")?;
                    offset += 8;
                }
                MoveSpecial::SpecialIllegalMove => {
                    sink.write_str("反則負け")?;
                    offset += 8;
                }
                MoveSpecial::SpecialJishogi => {
                    sink.write_str("持将棋")?;
                    offset += 6;
                }
                MoveSpecial::SpecialKachi => {
                    sink.write_str("入玉勝ち")?;
                    offset += 8;
                }
                MoveSpecial::SpecialTsumi => {
                    sink.write_str("詰み")?;
                    offset += 4;
                }
                // TODO: SpecialIllegalActionBlack, SpecialIllegalActionWhite, SpecialFuzumi, SpecialError, etc...
                _ => sink.write_str("中断")?,
            }
        } else {
            unreachable!()
        }
        if let Some(time) = mf.time {
            (0..13 - offset).try_for_each(|_| sink.write_char(' '))?;
            sink.write_fmt(format_args!(
                "({:2}:{:02}/{:02}:{:02}:{:02})",
                time.now.m,
                time.now.s,
                time.total.h.unwrap_or_default(),
                time.total.m,
                time.total.s
            ))?;
        }
        sink.write_char('\n')?;
        if let Some(comments) = &mf.comments {
            for comment in comments {
                if !comment.starts_with('&') {
                    sink.write_char('*')?;
                }
                sink.write_str(comment)?;
                sink.write_char('\n')?;
            }
        }
        if let Some(ref forks) = mf.forks {
            for fork in forks {
                forks_stack.push((i, fork));
            }
        }
    }
    while let Some((i, fork)) = forks_stack.pop() {
        sink.write_char('\n')?;
        sink.write_fmt(format_args!("変化：{}手\n", i))?;
        write_move_lines(fork, i, sink)?;
    }
    Ok(())
}

fn write_moves<W: Write>(moves: &[MoveFormat], sink: &mut W) -> Result {
    sink.write_str("手数----指手---------消費時間--\n")?;
    if let Some(comments) = &moves[0].comments {
        for comment in comments {
            if !comment.starts_with('&') {
                sink.write_char('*')?;
            }
            sink.write_str(comment)?;
            sink.write_char('\n')?;
        }
    }
    write_move_lines(&moves[1..], 1, sink)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_jkf_file;
    use std::path::Path;

    #[test]
    fn to_kif_default() {
        assert_eq!(
            r#"
手数----指手---------消費時間--
"#[1..],
            JsonKifuFormat::default().to_kif_owned()
        );
    }

    #[test]
    fn fork_moves() {
        let path = Path::new("data/tests/kif/forks.json");
        let jkf = parse_jkf_file(&path).expect("failed to parse kif");
        let kif = jkf.to_kif_owned();
        assert_eq!(
            &r#"
手数----指手---------消費時間--
   1 ７六歩(77)   ( 0:00/00:00:00)
   2 ８四歩(83)   ( 0:00/00:00:00)
   3 ６八銀(79)   ( 0:00/00:00:00)
   4 ３二金(41)   ( 0:00/00:00:00)
   5 ２六歩(27)   ( 0:00/00:00:00)
   6 ８五歩(84)   ( 0:00/00:00:00)
   7 ７七角(88)   ( 0:00/00:00:00)
   8 ３四歩(33)   ( 0:00/00:00:00)
   9 ７八金(69)   ( 0:00/00:00:00)
  10 ７七角成(22) ( 0:00/00:00:00)
  11 同　銀(68)   ( 0:00/00:00:00)
  12 ２二銀(31)   ( 0:00/00:00:00)

変化：10手
  10 ３三角(22)   ( 0:00/00:00:00)
  11 ６九玉(59)   ( 0:00/00:00:00)
  12 ４二銀(31)   ( 0:00/00:00:00)
  13 ３六歩(37)   ( 0:00/00:00:00)
  14 ７七角成(33) ( 0:00/00:00:00)

変化：5手
   5 ７七角(88)   ( 0:00/00:00:00)
   6 ３四歩(33)   ( 0:00/00:00:00)
   7 ４八銀(39)   ( 0:00/00:00:00)
   8 ６二銀(71)   ( 0:00/00:00:00)
   9 ３六歩(37)   ( 0:00/00:00:00)
  10 ８五歩(84)   ( 0:00/00:00:00)
  11 ７八金(69)   ( 0:00/00:00:00)
  12 ７四歩(73)   ( 0:00/00:00:00)

変化：9手
   9 １六歩(17)   ( 0:00/00:00:00)
  10 １四歩(13)   ( 0:00/00:00:00)
  11 ２六歩(27)   ( 0:00/00:00:00)
  12 ４二銀(31)   ( 0:00/00:00:00)
  13 ２二角成(77) ( 0:00/00:00:00)
  14 同　金(32)   ( 0:00/00:00:00)
  15 ７七銀(68)   ( 0:00/00:00:00)
"#[1..],
            kif.lines().skip(3).collect::<Vec<_>>().join("\n") + "\n"
        );
    }
}
