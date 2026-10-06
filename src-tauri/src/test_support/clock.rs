use chrono::NaiveDateTime;

use crate::app::common::timestamp;

/// A moment written the way the queue stores it, `YYYY-MM-DD HH:MM:SS`.
pub fn at(value: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(value, timestamp::FORMAT).unwrap()
}
