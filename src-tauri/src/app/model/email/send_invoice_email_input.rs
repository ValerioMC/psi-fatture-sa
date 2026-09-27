use serde::Deserialize;

/// The email as the professional left it. `remember_recipient` saves the address on
/// the patient's record once the email has gone.
#[derive(Debug, Clone, Deserialize)]
pub struct SendInvoiceEmailInput {
    pub invoice_id: i64,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub remember_recipient: bool,
}
