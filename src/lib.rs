//! # skedge
//!
//! `skedge` is a single-process job scheduler.
//!
//! Define a work function.  Currently, this is restricted to functions which take no arguments and return nothing:
//! ```rust
//! fn job() {
//!     println!("Hello, it's {}!", chrono::Local::now());
//! }
//! ```
//! Instantiate a `Scheduler` and schedule jobs:
//! ```rust
//! # use skedge::{Scheduler, every, every_single};
//! # use chrono::Local;
//! # use std::time::Duration;
//! # use std::thread::sleep;
//! # fn job() {
//! #    println!("Hello, it's {}!", Local::now());
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut schedule = Scheduler::new();
//!
//! every(10).seconds()?.run(&mut schedule, job)?;
//! every(10).minutes()?.run(&mut schedule, job)?;
//! every_single().hour()?.run(&mut schedule, job)?;
//! every_single().day()?.at("10:30")?.run(&mut schedule, job)?;
//! every(5).to(10)?.minutes()?.run(&mut schedule, job)?;
//! every_single().monday()?.run(&mut schedule, job)?;
//! every_single().wednesday()?.at("13:15")?.run(&mut schedule, job)?;
//! every_single().minute()?.at(":17")?.run(&mut schedule, job)?;
//! every(2)
//!    .to(8)?
//!    .seconds()?
//!    .until(Local::now() + chrono::Duration::seconds(30))?
//!    .run(&mut schedule, job)?;
//! #   Ok(())
//! # }
//! ```
//! In your main loop, you can use `Scheduler::run_pending()` to fire all scheduled jobs at the proper time:
//! ```no_run
//! # use skedge::Scheduler;
//! # let mut schedule = Scheduler::new();
//! loop {
//!     if let Err(e) = schedule.run_pending() {
//!         eprintln!("Error: {}", e);
//!     }
//!     std::thread::sleep(std::time::Duration::from_secs(1));
//! }
//! ```

mod callable;
mod error;
mod job;
mod scheduler;
mod time;

use callable::{Callable, UnitToUnit};
pub use job::{every, every_single, Interval, Job, Tag};
pub use scheduler::Scheduler;
use time::{TimeUnit, Timekeeper, Timestamp};
