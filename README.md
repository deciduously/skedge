# skedge

**WIP DO NOT USE**

Rust single-process scheduling.  Inspired by [`schedule`](https://github.com/dbader/schedule) (Python), in turn inspired by [`clockwork`](https://github.com/Rykian/clockwork) (Ruby), and ["Rethinking Cron"](https://adam.herokuapp.com/past/2010/4/13/rethinking_cron/) by [Adam Wiggins](https://github.com/adamwiggins).

## Usage

Don't.  But eventually:

```rust
use skedge::{Scheduler, every, every_single};
use std::time::Duration;
use std::thread::sleep;

fn job() {
    println!("Hello!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let mut schedule = Scheduler::new();

    every(10).seconds()?.run(&mut schedule, job);
    every(10).minutes()?.run(&mut schedule, job);
    every_single().hour()?.run(&mut schedule, job);
    every_single().day()?.at("10:30")?.run(&mut schedule, job);
    every(5).to(10)?.minutes()?.run(&mut schedule, job);
    every_single().monday()?.run(&mut schedule, job);
    every_single().wednesday()?.at("13:15")?.run(&mut schedule, job);
    every_single().minute()?.at(":17")?.run(&mut schedule, job);

    loop {
        schedule.run_pending();
        sleep(Duration::from_secs(1));
    }
}
```

## Development

Clone this repo.

### Dependencies
 
* **Stable [Rust](https://www.rust-lang.org/tools/install)**:  The default stable toolchain is fine.  Obtainable via `rustup` using the instructions at this link.

### Crates

* [chrono](https://github.com/chronotope/chrono) - Date and time handling
* [log](https://github.com/rust-lang/log) - Logging
* [pretty_env_logger](https://github.com/seanmonstar/pretty-env-logger) - Pretty logging
* [lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs) - Lazily evaluated statics
* [regex](https://github.com/rust-lang/regex) - Regular expressions
* [thiserror](https://github.com/dtolnay/thiserror) - Error derive macro
