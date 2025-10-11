//! In-Memory Storage for Scan Results
//!
//! Provides fast in-memory storage for scan results with no I/O overhead.
//! This is the default storage mode for maximum performance (~37ms for 10K ports).
//!
//! # Performance
//!
//! - **No disk I/O**: Results stored entirely in memory
//! - **Thread-safe**: Uses RwLock for concurrent access
//! - **Fast export**: Direct access to results for JSON/XML/text output
//! - **Zero overhead**: No database initialization, transactions, or indexes
//!
//! # Use Cases
//!
//! - One-time scans where persistence is not needed
//! - Fast exploratory scanning
//! - When results will be immediately exported to JSON/XML
//! - Performance-critical scenarios (< 50ms scan requirement)

use prtip_core::{Result, ScanResult};
use std::sync::RwLock;

/// In-memory storage for scan results
///
/// Stores results in a simple Vec with RwLock for thread-safety.
/// No persistence, no database overhead, maximum performance.
///
/// # Examples
///
/// ```
/// use prtip_scanner::MemoryStorage;
/// use prtip_core::ScanResult;
///
/// let storage = MemoryStorage::new();
///
/// let result = ScanResult::new(
///     "192.168.1.1".parse().unwrap(),
///     80,
///     prtip_core::PortState::Open,
/// );
///
/// storage.add_result(result).unwrap();
/// assert_eq!(storage.get_results().len(), 1);
/// ```
pub struct MemoryStorage {
    results: RwLock<Vec<ScanResult>>,
}

impl MemoryStorage {
    /// Create a new memory storage
    ///
    /// Allocates an empty Vec with default capacity.
    pub fn new() -> Self {
        Self {
            results: RwLock::new(Vec::new()),
        }
    }

    /// Create a new memory storage with pre-allocated capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Initial capacity for the results vector
    ///
    /// This is useful when you know approximately how many results to expect,
    /// reducing reallocation overhead.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            results: RwLock::new(Vec::with_capacity(capacity)),
        }
    }

    /// Add a single scan result
    ///
    /// # Arguments
    ///
    /// * `result` - The scan result to add
    ///
    /// # Thread Safety
    ///
    /// Uses a write lock to ensure thread-safe access.
    pub fn add_result(&self, result: ScanResult) -> Result<()> {
        let mut results = self
            .results
            .write()
            .map_err(|e| prtip_core::Error::Storage(format!("Lock poisoned: {}", e)))?;

        results.push(result);
        Ok(())
    }

    /// Add multiple scan results in bulk
    ///
    /// # Arguments
    ///
    /// * `batch` - Slice of scan results to add
    ///
    /// More efficient than calling `add_result` repeatedly due to
    /// acquiring the lock only once.
    pub fn add_results_batch(&self, batch: &[ScanResult]) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut results = self
            .results
            .write()
            .map_err(|e| prtip_core::Error::Storage(format!("Lock poisoned: {}", e)))?;

        results.extend_from_slice(batch);
        Ok(())
    }

    /// Get all scan results
    ///
    /// Returns a cloned vector of all results.
    /// This is a read operation and can be called concurrently.
    pub fn get_results(&self) -> Vec<ScanResult> {
        self.results.read().map(|r| r.clone()).unwrap_or_default()
    }

    /// Get the number of stored results
    ///
    /// # Returns
    ///
    /// The total count of results in memory.
    pub fn len(&self) -> usize {
        self.results.read().map(|r| r.len()).unwrap_or(0)
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all results
    ///
    /// Removes all stored results from memory.
    pub fn clear(&self) -> Result<()> {
        let mut results = self
            .results
            .write()
            .map_err(|e| prtip_core::Error::Storage(format!("Lock poisoned: {}", e)))?;

        results.clear();
        Ok(())
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::PortState;
    use std::net::IpAddr;

    #[test]
    fn test_new_storage() {
        let storage = MemoryStorage::new();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let storage = MemoryStorage::with_capacity(1000);
        assert_eq!(storage.len(), 0);
        // Capacity is internal, can't test directly
    }

    #[test]
    fn test_add_result() {
        let storage = MemoryStorage::new();

        let result = ScanResult::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80,
            PortState::Open,
        );

        storage.add_result(result).unwrap();
        assert_eq!(storage.len(), 1);
        assert!(!storage.is_empty());
    }

    #[test]
    fn test_add_multiple_results() {
        let storage = MemoryStorage::new();

        for port in 80..=90 {
            let result = ScanResult::new(
                "192.168.1.1".parse::<IpAddr>().unwrap(),
                port,
                PortState::Open,
            );
            storage.add_result(result).unwrap();
        }

        assert_eq!(storage.len(), 11);
    }

    #[test]
    fn test_add_results_batch() {
        let storage = MemoryStorage::new();

        let results: Vec<ScanResult> = (80..=100)
            .map(|port| {
                ScanResult::new(
                    "192.168.1.1".parse::<IpAddr>().unwrap(),
                    port,
                    PortState::Open,
                )
            })
            .collect();

        storage.add_results_batch(&results).unwrap();
        assert_eq!(storage.len(), 21);
    }

    #[test]
    fn test_get_results() {
        let storage = MemoryStorage::new();

        let result1 = ScanResult::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80,
            PortState::Open,
        );
        let result2 = ScanResult::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            443,
            PortState::Closed,
        );

        storage.add_result(result1).unwrap();
        storage.add_result(result2).unwrap();

        let results = storage.get_results();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].port, 80);
        assert_eq!(results[1].port, 443);
    }

    #[test]
    fn test_clear() {
        let storage = MemoryStorage::new();

        let result = ScanResult::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80,
            PortState::Open,
        );

        storage.add_result(result).unwrap();
        assert_eq!(storage.len(), 1);

        storage.clear().unwrap();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_empty_batch() {
        let storage = MemoryStorage::new();
        storage.add_results_batch(&[]).unwrap();
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let storage = Arc::new(MemoryStorage::new());
        let mut handles = vec![];

        // Spawn 10 threads, each adding 100 results
        for thread_id in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let handle = thread::spawn(move || {
                for port in 0..100 {
                    let result = ScanResult::new(
                        "192.168.1.1".parse::<IpAddr>().unwrap(),
                        (thread_id * 100 + port) as u16,
                        PortState::Open,
                    );
                    storage_clone.add_result(result).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(storage.len(), 1000);
    }
}
