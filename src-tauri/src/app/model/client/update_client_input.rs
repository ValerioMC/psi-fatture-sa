use serde::{Deserialize, Serialize};

use super::client_type::ClientType;

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
