use serde::{Deserialize, Serialize};

use super::invoice_line_input::InvoiceLineInput;
use super::invoice_status::InvoiceStatus;
use super::payment_method::PaymentMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInvoiceInput {
    pub id: i64,
    pub client_id: i64,
    /// New invoice number for manual renumbering (e.g. to fill the gap left
    /// by a deleted invoice). When None or blank the current number is kept.
    #[serde(default)]
    pub invoice_number: Option<String>,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub paid_date: Option<String>,
    pub lines: Vec<InvoiceLineInput>,
}
