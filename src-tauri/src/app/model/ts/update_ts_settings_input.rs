use serde::Deserialize;

use super::TsEnvironment;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTsSettingsInput {
    pub environment: TsEnvironment,
    pub username: String,
    pub vat_number: String,
}
