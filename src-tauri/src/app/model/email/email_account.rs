use serde::Serialize;

use super::{EmailProvider, EmailSecurity};

/// The mailbox invoices leave from. Before the first save it is proposed from the
/// profile (the PEC address, the professional's name) and `saved` is false.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailAccount {
    pub provider: EmailProvider,
    pub sender_address: String,
    pub sender_name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub security: EmailSecurity,
    pub username: String,
    pub bcc_self: bool,
    pub saved: bool,
}
