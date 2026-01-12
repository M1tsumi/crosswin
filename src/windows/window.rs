use crate::error::{CrosswinError, Result};

/// Basic information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
    pub class_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub is_visible: Option<bool>,
    pub process_id: Option<u32>,
}

/// Represents a handle to a window and operations on it
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Window {
    hwnd: u64,
}

impl Window {
    /// Create a `Window` from a platform handle (opaque)
    pub fn from_raw(hwnd: u64) -> Self {
        Self { hwnd }
    }

    /// Show the window
    pub fn show(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
            use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as isize);
                ShowWindow(hwnd, SW_SHOW);
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Hide the window
    pub fn hide(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
            use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as isize);
                ShowWindow(hwnd, SW_HIDE);
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Set window title (stub)
    pub fn set_title(&self, _title: &str) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::core::PCWSTR;
            use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
            use widestring::U16CString;

            let wide = U16CString::from_str(_title).map_err(|e| CrosswinError::invalid_parameter("title", format!("invalid unicode: {}", e)))?;
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as isize);
                SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()));
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }
}

/// List top-level windows. On non-Win32 builds returns an empty vector.
pub async fn list_windows() -> Result<Vec<WindowInfo>> {
    #[cfg(feature = "win32")]
    {
        use std::ptr::null_mut;
        use windows::core::PWSTR;
        use windows::Win32::Foundation::{HWND, LPARAM, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetClassNameW, IsWindowVisible, GetWindowRect, GetWindowThreadProcessId};

        unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> i32 {
            let vec_ptr = lparam.0 as *mut Vec<WindowInfo>;
            if vec_ptr.is_null() {
                return 1;
            }

            let list = &mut *vec_ptr;

            // Title
            let len = GetWindowTextLengthW(hwnd);
            let mut title = String::new();
            if len > 0 {
                let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
                let read = GetWindowTextW(hwnd, PWSTR(buf.as_mut_ptr()), len + 1);
                if read > 0 {
                    title = String::from_utf16_lossy(&buf[..read as usize]);
                }
            }

            // Class
            let mut class_buf: [u16; 256] = [0; 256];
            let class_len = GetClassNameW(hwnd, PWSTR(class_buf.as_mut_ptr()), 256);
            let class_name = if class_len > 0 {
                Some(String::from_utf16_lossy(&class_buf[..class_len as usize]))
            } else {
                None
            };

            // Rect
            let mut rect = RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect);
            let width = rect.right.saturating_sub(rect.left) as u32;
            let height = rect.bottom.saturating_sub(rect.top) as u32;

            // Visibility
            let visible = IsWindowVisible(hwnd).as_bool();

            // Process ID
            let mut pid: u32 = 0;
            let _ = GetWindowThreadProcessId(hwnd, &mut pid);

            list.push(WindowInfo {
                hwnd: hwnd.0 as u64,
                title,
                class_name,
                width: Some(width),
                height: Some(height),
                is_visible: Some(visible),
                process_id: Some(pid),
            });

            1
        }

        let mut list: Vec<WindowInfo> = Vec::new();
        let ptr = &mut list as *mut _ as isize;
        unsafe {
            let _ = EnumWindows(Some(callback), LPARAM(ptr));
        }

        Ok(list)
    }

    #[cfg(not(feature = "win32"))]
    {
        Ok(Vec::new())
    }
}

/// Find windows by title substring (case-insensitive)
pub async fn find_windows_by_title(title: &str) -> Result<Vec<WindowInfo>> {
    let all = list_windows().await?;
    let title_lower = title.to_lowercase();
    Ok(all.into_iter().filter(|w| w.title.to_lowercase().contains(&title_lower)).collect())
}

/// Find windows by class name
pub async fn find_windows_by_class(class: &str) -> Result<Vec<WindowInfo>> {
    let all = list_windows().await?;
    Ok(all.into_iter().filter(|w| w.class_name.as_deref().map_or(false, |c| c == class)).collect())
}

/// Find windows owned by a specific process
pub async fn find_windows_by_process(pid: u32) -> Result<Vec<WindowInfo>> {
    let all = list_windows().await?;
    Ok(all.into_iter().filter(|w| w.process_id == Some(pid)).collect())
}

/// Get text of a window
pub fn get_window_text(_hwnd: u64) -> Result<String> {
    #[cfg(feature = "win32")]
    {
        use windows::core::PWSTR;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
        use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;

        unsafe {
            let h = HWND(hwnd as isize);
            let len = GetWindowTextLengthW(h);
            if len <= 0 {
                return Ok(String::new());
            }
            let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
            let read = GetWindowTextW(h, PWSTR(buf.as_mut_ptr()), len + 1);
            if read > 0 {
                Ok(String::from_utf16_lossy(&buf[..read as usize]))
            } else {
                Ok(String::new())
            }
        }
    }

    #[cfg(not(feature = "win32"))]
    {
        Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
    }
}

/// Bring a window to the front
    pub fn bring_to_front(_hwnd: u64) -> Result<()> {
    #[cfg(feature = "win32")]
    {
        use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
        use windows::Win32::Foundation::HWND;
        unsafe {
            SetForegroundWindow(HWND(hwnd as isize));
        }
        Ok(())
    }

    #[cfg(not(feature = "win32"))]
    {
        Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
    }
}
