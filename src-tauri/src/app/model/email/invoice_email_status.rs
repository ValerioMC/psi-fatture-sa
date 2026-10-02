use crate::app::common::AppError;
use serde::{Deserialize, Serialize};

/// The outcome of one attempt to email an invoice. Only `Sent` counts as delivered to
/// the provider; what happens after that is the provider's business.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceEmailStatus {
    Sent,
    Failed,
}

impl InvoiceEmailStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoiceEmailStatus::Sent => "sent",
            InvoiceEmailStatus::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "sent" => Ok(InvoiceEmailStatus::Sent),
            "failed" => Ok(InvoiceEmailStatus::Failed),
            other => Err(AppError::Invalid(format!(
                "Esito di invio sconosciuto: {other}"
            ))),
        }
    }
}
