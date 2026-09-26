use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("Archivio credenziali non disponibile: {0}")]
    Unavailable(String),
    #[error("Credenziale salvata non leggibile su questo computer: inseriscila di nuovo nelle Impostazioni")]
    Unreadable,
}
