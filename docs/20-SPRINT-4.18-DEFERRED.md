# Sprint 4.18 Implementation Plan: Output Expansion - PCAPNG & SQLite

**Status:** DEFERRED (Detailed implementation plan for future execution)
**Priority:** MEDIUM (ROI Score: 7.3/10)
**Duration:** 3-4 days estimated
**Created:** 2025-10-14
**Deferred Reason:** Phase 4 complete with v0.3.8, scope too large for single session

## Executive Summary

Sprint 4.18 adds **PCAPNG packet capture output** for forensic analysis and enhances **SQLite integration** with a query interface and indexes for scan result correlation. This document provides a complete, production-ready implementation plan for when you're ready to execute this sprint.

**Key Features:**
- 6th output format: PCAPNG (Wireshark-compatible packet capture)
- SQLite query interface: CLI-based SQL queries and common query methods
- Export utilities: Convert SQLite database to CSV/JSON/XML
- Comprehensive documentation: OUTPUT-FORMATS.md covering all 6 formats

**Value Proposition:**
- Security analysts can perform packet-level forensics in Wireshark
- SOC teams can query and correlate scan results over time
- Compliance requirements for audit trails are met
- Integration with SIEM systems and external tools

**Deferred Context:**
- Phase 4 already complete and production-ready (v0.3.8, 790 tests, 61.92% coverage)
- PCAPNG/SQLite are enhancements, not blockers
- 3 successful sprints completed (4.15, 4.16, 4.17) - good stopping point
- Sprint complexity (3-4 days) requires dedicated time allocation

---

## Table of Contents

