//! Parsers for [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)

mod kakinoki;
mod ki2;
mod kif;

use crate::error::ParseError;
use crate::jkf::JsonKifuFormat;
use encoding_rs::{SHIFT_JIS, UTF_8};
use nom::error::convert_error;
use nom::Finish;
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
    match kif::parse(s).finish() {
        Ok((_, mut jkf)) => {
            if let Err(err) = jkf.normalize() {
                Err(ParseError::Normalize(err.to_string()))
            } else {
                Ok(jkf)
            }
        }
        Err(err) => Err(ParseError::Kif(convert_error(s, err))),
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
    match ki2::parse(s).finish() {
        Ok((_, mut jkf)) => {
            if let Err(err) = jkf.normalize() {
                Err(ParseError::Normalize(err.to_string()))
            } else {
                Ok(jkf)
            }
        }
        Err(err) => Err(ParseError::Ki2(convert_error(s, err))),
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
            assert!(jkf.to_csa_owned().contains(pi), "{name}");
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
