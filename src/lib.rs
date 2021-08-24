//! # skedge
//!
//! `skedge` is a single-process job scheduler.

use thiserror::Error;

// FIXME - this is probably not right
#[derive(Debug, Error)]
enum SkedgeError {
    #[error("Basic error")]
    ScheduleError,
    #[error("Value error")]
    ScheduleValueError,
    #[error("An improper interval was used")]
    IntervalError
}

/// A Tag is used to categorize a job.
struct Tag(String);

/// A Job is anything that can be scheduled to run periodically.
struct Job {
    tags: Vec<Tag>
}

/// A Scheduler creates jobs, tracks recorded jobs, and executes jobs.
struct Scheduler {
    jobs: Vec<Job>
}

impl Scheduler {
    /// Instantiate a Scheduler
    fn new() -> Self {
        Self {
            jobs: Vec::new()
        }
    }

    /// Run all jobs that are scheduled to run.  Does NOT run missed jobs!
    fn run_pending() {
        unimplemented!()
    }

    /// Run all jobs, regardless of schedule.
    fn run_all(delay_seconds: Option<u32>) {
        // if None, default to 0.
        unimplemented!()
    }

    /// Get all jobs, optionally with a given tag.
    fn get_jobs<'a>(tag: Option<Tag>) -> &'a [Job] {
        unimplemented!()
    }

    /// Clear all jobs, optionally only with given tag.
    fn clear(tag: Option<Tag>) {
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
