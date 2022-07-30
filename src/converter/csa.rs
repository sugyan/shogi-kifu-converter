use crate::jkf::*;
use std::collections::HashMap;
use std::fmt::{Result, Write};

/// A type that is convertible to CSA format.
pub trait ToCSA {
    /// Write `self` in CSA format.
    ///
    /// This function returns Err(core::fmt::Error)
    /// if and only if it fails to write to `sink`.
    fn to_csa<W: Write>(&self, sink: &mut W) -> Result;

    /// Returns `self`'s string representation.
    fn to_csa_owned(&self) -> String {
        let mut s = String::new();
        // guaranteed to be Ok(())
        let result = self.to_csa(&mut s);
        debug_assert_eq!(result, Ok(()));
        s
    }
}

impl ToCSA for JsonKifFormat {
    fn to_csa<W: Write>(&self, sink: &mut W) -> Result {
        write_header(&self.header, sink)?;
        write_initial(&self.initial, sink)?;
        write_moves(&self.moves[1..], sink)?;
        Ok(())
    }
}

fn write_color<W: Write>(c: Color, sink: &mut W) -> Result {
    match c {
        Color::Black => sink.write_str("+")?,
        Color::White => sink.write_str("-")?,
    }
    Ok(())
}

fn write_kind<W: Write>(kind: Kind, sink: &mut W) -> Result {
    match kind {
        Kind::FU => sink.write_str("FU")?,
        Kind::KY => sink.write_str("KY")?,
        Kind::KE => sink.write_str("KE")?,
        Kind::GI => sink.write_str("GI")?,
        Kind::KI => sink.write_str("KI")?,
        Kind::KA => sink.write_str("KA")?,
        Kind::HI => sink.write_str("HI")?,
        Kind::OU => sink.write_str("OU")?,
        Kind::TO => sink.write_str("TO")?,
        Kind::NY => sink.write_str("NY")?,
        Kind::NK => sink.write_str("NK")?,
        Kind::NG => sink.write_str("NG")?,
        Kind::UM => sink.write_str("UM")?,
        Kind::RY => sink.write_str("RY")?,
    }
    Ok(())
}

fn write_place<W: Write>(place: &Option<PlaceFormat>, sink: &mut W) -> Result {
    if let Some(p) = place {
        sink.write_fmt(format_args!("{}{}", p.x, p.y))?;
    } else {
        sink.write_str("00")?;
    }
    Ok(())
}

fn write_header<W: Write>(header: &HashMap<String, String>, sink: &mut W) -> Result {
    sink.write_str("V2.2\n")?;
    if let Some(s) = header.get("先手").or_else(|| header.get("下手")) {
        sink.write_fmt(format_args!("N+{}\n", s))?;
    }
    if let Some(s) = header.get("後手").or_else(|| header.get("上手")) {
        sink.write_fmt(format_args!("N-{}\n", s))?;
    }
    if let Some(s) = header.get("棋戦") {
        sink.write_fmt(format_args!("$EVENT:{}\n", s))?;
    }
    if let Some(s) = header.get("場所") {
        sink.write_fmt(format_args!("$SITE:{}\n", s))?;
    }
    // TODO: 開始日時
    // TODO: 終了日時
    // TODO: 持ち時間
    if let Some(s) = header.get("戦型") {
        sink.write_fmt(format_args!("$OPENING:{}\n", s))?;
    }
    Ok(())
}

fn write_initial_data<W: Write>(data: &StateFormat, sink: &mut W) -> Result {
    for i in 0..9 {
        sink.write_fmt(format_args!("P{}", i + 1))?;
        for j in 0..9 {
            let p = data.board[8 - j][i];
            if let (Some(c), Some(kind)) = (p.color, p.kind) {
                write_color(c, sink)?;
                write_kind(kind, sink)?;
            } else {
                sink.write_str(" * ")?;
            }
        }
        sink.write_str("\n")?;
    }
    for (i, hand) in data.hands.iter().enumerate() {
        if hand == &Hand::default() {
            continue;
        }
        // TODO: AL?
        if i == 0 {
            sink.write_str("P+")?;
        } else {
            sink.write_str("P-")?;
        }
        (0..hand.HI).try_for_each(|_| sink.write_str("00HI"))?;
        (0..hand.KA).try_for_each(|_| sink.write_str("00KA"))?;
        (0..hand.KI).try_for_each(|_| sink.write_str("00KI"))?;
        (0..hand.GI).try_for_each(|_| sink.write_str("00GI"))?;
        (0..hand.KE).try_for_each(|_| sink.write_str("00KE"))?;
        (0..hand.KY).try_for_each(|_| sink.write_str("00KY"))?;
        (0..hand.FU).try_for_each(|_| sink.write_str("00FU"))?;
        sink.write_str("\n")?;
    }
    write_color(data.color, sink)?;
    Ok(())
}

