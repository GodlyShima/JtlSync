use std::fmt; 
use anyhow::anyhow;
use tauri::ipc::InvokeError;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    Database(String),
    Api(String),
    Config(String),
    Sync(String),
    System(String),
    NotFound(String),
    ValidationError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database(msg) => write!(f, "Database error: {}", msg),
            Error::Api(msg) => write!(f, "API error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Sync(msg) => write!(f, "Synchronization error: {}", msg),
            Error::System(msg) => write!(f, "System error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl StdError for Error {}

impl From<Error> for InvokeError {
    fn from(error: Error) -> Self {
        InvokeError::from_error(std::io::Error::new(
            std::io::ErrorKind::Other, 
            error.to_string()
        ))
    }
}

// From implementations for common error types
impl From<mysql::Error> for Error {
    fn from(err: mysql::Error) -> Self {
        Error::Database(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Api(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::System(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Config(err.to_string())
    }
}

// Convenience type alias
pub type Result<T> = std::result::Result<T, Error>;