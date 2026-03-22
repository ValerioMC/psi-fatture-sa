use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceInput {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
}
