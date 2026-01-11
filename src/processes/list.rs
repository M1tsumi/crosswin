use crate::error::{CrosswinError, Result};

use super::{ProcessInfo, ProcessPriority};

#[cfg(feature = "win32")]
use windows::Win32::Foundation::{CloseHandle, FILETIME};
#[cfg(feature = "win32")]
use windows::core::PWSTR;
#[cfg(feature = "win32")]
use windows::Win32::System::Threading::{
    OpenProcess,
    QueryFullProcessImageNameW,
    GetProcessTimes,
    GetPriorityClass,
    PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(feature = "win32")]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(feature = "win32")]
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
#[cfg(feature = "win32")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "win32")]
pub async fn list_processes() -> Result<Vec<ProcessInfo>> {
    tokio::task::spawn_blocking(enumerate_processes_blocking)
        .await
        .map_err(|e| CrosswinError::win32(
            "list_processes",
            0,
            format!("join error: {}", e)
        ))?
}

#[cfg(not(feature = "win32"))]
pub async fn list_processes() -> Result<Vec<ProcessInfo>> {
    Ok(Vec::new())
}

#[cfg(feature = "win32")]
fn enumerate_processes_blocking() -> Result<Vec<ProcessInfo>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| CrosswinError::win32(
                "CreateToolhelp32Snapshot",
                e.code().0 as u32,
                format!("{}", e)
            ))?;

        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if let Err(e) = Process32FirstW(snapshot, &mut entry) {
            let _ = CloseHandle(snapshot);
            return Err(CrosswinError::win32(
                "Process32FirstW",
                e.code().0 as u32,
                format!("{}", e)
            ));
        }

        let mut processes = Vec::new();

        loop {
            let pid = entry.th32ProcessID;

            // Skip the idle/system process with PID 0; most callers expect positive PIDs
            if pid == 0 {
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
                continue;
            }

            let name = exe_name_from_entry(&entry);
            let parent_pid = Some(entry.th32ParentProcessID);
            let thread_count = Some(entry.cntThreads);

            let (executable_path, memory_info, cpu_times, priority) = query_process_details(pid);

            processes.push(ProcessInfo {
                pid,
                name,
                executable_path,
                parent_pid,
                memory_usage: memory_info.map(|m| m.working_set_size),
                user_cpu_time: cpu_times.as_ref().map(|t| t.user_time),
                kernel_cpu_time: cpu_times.as_ref().map(|t| t.kernel_time),
                thread_count,
                priority_class: priority,
                creation_time: cpu_times.and_then(|t| Some(t.creation_time)),
            });

            if Process32NextW(snapshot, &mut entry).is_err() {
                break;
            }
        }

        let _ = CloseHandle(snapshot);
        Ok(processes)
    }
}

#[cfg(feature = "win32")]
fn exe_name_from_entry(entry: &PROCESSENTRY32W) -> String {
    let raw = &entry.szExeFile;
    let nul_pos = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
    String::from_utf16_lossy(&raw[..nul_pos])
}

#[cfg(feature = "win32")]
fn query_process_details(
    pid: u32,
) -> (
    Option<String>,
    Option<super::MemoryInfo>,
    Option<super::CpuTimes>,
    Option<ProcessPriority>,
) {
    unsafe {
        let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => handle,
            Err(_) => return (None, None, None, None),
        };

        let executable_path = query_executable_path_from_handle(process_handle);
        let memory_info = query_memory_info(process_handle);
        let cpu_times = query_cpu_times(process_handle);
        let priority = query_priority_class(process_handle);

        let _ = CloseHandle(process_handle);

        (executable_path, memory_info, cpu_times, priority)
    }
}

#[cfg(feature = "win32")]
fn query_executable_path_from_handle(process_handle: windows::Win32::Foundation::HANDLE) -> Option<String> {
    unsafe {
        let mut buffer = [0u16; 512];
        let mut size = buffer.len() as u32;

        let result = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        if result.is_err() || size == 0 {
            return None;
        }

        let slice = &buffer[..size as usize];
        Some(String::from_utf16_lossy(slice))
    }
}

#[cfg(feature = "win32")]
fn query_memory_info(process_handle: windows::Win32::Foundation::HANDLE) -> Option<super::MemoryInfo> {
    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        if GetProcessMemoryInfo(
            process_handle,
            &mut counters,
            counters.cb,
        )
        .is_ok()
        {
            Some(super::MemoryInfo {
                working_set_size: counters.WorkingSetSize as u64,
                peak_working_set_size: counters.PeakWorkingSetSize as u64,
                page_file_usage: counters.PagefileUsage as u64,
                peak_page_file_usage: counters.PeakPagefileUsage as u64,
            })
        } else {
            None
        }
    }
}

#[cfg(feature = "win32")]
fn query_cpu_times(process_handle: windows::Win32::Foundation::HANDLE) -> Option<super::CpuTimes> {
    unsafe {
        let mut creation_time: FILETIME = std::mem::zeroed();
        let mut exit_time: FILETIME = std::mem::zeroed();
        let mut kernel_time: FILETIME = std::mem::zeroed();
        let mut user_time: FILETIME = std::mem::zeroed();

        if GetProcessTimes(
            process_handle,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )
        .is_ok()
        {
            Some(super::CpuTimes {
                user_time: filetime_to_duration(&user_time),
                kernel_time: filetime_to_duration(&kernel_time),
                creation_time: filetime_to_systemtime(&creation_time),
                exit_time: if exit_time.dwLowDateTime == 0 && exit_time.dwHighDateTime == 0 {
                    None
                } else {
                    Some(filetime_to_systemtime(&exit_time))
                },
            })
        } else {
            None
        }
    }
}

#[cfg(feature = "win32")]
fn query_priority_class(process_handle: windows::Win32::Foundation::HANDLE) -> Option<ProcessPriority> {
    unsafe {
        let priority = GetPriorityClass(process_handle);
        if priority == 0 {
            None
        } else {
            ProcessPriority::from_windows_constant(priority)
        }
    }
}

#[cfg(feature = "win32")]
fn filetime_to_duration(ft: &FILETIME) -> Duration {
    // FILETIME is in 100-nanosecond intervals
    let intervals = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
    let nanos = intervals * 100;
    Duration::from_nanos(nanos)
}

#[cfg(feature = "win32")]
fn filetime_to_systemtime(ft: &FILETIME) -> SystemTime {
    // FILETIME epoch is January 1, 1601
    // Unix epoch is January 1, 1970
    // Difference is 11644473600 seconds
    const FILETIME_TO_UNIX_EPOCH: u64 = 11644473600;
    
    let intervals = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
    let seconds = intervals / 10_000_000;
    let nanos = ((intervals % 10_000_000) * 100) as u32;
    
    if seconds >= FILETIME_TO_UNIX_EPOCH {
        UNIX_EPOCH + Duration::new(seconds - FILETIME_TO_UNIX_EPOCH, nanos)
    } else {
        UNIX_EPOCH
    }
}
