use serde::{Deserialize, Serialize};

use crate::app::model::tax::InvoiceLineData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineInput {
    pub service_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
    /// Net amount typed by hand, replacing quantity × unit price.
    #[serde(default)]
    pub amount_override: Option<f64>,
}

impl From<&InvoiceLineInput> for InvoiceLineData {
    fn from(line: &InvoiceLineInput) -> Self {
        InvoiceLineData {
            quantity: line.quantity,
            unit_price: line.unit_price,
            vat_rate: line.vat_rate,
            amount_override: line.amount_override,
        }
    }
}
