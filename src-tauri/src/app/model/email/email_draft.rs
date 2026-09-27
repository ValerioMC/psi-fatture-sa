use serde::Serialize;

/// The email proposed for one invoice, for the professional to read and edit before
/// sending. `recipient_on_file` is false when the patient has no address yet.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailDraft {
    pub invoice_id: i64,
    pub recipient: String,
    pub recipient_on_file: bool,
    pub subject: String,
    pub body: String,
    pub attachment_name: String,
}
