//! An in-memory database with a professional profile, a patient and invoices in
//! every state an email can meet: issued, paid, draft and cancelled.

use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub const ISSUED: i64 = 1;
pub const PAID: i64 = 2;
pub const DRAFT: i64 = 3;
pub const CANCELLED: i64 = 4;

pub async fn setup(patient_email: Option<&str>) -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    let email = patient_email.map_or("NULL".to_string(), |email| format!("'{email}'"));
    db.execute_unprepared(&format!(
        "INSERT INTO professional_config (id, title, first_name, last_name, fiscal_code, vat_number,
             address, city, province, zip_code, pec_email)
           VALUES (1, 'Dott.ssa', 'Maria', 'Demo', 'DMEMRA80A41H501X', '12345678903',
             'Corso Buenos Aires 10', 'Milano', 'MI', '20124', 'maria.demo@psypec.it');
         INSERT INTO clients (id, first_name, last_name, fiscal_code, email, sts_authorization)
           VALUES (1, 'Anna', 'Bianchi', 'BNCNNA80A41H501U', {email}, 1);
         INSERT INTO invoices (id, client_id, invoice_number, year, issue_date, status, total_net, total_gross, total_due)
           VALUES (1, 1, '12', 2026, '2026-03-20', 'issued', 80, 83.6, 83.6),
                  (2, 1, '13', 2026, '2026-03-21', 'paid', 80, 83.6, 83.6),
                  (3, 1, '14', 2026, '2026-03-22', 'draft', 80, 83.6, 83.6),
                  (4, 1, '15', 2026, '2026-03-23', 'cancelled', 80, 83.6, 83.6);
         INSERT INTO invoice_lines (invoice_id, description, quantity, unit_price, vat_rate, line_total)
           VALUES (1, 'Colloquio psicologico', 1, 80, 0, 80), (2, 'Colloquio psicologico', 1, 80, 0, 80);"
    ))
    .await
    .unwrap();
    db
}
