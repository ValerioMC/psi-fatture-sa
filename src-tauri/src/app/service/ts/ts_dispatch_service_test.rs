use super::*;
use crate::app::model::ts::{TsSubmissionStatus, TsVatTreatment};
use crate::app::repository::secret::SecretKind;
use crate::app::service::ts::ts_submission_service as queue;
use crate::test_support::clock::at;
use crate::test_support::ts_call_responses::{accepted, rejected};
use crate::test_support::ts_database::setup;
use crate::test_support::{FakeGateway, InMemorySecretStore, RecordedCall};
use sea_orm::ConnectionTrait;

fn store() -> InMemorySecretStore {
    let store = InMemorySecretStore::default();
    store.write(SecretKind::TsPassword, "Salve123").unwrap();
    store.write(SecretKind::TsPincode, "3489543096").unwrap();
    store
}

async fn run(db: &DatabaseConnection, gateway: &FakeGateway, now: &str) -> TsDispatchSummary {
    dispatch_due(db, &store(), gateway, at(now)).await.unwrap()
}

async fn load(db: &DatabaseConnection, id: i64) -> TsSubmission {
    ts_submission_repository::load(db, id).await.unwrap()
}

#[tokio::test]
async fn sends_a_queued_invoice_and_records_the_protocol() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Ok(accepted("99260926001866680")));

    let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert_eq!(summary.accepted, 1);

    let sent = load(&db, queued.id).await;
    assert_eq!(sent.status, TsSubmissionStatus::Accettata);
    assert_eq!(sent.protocol.as_deref(), Some("99260926001866680"));
    assert_eq!(sent.document.as_ref().unwrap().vat_number, "65498732105");

    let calls = gateway.recorded();
    let [RecordedCall::Insert(document)] = calls.as_slice() else {
        panic!("expected one insert");
    };
    assert_eq!(
        document.citizen_fiscal_code.as_deref(),
        Some("RSSMRA80A41H501Y")
    );
    assert_eq!(document.items[0].amount, 81.64);
    assert_eq!(
        document.items[0].vat,
        TsVatTreatment::Natura("N2.2".to_string())
    );
}

#[tokio::test]
async fn missing_credentials_block_the_pass_without_touching_the_queue() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    let summary = dispatch_due(
        &db,
        &InMemorySecretStore::default(),
        &gateway,
        at("2099-01-01 10:00:00"),
    )
    .await
    .unwrap();
    assert!(summary.blocked.unwrap().contains("password"));
    assert!(gateway.recorded().is_empty());
    assert_eq!(
        load(&db, queued.id).await.status,
        TsSubmissionStatus::NonInviata
    );
}

#[tokio::test]
async fn unreachable_service_stops_the_pass() {
    let db = setup().await;
    db.execute_unprepared(
        "UPDATE invoices SET status = 'paid', paid_date = '2026-03-07' WHERE id = 3",
    )
    .await
    .unwrap();
    queue::enqueue_invio(&db, 1).await.unwrap();
    let second = queue::enqueue_invio(&db, 3).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Err(TsGatewayError::Network("timed out".to_string())));

    let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert!(summary.blocked.unwrap().contains("non raggiungibile"));
    assert_eq!(gateway.recorded().len(), 1);
    assert_eq!(load(&db, second.id).await.attempt_count, 0);
}

#[tokio::test]
async fn network_errors_and_ws99_are_retried_later() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Err(TsGatewayError::Network("timeout".to_string())));
    gateway.script(Ok(rejected("WS99")));

    assert_eq!(run(&db, &gateway, "2099-01-01 10:00:00").await.retrying, 1);
    let first = load(&db, queued.id).await;
    assert_eq!(first.status, TsSubmissionStatus::NonInviata);
    assert!(first.last_error.unwrap().contains("timeout"));

    assert_eq!(
        run(&db, &gateway, "2099-01-01 10:00:30").await,
        TsDispatchSummary::default()
    );
    assert_eq!(run(&db, &gateway, "2099-01-01 10:01:00").await.retrying, 1);
    assert_eq!(run(&db, &gateway, "2099-01-01 10:03:00").await.accepted, 1);
    assert_eq!(load(&db, queued.id).await.attempt_count, 3);
}

