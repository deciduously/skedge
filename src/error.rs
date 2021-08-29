//! This module defines the error type and Result alias.

use super::TimeUnit;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkedgeError {
    #[error("Value error")]
    Value,
    #[error("Use {0}s() instead of {0}()")]
    Interval(TimeUnit),
    #[error("Cannot set {0}s mode, already using {1}s")]
    Unit(TimeUnit, TimeUnit),
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

pub(crate) type Result<T> = std::result::Result<T, SkedgeError>;
