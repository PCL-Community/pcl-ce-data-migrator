use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Can not found env data")]
    EnvNotFound,
    #[error("Process data fail")]
    DataError(#[from] crate::bak_data::DataError),
    #[error("IO Error: {0}")]
    IOError(#[from] tokio::io::Error),
}
