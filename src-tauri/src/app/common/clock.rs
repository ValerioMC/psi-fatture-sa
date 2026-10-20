use chrono::NaiveDateTime;

use super::timestamp;

/// Where the time comes from. Only the wiring builds the system clock; services receive it,
/// so every rule that depends on today runs in tests at a fixed moment.
pub trait Clock: Send + Sync {
    fn utc_now(&self) -> NaiveDateTime;

    /// The wall-clock time where the app runs: what dates an invoice and names a backup.
    fn local_now(&self) -> NaiveDateTime;

    /// `utc_now` as `created_at`-style columns store it.
    fn timestamp(&self) -> String {
        timestamp::format(self.utc_now())
    }

    /// Today's local date as `YYYY-MM-DD`.
    fn today(&self) -> String {
        self.local_now().format("%Y-%m-%d").to_string()
    }
}
