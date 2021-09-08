// This is the exact code from the README.md example

use chrono::Local;
use skedge::{every, Scheduler};
use std::thread::sleep;
use std::time::Duration;

fn greet(name: &str) {
    println!("Hello {}, it's {}!", name, Local::now().to_rfc2822());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut schedule = Scheduler::new();

    every(2)
        .to(8)?
        .seconds()?
        .until(Local::now() + chrono::Duration::seconds(30))?
        .run_one_arg(&mut schedule, greet, "Good-Looking")?;

    println!("Starting at {}", Local::now());
    loop {
        if let Err(e) = schedule.run_pending() {
            eprintln!("Error: {}", e);
        }
        sleep(Duration::from_secs(1));
    }
}
