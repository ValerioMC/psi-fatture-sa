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
