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
