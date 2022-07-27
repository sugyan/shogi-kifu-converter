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
