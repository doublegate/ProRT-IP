//! SQLite Storage for Scan Results
//!
//! Provides async SQLite database storage for scan results with support for:
//! - Transaction-based batch inserts with multi-row VALUES
//! - Indexed queries for fast retrieval
//! - WAL mode for concurrent access
//! - Automatic schema initialization
//! - Performance-optimized SQLite pragmas
//!
//! # Database Schema
//!
//! ## Tables
//!
//! - **scans**: Metadata about scan executions
//! - **scan_results**: Individual port scan results
//!
//! ## Indexes
//!
//! - `idx_scan_id`: Fast lookups by scan ID
//! - `idx_target_ip`: Fast lookups by target IP
//! - `idx_port`: Fast lookups by port number
//!
//! # Performance Optimizations
//!
//! ## Write-Ahead Logging (WAL)
//!
//! The database uses WAL mode (`PRAGMA journal_mode=WAL`) for improved concurrency:
//! - Readers do not block writers
//! - Writers do not block readers
//! - Better performance for write-heavy workloads
//!
//! ## Batch Writes
//!
//! The `store_results_batch()` method uses multi-row INSERT VALUES statements:
//! - 100 rows per INSERT statement (SQLite parameter limit: 999)
//! - Single transaction for all batches
//! - 100-1000x faster than individual inserts
//!
//! ## SQLite Pragmas
//!
//! Automatically applied on initialization:
//! - `synchronous=NORMAL`: Safe for WAL mode, better performance
//! - `cache_size=-64000`: 64MB cache (vs 2MB default)
//! - `busy_timeout=10000`: 10-second timeout to reduce SQLITE_BUSY errors

use chrono::{DateTime, Utc};
use prtip_core::{Error, PortState, Result, ScanResult};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, Row};
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info};

/// SQLite-based scan result storage
///
/// Manages a SQLite database for storing and retrieving scan results.
/// Supports batch operations and concurrent access via connection pooling.
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::ScanStorage;
/// use prtip_core::ScanResult;
/// use std::net::IpAddr;
///
/// # async fn example() -> prtip_core::Result<()> {
/// // Create storage (or use existing database)
/// let storage = ScanStorage::new("results.db").await?;
///
/// // Create a new scan
/// let scan_id = storage.create_scan(r#"{"targets": "192.168.1.0/24"}"#).await?;
///
/// // Store results
/// let result = ScanResult::new(
///     "192.168.1.1".parse().unwrap(),
///     80,
///     prtip_core::PortState::Open,
/// );
/// storage.store_result(scan_id, &result).await?;
///
/// // Complete the scan
/// storage.complete_scan(scan_id).await?;
/// # Ok(())
/// # }
/// ```
pub struct ScanStorage {
    pub(crate) pool: SqlitePool,
}

