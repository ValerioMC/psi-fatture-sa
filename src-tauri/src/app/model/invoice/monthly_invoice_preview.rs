use serde::{Deserialize, Serialize};

use super::invoice_line_input::InvoiceLineInput;

/// Preview for monthly batch invoice generation, one per client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyInvoicePreview {
    pub client_id: i64,
    pub client_name: String,
    pub appointment_count: i64,
    pub lines: Vec<InvoiceLineInput>,
    pub estimated_net: f64,
    pub estimated_due: f64,
}
