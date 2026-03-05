use serde::Serialize;
use thiserror::Error;
use vibrato::errors::VibratoError;
use zip::result::ZipError;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum CJDicError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("File name not found")]
    FileNameNotFound,

    #[error("Zip error: {0}")]
    ZipError(String),

    #[error("Vibrato error: {0}")]
    VibratoError(String),

    #[error("Not found")]
    NotFound,

    #[error("Error: {0}")]
    Error(String),
}

impl From<rusqlite::Error> for CJDicError {
    fn from(e: rusqlite::Error) -> Self {
        log::error!("{e:#}");
        CJDicError::Database(e.to_string())
    }
}

impl From<serde_json::Error> for CJDicError {
    fn from(e: serde_json::Error) -> Self {
        log::error!("{e:#}");
        CJDicError::Serialization(e.to_string())
    }
}

impl From<anyhow::Error> for CJDicError {
    fn from(e: anyhow::Error) -> Self {
        log::error!("{e:#}");
        CJDicError::AnyhowError(e.to_string())
    }
}

impl From<std::io::Error> for CJDicError {
    fn from(e: std::io::Error) -> Self {
        log::error!("{e:#}");
        CJDicError::IOError(e.to_string())
    }
}

impl From<ZipError> for CJDicError {
    fn from(e: ZipError) -> Self {
        log::error!("{e:#}");
        CJDicError::ZipError(e.to_string())
    }
}

impl From<VibratoError> for CJDicError {
    fn from(e: VibratoError) -> Self {
        log::error!("{e:#}");
        CJDicError::VibratoError(e.to_string())
    }
}
