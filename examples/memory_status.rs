use crosswin::prelude::*;
use std::time::Duration;

/// Example: Poll live memory pressure using `get_memory_status`.
///
/// Usage:
///   cargo run --example memory_status              # polls every second, 10 times
///   cargo run --example memory_status -- 5 2000    # 5 samples, 2 s apart

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let samples: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10);
    let interval_ms: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1000);

    println!("Memory status — {} samples every {} ms\n", samples, interval_ms);
    println!("{:<6} {:>12} {:>12} {:>8}", "#", "Avail (GB)", "Total (GB)", "Load %");
    println!("{:-<6} {:-<12} {:-<12} {:-<8}", "", "", "", "");

    for i in 1..=samples {
        let info = get_memory_status()?;

        let total_gb = info
            .total_physical_bytes
            .map(|b| b as f64 / (1024.0_f64).powi(3));
        let avail_gb = info
            .available_physical_bytes
            .map(|b| b as f64 / (1024.0_f64).powi(3));
        let load = info.memory_load_percent;

        println!(
            "{:<6} {:>12} {:>12} {:>8}",
            i,
            avail_gb.map(|v| format!("{:.2}", v)).unwrap_or("N/A".to_string()),
            total_gb.map(|v| format!("{:.2}", v)).unwrap_or("N/A".to_string()),
            load.map(|p| format!("{}%", p)).unwrap_or("N/A".to_string()),
        );

        if i < samples {
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }
    }

    Ok(())
}
