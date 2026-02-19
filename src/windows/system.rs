use crate::error::Result;
#[cfg(not(feature = "win32"))]
use crate::error::CrosswinError;
use std::fmt;
use std::time::{Duration, SystemTime};
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::MEMORYSTATUSEX;
#[cfg(feature = "win32")]
use windows::Win32::System::SystemInformation::GetTickCount64;

// ─── SystemInfo ───────────────────────────────────────────────────────────────

/// System-wide hardware and memory information.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemInfo {
    /// Logical CPU count.
    pub cpu_count: usize,
    /// Virtual memory page size in bytes.
    pub page_size: usize,
    /// Total installed physical memory in bytes.
    pub total_physical_bytes: Option<u64>,
    /// Currently available physical memory in bytes.
    pub available_physical_bytes: Option<u64>,
    /// Memory load as a percentage (0–100).
    pub memory_load_percent: Option<u32>,
    /// Total virtual address space in bytes.
    pub total_virtual_bytes: Option<u64>,
    /// Available virtual address space in bytes.
    pub available_virtual_bytes: Option<u64>,
}

impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CPUs: {}", self.cpu_count)?;
        if let (Some(total), Some(avail)) = (self.total_physical_bytes, self.available_physical_bytes) {
            write!(
                f,
                "  RAM: {:.1}/{:.1} GB",
                avail as f64 / (1024.0_f64).powi(3),
                total as f64 / (1024.0_f64).powi(3)
            )?;
        } else if let Some(total) = self.total_physical_bytes {
            write!(f, "  RAM total: {:.1} GB", total as f64 / (1024.0_f64).powi(3))?;
        }
        if let Some(load) = self.memory_load_percent {
            write!(f, "  load: {}%", load)?;
        }
        Ok(())
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Populate a `SystemInfo` from a `MEMORYSTATUSEX` struct.
#[cfg(feature = "win32")]
unsafe fn system_info_from_mem_status(mem: &MEMORYSTATUSEX, cpu_count: usize, page_size: usize) -> SystemInfo {
    SystemInfo {
        cpu_count,
        page_size,
        total_physical_bytes: Some(mem.ullTotalPhys),
        available_physical_bytes: Some(mem.ullAvailPhys),
        memory_load_percent: Some(mem.dwMemoryLoad),
        total_virtual_bytes: Some(mem.ullTotalVirtual),
        available_virtual_bytes: Some(mem.ullAvailVirtual),
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Retrieve full system information including CPU layout and memory.
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

            if GlobalMemoryStatusEx(&mut mem).is_ok() {
                Ok(system_info_from_mem_status(&mem, cpu_count, page_size))
            } else {
                Ok(SystemInfo {
                    cpu_count,
                    page_size,
                    total_physical_bytes: None,
                    available_physical_bytes: None,
                    memory_load_percent: None,
                    total_virtual_bytes: None,
                    available_virtual_bytes: None,
                })
            }
        }
    }

    #[cfg(not(feature = "win32"))]
    {
        Ok(SystemInfo {
            cpu_count: num_cpus::get(),
            page_size: 4096,
            total_physical_bytes: None,
            available_physical_bytes: None,
            memory_load_percent: None,
            total_virtual_bytes: None,
            available_virtual_bytes: None,
        })
    }
}

/// Lightweight call that queries only `GlobalMemoryStatusEx` — useful for
/// polling memory pressure without a full `GetSystemInfo` roundtrip.
pub fn get_memory_status() -> Result<SystemInfo> {
    #[cfg(feature = "win32")]
    {
        unsafe {
            let mut mem: MEMORYSTATUSEX = std::mem::zeroed();
            mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            GlobalMemoryStatusEx(&mut mem).map_err(|e| {
                crate::error::CrosswinError::win32(
                    "GlobalMemoryStatusEx",
                    e.code().0 as u32,
                    e.to_string(),
                )
            })?;
            Ok(system_info_from_mem_status(&mem, num_cpus::get(), 4096))
        }
    }

    #[cfg(not(feature = "win32"))]
    {
        Err(CrosswinError::invalid_parameter(
            "platform",
            "Not supported on this platform",
        ))
    }
}

/// Return system uptime as `Duration` since boot.
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

/// Return the approximate system boot time as a `SystemTime`.
pub fn get_boot_time() -> Result<SystemTime> {
    let uptime = get_system_uptime()?;
    Ok(SystemTime::now() - uptime)
}

