use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO Error: {0}")]
    IOError(#[from] tokio::io::Error),
    #[error("Failed to parse int: {0}")]
    IntParseError(#[from] ParseIntError),
    #[error("Can not found env data")]
    EnvNotFound,
    #[error("Process json fail: {0}")]
    JsonError(#[from] serde_json::Error),
}
