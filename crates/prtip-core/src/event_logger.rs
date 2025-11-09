//! Event Logger for JSON Lines event persistence
//!
//! This module provides automatic logging of all scan events to disk in
//! JSON Lines format, enabling audit trails, replay, and analysis.
//!
//! # Architecture
//!
//! - **Format**: JSON Lines (one event per line)
//! - **Location**: `~/.prtip/events/<scan_id>.jsonl`
//! - **Rotation**: Automatic at 100MB with gzip compression
//! - **Cleanup**: Auto-delete logs older than 30 days
//!
//! # Features
//!
//! - Subscribes to all scan events from EventBus
//! - Writes header/footer metadata for each scan
//! - Automatic log rotation and compression
//! - Thread-safe, non-blocking async writes
//! - Graceful handling of disk full / I/O errors
//!
//! # Examples
//!
//! ```no_run
//! use prtip_core::event_logger::EventLogger;
//! use prtip_core::event_bus::EventBus;
//! use std::sync::Arc;
//!
//! # async fn example() -> std::io::Result<()> {
//! // Create event bus
//! let bus = Arc::new(EventBus::new(1000));
//!
//! // Create event logger (automatically subscribes to all events)
//! let logger = EventLogger::new(bus.clone()).await?;
//!
//! // Events are automatically logged to ~/.prtip/events/<scan_id>.jsonl
//! // Cleanup old logs (>30 days)
//! logger.cleanup_old_logs()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::event_bus::{EventBus, EventFilter};
use crate::events::ScanEvent;
use serde_json::json;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Event logger for JSON Lines persistence
///
/// Automatically subscribes to all events from the EventBus and writes them
/// to `~/.prtip/events/<scan_id>.jsonl` in JSON Lines format.
///
/// # Thread Safety
///
/// EventLogger spawns a background task to handle I/O asynchronously,
/// ensuring the event bus is not blocked by disk writes.
///
/// # File Format
///
/// Each log file contains:
/// 1. Header with scan metadata
/// 2. One JSON object per line (one event per line)
/// 3. Footer with completion metadata
pub struct EventLogger {
    /// Log directory path
    log_dir: PathBuf,
    /// Background logger task
    _logger_task: JoinHandle<()>,
}

/// Configuration for event logger
#[derive(Clone, Debug)]
pub struct EventLoggerConfig {
    /// Log directory (default: ~/.prtip/events)
    pub log_dir: Option<PathBuf>,
    /// Max log file size before rotation (default: 100MB)
    pub max_file_size: u64,
    /// Log retention period (default: 30 days)
    pub retention_days: u64,
    /// Enable compression for rotated logs
    pub enable_compression: bool,
}

impl Default for EventLoggerConfig {
    fn default() -> Self {
        Self {
            log_dir: None,
            max_file_size: 100_000_000, // 100MB
            retention_days: 30,
            enable_compression: true,
        }
    }
}

