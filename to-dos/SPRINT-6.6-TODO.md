# Sprint 6.6: Advanced Features & Memory-Mapped I/O

**Status:** 🔄 IN PROGRESS
**Effort Estimate:** 15-20 hours
**Timeline:** 2025-11-22 to 2025-11-29 (7 days)
**Dependencies:** Sprint 6.5 (Interactive Selection) COMPLETE ✅
**Priority:** HIGH (Critical Path to Phase 6 completion)

---

## Sprint Overview

### Deliverables

1. **Memory-Mapped I/O (QW-3)** - 20-50% memory reduction for large-scale scans
2. **Scan History & Resume** - Pause/resume capability with state persistence
3. **Export Enhancements** - HTML and PDF report generation (CSV/JSON/XML exist)
4. **Real-Time Filtering** - Live result filtering in TUI
5. **Performance Dashboard** - CPU/memory/network metrics visualization

### Strategic Value

- **Memory-Mapped I/O**: Enables internet-scale scans (10M+ targets) on 8GB RAM systems
- **Pause/Resume**: Production-critical for long scans (power loss, network interruptions)
- **Scan History**: Audit trail for compliance (GDPR, PCI-DSS, SOC2)
- **Export Formats**: Integration with reporting tools (SIEM, ticketing systems, PDF reports)
- **Performance Dashboard**: Real-time resource monitoring prevents system overload

### Integration with Existing Codebase

**Existing Infrastructure (Leveraged):**
- ✅ `rusqlite` dependency (scan history database)
- ✅ `sysinfo` dependency (system metrics)
- ✅ CSV/JSON/XML/Text export (`crates/prtip-cli/src/export.rs`)
- ✅ Command history (CLI replay at `~/.prtip/history.json`)
- ✅ TUI event bus and widget system
- ✅ SQLite bundled build (no external dependencies)

**New Infrastructure (Required):**
- ❌ `memmap2` crate (memory-mapped file I/O)
- ❌ `tera` crate (HTML template engine)
- ❌ `csv` crate (already used in tests, needs workspace dependency)
- ❌ `printpdf` crate (optional, PDF generation)
- ❌ Scan state serialization (checkpoint system)

---

## Task Breakdown

### TASK AREA 1: Memory-Mapped I/O (6-8 hours)

**Purpose:** Reduce memory usage by 20-50% for large scans using memory-mapped files

#### Task 1.1: Add Dependencies & Setup (30 min)

**File:** `Cargo.toml` (workspace root)

**Changes:**
```toml
[workspace.dependencies]
# Memory-mapped I/O
memmap2 = "0.9"
csv = "1.3"  # Currently only in prtip-cli, promote to workspace

# Template engine for HTML reports
tera = "1.19"

# PDF generation (optional)
printpdf = "0.6"
```

**Test:** Dependency resolution
```bash
cargo check --all-features
```

**TDD Test Case:** None (dependency addition)

**Acceptance Criteria:**
- [ ] All crates compile successfully
- [ ] Zero new warnings from `cargo clippy`

---

#### Task 1.2: Implement MmapResultWriter (2.5 hours)

**File:** `crates/prtip-scanner/src/output/mmap_writer.rs` (new)

**Implementation:**
```rust
//! Memory-mapped result writer for efficient large-scale scans
//!
//! Uses memory-mapped files to reduce RAM usage by 20-50% compared to
//! in-memory buffering. Results are written to a binary format with
//! fixed-size entries for zero-copy random access.

use memmap2::{MmapMut, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use prtip_core::ScanResult;

const HEADER_SIZE: usize = 64;  // Version, entry_count, entry_size, checksum
const ENTRY_SIZE: usize = 512;  // Fixed-size entries (padded if needed)

/// Memory-mapped result writer
pub struct MmapResultWriter {
    file: File,
    mmap: MmapMut,
    entry_count: usize,
    capacity: usize,
}

impl MmapResultWriter {
    /// Create a new mmap writer with initial capacity
    pub fn new<P: AsRef<Path>>(path: P, initial_capacity: usize) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Pre-allocate file
        let file_size = HEADER_SIZE + (initial_capacity * ENTRY_SIZE);
        file.set_len(file_size as u64)?;

        // Create memory mapping
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        let mut writer = Self {
            file,
            mmap,
            entry_count: 0,
            capacity: initial_capacity,
        };

        writer.write_header()?;
        Ok(writer)
    }

    /// Write a scan result entry
    pub fn write_entry(&mut self, result: &ScanResult) -> io::Result<()> {
        if self.entry_count >= self.capacity {
            self.grow()?;
        }

        let offset = HEADER_SIZE + (self.entry_count * ENTRY_SIZE);
        let entry_bytes = bincode::serialize(result)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if entry_bytes.len() > ENTRY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Entry size {} exceeds maximum {}", entry_bytes.len(), ENTRY_SIZE),
            ));
        }

        // Write serialized data
        self.mmap[offset..offset + entry_bytes.len()].copy_from_slice(&entry_bytes);

        // Zero-fill remaining space
        for i in entry_bytes.len()..ENTRY_SIZE {
            self.mmap[offset + i] = 0;
        }

        self.entry_count += 1;
        self.update_header()?;
        Ok(())
    }

    /// Flush changes to disk
    pub fn flush(&mut self) -> io::Result<()> {
        self.mmap.flush()
    }

    /// Grow the mmap by 2x capacity
    fn grow(&mut self) -> io::Result<()> {
        self.capacity *= 2;
        let new_size = HEADER_SIZE + (self.capacity * ENTRY_SIZE);
        self.file.set_len(new_size as u64)?;

        // Re-create memory mapping
        drop(std::mem::replace(&mut self.mmap, unsafe {
            MmapOptions::new().map_mut(&self.file)?
        }));

        Ok(())
    }

    fn write_header(&mut self) -> io::Result<()> {
        // Version: 1
        self.mmap[0..8].copy_from_slice(&1u64.to_le_bytes());
        // Entry count: 0
        self.mmap[8..16].copy_from_slice(&0u64.to_le_bytes());
        // Entry size: ENTRY_SIZE
        self.mmap[16..24].copy_from_slice(&(ENTRY_SIZE as u64).to_le_bytes());
        // Checksum: 0 (TODO: implement CRC32)
        self.mmap[24..32].copy_from_slice(&0u64.to_le_bytes());
        Ok(())
    }

    fn update_header(&mut self) -> io::Result<()> {
        self.mmap[8..16].copy_from_slice(&(self.entry_count as u64).to_le_bytes());
        Ok(())
    }
}

impl Drop for MmapResultWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
```

**Module Registration:** Add to `crates/prtip-scanner/src/output/mod.rs`:
```rust
pub mod mmap_writer;
pub use mmap_writer::MmapResultWriter;
```

