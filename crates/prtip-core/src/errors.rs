//! Enhanced error categorization for network scanning operations.
//!
//! This module provides detailed error categorization with actionable user messages
//! and suggestions for resolving common scanning issues.
//!
//! # Examples
//!
//! ```
//! use prtip_core::errors::{ScanError, ScanErrorKind};
//! use std::net::SocketAddr;
//!
//! let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
//! let io_error = std::io::Error::new(
//!     std::io::ErrorKind::ConnectionRefused,
//!     "connection refused"
//! );
//!
//! let scan_error = ScanError::from_io_error(io_error, addr);
//! assert_eq!(scan_error.kind(), ScanErrorKind::ConnectionRefused);
//! println!("{}", scan_error.user_message());
//! ```

use crate::progress::ErrorCategory;
use std::fmt;
use std::io;
use std::net::SocketAddr;

/// Categories of scan errors with specific meanings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScanErrorKind {
    /// Connection refused - port is closed
    ConnectionRefused,
    /// Connection timeout - port is filtered or dropped
    Timeout,
    /// Network unreachable - routing issue
    NetworkUnreachable,
    /// Host unreachable - host is down or firewalled
    HostUnreachable,
    /// Permission denied - insufficient privileges
    PermissionDenied,
    /// Too many open files - ulimit exceeded
    TooManyOpenFiles,
    /// Other unclassified errors
    Other,
}

impl ScanErrorKind {
    /// Converts to the corresponding progress tracking error category.
    pub fn to_error_category(self) -> ErrorCategory {
        match self {
            ScanErrorKind::ConnectionRefused => ErrorCategory::ConnectionRefused,
            ScanErrorKind::Timeout => ErrorCategory::Timeout,
            ScanErrorKind::NetworkUnreachable => ErrorCategory::NetworkUnreachable,
            ScanErrorKind::HostUnreachable => ErrorCategory::HostUnreachable,
            ScanErrorKind::PermissionDenied => ErrorCategory::PermissionDenied,
            ScanErrorKind::TooManyOpenFiles => ErrorCategory::TooManyOpenFiles,
            ScanErrorKind::Other => ErrorCategory::Other,
        }
    }

    /// Returns a user-friendly description of the error.
    pub fn description(self) -> &'static str {
        match self {
            ScanErrorKind::ConnectionRefused => "Connection refused",
            ScanErrorKind::Timeout => "Connection timeout",
            ScanErrorKind::NetworkUnreachable => "Network unreachable",
            ScanErrorKind::HostUnreachable => "Host unreachable",
            ScanErrorKind::PermissionDenied => "Permission denied",
            ScanErrorKind::TooManyOpenFiles => "Too many open files",
            ScanErrorKind::Other => "Unknown error",
        }
    }

    /// Returns an actionable suggestion for resolving the error.
    pub fn suggestion(self) -> Option<&'static str> {
        match self {
            ScanErrorKind::ConnectionRefused => Some("Port is closed or service is not running"),
            ScanErrorKind::Timeout => Some(
                "Port may be filtered by firewall, try increasing timeout or using stealth scans",
            ),
            ScanErrorKind::NetworkUnreachable => {
                Some("Check network connectivity and routing tables")
            }
            ScanErrorKind::HostUnreachable => {
                Some("Verify target is online and reachable, check firewall rules")
            }
            ScanErrorKind::PermissionDenied => {
                Some("Run with elevated privileges (sudo/root) or use CAP_NET_RAW capability")
            }
            ScanErrorKind::TooManyOpenFiles => {
                Some("Reduce batch size (--batch-size) or increase ulimit (ulimit -n)")
            }
            ScanErrorKind::Other => None,
        }
    }
}

/// Detailed scan error with context and actionable messages.
#[derive(Debug, Clone)]
pub struct ScanError {
    /// Error category
    kind: ScanErrorKind,
    /// Target socket address that failed
    target: SocketAddr,
    /// Detailed error message
    message: String,
    /// Optional suggestion for resolution
    suggestion: Option<String>,
}

impl ScanError {
    /// Creates a new scan error.
    pub fn new(kind: ScanErrorKind, target: SocketAddr, message: String) -> Self {
        let suggestion = kind.suggestion().map(String::from);
        Self {
            kind,
            target,
            message,
            suggestion,
        }
    }

    /// Creates a scan error from a standard I/O error.
    ///
    /// Automatically categorizes the error based on the I/O error kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::errors::{ScanError, ScanErrorKind};
    /// use std::net::SocketAddr;
    ///
    /// let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
    /// let io_error = std::io::Error::new(
    ///     std::io::ErrorKind::ConnectionRefused,
    ///     "connection refused"
    /// );
    ///
    /// let scan_error = ScanError::from_io_error(io_error, addr);
    /// assert_eq!(scan_error.kind(), ScanErrorKind::ConnectionRefused);
    /// ```
    pub fn from_io_error(err: io::Error, target: SocketAddr) -> Self {
        let kind = categorize_io_error(&err);
        let message = err.to_string();
        Self::new(kind, target, message)
    }

    /// Returns the error category.
    pub fn kind(&self) -> ScanErrorKind {
        self.kind
    }

    /// Returns the target address that failed.
    pub fn target(&self) -> SocketAddr {
        self.target
    }

