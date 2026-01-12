# crosswin

[![Crates.io](https://img.shields.io/crates/v/crosswin.svg)](https://crates.io/crates/crosswin)
[![Docs.rs](https://docs.rs/crosswin/badge.svg)](https://docs.rs/crosswin)
[![CI](https://github.com/M1tsumi/crosswin/actions/workflows/ci.yml/badge.svg)](https://github.com/M1tsumi/crosswin/actions/workflows/ci.yml)

Async-friendly Windows primitives for Rust.

A high-level, ergonomic wrapper around the Windows API that provides async process management, memory monitoring, and system operations without hiding what the OS is actually doing. Built on `tokio` for async operations and the `windows` crate for safe Win32 API bindings.

## Status

Active development. Version 0.3.0 adds window discovery/manipulation and system information queries on top of the existing process-management surface.

**Supported (v0.3.0):**
- Process enumeration with rich metadata (PID, name, path, parent, memory, CPU time, threads, priority)
- Process operations (open, terminate, suspend, resume, wait for exit)
- Process filtering and search by name, memory usage, thread count, parent PID
- Memory usage monitoring
- Process priority management
- RAII handle management (automatic cleanup, prevents leaks)
- Window enumeration and basic window operations (stubs on non-Windows builds)
- Basic system information queries (CPU count, page size, uptime stub)

**Planned (future):**
- Registry operations
- Service management
- Advanced performance counters and metrics
- Event log reading

## Features

- **Async-first**: Built on `tokio` for non-blocking operations
- **Type-safe**: Strong typing for handles, priorities, and access rights
- **Resource-safe**: RAII handles prevent resource leaks
- **Structured errors**: Rich error types with operation context
- **Zero-cost abstractions**: Thin wrappers over Win32 API
- **Explicit API**: Clear about what Windows APIs are being called

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
crosswin = "0.2"
```

This crate targets Windows and requires `tokio` in your dependency tree.

## Examples

### List Processes with Rich Information

```rust
use crosswin::prelude::*;
use crosswin::processes::list_processes;

#[tokio::main]
async fn main() -> Result<()> {
    for process in list_processes().await? {
        println!(
            "PID {:>6}  {:30}  Memory: {:.2} MB  Threads: {}",
            process.pid,
            process.name,
            process.memory_usage_mb().unwrap_or(0.0),
            process.thread_count.unwrap_or(0)
        );
    }
    Ok(())
}
```

### Find and Terminate a Process

```rust
use crosswin::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Find process by name
    if let Some(proc_info) = find_process_by_name("notepad").await? {
        // Open with terminate access
        let process = Process::open(proc_info.pid, ProcessAccess::Terminate)?;
        
        // Terminate with exit code 0
        process.terminate(0)?;
        println!("Terminated process {}", proc_info.pid);
    }
    Ok(())
}
```

### Filter Processes

```rust
use crosswin::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Find memory-hungry Chrome processes
    let processes = ProcessFilter::new()
        .name_contains("chrome")
        .min_memory(100_000_000) // 100 MB
        .min_threads(5)
        .list()
        .await?;
    
    println!("Found {} Chrome processes using >100MB", processes.len());
    Ok(())
}
```

### Monitor Memory Usage

```rust
use crosswin::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let proc_info = find_process_by_name("myapp").await?.expect("Process not found");
    let process = Process::open(proc_info.pid, ProcessAccess::QueryInformation)?;
    
    loop {
        let mem = process.get_memory_info()?;
        println!("Memory: {:.2} MB", mem.working_set_size as f64 / (1024.0 * 1024.0));
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

### More Examples

Run the bundled examples:

```bash
cargo run --example list_processes    # List all processes
cargo run --example kill_process notepad  # Terminate by name
cargo run --example monitor_memory chrome # Track memory over time
cargo run --example process_tree      # Show parent-child relationships
cargo run --example top_cpu 10        # Top 10 CPU consumers
```

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
