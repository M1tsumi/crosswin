use crate::error::{CrosswinError, Result};

use super::ProcessInfo;

#[cfg(feature = "win32")]
use windows::Win32::Foundation::CloseHandle;
#[cfg(feature = "win32")]
use windows::core::PWSTR;
#[cfg(feature = "win32")]
use windows::Win32::System::Threading::{
    OpenProcess,
    QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(feature = "win32")]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

#[cfg(feature = "win32")]
pub async fn list_processes() -> Result<Vec<ProcessInfo>> {
    tokio::task::spawn_blocking(enumerate_processes_blocking)
        .await
        .map_err(|e| CrosswinError::Win32(format!("join error in list_processes: {e}")))?
}

#[cfg(not(feature = "win32"))]
pub async fn list_processes() -> Result<Vec<ProcessInfo>> {
    Ok(Vec::new())
}

#[cfg(feature = "win32")]
fn enumerate_processes_blocking() -> Result<Vec<ProcessInfo>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| CrosswinError::Win32(format!("CreateToolhelp32Snapshot failed: {e}")))?;

        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if let Err(e) = Process32FirstW(snapshot, &mut entry) {
            let _ = CloseHandle(snapshot);
            return Err(CrosswinError::Win32(format!("Process32FirstW failed: {e}")));
        }

        let mut processes = Vec::new();

        loop {
            let pid = entry.th32ProcessID;
            let name = exe_name_from_entry(&entry);

            let executable_path = query_executable_path(pid);

            processes.push(ProcessInfo {
                pid,
                name,
                executable_path,
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
fn query_executable_path(pid: u32) -> Option<String> {
    unsafe {
        let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => handle,
            Err(_) => return None,
        };

        let mut buffer = [0u16; 512];
        let mut size = buffer.len() as u32;

        let result = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        let _ = CloseHandle(process_handle);

        if result.is_err() || size == 0 {
            return None;
        }

        let slice = &buffer[..size as usize];
        Some(String::from_utf16_lossy(slice))
    }
}
