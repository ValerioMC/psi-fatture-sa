//! The one text format the database stores moments in.

use chrono::NaiveDateTime;

pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn format(moment: NaiveDateTime) -> String {
    moment.format(FORMAT).to_string()
}

#[cfg(test)]
#[path = "timestamp_test.rs"]
mod tests;
