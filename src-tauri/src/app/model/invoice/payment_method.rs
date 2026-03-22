use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Bonifico,
    Contanti,
    Pos,
    Altro,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentMethod::Bonifico => "bonifico",
            PaymentMethod::Contanti => "contanti",
            PaymentMethod::Pos => "pos",
            PaymentMethod::Altro => "altro",
        }
    }
}

impl From<String> for PaymentMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "contanti" => PaymentMethod::Contanti,
            "pos" => PaymentMethod::Pos,
            "altro" => PaymentMethod::Altro,
            _ => PaymentMethod::Bonifico,
        }
    }
}
