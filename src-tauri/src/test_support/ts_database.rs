//! An in-memory database migrated to the current schema, seeded with the
//! client, invoices and Sistema TS settings the queue tests work on.

use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use crate::app::model::ts::TsSubmissionStatus;

pub async fn setup() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(
        "INSERT INTO clients (id, first_name, last_name, fiscal_code, sts_authorization)
           VALUES (1, 'Anna', 'Bianchi', 'RSSMRA80A41H501Y', 1);
         INSERT INTO invoices (id, client_id, invoice_number, year, issue_date, status, paid_date, total_gross)
           VALUES (1, 1, '1', 2026, '2026-03-01', 'paid', '2026-03-02', 81.64),
                  (2, 1, '2', 2026, '2026-03-05', 'issued', NULL, 81.64),
                  (3, 1, '3', 2026, '2026-03-06', 'paid', NULL, 81.64);
         INSERT INTO ts_settings (id, environment, username, vat_number)
           VALUES (1, 'test', 'MTOMRA66A41G224M', '65498732105');",
    )
    .await
    .unwrap();
    db
}

/// Forces a status. An accepted submission was necessarily sent, so it carries the document
/// id it was sent under unless the test already gave it one.
pub async fn force_status(db: &DatabaseConnection, id: i64, status: TsSubmissionStatus) {
    db.execute_unprepared(&format!(
        "UPDATE ts_submissions SET status = '{}' WHERE id = {id}",
        status.as_str()
    ))
    .await
    .unwrap();
    if status == TsSubmissionStatus::Accettata {
        give_sent_document_id(db, id).await;
    }
}

async fn give_sent_document_id(db: &DatabaseConnection, id: i64) {
    db.execute_unprepared(&format!(
        "UPDATE ts_submissions SET
            document_vat_number = COALESCE(document_vat_number, '12345678901'),
            document_issue_date = COALESCE(document_issue_date,
                (SELECT issue_date FROM invoices WHERE invoices.id = ts_submissions.invoice_id)),
            document_number = COALESCE(document_number,
                (SELECT invoice_number FROM invoices WHERE invoices.id = ts_submissions.invoice_id))
         WHERE id = {id}"
    ))
    .await
    .unwrap();
}
