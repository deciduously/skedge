# skedge

[![Crates.io](https://img.shields.io/crates/v/skedge.svg)](https://crates.io/crates/skedge)
[![rust action](https://github.com/deciduously/skedge/actions/workflows/rust.yml/badge.svg)](https://github.com/deciduously/skedge/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/skedge)](https://docs.rs/skedge)

Rust single-process scheduling. Ported from [`schedule`](https://github.com/dbader/schedule) for Python, in turn inspired by [`clockwork`](https://github.com/Rykian/clockwork) (Ruby), and ["Rethinking Cron"](https://adam.herokuapp.com/past/2010/4/13/rethinking_cron/) by [Adam Wiggins](https://github.com/adamwiggins).

## Usage

Documentation can be found on [docs.rs](https://docs.rs/skedge).

This library uses the Builder pattern to define jobs. Instantiate a fresh `Scheduler`, then use the `every()` and `every_single()` functions to begin defining a job. Finalize configuration by calling `Job::run()` to add the new job to the scheduler. The `Scheduler::run_pending()` method is used to fire any jobs that have arrived at their next scheduled run time. Currently, precision can only be specified to the second, no smaller.

```rust
use jiff::Zoned;
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
        .until(Zoned::now() + Duration::from_secs(30))?
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
```

Check out the [example script](https://github.com/deciduously/skedge/blob/main/examples/basic.rs) to see more configuration options. Try `cargo run --example readme` or `cargo run --example basic` to see it in action.

### CFFI

There is an **experimental** C foreign function interface, which is feature-gated and not included by default. To build the library with this feature, use `cargo build --features ffi`. See the [Makefile](https://github.com/deciduously/skedge/blob/main/Makefile) and [examples/ffi/c](https://github.com/deciduously/skedge/tree/main/examples/ffi/c) directory for details on using this library from C. Execute `make run` to build and execute the included example C program. It currently **only** supports work functions which take no arguments.

## Development

Clone this repo. See [`CONTRIBUTING.md`](https://github.com/deciduously/skedge/blob/main/CONTRIBUTING.md) for contribution guidelines.

### Dependencies

- **Stable [Rust](https://www.rust-lang.org/tools/install)**: The default stable toolchain is fine. Obtainable via `rustup` using the instructions at this link.

### Crates

- [chrono](https://github.com/chronotope/chrono) - Date and time handling
- [log](https://github.com/rust-lang/log) - Logging
- [pretty_env_logger](https://github.com/seanmonstar/pretty-env-logger) - Pretty logging
- [lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs) - Lazily evaluated statics
- [rand](https://rust-random.github.io/book/) - Random number generation
- [regex](https://github.com/rust-lang/regex) - Regular expressions
- [thiserror](https://github.com/dtolnay/thiserror) - Error derive macro

#### Development-Only

- [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) - Colorful assertion output
