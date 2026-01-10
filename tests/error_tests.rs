use crosswin::error::{CrosswinError, Result};

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
    
    match err {
        CrosswinError::Io(_) => (),
        _ => panic!("Should convert to Io error"),
    }
}
