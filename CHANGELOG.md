# Changelog

All notable changes to this project will be documented in this file.

## 0.3.0 - 2026-01-11

### Major Features

- **Window Management**: Added window discovery and manipulation APIs under `src/windows/window.rs`:
  - `list_windows()` enumerates top-level windows and returns `WindowInfo` (title, class, size, visibility, owning PID).
  - `Window` wrapper and operations: `show`, `hide`, `set_title`, `bring_to_front` (Win32-backed when `win32` feature enabled; stubs otherwise).
  - Search helpers: `find_windows_by_title`, `find_windows_by_class`, `find_windows_by_process`.

- **System Information**: Added `src/windows/system.rs`:
  - `get_system_info()` (CPU count, page size, total physical memory when available).
  - `get_system_uptime()` and `get_boot_time()` (Win32-backed via `GetTickCount64`, stubs otherwise).

### Improvements

- **Examples**: Added examples demonstrating new APIs:
  - `examples/list_windows.rs`
  - `examples/find_window.rs`
  - `examples/system_info.rs`

- **Docs & Packaging**:
  - Bumped crate version to `0.3.0`.
  - Updated `README.md` and `current.txt` to document the new surface.

### Notes

- The Win32 implementations are gated behind the `win32` feature and use the `windows` crate; on non-Windows builds the APIs are graceful stubs that either return empty lists or `CrosswinError::invalid_parameter("platform")` where appropriate. Use `--features win32` to enable full functionality on Windows.
- Minor warnings from interim stubs were fixed (unused variables / dead code allowances) and `cargo check --no-default-features --features tokio` completes successfully on Linux.


## 0.2.0 - 2026-01-10

### Major Features

- **Enhanced Process Information**: ProcessInfo now includes:
  - Parent PID for process tree relationships
  - Memory usage (working set size)
  - CPU time (user and kernel mode)
  - Thread count
  - Process priority class
  - Process creation time

- **Process Operations API**: New `Process` type with methods for:
  - Opening processes with specific access rights
  - Terminating processes
  - Suspending and resuming all process threads
  - Waiting for process exit with timeout
  - Getting and setting process priority
  - Querying detailed memory information

- **Process Filtering and Search**:
  - `find_process_by_pid()` - Find a specific process
  - `find_process_by_name()` - Find processes by name (case-insensitive)
  - `ProcessFilter` builder for complex queries (filter by name, memory, threads, parent PID)

- **RAII Handle Management**:
  - Automatic handle cleanup on drop (prevents resource leaks)
  - Type-safe `ProcessHandle` and `ThreadHandle` wrappers
  - Proper handle validation

### Improvements

- **Better Error Handling**:
  - Structured error types with operation context and error codes
  - Dedicated error variants: `AccessDenied`, `ProcessNotFound`, `InvalidParameter`, `Timeout`
  - Error messages include relevant context (PID, operation name, etc.)

- **New Types**:
  - `MemoryInfo` - Detailed memory statistics
  - `CpuTimes` - CPU time and process lifetime information
  - `ProcessPriority` - Type-safe priority class enum
  - `ProcessAccess` - Type-safe access rights enum

### Examples

Added comprehensive examples:
- `kill_process.rs` - Find and terminate processes by name
- `monitor_memory.rs` - Monitor process memory usage over time
- `process_tree.rs` - Display parent-child process relationships
- `top_cpu.rs` - Show top processes by CPU time

### Testing

- Expanded test suite with 20+ new tests
- Process information validation tests
- Process operations tests
- Error handling tests
- Handle management tests

### Breaking Changes

- `CrosswinError::Win32` variant changed from `Win32(String)` to structured fields
- `ProcessInfo` struct has new fields (backward compatible for reading, but construction changed)
- `Handle` type is no longer `Clone` to prevent double-close bugs
- Updated minimum required Windows API features in Cargo.toml

### Dependencies

- Added Windows API features: `Win32_System_ProcessStatus`, `Win32_Security`

## 0.1.0 - 2025-12-03

- First public release of `crosswin`.
- Async process enumeration on Windows using `tokio` and the `windows` crate.
- Simple `ProcessInfo` type exposing PID, executable name, and best-effort executable path.
- Small prelude and error type for ergonomic imports.
- `list_processes` example and a Windows CI workflow that runs the test suite.
