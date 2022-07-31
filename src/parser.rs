//! Parsers for [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)

use crate::error::{ConvertError, ParserError};
use crate::jkf::JsonKifuFormat;
use crate::kif;
use encoding_rs::{SHIFT_JIS, UTF_8};
use nom::error::convert_error;
use nom::Finish;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Parses a CSA file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
pub fn parse_csa_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ConvertError> {
    let mut file = File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    parse_csa_str(&buf)
}

/// Parses a CSA formatted string to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
pub fn parse_csa_str(s: &str) -> Result<JsonKifuFormat, ConvertError> {
    let mut jkf = JsonKifuFormat::try_from(csa::parse_csa(s)?)?;
    jkf.normalize()?;
    Ok(jkf)
}

/// Parses a KIF file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
///
/// If the file extension is `.kif`, it is decoded as Shift-JIS, and if it is `.kifu`, it is decoded as UTF-8 and parsed.
///
/// See: [http://kakinoki.o.oo7.jp/kif_format.html](http://kakinoki.o.oo7.jp/kif_format.html)
pub fn parse_kif_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ConvertError> {
    let mut file = File::open(&path)?;
    let ext = path
        .as_ref()
        .extension()
        .ok_or(ParserError::FileExtensionError)?;
    let encoding = match ext.to_str() {
        Some("kif") => SHIFT_JIS,
        Some("kifu") => UTF_8,
        _ => return Err(ConvertError::Parser(ParserError::FileExtensionError)),
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let (cow, _, had_errors) = encoding.decode(&buf);
    if had_errors {
        return Err(ConvertError::Parser(ParserError::DecodeError));
    }
    parse_kif_str(&cow)
}

/// Parses a KIF formatted string to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
pub fn parse_kif_str(s: &str) -> Result<JsonKifuFormat, ConvertError> {
    match kif::parse(s).finish() {
        Ok((_, mut jkf)) => {
            jkf.normalize()?;
            Ok(jkf)
        }
        Err(err) => Err(ConvertError::KifError(convert_error(s, err))),
    }
}

/// Parses a JSON file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
pub fn parse_jkf_file<P: AsRef<Path>>(path: P) -> Result<JsonKifuFormat, ConvertError> {
    let file = File::open(&path)?;
    match serde_json::from_reader::<_, JsonKifuFormat>(BufReader::new(file)) {
        Ok(jkf) => Ok(jkf),
        Err(err) => Err(ConvertError::SerdeError(err)),
    }
}

/// Parses a JSON file to [`jkf::JsonKifuFormat`](crate::jkf::JsonKifuFormat)
pub fn parse_jkf_str(s: &str) -> Result<JsonKifuFormat, ConvertError> {
    match serde_json::from_str::<JsonKifuFormat>(s) {
        Ok(jkf) => Ok(jkf),
        Err(err) => Err(ConvertError::SerdeError(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::ffi::OsStr;
    use std::io::Result;

    #[test]
    fn csa_to_jkf() -> Result<()> {
        let dir = Path::new("data/tests/csa");
        for entry in dir.read_dir()? {
            // Parse and convert CSA to JKF
            let mut path = entry?.path();
            if path.extension() != Some(OsStr::new("csa")) {
                continue;
            }
            let jkf = parse_csa_file(&path).expect("failed to parse csa");

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
            let jkf = parse_kif_file(&path).expect("failed to parse kif");
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
