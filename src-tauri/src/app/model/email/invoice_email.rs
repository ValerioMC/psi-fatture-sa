use serde::Serialize;

use super::InvoiceEmailStatus;

/// One attempt to email an invoice to its patient, kept whether it went or not.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InvoiceEmail {
    pub id: i64,
    pub invoice_id: i64,
    pub recipient: String,
    pub subject: String,
    pub attachment_name: String,
    pub status: InvoiceEmailStatus,
    pub error: Option<String>,
    pub sent_at: String,
}
