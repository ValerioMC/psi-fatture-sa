use serde::Serialize;

/// What the UI may know about the stored Sistema TS secrets: never the secrets themselves.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TsCredentialsStatus {
    pub password_configured: bool,
    pub pincode_configured: bool,
}

/// Result of a harmless authenticated call used to check the credentials.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TsConnectionCheck {
    pub ok: bool,
    pub message: String,
}
