use thiserror::Error;

use crate::app::common::AppError;

/// A call that produced no Sistema TS verdict. `Authentication` is the
/// HTTP-level refusal of the username/password pair.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TsGatewayError {
    #[error("Sistema TS non raggiungibile: {0}")]
    Network(String),
    #[error("Utente o password del Sistema TS non validi")]
    Authentication,
    #[error("Il Sistema TS ha rifiutato la richiesta: {0}")]
    Fault(String),
    #[error("Risposta inattesa dal Sistema TS (HTTP {0})")]
    Http(u16),
    #[error("Risposta del Sistema TS non leggibile: {0}")]
    Malformed(String),
    #[error("Preparazione della richiesta non riuscita: {0}")]
    Request(String),
    #[error("Ambiente Sistema TS non disponibile in questa versione dell'app")]
    UnavailableEnvironment,
}

impl From<TsGatewayError> for AppError {
    fn from(error: TsGatewayError) -> Self {
        AppError::External(error.to_string())
    }
}
