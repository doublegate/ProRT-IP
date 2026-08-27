//! Integration tests for prtip-network

use prtip_network::*;

#[test]
fn test_privilege_check() {
    // Should not panic
    let result = has_raw_socket_capability();
    assert!(result.is_ok());

    // Log result for manual verification
    if let Ok(has_cap) = result {
        println!("Has raw socket capability: {}", has_cap);
    }
}

#[test]
fn test_check_privileges_function() {
    // Should return Ok or Err depending on privileges, but not panic
    let result = check_privileges();

    match result {
        Ok(()) => {
            println!("Privilege check passed (running with elevated privileges)");
        }
        Err(e) => {
            println!(
                "Privilege check failed (expected without elevated privileges): {}",
                e
            );
        }
    }
}

#[test]
fn test_create_capture() {
    // Should create a capture instance without error
    let result = create_capture();
    assert!(result.is_ok());
}

#[test]
fn test_capture_open_close_cycle() {
    let result = create_capture();
    assert!(result.is_ok());

    let mut capture = result.unwrap();

    // Try to close without opening (should succeed)
    let close_result = capture.close();
    assert!(close_result.is_ok());
}

#[test]
fn test_packet_operations_without_open() {
    let mut capture = create_capture().unwrap();

    // Try to send without opening (should fail)
    let send_result = capture.send_packet(&[0u8; 64]);
    assert!(send_result.is_err());

    // Try to receive without opening (should fail)
    let recv_result = capture.receive_packet(100);
    assert!(recv_result.is_err());
}

#[test]
fn test_send_invalid_packets() {
    let mut capture = create_capture().unwrap();

    // Empty packet should fail
    let result = capture.send_packet(&[]);
    assert!(result.is_err());

    // Oversized packet should fail
    let huge_packet = vec![0u8; 70000];
    let result = capture.send_packet(&huge_packet);
    assert!(result.is_err());
}

// Note: Tests that actually open interfaces require elevated privileges
// and are environment-specific, so they're not included here.
// Manual testing with elevated privileges is recommended.
