use std::time::Duration;

use crate::error::Result;
use super::{ProcessInfo, ProcessPriority, list_processes};

/// Find a process by its PID.
pub async fn find_process_by_pid(pid: u32) -> Result<Option<ProcessInfo>> {
    let processes = list_processes().await?;
    Ok(processes.into_iter().find(|p| p.pid == pid))
}

/// Find all processes whose name contains `name` (case-insensitive).
pub async fn find_processes_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    let processes = list_processes().await?;
    let name_lower = name.to_lowercase();
    Ok(processes
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&name_lower))
        .collect())
}

/// Find the *first* process whose name contains `name` (case-insensitive).
///
/// # Bug fix (v0.4.0)
/// Previously called `Vec::pop()` which returned the *last* match.  Now uses
/// `Iterator::next()` so the earliest snapshot entry is returned consistently.
pub async fn find_process_by_name(name: &str) -> Result<Option<ProcessInfo>> {
    let processes = find_processes_by_name(name).await?;
    Ok(processes.into_iter().next())
}

// ─── Sort order ───────────────────────────────────────────────────────────────

/// How `ProcessFilter::list()` should order results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Keep snapshot order (default).
    #[default]
    None,
    /// Sort ascending by PID.
    Pid,
    /// Sort descending by working-set memory.
    MemoryDesc,
    /// Sort descending by total CPU time.
    CpuDesc,
}

// ─── ProcessFilter ────────────────────────────────────────────────────────────

/// Builder for filtering processes with multiple criteria.
///
/// ```rust,no_run
/// # use crosswin::processes::{ProcessFilter, ProcessPriority};
/// # #[tokio::main] async fn main() {
/// let results = ProcessFilter::new()
///     .name_contains("chrome")
///     .min_memory(50 * 1024 * 1024)
///     .priority(ProcessPriority::Normal)
///     .sort_by_memory()
///     .list()
///     .await
///     .unwrap();
/// # }
/// ```
#[derive(Default, Debug, Clone)]
pub struct ProcessFilter {
    name_contains: Option<String>,
    min_memory_bytes: Option<u64>,
    max_memory_bytes: Option<u64>,
    min_threads: Option<u32>,
    parent_pid: Option<u32>,
    priority: Option<ProcessPriority>,
    min_cpu_time: Option<Duration>,
    max_cpu_time: Option<Duration>,
    sort: SortOrder,
}

impl ProcessFilter {
    /// Create a new, empty process filter.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Predicates ────────────────────────────────────────────────────────────

    /// Filter by name substring (case-insensitive).
    pub fn name_contains<S: Into<String>>(mut self, name: S) -> Self {
        self.name_contains = Some(name.into().to_lowercase());
        self
    }

    /// Filter by minimum memory usage in bytes.
    pub fn min_memory(mut self, bytes: u64) -> Self {
        self.min_memory_bytes = Some(bytes);
        self
    }

