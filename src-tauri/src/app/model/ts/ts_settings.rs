use serde::{Deserialize, Serialize};

use super::TsEnvironment;

/// Non-secret Sistema TS settings. `username` is the codice fiscale the
/// professional logs in with, and doubles as `cfProprietario`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TsSettings {
    pub environment: TsEnvironment,
    pub username: String,
    pub vat_number: String,
}
