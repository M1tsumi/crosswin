use std::fmt;
use std::time::Duration;

// ─── ProcessInfo ──────────────────────────────────────────────────────────────

/// Comprehensive information about a running process.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Process name (executable name).
    pub name: String,
    /// Full path to the executable.
    pub executable_path: Option<String>,
    /// Parent process ID.
    pub parent_pid: Option<u32>,
    /// Memory usage in bytes (working set size).
    pub memory_usage: Option<u64>,
    /// User mode CPU time.
    pub user_cpu_time: Option<Duration>,
    /// Kernel mode CPU time.
    pub kernel_cpu_time: Option<Duration>,
    /// Number of threads.
    pub thread_count: Option<u32>,
    /// Process priority class.
    pub priority_class: Option<ProcessPriority>,
    /// Process creation time.
    pub creation_time: Option<SystemTime>,
}

impl ProcessInfo {
    /// Create a basic `ProcessInfo` with just PID and name.
    pub fn basic(pid: u32, name: String) -> Self {
        Self {
            pid,
            name,
            executable_path: None,
            parent_pid: None,
            memory_usage: None,
            user_cpu_time: None,
            kernel_cpu_time: None,
            thread_count: None,
            priority_class: None,
            creation_time: None,
        }
    }

    /// Get total CPU time (user + kernel).
    pub fn total_cpu_time(&self) -> Option<Duration> {
        match (self.user_cpu_time, self.kernel_cpu_time) {
            (Some(user), Some(kernel)) => Some(user + kernel),
            (Some(user), None) => Some(user),
            (None, Some(kernel)) => Some(kernel),
            (None, None) => None,
        }
    }

    /// Get memory usage in megabytes.
    pub fn memory_usage_mb(&self) -> Option<f64> {
        self.memory_usage.map(|bytes| bytes as f64 / (1024.0 * 1024.0))
    }

    /// Returns `true` when the process with this PID appears to still be running.
    ///
    /// Opens the process with `QueryLimitedInformation` and polls its exit status.
    /// Returns `false` on any error (process gone, access denied, etc.).
    pub fn is_alive(&self) -> bool {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::Foundation::CloseHandle;
            use windows::Win32::System::Threading::{
                GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            };

            // STILL_ACTIVE (259) means the process is still running.
            const STILL_ACTIVE: u32 = 259;

            unsafe {
                let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, self.pid) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                let mut exit_code: u32 = 0;
                let ok = GetExitCodeProcess(handle, &mut exit_code).is_ok();
                let _ = CloseHandle(handle);
                ok && exit_code == STILL_ACTIVE
            }
        }

        #[cfg(not(feature = "win32"))]
        {
            false
        }
    }
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.pid, self.name)?;
        if let Some(mb) = self.memory_usage_mb() {
            write!(f, "  mem={:.1} MB", mb)?;
        }
        if let Some(t) = self.thread_count {
            write!(f, "  threads={}", t)?;
        }
        if let Some(ref p) = self.priority_class {
            write!(f, "  priority={}", p)?;
        }
        Ok(())
    }
}

// ─── ProcessPriority ──────────────────────────────────────────────────────────

/// Process priority class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProcessPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
}

impl fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ProcessPriority::Idle => "Idle",
            ProcessPriority::BelowNormal => "BelowNormal",
            ProcessPriority::Normal => "Normal",
            ProcessPriority::AboveNormal => "AboveNormal",
            ProcessPriority::High => "High",
            ProcessPriority::Realtime => "Realtime",
        };
        f.write_str(s)
    }
}

impl ProcessPriority {
    /// Get the Windows priority class constant.
    #[cfg(feature = "win32")]
    pub fn to_windows_constant(&self) -> u32 {
        use windows::Win32::System::Threading::*;
        match self {
            ProcessPriority::Idle => IDLE_PRIORITY_CLASS.0,
            ProcessPriority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS.0,
            ProcessPriority::Normal => NORMAL_PRIORITY_CLASS.0,
            ProcessPriority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS.0,
            ProcessPriority::High => HIGH_PRIORITY_CLASS.0,
            ProcessPriority::Realtime => REALTIME_PRIORITY_CLASS.0,
        }
    }

    /// Create from a Windows priority class constant.
    #[cfg(feature = "win32")]
    pub fn from_windows_constant(value: u32) -> Option<Self> {
        use windows::Win32::System::Threading::*;
        match value {
            v if v == IDLE_PRIORITY_CLASS.0 => Some(ProcessPriority::Idle),
            v if v == BELOW_NORMAL_PRIORITY_CLASS.0 => Some(ProcessPriority::BelowNormal),
            v if v == NORMAL_PRIORITY_CLASS.0 => Some(ProcessPriority::Normal),
            v if v == ABOVE_NORMAL_PRIORITY_CLASS.0 => Some(ProcessPriority::AboveNormal),
            v if v == HIGH_PRIORITY_CLASS.0 => Some(ProcessPriority::High),
            v if v == REALTIME_PRIORITY_CLASS.0 => Some(ProcessPriority::Realtime),
            _ => None,
        }
    }
}

use std::time::SystemTime;

// ─── MemoryInfo ───────────────────────────────────────────────────────────────

/// Detailed memory statistics for a process.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryInfo {
    /// Current working set size in bytes.
    pub working_set_size: u64,
    /// Peak working set size in bytes.
    pub peak_working_set_size: u64,
    /// Page file usage in bytes.
    pub page_file_usage: u64,
    /// Peak page file usage in bytes.
    pub peak_page_file_usage: u64,
}

// ─── CpuTimes ─────────────────────────────────────────────────────────────────

/// CPU time information for a process.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CpuTimes {
    /// Time spent in user mode.
    pub user_time: Duration,
    /// Time spent in kernel mode.
    pub kernel_time: Duration,
    /// Process creation time.
    pub creation_time: SystemTime,
    /// Process exit time (if terminated).
    pub exit_time: Option<SystemTime>,
}

impl CpuTimes {
    /// Get total CPU time.
    pub fn total(&self) -> Duration {
        self.user_time + self.kernel_time
    }
}
