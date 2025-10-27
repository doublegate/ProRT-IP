//! CLI-specific error types with user-friendly messages
//!
//! This module provides error types for command-line interface operations with:
//! - Exit codes compatible with Unix conventions
//! - User-facing error messages with actionable suggestions
//! - Structured error data for debugging
//!
//! Sprint 4.22 Phase 3: Error Handling & Resilience

use std::path::PathBuf;
use thiserror::Error;

/// CLI-specific error types
#[derive(Error, Debug)]
pub enum CliError {
    /// Invalid command line argument
    #[error("Invalid argument '--{arg}': {reason}")]
    InvalidArgument {
        arg: String,
        reason: String,
        suggestion: Option<String>,
    },

    /// Output file error
    #[error("Cannot access output file: {path}")]
    OutputFileError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Output file already exists
    #[error("Output file already exists: {path}")]
    OutputFileExists { path: PathBuf, suggestion: String },

    /// Failed to write output
    #[error("Failed to write {format} output to {path}")]
    OutputWriteError {
        format: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Invalid target specification
    #[error("Invalid target: {spec}")]
    InvalidTarget { spec: String, reason: String },

    /// No targets specified
    #[error("No valid targets specified")]
    NoTargets { suggestion: String },

    /// Database operation failed
    #[error("Database error: {operation}")]
    DatabaseError {
        operation: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl CliError {
    /// Returns exit code for this error (Unix-compatible)
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidArgument { .. } => exit_codes::INVALID_ARGS,
            Self::OutputFileError { .. } => exit_codes::IO_ERROR,
            Self::OutputFileExists { .. } => exit_codes::IO_ERROR,
            Self::OutputWriteError { .. } => exit_codes::IO_ERROR,
            Self::InvalidTarget { .. } => exit_codes::INVALID_ARGS,
            Self::NoTargets { .. } => exit_codes::INVALID_ARGS,
            Self::DatabaseError { .. } => exit_codes::IO_ERROR,
        }
    }

    /// Returns user-facing suggestion for this error
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::InvalidArgument { suggestion, .. } => suggestion.as_deref(),
            Self::OutputFileExists { suggestion, .. } => Some(suggestion),
            Self::NoTargets { suggestion, .. } => Some(suggestion),
            _ => None,
        }
    }

    /// Create InvalidArgument error with suggestion
    pub fn invalid_argument(arg: &str, reason: &str, suggestion: Option<String>) -> Self {
        Self::InvalidArgument {
            arg: arg.to_string(),
            reason: reason.to_string(),
            suggestion,
        }
    }

    /// Create OutputFileExists error with suggestion
    pub fn output_file_exists(path: PathBuf) -> Self {
        Self::OutputFileExists {
            path: path.clone(),
            suggestion: format!(
                "Use --force to overwrite or specify a different path with -oN/oX/oG {}",
                path.with_extension("2.txt").display()
            ),
        }
    }

    /// Create NoTargets error with helpful suggestion
    pub fn no_targets() -> Self {
        Self::NoTargets {
            suggestion: "Specify targets with: IP (192.168.1.1), CIDR (10.0.0.0/24), or hostname (example.com)".to_string(),
        }
    }
}

/// Exit codes for CLI (Unix-compatible)
pub mod exit_codes {
    /// Successful execution
    pub const SUCCESS: i32 = 0;

    /// General error (unspecified)
    pub const GENERAL_ERROR: i32 = 1;

    /// Invalid command line arguments
    pub const INVALID_ARGS: i32 = 2;

    /// Permission denied (needs sudo/admin)
    pub const PERMISSION_DENIED: i32 = 3;

    /// Network error (connection failed, unreachable)
    pub const NETWORK_ERROR: i32 = 4;

    /// Scan failed to complete
    pub const SCAN_FAILED: i32 = 5;

    /// Configuration error (invalid config file)
    pub const CONFIG_ERROR: i32 = 6;

    /// I/O error (file read/write failed)
    pub const IO_ERROR: i32 = 7;

    /// Resource exhausted (too many open files, out of memory)
    pub const RESOURCE_EXHAUSTED: i32 = 8;

    /// Cancelled by user (SIGINT/Ctrl+C)
    pub const CANCELLED: i32 = 130; // Standard Unix SIGINT exit code
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_invalid_argument_exit_code() {
        let err = CliError::invalid_argument("port", "must be 1-65535", None);
        assert_eq!(err.exit_code(), exit_codes::INVALID_ARGS);
    }

    #[test]
    fn test_output_file_error_exit_code() {
        let err = CliError::OutputFileError {
            path: PathBuf::from("/nonexistent/file.txt"),
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        };
        assert_eq!(err.exit_code(), exit_codes::IO_ERROR);
    }

    #[test]
    fn test_no_targets_exit_code() {
        let err = CliError::no_targets();
        assert_eq!(err.exit_code(), exit_codes::INVALID_ARGS);
        assert!(err.suggestion().is_some());
        assert!(err.suggestion().unwrap().contains("192.168.1.1"));
    }

    #[test]
    fn test_output_file_exists_suggestion() {
        let err = CliError::output_file_exists(PathBuf::from("results.txt"));
        assert!(err.suggestion().is_some());
        assert!(err.suggestion().unwrap().contains("--force"));
    }

    #[test]
    fn test_invalid_argument_with_suggestion() {
        let err = CliError::invalid_argument(
            "parallelism",
            "must be positive",
            Some("Try --parallelism 100".to_string()),
        );
        assert!(err.suggestion().is_some());
        assert_eq!(err.suggestion().unwrap(), "Try --parallelism 100");
    }

    #[test]
    fn test_error_display() {
        let err = CliError::invalid_argument("port", "must be 1-65535", None);
        let display = err.to_string();
        assert!(display.contains("port"));
        assert!(display.contains("must be 1-65535"));
    }

    #[test]
    fn test_cancelled_exit_code() {
        // Cancelled should use standard Unix SIGINT exit code
        assert_eq!(exit_codes::CANCELLED, 130);
    }
}
