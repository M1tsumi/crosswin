/// Unit and integration tests for `WindowFilter` and related window helpers.
use crosswin::prelude::*;
use crosswin::windows::window::{WindowFilter, WindowInfo};

// ─── Unit helpers ─────────────────────────────────────────────────────────────

fn make_window(hwnd: u64, title: &str, class: &str, visible: bool, pid: u32) -> WindowInfo {
    WindowInfo {
        hwnd,
        title: title.to_string(),
        class_name: Some(class.to_string()),
        width: Some(800),
        height: Some(600),
        x: Some(0),
        y: Some(0),
        is_visible: Some(visible),
        process_id: Some(pid),
    }
}

// ─── WindowInfo equality ──────────────────────────────────────────────────────

#[test]
fn window_info_eq_by_hwnd() {
    let a = make_window(0x100, "Foo", "FooClass", true, 1);
    let b = make_window(0x100, "Bar", "OtherClass", false, 2);
    assert_eq!(a, b, "WindowInfo equality is keyed on hwnd");
}

#[test]
fn window_info_ne_different_hwnd() {
    let a = make_window(0x100, "Foo", "FooClass", true, 1);
    let b = make_window(0x200, "Foo", "FooClass", true, 1);
    assert_ne!(a, b);
}

// ─── WindowFilter unit tests ──────────────────────────────────────────────────

#[test]
fn window_filter_builder_compiles() {
    let _f = WindowFilter::new()
        .title_contains("Visual Studio")
        .visible_only(true)
        .min_width(800)
        .min_height(600)
        .process_id(1234);
}

// ─── find_windows_by_class case-insensitivity ─────────────────────────────────

#[cfg(feature = "win32")]
#[tokio::test]
async fn find_by_class_is_case_insensitive() {
    use crosswin::prelude::*;

    // Enumerate all windows and pick the first that has a class name.
    let all = list_windows().await.expect("list_windows failed");
    let with_class: Vec<_> = all.iter().filter(|w| w.class_name.is_some()).collect();

    if with_class.is_empty() {
        // No windows with class names, skip.
        return;
    }

    let class = with_class[0].class_name.as_deref().unwrap();
    let upper = class.to_uppercase();
    let lower = class.to_lowercase();

    let by_upper = find_windows_by_class(&upper).await.expect("find_windows_by_class failed");
    let by_lower = find_windows_by_class(&lower).await.expect("find_windows_by_class failed");

    assert_eq!(
        by_upper.len(),
        by_lower.len(),
        "find_windows_by_class must be case-insensitive"
    );
}

// ─── Window::try_clone validity check ────────────────────────────────────────

#[cfg(feature = "win32")]
#[test]
fn try_clone_stale_hwnd_returns_err() {
    use crosswin::prelude::Window;
    // HWND 0xDEADBEEF is almost certainly not a valid window.
    let w = Window::from_raw(0xDEAD_BEEF_u64);
    assert!(w.try_clone().is_err(), "try_clone on a stale hwnd must return Err");
}

#[cfg(feature = "win32")]
#[tokio::test]
async fn try_clone_real_window_succeeds() {
    // Find a real visible window.
    let windows = list_windows().await.expect("list_windows");
    let visible: Vec<_> = windows.iter().filter(|w| w.is_visible == Some(true)).collect();
    if visible.is_empty() {
        return; // Nothing to test
    }
    let info = visible[0];
    let w = Window::from_raw(info.hwnd);
    assert!(w.is_valid(), "should be valid");
    assert!(w.try_clone().is_ok(), "try_clone on live hwnd must succeed");
}