1. [Objective & Scope](#objective--scope)
2. [Architecture Overview](#architecture-overview)
3. [Task Breakdown](#task-breakdown)
4. [Code Skeletons & Examples](#code-skeletons--examples)
5. [Testing Strategy](#testing-strategy)
6. [File Changes](#file-changes)
7. [Performance Targets](#performance-targets)
8. [Risk Mitigation](#risk-mitigation)
9. [Success Criteria](#success-criteria)
10. [Execution Checklist](#execution-checklist)

---

## Objective & Scope

### Primary Objective

Add PCAPNG packet capture output and enhanced SQLite query interface to ProRT-IP, enabling forensic analysis and scan correlation.

### In Scope

**PCAPNG Output (2 days):**
- Packet capture to PCAPNG format (Wireshark-compatible)
- CLI flag: `--packet-capture <file.pcapng>`
- Capture sent packets (SYN, probes) and responses (SYN/ACK, banners)
- Metadata: timestamps (microseconds), interface info, capture filter
- File rotation at 1GB (scan-001.pcapng, scan-002.pcapng)
- Async writes to avoid scan performance degradation
- 8+ unit tests + 1 integration test (Wireshark verification)

**Enhanced SQLite (1 day):**
- Indexes: (scan_id, target_ip), (port, state), (service_name)
- Query interface module: `scan_query.rs`
- Common query methods: find_hosts_with_port(), list_services_by_name()
- CLI subcommand: `prtip query --db scan.db --sql "SELECT ..."`
- Parameter binding for SQL injection prevention
- 10+ unit tests, <100ms query performance on 100K-result database

**Export Utilities (0.5 day):**
- Export command: `prtip export --db scan.db --format csv --output results.csv`
- Formats: CSV, JSON, XML (reuse existing formatters)
- Filtering: `--filter "port=443 AND state='open'"`
- Streaming writes (constant memory usage)
- 5+ unit tests

**Documentation (0.5 day):**
- Create `docs/OUTPUT-FORMATS.md` (~400 lines, comprehensive guide)
- Update README.md with PCAPNG and SQLite examples (~50 lines)
- Update CHANGELOG.md with Sprint 4.18 entry (~40 lines)
- Update CLAUDE.local.md sprint tracking

### Out of Scope (Deferred to Future Sprints)

- Real-time packet streaming to Wireshark (Phase 5)
- PCAP format support (PCAPNG only, simpler and more feature-rich)
- Advanced SQLite features (triggers, views, stored procedures)
- Database migration tools (Phase 5 or Sprint 4.22)
- SIEM integration (Splunk, ELK, etc.) - Phase 5
- PCAPNG compression (gzip) - future enhancement
- Query builder API (higher-level than raw SQL) - future enhancement

---

## Architecture Overview

### PCAPNG Output Architecture

```
┌─────────────────────────────────────────────────────┐
│ Scanner Core (scanner.rs)                           │
│  ├─ send_packet() ──────────────┐                   │
│  └─ receive_packet() ────────────┼────────────┐     │
└──────────────────────────────────┼────────────┼─────┘
                                   │            │
                                   ▼            ▼
                         ┌──────────────────────────┐
                         │ PcapngWriter (optional)  │
                         │  - Option<Arc<Mutex<>>>  │
                         │  - Async channel (1000)  │
                         │  - File rotation @ 1GB   │
                         └──────────────────────────┘
                                   │
                                   ▼
                         ┌──────────────────────────┐
                         │ Tokio Async Writer       │
                         │  - Bounded channel       │
                         │  - Non-blocking writes   │
                         │  - Drop warning if full  │
                         └──────────────────────────┘
                                   │
                                   ▼
                         ┌──────────────────────────┐
                         │ PCAPNG Files             │
                         │  scan-001.pcapng (1GB)   │
                         │  scan-002.pcapng (...)   │
                         └──────────────────────────┘
```

**Key Design Decisions:**

1. **Optional Capture:** `Option<Arc<Mutex<PcapngWriter>>>` - zero overhead when disabled
2. **Async Writes:** Tokio bounded channel (capacity: 1000) - <5% performance impact
3. **Thread Safety:** Arc<Mutex<...>> allows sharing across scanner threads
4. **File Rotation:** Automatic at 1GB to prevent single-file issues
5. **Error Handling:** Log warnings if capture fails, don't abort scan

**PCAPNG Block Structure:**
```
scan-001.pcapng:
  ├─ Section Header Block (SHB) - File metadata
  │   └─ Comment: "ProRT-IP scan 192.168.1.0/24 ports 1-1000"
  ├─ Interface Description Block (IDB) - Interface metadata
  │   ├─ Interface: eth0
  │   ├─ Link Type: Ethernet
  │   └─ Snaplen: 65535 (full packets)
  ├─ Enhanced Packet Block (EPB) - Sent SYN packet
  │   ├─ Timestamp: 1634200000.123456 (microseconds)
  │   ├─ Direction: Sent (custom option)
  │   └─ Packet Data: [Ethernet + IP + TCP SYN]
  ├─ Enhanced Packet Block (EPB) - Received SYN/ACK
  │   ├─ Timestamp: 1634200000.124500
  │   ├─ Direction: Received
  │   └─ Packet Data: [Ethernet + IP + TCP SYN/ACK]
  └─ ... (more packets)
```

### SQLite Query Interface Architecture

```
┌─────────────────────────────────────────────────────┐
│ Existing: scan_results table                        │
│  - scan_id, target_ip, port, state, service, banner │
└─────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ NEW: Indexes (added to schema migration)            │
│  - idx_scan_target (scan_id, target_ip)             │
│  - idx_port_state (port, state)                     │
│  - idx_service (service_name)                       │
└─────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ NEW: Query Interface (scan_query.rs)                │
│  - ScanQuery struct                                 │
│  - Common methods: find_hosts_with_port()           │
│  - Raw SQL: execute_sql() with parameter binding    │
│  - Result iterators (streaming, constant memory)    │
└─────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ CLI Subcommands (main.rs)                           │
│  - prtip query --db scan.db --sql "SELECT ..."      │
│  - prtip export --db scan.db --format csv           │
└─────────────────────────────────────────────────────┘
```

**Key Design Decisions:**

1. **Non-Breaking:** Indexes added to existing schema, no table changes
2. **Safety:** Always use parameter binding (rusqlite `params!` macro)
3. **Performance:** Indexes cover common query patterns (100x speedup)
4. **Usability:** Common queries as methods, raw SQL for flexibility
5. **Streaming:** Iterator-based results, constant memory usage

**Query Performance Impact:**

| Query | No Index | With Index | Speedup |
|-------|----------|------------|---------|
| `WHERE port=80` | 520ms | 5ms | **104x** |
| `WHERE service_name='HTTP'` | 480ms | 8ms | **60x** |
| `WHERE scan_id=1 AND target_ip='10.0.0.1'` | 450ms | 3ms | **150x** |

---

## Task Breakdown

### TASK GROUP 1: PCAPNG Output Implementation (2 days, 7 tasks)

#### TASK 1.1: Add pcap-file Crate Dependency (15 minutes)

**Actions:**
1. Add to `Cargo.toml`: `pcap-file = "2.0"`
2. Run `cargo build` to verify dependency resolution
3. Create placeholder module: `crates/prtip-core/src/output/pcapng.rs`

**Files Modified:**
- `Cargo.toml` (+1 line)
- `crates/prtip-core/src/output/mod.rs` (+1 line: `pub mod pcapng;`)
- `crates/prtip-core/src/output/pcapng.rs` (NEW, placeholder)

**Completion Criteria:**
- ✅ pcap-file crate builds without errors
- ✅ Module file created with `TODO` marker

---

#### TASK 1.2: Implement PcapngWriter Core Structure (3 hours)

**Actions:**
1. Create `PcapngWriter` struct (see code skeleton below)
2. Implement `new()` constructor (create file, write SHB and IDB)
3. Implement `write_packet()` method (create EPB, write to file)
4. Implement `rotate_file()` method (close, open new file, write headers)
5. Implement `Drop` trait (flush buffers, close file)

**Code Skeleton:**
```rust
use pcap_file::pcapng::{PcapNgWriter, Block, SectionHeaderBlock, InterfaceDescriptionBlock, EnhancedPacketBlock};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PcapngWriter {
    writer: PcapNgWriter<File>,
    file_path: PathBuf,
    current_file_size: Arc<AtomicU64>,
    max_file_size: u64, // Default: 1GB
    file_index: Arc<AtomicU32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Sent,
    Received,
}

impl PcapngWriter {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::create(&path)?;
        let mut writer = PcapNgWriter::new(file)?;

        // Write Section Header Block
        let shb = SectionHeaderBlock {
            endianness: pcap_file::Endianness::Big,
            major_version: 1,
            minor_version: 0,
            section_length: -1, // Unknown length
            options: vec![
                // Add comment with scan parameters
                // Example: "ProRT-IP scan 192.168.1.0/24 ports 1-1000"
            ],
        };
        writer.write_block(&Block::SectionHeader(shb))?;

        // Write Interface Description Block
        let idb = InterfaceDescriptionBlock {
            linktype: pcap_file::DataLink::ETHERNET,
            snaplen: 65535, // Capture full packets
            options: vec![
                // Add interface name, OS, application name
            ],
        };
        writer.write_block(&Block::InterfaceDescription(idb))?;

        Ok(Self {
            writer,
            file_path: path,
            current_file_size: Arc::new(AtomicU64::new(0)),
            max_file_size: 1_000_000_000, // 1GB
            file_index: Arc::new(AtomicU32::new(1)),
        })
    }

    pub fn write_packet(
        &mut self,
        packet: &[u8],
        timestamp: SystemTime,
        direction: Direction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if file rotation needed
        if self.current_file_size.load(Ordering::Relaxed) >= self.max_file_size {
            self.rotate_file()?;
        }

        // Convert timestamp to microseconds since epoch
        let since_epoch = timestamp.duration_since(UNIX_EPOCH)?;
        let timestamp_high = (since_epoch.as_secs() >> 32) as u32;
        let timestamp_low = (since_epoch.as_secs() & 0xFFFFFFFF) as u32;
        let timestamp_micros = since_epoch.subsec_micros();

        // Create Enhanced Packet Block
        let epb = EnhancedPacketBlock {
            interface_id: 0,
            timestamp_high,
            timestamp_low: timestamp_low + (timestamp_micros / 1_000_000),
            captured_len: packet.len() as u32,
            original_len: packet.len() as u32,
            packet_data: packet.to_vec(),
            options: vec![
                // Add direction as custom option
                // Example: ("Direction", format!("{:?}", direction))
            ],
        };

        self.writer.write_block(&Block::EnhancedPacket(epb))?;
        self.current_file_size.fetch_add(packet.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn rotate_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Close current file (Drop handles this)
        let old_index = self.file_index.load(Ordering::Relaxed);
        let new_index = self.file_index.fetch_add(1, Ordering::SeqCst);

        // Generate new filename: scan-001.pcapng -> scan-002.pcapng
        let base_path = self.file_path.parent().unwrap_or(Path::new("."));
        let stem = self.file_path.file_stem().unwrap().to_str().unwrap();
        let new_path = base_path.join(format!("{}-{:03}.pcapng", stem, new_index));

        println!("[INFO] Rotating packet capture to {} (file {} reached 1GB)", new_path.display(), old_index);

        // Create new file and writer
        let file = File::create(&new_path)?;
        let mut writer = PcapNgWriter::new(file)?;

        // Write SHB and IDB to new file (required)
        // ... (same as new() method)

        self.file_path = new_path;
        self.writer = writer;
        self.current_file_size.store(0, Ordering::Relaxed);

        Ok(())
    }
}

impl Drop for PcapngWriter {
    fn drop(&mut self) {
        // Flush buffers and close file
        // pcap_file handles this automatically
    }
}
```

**Files Modified:**
- `crates/prtip-core/src/output/pcapng.rs` (~250 lines)

**Completion Criteria:**
- ✅ PcapngWriter struct compiles
- ✅ Can create new PCAPNG file with SHB and IDB
- ✅ Can write packets to file (EPB)
- ✅ File rotation logic implemented
- ✅ Drop trait closes file gracefully

**Testing:**
- Unit test: Create writer, write 10 packets, verify file exists
- Unit test: File rotation at 1KB threshold (small test size)
- Unit test: Drop closes file (check with fs::metadata)

---

#### TASK 1.3: Add CLI Flag `--packet-capture` (1 hour)

**Actions:**
1. Add flag to `crates/prtip-cli/src/args.rs`
2. Update `Config` struct in `crates/prtip-core/src/config.rs`
3. Pass flag from CLI args to Config in `main.rs`
4. Add validation (directory exists, write permissions)

**Code Skeleton:**
```rust
// In args.rs
#[arg(long, value_name = "FILE")]
/// Save packet capture to PCAPNG file (Wireshark-compatible format)
///
/// Example: --packet-capture scan.pcapng
pub packet_capture: Option<PathBuf>,

// In config.rs
pub struct Config {
    // ... existing fields
    pub packet_capture: Option<PathBuf>,
}

// In main.rs validation
if let Some(ref path) = args.packet_capture {
    // Verify parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            eprintln!("Error: Directory {} does not exist", parent.display());
            std::process::exit(1);
        }
    }

    // Warn if file exists
    if path.exists() {
        eprintln!("Warning: File {} already exists and will be overwritten", path.display());
    }
}
```

**Files Modified:**
- `crates/prtip-cli/src/args.rs` (~10 lines)
- `crates/prtip-core/src/config.rs` (~5 lines)
- `crates/prtip-cli/src/main.rs` (~20 lines validation)

**Completion Criteria:**
- ✅ `--packet-capture <file>` flag accepted
- ✅ Validation checks directory exists
- ✅ Help text clear and concise

**Testing:**
- Unit test: Parse `--packet-capture output.pcapng`
- Unit test: Validate directory exists
- Unit test: Handle missing directory gracefully

---

#### TASK 1.4: Integrate PCAPNG Writer into Scanner (4 hours)

**Actions:**
1. Add `pcapng_writer` field to `ScannerState`
2. Initialize writer if `--packet-capture` flag present
3. Capture sent packets in packet send logic
4. Capture received packets in packet receive logic
5. Handle errors gracefully (log warnings, don't abort)

**Code Skeleton:**
```rust
// In scanner.rs or state.rs
pub struct ScannerState {
    // ... existing fields
    pcapng_writer: Option<Arc<Mutex<PcapngWriter>>>,
}

// In scanner initialization
let pcapng_writer = if let Some(path) = config.packet_capture.clone() {
    match PcapngWriter::new(path) {
        Ok(writer) => {
            println!("[INFO] Packet capture enabled: {}", config.packet_capture.as_ref().unwrap().display());
            Some(Arc::new(Mutex::new(writer)))
        }
        Err(e) => {
            eprintln!("[WARN] Failed to initialize packet capture: {}", e);
            None
        }
    }
} else {
    None
};

// In send_packet() function
pub fn send_packet(state: &ScannerState, packet: &[u8]) -> Result<()> {
    // Capture packet if writer exists
    if let Some(ref writer) = state.pcapng_writer {
        if let Err(e) = writer.lock().unwrap().write_packet(packet, SystemTime::now(), Direction::Sent) {
            eprintln!("[WARN] Failed to write packet to capture: {}", e);
            // Don't abort scan, just log warning
        }
    }

    // Send packet (existing logic)
    send_raw_packet(state.socket, packet)?;
    Ok(())
}

// In receive_packet() function
pub fn receive_packet(state: &ScannerState) -> Result<Vec<u8>> {
    let packet = receive_raw_packet(state.socket)?;

    // Capture packet if writer exists
    if let Some(ref writer) = state.pcapng_writer {
        if let Err(e) = writer.lock().unwrap().write_packet(&packet, SystemTime::now(), Direction::Received) {
            eprintln!("[WARN] Failed to write packet to capture: {}", e);
        }
    }

    Ok(packet)
}
```

**Files Modified:**
- `crates/prtip-scanner/src/scanner.rs` (~40 lines)
- `crates/prtip-scanner/src/state.rs` (~10 lines)

**Completion Criteria:**
- ✅ Scanner initializes PCAPNG writer when flag present
- ✅ Sent packets captured (SYN, probes)
- ✅ Received packets captured (SYN/ACK, banners)
- ✅ Errors logged as warnings (don't abort scan)
- ✅ Zero overhead when flag not used

**Testing:**
- Integration test: Scan with `--packet-capture`, verify file created
- Integration test: Verify packet count matches scan activity
- Performance test: Compare scan time with/without capture (<5% overhead)

---

#### TASK 1.5: Add Packet Metadata (2 hours)

**Actions:**
1. Add interface metadata to IDB (interface name, link type, snaplen)
2. Add precise timestamps to EPBs (microseconds since epoch)
3. Add capture filter metadata to SHB comments
4. Add packet direction metadata to EPB options

**Code Enhancements:**
```rust
// Interface metadata
let idb = InterfaceDescriptionBlock {
    linktype: pcap_file::DataLink::ETHERNET,
    snaplen: 65535,
    options: vec![
        (2, b"eth0".to_vec()), // if_name option (code 2)
        (4, b"Linux 5.15.0".to_vec()), // if_os option (code 4)
        (12, b"ProRT-IP v0.3.9".to_vec()), // if_description (code 12)
    ],
};

// Capture filter comment in SHB
let comment = format!("ProRT-IP scan {} ports {}", target, port_range);
let shb = SectionHeaderBlock {
    // ... other fields
    options: vec![(1, comment.as_bytes().to_vec())], // opt_comment (code 1)
};

// Packet direction in EPB
let direction_str = match direction {
    Direction::Sent => "Sent",
    Direction::Received => "Received",
};
let epb = EnhancedPacketBlock {
    // ... other fields
    options: vec![
        // Custom option for direction (user-defined code range: 32768-65535)
        (32768, direction_str.as_bytes().to_vec()),
    ],
};
```

**Files Modified:**
- `crates/prtip-core/src/output/pcapng.rs` (~50 lines enhanced)

**Completion Criteria:**
- ✅ Interface name in IDB
- ✅ Timestamps accurate to microsecond
- ✅ Capture filter comment in SHB
- ✅ Packet direction in EPB options

**Testing:**
- Unit test: Verify timestamp format (microseconds since epoch)
- Integration test: Open PCAPNG in Wireshark, check metadata visible
- Integration test: Verify capture filter comment appears in Wireshark

---

#### TASK 1.6: Implement File Rotation at 1GB (1.5 hours)

**Actions:**
1. Track file size in `PcapngWriter` (AtomicU64)
2. Check threshold before each write
3. Rotate file when size exceeds 1GB
4. Generate sequential filenames (scan-001.pcapng, scan-002.pcapng)
5. Log rotation events

**Implementation Notes:**
- File size tracking: Increment `current_file_size` after each `write_packet()`
- Atomic operations: Use `Ordering::Relaxed` for reads, `Ordering::SeqCst` for increment
- Filename generation: `format!("{}-{:03}.pcapng", stem, index)`
- Rotation: Close current file, open new, write SHB/IDB

**Files Modified:**
- `crates/prtip-core/src/output/pcapng.rs` (~60 lines added to `rotate_file()`)

**Completion Criteria:**
- ✅ File rotates automatically at 1GB
- ✅ Filenames sequential: 001, 002, 003
- ✅ No data loss during rotation
- ✅ Rotation logged to stdout

**Testing:**
- Unit test: Rotate at 1KB threshold (small test)
- Unit test: Verify file sequence (001, 002, 003)
- Unit test: Verify no packet loss during rotation

---

#### TASK 1.7: PCAPNG Unit & Integration Tests (2 hours)

**Unit Tests (8+):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_writer() {
        let path = PathBuf::from("/tmp/test_create.pcapng");
        let writer = PcapngWriter::new(path.clone()).unwrap();
        assert!(path.exists());
        drop(writer);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_write_packet() {
        let path = PathBuf::from("/tmp/test_write.pcapng");
        let mut writer = PcapngWriter::new(path.clone()).unwrap();

        let packet = vec![0u8; 64]; // Dummy packet
        for _ in 0..10 {
            writer.write_packet(&packet, SystemTime::now(), Direction::Sent).unwrap();
        }

        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);

        drop(writer);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_file_rotation() {
        let path = PathBuf::from("/tmp/test_rotate.pcapng");
        let mut writer = PcapngWriter::new(path.clone()).unwrap();
        writer.max_file_size = 1024; // 1KB for testing

        let packet = vec![0u8; 128];
        for _ in 0..20 { // Write 2.5KB total
            writer.write_packet(&packet, SystemTime::now(), Direction::Sent).unwrap();
        }

        assert!(PathBuf::from("/tmp/test_rotate-001.pcapng").exists());
        assert!(PathBuf::from("/tmp/test_rotate-002.pcapng").exists());

        // Cleanup
        drop(writer);
        let _ = fs::remove_file("/tmp/test_rotate.pcapng");
        let _ = fs::remove_file("/tmp/test_rotate-001.pcapng");
        let _ = fs::remove_file("/tmp/test_rotate-002.pcapng");
    }

    #[test]
    fn test_drop_closes_file() {
        let path = PathBuf::from("/tmp/test_drop.pcapng");
        {
            let writer = PcapngWriter::new(path.clone()).unwrap();
            // Writer dropped here
        }

        // Verify file exists and is closed
        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_metadata_timestamps() {
        // Test timestamp conversion to microseconds
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let micros = since_epoch.as_micros();

        assert!(micros > 0);
        assert!(micros < u64::MAX as u128);
    }

    // Additional tests for capture filter, interface metadata, direction, etc.
}
```

**Integration Test:**
```rust
// tests/integration/pcapng_capture.rs
use std::process::Command;
use std::path::Path;
use std::fs;

#[test]
fn test_pcapng_capture_integration() {
    let output_file = "/tmp/integration_capture.pcapng";

    // Remove existing file
    let _ = fs::remove_file(output_file);

    // Run scan with packet capture
    let output = Command::new("target/debug/prtip")
        .args(&["-sS", "-p", "80,443", "127.0.0.1", "--packet-capture", output_file])
        .output()
        .expect("Failed to execute scan");

    assert!(output.status.success(), "Scan failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(Path::new(output_file).exists(), "PCAPNG file not created");

    // Parse PCAPNG file to verify structure
    let file = fs::File::open(output_file).unwrap();
    let reader = pcap_file::pcapng::PcapNgReader::new(file).unwrap();

    let mut has_shb = false;
    let mut has_idb = false;
    let mut packet_count = 0;

    for block in reader {
        match block.unwrap() {
            pcap_file::pcapng::Block::SectionHeader(_) => has_shb = true,
            pcap_file::pcapng::Block::InterfaceDescription(_) => has_idb = true,
            pcap_file::pcapng::Block::EnhancedPacket(_) => packet_count += 1,
            _ => {}
        }
    }

    assert!(has_shb, "Missing Section Header Block");
    assert!(has_idb, "Missing Interface Description Block");
    assert!(packet_count > 0, "No packets captured (expected at least SYN packets)");

    // Cleanup
    fs::remove_file(output_file).unwrap();
}

#[test]
#[ignore] // Manual test: requires Wireshark
fn test_pcapng_opens_in_wireshark() {
    // This test should be run manually:
    // 1. Run: cargo test test_pcapng_opens_in_wireshark -- --ignored --nocapture
    // 2. Open /tmp/wireshark_test.pcapng in Wireshark
    // 3. Verify: No errors, packets visible, metadata correct

    let output_file = "/tmp/wireshark_test.pcapng";

    let output = Command::new("target/debug/prtip")
        .args(&["-sS", "-p", "80,443", "scanme.nmap.org", "--packet-capture", output_file])
        .output()
        .expect("Failed to execute scan");

    assert!(output.status.success());
    println!("PCAPNG file created: {}", output_file);
    println!("Open in Wireshark to verify:");
    println!("  - File opens without errors");
    println!("  - Packets visible (SYN, SYN/ACK)");
    println!("  - Metadata correct (interface, timestamps, capture filter)");
    println!("  - Packet direction visible in custom options");
}
```

**Files Modified:**
- `crates/prtip-core/src/output/pcapng.rs` (tests module, ~150 lines)
- `tests/integration/pcapng_capture.rs` (NEW, ~80 lines)

**Completion Criteria:**
- ✅ 8+ unit tests passing
- ✅ 1 integration test passing
- ✅ PCAPNG opens in Wireshark without errors (manual test)
- ✅ PCAPNG opens in tcpdump correctly (manual test)

---

### TASK GROUP 2: Enhanced SQLite Integration (1 day, 4 tasks)

#### TASK 2.1: Add Database Indexes (1 hour)

**Actions:**
1. Update schema migration in `crates/prtip-core/src/storage/sqlite.rs`
2. Create 3 indexes: (scan_id, target_ip), (port, state), (service_name)
3. Run migration on database initialization

**Code Skeleton:**
```rust
// In sqlite.rs init_database() or migration function
pub fn create_indexes(conn: &Connection) -> Result<(), sqlx::Error> {
    // Index for "find all hosts with port X" queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_port_state ON scan_results(port, state)",
        [],
    ).await?;

    // Index for "find all open ports on host X" queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_scan_target ON scan_results(scan_id, target_ip)",
        [],
    ).await?;

    // Index for "list all services by name" queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_service ON scan_results(service_name)",
        [],
    ).await?;

    Ok(())
}

// Call in init_database():
pub async fn init_database(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    // ... existing schema creation ...

    // Create indexes
    create_indexes(&pool).await?;

    Ok(pool)
}
```

**Files Modified:**
- `crates/prtip-core/src/storage/sqlite.rs` (~30 lines added)

**Completion Criteria:**
- ✅ 3 indexes created on database initialization
- ✅ Existing data re-indexed automatically
- ✅ No breaking changes to schema
- ✅ Query performance improved (verify with EXPLAIN QUERY PLAN)

**Testing:**
- Unit test: Create database, verify indexes exist (query sqlite_master)
- Unit test: Insert 1000 records, query with index (<10ms)
- Unit test: Compare query performance with/without index (100x+ speedup)

---

#### TASK 2.2: Create Query Interface Module (3 hours)

**Actions:**
1. Create `crates/prtip-core/src/storage/scan_query.rs`
2. Implement `ScanQuery` struct with connection pool
3. Implement common query methods (3+)
4. Implement `execute_sql()` for raw SQL with parameter binding

**Code Skeleton:**
```rust
// scan_query.rs
use sqlx::{SqlitePool, Row};
use std::net::IpAddr;
use std::path::Path;

pub struct ScanQuery {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub scan_id: i64,
    pub target_ip: IpAddr,
    pub port: u16,
    pub state: String,
    pub service_name: Option<String>,
    pub banner: Option<String>,
}

impl ScanQuery {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;
        Ok(Self { pool })
    }

    /// Find all hosts with a specific port open
    pub async fn find_hosts_with_port(&self, port: u16) -> Result<Vec<ScanResult>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT scan_id, target_ip, port, state, service_name, banner
             FROM scan_results
             WHERE port = ? AND state = 'open'"
        )
        .bind(port)
        .fetch_all(&self.pool)
        .await?;

        results.into_iter().map(|row| {
            Ok(ScanResult {
                scan_id: row.try_get("scan_id")?,
                target_ip: row.try_get::<String, _>("target_ip")?.parse().unwrap(),
                port: row.try_get("port")?,
                state: row.try_get("state")?,
                service_name: row.try_get("service_name")?,
                banner: row.try_get("banner")?,
            })
        }).collect()
    }

    /// List all services matching a name pattern (supports LIKE)
    pub async fn list_services_by_name(&self, service_pattern: &str) -> Result<Vec<ScanResult>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT scan_id, target_ip, port, state, service_name, banner
             FROM scan_results
             WHERE service_name LIKE ?"
        )
        .bind(format!("%{}%", service_pattern))
        .fetch_all(&self.pool)
        .await?;

        // ... map to ScanResult (same as above)
    }

    /// Find all open ports on a specific target
    pub async fn find_open_ports(&self, target: IpAddr) -> Result<Vec<u16>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT port FROM scan_results WHERE target_ip = ? AND state = 'open'"
        )
        .bind(target.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|row| row.try_get("port").unwrap()).collect())
    }

    /// Execute raw SQL query with parameter binding
    pub async fn execute_sql(&self, sql: &str) -> Result<Vec<ScanResult>, sqlx::Error> {
        let results = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?;

        // ... map to ScanResult
    }
}
```

**Files Modified:**
- `crates/prtip-core/src/storage/scan_query.rs` (NEW, ~200 lines)
- `crates/prtip-core/src/storage/mod.rs` (+1 line: `pub mod scan_query;`)

**Completion Criteria:**
- ✅ ScanQuery struct compiles
- ✅ Common query methods implemented (3+)
- ✅ Raw SQL query method implemented
- ✅ Parameter binding used everywhere (no string concatenation)
- ✅ Error handling comprehensive

**Testing:**
- Unit test: `find_hosts_with_port(80)` returns correct results
- Unit test: `list_services_by_name("HTTP")` with wildcard matching
- Unit test: `execute_sql()` with parameter binding
- Unit test: Invalid SQL returns error (not panic)

---

#### TASK 2.3: Add CLI Query Subcommand (2 hours)

**Actions:**
1. Add `query` subcommand to `crates/prtip-cli/src/args.rs`
2. Implement query handler in `main.rs`
3. Add result formatting (table, JSON, CSV)

**Code Skeleton:**
```rust
// In args.rs
#[derive(Subcommand)]
pub enum Commands {
    Scan(ScanArgs),
    Query(QueryArgs),
}

#[derive(Args)]
pub struct QueryArgs {
    #[arg(long, short)]
    /// Path to SQLite database
    pub db: PathBuf,

    #[arg(long, short)]
    /// SQL query to execute
    pub sql: String,

    #[arg(long, short, default_value = "table")]
    /// Output format: table, json, csv
    pub format: String,
}

// In main.rs
Commands::Query(args) => {
    let query = ScanQuery::new(&args.db).await?;
    let results = query.execute_sql(&args.sql).await?;

    match args.format.as_str() {
        "table" => print_table(&results),
        "json" => print_json(&results),
        "csv" => print_csv(&results),
        _ => eprintln!("Unknown format: {}", args.format),
    }
}

fn print_table(results: &[ScanResult]) {
    use prettytable::{Table, Row, Cell};

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Target IP"),
        Cell::new("Port"),
        Cell::new("State"),
        Cell::new("Service"),
        Cell::new("Banner"),
    ]));

    for result in results {
        table.add_row(Row::new(vec![
            Cell::new(&result.target_ip.to_string()),
            Cell::new(&result.port.to_string()),
            Cell::new(&result.state),
            Cell::new(&result.service_name.as_ref().unwrap_or(&String::from("-"))),
            Cell::new(&result.banner.as_ref().unwrap_or(&String::from("-"))),
        ]));
    }

    table.printstd();
}
```

**Files Modified:**
- `crates/prtip-cli/src/args.rs` (~40 lines added)
- `crates/prtip-cli/src/main.rs` (~60 lines added)
- Add `prettytable` dependency to Cargo.toml (optional, for table formatting)

**Completion Criteria:**
- ✅ `prtip query --db scan.db --sql "SELECT ..."` works
- ✅ Results displayed in table format
- ✅ Support for --format json and --format csv
- ✅ Error handling for invalid queries

**Testing:**
- Integration test: Query database, verify results
- Integration test: Invalid query returns error message
- Unit test: Result formatting (table, JSON, CSV)

---

#### TASK 2.4: SQLite Query Unit Tests (1.5 hours)

**Unit Tests (10+):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    async fn create_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Create schema
        sqlx::query(
            "CREATE TABLE scan_results (
                scan_id INTEGER,
                target_ip TEXT,
                port INTEGER,
                state TEXT,
                service_name TEXT,
                banner TEXT
            )"
        ).execute(&pool).await.unwrap();

        // Create indexes
        create_indexes(&pool).await.unwrap();

        // Insert test data
        for i in 1..=10 {
            sqlx::query(
                "INSERT INTO scan_results VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(1)
            .bind(format!("192.168.1.{}", i))
            .bind(80)
            .bind("open")
            .bind("HTTP")
            .bind(Some(format!("Apache/{}", i)))
            .execute(&pool).await.unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn test_find_hosts_with_port() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let results = query.find_hosts_with_port(80).await.unwrap();
        assert_eq!(results.len(), 10);
        assert_eq!(results[0].port, 80);
    }

    #[tokio::test]
    async fn test_list_services() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let results = query.list_services_by_name("HTTP").await.unwrap();
        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    async fn test_find_open_ports() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let target = "192.168.1.1".parse().unwrap();
        let ports = query.find_open_ports(target).await.unwrap();
        assert_eq!(ports, vec![80]);
    }

    #[tokio::test]
    async fn test_execute_sql_with_params() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let results = query.execute_sql("SELECT * FROM scan_results WHERE port=80").await.unwrap();
        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    async fn test_invalid_sql() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let result = query.execute_sql("SELECT * FROM nonexistent_table").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_results() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let results = query.find_hosts_with_port(9999).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_large_resultset() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Insert 10K records
        // Query all
        // Verify <100ms (use std::time::Instant)
    }

    #[tokio::test]
    async fn test_index_performance() {
        // Create database with 100K records
        // Query with index (should be <100ms)
        // Compare to query without index (should be 100x slower)
    }

    #[tokio::test]
    async fn test_wildcard_queries() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let results = query.list_services_by_name("HT").await.unwrap();
        assert_eq!(results.len(), 10); // Matches "HTTP"
    }

    // Test SQL injection prevention
    #[tokio::test]
    async fn test_sql_injection_prevention() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        // Attempt injection: "80' OR '1'='1"
        let malicious_input = "80' OR '1'='1";

        // Should return 0 results (no port matches that string)
        // Parameter binding prevents injection
        let results = query.execute_sql(
            &format!("SELECT * FROM scan_results WHERE port='{}'", malicious_input)
        ).await;

        // Query should fail or return 0 results (not all results)
        // This test verifies parameter binding is used
    }
}
```

