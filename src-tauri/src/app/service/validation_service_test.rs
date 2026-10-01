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
fn parses_valid_iso_date() {
    assert!(parse_iso_date("2026-07-04", "test").is_ok());
}

#[test]
fn rejects_malformed_dates() {
    assert!(parse_iso_date("04/07/2026", "test").is_err());
    assert!(parse_iso_date("2026-13-01", "test").is_err());
    assert!(parse_iso_date("", "test").is_err());
}

#[test]
fn validates_month_bounds() {
    assert!(validate_month(1).is_ok());
    assert!(validate_month(12).is_ok());
    assert!(validate_month(0).is_err());
    assert!(validate_month(13).is_err());
}

#[test]
fn validates_year_bounds() {
    assert!(validate_year(2026).is_ok());
    assert!(validate_year(1900).is_err());
}

#[test]
fn validates_time_format() {
    assert!(validate_time("09:30", "test").is_ok());
    assert!(validate_time("23:59", "test").is_ok());
    assert!(validate_time("24:00", "test").is_err());
    assert!(validate_time("9:30", "test").is_err());
    assert!(validate_time("09.30", "test").is_err());
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

#[test]
fn validates_fiscal_code_shapes() {
    assert!(validate_fiscal_code("RSSMRA80A01H501U").is_ok());
    assert!(validate_fiscal_code("rssmra80a01h501u").is_ok());
    assert!(validate_fiscal_code("12345678903").is_ok());
    assert!(validate_fiscal_code("").is_ok());
    assert!(validate_fiscal_code("SHORT").is_err());
    assert!(validate_fiscal_code("RSSMRA80A01H501!").is_err());
}

#[test]
fn rejects_fiscal_code_with_wrong_control_char() {
    let result = validate_fiscal_code("RSSMRA80A01H501X");
    assert!(result
        .unwrap_err()
        .contains("carattere di controllo errato"));
}

#[test]
fn accepts_omocodia_fiscal_code() {
    // RSSMRA80A01H501U with the last three digits replaced by omocodia
    // letters (501 -> RLM) and the control char recomputed accordingly.
    assert!(validate_fiscal_code("RSSMRA80A01HRLMS").is_ok());
}

#[test]
fn validates_vat_number_shape() {
    assert!(validate_vat_number("12345678903").is_ok());
    assert!(validate_vat_number("").is_ok());
    assert!(validate_vat_number("0123456789").is_err());
    assert!(validate_vat_number("0123456789A").is_err());
}

#[test]
fn rejects_vat_number_with_wrong_check_digit() {
    let result = validate_vat_number("01234567890");
    assert!(result.unwrap_err().contains("cifra di controllo errata"));
}

#[test]
fn validates_email_shape() {
    assert!(validate_email("").is_ok());
    assert!(validate_email(" anna@example.it ").is_ok());
    assert!(validate_email("maria.demo@psypec.it").is_ok());
    assert!(validate_email("anna.example.it").is_err());
    assert!(validate_email("anna@example").is_err());
    assert!(validate_email("anna@example.i").is_err());
    assert!(validate_email("an na@example.it").is_err());
    assert!(validate_email("@example.it").is_err());
    assert!(validate_email("anna@@example.it").is_err());
}
