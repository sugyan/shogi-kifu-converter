//! Error definitions

use thiserror::Error;

/// An error that can occur while converting from/into [`shogi_core::Position`]
#[derive(Error, Debug, PartialEq)]
pub enum ConvertError {
    /// Board data is required if `preset` is [`PresetOther`](crate::jkf::Preset::PresetOther)
    #[error("Invalid initial board: no data with preset `OTHER`")]
    InitialBoardNoDataWithPresetOTHER,
    /// [`shogi_core::Square::new()`] was failed for the `(file, rank)`
    #[error("Invalid (file, rank) for `Square`: {0:?}")]
    InvalidSquare((u8, u8)),
    /// Invalid [`shogi_core::PieceKind`] for [`shogi_core::Hand`]
    #[error("Invalid piece kind for `Hand`: {0:?}")]
    InvalidHandPiece(shogi_core::PieceKind),
    /// An error that occurred while normalizing [`JsonKifuFormat`](crate::jkf::JsonKifuFormat)
    #[error("Failed to normalize: {0}")]
    Normalize(String),
}

/// An error that can occur while normalizing [`JsonKifuFormat`](crate::jkf::JsonKifuFormat)
#[derive(Error, Debug, PartialEq)]
pub enum NormalizeError {
    /// Couldn't disambiguous the [`jkf::MoveMoveFormat.from`](crate::jkf::MoveMoveFormat::from)
    #[error("Move `from` is ambiguous: {0:?}")]
    AmbiguousMoveFrom(Vec<shogi_core::Square>),
    /// [`shogi_core::PartialPosition::last_move()`] is required if [`jkf::MoveMoveFormat::same`](crate::jkf::MoveMoveFormat::same) is not `None`
    #[error("No previous move")]
    NoLastMove,
    /// There are no pieces at the [`shogi_core::Square`]
    #[error("No pieces at {0:?}")]
    NoPieceAt(shogi_core::Square),
    /// [`shogi_core::PartialPosition::make_move()`] was failed for the [`shogi_core::Move`]
    #[error("Invalid move: {0:?}")]
    MakeMoveFailed(shogi_core::Move),
    /// An error that occurred while converting from/into [`shogi_core::Position`]
    #[error("Failed to convert: {0}")]
    Convert(String),
}

/// An error that can occur while parsing kifu data
#[derive(Error, Debug)]
pub enum ParseError {
    /// From [`std::io::Error`]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// From [`csa::CsaError`]
    #[error(transparent)]
    Csa(#[from] csa::CsaError),
    /// From [`serde_json::Error`]
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    /// An error that occurred while converting from [`csa::GameRecord`]
    #[error("Failed to convert from `csa::GameRecord`: {0}")]
    CsaConvert(&'static str),
    /// An error that occurred while parsing a KIF string
    #[error("KIF Error: {0}")]
    Kif(String),
    /// An error that occurred while parsing a KI2 string
    #[error("KI2 Error: {0}")]
    Ki2(String),
    /// Decoding the string had failed
    #[error("Decode Error")]
    Decode,
    /// The file extension was unexpected
    #[error("File extension Error")]
    FileExtension,
    /// An error that occurred while normalizing [`JsonKifuFormat`](crate::jkf::JsonKifuFormat)
    #[error("Faield to normalize: {0}")]
    Normalize(String),
}

/// An error that can occur while converting from/into [`pkf`](crate::pkf)
#[derive(Error, Debug, PartialEq)]
pub enum PkfConvertError {
    /// From [`std::num::TryFromIntError`]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),
    /// Got unknown value from `enum_value`
    #[error("Unknown value for {name}: {value}")]
    UnknownEnumValue { name: &'static str, value: i32 },
    /// Got default value for required field
    #[error("Missing field value: {name}")]
    MissingField { name: &'static str },
    /// Color value must not be default
    #[error("No color value")]
    ColorRequired,
    /// PieceKind value must not be default
    #[error("No piece kind value")]
    PieceKindRequired,
    /// Neither `move_` nor `special` is specified
    #[error("Empty move info")]
    EmptyMove,
    /// Got unsupported preset value
    #[error("Preset {0:?} is not supported")]
    UnsupportedPreset(crate::jkf::Preset),
}