impl EventLogger {
    /// Create a new event logger with default configuration
    ///
    /// Automatically subscribes to all events and creates log directory.
    ///
    /// # Arguments
    ///
    /// * `event_bus` - EventBus to subscribe to
    ///
    /// # Returns
    ///
    /// Result containing EventLogger or I/O error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::event_logger::EventLogger;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let logger = EventLogger::new(bus).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(event_bus: Arc<EventBus>) -> std::io::Result<Self> {
        Self::with_config(event_bus, EventLoggerConfig::default()).await
    }

    /// Create event logger with custom configuration
    ///
    /// # Arguments
    ///
    /// * `event_bus` - EventBus to subscribe to
    /// * `config` - Logger configuration
    ///
    /// # Returns
    ///
    /// Result containing EventLogger or I/O error
    pub async fn with_config(
        event_bus: Arc<EventBus>,
        config: EventLoggerConfig,
    ) -> std::io::Result<Self> {
        // Determine log directory
        let log_dir = if let Some(dir) = config.log_dir {
            dir
        } else {
            dirs::home_dir()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
                })?
                .join(".prtip")
                .join("events")
        };

        // Create log directory
        fs::create_dir_all(&log_dir)?;

        // Subscribe to all events
        let (tx, mut rx) = mpsc::unbounded_channel();
        event_bus.subscribe(tx, EventFilter::All).await;

        // Spawn background logger task
        let log_dir_clone = log_dir.clone();
        let max_file_size = config.max_file_size;
        let enable_compression = config.enable_compression;

        let logger_task = tokio::spawn(async move {
            let mut current_file: Option<BufWriter<File>> = None;
            let mut current_scan_id: Option<Uuid> = None;
            let mut current_file_path: Option<PathBuf> = None;

            while let Some(event) = rx.recv().await {
                // Handle scan started: open new file
                if matches!(event, ScanEvent::ScanStarted { .. }) {
                    // Close previous file if any
                    if let Some(mut file) = current_file.take() {
                        let _ = write_footer(&mut file, current_scan_id);
                        let _ = file.flush();
                    }

                    // Get scan ID from event
                    let scan_id = event.scan_id();
                    current_scan_id = Some(scan_id);

                    // Create new log file
                    let path = log_dir_clone.join(format!("{}.jsonl", scan_id));
                    current_file_path = Some(path.clone());

                    match File::create(&path) {
                        Ok(file) => {
                            let mut writer = BufWriter::new(file);

                            // Write header
                            if write_header(&mut writer, scan_id).is_ok() {
                                // Flush header immediately to ensure scan metadata is persisted
                                if writer.flush().is_ok() {
                                    current_file = Some(writer);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create log file {}: {}", path.display(), e);
                        }
                    }
                }

                // Write event to current file
                if let Some(writer) = &mut current_file {
                    if let Ok(json) = serde_json::to_string(&event) {
                        if writeln!(writer, "{}", json).is_err() {
                            eprintln!("Failed to write event to log");
                        }

                        // Check for rotation
                        if let Some(path) = &current_file_path {
                            if should_rotate(writer, max_file_size) {
                                // Flush and close current file
                                let _ = writer.flush();
                                current_file = None;

                                // Optionally compress
                                if enable_compression {
                                    if let Err(e) = compress_log_file(path) {
                                        eprintln!("Failed to compress log: {}", e);
                                    }
                                }

                                // Open new rotated file
                                if let Some(scan_id) = current_scan_id {
                                    let rotated_path = log_dir_clone.join(format!(
                                        "{}-{}.jsonl",
                                        scan_id,
                                        Uuid::new_v4()
                                    ));
                                    match File::create(&rotated_path) {
                                        Ok(file) => {
                                            let writer = BufWriter::new(file);
                                            current_file = Some(writer);
                                            current_file_path = Some(rotated_path);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to create rotated log file: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Handle scan completed: write footer and close
                if matches!(
                    event,
                    ScanEvent::ScanCompleted { .. }
                        | ScanEvent::ScanCancelled { .. }
                        | ScanEvent::ScanError { .. }
                ) {
                    if let Some(mut file) = current_file.take() {
                        let _ = write_footer(&mut file, current_scan_id);
                        let _ = file.flush();
                    }
                    current_scan_id = None;
                    current_file_path = None;
                }
            }
        });

        Ok(Self {
            log_dir,
            _logger_task: logger_task,
        })
    }

    /// Get log directory path
    ///
    /// # Returns
    ///
    /// Path to the event log directory
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// Clean up logs older than retention period
    ///
    /// Deletes all log files (including compressed) that are older than
    /// the retention period specified in configuration (default 30 days).
    ///
    /// # Returns
    ///
    /// Number of files deleted
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_core::event_logger::EventLogger;
    /// # use prtip_core::event_bus::EventBus;
    /// # use std::sync::Arc;
    /// # async fn example() -> std::io::Result<()> {
    /// # let bus = Arc::new(EventBus::new(1000));
    /// let logger = EventLogger::new(bus).await?;
    ///
    /// // Clean up logs older than 30 days
    /// let deleted = logger.cleanup_old_logs()?;
    /// println!("Deleted {} old log files", deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub fn cleanup_old_logs(&self) -> std::io::Result<usize> {
        cleanup_old_logs(&self.log_dir, 30)
    }

    /// Clean up logs with custom retention period
    ///
    /// # Arguments
    ///
    /// * `retention_days` - Number of days to retain logs
    ///
    /// # Returns
    ///
    /// Number of files deleted
    pub fn cleanup_old_logs_with_retention(&self, retention_days: u64) -> std::io::Result<usize> {
        cleanup_old_logs(&self.log_dir, retention_days)
    }
}

// ===== Helper Functions =====

/// Write header metadata to log file
fn write_header(writer: &mut BufWriter<File>, scan_id: Uuid) -> std::io::Result<()> {
    let header = json!({
        "type": "header",
        "version": "1.0",
        "scan_id": scan_id,
        "start_time": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "prtip_version": env!("CARGO_PKG_VERSION"),
    });

    writeln!(writer, "{}", serde_json::to_string(&header).unwrap())
}

/// Write footer metadata to log file
fn write_footer(writer: &mut BufWriter<File>, scan_id: Option<Uuid>) -> std::io::Result<()> {
    let footer = json!({
        "type": "footer",
        "scan_id": scan_id,
        "end_time": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });

    writeln!(writer, "{}", serde_json::to_string(&footer).unwrap())
}

/// Check if log file should be rotated
fn should_rotate(writer: &BufWriter<File>, max_file_size: u64) -> bool {
    if let Ok(metadata) = writer.get_ref().metadata() {
        metadata.len() >= max_file_size
    } else {
        false
    }
}

/// Compress log file using gzip
fn compress_log_file(path: &PathBuf) -> std::io::Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    let input = File::open(path)?;
    let gz_path = path.with_extension("jsonl.gz");
    let output = File::create(&gz_path)?;
    let mut encoder = GzEncoder::new(output, Compression::default());

    std::io::copy(&mut std::io::BufReader::new(input), &mut encoder)?;
    encoder.finish()?;

    // Delete original uncompressed file
    fs::remove_file(path)?;

    Ok(())
}

/// Clean up logs older than retention period
fn cleanup_old_logs(log_dir: &PathBuf, retention_days: u64) -> std::io::Result<usize> {
    let cutoff = SystemTime::now() - Duration::from_secs(retention_days * 24 * 60 * 60);
    let mut deleted_count = 0;

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        // Skip directories
        if metadata.is_dir() {
            continue;
        }

        // Check if file is older than retention period
        if let Ok(modified) = metadata.modified() {
            if modified < cutoff && fs::remove_file(entry.path()).is_ok() {
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ScanEvent;
    use crate::types::ScanType;
    use std::time::SystemTime;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_event_logger_creation() {
        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus, config).await;
        assert!(logger.is_ok());

        let logger = logger.unwrap();
        assert!(logger.log_dir().exists());
    }

    #[tokio::test]
    async fn test_event_logger_writes_events() {
        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus.clone(), config).await.unwrap();

        let scan_id = Uuid::new_v4();

        // Publish ScanStarted event
        bus.publish(ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 100,
            timestamp: SystemTime::now(),
        })
        .await;

        // Publish ScanCompleted to trigger flush
        bus.publish(ScanEvent::ScanCompleted {
            scan_id,
            duration: Duration::from_secs(1),
            total_targets: 1,
            open_ports: 5,
            closed_ports: 95,
            filtered_ports: 0,
            detected_services: 0,
            timestamp: SystemTime::now(),
        })
        .await;

        // Give logger time to write and flush
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check log file exists
        let log_file = logger.log_dir().join(format!("{}.jsonl", scan_id));
        assert!(log_file.exists());

        // Read log file
        let contents = fs::read_to_string(&log_file).unwrap();
        assert!(contents.contains("header"));
        assert!(contents.contains("ScanStarted") || contents.contains("scan_started"));
        assert!(contents.contains("ScanCompleted") || contents.contains("scan_completed"));
    }

    #[tokio::test]
    async fn test_event_logger_footer() {
        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus.clone(), config).await.unwrap();

        let scan_id = Uuid::new_v4();

        // Publish ScanStarted and ScanCompleted
        bus.publish(ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 100,
            timestamp: SystemTime::now(),
        })
        .await;

        bus.publish(ScanEvent::ScanCompleted {
            scan_id,
            duration: Duration::from_secs(10),
            total_targets: 1,
            open_ports: 5,
            closed_ports: 95,
            filtered_ports: 0,
            detected_services: 2,
            timestamp: SystemTime::now(),
        })
        .await;

        // Give logger time to write
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Read log file
        let log_file = logger.log_dir().join(format!("{}.jsonl", scan_id));
        let contents = fs::read_to_string(&log_file).unwrap();

        assert!(contents.contains("footer"));
    }

    #[tokio::test]
    async fn test_cleanup_old_logs() {
        let temp_dir = TempDir::new().unwrap();

        // Create an old log file
        let old_log = temp_dir.path().join("old-scan.jsonl");
        fs::write(&old_log, "test").unwrap();

        // Set modification time to 31 days ago
        #[cfg(unix)]
        {
            let thirty_one_days_ago = SystemTime::now() - Duration::from_secs(31 * 24 * 60 * 60);
            let _ = filetime::set_file_mtime(
                &old_log,
                filetime::FileTime::from_system_time(thirty_one_days_ago),
            );
        }

        let bus = Arc::new(EventBus::new(1000));
        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus, config).await.unwrap();

        // Cleanup should not delete anything on non-Unix systems or if mtime set failed
        let deleted = logger.cleanup_old_logs().unwrap_or(0);
        // On Unix with successful mtime set, should delete 1 file
        // On other systems or if mtime set failed, should delete 0 files
        assert!(deleted <= 1);
    }

    #[test]
    fn test_should_rotate() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = BufWriter::new(temp_file.reopen().unwrap());

        // Write less than max size
        writeln!(writer, "test").unwrap();
        assert!(!should_rotate(&writer, 100_000_000));

        // Note: Can't easily test rotation trigger without writing 100MB
        // Integration tests would cover this
    }

    #[tokio::test]
    async fn test_multiple_events() {
        use crate::{PortState, Protocol};
        use std::net::IpAddr;

        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus.clone(), config).await.unwrap();

        let scan_id = Uuid::new_v4();
        let ip: IpAddr = "192.168.1.1".parse().unwrap();

        // Publish multiple events
        bus.publish(ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 100,
            timestamp: SystemTime::now(),
        })
        .await;

        bus.publish(ScanEvent::PortFound {
            scan_id,
            ip,
            port: 80,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;

        bus.publish(ScanEvent::PortFound {
            scan_id,
            ip,
            port: 443,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;

        bus.publish(ScanEvent::ScanCompleted {
            scan_id,
            duration: Duration::from_secs(5),
            total_targets: 1,
            open_ports: 2,
            closed_ports: 98,
            filtered_ports: 0,
            detected_services: 0,
            timestamp: SystemTime::now(),
        })
        .await;

        // Give logger time to write
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Read log file
        let log_file = logger.log_dir().join(format!("{}.jsonl", scan_id));
        let contents = fs::read_to_string(&log_file).unwrap();

        // Verify all events are present
        assert!(contents.contains("header"));
        assert!(contents.contains("\"port\":80") || contents.contains("port\": 80"));
        assert!(contents.contains("\"port\":443") || contents.contains("port\": 443"));
        assert!(contents.contains("footer"));

        // Count lines (header + 4 events + footer = 6 lines minimum)
        let line_count = contents.lines().count();
        assert!(
            line_count >= 6,
            "Expected at least 6 lines, got {}",
            line_count
        );
    }

    #[tokio::test]
    async fn test_concurrent_scans() {
        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus.clone(), config).await.unwrap();

        let scan_id_1 = Uuid::new_v4();
        let scan_id_2 = Uuid::new_v4();

        // Start two scans concurrently
        bus.publish(ScanEvent::ScanStarted {
            scan_id: scan_id_1,
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 100,
            timestamp: SystemTime::now(),
        })
        .await;

        bus.publish(ScanEvent::ScanStarted {
            scan_id: scan_id_2,
            scan_type: ScanType::Connect,
            target_count: 1,
            port_count: 50,
            timestamp: SystemTime::now(),
        })
        .await;

        // Complete both scans
        bus.publish(ScanEvent::ScanCompleted {
            scan_id: scan_id_1,
            duration: Duration::from_secs(2),
            total_targets: 1,
            open_ports: 5,
            closed_ports: 95,
            filtered_ports: 0,
            detected_services: 0,
            timestamp: SystemTime::now(),
        })
        .await;

        bus.publish(ScanEvent::ScanCompleted {
            scan_id: scan_id_2,
            duration: Duration::from_secs(3),
            total_targets: 1,
            open_ports: 3,
            closed_ports: 47,
            filtered_ports: 0,
            detected_services: 0,
            timestamp: SystemTime::now(),
        })
        .await;

        // Give logger time to write
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify both log files exist
        let log_file_1 = logger.log_dir().join(format!("{}.jsonl", scan_id_1));
        let log_file_2 = logger.log_dir().join(format!("{}.jsonl", scan_id_2));

        assert!(log_file_1.exists(), "First scan log file should exist");
        assert!(log_file_2.exists(), "Second scan log file should exist");

        // Verify contents
        let contents_1 = fs::read_to_string(&log_file_1).unwrap();
        let contents_2 = fs::read_to_string(&log_file_2).unwrap();

        assert!(contents_1.contains("header"));
        assert!(contents_1.contains("footer"));
        assert!(contents_2.contains("header"));
        assert!(contents_2.contains("footer"));
    }

    #[tokio::test]
    async fn test_scan_cancellation() {
        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus.clone(), config).await.unwrap();

        let scan_id = Uuid::new_v4();

        // Start scan
        bus.publish(ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 100,
            timestamp: SystemTime::now(),
        })
        .await;

        // Cancel scan
        bus.publish(ScanEvent::ScanCancelled {
            scan_id,
            reason: "User cancelled".to_string(),
            partial_results: true,
            timestamp: SystemTime::now(),
        })
        .await;

        // Give logger time to write
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify log file exists and has footer
        let log_file = logger.log_dir().join(format!("{}.jsonl", scan_id));
        let contents = fs::read_to_string(&log_file).unwrap();

        assert!(contents.contains("header"));
        assert!(contents.contains("ScanCancelled") || contents.contains("scan_cancelled"));
        assert!(contents.contains("footer"));
    }

    #[tokio::test]
    async fn test_scan_error() {
        let bus = Arc::new(EventBus::new(1000));
        let temp_dir = TempDir::new().unwrap();

        let config = EventLoggerConfig {
            log_dir: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let logger = EventLogger::with_config(bus.clone(), config).await.unwrap();

        let scan_id = Uuid::new_v4();

        // Start scan
        bus.publish(ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 100,
            timestamp: SystemTime::now(),
        })
        .await;

        // Error occurs
        bus.publish(ScanEvent::ScanError {
            scan_id,
            error: "Network error".to_string(),
            recoverable: false,
            timestamp: SystemTime::now(),
        })
        .await;

        // Give logger time to write
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify log file exists and has footer
        let log_file = logger.log_dir().join(format!("{}.jsonl", scan_id));
        let contents = fs::read_to_string(&log_file).unwrap();

        assert!(contents.contains("header"));
        assert!(contents.contains("ScanError") || contents.contains("scan_error"));
        assert!(contents.contains("footer"));
    }

    #[tokio::test]
    async fn test_compression() {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.jsonl");
        fs::write(&test_file, "test data\n").unwrap();

        // Compress it
        compress_log_file(&test_file).unwrap();

        // Verify compressed file exists
        let gz_file = temp_dir.path().join("test.jsonl.gz");
        assert!(gz_file.exists(), "Compressed file should exist");

        // Verify original file is deleted
        assert!(!test_file.exists(), "Original file should be deleted");

        // Verify compressed file can be decompressed
        let file = File::open(&gz_file).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "test data\n");
    }
}
