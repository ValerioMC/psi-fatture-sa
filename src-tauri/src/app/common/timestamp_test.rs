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
