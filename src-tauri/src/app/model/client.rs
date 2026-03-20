use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: i64,
    pub client_type: ClientType,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientInput {
    pub client_type: ClientType,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientInput {
    pub id: i64,
    pub client_type: ClientType,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    PersonaFisica,
    Azienda,
}

impl From<String> for ClientType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "azienda" => ClientType::Azienda,
            _ => ClientType::PersonaFisica,
        }
    }
}

impl ClientType {
    pub fn as_str(&self) -> &str {
        match self {
            ClientType::PersonaFisica => "persona_fisica",
            ClientType::Azienda => "azienda",
        }
    }
}
