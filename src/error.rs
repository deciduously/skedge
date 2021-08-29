//! This module defines the error type and Result alias.

use super::TimeUnit;
use chrono::Weekday;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkedgeError {
    #[error("Value error")]
    Value,
    #[error("Use {0}s() instead of {0}()")]
    Interval(TimeUnit),
    #[error("Cannot set {0}s mode, already using {1}s")]
    Unit(TimeUnit, TimeUnit),
    #[error("Scheduling jobs on {0} is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported")]
    Weekday(Weekday),
    #[error("Cannot schedule {0} job, already scheduled for {1}")]
    WeekdayCollision(Weekday, Weekday),
}

/// Construct a new Value error
pub(crate) fn value_error() -> SkedgeError {
    SkedgeError::Value
}

/// Construct a new Unit error
pub(crate) fn unit_error(intended: TimeUnit, existing: TimeUnit) -> SkedgeError {
    SkedgeError::Unit(intended, existing)
}

/// Construct a new Interval error
pub(crate) fn interval_error(interval: TimeUnit) -> SkedgeError {
    SkedgeError::Interval(interval)
}

/// Construct a new Weekday error
pub(crate) fn weekday_error(weekday: Weekday) -> SkedgeError {
    SkedgeError::Weekday(weekday)
}

pub(crate) fn weekday_collision_error(intended: Weekday, existing: Weekday) -> SkedgeError {
    SkedgeError::WeekdayCollision(intended, existing)
}

pub(crate) type Result<T> = std::result::Result<T, SkedgeError>;