    /// Returns the detailed error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the suggestion for resolution, if any.
    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    /// Returns a formatted user-friendly message with suggestion.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::errors::{ScanError, ScanErrorKind};
    /// use std::net::SocketAddr;
    ///
    /// let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
    /// let error = ScanError::new(
    ///     ScanErrorKind::ConnectionRefused,
    ///     addr,
    ///     "connection refused".to_string()
    /// );
    ///
    /// let message = error.user_message();
    /// assert!(message.contains("127.0.0.1:80"));
    /// assert!(message.contains("Connection refused"));
    /// ```
    pub fn user_message(&self) -> String {
        let mut msg = format!(
            "{} - {} ({})",
            self.target,
            self.kind.description(),
            self.message
        );

        if let Some(suggestion) = &self.suggestion {
            msg.push_str(&format!("\nSuggestion: {}", suggestion));
        }

        msg
    }

    /// Returns the error category for progress tracking.
    pub fn to_error_category(&self) -> ErrorCategory {
        self.kind.to_error_category()
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ScanError {}

/// Categorizes a standard I/O error into a scan error kind.
///
/// Maps `std::io::ErrorKind` values to more specific `ScanErrorKind` categories
/// for better error reporting and statistics tracking.
fn categorize_io_error(err: &io::Error) -> ScanErrorKind {
    match err.kind() {
        io::ErrorKind::ConnectionRefused => ScanErrorKind::ConnectionRefused,
        io::ErrorKind::TimedOut => ScanErrorKind::Timeout,
        io::ErrorKind::PermissionDenied => ScanErrorKind::PermissionDenied,
        _ => {
            // Check for POSIX error codes in the raw OS error
            if let Some(code) = err.raw_os_error() {
                match code {
                    // ENETUNREACH (101 on Linux)
                    101 => ScanErrorKind::NetworkUnreachable,
                    // EHOSTUNREACH (113 on Linux)
                    113 => ScanErrorKind::HostUnreachable,
                    // EMFILE (24 on Linux/macOS) - too many open files
                    24 => ScanErrorKind::TooManyOpenFiles,
                    // ENFILE (23 on Linux/macOS) - system file table full
                    23 => ScanErrorKind::TooManyOpenFiles,
                    _ => ScanErrorKind::Other,
                }
            } else {
                ScanErrorKind::Other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_description() {
        assert_eq!(
            ScanErrorKind::ConnectionRefused.description(),
            "Connection refused"
        );
        assert_eq!(ScanErrorKind::Timeout.description(), "Connection timeout");
    }

    #[test]
    fn test_error_kind_suggestion() {
        assert!(ScanErrorKind::ConnectionRefused.suggestion().is_some());
        assert!(ScanErrorKind::PermissionDenied.suggestion().is_some());
        assert!(ScanErrorKind::Other.suggestion().is_none());
    }

    #[test]
    fn test_error_kind_to_category() {
        assert_eq!(
            ScanErrorKind::Timeout.to_error_category(),
            ErrorCategory::Timeout
        );
        assert_eq!(
            ScanErrorKind::PermissionDenied.to_error_category(),
            ErrorCategory::PermissionDenied
        );
    }

    #[test]
    fn test_scan_error_new() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let error = ScanError::new(
            ScanErrorKind::ConnectionRefused,
            addr,
            "connection refused".to_string(),
        );

        assert_eq!(error.kind(), ScanErrorKind::ConnectionRefused);
        assert_eq!(error.target(), addr);
        assert_eq!(error.message(), "connection refused");
        assert!(error.suggestion().is_some());
    }

    #[test]
    fn test_from_io_error_connection_refused() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let io_error = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");

        let scan_error = ScanError::from_io_error(io_error, addr);
        assert_eq!(scan_error.kind(), ScanErrorKind::ConnectionRefused);
    }

    #[test]
    fn test_from_io_error_timeout() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let io_error = io::Error::new(io::ErrorKind::TimedOut, "timed out");

        let scan_error = ScanError::from_io_error(io_error, addr);
        assert_eq!(scan_error.kind(), ScanErrorKind::Timeout);
    }

    #[test]
    fn test_from_io_error_permission_denied() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");

        let scan_error = ScanError::from_io_error(io_error, addr);
        assert_eq!(scan_error.kind(), ScanErrorKind::PermissionDenied);
    }

    #[test]
    fn test_user_message() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let error = ScanError::new(
            ScanErrorKind::ConnectionRefused,
            addr,
            "connection refused".to_string(),
        );

        let message = error.user_message();
        assert!(message.contains("127.0.0.1:80"));
        assert!(message.contains("Connection refused"));
        assert!(message.contains("Suggestion:"));
    }

    #[test]
    fn test_display() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let error = ScanError::new(ScanErrorKind::Timeout, addr, "timed out".to_string());

        let display = format!("{}", error);
        assert!(display.contains("127.0.0.1:80"));
        assert!(display.contains("Connection timeout"));
    }

    #[test]
    fn test_categorize_io_error() {
        let err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        assert_eq!(categorize_io_error(&err), ScanErrorKind::ConnectionRefused);

        let err = io::Error::new(io::ErrorKind::TimedOut, "timeout");
        assert_eq!(categorize_io_error(&err), ScanErrorKind::Timeout);

        let err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        assert_eq!(categorize_io_error(&err), ScanErrorKind::PermissionDenied);

        let err = io::Error::new(io::ErrorKind::Other, "other");
        assert_eq!(categorize_io_error(&err), ScanErrorKind::Other);
    }

    #[test]
    fn test_error_category_conversion() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let error = ScanError::new(ScanErrorKind::Timeout, addr, "timeout".to_string());

        assert_eq!(error.to_error_category(), ErrorCategory::Timeout);
    }
}
