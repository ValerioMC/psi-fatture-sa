use serde::{Deserialize, Serialize};

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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertConfigInput {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaxRegime {
    Forfettario,
    Ordinario,
}

impl TaxRegime {
    pub fn as_str(&self) -> &str {
        match self {
            TaxRegime::Forfettario => "forfettario",
            TaxRegime::Ordinario => "ordinario",
        }
    }
}

impl From<String> for TaxRegime {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ordinario" => TaxRegime::Ordinario,
            _ => TaxRegime::Forfettario,
        }
    }
}
