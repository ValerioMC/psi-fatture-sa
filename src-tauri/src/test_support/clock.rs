use chrono::NaiveDateTime;

use crate::app::service::ts::ts_transition_service::TIMESTAMP_FORMAT;

/// A moment written the way the queue stores it, `YYYY-MM-DD HH:MM:SS`.
pub fn at(value: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(value, TIMESTAMP_FORMAT).unwrap()
}
