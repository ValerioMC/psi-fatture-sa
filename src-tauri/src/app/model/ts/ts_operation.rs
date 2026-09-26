use serde::{Deserialize, Serialize};

use super::TsSubmissionStatus;

/// The three Sistema TS operations. Cancellation and replacement always
/// target a previously accepted `Invio` or `Sostituzione`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TsOperation {
    Invio,
    Sostituzione,
    Annullamento,
}

impl TsOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            TsOperation::Invio => "invio",
            TsOperation::Sostituzione => "sostituzione",
            TsOperation::Annullamento => "annullamento",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "invio" => Ok(TsOperation::Invio),
            "sostituzione" => Ok(TsOperation::Sostituzione),
            "annullamento" => Ok(TsOperation::Annullamento),
            other => Err(format!("Operazione STS sconosciuta: {other}")),
        }
    }

    /// The status the target submission takes once this operation is accepted.
    pub fn target_status_on_acceptance(&self) -> Option<TsSubmissionStatus> {
        match self {
            TsOperation::Invio => None,
            TsOperation::Sostituzione => Some(TsSubmissionStatus::Sostituita),
            TsOperation::Annullamento => Some(TsSubmissionStatus::Annullata),
        }
    }
}

#[cfg(test)]
#[path = "ts_operation_test.rs"]
mod tests;
