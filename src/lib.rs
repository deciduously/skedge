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
//!
//!     // loop {
//!     //     schedule.run_pending();
//!     //     sleep(Duration::from_secs(1));
//!     // }
//!     Ok(())
//! }
//! ```

use chrono::{prelude::*, Duration};
use lazy_static::lazy_static;
use log::*;
use rand::prelude::*;
use regex::Regex;
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

/// A Job is anything that can be scheduled to run periodically.
///
/// Usually created by the `Scheduler#every` method.
#[derive(Debug, PartialEq, Eq)]
pub struct Job {
    /// A quantity of a given time unit
    interval: Interval, // pause interval * unit between runs
    /// Upper limit to interval for randomized job timing
    latest: Option<Interval>,
    /// The actual function to execute
    job: Option<Box<dyn Callable>>,
    /// Tags used to group jobs
    tags: HashSet<Tag>,
    /// Unit of time described by intervals
    unit: Option<TimeUnit>,
    /// Optional set time at which this job runs
    at_time: Option<NaiveTime>,
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
            latest: None,
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

    /// Specify a particular concrete time to run the job.
    ///
    /// * Daily jobs: `HH:MM:SS` or `HH:MM`
    ///
    /// * Hourly jobs: `MM:SS` or `:MM`
    ///
    /// * Minute jobs: `:SS`
    ///
    /// Not supported on weekly, monthly, or yearly jobs.
    pub fn at(mut self, time_str: &str) -> Result<Self> {
        use TimeUnit::*;

        // Validate time unit
        if ![Week, Day, Hour, Minute].contains(&self.unit.unwrap_or(Year)) {
            dbg!(self.unit);
            return Err(SkedgeError::InvalidUnit);
        }

        // Validate time_str for set time unit
        lazy_static! {
            static ref DAILY_RE: Regex = Regex::new(r"^([0-2]\d:)?[0-5]\d:[0-5]\d$").unwrap();
            static ref HOURLY_RE: Regex = Regex::new(r"^([0-5]\d)?:[0-5]\d$").unwrap();
            static ref MINUTE_RE: Regex = Regex::new(r"^:[0-5]\d$").unwrap();
        }

        if self.unit == Some(Day) || self.start_day.is_some() {
            if !DAILY_RE.is_match(time_str) {
                return Err(SkedgeError::InvalidDailyAtStr);
            }
        }

        if self.unit == Some(Hour) {
            if !HOURLY_RE.is_match(time_str) {
                return Err(SkedgeError::InvalidHourlyAtStr);
            }
        }

        if self.unit == Some(Minute) {
            if !MINUTE_RE.is_match(time_str) {
                return Err(SkedgeError::InvalidMinuteAtStr);
            }
        }

        // Parse time_str and store timestamp
        let time_vals = time_str.split(':').collect::<Vec<&str>>();
        let mut hour = 0;
        let mut minute = 0;
        let mut second = 0;
        // ALl unwraps are safe - already validated by regex
        let num_vals = time_vals.len();
        if num_vals == 3 {
            hour = time_vals[0].parse().unwrap();
            minute = time_vals[1].parse().unwrap();
            second = time_vals[2].parse().unwrap();
        } else if num_vals == 2 && self.unit == Some(Minute) {
            second = time_vals[1].parse().unwrap();
        } else if num_vals == 2 && self.unit == Some(Hour) {
            minute = time_vals[0].parse().unwrap();
            second = time_vals[1].parse().unwrap();
        } else {
            hour = time_vals[0].parse().unwrap();
            minute = time_vals[0].parse().unwrap();
        }

        if self.unit == Some(Day) || self.start_day.is_some() {
            if !hour <= 23 {
                return Err(invalid_hour_error(hour));
            }
        } else if self.unit == Some(Hour) {
            hour = 0;
        } else if self.unit == Some(Minute) {
            hour = 0;
            minute = 0;
        }

        // Store timestamp and return
        self.at_time = Some(NaiveTime::from_hms(hour, minute, second));
        Ok(self)
    }

