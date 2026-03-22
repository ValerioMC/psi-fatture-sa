use serde::{Deserialize, Serialize};

use super::invoice_line_input::InvoiceLineInput;
use super::invoice_status::InvoiceStatus;
use super::payment_method::PaymentMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceInput {
    pub client_id: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub lines: Vec<InvoiceLineInput>,
}