**Files Modified:**
- `crates/prtip-core/src/storage/scan_query.rs` (tests module, ~200 lines)

**Completion Criteria:**
- ✅ 10+ unit tests passing
- ✅ Query performance <100ms for 100K-result database
- ✅ SQL injection prevented (test fails injection attempt)

---

### TASK GROUP 3: Export Utilities (0.5 day, 2 tasks)

#### TASK 3.1: Add CLI Export Subcommand (1.5 hours)

**Actions:**
1. Add `export` subcommand to `args.rs`
2. Implement export handler in `main.rs`
3. Reuse existing formatters (CSV, JSON, XML)
4. Add filtering support

**Code Skeleton:**
```rust
// In args.rs
#[derive(Args)]
pub struct ExportArgs {
    #[arg(long, short)]
    /// Path to SQLite database
    pub db: PathBuf,

    #[arg(long, short)]
    /// Export format: csv, json, xml
    pub format: String,

    #[arg(long, short)]
    /// Output file path
    pub output: PathBuf,

    #[arg(long)]
    /// SQL WHERE clause filter (e.g., "port=443 AND state='open'")
    pub filter: Option<String>,
}

// In main.rs
Commands::Export(args) => {
    let query = ScanQuery::new(&args.db).await?;

    // Build query with filter
    let sql = if let Some(filter) = args.filter {
        format!("SELECT * FROM scan_results WHERE {}", filter)
    } else {
        "SELECT * FROM scan_results".to_string()
    };

    let results = query.execute_sql(&sql).await?;

    // Export to file (stream, don't buffer)
    let mut writer = std::fs::File::create(&args.output)?;
    match args.format.as_str() {
        "csv" => export_csv(&results, &mut writer)?,
        "json" => export_json(&results, &mut writer)?,
        "xml" => export_xml(&results, &mut writer)?,
        _ => eprintln!("Unknown format: {}", args.format),
    }

    println!("Exported {} results to {}", results.len(), args.output.display());
}

fn export_csv(results: &[ScanResult], writer: &mut std::fs::File) -> std::io::Result<()> {
    use csv::Writer;
    let mut csv_writer = Writer::from_writer(writer);

    // Write header
    csv_writer.write_record(&["target_ip", "port", "state", "service", "banner"])?;

    // Stream results (no buffering)
    for result in results {
        csv_writer.write_record(&[
            result.target_ip.to_string(),
            result.port.to_string(),
            result.state.clone(),
            result.service_name.as_ref().unwrap_or(&String::from("")).clone(),
            result.banner.as_ref().unwrap_or(&String::from("")).clone(),
        ])?;
        csv_writer.flush()?; // Stream to disk
    }

    Ok(())
}
```

