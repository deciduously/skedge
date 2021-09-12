//! For mocking purposes, access to the current time is controlled directed through this struct.

use chrono::{prelude::*, Duration};
use std::fmt;

/// Timestamps are in the users local timezone
pub type Timestamp = DateTime<Local>;

pub(crate) trait Timekeeper: std::fmt::Debug {
    /// Return the current time
    fn now(&self) -> Timestamp;
    /// Add a specific duration for testing purposes
    #[cfg(test)]
    fn add_duration(&mut self, duration: Duration);
}

impl PartialEq for dyn Timekeeper {
    fn eq(&self, other: &Self) -> bool {
        self.now() - other.now() < Duration::milliseconds(10)
    }
}

impl Eq for dyn Timekeeper {}

#[derive(Debug, Default, Clone, Copy)]
pub struct RealTime;

impl Timekeeper for RealTime {
    fn now(&self) -> Timestamp {
        Local::now()
    }
    #[cfg(test)]
    fn add_duration(&mut self, _duration: Duration) {
        unreachable!() // unneeded
    }
}

/// Jobs can be periodic over one of these units of time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl TimeUnit {
    /// Get a chrono::Duration from an interval based on time unit
    pub fn duration(&self, interval: u32) -> Duration {
        use TimeUnit::*;
        let interval = interval as i64;
        match self {
            Second => Duration::seconds(interval),
            Minute => Duration::minutes(interval),
            Hour => Duration::hours(interval),
            Day => Duration::days(interval),
            Week => Duration::weeks(interval),
            Month => Duration::weeks(interval * 4),
            Year => Duration::weeks(interval * 52),
        }
    }
}

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TimeUnit::*;
        let s = match self {
            Second => "second",
            Minute => "minute",
            Hour => "hour",
            Day => "day",
            Week => "week",
            Month => "month",
            Year => "year",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        /// Default starting time
        pub static ref START: Timestamp = Local.ymd(2021, 1, 1).and_hms(12, 0, 0);
    }

    /// Mock the datetime for predictable results.
    #[derive(Debug, Clone, Copy)]
    pub struct MockTime {
        stamp: Timestamp,
    }

    impl MockTime {
        pub fn new(stamp: Timestamp) -> Self {
            Self { stamp }
        }
    }

    impl Default for MockTime {
        fn default() -> Self {
            Self::new(*START)
        }
    }

    impl Timekeeper for MockTime {
        fn now(&self) -> Timestamp {
            self.stamp
        }

        fn add_duration(&mut self, duration: chrono::Duration) {
            self.stamp = self.stamp + duration;
        }
    }
}
