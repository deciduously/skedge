use chrono::Local;
use skedge::{every, every_single, Scheduler};
use std::thread::sleep;
use std::time::Duration;

fn job() {
    println!("Hello, it's {}!", Local::now());
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

    println!("Starting at {}", Local::now());
    loop {
        if let Err(e) = schedule.run_pending() {
            eprintln!("Error: {}", e);
        }
        sleep(Duration::from_secs(1));
    }
}
