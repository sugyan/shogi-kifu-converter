//! Error definitions

use std::io;
use thiserror::Error;

/// An error that can occur while converting
#[derive(Error, Debug)]
pub enum ConvertError {
    /// From [`std::io::Error`]
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    /// From [`csa::CsaError`]
    #[error("CSA Error: {0}")]
    Csa(#[from] csa::CsaError),
    /// From [`CsaConvertError`]
    #[error("CSA converter Error: {0}")]
    CsaConvert(#[from] CsaConvertError),
    /// From [`NormalizerError`]
    #[error("Normalization Error: {0}")]
    Normalizer(#[from] NormalizerError),
    /// From [`ParserError`]
    #[error("Parser Error: {0}")]
    Parser(#[from] ParserError),
    /// From [`serde_json::Error`]
    #[error("JSON Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    /// An error that occurred while parsing a KIF string
    #[error("KIF Error: {0}")]
    KifError(String),
    /// An error that occurred while parsing a KI2 string
    #[error("KI2 Error: {0}")]
    Ki2Error(String),
}

/// An error that can occur while converting to [`shogi_core`](shogi_core)
#[derive(Error, Debug)]
pub enum CoreConvertError {
    /// Board data is required if preset is [`PresetOther`](crate::jkf::Preset::PresetOther)
    #[error("Invalid initial board: no data with preset `OTHER`")]
    InitialBoardNoDataWithPresetOTHER,
    /// [`shogi_core::Hand::added()`] was failed for the The [`shogi_core::PieceKind`]
    #[error("Invalid initial hands: {0:?}")]
    InitialHands(shogi_core::PieceKind),
    /// [`shogi_core::Square::new()`] was failed for the `(file, rank)`
    #[error("Invalid place: {0:?}")]
    InvalidPlace((u8, u8)),
    /// [`shogi_core::PartialPosition::make_move()`] was failed for the [`shogi_core::Move`]
    #[error("Invalid move: {0:?}")]
    InvalidMove(shogi_core::Move),
}

/// An error that can occur while converting from [`csa`]
#[derive(Error, Debug)]
pub enum CsaConvertError {
    /// [`csa::PieceType::All`] cannot be converted to [`jkf::Kind`](crate::jkf::Kind)
    #[error("AL PieceType")]
    PieceTypeAll,
}

/// An error that can occur while normalizing [`JsonKifuFormat`](crate::jkf::JsonKifuFormat)
#[derive(Error, Debug)]
pub enum NormalizerError {
    /// From [`CoreConvertError`]
    #[error("Convert Error: {0}")]
    CoreConvert(#[from] CoreConvertError),
    /// [`shogi_core::PartialPosition::last_move()`] is required if [`jkf::MoveMoveFormat::same`](crate::jkf::MoveMoveFormat::same) is some
    #[error("Invalid move")]
    NoLastMove,
    /// The [`jkf::MoveMoveFormat`](crate::jkf::MoveMoveFormat) data is invalid for current [`shogi_core::PartialPosition`]
    #[error("Invalid move: {0}")]
    MoveInconsistent(&'static str),
    /// Couldn't disambiguous the [`jkf::MoveMoveFormat.from`](crate::jkf::MoveMoveFormat::from)
    #[error("Move `from` is ambiguous: {0:?}")]
    AmbiguousMoveFrom(Vec<shogi_core::Square>),
}

/// An error that can occur while parsing files
#[derive(Error, Debug)]
pub enum ParserError {
    /// The decoding of the string had failed
    #[error("Decode Error")]
    DecodeError,
    /// The file extension was unexpected
    #[error("File extension Error")]
    FileExtensionError,
}
