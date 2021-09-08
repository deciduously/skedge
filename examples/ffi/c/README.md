# CFFI

This directory demonstrates how to use `skedge` from C.

## Usage

```
$ git clone https://github.com/deciduously/skedge
$ cd skedge
$ cargo build --release
$ cc -o skedge_demo -L ./target/release -l skedge ./examples/ffi/c/main.c
```

This will place a `skedge_demo` executable in the top level of the project.