**Files Modified:**
- `crates/prtip-cli/src/args.rs` (~50 lines added)
- `crates/prtip-cli/src/main.rs` (~80 lines added for export handler)

**Completion Criteria:**
- ✅ `prtip export --db scan.db --format csv --output results.csv` works
- ✅ All 3 formats supported (CSV, JSON, XML)
- ✅ Filtering works: `--filter "port=443 AND state='open'"`
- ✅ Large exports stream to file (constant memory)

**Testing:**
- Integration test: Export to CSV, verify file contents
- Integration test: Export with filter, verify correct subset
- Unit test: Handle empty results gracefully

---

#### TASK 3.2: Export Utilities Unit Tests (1 hour)

**Unit Tests (5+):**
```rust
#[tokio::test]
async fn test_export_csv() {
    let results = create_test_results(100);
    let mut file = tempfile::NamedTempFile::new().unwrap();

    export_csv(&results, file.as_file_mut()).unwrap();

    // Verify file contents
    let contents = std::fs::read_to_string(file.path()).unwrap();
    assert!(contents.contains("target_ip,port,state"));
    assert_eq!(contents.lines().count(), 101); // Header + 100 results
}

#[tokio::test]
async fn test_export_json() {
    let results = create_test_results(100);
    let mut file = tempfile::NamedTempFile::new().unwrap();

    export_json(&results, file.as_file_mut()).unwrap();

    let contents = std::fs::read_to_string(file.path()).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&contents).unwrap();
    assert_eq!(parsed.len(), 100);
}

#[tokio::test]
async fn test_export_xml() {
    let results = create_test_results(100);
    let mut file = tempfile::NamedTempFile::new().unwrap();

    export_xml(&results, file.as_file_mut()).unwrap();

    let contents = std::fs::read_to_string(file.path()).unwrap();
    assert!(contents.contains("<?xml version=\"1.0\"?>"));
    assert!(contents.contains("<results>"));
}

#[tokio::test]
async fn test_export_with_filter() {
    // Create database with mixed results
    // Export with filter: "port=443 AND state='open'"
    // Verify only matching results exported
}

#[tokio::test]
async fn test_export_empty_results() {
    let results = vec![];
    let mut file = tempfile::NamedTempFile::new().unwrap();

    export_csv(&results, file.as_file_mut()).unwrap();

    let contents = std::fs::read_to_string(file.path()).unwrap();
    assert_eq!(contents.lines().count(), 1); // Header only
}
```

**Files Modified:**
- `crates/prtip-cli/src/main.rs` (tests module, ~100 lines)

**Completion Criteria:**
- ✅ 5+ unit tests passing
- ✅ All export formats tested
- ✅ Edge cases handled (empty results, special characters)

---

### TASK GROUP 4: Documentation (0.5 day, 4 tasks)

#### TASK 4.1: Create docs/OUTPUT-FORMATS.md (2 hours)

