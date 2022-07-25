use crate::error::ConvertError;
use crate::jkf::JsonKifFormat;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn parse_csa<P: AsRef<Path>>(path: P) -> Result<JsonKifFormat, ConvertError> {
    let mut file = File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(csa::parse_csa(&buf)?.into())
}
