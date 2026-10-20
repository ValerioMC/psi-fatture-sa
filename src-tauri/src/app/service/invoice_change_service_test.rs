use sea_orm::ConnectionTrait;

use super::*;
use crate::app::model::invoice::InvoiceStatus;
use crate::app::model::ts::{TsSubmissionFilters, TsSubmissionStatus};
use crate::test_support::ts_database::{force_status, setup};
use crate::test_support::FixedClock;

async fn use_production(db: &DatabaseConnection) {
    db.execute_unprepared("UPDATE ts_settings SET environment = 'produzione'")
        .await
        .unwrap();
}

async fn accepted_in_production(db: &DatabaseConnection) -> i64 {
    use_production(db).await;
    let original = ts_submission_service::enqueue_invio(db, &FixedClock::default(), 1)
        .await
        .unwrap();
    force_status(db, original.id, TsSubmissionStatus::Accettata).await;
    original.id
}

#[tokio::test]
async fn deletion_is_refused_while_accepted_and_cascades_after_cancellation() {
    let db = setup().await;
    let original = accepted_in_production(&db).await;
    let error = remove(&db, 1).await.unwrap_err();
    assert!(error.to_string().contains("Sistema TS"));

    let cancellation =
        ts_submission_service::enqueue_cancellation(&db, &FixedClock::default(), original)
            .await
            .unwrap();
    force_status(&db, cancellation.id, TsSubmissionStatus::Accettata).await;
    force_status(&db, original, TsSubmissionStatus::Annullata).await;
    db.execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .unwrap();

    remove(&db, 1).await.unwrap();
    let left = ts_submission_service::list(&db, TsSubmissionFilters::default())
        .await
        .unwrap();
    assert!(left.is_empty());
}

#[tokio::test]
async fn a_bulk_change_off_paid_is_refused_for_an_accepted_invoice() {
    let db = setup().await;
    accepted_in_production(&db).await;

    let input = BulkUpdateStatusInput {
        ids: vec![1, 2],
        status: InvoiceStatus::Issued,
        paid_date: None,
    };
    let error = bulk_update_status(&db, &FixedClock::default(), input)
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Conflict(_)));
    let unchanged = invoice_service::get(&db, 2).await.unwrap();
    assert_eq!(unchanged.status, InvoiceStatus::Issued);
}
