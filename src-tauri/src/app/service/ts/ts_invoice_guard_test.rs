use sea_orm::{ConnectionTrait, DatabaseConnection};

use super::*;
use crate::test_support::ts_database::setup;

async fn transmit(db: &DatabaseConnection, invoice_id: i64, status: &str, environment: &str) {
    db.execute_unprepared(&format!(
        "INSERT INTO ts_submissions (invoice_id, operation, status, environment)
         VALUES ({invoice_id}, 'invio', '{status}', '{environment}')"
    ))
    .await
    .unwrap();
}

#[tokio::test]
async fn a_transmitted_invoice_keeps_number_date_and_paid_status() {
    let db = setup().await;
    transmit(&db, 1, "accettata", "produzione").await;
    let current = invoice_repository::require(&db, 1).await.unwrap();
    let update = |number, date: &'static str, status| {
        let (db, current) = (&db, &current);
        async move { ensure_invoice_update_allowed(db, current, number, date, &status).await }
    };

    assert!(update(1, "2026-03-01", InvoiceStatus::Paid).await.is_ok());
    let renumbered = update(7, "2026-03-01", InvoiceStatus::Paid).await;
    assert!(matches!(renumbered, Err(AppError::Conflict(_))));
    let redated = update(1, "2026-03-02", InvoiceStatus::Paid).await;
    assert!(matches!(redated, Err(AppError::Conflict(_))));
    let unpaid = update(1, "2026-03-01", InvoiceStatus::Issued).await;
    assert!(matches!(unpaid, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn an_invoice_never_accepted_can_change_freely() {
    let db = setup().await;
    transmit(&db, 1, "scartata", "produzione").await;
    let current = invoice_repository::require(&db, 1).await.unwrap();

    let changed =
        ensure_invoice_update_allowed(&db, &current, 9, "2026-05-01", &InvoiceStatus::Issued);
    assert!(changed.await.is_ok());
}

#[tokio::test]
async fn bulk_status_changes_spare_transmitted_invoices() {
    let db = setup().await;
    transmit(&db, 1, "accettata", "produzione").await;

    let error = ensure_invoice_status_allowed(&db, &[2, 1], &InvoiceStatus::Cancelled)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("n. 1 "));
    assert!(
        ensure_invoice_status_allowed(&db, &[2, 1], &InvoiceStatus::Paid)
            .await
            .is_ok()
    );
    assert!(
        ensure_invoice_status_allowed(&db, &[2, 3], &InvoiceStatus::Cancelled)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn removal_waits_for_the_production_data_to_be_cancelled() {
    let db = setup().await;
    assert!(ensure_invoice_removable(&db, 1).await.is_ok());

    transmit(&db, 1, "accettata", "produzione").await;
    let error = ensure_invoice_removable(&db, 1).await.unwrap_err();
    assert!(matches!(error, AppError::Conflict(_)));

    db.execute_unprepared("UPDATE ts_submissions SET status = 'annullata'")
        .await
        .unwrap();
    assert!(ensure_invoice_removable(&db, 1).await.is_ok());
}

#[tokio::test]
async fn test_environment_data_never_blocks_removal() {
    let db = setup().await;
    transmit(&db, 1, "accettata", "test").await;

    assert!(ensure_invoice_removable(&db, 1).await.is_ok());
}
