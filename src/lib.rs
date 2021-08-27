//! # skedge
//!
//! `skedge` is a single-process job scheduler.
//!
//! ```rust
//! use skedge::{Scheduler, every, every_single};
//! use std::time::Duration;
//! use std::thread::sleep;
//!
//! fn job() {
//!     println!("Hello!");
//! }
//!
//! fn main() {
//!    let mut schedule = Scheduler::new();
//!
//!     every(10).seconds().run(job, &schedule);
//!     every(10).minutes().run(job, &schedule);
//!     every_single().hour().run(job, &schedule);
//!     every_single().day().at("10:30").run(job, &schedule);
//!     every(5).to(10).minutes().run(job, &schedule);
//!     every_single().monday().run(job, &schedule);
//!     every_single().wednesday().at("13:15").run(job, &schedule);
//!     every_single().minute().at(":17").run(job, &schedule);
//!
//!     loop {
//!         schedule.run_pending();
//!         sleep(Duration::from_secs(1));
//!     }
//! }
//! ```

use chrono::{Duration, prelude::*};
use log::*;
use std::{
    cmp::{Ord, Ordering},
    collections::HashSet,
};
use thiserror::Error;

/// Each interval value is an unsigned 32-bit integer
type Interval = u32;

/// Timestamps are stored in UTC
type Timestamp = DateTime<Utc>;

/// A Job is a function with no parameters, returning nothing.
// FIXME: how to support more options?  This is just to get it wired up.
type JobFn = fn() -> ();

/// A Tag is used to categorize a job.
type Tag = String;

// FIXME - this is probably not right
#[derive(Debug, Error)]
enum SkedgeError {
    #[error("Basic error")]
    ScheduleError,
    #[error("Value error")]
    ScheduleValueError,
    #[error("An improper interval was used")]
    IntervalError,
}

/// Jobs can be periodic over one of these units of time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

/// A Job is anything that can be scheduled to run periodically.
///
/// Usually created by the `Scheduler#every` method.
// NOTE - the python one holds a reference to the scheduler, this sounds problematic in Rust...
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// A quantity of a given time unit
    interval: Interval, // pause interval * unit between runs
    /// The actual function to execute
    job_fn: Option<JobFn>,
    /// Tags used to group jobs
    tags: HashSet<Tag>,
    /// Unit of time described by intervals
    unit: Option<TimeUnit>,
    /// Optional set time at which this job runs
    at_time: Option<Timestamp>,
    /// Timestamp of last run
    last_run: Option<Timestamp>,
    /// Timestamp of next run
    next_run: Option<Timestamp>,
    /// Time delta between runs
    period: Option<Duration>,
    /// Specific day of the week to start on
    start_day: Option<Weekday>,
    /// Optional time of final run
    cancel_after: Option<Timestamp>,
}

impl Job {
    pub fn new(interval: Interval) -> Self {
        Self {
            interval,
            job_fn: None,
            tags: HashSet::new(),
            unit: None,
            at_time: None,
            last_run: None,
            next_run: None,
            period: None,
            start_day: None,
            cancel_after: None,
        }
    }

    /// Tag the job with one or more unique identifiers
    pub fn tag(&mut self, tags: Vec<impl Into<Tag>>) {
        unimplemented!()
    }

    /// Check if the job has the given tag
    fn has_tag(&self, tag: &str) -> bool {
        unimplemented!()
    }

    /// Specify a particular concrete time to run the job
    // FIXME: should this be impl Into<DateTime<Utc>>?
    pub fn at(&mut self, time_str: &str) {
        unimplemented!()
    }

    /// Schedule the job to run at a regular randomized interval.
    ///
    /// E.g. every(3).to(6).seconds
    pub fn to(&mut self, latest: Interval) {
        unimplemented!()
    }

    /// Schedule job to run until the specified moment.
    ///
    /// The job is canceled whenever the next run is calculated and it turns out the
    /// next run is after the until_time. The job is also canceled right before it runs,
    /// if the current time is after until_time. This latter case can happen when the
    /// the job was scheduled to run before until_time, but runs after until_time.
    /// If until_time is a moment in the past, we should get a ScheduleValueError.
    pub fn until(&mut self, until_time: impl Into<Timestamp>) {
        unimplemented!()
    }

    /// Specify the work function that will execute when this job runs and add it to the schedule
    pub fn run(self, job_fn: JobFn, scheduler: &mut Scheduler) -> Self {
        unimplemented!()
    }

    /// Check whether this job should be run now
    pub fn should_run(&self) -> bool {
        unimplemented!()
    }

    /// Run this job and immediately reschedule it
    pub fn execute(&self) {
        unimplemented!()
    }

    /// Compute the timestamp for the next run
    fn schedule_next_run(&mut self) {
        unimplemented!()
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sorting is based on the next scheduled run
        self.next_run.cmp(&other.next_run)
    }
}

/// Convenience function wrapping the Job constructor
#[inline] pub fn every(interval: Interval) -> Job {
    Job::new(interval)
}

/// Convenience function wrapping the Job constructor with a default of 1
#[inline] pub fn every_single() -> Job {
    Job::new(1)
}

/// A Scheduler creates jobs, tracks recorded jobs, and executes jobs.
#[derive(Debug, Default)]
pub struct Scheduler {
    /// The currently scheduled lob list
    jobs: Vec<Job>,
}

impl Scheduler {
    /// Instantiate a Scheduler
    pub fn new() -> Self {
        pretty_env_logger::init(); //hmmm, probably not here?

        Self::default()
    }

    /// Run all jobs that are scheduled to run.  Does NOT run missed jobs!
    pub fn run_pending(&self) {
        let mut jobs_to_run: Vec<&Job> = self.jobs.iter().filter(|el| el.should_run()).collect();
        jobs_to_run.sort();
        for job in &jobs_to_run {
            self.run_job(job);
        }
    }

    /// Run all jobs, regardless of schedule.
    fn run_all(&self, delay_seconds: u32) {
        debug!(
            "Running all {} jobs with {}s delay",
            self.jobs.len(),
            delay_seconds
        );
        for job in &self.jobs {}
    }

    /// Get all jobs, optionally with a given tag.
    fn get_jobs(&self, tag: Option<Tag>) -> Vec<&Job> {
        if let Some(t) = tag {
            self.jobs
                .iter()
                .filter(|el| el.has_tag(&t))
                .collect::<Vec<&Job>>()
        } else {
            self.jobs.iter().collect::<Vec<&Job>>()
        }
    }

    /// Clear all jobs, optionally only with given tag.
    fn clear(&mut self, tag: Option<Tag>) {
        if let Some(t) = tag {
            debug!("Deleting all jobs tagged {}", t);
            self.jobs.retain(|el| !el.has_tag(&t));
        } else {
            debug!("Deleting ALL jobs!!");
            let _ = self.jobs.drain(..);
        }
    }

    fn cancel_job(&self, job: &Job) {
        // FIXME: What should "job" actually be?
        unimplemented!()
    }

    /// Schedule a new periodic Job
    fn every(&self, interval: Interval) -> Job {
        // NOTE - may need a separate fn that doesn't take an argument, defaulting to 1.
        unimplemented!()
    }

    /// Run given job.
    fn run_job(&self, job: &Job) {
        unimplemented!()
    }

    /// Property getter - number of seconds until next run.  None if no jobs scheduled
    fn idle_seconds(&self) -> Option<u32> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