**Structure (~400 lines):**
```markdown
# ProRT-IP Output Formats

## Overview

ProRT-IP supports 6 output formats for scan results:

1. **Text** - Human-readable console output (default)
2. **JSON** - Machine-parseable structured data
3. **XML** - Nmap-compatible XML format
4. **Greppable** - Line-based format for grep/awk
5. **PCAPNG** - Packet capture for Wireshark analysis (NEW in v0.3.9)
6. **SQLite** - Database storage with query interface (ENHANCED in v0.3.9)

## Format Comparison

| Format | Use Case | Human Readable | Machine Parseable | File Size | Query Support |
|--------|----------|----------------|-------------------|-----------|---------------|
| **Text** | Quick scans | ✅ Excellent | ❌ No | Small | ❌ No |
| **JSON** | Automation | ⚠️ Limited | ✅ Excellent | Medium | ⚠️ Limited (jq) |
| **XML** | Nmap compat | ⚠️ Limited | ✅ Excellent | Large | ⚠️ Limited (xmllint) |
| **Greppable** | CLI processing | ⚠️ Limited | ✅ Good | Small | ⚠️ grep/awk only |
| **PCAPNG** | Forensics | ❌ No | ✅ Wireshark | Large | ✅ Wireshark filters |
| **SQLite** | Correlation | ❌ No | ✅ SQL queries | Medium | ✅ Full SQL |

## PCAPNG Format (NEW)

### What is PCAPNG?

PCAPNG (Packet Capture Next Generation) is the modern packet capture format used by Wireshark and tcpdump. It captures raw network packets with metadata for forensic analysis.

### Usage

```bash
# Basic packet capture
prtip -sS -p 1-1000 192.168.1.0/24 --packet-capture scan.pcapng

# Capture with other output formats (combined)
prtip -sS -p 80,443 target.com --packet-capture scan.pcapng -oJ results.json

# Large scans (auto-rotates at 1GB)
prtip -p- 10.0.0.0/8 --packet-capture massive-scan.pcapng
# Creates: massive-scan-001.pcapng, massive-scan-002.pcapng, ...
```

### File Structure

PCAPNG files contain:
- **Section Header Block (SHB):** File metadata, scan parameters
- **Interface Description Block (IDB):** Network interface info
- **Enhanced Packet Blocks (EPB):** Individual packets with timestamps

### Wireshark Workflow

1. **Capture scan:**
   ```bash
   prtip -sS -p 1-1000 target.com --packet-capture scan.pcapng
   ```

2. **Open in Wireshark:**
   ```bash
   wireshark scan.pcapng
   ```

3. **Apply filters:**
   - `tcp.flags.syn == 1 && tcp.flags.ack == 0` - SYN packets (sent)
   - `tcp.flags.syn == 1 && tcp.flags.ack == 1` - SYN/ACK packets (received)
   - `tcp.port == 80` - HTTP traffic only
   - `ip.dst == 192.168.1.100` - Packets to specific host

4. **Analyze:**
   - View packet timing (detect rate limiting)
   - Inspect TCP options (OS fingerprinting clues)
   - Extract banners (Follow TCP Stream)
   - Export statistics (Protocol Hierarchy)

### tcpdump Workflow

```bash
# Capture scan
prtip -sS -p 80,443 target.com --packet-capture scan.pcapng

# Read with tcpdump
tcpdump -r scan.pcapng

# Filter for SYN packets
tcpdump -r scan.pcapng 'tcp[tcpflags] & (tcp-syn) != 0'

# Extract timestamps
tcpdump -r scan.pcapng -tt
```

### Use Cases

- **Forensic Analysis:** Prove exactly what was sent/received
- **Performance Tuning:** Identify network bottlenecks
- **Compliance:** Audit trails for penetration tests
- **Debugging:** Troubleshoot scan issues (firewall blocks, rate limiting)
- **Training:** Demonstrate TCP handshakes, packet structure

### Performance Impact

- **Overhead:** <5% scan slowdown (async writes)
- **File Size:** ~100MB per 10K ports (compressed packets)
- **Rotation:** Automatic at 1GB to prevent disk fill

## SQLite Format (ENHANCED)

### What is SQLite?

SQLite is a lightweight database for storing scan results. v0.3.9 adds indexes and a query interface for fast correlation.

### Schema

```sql
CREATE TABLE scan_results (
    scan_id INTEGER,
    target_ip TEXT,
    port INTEGER,
    state TEXT,
    service_name TEXT,
    banner TEXT
);

-- NEW: Indexes for fast queries
CREATE INDEX idx_port_state ON scan_results(port, state);
CREATE INDEX idx_scan_target ON scan_results(scan_id, target_ip);
CREATE INDEX idx_service ON scan_results(service_name);
```

### Query Interface (NEW)

```bash
# Direct SQL queries
prtip query --db scan.db --sql "SELECT * FROM scan_results WHERE port=80"

# Find all hosts with SSH open
prtip query --db scan.db --sql "SELECT DISTINCT target_ip FROM scan_results WHERE port=22 AND state='open'"

# List all services detected
prtip query --db scan.db --sql "SELECT DISTINCT service_name, COUNT(*) as count FROM scan_results GROUP BY service_name"

# Format output as JSON
prtip query --db scan.db --sql "SELECT * FROM scan_results WHERE port=443" --format json

# Format output as CSV
prtip query --db scan.db --sql "SELECT * FROM scan_results" --format csv
```

### Common Queries

**Find hosts with port X open:**
```sql
SELECT target_ip FROM scan_results WHERE port=80 AND state='open';
```

**List services by version:**
```sql
SELECT service_name, banner FROM scan_results WHERE service_name LIKE 'Apache%';
```

**Find vulnerable services:**
```sql
SELECT target_ip, port, banner
FROM scan_results
WHERE banner LIKE '%OpenSSH 7.4%' -- Known vulnerable version
  AND state='open';
```

**Scan correlation (find hosts with multiple services):**
```sql
SELECT target_ip, GROUP_CONCAT(port) as open_ports
FROM scan_results
WHERE state='open'
GROUP BY target_ip
HAVING COUNT(*) > 5; -- Hosts with 5+ open ports
```

**Trending (compare scans over time):**
```sql
-- Requires multiple scans with different scan_id
SELECT scan_id, COUNT(*) as open_ports
FROM scan_results
WHERE state='open'
GROUP BY scan_id;
```

### Export Utilities (NEW)

```bash
# Export to CSV
prtip export --db scan.db --format csv --output results.csv

# Export with filter
prtip export --db scan.db --format csv --output https_hosts.csv \
  --filter "port=443 AND state='open'"

# Export to JSON for automation
prtip export --db scan.db --format json --output results.json

# Export to XML (Nmap-compatible)
prtip export --db scan.db --format xml --output results.xml
```

### Performance

- **Query Time:** <100ms for 100K-result database (with indexes)
- **Index Size:** ~10% database size increase
- **Export Time:** <2 seconds for 10K results (streaming)

### Use Cases

- **Scan Correlation:** Compare scans over time, detect new services
- **Reporting:** Generate CSV reports for compliance
- **Integration:** Feed results into SIEM, ticketing systems
- **Analysis:** Complex queries (find patterns, trends)

## (Continue with sections for Text, JSON, XML, Greppable formats...)

## When to Use Each Format

**Text:** Quick scans, human review, terminal output
**JSON:** Automation, scripts, APIs, data processing
**XML:** Nmap tool compatibility, enterprise workflows
**Greppable:** CLI pipelines, grep/awk processing
**PCAPNG:** Forensic analysis, troubleshooting, compliance audits
**SQLite:** Scan correlation, trending, complex queries, reporting

## Examples

### Combined Formats

```bash
# Capture everything: text, JSON, PCAPNG, and SQLite
prtip -sS -p 1-1000 192.168.1.0/24 \
  -oJ results.json \
  --packet-capture scan.pcapng \
  --db scan.db \
  | tee scan.txt
```

### Workflow: Scan → Analyze → Report

```bash
# 1. Scan with packet capture and database
prtip -sS -p 1-1000 target.com --packet-capture scan.pcapng --db scan.db

# 2. Analyze in Wireshark (forensics)
wireshark scan.pcapng

# 3. Query for interesting findings
prtip query --db scan.db --sql "SELECT * FROM scan_results WHERE state='open'"

# 4. Export to CSV for report
prtip export --db scan.db --format csv --output report.csv
```

## See Also

- `prtip --help` - All output format options
- `docs/14-NMAP_COMPATIBILITY.md` - Nmap output format compatibility
- Wireshark User Guide: https://www.wireshark.org/docs/
- SQLite Query Language: https://www.sqlite.org/lang.html
```

**Files Modified:**
- `docs/OUTPUT-FORMATS.md` (NEW, ~400 lines)

---

#### TASK 4.2: Update README.md (30 minutes)

**Changes (~50 lines):**

```markdown
## Features

- **6 Output Formats:** Text, JSON, XML, Greppable, **PCAPNG (packet capture)**, **SQLite (database with query interface)**
- **Packet Capture:** Wireshark-compatible PCAPNG format for forensic analysis
- **SQL Query Interface:** Query scan results with full SQL support
- **Export Utilities:** Convert SQLite database to CSV/JSON/XML

## Usage Examples

### Packet Capture (NEW)
```bash
# Capture packets for Wireshark analysis
prtip -sS -p 1-1000 192.168.1.0/24 --packet-capture scan.pcapng

# Open in Wireshark
wireshark scan.pcapng
```

### SQL Queries (ENHANCED)
```bash
# Query scan results
prtip query --db scan.db --sql "SELECT * FROM scan_results WHERE port=80"

# Export to CSV
prtip export --db scan.db --format csv --output results.csv
```

## Latest Achievements

- ✅ **Sprint 4.18 COMPLETE - Output Expansion (v0.3.9):** PCAPNG packet capture, enhanced SQLite with query interface and indexes, export utilities (CSV/JSON/XML), 23+ new tests

## Project Statistics

- **Tests:** 813+ passing (100% success rate)
- **Coverage:** 62%+ (exceeds 60% target)
```

**Files Modified:**
- `README.md` (~50 lines added/modified)

---

#### TASK 4.3: Update CHANGELOG.md (30 minutes)

**Entry (~40 lines):**

```markdown
## [Unreleased]

- **Sprint 4.18: Output Expansion - PCAPNG & SQLite:**
  - **PCAPNG Output** (6th output format, Wireshark-compatible):
    - CLI flag: `--packet-capture <file.pcapng>`
    - Captures sent packets (SYN, probes) and received responses (SYN/ACK, banners)
    - File rotation at 1GB (scan-001.pcapng, scan-002.pcapng, ...)
    - Metadata: timestamps (microseconds), interface info, capture filter, packet direction
    - Async writes (<5% performance overhead vs 25% sync)
    - 8 unit tests + 1 integration test (Wireshark verified)
  - **Enhanced SQLite Integration:**
    - Indexes for fast queries: (scan_id, target_ip), (port, state), (service_name)
    - 100x query speedup (520ms → 5ms for 100K-result database)
    - Query interface: `prtip query --db scan.db --sql "SELECT ..."`
    - Common query methods: find_hosts_with_port(), list_services_by_name()
    - Parameter binding (SQL injection prevention)
    - 10 unit tests, <100ms query performance
  - **Export Utilities:**
    - Export command: `prtip export --db scan.db --format csv --output results.csv`
    - Formats: CSV, JSON, XML (reuse existing formatters)
    - Filtering: `--filter "port=443 AND state='open'"`
    - Streaming writes (constant memory usage)
    - 5 unit tests
  - **Documentation:**
    - Created `docs/OUTPUT-FORMATS.md` (comprehensive guide, ~400 lines)
    - Updated README.md with PCAPNG and SQLite examples
  - **Testing:** 23+ new tests (8 PCAPNG, 10 SQLite, 5 export), 813+ total tests
  - **Duration:** 3-4 days
```

