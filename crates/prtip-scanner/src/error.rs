//! Scanner-specific error types with recovery hints
//!
//! This module provides enhanced error types for scanner operations with:
//! - Retriability information for automatic retry logic
//! - Recovery suggestions for user-facing error messages
//! - Error categorization for progress tracking
//! - Structured error data for debugging
//!
//! Sprint 4.22 Phase 3: Error Handling & Resilience

use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;

/// Result type alias for scanner operations
pub type ScannerResult<T> = std::result::Result<T, ScannerError>;

/// Comprehensive scanner error type with recovery hints
#[derive(Error, Debug, Clone)]
pub enum ScannerError {
    /// Connection operation failed
    #[error("Connection to {target} failed: {reason}")]
    ConnectionFailed {
        target: SocketAddr,
        reason: String,
        retriable: bool,
    },

    /// Scan operation timed out
    #[error("Scan timeout for {target} after {duration:?}")]
    Timeout {
        target: SocketAddr,
        duration: Duration,
        retriable: bool,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {current_rate} pps > {max_rate} pps")]
    RateLimitExceeded { current_rate: u64, max_rate: u64 },

    /// Target is unreachable
    #[error("Target unreachable: {target} - {reason}")]
    TargetUnreachable { target: SocketAddr, reason: String },

    /// Insufficient privileges to perform scan
    #[error("Insufficient privileges for {scan_type} scan")]
    InsufficientPrivileges {
        scan_type: String,
        suggestion: String,
    },

    /// Resource exhaustion (file descriptors, memory)
    #[error("Resource exhausted: {resource} (current: {current}, limit: {limit})")]
    ResourceExhausted {
        resource: String,
        current: u64,
        limit: u64,
        suggestion: String,
    },

    /// Scanner configuration error
    #[error("Invalid scanner configuration: {0}")]
    InvalidConfiguration(String),

    /// Probe failed (OS fingerprinting, service detection)
    #[error("Probe failed for {target}: {reason}")]
    ProbeFailed {
        target: SocketAddr,
        probe_type: String,
        reason: String,
        retriable: bool,
    },

    /// Scan was cancelled by user or system
    #[error("Scan cancelled: {reason}")]
    Cancelled { reason: String },
}

impl ScannerError {
    /// Returns whether this error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::ConnectionFailed { retriable, .. } => *retriable,
            Self::Timeout { retriable, .. } => *retriable,
            Self::RateLimitExceeded { .. } => true,
            Self::TargetUnreachable { .. } => false,
            Self::InsufficientPrivileges { .. } => false,
            Self::ResourceExhausted { .. } => true,
            Self::ProbeFailed { retriable, .. } => *retriable,
            Self::InvalidConfiguration(_) => false,
            Self::Cancelled { .. } => false,
        }
    }

    /// Returns recovery suggestion for user-facing error messages
    pub fn recovery_suggestion(&self) -> Option<&str> {
        match self {
            Self::InsufficientPrivileges { suggestion, .. } => Some(suggestion),
            Self::ResourceExhausted { suggestion, .. } => Some(suggestion),
            Self::RateLimitExceeded { .. } => {
                Some("Reduce scan rate with -T0 through -T3, or --max-rate")
            }
            Self::Timeout { .. } => {
                Some("Increase timeout with --timeout or use faster timing template (-T3, -T4)")
            }
            Self::ConnectionFailed {
                retriable: false, ..
            } => Some("Target may be down or unreachable"),
            _ => None,
        }
    }

    /// Returns error category for progress tracking
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ConnectionFailed { .. } => ErrorCategory::ConnectionFailed,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::RateLimitExceeded { .. } => ErrorCategory::RateLimit,
            Self::TargetUnreachable { .. } => ErrorCategory::HostUnreachable,
            Self::InsufficientPrivileges { .. } => ErrorCategory::PermissionDenied,
            Self::ResourceExhausted { .. } => ErrorCategory::TooManyOpenFiles,
            Self::ProbeFailed { .. } => ErrorCategory::ProbeFailed,
            Self::InvalidConfiguration(_) => ErrorCategory::ConfigError,
            Self::Cancelled { .. } => ErrorCategory::Cancelled,
        }
    }

    /// Create error from io::Error with automatic type and retriability detection
    pub fn from_io_error(err: std::io::Error, target: SocketAddr) -> Self {
        match err.kind() {
            std::io::ErrorKind::TimedOut => Self::Timeout {
                target,
                duration: Duration::from_secs(0), // Duration unknown from io::Error
                retriable: true,
            },
            std::io::ErrorKind::ConnectionRefused => Self::ConnectionFailed {
                target,
                reason: err.to_string(),
                retriable: false,
            },
            std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted => Self::ConnectionFailed {
                target,
                reason: err.to_string(),
                retriable: true,
            },
            _ => Self::ConnectionFailed {
                target,
                reason: err.to_string(),
                retriable: true, // Default to retriable for unknown errors
            },
        }
    }

    /// Create ResourceExhausted error for "too many open files"
    pub fn too_many_open_files(current: u64, suggested: u64) -> Self {
        Self::ResourceExhausted {
            resource: "file descriptors".to_string(),
            current,
            limit: current, // Use current as limit since we hit it
            suggestion: format!(
                "Reduce parallelism from {} to {} with --max-parallelism",
                current, suggested
            ),
        }
    }

    /// Create InsufficientPrivileges error with platform-specific suggestion
    pub fn insufficient_privileges(scan_type: &str) -> Self {
        let suggestion = if cfg!(target_os = "windows") {
            "Run as Administrator".to_string()
        } else {
            "Run with sudo or set CAP_NET_RAW capability: sudo setcap cap_net_raw+ep $(which prtip)"
                .to_string()
        };

        Self::InsufficientPrivileges {
            scan_type: scan_type.to_string(),
            suggestion,
        }
    }
}

