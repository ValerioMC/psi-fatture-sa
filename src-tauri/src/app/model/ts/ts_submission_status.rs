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

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "non_inviata" => Ok(TsSubmissionStatus::NonInviata),
            "inviata" => Ok(TsSubmissionStatus::Inviata),
            "accettata" => Ok(TsSubmissionStatus::Accettata),
            "scartata" => Ok(TsSubmissionStatus::Scartata),
            "annullata" => Ok(TsSubmissionStatus::Annullata),
            "sostituita" => Ok(TsSubmissionStatus::Sostituita),
            other => Err(format!("Stato di trasmissione STS sconosciuto: {other}")),
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
mod tests {
    use super::TsSubmissionStatus::*;
    use super::*;

    const ALL: [TsSubmissionStatus; 6] = [
        NonInviata, Inviata, Accettata, Scartata, Annullata, Sostituita,
    ];

    #[test]
    fn round_trips_through_storage_string() {
        for status in ALL {
            assert_eq!(TsSubmissionStatus::parse(status.as_str()), Ok(status));
        }
        assert!(TsSubmissionStatus::parse("inviato").is_err());
    }

    #[test]
    fn allows_only_the_documented_transitions() {
        let allowed = [
            (NonInviata, Inviata),
            (NonInviata, Scartata),
            (Inviata, NonInviata),
            (Inviata, Accettata),
            (Inviata, Scartata),
            (Accettata, Annullata),
            (Accettata, Sostituita),
        ];
        for from in ALL {
            for to in ALL {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed.contains(&(from, to)),
                    "{from:?} -> {to:?}"
                );
            }
        }
    }

    #[test]
    fn terminal_states_have_no_exit() {
        for terminal in [Scartata, Annullata, Sostituita] {
            assert!(ALL.iter().all(|next| !terminal.can_transition_to(*next)));
        }
    }

    #[test]
    fn serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(NonInviata).unwrap(),
            serde_json::json!("non_inviata")
        );
    }
}
