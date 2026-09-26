use serde::Serialize;

/// Result of a harmless authenticated call used to check the credentials.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TsConnectionCheck {
    pub ok: bool,
    pub message: String,
}
