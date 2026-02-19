use std::fmt;
use std::hash::{Hash, Hasher};
use crate::error::{CrosswinError, Result};

// ─── WindowInfo ───────────────────────────────────────────────────────────────

/// A snapshot of information about a top-level window.
///
/// `PartialEq` and `Hash` are keyed on the raw `hwnd` value.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowInfo {
    /// Raw HWND handle value.
    pub hwnd: u64,
    /// Window title text.
    pub title: String,
    /// Window class name.
    pub class_name: Option<String>,
    /// Width in screen pixels.
    pub width: Option<u32>,
    /// Height in screen pixels.
    pub height: Option<u32>,
    /// Left edge of the window in screen coordinates.
    pub x: Option<i32>,
    /// Top edge of the window in screen coordinates.
    pub y: Option<i32>,
    /// Whether the window is currently visible.
    pub is_visible: Option<bool>,
    /// PID of the process that owns this window.
    pub process_id: Option<u32>,
}

impl PartialEq for WindowInfo {
    fn eq(&self, other: &Self) -> bool {
        self.hwnd == other.hwnd
    }
}

impl Eq for WindowInfo {}

impl Hash for WindowInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hwnd.hash(state);
    }
}

impl fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[HWND:0x{:X}] \"{}\"", self.hwnd, self.title)?;
        if let (Some(w), Some(h)) = (self.width, self.height) {
            write!(f, " ({}×{}", w, h)?;
            if let (Some(x), Some(y)) = (self.x, self.y) {
                write!(f, " @ {},{}", x, y)?;
            }
            write!(f, ")")?;
        }
        if let Some(pid) = self.process_id {
            write!(f, "  PID={}", pid)?;
        }
        Ok(())
    }
}

// ─── Window ───────────────────────────────────────────────────────────────────

/// A handle to a window with operations.
///
/// Not `Clone` — use `Window::try_clone()` which validates the handle first.
#[derive(Debug)]
pub struct Window {
    hwnd: u64,
}

impl Window {
    /// Create a `Window` from a raw platform handle value.
    pub fn from_raw(hwnd: u64) -> Self {
        Self { hwnd }
    }

    /// Return the raw HWND value.
    pub fn hwnd(&self) -> u64 {
        self.hwnd
    }

