use crate::csa::CsaConverterError;
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
    CsaConverter(#[from] CsaConverterError),
    #[error("Normalization Error: {0}")]
    Normalizer(#[from] NormalizerError),
}
