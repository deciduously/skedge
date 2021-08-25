//! # skedge
//!
//! `skedge` is a single-process job scheduler.

use chrono::{prelude::*, Duration};
use std::{cmp::Ordering, collections::HashSet};
use thiserror::Error;

/// Each interval value is an unsigned 32-bit integer
type Interval = u32;

/// Timestamps are stored in UTC
type Timestamp = DateTime<Utc>;

/// A Job is a function with no parameters, returning nothing.
// FIXME: how to support more options?
type JobFn = fn() -> ();

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

/// A Tag is used to categorize a job.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Tag(String);

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Jobs can be periodic over one of these units of time
#[derive(Debug, Clone, Copy, PartialEq)]
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
/// Usually created by the `Scheduler::every`.
// NOTE - the python one holds a reference to the scheduler, this sounds problematic in Rust...
#[derive(Debug, PartialEq)]
pub(crate) struct Job {
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

    /// Specify the work function that will execute when this job runs
    pub fn run(self, job_fn: JobFn) -> Self {
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
        // Sorting is based on the next scheduled run
        Some(self.next_run.cmp(&other.next_run))
    }
}

/// A Scheduler creates jobs, tracks recorded jobs, and executes jobs.
pub struct Scheduler {
    /// The currently scheduled lob list
    jobs: Vec<Job>,
}

impl Scheduler {
    /// Instantiate a Scheduler
    pub fn new() -> Self {
        Self::default()
    }

    /// Run all jobs that are scheduled to run.  Does NOT run missed jobs!
    pub fn run_pending() {
        unimplemented!()
    }

    /// Run all jobs, regardless of schedule.
    fn run_all(&self, delay_seconds: Option<u32>) {
        // if None, default to 0.
        unimplemented!()
    }

    /// Get all jobs, optionally with a given tag.
    fn get_jobs<'a>(&self, tag: Option<Tag>) -> &'a [Job] {
        unimplemented!()
    }

    /// Clear all jobs, optionally only with given tag.
    fn clear(&self, tag: Option<Tag>) {
        unimplemented!()
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

    /// Private fn to run given job.
    fn run_job(&self, job: Job) {
        unimplemented!()
    }

    /// Property getter - number of seconds until next run.  None if no jobs scheduled
    fn idle_seconds(&self) -> Option<u32> {
        unimplemented!()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self { jobs: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
