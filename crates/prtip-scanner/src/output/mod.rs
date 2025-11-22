//! Output formatters and exporters for scan results
//!
//! This module provides various output formats for ProRT-IP scan results:
//! - Memory-mapped binary output (efficient for large scans)
//! - Standard file formats (JSON, XML, CSV)
//! - ResultWriter enum for unified mmap/in-memory abstraction (Sprint 6.6)

pub mod mmap_reader;
pub mod mmap_writer;

pub use mmap_reader::MmapResultReader;
pub use mmap_writer::MmapResultWriter;

use prtip_core::{Result, ScanResult};
use std::path::PathBuf;

/// Unified result writer abstraction
///
/// Provides a common interface for writing scan results to either
/// memory-mapped files (for large scans) or in-memory vectors (for small scans).
///
/// # Sprint 6.6 Task Area 2: Scanner Integration
///
/// This enum allows scanners to seamlessly switch between storage backends
/// based on configuration (`--use-mmap` flag) without changing scanner code.
///
/// # Usage
///
/// ```no_run
/// use prtip_scanner::output::ResultWriter;
/// use prtip_core::{ScanResult, PortState};
///
/// # async fn example() -> prtip_core::Result<()> {
/// // Memory-backed (default)
/// let mut writer = ResultWriter::new_memory();
///
/// // Mmap-backed (for large scans)
/// let mut writer = ResultWriter::new_mmap("/tmp/results.mmap", 10000)?;
///
/// // Write results (same interface)
/// let result = ScanResult::new("192.168.1.1".parse().unwrap(), 80, PortState::Open);
/// writer.write(&result)?;
///
/// // Flush when done
/// writer.flush()?;
///
/// // Collect results
/// let results = writer.collect()?;
/// # Ok(())
/// # }
/// ```
pub enum ResultWriter {
    /// In-memory storage (default, backward compatible)
    Memory(Vec<ScanResult>),
    /// Memory-mapped file storage (opt-in via --use-mmap)
    Mmap {
        writer: MmapResultWriter,
        path: PathBuf,
    },
}

impl ResultWriter {
    /// Create a new in-memory result writer (default)
    pub fn new_memory() -> Self {
        Self::Memory(Vec::new())
    }

    /// Create a new memory-mapped result writer
    ///
    /// # Arguments
    ///
    /// * `path` - Path to mmap file
    /// * `initial_capacity` - Initial capacity (number of entries)
    ///
    /// # Returns
    ///
    /// Result containing the writer or an error
    pub fn new_mmap<P: Into<PathBuf>>(path: P, initial_capacity: usize) -> Result<Self> {
        let path_buf = path.into();
        let writer =
            MmapResultWriter::new(&path_buf, initial_capacity).map_err(prtip_core::Error::Io)?;

        Ok(Self::Mmap {
            writer,
            path: path_buf,
        })
    }

    /// Create a ResultWriter from configuration
    ///
    /// Checks `config.output.use_mmap` and creates either an in-memory
    /// or memory-mapped writer accordingly.
    ///
    /// # Arguments
    ///
    /// * `config` - Scan configuration
    /// * `estimated_results` - Estimated number of results (for mmap capacity)
    ///
    /// # Returns
    ///
    /// Result containing the appropriate writer
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_scanner::output::ResultWriter;
    /// use prtip_core::Config;
    ///
    /// # async fn example() -> prtip_core::Result<()> {
    /// let config = Config::default();
    /// let writer = ResultWriter::from_config(&config, 1000)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config(config: &prtip_core::Config, estimated_results: usize) -> Result<Self> {
        if config.output.use_mmap {
            // Mmap mode: use configured path or generate default
            let path = config.output.mmap_output_path.clone().unwrap_or_else(|| {
                // Generate default path in temp directory
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                PathBuf::from(format!("/tmp/prtip_scan_{}.mmap", timestamp))
            });

            Self::new_mmap(path, estimated_results)
        } else {
            // Default: in-memory mode
            Ok(Self::new_memory())
        }
    }

    /// Write a scan result
    ///
    /// # Arguments
    ///
    /// * `result` - Scan result to write
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn write(&mut self, result: &ScanResult) -> Result<()> {
        match self {
            Self::Memory(vec) => {
                vec.push(result.clone());
                Ok(())
            }
            Self::Mmap { writer, .. } => writer.write_entry(result).map_err(prtip_core::Error::Io),
        }
    }

    /// Flush buffered data to disk (no-op for in-memory)
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn flush(&mut self) -> Result<()> {
        match self {
            Self::Memory(_) => Ok(()), // No-op for in-memory
            Self::Mmap { writer, .. } => writer.flush().map_err(prtip_core::Error::Io),
        }
    }

    /// Collect all results
    ///
    /// For in-memory storage, returns a clone of the vector.
    /// For mmap storage, reads all entries from the file.
    ///
    /// # Returns
    ///
    /// Vector of all scan results
    pub fn collect(&self) -> Result<Vec<ScanResult>> {
        match self {
            Self::Memory(vec) => Ok(vec.clone()),
            Self::Mmap { path, .. } => {
                // Open reader and collect all entries
                let reader = MmapResultReader::open(path).map_err(prtip_core::Error::Io)?;
                Ok(reader.iter().collect())
            }
        }
    }

    /// Get the number of results written
    ///
    /// # Returns
    ///
    /// Number of results
    pub fn len(&self) -> usize {
        match self {
            Self::Memory(vec) => vec.len(),
            Self::Mmap { path, .. } => {
                // Open reader and get count
                MmapResultReader::open(path).map(|r| r.len()).unwrap_or(0)
            }
        }
    }

    /// Check if empty
    ///
    /// # Returns
    ///
    /// True if no results have been written
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the underlying path for mmap storage
    ///
    /// # Returns
    ///
    /// Some(path) for mmap storage, None for in-memory
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Memory(_) => None,
            Self::Mmap { path, .. } => Some(path),
        }
    }
}

