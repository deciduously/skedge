//! A Job is a piece of work that can be configured and added to the scheduler

use chrono::{prelude::*, Datelike, Duration, Timelike};
use lazy_static::lazy_static;
use log::*;
use rand::prelude::*;
use regex::Regex;
use std::{
    cmp::{Ord, Ordering},
    collections::HashSet,
    convert::TryInto,
    fmt,
};

use crate::{
    error::*, time::RealTime, Callable, Scheduler, TimeUnit, Timekeeper, Timestamp, UnitToUnit,
};

/// A Tag is used to categorize a job.
pub type Tag = String;

/// Each interval value is an unsigned 32-bit integer
pub type Interval = u32;

lazy_static! {
    // Regexes for validating `.at()` strings are only computed once
    static ref DAILY_RE: Regex = Regex::new(r"^([0-2]\d:)?[0-5]\d:[0-5]\d$").unwrap();
    static ref HOURLY_RE: Regex = Regex::new(r"^([0-5]\d)?:[0-5]\d$").unwrap();
    static ref MINUTE_RE: Regex = Regex::new(r"^:[0-5]\d$").unwrap();
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

/// A Job is anything that can be scheduled to run periodically.
///
/// Usually created by the `every` function.
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
    pub(crate) next_run: Option<Timestamp>,
    /// Time delta between runs
    period: Option<Duration>,
    /// Specific day of the week to start on
    start_day: Option<Weekday>,
    /// Optional time of final run
    cancel_after: Option<Timestamp>,
    /// Interface to current time
    clock: Option<Box<dyn Timekeeper>>,
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
            clock: Some(Box::new(RealTime::default())),
        }
    }

    #[cfg(test)]
    /// Build a job with a fake timer
    pub fn with_mock_time(interval: Interval, clock: crate::time::mock::MockTime) -> Self {
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
            clock: Some(Box::new(clock)),
        }
    }

    #[cfg(test)]
    /// Add a duration to the clock
    pub fn add_duration(&mut self, duration: Duration) {
        self.clock.as_mut().unwrap().add_duration(duration);
    }

    /// Helper function to get the current time
    fn now(&self) -> Timestamp {
        // unwrap is safe, there will always be one
        self.clock.as_ref().unwrap().now()
    }

    /// Tag the job with one or more unique identifiers
    pub fn tag(&mut self, tags: &[&str]) {
        for &t in tags {
            let new_tag = t.to_string();
            if !self.tags.contains(&new_tag) {
                self.tags.insert(new_tag);
            }
        }
    }

    /// Check if the job has the given tag
    pub(crate) fn has_tag(&self, tag: &str) -> bool {
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
            return Err(SkedgeError::InvalidUnit);
        }

        // Validate time_str for set time unit
        if (self.unit == Some(Day) || self.start_day.is_some()) && !DAILY_RE.is_match(time_str) {
            return Err(SkedgeError::InvalidDailyAtStr);
        }

        if self.unit == Some(Hour) && !HOURLY_RE.is_match(time_str) {
            return Err(SkedgeError::InvalidHourlyAtStr);
        }

        if self.unit == Some(Minute) && !MINUTE_RE.is_match(time_str) {
            return Err(SkedgeError::InvalidMinuteAtStr);
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
            if hour > 23 {
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
        if latest <= self.interval {
            Err(SkedgeError::InvalidInterval)
        } else {
            self.latest = Some(latest);
            Ok(self)
        }
    }

    /// Schedule job to run until the specified moment.
    ///
    /// The job is canceled whenever the next run is calculated and it turns out the
    /// next run is after the until_time. The job is also canceled right before it runs,
    /// if the current time is after until_time. This latter case can happen when the
    /// the job was scheduled to run before until_time, but runs after until_time.
    /// If until_time is a moment in the past, returns an error.
    ///
    ///
    pub fn until(mut self, until_time: Timestamp) -> Result<Self> {
        if until_time < self.now() {
            return Err(SkedgeError::InvalidUntilTime);
        }
        self.cancel_after = Some(until_time);
        Ok(self)
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
        self.next_run.is_some() && self.now() >= self.next_run.unwrap()
    }

    /// Run this job and immediately reschedule it, returning true.  If job should cancel, return false.
    ///
    /// If the job's deadline has arrived already, the job does not run and returns false.
    ///
    /// If this execution causes the deadline to reach, it will run once and then return false.
    // FIXME: if we support return values from job fns, this fn should return that.
    pub fn execute(&mut self) -> Result<bool> {
        if self.is_overdue(self.now()) {
            debug!("Deadline already reached, cancelling job {}", self);
            return Ok(false);
        }

        debug!("Running job {}", self);
        if self.job.is_none() {
            debug!("No work scheduled, moving on...");
            return Ok(true);
        }
        // FIXME - here's the return value capture
        let _ = self.job.as_ref().unwrap().call();
        self.last_run = Some(self.now());
        self.schedule_next_run()?;

        if self.is_overdue(self.now()) {
            debug!("Execution went over deadline, cancelling job {}", self);
            return Ok(false);
        }

        Ok(true)
    }

    /// Shared logic for setting the job to a particular unit
    fn set_unit_mode(mut self, unit: TimeUnit) -> Result<Self> {
        if let Some(u) = self.unit {
            Err(unit_error(unit, u))
        } else {
            self.unit = Some(unit);
            Ok(self)
        }
    }

    /// Shared logic for setting single-interval units: second(), minute(), etc.
    fn set_single_unit_mode(self, unit: TimeUnit) -> Result<Self> {
        if self.interval != 1 {
            Err(interval_error(unit))
        } else {
            self.set_unit_mode(unit)
        }
    }

    /// Set single second mode
    pub fn second(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Second)
    }

    /// Set seconds mode
    pub fn seconds(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Second)
    }

    /// Set single minute mode
    pub fn minute(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Minute)
    }

    /// Set minutes mode
    pub fn minutes(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Minute)
    }

    /// Set single hour mode
    pub fn hour(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Hour)
    }

    /// Set hours mode
    pub fn hours(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Hour)
    }

    /// Set single day mode
    pub fn day(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Day)
    }

    /// Set days mode
    pub fn days(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Day)
    }

    /// Set single week mode
    pub fn week(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Week)
    }

    /// Set weeks mode
    pub fn weeks(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Week)
    }

    /// Set single month mode
    pub fn month(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Month)
    }

    /// Set months mode
    pub fn months(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Month)
    }

    /// Set single year mode
    pub fn year(self) -> Result<Self> {
        self.set_single_unit_mode(TimeUnit::Year)
    }

    /// Set years mode
    pub fn years(self) -> Result<Self> {
        self.set_unit_mode(TimeUnit::Year)
    }

    /// Set weekly mode on a specific day of the week
    fn set_weekday_mode(mut self, weekday: Weekday) -> Result<Self> {
        if self.interval != 1 {
            Err(weekday_error(weekday))
        } else if let Some(w) = self.start_day {
            Err(weekday_collision_error(weekday, w))
        } else {
            self.start_day = Some(weekday);
            self.weeks()
        }
    }

    /// Set weekly mode on Monday
    pub fn monday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Mon)
    }

    /// Set weekly mode on Tuesday
    pub fn tuesday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Tue)
    }

    /// Set weekly mode on Wednesday
    pub fn wednesday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Wed)
    }

    /// Set weekly mode on Thursday
    pub fn thursday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Thu)
    }

    /// Set weekly mode on Friday
    pub fn friday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Fri)
    }

    /// Set weekly mode on Saturday
    pub fn saturday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Sat)
    }

    /// Set weekly mode on Sunday
    pub fn sunday(self) -> Result<Self> {
        self.set_weekday_mode(Weekday::Sun)
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
        let period = self.unit.unwrap().duration(interval);
        self.period = Some(period);
        self.next_run = Some(self.now() + period);

        // Handle start day for weekly jobs
        if let Some(w) = self.start_day {
            // This only makes sense for weekly jobs
            if self.unit != Some(TimeUnit::Week) {
                return Err(SkedgeError::StartDayError);
            }

            let weekday_num = w.num_days_from_monday();
            let mut days_ahead = weekday_num as i64
                - self
                    .next_run
                    .unwrap()
                    .date()
                    .weekday()
                    .num_days_from_monday() as i64;

            // Check if the weekday already happened this week, advance a week if so
            if days_ahead <= 0 {
                days_ahead += 7;
            }

            self.next_run = Some(
                self.next_run.unwrap() + TimeUnit::Day.duration(days_ahead.try_into().unwrap())
                    - self.period.unwrap(),
            )
        }

        // Handle specified at_time
        if let Some(at_t) = self.at_time {
            use TimeUnit::*;
            // Validate configuration
            if ![Some(Day), Some(Hour), Some(Minute)].contains(&self.unit)
                && self.start_day.is_none()
            {
                return Err(SkedgeError::UnspecifiedStartDay);
            }

            // Update next_run appropriately
            let next_run = self.next_run.unwrap();
            let second = at_t.second();
            let hour = if self.unit == Some(Day) || self.start_day.is_some() {
                at_t.hour()
            } else {
                next_run.hour()
            };
            let minute = if [Some(Day), Some(Hour)].contains(&self.unit) || self.start_day.is_some()
            {
                at_t.minute()
            } else {
                next_run.minute()
            };
            let naive_time = NaiveTime::from_hms(hour, minute, second);
            let date = next_run.date();
            self.next_run = Some(date.and_time(naive_time).unwrap());

            // Make sure job gets run TODAY or THIS HOUR
            // Accounting for jobs take long enough that they finish in the next period
            if self.last_run.is_none()
                || (self.next_run.unwrap() - self.last_run.unwrap()) > self.period.unwrap()
            {
                let now = self.now();
                if self.unit == Some(Day)
                    && self.at_time.unwrap() > now.time()
                    && self.interval == 1
                {
                    self.next_run = Some(self.next_run.unwrap() - Day.duration(1));
                } else if self.unit == Some(Hour)
                    && (self.at_time.unwrap().minute() > now.minute()
                        || self.at_time.unwrap().minute() == now.minute()
                            && self.at_time.unwrap().second() > now.second())
                {
                    self.next_run = Some(self.next_run.unwrap() - Hour.duration(1));
                } else if self.unit == Some(Minute) && self.at_time.unwrap().second() > now.second()
                {
                    self.next_run = Some(self.next_run.unwrap() - Minute.duration(1));
                }
            }
        }

        // Check if at_time on given day should fire today or next week
        if self.start_day.is_some() && self.at_time.is_some() {
            // unwraps are safe, we already set them in this function
            let next = self.next_run.unwrap(); // safe, we already set it
            if (next - self.now()).num_days() >= 7 {
                self.next_run = Some(next - self.period.unwrap());
            }
        }

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
            "Job(interval={}, unit={:?}, run={})",
            self.interval, self.unit, name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_plural_time_units() -> Result<()> {
        use TimeUnit::*;
        assert_eq!(every(2).seconds()?.unit, Some(Second));
        assert_eq!(every(2).minutes()?.unit, Some(Minute));
        assert_eq!(every(2).hours()?.unit, Some(Hour));
        assert_eq!(every(2).days()?.unit, Some(Day));
        assert_eq!(every(2).weeks()?.unit, Some(Week));
        assert_eq!(every(2).months()?.unit, Some(Month));
        assert_eq!(every(2).years()?.unit, Some(Year));
        // Okay to use plural method with singular interval:
        assert_eq!(every(1).seconds()?.unit, Some(Second));
        assert_eq!(every(1).minutes()?.unit, Some(Minute));
        assert_eq!(every(1).hours()?.unit, Some(Hour));
        assert_eq!(every(1).days()?.unit, Some(Day));
        assert_eq!(every(1).weeks()?.unit, Some(Week));
        assert_eq!(every(1).months()?.unit, Some(Month));
        assert_eq!(every(1).years()?.unit, Some(Year));
        Ok(())
    }

    #[test]
    fn test_singular_time_units() -> Result<()> {
        use TimeUnit::*;
        assert_eq!(every(1), every_single());
        assert_eq!(every_single().second()?.unit, Some(Second));
        assert_eq!(every_single().minute()?.unit, Some(Minute));
        assert_eq!(every_single().hour()?.unit, Some(Hour));
        assert_eq!(every_single().day()?.unit, Some(Day));
        assert_eq!(every_single().week()?.unit, Some(Week));
        assert_eq!(every_single().month()?.unit, Some(Month));
        assert_eq!(every_single().year()?.unit, Some(Year));
        Ok(())
    }

    #[test]
    fn test_singular_unit_plural_interval_mismatch() {
        assert_eq!(
            every(2).second().unwrap_err().to_string(),
            "Use seconds() instead of second()".to_string()
        );
        assert_eq!(
            every(2).minute().unwrap_err().to_string(),
            "Use minutes() instead of minute()".to_string()
        );
        assert_eq!(
            every(2).hour().unwrap_err().to_string(),
            "Use hours() instead of hour()".to_string()
        );
        assert_eq!(
            every(2).day().unwrap_err().to_string(),
            "Use days() instead of day()".to_string()
        );
        assert_eq!(
            every(2).week().unwrap_err().to_string(),
            "Use weeks() instead of week()".to_string()
        );
        assert_eq!(
            every(2).month().unwrap_err().to_string(),
            "Use months() instead of month()".to_string()
        );
        assert_eq!(
            every(2).year().unwrap_err().to_string(),
            "Use years() instead of year()".to_string()
        );
    }

    #[test]
    fn test_singular_units_match_plural_units() -> Result<()> {
        assert_eq!(every(1).second()?.unit, every(1).seconds()?.unit);
        assert_eq!(every(1).minute()?.unit, every(1).minutes()?.unit);
        assert_eq!(every(1).hour()?.unit, every(1).hours()?.unit);
        assert_eq!(every(1).day()?.unit, every(1).days()?.unit);
        assert_eq!(every(1).week()?.unit, every(1).weeks()?.unit);
        assert_eq!(every(1).month()?.unit, every(1).months()?.unit);
        assert_eq!(every(1).year()?.unit, every(1).years()?.unit);
        Ok(())
    }

    #[test]
    fn test_reject_weekday_multiple_weeks() {
        assert_eq!(
            every(2).monday().unwrap_err().to_string(),
            "Scheduling jobs on Mon is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
        assert_eq!(
            every(2).tuesday().unwrap_err().to_string(),
            "Scheduling jobs on Tue is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
        assert_eq!(
            every(2).wednesday().unwrap_err().to_string(),
            "Scheduling jobs on Wed is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
        assert_eq!(
            every(2).thursday().unwrap_err().to_string(),
            "Scheduling jobs on Thu is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
        assert_eq!(
            every(2).friday().unwrap_err().to_string(),
            "Scheduling jobs on Fri is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
        assert_eq!(
            every(2).saturday().unwrap_err().to_string(),
            "Scheduling jobs on Sat is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
        assert_eq!(
            every(2).sunday().unwrap_err().to_string(),
            "Scheduling jobs on Sun is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported".to_string()
        );
    }

    #[test]
    fn test_reject_start_day_unless_weekly() -> Result<()> {
        let mut job = every_single();
        let expected = "Attempted to use a start day for a unit other than `weeks`".to_string();
        job.unit = Some(TimeUnit::Day);
        job.start_day = Some(Weekday::Wed);
        assert_eq!(job.schedule_next_run().unwrap_err().to_string(), expected);
        Ok(())
    }

    #[test]
    fn test_reject_multiple_time_units() -> Result<()> {
        assert_eq!(
            every_single().day()?.wednesday().unwrap_err().to_string(),
            "Cannot set weeks mode, already using days".to_string()
        );
        assert_eq!(
            every_single().minute()?.second().unwrap_err().to_string(),
            "Cannot set seconds mode, already using minutes".to_string()
        );
        // TODO etc...
        Ok(())
    }

    #[test]
    fn test_reject_invalid_at_time() -> Result<()> {
        let bad_hour = "Invalid hour (25 is not between 0 and 23)".to_string();
        let bad_daily =
            "Invalid time format for daily job (valid format is HH:MM(:SS)?)".to_string();
        let bad_hourly =
            "Invalid time format for hourly job (valid format is (MM)?:SS)".to_string();
        let bad_minutely = "Invalid time format for minutely job (valid format is :SS)".to_string();
        let bad_unit = "Invalid unit (valid units are `days`, `hours`, and `minutes`)".to_string();
        assert_eq!(
            every_single()
                .second()?
                .at("13:15")
                .unwrap_err()
                .to_string(),
            bad_unit
        );
        assert_eq!(
            every_single()
                .day()?
                .at("25:00:00")
                .unwrap_err()
                .to_string(),
            bad_hour
        );
        assert_eq!(
            every_single()
                .day()?
                .at("00:61:00")
                .unwrap_err()
                .to_string(),
            bad_daily
        );
        assert_eq!(
            every_single()
                .day()?
                .at("00:00:61")
                .unwrap_err()
                .to_string(),
            bad_daily
        );
        assert_eq!(
            every_single()
                .day()?
                .at("00:61:00")
                .unwrap_err()
                .to_string(),
            bad_daily
        );
        assert_eq!(
            every_single().day()?.at("25:0:0").unwrap_err().to_string(),
            bad_daily
        );
        assert_eq!(
            every_single().day()?.at("0:61:0").unwrap_err().to_string(),
            bad_daily
        );
        assert_eq!(
            every_single().day()?.at("0:0:61").unwrap_err().to_string(),
            bad_daily
        );
        assert_eq!(
            every_single()
                .hour()?
                .at("23:59:29")
                .unwrap_err()
                .to_string(),
            bad_hourly
        );
        assert_eq!(
            every_single().hour()?.at("61:00").unwrap_err().to_string(),
            bad_hourly
        );
        assert_eq!(
            every_single().hour()?.at("00:61").unwrap_err().to_string(),
            bad_hourly
        );
        assert_eq!(
            every_single().hour()?.at(":61").unwrap_err().to_string(),
            bad_hourly
        );
        assert_eq!(
            every_single()
                .minute()?
                .at("22:45:34")
                .unwrap_err()
                .to_string(),
            bad_minutely
        );
        assert_eq!(
            every_single().minute()?.at(":61").unwrap_err().to_string(),
            bad_minutely
        );
        Ok(())
    }

    #[test]
    fn test_latest_greater_than_interval() {
        assert_eq!(
            every(2).to(1).unwrap_err().to_string(),
            "Latest val is greater than interval val".to_string()
        );
        assert_eq!(every(2).to(3).unwrap().latest, Some(3))
    }
}
