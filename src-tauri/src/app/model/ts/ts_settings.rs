use serde::{Deserialize, Serialize};

use super::ts_environment::TsEnvironment;

/// Non-secret Sistema TS settings. `username` is the codice fiscale the
/// professional logs in with, and doubles as `cfProprietario`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TsSettings {
    pub environment: TsEnvironment,
    pub username: String,
    pub vat_number: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTsSettingsInput {
    pub environment: TsEnvironment,
    pub username: String,
    pub vat_number: String,
}
