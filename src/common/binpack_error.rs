use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BinpackError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid magic bytes")]
    InvalidMagic,
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, BinpackError>;
