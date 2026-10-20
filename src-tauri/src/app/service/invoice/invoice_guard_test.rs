use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use super::*;

async fn database() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(
        "INSERT INTO clients (client_type, first_name, last_name, fiscal_code,
         address, city, province, zip_code, phone, sts_authorization)
         VALUES ('persona_fisica', 'Luca', 'Bianchi', 'BNCLCU85B02H501K',
         'Via Milano 2', 'Roma', 'RM', '00100', '', 1);
         INSERT INTO invoices (id, client_id, invoice_number, year, issue_date, status, paid_date)
         VALUES (1, 1, '001', 2026, '2026-03-10', 'paid', '2026-03-10'),
                (2, 1, '002', 2026, '2026-03-20', 'issued', NULL),
                (3, 1, '003', 2026, '2026-04-05', 'issued', NULL);",
    )
    .await
    .unwrap();
    db
}

#[tokio::test]
async fn accepts_a_date_between_its_neighbours() {
    let db = database().await;

    assert!(ensure_chronological(&db, 2026, 2, "2026-03-15", 2)
        .await
        .is_ok());
    assert!(ensure_chronological(&db, 2026, 4, "2026-04-05", 0)
        .await
        .is_ok());
}

#[tokio::test]
async fn refuses_a_higher_number_dated_before_a_lower_one() {
    let db = database().await;

    let error = ensure_chronological(&db, 2026, 4, "2026-04-01", 0)
        .await
        .unwrap_err();

    assert!(matches!(error, AppError::Conflict(_)));
    assert!(error.to_string().contains("n. 003"));
    assert!(error.to_string().contains("dal 5 aprile 2026 in poi"));
}

#[tokio::test]
async fn refuses_a_lower_number_dated_after_a_higher_one() {
    let db = database().await;

    let error = ensure_chronological(&db, 2026, 2, "2026-04-10", 2)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("fino al 5 aprile 2026"));
}
