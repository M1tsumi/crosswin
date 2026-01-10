#[cfg(feature = "win32")]
use windows::Win32::Foundation::{CloseHandle, HANDLE};

/// A raw Windows handle value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawHandle(pub isize);

impl RawHandle {
    /// Check if the handle is valid (not null or INVALID_HANDLE_VALUE)
    pub fn is_valid(&self) -> bool {
        self.0 != 0 && self.0 != -1
    }

    /// Get the handle as a Windows HANDLE type
    #[cfg(feature = "win32")]
    pub fn as_windows_handle(&self) -> HANDLE {
        HANDLE(self.0)
    }
}

/// A RAII wrapper for Windows handles that automatically closes on drop
#[derive(Debug)]
pub struct Handle {
    raw: RawHandle,
}

impl Handle {
    /// Create a new handle from a raw isize value
    /// 
    /// # Safety
    /// The caller must ensure that the handle is valid and owned.
    /// The handle will be closed when this wrapper is dropped.
    pub unsafe fn from_raw(raw: isize) -> Self {
        Handle {
            raw: RawHandle(raw),
        }
    }

    /// Create a handle from a Windows HANDLE
    #[cfg(feature = "win32")]
    pub unsafe fn from_windows_handle(handle: HANDLE) -> Self {
        Handle {
            raw: RawHandle(handle.0),
        }
    }

    /// Get the raw handle value
    pub fn raw(&self) -> RawHandle {
        self.raw
    }

    /// Check if the handle is valid
    pub fn is_valid(&self) -> bool {
        self.raw.is_valid()
    }

    /// Convert to a Windows HANDLE
    #[cfg(feature = "win32")]
    pub fn as_windows_handle(&self) -> HANDLE {
        self.raw.as_windows_handle()
    }

    /// Take ownership of the handle, preventing automatic closure
    /// Returns the raw handle value
    pub fn into_raw(self) -> isize {
        let raw = self.raw.0;
        std::mem::forget(self); // Prevent Drop from running
        raw
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        #[cfg(feature = "win32")]
        {
            if self.is_valid() {
                unsafe {
                    let _ = CloseHandle(self.as_windows_handle());
                }
            }
        }
    }
}

// Note: Handle is not Clone to prevent double-close bugs
// (negative trait bounds are not yet stable, so we simply don't implement Clone)

/// Type-safe wrapper for process handles
#[derive(Debug)]
pub struct ProcessHandle(Handle);

impl ProcessHandle {
    /// Create a new process handle
    /// 
    /// # Safety
    /// The caller must ensure that the handle is a valid process handle
    pub unsafe fn from_raw(raw: isize) -> Self {
        ProcessHandle(Handle::from_raw(raw))
    }

    #[cfg(feature = "win32")]
    pub unsafe fn from_windows_handle(handle: HANDLE) -> Self {
        ProcessHandle(Handle::from_windows_handle(handle))
    }

    /// Get the underlying handle
    pub fn as_handle(&self) -> &Handle {
        &self.0
    }

    /// Convert to raw handle value
    pub fn into_raw(self) -> isize {
        self.0.into_raw()
    }
}

/// Type-safe wrapper for thread handles
#[derive(Debug)]
pub struct ThreadHandle(Handle);

impl ThreadHandle {
    /// Create a new thread handle
    /// 
    /// # Safety
    /// The caller must ensure that the handle is a valid thread handle
    pub unsafe fn from_raw(raw: isize) -> Self {
        ThreadHandle(Handle::from_raw(raw))
    }

    #[cfg(feature = "win32")]
    pub unsafe fn from_windows_handle(handle: HANDLE) -> Self {
        ThreadHandle(Handle::from_windows_handle(handle))
    }

    /// Get the underlying handle
    pub fn as_handle(&self) -> &Handle {
        &self.0
    }

    /// Convert to raw handle value
    pub fn into_raw(self) -> isize {
        self.0.into_raw()
    }
}