#[tokio::test]
async fn refused_login_stops_the_pass_and_keeps_everything_queued() {
    let db = setup().await;
    db.execute_unprepared(
        "UPDATE invoices SET status = 'paid', paid_date = '2026-03-07' WHERE id = 3",
    )
    .await
    .unwrap();
    let first = queue::enqueue_invio(&db, 1).await.unwrap();
    let second = queue::enqueue_invio(&db, 3).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Err(TsGatewayError::Authentication));

    let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert!(summary.blocked.unwrap().contains("password"));
    assert_eq!(gateway.recorded().len(), 1);
    assert_eq!(
        load(&db, first.id).await.status,
        TsSubmissionStatus::NonInviata
    );
    assert_eq!(
        load(&db, second.id).await.status,
        TsSubmissionStatus::NonInviata
    );
}

#[tokio::test]
async fn wrong_pincode_is_a_credential_problem_not_a_rejection() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Ok(rejected("005")));
    let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert!(summary.blocked.is_some());
    assert_eq!(
        load(&db, queued.id).await.status,
        TsSubmissionStatus::NonInviata
    );
}

#[tokio::test]
async fn document_errors_are_final() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Ok(rejected("WS19")));
    assert_eq!(run(&db, &gateway, "2099-01-01 10:00:00").await.rejected, 1);
    let rejected = load(&db, queued.id).await;
    assert_eq!(rejected.status, TsSubmissionStatus::Scartata);
    assert_eq!(rejected.outcome_code.as_deref(), Some("WS19"));
}

#[tokio::test]
async fn duplicate_after_a_lost_answer_counts_as_accepted() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Err(TsGatewayError::Network("reset".to_string())));
    gateway.script(Ok(rejected("S017")));

    run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert_eq!(run(&db, &gateway, "2099-01-01 10:01:00").await.accepted, 1);
    assert_eq!(
        load(&db, queued.id).await.status,
        TsSubmissionStatus::Accettata
    );
}

#[tokio::test]
async fn duplicate_on_a_first_attempt_is_a_rejection() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    gateway.script(Ok(rejected("S017")));
    run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert_eq!(
        load(&db, queued.id).await.status,
        TsSubmissionStatus::Scartata
    );
}

#[tokio::test]
async fn replacement_and_cancellation_reuse_the_original_document_id() {
    let db = setup().await;
    let original = queue::enqueue_invio(&db, 1).await.unwrap();
    let gateway = FakeGateway::default();
    run(&db, &gateway, "2099-01-01 10:00:00").await;
    let sent_id = load(&db, original.id).await.document.unwrap();

    db.execute_unprepared(
        "UPDATE invoices SET invoice_number = '1-bis', total_gross = 90 WHERE id = 1",
    )
    .await
    .unwrap();
    let replacement = queue::enqueue_replacement(&db, original.id).await.unwrap();
    run(&db, &gateway, "2099-01-01 10:05:00").await;
    assert_eq!(
        load(&db, original.id).await.status,
        TsSubmissionStatus::Sostituita
    );

    let cancellation = queue::enqueue_cancellation(&db, replacement.id)
        .await
        .unwrap();
    run(&db, &gateway, "2099-01-01 10:10:00").await;
    assert_eq!(
        load(&db, replacement.id).await.status,
        TsSubmissionStatus::Annullata
    );
    assert_eq!(
        load(&db, cancellation.id).await.status,
        TsSubmissionStatus::Accettata
    );

    let calls = gateway.recorded();
    let RecordedCall::Update(updated) = &calls[1] else {
        panic!("expected an update")
    };
    assert_eq!(updated.id, sent_id);
    assert_eq!(updated.items[0].amount, 90.0);
    assert_eq!(calls[2], RecordedCall::Cancel(sent_id));
}

#[tokio::test]
async fn invoice_changed_since_queueing_is_rejected_locally() {
    let db = setup().await;
    let queued = queue::enqueue_invio(&db, 1).await.unwrap();
    db.execute_unprepared("UPDATE invoices SET status = 'issued' WHERE id = 1")
        .await
        .unwrap();
    let gateway = FakeGateway::default();
    assert_eq!(run(&db, &gateway, "2099-01-01 10:00:00").await.rejected, 1);
    assert!(gateway.recorded().is_empty());
    let rejected = load(&db, queued.id).await;
    assert_eq!(rejected.outcome_code.as_deref(), Some("LOCALE"));
}

#[tokio::test]
async fn submissions_for_another_environment_wait() {
    let db = setup().await;
    queue::enqueue_invio(&db, 1).await.unwrap();
    db.execute_unprepared("UPDATE ts_settings SET environment = 'produzione'")
        .await
        .unwrap();
    let gateway = FakeGateway::default();
    let summary = run(&db, &gateway, "2099-01-01 10:00:00").await;
    assert_eq!(summary.waiting_other_environment, 1);
    assert!(gateway.recorded().is_empty());
}
