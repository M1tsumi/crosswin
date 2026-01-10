use std::time::Duration;

/// Comprehensive information about a running process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name (executable name)
    pub name: String,
    /// Full path to the executable
    pub executable_path: Option<String>,
    /// Parent process ID
    pub parent_pid: Option<u32>,
    /// Memory usage in bytes (working set size)
    pub memory_usage: Option<u64>,
    /// User mode CPU time
    pub user_cpu_time: Option<Duration>,
    /// Kernel mode CPU time
    pub kernel_cpu_time: Option<Duration>,
    /// Number of threads
    pub thread_count: Option<u32>,
    /// Process priority class
    pub priority_class: Option<ProcessPriority>,
    /// Process creation time
    pub creation_time: Option<SystemTime>,
}

impl ProcessInfo {
    /// Create a basic ProcessInfo with just PID and name
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

    /// Get total CPU time (user + kernel)
    pub fn total_cpu_time(&self) -> Option<Duration> {
        match (self.user_cpu_time, self.kernel_cpu_time) {
            (Some(user), Some(kernel)) => Some(user + kernel),
            (Some(user), None) => Some(user),
            (None, Some(kernel)) => Some(kernel),
            (None, None) => None,
        }
    }

    /// Get memory usage in megabytes
    pub fn memory_usage_mb(&self) -> Option<f64> {
        self.memory_usage.map(|bytes| bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Process priority class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
}

impl ProcessPriority {
    /// Get the Windows priority class constant
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

    /// Create from Windows priority class constant
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

/// Memory information for a process
#[derive(Debug, Clone, Copy)]
pub struct MemoryInfo {
    /// Current working set size in bytes
    pub working_set_size: u64,
    /// Peak working set size in bytes
    pub peak_working_set_size: u64,
    /// Page file usage in bytes
    pub page_file_usage: u64,
    /// Peak page file usage in bytes
    pub peak_page_file_usage: u64,
}

/// CPU time information for a process
#[derive(Debug, Clone, Copy)]
pub struct CpuTimes {
    /// Time spent in user mode
    pub user_time: Duration,
    /// Time spent in kernel mode
    pub kernel_time: Duration,
    /// Process creation time
    pub creation_time: SystemTime,
    /// Process exit time (if terminated)
    pub exit_time: Option<SystemTime>,
}

impl CpuTimes {
    /// Get total CPU time
    pub fn total(&self) -> Duration {
        self.user_time + self.kernel_time
    }
}
