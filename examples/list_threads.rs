use crosswin::prelude::*;

/// Example: List all threads of a given process.
///
/// Usage:
///   cargo run --example list_threads -- <pid>
///   cargo run --example list_threads -- <pid>   (defaults to current PID)

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let pid: u32 = if let Some(s) = args.get(1) {
        match s.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Usage: list_threads [pid]");
                std::process::exit(1);
            }
        }
    } else {
        std::process::id()
    };

    println!("Threads in process PID {}:\n", pid);

    let threads = list_threads(pid).await?;

    if threads.is_empty() {
        println!("  (none found — process may not exist or access was denied)");
        return Ok(());
    }

    println!("{:<10} {:<10} {:<12}", "Thread ID", "PID", "Base Priority");
    println!("{:-<10} {:-<10} {:-<12}", "", "", "");

    for t in &threads {
        println!("{:<10} {:<10} {:<12}", t.thread_id, t.process_id, t.base_priority);
    }

    println!("\nTotal: {} thread(s)", threads.len());
    Ok(())
}
