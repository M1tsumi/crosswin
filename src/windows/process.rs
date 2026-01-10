use crate::error::{CrosswinError, Result};
use crate::windows::handles::ProcessHandle;
use crate::processes::{MemoryInfo, ProcessPriority};

#[cfg(feature = "win32")]
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0, WAIT_TIMEOUT};
#[cfg(feature = "win32")]
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, GetCurrentProcess, GetExitCodeProcess,
    WaitForSingleObject, SetPriorityClass, GetPriorityClass,
    PROCESS_TERMINATE, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_INFORMATION, PROCESS_SUSPEND_RESUME, PROCESS_ALL_ACCESS,
    INFINITE,
};
#[cfg(feature = "win32")]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
};
#[cfg(feature = "win32")]
use windows::Win32::System::Threading::{OpenThread, SuspendThread, ResumeThread, THREAD_SUSPEND_RESUME};
#[cfg(feature = "win32")]
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

/// Access rights for opening a process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAccess {
    /// Query limited information
    QueryLimitedInformation,
    /// Query full information
    QueryInformation,
    /// Terminate the process
    Terminate,
    /// Set process information
    SetInformation,
    /// Suspend/resume threads
    SuspendResume,
    /// All access rights
    AllAccess,
}

#[cfg(feature = "win32")]
impl ProcessAccess {
    fn to_windows_flags(&self) -> u32 {
        match self {
            ProcessAccess::QueryLimitedInformation => PROCESS_QUERY_LIMITED_INFORMATION.0,
            ProcessAccess::QueryInformation => PROCESS_QUERY_INFORMATION.0,
            ProcessAccess::Terminate => PROCESS_TERMINATE.0,
            ProcessAccess::SetInformation => PROCESS_SET_INFORMATION.0,
            ProcessAccess::SuspendResume => PROCESS_SUSPEND_RESUME.0,
            ProcessAccess::AllAccess => PROCESS_ALL_ACCESS.0,
        }
    }
}

/// Represents an opened process with various operations
#[derive(Debug)]
pub struct Process {
    handle: Option<ProcessHandle>,
    pid: u32,
}

