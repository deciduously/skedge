// This is the exact code from the README.md example

use skedge::{every, Scheduler};
use std::{
	thread,
	time::{Duration, SystemTime},
};

fn seconds_from_epoch() -> u64 {
	SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs()
}

fn greet(name: &str) {
	let timestamp = seconds_from_epoch();
	println!("Hello {name}, it's been {timestamp} seconds since the Unix epoch!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut schedule = Scheduler::new();

	every(10)
		.minutes()?
		.at(":17")?
		.until(
			SystemTime::now()
				.checked_add(Duration::from_secs(2 * 60 * 60))
				.unwrap()
				.try_into()?,
		)?
		.run_one_arg(&mut schedule, greet, "Cool Person")?;

	let now = seconds_from_epoch();
	println!("Starting at {now}");
	loop {
		if let Err(e) = schedule.run_pending() {
			eprintln!("Error: {e}");
		}
		thread::sleep(Duration::from_secs(1));
	}
}