/// Error category for progress tracking and statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    ConnectionFailed,
    Timeout,
    RateLimit,
    HostUnreachable,
    PermissionDenied,
    TooManyOpenFiles,
    ProbeFailed,
    ConfigError,
    Cancelled,
}

impl ErrorCategory {
    /// Returns human-readable name for this category
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConnectionFailed => "Connection Failed",
            Self::Timeout => "Timeout",
            Self::RateLimit => "Rate Limit",
            Self::HostUnreachable => "Host Unreachable",
            Self::PermissionDenied => "Permission Denied",
            Self::TooManyOpenFiles => "Too Many Open Files",
            Self::ProbeFailed => "Probe Failed",
            Self::ConfigError => "Configuration Error",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Returns whether errors in this category should be displayed in progress output
    pub fn is_reportable(&self) -> bool {
        match self {
            Self::Timeout | Self::ConnectionFailed | Self::ProbeFailed => false, // Common, don't spam
            Self::PermissionDenied | Self::TooManyOpenFiles | Self::ConfigError => true, // Critical
            Self::RateLimit | Self::HostUnreachable | Self::Cancelled => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_connection_failed_retriability() {
        let target = "127.0.0.1:80".parse().unwrap();

        // Connection refused should NOT be retriable
        let err_refused = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let scanner_err = ScannerError::from_io_error(err_refused, target);
        assert!(!scanner_err.is_retriable());

        // Timeout SHOULD be retriable
        let err_timeout = io::Error::new(io::ErrorKind::TimedOut, "timeout");
        let scanner_err = ScannerError::from_io_error(err_timeout, target);
        assert!(scanner_err.is_retriable());

        // Interrupted SHOULD be retriable
        let err_interrupted = io::Error::new(io::ErrorKind::Interrupted, "interrupted");
        let scanner_err = ScannerError::from_io_error(err_interrupted, target);
        assert!(scanner_err.is_retriable());
    }

    #[test]
    fn test_rate_limit_retriability() {
        let err = ScannerError::RateLimitExceeded {
            current_rate: 100_000,
            max_rate: 50_000,
        };
        assert!(err.is_retriable());
    }

    #[test]
    fn test_resource_exhausted_retriability() {
        let err = ScannerError::ResourceExhausted {
            resource: "file descriptors".to_string(),
            current: 1024,
            limit: 1024,
            suggestion: "Reduce parallelism".to_string(),
        };
        assert!(err.is_retriable());
    }

    #[test]
    fn test_insufficient_privileges_not_retriable() {
        let err = ScannerError::insufficient_privileges("SYN");
        assert!(!err.is_retriable());
    }

    #[test]
    fn test_recovery_suggestions() {
        let err = ScannerError::RateLimitExceeded {
            current_rate: 100_000,
            max_rate: 50_000,
        };
        assert!(err.recovery_suggestion().is_some());
        assert!(err
            .recovery_suggestion()
            .unwrap()
            .contains("Reduce scan rate"));

        let err = ScannerError::insufficient_privileges("SYN");
        assert!(err.recovery_suggestion().is_some());

        let err = ScannerError::Cancelled {
            reason: "User interrupt".to_string(),
        };
        assert!(err.recovery_suggestion().is_none());
    }

    #[test]
    fn test_error_categories() {
        let target = "127.0.0.1:80".parse().unwrap();
        let err =
            ScannerError::from_io_error(io::Error::new(io::ErrorKind::TimedOut, "timeout"), target);
        assert_eq!(err.category(), ErrorCategory::Timeout);

        let err = ScannerError::RateLimitExceeded {
            current_rate: 100_000,
            max_rate: 50_000,
        };
        assert_eq!(err.category(), ErrorCategory::RateLimit);
    }

    #[test]
    fn test_too_many_open_files_constructor() {
        let err = ScannerError::too_many_open_files(1024, 512);
        assert!(err.is_retriable());
        assert!(err.recovery_suggestion().is_some());
        assert!(err
            .recovery_suggestion()
            .unwrap()
            .contains("Reduce parallelism"));
        assert_eq!(err.category(), ErrorCategory::TooManyOpenFiles);
    }

    #[test]
    fn test_error_category_names() {
        assert_eq!(ErrorCategory::ConnectionFailed.name(), "Connection Failed");
        assert_eq!(ErrorCategory::Timeout.name(), "Timeout");
        assert_eq!(ErrorCategory::PermissionDenied.name(), "Permission Denied");
    }

    #[test]
    fn test_error_category_reportability() {
        assert!(!ErrorCategory::Timeout.is_reportable()); // Common, don't spam
        assert!(ErrorCategory::PermissionDenied.is_reportable()); // Critical
        assert!(ErrorCategory::TooManyOpenFiles.is_reportable()); // Critical
    }

    #[test]
    fn test_error_display() {
        let target = "127.0.0.1:80".parse().unwrap();
        let err = ScannerError::ConnectionFailed {
            target,
            reason: "Connection refused".to_string(),
            retriable: false,
        };
        let display = err.to_string();
        assert!(display.contains("127.0.0.1:80"));
        assert!(display.contains("Connection refused"));
    }
}
