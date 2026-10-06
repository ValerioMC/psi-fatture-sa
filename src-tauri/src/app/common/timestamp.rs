//! The one text format the database stores moments in, and the clock reads that produce it.

use chrono::NaiveDateTime;

pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn format(moment: NaiveDateTime) -> String {
    moment.format(FORMAT).to_string()
}

/// The current UTC moment, as stored in `created_at`-style columns.
pub fn now() -> String {
    format(chrono::Utc::now().naive_utc())
}

/// Today's local date as `YYYY-MM-DD`: what an invoice or a payment is dated with.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
#[path = "timestamp_test.rs"]
mod tests;
