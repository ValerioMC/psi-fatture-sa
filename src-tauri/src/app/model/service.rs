use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceInput {
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceInput {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
}
