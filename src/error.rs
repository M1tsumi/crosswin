use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum CrosswinError {
    Win32 {
        operation: String,
        code: u32,
        message: String,
    },
    Io(std::io::Error),
    AccessDenied {
        resource: String,
        pid: Option<u32>,
    },
    ProcessNotFound {
        pid: u32,
    },
    InvalidParameter {
        parameter: String,
        reason: String,
    },
    Timeout {
        operation: String,
        duration_ms: u64,
    },
}

pub type Result<T> = std::result::Result<T, CrosswinError>;

impl CrosswinError {
    /// Create a Win32 error with operation context
    pub fn win32<S: Into<String>, T: Into<String>>(operation: S, code: u32, message: T) -> Self {
        CrosswinError::Win32 {
            operation: operation.into(),
            code,
            message: message.into(),
        }
    }

    /// Create an access denied error
    pub fn access_denied<S: Into<String>>(resource: S, pid: Option<u32>) -> Self {
        CrosswinError::AccessDenied {
            resource: resource.into(),
            pid,
        }
    }

    /// Create a process not found error
    pub fn process_not_found(pid: u32) -> Self {
        CrosswinError::ProcessNotFound { pid }
    }

    /// Create an invalid parameter error
    pub fn invalid_parameter<S: Into<String>, T: Into<String>>(parameter: S, reason: T) -> Self {
        CrosswinError::InvalidParameter {
            parameter: parameter.into(),
            reason: reason.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S, duration_ms: u64) -> Self {
        CrosswinError::Timeout {
            operation: operation.into(),
            duration_ms,
        }
    }
}

impl fmt::Display for CrosswinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrosswinError::Win32 { operation, code, message } => {
                write!(f, "Win32 error in {}: {} (code: 0x{:08X})", operation, message, code)
            }
            CrosswinError::Io(err) => write!(f, "IO error: {}", err),
            CrosswinError::AccessDenied { resource, pid } => {
                if let Some(pid) = pid {
                    write!(f, "Access denied to {} (PID: {})", resource, pid)
                } else {
                    write!(f, "Access denied to {}", resource)
                }
            }
            CrosswinError::ProcessNotFound { pid } => {
                write!(f, "Process not found (PID: {})", pid)
            }
            CrosswinError::InvalidParameter { parameter, reason } => {
                write!(f, "Invalid parameter '{}': {}", parameter, reason)
            }
            CrosswinError::Timeout { operation, duration_ms } => {
                write!(f, "Timeout in {} after {}ms", operation, duration_ms)
            }
        }
    }
}

impl StdError for CrosswinError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CrosswinError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CrosswinError {
    fn from(err: std::io::Error) -> Self {
        CrosswinError::Io(err)
    }
}
