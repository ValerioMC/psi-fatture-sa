use super::*;

#[test]
fn groups_thousands_and_keeps_two_decimals() {
    assert_eq!(currency(0.0), "0,00 €");
    assert_eq!(currency(81.6), "81,60 €");
    assert_eq!(currency(2583.1), "2.583,10 €");
    assert_eq!(currency(1_234_567.891), "1.234.567,89 €");
}

#[test]
fn rounds_to_the_cent_and_signs_negatives() {
    assert_eq!(currency(0.005), "0,01 €");
    assert_eq!(currency(-16.0), "-16,00 €");
    assert_eq!(currency(-0.001), "0,00 €");
}

#[test]
fn writes_long_italian_dates() {
    assert_eq!(date_long("2026-03-20"), "20 marzo 2026");
    assert_eq!(date_long("2026-01-01T10:00:00"), "1 gennaio 2026");
    assert_eq!(date_long("2026-12-09"), "9 dicembre 2026");
}

#[test]
fn leaves_unparseable_dates_alone() {
    assert_eq!(date_long(""), "");
    assert_eq!(date_long("2026-13-01"), "2026-13-01");
    assert_eq!(date_long("ieri"), "ieri");
}
