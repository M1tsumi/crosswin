use crosswin::prelude::*;

#[tokio::test]
async fn test_open_current_process() -> Result<()> {
    let process = Process::current()?;
    assert!(process.pid() > 0);
    Ok(())
}

#[tokio::test]
async fn test_open_process_by_pid() -> Result<()> {
    let current_pid = std::process::id();
    let process = Process::open(current_pid, ProcessAccess::QueryLimitedInformation)?;
    assert_eq!(process.pid(), current_pid);
    Ok(())
}

#[tokio::test]
async fn test_open_invalid_process() {
    let result = Process::open(9999999, ProcessAccess::QueryLimitedInformation);
    assert!(result.is_err(), "Should fail to open non-existent process");
}

#[tokio::test]
async fn test_get_memory_info() -> Result<()> {
    let current_pid = std::process::id();
    let process = Process::open(current_pid, ProcessAccess::QueryInformation)?;
    
    let mem_info = process.get_memory_info()?;
    assert!(mem_info.working_set_size > 0, "Working set should be positive");
    assert!(mem_info.peak_working_set_size >= mem_info.working_set_size);
    Ok(())
}

#[tokio::test]
async fn test_get_priority() -> Result<()> {
    let current_pid = std::process::id();
    let process = Process::open(current_pid, ProcessAccess::QueryInformation)?;
    
    let priority = process.get_priority()?;
    // Most processes run at Normal priority
    println!("Process priority: {:?}", priority);
    Ok(())
}

#[tokio::test]
async fn test_set_priority() -> Result<()> {
    let current_pid = std::process::id();
    let process = Process::open(current_pid, ProcessAccess::SetInformation)?;
    
    // Try to set priority to the same value (should not fail)
    let current_priority = process.get_priority().unwrap_or(ProcessPriority::Normal);
    process.set_priority(current_priority)?;
    Ok(())
}

// Note: Skipping terminate, suspend, resume tests as they would affect the test process itself
// These should be tested manually or with separate test processes

#[test]
fn test_process_priority_conversion() {
    // Test all priority values can round-trip
    let priorities = [
        ProcessPriority::Idle,
        ProcessPriority::BelowNormal,
        ProcessPriority::Normal,
        ProcessPriority::AboveNormal,
        ProcessPriority::High,
        ProcessPriority::Realtime,
    ];
    
    for priority in &priorities {
        let constant = priority.to_windows_constant();
        let restored = ProcessPriority::from_windows_constant(constant);
        assert_eq!(restored, Some(*priority), "Priority should round-trip: {:?}", priority);
    }
}