**TDD Test Cases** (`crates/prtip-scanner/src/output/mmap_writer.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::net::IpAddr;
    use chrono::Utc;
    use std::time::Duration;
    use prtip_core::PortState;

    fn create_test_result(port: u16) -> ScanResult {
        ScanResult {
            target_ip: "192.168.1.1".parse().unwrap(),
            port,
            state: PortState::Open,
            service: Some("http".to_string()),
            version: Some("Apache/2.4".to_string()),
            banner: Some(b"HTTP/1.1 200 OK".to_vec()),
            response_time: Duration::from_millis(42),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_mmap_write_single_entry() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = MmapResultWriter::new(temp.path(), 10).unwrap();

        let result = create_test_result(80);
        writer.write_entry(&result).unwrap();
        writer.flush().unwrap();

        assert_eq!(writer.entry_count, 1);
    }

    #[test]
    fn test_mmap_write_multiple_entries() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = MmapResultWriter::new(temp.path(), 10).unwrap();

        for port in 80..90 {
            writer.write_entry(&create_test_result(port)).unwrap();
        }
        writer.flush().unwrap();

        assert_eq!(writer.entry_count, 10);
    }

    #[test]
    fn test_mmap_growth() {
        let temp = NamedTempFile::new().unwrap();
        let mut writer = MmapResultWriter::new(temp.path(), 5).unwrap();

        // Write more than initial capacity
        for port in 80..90 {
            writer.write_entry(&create_test_result(port)).unwrap();
        }

        assert_eq!(writer.entry_count, 10);
        assert!(writer.capacity >= 10);
    }

    #[test]
    fn test_mmap_flush_persistence() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        {
            let mut writer = MmapResultWriter::new(&path, 10).unwrap();
            writer.write_entry(&create_test_result(80)).unwrap();
            writer.flush().unwrap();
        } // Drop writer

        // Verify file exists and has correct size
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() >= (HEADER_SIZE + ENTRY_SIZE) as u64);
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-scanner mmap_writer -- --nocapture
```

**Acceptance Criteria:**
- [ ] 4/4 tests passing
- [ ] Zero clippy warnings
- [ ] Handles growth correctly (capacity doubling)
- [ ] Drop calls flush() automatically

---

#### Task 1.3: Implement MmapResultReader (2 hours)

**File:** `crates/prtip-scanner/src/output/mmap_reader.rs` (new)

**Implementation:**
```rust
//! Memory-mapped result reader for zero-copy access to scan results

use memmap2::Mmap;
use std::fs::File;
use std::io;
use std::path::Path;
use prtip_core::ScanResult;

const HEADER_SIZE: usize = 64;
const ENTRY_SIZE: usize = 512;

/// Memory-mapped result reader
pub struct MmapResultReader {
    #[allow(dead_code)]
    file: File,
    mmap: Mmap,
    entry_count: usize,
    entry_size: usize,
}

impl MmapResultReader {
    /// Open an existing mmap result file
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too small to contain valid header",
            ));
        }

        // Parse header
        let version = u64::from_le_bytes(mmap[0..8].try_into().unwrap());
        if version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported version: {}", version),
            ));
        }

        let entry_count = u64::from_le_bytes(mmap[8..16].try_into().unwrap()) as usize;
        let entry_size = u64::from_le_bytes(mmap[16..24].try_into().unwrap()) as usize;

        if entry_size != ENTRY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Entry size mismatch: {} != {}", entry_size, ENTRY_SIZE),
            ));
        }

        Ok(Self {
            file,
            mmap,
            entry_count,
            entry_size,
        })
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entry_count
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }

    /// Get a specific entry by index
    pub fn get_entry(&self, index: usize) -> Option<ScanResult> {
        if index >= self.entry_count {
            return None;
        }

        let offset = HEADER_SIZE + (index * self.entry_size);
        let entry_bytes = &self.mmap[offset..offset + self.entry_size];

        // Find actual data length (stop at first zero byte)
        let data_len = entry_bytes.iter().position(|&b| b == 0).unwrap_or(self.entry_size);

        bincode::deserialize(&entry_bytes[..data_len]).ok()
    }

    /// Create an iterator over all entries
    pub fn iter(&self) -> MmapResultIterator {
        MmapResultIterator {
            reader: self,
            current: 0,
        }
    }
}

/// Iterator over mmap results
pub struct MmapResultIterator<'a> {
    reader: &'a MmapResultReader,
    current: usize,
}

impl<'a> Iterator for MmapResultIterator<'a> {
    type Item = ScanResult;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.get_entry(self.current)?;
        self.current += 1;
        Some(result)
    }
}
```

**Module Registration:** Add to `crates/prtip-scanner/src/output/mod.rs`:
```rust
pub mod mmap_reader;
pub use mmap_reader::MmapResultReader;
```

**TDD Test Cases** (`crates/prtip-scanner/src/output/mmap_reader.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::mmap_writer::MmapResultWriter;
    use tempfile::NamedTempFile;
    use prtip_core::{PortState, ScanResult};
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_result(port: u16) -> ScanResult {
        ScanResult {
            target_ip: "192.168.1.1".parse().unwrap(),
            port,
            state: PortState::Open,
            service: Some("http".to_string()),
            version: Some("Apache/2.4".to_string()),
            banner: Some(b"HTTP/1.1 200 OK".to_vec()),
            response_time: Duration::from_millis(42),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_mmap_read_single_entry() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        // Write
        {
            let mut writer = MmapResultWriter::new(&path, 10).unwrap();
            writer.write_entry(&create_test_result(80)).unwrap();
            writer.flush().unwrap();
        }

        // Read
        let reader = MmapResultReader::open(&path).unwrap();
        assert_eq!(reader.len(), 1);

        let result = reader.get_entry(0).unwrap();
        assert_eq!(result.port, 80);
        assert_eq!(result.target_ip.to_string(), "192.168.1.1");
    }

    #[test]
    fn test_mmap_read_multiple_entries() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        // Write 10 entries
        {
            let mut writer = MmapResultWriter::new(&path, 10).unwrap();
            for port in 80..90 {
                writer.write_entry(&create_test_result(port)).unwrap();
            }
            writer.flush().unwrap();
        }

        // Read
        let reader = MmapResultReader::open(&path).unwrap();
        assert_eq!(reader.len(), 10);

        for i in 0..10 {
            let result = reader.get_entry(i).unwrap();
            assert_eq!(result.port, 80 + i as u16);
        }
    }

    #[test]
    fn test_mmap_iterator() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        // Write
        {
            let mut writer = MmapResultWriter::new(&path, 5).unwrap();
            for port in 80..85 {
                writer.write_entry(&create_test_result(port)).unwrap();
            }
            writer.flush().unwrap();
        }

        // Iterate
        let reader = MmapResultReader::open(&path).unwrap();
        let results: Vec<_> = reader.iter().collect();

        assert_eq!(results.len(), 5);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.port, 80 + i as u16);
        }
    }

    #[test]
    fn test_mmap_out_of_bounds() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        {
            let mut writer = MmapResultWriter::new(&path, 5).unwrap();
            writer.write_entry(&create_test_result(80)).unwrap();
            writer.flush().unwrap();
        }

        let reader = MmapResultReader::open(&path).unwrap();
        assert_eq!(reader.len(), 1);
        assert!(reader.get_entry(0).is_some());
        assert!(reader.get_entry(1).is_none());
        assert!(reader.get_entry(100).is_none());
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-scanner mmap_reader -- --nocapture
```

**Acceptance Criteria:**
- [ ] 4/4 tests passing
- [ ] Iterator correctly iterates all entries
- [ ] Out-of-bounds access returns None
- [ ] Write-then-read round-trip preserves data

---

#### Task 1.4: Integrate Mmap with Scanner (1.5 hours)

**File:** `crates/prtip-scanner/src/scanner.rs` (modify)

**Changes:** Add mmap output option to scanner configuration

**Implementation:** Add to scanner configuration struct:
```rust
pub struct ScannerConfig {
    // ... existing fields ...

    /// Use memory-mapped output instead of in-memory buffering
    pub use_mmap: bool,
    /// Path to mmap output file (if use_mmap is true)
    pub mmap_output_path: Option<PathBuf>,
}
```

**File:** `crates/prtip-cli/src/main.rs` (modify)

**Changes:** Add CLI flag for mmap output:
```rust
#[arg(long, help = "Use memory-mapped file output (reduces memory usage)")]
use_mmap: bool,

#[arg(long = "mmap-output", value_name = "FILE", help = "Path to memory-mapped output file")]
mmap_output_path: Option<PathBuf>,
```

**TDD Test Case** (`crates/prtip-scanner/tests/mmap_integration.rs` - new file):
```rust
use prtip_scanner::{ScannerConfig, scanner::RawSocketScanner};
use prtip_scanner::output::{MmapResultWriter, MmapResultReader};
use std::net::IpAddr;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_scanner_mmap_output() {
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path().to_owned();

    let config = ScannerConfig {
        use_mmap: true,
        mmap_output_path: Some(path.clone()),
        // ... other config fields ...
    };

    // Run scan (mock or real, depending on test environment)
    // Scanner should write results to mmap file

    // Verify mmap file was created and contains results
    let reader = MmapResultReader::open(&path).unwrap();
    assert!(reader.len() > 0);
}
```

**Test Command:**
```bash
cargo test -p prtip-scanner mmap_integration -- --nocapture
```

**Acceptance Criteria:**
- [ ] `--use-mmap` flag accepted by CLI
- [ ] Scanner writes to mmap file when enabled
- [ ] Results readable after scan completion
- [ ] No memory regression when mmap disabled

---

#### Task 1.5: Benchmark Mmap Performance (1.5 hours)

**File:** `benchmarks/sprint-6.6-mmap/README.md` (new)

**Benchmark Script:** `benchmarks/sprint-6.6-mmap/run_benchmarks.sh`
```bash
#!/bin/bash
set -euo pipefail

# Memory usage comparison: standard vs mmap output
TARGETS="10.0.0.0/20"  # 4,096 hosts
PORTS="1-1000"

echo "Benchmark 1: Standard in-memory output"
/usr/bin/time -v cargo run --release -- \
    -sS -p $PORTS $TARGETS -oN /tmp/standard.txt \
    2>&1 | grep "Maximum resident set size" | tee results/standard_memory.txt

echo "Benchmark 2: Memory-mapped output"
/usr/bin/time -v cargo run --release -- \
    -sS -p $PORTS $TARGETS --use-mmap --mmap-output /tmp/mmap.bin \
    2>&1 | grep "Maximum resident set size" | tee results/mmap_memory.txt

# Calculate reduction
standard_mem=$(grep "Maximum resident set size" results/standard_memory.txt | awk '{print $6}')
mmap_mem=$(grep "Maximum resident set size" results/mmap_memory.txt | awk '{print $6}')
reduction=$(echo "scale=2; 100 * (1 - $mmap_mem / $standard_mem)" | bc)

echo "Memory reduction: ${reduction}%"
echo "Target: 20-50% reduction"

if (( $(echo "$reduction >= 20" | bc -l) )); then
    echo "✅ SUCCESS: Memory reduction target met"
    exit 0
else
    echo "❌ FAILED: Memory reduction below 20% target"
    exit 1
fi
```

**Performance Target:**
- 20-50% memory reduction (verified with `/usr/bin/time -v`)
- No scan time regression (±5% acceptable)
- Disk I/O <100 MB/s (not CPU-bound)

**Test Command:**
```bash
chmod +x benchmarks/sprint-6.6-mmap/run_benchmarks.sh
./benchmarks/sprint-6.6-mmap/run_benchmarks.sh
```

**Acceptance Criteria:**
- [ ] Benchmark script runs successfully
- [ ] Memory reduction ≥20% achieved
- [ ] Scan time within ±5% of standard output
- [ ] Results documented in `/tmp/ProRT-IP/SPRINT-6.6-MMAP-BENCHMARKS.md`

---

### TASK AREA 2: Scan History & Resume (4-5 hours)

**Purpose:** Enable pause/resume of long-running scans with state persistence

**Note:** Distinct from existing CLI command history in `crates/prtip-cli/src/history.rs`

#### Task 2.1: Design Scan History Database Schema (1 hour)

**File:** `crates/prtip-core/src/scan_history/schema.sql` (new, for documentation)

**Schema:**
```sql
-- Scan history table
CREATE TABLE IF NOT EXISTS scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT NOT NULL,           -- ISO 8601 timestamp
    end_time TEXT,                       -- NULL if still running
    status TEXT NOT NULL,                -- 'running', 'paused', 'completed', 'failed'
    target_spec TEXT NOT NULL,           -- e.g., "192.168.1.0/24"
    scan_type TEXT NOT NULL,             -- e.g., "SYN", "UDP", "Stealth"
    scan_config TEXT NOT NULL,           -- JSON serialized config
    results_path TEXT,                   -- Path to mmap/JSON output file
    total_targets INTEGER,               -- Total targets to scan
    targets_scanned INTEGER DEFAULT 0,   -- Targets scanned so far
    ports_discovered INTEGER DEFAULT 0   -- Ports discovered
);

CREATE INDEX IF NOT EXISTS idx_scans_status ON scans(status);
CREATE INDEX IF NOT EXISTS idx_scans_start_time ON scans(start_time);

-- Scan checkpoints table (for pause/resume)
CREATE TABLE IF NOT EXISTS scan_checkpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    checkpoint_time TEXT NOT NULL,       -- ISO 8601 timestamp
    targets_scanned INTEGER NOT NULL,    -- Progress at checkpoint
    scanner_state BLOB NOT NULL,         -- Serialized scanner state
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_checkpoints_scan_id ON scan_checkpoints(scan_id);
```

**File:** `crates/prtip-core/src/scan_history/mod.rs` (new)

**Implementation:**
```rust
//! Scan history and checkpoint management
//!
//! Provides persistence for scan state enabling pause/resume functionality.
//! Uses SQLite database at `~/.prtip/scan_history.db`.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Database initialization SQL
const SCHEMA_SQL: &str = include_str!("schema.sql");

/// Scan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanStatus {
    Running,
    Paused,
    Completed,
    Failed,
}

impl std::fmt::Display for ScanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScanStatus::Running => write!(f, "running"),
            ScanStatus::Paused => write!(f, "paused"),
            ScanStatus::Completed => write!(f, "completed"),
            ScanStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for ScanStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "running" => Ok(ScanStatus::Running),
            "paused" => Ok(ScanStatus::Paused),
            "completed" => Ok(ScanStatus::Completed),
            "failed" => Ok(ScanStatus::Failed),
            _ => Err(anyhow::anyhow!("Invalid scan status: {}", s)),
        }
    }
}

/// Scan history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanHistoryEntry {
    pub id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: ScanStatus,
    pub target_spec: String,
    pub scan_type: String,
    pub scan_config: String,  // JSON
    pub results_path: Option<String>,
    pub total_targets: i64,
    pub targets_scanned: i64,
    pub ports_discovered: i64,
}

/// Scan checkpoint (for pause/resume)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCheckpoint {
    pub id: i64,
    pub scan_id: i64,
    pub checkpoint_time: DateTime<Utc>,
    pub targets_scanned: i64,
    pub scanner_state: Vec<u8>,  // Serialized state
}

/// Scan history manager
pub struct ScanHistoryManager {
    conn: Connection,
}

impl ScanHistoryManager {
    /// Create new manager (opens/creates database)
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

        // Initialize schema
        conn.execute_batch(SCHEMA_SQL)?;

        Ok(Self { conn })
    }

    /// Get database path
    fn get_db_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".prtip").join("scan_history.db"))
    }

    /// Create a new scan entry
    pub fn create_scan(&mut self, entry: &ScanHistoryEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO scans (start_time, status, target_spec, scan_type, scan_config, \
             total_targets, targets_scanned, ports_discovered) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.start_time.to_rfc3339(),
                entry.status.to_string(),
                entry.target_spec,
                entry.scan_type,
                entry.scan_config,
                entry.total_targets,
                entry.targets_scanned,
                entry.ports_discovered,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Update scan status
    pub fn update_scan_status(&mut self, scan_id: i64, status: ScanStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE scans SET status = ?1 WHERE id = ?2",
            params![status.to_string(), scan_id],
        )?;
        Ok(())
    }

    /// Save a checkpoint
    pub fn save_checkpoint(&mut self, checkpoint: &ScanCheckpoint) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO scan_checkpoints (scan_id, checkpoint_time, targets_scanned, scanner_state) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                checkpoint.scan_id,
                checkpoint.checkpoint_time.to_rfc3339(),
                checkpoint.targets_scanned,
                checkpoint.scanner_state,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get latest checkpoint for a scan
    pub fn get_latest_checkpoint(&self, scan_id: i64) -> Result<Option<ScanCheckpoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, scan_id, checkpoint_time, targets_scanned, scanner_state \
             FROM scan_checkpoints WHERE scan_id = ?1 ORDER BY checkpoint_time DESC LIMIT 1"
        )?;

        let result = stmt.query_row(params![scan_id], |row| {
            Ok(ScanCheckpoint {
                id: row.get(0)?,
                scan_id: row.get(1)?,
                checkpoint_time: row.get::<_, String>(2)?.parse().unwrap(),
                targets_scanned: row.get(3)?,
                scanner_state: row.get(4)?,
            })
        });

        match result {
            Ok(checkpoint) => Ok(Some(checkpoint)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all scans
    pub fn list_scans(&self) -> Result<Vec<ScanHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, start_time, end_time, status, target_spec, scan_type, scan_config, \
             results_path, total_targets, targets_scanned, ports_discovered \
             FROM scans ORDER BY start_time DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ScanHistoryEntry {
                id: row.get(0)?,
                start_time: row.get::<_, String>(1)?.parse().unwrap(),
                end_time: row.get::<_, Option<String>>(2)?.map(|s| s.parse().unwrap()),
                status: row.get::<_, String>(3)?.parse().unwrap(),
                target_spec: row.get(4)?,
                scan_type: row.get(5)?,
                scan_config: row.get(6)?,
                results_path: row.get(7)?,
                total_targets: row.get(8)?,
                targets_scanned: row.get(9)?,
                ports_discovered: row.get(10)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }
}
```

**TDD Test Cases** (`crates/prtip-core/src/scan_history/mod.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_entry() -> ScanHistoryEntry {
        ScanHistoryEntry {
            id: 0,
            start_time: Utc::now(),
            end_time: None,
            status: ScanStatus::Running,
            target_spec: "192.168.1.0/24".to_string(),
            scan_type: "SYN".to_string(),
            scan_config: "{}".to_string(),
            results_path: None,
            total_targets: 256,
            targets_scanned: 0,
            ports_discovered: 0,
        }
    }

    #[test]
    fn test_create_scan() {
        let temp = NamedTempFile::new().unwrap();
        let mut manager = ScanHistoryManager::new().unwrap();

        let entry = create_test_entry();
        let scan_id = manager.create_scan(&entry).unwrap();

        assert!(scan_id > 0);
    }

    #[test]
    fn test_update_scan_status() {
        let mut manager = ScanHistoryManager::new().unwrap();

        let entry = create_test_entry();
        let scan_id = manager.create_scan(&entry).unwrap();

        manager.update_scan_status(scan_id, ScanStatus::Paused).unwrap();

        let scans = manager.list_scans().unwrap();
        assert_eq!(scans[0].status, ScanStatus::Paused);
    }

    #[test]
    fn test_save_checkpoint() {
        let mut manager = ScanHistoryManager::new().unwrap();

        let entry = create_test_entry();
        let scan_id = manager.create_scan(&entry).unwrap();

        let checkpoint = ScanCheckpoint {
            id: 0,
            scan_id,
            checkpoint_time: Utc::now(),
            targets_scanned: 100,
            scanner_state: vec![1, 2, 3, 4],
        };

        let checkpoint_id = manager.save_checkpoint(&checkpoint).unwrap();
        assert!(checkpoint_id > 0);
    }

    #[test]
    fn test_get_latest_checkpoint() {
        let mut manager = ScanHistoryManager::new().unwrap();

        let entry = create_test_entry();
        let scan_id = manager.create_scan(&entry).unwrap();

        // Save multiple checkpoints
        for i in 1..=3 {
            let checkpoint = ScanCheckpoint {
                id: 0,
                scan_id,
                checkpoint_time: Utc::now(),
                targets_scanned: i * 100,
                scanner_state: vec![i as u8],
            };
            manager.save_checkpoint(&checkpoint).unwrap();
        }

        // Get latest
        let latest = manager.get_latest_checkpoint(scan_id).unwrap().unwrap();
        assert_eq!(latest.targets_scanned, 300);
    }

    #[test]
    fn test_list_scans() {
        let mut manager = ScanHistoryManager::new().unwrap();

        for i in 1..=5 {
            let mut entry = create_test_entry();
            entry.target_spec = format!("10.0.{}.0/24", i);
            manager.create_scan(&entry).unwrap();
        }

        let scans = manager.list_scans().unwrap();
        assert_eq!(scans.len(), 5);
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-core scan_history -- --nocapture
```

**Acceptance Criteria:**
- [ ] 5/5 tests passing
- [ ] Database schema created successfully
- [ ] Checkpoint save/load round-trip works
- [ ] List scans ordered by start time (DESC)

---

#### Task 2.2: Implement Scanner Pause/Resume (2 hours)

**File:** `crates/prtip-scanner/src/scanner.rs` (modify)

**Implementation:** Add pause/resume methods to `RawSocketScanner`:

```rust
impl RawSocketScanner {
    /// Pause the scan and save checkpoint
    pub async fn pause(&mut self) -> io::Result<ScanCheckpoint> {
        // Stop sending new packets
        self.paused = true;

        // Wait for in-flight packets to complete (drain response queue)
        self.drain_response_queue().await?;

        // Serialize scanner state
        let scanner_state = bincode::serialize(&self.get_serializable_state())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let checkpoint = ScanCheckpoint {
            id: 0,
            scan_id: self.scan_id,
            checkpoint_time: Utc::now(),
            targets_scanned: self.progress.targets_scanned as i64,
            scanner_state,
        };

        // Save to database
        let mut history = ScanHistoryManager::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        history.save_checkpoint(&checkpoint)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Emit event
        self.event_bus.publish(Event::ScanPaused {
            scan_id: self.scan_id,
            checkpoint_time: checkpoint.checkpoint_time,
        });

        Ok(checkpoint)
    }

    /// Resume scan from checkpoint
    pub async fn resume(checkpoint: ScanCheckpoint) -> io::Result<Self> {
        // Deserialize scanner state
        let mut scanner: Self = bincode::deserialize(&checkpoint.scanner_state)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Resume scanning
        scanner.paused = false;

        // Emit event
        scanner.event_bus.publish(Event::ScanResumed {
            scan_id: checkpoint.scan_id,
            resume_time: Utc::now(),
        });

        Ok(scanner)
    }

    /// Drain response queue (wait for in-flight packets)
    async fn drain_response_queue(&mut self) -> io::Result<()> {
        // Wait up to 5 seconds for in-flight responses
        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        while !self.response_queue.is_empty() && start.elapsed() < timeout {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Get serializable state (subset of scanner fields)
    fn get_serializable_state(&self) -> SerializableS cannerState {
        SerializableScannerState {
            scan_id: self.scan_id,
            targets: self.targets.clone(),
            ports: self.ports.clone(),
            progress: self.progress.clone(),
            config: self.config.clone(),
        }
    }
}

/// Serializable scanner state (subset of fields)
#[derive(Serialize, Deserialize)]
struct SerializableScannerState {
    scan_id: i64,
    targets: Vec<IpAddr>,
    ports: Vec<u16>,
    progress: ScanProgress,
    config: ScannerConfig,
}
```

**TDD Test Case** (`crates/prtip-scanner/tests/pause_resume.rs` - new file):
```rust
use prtip_scanner::scanner::RawSocketScanner;
use prtip_core::scan_history::ScanHistoryManager;

#[tokio::test]
async fn test_pause_resume_preserves_state() {
    let mut scanner = RawSocketScanner::new(/* config */).unwrap();

    // Start scan (mock or real)
    tokio::spawn(async move {
        scanner.run().await
    });

    // Pause after some progress
    tokio::time::sleep(Duration::from_secs(2)).await;
    let checkpoint = scanner.pause().await.unwrap();

    assert_eq!(checkpoint.targets_scanned > 0);

    // Resume
    let mut resumed_scanner = RawSocketScanner::resume(checkpoint).await.unwrap();

    // Verify state preserved
    assert_eq!(resumed_scanner.progress.targets_scanned, checkpoint.targets_scanned);
}

#[tokio::test]
async fn test_pause_waits_for_in_flight() {
    let mut scanner = RawSocketScanner::new(/* config */).unwrap();

    // Send some packets
    // ... send logic ...

    // Pause should wait for responses
    let start = Instant::now();
    scanner.pause().await.unwrap();
    let elapsed = start.elapsed();

    // Should take time to drain responses
    assert!(elapsed > Duration::from_millis(100));
    assert!(scanner.response_queue.is_empty());
}
```

**Test Command:**
```bash
cargo test -p prtip-scanner pause_resume -- --nocapture
```

**Acceptance Criteria:**
- [ ] 2/2 tests passing
- [ ] Pause waits for in-flight packets (drain queue)
- [ ] Resume restores scanner state correctly
- [ ] Events emitted (ScanPaused, ScanResumed)

---

#### Task 2.3: TUI Pause/Resume Controls (1 hour)

**File:** `crates/prtip-tui/src/app.rs` (modify)

**Implementation:** Add keyboard handlers:
```rust
impl App {
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            // ... existing handlers ...

            KeyCode::Char('p') | KeyCode::Char('P') => {
                if self.scan_state.is_running() {
                    self.pause_scan().await?;
                }
            },

            KeyCode::Char('r') | KeyCode::Char('R') => {
                if self.scan_state.is_paused() {
                    self.resume_scan().await?;
                }
            },

            _ => {}
        }
        Ok(())
    }

    async fn pause_scan(&mut self) -> io::Result<()> {
        // Trigger pause via event bus
        self.event_bus.publish(Event::PauseRequested);
        self.scan_state.set_paused(true);
        Ok(())
    }

    async fn resume_scan(&mut self) -> io::Result<()> {
        // Load checkpoint and resume
        let history = ScanHistoryManager::new()?;
        if let Some(checkpoint) = history.get_latest_checkpoint(self.scan_id)? {
            self.event_bus.publish(Event::ResumeRequested { checkpoint });
            self.scan_state.set_paused(false);
        }
        Ok(())
    }
}
```

**File:** `crates/prtip-tui/src/widgets/status.rs` (modify)

**Display pause status:**
```rust
impl StatusWidget {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let status_text = if self.scan_state.is_paused() {
            format!("PAUSED - Press 'r' to resume (checkpoint: {})",
                    self.scan_state.checkpoint_time.format("%H:%M:%S"))
        } else {
            format!("RUNNING - Press 'p' to pause")
        };

        // Render status text...
    }
}
```

**TDD Test Case** (`crates/prtip-tui/tests/pause_resume_ui.rs` - new file):
```rust
use prtip_tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};

#[tokio::test]
async fn test_pause_key_handler() {
    let mut app = App::new(/* config */).unwrap();

    // Simulate 'p' key press
    let event = KeyEvent::from(KeyCode::Char('p'));
    app.handle_key_event(event).await.unwrap();

    assert!(app.scan_state.is_paused());
}

#[tokio::test]
async fn test_resume_key_handler() {
    let mut app = App::new(/* config */).unwrap();
    app.scan_state.set_paused(true);

    // Simulate 'r' key press
    let event = KeyEvent::from(KeyCode::Char('r'));
    app.handle_key_event(event).await.unwrap();

    assert!(!app.scan_state.is_paused());
}
```

**Test Command:**
```bash
cargo test -p prtip-tui pause_resume_ui -- --nocapture
```

**Acceptance Criteria:**
- [ ] 2/2 tests passing
- [ ] 'p' key pauses scan (event emitted)
- [ ] 'r' key resumes scan (checkpoint loaded)
- [ ] Status widget displays pause/resume instructions

---

#### Task 2.4: CLI Scan History Commands (30 min)

**File:** `crates/prtip-cli/src/commands/history.rs` (new)

**Implementation:**
```rust
//! Scan history CLI commands (distinct from command history in history.rs)

use clap::Subcommand;
use prtip_core::scan_history::{ScanHistoryManager, ScanStatus};
use anyhow::Result;

#[derive(Subcommand)]
pub enum HistoryCommand {
    /// List scan history
    List {
        #[arg(long, help = "Filter by status")]
        status: Option<ScanStatus>,
    },

    /// Resume most recent paused scan
    Resume {
        #[arg(long, help = "Specific scan ID to resume")]
        scan_id: Option<i64>,
    },

    /// Clean old scan history
    Clean {
        #[arg(long, help = "Delete scans older than N days")]
        older_than: u32,
    },
}

impl HistoryCommand {
    pub fn execute(&self) -> Result<()> {
        match self {
            HistoryCommand::List { status } => self.list_scans(status.as_ref()),
            HistoryCommand::Resume { scan_id } => self.resume_scan(*scan_id),
            HistoryCommand::Clean { older_than } => self.clean_history(*older_than),
        }
    }

    fn list_scans(&self, status_filter: Option<&ScanStatus>) -> Result<()> {
        let manager = ScanHistoryManager::new()?;
        let scans = manager.list_scans()?;

        println!("Scan History ({} entries):", scans.len());
        println!("{:<6} {:<20} {:<12} {:<20}", "ID", "Start Time", "Status", "Target");
        println!("{}", "-".repeat(70));

        for scan in scans {
            if let Some(filter) = status_filter {
                if scan.status != *filter {
                    continue;
                }
            }

            println!(
                "{:<6} {:<20} {:<12} {:<20}",
                scan.id,
                scan.start_time.format("%Y-%m-%d %H:%M:%S"),
                format!("{:?}", scan.status),
                scan.target_spec
            );
        }

        Ok(())
    }

    fn resume_scan(&self, scan_id: Option<i64>) -> Result<()> {
        let manager = ScanHistoryManager::new()?;

        let scan_id = if let Some(id) = scan_id {
            id
        } else {
            // Get most recent paused scan
            let scans = manager.list_scans()?;
            scans.iter()
                .find(|s| s.status == ScanStatus::Paused)
                .map(|s| s.id)
                .ok_or_else(|| anyhow::anyhow!("No paused scans found"))?
        };

        let checkpoint = manager.get_latest_checkpoint(scan_id)?
            .ok_or_else(|| anyhow::anyhow!("No checkpoint found for scan {}", scan_id))?;

        println!("Resuming scan {} from checkpoint...", scan_id);
        println!("Progress: {} targets scanned", checkpoint.targets_scanned);

        // Resume scanner (integration with scanner module)
        // ... resume logic ...

        Ok(())
    }

    fn clean_history(&self, days: u32) -> Result<()> {
        println!("Cleaning scans older than {} days...", days);
        // ... cleanup logic ...
        Ok(())
    }
}
```

**CLI Integration:** Add to `crates/prtip-cli/src/main.rs`:
```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Manage scan history (pause/resume)
    #[command(subcommand)]
    History(HistoryCommand),
}
```

**Test Command:**
```bash
cargo run -- history list
cargo run -- history resume --scan-id 1
cargo run -- history clean --older-than 30
```

**Acceptance Criteria:**
- [ ] `prtip history list` displays all scans
- [ ] `prtip history resume` loads checkpoint
- [ ] `prtip history clean` removes old entries
- [ ] Help text clear and accurate

---

### TASK AREA 3: Export Enhancements (3-4 hours)

**Purpose:** Add HTML and PDF export formats (CSV/JSON/XML exist)

**Note:** Existing exports in `crates/prtip-cli/src/export.rs`: JSON ✅, CSV ✅, XML ✅, Text ✅

#### Task 3.1: Add HTML Report Generation (2 hours)

**File:** `crates/prtip-cli/src/export/html_report.rs` (new)

**Implementation:**
```rust
//! HTML report generation using Tera templates

use tera::{Tera, Context};
use prtip_core::ScanResult;
use std::collections::HashMap;
use chrono::Utc;
use anyhow::Result;

/// HTML report generator
pub struct HtmlReporter {
    tera: Tera,
}

impl HtmlReporter {
    /// Create new HTML reporter
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register templates
        tera.add_raw_template("scan_report", include_str!("templates/scan_report.html"))?;

        Ok(Self { tera })
    }

    /// Generate HTML report from scan results
    pub fn generate(&self, results: &[ScanResult]) -> Result<String> {
        let mut context = Context::new();

        // Statistics
        let total_hosts = results.iter()
            .map(|r| r.target_ip)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let total_ports = results.len();

        let open_ports = results.iter()
            .filter(|r| matches!(r.state, prtip_core::PortState::Open))
            .count();

        // Group by host
        let mut hosts: HashMap<String, Vec<&ScanResult>> = HashMap::new();
        for result in results {
            hosts.entry(result.target_ip.to_string())
                .or_default()
                .push(result);
        }

        // Service breakdown
        let mut services: HashMap<String, usize> = HashMap::new();
        for result in results {
            if let Some(ref service) = result.service {
                *services.entry(service.clone()).or_insert(0) += 1;
            }
        }

        // Add to context
        context.insert("report_time", &Utc::now().to_rfc3339());
        context.insert("total_hosts", &total_hosts);
        context.insert("total_ports", &total_ports);
        context.insert("open_ports", &open_ports);
        context.insert("hosts", &hosts);
        context.insert("services", &services);
        context.insert("prtip_version", env!("CARGO_PKG_VERSION"));

        // Render template
        let html = self.tera.render("scan_report", &context)?;

        Ok(html)
    }
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new().expect("Failed to create HTML reporter")
    }
}
```

**File:** `crates/prtip-cli/src/export/templates/scan_report.html` (new)

**Template:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ProRT-IP Scan Report</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
    <style>
        body { padding: 20px; }
        .header { background: #2c3e50; color: white; padding: 30px; margin-bottom: 30px; }
        .stat-card { margin-bottom: 20px; }
        .host-section { margin-bottom: 40px; }
        .port-table { font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="header">
        <h1>ProRT-IP WarScan Report</h1>
        <p>Generated: {{ report_time }}</p>
        <p>ProRT-IP Version: {{ prtip_version }}</p>
    </div>

    <div class="container">
        <!-- Summary Statistics -->
        <div class="row">
            <div class="col-md-4">
                <div class="card stat-card">
                    <div class="card-body">
                        <h5>Total Hosts</h5>
                        <h2>{{ total_hosts }}</h2>
                    </div>
                </div>
            </div>
            <div class="col-md-4">
                <div class="card stat-card">
                    <div class="card-body">
                        <h5>Total Ports</h5>
                        <h2>{{ total_ports }}</h2>
                    </div>
                </div>
            </div>
            <div class="col-md-4">
                <div class="card stat-card">
                    <div class="card-body">
                        <h5>Open Ports</h5>
                        <h2 class="text-success">{{ open_ports }}</h2>
                    </div>
                </div>
            </div>
        </div>

        <!-- Service Breakdown -->
        <div class="card mb-4">
            <div class="card-header">
                <h4>Service Breakdown</h4>
            </div>
            <div class="card-body">
                <table class="table table-sm">
                    <thead>
                        <tr>
                            <th>Service</th>
                            <th>Count</th>
                        </tr>
                    </thead>
                    <tbody>
                        {% for service, count in services %}
                        <tr>
                            <td>{{ service }}</td>
                            <td><span class="badge bg-primary">{{ count }}</span></td>
                        </tr>
                        {% endfor %}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Discovered Hosts -->
        <h3>Discovered Hosts</h3>
        {% for ip, ports in hosts %}
        <div class="host-section">
            <div class="card">
                <div class="card-header">
                    <h5>Host: {{ ip }}</h5>
                </div>
                <div class="card-body">
                    <table class="table table-striped port-table">
                        <thead>
                            <tr>
                                <th>Port</th>
                                <th>State</th>
                                <th>Service</th>
                                <th>Version</th>
                                <th>Banner</th>
                            </tr>
                        </thead>
                        <tbody>
                            {% for port in ports %}
                            <tr>
                                <td>{{ port.port }}</td>
                                <td>
                                    {% if port.state == "Open" %}
                                    <span class="badge bg-success">{{ port.state }}</span>
                                    {% elif port.state == "Closed" %}
                                    <span class="badge bg-secondary">{{ port.state }}</span>
                                    {% else %}
                                    <span class="badge bg-warning">{{ port.state }}</span>
                                    {% endif %}
                                </td>
                                <td>{{ port.service | default(value="unknown") }}</td>
                                <td>{{ port.version | default(value="-") }}</td>
                                <td><code>{{ port.banner | default(value="-") | truncate(length=50) }}</code></td>
                            </tr>
                            {% endfor %}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        {% endfor %}
    </div>

    <footer class="text-center mt-5 mb-3">
        <p class="text-muted">Generated by ProRT-IP WarScan v{{ prtip_version }}</p>
    </footer>
</body>
</html>
```

**Module Registration:** Add to `crates/prtip-cli/src/export/mod.rs`:
```rust
pub mod html_report;
pub use html_report::HtmlReporter;
```

**TDD Test Case** (`crates/prtip-cli/src/export/html_report.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::{PortState, ScanResult};
    use std::time::Duration;
    use chrono::Utc;

    fn create_test_result(port: u16) -> ScanResult {
        ScanResult {
            target_ip: "192.168.1.100".parse().unwrap(),
            port,
            state: PortState::Open,
            service: Some("http".to_string()),
            version: Some("Apache/2.4".to_string()),
            banner: Some(b"HTTP/1.1 200 OK".to_vec()),
            response_time: Duration::from_millis(42),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_html_generation() {
        let reporter = HtmlReporter::new().unwrap();
        let results = vec![
            create_test_result(80),
            create_test_result(443),
        ];

        let html = reporter.generate(&results).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("ProRT-IP WarScan Report"));
        assert!(html.contains("192.168.1.100"));
        assert!(html.contains("http"));
    }

    #[test]
    fn test_html_statistics() {
        let reporter = HtmlReporter::new().unwrap();
        let results = vec![
            create_test_result(80),
            create_test_result(443),
        ];

        let html = reporter.generate(&results).unwrap();

        // Check statistics
        assert!(html.contains("Total Hosts"));
        assert!(html.contains("Total Ports"));
        assert!(html.contains("Open Ports"));
    }

    #[test]
    fn test_html_service_breakdown() {
        let reporter = HtmlReporter::new().unwrap();
        let results = vec![
            create_test_result(80),
            create_test_result(443),
        ];

        let html = reporter.generate(&results).unwrap();

        assert!(html.contains("Service Breakdown"));
        assert!(html.contains("http"));
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-cli html_report -- --nocapture
```

**Acceptance Criteria:**
- [ ] 3/3 tests passing
- [ ] HTML valid (DOCTYPE, Bootstrap CSS)
- [ ] Statistics section present (hosts, ports, open)
- [ ] Service breakdown table rendered
- [ ] Host sections with port tables

---

#### Task 3.2: Add PDF Export (Optional, 1.5 hours)

**File:** `crates/prtip-cli/src/export/pdf_report.rs` (new)

**Implementation:**
```rust
//! PDF report generation using printpdf

use printpdf::*;
use prtip_core::ScanResult;
use std::fs::File;
use std::io::BufWriter;
use anyhow::Result;

/// PDF report generator
pub struct PdfReporter;

impl PdfReporter {
    /// Generate PDF report from scan results
    pub fn generate(results: &[ScanResult], output_path: &str) -> Result<()> {
        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new("ProRT-IP Scan Report", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Add title
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        current_layer.use_text("ProRT-IP WarScan Report", 24.0, Mm(10.0), Mm(280.0), &font);

        // Add statistics
        let total_ports = results.len();
        current_layer.use_text(
            &format!("Total Ports Scanned: {}", total_ports),
            12.0,
            Mm(10.0),
            Mm(260.0),
            &font
        );

        // Add port table (simplified)
        let mut y = 250.0;
        for (i, result) in results.iter().take(50).enumerate() {
            let line = format!("{} {} {:?}", result.target_ip, result.port, result.state);
            current_layer.use_text(&line, 10.0, Mm(10.0), Mm(y), &font);
            y -= 5.0;

            if y < 10.0 {
                break;  // Page limit
            }
        }

        // Save PDF
        doc.save(&mut BufWriter::new(File::create(output_path)?))?;

        Ok(())
    }
}
```

**TDD Test Case** (`crates/prtip-cli/src/export/pdf_report.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pdf_generation() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_str().unwrap();

        let results = vec![/* test results */];

        PdfReporter::generate(&results, path).unwrap();

        // Verify PDF file exists and has content
        let metadata = std::fs::metadata(path).unwrap();
        assert!(metadata.len() > 0);
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-cli pdf_report -- --nocapture
```

**Acceptance Criteria:**
- [ ] 1/1 test passing
- [ ] PDF file created successfully
- [ ] Title and statistics visible
- [ ] Port table rendered (first 50 entries)

**Note:** PDF export is OPTIONAL. Can be skipped if time-constrained (focus on HTML first).

---

#### Task 3.3: TUI Export Menu (30 min)

**File:** `crates/prtip-tui/src/widgets/export_menu.rs` (new)

**Implementation:**
```rust
//! Export menu widget for TUI

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};

pub struct ExportMenuWidget {
    formats: Vec<String>,
    selected_index: usize,
}

impl ExportMenuWidget {
    pub fn new() -> Self {
        Self {
            formats: vec![
                "Plain Text (.txt)".to_string(),
                "JSON (.json)".to_string(),
                "XML (Nmap format)".to_string(),
                "CSV (.csv)".to_string(),
                "HTML Report (.html)".to_string(),
                "PDF Report (.pdf)".to_string(),
            ],
            selected_index: 0,
        }
    }

    pub fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.formats.len();
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.formats.len() - 1;
        }
    }

    pub fn get_selected_format(&self) -> &str {
        &self.formats[self.selected_index]
    }
}

impl ratatui::widgets::Widget for &ExportMenuWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Export Results")
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self.formats.iter()
            .enumerate()
            .map(|(i, format)| {
                let prefix = if i == self.selected_index { "> " } else { "  " };
                ListItem::new(format!("{}{}", prefix, format))
            })
            .collect();

        let list = List::new(items).block(block);
        list.render(area, buf);
    }
}
```

**File:** `crates/prtip-tui/src/app.rs` (modify)

**Add keyboard handler:**
```rust
KeyCode::Char('e') | KeyCode::Char('E') => {
    self.show_export_menu = true;
}
```

**Test Command:**
```bash
cargo test -p prtip-tui export_menu -- --nocapture
```

**Acceptance Criteria:**
- [ ] Export menu displays on 'e' key
- [ ] Up/Down arrows navigate formats
- [ ] Enter key triggers export
- [ ] ESC closes menu

---

### TASK AREA 4: Real-Time Filtering (2-3 hours)

**Purpose:** Filter live scan results in TUI by port range, protocol, service, etc.

#### Task 4.1: Implement LiveFilter (1.5 hours)

**File:** `crates/prtip-tui/src/filters/live_filter.rs` (new)

**Implementation:**
```rust
//! Live result filtering for TUI

use prtip_core::{ScanResult, PortState};
use regex::Regex;
use std::net::IpAddr;

/// Live result filter
#[derive(Debug, Clone)]
pub struct LiveFilter {
    pub port_range: Option<(u16, u16)>,
    pub protocols: Option<Vec<String>>,  // "tcp", "udp", etc.
    pub states: Option<Vec<PortState>>,
    pub service_regex: Option<Regex>,
    pub min_confidence: Option<f64>,
    pub ip_filter: Option<Vec<IpAddr>>,
}

impl LiveFilter {
    /// Create empty filter (matches all)
    pub fn new() -> Self {
        Self {
            port_range: None,
            protocols: None,
            states: None,
            service_regex: None,
            min_confidence: None,
            ip_filter: None,
        }
    }

    /// Check if result matches all filter criteria
    pub fn matches(&self, result: &ScanResult) -> bool {
        // Port range check
        if let Some((min, max)) = self.port_range {
            if result.port < min || result.port > max {
                return false;
            }
        }

        // State check
        if let Some(ref states) = self.states {
            if !states.contains(&result.state) {
                return false;
            }
        }

        // Service regex check
        if let Some(ref regex) = self.service_regex {
            if !result.service.as_ref().map_or(false, |s| regex.is_match(s)) {
                return false;
            }
        }

        // IP filter check
        if let Some(ref ips) = self.ip_filter {
            if !ips.contains(&result.target_ip) {
                return false;
            }
        }

        true
    }

    /// Builder methods
    pub fn with_port_range(mut self, min: u16, max: u16) -> Self {
        self.port_range = Some((min, max));
        self
    }

    pub fn with_states(mut self, states: Vec<PortState>) -> Self {
        self.states = Some(states);
        self
    }

    pub fn with_service_regex(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.service_regex = Some(Regex::new(pattern)?);
        Ok(self)
    }
}

impl Default for LiveFilter {
    fn default() -> Self {
        Self::new()
    }
}
```

**TDD Test Cases** (`crates/prtip-tui/src/filters/live_filter.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use chrono::Utc;

    fn create_test_result(port: u16, state: PortState, service: Option<&str>) -> ScanResult {
        ScanResult {
            target_ip: "192.168.1.1".parse().unwrap(),
            port,
            state,
            service: service.map(|s| s.to_string()),
            version: None,
            banner: None,
            response_time: Duration::from_millis(42),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_filter_port_range() {
        let filter = LiveFilter::new().with_port_range(80, 443);

        assert!(filter.matches(&create_test_result(80, PortState::Open, None)));
        assert!(filter.matches(&create_test_result(443, PortState::Open, None)));
        assert!(!filter.matches(&create_test_result(22, PortState::Open, None)));
        assert!(!filter.matches(&create_test_result(8080, PortState::Open, None)));
    }

    #[test]
    fn test_filter_state() {
        let filter = LiveFilter::new().with_states(vec![PortState::Open]);

        assert!(filter.matches(&create_test_result(80, PortState::Open, None)));
        assert!(!filter.matches(&create_test_result(80, PortState::Closed, None)));
        assert!(!filter.matches(&create_test_result(80, PortState::Filtered, None)));
    }

    #[test]
    fn test_filter_service_regex() {
        let filter = LiveFilter::new()
            .with_service_regex("^http").unwrap();

        assert!(filter.matches(&create_test_result(80, PortState::Open, Some("http"))));
        assert!(filter.matches(&create_test_result(443, PortState::Open, Some("https"))));
        assert!(!filter.matches(&create_test_result(22, PortState::Open, Some("ssh"))));
    }

    #[test]
    fn test_filter_combined() {
        let filter = LiveFilter::new()
            .with_port_range(80, 443)
            .with_states(vec![PortState::Open])
            .with_service_regex("http").unwrap();

        // Matches all criteria
        assert!(filter.matches(&create_test_result(80, PortState::Open, Some("http"))));

        // Fails port range
        assert!(!filter.matches(&create_test_result(22, PortState::Open, Some("http"))));

        // Fails state
        assert!(!filter.matches(&create_test_result(80, PortState::Closed, Some("http"))));

        // Fails service
        assert!(!filter.matches(&create_test_result(80, PortState::Open, Some("ssh"))));
    }

    #[test]
    fn test_empty_filter_matches_all() {
        let filter = LiveFilter::new();

        assert!(filter.matches(&create_test_result(80, PortState::Open, Some("http"))));
        assert!(filter.matches(&create_test_result(22, PortState::Closed, Some("ssh"))));
        assert!(filter.matches(&create_test_result(443, PortState::Filtered, None)));
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-tui live_filter -- --nocapture
```

**Acceptance Criteria:**
- [ ] 5/5 tests passing
- [ ] Port range filtering works
- [ ] State filtering works (Open/Closed/Filtered)
- [ ] Service regex filtering works
- [ ] Combined filters use AND logic

---

#### Task 4.2: Apply Filter to PortTableWidget (1 hour)

**File:** `crates/prtip-tui/src/widgets/port_table.rs` (modify)

**Implementation:**
```rust
use crate::filters::live_filter::LiveFilter;

pub struct PortTableWidget {
    all_results: Vec<ScanResult>,
    filtered_results: Vec<ScanResult>,
    filter: Option<LiveFilter>,
}

impl PortTableWidget {
    pub fn set_filter(&mut self, filter: LiveFilter) {
        self.filter = Some(filter);
        self.apply_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.filtered_results = self.all_results.clone();
    }

    fn apply_filter(&mut self) {
        if let Some(ref filter) = self.filter {
            self.filtered_results = self.all_results.iter()
                .filter(|r| filter.matches(r))
                .cloned()
                .collect();
        } else {
            self.filtered_results = self.all_results.clone();
        }
    }

    pub fn add_result(&mut self, result: ScanResult) {
        self.all_results.push(result.clone());

        // Apply filter to new result
        if let Some(ref filter) = self.filter {
            if filter.matches(&result) {
                self.filtered_results.push(result);
            }
        } else {
            self.filtered_results.push(result);
        }
    }

    /// Get filter status for display
    pub fn get_filter_status(&self) -> String {
        if let Some(ref filter) = self.filter {
            format!("Showing {}/{} results (filtered)",
                    self.filtered_results.len(),
                    self.all_results.len())
        } else {
            format!("Showing {} results", self.all_results.len())
        }
    }
}
```

**TDD Test Case** (`crates/prtip-tui/src/widgets/port_table.rs` - add to existing tests):
```rust
#[test]
fn test_port_table_filtering() {
    let mut widget = PortTableWidget::new();

    // Add results
    widget.add_result(create_test_result(80, PortState::Open, Some("http")));
    widget.add_result(create_test_result(443, PortState::Open, Some("https")));
    widget.add_result(create_test_result(22, PortState::Closed, Some("ssh")));

    assert_eq!(widget.filtered_results.len(), 3);

    // Apply filter (only Open ports)
    let filter = LiveFilter::new().with_states(vec![PortState::Open]);
    widget.set_filter(filter);

    assert_eq!(widget.filtered_results.len(), 2);
    assert_eq!(widget.get_filter_status(), "Showing 2/3 results (filtered)");
}

#[test]
fn test_port_table_clear_filter() {
    let mut widget = PortTableWidget::new();

    widget.add_result(create_test_result(80, PortState::Open, None));
    widget.add_result(create_test_result(22, PortState::Closed, None));

    // Apply filter
    let filter = LiveFilter::new().with_states(vec![PortState::Open]);
    widget.set_filter(filter);
    assert_eq!(widget.filtered_results.len(), 1);

    // Clear filter
    widget.clear_filter();
    assert_eq!(widget.filtered_results.len(), 2);
}
```

**Test Command:**
```bash
cargo test -p prtip-tui port_table::test_port_table_filtering -- --nocapture
```

**Acceptance Criteria:**
- [ ] 2/2 tests passing
- [ ] Filter applies to existing results
- [ ] New results filtered in real-time
- [ ] Filter status displayed ("X/Y filtered")

---

### TASK AREA 5: Performance Dashboard (2-3 hours)

**Purpose:** Real-time CPU/memory/network metrics visualization in TUI

#### Task 5.1: Create MetricsCollector (1 hour)

**File:** `crates/prtip-core/src/metrics/collector.rs` (new)

**Implementation:**
```rust
//! System metrics collection using sysinfo

use sysinfo::{System, SystemExt, ProcessExt, Pid};
use std::time::Duration;
use tokio::time::interval;

/// System metrics snapshot
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_rss: u64,      // Bytes
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub timestamp: std::time::Instant,
}

/// Metrics collector
pub struct MetricsCollector {
    system: System,
    pid: Pid,
    interval_secs: u64,
}

impl MetricsCollector {
    /// Create new collector
    pub fn new(interval_secs: u64) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = Pid::from(std::process::id() as usize);

        Self {
            system,
            pid,
            interval_secs,
        }
    }

    /// Collect current metrics
    pub fn collect(&mut self) -> SystemMetrics {
        self.system.refresh_all();

        let process = self.system.process(self.pid).expect("Process not found");

        SystemMetrics {
            cpu_percent: process.cpu_usage() as f64,
            memory_rss: process.memory(),
            bytes_sent: 0,  // TODO: network stats
            bytes_received: 0,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Start collecting metrics in background
    pub async fn start_collecting<F>(mut self, mut callback: F)
    where
        F: FnMut(SystemMetrics) + Send + 'static,
    {
        let mut ticker = interval(Duration::from_secs(self.interval_secs));

        loop {
            ticker.tick().await;
            let metrics = self.collect();
            callback(metrics);
        }
    }
}
```

**TDD Test Case** (`crates/prtip-core/src/metrics/collector.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut collector = MetricsCollector::new(1);

        let metrics = collector.collect();

        assert!(metrics.cpu_percent >= 0.0);
        assert!(metrics.memory_rss > 0);
    }

    #[tokio::test]
    async fn test_continuous_collection() {
        use std::sync::{Arc, Mutex};

        let collector = MetricsCollector::new(1);
        let samples = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = samples.clone();

        let handle = tokio::spawn(async move {
            collector.start_collecting(move |metrics| {
                samples_clone.lock().unwrap().push(metrics);
            }).await;
        });

        // Wait for a few samples
        tokio::time::sleep(Duration::from_secs(3)).await;

        let collected = samples.lock().unwrap();
        assert!(collected.len() >= 2);

        handle.abort();
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-core metrics::collector -- --nocapture
```

**Acceptance Criteria:**
- [ ] 2/2 tests passing
- [ ] CPU usage collected (>= 0%)
- [ ] Memory RSS collected (> 0 bytes)
- [ ] Background collection works

---

#### Task 5.2: Create PerformanceWidget (1.5 hours)

**File:** `crates/prtip-tui/src/widgets/performance.rs` (new)

**Implementation:**
```rust
//! Performance metrics dashboard widget

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Chart, Dataset, GraphType, Axis},
};
use std::collections::VecDeque;
use prtip_core::metrics::SystemMetrics;

const MAX_SAMPLES: usize = 60;  // Keep last 60 samples (1 minute at 1Hz)

/// Performance dashboard widget
pub struct PerformanceWidget {
    cpu_usage: VecDeque<(f64, f64)>,      // (time, %)
    memory_usage: VecDeque<(f64, f64)>,   // (time, MB)
    network_io: VecDeque<(f64, f64)>,     // (time, bytes/sec)
    current_time: f64,
}

impl PerformanceWidget {
    pub fn new() -> Self {
        Self {
            cpu_usage: VecDeque::new(),
            memory_usage: VecDeque::new(),
            network_io: VecDeque::new(),
            current_time: 0.0,
        }
    }

    /// Update with new metrics
    pub fn update(&mut self, metrics: &SystemMetrics) {
        self.current_time += 1.0;

        // Add samples
        self.cpu_usage.push_back((self.current_time, metrics.cpu_percent));
        self.memory_usage.push_back((self.current_time, metrics.memory_rss as f64 / 1_048_576.0)); // Convert to MB
        self.network_io.push_back((self.current_time, (metrics.bytes_sent + metrics.bytes_received) as f64));

        // Keep only last MAX_SAMPLES
        if self.cpu_usage.len() > MAX_SAMPLES {
            self.cpu_usage.pop_front();
            self.memory_usage.pop_front();
            self.network_io.pop_front();
        }
    }
}

impl ratatui::widgets::Widget for &PerformanceWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split into 3 horizontal sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // CPU chart
        let cpu_data: Vec<(f64, f64)> = self.cpu_usage.iter().cloned().collect();
        let cpu_dataset = vec![
            Dataset::default()
                .name("CPU %")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&cpu_data)
        ];

        let cpu_chart = Chart::new(cpu_dataset)
            .block(Block::default().title("CPU Usage").borders(Borders::ALL))
            .x_axis(Axis::default().bounds([self.current_time - 60.0, self.current_time]))
            .y_axis(Axis::default().bounds([0.0, 100.0]));

        cpu_chart.render(chunks[0], buf);

        // Memory chart
        let mem_data: Vec<(f64, f64)> = self.memory_usage.iter().cloned().collect();
        let mem_dataset = vec![
            Dataset::default()
                .name("Memory MB")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&mem_data)
        ];

        let max_mem = mem_data.iter().map(|(_, y)| y).fold(0.0, |a, &b| a.max(b)) * 1.2; // 20% headroom
        let mem_chart = Chart::new(mem_dataset)
            .block(Block::default().title("Memory Usage (MB)").borders(Borders::ALL))
            .x_axis(Axis::default().bounds([self.current_time - 60.0, self.current_time]))
            .y_axis(Axis::default().bounds([0.0, max_mem]));

        mem_chart.render(chunks[1], buf);

        // Network chart
        let net_data: Vec<(f64, f64)> = self.network_io.iter().cloned().collect();
        let net_dataset = vec![
            Dataset::default()
                .name("Network I/O")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&net_data)
        ];

        let max_io = net_data.iter().map(|(_, y)| y).fold(1.0, |a, &b| a.max(b)) * 1.2;
        let net_chart = Chart::new(net_dataset)
            .block(Block::default().title("Network I/O (bytes/sec)").borders(Borders::ALL))
            .x_axis(Axis::default().bounds([self.current_time - 60.0, self.current_time]))
            .y_axis(Axis::default().bounds([0.0, max_io]));

        net_chart.render(chunks[2], buf);
    }
}
```

**TDD Test Case** (`crates/prtip-tui/src/widgets/performance.rs` - bottom of file):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics() -> SystemMetrics {
        SystemMetrics {
            cpu_percent: 42.5,
            memory_rss: 1024 * 1024 * 128,  // 128 MB
            bytes_sent: 1000,
            bytes_received: 2000,
            timestamp: std::time::Instant::now(),
        }
    }

    #[test]
    fn test_performance_widget_update() {
        let mut widget = PerformanceWidget::new();

        let metrics = create_test_metrics();
        widget.update(&metrics);

        assert_eq!(widget.cpu_usage.len(), 1);
        assert_eq!(widget.memory_usage.len(), 1);
        assert_eq!(widget.network_io.len(), 1);
    }

    #[test]
    fn test_performance_widget_max_samples() {
        let mut widget = PerformanceWidget::new();

        // Add more than MAX_SAMPLES
        for _ in 0..70 {
            widget.update(&create_test_metrics());
        }

        assert_eq!(widget.cpu_usage.len(), MAX_SAMPLES);
        assert_eq!(widget.memory_usage.len(), MAX_SAMPLES);
        assert_eq!(widget.network_io.len(), MAX_SAMPLES);
    }

    #[test]
    fn test_performance_widget_memory_conversion() {
        let mut widget = PerformanceWidget::new();

        let metrics = SystemMetrics {
            cpu_percent: 0.0,
            memory_rss: 1024 * 1024 * 100,  // 100 MB
            bytes_sent: 0,
            bytes_received: 0,
            timestamp: std::time::Instant::now(),
        };

        widget.update(&metrics);

        let (_, mb) = widget.memory_usage[0];
        assert!((mb - 100.0).abs() < 0.1);  // ~100 MB
    }
}
```

