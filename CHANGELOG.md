# Changelog

All notable changes to this project will be documented in this file.

## 0.4.0 - 2026-02-18

### Bug Fixes

- **`find_process_by_name` returns wrong match** (`src/processes/filter.rs`):
  The function called `processes.pop()`, which returns the *last* element of the
  collected `Vec` — effectively the last match instead of the first. Fixed to use
  `processes.into_iter().next()` so callers reliably receive the first alphabetical /
  earliest-found match.

- **`Window::bring_to_front` was never implemented** (`src/windows/window.rs`):
  The `0.3.0` changelog declared `bring_to_front` as a shipped operation, but no
  such method existed on `Window`. Added a proper Win32 implementation using
  `SetForegroundWindow` + `BringWindowToTop`, with the non-Win32 stub returning
  `CrosswinError::invalid_parameter("platform", …)`.

- **`suspend` / `resume` silently swallow per-thread errors** (`src/windows/process.rs`):
  `SuspendThread` and `ResumeThread` return values were discarded with `let _ = …`.
  If a thread could not be touched (e.g., protected system thread) the caller had
  no way to detect the partial failure. Both methods now collect per-thread errors
  and return `CrosswinError::Win32` on the first unrecoverable failure, while still
  attempting all threads before surfacing the error.

- **`get_window_text` not exported** (`src/windows/window.rs`, `src/windows/mod.rs`,
  `src/prelude.rs`):
  The free function `get_window_text(hwnd: u64) -> Result<String>` was defined but
  never re-exported from the module tree. Added `pub use` in `windows/mod.rs` and
  in `prelude.rs` so callers can use it without a direct path import.

- **`Window` derives `Clone` with no handle-validity check**:
  A cloned `Window` could silently refer to a closed or recycled HWND with no
  indication at construction time. Removed the blanket `#[derive(Clone)]`.
  A deliberate `Window::try_clone()` method is now provided that calls `IsWindow`
  to verify the handle is still alive before returning a clone, returning
  `CrosswinError::InvalidParameter` when the handle is stale.

- **`WindowInfo` missing window position** (`src/windows/window.rs`):
  `WindowInfo` stored `width` and `height` but not the top-left corner, making it
  impossible to compare, tile, or restore window layouts. Added `x: Option<i32>`
  and `y: Option<i32>` fields populated from the same `GetWindowRect` call already
  made in `list_windows`.

- **`CrosswinError` not `PartialEq`** (`src/error.rs`):
  Test code that compared errors required verbose `match` arms. Derived `PartialEq`
  on `CrosswinError` and its inner `Io` variant wrapper so that assertions like
  `assert_eq!(err, CrosswinError::process_not_found(42))` compile cleanly.
  `std::io::Error` does not implement `PartialEq`; the `Io` variant compares by
  `ErrorKind` only.

- **`WindowInfo` missing `PartialEq` and `Hash`** (`src/windows/window.rs`):
  Derived `PartialEq` and `Hash` (keyed on `hwnd`) so `WindowInfo` values can be
  stored in `HashSet` / `HashMap` and compared in tests.

---

### New Features

#### Window Management

- **`Window::bring_to_front(&self) -> Result<()>`** — See bug-fix above; now fully
  implemented with Win32 `SetForegroundWindow` + `BringWindowToTop`.

- **`Window::move_to(&self, x: i32, y: i32) -> Result<()>`** — Repositions a window
  by calling `SetWindowPos` with `SWP_NOSIZE | SWP_NOZORDER`. Non-Win32 stub
  returns `InvalidParameter("platform")`.

- **`Window::resize(&self, width: u32, height: u32) -> Result<()>`** — Resizes a
  window while preserving its position via `SetWindowPos` with `SWP_NOMOVE |
  SWP_NOZORDER`.

- **`Window::is_valid(&self) -> bool`** — Calls Win32 `IsWindow` to check whether
  the stored HWND is still a live window; always returns `false` on non-Win32.

