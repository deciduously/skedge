//! For mocking purposes, access to the current time is controlled directed through this struct.

use jiff::{Span, ToSpan as _, Zoned};
use std::fmt;

pub(crate) trait Timekeeper: std::fmt::Debug {
	/// Return the current time
	fn now(&self) -> Zoned;
	/// Add a specific duration for testing purposes
	#[cfg(test)]
	fn add_duration(&mut self, duration: impl Into<jiff::ZonedArithmetic>) -> crate::Result<()>;
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
	fn add_duration(&mut self, duration: impl Into<jiff::ZonedArithmetic>) -> crate::Result<()> {
		match self {
			Clock::Real => unreachable!(),
			Clock::Mock(mock) => mock.add_duration(duration),
		}
	}
}

#[cfg(test)]
pub mod mock {
	use super::Timekeeper;
	use jiff::{Zoned, ZonedArithmetic};
	use std::sync::LazyLock;

	pub(crate) static START: LazyLock<Zoned> =
		LazyLock::new(|| "2024-01-01T07:00:00[America/New_York]".parse().unwrap());

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

		fn add_duration(&mut self, duration: impl Into<ZonedArithmetic>) -> crate::Result<()> {
			self.instant = self.instant.checked_add(duration)?;
			Ok(())
		}
	}
}
