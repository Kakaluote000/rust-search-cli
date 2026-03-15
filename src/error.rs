//! Error handling module - custom error types for the application

use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;

/// Application-specific error types
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    /// Invalid regex pattern
    InvalidPattern(String),
    
    /// Invalid path specified
    InvalidPath(PathBuf, String),
    
    /// Path does not exist
    PathNotFound(PathBuf),
    
    /// Permission denied
    PermissionDenied(PathBuf),
    
    /// File read error
    FileReadError(PathBuf, io::Error),
    
    /// Invalid command line argument
    InvalidArgument(String),
    
    /// Search configuration error
    ConfigError(String),
    
    /// Walk error (directory traversal)
    WalkError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidPattern(pattern) => {
                write!(f, "Invalid regex pattern: {}", pattern)
            }
            AppError::InvalidPath(path, reason) => {
                write!(f, "Invalid path {:?}: {}", path, reason)
            }
            AppError::PathNotFound(path) => {
                write!(f, "Path does not exist: {:?}", path)
            }
            AppError::PermissionDenied(path) => {
                write!(f, "Permission denied: {:?}", path)
            }
            AppError::FileReadError(path, err) => {
                write!(f, "Error reading file {:?}: {}", path, err)
            }
            AppError::InvalidArgument(msg) => {
                write!(f, "Invalid argument: {}", msg)
            }
            AppError::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            AppError::WalkError(msg) => {
                write!(f, "Directory traversal error: {}", msg)
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::FileReadError(_, err) => Some(err),
            _ => None,
        }
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> Self {
        AppError::InvalidPattern(err.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::WalkError(err.to_string())
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::InvalidArgument(err.to_string())
    }
}

/// Result type alias using our custom error
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::InvalidPattern("invalid[".to_string());
        assert_eq!(err.to_string(), "Invalid regex pattern: invalid[");
        
        let err = AppError::PathNotFound(PathBuf::from("/nonexistent"));
        assert!(err.to_string().contains("/nonexistent"));
    }

    #[test]
    fn test_error_conversion() {
        let regex_err = regex::Error::Syntax("invalid".to_string());
        let app_err: AppError = regex_err.into();
        assert!(matches!(app_err, AppError::InvalidPattern(_)));
    }
}