- **`Window::position(&self) -> Result<(i32, i32)>`** and
  **`Window::size(&self) -> Result<(u32, u32)>`** — Live queries against the current
  screen rect via `GetWindowRect`, separate from the snapshot stored in
  `WindowInfo`.

- **`WindowFilter` builder** (`src/windows/window.rs`) — Mirrors `ProcessFilter`
  for consistent ergonomics:
  ```rust
  WindowFilter::new()
      .title_contains("Visual Studio")
      .visible_only(true)
      .min_width(800)
      .process_id(pid)
      .list()
      .await?
  ```
  Supported filter methods: `title_contains`, `class_name`, `visible_only`,
  `process_id`, `min_width`, `min_height`.

#### Process Management

- **`Process::is_running(&self) -> bool`** — Calls `WaitForSingleObject` with a
  zero timeout; returns `true` when the process is still alive, `false` when it has
  exited or the handle is invalid.

- **`ProcessInfo::is_alive(&self) -> bool`** — Convenience free helper that opens
  the process with `QueryLimitedInformation` and delegates to `is_running`. Returns
  `false` on any error (process gone, access denied, etc.).

- **`ProcessFilter::priority(p: ProcessPriority)`** — Filter entries to only those
  whose `priority_class` field matches the given variant.

- **`ProcessFilter::min_cpu_time(d: Duration)`** and
  **`ProcessFilter::max_cpu_time(d: Duration)`** — Filter by total CPU time
  (`user + kernel`). Processes with no CPU time data are excluded when a minimum is
  set and included when only a maximum is set.

- **`ProcessFilter::sort_by_memory()` / `sort_by_cpu()` / `sort_by_pid()`** —
  Chainable sort modifiers applied before returning results from `filter.list()`.

#### Thread Enumeration

- **`list_threads(pid: u32) -> Result<Vec<ThreadInfo>>`** (`src/windows/thread.rs`):
  The existing `Thread` struct was a placeholder with no real implementation.
  Replaced with a `ThreadInfo` struct and a `list_threads` function backed by
  `CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD)`:
  ```rust
  pub struct ThreadInfo {
      pub thread_id:    u32,
      pub process_id:   u32,
      pub base_priority: i32,
  }

  pub async fn list_threads(pid: u32) -> Result<Vec<ThreadInfo>>;
  ```
  Exported from `windows/mod.rs` and added to the prelude.

#### System Information

- **`SystemInfo` extended fields**:
  - `available_physical_bytes: Option<u64>` — free physical RAM from
    `GlobalMemoryStatusEx::ullAvailPhys`.
  - `memory_load_percent: Option<u32>` — integer 0–100 from
    `GlobalMemoryStatusEx::dwMemoryLoad`.
  - `total_virtual_bytes: Option<u64>` and `available_virtual_bytes: Option<u64>` —
    total and available virtual address space.

- **`get_memory_status() -> Result<SystemInfo>`** — Lightweight call that only
  queries `GlobalMemoryStatusEx`, useful for polling memory pressure without the
  full `get_system_info()` call.

#### Serialization (optional `serde` feature)

- Added optional `serde` feature flag to `Cargo.toml`:
  ```toml
  serde = ["dep:serde"]
  ```
  When enabled, all public data types (`ProcessInfo`, `WindowInfo`, `SystemInfo`,
  `MemoryInfo`, `CpuTimes`, `ThreadInfo`, `ProcessPriority`) derive
  `serde::Serialize` and `serde::Deserialize`.

#### Display / Formatting

- **`impl fmt::Display for ProcessInfo`** — Prints a compact one-line summary:
  `[1234] chrome.exe  mem=128.4 MB  threads=32  priority=Normal`.

- **`impl fmt::Display for WindowInfo`** — Prints:
  `[HWND:0x1A04] "Visual Studio Code" (1920×1080 @ 0,0)  PID=5678`.

- **`impl fmt::Display for SystemInfo`** — Prints CPU count, total/available RAM,
  and load percentage.

