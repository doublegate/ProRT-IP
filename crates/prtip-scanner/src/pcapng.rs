//! PCAPNG packet capture output module
//!
//! Provides packet capture functionality compatible with Wireshark/tshark.
//! Captures both sent packets (probes) and received responses for forensic analysis.

use anyhow::{Context, Result};
use pcap_file::pcapng::{
    blocks::{
        enhanced_packet::EnhancedPacketBlock, interface_description::InterfaceDescriptionBlock,
        section_header::SectionHeaderBlock,
    },
    Block, PcapNgWriter,
};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Maximum file size before rotation (1GB)
const MAX_FILE_SIZE: u64 = 1_000_000_000;

/// Packet direction for metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Packet sent from scanner
    Sent,
    /// Packet received by scanner
    Received,
}

/// PCAPNG packet writer with automatic file rotation
///
/// Thread-safe writer that captures network packets to PCAPNG format.
/// Automatically rotates files when they exceed 1GB to prevent single large files.
pub struct PcapngWriter {
    writer: Arc<Mutex<PcapNgWriter<BufWriter<File>>>>,
    base_path: PathBuf,
    current_file_size: Arc<AtomicU64>,
    file_index: Arc<AtomicU32>,
    max_file_size: u64,
}

impl PcapngWriter {
    /// Create a new PCAPNG writer
    ///
    /// # Arguments
    ///
    /// * `path` - Base path for PCAPNG files (e.g., "scan.pcapng" becomes "scan-001.pcapng")
    ///
    /// # Returns
    ///
    /// A thread-safe PCAPNG writer
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        let file_index = Arc::new(AtomicU32::new(1));
        let current_file_size = Arc::new(AtomicU64::new(0));

        let writer = Self::create_writer(&base_path, &file_index, &current_file_size)?;

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            base_path,
            current_file_size,
            file_index,
            max_file_size: MAX_FILE_SIZE,
        })
    }

    /// Create a new PCAPNG writer file
    fn create_writer(
        base_path: &Path,
        file_index: &Arc<AtomicU32>,
        current_file_size: &Arc<AtomicU64>,
    ) -> Result<PcapNgWriter<BufWriter<File>>> {
        let index = file_index.load(Ordering::SeqCst);
        let file_path = Self::get_indexed_path(base_path, index);

        debug!("Creating PCAPNG file: {}", file_path.display());

        let file = File::create(&file_path)
            .with_context(|| format!("Failed to create PCAPNG file: {}", file_path.display()))?;
        let buf_writer = BufWriter::with_capacity(8192, file);
        let mut writer = PcapNgWriter::new(buf_writer).context("Failed to create PCAPNG writer")?;

        // Write Section Header Block (SHB)
        let shb = SectionHeaderBlock {
            endianness: pcap_file::Endianness::Big,
            major_version: 1,
            minor_version: 0,
            section_length: -1, // Unknown length
            options: vec![],
        };
        writer
            .write_block(&Block::SectionHeader(shb))
            .context("Failed to write Section Header Block")?;

        // Write Interface Description Block (IDB)
        let idb = InterfaceDescriptionBlock {
            linktype: pcap_file::DataLink::ETHERNET,
            snaplen: 65535, // Capture full packets
            options: vec![],
        };
        writer
            .write_block(&Block::InterfaceDescription(idb))
            .context("Failed to write Interface Description Block")?;

        // Reset file size counter
        current_file_size.store(0, Ordering::SeqCst);

        Ok(writer)
    }

    /// Get indexed file path (e.g., "scan.pcapng" -> "scan-001.pcapng")
    fn get_indexed_path(base_path: &Path, index: u32) -> PathBuf {
        let stem = base_path.file_stem().unwrap_or_default();
        let extension = base_path.extension().unwrap_or_default();
        let parent = base_path.parent().unwrap_or_else(|| Path::new("."));

        parent.join(format!(
            "{}-{:03}.{}",
            stem.to_string_lossy(),
            index,
            extension.to_string_lossy()
        ))
    }

    /// Write a packet to the PCAPNG file
    ///
    /// # Arguments
    ///
    /// * `packet_data` - Raw packet bytes (Ethernet frame)
    /// * `direction` - Whether packet was sent or received
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if write fails
    pub fn write_packet(&self, packet_data: &[u8], direction: Direction) -> Result<()> {
        // Check if rotation is needed
        let current_size = self.current_file_size.load(Ordering::SeqCst);
        if current_size >= self.max_file_size {
            self.rotate_file()?;
        }

        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        // Create Enhanced Packet Block
        let epb = EnhancedPacketBlock {
            interface_id: 0,
            timestamp,
            original_len: packet_data.len() as u32,
            data: packet_data.into(),
            options: vec![],
        };

        // Write block
        let mut writer = self.writer.lock().unwrap();
        writer
            .write_block(&Block::EnhancedPacket(epb))
            .context("Failed to write Enhanced Packet Block")?;

        // Update file size estimate (block header + data + padding)
        let block_size = 32 + packet_data.len() + (4 - (packet_data.len() % 4)) % 4;
        self.current_file_size
            .fetch_add(block_size as u64, Ordering::SeqCst);

        debug!(
            "Wrote {} byte packet ({:?}) to PCAPNG",
            packet_data.len(),
            direction
        );

        Ok(())
    }

    /// Rotate to a new file
    fn rotate_file(&self) -> Result<()> {
        let new_index = self.file_index.fetch_add(1, Ordering::SeqCst) + 1;
        debug!("Rotating PCAPNG file to index {}", new_index);

        let new_writer =
            Self::create_writer(&self.base_path, &self.file_index, &self.current_file_size)?;

        let mut writer = self.writer.lock().unwrap();
        *writer = new_writer;

        Ok(())
    }

    /// Flush buffered data to disk
    pub fn flush(&self) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer
            .get_mut()
            .flush()
            .context("Failed to flush PCAPNG writer")?;
        Ok(())
    }

    /// Get the current file index
    pub fn current_index(&self) -> u32 {
        self.file_index.load(Ordering::SeqCst)
    }

    /// Get the current file size
    pub fn current_size(&self) -> u64 {
        self.current_file_size.load(Ordering::SeqCst)
    }
}

