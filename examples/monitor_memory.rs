use crosswin::prelude::*;
use crosswin::processes::find_process_by_name;
use std::time::Duration;
use tokio::time::sleep;

/// Example: Monitor memory usage of a process over time
/// 
/// Usage: cargo run --example monitor_memory <process_name>
/// Example: cargo run --example monitor_memory chrome

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <process_name>", args[0]);
        eprintln!("Example: {} chrome", args[0]);
        std::process::exit(1);
    }

    let process_name = &args[1];
    println!("Monitoring memory usage for '{}' (Ctrl+C to stop)", process_name);
    println!("{:>8} | {:>12} | {:>12} | {:>12}", "Time", "Working Set", "Peak WS", "Page File");
    println!("{:-<8}-+-{:-<12}-+-{:-<12}-+-{:-<12}", "", "", "", "");

    let mut elapsed = 0u64;

    loop {
        match find_process_by_name(process_name).await? {
            Some(proc_info) => {
                // Open the process to get detailed memory info
                match Process::open(proc_info.pid, ProcessAccess::QueryInformation) {
                    Ok(process) => {
                        match process.get_memory_info() {
                            Ok(mem) => {
                                println!(
                                    "{:>7}s | {:>10.2} MB | {:>10.2} MB | {:>10.2} MB",
                                    elapsed,
                                    mem.working_set_size as f64 / (1024.0 * 1024.0),
                                    mem.peak_working_set_size as f64 / (1024.0 * 1024.0),
                                    mem.page_file_usage as f64 / (1024.0 * 1024.0),
                                );
                            }
                            Err(e) => {
                                eprintln!("Error getting memory info: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error opening process: {}", e);
                    }
                }
            }
            None => {
                println!("{:>7}s | Process not found", elapsed);
            }
        }

        sleep(Duration::from_secs(2)).await;
        elapsed += 2;
    }
}
