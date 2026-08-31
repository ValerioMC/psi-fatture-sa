use serde::{Deserialize, Serialize};

/// Base profession shown on invoices. Psicoanalista is not a variant here
/// because it is an additional qualification tracked by `is_psicoanalista`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Profession {
    Psicologo,
    Psicoterapeuta,
}

impl Profession {
    pub fn as_str(&self) -> &str {
        match self {
            Profession::Psicologo => "psicologo",
            Profession::Psicoterapeuta => "psicoterapeuta",
        }
    }
}

impl From<String> for Profession {
    fn from(s: String) -> Self {
        match s.as_str() {
            "psicoterapeuta" => Profession::Psicoterapeuta,
            _ => Profession::Psicologo,
        }
    }
}