impl Process {
    /// Get the current process
    pub fn current() -> Result<Self> {
        #[cfg(feature = "win32")]
        {
            unsafe {
                let handle = GetCurrentProcess();
                let pid = windows::Win32::System::Threading::GetCurrentProcessId();
                Ok(Process {
                    handle: Some(ProcessHandle::from_windows_handle(handle)),
                    pid,
                })
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Ok(Process {
                handle: None,
                pid: 0,
            })
        }
    }

    /// Open a process by PID with specified access rights
    pub fn open(pid: u32, access: ProcessAccess) -> Result<Self> {
        #[cfg(feature = "win32")]
        {
            unsafe {
                let handle = OpenProcess(
                    windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS(access.to_windows_flags()),
                    false,
                    pid,
                )
                .map_err(|e| {
                    if e.code().0 == 0x80070005u32 as i32 {
                        // ERROR_ACCESS_DENIED
                        CrosswinError::access_denied("process", Some(pid))
                    } else if e.code().0 == 0x80070057u32 as i32 {
                        // ERROR_INVALID_PARAMETER (process doesn't exist)
                        CrosswinError::process_not_found(pid)
                    } else {
                        CrosswinError::win32("OpenProcess", e.code().0 as u32, e.to_string())
                    }
                })?;

                Ok(Process {
                    handle: Some(ProcessHandle::from_windows_handle(handle)),
                    pid,
                })
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Ok(Process {
                handle: None,
                pid,
            })
        }
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get the process handle (if available)
    pub fn handle(&self) -> Option<&ProcessHandle> {
        self.handle.as_ref()
    }

    /// Terminate the process with the given exit code
    pub fn terminate(&self, exit_code: u32) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            let handle = self.handle.as_ref()
                .ok_or_else(|| CrosswinError::invalid_parameter("handle", "Process handle is not available"))?;
            
            unsafe {
                TerminateProcess(handle.as_handle().as_windows_handle(), exit_code)
                    .map_err(|e| CrosswinError::win32(
                        "TerminateProcess",
                        e.code().0 as u32,
                        e.to_string()
                    ))?;
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Suspend all threads in the process
    pub fn suspend(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)
                    .map_err(|e| CrosswinError::win32(
                        "CreateToolhelp32Snapshot",
                        e.code().0 as u32,
                        e.to_string()
                    ))?;

                let mut entry = THREADENTRY32::default();
                entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

                if Thread32First(snapshot, &mut entry).is_ok() {
                    loop {
                        if entry.th32OwnerProcessID == self.pid {
                            if let Ok(thread_handle) = OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID) {
                                let _ = SuspendThread(thread_handle);
                                let _ = CloseHandle(thread_handle);
                            }
                        }

                        if Thread32Next(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }

                let _ = CloseHandle(snapshot);
                Ok(())
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Resume all threads in the process
    pub fn resume(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            unsafe {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)
                    .map_err(|e| CrosswinError::win32(
                        "CreateToolhelp32Snapshot",
                        e.code().0 as u32,
                        e.to_string()
                    ))?;

                let mut entry = THREADENTRY32::default();
                entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

                if Thread32First(snapshot, &mut entry).is_ok() {
                    loop {
                        if entry.th32OwnerProcessID == self.pid {
                            if let Ok(thread_handle) = OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID) {
                                let _ = ResumeThread(thread_handle);
                                let _ = CloseHandle(thread_handle);
                            }
                        }

                        if Thread32Next(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }

                let _ = CloseHandle(snapshot);
                Ok(())
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Wait for the process to exit, with an optional timeout in milliseconds
    /// Returns the exit code if the process exits, or None if timeout occurs
    pub fn wait_for_exit(&self, timeout_ms: Option<u32>) -> Result<Option<u32>> {
        #[cfg(feature = "win32")]
        {
            let handle = self.handle.as_ref()
                .ok_or_else(|| CrosswinError::invalid_parameter("handle", "Process handle is not available"))?;
            
            unsafe {
                let timeout = timeout_ms.unwrap_or(INFINITE);
                let wait_result = WaitForSingleObject(handle.as_handle().as_windows_handle(), timeout);

                match wait_result {
                    WAIT_OBJECT_0 => {
                        let mut exit_code = 0u32;
                        GetExitCodeProcess(handle.as_handle().as_windows_handle(), &mut exit_code)
                            .map_err(|e| CrosswinError::win32(
                                "GetExitCodeProcess",
                                e.code().0 as u32,
                                e.to_string()
                            ))?;
                        Ok(Some(exit_code))
                    }
                    WAIT_TIMEOUT => Ok(None),
                    _ => Err(CrosswinError::win32(
                        "WaitForSingleObject",
                        wait_result.0,
                        "Wait failed".to_string()
                    )),
                }
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Get memory information for the process
    pub fn get_memory_info(&self) -> Result<MemoryInfo> {
        #[cfg(feature = "win32")]
        {
            let handle = self.handle.as_ref()
                .ok_or_else(|| CrosswinError::invalid_parameter("handle", "Process handle is not available"))?;
            
            unsafe {
                let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
                counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

                GetProcessMemoryInfo(
                    handle.as_handle().as_windows_handle(),
                    &mut counters,
                    counters.cb,
                )
                .map_err(|e| CrosswinError::win32(
                    "GetProcessMemoryInfo",
                    e.code().0 as u32,
                    e.to_string()
                ))?;

                Ok(MemoryInfo {
                    working_set_size: counters.WorkingSetSize as u64,
                    peak_working_set_size: counters.PeakWorkingSetSize as u64,
                    page_file_usage: counters.PagefileUsage as u64,
                    peak_page_file_usage: counters.PeakPagefileUsage as u64,
                })
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Get the priority class of the process
    pub fn get_priority(&self) -> Result<ProcessPriority> {
        #[cfg(feature = "win32")]
        {
            let handle = self.handle.as_ref()
                .ok_or_else(|| CrosswinError::invalid_parameter("handle", "Process handle is not available"))?;
            
            unsafe {
                let priority = GetPriorityClass(handle.as_handle().as_windows_handle());
                if priority == 0 {
                    return Err(CrosswinError::win32(
                        "GetPriorityClass",
                        windows::Win32::Foundation::GetLastError().0,
                        "Failed to get priority class".to_string()
                    ));
                }
                ProcessPriority::from_windows_constant(priority)
                    .ok_or_else(|| CrosswinError::invalid_parameter(
                        "priority",
                        &format!("Unknown priority class: {}", priority)
                    ))
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Set the priority class of the process
    pub fn set_priority(&self, priority: ProcessPriority) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            let handle = self.handle.as_ref()
                .ok_or_else(|| CrosswinError::invalid_parameter("handle", "Process handle is not available"))?;
            
            unsafe {
                SetPriorityClass(
                    handle.as_handle().as_windows_handle(),
                    windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(priority.to_windows_constant())
                )
                .map_err(|e| CrosswinError::win32(
                    "SetPriorityClass",
                    e.code().0 as u32,
                    e.to_string()
                ))?;
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }
}
