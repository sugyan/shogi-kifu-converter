# shogi-kifu-converter

[![Crates.io](https://img.shields.io/crates/v/shogi-kifu-converter)](https://crates.io/crates/shogi-kifu-converter)
[![docs.rs](https://img.shields.io/docsrs/shogi-kifu-converter)](https://docs.rs/shogi-kifu-converter)
[![Crates.io](https://img.shields.io/crates/l/shogi-kifu-converter)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/sugyan/shogi-kifu-converter/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/sugyan/shogi-kifu-converter/actions/workflows/rust.yml)

A Rust library that defines structs compatible with [json-kifu-format](https://github.com/na2hiro/json-kifu-format), containing parsers and converters for Shogi kifu (game record) for converting to and from json-kifu-format. And, it also provides conversion from `JsonKifuFormat` type to [`shogi_core`](https://crates.io/crates/shogi_core)'s `Position` type.

![mermaid-diagram-2022-07-31-125435](https://user-images.githubusercontent.com/80381/182141863-62c048d0-460b-4dea-ba46-6935305bc71a.png)

## About json-kifu-format (JKF)

See [github.com/na2hiro/json-kifu-format](https://github.com/na2hiro/json-kifu-format).

## Supporting formats and types

### Parsers

- CSA format
- KIF format

### Converters

- [`ToUsi`](https://docs.rs/shogi-kifu-converter/latest/shogi_kifu_converter/jkf/struct.JsonKifuFormat.html#impl-ToUsi-for-JsonKifuFormat)
- [`ToCsi`](https://docs.rs/shogi-kifu-converter/latest/shogi_kifu_converter/jkf/struct.JsonKifuFormat.html#impl-ToCsa-for-JsonKifuFormat)
- [`ToKif`](https://docs.rs/shogi-kifu-converter/latest/shogi_kifu_converter/jkf/struct.JsonKifuFormat.html#impl-ToKif-for-JsonKifuFormat)
- [`TryFrom<&jkf::JsonKifuFormat> for shogi_core::Position`](https://docs.rs/shogi-kifu-converter/latest/shogi_kifu_converter/jkf/struct.JsonKifuFormat.html#impl-TryFrom%3C%26JsonKifuFormat%3E-for-Position)
