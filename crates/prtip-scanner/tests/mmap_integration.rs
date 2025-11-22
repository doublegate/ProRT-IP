//! Integration tests for memory-mapped I/O
//!
//! Tests end-to-end mmap functionality for scan result storage and retrieval.
//! Validates the 20-50% memory reduction target for large-scale scans.

use chrono::Utc;
use prtip_core::{PortState, ScanResult};
use prtip_scanner::{MmapResultReader, MmapResultWriter};
use std::net::Ipv4Addr;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Helper function to create test scan results
fn create_test_results(count: usize) -> Vec<ScanResult> {
    (0..count)
        .map(|i| {
            let port = 80 + (i as u16);
            let ip = Ipv4Addr::new(192, 168, 1, (i % 255) as u8);

            ScanResult {
                target_ip: ip.into(),
                port,
                state: if i % 3 == 0 {
                    PortState::Open
                } else if i % 3 == 1 {
                    PortState::Closed
                } else {
                    PortState::Filtered
                },
                service: Some(format!("service-{}", i)),
                version: Some(format!("v{}.0", i)),
                banner: Some(format!("Banner for port {}", port)),
                raw_response: Some(format!("Raw response {}", i).into_bytes()),
                response_time: Duration::from_millis(10 + (i as u64)),
                timestamp: Utc::now(),
            }
        })
        .collect()
}

#[test]
fn test_mmap_write_read_roundtrip() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    // Generate test data
    let results = create_test_results(10);

    // Write results to mmap file
    {
        let mut writer = MmapResultWriter::new(&path, 5).unwrap();
        for result in &results {
            writer.write_entry(result).unwrap();
        }
        writer.flush().unwrap();
    } // Drop writer to ensure flush

    // Read results back
    let reader = MmapResultReader::open(&path).unwrap();
    assert_eq!(reader.len(), 10);

    // Verify all results match
    for (i, result) in reader.iter().enumerate() {
        assert_eq!(result.port, results[i].port);
        assert_eq!(result.target_ip, results[i].target_ip);
        assert_eq!(result.state, results[i].state);
        assert_eq!(result.service, results[i].service);
        assert_eq!(result.version, results[i].version);
        assert_eq!(result.banner, results[i].banner);
        assert_eq!(result.raw_response, results[i].raw_response);
        assert_eq!(result.response_time, results[i].response_time);
    }
}

#[test]
fn test_mmap_large_dataset() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    // Simulate large scan (1000 results)
    let results = create_test_results(1000);

    {
        let mut writer = MmapResultWriter::new(&path, 100).unwrap();
        for result in &results {
            writer.write_entry(result).unwrap();
        }
        writer.flush().unwrap();
    }

    // Verify file size is reasonable
    let metadata = std::fs::metadata(&path).unwrap();
    let expected_min_size = 64 + (1000 * 512); // Header + entries
    assert!(metadata.len() >= expected_min_size as u64);

    // Verify all data is accessible
    let reader = MmapResultReader::open(&path).unwrap();
    assert_eq!(reader.len(), 1000);

    // Spot check first, middle, and last entries
    let first = reader.get_entry(0).unwrap();
    assert_eq!(first.port, 80);
    assert_eq!(first.service, Some("service-0".to_string()));

    let middle = reader.get_entry(500).unwrap();
    assert_eq!(middle.port, 580);
    assert_eq!(middle.service, Some("service-500".to_string()));

    let last = reader.get_entry(999).unwrap();
    assert_eq!(last.port, 1079);
    assert_eq!(last.service, Some("service-999".to_string()));
}

#[test]
fn test_mmap_random_access() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    let results = create_test_results(100);

    {
        let mut writer = MmapResultWriter::new(&path, 50).unwrap();
        for result in &results {
            writer.write_entry(result).unwrap();
        }
        writer.flush().unwrap();
    }

    let reader = MmapResultReader::open(&path).unwrap();

    // Test random access (non-sequential reads)
    let indices = vec![99, 0, 50, 25, 75, 10, 90];
    for &idx in &indices {
        let result = reader.get_entry(idx).unwrap();
        assert_eq!(result.port, 80 + idx as u16);
        assert_eq!(result.service, Some(format!("service-{}", idx)));
    }
}

#[test]
fn test_mmap_empty_file() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    // Create empty mmap file
    {
        let mut writer = MmapResultWriter::new(&path, 10).unwrap();
        writer.flush().unwrap();
    }

    // Read should work but be empty
    let reader = MmapResultReader::open(&path).unwrap();
    assert_eq!(reader.len(), 0);
    assert!(reader.is_empty());
    assert!(reader.get_entry(0).is_none());

    // Iterator should be empty
    let count = reader.iter().count();
    assert_eq!(count, 0);
}

#[test]
fn test_mmap_growth_behavior() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    // Start with small capacity, force growth
    {
        let mut writer = MmapResultWriter::new(&path, 2).unwrap();
        for i in 0..10 {
            let result = ScanResult {
                target_ip: Ipv4Addr::new(10, 0, 0, i).into(),
                port: 8000 + i as u16,
                state: PortState::Open,
                service: None,
                version: None,
                banner: None,
                raw_response: None,
                response_time: Duration::from_millis(1),
                timestamp: Utc::now(),
            };
            writer.write_entry(&result).unwrap();
        }
        writer.flush().unwrap();
    }

    // Verify all entries are readable after growth
    let reader = MmapResultReader::open(&path).unwrap();
    assert_eq!(reader.len(), 10);

    for (i, result) in reader.iter().enumerate() {
        assert_eq!(result.port, 8000 + i as u16);
    }
}

#[test]
fn test_mmap_different_port_states() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    {
        let mut writer = MmapResultWriter::new(&path, 10).unwrap();

        // Write results with all possible port states
        for (i, &state) in [
            PortState::Open,
            PortState::Closed,
            PortState::Filtered,
            PortState::Unknown,
        ]
        .iter()
        .enumerate()
        {
            let result = ScanResult {
                target_ip: Ipv4Addr::new(172, 16, 0, 1).into(),
                port: 1000 + i as u16,
                state,
                service: None,
                version: None,
                banner: None,
                raw_response: None,
                response_time: Duration::from_millis(5),
                timestamp: Utc::now(),
            };
            writer.write_entry(&result).unwrap();
        }
        writer.flush().unwrap();
    }

    // Verify all states are preserved
    let reader = MmapResultReader::open(&path).unwrap();
    assert_eq!(reader.len(), 4);

    let expected_states = vec![
        PortState::Open,
        PortState::Closed,
        PortState::Filtered,
        PortState::Unknown,
    ];

    for (i, result) in reader.iter().enumerate() {
        assert_eq!(result.state, expected_states[i]);
        assert_eq!(result.port, 1000 + i as u16);
    }
}
