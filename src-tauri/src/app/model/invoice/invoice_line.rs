use serde::{Deserialize, Serialize};

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
