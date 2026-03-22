use serde::{Deserialize, Serialize};

use super::invoice_line::InvoiceLine;
use super::invoice_status::InvoiceStatus;
use super::payment_method::PaymentMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub invoice_number: String,
    pub year: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub contributo_enpap: f64,
    pub ritenuta_acconto: f64,
    pub marca_da_bollo: bool,
    pub total_net: f64,
    pub total_tax: f64,
    pub total_gross: f64,
    pub total_due: f64,
    pub paid_date: Option<String>,
    pub lines: Vec<InvoiceLine>,
    pub created_at: String,
    pub updated_at: String,
}