impl Drop for PcapngWriter {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            warn!("Failed to flush PCAPNG writer on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_create_pcapng_writer() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pcapng");

        let writer = PcapngWriter::new(&path);
        assert!(writer.is_ok());

        // Check that indexed file was created
        let expected_path = dir.path().join("test-001.pcapng");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_get_indexed_path() {
        let base = PathBuf::from("/tmp/scan.pcapng");
        let indexed = PcapngWriter::get_indexed_path(&base, 1);
        assert_eq!(indexed, PathBuf::from("/tmp/scan-001.pcapng"));

        let indexed = PcapngWriter::get_indexed_path(&base, 42);
        assert_eq!(indexed, PathBuf::from("/tmp/scan-042.pcapng"));
    }

    #[test]
    fn test_write_packet() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pcapng");
        let writer = PcapngWriter::new(&path).unwrap();

        // Create a minimal Ethernet frame (64 bytes minimum)
        let packet = vec![0u8; 64];

        let result = writer.write_packet(&packet, Direction::Sent);
        assert!(result.is_ok());

        // Verify file size increased
        assert!(writer.current_size() > 0);
    }

    #[test]
    fn test_write_multiple_packets() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pcapng");
        let writer = PcapngWriter::new(&path).unwrap();

        // Write 10 packets
        for i in 0..10 {
            let packet = vec![i as u8; 64];
            let direction = if i % 2 == 0 {
                Direction::Sent
            } else {
                Direction::Received
            };
            writer.write_packet(&packet, direction).unwrap();
        }

        // Verify file size increased
        assert!(writer.current_size() > 640); // At least 10 * 64 bytes
    }

    #[test]
    fn test_file_rotation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pcapng");

        // Create writer with small max file size for testing
        let mut writer = PcapngWriter::new(&path).unwrap();
        writer.max_file_size = 1024; // 1KB for testing

        // Write packets until rotation
        let packet = vec![0u8; 200];
        for _ in 0..10 {
            writer.write_packet(&packet, Direction::Sent).unwrap();
        }

        // Should have rotated to at least file 2
        assert!(writer.current_index() >= 2);

        // Verify both files exist
        assert!(dir.path().join("test-001.pcapng").exists());
        assert!(dir.path().join("test-002.pcapng").exists());
    }

    #[test]
    fn test_flush() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pcapng");
        let writer = PcapngWriter::new(&path).unwrap();

        let packet = vec![0u8; 64];
        writer.write_packet(&packet, Direction::Sent).unwrap();

        // Flush should succeed
        assert!(writer.flush().is_ok());

        // File should have data
        let file_path = dir.path().join("test-001.pcapng");
        let mut file = File::open(file_path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        assert!(!contents.is_empty());
    }

    #[test]
    fn test_direction_enum() {
        assert_eq!(Direction::Sent, Direction::Sent);
        assert_eq!(Direction::Received, Direction::Received);
        assert_ne!(Direction::Sent, Direction::Received);
    }

    #[test]
    fn test_concurrent_writes() {
        use std::thread;

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.pcapng");
        let writer = Arc::new(PcapngWriter::new(&path).unwrap());

        let mut handles = vec![];
        for i in 0..5 {
            let writer_clone = Arc::clone(&writer);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let packet = vec![(i * 10 + j) as u8; 64];
                    writer_clone.write_packet(&packet, Direction::Sent).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify 50 packets were written
        assert!(writer.current_size() > 3200); // At least 50 * 64 bytes
    }
}
