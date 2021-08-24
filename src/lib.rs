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

/// A Job is anything that can be scheduled to run periodically.
trait Job {}

/// A Scheduler creates jobs, tracks recorded jobs, and executes jobs.
struct Scheduler {
    jobs: Vec<Box<dyn Job>>
}

impl Scheduler {
    fn new() -> Self {
        Self {
            jobs: Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
