//! The scheduler is responsible for managing all scheduled jobs.

use crate::{Clock, Job, Result, Tag, Timekeeper};
use jiff::{SpanRound, Unit, Zoned};
use tracing::debug;

/// A Scheduler creates jobs, tracks recorded jobs, and executes jobs.
#[derive(Debug, Default)]
pub struct Scheduler {
	/// The currently scheduled lob list
	jobs: Vec<Job>,
	/// Interface to current time
	clock: Clock,
}

impl Scheduler {
	/// Instantiate a Scheduler
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Instantiate with mocked time
	#[cfg(test)]
	fn with_mock_time(clock: crate::time::mock::Mock) -> Self {
		Self {
			clock: Clock::Mock(clock),
			..Default::default()
		}
	}

	/// Add a new job to the list
	pub(crate) fn add_job(&mut self, job: Job) {
		self.jobs.push(job);
	}

	/// Run all jobs that are scheduled to run.  Does NOT run missed jobs!
	/// ```rust
	/// # use skedge::{every, Scheduler};
	/// # fn job() {}
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let mut scheduler = Scheduler::new();
	/// every(5).seconds()?.run(&mut scheduler, job)?;
	/// scheduler.run_pending()?;
	/// # Ok(())
	/// # }
	/// ```
	///
	/// # Errors
	///
	/// Returns an error if any job failes to execute.
	pub fn run_pending(&mut self) -> Result<()> {
		//let mut jobs_to_run: Vec<&Job> = self.jobs.iter().filter(|el| el.should_run()).collect();
		self.jobs.sort();
		let mut to_remove = Vec::new();
		let now = self.now();
		for (idx, job) in self.jobs.iter_mut().enumerate() {
			if job.should_run(&now) {
				let keep_going = job.execute(&now)?;
				if !keep_going {
					debug!("Cancelling job {job}");
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
		let num_jobs = self.jobs.len();
		debug!("Running all {num_jobs} jobs with {delay_seconds}s delay");
		let now = self.now();
		for job in &mut self.jobs {
			if let Err(e) = job.execute(&now) {
				eprintln!("Error: {e}");
			}
			std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
		}
	}

	/// Get all jobs, optionally with a given tag.
	/// ```rust
	/// # use skedge::{every, Scheduler};
	/// # fn job() {}
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let mut scheduler = Scheduler::new();
	/// every(5).seconds()?.run(&mut scheduler, job)?;
	/// every(10).minutes()?.run(&mut scheduler, job)?;
	/// let jobs = scheduler.get_jobs(None);
	/// assert_eq!(jobs.len(), 2);
	/// # Ok(())
	/// # }
	/// ```
	#[must_use]
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
	/// ```rust
	/// # use skedge::{every, Scheduler};
	/// # fn job() {}
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let mut scheduler = Scheduler::new();
	/// every(5).seconds()?.run(&mut scheduler, job)?;
	/// every(10).minutes()?.run(&mut scheduler, job)?;
	/// assert_eq!(scheduler.get_jobs(None).len(), 2);
	/// scheduler.clear(None);
	/// assert_eq!(scheduler.get_jobs(None).len(), 0);
	/// # Ok(())
	/// # }
	/// ```
	pub fn clear(&mut self, tag: Option<Tag>) {
		if let Some(tag) = tag {
			debug!(?tag, "Deleting all jobs with tag");
			self.jobs.retain(|el| !el.has_tag(&tag));
		} else {
			debug!("Deleting ALL jobs!!");
			drop(self.jobs.drain(..));
		}
	}

	/// Grab the next upcoming timestamp
	/// ```rust
	/// # use skedge::{every, Scheduler};
	/// # use jiff::ToSpan as _;
	/// # fn job() {}
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let mut scheduler = Scheduler::new();
	/// every(10).minutes()?.run(&mut scheduler, job)?;
	/// let expected = jiff::Zoned::now().checked_add(10.minutes())?;
	/// assert!(scheduler.next_run().unwrap() == expected);
	/// # Ok(())
	/// # }
	/// ```
	///
	/// # Panics
	///
	/// Would panic if it can't call `min()` on an array that we know has at least one element.
	#[must_use]
	pub fn next_run(&self) -> Option<&Zoned> {
		if self.jobs.is_empty() {
			None
		} else {
			// unwrap is safe, we know there's at least one job
			self.jobs.iter().min().unwrap().next_run.as_ref()
		}
	}

	/// Number of whole seconds until next run.  None if no jobs scheduled.
	/// ```rust
	/// # use skedge::{every, Scheduler};
	/// # fn job() {}
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// let mut scheduler = Scheduler::new();
	/// every(10).minutes()?.run(&mut scheduler, job)?;
	/// // Subtract one - we're already partway through the first second, so there's 599 left.
	/// assert_eq!(scheduler.idle_seconds()?.unwrap(), 10 * 60 - 1);
	/// # Ok(())
	/// # }
	/// ```
	#[must_use]
	pub fn idle_seconds(&self) -> Result<Option<i64>> {
		let seconds = self
			.next_run()
			.map(|zdt| {
				Ok::<_, crate::Error>(
					self.now()
						.until(zdt)?
						.round(SpanRound::new().largest(Unit::Second))?
						.get_seconds(),
				)
			})
			.transpose()?;
		Ok(seconds)
	}

	/// Get the most recently added job, for testing
	#[cfg(test)]
	fn most_recent_job(&self) -> Option<&Job> {
		if self.jobs.is_empty() {
			return None;
		}
		Some(&self.jobs[self.jobs.len() - 1])
	}
}

impl Timekeeper for Scheduler {
	fn now(&self) -> Zoned {
		self.clock.now()
	}

	#[cfg(test)]
	fn add_duration(&mut self, duration: impl Into<jiff::ZonedArithmetic>) {
		self.clock.add_duration(duration)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		error::Result,
		every, every_single,
		time::mock::{Mock, START},
	};
	use jiff::{civil, ToSpan as _};
	use pretty_assertions::assert_eq;

	/// Overshadow scheduler, `every()` and `every_single()` to use our clock instead
	fn setup() -> Scheduler {
		let clock = Mock::default();
		let scheduler = Scheduler::with_mock_time(clock);

		scheduler
	}

	/// Empty mock job
	fn job() {}

	#[test]
	fn test_two_jobs() -> Result<()> {
		let mut scheduler = setup();

		assert_eq!(scheduler.idle_seconds().unwrap(), None);

		every(17).seconds()?.run(&mut scheduler, job)?;
		assert_eq!(scheduler.idle_seconds().unwrap(), Some(17));

		every_single().minute()?.run(&mut scheduler, job)?;
		assert_eq!(scheduler.idle_seconds().unwrap(), Some(17));
		assert_eq!(
			scheduler.next_run(),
			Some(&START.checked_add(17.seconds()).unwrap())
		);

		scheduler.add_duration(17.seconds());
		scheduler.run_pending()?;
		println!("after one: {}", scheduler.now());
		assert_eq!(
			scheduler.next_run(),
			Some(&START.checked_add((17 * 2).seconds()).unwrap())
		);

		scheduler.add_duration(17.seconds());
		scheduler.run_pending()?;
		assert_eq!(
			scheduler.next_run(),
			Some(&START.checked_add((17 * 3).seconds()).unwrap())
		);

		// This time, we should hit the minute mark next, not the next 17 second mark
		scheduler.add_duration(17.seconds());
		scheduler.run_pending()?;
		assert_eq!(scheduler.idle_seconds().unwrap(), Some(9));
		assert_eq!(
			scheduler.next_run(),
			Some(&START.checked_add(1.minutes()).unwrap())
		);

		// Afterwards, back to the 17 second job
		scheduler.add_duration(9.seconds());
		scheduler.run_pending()?;
		assert_eq!(scheduler.idle_seconds().unwrap(), Some(8));
		assert_eq!(
			scheduler.next_run(),
			Some(&START.checked_add((17 * 4).seconds()).unwrap())
		);

		Ok(())
	}

	#[test]
	#[cfg(feature = "random")]
	fn test_time_range() -> Result<()> {
		let mut scheduler = setup();

		// Set up 100 jobs, store the minute of the next run
		let num_jobs = 100;
		let mut minutes = std::collections::HashSet::with_capacity(num_jobs);
		for _ in 0..num_jobs {
			every(5).to(30)?.minutes()?.run(&mut scheduler, job)?;
			minutes.insert(
				scheduler
					.most_recent_job()
					.unwrap()
					.next_run
					.as_ref()
					.unwrap()
					.minute(),
			);
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
		let mut scheduler = setup();

		every_single()
			.day()?
			.at("10:30:50")?
			.run(&mut scheduler, job)?;
		assert_eq!(
			scheduler
				.most_recent_job()
				.unwrap()
				.next_run
				.as_ref()
				.unwrap()
				.hour(),
			10
		);
		assert_eq!(
			scheduler
				.most_recent_job()
				.unwrap()
				.next_run
				.as_ref()
				.unwrap()
				.minute(),
			30
		);
		assert_eq!(
			scheduler
				.most_recent_job()
				.unwrap()
				.next_run
				.as_ref()
				.unwrap()
				.second(),
			50
		);

		Ok(())
	}

	#[test]
	fn test_clear_scheduler() -> Result<()> {
		let mut scheduler = setup();

		every_single().day()?.run(&mut scheduler, job)?;
		every_single().minute()?.run(&mut scheduler, job)?;
		assert_eq!(scheduler.jobs.len(), 2);
		scheduler.clear(None);
		assert_eq!(scheduler.jobs.len(), 0);

		Ok(())
	}

	#[test]
	fn test_until_time() -> Result<()> {
		let mut scheduler = setup();

		// Make sure it stores a deadline

		let deadline = civil::date(3000, 1, 1)
			.at(12, 0, 0, 0)
			.intz("America/New_York")
			.unwrap();
		every_single()
			.day()?
			.until(deadline.clone())?
			.run(&mut scheduler, job)?;
		assert_eq!(
			scheduler
				.most_recent_job()
				.unwrap()
				.cancel_after
				.clone()
				.unwrap(),
			deadline
		);

		// Make sure it cancels a job after next_run passes the deadline
		// FIXME - this test fails? call count never increments

		scheduler.clear(None);
		let deadline = civil::date(2024, 1, 1)
			.at(7, 0, 10, 0)
			.intz("America/New_York")
			.unwrap();
		every(5)
			.seconds()?
			.until(deadline)?
			.run(&mut scheduler, job)?;
		assert_eq!(scheduler.most_recent_job().unwrap().call_count, 0);
		scheduler.add_duration(5.seconds());
		scheduler.run_pending()?;
		assert_eq!(scheduler.most_recent_job().unwrap().call_count, 1);
		assert_eq!(scheduler.jobs.len(), 1);
		scheduler.add_duration(5.seconds());
		scheduler.run_pending()?;
		assert_eq!(scheduler.jobs.len(), 1);
		assert_eq!(scheduler.most_recent_job().unwrap().call_count, 2);
		scheduler.add_duration(5.seconds());
		scheduler.run_pending()?;
		// TODO - how to test to ensure the job did not run?
		// FIXME - job doesnt disappear?
		assert_eq!(scheduler.jobs.len(), 0);

		// Make sure it cancels a job if current execution passes the deadline

		scheduler.clear(None);
		let deadline = START.clone();
		every(5)
			.seconds()?
			.until(deadline)?
			.run(&mut scheduler, job)?;
		scheduler.add_duration(5.seconds());
		scheduler.run_pending()?;
		// TODO - how to test to ensure the job did not run?
		assert_eq!(scheduler.jobs.len(), 0);

		Ok(())
	}

	#[test]
	fn test_weekday_at_time() -> Result<()> {
		let mut scheduler = setup();

		every_single()
			.wednesday()?
			.at("22:38:10")?
			.run(&mut scheduler, job)?;
		let j = scheduler.most_recent_job().unwrap();

		assert_eq!(j.next_run.as_ref().unwrap().year(), 2024);
		assert_eq!(j.next_run.as_ref().unwrap().month(), 1);
		assert_eq!(j.next_run.as_ref().unwrap().day(), 3);
		assert_eq!(j.next_run.as_ref().unwrap().hour(), 22);
		assert_eq!(j.next_run.as_ref().unwrap().minute(), 38);
		assert_eq!(j.next_run.as_ref().unwrap().second(), 10);

		scheduler.clear(None);

		every_single()
			.wednesday()?
			.at("22:39")?
			.run(&mut scheduler, job)?;
		let j = scheduler.most_recent_job().unwrap();

		assert_eq!(j.next_run.as_ref().unwrap().year(), 2024);
		assert_eq!(j.next_run.as_ref().unwrap().month(), 1);
		assert_eq!(j.next_run.as_ref().unwrap().day(), 3);
		assert_eq!(j.next_run.as_ref().unwrap().hour(), 22);
		assert_eq!(j.next_run.as_ref().unwrap().minute(), 39);
		assert_eq!(j.next_run.as_ref().unwrap().second(), 0);

		Ok(())
	}
}
