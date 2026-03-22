use serde::{Deserialize, Serialize};

use super::tax_regime::TaxRegime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalConfig {
    pub id: i64,
    pub title: String,
    pub first_name: String,
    pub last_name: String,
    pub vat_number: String,
    pub fiscal_code: String,
    pub tax_regime: TaxRegime,
    pub albo_number: String,
    pub albo_region: String,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub country: String,
    pub phone: String,
    pub pec_email: String,
    pub iban: String,
    pub coefficient: f64,
    pub is_psicoanalista: bool,
    pub initial_invoice_number: i64,
    pub created_at: String,
    pub updated_at: String,
}
