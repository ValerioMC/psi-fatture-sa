use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    PersonaFisica,
    Azienda,
}

impl ClientType {
    pub fn as_str(&self) -> &str {
        match self {
            ClientType::PersonaFisica => "persona_fisica",
            ClientType::Azienda => "azienda",
        }
    }
}

impl From<String> for ClientType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "azienda" => ClientType::Azienda,
            _ => ClientType::PersonaFisica,
        }
    }
}