    /// Returns `true` when the stored HWND is still a live window.
    pub fn is_valid(&self) -> bool {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::IsWindow;
            use windows::Win32::Foundation::HWND;
            unsafe { IsWindow(HWND(self.hwnd as isize)).as_bool() }
        }
        #[cfg(not(feature = "win32"))]
        {
            false
        }
    }

    /// Clone this handle only when the HWND is still valid.
    ///
    /// Returns `Err(CrosswinError::InvalidParameter)` if the window no longer
    /// exists.
    pub fn try_clone(&self) -> Result<Window> {
        if self.is_valid() {
            Ok(Window { hwnd: self.hwnd })
        } else {
            Err(CrosswinError::invalid_parameter(
                "hwnd",
                "Window handle is no longer valid",
            ))
        }
    }

    // ── Visibility ────────────────────────────────────────────────────────────

    /// Show the window.
    pub fn show(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOW};
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as isize);
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Hide the window.
    pub fn hide(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as isize);
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Bring the window to the foreground.
    pub fn bring_to_front(&self) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                BringWindowToTop, SetForegroundWindow,
            };
            use windows::Win32::Foundation::HWND;
            unsafe {
                let hwnd = HWND(self.hwnd as isize);
                let _ = SetForegroundWindow(hwnd);
                BringWindowToTop(hwnd).map_err(|e| CrosswinError::win32(
                    "BringWindowToTop",
                    e.code().0 as u32,
                    e.to_string(),
                ))?;
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    // ── Geometry ──────────────────────────────────────────────────────────────

    /// Move the window to (`x`, `y`) while preserving its size.
    pub fn move_to(&self, _x: i32, _y: i32) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, SWP_NOSIZE, SWP_NOZORDER,
            };
            use windows::Win32::Foundation::HWND;
            unsafe {
                SetWindowPos(
                    HWND(self.hwnd as isize),
                    HWND(0),
                    _x, _y, 0, 0,
                    SWP_NOSIZE | SWP_NOZORDER,
                )
                .map_err(|e| CrosswinError::win32("SetWindowPos", e.code().0 as u32, e.to_string()))?;
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Resize the window to `width` × `height` while preserving its position.
    pub fn resize(&self, _width: u32, _height: u32) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                SetWindowPos, SWP_NOMOVE, SWP_NOZORDER,
            };
            use windows::Win32::Foundation::HWND;
            unsafe {
                SetWindowPos(
                    HWND(self.hwnd as isize),
                    HWND(0),
                    0, 0, _width as i32, _height as i32,
                    SWP_NOMOVE | SWP_NOZORDER,
                )
                .map_err(|e| CrosswinError::win32("SetWindowPos", e.code().0 as u32, e.to_string()))?;
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Query the current top-left position of the window.
    pub fn position(&self) -> Result<(i32, i32)> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::Foundation::{HWND, RECT};
            use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
            unsafe {
                let mut rect = RECT::default();
                GetWindowRect(HWND(self.hwnd as isize), &mut rect)
                    .map_err(|e| CrosswinError::win32("GetWindowRect", e.code().0 as u32, e.to_string()))?;
                Ok((rect.left, rect.top))
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    /// Query the current size of the window.
    pub fn size(&self) -> Result<(u32, u32)> {
        #[cfg(feature = "win32")]
        {
            use windows::Win32::Foundation::{HWND, RECT};
            use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
            unsafe {
                let mut rect = RECT::default();
                GetWindowRect(HWND(self.hwnd as isize), &mut rect)
                    .map_err(|e| CrosswinError::win32("GetWindowRect", e.code().0 as u32, e.to_string()))?;
                Ok((
                    rect.right.saturating_sub(rect.left) as u32,
                    rect.bottom.saturating_sub(rect.top) as u32,
                ))
            }
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }

    // ── Text ──────────────────────────────────────────────────────────────────

    /// Set window title text.
    pub fn set_title(&self, _title: &str) -> Result<()> {
        #[cfg(feature = "win32")]
        {
            use windows::core::PCWSTR;
            use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
            use widestring::U16CString;

            let wide = U16CString::from_str(_title).map_err(|e| {
                CrosswinError::invalid_parameter("title", format!("invalid unicode: {}", e))
            })?;
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as isize);
                SetWindowTextW(hwnd, PCWSTR(wide.as_ptr()))
                    .map_err(|e| CrosswinError::win32("SetWindowTextW", e.code().0 as u32, e.to_string()))?;
            }
            Ok(())
        }
        #[cfg(not(feature = "win32"))]
        {
            Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
        }
    }
}

// ─── Free functions ───────────────────────────────────────────────────────────

/// Retrieve the title text for a window identified by its raw HWND.
pub fn get_window_text(_hwnd: u64) -> Result<String> {
    #[cfg(feature = "win32")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW};

        unsafe {
            let h = HWND(_hwnd as isize);
            let len = GetWindowTextLengthW(h);
            if len <= 0 {
                return Ok(String::new());
            }
            let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
            let read = GetWindowTextW(h, &mut buf);
            if read > 0 {
                Ok(String::from_utf16_lossy(&buf[..read as usize]))
            } else {
                Ok(String::new())
            }
        }
    }
    #[cfg(not(feature = "win32"))]
    {
        let _ = _hwnd;
        Err(CrosswinError::invalid_parameter("platform", "Not supported on this platform"))
    }
}

/// List all top-level windows. Returns an empty `Vec` on non-Win32 builds.
pub async fn list_windows() -> Result<Vec<WindowInfo>> {
    #[cfg(feature = "win32")]
    {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
            GetWindowThreadProcessId, IsWindowVisible,
        };

        unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let vec_ptr = lparam.0 as *mut Vec<WindowInfo>;
            if vec_ptr.is_null() {
                return BOOL(1);
            }
            let list = &mut *vec_ptr;

            // Title
            let len = GetWindowTextLengthW(hwnd);
            let mut title = String::new();
            if len > 0 {
                let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
                let read = GetWindowTextW(hwnd, &mut buf);
                if read > 0 {
                    title = String::from_utf16_lossy(&buf[..read as usize]);
                }
            }

            // Class name
            let mut class_buf: [u16; 256] = [0; 256];
            let class_len = GetClassNameW(hwnd, &mut class_buf);
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
            let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));

            list.push(WindowInfo {
                hwnd: hwnd.0 as u64,
                title,
                class_name,
                width: Some(width),
                height: Some(height),
                x: Some(rect.left),
                y: Some(rect.top),
                is_visible: Some(visible),
                process_id: Some(pid),
            });

            BOOL(1)
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

