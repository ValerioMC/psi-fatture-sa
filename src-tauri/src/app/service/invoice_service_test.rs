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
        hide_quantity: None,
        lines: vec![InvoiceLineInput {
            service_id: None,
            description: "Seduta di psicoterapia".to_string(),
            quantity: 1,
            unit_price: 70.0,
            vat_rate: 0.0,
            amount_override: None,
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
        hide_quantity: None,
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
                amount_override: l.amount_override,
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

async fn set_profile(db: &DatabaseConnection, tax_regime: &str, enpap_excludes_bollo: bool) {
    db.execute_unprepared(&format!(
        "INSERT INTO professional_config (id, tax_regime, enpap_excludes_bollo)
         VALUES (1, '{tax_regime}', {})",
        enpap_excludes_bollo as i32
    ))
    .await
    .unwrap();
}

fn package_input() -> CreateInvoiceInput {
    let mut input = invoice_input("2026-03-01");
    input.lines[0].quantity = 4;
    input.lines[0].amount_override = Some(250.0);
    input
}

#[tokio::test]
async fn hand_typed_amount_is_taxed_and_kept_on_the_line() {
    let db = test_db().await;

    let invoice = create(&db, package_input()).await.unwrap();
    assert_eq!(invoice.total_net, 250.0);
    assert_eq!(invoice.contributo_enpap, 5.04);
    assert_eq!(invoice.total_due, 257.04);
    assert_eq!(invoice.lines[0].amount_override, Some(250.0));
    assert_eq!(invoice.lines[0].line_total, 250.0);

    // Saving it again unchanged keeps the amount instead of recomputing 4 × 70.
    let resaved = update(&db, update_input_from(&invoice, None))
        .await
        .unwrap();
    assert_eq!(resaved.total_net, 250.0);
}

#[tokio::test]
async fn invalid_hand_typed_amount_is_refused() {
    let db = test_db().await;
    let mut input = package_input();
    input.lines[0].amount_override = Some(-1.0);
    assert!(create(&db, input).await.is_err());
}

#[tokio::test]
async fn ordinario_leaves_bollo_out_of_enpap() {
    let db = test_db().await;
    set_profile(&db, "ordinario", false).await;

    let mut input = invoice_input("2026-03-01");
    input.lines[0].unit_price = 100.0;
    let invoice = create(&db, input).await.unwrap();
    assert!(invoice.marca_da_bollo);
    assert_eq!(invoice.contributo_enpap, 2.0);
}

#[tokio::test]
async fn forfettario_profile_can_force_bollo_out_of_enpap() {
    let db = test_db().await;
    set_profile(&db, "forfettario", true).await;

    let mut input = invoice_input("2026-03-01");
    input.lines[0].unit_price = 100.0;
    let invoice = create(&db, input).await.unwrap();
    assert_eq!(invoice.contributo_enpap, 2.0);
}

#[tokio::test]
async fn quantity_visibility_follows_patient_then_profile() {
    let db = test_db().await;
    set_profile(&db, "forfettario", false).await;
    db.execute_unprepared("UPDATE professional_config SET hide_quantity_in_invoice = 1")
        .await
        .unwrap();

    let from_profile = create(&db, invoice_input("2026-03-01")).await.unwrap();
    assert!(from_profile.hide_quantity);

    db.execute_unprepared("UPDATE clients SET hide_quantity_in_invoice = 0")
        .await
        .unwrap();
    let from_patient = create(&db, invoice_input("2026-03-02")).await.unwrap();
    assert!(!from_patient.hide_quantity);

    let mut explicit = invoice_input("2026-03-03");
    explicit.hide_quantity = Some(true);
    assert!(create(&db, explicit).await.unwrap().hide_quantity);
}

#[tokio::test]
async fn update_without_visibility_keeps_the_saved_one() {
    let db = test_db().await;
    let mut input = invoice_input("2026-03-01");
    input.hide_quantity = Some(true);
    let invoice = create(&db, input).await.unwrap();

    let kept = update(&db, update_input_from(&invoice, None))
        .await
        .unwrap();
    assert!(kept.hide_quantity);

    let mut shown = update_input_from(&invoice, None);
    shown.hide_quantity = Some(false);
    assert!(!update(&db, shown).await.unwrap().hide_quantity);
}
