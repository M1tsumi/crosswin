use crosswin::prelude::*;
use crosswin::processes::find_processes_by_name;

/// Example: Find and terminate a process by name
/// 
/// Usage: cargo run --example kill_process <process_name>
/// Example: cargo run --example kill_process notepad

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <process_name>", args[0]);
        eprintln!("Example: {} notepad", args[0]);
        std::process::exit(1);
    }

    let process_name = &args[1];
    println!("Searching for processes matching '{}'...", process_name);

    let processes = find_processes_by_name(process_name).await?;

    if processes.is_empty() {
        println!("No processes found matching '{}'", process_name);
        return Ok(());
    }

    println!("Found {} matching process(es):", processes.len());
    for (i, proc) in processes.iter().enumerate() {
        println!("  {}. PID {} - {}", i + 1, proc.pid, proc.name);
    }

    println!("\nAttempting to terminate process(es)...");

    for proc in &processes {
        match Process::open(proc.pid, ProcessAccess::Terminate) {
            Ok(process) => {
                match process.terminate(0) {
                    Ok(_) => println!("✓ Successfully terminated PID {}", proc.pid),
                    Err(e) => println!("✗ Failed to terminate PID {}: {}", proc.pid, e),
                }
            }
            Err(e) => {
                println!("✗ Failed to open PID {}: {}", proc.pid, e);
            }
        }
    }

    Ok(())
}
