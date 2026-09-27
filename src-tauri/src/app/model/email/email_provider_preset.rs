use serde::Serialize;

use super::{EmailProvider, EmailSecurity};

/// A provider's SMTP settings as the app knows them. `certified` marks a PEC
/// mailbox, whose messages reach an ordinary inbox inside a certified envelope.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailProviderPreset {
    pub provider: EmailProvider,
    pub label: &'static str,
    pub host: &'static str,
    pub port: u16,
    pub security: EmailSecurity,
    pub domains: &'static [&'static str],
    pub certified: bool,
    pub note: &'static str,
}
