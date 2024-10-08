# skedge

[![Crates.io](https://img.shields.io/crates/v/skedge.svg)](https://crates.io/crates/skedge)
[![rust action](https://github.com/deciduously/skedge/actions/workflows/rust.yml/badge.svg)](https://github.com/deciduously/skedge/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/skedge)](https://docs.rs/skedge)

Rust single-process scheduling. Ported from [`schedule`](https://github.com/dbader/schedule) for Python, in turn inspired by [`clockwork`](https://github.com/Rykian/clockwork) (Ruby), and ["Rethinking Cron"](https://adam.herokuapp.com/past/2010/4/13/rethinking_cron/) by [Adam Wiggins](https://github.com/adamwiggins).

## Usage

Documentation can be found on [docs.rs](https://docs.rs/skedge).

This library uses the Builder pattern to define jobs. Instantiate a fresh `Scheduler`, then use the `every()` and `every_single()` functions to begin defining a job. Finalize configuration by calling `Job::run()` to add the new job to the scheduler. The `Scheduler::run_pending()` method is used to fire any jobs that have arrived at their next scheduled run time. Currently, precision can only be specified to the second, no smaller.

```rust
use skedge::{every, Scheduler};
use std::{
	thread,
	time::{Duration, SystemTime},
};

fn seconds_from_epoch() -> u64 {
	SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs()
}

fn greet(name: &str) {
	let timestamp = seconds_from_epoch();
	println!("Hello {name}, it's been {timestamp} seconds since the Unix epoch!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut schedule = Scheduler::new();

	every(10)
		.minutes()?
		.at(":17")?
		.until(
			SystemTime::now()
				.checked_add(Duration::from_secs(2 * 60 * 60))
				.unwrap()
				.try_into()?,
		)?
		.run_one_arg(&mut schedule, greet, "Cool Person")?;

	let now = seconds_from_epoch();
	println!("Starting at {now}");
	loop {
		if let Err(e) = schedule.run_pending() {
			eprintln!("Error: {e}");
		}
		thread::sleep(Duration::from_secs(1));
	}
}
```

Check out the [example script](https://github.com/deciduously/skedge/blob/main/examples/basic.rs) to see more configuration options. Try `cargo run --example readme` or `cargo run --example basic` to see it in action.

### CFFI

There is an **experimental** C foreign function interface, which is feature-gated and not included by default. To build the library with this feature, use `cargo build --features ffi`. See the [Makefile](https://github.com/deciduously/skedge/blob/main/Makefile) and [examples/ffi/c](https://github.com/deciduously/skedge/tree/main/examples/ffi/c) directory for details on using this library from C. Execute `make run` to build and execute the included example C program. It currently **only** supports work functions which take no arguments.

## Development

Clone this repo. See [`CONTRIBUTING.md`](https://github.com/deciduously/skedge/blob/main/CONTRIBUTING.md) for contribution guidelines.

### Dependencies

- **Stable [Rust](https://www.rust-lang.org/tools/install)**. Obtainable via `rustup` using the instructions at this link.

### Crates

- [jiff](https://github.com/BurntSushi/jiff) - Date and time handling
- [libc](https://github.com/rust-lang/libc) - libc bindings for CFFI (optional)
- [rand](https://rust-random.github.io/book/) - Random number generation (optional)
- [regex](https://github.com/rust-lang/regex) - Regular expressions
- [thiserror](https://github.com/dtolnay/thiserror) - Error derive macro
- [tracing](https://github.com/tokio-rs/tracing) - what it says on the tin

#### Development-Only

- [pretty_assertions](https://github.com/colin-kiegel/rust-pretty-assertions) - Colorful assertion output
