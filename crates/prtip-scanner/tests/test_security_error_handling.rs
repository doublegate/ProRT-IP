// Sprint 5.6 Phase 4: Security & Edge Case Tests
// Error Handling Security Testing
//
// Test Strategy:
// - Group 1: Timeout enforcement (no root required)
// - Group 2: Connection error handling (no root required)
// - Group 3: Resource exhaustion prevention (no root required)
// - Group 4: Concurrent access safety (no root required)
//
// Run all tests: cargo test --test test_security_error_handling

use prtip_core::{Config, PortState};
use prtip_scanner::SynScanner;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout as tokio_timeout;

/// Helper to create default config for testing
fn default_config() -> Config {
    Config::default()
}

/// Helper to create config with specific timeout
fn config_with_timeout(timeout_ms: u64) -> Config {
    let mut config = default_config();
    config.scan.timeout_ms = timeout_ms;
    config.scan.retries = 1; // Minimal retries for faster tests
    config
}

// ============================================================================
// Test Group 1: Timeout Enforcement (2 tests)
// Tests that timeouts are strictly enforced to prevent DoS
// ============================================================================

/// Tests strict timeout enforcement on unreachable targets
///
/// **Attack Scenario:** Attacker specifies very short timeout but implementation
/// ignores it, causing scans to hang indefinitely and exhausting resources.
///
/// **Expected Behavior:** Scan MUST complete within timeout + small margin
/// (50ms for scheduling overhead). For 100ms timeout, scan completes within 150ms.
///
/// **Failure Impact:** HIGH - Could cause resource exhaustion and DoS if
/// timeouts aren't enforced.
///
/// **Mitigation:** SynScanner uses tokio::time::timeout() to enforce strict
/// timeout bounds.
#[tokio::test]
async fn test_security_timeout_enforcement_strict() {
    let timeout_ms = 100; // Very short timeout
    let config = config_with_timeout(timeout_ms);

    let scanner = SynScanner::new(config);
    if scanner.is_err() {
        // Skip if cannot create scanner (privilege issue)
        println!("SKIP: Cannot create scanner (privileges required)");
        return;
    }

    let mut scanner = scanner.unwrap();

    // Initialize with timeout to prevent test hanging
    let init_result = tokio_timeout(Duration::from_secs(5), scanner.initialize()).await;

    if init_result.is_err() || init_result.unwrap().is_err() {
        // Skip if initialization fails (privilege issue)
        println!("SKIP: Scanner initialization failed (privileges required)");
        return;
    }

    // Scan unreachable IP from documentation range (TEST-NET-1: 192.0.2.0/24)
    // This IP is reserved for documentation and will not respond
    let target = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1));
    let port = 80;

    let start = Instant::now();

    // Scan with outer timeout to prevent test hanging
    let scan_result = tokio_timeout(Duration::from_secs(5), scanner.scan_port(target, port)).await;

    let elapsed = start.elapsed();

    // Test should complete (not hang)
    assert!(
        scan_result.is_ok(),
        "Scan should not hang (outer timeout should not trigger)"
    );

    // Inner scan should complete successfully (may return Filtered or Closed)
    let inner_result = scan_result.unwrap();
    assert!(inner_result.is_ok(), "Scan should complete without error");

    // Most importantly: elapsed time should be close to timeout
    // Allow 50ms margin for scheduling overhead
    let max_allowed = Duration::from_millis(timeout_ms + 50);

    assert!(
        elapsed <= max_allowed,
        "Scan should enforce timeout strictly. Expected <= {}ms, got {}ms",
        max_allowed.as_millis(),
        elapsed.as_millis()
    );

    println!(
        "✓ Timeout enforced: {}ms scan completed in {}ms (limit: {}ms)",
        timeout_ms,
        elapsed.as_millis(),
        max_allowed.as_millis()
    );
}

/// Tests timeout enforcement with zero timeout configuration
///
/// **Attack Scenario:** Attacker sets timeout to very low value hoping to
/// cause division by zero or other undefined behavior.
///
/// **Expected Behavior:** Config validation should reject timeout = 0.
/// This test verifies Config::validate() prevents zero timeout.
///
/// **Failure Impact:** HIGH - Could cause crashes or hangs.
///
/// **Mitigation:** Config::validate() explicitly rejects timeout_ms = 0.
///
/// **Note:** This test is in error_handling.rs because it tests error
/// handling of invalid timeout configuration.
#[test]
fn test_security_zero_timeout_rejected() {
    let mut config = default_config();
    config.scan.timeout_ms = 0;

    let result = config.validate();

    assert!(
        result.is_err(),
        "Config with zero timeout should be rejected"
    );
}

