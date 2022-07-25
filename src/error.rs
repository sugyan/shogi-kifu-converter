use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("CSA Error")]
    Csa(#[from] csa::CsaError),
}
