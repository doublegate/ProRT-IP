//! Error injection testing
//!
//! Tests error handling using the error injection framework.
//! Sprint 4.22 Phase 7: Comprehensive Testing

mod common;

use common::error_injection::{ErrorInjector, FailureMode};
use prtip_scanner::ScannerError;
use std::io;
use std::time::Duration;

#[test]
fn test_connection_refused_error_conversion() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::ConnectionRefused);

    let result = injector.inject_connection_error();
    assert!(result.is_err());

    // Convert to ScannerError
    let scanner_err = ScannerError::from_io_error(result.unwrap_err(), target);
    assert!(!scanner_err.is_retriable()); // Connection refused should NOT be retriable
}

#[test]
fn test_timeout_error_conversion() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::Timeout(Duration::from_secs(5)));

    let result = injector.inject_connection_error();
    assert!(result.is_err());

    // Convert to ScannerError
    let scanner_err = ScannerError::from_io_error(result.unwrap_err(), target);
    assert!(scanner_err.is_retriable()); // Timeout SHOULD be retriable
}

#[test]
fn test_connection_reset_is_retriable() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::ConnectionReset);

    let result = injector.inject_connection_error();
    assert!(result.is_err());

    // Convert to ScannerError
    let scanner_err = ScannerError::from_io_error(result.unwrap_err(), target);
    assert!(scanner_err.is_retriable()); // Reset SHOULD be retriable
}

#[test]
fn test_connection_aborted_is_retriable() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::ConnectionAborted);

    let result = injector.inject_connection_error();
    assert!(result.is_err());

    // Convert to ScannerError
    let scanner_err = ScannerError::from_io_error(result.unwrap_err(), target);
    assert!(scanner_err.is_retriable()); // Aborted SHOULD be retriable
}

#[test]
fn test_would_block_is_retriable() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::WouldBlock);

    let result = injector.inject_connection_error();
    assert!(result.is_err());

    // Convert to ScannerError
    let scanner_err = ScannerError::from_io_error(result.unwrap_err(), target);
    assert!(scanner_err.is_retriable()); // WouldBlock SHOULD be retriable
}

#[test]
fn test_interrupted_is_retriable() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::Interrupted);

    let result = injector.inject_connection_error();
    assert!(result.is_err());

    // Convert to ScannerError
    let scanner_err = ScannerError::from_io_error(result.unwrap_err(), target);
    assert!(scanner_err.is_retriable()); // Interrupted SHOULD be retriable
}

#[test]
fn test_success_after_retries() {
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
fn test_reset_clears_attempt_count() {
    let target = "127.0.0.1:80".parse().unwrap();
    let injector = ErrorInjector::new(target, FailureMode::ConnectionRefused);

    // Make some attempts
    let _ = injector.inject_connection_error();
    let _ = injector.inject_connection_error();
    assert_eq!(injector.attempt_count(), 2);

    // Reset should clear count
    injector.reset();
    assert_eq!(injector.attempt_count(), 0);
}

#[test]
fn test_network_unreachable_not_retriable() {
    let mode = FailureMode::NetworkUnreachable;
    assert!(!mode.is_retriable());
}

#[test]
fn test_host_unreachable_not_retriable() {
    let mode = FailureMode::HostUnreachable;
    assert!(!mode.is_retriable());
}

#[test]
fn test_too_many_open_files_is_retriable() {
    let mode = FailureMode::TooManyOpenFiles;
    assert!(mode.is_retriable());

    let result = mode.to_io_error();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too many open files"));
}