impl Drop for ResultWriter {
    fn drop(&mut self) {
        // Ensure mmap data is flushed on drop
        if let Self::Mmap { writer, .. } = self {
            let _ = writer.flush();
        }
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
    fn test_memory_writer() {
        let mut writer = ResultWriter::new_memory();

        // Write results
        writer.write(&create_test_result(80)).unwrap();
        writer.write(&create_test_result(443)).unwrap();

        assert_eq!(writer.len(), 2);
        assert!(!writer.is_empty());

        // Collect results
        let results = writer.collect().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].port, 80);
        assert_eq!(results[1].port, 443);
    }

    #[test]
    fn test_mmap_writer() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        let mut writer = ResultWriter::new_mmap(&path, 10).unwrap();

        // Write results
        writer.write(&create_test_result(80)).unwrap();
        writer.write(&create_test_result(443)).unwrap();
        writer.flush().unwrap();

        assert_eq!(writer.len(), 2);
        assert!(!writer.is_empty());
        assert_eq!(writer.path(), Some(&path));

        // Collect results
        let results = writer.collect().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].port, 80);
        assert_eq!(results[1].port, 443);
    }

    #[test]
    fn test_mmap_writer_auto_grow() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        // Start with small capacity
        let mut writer = ResultWriter::new_mmap(&path, 2).unwrap();

        // Write beyond initial capacity
        for port in 80..90 {
            writer.write(&create_test_result(port)).unwrap();
        }
        writer.flush().unwrap();

        assert_eq!(writer.len(), 10);

        // Verify all results
        let results = writer.collect().unwrap();
        assert_eq!(results.len(), 10);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.port, 80 + i as u16);
        }
    }

    #[test]
    fn test_memory_writer_empty() {
        let writer = ResultWriter::new_memory();
        assert_eq!(writer.len(), 0);
        assert!(writer.is_empty());
        assert_eq!(writer.path(), None);

        let results = writer.collect().unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_mmap_writer_empty() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        let mut writer = ResultWriter::new_mmap(&path, 10).unwrap();
        writer.flush().unwrap();

        assert_eq!(writer.len(), 0);
        assert!(writer.is_empty());
        assert_eq!(writer.path(), Some(&path));

        let results = writer.collect().unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_from_config_memory_mode() {
        use prtip_core::Config;

        let config = Config::default();
        assert!(!config.output.use_mmap); // Default is false

        let writer = ResultWriter::from_config(&config, 100).unwrap();

        // Should create in-memory writer
        assert!(matches!(writer, ResultWriter::Memory(_)));
        assert_eq!(writer.path(), None);
    }

    #[test]
    fn test_from_config_mmap_mode() {
        use prtip_core::{Config, OutputConfig};

        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        let mut config = Config::default();
        config.output = OutputConfig {
            use_mmap: true,
            mmap_output_path: Some(path.clone()),
            ..Default::default()
        };

        let writer = ResultWriter::from_config(&config, 100).unwrap();

        // Should create mmap writer
        assert!(matches!(writer, ResultWriter::Mmap { .. }));
        assert_eq!(writer.path(), Some(&path));
    }

    #[test]
    fn test_from_config_mmap_default_path() {
        use prtip_core::{Config, OutputConfig};

        let mut config = Config::default();
        config.output = OutputConfig {
            use_mmap: true,
            mmap_output_path: None, // Use default path
            ..Default::default()
        };

        let writer = ResultWriter::from_config(&config, 100).unwrap();

        // Should create mmap writer with auto-generated path
        assert!(matches!(writer, ResultWriter::Mmap { .. }));
        assert!(writer.path().is_some());

        // Path should be in /tmp and contain timestamp
        let path_str = writer.path().unwrap().to_string_lossy();
        assert!(path_str.starts_with("/tmp/prtip_scan_"));
        assert!(path_str.ends_with(".mmap"));
    }
}
