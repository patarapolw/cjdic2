use std::{
    backtrace::Backtrace,
    string::FromUtf8Error,
    sync::{MutexGuard, PoisonError},
};

use rusqlite::Connection;
use serde::Serialize;
use thiserror::Error;
use vibrato_rkyv::{Tokenizer, errors::VibratoError};
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

    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(String),

    #[error("ConnectionMutexGuardError: {0}")]
    ConnectionMutexGuardError(String),

    #[error("TokenizerMutexGuardError: {0}")]
    TokenizerMutexGuardError(String),

    #[error("Not found")]
    NotFound,

    #[error("Error: {0}")]
    Error(String),
}

impl From<rusqlite::Error> for CJDicError {
    fn from(e: rusqlite::Error) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::Database(e.to_string())
    }
}

impl From<serde_json::Error> for CJDicError {
    fn from(e: serde_json::Error) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::Serialization(e.to_string())
    }
}

impl From<anyhow::Error> for CJDicError {
    fn from(e: anyhow::Error) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::AnyhowError(e.to_string())
    }
}

impl From<std::io::Error> for CJDicError {
    fn from(e: std::io::Error) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::IOError(e.to_string())
    }
}

impl From<ZipError> for CJDicError {
    fn from(e: ZipError) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::ZipError(e.to_string())
    }
}

impl From<VibratoError> for CJDicError {
    fn from(e: VibratoError) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::VibratoError(e.to_string())
    }
}

impl From<FromUtf8Error> for CJDicError {
    fn from(e: FromUtf8Error) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::FromUtf8Error(e.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, Connection>>> for CJDicError {
    fn from(e: PoisonError<MutexGuard<'_, Connection>>) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::ConnectionMutexGuardError(e.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, Tokenizer>>> for CJDicError {
    fn from(e: PoisonError<MutexGuard<'_, Tokenizer>>) -> Self {
        let bt = Backtrace::capture();
        eprintln!("{e:#}\n{bt}");
        CJDicError::TokenizerMutexGuardError(e.to_string())
    }
}
