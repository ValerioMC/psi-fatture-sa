use serde::{Deserialize, Serialize};

use super::client_type::ClientType;
use super::create_client_input::CreateClientInput;

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
    /// None follows the profile's setting.
    #[serde(default)]
    pub hide_quantity_in_invoice: Option<bool>,
}

/// The fields an update shares with a creation; the caller keeps the id.
impl From<UpdateClientInput> for CreateClientInput {
    fn from(input: UpdateClientInput) -> Self {
        CreateClientInput {
            client_type: input.client_type,
            first_name: input.first_name,
            last_name: input.last_name,
            birth_date: input.birth_date,
            gender: input.gender,
            fiscal_code: input.fiscal_code,
            vat_number: input.vat_number,
            address: input.address,
            city: input.city,
            province: input.province,
            zip_code: input.zip_code,
            email: input.email,
            phone: input.phone,
            notes: input.notes,
            sts_authorization: input.sts_authorization,
            hide_quantity_in_invoice: input.hide_quantity_in_invoice,
        }
    }
}