/// Find windows by title substring (case-insensitive).
pub async fn find_windows_by_title(title: &str) -> Result<Vec<WindowInfo>> {
    let all = list_windows().await?;
    let lower = title.to_lowercase();
    Ok(all.into_iter().filter(|w| w.title.to_lowercase().contains(&lower)).collect())
}

/// Find windows by class name (case-insensitive).
///
/// # Bug fix (v0.4.0)
/// Previously used a case-sensitive exact match; now normalises to lowercase
/// for consistency with `find_windows_by_title`.
pub async fn find_windows_by_class(class: &str) -> Result<Vec<WindowInfo>> {
    let all = list_windows().await?;
    let lower = class.to_lowercase();
    Ok(all
        .into_iter()
        .filter(|w| {
            w.class_name
                .as_deref()
                .map_or(false, |c| c.to_lowercase() == lower)
        })
        .collect())
}

/// Find windows owned by a specific process.
pub async fn find_windows_by_process(pid: u32) -> Result<Vec<WindowInfo>> {
    let all = list_windows().await?;
    Ok(all.into_iter().filter(|w| w.process_id == Some(pid)).collect())
}

// ─── WindowFilter ─────────────────────────────────────────────────────────────

/// Builder for filtering windows with multiple criteria.
///
/// ```rust,no_run
/// # use crosswin::windows::window::WindowFilter;
/// # #[tokio::main] async fn main() {
/// let results = WindowFilter::new()
///     .title_contains("Visual Studio")
///     .visible_only(true)
///     .min_width(800)
///     .list()
///     .await
///     .unwrap();
/// # }
/// ```
#[derive(Default, Debug, Clone)]
pub struct WindowFilter {
    title_contains: Option<String>,
    class_name: Option<String>,
    visible_only: Option<bool>,
    process_id: Option<u32>,
    min_width: Option<u32>,
    min_height: Option<u32>,
}

impl WindowFilter {
    /// Create a new, empty window filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by title substring (case-insensitive).
    pub fn title_contains<S: Into<String>>(mut self, s: S) -> Self {
        self.title_contains = Some(s.into().to_lowercase());
        self
    }

    /// Filter by exact class name (case-insensitive).
    pub fn class_name<S: Into<String>>(mut self, s: S) -> Self {
        self.class_name = Some(s.into().to_lowercase());
        self
    }

    /// When `true`, only include windows where `IsWindowVisible` returns `true`.
    pub fn visible_only(mut self, v: bool) -> Self {
        self.visible_only = Some(v);
        self
    }

    /// Only include windows owned by the given PID.
    pub fn process_id(mut self, pid: u32) -> Self {
        self.process_id = Some(pid);
        self
    }

    /// Only include windows at least this many pixels wide.
    pub fn min_width(mut self, w: u32) -> Self {
        self.min_width = Some(w);
        self
    }

    /// Only include windows at least this many pixels tall.
    pub fn min_height(mut self, h: u32) -> Self {
        self.min_height = Some(h);
        self
    }

    /// Execute the filter and return matching windows.
    pub async fn list(self) -> Result<Vec<WindowInfo>> {
        let all = list_windows().await?;
        Ok(all.into_iter().filter(|w| self.matches(w)).collect())
    }

    fn matches(&self, w: &WindowInfo) -> bool {
        if let Some(ref title) = self.title_contains {
            if !w.title.to_lowercase().contains(title) {
                return false;
            }
        }
        if let Some(ref class) = self.class_name {
            if !w.class_name.as_deref().map_or(false, |c| c.to_lowercase() == *class) {
                return false;
            }
        }
        if let Some(vis) = self.visible_only {
            if vis && !w.is_visible.unwrap_or(false) {
                return false;
            }
        }
        if let Some(pid) = self.process_id {
            if w.process_id != Some(pid) {
                return false;
            }
        }
        if let Some(min_w) = self.min_width {
            if w.width.map_or(true, |width| width < min_w) {
                return false;
            }
        }
        if let Some(min_h) = self.min_height {
            if w.height.map_or(true, |height| height < min_h) {
                return false;
            }
        }
        true
    }
}