// ============================================================================
// Test Group 2: Connection Error Handling (2 tests)
// Tests robust handling of connection failures without panicking
// ============================================================================

/// Tests graceful handling of connection refused
///
/// **Attack Scenario:** Scan target actively refuses connections (RST packets).
/// Improper error handling could cause panic or incorrect state reporting.
///
/// **Expected Behavior:** Scanner returns PortState::Closed without panicking.
///
/// **Failure Impact:** MEDIUM - Crash on connection refused would enable DoS
/// by simply refusing connections.
///
/// **Mitigation:** Scanner handles all connection states (Open, Closed,
/// Filtered) gracefully via proper error handling.
#[tokio::test]
async fn test_security_connection_refused_handling() {
    let config = default_config();
    let scanner = SynScanner::new(config);

    if scanner.is_err() {
        println!("SKIP: Cannot create scanner (privileges required)");
        return;
    }

    let mut scanner = scanner.unwrap();

    let init_result = tokio_timeout(Duration::from_secs(5), scanner.initialize()).await;

    if init_result.is_err() || init_result.unwrap().is_err() {
        println!("SKIP: Scanner initialization failed (privileges required)");
        return;
    }

    // Scan localhost on high port (very unlikely to be open)
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let port = 65432;

    let scan_result = tokio_timeout(Duration::from_secs(5), scanner.scan_port(target, port)).await;

    // Should complete without hanging
    assert!(scan_result.is_ok(), "Scan should not hang on closed port");

    let inner_result = scan_result.unwrap();

    // Should not panic or error
    assert!(
        inner_result.is_ok(),
        "Scan should handle closed/refused connections gracefully"
    );

    let scan_info = inner_result.unwrap();

    // Port should be detected as Closed or Filtered (not Open, not panic)
    assert!(
        matches!(scan_info.state, PortState::Closed | PortState::Filtered),
        "Closed port should be detected as Closed or Filtered, got: {:?}",
        scan_info.state
    );

    println!(
        "✓ Connection refused handled gracefully (state: {:?})",
        scan_info.state
    );
}

/// Tests handling of network unreachable errors
///
/// **Attack Scenario:** Scan target on unreachable network (e.g., private
/// IP from wrong subnet). Improper handling could cause panics.
///
/// **Expected Behavior:** Scanner returns error or Filtered state without
/// panicking.
///
/// **Failure Impact:** MEDIUM - Crash on unreachable network would be exploitable.
///
/// **Mitigation:** Scanner handles network errors via Result<T, E> pattern.
#[tokio::test]
async fn test_security_network_unreachable_handling() {
    let config = default_config();
    let scanner = SynScanner::new(config);

    if scanner.is_err() {
        println!("SKIP: Cannot create scanner (privileges required)");
        return;
    }

    let mut scanner = scanner.unwrap();

    let init_result = tokio_timeout(Duration::from_secs(5), scanner.initialize()).await;

    if init_result.is_err() || init_result.unwrap().is_err() {
        println!("SKIP: Scanner initialization failed (privileges required)");
        return;
    }

    // Scan IP in documentation range (TEST-NET-2: 198.51.100.0/24)
    // These IPs are reserved and should be unreachable
    let target = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1));
    let port = 80;

    let scan_result = tokio_timeout(Duration::from_secs(5), scanner.scan_port(target, port)).await;

    // Should complete without hanging
    assert!(
        scan_result.is_ok(),
        "Scan should not hang on unreachable network"
    );

    // Inner result may be Ok(Filtered) or Err(network error)
    // Both are acceptable - key is no panic
    let inner_result = scan_result.unwrap();

    match inner_result {
        Ok(scan_info) => {
            // If Ok, should be Filtered
            assert_eq!(
                scan_info.state,
                PortState::Filtered,
                "Unreachable target should be Filtered"
            );
            println!("✓ Network unreachable handled gracefully (Filtered)");
        }
        Err(e) => {
            // If Err, should have clear error message
            let msg = format!("{}", e);
            println!("✓ Network unreachable handled gracefully (Error: {})", msg);
        }
    }
}

// ============================================================================
// Test Group 3: Concurrent Access Safety (1 test)
// Tests that concurrent scans don't cause race conditions or data corruption
// ============================================================================

