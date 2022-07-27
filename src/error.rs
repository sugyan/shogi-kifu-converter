use crate::csa::CsaConvertError;
use crate::normalizer::NormalizerError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("CSA Error: {0}")]
    Csa(#[from] csa::CsaError),
    #[error("CSA converter Error: {0}")]
    CsaConvert(#[from] CsaConvertError),
    #[error("KIF Error: {0}")]
    KifError(String),
    #[error("Normalization Error: {0}")]
    Normalizer(#[from] NormalizerError),
    #[error("Decode Error")]
    DecodeError,
    #[error("File extension Error")]
    FileExtensionError,
}
