use chrono::NaiveDateTime;

use crate::app::common::Clock;

use super::clock::at;

/// A clock stopped at one moment, the same in UTC and local time.
pub struct FixedClock {
    moment: NaiveDateTime,
}

impl FixedClock {
    pub fn at(moment: &str) -> Self {
        Self { moment: at(moment) }
    }
}

/// 10 March 2026, 09:00: before every moment the queue tests dispatch at.
impl Default for FixedClock {
    fn default() -> Self {
        Self::at("2026-03-10 09:00:00")
    }
}

impl Clock for FixedClock {
    fn utc_now(&self) -> NaiveDateTime {
        self.moment
    }

    fn local_now(&self) -> NaiveDateTime {
        self.moment
    }
}
