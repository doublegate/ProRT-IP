//! Error injection testing infrastructure
//!
//! Provides mock implementations and failure simulation for testing error handling:
//! - Network failures (connection refused, timeout, reset)
//! - Malformed responses (truncated, invalid encoding)
//! - Resource exhaustion (file descriptors, memory)
//! - Deterministic failure modes (no flaky tests)
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing

use std::io;
use std::net::SocketAddr;
use std::time::Duration;

/// Failure mode for error injection
#[derive(Debug, Clone)]
pub enum FailureMode {
    /// Connection refused (ECONNREFUSED)
    ConnectionRefused,

    /// Operation timed out (ETIMEDOUT)
    Timeout(Duration),

    /// Network unreachable (ENETUNREACH)
    NetworkUnreachable,

    /// Host unreachable (EHOSTUNREACH)
    HostUnreachable,

    /// Connection reset by peer (ECONNRESET)
    ConnectionReset,

    /// Connection aborted (ECONNABORTED)
    ConnectionAborted,

    /// Would block / try again (EWOULDBLOCK)
    WouldBlock,

    /// Operation interrupted (EINTR)
    Interrupted,

    /// Too many open files (EMFILE)
    TooManyOpenFiles,

    /// Malformed response (truncated data)
    MalformedResponse { data: Vec<u8> },

    /// Invalid encoding (bad UTF-8)
    InvalidEncoding { data: Vec<u8> },

    /// Success after N attempts
    SuccessAfter { attempts: u32 },

    /// Probabilistic failure (0.0 = never, 1.0 = always)
    Probabilistic { rate: f64 },
}

impl FailureMode {
    /// Convert failure mode to io::Error
    pub fn to_io_error(&self) -> io::Result<()> {
        match self {
            Self::ConnectionRefused => {
                Err(io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused"))
            }
            Self::Timeout(_) => {
                Err(io::Error::new(io::ErrorKind::TimedOut, "operation timed out"))
            }
            Self::NetworkUnreachable => {
                Err(io::Error::new(io::ErrorKind::Other, "network unreachable"))
            }
            Self::HostUnreachable => {
                Err(io::Error::new(io::ErrorKind::Other, "host unreachable"))
            }
            Self::ConnectionReset => {
                Err(io::Error::new(io::ErrorKind::ConnectionReset, "connection reset"))
            }
            Self::ConnectionAborted => {
                Err(io::Error::new(io::ErrorKind::ConnectionAborted, "connection aborted"))
            }
            Self::WouldBlock => {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "would block"))
            }
            Self::Interrupted => {
                Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"))
            }
            Self::TooManyOpenFiles => {
                Err(io::Error::new(io::ErrorKind::Other, "too many open files"))
            }
            _ => Ok(()), // Other modes handled differently
        }
    }

    /// Check if error should be retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_)
                | Self::WouldBlock
                | Self::Interrupted
                | Self::ConnectionReset
                | Self::ConnectionAborted
                | Self::TooManyOpenFiles
        )
    }
}

/// Test helper to create common failure modes
pub struct ErrorInjector {
    target: SocketAddr,
    failure_mode: FailureMode,
    attempt_count: std::cell::RefCell<u32>,
}

impl ErrorInjector {
    /// Create new error injector for target
    pub fn new(target: SocketAddr, failure_mode: FailureMode) -> Self {
        Self {
            target,
            failure_mode,
            attempt_count: std::cell::RefCell::new(0),
        }
    }

    /// Simulate connection attempt and return result
    pub fn inject_connection_error(&self) -> io::Result<()> {
        let mut count = self.attempt_count.borrow_mut();
        *count += 1;

        match &self.failure_mode {
            FailureMode::SuccessAfter { attempts } => {
                if *count >= *attempts {
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::ConnectionRefused, "not yet"))
                }
            }
            FailureMode::Probabilistic { rate } => {
                use rand::Rng;
                if rand::thread_rng().gen::<f64>() < *rate {
                    Err(io::Error::new(io::ErrorKind::ConnectionRefused, "probabilistic failure"))
                } else {
                    Ok(())
                }
            }
            _ => self.failure_mode.to_io_error(),
        }
    }

    /// Get current attempt count
    pub fn attempt_count(&self) -> u32 {
        *self.attempt_count.borrow()
    }

    /// Reset attempt count
    pub fn reset(&self) {
        *self.attempt_count.borrow_mut() = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_refused_error() {
        let target = "127.0.0.1:80".parse().unwrap();
        let injector = ErrorInjector::new(target, FailureMode::ConnectionRefused);

        let result = injector.inject_connection_error();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::ConnectionRefused);
    }

    #[test]
    fn test_timeout_error() {
        let target = "127.0.0.1:80".parse().unwrap();
        let injector = ErrorInjector::new(target, FailureMode::Timeout(Duration::from_secs(5)));

        let result = injector.inject_connection_error();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn test_retriable_errors() {
        assert!(FailureMode::Timeout(Duration::from_secs(1)).is_retriable());
        assert!(FailureMode::Interrupted.is_retriable());
        assert!(FailureMode::WouldBlock.is_retriable());
        assert!(!FailureMode::ConnectionRefused.is_retriable());
        assert!(!FailureMode::NetworkUnreachable.is_retriable());
    }

    #[test]
    fn test_success_after_attempts() {
        let target = "127.0.0.1:80".parse().unwrap();
        let injector = ErrorInjector::new(target, FailureMode::SuccessAfter { attempts: 3 });

        // First 2 attempts should fail
        assert!(injector.inject_connection_error().is_err());
        assert_eq!(injector.attempt_count(), 1);
        assert!(injector.inject_connection_error().is_err());
        assert_eq!(injector.attempt_count(), 2);

        // 3rd attempt should succeed
        assert!(injector.inject_connection_error().is_ok());
        assert_eq!(injector.attempt_count(), 3);
    }

    #[test]
    fn test_reset_attempt_count() {
        let target = "127.0.0.1:80".parse().unwrap();
        let injector = ErrorInjector::new(target, FailureMode::ConnectionRefused);

        assert_eq!(injector.attempt_count(), 0);
        let _ = injector.inject_connection_error();
        assert_eq!(injector.attempt_count(), 1);

        injector.reset();
        assert_eq!(injector.attempt_count(), 0);
    }

    #[test]
    fn test_connection_reset_retriable() {
        let mode = FailureMode::ConnectionReset;
        assert!(mode.is_retriable());
        let result = mode.to_io_error();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::ConnectionReset);
    }

    #[test]
    fn test_connection_aborted_retriable() {
        let mode = FailureMode::ConnectionAborted;
        assert!(mode.is_retriable());
        let result = mode.to_io_error();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::ConnectionAborted);
    }

    #[test]
    fn test_would_block_retriable() {
        let mode = FailureMode::WouldBlock;
        assert!(mode.is_retriable());
        let result = mode.to_io_error();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn test_interrupted_retriable() {
        let mode = FailureMode::Interrupted;
        assert!(mode.is_retriable());
        let result = mode.to_io_error();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Interrupted);
    }

    #[test]
    fn test_too_many_open_files_retriable() {
        let mode = FailureMode::TooManyOpenFiles;
        assert!(mode.is_retriable());
        let result = mode.to_io_error();
        assert!(result.is_err());
    }
}
