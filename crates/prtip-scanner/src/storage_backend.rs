//! Storage Backend Abstraction
//!
//! Provides a unified interface for different storage backends:
//! - **Memory**: Fast in-memory storage (default, ~37ms for 10K ports)
//! - **AsyncDatabase**: Non-blocking async SQLite storage (~40-50ms for 10K ports)
//!
//! # Architecture
//!
//! The storage backend abstracts away the details of where and how
//! results are stored, allowing the scheduler to work with both
//! in-memory and database storage seamlessly.

use crate::{async_storage::async_storage_worker, MemoryStorage, ScanStorage};
use prtip_core::{Result, ScanResult, ScanType};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::{debug, info};

/// Storage backend for scan results
///
/// Supports two modes:
/// - **Memory**: In-memory storage (default, fastest)
/// - **AsyncDatabase**: Async SQLite storage (optional, --with-db)
pub enum StorageBackend {
    /// In-memory storage (default mode)
    ///
    /// Results stored in memory only, no persistence.
    /// Performance: ~37ms for 10K ports.
    Memory(Arc<MemoryStorage>),

    /// Async database storage (--with-db mode)
    ///
    /// Results sent to background worker via unbounded channel.
    /// Worker writes to SQLite without blocking scanning threads.
    /// Performance: ~40-50ms for 10K ports.
    ///
    /// The tx is wrapped in Option<Mutex<>> to allow explicit drop for channel closure.
    /// The completion_rx signals when the worker has finished all writes.
    AsyncDatabase {
        storage: Arc<ScanStorage>,
        scan_id: i64,
        tx: Arc<Mutex<Option<UnboundedSender<Vec<ScanResult>>>>>,
        completion_rx: Arc<Mutex<Option<oneshot::Receiver<Result<()>>>>>,
    },
}

impl StorageBackend {
    /// Create a memory storage backend
    ///
    /// # Arguments
    ///
    /// * `capacity` - Estimated result capacity for pre-allocation
    pub fn memory(capacity: usize) -> Self {
        info!("Storage backend: Memory (capacity: {})", capacity);
        Self::Memory(Arc::new(MemoryStorage::with_capacity(capacity)))
    }

    /// Create an async database storage backend
    ///
    /// # Arguments
    ///
    /// * `storage` - Arc-wrapped SQLite storage
    /// * `scan_type` - Type of scan being performed
    /// * `target` - Target specification (for scan record)
    ///
    /// # Returns
    ///
    /// A storage backend with background worker spawned.
    pub async fn async_database(
        storage: Arc<ScanStorage>,
        scan_type: ScanType,
        target: &str,
    ) -> Result<Self> {
        // Create scan record
        let config_json = format!(
            r#"{{"scan_type": "{}", "target": "{}"}}"#,
            scan_type, target
        );

        let scan_id = storage.create_scan(&config_json).await?;
        info!("Created scan ID: {} (async mode)", scan_id);

        // Create unbounded channel (never blocks sender)
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        // Spawn background worker
        let storage_clone = Arc::clone(&storage);
        tokio::spawn(async move {
            if let Err(e) = async_storage_worker(storage_clone, scan_id, rx, completion_tx).await {
                tracing::error!("Async storage worker failed: {}", e);
            }
        });

        info!("Storage backend: AsyncDatabase (scan_id: {})", scan_id);

        Ok(Self::AsyncDatabase {
            storage,
            scan_id,
            tx: Arc::new(Mutex::new(Some(tx))),
            completion_rx: Arc::new(Mutex::new(Some(completion_rx))),
        })
    }

    /// Add a single scan result
    ///
    /// # Arguments
    ///
    /// * `result` - The scan result to store
    ///
    /// # Performance
    ///
    /// - Memory: Direct write to Vec (~10ns)
    /// - AsyncDatabase: Channel send (~100ns, non-blocking)
    pub fn add_result(&self, result: ScanResult) -> Result<()> {
        match self {
            Self::Memory(storage) => storage.add_result(result),
            Self::AsyncDatabase { tx, .. } => {
                // Send to async worker (non-blocking!)
                let tx_guard = tx.lock().unwrap();
                if let Some(sender) = tx_guard.as_ref() {
                    sender.send(vec![result]).map_err(|_| {
                        prtip_core::Error::Storage("Async storage worker died".to_string())
                    })?;
                    Ok(())
                } else {
                    Err(prtip_core::Error::Storage(
                        "Storage backend already flushed".to_string(),
                    ))
                }
            }
        }
    }

