use crosswin::prelude::*;

/// Example: Inspect all top-level windows, with optional process name filtering.
///
/// Usage:
///   cargo run --example window_inspector
///   cargo run --example window_inspector -- notepad
///   cargo run --example window_inspector -- --visible

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let name_filter = args.get(1).filter(|s| s.as_str() != "--visible").cloned();
    let visible_only = args.iter().any(|a| a == "--visible");

    // Build filter
    let mut filter = WindowFilter::new().visible_only(visible_only);
    if let Some(ref name) = name_filter {
        // Find the PID(s) matching the process name, then filter windows by PID.
        let procs = find_processes_by_name(name).await?;
        if procs.is_empty() {
            eprintln!("No process found matching '{}'", name);
            return Ok(());
        }
        // For simplicity take the first matching PID.
        let pid = procs[0].pid;
        filter = filter.process_id(pid);
        println!("Showing windows for '{}' (PID {}):\n", procs[0].name, pid);
    } else {
        println!("Showing all {} windows:\n", if visible_only { "visible" } else { "top-level" });
    }

    let windows = filter.list().await?;

    if windows.is_empty() {
        println!("  (no matching windows found)");
        return Ok(());
    }

    println!(
        "{:<18} {:<40} {:<20} {:<16}",
        "HWND", "Title", "Class", "Size / Pos"
    );
    println!("{:-<18} {:-<40} {:-<20} {:-<16}", "", "", "", "");

    for w in &windows {
        let title = if w.title.len() > 38 {
            format!("{}…", &w.title[..38])
        } else {
            w.title.clone()
        };
        let class = w.class_name.as_deref().unwrap_or("?");
        let class_str = if class.len() > 18 { &class[..18] } else { class };
        let geom = match (w.width, w.height, w.x, w.y) {
            (Some(w), Some(h), Some(x), Some(y)) => format!("{}×{} @{},{}", w, h, x, y),
            (Some(w), Some(h), _, _) => format!("{}×{}", w, h),
            _ => "?".to_string(),
        };
        println!(
            "{:<18} {:<40} {:<20} {:<16}",
            format!("0x{:X}", w.hwnd),
            title,
            class_str,
            geom,
        );
    }

    println!("\nTotal: {} window(s)", windows.len());
    Ok(())
}
