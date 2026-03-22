use serde::{Deserialize, Serialize};

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
