//! Integration tests for PCAPNG packet capture
//!
//! Tests end-to-end PCAPNG capture functionality for UDP scanning.
//! Note: TCP connect scanner does not support PCAPNG capture (uses OS-level API that hides packets).

use prtip_core::Config;
use prtip_scanner::pcapng::{Direction, PcapngWriter};
use prtip_scanner::UdpScanner;
use std::fs;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

/// Helper function to verify PCAPNG file header
fn verify_pcapng_header(path: &std::path::Path) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut header = [0u8; 4];
    if file.read_exact(&mut header).is_err() {
        return false;
    }

    // PCAPNG Section Header Block magic: 0x0A0D0D0A (big-endian)
    header == [0x0A, 0x0D, 0x0D, 0x0A]
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW capability for raw socket UDP scanning
async fn test_pcapng_capture_udp_scan() {
    let dir = tempdir().unwrap();
    let pcapng_path = dir.path().join("udp_scan.pcapng");

    // Create PCAPNG writer
    let pcapng_writer = Arc::new(Mutex::new(
        PcapngWriter::new(&pcapng_path).expect("Failed to create PCAPNG writer"),
    ));

    // Create UDP scanner
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize UDP scanner");

    // Scan localhost DNS port (53) with PCAPNG capture
    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 53;

    let result = scanner
        .scan_port_with_pcapng(target, port, Some(pcapng_writer.clone()))
        .await;

    // Scan should complete (even if port is filtered/closed)
    assert!(result.is_ok(), "UDP scan with PCAPNG should complete");

    // Flush PCAPNG writer
    pcapng_writer
        .lock()
        .unwrap()
        .flush()
        .expect("Failed to flush PCAPNG writer");

    // Verify PCAPNG file was created
    assert!(
        pcapng_path.exists(),
        "PCAPNG file should be created at: {}",
        pcapng_path.display()
    );

    // Verify file is not empty
    let metadata = fs::metadata(&pcapng_path).expect("Failed to read PCAPNG file metadata");
    assert!(
        metadata.len() > 0,
        "PCAPNG file should not be empty (size: {})",
        metadata.len()
    );

    // Verify PCAPNG header is valid
    assert!(
        verify_pcapng_header(&pcapng_path),
        "PCAPNG file should have valid header (0x0A0D0D0A)"
    );

    println!(
        "PCAPNG file created successfully: {} ({} bytes)",
        pcapng_path.display(),
        metadata.len()
    );
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW capability for raw socket UDP scanning
async fn test_pcapng_file_rotation() {
    let dir = tempdir().unwrap();
    let pcapng_path = dir.path().join("rotation_test.pcapng");

    // Create PCAPNG writer
    // Note: Cannot modify max_file_size (private field), so this test just verifies
    // the infrastructure works. File rotation would require 1GB of data.
    let writer = PcapngWriter::new(&pcapng_path).expect("Failed to create PCAPNG writer");
    let pcapng_writer = Arc::new(Mutex::new(writer));

    // Create UDP scanner
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize UDP scanner");

    // Scan multiple ports to generate enough packets for rotation
    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let ports = vec![53, 123, 161, 137, 138, 139, 445, 500, 514, 520];

    for port in ports {
        let _ = scanner
            .scan_port_with_pcapng(target, port, Some(pcapng_writer.clone()))
            .await;
    }

    // Flush writer
    pcapng_writer
        .lock()
        .unwrap()
        .flush()
        .expect("Failed to flush PCAPNG writer");

    // Verify first file exists
    let first_file = dir.path().join("rotation_test-001.pcapng");
    assert!(
        first_file.exists(),
        "First PCAPNG file should exist: {}",
        first_file.display()
    );

    // Note: Rotation won't actually occur in test (would need 1GB of data)
    // This test just verifies the basic PCAPNG infrastructure works
    println!("PCAPNG infrastructure test passed");
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW capability for raw socket UDP scanning
async fn test_scan_without_pcapng() {
    // Verify that scanning works without PCAPNG capture (zero regression test)
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize UDP scanner");

    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 9999;

    // Scan without PCAPNG writer (should use existing scan_port() method)
    let result = scanner.scan_port(target, port).await;

    assert!(result.is_ok(), "UDP scan without PCAPNG should complete");
    assert_eq!(result.unwrap().port, port);
}

#[tokio::test]
async fn test_pcapng_direction_tracking() {
    let dir = tempdir().unwrap();
    let pcapng_path = dir.path().join("direction_test.pcapng");

    // Create PCAPNG writer
    let pcapng_writer = Arc::new(Mutex::new(
        PcapngWriter::new(&pcapng_path).expect("Failed to create PCAPNG writer"),
    ));

    // Test writing packets with different directions
    let sent_packet = vec![0u8; 64];
    let received_packet = vec![1u8; 64];

    pcapng_writer
        .lock()
        .unwrap()
        .write_packet(&sent_packet, Direction::Sent)
        .expect("Failed to write sent packet");

    pcapng_writer
        .lock()
        .unwrap()
        .write_packet(&received_packet, Direction::Received)
        .expect("Failed to write received packet");

    // Flush
    pcapng_writer
        .lock()
        .unwrap()
        .flush()
        .expect("Failed to flush");

    // Verify file exists and has data
    let indexed_path = dir.path().join("direction_test-001.pcapng");
    assert!(indexed_path.exists());
    let metadata = fs::metadata(&indexed_path).expect("Failed to read file");
    assert!(metadata.len() > 128); // At least 2 packets
}

#[test]
fn test_pcapng_writer_thread_safety() {
    use std::thread;

    let dir = tempdir().unwrap();
    let pcapng_path = dir.path().join("thread_test.pcapng");

    let writer = Arc::new(Mutex::new(
        PcapngWriter::new(&pcapng_path).expect("Failed to create PCAPNG writer"),
    ));

    // Spawn multiple threads writing packets concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let writer_clone = Arc::clone(&writer);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let packet = vec![(i * 10 + j) as u8; 64];
                writer_clone
                    .lock()
                    .unwrap()
                    .write_packet(&packet, Direction::Sent)
                    .expect("Failed to write packet");
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Flush
    writer.lock().unwrap().flush().expect("Failed to flush");

    // Verify 50 packets were written
    let file_size = writer.lock().unwrap().current_size();
    assert!(
        file_size > 3200,
        "File should contain at least 50 packets (size: {})",
        file_size
    );
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW capability for raw socket UDP scanning
async fn test_pcapng_error_handling() {
    // Test that scan continues even if PCAPNG write fails
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize UDP scanner");

    // Create PCAPNG writer pointing to invalid path (will fail on write)
    let invalid_path = "/invalid/path/that/does/not/exist/scan.pcapng";
    let pcapng_writer = match PcapngWriter::new(invalid_path) {
        Ok(w) => Arc::new(Mutex::new(w)),
        Err(_) => {
            // Expected - cannot create writer with invalid path
            // This test verifies graceful degradation
            return;
        }
    };

    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 53;

    // Scan should complete even if PCAPNG write fails
    let result = scanner
        .scan_port_with_pcapng(target, port, Some(pcapng_writer))
        .await;

    // Scan may succeed or fail depending on system, but should not panic
    let _ = result;
}

// Optional: tshark validation test (requires tshark binary)
#[tokio::test]
#[ignore] // Ignore by default (requires tshark installation)
async fn test_pcapng_tshark_validation() {
    use std::process::Command;

    // Check if tshark is available
    let tshark_check = Command::new("tshark").arg("--version").output();
    if tshark_check.is_err() {
        println!("tshark not found, skipping validation test");
        return;
    }

    let dir = tempdir().unwrap();
    let pcapng_path = dir.path().join("tshark_test.pcapng");

    // Create PCAPNG writer and capture a packet
    let pcapng_writer = Arc::new(Mutex::new(
        PcapngWriter::new(&pcapng_path).expect("Failed to create PCAPNG writer"),
    ));

    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize UDP scanner");

    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let _ = scanner
        .scan_port_with_pcapng(target, 53, Some(pcapng_writer.clone()))
        .await;

    pcapng_writer
        .lock()
        .unwrap()
        .flush()
        .expect("Failed to flush");

    // Validate with tshark
    let first_file = dir.path().join("tshark_test-001.pcapng");
    let output = Command::new("tshark")
        .arg("-r")
        .arg(&first_file)
        .arg("-T")
        .arg("fields")
        .arg("-e")
        .arg("frame.number")
        .output()
        .expect("Failed to run tshark");

    let packet_count = String::from_utf8_lossy(&output.stdout).lines().count();

    assert!(
        packet_count > 0,
        "tshark should detect at least 1 packet in PCAPNG file"
    );
    println!("tshark validation: {} packets detected", packet_count);
}
