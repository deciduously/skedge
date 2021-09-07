//! # skedge
//!
//! `skedge` is a single-process job scheduler.
//!
//! ```rust
//! use skedge::{Scheduler, every, every_single};
//! use chrono::Local;
//! use std::time::Duration;
//! use std::thread::sleep;
//!
//! fn job() {
//!     println!("Hello, it's {}!", Local::now());
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    let mut schedule = Scheduler::new();
//!
//!     every(10).seconds()?.run(&mut schedule, job)?;
//!     every(10).minutes()?.run(&mut schedule, job)?;
//!     every_single().hour()?.run(&mut schedule, job)?;
//!     every_single().day()?.at("10:30")?.run(&mut schedule, job)?;
//!     every(5).to(10)?.minutes()?.run(&mut schedule, job)?;
//!     every_single().monday()?.run(&mut schedule, job)?;
//!     every_single().wednesday()?.at("13:15")?.run(&mut schedule, job)?;
//!     every_single().minute()?.at(":17")?.run(&mut schedule, job)?;
//!     every(2).to(8)?.seconds()?.until(Local::now() + chrono::Duration::seconds(30))?.run(&mut schedule, job)?;
//!
//!     // loop {
//!     //     if let Err(e) = schedule.run_pending() {
//!     //         eprintln!("Error: {}", e);
//!     //     }
//!     //     sleep(Duration::from_secs(1));
//!     // }
//!     Ok(())
//! }
//! ```

use chrono::{prelude::*, Duration};
use std::fmt;

mod callable;
mod error;
mod job;
mod scheduler;

use callable::{Callable, UnitToUnit};
pub use job::{every, every_single};
pub use scheduler::Scheduler;

/// Each interval value is an unsigned 32-bit integer
type Interval = u32;

/// A Tag is used to categorize a job.
type Tag = String;

/// Timestamps are in the users local timezone
type Timestamp = DateTime<Local>;

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
    fn duration(&self, interval: u32) -> Duration {
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
