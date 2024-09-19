// This is the exact code from the README.md example

use jiff::{ToSpan, Zoned};
use skedge::{every, Scheduler};
use std::thread::sleep;
use std::time::Duration;

fn greet(name: &str) {
	let now = Zoned::now();
	println!("Hello {name}, it's {now}!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut schedule = Scheduler::new();

	every(2)
		.to(8)?
		.seconds()?
		.until(Zoned::now().checked_add(30.seconds()).unwrap())?
		.run_one_arg(&mut schedule, greet, "Cool Person")?;

	let now = Zoned::now();
	println!("Starting at {now}");
	loop {
		if let Err(e) = schedule.run_pending() {
			eprintln!("Error: {e}");
		}
		sleep(Duration::from_secs(1));
	}
}
