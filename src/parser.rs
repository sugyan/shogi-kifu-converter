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