    /// Filter by maximum memory usage in bytes.
    pub fn max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }

    /// Filter by minimum thread count.
    pub fn min_threads(mut self, count: u32) -> Self {
        self.min_threads = Some(count);
        self
    }

    /// Filter by parent process ID.
    pub fn parent_pid(mut self, pid: u32) -> Self {
        self.parent_pid = Some(pid);
        self
    }

    /// Filter by exact priority class.
    ///
    /// Processes whose `priority_class` field is `None` are excluded when this
    /// filter is active.
    pub fn priority(mut self, p: ProcessPriority) -> Self {
        self.priority = Some(p);
        self
    }

    /// Only include processes whose total CPU time is ≥ `d`.
    ///
    /// Processes with no CPU time data are **excluded** when this filter is set.
    pub fn min_cpu_time(mut self, d: Duration) -> Self {
        self.min_cpu_time = Some(d);
        self
    }

    /// Only include processes whose total CPU time is ≤ `d`.
    ///
    /// Processes with no CPU time data are **included** (treated as zero CPU
    /// time), letting callers find idle / freshly-started processes.
    pub fn max_cpu_time(mut self, d: Duration) -> Self {
        self.max_cpu_time = Some(d);
        self
    }

    // ── Sort modifiers ────────────────────────────────────────────────────────

    /// Sort results ascending by PID.
    pub fn sort_by_pid(mut self) -> Self {
        self.sort = SortOrder::Pid;
        self
    }

    /// Sort results descending by working-set memory (largest first).
    pub fn sort_by_memory(mut self) -> Self {
        self.sort = SortOrder::MemoryDesc;
        self
    }

    /// Sort results descending by total CPU time (highest first).
    pub fn sort_by_cpu(mut self) -> Self {
        self.sort = SortOrder::CpuDesc;
        self
    }

    // ── Execution ─────────────────────────────────────────────────────────────

    /// Execute the filter and return matching processes.
    pub async fn list(self) -> Result<Vec<ProcessInfo>> {
        let processes = list_processes().await?;
        let mut results: Vec<ProcessInfo> = processes
            .into_iter()
            .filter(|p| self.matches(p))
            .collect();

        match self.sort {
            SortOrder::None => {}
            SortOrder::Pid => results.sort_by_key(|p| p.pid),
            SortOrder::MemoryDesc => results.sort_by(|a, b| {
                b.memory_usage
                    .unwrap_or(0)
                    .cmp(&a.memory_usage.unwrap_or(0))
            }),
            SortOrder::CpuDesc => results.sort_by(|a, b| {
                let a_cpu = a.total_cpu_time().map(|d| d.as_nanos()).unwrap_or(0);
                let b_cpu = b.total_cpu_time().map(|d| d.as_nanos()).unwrap_or(0);
                b_cpu.cmp(&a_cpu)
            }),
        }

        Ok(results)
    }

    // ── Private ───────────────────────────────────────────────────────────────

    /// Returns `true` when `process` satisfies all active predicates.
    fn matches(&self, process: &ProcessInfo) -> bool {
        if let Some(ref name) = self.name_contains {
            if !process.name.to_lowercase().contains(name) {
                return false;
            }
        }

        if let Some(min_mem) = self.min_memory_bytes {
            if process.memory_usage.map_or(true, |m| m < min_mem) {
                return false;
            }
        }
        if let Some(max_mem) = self.max_memory_bytes {
            if process.memory_usage.map_or(false, |m| m > max_mem) {
                return false;
            }
        }

        if let Some(min_threads) = self.min_threads {
            if process.thread_count.map_or(true, |t| t < min_threads) {
                return false;
            }
        }

        if let Some(parent) = self.parent_pid {
            if process.parent_pid != Some(parent) {
                return false;
            }
        }

        if let Some(ref priority) = self.priority {
            if process.priority_class.as_ref() != Some(priority) {
                return false;
            }
        }

        if let Some(min_cpu) = self.min_cpu_time {
            if process.total_cpu_time().map_or(true, |t| t < min_cpu) {
                return false;
            }
        }
        if let Some(max_cpu) = self.max_cpu_time {
            if process.total_cpu_time().map_or(false, |t| t > max_cpu) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::*;

    fn make_process(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo::basic(pid, name.to_string())
    }

    #[test]
    fn test_filter_builder_name() {
        let filter = ProcessFilter::new().name_contains("test");

        let mut p = make_process(1, "test.exe");
        p.memory_usage = Some(2048);
        p.thread_count = Some(3);
        assert!(filter.matches(&p));

        let p2 = make_process(2, "other.exe");
        assert!(!filter.matches(&p2));
    }

    #[test]
    fn test_filter_memory_range() {
        let filter = ProcessFilter::new().min_memory(1024).max_memory(4096);

        let mut p = make_process(1, "a.exe");
        p.memory_usage = Some(2048);
        assert!(filter.matches(&p));

        let mut too_small = make_process(2, "b.exe");
        too_small.memory_usage = Some(512);
        assert!(!filter.matches(&too_small));

        let mut too_large = make_process(3, "c.exe");
        too_large.memory_usage = Some(8192);
        assert!(!filter.matches(&too_large));
    }

    #[test]
    fn test_filter_priority() {
        let filter = ProcessFilter::new().priority(ProcessPriority::Normal);

        let mut p_normal = make_process(1, "a.exe");
        p_normal.priority_class = Some(ProcessPriority::Normal);
        assert!(filter.matches(&p_normal));

        let mut p_high = make_process(2, "b.exe");
        p_high.priority_class = Some(ProcessPriority::High);
        assert!(!filter.matches(&p_high));

        let p_unknown = make_process(3, "c.exe");
        assert!(!filter.matches(&p_unknown));
    }

    #[test]
    fn test_filter_cpu_time() {
        let min = Duration::from_secs(5);
        let filter = ProcessFilter::new().min_cpu_time(min);

        let mut p_long = make_process(1, "a.exe");
        p_long.user_cpu_time = Some(Duration::from_secs(10));
        p_long.kernel_cpu_time = Some(Duration::from_secs(0));
        assert!(filter.matches(&p_long));

        let mut p_short = make_process(2, "b.exe");
        p_short.user_cpu_time = Some(Duration::from_secs(2));
        p_short.kernel_cpu_time = Some(Duration::from_secs(1));
        assert!(!filter.matches(&p_short));

        let p_none = make_process(3, "c.exe");
        assert!(!filter.matches(&p_none));
    }

    #[test]
    fn test_filter_max_cpu_time_no_data_passes() {
        let max = Duration::from_secs(10);
        let filter = ProcessFilter::new().max_cpu_time(max);

        let p_none = make_process(1, "a.exe");
        assert!(filter.matches(&p_none));
    }

    #[test]
    fn test_sort_order_default() {
        let filter = ProcessFilter::new();
        assert_eq!(filter.sort, SortOrder::None);
    }
}
