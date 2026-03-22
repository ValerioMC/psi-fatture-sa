use serde::{Deserialize, Serialize};

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
