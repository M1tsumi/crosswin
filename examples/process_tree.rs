use crosswin::prelude::*;
use crosswin::processes::list_processes;
use std::collections::HashMap;

/// Example: Display a process tree showing parent-child relationships
/// 
/// Usage: cargo run --example process_tree

#[tokio::main]
async fn main() -> Result<()> {
    println!("Building process tree...\n");

    let processes = list_processes().await?;

    // Build a map of PID -> ProcessInfo
    let process_map: HashMap<u32, &ProcessInfo> = 
        processes.iter().map(|p| (p.pid, p)).collect();

    // Build parent -> children mapping
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for proc in &processes {
        if let Some(parent_pid) = proc.parent_pid {
            children.entry(parent_pid).or_insert_with(Vec::new).push(proc.pid);
        }
    }

    // Find root processes (those whose parent doesn't exist or is 0)
    let mut roots: Vec<u32> = processes
        .iter()
        .filter(|p| {
            p.parent_pid.is_none() 
            || p.parent_pid == Some(0) 
            || !process_map.contains_key(&p.parent_pid.unwrap())
        })
        .map(|p| p.pid)
        .collect();

    roots.sort_unstable();

    // Print the tree
    for root_pid in roots {
        if let Some(proc) = process_map.get(&root_pid) {
            print_process_tree(proc, &process_map, &children, "", true);
        }
    }

    Ok(())
}

fn print_process_tree(
    proc: &ProcessInfo,
    process_map: &HashMap<u32, &ProcessInfo>,
    children: &HashMap<u32, Vec<u32>>,
    prefix: &str,
    is_last: bool,
) {
    // Print current process
    let branch = if is_last { "└─" } else { "├─" };
    let memory_str = proc.memory_usage
        .map(|m| format!("{:.1} MB", m as f64 / (1024.0 * 1024.0)))
        .unwrap_or_else(|| "?".to_string());
    
    let threads_str = proc.thread_count
        .map(|t| format!("{} threads", t))
        .unwrap_or_else(|| "?".to_string());

    println!(
        "{}{} [{}] {} ({}, {})",
        prefix, branch, proc.pid, proc.name, memory_str, threads_str
    );

    // Print children
    if let Some(child_pids) = children.get(&proc.pid) {
        let child_prefix = if is_last {
            format!("{}  ", prefix)
        } else {
            format!("{}│ ", prefix)
        };

        for (i, &child_pid) in child_pids.iter().enumerate() {
            let is_last_child = i == child_pids.len() - 1;
            if let Some(child_proc) = process_map.get(&child_pid) {
                print_process_tree(
                    child_proc,
                    process_map,
                    children,
                    &child_prefix,
                    is_last_child,
                );
            }
        }
    }
}