fn write_initial_preset<W: Write>(preset: Preset, sink: &mut W) -> Result {
    match preset {
        Preset::PresetHirate => sink.write_str("PI\n")?,
        Preset::PresetKY => sink.write_str("PI11KY\n")?,
        Preset::PresetKYR => sink.write_str("PI91KY\n")?,
        Preset::PresetKA => sink.write_str("PI22KA\n")?,
        Preset::PresetHI => sink.write_str("PI82HI\n")?,
        Preset::PresetHIKY => sink.write_str("PI82HI11KY\n")?,
        Preset::Preset2 => sink.write_str("PI82HI22KA\n")?,
        Preset::Preset4 => sink.write_str("PI82HI22KA91KY11KY\n")?,
        Preset::Preset6 => sink.write_str("PI82HI22KA91KY11KY81KE21KE\n")?,
        Preset::Preset8 => sink.write_str("PI82HI22KA91KY11KY81KE21KE71GI31GI\n")?,
        Preset::Preset10 => sink.write_str("PI82HI22KA91KY11KY81KE21KE71GI31GI61KI41KI\n")?,
        _ => unimplemented!(),
    }
    if preset == Preset::PresetHirate {
        sink.write_str("+")?;
    } else {
        sink.write_str("-")?;
    }
    Ok(())
}

fn write_initial<W: Write>(initial: &Option<Initial>, sink: &mut W) -> Result {
    if let Some(initial) = initial {
        if let Some(data) = &initial.data {
            write_initial_data(data, sink)?;
        } else {
            write_initial_preset(initial.preset, sink)?;
        }
    } else {
        sink.write_str("PI\n+")?;
    }
    sink.write_str("\n")?;
    Ok(())
}

fn write_moves<W: Write>(moves: &[MoveFormat], sink: &mut W) -> Result {
    for mv in moves {
        if let Some(mv) = mv.move_ {
            write_color(mv.color, sink)?;
            write_place(&mv.from, sink)?;
            write_place(&Some(mv.to), sink)?;
            let kind = if mv.promote.unwrap_or_default() {
                mv.piece.promoted()
            } else {
                mv.piece
            };
            write_kind(kind, sink)?;
        } else if let Some(special) = &mv.special {
            sink.write_str("%")?;
            match special {
                MoveSpecial::SpecialToryo => sink.write_str("TORYO")?,
                MoveSpecial::SpecialChudan => sink.write_str("CHUDAN")?,
                MoveSpecial::SpecialSennichite => sink.write_str("SENNICHITE")?,
                MoveSpecial::SpecialTimeUp => sink.write_str("TIME_UP")?,
                MoveSpecial::SpecialIllegalMove => sink.write_str("ILLEGAL_MOVE")?,
                MoveSpecial::SpecialIllegalActionBlack => sink.write_str("+ILLEGAL_ACTION")?,
                MoveSpecial::SpecialIllegalActionWhite => sink.write_str("-ILLEGAL_ACTION")?,
                MoveSpecial::SpecialJishogi => sink.write_str("JISHOGI")?,
                MoveSpecial::SpecialKachi => sink.write_str("KACHI")?,
                MoveSpecial::SpecialHikiwake => sink.write_str("HIKIWAKE")?,
                MoveSpecial::SpecialMatta => sink.write_str("MATTA")?,
                MoveSpecial::SpecialTsumi => sink.write_str("TSUMI")?,
                MoveSpecial::SpecialFuzumi => sink.write_str("FUZUMI")?,
                MoveSpecial::SpecialError => sink.write_str("ERROR")?,
            }
        } else {
            unreachable!()
        }
        sink.write_str("\n")?;
        if let Some(time) = &mv.time {
            let sec = time.now.h.unwrap_or_default() as u64 * 3600
                + time.now.m as u64 * 60
                + time.now.s as u64;
            sink.write_fmt(format_args!("T{}\n", sec))?;
        }
        if let Some(comments) = &mv.comments {
            for comment in comments {
                sink.write_fmt(format_args!("'{}\n", comment))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_csa_default() {
        assert_eq!(
            r#"
V2.2
PI
+
"#[1..],
            JsonKifFormat::default().to_csa_owned()
        );
    }
}
