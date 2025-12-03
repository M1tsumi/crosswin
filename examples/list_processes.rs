use crosswin::prelude::*;
use crosswin::processes::list_processes;

#[tokio::main]
async fn main() -> Result<()> {
    let processes = list_processes().await?;

    for process in processes {
        println!(
            "pid={} name={} path={:?}",
            process.pid, process.name, process.executable_path
        );
    }

    Ok(())
}
