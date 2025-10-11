//! Async Storage Worker for Non-Blocking Database Writes
//!
//! This module implements a background worker that receives scan results
//! via an unbounded channel and writes them to SQLite asynchronously,
//! preventing database I/O from blocking the scanning threads.
//!
//! # Architecture
//!
//! ```text
//! Scanning Threads          Async Worker Thread         SQLite Database
//!     │                            │                           │
//!     ├─ Scan port 1              │                           │
//!     ├─ Scan port 2              │                           │
//!     ├─ Send result ──────────>  │                           │
//!     ├─ Scan port 3 (no block!)  ├─ Buffer result           │
//!     ├─ Scan port 4              ├─ Buffer result           │
//!     ├─ Send result ──────────>  │                           │
//!     │  ...                       ├─ Batch flush ──────────> │
//!     │  Continue scanning!        │  (500 results)           ├─ Write batch
//!     │                            │                           │  (~150-180ms)
//!     │                            │                           │
//!     └─ Scanning done (37.9ms)   └─ Final flush ──────────> └─ Write batch
//! ```
//!
//! # Performance
//!
//! - **Scanning threads**: ~37ms for 10K ports (no blocking)
//! - **Background writes**: ~150-180ms (happens in parallel)
//! - **Total user-perceived time**: ~40ms (scanning + small overhead)
//!
//! # Configuration
//!
//! - **Batch size**: 500 results (tunable via `ASYNC_BATCH_SIZE`)
//! - **Flush interval**: 100ms (tunable via `FLUSH_INTERVAL_MS`)
//! - **Channel**: Unbounded (never blocks sender)

use crate::storage::ScanStorage;
use prtip_core::{Result, ScanResult};
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};
use tracing::{debug, error, info};

/// Batch size for async storage writes (tuned for optimal throughput)
const ASYNC_BATCH_SIZE: usize = 500;

/// Flush interval in milliseconds (write batches periodically even if not full)
const FLUSH_INTERVAL_MS: u64 = 100;

/// Async storage worker that receives results via channel and writes to SQLite
///
/// This worker runs in a background task and:
/// 1. Receives results from scanning threads via unbounded channel
/// 2. Batches results in memory (up to ASYNC_BATCH_SIZE)
/// 3. Periodically flushes to SQLite (every FLUSH_INTERVAL_MS)
/// 4. Never blocks scanning threads (channel is unbounded)
/// 5. Signals completion via oneshot channel when all writes are done
///
/// # Performance
///
/// - Scanning threads: ~37ms for 10K ports (no blocking)
/// - Background writes: ~150-180ms (happens in parallel)
/// - Total user-perceived time: ~40ms (scanning + small overhead)
///
/// # Arguments
///
/// * `storage` - Arc-wrapped SQLite storage
/// * `scan_id` - Scan ID to associate results with
/// * `rx` - Unbounded receiver for scan results
/// * `completion_tx` - Oneshot sender to signal completion (success or error)
///
/// # Errors
///
/// Returns an error if database writes fail. Scanning threads will detect
/// this when the channel send fails (worker died). The error is also sent
/// via the completion channel.
pub async fn async_storage_worker(
    storage: Arc<ScanStorage>,
    scan_id: i64,
    mut rx: UnboundedReceiver<Vec<ScanResult>>,
    completion_tx: oneshot::Sender<Result<()>>,
) -> Result<()> {
    let mut buffer: Vec<ScanResult> = Vec::with_capacity(ASYNC_BATCH_SIZE);
    let mut last_flush = std::time::Instant::now();
    let mut total_written = 0usize;

    info!(
        "Async storage worker started (scan_id: {}, batch_size: {}, flush_interval: {}ms)",
        scan_id, ASYNC_BATCH_SIZE, FLUSH_INTERVAL_MS
    );

    let result: Result<()> = async {
        loop {
            // Use timeout on recv to allow periodic flushing
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(FLUSH_INTERVAL_MS),
                rx.recv(),
            )
            .await
            {
                // Received results before timeout
                Ok(Some(results)) => {
                    debug!("Received {} results from channel", results.len());
                    buffer.extend(results);

                    // Flush if buffer is full
                    if buffer.len() >= ASYNC_BATCH_SIZE {
                        match flush_buffer(&storage, scan_id, &mut buffer).await {
                            Ok(count) => {
                                total_written += count;
                                debug!("Flushed {} results (total: {})", count, total_written);
                            }
                            Err(e) => {
                                error!("Failed to flush buffer: {}", e);
                                return Err(e);
                            }
                        }
                        last_flush = std::time::Instant::now();
                    }
                }

                // Channel closed (recv returned None)
                Ok(None) => {
                    // Final flush of any remaining results
                    if !buffer.is_empty() {
                        match flush_buffer(&storage, scan_id, &mut buffer).await {
                            Ok(count) => {
                                total_written += count;
                                info!("Final flush: {} results (total: {})", count, total_written);
                            }
                            Err(e) => {
                                error!("Failed to flush final buffer: {}", e);
                                return Err(e);
                            }
                        }
                    }

                    info!(
                        "Async storage worker complete (scan_id: {}, total_written: {})",
                        scan_id, total_written
                    );
                    break;
                }

                // Timeout - do periodic flush
                Err(_) => {
                    if !buffer.is_empty()
                        && last_flush.elapsed().as_millis() >= FLUSH_INTERVAL_MS as u128
                    {
                        match flush_buffer(&storage, scan_id, &mut buffer).await {
                            Ok(count) => {
                                total_written += count;
                                debug!(
                                    "Periodic flush: {} results (total: {})",
                                    count, total_written
                                );
                            }
                            Err(e) => {
                                error!("Failed to flush buffer: {}", e);
                                return Err(e);
                            }
                        }
                        last_flush = std::time::Instant::now();
                    }
                }
            }
        }
        Ok(())
    }
    .await;

    // Signal completion (success or error) - receiver will get the result
    let signal_result = if result.is_ok() {
        Ok(())
    } else {
        Err(prtip_core::Error::Storage(
            "Async worker failed".to_string(),
        ))
    };
    let _ = completion_tx.send(signal_result);

    result
}

