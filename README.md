# skedge

[![Crates.io](https://img.shields.io/crates/v/skedge.svg)](https://crates.io/crates/skedge)
[![rust action](https://github.com/deciduously/skedge/actions/workflows/rust.yml/badge.svg)](https://github.com/deciduously/skedge/actions/workflows/rust.yml)

Rust single-process scheduling.  Ported from [`schedule`](https://github.com/dbader/schedule) for Python, in turn inspired by [`clockwork`](https://github.com/Rykian/clockwork) (Ruby), and ["Rethinking Cron"](https://adam.herokuapp.com/past/2010/4/13/rethinking_cron/) by [Adam Wiggins](https://github.com/adamwiggins).

**NOTE**: This library is currently limited to jobs which take no arguments and return nothing.

## Usage

Documentation can be found on [docs.rs](https://docs.rs/skedge).

This library uses the Builder pattern to define jobs.  Instantiate a fresh `Scheduler`, then use the `every()` and `every_single()` functions to begin defining a job.  Finalize configuration by calling `Job::run()` to add the new job to the scheduler.  The `Scheduler::run_pending()` method is used to fire any jobs that have arrived at their next scheduled run time.  Currently, precision can only be specified to the second, no smaller.

```rust
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
    every_single().day()?.at("10:30")?.run(&mut schedule, job);
    every(5).to(10)?.minutes()?.run(&mut schedule, job);
    every_single().monday()?.run(&mut schedule, job);
    every_single().wednesday()?.at("13:15")?.run(&mut schedule, job);
    every_single().minute()?.at(":17")?.run(&mut schedule, job);
    every(2)
        .to(8)?
        .seconds()?
        .until(Local::now() + chrono::Duration::seconds(30))?
        .run(&mut schedule, job)?;

    println!("Starting at {}", Local::now());
    loop {
        if let Err(e) = schedule.run_pending() {
            eprintln!("Error: {}", e);
        }
        sleep(Duration::from_secs(1));
    }
}
```

Try `cargo run --example basic` to see it in action.

## Development

Clone this repo.  See [`CONTRIBUTING.md`](https://github.com/deciduously/skedge/blob/main/CONTRIBUTING.md) for contribution guidelines.

### Dependencies
 
* **Stable [Rust](https://www.rust-lang.org/tools/install)**:  The default stable toolchain is fine.  Obtainable via `rustup` using the instructions at this link.

### Crates

* [chrono](https://github.com/chronotope/chrono) - Date and time handling
* [log](https://github.com/rust-lang/log) - Logging
* [pretty_env_logger](https://github.com/seanmonstar/pretty-env-logger) - Pretty logging
* [lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs) - Lazily evaluated statics
* [rand](https://rust-random.github.io/book/) - Random number generation
* [regex](https://github.com/rust-lang/regex) - Regular expressions
* [thiserror](https://github.com/dtolnay/thiserror) - Error derive macro

#### Development-Only

 * [mockall](https://github.com/asomers/mockall) - Mock objects
 * [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) - Colorful assertion output