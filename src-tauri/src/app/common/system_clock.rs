use chrono::NaiveDateTime;

use super::Clock;

/// The operating system's clock, read nowhere but here.
pub struct SystemClock;

impl Clock for SystemClock {
    fn utc_now(&self) -> NaiveDateTime {
        chrono::Utc::now().naive_utc()
    }

    fn local_now(&self) -> NaiveDateTime {
        chrono::Local::now().naive_local()
    }
}

#[cfg(test)]
#[path = "system_clock_test.rs"]
mod tests;
