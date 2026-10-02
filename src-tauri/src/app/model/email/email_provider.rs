use crate::app::common::AppError;
use serde::{Deserialize, Serialize};

use super::{EmailProviderPreset, EmailSecurity};

/// Where the professional's mailbox lives. The presets carry the SMTP server,
/// so choosing one is all most people have to do; `Custom` asks for the server.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmailProvider {
    Psypec,
    ArubaPec,
    Gmail,
    Custom,
}

impl EmailProvider {
    pub const ALL: [EmailProvider; 4] = [
        EmailProvider::Psypec,
        EmailProvider::ArubaPec,
        EmailProvider::Gmail,
        EmailProvider::Custom,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            EmailProvider::Psypec => "psypec",
            EmailProvider::ArubaPec => "aruba_pec",
            EmailProvider::Gmail => "gmail",
            EmailProvider::Custom => "custom",
        }
    }

    pub fn parse(value: &str) -> Result<Self, AppError> {
        EmailProvider::ALL
            .into_iter()
            .find(|provider| provider.as_str() == value)
            .ok_or_else(|| AppError::Invalid(format!("Provider email sconosciuto: {value}")))
    }

    /// psypec.it is the free PEC the Ordine gives every psychologist, run by Namirial.
    pub fn preset(&self) -> EmailProviderPreset {
        match self {
            EmailProvider::Psypec => EmailProviderPreset {
                provider: *self,
                label: "PEC psypec.it",
                host: "smtps.sicurezzapostale.it",
                port: 465,
                security: EmailSecurity::Tls,
                domains: &["psypec.it"],
                certified: true,
                note: "La casella PEC gratuita dell'Ordine degli Psicologi, gestita da Namirial. Usa l'indirizzo completo e la password della casella.",
            },
            EmailProvider::ArubaPec => EmailProviderPreset {
                provider: *self,
                label: "PEC Aruba",
                host: "smtps.pec.aruba.it",
                port: 465,
                security: EmailSecurity::Tls,
                domains: &["pec.it", "arubapec.it"],
                certified: true,
                note: "Una casella PEC di Aruba. Usa l'indirizzo completo e la password della casella.",
            },
            EmailProvider::Gmail => EmailProviderPreset {
                provider: *self,
                label: "Gmail",
                host: "smtp.gmail.com",
                port: 465,
                security: EmailSecurity::Tls,
                domains: &["gmail.com", "googlemail.com"],
                certified: false,
                note: "Gmail accetta solo una password per le app: si crea da account Google → Sicurezza, con la verifica in due passaggi attiva.",
            },
            EmailProvider::Custom => EmailProviderPreset {
                provider: *self,
                label: "Altro provider",
                host: "",
                port: 465,
                security: EmailSecurity::Tls,
                domains: &[],
                certified: false,
                note: "Server, porta e sicurezza si trovano nella guida del tuo provider, alla voce SMTP o posta in uscita.",
            },
        }
    }

    /// The provider an address belongs to, by its domain; `Custom` when none claims it.
    pub fn detect(address: &str) -> EmailProvider {
        let domain = address
            .rsplit_once('@')
            .map(|(_, domain)| domain.trim().to_lowercase())
            .unwrap_or_default();
        EmailProvider::ALL
            .into_iter()
            .find(|provider| provider.preset().domains.contains(&domain.as_str()))
            .unwrap_or(EmailProvider::Custom)
    }
}