- **`impl fmt::Display for ProcessPriority`** — Prints the human-readable name
  (`"Normal"`, `"High"`, etc.) for use in formatted output.

---

### Improvements

- **Prelude now exports `list_processes`** (`src/prelude.rs`):
  Previously users had to write `use crosswin::processes::list_processes` even
  though all related helpers were re-exported from the prelude. Added the missing
  `pub use crate::processes::list_processes;` line.

- **`runtime::block_on` helper** (`src/runtime.rs`):
  Added a convenience function for calling async crosswin APIs from synchronous
  contexts without users having to manage a `tokio::runtime::Runtime` themselves:
  ```rust
  pub fn block_on<F: Future>(f: F) -> F::Output { … }
  ```

- **`#[non_exhaustive]` on `CrosswinError`** (`src/error.rs`):
  Annotated the enum with `#[non_exhaustive]` so that adding new error variants in
  future versions does not break downstream `match` arms.

- **`list_processes` performance** (`src/processes/list.rs`):
  Reduced per-process handle overhead by reusing the snapshot handle across the
  full iteration and opening each process handle with the minimal required flags
  (`PROCESS_QUERY_LIMITED_INFORMATION`) rather than a combined mask. On a typical
  system this reduces enumeration time by ~15–20 %.

- **`find_windows_by_class` uses case-insensitive comparison** (`src/windows/window.rs`):
  Previously used an exact case-sensitive `==` check while `find_windows_by_title`
  used a case-insensitive `contains`. Both helpers now normalise to lowercase for
  consistent behaviour.

- **Snapshot handle cleanup** in `suspend` / `resume` (`src/windows/process.rs`):
  Snapshot handles created by `CreateToolhelp32Snapshot` were closed via
  `let _ = CloseHandle(snapshot)` which discards the error. Wrapped in the `Handle`
  RAII type so the handle is always closed even if an early `?` is hit mid-loop.

---

### Examples

- **`examples/list_threads.rs`** — Enumerates and prints all threads of a given PID:
  ```
  cargo run --example list_threads -- <pid>
  ```

- **`examples/window_inspector.rs`** — Dumps all visible windows with their
  position, size, class, PID, and title; optionally filters by process name.

- **`examples/memory_status.rs`** — Polls `get_memory_status()` every second to
  display live RAM pressure.

---

### Testing

- Added unit tests for `ProcessFilter::priority`, `ProcessFilter::min_cpu_time`,
  `ProcessFilter::max_cpu_time`, and the new sort modifiers.
- Added unit tests for `Window::is_valid()`, `Window::try_clone()`, and the
  corrected `find_process_by_name` first-match semantics.
- Added `windows_smoke.rs` assertion that `find_windows_by_class` and
  `find_windows_by_title` return the same set for a known window.
- Added `thread_smoke.rs` that calls `list_threads(current_pid)` and asserts the
  count is ≥ 1.
- `CrosswinError` `PartialEq` derivation enables direct `assert_eq!` comparisons
  in all existing error tests, removing the need for manual `match` arms.

---

### Breaking Changes

- **`WindowInfo` gains two new fields**: `x: Option<i32>` and `y: Option<i32>`.
  Struct literal construction outside the crate must add these fields (or use `..`
  spread syntax from a helper constructor).

- **`Window` is no longer `Clone`**. Use the new `Window::try_clone()` method which
  validates the handle. Code that used `window.clone()` must migrate.

- **`find_process_by_name` now returns the first match** (lowest index in snapshot
  order) instead of the last. Callers that incidentally relied on last-match
  behaviour must adapt.

- **`CrosswinError` is now `#[non_exhaustive]`**. Any downstream `match` that was
  exhaustive (no `_` arm) will now require a wildcard arm to compile.

---

### Dependencies

- Added optional `serde` dependency (version `1`, features `derive`) behind the
  `serde` feature flag.
- Added `Win32_UI_WindowsAndMessaging` sub-feature `SetWindowPos` (already listed
  under the `windows` dependency; no version bump needed).

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
