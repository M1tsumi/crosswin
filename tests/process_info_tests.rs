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
        assert!(proc.parent_pid.unwrap() >= 0);
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
