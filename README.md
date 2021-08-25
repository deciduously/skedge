# skedge

Rust single-process scheduling.  Inspired by [`schedule`](https://github.com/dbader/schedule) (Python), in turn inspired by [`clockwork`](https://github.com/Rykian/clockwork) (Ruby), and ["Rethinking Cron"](https://adam.herokuapp.com/past/2010/4/13/rethinking_cron/) by [Adam Wiggins](https://github.com/adamwiggins).

## Usage

Don't (yet).  But eventually:

```rust
use skedge::{Scheduler, every, every_single};
use std::time::Duration;
use std::thread::sleep;

fn job() {
    println!("Hello!");
}

fn main() {
   let mut schedule = Scheduler::new();

    every(10).seconds().run(job, &schedule);
    every(10).minutes().run(job, &schedule);
    every_single().hour().run(job, &schedule);
    every_single().day().at("10:30").run(job, &schedule);
    every(5).to(10).minutes().run(job, &schedule);
    every_single().monday().run(job, &schedule);
    every_single().wednesday().at("13:15").run(job, &schedule);
    every_single().minute().at(":17").run(job, &schedule);
    
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
* [thiserror](https://github.com/dtolnay/thiserror) - Error derive macro
