use crate::error::ConvertError;
use crate::jkf::JsonKifFormat;
use crate::kif;
use encoding_rs::{SHIFT_JIS, UTF_8};
use nom::error::convert_error;
use nom::Finish;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn parse_csa_file<P: AsRef<Path>>(path: P) -> Result<JsonKifFormat, ConvertError> {
    let mut file = File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    parse_csa_str(&buf)
}

pub fn parse_csa_str(s: &str) -> Result<JsonKifFormat, ConvertError> {
    csa::parse_csa(s)?.try_into()
}

pub fn parse_kif_file<P: AsRef<Path>>(path: P) -> Result<JsonKifFormat, ConvertError> {
    let mut file = File::open(&path)?;
    let ext = path
        .as_ref()
        .extension()
        .ok_or(ConvertError::FileExtensionError)?;
    let encoding = match ext.to_str() {
        Some("kif") => SHIFT_JIS,
        Some("kifu") => UTF_8,
        _ => return Err(ConvertError::FileExtensionError),
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let (cow, _, had_errors) = encoding.decode(&buf);
    if had_errors {
        return Err(ConvertError::DecodeError);
    }
    parse_kif_str(&cow)
}

pub fn parse_kif_str(s: &str) -> Result<JsonKifFormat, ConvertError> {
    match kif::parse(s).finish() {
        Ok((_, jkf)) => Ok(jkf),
        Err(err) => Err(ConvertError::KifError(convert_error(s, err))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::ffi::OsStr;
    use std::io::{BufReader, Result};

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
            let mut expected = serde_json::from_reader::<_, JsonKifFormat>(BufReader::new(file))
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
