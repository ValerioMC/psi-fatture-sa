use super::*;
use crate::test_support::ts_database::{force_status, setup};

#[tokio::test]
async fn queues_a_paid_invoice_with_its_details() {
    let db = setup().await;
    let queued = enqueue_invio(&db, 1).await.unwrap();
    assert_eq!(queued.status, TsSubmissionStatus::NonInviata);
    assert_eq!(queued.operation, TsOperation::Invio);
    assert_eq!(queued.invoice_number, "1");
    assert_eq!(queued.client_name, "Anna Bianchi");
    assert_eq!(queued.attempt_count, 0);
}

#[tokio::test]
async fn refuses_unpaid_or_undated_invoices() {
    let db = setup().await;
    assert!(enqueue_invio(&db, 2)
        .await
        .unwrap_err()
        .to_string()
        .contains("pagate"));
    assert!(enqueue_invio(&db, 3)
        .await
        .unwrap_err()
        .to_string()
        .contains("data di pagamento"));
    assert!(enqueue_invio(&db, 99).await.is_err());
}

#[tokio::test]
async fn allows_one_in_flight_submission_per_invoice() {
    let db = setup().await;
    let first = enqueue_invio(&db, 1).await.unwrap();
    assert!(enqueue_invio(&db, 1)
        .await
        .unwrap_err()
        .to_string()
        .contains("in corso"));

    force_status(&db, first.id, TsSubmissionStatus::Inviata).await;
    assert!(enqueue_invio(&db, 1)
        .await
        .unwrap_err()
        .to_string()
        .contains("in corso"));
}

#[tokio::test]
async fn requeues_after_rejection_but_not_after_acceptance() {
    let db = setup().await;
    let first = enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, first.id, TsSubmissionStatus::Scartata).await;
    let retry = enqueue_invio(&db, 1).await.unwrap();

    force_status(&db, retry.id, TsSubmissionStatus::Accettata).await;
    assert!(enqueue_invio(&db, 1)
        .await
        .unwrap_err()
        .to_string()
        .contains("sostituzione"));
}

#[tokio::test]
async fn follow_ups_target_only_accepted_submissions() {
    let db = setup().await;
    let original = enqueue_invio(&db, 1).await.unwrap();
    assert!(enqueue_cancellation(&db, original.id).await.is_err());

    force_status(&db, original.id, TsSubmissionStatus::Accettata).await;
    let cancellation = enqueue_cancellation(&db, original.id).await.unwrap();
    assert_eq!(cancellation.operation, TsOperation::Annullamento);
    assert_eq!(cancellation.target_submission_id, Some(original.id));

    assert!(enqueue_replacement(&db, original.id)
        .await
        .unwrap_err()
        .to_string()
        .contains("in corso"));
}

#[tokio::test]
async fn replacement_requires_the_invoice_to_still_be_paid() {
    let db = setup().await;
    let original = enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, original.id, TsSubmissionStatus::Accettata).await;
    db.execute_unprepared("UPDATE invoices SET status = 'issued' WHERE id = 1")
        .await
        .unwrap();

    assert!(enqueue_replacement(&db, original.id)
        .await
        .unwrap_err()
        .to_string()
        .contains("pagate"));
    assert!(enqueue_cancellation(&db, original.id).await.is_ok());
}

#[tokio::test]
async fn withdraws_only_unsent_submissions() {
    let db = setup().await;
    let queued = enqueue_invio(&db, 1).await.unwrap();
    withdraw(&db, queued.id).await.unwrap();
    assert!(list(&db, TsSubmissionFilters::default())
        .await
        .unwrap()
        .is_empty());

    let sent = enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, sent.id, TsSubmissionStatus::Inviata).await;
    assert!(withdraw(&db, sent.id)
        .await
        .unwrap_err()
        .to_string()
        .contains("non ancora inviate"));
}

