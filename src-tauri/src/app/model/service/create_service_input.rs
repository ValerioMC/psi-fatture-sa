use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceInput {
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
}
