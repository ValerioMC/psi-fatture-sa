use super::*;
use crate::app::service::ts::ts_submission_service as queue;
use crate::test_support::clock::at;
use crate::test_support::ts_database::{force_status, setup};

fn document() -> TsDocumentId {
    TsDocumentId {
        vat_number: "12345678903".to_string(),
        issue_date: "2026-03-01".to_string(),
        number: "1".to_string(),
    }
}

fn verdict(accepted: bool) -> TsOutcome {
    TsOutcome {
        accepted,
        protocol: accepted.then(|| "99260926001866680".to_string()),
        code: (!accepted).then(|| "S017".to_string()),
        message: Some("esito".to_string()),
    }
}

async fn claimed(db: &sea_orm::DatabaseConnection, id: i64, now: NaiveDateTime) {
    assert!(claim(db, id, &document(), now).await.unwrap());
}

#[test]
fn backoff_doubles_from_one_minute_and_caps_at_six_hours() {
    assert_eq!(retry_delay(1), Duration::seconds(60));
    assert_eq!(retry_delay(2), Duration::seconds(120));
    assert_eq!(retry_delay(5), Duration::seconds(960));
    assert_eq!(retry_delay(9), Duration::seconds(15_360));
    assert_eq!(retry_delay(10), Duration::seconds(RETRY_MAX_SECONDS));
    assert_eq!(retry_delay(1_000), Duration::seconds(RETRY_MAX_SECONDS));
    assert_eq!(retry_delay(0), Duration::seconds(60));
}

#[tokio::test]
async fn claim_is_won_once_and_records_the_document_id() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let now = at("2099-01-01 10:00:00");

    assert!(claim(&db, queued.id, &document(), now).await.unwrap());
    assert!(!claim(&db, queued.id, &document(), now).await.unwrap());

    let row = ts_submission_repository::load(&db, queued.id)
        .await
        .unwrap();
    assert_eq!(row.status, TsSubmissionStatus::Inviata);
    assert_eq!(row.attempt_count, 1);
    assert_eq!(row.document, Some(document()));
    assert!(due(&db, now).await.unwrap().is_empty());
}

#[tokio::test]
async fn failed_call_goes_back_to_the_queue_with_backoff() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let now = at("2099-01-01 10:00:00");
    claimed(&db, queued.id, now).await;

    let released = release_for_retry(&db, queued.id, "timeout", now)
        .await
        .unwrap();
    assert_eq!(released.status, TsSubmissionStatus::NonInviata);
    assert_eq!(released.last_error.as_deref(), Some("timeout"));
    assert_eq!(released.next_attempt_at, "2099-01-01 10:01:00");
    assert!(due(&db, at("2099-01-01 10:00:30"))
        .await
        .unwrap()
        .is_empty());
    assert_eq!(due(&db, at("2099-01-01 10:01:00")).await.unwrap().len(), 1);

    claimed(&db, queued.id, at("2099-01-01 10:01:00")).await;
    let again = release_for_retry(&db, queued.id, "dns", at("2099-01-01 10:01:00"))
        .await
        .unwrap();
    assert_eq!(again.next_attempt_at, "2099-01-01 10:03:00");
}

#[tokio::test]
async fn acceptance_records_protocol_and_clears_the_last_error() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let now = at("2099-01-01 10:00:00");
    claimed(&db, queued.id, now).await;
    release_for_retry(&db, queued.id, "timeout", now)
        .await
        .unwrap();
    claimed(&db, queued.id, now).await;

    let accepted = record_outcome(&db, queued.id, &verdict(true), now)
        .await
        .unwrap();
    assert_eq!(accepted.status, TsSubmissionStatus::Accettata);
    assert_eq!(accepted.protocol.as_deref(), Some("99260926001866680"));
    assert!(accepted.last_error.is_none());
    assert_eq!(accepted.sent_at.as_deref(), Some("2099-01-01 10:00:00"));
}

#[tokio::test]
async fn rejection_keeps_the_code() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let now = at("2099-01-01 10:00:00");
    claimed(&db, queued.id, now).await;
    let rejected = record_outcome(&db, queued.id, &verdict(false), now)
        .await
        .unwrap();
    assert_eq!(rejected.status, TsSubmissionStatus::Scartata);
    assert_eq!(rejected.outcome_code.as_deref(), Some("S017"));
    assert!(rejected.sent_at.is_none());
}

#[tokio::test]
async fn local_rejection_ends_a_queued_submission() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let rejected = reject_locally(&db, queued.id, "manca la data", at("2099-01-01 10:00:00"))
        .await
        .unwrap();
    assert_eq!(rejected.status, TsSubmissionStatus::Scartata);
    assert_eq!(rejected.outcome_code.as_deref(), Some("LOCALE"));
}

#[tokio::test]
async fn refuses_transitions_out_of_order() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let now = at("2099-01-01 10:00:00");
    assert!(record_outcome(&db, queued.id, &verdict(true), now)
        .await
        .is_err());
    assert!(release_for_retry(&db, queued.id, "x", now).await.is_err());

    force_status(&db, queued.id, TsSubmissionStatus::Accettata).await;
    assert!(!claim(&db, queued.id, &document(), now).await.unwrap());
    assert!(reject_locally(&db, queued.id, "x", now).await.is_err());
}

#[tokio::test]
async fn accepted_cancellation_retires_its_target() {
    let db = setup().await;
    let now = at("2099-01-01 10:00:00");
    let original = queue::enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, original.id, TsSubmissionStatus::Accettata).await;

    let cancellation = queue::enqueue_cancellation(&db, original.id).await.unwrap();
    claimed(&db, cancellation.id, now).await;
    record_outcome(&db, cancellation.id, &verdict(true), now)
        .await
        .unwrap();

    let original = ts_submission_repository::load(&db, original.id)
        .await
        .unwrap();
    assert_eq!(original.status, TsSubmissionStatus::Annullata);
}

#[tokio::test]
async fn accepted_replacement_supersedes_its_target_and_rejection_leaves_it() {
    let db = setup().await;
    let now = at("2099-01-01 10:00:00");
    let original = queue::enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, original.id, TsSubmissionStatus::Accettata).await;

    let rejected = queue::enqueue_replacement(&db, original.id).await.unwrap();
    claimed(&db, rejected.id, now).await;
    record_outcome(&db, rejected.id, &verdict(false), now)
        .await
        .unwrap();
    let still = ts_submission_repository::load(&db, original.id)
        .await
        .unwrap();
    assert_eq!(still.status, TsSubmissionStatus::Accettata);

    let replacement = queue::enqueue_replacement(&db, original.id).await.unwrap();
    claimed(&db, replacement.id, now).await;
    record_outcome(&db, replacement.id, &verdict(true), now)
        .await
        .unwrap();
    let original = ts_submission_repository::load(&db, original.id)
        .await
        .unwrap();
    assert_eq!(original.status, TsSubmissionStatus::Sostituita);
}

#[tokio::test]
async fn start_up_requeues_calls_left_in_flight() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let now = at("2099-01-01 10:00:00");
    claimed(&db, queued.id, now).await;

    assert_eq!(requeue_interrupted(&db, now).await.unwrap(), 1);
    let row = ts_submission_repository::load(&db, queued.id)
        .await
        .unwrap();
    assert_eq!(row.status, TsSubmissionStatus::NonInviata);
    assert_eq!(due(&db, now).await.unwrap().len(), 1);
}