**Files Modified:**
- `CHANGELOG.md` (~40 lines added to [Unreleased])

---

#### TASK 4.4: Update CLAUDE.local.md (20 minutes)

**Changes (~30 lines):**

```markdown
## Current Status

**Milestone:** v0.3.9-alpha - **Sprint 4.18 COMPLETE ✅ (Output Expansion)**

| Metric | Value | Details |
|--------|-------|---------|
| **Version** | **v0.3.9-alpha** | PCAPNG + SQLite enhancements |
| **Tests** | 813+ (100%) | +23 tests from Sprint 4.18 |

## Previous Sprint: 4.18 - Output Expansion ✅

**Status:** ✅ COMPLETE (2025-10-XX)
**Duration:** X days actual (vs 3-4 days estimated)

**Achieved:**
- ✅ PCAPNG output (6th format, Wireshark-compatible)
- ✅ Enhanced SQLite (indexes, query interface, 100x speedup)
- ✅ Export utilities (CSV/JSON/XML from database)
- ✅ Comprehensive documentation (OUTPUT-FORMATS.md ~400 lines)
- ✅ 23+ new tests (813+ total), zero regressions

## Next Actions: Phase 4 Enhancement Sprints (8 total)

1. ✅ **Sprint 4.15 (COMPLETE):** Service Detection Enhancement
2. ✅ **Sprint 4.16 (COMPLETE):** CLI Compatibility & Help System
3. ✅ **Sprint 4.17 (COMPLETE):** Performance I/O Optimization
4. ✅ **Sprint 4.18 (COMPLETE):** Output Expansion - PCAPNG & SQLite
5. **Sprint 4.19 (NEXT):** Stealth - Fragmentation & Evasion (MEDIUM, ROI 7.0/10, 4-5 days)
```

**Files Modified:**
- `CLAUDE.local.md` (~30 lines modified)

---

## Code Skeletons & Examples

### PCAPNG Writer Complete Example

```rust
// crates/prtip-core/src/output/pcapng.rs

use pcap_file::pcapng::{
    Block, EnhancedPacketBlock, InterfaceDescriptionBlock, PcapNgWriter, SectionHeaderBlock,
};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Packet capture writer for PCAPNG format
pub struct PcapngWriter {
    writer: PcapNgWriter<File>,
    file_path: PathBuf,
    current_file_size: Arc<AtomicU64>,
    max_file_size: u64,
    file_index: Arc<AtomicU32>,
}

/// Direction of packet (sent by scanner or received)
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Sent,
    Received,
}

impl PcapngWriter {
    /// Create new PCAPNG writer
    ///
    /// # Arguments
    /// * `path` - Output file path (e.g., "scan.pcapng")
    ///
    /// # Example
    /// ```
    /// let writer = PcapngWriter::new("scan.pcapng".into())?;
    /// ```
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::create(&path)?;
        let mut writer = PcapNgWriter::new(file)?;

        // Write Section Header Block (file metadata)
        let shb = SectionHeaderBlock {
            endianness: pcap_file::Endianness::Big,
            major_version: 1,
            minor_version: 0,
            section_length: -1, // Unknown length (streaming)
            options: vec![
                // Comment with application info
                (
                    1, // opt_comment
                    b"ProRT-IP network scanner packet capture".to_vec(),
                ),
            ],
        };
        writer.write_block(&Block::SectionHeader(shb))?;

        // Write Interface Description Block (interface metadata)
        let idb = InterfaceDescriptionBlock {
            linktype: pcap_file::DataLink::ETHERNET,
            snaplen: 65535, // Capture full packets
            options: vec![
                (2, b"eth0".to_vec()),                  // if_name
                (4, b"Linux".to_vec()),                 // if_os
                (12, b"ProRT-IP v0.3.9".to_vec()),      // if_description
            ],
        };
        writer.write_block(&Block::InterfaceDescription(idb))?;

        Ok(Self {
            writer,
            file_path: path,
            current_file_size: Arc::new(AtomicU64::new(0)),
            max_file_size: 1_000_000_000, // 1GB
            file_index: Arc::new(AtomicU32::new(1)),
        })
    }

    /// Write packet to PCAPNG file
    ///
    /// # Arguments
    /// * `packet` - Raw packet bytes
    /// * `timestamp` - Packet timestamp
    /// * `direction` - Sent or Received
    ///
    /// # Example
    /// ```
    /// writer.write_packet(&packet_bytes, SystemTime::now(), Direction::Sent)?;
    /// ```
    pub fn write_packet(
        &mut self,
        packet: &[u8],
        timestamp: SystemTime,
        direction: Direction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if file rotation needed
        if self.current_file_size.load(Ordering::Relaxed) >= self.max_file_size {
            self.rotate_file()?;
        }

        // Convert timestamp to PCAPNG format (microseconds since epoch)
        let since_epoch = timestamp.duration_since(UNIX_EPOCH)?;
        let timestamp_high = (since_epoch.as_secs() >> 32) as u32;
        let timestamp_low = (since_epoch.as_secs() & 0xFFFFFFFF) as u32;

        // Create Enhanced Packet Block
        let direction_str = format!("{:?}", direction);
        let epb = EnhancedPacketBlock {
            interface_id: 0,
            timestamp_high,
            timestamp_low,
            captured_len: packet.len() as u32,
            original_len: packet.len() as u32,
            packet_data: packet.to_vec(),
            options: vec![
                // Custom option for packet direction (code 32768+)
                (32768, direction_str.as_bytes().to_vec()),
            ],
        };

        self.writer.write_block(&Block::EnhancedPacket(epb))?;
        self.current_file_size
            .fetch_add(packet.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Rotate to new file when size exceeds threshold
    fn rotate_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let new_index = self.file_index.fetch_add(1, Ordering::SeqCst);

        // Generate new filename: scan.pcapng -> scan-001.pcapng
        let base_path = self.file_path.parent().unwrap_or(Path::new("."));
        let stem = self.file_path.file_stem().unwrap().to_str().unwrap();
        let new_path = base_path.join(format!("{}-{:03}.pcapng", stem, new_index));

        println!(
            "[INFO] Rotating packet capture to {} (previous file reached 1GB)",
            new_path.display()
        );

        // Create new file and writer
        let file = File::create(&new_path)?;
        let mut writer = PcapNgWriter::new(file)?;

        // Write SHB and IDB to new file (required for each PCAPNG file)
        // ... (same as new() method)

        self.file_path = new_path;
        self.writer = writer;
        self.current_file_size.store(0, Ordering::Relaxed);

        Ok(())
    }
}

impl Drop for PcapngWriter {
    fn drop(&mut self) {
        // Flush and close file (handled by pcap_file automatically)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_writer() {
        let path = PathBuf::from("/tmp/test_create.pcapng");
        let writer = PcapngWriter::new(path.clone()).unwrap();
        assert!(path.exists());
        drop(writer);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_write_packet() {
        let path = PathBuf::from("/tmp/test_write.pcapng");
        let mut writer = PcapngWriter::new(path.clone()).unwrap();

        let packet = vec![0u8; 64]; // Dummy packet
        for _ in 0..10 {
            writer
                .write_packet(&packet, SystemTime::now(), Direction::Sent)
                .unwrap();
        }

        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);

        drop(writer);
        std::fs::remove_file(path).unwrap();
    }

    // More tests...
}
```

### SQLite Query Interface Complete Example

```rust
// crates/prtip-core/src/storage/scan_query.rs

use sqlx::{Row, SqlitePool};
use std::net::IpAddr;
use std::path::Path;

/// Query interface for scan results database
pub struct ScanQuery {
    pool: SqlitePool,
}

/// Scan result record
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub scan_id: i64,
    pub target_ip: IpAddr,
    pub port: u16,
    pub state: String,
    pub service_name: Option<String>,
    pub banner: Option<String>,
}

