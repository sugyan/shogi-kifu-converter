# shogi-kifu-converter

[![Crates.io](https://img.shields.io/crates/v/shogi-kifu-converter)](https://crates.io/crates/shogi-kifu-converter)
[![docs.rs](https://img.shields.io/docsrs/shogi-kifu-converter)](https://docs.rs/shogi-kifu-converter)
[![Crates.io](https://img.shields.io/crates/l/shogi-kifu-converter)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/sugyan/shogi-kifu-converter/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/sugyan/shogi-kifu-converter/actions/workflows/rust.yml)

A Rust library that defines structs compatible with [json-kifu-format](https://github.com/na2hiro/json-kifu-format), containing parsers and converters for Shogi kifu (game record) for converting to and from json-kifu-format. And, it also provides conversion from `JsonKifuFormat` type to [`shogi_core`](https://crates.io/crates/shogi_core)'s `Position` type.

![mermaid-diagram-2022-07-31-125435](https://user-images.githubusercontent.com/80381/182141863-62c048d0-460b-4dea-ba46-6935305bc71a.png)

## About json-kifu-format (JKF)

See [github.com/na2hiro/json-kifu-format](https://github.com/na2hiro/json-kifu-format).

## Examples

```
cargo run --example csa2jkf <CSA file>
cargo run --example csa2kif <CSA file>
cargo run --example kif2jkf <KIF file>
cargo run --example kif2csa <KIF file>
cargo run --example jkf2usi <JKF file>
```
