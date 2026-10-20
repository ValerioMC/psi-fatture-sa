use thiserror::Error;

use crate::app::common::AppError;

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("Archivio credenziali non disponibile: {0}")]
    Unavailable(String),
    #[error("Credenziale salvata non leggibile su questo computer: inseriscila di nuovo nelle Impostazioni")]
    Unreadable,
}

impl From<SecretStoreError> for AppError {
    fn from(error: SecretStoreError) -> Self {
        AppError::External(error.to_string())
    }
}