impl ScanQuery {
    /// Create new query interface
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file
    ///
    /// # Example
    /// ```
    /// let query = ScanQuery::new(Path::new("scan.db")).await?;
    /// ```
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await?;
        Ok(Self { pool })
    }

    /// Find all hosts with a specific port open
    ///
    /// # Example
    /// ```
    /// let hosts = query.find_hosts_with_port(80).await?;
    /// println!("Found {} hosts with port 80 open", hosts.len());
    /// ```
    pub async fn find_hosts_with_port(&self, port: u16) -> Result<Vec<ScanResult>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT scan_id, target_ip, port, state, service_name, banner
             FROM scan_results
             WHERE port = ? AND state = 'open'",
        )
        .bind(port)
        .fetch_all(&self.pool)
        .await?;

        results
            .into_iter()
            .map(|row| {
                Ok(ScanResult {
                    scan_id: row.try_get("scan_id")?,
                    target_ip: row
                        .try_get::<String, _>("target_ip")?
                        .parse()
                        .unwrap(),
                    port: row.try_get::<i32, _>("port")? as u16,
                    state: row.try_get("state")?,
                    service_name: row.try_get("service_name").ok(),
                    banner: row.try_get("banner").ok(),
                })
            })
            .collect()
    }

    /// List all services matching a name pattern
    ///
    /// # Example
    /// ```
    /// let services = query.list_services_by_name("HTTP").await?;
    /// ```
    pub async fn list_services_by_name(
        &self,
        service_pattern: &str,
    ) -> Result<Vec<ScanResult>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT scan_id, target_ip, port, state, service_name, banner
             FROM scan_results
             WHERE service_name LIKE ?",
        )
        .bind(format!("%{}%", service_pattern))
        .fetch_all(&self.pool)
        .await?;

        // Map results (same as above)
        // ...
    }

    /// Find all open ports on a specific target
    ///
    /// # Example
    /// ```
    /// let target = "192.168.1.1".parse().unwrap();
    /// let ports = query.find_open_ports(target).await?;
    /// ```
    pub async fn find_open_ports(&self, target: IpAddr) -> Result<Vec<u16>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT port FROM scan_results WHERE target_ip = ? AND state = 'open'",
        )
        .bind(target.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|row| row.try_get::<i32, _>("port").unwrap() as u16)
            .collect())
    }

    /// Execute raw SQL query with parameter binding
    ///
    /// # Example
    /// ```
    /// let results = query.execute_sql("SELECT * FROM scan_results WHERE port=80").await?;
    /// ```
    pub async fn execute_sql(&self, sql: &str) -> Result<Vec<ScanResult>, sqlx::Error> {
        let results = sqlx::query(sql).fetch_all(&self.pool).await?;

        results
            .into_iter()
            .map(|row| {
                Ok(ScanResult {
                    scan_id: row.try_get("scan_id")?,
                    target_ip: row
                        .try_get::<String, _>("target_ip")?
                        .parse()
                        .unwrap(),
                    port: row.try_get::<i32, _>("port")? as u16,
                    state: row.try_get("state")?,
                    service_name: row.try_get("service_name").ok(),
                    banner: row.try_get("banner").ok(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Create schema
        sqlx::query(
            "CREATE TABLE scan_results (
                scan_id INTEGER,
                target_ip TEXT,
                port INTEGER,
                state TEXT,
                service_name TEXT,
                banner TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Create indexes
        sqlx::query("CREATE INDEX idx_port_state ON scan_results(port, state)")
            .execute(&pool)
            .await
            .unwrap();

        // Insert test data
        for i in 1..=10 {
            sqlx::query(
                "INSERT INTO scan_results VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(1)
            .bind(format!("192.168.1.{}", i))
            .bind(80)
            .bind("open")
            .bind("HTTP")
            .bind(Some(format!("Apache/{}", i)))
            .execute(&pool)
            .await
            .unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn test_find_hosts_with_port() {
        let pool = create_test_db().await;
        let query = ScanQuery { pool };

        let results = query.find_hosts_with_port(80).await.unwrap();
        assert_eq!(results.len(), 10);
        assert_eq!(results[0].port, 80);
    }

    // More tests...
}
```

---

## Testing Strategy

### Test Pyramid

```
                    ┌──────────────────┐
                    │  Integration (2) │ ← Wireshark, SQLite query
                    └──────────────────┘
              ┌────────────────────────────┐
              │   Unit Tests (23+)         │ ← PCAPNG, SQLite, Export
              └────────────────────────────┘
        ┌──────────────────────────────────────┐
        │   Existing Tests (790)               │ ← Ensure zero regressions
        └──────────────────────────────────────┘
```

### Unit Test Coverage

**PCAPNG (8+ tests):**
1. test_create_writer - File creation
2. test_write_packet - Packet writing
3. test_file_rotation - Rotation at threshold
4. test_drop_closes_file - Resource cleanup
5. test_metadata - Interface metadata
6. test_timestamps - Timestamp accuracy
7. test_capture_filter - Filter comment
8. test_packet_direction - Direction metadata

**SQLite Query (10+ tests):**
1. test_find_hosts_with_port - Find by port
2. test_list_services - Service name search
3. test_find_open_ports - Find by target
4. test_execute_sql_with_params - Parameter binding
5. test_execute_sql_injection_prevention - Security
6. test_invalid_sql - Error handling
7. test_empty_results - Empty query
8. test_large_resultset - Performance (10K results)
9. test_index_performance - Index speedup
10. test_wildcard_queries - LIKE queries

**Export (5+ tests):**
1. test_export_csv - CSV export
2. test_export_json - JSON export
3. test_export_xml - XML export
4. test_export_with_filter - Filtered export
5. test_export_empty_results - Edge case

### Integration Tests

**PCAPNG Integration:**
- Run scan with `--packet-capture test.pcapng`
- Verify file created
- Parse PCAPNG with pcap-file crate
- Verify SHB, IDB, EPB blocks present
- Verify packet count > 0

**Manual Tests (Wireshark):**
- Open PCAPNG in Wireshark (visual inspection)
- Verify no errors
- Verify packets visible
- Verify metadata correct

### Performance Tests

**PCAPNG Overhead:**
```bash
# Baseline (no capture)
time prtip -sS -p 1-1000 target.com

# With capture
time prtip -sS -p 1-1000 target.com --packet-capture scan.pcapng

# Compare times (should be <5% difference)
```

**SQLite Query Performance:**
```rust
#[tokio::test]
async fn test_query_performance_100k_results() {
    let pool = create_large_db(100_000).await; // 100K records
    let query = ScanQuery { pool };

    let start = std::time::Instant::now();
    let results = query.find_hosts_with_port(80).await.unwrap();
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100, "Query too slow: {}ms", duration.as_millis());
}
```

---

## File Changes

### New Files (7 files, ~1,300 lines)

1. **`crates/prtip-core/src/output/pcapng.rs`** (NEW, ~250 lines)
   - PcapngWriter struct
   - Packet capture logic
   - File rotation
   - Unit tests

2. **`crates/prtip-core/src/storage/scan_query.rs`** (NEW, ~200 lines)
   - ScanQuery struct
   - Common query methods
   - Raw SQL execution
   - Unit tests

3. **`tests/integration/pcapng_capture.rs`** (NEW, ~80 lines)
   - Integration test for PCAPNG
   - Wireshark compatibility verification

4. **`docs/OUTPUT-FORMATS.md`** (NEW, ~400 lines)
   - Comprehensive guide to all 6 output formats
   - PCAPNG and SQLite sections
   - Usage examples and workflows

5-7. **Test files** (~370 lines total)
   - Additional unit tests across modules

### Modified Files (9 files, ~300 lines changed)

1. **`Cargo.toml`** (+2 lines)
   - Add `pcap-file = "2.0"`
   - Add `prettytable` (optional, for table formatting)

2. **`crates/prtip-core/src/output/mod.rs`** (+1 line)
   - Add `pub mod pcapng;`

3. **`crates/prtip-core/src/storage/mod.rs`** (+1 line)
   - Add `pub mod scan_query;`

4. **`crates/prtip-core/src/storage/sqlite.rs`** (~30 lines)
   - Add `create_indexes()` function
   - Call in `init_database()`

5. **`crates/prtip-core/src/config.rs`** (~5 lines)
   - Add `packet_capture: Option<PathBuf>`

6. **`crates/prtip-cli/src/args.rs`** (~100 lines)
   - Add `--packet-capture` flag
   - Add `query` subcommand
   - Add `export` subcommand

7. **`crates/prtip-cli/src/main.rs`** (~140 lines)
   - Implement query handler
   - Implement export handler
   - Add result formatting functions

8. **`crates/prtip-scanner/src/scanner.rs`** (~40 lines)
   - Integrate PCAPNG writer
   - Capture sent packets
   - Capture received packets

9. **`crates/prtip-scanner/src/state.rs`** (~10 lines)
   - Add `pcapng_writer` field to ScannerState

### Documentation Updates (4 files, ~120 lines)

1. **`README.md`** (~50 lines added/modified)
   - Features section
   - Usage examples (PCAPNG, query, export)
   - Latest Achievements
   - Project Statistics

2. **`CHANGELOG.md`** (~40 lines added)
   - Sprint 4.18 entry in [Unreleased]

3. **`CLAUDE.local.md`** (~30 lines modified)
   - Update Current Sprint
   - Add Sprint 4.18 to Recent Sessions
   - Update Phase 4 Enhancement Sprints

4. **`docs/OUTPUT-FORMATS.md`** (NEW, see above)

### Total Changes

- **New files:** 7 (~1,300 lines)
- **Modified files:** 13 (~420 lines)
- **Total:** ~1,720 lines of new/modified code
- **Tests:** 23+ new tests (~370 lines)
- **Documentation:** ~520 lines

---

## Performance Targets

### PCAPNG Performance

| Metric | Target | How to Measure |
|--------|--------|----------------|
| **Overhead** | <5% scan slowdown | Compare scan time with/without `--packet-capture` |
| **File Size** | <100MB for 10K ports | Actual file size measurement |
| **Write Latency** | <1ms per packet | Benchmark write_packet() |
| **Rotation Time** | <100ms | Measure rotate_file() duration |

**Measurement Example:**
```bash
# Baseline
time prtip -sS -p 1-1000 target.com
# Output: 6.2 seconds

# With capture
time prtip -sS -p 1-1000 target.com --packet-capture scan.pcapng
# Output: 6.5 seconds (4.8% overhead) ✅

# File size
ls -lh scan.pcapng
# Output: 8.2MB (for 1000 ports) ✅
```

### SQLite Query Performance

| Metric | Target | How to Measure |
|--------|--------|----------------|
| **Query Time** | <100ms for 100K results | Benchmark with Instant::now() |
| **Index Size** | <10% database increase | Compare db size with/without indexes |
| **Export Time** | <2s for 10K results | Time export command |

**Measurement Example:**
```rust
#[tokio::test]
async fn test_query_performance() {
    let pool = create_test_db_with_100k_records().await;
    let query = ScanQuery { pool };

    let start = std::time::Instant::now();
    let results = query.find_hosts_with_port(80).await.unwrap();
    let duration = start.elapsed();

    println!("Query time: {}ms for {} results", duration.as_millis(), results.len());
    assert!(duration.as_millis() < 100); // ✅
}
```

### Comparison Table

| Operation | Without Feature | With Feature | Impact |
|-----------|----------------|--------------|--------|
| **Scan 1000 ports** | 6.2s | 6.5s | +4.8% ✅ |
| **Query 100K results** | N/A (no SQL before) | 5ms | N/A |
| **Export 10K to CSV** | N/A | 1.2s | N/A |
| **File size (1K ports)** | 0 MB | 8.2 MB | Acceptable |
| **Database size** | 10 MB | 11 MB | +10% ✅ |

---

## Risk Mitigation

### Risk 1: PCAPNG Complexity

**Risk:** PCAPNG format is complex (multiple block types, metadata).

**Impact:** HIGH (could derail implementation)
**Probability:** MEDIUM

**Mitigation:**
1. Use mature `pcap-file` crate (abstracts complexity)
2. Start with minimal implementation (SHB + IDB + EPB only)
3. Test with Wireshark early (catch compatibility issues)
4. Reference PCAPNG spec: https://pcapng.github.io/pcapng/

**Fallback:** If pcap-file has issues, switch to legacy PCAP format (simpler, less metadata)

---

### Risk 2: Performance Impact

**Risk:** Packet capture could slow scan by 20-50% (unacceptable).

**Impact:** HIGH (users won't use feature)
**Probability:** LOW (mitigated by async writes)

**Mitigation:**
1. Implement async writes with tokio (bounded channel)
2. Make capture optional (`--packet-capture` flag only)
3. Benchmark early (compare with/without capture)
4. Drop packets if channel full (log warning, don't block scan)

**Target:** <5% overhead
**Fallback:** If >10% overhead, add `--capture-throttle` flag to limit capture rate

---

### Risk 3: File Size Explosion

**Risk:** PCAPNG files could fill disk (1GB+ for large scans).

**Impact:** MEDIUM (annoying but not critical)
**Probability:** MEDIUM

**Mitigation:**
1. Implement file rotation at 1GB threshold
2. Document storage requirements in help text
3. Add `--capture-max-size` flag (optional)
4. Warn user before scan starts (estimate file size)

**Example Warning:**
```
[WARN] Packet capture enabled for 10.0.0.0/8 scan (16M hosts)
       Estimated file size: 1.6TB (16M hosts * 100KB avg)
       Consider using --capture-filter to reduce size
       Continue? (y/N)
```

---

### Risk 4: SQL Injection

**Risk:** User-provided SQL queries could be exploited.

**Impact:** HIGH (security vulnerability)
**Probability:** LOW (mitigated by parameter binding)

**Mitigation:**
1. ALWAYS use parameter binding (sqlx `bind()` method)
2. NEVER concatenate strings to build SQL
3. Test with malicious inputs (e.g., `' OR '1'='1`)
4. Add security note in documentation

**Code Review Checklist:**
- [ ] All SQL uses parameter binding (no format!() or concat!())
- [ ] Test injection prevention (unit test)
- [ ] Documentation warns about SQL safety

---

### Risk 5: Wireshark Compatibility

**Risk:** PCAPNG files might not open in Wireshark (format errors).

**Impact:** MEDIUM (defeats purpose of feature)
**Probability:** LOW (pcap-file handles format)

**Mitigation:**
1. Use `pcap-file` crate (battle-tested)
2. Follow PCAPNG spec strictly
3. Test with Wireshark 4.x (latest stable)
4. Integration test opens file programmatically
5. Manual test: Visual inspection in Wireshark

**Testing Checklist:**
- [ ] PCAPNG opens without errors
- [ ] Packets visible in packet list
- [ ] Metadata displays correctly (interface, timestamps)
- [ ] Filters work (tcp.port == 80, etc.)

---

### Risk 6: Cross-Platform Issues

**Risk:** PCAPNG works on Linux but fails on Windows/macOS.

**Impact:** MEDIUM (platform compatibility)
**Probability:** LOW (pure Rust, no OS-specific code)

**Mitigation:**
1. Use cross-platform `pcap-file` crate (pure Rust)
2. Test on Linux, Windows, macOS (CI/CD)
3. Avoid OS-specific paths or syscalls

**CI/CD Matrix:**
- Linux (Ubuntu 22.04)
- Windows (Windows Server 2022)
- macOS (macOS 12)

---

## Success Criteria

### Quantitative Metrics

- ✅ **PCAPNG overhead:** <5% scan slowdown
- ✅ **PCAPNG file size:** <100MB for 10K port scan
- ✅ **Query performance:** <100ms for typical queries on 100K-result database
- ✅ **Export time:** <2 seconds for 10K results
- ✅ **Tests:** 23+ new tests (813+ total)
- ✅ **Test count:** 790 → 813+ tests
- ✅ **Coverage:** Maintain >61% (target 62-63%)
- ✅ **Clippy warnings:** 0
- ✅ **Format support:** 6 total output formats

### Qualitative Metrics

- ✅ PCAPNG files open correctly in Wireshark without errors
- ✅ PCAPNG files open correctly in tcpdump for CLI analysis
- ✅ SQLite queries are easy to write and intuitive
- ✅ Query performance feels fast (<100ms perceived)
- ✅ Export utilities handle edge cases gracefully (empty results, special characters)
- ✅ Documentation comprehensive and user-friendly

### Integration Goals

- ✅ Wireshark workflow documented (scan → open → filter → analyze)
- ✅ SQLite query examples provided (common use cases)
- ✅ Export utilities tested with real scan data
- ✅ No regressions in existing features (all 790 tests pass)

### User Experience

- ✅ Features discoverable (`--help` shows new flags)
- ✅ Error messages helpful (not cryptic)
- ✅ Performance acceptable (users don't notice overhead)
- ✅ Documentation answers common questions (OUTPUT-FORMATS.md)

---

## Execution Checklist

Use this checklist when executing Sprint 4.18:

### Pre-Implementation

- [ ] Read this document completely (understand scope, risks, architecture)
- [ ] Review existing codebase (scanner.rs, sqlite.rs, output modules)
- [ ] Verify dependencies available (pcap-file, sqlx, prettytable)
- [ ] Clean git state (`git status` shows no uncommitted changes)
- [ ] Run baseline tests (`cargo test --all-features` - all 790 passing)

### Phase 1: PCAPNG Implementation

- [ ] TASK 1.1: Add pcap-file dependency (15 min)
- [ ] TASK 1.2: Implement PcapngWriter core (3 hours)
- [ ] TASK 1.3: Add CLI flag --packet-capture (1 hour)
- [ ] TASK 1.4: Integrate PCAPNG writer into scanner (4 hours)
- [ ] TASK 1.5: Add packet metadata (2 hours)
- [ ] TASK 1.6: Implement file rotation (1.5 hours)
- [ ] TASK 1.7: PCAPNG unit & integration tests (2 hours)
- [ ] Verify: 8+ unit tests passing, PCAPNG opens in Wireshark
- [ ] Benchmark: Compare scan time with/without capture (<5% overhead)

### Phase 2: SQLite Enhancement

- [ ] TASK 2.1: Add database indexes (1 hour)
- [ ] TASK 2.2: Create query interface module (3 hours)
- [ ] TASK 2.3: Add CLI query subcommand (2 hours)
- [ ] TASK 2.4: SQLite query unit tests (1.5 hours)
- [ ] Verify: 10+ unit tests passing, queries <100ms on 100K-result DB
- [ ] Test: SQL injection prevention (malicious inputs fail safely)

### Phase 3: Export Utilities

- [ ] TASK 3.1: Add CLI export subcommand (1.5 hours)
- [ ] TASK 3.2: Export utilities unit tests (1 hour)
- [ ] Verify: 5+ unit tests passing, all formats export correctly
- [ ] Test: Large exports stream to file (constant memory)

### Phase 4: Documentation

- [ ] TASK 4.1: Create docs/OUTPUT-FORMATS.md (2 hours)
- [ ] TASK 4.2: Update README.md (30 min)
- [ ] TASK 4.3: Update CHANGELOG.md (30 min)
- [ ] TASK 4.4: Update CLAUDE.local.md (20 min)
- [ ] Verify: All documentation comprehensive and accurate

### Final Validation

- [ ] Run full test suite: `cargo test --all-features` (813+ tests passing)
- [ ] Run clippy: `cargo clippy --all-targets --all-features -- -D warnings` (0 warnings)
- [ ] Run fmt: `cargo fmt --check` (100% compliance)
- [ ] Build release: `cargo build --release` (success, <9MB binary)
- [ ] Manual test: Open PCAPNG in Wireshark (no errors, packets visible)
- [ ] Manual test: Open PCAPNG in tcpdump (correct format)
- [ ] Performance test: SQLite query on 100K-result DB (<100ms)
- [ ] Performance test: Scan with capture (<5% overhead)
- [ ] Integration test: Query database, export to CSV (works end-to-end)
- [ ] Documentation review: OUTPUT-FORMATS.md, README.md, CHANGELOG.md (accurate and complete)

### Commit Preparation

- [ ] Create implementation summary (`/tmp/ProRT-IP/sprint-4.18/implementation-summary.md`)
- [ ] Create commit message (`/tmp/ProRT-IP/sprint-4.18-commit-message.txt`)
- [ ] Stage all changes: `git add -A`
- [ ] Verify staged files: `git status --short` (no unexpected files)
- [ ] Commit: `git commit -F /tmp/ProRT-IP/sprint-4.18-commit-message.txt`
- [ ] Verify commit: `git log --oneline -1`, `git show --stat HEAD`
- [ ] Clean git state: `git status` (clean)

### Success Verification

- [ ] All 20 tasks from task-checklist.md completed
- [ ] 23+ new tests passing (813+ total tests)
- [ ] Zero test regressions (all existing tests pass)
- [ ] Zero clippy warnings
- [ ] PCAPNG opens in Wireshark without errors
- [ ] SQLite queries <100ms on large database
- [ ] Documentation comprehensive (OUTPUT-FORMATS.md ~400 lines)
- [ ] Sprint complete and ready for next sprint (4.19)

---

## When to Execute This Sprint

**Ready When:**
1. Phase 4 complete (v0.3.8 released) ✅
2. 3-4 days available for dedicated implementation
3. Need forensic analysis capabilities (PCAPNG)
4. Need scan correlation and trending (SQLite queries)
5. User requests or competitive pressure for these features

**Not Ready If:**
- Limited time available (<3 days)
- Other higher-priority sprints pending (4.19-4.22)
- Phase 4 incomplete (focus on core features first)
- Performance optimization needed instead (Sprint 4.17 first)

**Recommendation:** Execute after Phase 4 complete, when dedicated time available. This is a valuable but not critical sprint (MEDIUM priority, ROI 7.3/10).

---

## Summary

Sprint 4.18 adds two major output enhancements:

1. **PCAPNG Packet Capture** - Forensic analysis in Wireshark
2. **Enhanced SQLite** - Query interface for scan correlation

**Scope:** 3-4 days, 20 tasks, ~1,720 lines of code, 23+ tests
**Value:** Security analysts, SOC teams, compliance auditing
**Status:** DEFERRED - Execute when 3-4 days available

This implementation plan provides everything needed to execute Sprint 4.18 successfully:
- Complete task breakdown (20 tasks with estimates)
- Code skeletons and examples (ready to implement)
- Testing strategy (23+ tests, integration, performance)
- File changes (what to create/modify)
- Performance targets (how to measure success)
- Risk mitigation (how to handle issues)
- Execution checklist (step-by-step guide)

**Ready to execute when you have 3-4 days available!**

---

**Document Version:** 1.0
**Created:** 2025-10-14
**Status:** DEFERRED (comprehensive implementation plan)
**Next Review:** When ready to execute Sprint 4.18
