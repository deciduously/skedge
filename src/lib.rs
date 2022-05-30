//! # skedge
//!
//! `skedge` is a single-process job scheduler.
//! To use the optional CFFI, enable the "ffi" feature.
//!
//! Define a work function:
//! ```rust
//! fn job() {
//!     println!("Hello, it's {}!", chrono::Local::now().to_rfc2822());
//! }
//! ```
//! You can use up to six arguments:
//! ```rust
//! fn greet(name: &str) {
//!     println!("Hello, {}!", name);
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
//! # fn greet(name: &str) {
//! #     println!("Hello, {}!", name);
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
//!     .to(8)?
//!     .seconds()?
//!     .until(Local::now() + chrono::Duration::seconds(30))?
//!     .run_one_arg(&mut schedule, greet, "Good-Looking")?;
//! #   Ok(())
//! # }
//! ```
//! Note that you must use the appropriate `run_x_args()` method for job functions taking multiple arguments.
//! In your main loop, you can use `Scheduler::run_pending()` to fire all scheduled jobs at the proper time:
//! ```no_run
//! # use skedge::Scheduler;
//! # let mut schedule = Scheduler::new();
//! loop {
//!     if let Err(e) = schedule.run_pending() {
//!         eprintln!("Error: {e}");
//!     }
//!     std::thread::sleep(std::time::Duration::from_secs(1));
//! }
//! ```

#![warn(clippy::pedantic)]

mod callable;
mod error;
mod job;
mod scheduler;
mod time;

use callable::{
	Callable, FiveToUnit, FourToUnit, OneToUnit, SixToUnit, ThreeToUnit, TwoToUnit, UnitToUnit,
};
pub use error::*;
pub use job::{every, every_single, Interval, Job, Tag};
pub use scheduler::Scheduler;
use time::{Real, Timekeeper, Timestamp, Unit};

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(feature = "ffi")]
pub use ffi::*;
