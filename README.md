# crosswin

[![Crates.io](https://img.shields.io/crates/v/crosswin.svg)](https://crates.io/crates/crosswin)
[![Docs.rs](https://docs.rs/crosswin/badge.svg)](https://docs.rs/crosswin)
[![CI](https://github.com/M1tsumi/crosswin/actions/workflows/ci.yml/badge.svg)](https://github.com/M1tsumi/crosswin/actions/workflows/ci.yml)

Async-friendly Windows primitives for Rust.

This crate starts small: it gives you an async way to inspect running processes on Windows. Over time the idea is to grow into an ergonomic wrapper over the Windows API without hiding what the OS is actually doing.

## Status

Early MVP.

- Supported: process enumeration (PID and executable name) on Windows.
- Planned: richer process info, stronger handle types, additional Win32 modules.

## Features

- Async process listing built on `tokio`.
- Straightforward, explicit API surface.
- Designed as a building block for higher-level Windows tooling.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
crosswin = "0.1"
```

This crate targets Windows and expects `tokio` in your dependency tree. The default feature set enables the Win32-backed implementation.

## Example

List the current processes and print a short line for each one:

```rust
use crosswin::prelude::*;
use crosswin::processes::list_processes;

#[tokio::main]
async fn main() -> Result<()> {
    for process in list_processes().await? {
        println!("{:>6}  {}", process.pid, process.name);
    }

    Ok(())
}
```

Run the bundled example from this repository with:

```bash
cargo run --example list_processes
```

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