    /// Schedule the job to run at a regular randomized interval.
    ///
    /// E.g. every(3).to(6).seconds
    pub fn to(mut self, latest: Interval) -> Result<Self> {
        self.latest = Some(latest);
        Ok(self)
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
    pub fn run(mut self, scheduler: &mut Scheduler, job: fn() -> ()) -> Result<()> {
        // FIXME how does job naming work?  without reflection?
        self.job = Some(Box::new(UnitToUnit::new("job", job)));
        self.schedule_next_run()?;
        scheduler.add_job(self);
        Ok(())
    }

    /// Check whether this job should be run now
    pub fn should_run(&self) -> bool {
        self.next_run.is_some() && Local::now() >= self.next_run.unwrap()
    }

    /// Run this job and immediately reschedule it, returning true.  If job should cancel, return false.
    ///
    /// If the job's deadline has arrived already, the job does not run and returns false.
    ///
    /// If this execution causes the deadline to reach, it will run once and then return false.
    // FIXME: if we support return values from job fns, this fn should return that.
    pub fn execute(&mut self) -> Result<bool> {
        if self.is_overdue(Local::now()) {
            debug!("Deadline already reached, cancelling job {}", self);
            return Ok(false);
        }

        debug!("Running job {}", self);
        if self.job.is_none() {
            debug!("No work scheduled, moving on...");
            return Ok(true);
        }
        self.job.as_ref().unwrap().call();
        self.last_run = Some(Local::now());
        self.schedule_next_run()?;

        if self.is_overdue(Local::now()) {
            debug!("Execution went over deadline, cancelling job {}", self);
            return Ok(false);
        }

        Ok(true)
    }

    // TODO - the below can probably be refactored, lots of repeated code.

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
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Set weekly mode on Tuesday
    pub fn tuesday(mut self) -> Result<Self> {
        let day = Weekday::Tue;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Set weekly mode on Wednesday
    pub fn wednesday(mut self) -> Result<Self> {
        let day = Weekday::Wed;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Set weekly mode on Thursday
    pub fn thursday(mut self) -> Result<Self> {
        let day = Weekday::Thu;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Set weekly mode on Friday
    pub fn friday(mut self) -> Result<Self> {
        let day = Weekday::Fri;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Set weekly mode on Saturday
    pub fn saturday(mut self) -> Result<Self> {
        let day = Weekday::Sat;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Set weekly mode on Sunday
    pub fn sunday(mut self) -> Result<Self> {
        let day = Weekday::Sun;
        if self.interval != 1 {
            Err(weekday_error(day))
        } else {
            if let Some(w) = self.start_day {
                Err(weekday_collision_error(day, w))
            } else {
                self.start_day = Some(day);
                self.weeks()
            }
        }
    }

    /// Compute the timestamp for the next run
    fn schedule_next_run(&mut self) -> Result<()> {
        // If "latest" is set, find the actual interval for this run, otherwise just used stored val
        let interval = match self.latest {
            Some(v) => {
                if v < self.interval {
                    return Err(SkedgeError::InvalidInterval);
                }
                thread_rng().gen_range(self.interval..v)
            }
            None => self.interval,
        };

        // Calculate period (Duration)
        let period = self.unit.unwrap().duration(self.interval);
        self.period = Some(period);
        self.next_run = Some(Local::now() + period);

        if self.start_day.is_some() {}

        if self.at_time.is_some() {}

        Ok(())
    }

    /// Check if given time is after the cancel_after time
    fn is_overdue(&self, when: Timestamp) -> bool {
        self.cancel_after.is_some() && when > self.cancel_after.unwrap()
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

    /// Add a new job to the list
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    /// Run all jobs that are scheduled to run.  Does NOT run missed jobs!
    pub fn run_pending(&mut self) {
        //let mut jobs_to_run: Vec<&Job> = self.jobs.iter().filter(|el| el.should_run()).collect();
        self.jobs.sort();
        let mut to_remove = Vec::new();
        for (idx, job) in self.jobs.iter_mut().enumerate() {
            if job.should_run() {
                let keep_going = job.execute().unwrap();
                if !keep_going {
                    debug!("Cancelling job {}", job);
                    to_remove.push(idx);
                }
            }
        }
        // Remove any cancelled jobs
        to_remove.sort_unstable();
        to_remove.reverse();
        for &idx in &to_remove {
            self.jobs.remove(idx);
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

    /// Property getter - number of seconds until next run.  None if no jobs scheduled
    fn idle_seconds(&self) -> Option<u32> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    // TODO: add unit tests!
}
