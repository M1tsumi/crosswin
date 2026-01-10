use crate::error::Result;
use super::{ProcessInfo, list_processes};

/// Find a process by its PID
pub async fn find_process_by_pid(pid: u32) -> Result<Option<ProcessInfo>> {
    let processes = list_processes().await?;
    Ok(processes.into_iter().find(|p| p.pid == pid))
}

/// Find all processes matching a name (case-insensitive)
pub async fn find_processes_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    let processes = list_processes().await?;
    let name_lower = name.to_lowercase();
    Ok(processes
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&name_lower))
        .collect())
}

/// Find the first process matching a name (case-insensitive)
pub async fn find_process_by_name(name: &str) -> Result<Option<ProcessInfo>> {
    let mut processes = find_processes_by_name(name).await?;
    Ok(processes.pop())
}

/// Builder for filtering processes with multiple criteria
#[derive(Default, Debug, Clone)]
pub struct ProcessFilter {
    name_contains: Option<String>,
    min_memory_bytes: Option<u64>,
    max_memory_bytes: Option<u64>,
    min_threads: Option<u32>,
    parent_pid: Option<u32>,
}

impl ProcessFilter {
    /// Create a new process filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by name (case-insensitive substring match)
    pub fn name_contains<S: Into<String>>(mut self, name: S) -> Self {
        self.name_contains = Some(name.into().to_lowercase());
        self
    }

    /// Filter by minimum memory usage in bytes
    pub fn min_memory(mut self, bytes: u64) -> Self {
        self.min_memory_bytes = Some(bytes);
        self
    }

    /// Filter by maximum memory usage in bytes
    pub fn max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = Some(bytes);
        self
    }

    /// Filter by minimum thread count
    pub fn min_threads(mut self, count: u32) -> Self {
        self.min_threads = Some(count);
        self
    }

    /// Filter by parent process ID
    pub fn parent_pid(mut self, pid: u32) -> Self {
        self.parent_pid = Some(pid);
        self
    }

    /// Execute the filter and return matching processes
    pub async fn list(self) -> Result<Vec<ProcessInfo>> {
        let processes = list_processes().await?;
        Ok(processes.into_iter().filter(|p| self.matches(p)).collect())
    }

    /// Check if a process matches the filter criteria
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

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filter = ProcessFilter::new()
            .name_contains("test")
            .min_memory(1024)
            .min_threads(2);

        // Create a test process
        let mut process = ProcessInfo::basic(123, "test.exe".to_string());
        process.memory_usage = Some(2048);
        process.thread_count = Some(3);

        assert!(filter.matches(&process));

        // Test failing conditions
        let mut process2 = ProcessInfo::basic(124, "other.exe".to_string());
        process2.memory_usage = Some(2048);
        process2.thread_count = Some(3);
        assert!(!filter.matches(&process2)); // Name doesn't match
    }
}
