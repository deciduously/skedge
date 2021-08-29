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
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    let mut schedule = Scheduler::new();
//!
//!     every(10).seconds()?.run(&mut schedule, job);
//!     every(10).minutes()?.run(&mut schedule, job);
//!     every_single().hour()?.run(&mut schedule, job);
//!     every_single().day()?.at("10:30")?.run(&mut schedule, job);
//!     every(5).to(10)?.minutes()?.run(&mut schedule, job);
//!     every_single().monday()?.run(&mut schedule, job);
//!     every_single().wednesday()?.at("13:15")?.run(&mut schedule, job);
//!     every_single().minute()?.at(":17")?.run(&mut schedule, job);
//!
//!     loop {
//!         schedule.run_pending();
//!         sleep(Duration::from_secs(1));
//!     }
//! }
//! ```

use chrono::{prelude::*, Duration};
use log::*;
use std::{
    cmp::{Ord, Ordering},
    collections::HashSet,
    fmt,
};

mod error;
use error::*;

/// Each interval value is an unsigned 32-bit integer
type Interval = u32;

/// Timestamps are in the users local timezone
type Timestamp = DateTime<Local>;

/// A job is anything that implements this trait
// FIXME: This doesn't work yet
pub trait Callable: fmt::Debug {
    /// Execute this callable
    fn call(&self);
    /// Get the name of this callable
    fn name(&self) -> &str;
}

impl PartialEq for dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        // Callable objects are equal if their names are equal
        // FIXME: this seems fishy
        self.name() == other.name()
    }
}

impl Eq for dyn Callable {}

/// A named callable function taking no parameters and returning nothing.
#[derive(Debug)]
pub struct UnitToUnit {
    name: String,
    work: fn() -> (),
}

impl UnitToUnit {
    pub fn new(name: &str, work: fn() -> ()) -> Self {
        Self {
            name: name.into(),
            work,
        }
    }
}

impl Callable for UnitToUnit {
    fn call(&self) {
        (self.work)();
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A Tag is used to categorize a job.
type Tag = String;

/// Jobs can be periodic over one of these units of time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
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

/// A Job is anything that can be scheduled to run periodically.
///
/// Usually created by the `Scheduler#every` method.
#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    /// A quantity of a given time unit
    interval: Interval, // pause interval * unit between runs
    /// The actual function to execute
    job: Option<Box<dyn Callable>>,
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
            job: None,
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
        for t in tags {
            let new_tag = t.into();
            if !self.tags.contains(&new_tag) {
                self.tags.insert(new_tag);
            }
        }
    }

    /// Check if the job has the given tag
    fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Specify a particular concrete time to run the job
    // FIXME: should this be impl Into<DateTime<Utc>>?
    pub fn at(&mut self, time_str: &str) -> Result<Self> {
        unimplemented!()
    }

    /// Schedule the job to run at a regular randomized interval.
    ///
    /// E.g. every(3).to(6).seconds
    pub fn to(&mut self, latest: Interval) -> Result<Self> {
        unimplemented!()
    }

    /// Schedule job to run until the specified moment.
    ///
    /// The job is canceled whenever the next run is calculated and it turns out the
    /// next run is after the until_time. The job is also canceled right before it runs,
    /// if the current time is after until_time. This latter case can happen when the
    /// the job was scheduled to run before until_time, but runs after until_time.
    /// If until_time is a moment in the past, we should get a ScheduleValueError.
    pub fn until(&mut self, until_time: impl Into<Timestamp>) -> Result<Self> {
        unimplemented!()
    }

    /// Specify the work function that will execute when this job runs and add it to the schedule
    pub fn run(mut self, scheduler: &mut Scheduler, job: fn() -> ()) -> Result<Self> {
        // FIXME how does job naming work?  without reflection?
        self.job = Some(Box::new(UnitToUnit::new("job", job)));
        unimplemented!()
    }