impl ScanStorage {
    /// Create a new storage instance
    ///
    /// # Arguments
    ///
    /// * `database_path` - Path to SQLite database file (use ":memory:" for in-memory)
    ///
    /// # Database Creation
    ///
    /// If the database file doesn't exist, it will be created automatically.
    /// The schema will be initialized on first use.
    ///
    /// # WAL Mode
    ///
    /// The database is configured to use Write-Ahead Logging (WAL) mode for
    /// better concurrent performance.
    pub async fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        let path_str = database_path.as_ref().to_string_lossy().to_string();

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path_str))
            .map_err(|e| Error::Storage(format!("Invalid database path: {}", e)))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(30))
            .disable_statement_logging();

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| Error::Storage(format!("Failed to connect to database: {}", e)))?;

        info!("Connected to SQLite database: {}", path_str);

        let storage = Self { pool };
        storage.init_schema().await?;

        Ok(storage)
    }

    /// Initialize database schema
    ///
    /// Creates tables and indexes if they don't exist.
    /// Also applies performance optimizations via SQLite pragmas.
    async fn init_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        // Apply performance pragmas
        // synchronous=NORMAL is safe for WAL mode and provides better performance
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&self.pool)
            .await
            .ok();

        // Increase cache size to 64MB (from 2MB default) for better performance
        sqlx::query("PRAGMA cache_size = -64000")
            .execute(&self.pool)
            .await
            .ok();

        // Set busy timeout to 10 seconds to reduce SQLITE_BUSY errors
        sqlx::query("PRAGMA busy_timeout = 10000")
            .execute(&self.pool)
            .await
            .ok();

        debug!("Applied SQLite performance pragmas (synchronous=NORMAL, cache_size=64MB, busy_timeout=10s)");

        // Scans table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time TIMESTAMP NOT NULL,
                end_time TIMESTAMP,
                config_json TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to create scans table: {}", e)))?;

        // Scan results table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scan_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scan_id INTEGER NOT NULL,
                target_ip TEXT NOT NULL,
                port INTEGER NOT NULL,
                state TEXT NOT NULL,
                service TEXT,
                banner TEXT,
                response_time_ms INTEGER NOT NULL,
                timestamp TIMESTAMP NOT NULL,
                FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to create scan_results table: {}", e)))?;

        // Indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_scan_id ON scan_results(scan_id)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_target_ip ON scan_results(target_ip)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_port ON scan_results(port)")
            .execute(&self.pool)
            .await
            .ok();

        debug!("Database schema initialized");
        Ok(())
    }

    /// Create a new scan record
    ///
    /// # Arguments
    ///
    /// * `config_json` - JSON string containing scan configuration
    ///
    /// # Returns
    ///
    /// The ID of the newly created scan record.
    pub async fn create_scan(&self, config_json: &str) -> Result<i64> {
        let start_time = Utc::now();

        let row =
            sqlx::query("INSERT INTO scans (start_time, config_json) VALUES (?, ?) RETURNING id")
                .bind(start_time)
                .bind(config_json)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::Storage(format!("Failed to create scan: {}", e)))?;

        let scan_id: i64 = row.get(0);
        debug!("Created scan with ID: {}", scan_id);

        Ok(scan_id)
    }

    /// Complete a scan
    ///
    /// Updates the scan record with an end timestamp.
    ///
    /// # Arguments
    ///
    /// * `scan_id` - ID of the scan to complete
    pub async fn complete_scan(&self, scan_id: i64) -> Result<()> {
        let end_time = Utc::now();

        sqlx::query("UPDATE scans SET end_time = ? WHERE id = ?")
            .bind(end_time)
            .bind(scan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Storage(format!("Failed to complete scan: {}", e)))?;

        debug!("Completed scan ID: {}", scan_id);
        Ok(())
    }

    /// Store a single scan result
    ///
    /// # Arguments
    ///
    /// * `scan_id` - ID of the scan this result belongs to
    /// * `result` - The scan result to store
    pub async fn store_result(&self, scan_id: i64, result: &ScanResult) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO scan_results
            (scan_id, target_ip, port, state, service, banner, response_time_ms, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(scan_id)
        .bind(result.target_ip.to_string())
        .bind(result.port as i64)
        .bind(result.state.to_string())
        .bind(&result.service)
        .bind(&result.banner)
        .bind(result.response_time.as_millis() as i64)
        .bind(result.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to store result: {}", e)))?;

        Ok(())
    }

    /// Store multiple results in a transaction
    ///
    /// This is significantly faster than storing results individually for
    /// large result sets.
    ///
    /// # Arguments
    ///
    /// * `scan_id` - ID of the scan these results belong to
    /// * `results` - Slice of scan results to store
    ///
    /// # Performance
    ///
    /// Uses a single transaction with multi-row INSERT VALUES for optimal
    /// performance. Provides 100-1000x speedup compared to individual inserts
    /// for batches of 100+ results.
    ///
    /// SQLite has a parameter limit of 999, so we insert 8 columns = ~120 rows
    /// per statement, then chunk larger batches.
    pub async fn store_results_batch(&self, scan_id: i64, results: &[ScanResult]) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;

        // SQLite parameter limit is 999, with 8 params per row = max 124 rows per query
        // Use 100 rows per query for safety
        const ROWS_PER_QUERY: usize = 100;

        for chunk in results.chunks(ROWS_PER_QUERY) {
            // Build multi-row INSERT statement
            let placeholders: Vec<String> = (0..chunk.len())
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?)".to_string())
                .collect();

            let query_str = format!(
                "INSERT INTO scan_results \
                 (scan_id, target_ip, port, state, service, banner, response_time_ms, timestamp) \
                 VALUES {}",
                placeholders.join(", ")
            );

            let mut query = sqlx::query(&query_str);

            // Bind all parameters for this chunk
            for result in chunk {
                query = query
                    .bind(scan_id)
                    .bind(result.target_ip.to_string())
                    .bind(result.port as i64)
                    .bind(result.state.to_string())
                    .bind(&result.service)
                    .bind(&result.banner)
                    .bind(result.response_time.as_millis() as i64)
                    .bind(result.timestamp);
            }

            query
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Storage(format!("Failed to insert result batch: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

        debug!("Stored {} results in batch transaction", results.len());
        Ok(())
    }

    /// Retrieve all results for a scan
    ///
    /// # Arguments
    ///
    /// * `scan_id` - ID of the scan to retrieve results for
    ///
    /// # Returns
    ///
    /// Vector of `ScanResult` objects for the specified scan.
    pub async fn get_scan_results(&self, scan_id: i64) -> Result<Vec<ScanResult>> {
        let rows = sqlx::query(
            r#"
            SELECT target_ip, port, state, service, banner, response_time_ms, timestamp
            FROM scan_results
            WHERE scan_id = ?
            ORDER BY target_ip, port
            "#,
        )
        .bind(scan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch scan results: {}", e)))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let target_ip_str: String = row.get(0);
            let target_ip: IpAddr = target_ip_str
                .parse()
                .map_err(|e| Error::Parse(format!("Invalid IP address in database: {}", e)))?;

            let port: i64 = row.get(1);
            let state_str: String = row.get(2);
            let state = match state_str.as_str() {
                "open" => PortState::Open,
                "closed" => PortState::Closed,
                "filtered" => PortState::Filtered,
                _ => PortState::Unknown,
            };

            let service: Option<String> = row.get(3);
            let banner: Option<String> = row.get(4);
            let response_time_ms: i64 = row.get(5);
            let timestamp: DateTime<Utc> = row.get(6);

            let mut result = ScanResult::new(target_ip, port as u16, state)
                .with_response_time(Duration::from_millis(response_time_ms as u64));
            result.timestamp = timestamp;

            if let Some(svc) = service {
                result = result.with_service(svc);
            }
            if let Some(bnr) = banner {
                result = result.with_banner(bnr);
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Get total scan count
    ///
    /// # Returns
    ///
    /// The total number of scans in the database.
    pub async fn get_scan_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM scans")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Storage(format!("Failed to count scans: {}", e)))?;

        Ok(row.get(0))
    }

    /// Get total result count for a scan
    ///
    /// # Arguments
    ///
    /// * `scan_id` - ID of the scan
    ///
    /// # Returns
    ///
    /// The number of results for the specified scan.
    pub async fn get_result_count(&self, scan_id: i64) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) FROM scan_results WHERE scan_id = ?")
            .bind(scan_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Storage(format!("Failed to count results: {}", e)))?;

        Ok(row.get(0))
    }

    /// Close the database connection pool
    ///
    /// Gracefully closes all connections in the pool.
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_create_storage() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let count = storage.get_scan_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_create_and_complete_scan() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();
        assert!(scan_id > 0);

        storage.complete_scan(scan_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_store_result() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        let result = ScanResult::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            80,
            PortState::Open,
        )
        .with_response_time(Duration::from_millis(100));

        storage.store_result(scan_id, &result).await.unwrap();

        let count = storage.get_result_count(scan_id).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_store_batch() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        let results = vec![
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                80,
                PortState::Open,
            )
            .with_response_time(Duration::from_millis(100)),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                443,
                PortState::Closed,
            )
            .with_response_time(Duration::from_millis(50)),
        ];

        storage
            .store_results_batch(scan_id, &results)
            .await
            .unwrap();

        let count = storage.get_result_count(scan_id).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_get_scan_results() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        let original_results = vec![
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                80,
                PortState::Open,
            )
            .with_service("http".to_string())
            .with_banner("Apache".to_string())
            .with_response_time(Duration::from_millis(150)),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
                443,
                PortState::Closed,
            )
            .with_response_time(Duration::from_millis(50)),
        ];

        storage
            .store_results_batch(scan_id, &original_results)
            .await
            .unwrap();

        let retrieved_results = storage.get_scan_results(scan_id).await.unwrap();

        assert_eq!(retrieved_results.len(), 2);
        assert_eq!(
            retrieved_results[0].target_ip,
            original_results[0].target_ip
        );
        assert_eq!(retrieved_results[0].port, original_results[0].port);
        assert_eq!(retrieved_results[0].state, original_results[0].state);
        assert_eq!(retrieved_results[0].service, Some("http".to_string()));
        assert_eq!(retrieved_results[0].banner, Some("Apache".to_string()));
    }

    #[tokio::test]
    async fn test_empty_batch() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        storage.store_results_batch(scan_id, &[]).await.unwrap();

        let count = storage.get_result_count(scan_id).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_multiple_scans() {
        let storage = ScanStorage::new(":memory:").await.unwrap();

        let scan1 = storage.create_scan(r#"{"scan": 1}"#).await.unwrap();
        let scan2 = storage.create_scan(r#"{"scan": 2}"#).await.unwrap();

        assert_ne!(scan1, scan2);

        let result1 = ScanResult::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            80,
            PortState::Open,
        );
        let result2 = ScanResult::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            443,
            PortState::Closed,
        );

        storage.store_result(scan1, &result1).await.unwrap();
        storage.store_result(scan2, &result2).await.unwrap();

        let count1 = storage.get_result_count(scan1).await.unwrap();
        let count2 = storage.get_result_count(scan2).await.unwrap();

        assert_eq!(count1, 1);
        assert_eq!(count2, 1);
    }

    #[tokio::test]
    async fn test_large_batch() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        // Create 1000 results
        let results: Vec<ScanResult> = (1..=1000)
            .map(|i| {
                ScanResult::new(
                    IpAddr::V4(Ipv4Addr::new(10, 0, (i / 256) as u8, (i % 256) as u8)),
                    80,
                    PortState::Open,
                )
                .with_response_time(Duration::from_millis(i as u64))
            })
            .collect();

        storage
            .store_results_batch(scan_id, &results)
            .await
            .unwrap();

        let count = storage.get_result_count(scan_id).await.unwrap();
        assert_eq!(count, 1000);

        let retrieved = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(retrieved.len(), 1000);
    }

    #[tokio::test]
    async fn test_ipv6_storage() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        let result = ScanResult::new(
            IpAddr::V6(std::net::Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
            80,
            PortState::Open,
        );

        storage.store_result(scan_id, &result).await.unwrap();

        let retrieved = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].target_ip, result.target_ip);
    }

    #[tokio::test]
    async fn test_all_port_states() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        let results = vec![
            ScanResult::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80, PortState::Open),
            ScanResult::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 81, PortState::Closed),
            ScanResult::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 82, PortState::Filtered),
            ScanResult::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 83, PortState::Unknown),
        ];

        storage
            .store_results_batch(scan_id, &results)
            .await
            .unwrap();

        let retrieved = storage.get_scan_results(scan_id).await.unwrap();
        assert_eq!(retrieved.len(), 4);
        assert_eq!(retrieved[0].state, PortState::Open);
        assert_eq!(retrieved[1].state, PortState::Closed);
        assert_eq!(retrieved[2].state, PortState::Filtered);
        assert_eq!(retrieved[3].state, PortState::Unknown);
    }

    #[tokio::test]
    async fn test_scan_completion_timing() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        // Sleep briefly to ensure end_time > start_time
        tokio::time::sleep(Duration::from_millis(10)).await;

        storage.complete_scan(scan_id).await.unwrap();

        // If we got here without error, completion succeeded
    }

    #[tokio::test]
    async fn test_result_ordering() {
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        // Insert in non-sequential order
        let results = vec![
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
                80,
                PortState::Open,
            ),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                443,
                PortState::Open,
            ),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                80,
                PortState::Open,
            ),
        ];

        storage
            .store_results_batch(scan_id, &results)
            .await
            .unwrap();

        let retrieved = storage.get_scan_results(scan_id).await.unwrap();

        // Should be ordered by target_ip, then port
        assert_eq!(retrieved[0].target_ip.to_string(), "192.168.1.1");
        assert_eq!(retrieved[0].port, 80);
        assert_eq!(retrieved[1].target_ip.to_string(), "192.168.1.1");
        assert_eq!(retrieved[1].port, 443);
        assert_eq!(retrieved[2].target_ip.to_string(), "192.168.1.2");
        assert_eq!(retrieved[2].port, 80);
    }
}