**Test Command:**
```bash
cargo test -p prtip-tui performance -- --nocapture
```

**Acceptance Criteria:**
- [ ] 3/3 tests passing
- [ ] Ring buffer keeps last 60 samples
- [ ] Memory converted to MB correctly
- [ ] Charts render without panic

---

#### Task 5.3: Integrate Performance Dashboard into TUI (30 min)

**File:** `crates/prtip-tui/src/app.rs` (modify)

**Implementation:**
```rust
use crate::widgets::performance::PerformanceWidget;
use prtip_core::metrics::{MetricsCollector, SystemMetrics};

pub struct App {
    // ... existing fields ...
    performance_widget: PerformanceWidget,
    show_performance: bool,
}

impl App {
    pub async fn run(&mut self) -> io::Result<()> {
        // Start metrics collection
        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            let collector = MetricsCollector::new(1);
            collector.start_collecting(move |metrics| {
                event_bus.publish(Event::PerformanceMetrics(metrics));
            }).await;
        });

        // ... existing run logic ...
    }

    pub fn handle_performance_metrics(&mut self, metrics: SystemMetrics) {
        self.performance_widget.update(&metrics);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            // ... existing handlers ...

            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.show_performance = !self.show_performance;
            },

            _ => {}
        }
        Ok(())
    }
}
```

