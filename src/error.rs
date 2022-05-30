//! This module defines the error type and Result alias.

use crate::Unit;
use chrono::Weekday;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum Error {
	#[error("Tried to reference this job's inner subroutine but failed")]
	CallableUnreachable,
	#[error("Use {0}s() instead of {0}()")]
	Interval(Unit),
	#[error("Cannot set {0}s mode, already using {1}s")]
	Unit(Unit, Unit),
	#[error("Latest val is greater than interval val")]
	InvalidInterval,
	#[error("Invalid unit (valid units are `days`, `hours`, and `minutes`)")]
	InvalidUnit,
	#[error("Invalid hour ({0} is not between 0 and 23)")]
	InvalidHour(u32),
	#[error("Invalid time format for daily job (valid format is HH:MM(:SS)?)")]
	InvalidDailyAtStr,
	#[error("Invalid time format for hourly job (valid format is (MM)?:SS)")]
	InvalidHourlyAtStr,
	#[error("Invalid time format for minutely job (valid format is :SS)")]
	InvalidMinuteAtStr,
	#[error("Invalid string format for until()")]
	InvalidUntilStr,
	#[error("Cannot schedule a job to run until a time in the past")]
	InvalidUntilTime,
	#[error("Attempted to reference the next run time but failed")]
	NextRunUnreachable,
	#[error("Attempted to reference the last run time but failed")]
	LastRunUnreachable,
	#[error("Attempted to reference the period but failed")]
	PeriodUnreachable,
	#[error("Attempted to reference the period but failed")]
	UnitUnreachable,
	#[error("Attempted to use a start day for a unit other than `weeks`")]
	StartDayError,
	#[error("{0}")]
	ParseInt(#[from] std::num::ParseIntError),
	#[error("Scheduling jobs on {0} is only allowed for weekly jobs.  Using specific days on a job scheduled to run every 2 or more weeks is not supported")]
	Weekday(Weekday),
	#[error("Cannot schedule {0} job, already scheduled for {1}")]
	WeekdayCollision(Weekday, Weekday),
	#[error("Invalid unit without specifying start day")]
	UnspecifiedStartDay,
}

/// Construct a new Unit error
pub(crate) fn unit_error(intended: Unit, existing: Unit) -> Error {
	Error::Unit(intended, existing)
}

pub(crate) fn invalid_hour_error(hour: u32) -> Error {
	Error::InvalidHour(hour)
}

/// Construct a new Interval error
pub(crate) fn interval_error(interval: Unit) -> Error {
	Error::Interval(interval)
}

/// Construct a new Weekday error
pub(crate) fn weekday_error(weekday: Weekday) -> Error {
	Error::Weekday(weekday)
}

pub(crate) fn weekday_collision_error(intended: Weekday, existing: Weekday) -> Error {
	Error::WeekdayCollision(intended, existing)
}

pub type Result<T> = std::result::Result<T, Error>;
