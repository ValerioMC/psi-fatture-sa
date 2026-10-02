use sea_orm::DbErr;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::app::repository::secret::SecretStoreError;
use crate::app::repository::ts::sistema_ts::TsGatewayError;

/// Every way an application command can fail. The kind drives handling in
/// Rust; the frontend receives only the Italian message, serialized as a string.
#[derive(Debug, Error)]
pub enum AppError {
    /// Input the user can correct.
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    NotFound(String),
    /// The request clashes with stored state: a taken number, a transmitted invoice.
    #[error("{0}")]
    Conflict(String),
    #[error("Errore del database: {0}")]
    Database(#[from] DbErr),
    /// A failure outside the app: Sistema TS, mail server, file system, credentials.
    #[error("{0}")]
    External(String),
}

impl From<SecretStoreError> for AppError {
    fn from(error: SecretStoreError) -> Self {
        AppError::External(error.to_string())
    }
}

impl From<TsGatewayError> for AppError {
    fn from(error: TsGatewayError) -> Self {
        AppError::External(error.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        AppError::External(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::External(error.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
#[path = "app_error_test.rs"]
mod tests;
