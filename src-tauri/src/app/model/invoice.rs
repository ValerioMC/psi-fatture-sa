use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: Option<i64>,
    pub invoice_id: Option<i64>,
    pub service_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
    pub line_total: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInvoiceInput {
    pub id: i64,
    pub client_id: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub paid_date: Option<String>,
    pub lines: Vec<InvoiceLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineInput {
    pub service_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    Overdue,
    Cancelled,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            InvoiceStatus::Draft => "draft",
            InvoiceStatus::Issued => "issued",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Overdue => "overdue",
            InvoiceStatus::Cancelled => "cancelled",
        }
    }
}

impl From<String> for InvoiceStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "issued" => InvoiceStatus::Issued,
            "paid" => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            "cancelled" => InvoiceStatus::Cancelled,
            _ => InvoiceStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Bonifico,
    Contanti,
    Pos,
    Altro,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentMethod::Bonifico => "bonifico",
            PaymentMethod::Contanti => "contanti",
            PaymentMethod::Pos => "pos",
            PaymentMethod::Altro => "altro",
        }
    }
}

impl From<String> for PaymentMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "contanti" => PaymentMethod::Contanti,
            "pos" => PaymentMethod::Pos,
            "altro" => PaymentMethod::Altro,
            _ => PaymentMethod::Bonifico,
        }
    }
}

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

/// Input for generating monthly invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateMonthlyInput {
    pub year: i64,
    pub month: i64,
    pub client_ids: Vec<i64>,
    pub payment_method: PaymentMethod,
    pub apply_enpap: bool,
}

/// Input for bulk-updating the status of multiple invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateStatusInput {
    pub ids: Vec<i64>,
    pub status: InvoiceStatus,
    pub paid_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceFilters {
    pub year: Option<i64>,
    pub status: Option<String>,
    pub client_id: Option<i64>,
    pub search: Option<String>,
}