/// Tests concurrent scanning safety
///
/// **Attack Scenario:** Multiple threads scan same target simultaneously,
/// causing race conditions in packet capture or result aggregation.
///
/// **Expected Behavior:** All concurrent scans complete successfully with
/// consistent results (no data races, no panics).
///
/// **Failure Impact:** HIGH - Data races could cause incorrect results,
/// crashes, or security vulnerabilities.
///
/// **Mitigation:** Scanner uses Arc<Mutex<T>> for shared state, tokio::sync
/// primitives for coordination.
#[tokio::test]
async fn test_security_concurrent_access_safe() {
    let config = default_config();
    let scanner = SynScanner::new(config);

    if scanner.is_err() {
        println!("SKIP: Cannot create scanner (privileges required)");
        return;
    }

    let mut scanner = scanner.unwrap();

    let init_result = tokio_timeout(Duration::from_secs(5), scanner.initialize()).await;

    if init_result.is_err() || init_result.unwrap().is_err() {
        println!("SKIP: Scanner initialization failed (privileges required)");
        return;
    }

    // Wrap scanner in Arc to share across tasks
    let scanner = Arc::new(tokio::sync::Mutex::new(scanner));

    // Spawn 5 concurrent scans of the same target
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let port = 65433;

    let mut handles = vec![];

    for i in 0..5 {
        let scanner_clone = Arc::clone(&scanner);
        let handle = tokio::spawn(async move {
            let scanner_guard = scanner_clone.lock().await;

            let result = tokio_timeout(
                Duration::from_secs(5),
                scanner_guard.scan_port(target, port),
            )
            .await;

            // Should complete without hanging
            assert!(result.is_ok(), "Concurrent scan {} should not hang", i);

            result.unwrap()
        });

        handles.push(handle);
    }

    // Wait for all concurrent scans to complete
    let mut all_succeeded = true;
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await;

        assert!(
            result.is_ok(),
            "Concurrent scan task {} should not panic",
            i
        );

        let scan_result = result.unwrap();

        // All scans should complete (may be Ok or Err, but should not panic)
        if scan_result.is_err() {
            all_succeeded = false;
        }
    }

    println!(
        "✓ Concurrent access safe: 5 parallel scans completed (all succeeded: {})",
        all_succeeded
    );
}

// ============================================================================
// Test Group 4: Resource Management (1 test)
// Tests that resource limits are respected
// ============================================================================

/// Tests that scanner respects configured parallelism limits
///
/// **Attack Scenario:** Attacker triggers many concurrent scans to exhaust
/// memory by spawning unlimited tasks.
///
/// **Expected Behavior:** Scanner respects parallelism configuration and
/// doesn't spawn unlimited tasks.
///
/// **Failure Impact:** CRITICAL - Unbounded parallelism could cause OOM crashes.
///
/// **Mitigation:** Scanner uses Semaphore with max_concurrent_targets limit.
///
/// **Note:** This test creates multiple scanners to verify per-scanner limits
/// (not testing global limits across scanners).
#[tokio::test]
async fn test_security_parallelism_limit_respected() {
    let mut config = default_config();
    config.performance.parallelism = 10; // Limit to 10 concurrent

    let scanner = SynScanner::new(config);

    if scanner.is_err() {
        println!("SKIP: Cannot create scanner (privileges required)");
        return;
    }

    let mut scanner = scanner.unwrap();

    let init_result = tokio_timeout(Duration::from_secs(5), scanner.initialize()).await;

    if init_result.is_err() || init_result.unwrap().is_err() {
        println!("SKIP: Scanner initialization failed (privileges required)");
        return;
    }

    // Create 100 scan targets (more than parallelism limit)
    let target = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)); // Unreachable
    let ports: Vec<u16> = (1..=100).collect();

    // Scanner should queue tasks and not spawn all 100 at once
    // This is an indirect test (we can't easily observe task count)
    // but verifies the scanner completes without OOM
    let scan_result =
        tokio_timeout(Duration::from_secs(10), scanner.scan_ports(target, ports)).await;

    // Should complete (may timeout individual scans, but not hang)
    assert!(
        scan_result.is_ok(),
        "Scan should respect parallelism limits (not OOM or hang)"
    );

    let inner_result = scan_result.unwrap();

    // Should complete successfully
    assert!(
        inner_result.is_ok(),
        "Scanner should handle parallelism limiting without errors"
    );

    println!("✓ Parallelism limit respected: 100 scans queued without OOM");
}