**Test Command:**
```bash
cargo build --release
cargo run --release -- -sS -p 80 192.168.1.1
# Press 'm' to toggle performance dashboard
```

**Acceptance Criteria:**
- [ ] 'm' key toggles performance dashboard
- [ ] Metrics update every 1 second
- [ ] CPU/memory/network graphs visible
- [ ] No performance regression (TUI still 60 FPS)

---

## Definition of Done

### Functional Requirements

- [ ] **Memory-Mapped I/O**
  - [ ] MmapResultWriter writes scan results to disk
  - [ ] MmapResultReader reads results zero-copy
  - [ ] 20-50% memory reduction verified (benchmark)
  - [ ] CLI flag `--use-mmap` works

- [ ] **Scan History & Resume**
  - [ ] SQLite database stores scan history
  - [ ] Pause/resume preserves scanner state
  - [ ] TUI 'p' key pauses, 'r' key resumes
  - [ ] CLI `prtip history list/resume` commands work

- [ ] **Export Enhancements**
  - [ ] HTML report generation works (Bootstrap template)
  - [ ] PDF export works (optional, can defer)
  - [ ] TUI export menu displays on 'e' key
  - [ ] All 6 formats selectable (Text/JSON/XML/CSV/HTML/PDF)

- [ ] **Real-Time Filtering**
  - [ ] LiveFilter matches port/state/service criteria
  - [ ] PortTableWidget applies filter in real-time
  - [ ] Filter status displayed ("X/Y filtered")
  - [ ] TUI 'f' key opens filter builder (optional)

