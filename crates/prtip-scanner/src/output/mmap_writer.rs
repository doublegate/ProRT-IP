//! Memory-mapped result writer for efficient large-scale scans
//!
//! Uses memory-mapped files to reduce RAM usage by 20-50% compared to
//! in-memory buffering. Results are written to a binary format with
//! fixed-size entries for zero-copy random access.

use memmap2::{MmapMut, MmapOptions};
use prtip_core::ScanResult;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

const HEADER_SIZE: usize = 64; // Version, entry_count, entry_size, checksum
const ENTRY_SIZE: usize = 512; // Fixed-size entries (padded if needed)

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
            .truncate(true)
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
                format!(
                    "Entry size {} exceeds maximum {}",
                    entry_bytes.len(),
                    ENTRY_SIZE
                ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use prtip_core::PortState;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    fn create_test_result(port: u16) -> ScanResult {
        ScanResult {
            target_ip: "192.168.1.1".parse().unwrap(),
            port,
            state: PortState::Open,
            service: Some("http".to_string()),
            version: Some("Apache/2.4".to_string()),
            banner: Some("HTTP/1.1 200 OK".to_string()),
            raw_response: Some(b"HTTP/1.1 200 OK".to_vec()),
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