    /// Check whether this job should be run now
    pub fn should_run(&self) -> bool {
        unimplemented!()
    }

    /// Run this job and immediately reschedule it, returning true.  If job should cancel, return false
    pub fn execute(&self) -> bool {
        unimplemented!()
    }

    /// Set single second mode
    pub fn second(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Second))
        } else {
            self.seconds()
        }
    }

    /// Set seconds mode
    pub fn seconds(mut self) -> Result<Self> {
        let unit = TimeUnit::Second;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set single minute mode
    pub fn minute(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Minute))
        } else {
            self.minutes()
        }
    }

    /// Set minutes mode
    pub fn minutes(mut self) -> Result<Self> {
        let unit = TimeUnit::Minute;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set single hour mode
    pub fn hour(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Hour))
        } else {
            self.hours()
        }
    }

    /// Set hours mode
    pub fn hours(mut self) -> Result<Self> {
        let unit = TimeUnit::Hour;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set single day mode
    pub fn day(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Day))
        } else {
            self.days()
        }
    }

    /// Set days mode
    pub fn days(mut self) -> Result<Self> {
        let unit = TimeUnit::Day;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set single week mode
    pub fn week(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Week))
        } else {
            self.weeks()
        }
    }

    /// Set weeks mode
    pub fn weeks(mut self) -> Result<Self> {
        let unit = TimeUnit::Week;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set single month mode
    pub fn month(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Month))
        } else {
            self.months()
        }
    }

    /// Set months mode
    pub fn months(mut self) -> Result<Self> {
        let unit = TimeUnit::Month;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set single year mode
    pub fn year(self) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(TimeUnit::Year))
        } else {
            self.years()
        }
    }

    /// Set years mode
    pub fn years(mut self) -> Result<Self> {
        let unit = TimeUnit::Year;
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Set weekly mode on Monday
    pub fn monday(mut self) -> Result<Self> {
        let day = Weekday::Mon;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
    }

    /// Set weekly mode on Tuesday
    pub fn tuesday(mut self) -> Result<Self> {
        let day = Weekday::Tue;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
    }

    /// Set weekly mode on Wednesday
    pub fn wednesday(mut self) -> Result<Self> {
        let day = Weekday::Wed;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
    }

    /// Set weekly mode on Thursday
    pub fn thursday(mut self) -> Result<Self> {
        let day = Weekday::Thu;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
    }

    /// Set weekly mode on Friday
    pub fn friday(mut self) -> Result<Self> {
        let day = Weekday::Fri;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
    }

    /// Set weekly mode on Saturday
    pub fn saturday(mut self) -> Result<Self> {
        let day = Weekday::Sat;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
    }

    /// Set weekly mode on Sunday
    pub fn sunday(mut self) -> Result<Self> {
        let day = Weekday::Sun;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            self.start_day = Some(day);
            self.weeks()
        }
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

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = if self.job.is_none() {
            "No Job"
        } else {
            let j = self.job.as_ref().unwrap();
            j.name()
        };
        write!(
            f,
            "Job(interval={}, unit={:?}, run={}",
            self.interval, self.unit, name
        )
    }
}

/// Convenience function wrapping the Job constructor.
///
/// E.g.: `every(10).seconds()?.run(&schedule, job)`;
#[inline]
pub fn every(interval: Interval) -> Job {
    Job::new(interval)
}

/// Convenience function wrapping the Job constructor with a default of 1.
///
/// Equivalent to `every(1)`.
#[inline]
pub fn every_single() -> Job {
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
        pretty_env_logger::init(); //FIXME hmmm, probably not here?

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

    /// Run given job.
    fn run_job(&self, job: &Job) {
        let keep_going = job.execute();
        if !keep_going {
            self.cancel_job(job);
        }
    }

    /// Property getter - number of seconds until next run.  None if no jobs scheduled
    fn idle_seconds(&self) -> Option<u32> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    // TODO: add unit tests!
}
