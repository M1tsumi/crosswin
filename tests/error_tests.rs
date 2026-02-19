use crosswin::error::CrosswinError;

#[test]
fn test_error_display() {
    let err = CrosswinError::win32("test_op", 0x5, "Access denied");
    let display = format!("{}", err);
    assert!(display.contains("test_op"));
    assert!(display.contains("0x00000005"));
}

#[test]
fn test_access_denied_error() {
    let err = CrosswinError::access_denied("process", Some(1234));
    let display = format!("{}", err);
    assert!(display.contains("Access denied"));
    assert!(display.contains("1234"));
}

#[test]
fn test_process_not_found_error() {
    let err = CrosswinError::process_not_found(5678);
    let display = format!("{}", err);
    assert!(display.contains("Process not found"));
    assert!(display.contains("5678"));
}

#[test]
fn test_invalid_parameter_error() {
    let err = CrosswinError::invalid_parameter("timeout", "must be positive");
    let display = format!("{}", err);
    assert!(display.contains("timeout"));
    assert!(display.contains("must be positive"));
}

#[test]
fn test_timeout_error() {
    let err = CrosswinError::timeout("wait_for_exit", 5000);
    let display = format!("{}", err);
    assert!(display.contains("Timeout"));
    assert!(display.contains("5000"));
}

#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: CrosswinError = io_err.into();

    // Pattern matching still works as before.
    match err {
        CrosswinError::Io(_) => (),
        _ => panic!("Should convert to Io error"),
    }
}

// ─── v0.4.0: PartialEq tests ──────────────────────────────────────────────────

#[test]
fn test_process_not_found_equality() {
    let a = CrosswinError::process_not_found(42);
    let b = CrosswinError::process_not_found(42);
    assert_eq!(a, b);
}

#[test]
fn test_process_not_found_different_pid() {
    let a = CrosswinError::process_not_found(42);
    let b = CrosswinError::process_not_found(99);
    assert_ne!(a, b);
}

#[test]
fn test_timeout_equality() {
    let a = CrosswinError::timeout("op", 1000);
    let b = CrosswinError::timeout("op", 1000);
    assert_eq!(a, b);
}

#[test]
fn test_access_denied_equality() {
    let a = CrosswinError::access_denied("resource", Some(10));
    let b = CrosswinError::access_denied("resource", Some(10));
    assert_eq!(a, b);
}

#[test]
fn test_io_errors_compare_by_kind() {
    let e1: CrosswinError = std::io::Error::new(std::io::ErrorKind::NotFound, "a").into();
    let e2: CrosswinError = std::io::Error::new(std::io::ErrorKind::NotFound, "b").into();
    // Different messages, same kind → equal
    assert_eq!(e1, e2);

    let e3: CrosswinError = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "a").into();
    // Different kind → not equal
    assert_ne!(e1, e3);
}

// ─── v0.4.0: non_exhaustive ensures wildcard arms are required ────────────────
//
// This is a compile-time property; the test below documents the expectation
// while remaining compilable.  Downstream crates must include a `_` arm when
// matching `CrosswinError` because the enum is `#[non_exhaustive]`.

#[test]
fn test_non_exhaustive_match_requires_wildcard() {
    let err = CrosswinError::process_not_found(1);
    let _description = match err {
        CrosswinError::ProcessNotFound { pid } => format!("pid {}", pid),
        CrosswinError::Win32 { operation, .. } => format!("win32 {}", operation),
        CrosswinError::Io(_) => "io".to_string(),
        CrosswinError::AccessDenied { .. } => "access denied".to_string(),
        CrosswinError::InvalidParameter { .. } => "invalid param".to_string(),
        CrosswinError::Timeout { .. } => "timeout".to_string(),
        // Wildcard required because CrosswinError is #[non_exhaustive]
        _ => "unknown".to_string(),
    };
}
