use super::*;
use crate::app::common::timestamp;

#[test]
fn the_timestamp_reads_back_with_the_stored_format() {
    let stamp = SystemClock.timestamp();

    assert!(NaiveDateTime::parse_from_str(&stamp, timestamp::FORMAT).is_ok());
}

#[test]
fn today_is_an_iso_date() {
    assert!(chrono::NaiveDate::parse_from_str(&SystemClock.today(), "%Y-%m-%d").is_ok());
}
