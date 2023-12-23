use std::{
    num::{ParseIntError, TryFromIntError},
    time::SystemTimeError,
};

use reqwest::header::ToStrError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum TransferError {
    Io(std::io::Error),
    Request(reqwest::Error),
    Generic(String),
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    AsyncMutex(tokio::sync::TryLockError),
}

impl std::error::Error for TransferError {}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::Io(err) => write!(f, "{err}"),
            TransferError::Request(err) => write!(f, "{err}"),
            TransferError::Generic(err) => write!(f, "{err}"),
            TransferError::Database(err) => write!(f, "{err}"),
            TransferError::Serialization(err) => write!(f, "{err}"),
            TransferError::AsyncMutex(err) => write!(f, "{err}"),
        }
    }
}

impl From<tokio::sync::TryLockError> for TransferError {
    fn from(err: tokio::sync::TryLockError) -> Self {
        TransferError::AsyncMutex(err)
    }
}

impl From<serde_json::Error> for TransferError {
    fn from(err: serde_json::Error) -> Self {
        TransferError::Serialization(err)
    }
}

impl From<rusqlite::Error> for TransferError {
    fn from(err: rusqlite::Error) -> Self {
        TransferError::Database(err)
    }
}

impl From<ParseIntError> for TransferError {
    fn from(err: ParseIntError) -> Self {
        TransferError::Generic(err.to_string())
    }
}

impl From<TryFromIntError> for TransferError {
    fn from(err: TryFromIntError) -> Self {
        TransferError::Generic(err.to_string())
    }
}

impl From<&str> for TransferError {
    fn from(err: &str) -> Self {
        TransferError::Generic(err.to_string())
    }
}

impl From<SystemTimeError> for TransferError {
    fn from(err: SystemTimeError) -> Self {
        TransferError::Generic(err.to_string())
    }
}

impl From<JoinError> for TransferError {
    fn from(err: JoinError) -> Self {
        TransferError::Generic(err.to_string())
    }
}

impl From<String> for TransferError {
    fn from(err: String) -> Self {
        TransferError::Generic(err)
    }
}

impl From<ToStrError> for TransferError {
    fn from(err: ToStrError) -> Self {
        TransferError::Generic(err.to_string())
    }
}

impl From<std::io::Error> for TransferError {
    fn from(err: std::io::Error) -> Self {
        TransferError::Io(err)
    }
}

impl From<reqwest::Error> for TransferError {
    fn from(err: reqwest::Error) -> Self {
        TransferError::Request(err)
    }
}
