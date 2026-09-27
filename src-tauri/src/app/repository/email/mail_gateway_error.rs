use thiserror::Error;

/// Why an email did not leave. The messages are the ones the professional reads.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MailGatewayError {
    #[error("Server di posta non raggiungibile: {0}")]
    Connection(String),
    #[error(
        "Il server di posta non ha accettato indirizzo e password: controllali nelle Impostazioni"
    )]
    Authentication,
    #[error("Il server di posta ha rifiutato il messaggio: {0}")]
    Rejected(String),
    #[error("Il server di posta non risponde: riprova tra qualche minuto")]
    Timeout,
    #[error("Messaggio non valido: {0}")]
    InvalidMessage(String),
}
