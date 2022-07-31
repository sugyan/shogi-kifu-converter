use crate::jkf::*;
use std::collections::HashMap;
use std::fmt::{Result, Write};

const SANYOU_SUJI: [char; 9] = ['１', '２', '３', '４', '５', '６', '７', '８', '９'];
const KANSUJI: [char; 10] = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];

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

impl ToKif for JsonKifFormat {
    fn to_kif<W: Write>(&self, sink: &mut W) -> Result {
        write_header(&self.header, sink)?;
        write_initial(&self.initial, sink)?;
        write_moves(&self.moves, sink)?;
        Ok(())
    }
}

fn write_sanyou_suji<W: Write>(num: u8, sink: &mut W) -> Result {
    sink.write_char(SANYOU_SUJI[num as usize - 1])?;
    Ok(())
}

fn write_kansuji<W: Write>(mut num: u8, sink: &mut W) -> Result {
    if num > 10 {
        sink.write_char('十')?;
        num -= 10;
    }
    sink.write_char(KANSUJI[num as usize - 1])?;
    Ok(())
}

fn write_board_kind<W: Write>(kind: Kind, sink: &mut W) -> Result {
    match kind {
        Kind::FU => sink.write_char('歩')?,
        Kind::KY => sink.write_char('香')?,
        Kind::KE => sink.write_char('桂')?,
        Kind::GI => sink.write_char('銀')?,
        Kind::KI => sink.write_char('金')?,
        Kind::KA => sink.write_char('角')?,
        Kind::HI => sink.write_char('飛')?,
        Kind::OU => sink.write_char('玉')?,
        Kind::TO => sink.write_char('と')?,
        Kind::NY => sink.write_char('杏')?,
        Kind::NK => sink.write_char('圭')?,
        Kind::NG => sink.write_char('全')?,
        Kind::UM => sink.write_char('馬')?,
        Kind::RY => sink.write_char('龍')?,
    }
    Ok(())
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

fn write_header<W: Write>(header: &HashMap<String, String>, sink: &mut W) -> Result {
    for (k, v) in header {
        sink.write_str(k)?;
        sink.write_char('：')?;
        sink.write_str(v)?;
        sink.write_char('\n')?;
    }
    Ok(())
}

fn write_hand<W: Write>(hand: &Hand, sink: &mut W) -> Result {
    for (c, num) in [
        ('飛', hand.HI),
        ('角', hand.KA),
        ('金', hand.KI),
        ('銀', hand.GI),
        ('桂', hand.KE),
        ('香', hand.KY),
        ('歩', hand.FU),
    ] {
        if num > 0 {
            sink.write_char(c)?;
            if num > 1 {
                write_kansuji(num, sink)?;
            }
            sink.write_char('　')?;
        }
    }
    Ok(())
}

fn write_initial_data<W: Write>(data: &StateFormat, sink: &mut W) -> Result {
    sink.write_str("手合割：その他\n")?;
    sink.write_str("後手の持駒：")?;
    if data.hands[1] != Hand::default() {
        write_hand(&data.hands[1], sink)?;
    } else {
        sink.write_str("なし")?;
    }
    sink.write_char('\n')?;
    sink.write_str("  ９ ８ ７ ６ ５ ４ ３ ２ １\n")?;
    sink.write_str("+---------------------------+\n")?;
    for i in 0..9 {
        sink.write_char('|')?;
        for j in 0..9 {
            let p = data.board[8 - j][i];
            if let (Some(c), Some(kind)) = (p.color, p.kind) {
                match c {
                    Color::Black => sink.write_char(' ')?,
                    Color::White => sink.write_char('v')?,
                };
                write_board_kind(kind, sink)?;
            } else {
                sink.write_str(" ・")?;
            }
        }
        sink.write_char('|')?;
        write_kansuji(i as u8 + 1, sink)?;
        sink.write_char('\n')?;
    }
    sink.write_str("+---------------------------+\n")?;
    sink.write_str("先手の持駒：")?;
    if data.hands[0] != Hand::default() {
        write_hand(&data.hands[0], sink)?;
    } else {
        sink.write_str("なし")?;
    }
    sink.write_char('\n')?;
    Ok(())
}

fn write_initial_preset<W: Write>(preset: Preset, sink: &mut W) -> Result {
    sink.write_str("手合割：")?;
    match preset {
        Preset::PresetHirate => sink.write_str("平手")?,
        Preset::PresetKY => sink.write_str("香落ち")?,
        Preset::PresetKYR => sink.write_str("右香落ち")?,
        Preset::PresetKA => sink.write_str("角落ち")?,
        Preset::PresetHI => sink.write_str("飛車落ち")?,
        Preset::PresetHIKY => sink.write_str("飛香落ち")?,
        Preset::Preset2 => sink.write_str("二枚落ち")?,
        Preset::Preset4 => sink.write_str("四枚落ち")?,
        Preset::Preset6 => sink.write_str("六枚落ち")?,
        Preset::Preset8 => sink.write_str("八枚落ち")?,
        Preset::Preset10 => sink.write_str("十枚落ち")?,
        _ => unimplemented!(),
    }
    sink.write_char('\n')?;
    Ok(())
}

fn write_initial<W: Write>(initial: &Option<Initial>, sink: &mut W) -> Result {
    if let Some(initial) = initial {
        if let Some(data) = &initial.data {
            write_initial_data(data, sink)?;
        } else {
            write_initial_preset(initial.preset, sink)?;
        }
    }
    Ok(())
}

fn write_moves<W: Write>(moves: &[MoveFormat], sink: &mut W) -> Result {
    sink.write_str("手数----指手---------消費時間--\n")?;
    if moves.len() == 1 {
        sink.write_str("   1 中断\n")?;
    }
    for (i, mf) in moves.iter().enumerate() {
        if i > 0 {
            sink.write_fmt(format_args!("{:4} ", i))?;
            let mut offset = 0;
            if let Some(mv) = mf.move_ {
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
        }
        if let Some(comments) = &mf.comments {
            for comment in comments {
                if !comment.starts_with('&') {
                    sink.write_char('*')?;
                }
                sink.write_str(comment)?;
                sink.write_char('\n')?;
            }
        }
    }
    // TODO: forks
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_kif_default() {
        assert_eq!(
            r#"
手数----指手---------消費時間--
   1 中断
"#[1..],
            JsonKifFormat::default().to_kif_owned()
        );
    }
}
