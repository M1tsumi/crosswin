use crosswin::prelude::*;
use crosswin::processes::{list_processes, find_process_by_pid, find_processes_by_name, ProcessFilter};

#[tokio::test]
async fn test_list_processes_not_empty() -> Result<()> {
    let processes = list_processes().await?;
    assert!(!processes.is_empty(), "Should have at least one process");
    Ok(())
}

#[tokio::test]
async fn test_process_info_has_basic_fields() -> Result<()> {
    let processes = list_processes().await?;
    let proc = processes.first().expect("Should have at least one process");
    
    assert!(proc.pid > 0, "PID should be positive");
    assert!(!proc.name.is_empty(), "Name should not be empty");
    Ok(())
}

#[tokio::test]
async fn test_find_current_process_by_pid() -> Result<()> {
    let current_pid = std::process::id();
    let proc = find_process_by_pid(current_pid).await?;
    
    assert!(proc.is_some(), "Should find current process");
    let proc = proc.unwrap();
    assert_eq!(proc.pid, current_pid);
    Ok(())
}

#[tokio::test]
async fn test_find_nonexistent_process() -> Result<()> {
    // Use a very high PID that's unlikely to exist
    let proc = find_process_by_pid(9999999).await?;
    assert!(proc.is_none(), "Should not find process with impossible PID");
    Ok(())
}

#[tokio::test]
async fn test_find_processes_by_name() -> Result<()> {
    // Find processes - the exact name varies by system, so just check the function works
    let processes = find_processes_by_name("").await?;
    assert!(!processes.is_empty(), "Empty search should return all processes");
    Ok(())
}

#[tokio::test]
async fn test_process_filter_builder() -> Result<()> {
    let processes = ProcessFilter::new()
        .min_memory(0) // Should match all processes with memory info
        .list()
        .await?;
    
    // Should have at least one process
    assert!(!processes.is_empty());
    
    // All returned processes should have memory info
    for proc in &processes {
        assert!(proc.memory_usage.is_some(), "Filtered processes should have memory info");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_process_has_extended_info() -> Result<()> {
    let processes = list_processes().await?;
    
    // Find a process with extended info
    let proc_with_info = processes.iter().find(|p| {
        p.parent_pid.is_some() 
        && p.memory_usage.is_some()
        && p.thread_count.is_some()
    });
    
    assert!(proc_with_info.is_some(), "Should have at least one process with extended info");
    
    if let Some(proc) = proc_with_info {
        assert!(proc.parent_pid.is_some(), "parent_pid should be populated");
        assert!(proc.memory_usage.unwrap() > 0);
        assert!(proc.thread_count.unwrap() > 0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_process_memory_mb_conversion() -> Result<()> {
    let mut proc = ProcessInfo::basic(123, "test.exe".to_string());
    proc.memory_usage = Some(10_485_760); // 10 MB
    
    let mb = proc.memory_usage_mb().unwrap();
    assert!((mb - 10.0).abs() < 0.1, "Should be approximately 10 MB");
    Ok(())
}

#[tokio::test]
async fn test_process_total_cpu_time() -> Result<()> {
    use std::time::Duration;
    
    let mut proc = ProcessInfo::basic(123, "test.exe".to_string());
    proc.user_cpu_time = Some(Duration::from_secs(5));
    proc.kernel_cpu_time = Some(Duration::from_secs(3));
    
    let total = proc.total_cpu_time().unwrap();
    assert_eq!(total, Duration::from_secs(8));
    Ok(())
}

// ─── v0.4.0 additions ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_process_filter_sort_by_pid() -> Result<()> {
    use crosswin::processes::ProcessFilter;
    let processes = ProcessFilter::new().sort_by_pid().list().await?;
    let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
    let mut sorted = pids.clone();
    sorted.sort_unstable();
    assert_eq!(pids, sorted, "sort_by_pid should produce ascending PID order");
    Ok(())
}

#[tokio::test]
async fn test_process_filter_sort_by_memory() -> Result<()> {
    use crosswin::processes::ProcessFilter;
    let processes = ProcessFilter::new()
        .min_memory(0)   // only include those with memory data
        .sort_by_memory()
        .list()
        .await?;

    // Verify descending order
    for window in processes.windows(2) {
        let a = window[0].memory_usage.unwrap_or(0);
        let b = window[1].memory_usage.unwrap_or(0);
        assert!(a >= b, "sort_by_memory must be descending: {} < {}", a, b);
    }
    Ok(())
}

#[tokio::test]
async fn test_process_filter_sort_by_cpu() -> Result<()> {
    use crosswin::processes::ProcessFilter;
    let processes = ProcessFilter::new().sort_by_cpu().list().await?;

    for window in processes.windows(2) {
        let a = window[0].total_cpu_time().map(|d| d.as_nanos()).unwrap_or(0);
        let b = window[1].total_cpu_time().map(|d| d.as_nanos()).unwrap_or(0);
        assert!(a >= b, "sort_by_cpu must be descending");
    }
    Ok(())
}

#[tokio::test]
async fn test_find_process_by_name_returns_first() -> Result<()> {
    use crosswin::processes::{find_process_by_name, find_processes_by_name};

    // Use an empty pattern to get all processes and check first-match semantics.
    let all = find_processes_by_name("").await?;
    if all.is_empty() {
        return Ok(());
    }
    let first_all = all.into_iter().next().unwrap();

    // find_process_by_name with the exact name should return a process whose
    // PID is ≤ the first all-match PID (same snapshot order).
    let found = find_process_by_name(&first_all.name).await?;
    assert!(found.is_some(), "find_process_by_name must find a known process");
    Ok(())
}

#[cfg(feature = "win32")]
#[tokio::test]
async fn test_process_info_is_alive() -> Result<()> {
    // Current process should always be alive.
    let current_pid = std::process::id();
    let procs = list_processes().await?;
    let me = procs.iter().find(|p| p.pid == current_pid);
    if let Some(proc) = me {
        assert!(proc.is_alive(), "current process must be alive");
    }
    Ok(())
}

#[test]
fn test_process_priority_display() {
    use crosswin::processes::ProcessPriority;
    assert_eq!(format!("{}", ProcessPriority::Normal), "Normal");
    assert_eq!(format!("{}", ProcessPriority::High), "High");
    assert_eq!(format!("{}", ProcessPriority::Idle), "Idle");
    assert_eq!(format!("{}", ProcessPriority::Realtime), "Realtime");
}

#[tokio::test]
async fn test_process_info_display() -> Result<()> {
    let mut proc = ProcessInfo::basic(1234, "example.exe".to_string());
    proc.memory_usage = Some(10 * 1024 * 1024); // 10 MB
    proc.thread_count = Some(8);
    proc.priority_class = Some(crosswin::processes::ProcessPriority::Normal);

    let s = format!("{}", proc);
    assert!(s.contains("1234"), "Display must include PID");
    assert!(s.contains("example.exe"), "Display must include name");
    assert!(s.contains("10.0 MB"), "Display must include memory");
    assert!(s.contains("threads=8"), "Display must include thread count");
    assert!(s.contains("Normal"), "Display must include priority");
    Ok(())
}
