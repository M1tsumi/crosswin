use crosswin::prelude::*;
use crosswin::processes::list_processes;

#[tokio::test]
async fn list_processes_smoke() -> Result<()> {
    let processes = list_processes().await?;
    assert!(!processes.is_empty());
    Ok(())
}
