use super::*;

#[test]
fn last_day_of_month_handles_regular_and_leap_years() {
    assert_eq!(last_day_of_month(2026, 1).unwrap(), "2026-01-31");
    assert_eq!(last_day_of_month(2026, 12).unwrap(), "2026-12-31");
    assert_eq!(last_day_of_month(2024, 2).unwrap(), "2024-02-29");
    assert_eq!(last_day_of_month(2026, 2).unwrap(), "2026-02-28");
}

#[test]
fn paid_date_kept_for_paid_and_cleared_otherwise() {
    let kept = resolve_paid_date(&InvoiceStatus::Paid, Some("2026-07-01".to_string()));
    assert_eq!(kept, Some("2026-07-01".to_string()));

    let defaulted = resolve_paid_date(&InvoiceStatus::Paid, None);
    assert!(defaulted.is_some());

    let cleared = resolve_paid_date(&InvoiceStatus::Issued, Some("2026-07-01".to_string()));
    assert_eq!(cleared, None);
}

#[test]
fn extract_year_requires_valid_date() {
    assert_eq!(extract_year("2026-07-04").unwrap(), 2026);
    assert!(extract_year("not-a-date").is_err());
}

#[test]
fn invoice_number_falls_back_to_current_when_not_provided() {
    assert_eq!(resolve_invoice_number(None, "007").unwrap(), 7);
    assert_eq!(resolve_invoice_number(Some(""), "007").unwrap(), 7);
    assert_eq!(resolve_invoice_number(Some("  "), "007").unwrap(), 7);
    assert_eq!(resolve_invoice_number(Some("12"), "007").unwrap(), 12);
    assert_eq!(resolve_invoice_number(Some(" 042 "), "007").unwrap(), 42);
}

#[test]
fn invoice_number_rejects_non_positive_or_non_numeric() {
    assert!(resolve_invoice_number(Some("0"), "007").is_err());
    assert!(resolve_invoice_number(Some("-3"), "007").is_err());
    assert!(resolve_invoice_number(Some("abc"), "007").is_err());
}

async fn test_db() -> DatabaseConnection {
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;

    let db = Database::connect("sqlite::memory:").await.unwrap();
    crate::migration::Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(
        "INSERT INTO clients (client_type, first_name, last_name, fiscal_code,
         address, city, province, zip_code, phone, sts_authorization)
         VALUES ('persona_fisica', 'Luca', 'Bianchi', 'BNCLCU85B02H501K',
         'Via Milano 2', 'Roma', 'RM', '00100', '', 0)",
    )
    .await
    .unwrap();
    db
}

fn invoice_input(issue_date: &str) -> CreateInvoiceInput {
    CreateInvoiceInput {
        client_id: 1,
        issue_date: issue_date.to_string(),
        due_date: None,
        status: InvoiceStatus::Issued,
        payment_method: crate::app::model::invoice::PaymentMethod::Bonifico,
        notes: String::new(),
        apply_enpap: true,
        lines: vec![InvoiceLineInput {
            service_id: None,
            description: "Seduta di psicoterapia".to_string(),
            quantity: 1,
            unit_price: 70.0,
            vat_rate: 0.0,
        }],
    }
}

fn update_input_from(invoice: &Invoice, new_number: Option<&str>) -> UpdateInvoiceInput {
    UpdateInvoiceInput {
        id: invoice.id,
        client_id: invoice.client_id,
        invoice_number: new_number.map(str::to_string),
        issue_date: invoice.issue_date.clone(),
        due_date: None,
        status: InvoiceStatus::Issued,
        payment_method: crate::app::model::invoice::PaymentMethod::Bonifico,
        notes: String::new(),
        apply_enpap: true,
        paid_date: None,
        lines: invoice
            .lines
            .iter()
            .map(|l| InvoiceLineInput {
                service_id: l.service_id,
                description: l.description.clone(),
                quantity: l.quantity,
                unit_price: l.unit_price,
                vat_rate: l.vat_rate,
            })
            .collect(),
    }
}

#[tokio::test]
async fn renumbering_enforces_uniqueness_per_year_and_fills_gaps() {
    let db = test_db().await;

    let first = create(&db, invoice_input("2026-03-01")).await.unwrap();
    let second = create(&db, invoice_input("2026-04-01")).await.unwrap();
    assert_eq!(first.invoice_number, "001");
    assert_eq!(second.invoice_number, "002");

    // Taking a number already in use must fail with a clear message.
    let err = update(&db, update_input_from(&second, Some("001")))
        .await
        .unwrap_err();
    assert!(err.contains("già utilizzato"));

    // Renumbering to a free number works and is zero-padded.
    let renumbered = update(&db, update_input_from(&second, Some("7")))
        .await
        .unwrap();
    assert_eq!(renumbered.invoice_number, "007");

    // The freed number 002 can now be reassigned (gap filling).
    let third = create(&db, invoice_input("2026-05-01")).await.unwrap();
    let filled = update(&db, update_input_from(&third, Some("2")))
        .await
        .unwrap();
    assert_eq!(filled.invoice_number, "002");

    // Update without a number keeps the current one.
    let untouched = update(&db, update_input_from(&renumbered, None))
        .await
        .unwrap();
    assert_eq!(untouched.invoice_number, "007");
}

#[tokio::test]
async fn moving_issue_date_to_another_year_updates_year_column() {
    let db = test_db().await;

    let invoice = create(&db, invoice_input("2026-03-01")).await.unwrap();
    assert_eq!(invoice.year, 2026);

    let mut input = update_input_from(&invoice, None);
    input.issue_date = "2025-12-31".to_string();
    let moved = update(&db, input).await.unwrap();
    assert_eq!(moved.year, 2025);
}