- [ ] **Performance Dashboard**
  - [ ] MetricsCollector polls CPU/memory every 1s
  - [ ] PerformanceWidget displays 3 charts (CPU/memory/network)
  - [ ] TUI 'm' key toggles dashboard
  - [ ] No TUI performance regression

### Quality Requirements

- [ ] **Tests**: 43-51 new tests passing (100% success rate)
  - [ ] Mmap: 8 tests (writer, reader, integration)
  - [ ] Scan History: 10 tests (database, pause/resume)
  - [ ] Export: 6 tests (HTML, PDF)
  - [ ] Filtering: 7 tests (LiveFilter, PortTableWidget)
  - [ ] Performance: 5 tests (MetricsCollector, PerformanceWidget)
  - [ ] Integration: 13 tests

- [ ] **Code Quality**
  - [ ] Zero clippy warnings (`cargo clippy --all-targets -- -D warnings`)
  - [ ] Zero rustdoc warnings
  - [ ] Formatted with `cargo fmt`
  - [ ] All public APIs documented

- [ ] **Performance**
  - [ ] Memory reduction: 20-50% (mmap vs standard)
  - [ ] Export time (10K results): CSV <5s, HTML <10s
  - [ ] Checkpoint save: <500ms
  - [ ] Resume time: <1s
  - [ ] Filter latency: <50ms