/// Flush buffered results to SQLite
///
/// # Arguments
///
/// * `storage` - Arc-wrapped SQLite storage
/// * `scan_id` - Scan ID to associate results with
/// * `buffer` - Buffer of results to flush (cleared after successful write)
///
/// # Returns
///
/// The number of results flushed.
async fn flush_buffer(
    storage: &Arc<ScanStorage>,
    scan_id: i64,
    buffer: &mut Vec<ScanResult>,
) -> Result<usize> {
    if buffer.is_empty() {
        return Ok(0);
    }

    let count = buffer.len();

    // Use existing batch insert method
    storage.store_results_batch(scan_id, buffer).await?;
    buffer.clear();

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::{PortState, ScanType};

    #[tokio::test]
    async fn test_async_worker_basic() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let scan_id = storage
            .create_scan(&format!(r#"{{"scan_type": "{}"}}"#, ScanType::Connect))
            .await
            .unwrap();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        // Spawn worker
        let worker = tokio::spawn(async_storage_worker(
            Arc::clone(&storage),
            scan_id,
            rx,
            completion_tx,
        ));

        // Send 10 results
        for i in 1..=10 {
            let result = ScanResult::new("127.0.0.1".parse().unwrap(), i as u16, PortState::Open);
            tx.send(vec![result]).unwrap();
        }

        // Close channel (signals completion)
        drop(tx);

        // Wait for completion signal
        completion_rx.await.unwrap().unwrap();

        // Wait for worker to finish
        worker.await.unwrap().unwrap();

        // Verify all results saved
        let results = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    async fn test_async_worker_batching() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let scan_id = storage
            .create_scan(&format!(r#"{{"scan_type": "{}"}}"#, ScanType::Connect))
            .await
            .unwrap();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        let worker = tokio::spawn(async_storage_worker(
            Arc::clone(&storage),
            scan_id,
            rx,
            completion_tx,
        ));

        // Send 1000 results (tests batching)
        for i in 1..=1000 {
            let result = ScanResult::new(
                "127.0.0.1".parse().unwrap(),
                (i % 65535) as u16 + 1,
                PortState::Open,
            );
            tx.send(vec![result]).unwrap();
        }

        drop(tx);
        completion_rx.await.unwrap().unwrap();
        worker.await.unwrap().unwrap();

        let results = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(results.len(), 1000);
    }

    #[tokio::test]
    async fn test_async_worker_empty() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let scan_id = storage
            .create_scan(&format!(r#"{{"scan_type": "{}"}}"#, ScanType::Connect))
            .await
            .unwrap();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        let worker = tokio::spawn(async_storage_worker(
            Arc::clone(&storage),
            scan_id,
            rx,
            completion_tx,
        ));

        // Close immediately without sending results
        drop(tx);
        completion_rx.await.unwrap().unwrap();
        worker.await.unwrap().unwrap();

        let results = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_async_worker_large_batches() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let scan_id = storage
            .create_scan(&format!(r#"{{"scan_type": "{}"}}"#, ScanType::Connect))
            .await
            .unwrap();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        let worker = tokio::spawn(async_storage_worker(
            Arc::clone(&storage),
            scan_id,
            rx,
            completion_tx,
        ));

        // Send 10,000 results to test multiple batch flushes
        for i in 1..=10_000 {
            let result = ScanResult::new(
                "127.0.0.1".parse().unwrap(),
                (i % 65535) as u16 + 1,
                PortState::Open,
            );
            tx.send(vec![result]).unwrap();
        }

        drop(tx);
        completion_rx.await.unwrap().unwrap();
        worker.await.unwrap().unwrap();

        let results = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(results.len(), 10_000);
    }

    #[tokio::test]
    async fn test_async_worker_bulk_send() {
        let storage = Arc::new(ScanStorage::new(":memory:").await.unwrap());
        let scan_id = storage
            .create_scan(&format!(r#"{{"scan_type": "{}"}}"#, ScanType::Connect))
            .await
            .unwrap();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (completion_tx, completion_rx) = oneshot::channel();

        let worker = tokio::spawn(async_storage_worker(
            Arc::clone(&storage),
            scan_id,
            rx,
            completion_tx,
        ));

        // Send results in bulk (100 at a time)
        for batch_num in 0..10 {
            let batch: Vec<ScanResult> = (0..100)
                .map(|i| {
                    ScanResult::new(
                        "127.0.0.1".parse().unwrap(),
                        (batch_num * 100 + i) as u16 + 1,
                        PortState::Open,
                    )
                })
                .collect();
            tx.send(batch).unwrap();
        }

        drop(tx);
        completion_rx.await.unwrap().unwrap();
        worker.await.unwrap().unwrap();

        let results = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(results.len(), 1000);
    }
}
