//! # shogi-kifu-converter
//!
//! A Rust library that defines structs compatible with [json-kifu-format](https://github.com/na2hiro/json-kifu-format), containing parsers and converters for Shogi kifu (game record) for converting to and from json-kifu-format.
//! And, it also provides conversion from `JsonKifuFormat` type to [`shogi_core`](https://crates.io/crates/shogi_core)'s `Position` type.
//!
//! ## About json-kifu-format (JKF)
//!
//! See [https://github.com/na2hiro/json-kifu-format](https://github.com/na2hiro/json-kifu-format).

pub mod converter;
mod csa;
pub mod error;
pub mod jkf;
mod normalizer;
pub mod parser;
mod shogi_core;
include!(concat!(env!("OUT_DIR"), "/pkf/mod.rs"));

#[cfg(test)]
mod tests {
    use crate::jkf::JsonKifuFormat;
    use crate::pkf::Kifu;
    use std::ffi::OsStr;
    use std::fs::{DirEntry, File};
    use std::io::{BufReader, Result};
    use std::path::Path;

    fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry) -> Result<()>) -> Result<()> {
        if dir.is_dir() {
            for entry in dir.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry)?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn jkf_pkf_jkf() -> Result<()> {
        visit_dirs(Path::new("data/tests"), &|entry: &DirEntry| -> Result<()> {
            let path = entry.path();
            if path.extension() != Some(OsStr::new("json")) {
                return Ok(());
            }

            let file = File::open(&path)?;
            let orig = serde_json::from_reader::<_, JsonKifuFormat>(BufReader::new(file))
                .expect("failed to parse json");
            let pkf = match Kifu::try_from(&orig) {
                Ok(pos) => pos,
                Err(err) => panic!("failed to convert jkf to pkf {}: {err}", path.display()),
            };
            let curr = match JsonKifuFormat::try_from(&pkf) {
                Ok(jkf) => jkf,
                Err(err) => panic!(
                    "failed to convert position to jkf {}: {err}",
                    path.display()
                ),
            };
            assert_eq!(orig, curr, "difference in {}", path.display());
            Ok(())
        })
    }
}
