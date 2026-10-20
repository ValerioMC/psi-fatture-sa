use super::*;

#[test]
fn pads_a_number_to_three_digits() {
    assert_eq!(format_invoice_number(7), "007");
    assert_eq!(format_invoice_number(1234), "1234");
}

#[test]
fn reads_a_stored_number_with_its_padding() {
    assert_eq!(parse_stored_number("007").unwrap(), 7);
}

#[test]
fn refuses_a_stored_number_that_is_not_numeric() {
    let error = parse_stored_number("7bis").unwrap_err();

    assert!(matches!(error, AppError::Database(_)));
    assert!(error.to_string().contains("«7bis»"));
}
