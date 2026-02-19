use crate::error::{CrosswinError, Result};
use crate::windows::handles::Handle;

#[cfg(feature = "win32")]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
};

// ─── ThreadInfo ───────────────────────────────────────────────────────────────

/// Basic information about a thread returned by `list_threads`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThreadInfo {
    /// Thread ID.
    pub thread_id: u32,
    /// Owning process ID.
    pub process_id: u32,
    /// Base priority reported by the snapshot.
    pub base_priority: i32,
}

// ─── Thread ───────────────────────────────────────────────────────────────────

/// RAII wrapper around a Windows thread handle.
#[derive(Debug)]
pub struct Thread {
    handle: Option<Handle>,
}

impl Thread {
    /// Obtain a pseudo-handle for the calling thread.
    pub fn current() -> Result<Self> {
        Ok(Thread { handle: None })
    }

    /// Access the underlying handle, if any.
    pub fn handle(&self) -> Option<&Handle> {
        self.handle.as_ref()
    }
}

// ─── list_threads ─────────────────────────────────────────────────────────────

/// Enumerate all threads belonging to `pid`.
///
/// Returns an empty `Vec` on non-Win32 builds.
pub async fn list_threads(pid: u32) -> Result<Vec<ThreadInfo>> {
    #[cfg(feature = "win32")]
    {
        tokio::task::spawn_blocking(move || enumerate_threads_blocking(pid))
            .await
            .map_err(|e| CrosswinError::win32("list_threads", 0, format!("join error: {}", e)))?
    }

    #[cfg(not(feature = "win32"))]
    {
        let _ = pid;
        Ok(Vec::new())
    }
}

#[cfg(feature = "win32")]
fn enumerate_threads_blocking(pid: u32) -> Result<Vec<ThreadInfo>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).map_err(|e| {
            CrosswinError::win32(
                "CreateToolhelp32Snapshot",
                e.code().0 as u32,
                e.to_string(),
            )
        })?;

        // Wrap in RAII so it is closed even on early return.
        let _guard = Handle::from_windows_handle(snapshot);

        let mut entry = THREADENTRY32::default();
        entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

        if Thread32First(snapshot, &mut entry).is_err() {
            // No threads in snapshot (very unlikely but handle it).
            return Ok(Vec::new());
        }

        let mut threads = Vec::new();
        loop {
            if entry.th32OwnerProcessID == pid {
                threads.push(ThreadInfo {
                    thread_id: entry.th32ThreadID,
                    process_id: entry.th32OwnerProcessID,
                    base_priority: entry.tpBasePri,
                });
            }
            if Thread32Next(snapshot, &mut entry).is_err() {
                break;
            }
        }

        Ok(threads)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "win32")]
    #[tokio::test]
    async fn list_threads_current_process() {
        let pid = std::process::id();
        let threads = super::list_threads(pid).await.expect("list_threads failed");
        assert!(!threads.is_empty(), "current process must have at least one thread");
        for t in &threads {
            assert_eq!(t.process_id, pid);
        }
    }
}

