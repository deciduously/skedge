//! The scheduler is responsible for managing all scheduled jobs.

use crate::{Job, Result, Tag, Timekeeper, Timestamp};
use log::*;

/// A Scheduler creates jobs, tracks recorded jobs, and executes jobs.
#[derive(Debug, Default)]
pub struct Scheduler {
    /// The currently scheduled lob list
    jobs: Vec<Job>,
    /// Interface to current time
    clock: Option<Box<dyn Timekeeper>>,
}

impl Scheduler {
    /// Instantiate a Scheduler
    pub fn new() -> Self {
        let _ = pretty_env_logger::try_init();
        Self::default()
    }

    /// Instantiate with mocked time
    #[cfg(test)]
    fn with_mock_time(clock: crate::time::mock::MockTime) -> Self {
        let mut ret = Self::new();
        ret.clock = Some(Box::new(clock));
        ret
    }

    /// Advance all clocks by a certain duration
    #[cfg(test)]
    fn bump_times(&mut self, duration: chrono::Duration) -> Result<()> {
        self.clock.as_mut().unwrap().add_duration(duration);
        for job in &mut self.jobs {
            job.add_duration(duration);
        }
        self.run_pending()?;
        Ok(())
    }

    /// Helper function to get the current time
    fn now(&self) -> Timestamp {
        // unwrap is safe, there will always be one
        self.clock.as_ref().unwrap().now()
    }

    /// Add a new job to the list
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    /// Run all jobs that are scheduled to run.  Does NOT run missed jobs!
    pub fn run_pending(&mut self) -> Result<()> {
        //let mut jobs_to_run: Vec<&Job> = self.jobs.iter().filter(|el| el.should_run()).collect();
        self.jobs.sort();
        let mut to_remove = Vec::new();
        for (idx, job) in self.jobs.iter_mut().enumerate() {
            if job.should_run() {
                let keep_going = job.execute()?;
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

        Ok(())
    }

    /// Run all jobs, regardless of schedule.
    pub fn run_all(&mut self, delay_seconds: u64) {
        debug!(
            "Running all {} jobs with {}s delay",
            self.jobs.len(),
            delay_seconds
        );
        for job in &mut self.jobs {
            if let Err(e) = job.execute() {
                eprintln!("Error: {}", e);
            }
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds))
        }
    }

    /// Get all jobs, optionally with a given tag.
    pub fn get_jobs(&self, tag: Option<Tag>) -> Vec<&Job> {
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
    pub fn clear(&mut self, tag: Option<Tag>) {
        if let Some(t) = tag {
            debug!("Deleting all jobs tagged {}", t);
            self.jobs.retain(|el| !el.has_tag(&t));
        } else {
            debug!("Deleting ALL jobs!!");
            let _ = self.jobs.drain(..);
        }
    }

    /// Grab the next upcoming timestamp
    pub fn next_run(&self) -> Option<Timestamp> {
        if self.jobs.is_empty() {
            None
        } else {
            // unwrap is safe, we know there's at least one job
            self.jobs.iter().min().unwrap().next_run
        }
    }

    /// Number of seconds until next run.  None if no jobs scheduled
    pub fn idle_seconds(&self) -> Option<i64> {
        Some((self.next_run()? - self.now()).num_seconds())
    }

    /// Get the most recently added job, for testing
    #[cfg(test)]
    fn most_recent_job(&self) -> &Job {
        &self.jobs[self.jobs.len() - 1]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        error::Result,
        time::mock::{MockTime, START},
        Interval,
    };
    use chrono::{Duration, Timelike};
    use pretty_assertions::assert_eq;

    /// Overshadow scheduler, every() and every_single() to use our clock instead
    fn setup() -> (Scheduler, impl Fn(Interval) -> Job, impl Fn() -> Job) {
        let clock = MockTime::default();
        let scheduler = Scheduler::with_mock_time(clock);

        let every = move |interval: Interval| -> Job { Job::with_mock_time(interval, clock) };

        let every_single = move || -> Job { Job::with_mock_time(1, clock) };

        (scheduler, every, every_single)
    }

    /// Empty mock job
    fn job() {}

    #[test]
    fn test_two_jobs() -> Result<()> {
        let (mut scheduler, every, every_single) = setup();

        assert_eq!(scheduler.idle_seconds(), None);

        every(17).seconds()?.run(&mut scheduler, job)?;
        assert_eq!(scheduler.idle_seconds(), Some(17));

        every_single().minute()?.run(&mut scheduler, job)?;
        assert_eq!(scheduler.idle_seconds(), Some(17));
        assert_eq!(scheduler.next_run(), Some(*START + Duration::seconds(17)));

        scheduler.bump_times(Duration::seconds(17))?;
        assert_eq!(
            scheduler.next_run(),
            Some(*START + Duration::seconds(17 * 2))
        );

        scheduler.bump_times(Duration::seconds(17))?;
        assert_eq!(
            scheduler.next_run(),
            Some(*START + Duration::seconds(17 * 3))
        );

        // This time, we should hit the minute mark next, not the next 17 second mark
        scheduler.bump_times(Duration::seconds(17))?;
        assert_eq!(scheduler.idle_seconds(), Some(9));
        assert_eq!(scheduler.next_run(), Some(*START + Duration::minutes(1)));

        // Afterwards, back to the 17 second job
        scheduler.bump_times(Duration::seconds(9))?;
        assert_eq!(scheduler.idle_seconds(), Some(8));
        assert_eq!(
            scheduler.next_run(),
            Some(*START + Duration::seconds(17 * 4))
        );

        Ok(())
    }

    #[test]
    fn test_time_range() -> Result<()> {
        let (mut scheduler, every, _) = setup();

        // Set up 100 jobs, store the minute of the next run
        let num_jobs = 100;
        let mut minutes = HashSet::with_capacity(num_jobs);
        for _ in 0..num_jobs {
            every(5).to(30)?.minutes()?.run(&mut scheduler, job)?;
            minutes.insert(scheduler.most_recent_job().next_run.unwrap().minute());
        }

        // Make sure each job got a run time within the specified bounds
        assert!(minutes.len() > 1);
        assert!(minutes.iter().min().unwrap() >= &5);
        assert!(minutes.iter().max().unwrap() <= &30);

        Ok(())
    }

    // TODO - job repr
    // #[test]
    // fn test_time_range_debug() -> Result<()> {
    //     let (mut scheduler, every, _) = setup();
    //
    //     every(5).to(30)?.minutes()?.run(&mut &mut scheduler, job)?;
    //
    //     assert_eq!(
    //         scheduler.most_recent_job().to_string(),
    //         "Every 5 to 30 minutes do job()"
    //     );
    //
    //     Ok(())
    // }

    #[test]
    fn test_at_time() -> Result<()> {
        let (mut scheduler, _, every_single) = setup();

        every_single()
            .day()?
            .at("10:30:50")?
            .run(&mut scheduler, job)?;
        assert_eq!(scheduler.most_recent_job().next_run.unwrap().hour(), 10);
        assert_eq!(scheduler.most_recent_job().next_run.unwrap().minute(), 30);
        assert_eq!(scheduler.most_recent_job().next_run.unwrap().second(), 50);

        Ok(())
    }
}
