use crate::error::Result;
#[cfg(not(feature = "win32"))]
use crate::error::CrosswinError;
use std::time::{Duration, SystemTime};
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::MEMORYSTATUSEX;
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::GetTickCount64;

/// Basic system information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub cpu_count: usize,
    pub page_size: usize,
    pub total_physical_bytes: Option<u64>,
}

/// Retrieve system information (stubbed on non-Windows)
pub fn get_system_info() -> Result<SystemInfo> {
    #[cfg(feature = "win32")]
    {
        unsafe {
            let mut sysinfo: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut sysinfo);
            let cpu_count = sysinfo.dwNumberOfProcessors as usize;
            let page_size = sysinfo.dwPageSize as usize;

            let mut mem: MEMORYSTATUSEX = std::mem::zeroed();
            mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            let total_physical = match GlobalMemoryStatusEx(&mut mem) {
                Ok(_) => Some(mem.ullTotalPhys as u64),
                Err(_) => None,
            };

            Ok(SystemInfo {
                cpu_count,
                page_size,
                total_physical_bytes: total_physical,
            })
        }
    }

    #[cfg(not(feature = "win32"))]
    {
        Ok(SystemInfo {
            cpu_count: num_cpus::get(),
            page_size: 4096,
            total_physical_bytes: None,
        })
    }
}

/// Return system uptime as `Duration` since boot (stub)
pub fn get_system_uptime() -> Result<Duration> {
    #[cfg(feature = "win32")]
    {
        unsafe {
            let ms = GetTickCount64();
            Ok(Duration::from_millis(ms))
        }
    }

    #[cfg(not(feature = "win32"))]
    {
        Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
    }
}

/// Return the system boot time (approx) as `SystemTime` (stub)
pub fn get_boot_time() -> Result<SystemTime> {
    let uptime = get_system_uptime()?;
    Ok(SystemTime::now() - uptime)
}
