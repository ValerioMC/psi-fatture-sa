use crate::app::common::AppError;
use serde::{Deserialize, Serialize};

/// Lifecycle of one Sistema TS transmission.
///
/// `NonInviata` waits in the queue, `Inviata` is a call in flight (back to
/// `NonInviata` when it fails before a verdict), `Accettata`/`Scartata` are the
/// verdict. An accepted submission becomes `Annullata` or `Sostituita` when a
/// cancellation or replacement targeting it is accepted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TsSubmissionStatus {
    NonInviata,
    Inviata,
    Accettata,
    Scartata,
    Annullata,
    Sostituita,
}

impl TsSubmissionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TsSubmissionStatus::NonInviata => "non_inviata",
            TsSubmissionStatus::Inviata => "inviata",
            TsSubmissionStatus::Accettata => "accettata",
            TsSubmissionStatus::Scartata => "scartata",
            TsSubmissionStatus::Annullata => "annullata",
            TsSubmissionStatus::Sostituita => "sostituita",
        }
    }

    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "non_inviata" => Ok(TsSubmissionStatus::NonInviata),
            "inviata" => Ok(TsSubmissionStatus::Inviata),
            "accettata" => Ok(TsSubmissionStatus::Accettata),
            "scartata" => Ok(TsSubmissionStatus::Scartata),
            "annullata" => Ok(TsSubmissionStatus::Annullata),
            "sostituita" => Ok(TsSubmissionStatus::Sostituita),
            other => Err(AppError::Invalid(format!(
                "Stato di trasmissione STS sconosciuto: {other}"
            ))),
        }
    }

    pub fn can_transition_to(&self, next: TsSubmissionStatus) -> bool {
        use TsSubmissionStatus::*;
        matches!(
            (self, next),
            (NonInviata, Inviata)
                | (NonInviata, Scartata)
                | (Inviata, NonInviata)
                | (Inviata, Accettata)
                | (Inviata, Scartata)
                | (Accettata, Annullata)
                | (Accettata, Sostituita)
        )
    }
}

#[cfg(test)]
#[path = "ts_submission_status_test.rs"]
mod tests;
