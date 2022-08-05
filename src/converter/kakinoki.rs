use crate::jkf::*;
use std::collections::HashMap;
use std::fmt::{Result, Write};

const SANYOU_SUJI: [char; 9] = ['１', '２', '３', '４', '５', '６', '７', '８', '９'];
const KANSUJI: [char; 10] = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];

pub(super) fn write_sanyou_suji<W: Write>(num: u8, sink: &mut W) -> Result {
    sink.write_char(SANYOU_SUJI[num as usize - 1])?;
    Ok(())
}

pub(super) fn write_kansuji<W: Write>(mut num: u8, sink: &mut W) -> Result {
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

pub(super) fn write_header<W: Write>(header: &HashMap<String, String>, sink: &mut W) -> Result {
    for (k, v) in header {
        sink.write_str(k)?;
        sink.write_char('：')?;
        sink.write_str(v)?;
        sink.write_char('\n')?;
    }
    Ok(())
}

pub(super) fn write_initial<W: Write>(
    initial: &Option<Initial>,
    omit_hirate: bool,
    sink: &mut W,
) -> Result {
    if let Some(initial) = initial {
        if let Some(data) = &initial.data {
            write_initial_data(data, sink)?;
        } else {
            if omit_hirate && initial.preset == Preset::PresetHirate {
                return Ok(());
            }
            write_initial_preset(initial.preset, sink)?;
        }
    }
    Ok(())
}
