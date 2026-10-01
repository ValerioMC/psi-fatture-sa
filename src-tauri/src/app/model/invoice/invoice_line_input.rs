use serde::{Deserialize, Serialize};

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
