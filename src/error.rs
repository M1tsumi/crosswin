use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum CrosswinError {
    Win32(String),
    Io(std::io::Error),
}

pub type Result<T> = std::result::Result<T, CrosswinError>;

impl fmt::Display for CrosswinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrosswinError::Win32(message) => write!(f, "Win32 error: {message}"),
            CrosswinError::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl StdError for CrosswinError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CrosswinError::Win32(_) => None,
            CrosswinError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for CrosswinError {
    fn from(err: std::io::Error) -> Self {
        CrosswinError::Io(err)
    }
}
