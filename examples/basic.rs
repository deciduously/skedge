// Some more varied usage examples.

#[cfg(feature = "random")]
use jiff::ToSpan as _;
use jiff::Zoned;
use skedge::{every, every_single, Scheduler};
use std::thread::sleep;
use std::time::Duration;

fn job() {
	let now = Zoned::now();
	println!("Hello, it's {now}!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut schedule = Scheduler::new();

	every(10).seconds()?.run(&mut schedule, job)?;

	every(10).minutes()?.run(&mut schedule, job)?;

	every_single().hour()?.run(&mut schedule, job)?;

	every_single().day()?.at("10:30")?.run(&mut schedule, job)?;

	#[cfg(feature = "random")]
	every(5).to(10)?.minutes()?.run(&mut schedule, job)?;

	every_single().monday()?.run(&mut schedule, job)?;

	every_single()
		.wednesday()?
		.at("13:15")?
		.run(&mut schedule, job)?;

	#[cfg(feature = "random")]
	every(2)
		.to(8)?
		.seconds()?
		.until(Zoned::now().checked_add(5.seconds()).unwrap())?
		.run(&mut schedule, job)?;

	let now = Zoned::now();
	println!("Starting at {now}");
	loop {
		if let Err(e) = schedule.run_pending() {
			eprintln!("Error: {e}");
		}
		sleep(Duration::from_secs(1));
	}
}
