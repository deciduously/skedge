// This is the exact code from the README.md example

use chrono::Local;
use skedge::{every, Scheduler};
use std::thread::sleep;
use std::time::Duration;

fn greet(name: &str) {
	let now = Local::now().to_rfc2822();
	println!("Hello {name}, it's {now}!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut schedule = Scheduler::new();

	every(2)
		.to(8)?
		.seconds()?
		.until(Local::now() + chrono::Duration::seconds(30))?
		.run_one_arg(&mut schedule, greet, "Cool Person")?;

	let now = Local::now();
	println!("Starting at {now}");
	loop {
		if let Err(e) = schedule.run_pending() {
			eprintln!("Error: {e}");
		}
		sleep(Duration::from_secs(1));
	}
}
