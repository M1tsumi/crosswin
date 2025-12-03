# Changelog

All notable changes to this project will be documented in this file.

## 0.1.0 - 2025-12-03

- First public release of `crosswin`.
- Async process enumeration on Windows using `tokio` and the `windows` crate.
- Simple `ProcessInfo` type exposing PID, executable name, and best-effort executable path.
- Small prelude and error type for ergonomic imports.
- `list_processes` example and a Windows CI workflow that runs the test suite.