    /// Add multiple scan results in batch
    ///
    /// # Arguments
    ///
    /// * `results` - Slice of scan results to store
    ///
    /// # Performance
    ///
    /// More efficient than calling `add_result` repeatedly.
    pub fn add_results_batch(&self, results: Vec<ScanResult>) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        match self {
            Self::Memory(storage) => storage.add_results_batch(&results),
            Self::AsyncDatabase { tx, .. } => {
                let tx_guard = tx.lock().unwrap();
                if let Some(sender) = tx_guard.as_ref() {
                    sender.send(results).map_err(|_| {
                        prtip_core::Error::Storage("Async storage worker died".to_string())
                    })?;
                    Ok(())
                } else {
                    Err(prtip_core::Error::Storage(
                        "Storage backend already flushed".to_string(),
                    ))
                }
            }
        }
    }

    /// Flush any pending writes
    ///
    /// For async database storage, this drops the sender to signal completion
    /// and waits for the worker to finish. For memory storage, this is a no-op.
    ///
    /// # Implementation Note
    ///
    /// The critical fix: We take ownership of the sender (Option::take()),
    /// then explicitly drop it. This signals channel closure to the worker's
    /// `else` branch in the select! loop. Then we await the oneshot completion
    /// signal for true async waiting.
    pub async fn flush(&self) -> Result<()> {
        match self {
            Self::Memory(_) => Ok(()),
            Self::AsyncDatabase {
                tx, completion_rx, ..
            } => {
                debug!("Flushing async storage (taking ownership of tx and dropping)");

                // Step 1: Take ownership of tx and drop it (signals channel closure)
                {
                    let mut tx_guard = tx.lock().unwrap();
                    if let Some(sender) = tx_guard.take() {
                        debug!("Dropping sender to signal channel closure");
                        drop(sender); // Explicit drop signals channel closure to worker
                    } else {
                        debug!("Sender already dropped, skipping flush");
                        return Ok(()); // Already flushed
                    }
                }
                // tx is now dropped → worker will detect channel closure!

                // Step 2: Wait for worker completion signal
                let rx = {
                    let mut rx_guard = completion_rx.lock().unwrap();
                    rx_guard.take() // Take ownership of receiver
                };

                if let Some(rx) = rx {
                    debug!("Awaiting worker completion signal");
                    // True async wait for completion!
                    match rx.await {
                        Ok(result) => {
                            debug!("Worker completed successfully");
                            result
                        }
                        Err(_) => Err(prtip_core::Error::Storage(
                            "Worker completion channel closed unexpectedly".to_string(),
                        )),
                    }
                } else {
                    debug!("Completion receiver already consumed");
                    Ok(()) // Already flushed
                }
            }
        }
    }

    /// Get all scan results
    ///
    /// # Returns
    ///
    /// Vector of all scan results.
    ///
    /// For async database storage, this waits for the worker to complete
    /// and then retrieves results from SQLite.
    pub async fn get_results(&self) -> Result<Vec<ScanResult>> {
        match self {
            Self::Memory(storage) => Ok(storage.get_results()),
            Self::AsyncDatabase {
                storage, scan_id, ..
            } => {
                // Flush first to ensure all results written
                self.flush().await?;

                // Retrieve from database
                storage.get_scan_results(*scan_id).await
            }
        }
    }

    /// Complete the scan (for database mode only)
    ///
    /// Marks the scan as complete in the database.
    pub async fn complete_scan(&self) -> Result<()> {
        match self {
            Self::Memory(_) => Ok(()),
            Self::AsyncDatabase {
                storage, scan_id, ..
            } => {
                // Flush first to ensure all writes complete
                self.flush().await?;

                storage.complete_scan(*scan_id).await
            }
        }
    }

    /// Get the number of results stored
    pub fn len(&self) -> usize {
        match self {
            Self::Memory(storage) => storage.len(),
            Self::AsyncDatabase { .. } => {
                // Can't get count from async storage without blocking
                // Caller should use get_results() for accurate count
                0
            }
        }
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Memory(storage) => storage.is_empty(),
            Self::AsyncDatabase { .. } => false, // Unknown for async
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::PortState;

    #[test]
    fn test_memory_backend() {
        let backend = StorageBackend::memory(100);

        let result = ScanResult::new("192.168.1.1".parse().unwrap(), 80, PortState::Open);

        backend.add_result(result).unwrap();
        assert_eq!(backend.len(), 1);
    }

    #[test]
    fn test_memory_backend_batch() {
        let backend = StorageBackend::memory(100);

        let results: Vec<ScanResult> = (80..=90)
            .map(|port| ScanResult::new("192.168.1.1".parse().unwrap(), port, PortState::Open))
            .collect();

        backend.add_results_batch(results).unwrap();
        assert_eq!(backend.len(), 11);
    }

    #[tokio::test]
    async fn test_async_database_backend() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let backend = StorageBackend::async_database(storage, ScanType::Connect, "192.168.1.1/32")
            .await
            .unwrap();

        let result = ScanResult::new("192.168.1.1".parse().unwrap(), 80, PortState::Open);

        backend.add_result(result).unwrap();

        // Flush and retrieve
        backend.flush().await.unwrap();
        let results = backend.get_results().await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, 80);
    }

    #[tokio::test]
    async fn test_async_database_batch() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let backend = StorageBackend::async_database(storage, ScanType::Connect, "192.168.1.1/32")
            .await
            .unwrap();

        let results: Vec<ScanResult> = (80..=100)
            .map(|port| ScanResult::new("192.168.1.1".parse().unwrap(), port, PortState::Open))
            .collect();

        backend.add_results_batch(results).unwrap();

        backend.flush().await.unwrap();
        let retrieved = backend.get_results().await.unwrap();

        assert_eq!(retrieved.len(), 21);
    }

    #[tokio::test]
    async fn test_memory_get_results() {
        let backend = StorageBackend::memory(10);

        let results: Vec<ScanResult> = (1..=5)
            .map(|port| ScanResult::new("127.0.0.1".parse().unwrap(), port, PortState::Open))
            .collect();

        backend.add_results_batch(results).unwrap();

        let retrieved = backend.get_results().await.unwrap();
        assert_eq!(retrieved.len(), 5);
    }
}
