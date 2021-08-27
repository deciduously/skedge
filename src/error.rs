//! This module defines the error type and Result alias.

// FIXME - this is probably not right
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkedgeError {
    #[error("Basic error")]
    ScheduleError,
    #[error("Value error")]
    ScheduleValueError,
    #[error("An improper interval was used")]
    IntervalError,
}

pub type Result<T> = std::result::Result<T, SkedgeError>;