### Documentation Requirements

- [ ] **Guide**: `docs/30-ADVANCED-FEATURES-GUIDE.md` complete (1,500-2,000 lines)
  - [ ] Memory-mapped I/O usage examples
  - [ ] Pause/resume workflow
  - [ ] Export format comparisons
  - [ ] Real-time filtering syntax
  - [ ] Performance dashboard interpretation

- [ ] **Rustdoc**: All public APIs documented
- [ ] **Examples**: Export format examples in `docs/examples/`
- [ ] **CHANGELOG**: Sprint 6.6 section added

### Sprint Completion Checklist

- [ ] All tasks completed (or explicitly deferred)
- [ ] Test suite passing (2,246+ tests)
- [ ] Benchmarks run and documented
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped to v0.5.6
- [ ] Completion report written to `/tmp/ProRT-IP/SPRINT-6.6-COMPLETE.md`
- [ ] Git commit with detailed message
- [ ] GitHub release created (optional)

---

## Notes

1. **PDF Export Optional**: If time-constrained, defer PDF to Sprint 6.7. HTML is higher priority.
2. **Network Stats Stub**: `bytes_sent/bytes_received` in MetricsCollector currently stubbed (0). Can be implemented in Sprint 6.7 using netstat or proc filesystem.
3. **Filter Builder UI**: TUI filter builder ('f' key) is optional. CLI filtering more important.
4. **Incremental Testing**: Run tests after each task area (don't wait until end).
5. **TDD Discipline**: Write failing test FIRST, then implement, then verify pass.

---

**This TODO follows the ProRT-IP project methodology: Test-Driven Development, systematic quality gates, comprehensive documentation, and strategic feature prioritization.**
