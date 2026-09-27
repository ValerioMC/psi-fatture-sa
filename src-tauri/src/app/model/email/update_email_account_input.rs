use serde::Deserialize;

use super::{EmailProvider, EmailSecurity};

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateEmailAccountInput {
    pub provider: EmailProvider,
    pub sender_address: String,
    pub sender_name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub security: EmailSecurity,
    pub username: String,
    pub bcc_self: bool,
}
