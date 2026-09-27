use serde::{Deserialize, Serialize};

/// How the SMTP connection is protected: `Tls` from the first byte (port 465),
/// `Starttls` upgraded after the greeting (port 587). Plain text is never offered.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmailSecurity {
    Tls,
    Starttls,
}

impl EmailSecurity {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailSecurity::Tls => "tls",
            EmailSecurity::Starttls => "starttls",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "tls" => Ok(EmailSecurity::Tls),
            "starttls" => Ok(EmailSecurity::Starttls),
            other => Err(format!("Sicurezza della connessione sconosciuta: {other}")),
        }
    }
}
