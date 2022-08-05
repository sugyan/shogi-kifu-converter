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

/// An alias for [`jkf::JsonKifuFormat`]
pub type JKF = jkf::JsonKifuFormat;
