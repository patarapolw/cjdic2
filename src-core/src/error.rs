use thiserror::Error;

#[derive(Debug, Error)]
pub enum CJDicError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found")]
    NotFound,
}
