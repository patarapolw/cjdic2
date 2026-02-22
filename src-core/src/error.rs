use thiserror::Error;
use zip::result::ZipError;

#[derive(Debug, Error)]
pub enum CJDicError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    ZipError(#[from] ZipError),

    #[error("Not found")]
    NotFound,
}
