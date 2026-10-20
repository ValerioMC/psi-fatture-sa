use super::*;

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
        .to_string()
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
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cifra di controllo errata"));
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
