//! Memory-mapped result reader for zero-copy access to scan results

use memmap2::Mmap;
use prtip_core::{ScanResult, ScanResultRkyv};
use std::fs::File;
use std::io;
use std::path::Path;

const HEADER_SIZE: usize = 64;
const ENTRY_SIZE: usize = 512;
const LENGTH_PREFIX_SIZE: usize = 8; // u64 length prefix for each entry

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
        if version == 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incompatible file format: version 1 (bincode) is no longer supported. \
                 This file was created with an older version of the scanner. \
                 Please regenerate scan results with the current version (rkyv format, version 2).",
            ));
        }
        if version != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported version: {}. Expected version 2 (rkyv format).", version),
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

        // Read length prefix (u64 in little-endian)
        let len = u64::from_le_bytes(
            entry_bytes[..LENGTH_PREFIX_SIZE]
                .try_into()
                .expect("LENGTH_PREFIX_SIZE is 8 bytes"),
        ) as usize;

        // Validate length
        if len == 0 || len + LENGTH_PREFIX_SIZE > self.entry_size {
            eprintln!(
                "MmapResultReader: invalid entry length {} at index {}",
                len, index
            );
            return None;
        }

        // Use zero-copy deserialization without unnecessary allocation
        let data_bytes = &entry_bytes[LENGTH_PREFIX_SIZE..LENGTH_PREFIX_SIZE + len];
        match rkyv::from_bytes::<ScanResultRkyv, rkyv::rancor::Error>(data_bytes) {
            Ok(rkyv_result) => Some(ScanResult::from(rkyv_result)),
            Err(e) => {
                eprintln!(
                    "MmapResultReader: failed to deserialize entry at index {} with length {}: {}",
                    index, len, e
                );
                None
            }
        }
    }

    /// Create an iterator over all entries
    pub fn iter(&self) -> MmapResultIterator<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::mmap_writer::MmapResultWriter;
    use chrono::Utc;
    use prtip_core::{PortState, ScanResult};
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

    #[test]
    fn test_mmap_version_1_rejected() {
        use std::io::Write;

        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        // Create a file with version 1 header (old bincode format)
        {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .unwrap();

            // Write a version 1 header
            file.write_all(&1u64.to_le_bytes()).unwrap(); // version = 1
            file.write_all(&0u64.to_le_bytes()).unwrap(); // entry_count = 0
            file.write_all(&(ENTRY_SIZE as u64).to_le_bytes())
                .unwrap(); // entry_size
            file.write_all(&0u64.to_le_bytes()).unwrap(); // checksum
            // Pad to HEADER_SIZE
            file.write_all(&vec![0u8; HEADER_SIZE - 32]).unwrap();
        }

        // Attempt to open should fail with clear error message
        let result = MmapResultReader::open(&path);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("version 1") && err_msg.contains("bincode"),
                "Error message should mention version 1 and bincode format: {}",
                err_msg
            );
        }
    }
}
