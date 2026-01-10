use crosswin::prelude::*;
use crosswin::processes::list_processes;

/// Example: Find processes consuming the most CPU time
/// 
/// Usage: cargo run --example top_cpu [count]
/// Example: cargo run --example top_cpu 10

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    println!("Top {} processes by CPU time:\n", count);

    let mut processes = list_processes().await?;

    // Sort by total CPU time (descending)
    processes.sort_by(|a, b| {
        let a_time = a.total_cpu_time().map(|d| d.as_secs_f64()).unwrap_or(0.0);
        let b_time = b.total_cpu_time().map(|d| d.as_secs_f64()).unwrap_or(0.0);
        b_time.partial_cmp(&a_time).unwrap()
    });

    // Print header
    println!("{:<8} {:<30} {:>12} {:>12} {:>10}", 
        "PID", "Name", "User Time", "Kernel Time", "Total");
    println!("{:-<8} {:-<30} {:-<12} {:-<12} {:-<10}", 
        "", "", "", "", "");

    // Print top processes
    for proc in processes.iter().take(count) {
        let user_time = proc.user_cpu_time
            .map(|d| format!("{:.2}s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        
        let kernel_time = proc.kernel_cpu_time
            .map(|d| format!("{:.2}s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        
        let total_time = proc.total_cpu_time()
            .map(|d| format!("{:.2}s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{:<8} {:<30} {:>12} {:>12} {:>10}",
            proc.pid,
            // Truncate long names
            if proc.name.len() > 30 {
                format!("{}...", &proc.name[..27])
            } else {
                proc.name.clone()
            },
            user_time,
            kernel_time,
            total_time
        );
    }

    Ok(())
}
