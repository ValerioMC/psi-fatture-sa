use super::*;

fn valid_line() -> InvoiceLineInput {
    InvoiceLineInput {
        service_id: None,
        description: "Seduta di psicoterapia".to_string(),
        quantity: 1,
        unit_price: 70.0,
        vat_rate: 0.0,
        amount_override: None,
    }
}

#[test]
fn due_date_must_not_precede_issue_date() {
    assert!(validate_invoice_dates("2026-07-04", Some("2026-07-03")).is_err());
    assert!(validate_invoice_dates("2026-07-04", Some("2026-07-04")).is_ok());
    assert!(validate_invoice_dates("2026-07-04", None).is_ok());
    assert!(validate_invoice_dates("2026-07-04", Some("")).is_ok());
}

#[test]
fn rejects_empty_invoice_lines() {
    assert!(validate_invoice_lines(&[]).is_err());
}

#[test]
fn rejects_invalid_line_values() {
    let mut blank_description = valid_line();
    blank_description.description = "  ".to_string();
    assert!(validate_invoice_lines(&[blank_description]).is_err());

    let mut zero_quantity = valid_line();
    zero_quantity.quantity = 0;
    assert!(validate_invoice_lines(&[zero_quantity]).is_err());

    let mut negative_price = valid_line();
    negative_price.unit_price = -1.0;
    assert!(validate_invoice_lines(&[negative_price]).is_err());

    let mut bad_vat = valid_line();
    bad_vat.vat_rate = 101.0;
    assert!(validate_invoice_lines(&[bad_vat]).is_err());

    let mut bad_amount = valid_line();
    bad_amount.amount_override = Some(f64::NAN);
    assert!(validate_invoice_lines(&[bad_amount]).is_err());

    assert!(validate_invoice_lines(&[valid_line()]).is_ok());
}
