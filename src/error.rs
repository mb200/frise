use core::fmt;
use std::{error::Error, io};

#[derive(Debug)]
pub enum FriseError {
    InquireError(inquire::InquireError),
    Utf8Error(std::string::FromUtf8Error),
    IOError(std::io::Error),
    Custom(String),
}

impl Error for FriseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // InquireError::IO(err) => Some(err),
            FriseError::IOError(e) => Some(e),
            FriseError::Utf8Error(e) => Some(e),
            FriseError::InquireError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<inquire::InquireError> for FriseError {
    fn from(err: inquire::InquireError) -> Self {
        FriseError::InquireError(err)
    }
}

impl From<std::string::FromUtf8Error> for FriseError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        FriseError::Utf8Error(err)
    }
}

impl From<io::Error> for FriseError {
    fn from(err: io::Error) -> Self {
        match err.raw_os_error() {
            _ => FriseError::IOError(err),
        }
    }
}

impl fmt::Display for FriseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FriseError::IOError(e) => write!(f, "IO error: {}", e),
            FriseError::InquireError(e) => e.fmt(f),
            FriseError::Utf8Error(e) => write!(f, "UTF8 error: {}", e),
            FriseError::Custom(msg) => {
                write!(f, "Error: {msg}")
            }
        }
    }
}

pub type FriseResult<T> = Result<T, FriseError>;
