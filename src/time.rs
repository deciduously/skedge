//! For mocking purposes, access to the current time is controlled directed through this struct.

use jiff::{Span, ToSpan, Zoned};
use std::fmt;

pub(crate) trait Timekeeper: std::fmt::Debug {
	/// Return the current time
	fn now(&self) -> Zoned;
	/// Add a specific duration for testing purposes
	#[cfg(test)]
	fn add_duration(&mut self, duration: impl Into<jiff::ZonedArithmetic>);
}

#[derive(Debug, Default)]
pub(crate) enum Clock {
	#[default]
	Real,
	#[cfg(test)]
	Mock(mock::Mock),
}

impl Timekeeper for Clock {
	fn now(&self) -> Zoned {
		match self {
			Clock::Real => Zoned::now(),
			#[cfg(test)]
			Clock::Mock(mock) => mock.now(),
		}
	}

	#[cfg(test)]
	fn add_duration(&mut self, duration: impl Into<jiff::ZonedArithmetic>) {
		match self {
			Clock::Real => unreachable!(),
			Clock::Mock(mock) => mock.add_duration(duration),
		}
	}
}

/// Jobs can be periodic over one of these units of time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
	Second,
	Minute,
	Hour,
	Day,
	Week,
	Month,
	Year,
}

impl Unit {
	/// Get a [`jiff::SignedDuration`] from an interval based on time unit.
	pub fn duration(self, interval: u32) -> Span {
		use Unit::{Day, Hour, Minute, Month, Second, Week, Year};
		let interval = i64::from(interval);
		match self {
			Second => interval.seconds(),
			Minute => interval.minutes(),
			Hour => interval.hours(),
			Day => interval.days(),
			Week => interval.weeks(),
			Month => interval.months(),
			Year => interval.years(),
		}
	}
}

impl fmt::Display for Unit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Unit::{Day, Hour, Minute, Month, Second, Week, Year};
		let s = match self {
			Second => "second",
			Minute => "minute",
			Hour => "hour",
			Day => "day",
			Week => "week",
			Month => "month",
			Year => "year",
		};
		write!(f, "{s}")
	}
}

#[cfg(test)]
pub mod mock {
	use super::Timekeeper;
	use jiff::{Zoned, ZonedArithmetic};
	use std::sync::LazyLock;

	pub(crate) static START: LazyLock<Zoned> =
		LazyLock::new(|| "2024-01-01:22:00:00[America/New_York]".parse().unwrap());

	/// Mock the datetime for predictable results.
	#[derive(Debug)]
	pub struct Mock {
		instant: Zoned,
	}

	impl Mock {
		pub fn new(stamp: Zoned) -> Self {
			Self { instant: stamp }
		}
	}

	impl Default for Mock {
		fn default() -> Self {
			Self::new(START.clone())
		}
	}

	impl Timekeeper for Mock {
		fn now(&self) -> Zoned {
			self.instant.clone()
		}

		fn add_duration(&mut self, duration: impl Into<ZonedArithmetic>) {
			let _ = self.instant.checked_add(duration);
		}
	}
}
