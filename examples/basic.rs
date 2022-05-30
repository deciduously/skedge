// Some more varied usage examples.

use chrono::Local;
use skedge::{every, every_single, Scheduler};
use std::thread::sleep;
use std::time::Duration;

fn job() {
	let now = Local::now().to_rfc2822();
	println!("Hello, it's {now}!");
}

fn flirt(name: &str, time: &str, hour: u8, jackpot: i32, restaurant: &str, meal: &str) {
	println!(
		"Hello, {name}!  What are you doing {time}?  I'm free around {hour}.  \
        I just won ${jackpot} off a scratch ticket, you can get anything you want.  \
        Have you ever been to {restaurant}?  They're getting rave reviews over their {meal}."
	);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut schedule = Scheduler::new();

	every(10).seconds()?.run(&mut schedule, job)?;

	every(10).minutes()?.run(&mut schedule, job)?;

	every_single().hour()?.run(&mut schedule, job)?;

	every_single().day()?.at("10:30")?.run(&mut schedule, job)?;

	every(5).to(10)?.minutes()?.run(&mut schedule, job)?;

	every_single().monday()?.run(&mut schedule, job)?;

	every_single()
		.wednesday()?
		.at("13:15")?
		.run(&mut schedule, job)?;

	every(2)
		.to(8)?
		.seconds()?
		.until(Local::now() + chrono::Duration::days(5))?
		.run_six_args(
			&mut schedule,
			flirt,
			"Good-Looking",
			"Friday",
			7,
			40,
			"Dorsia",
			"foraged chanterelle croque monsieur",
		)?;

	let now = Local::now().to_rfc3339();
	println!("Starting at {now}");
	loop {
		if let Err(e) = schedule.run_pending() {
			eprintln!("Error: {e}");
		}
		sleep(Duration::from_secs(1));
	}
}
