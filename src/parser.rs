//! Parsers for [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)

mod kakinoki;
mod ki2;
mod kif;

use crate::error::ParseError;
use crate::jkf::JsonKifuFormat;
use encoding_rs::{SHIFT_JIS, UTF_8};
use nom::error::convert_error;
use nom::Finish;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Parses a CSA file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the file.
pub fn parse_csa_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ParseError> {
    let mut file = File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    parse_csa_str(&buf)
}

/// Parses a CSA formatted string to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the string.
pub fn parse_csa_str(s: &str) -> Result<JsonKifuFormat, ParseError> {
    let mut jkf = JsonKifuFormat::try_from(csa::parse_csa(s)?)?;
    if let Err(err) = jkf.normalize() {
        Err(ParseError::Normalize(err.to_string()))
    } else {
        Ok(jkf)
    }
}

// The KIF and KI2 grammars read whole lines, each ending in a line ending, and reject
// input they cannot read to the end. A last line without one would be left over:
// `data/tests/kif/piyo-shogi_KY.kif` ends that way.
fn with_final_line_ending(s: &str) -> Cow<'_, str> {
    if s.is_empty() || s.ends_with('\n') {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(format!("{s}\n"))
    }
}

/// Parses a KIF file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// If the file extension is `.kif`, it is decoded as Shift-JIS, and if it is `.kifu`, it is decoded as UTF-8 and parsed.
///
/// See: [http://kakinoki.o.oo7.jp/kif_format.html](http://kakinoki.o.oo7.jp/kif_format.html)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the file.
pub fn parse_kif_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ParseError> {
    let mut file = File::open(&path)?;
    let ext = path.as_ref().extension().ok_or(ParseError::FileExtension)?;
    let encoding = match ext.to_str() {
        Some("kif") => SHIFT_JIS,
        Some("kifu") => UTF_8,
        _ => return Err(ParseError::FileExtension),
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let (cow, _, had_errors) = encoding.decode(&buf);
    if had_errors {
        return Err(ParseError::Decode);
    }
    parse_kif_str(&cow)
}

/// Parses a KIF formatted string to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the string.
pub fn parse_kif_str(s: &str) -> Result<JsonKifuFormat, ParseError> {
    let s = with_final_line_ending(s);
    match kif::parse(&s).finish() {
        Ok((_, mut jkf)) => {
            if let Err(err) = jkf.normalize() {
                Err(ParseError::Normalize(err.to_string()))
            } else {
                Ok(jkf)
            }
        }
        Err(err) => Err(ParseError::Kif(convert_error(&*s, err))),
    }
}

/// Parses a KI2 file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// If the file extension is `.ki2`, it is decoded as Shift-JIS, and if it is `.ki2u`, it is decoded as UTF-8 and parsed.
///
/// See: [http://kakinoki.o.oo7.jp/KifuwInt.htm](http://kakinoki.o.oo7.jp/KifuwInt.htm)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the file.
pub fn parse_ki2_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ParseError> {
    let mut file = File::open(&path)?;
    let ext = path.as_ref().extension().ok_or(ParseError::FileExtension)?;
    let encoding = match ext.to_str() {
        Some("ki2") => SHIFT_JIS,
        Some("ki2u") => UTF_8,
        _ => return Err(ParseError::FileExtension),
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let (cow, _, had_errors) = encoding.decode(&buf);
    if had_errors {
        return Err(ParseError::Decode);
    }
    parse_ki2_str(&cow)
}

/// Parses a KI2 formatted string to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the string.
pub fn parse_ki2_str(s: &str) -> Result<JsonKifuFormat, ParseError> {
    let s = with_final_line_ending(s);
    match ki2::parse(&s).finish() {
        Ok((_, mut jkf)) => {
            if let Err(err) = jkf.normalize() {
                Err(ParseError::Normalize(err.to_string()))
            } else {
                Ok(jkf)
            }
        }
        Err(err) => Err(ParseError::Ki2(convert_error(&*s, err))),
    }
}

/// Parses a JSON file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the file.
pub fn parse_jkf_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ParseError> {
    let file = File::open(&path)?;
    let mut jkf = serde_json::from_reader::<_, JsonKifuFormat>(BufReader::new(file))?;
    if let Err(err) = jkf.normalize() {
        Err(ParseError::Normalize(err.to_string()))
    } else {
        Ok(jkf)
    }
}

/// Parses a JSON file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// # Errors
///
/// This function returns [`ConvertError`](crate::error::ConvertError) if it fails to parse the file.
pub fn parse_jkf_str(s: &str) -> Result<JsonKifuFormat, ParseError> {
    let mut jkf = serde_json::from_str::<JsonKifuFormat>(s)?;
    if let Err(err) = jkf.normalize() {
        Err(ParseError::Normalize(err.to_string()))
    } else {
        Ok(jkf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::ffi::OsStr;
    use std::io::Result;

    #[test]
    fn jkf_with_empty_moves() {
        // The JKF schema puts no lower bound on `moves`, so `[]` is valid input
        // and must not panic. It is left as-is, not padded with a lead-in entry.
        let jkf = parse_jkf_str(r#"{"header":{},"moves":[]}"#).expect("should parse");
        assert!(jkf.moves.is_empty());

        // The converters make the same `moves[0]` / `moves[1..]` assumption.
        use crate::converter::{ToCsa, ToKi2, ToKif};
        assert_eq!(jkf.to_csa_owned(), "V2.2\nPI\n+\n");
        assert_eq!(jkf.to_kif_owned(), "手数----指手---------消費時間--\n");
        assert_eq!(jkf.to_ki2_owned(), "\n");
    }

    #[test]
    fn kif_fork_beyond_main_line() {
        // The ply in a `変化` header is taken from the file, so it can name a ply the
        // line it attaches to does not have. That must be a parse error, not a panic.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n\n変化：5手\n   5 ８四歩(83)\n";
        assert!(parse_kif_str(kif).is_err());

        // A fork that does name an existing ply still works.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n\n変化：1手\n   1 ２六歩(27)\n";
        let jkf = parse_kif_str(kif).expect("should parse");
        assert_eq!(jkf.moves.len(), 2);
        assert_eq!(jkf.moves[1].forks.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn kif_handicap_presets_round_trip() {
        use crate::converter::{ToCsa, ToKif};

        // Every 手合割 the parser accepts must survive parse -> convert -> parse.
        // 三枚落ち, 五枚落ち, 左五枚落ち, 左七枚落ち and 右七枚落ち used to panic.
        #[rustfmt::skip]
        let cases = [
            ("平手", "PI"),
            ("香落ち", "PI11KY"),
            ("右香落ち", "PI91KY"),
            ("角落ち", "PI22KA"),
            ("飛車落ち", "PI82HI"),
            ("飛香落ち", "PI82HI11KY"),
            ("二枚落ち", "PI82HI22KA"),
            ("三枚落ち", "PI82HI22KA11KY"),
            ("四枚落ち", "PI82HI22KA91KY11KY"),
            ("五枚落ち", "PI82HI22KA91KY11KY81KE"),
            ("左五枚落ち", "PI82HI22KA91KY11KY21KE"),
            ("六枚落ち", "PI82HI22KA91KY11KY81KE21KE"),
            ("左七枚落ち", "PI82HI22KA91KY11KY81KE21KE31GI"),
            ("右七枚落ち", "PI82HI22KA91KY11KY81KE21KE71GI"),
            ("八枚落ち", "PI82HI22KA91KY11KY81KE21KE71GI31GI"),
            ("十枚落ち", "PI82HI22KA91KY11KY81KE21KE71GI31GI61KI41KI"),
        ];
        for (name, pi) in cases {
            let kif = format!("手合割：{name}\n手数----指手---------消費時間--\n");
            let jkf = parse_kif_str(&kif).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(jkf.to_kif_owned(), kif, "{name}");
            assert!(jkf.to_csa_owned().contains(&format!("\n{pi}\n")), "{name}");
            // The same layout arriving as board data has to come back as the same preset.
            // KY_R and the five presets above that used to panic stayed `PresetOther`, so
            // `csa2kif` wrote a diagram where `kif2kif` wrote the name.
            let turn = if name == "平手" { '+' } else { '-' };
            let csa = parse_csa_str(&format!("V2.2\n{pi}\n{turn}\n"))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(csa.initial, jkf.initial, "{name}");
            assert_eq!(parse_kif_str(&jkf.to_kif_owned()).ok(), Some(jkf), "{name}");
        }
    }

    #[test]
    fn csa_square_off_the_board() {
        // `csa::Square` is two bare digits with no range check. File 0 with a non-zero
        // rank is not the "in hand" encoding, and used to underflow a board index.
        assert!(parse_csa_str("V2.2\nP+01FU\nP-51OU\n+\n").is_err());
        assert!(parse_csa_str("V2.2\nP+90FU\nP-51OU\n+\n").is_err());
        // `PI` with an off-board drop square.
        assert!(parse_csa_str("V2.2\nPI09KY\n+\n").is_err());
        // `(0, 0)` is the legal "in hand" encoding and must still be accepted.
        let jkf = parse_csa_str("V2.2\nP+55FU\nP-51OU\nP+59OU\nP+00FU\n+\n").expect("in hand");
        let data = jkf.initial.and_then(|i| i.data).expect("initial data");
        assert_eq!(data.hands[0].FU, 1);
    }

    #[test]
    fn off_board_move_square_is_not_a_drop() {
        // `normalize_move` used to fall back to `from: None` for any from-square it could
        // not convert, so an off-board origin was read as a drop of the named piece.
        for csa in ["V2.2\nPI\n+\n+0576FU\n", "V2.2\nPI\n+\n+7770FU\n"] {
            let err = parse_csa_str(csa).expect_err(csa);
            assert!(matches!(err, ParseError::CsaConvert(_)), "{err:?}");
        }
        // With that piece in hand the made-up drop was a legal move, and the record was
        // accepted with a move it does not contain.
        let csa = "V2.2\nP+59OU\nP-51OU\nP+00FU\n+\n+0576FU\n";
        let err = parse_csa_str(csa).expect_err("off-board origin");
        assert!(matches!(err, ParseError::CsaConvert(_)), "{err:?}");
        // `00` is the from-square of a real drop, and still reads as one.
        let csa = "V2.2\nP+59OU\nP-51OU\nP+00FU\n+\n+0076FU\n";
        let jkf = parse_csa_str(csa).expect("should parse");
        assert_eq!(jkf.moves[1].move_.map(|m| m.from), Some(None));

        // KIF's `(90)` took the same path.
        let kif = "手合割：その他\n後手の持駒：なし\n  ９ ８ ７ ６ ５ ４ ３ ２ １\n+---------------------------+\n| ・ ・ ・ ・v玉 ・ ・ ・ ・|一\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|二\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|三\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|四\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|五\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|六\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|七\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|八\n| ・ ・ ・ ・ 玉 ・ ・ ・ ・|九\n+---------------------------+\n先手の持駒：歩\n手数----指手---------消費時間--\n   1 ７六歩(90)\n";
        let err = parse_kif_str(kif).expect_err("off-board origin");
        assert!(matches!(err, ParseError::Normalize(_)), "{err:?}");
        let kif = kif.replace("７六歩(90)", "７六歩打");
        assert!(parse_kif_str(&kif).is_ok());
    }

    #[test]
    fn move_entry_with_neither_move_nor_special() {
        use crate::converter::{ToCsa, ToKi2, ToKif};
        use crate::jkf::{Initial, Preset};
        use shogi_core::ToUsi;

        // Every field of `MoveFormat` is optional, so `{}` deserializes, and
        // `normalize_moves` stopped at one without a word. The KIF and CSA writers then
        // hit `unreachable!()` on it, while KI2 quietly wrote nothing for it.
        let p76 = r#"{"color":0,"from":{"x":7,"y":7},"to":{"x":7,"y":6},"piece":"FU"}"#;
        let p34 = r#"{"color":1,"from":{"x":3,"y":3},"to":{"x":3,"y":4},"piece":"FU"}"#;
        let record = |moves: &str| format!(r#"{{"header":{{}},"moves":[{{}},{moves}]}}"#);
        for json in [
            record("{}"),
            // after a special, which `normalize_moves` does not replay
            record(&format!(r#"{{"move":{p76}}},{{"special":"TORYO"}},{{}}"#)),
            // inside a fork
            record(&format!(r#"{{"move":{p76},"forks":[[{{}}]]}}"#)),
        ] {
            let err = parse_jkf_str(&json).expect_err(&json);
            assert!(matches!(err, ParseError::Normalize(_)), "{json}: {err:?}");
        }

        // A value that never went through `normalize` must not panic a writer either: the
        // entry is left out, as if it were not there.
        let json = record(&format!(r#"{{"move":{p76}}},{{}},{{"move":{p34}}}"#));
        let with = serde_json::from_str::<JsonKifuFormat>(&json).expect("deserializes");
        let mut without = with.clone();
        without.moves.remove(2);
        assert_eq!(with.to_kif_owned(), without.to_kif_owned());
        assert_eq!(with.to_csa_owned(), without.to_csa_owned());
        assert_eq!(with.to_ki2_owned(), without.to_ki2_owned());
        assert_eq!(with.to_usi_owned(), "startpos moves 7g7f 3c3d");

        // `PresetOther` with no board has no position to write, in CSA or in USI.
        let jkf = JsonKifuFormat {
            initial: Some(Initial {
                preset: Preset::PresetOther,
                data: None,
            }),
            ..Default::default()
        };
        assert_eq!(jkf.to_csa_owned(), "V2.2\n");
        assert_eq!(jkf.to_usi_owned(), "");
    }

    #[test]
    fn moves_after_a_special_are_not_played() {
        use shogi_core::ToUsi;

        // `normalize` stops at the first special and does not check what follows it, but
        // `Position::try_from` went on playing those moves, so a record the parser had
        // accepted made `to_usi` panic.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n   2 投了\n   3 ７六歩(77)\n";
        let jkf = parse_kif_str(kif).expect("should parse");
        assert_eq!(jkf.to_usi_owned(), "startpos moves 7g7f");

        // `moves[0]` is the initial position's comment slot, which `normalize` and the
        // writers skip, but `Position::try_from` did not: a special there stopped it
        // before the first move, and a move there was played.
        let p76 = r#"{"color":0,"from":{"x":7,"y":7},"to":{"x":7,"y":6},"piece":"FU"}"#;
        let p26 = r#"{"color":0,"from":{"x":2,"y":7},"to":{"x":2,"y":6},"piece":"FU"}"#;
        for first in [
            String::from(r#"{"special":"TORYO"}"#),
            format!(r#"{{"move":{p76}}}"#),
        ] {
            let json = format!(r#"{{"header":{{}},"moves":[{first},{{"move":{p26}}}]}}"#);
            let jkf = parse_jkf_str(&json).expect("should parse");
            assert_eq!(jkf.to_usi_owned(), "startpos moves 2g2f", "{first}");
        }
    }

    #[test]
    fn ki2_disambiguated_promotion() {
        use crate::converter::ToKi2;
        use crate::jkf::{PlaceFormat, Relative};

        // Black silvers on 2四 and 4四 both reach 3三, inside the promotion zone, so the
        // move needs a disambiguator and 成/不成 at once. The parser wanted 成/不成 first,
        // the reverse of the official order its own writer uses: `▲３三銀左` was read,
        // its promotion came back as 不成, and the rest of the game was dropped.
        let board = "手合割：その他\n後手の持駒：なし\n  ９ ８ ７ ６ ５ ４ ３ ２ １\n+---------------------------+\n| ・ ・ ・ ・v玉 ・ ・ ・ ・|一\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|二\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|三\n| ・ ・ ・ ・ ・ 銀 ・ 銀 ・|四\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|五\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|六\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|七\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|八\n| ・ ・ ・ ・ 玉 ・ ・ ・ ・|九\n+---------------------------+\n先手の持駒：なし\n";
        for (mv, from, promote, relative) in [
            ("▲３三銀左成", (4, 4), true, Relative::L),
            ("▲３三銀右成", (2, 4), true, Relative::R),
            ("▲３三銀左不成", (4, 4), false, Relative::L),
        ] {
            let ki2 = format!("{board}{mv} △５二玉\n");
            let jkf = parse_ki2_str(&ki2).unwrap_or_else(|e| panic!("{mv}: {e}"));
            assert_eq!(jkf.moves.len(), 3, "{mv}");
            let m = jkf.moves[1].move_.expect("move");
            assert_eq!(
                (m.from, m.promote, m.relative),
                (
                    Some(PlaceFormat {
                        x: from.0,
                        y: from.1
                    }),
                    Some(promote),
                    Some(relative)
                ),
                "{mv}"
            );
            // The writer spells it the same way, and that reads back.
            let written = jkf.to_ki2_owned();
            assert!(written.contains(mv), "{mv}: {written}");
            assert_eq!(parse_ki2_str(&written).ok(), Some(jkf), "{mv}");
        }

        // From KIF the disambiguator is worked out from the position, and that stopped at
        // the trailing 成: the KI2 writer got no disambiguator and wrote `▲３三銀成`,
        // which cannot be read back.
        let kif =
            format!("{board}手数----指手---------消費時間--\n   1 ３三銀成(44)\n   2 ５二玉(51)\n");
        let jkf = parse_kif_str(&kif).expect("should parse");
        let written = jkf.to_ki2_owned();
        assert!(written.contains("▲３三銀左成"), "{written}");
        assert_eq!(parse_ki2_str(&written).ok(), Some(jkf));
    }

    #[test]
    fn kif_unread_input_is_an_error() {
        // `parse_kif_str` threw away whatever nom left unread, so a line no rule takes
        // ended the game there and the call still returned `Ok`. KIF has no word for
        // JKF's HIKIWAKE or MATTA, so these are not moves.
        let head =
            "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n   2 ３四歩(33)\n";
        for line in ["   3 引き分け\n", "   3 待った\n", "   3 ２六歩(27\n"] {
            let kif = format!("{head}{line}   4 ８四歩(83)\n");
            let err = parse_kif_str(&kif).expect_err(line);
            assert!(matches!(err, ParseError::Kif(_)), "{line}: {err:?}");
        }
        // Trailing blank lines are not unread input, and neither is a last line with no
        // line ending, which used to be dropped.
        for tail in ["\n\n", "  \n", "まで2手で中断", "   3 ２六歩(27)"] {
            let kif = format!("{head}{tail}");
            let jkf = parse_kif_str(&kif).unwrap_or_else(|e| panic!("{tail:?}: {e}"));
            let moves = if tail.ends_with("(27)") { 4 } else { 3 };
            assert_eq!(jkf.moves.len(), moves, "{tail:?}");
        }
    }

    #[test]
    fn kif_blank_lines_and_comment_lines() {
        // A blank line was read as a line of its own with CRLF, and together with the next
        // line with LF. So with CRLF it took the one place for a line after the moves, and
        // a `まで…` after it was unread input; with LF, a move or comment after it was
        // lost. Blank lines now mean nothing in between the moves.
        let moves = [
            "手合割：平手",
            "手数----指手---------消費時間--",
            "   1 ７六歩(77)",
            "",
            "   2 ３四歩(33)",
            "",
            "*二手目",
            "   3 投了",
        ];
        // `#` lines are comments, and IS-SHOGI ends a record with one.
        for eol in ["\n", "\r\n"] {
            for tail in [
                &["", "まで2手で後手の勝ち"][..],
                &["まで2手で後手の勝ち", "#コメント"],
                &["", "", "まで2手で後手の勝ち", "", "#コメント", ""],
            ] {
                let kif = [&moves[..], tail, &[""]].concat().join(eol);
                let jkf = parse_kif_str(&kif).unwrap_or_else(|e| panic!("{kif:?}: {e}"));
                assert_eq!(jkf.moves.len(), 4, "{kif:?}");
                assert_eq!(
                    jkf.moves[2].comments,
                    Some(vec![String::from("二手目")]),
                    "{kif:?}"
                );
            }
        }
        // Even a record with no moves.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n#--separator--\n";
        let jkf = parse_kif_str(kif).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(jkf.moves.len(), 1);
    }

    #[test]
    fn sealed_move() {
        use crate::converter::ToKif;
        use crate::jkf::MoveSpecial::SpecialChudan;

        // A record saved while the game is adjourned ends at the sealed move, and had no
        // rule. json-kifu-format reads 封じ手 as CHUDAN, and so does this, so the KIF
        // writer puts it back as 中断.
        let head =
            "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n   2 ３四歩(33)\n";
        let jkf = parse_kif_str(&format!("{head}   3 封じ手\n")).expect("should parse");
        assert_eq!(jkf.moves[3].special, Some(SpecialChudan));
        assert_eq!(jkf.to_kif_owned(), format!("{head}   3 中断\n"));
    }

    #[test]
    fn kif_fouls_and_unsolved_mates() {
        use crate::converter::ToKif;
        use crate::jkf::MoveSpecial::*;

        // 不詰 and 反則勝ち had no rule, so a game ending in one lost that line, and would
        // now fail to parse. 反則勝ち is a win for the side to move on the other side's
        // foul; which side is to move follows from the ply, and 上手 (White) moves first
        // in a handicap game.
        let hirate = "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n";
        let kyo = "手合割：香落ち\n手数----指手---------消費時間--\n   1 ５四歩(53)\n";
        for (kif, ply, special) in [
            (
                format!("{hirate}   2 ３四歩(33)\n   3 不詰\n"),
                3,
                SpecialFuzumi,
            ),
            (
                format!("{hirate}   2 反則勝ち\n"),
                2,
                SpecialIllegalActionBlack,
            ),
            (
                format!("{hirate}   2 ３四歩(33)\n   3 反則勝ち\n"),
                3,
                SpecialIllegalActionWhite,
            ),
            (
                format!("{kyo}   2 反則勝ち\n"),
                2,
                SpecialIllegalActionWhite,
            ),
            (
                format!("{kyo}   2 ７六歩(77)\n   3 反則勝ち\n"),
                3,
                SpecialIllegalActionBlack,
            ),
        ] {
            let jkf = parse_kif_str(&kif).unwrap_or_else(|e| panic!("{kif}: {e}"));
            assert_eq!(jkf.moves.len(), ply + 1, "{kif}");
            assert_eq!(jkf.moves[ply].special, Some(special), "{kif}");
            // The writer puts the same word back.
            assert_eq!(jkf.to_kif_owned(), kif);
        }

        // A JKF (or CSA) foul names its side, which can be the side to move: an illegal 入玉
        // declaration, say. KIF writes that as the side to move's loss. Both used to come
        // out as 中断, and so did FUZUMI.
        for (special, word) in [
            ("+ILLEGAL_ACTION", "反則勝ち"),
            ("-ILLEGAL_ACTION", "反則負け"),
            ("FUZUMI", "不詰"),
        ] {
            let json = format!(
                r#"{{"header":{{}},"moves":[{{}},{{"move":{{"color":0,"from":{{"x":7,"y":7}},"to":{{"x":7,"y":6}},"piece":"FU"}}}},{{"special":"{special}"}}]}}"#
            );
            let kif = parse_jkf_str(&json).expect("should parse").to_kif_owned();
            assert!(kif.contains(&format!("   2 {word}\n")), "{special}: {kif}");
        }

        // The 中断 arm forgot to count its own width, so the time column ran four columns
        // wide of the other lines.
        let jkf = parse_kif_file("data/tests/kif/example.kif").expect("should parse");
        assert!(jkf
            .to_kif_owned()
            .contains("   3 中断         ( 0:03/00:00:19)\n"));
    }

    #[test]
    fn ki2_unread_input_is_an_error() {
        // `many0(single_move)` stopped at anything it could not take, and the rest of the
        // input was thrown away: a stray line, a 変化 block (which KI2 parsing does not
        // support), or a file that is not a record at all.
        for ki2 in [
            "▲２六歩 △８四歩\n投了\n",
            "▲２六歩 △８四歩\n変化：2手\n△３四歩\n",
            "これは棋譜ではありません\n",
        ] {
            let err = parse_ki2_str(ki2).expect_err(ki2);
            assert!(matches!(err, ParseError::Ki2(_)), "{ki2}: {err:?}");
        }
        // A blank line between moves does not end the game.
        let jkf = parse_ki2_str("▲２六歩\n\n△８四歩\n").expect("should parse");
        assert_eq!(jkf.moves.len(), 3);
    }

    #[test]
    fn ki2_result_line() {
        use crate::jkf::MoveSpecial::*;

        // The `まで…` line is the only place KI2 says how the game ended. It used to be left
        // unread, so the special was lost.
        let moves = "▲２六歩 △８四歩\n";
        for (line, special) in [
            ("まで2手で後手の勝ち", Some(SpecialToryo)),
            ("まで2手で時間切れにより後手の勝ち", Some(SpecialTimeUp)),
            // The winner is named, so the foul is the other side's.
            ("まで2手で後手の反則勝ち", Some(SpecialIllegalActionBlack)),
            ("まで2手で先手の反則勝ち", Some(SpecialIllegalActionWhite)),
            ("まで2手で先手の反則負け", Some(SpecialIllegalMove)),
            ("まで2手で先手の入玉勝ち", Some(SpecialKachi)),
            ("まで2手で中断", Some(SpecialChudan)),
            ("まで2手で封じ手", Some(SpecialChudan)),
            ("まで2手で持将棋", Some(SpecialJishogi)),
            ("まで2手で千日手", Some(SpecialSennichite)),
            ("まで2手で詰み", Some(SpecialTsumi)),
            ("まで2手詰", Some(SpecialTsumi)),
            ("まで2手で不詰", Some(SpecialFuzumi)),
            // A wording this does not know still closes the record: the moves are all there.
            ("まで2手で終局", None),
        ] {
            // With and without a line ending after it.
            for ki2 in [format!("{moves}{line}\n"), format!("{moves}{line}")] {
                let jkf = parse_ki2_str(&ki2).unwrap_or_else(|e| panic!("{line}: {e}"));
                assert_eq!(
                    jkf.moves.len(),
                    3 + usize::from(special.is_some()),
                    "{line}"
                );
                assert_eq!(
                    jkf.moves.last().and_then(|mf| mf.special),
                    special,
                    "{line}"
                );
            }
        }
    }

    #[test]
    fn ki2_comments_after_the_last_move() {
        use crate::converter::ToKi2;
        use crate::jkf::MoveSpecial::*;

        let comments = |v: &[&str]| Some(v.iter().map(|&s| String::from(s)).collect());
        // tsshogi writes a special's comments and bookmark after the `まで…` line, and
        // reads them back onto the special. No rule read them, so the record was rejected.
        for eol in ["\n", "\r\n"] {
            let ki2 = [
                "手合割：平手",
                "▲７六歩    △３四歩",
                "*最終手",
                "まで2手で後手の勝ち",
                "*投了図",
                "&終局",
                "",
            ]
            .join(eol);
            let jkf = parse_ki2_str(&ki2).unwrap_or_else(|e| panic!("{eol:?}: {e}"));
            assert_eq!(jkf.moves.len(), 4, "{eol:?}");
            assert_eq!(jkf.moves[2].comments, comments(&["最終手"]), "{eol:?}");
            assert_eq!(jkf.moves[3].special, Some(SpecialToryo), "{eol:?}");
            assert_eq!(
                jkf.moves[3].comments,
                comments(&["投了図", "&終局"]),
                "{eol:?}"
            );
            // `#` lines are comments, as in KIF.
            let jkf = parse_ki2_str(&format!("{ki2}#コメント{eol}"))
                .unwrap_or_else(|e| panic!("{eol:?}: {e}"));
            assert_eq!(jkf.moves.len(), 4, "{eol:?}");
        }
        // With no `まで…` line, comments past a blank line go to the last move. This
        // crate's writer leaves a blank line before a special's comments when the last
        // move has comments too, and writes no `まで…` line, so its own output was
        // rejected.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n   2 ３四歩(33)\n*最終手\n   3 投了\n*投了図\n";
        let written = parse_kif_str(kif).expect("should parse").to_ki2_owned();
        let jkf = parse_ki2_str(&written).unwrap_or_else(|e| panic!("{written}: {e}"));
        assert_eq!(jkf.moves.len(), 3, "{written}");
        assert_eq!(jkf.moves[2].comments, comments(&["最終手", "投了図"]));
        // `floodgate.json` has that shape: an evaluation on its last move, and the game
        // summary on its TORYO. All of it reads back but the TORYO, which the KI2 writer
        // does not write.
        let jkf = parse_jkf_file("data/tests/csa/floodgate.json").expect("should parse");
        let written = jkf.to_ki2_owned();
        let reparsed = parse_ki2_str(&written).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(reparsed.moves.len(), jkf.moves.len() - 1);
    }

    #[test]
    fn ki2_result_token() {
        use crate::jkf::MoveSpecial::*;

        // Shogi DB2 writes how the game ended as one more "move" before the `まで…` line,
        // with a mark that need not be the side to move's. These are tsshogi's samples of
        // it: the token was left unread, so each was rejected.
        let opening = "▲２六歩 △３四歩 ▲７六歩 △５四歩 ▲４八銀 △５二飛 ▲６八玉 △５五歩";
        for (ki2, len, special) in [
            (
                format!("{opening} ▲７八玉 △投了\nまで9手で先手の勝ち"),
                11,
                SpecialToryo,
            ),
            (
                format!("{opening} △投了\nまで8手で後手の勝ち"),
                10,
                SpecialToryo,
            ),
            (
                format!("{opening} ▲７八玉 △千日手\nまで9手で千日手"),
                11,
                SpecialSennichite,
            ),
            (
                format!("{opening} △千日手\nまで8手で千日手"),
                10,
                SpecialSennichite,
            ),
            // With no `まで…` line, the token says how the game ended. 反則勝ち is a win
            // for the side to move, on the foul of whoever moved last.
            (format!("{opening} ▲投了\n"), 10, SpecialToryo),
            (
                format!("{opening}\n▲反則勝ち\n"),
                10,
                SpecialIllegalActionWhite,
            ),
        ] {
            let jkf = parse_ki2_str(&ki2).unwrap_or_else(|e| panic!("{ki2}: {e}"));
            assert_eq!(jkf.moves.len(), len, "{ki2}");
            assert_eq!(
                jkf.moves.last().and_then(|mf| mf.special),
                Some(special),
                "{ki2}"
            );
        }
        // It ends the moves.
        let err = parse_ki2_str("▲２六歩 △投了 ▲７六歩\n").expect_err("a token mid-game");
        assert!(matches!(err, ParseError::Ki2(_)), "{err:?}");
    }

    #[test]
    fn kif_declined_promotion() {
        // A `不成` move line used to fail to parse, which stopped `many1` and left the
        // rest of the file unconsumed — the call still returned `Ok`, with the game
        // silently truncated at that move.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n   1 ７六歩(77)\n   2 ３四歩(33)\n   3 ２二角不成(88)\n   4 ３二金(41)\n";
        let jkf = parse_kif_str(kif).expect("should parse");
        assert_eq!(jkf.moves.len(), 5);
        assert_eq!(jkf.moves[3].move_.map(|m| m.promote), Some(Some(false)));

        // A `変化` block after a `不成` line was dropped along with the moves.
        let kif = format!("{kif}\n変化：4手\n   4 ８四歩(83)\n");
        let jkf = parse_kif_str(&kif).expect("should parse");
        assert_eq!(jkf.moves.len(), 5);
        assert_eq!(jkf.moves[4].forks.as_ref().map(Vec::len), Some(1));

        // The KIF spec leaves a declined promotion unmarked, so the writer does not
        // put `不成` back. The position is unchanged either way.
        use crate::converter::ToKif;
        assert!(jkf.to_kif_owned().contains("   3 ２二角(88)"));
    }

    #[test]
    fn kif_initial_side_to_move() {
        use crate::converter::ToKif;
        use crate::jkf::Color;

        // Two kings: White's on 5一, Black's on 5九.
        let board = "手合割：その他\n後手の持駒：なし\n  ９ ８ ７ ６ ５ ４ ３ ２ １\n+---------------------------+\n| ・ ・ ・ ・v玉 ・ ・ ・ ・|一\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|二\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|三\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|四\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|五\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|六\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|七\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|八\n| ・ ・ ・ ・ 玉 ・ ・ ・ ・|九\n+---------------------------+\n先手の持駒：なし\n";

        // White to move. The `後手番` line used to be swallowed as a non-move line, the
        // initial state was hard-coded to Black, and move colours came from the ply
        // parity, so the whole game came back inverted — here the White king's move
        // would have been attributed to Black.
        let kif = format!("{board}後手番\n手数----指手---------消費時間--\n   1 ５二玉(51)\n");
        let jkf = parse_kif_str(&kif).expect("should parse");
        let data = jkf.initial.and_then(|i| i.data).expect("initial data");
        assert_eq!(data.color, Color::White);
        assert_eq!(jkf.moves[1].move_.map(|m| m.color), Some(Color::White));
        // The writer puts the line back, so the record round-trips.
        assert_eq!(jkf.to_kif_owned(), kif);

        // Black to move writes no line, which is what readers take as the default, so
        // output for an existing record is unchanged.
        let kif = format!("{board}手数----指手---------消費時間--\n   1 ５八玉(59)\n");
        let jkf = parse_kif_str(&kif).expect("should parse");
        let data = jkf.initial.and_then(|i| i.data).expect("initial data");
        assert_eq!(data.color, Color::Black);
        assert_eq!(jkf.moves[1].move_.map(|m| m.color), Some(Color::Black));
        assert_eq!(jkf.to_kif_owned(), kif);
    }

    #[test]
    fn kif_hand_counts_round_trip() {
        use crate::converter::ToKif;
        use crate::error::NormalizeError;
        use crate::jkf::*;
        use crate::normalizer::MAX_HAND_COUNT;

        let mut board = [[Piece::default(); 9]; 9];
        board[4][0] = Piece {
            color: Some(Color::White),
            kind: Some(Kind::OU),
        };
        board[4][8] = Piece {
            color: Some(Color::Black),
            kind: Some(Kind::OU),
        };
        let with_pawns = |n: u8| JsonKifuFormat {
            header: Default::default(),
            initial: Some(Initial {
                preset: Preset::PresetOther,
                data: Some(StateFormat {
                    color: Color::Black,
                    board,
                    hands: [
                        Hand {
                            FU: n,
                            ..Hand::default()
                        },
                        Hand::default(),
                    ],
                }),
            }),
            moves: vec![MoveFormat::default()],
        };

        // The counts from the issue, plus the bound. 19 and above used to render as
        // something the parser could not read: the hands line fell through to
        // `information_line_keyvalue` and the hand was lost without an error.
        for n in [18, 19, 20, 21, MAX_HAND_COUNT] {
            let kif = with_pawns(n).to_kif_owned();
            let back = parse_kif_str(&kif).unwrap_or_else(|e| panic!("{n}: {e}"));
            let data = back.initial.and_then(|i| i.data).expect("initial data");
            assert_eq!(data.hands[0].FU, n, "{n}");
        }

        // Past that a kifu has no notation for the count, so it is an error naming the
        // piece rather than output this crate cannot read back.
        let mut jkf = with_pawns(MAX_HAND_COUNT + 1);
        assert_eq!(
            jkf.normalize(),
            Err(NormalizeError::HandCountOutOfRange(Kind::FU, 100))
        );
    }

    #[test]
    fn kif_unreadable_hands_line_is_an_error() {
        // A hands line the parser cannot read used to be picked up by the key-value rule
        // and stored as `header["先手の持駒"]`: the call returned `Ok`, the position came
        // back with no hand, and writing it out produced two 先手の持駒 lines.
        let board = "  ９ ８ ７ ６ ５ ４ ３ ２ １\n+---------------------------+\n| ・ ・ ・ ・v玉 ・ ・ ・ ・|一\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|二\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|三\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|四\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|五\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|六\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|七\n| ・ ・ ・ ・ ・ ・ ・ ・ ・|八\n| ・ ・ ・ ・ 玉 ・ ・ ・ ・|九\n+---------------------------+\n";
        let kif = format!("手合割：その他\n後手の持駒：なし\n{board}先手の持駒：歩百　\n手数----指手---------消費時間--\n");
        let err = parse_kif_str(&kif).expect_err("should not be accepted");
        assert!(matches!(err, ParseError::Kif(_)), "{err:?}");

        // The same file with a count the format can express is read as a hand, not as a
        // header entry.
        let kif = kif.replace("歩百", "歩九十九");
        let jkf = parse_kif_str(&kif).expect("should parse");
        assert!(!jkf.header.contains_key("先手の持駒"));
        let data = jkf.initial.and_then(|i| i.data).expect("initial data");
        assert_eq!(data.hands[0].FU, 99);
    }

    #[test]
    fn kif_initial_bookmark_comment() {
        use crate::jkf::MoveFormat;

        // `&` marks the position a reader should open on. `not_move_line` excludes `*`
        // but not `&`, so skipping more than the 手数 header before the move block ate
        // the bookmark and `moves[0].comments` lost it.
        let kif = "手合割：平手\n手数----指手---------消費時間--\n&読み込み時表示\n*ふつうのコメント\n   1 ７六歩(77)\n";
        let jkf = parse_kif_str(kif).expect("should parse");
        assert_eq!(
            jkf.moves
                .first()
                .and_then(|mf: &MoveFormat| mf.comments.clone()),
            Some(vec![
                String::from("&読み込み時表示"),
                String::from("ふつうのコメント"),
            ])
        );
    }

    #[test]
    fn kif_handicap_with_a_fork() {
        use crate::jkf::Color;

        // 上手 moves first in a handicap game, but the parser colours every move from
        // the ply's parity and `normalize` inverts them afterwards. That inversion has
        // to reach the moves inside a `変化` block as well: it did not, so the fork kept
        // the parity colours, disagreed with the position, and the whole record was
        // rejected with `Invalid color`.
        let kif = "手合割：香落ち\n手数----指手---------消費時間--\n   1 ５四歩(53)\n   2 ７六歩(77)\n   3 ５二飛(82)\n\n変化：3手\n   3 ３四歩(33)\n";
        let jkf = parse_kif_str(kif).expect("should parse");

        let colors = |moves: &[crate::jkf::MoveFormat]| -> Vec<Color> {
            moves
                .iter()
                .filter_map(|mf| mf.move_.map(|m| m.color))
                .collect()
        };
        assert_eq!(
            colors(&jkf.moves),
            [Color::White, Color::Black, Color::White]
        );
        // The fork replaces ply 3, so it starts on the same side.
        let forks = jkf.moves[3].forks.as_ref().expect("fork at ply 3");
        assert_eq!(colors(&forks[0]), [Color::White]);
    }

    #[test]
    fn write_then_parse_round_trips_the_corpus() -> Result<()> {
        use crate::converter::{ToCsa, ToKi2, ToKif};
        use crate::jkf::{Color, Initial, Kind};

        // Every other fixture test stops at input -> JKF; nothing checked that the
        // writers emit something these parsers can read. That is the property #20 and
        // #21 touch, and 0.2.2 did not have it: its own `csa2kif` output for a
        // White-to-move board diagram was rejected by its own parser.
        //
        // KI2 carries no from-square and no times, so compare the initial position and
        // the part of each move every format does carry.
        // (color, piece, to.x, to.y, promote)
        type MoveEssence = (Color, Kind, u8, u8, Option<bool>);
        fn essence(jkf: &JsonKifuFormat) -> (Option<Initial>, Vec<MoveEssence>) {
            (
                jkf.initial,
                jkf.moves
                    .iter()
                    .filter_map(|mf| mf.move_)
                    .map(|m| (m.color, m.piece, m.to.x, m.to.y, m.promote))
                    .collect(),
            )
        }

        for (dir, ext) in [
            ("data/tests/csa", "csa"),
            ("data/tests/kif", "kif"),
            ("data/tests/ki2", "ki2"),
        ] {
            for entry in Path::new(dir).read_dir()? {
                let path = entry?.path();
                if path.extension() != Some(OsStr::new(ext)) {
                    continue;
                }
                let original = match ext {
                    "csa" => parse_csa_file(&path),
                    "kif" => parse_kif_file(&path),
                    _ => parse_ki2_file(&path),
                }
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                // Into KIF and KI2 from every format, not only back into its own. Into CSA
                // only from CSA: the csa crate stops at a comment with no text or one
                // holding a comma (`oui202106290101.kif` has both) and drops the rest
                // without an error.
                let mut written = vec![
                    ("KIF", parse_kif_str(&original.to_kif_owned())),
                    ("KI2", parse_ki2_str(&original.to_ki2_owned())),
                ];
                if ext == "csa" {
                    written.push(("CSA", parse_csa_str(&original.to_csa_owned())));
                }
                for (format, reparsed) in written {
                    let reparsed = reparsed.unwrap_or_else(|e| {
                        panic!(
                            "{} as {format}: reading our own output back: {e}",
                            path.display()
                        )
                    });
                    assert_eq!(
                        essence(&original),
                        essence(&reparsed),
                        "{} as {format}",
                        path.display()
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn utf8_fixtures_match_their_shift_jis_twins() -> Result<()> {
        // `.kifu` / `.ki2u` are the same records as their `.kif` / `.ki2` twins, so any
        // difference between the two parses is a bug in decoding. Walking from the UTF-8
        // side skips `oui202106290101.kif`, which has no twin.
        for (dir, utf8, shift_jis) in [
            ("data/tests/kif", "kifu", "kif"),
            ("data/tests/ki2", "ki2u", "ki2"),
        ] {
            let parse = |path: &Path| match utf8 {
                "kifu" => parse_kif_file(path),
                _ => parse_ki2_file(path),
            };
            let mut pairs = 0;
            for entry in Path::new(dir).read_dir()? {
                let path = entry?.path();
                if path.extension() != Some(OsStr::new(utf8)) {
                    continue;
                }
                let twin = path.with_extension(shift_jis);
                let from_utf8 = parse(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let from_shift_jis =
                    parse(&twin).unwrap_or_else(|e| panic!("{}: {e}", twin.display()));
                assert_eq!(from_utf8, from_shift_jis, "{}", path.display());
                pairs += 1;
            }
            assert!(pairs > 0, "no .{utf8} fixtures in {dir}");
        }
        Ok(())
    }

    #[test]
    fn ki2_initial_side_to_move() {
        use crate::converter::ToKi2;
        use crate::jkf::*;

        // `write_initial_data` and `informations` are shared, so #20's 後手番 line and the
        // hands rule it sits next to are on the KI2 path too — which had no coverage.
        let mut board = [[Piece::default(); 9]; 9];
        board[4][0] = Piece {
            color: Some(Color::White),
            kind: Some(Kind::OU),
        };
        board[4][8] = Piece {
            color: Some(Color::Black),
            kind: Some(Kind::OU),
        };
        let jkf = JsonKifuFormat {
            header: Default::default(),
            initial: Some(Initial {
                preset: Preset::PresetOther,
                data: Some(StateFormat {
                    color: Color::White,
                    board,
                    hands: [Hand::default(); 2],
                }),
            }),
            moves: vec![MoveFormat::default()],
        };
        let ki2 = jkf.to_ki2_owned();
        assert!(ki2.contains("後手番\n"), "{ki2}");
        let back = parse_ki2_str(&ki2).expect("should parse");
        let data = back.initial.and_then(|i| i.data).expect("initial data");
        assert_eq!(data.color, Color::White);
    }

    #[test]
    fn csa_to_jkf() -> Result<()> {
        let dir = Path::new("data/tests/csa");
        for entry in dir.read_dir()? {
            // Parse and convert CSA to JKF
            let mut path = entry?.path();
            if path.extension() != Some(OsStr::new("csa")) {
                continue;
            }
            let jkf = match parse_csa_file(&path) {
                Ok(jkf) => jkf,
                Err(err) => panic!("failed to parse csa {}: {err}", path.display()),
            };
            // Load exptected JSON
            assert!(path.set_extension("json"));
            let file = File::open(&path)?;
            let mut expected = serde_json::from_reader::<_, JsonKifuFormat>(BufReader::new(file))
                .expect("failed to parse json");
            // Remove all move comments (they cannot be restored from csa...)
            expected.moves.iter_mut().for_each(|m| m.comments = None);

            assert_eq!(expected, jkf, "different from expected: {}", path.display());
        }
        Ok(())
    }

    #[test]
    fn kif_to_jkf() -> Result<()> {
        let dir = Path::new("data/tests/kif");
        for entry in dir.read_dir()? {
            // Parse and convert KIF to JKF, and serialize to Value
            let mut path = entry?.path();
            if path.extension() != Some(OsStr::new("kif")) {
                continue;
            }
            let jkf = match parse_kif_file(&path) {
                Ok(jkf) => jkf,
                Err(err) => {
                    panic!("failed to parse kif file {}: {err}", path.display());
                }
            };
            let val = serde_json::to_value(&jkf).expect("failed to serialize jkf");
            // Load exptected JSON Value
            assert!(path.set_extension("json"));
            let file = File::open(&path)?;
            let expected = serde_json::from_reader::<_, Value>(BufReader::new(file))
                .expect("failed to parse json");

            assert_eq!(expected, val, "different from expected: {}", path.display());
        }
        Ok(())
    }

    #[test]
    fn ki2_to_jkf() -> Result<()> {
        let dir = Path::new("data/tests/ki2");
        for entry in dir.read_dir()? {
            // Parse and convert KI2 to JKF, and serialize to Value
            let mut path = entry?.path();
            if path.extension() != Some(OsStr::new("ki2")) {
                continue;
            }
            let jkf = match parse_ki2_file(&path) {
                Ok(jkf) => jkf,
                Err(err) => {
                    panic!("failed to parse ki2 file {}: {err}", path.display());
                }
            };
            let val = serde_json::to_value(&jkf).expect("failed to serialize jkf");
            // Load exptected JSON Value
            assert!(path.set_extension("json"));
            let file = File::open(&path)?;
            let expected = serde_json::from_reader::<_, Value>(BufReader::new(file))
                .expect("failed to parse json");

            assert_eq!(expected, val, "different from expected: {}", path.display());
        }
        Ok(())
    }
}
