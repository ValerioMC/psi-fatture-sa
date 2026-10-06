use super::*;

use chrono::NaiveDate;

#[test]
fn formats_a_moment_as_the_stored_text() {
    let moment = NaiveDate::from_ymd_opt(2026, 3, 5)
        .unwrap()
        .and_hms_opt(9, 7, 3)
        .unwrap();
    assert_eq!(format(moment), "2026-03-05 09:07:03");
}

#[test]
fn the_current_moment_reads_back_with_the_stored_format() {
    assert!(NaiveDateTime::parse_from_str(&now(), FORMAT).is_ok());
}

#[test]
fn today_is_an_iso_date() {
    assert!(chrono::NaiveDate::parse_from_str(&today(), "%Y-%m-%d").is_ok());
}
