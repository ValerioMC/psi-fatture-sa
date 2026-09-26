use serde::{Deserialize, Serialize};

use super::ts_submission_status::TsSubmissionStatus;

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
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_storage_string() {
        for operation in [
            TsOperation::Invio,
            TsOperation::Sostituzione,
            TsOperation::Annullamento,
        ] {
            assert_eq!(TsOperation::parse(operation.as_str()), Ok(operation));
        }
        assert!(TsOperation::parse("variazione").is_err());
    }

    #[test]
    fn acceptance_retires_the_target_only_for_follow_up_operations() {
        assert_eq!(TsOperation::Invio.target_status_on_acceptance(), None);
        assert_eq!(
            TsOperation::Sostituzione.target_status_on_acceptance(),
            Some(TsSubmissionStatus::Sostituita)
        );
        assert_eq!(
            TsOperation::Annullamento.target_status_on_acceptance(),
            Some(TsSubmissionStatus::Annullata)
        );
    }
}
