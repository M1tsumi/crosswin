/// Smoke tests for thread enumeration (requires the `win32` feature).
use crosswin::prelude::*;

#[cfg(feature = "win32")]
#[tokio::test]
async fn list_threads_current_process() -> Result<()> {
    let pid = std::process::id();
    let threads = list_threads(pid).await?;

    assert!(!threads.is_empty(), "current process must have at least one thread");

    for t in &threads {
        assert_eq!(t.process_id, pid, "all threads must belong to the queried PID");
        assert!(t.thread_id > 0, "thread_id must be positive");
    }

    Ok(())
}

#[cfg(feature = "win32")]
#[tokio::test]
async fn list_threads_invalid_pid_returns_empty() -> Result<()> {
    // PID 0 is the idle process and has no user-accessible threads via snapshot.
    // Very high fake PID should return an empty list (process doesn't exist).
    let threads = list_threads(0xFFFF_FF00).await?;
    assert!(threads.is_empty(), "non-existent PID should yield empty thread list");
    Ok(())
}
