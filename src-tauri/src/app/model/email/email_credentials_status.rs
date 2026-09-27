use serde::Serialize;

/// Whether the mailbox password is stored; the password itself never leaves the backend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailCredentialsStatus {
    pub password_configured: bool,
}