#[tokio::test]
async fn lists_by_invoice_and_status() {
    let db = setup().await;
    db.execute_unprepared(
        "UPDATE invoices SET status = 'paid', paid_date = '2026-03-07' WHERE id = 3",
    )
    .await
    .unwrap();
    let first = enqueue_invio(&db, 1).await.unwrap();
    enqueue_invio(&db, 3).await.unwrap();
    force_status(&db, first.id, TsSubmissionStatus::Accettata).await;

    let for_invoice = list(
        &db,
        TsSubmissionFilters {
            invoice_id: Some(3),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(for_invoice.len(), 1);
    assert_eq!(for_invoice[0].invoice_id, 3);

    let accepted = list(
        &db,
        TsSubmissionFilters {
            status: Some(TsSubmissionStatus::Accettata),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].id, first.id);
}

#[tokio::test]
async fn blocks_deletion_while_production_holds_the_data() {
    let db = setup().await;
    use_production(&db).await;
    let queued = enqueue_invio(&db, 1).await.unwrap();
    assert!(!blocks_invoice_deletion(&db, 1).await.unwrap());

    force_status(&db, queued.id, TsSubmissionStatus::Accettata).await;
    assert!(blocks_invoice_deletion(&db, 1).await.unwrap());

    force_status(&db, queued.id, TsSubmissionStatus::Annullata).await;
    assert!(!blocks_invoice_deletion(&db, 1).await.unwrap());
}

#[tokio::test]
async fn test_environment_data_never_blocks_deletion() {
    let db = setup().await;
    let queued = enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, queued.id, TsSubmissionStatus::Accettata).await;
    assert!(!blocks_invoice_deletion(&db, 1).await.unwrap());
}

async fn use_production(db: &DatabaseConnection) {
    db.execute_unprepared("UPDATE ts_settings SET environment = 'produzione'")
        .await
        .unwrap();
}
#[tokio::test]
async fn invoice_deletion_is_refused_while_accepted_and_cascades_after_cancellation() {
    use crate::app::service::invoice_service;

    let db = setup().await;
    use_production(&db).await;
    let original = enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, original.id, TsSubmissionStatus::Accettata).await;
    let err = invoice_service::remove(&db, 1).await.unwrap_err();
    assert!(err.to_string().contains("Sistema TS"));

    let cancellation = enqueue_cancellation(&db, original.id).await.unwrap();
    force_status(&db, cancellation.id, TsSubmissionStatus::Accettata).await;
    force_status(&db, original.id, TsSubmissionStatus::Annullata).await;
    db.execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .unwrap();

    invoice_service::remove(&db, 1).await.unwrap();
    let left = list(&db, TsSubmissionFilters::default()).await.unwrap();
    assert!(left.is_empty());
}

async fn accept_as_sent(db: &sea_orm::DatabaseConnection, submission_id: i64) {
    let settings = ts_settings_service::get(db).await.unwrap();
    let sent = ts_document_service::build(db, 1, &settings.vat_number, None)
        .await
        .unwrap();
    let claimed = ts_submission_repository::claim(
        db,
        submission_id,
        &sent.id,
        Some(sent.fingerprint()),
        "2026-03-20 10:00:00",
    )
    .await
    .unwrap();
    assert!(claimed);
    force_status(db, submission_id, TsSubmissionStatus::Accettata).await;
}

#[tokio::test]
async fn an_unchanged_invoice_is_in_step_with_the_sistema_ts() {
    let db = setup().await;
    let sent = enqueue_invio(&db, 1).await.unwrap();
    accept_as_sent(&db, sent.id).await;

    assert!(!invoice_out_of_date(&db, 1).await.unwrap());
}

#[tokio::test]
async fn an_edited_amount_after_acceptance_asks_for_a_replacement() {
    let db = setup().await;
    let sent = enqueue_invio(&db, 1).await.unwrap();
    accept_as_sent(&db, sent.id).await;

    db.execute_unprepared("UPDATE invoices SET total_gross = 91.64 WHERE id = 1")
        .await
        .unwrap();

    assert!(invoice_out_of_date(&db, 1).await.unwrap());
}

#[tokio::test]
async fn a_submission_without_fingerprint_never_reports_a_change() {
    let db = setup().await;
    let sent = enqueue_invio(&db, 1).await.unwrap();
    force_status(&db, sent.id, TsSubmissionStatus::Accettata).await;

    assert!(!invoice_out_of_date(&db, 1).await.unwrap());
}
