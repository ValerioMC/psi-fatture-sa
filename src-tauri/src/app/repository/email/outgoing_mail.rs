use super::MailAttachment;

/// A plain-text email with at most one attachment, as the gateway sends it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMail {
    pub from_address: String,
    pub from_name: String,
    pub to: String,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub attachment: Option<MailAttachment>,
}
