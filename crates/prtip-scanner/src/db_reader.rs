//! Database Reader Module
//!
//! Provides high-level query interface for scan results stored in SQLite.
//!
//! This module extends the basic `ScanStorage` functionality with advanced
//! query capabilities for data analysis and historical comparisons.
//!
//! # Query Types
//!
//! - **List scans:** Get metadata for all scans in database
//! - **Query by target:** Find all ports open on a specific host
//! - **Query by port:** Find all hosts with a specific port open
//! - **Query by service:** Find all hosts running a specific service
//! - **Compare scans:** Identify changes between two scans
//!
//! # Examples
//!
//! ```no_run
//! use prtip_scanner::DbReader;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let reader = DbReader::new("results.db").await?;
//!
//! // List all scans
//! let scans = reader.list_scans().await?;
//! for scan in scans {
//!     println!("Scan {}: {} results", scan.id, scan.result_count);
//! }
//!
//! // Find open ports on specific target
//! let ports = reader.query_open_ports("192.168.1.1").await?;
//! for port in ports {
//!     println!("Port {}: {}", port.port, port.service.unwrap_or("unknown".to_string()));
//! }
//!
//! // Find hosts with SSH open
//! let ssh_hosts = reader.query_by_port(22).await?;
//! # Ok(())
//! # }
//! ```

use crate::ScanStorage;
use chrono::{DateTime, Utc};
use prtip_core::{Error, PortState, Result, ScanResult};
use sqlx::Row;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;

/// Information about a stored scan
#[derive(Debug, Clone)]
pub struct ScanInfo {
    /// Scan ID
    pub id: i64,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp (None if scan incomplete)
    pub end_time: Option<DateTime<Utc>>,
    /// Scan configuration (JSON)
    pub config_json: String,
    /// Number of results for this scan
    pub result_count: i64,
}

/// Port information for query results
#[derive(Debug, Clone)]
pub struct PortInfo {
    /// Port number
    pub port: u16,
    /// Protocol (always "TCP" or "UDP" from current implementation)
    pub protocol: String,
    /// Service name (if detected)
    pub service: Option<String>,
    /// Service version (if detected)
    pub version: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: i64,
}

/// Host information for query results
#[derive(Debug, Clone)]
pub struct HostInfo {
    /// Target IP address
    pub target_ip: IpAddr,
    /// Port number
    pub port: u16,
    /// Service name (if detected)
    pub service: Option<String>,
    /// Service version (if detected)
    pub version: Option<String>,
    /// Port state
    pub state: PortState,
}

/// Comparison result between two scans
#[derive(Debug, Clone)]
pub struct ScanComparison {
    /// Scan 1 ID
    pub scan1_id: i64,
    /// Scan 2 ID
    pub scan2_id: i64,
    /// Ports that became open (new in scan2)
    pub new_open_ports: Vec<ScanResult>,
    /// Ports that closed (open in scan1, not in scan2)
    pub closed_ports: Vec<ScanResult>,
    /// Services that changed version
    pub changed_services: Vec<(ScanResult, ScanResult)>, // (old, new)
    /// Hosts that appeared in scan2
    pub new_hosts: Vec<IpAddr>,
    /// Hosts that disappeared from scan2
    pub disappeared_hosts: Vec<IpAddr>,
}

/// Database reader for querying scan results
///
/// Provides high-level query interface on top of `ScanStorage`.
pub struct DbReader {
    storage: ScanStorage,
}

impl DbReader {
    /// Create a new database reader
    ///
    /// # Arguments
    ///
    /// * `database_path` - Path to SQLite database file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// let reader = prtip_scanner::DbReader::new("results.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        let storage = ScanStorage::new(database_path).await?;
        Ok(Self { storage })
    }

    /// List all scans in the database
    ///
    /// Returns scan metadata including result counts, ordered by start time (descending).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # let reader = prtip_scanner::DbReader::new(":memory:").await?;
    /// let scans = reader.list_scans().await?;
    /// for scan in scans {
    ///     println!("Scan {} started at {}", scan.id, scan.start_time);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_scans(&self) -> Result<Vec<ScanInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.id,
                s.start_time,
                s.end_time,
                s.config_json,
                COUNT(r.id) as result_count
            FROM scans s
            LEFT JOIN scan_results r ON s.id = r.scan_id
            GROUP BY s.id
            ORDER BY s.start_time DESC
            "#,
        )
        .fetch_all(&self.storage.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to list scans: {}", e)))?;

        let mut scans = Vec::with_capacity(rows.len());
        for row in rows {
            scans.push(ScanInfo {
                id: row.get(0),
                start_time: row.get(1),
                end_time: row.get(2),
                config_json: row.get(3),
                result_count: row.get(4),
            });
        }

        Ok(scans)
    }

    /// Get scan results by scan ID
    ///
    /// Delegates to `ScanStorage::get_scan_results()`.
    ///
    /// # Arguments
    ///
    /// * `scan_id` - ID of the scan
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # let reader = prtip_scanner::DbReader::new(":memory:").await?;
    /// let results = reader.get_scan_results(1).await?;
    /// println!("Found {} results", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_scan_results(&self, scan_id: i64) -> Result<Vec<ScanResult>> {
        self.storage.get_scan_results(scan_id).await
    }

    /// Query open ports on a specific target
    ///
    /// Finds all open ports across all scans for the given target IP.
    ///
    /// # Arguments
    ///
    /// * `target_ip` - IP address to query
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # let reader = prtip_scanner::DbReader::new(":memory:").await?;
    /// let ports = reader.query_open_ports("192.168.1.1").await?;
    /// for port in ports {
    ///     println!("Port {}/{}: {}", port.port, port.protocol,
    ///         port.service.unwrap_or("unknown".to_string()));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_open_ports(&self, target_ip: &str) -> Result<Vec<PortInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT port,
                   FIRST_VALUE(service) OVER (PARTITION BY port ORDER BY timestamp DESC) as service,
                   FIRST_VALUE(response_time_ms) OVER (PARTITION BY port ORDER BY timestamp DESC) as response_time_ms
            FROM scan_results
            WHERE target_ip = ? AND state = 'open'
            ORDER BY port
            "#,
        )
        .bind(target_ip)
        .fetch_all(&self.storage.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to query open ports: {}", e)))?;

        // Deduplicate by port (window function may return duplicates)
        let mut seen_ports = std::collections::HashSet::new();
        let mut ports = Vec::new();

        for row in rows {
            let port: i64 = row.get(0);
            let port_u16 = port as u16;

            if seen_ports.insert(port_u16) {
                ports.push(PortInfo {
                    port: port_u16,
                    protocol: "TCP".to_string(), // Current implementation is TCP-only
                    service: row.get(1),
                    version: None, // TODO: Add version column to schema
                    response_time_ms: row.get(2),
                });
            }
        }

        Ok(ports)
    }

    /// Query all hosts that have a specific port open
    ///
    /// # Arguments
    ///
    /// * `port` - Port number to search for
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # let reader = prtip_scanner::DbReader::new(":memory:").await?;
    /// // Find all hosts with SSH (port 22) open
    /// let ssh_hosts = reader.query_by_port(22).await?;
    /// for host in ssh_hosts {
    ///     println!("{} has port 22 open", host.target_ip);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_port(&self, port: u16) -> Result<Vec<HostInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT target_ip, service, state
            FROM scan_results
            WHERE port = ? AND state = 'open'
            ORDER BY target_ip
            "#,
        )
        .bind(port as i64)
        .fetch_all(&self.storage.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to query by port: {}", e)))?;

        let mut hosts = Vec::with_capacity(rows.len());
        for row in rows {
            let target_ip_str: String = row.get(0);
            let target_ip: IpAddr = target_ip_str
                .parse()
                .map_err(|e| Error::Parse(format!("Invalid IP address in database: {}", e)))?;

            let state_str: String = row.get(2);
            let state = match state_str.as_str() {
                "open" => PortState::Open,
                "closed" => PortState::Closed,
                "filtered" => PortState::Filtered,
                _ => PortState::Unknown,
            };

            hosts.push(HostInfo {
                target_ip,
                port,
                service: row.get(1),
                version: None, // TODO: Add version column
                state,
            });
        }

        Ok(hosts)
    }

    /// Query all hosts running a specific service
    ///
    /// Performs case-insensitive partial match (LIKE query).
    ///
    /// # Arguments
    ///
    /// * `service_name` - Service name to search for (e.g., "http", "ssh")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # let reader = prtip_scanner::DbReader::new(":memory:").await?;
    /// // Find all web servers
    /// let web_servers = reader.query_by_service("http").await?;
    /// for host in web_servers {
    ///     println!("{} is running {}",
    ///         host.target_ip, host.service.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_service(&self, service_name: &str) -> Result<Vec<HostInfo>> {
        let pattern = format!("%{}%", service_name);
        let rows = sqlx::query(
            r#"
            SELECT target_ip, port, service, state
            FROM scan_results
            WHERE service LIKE ? AND state = 'open'
            ORDER BY target_ip, port
            "#,
        )
        .bind(pattern)
        .fetch_all(&self.storage.pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to query by service: {}", e)))?;

        let mut hosts = Vec::with_capacity(rows.len());
        for row in rows {
            let target_ip_str: String = row.get(0);
            let target_ip: IpAddr = target_ip_str
                .parse()
                .map_err(|e| Error::Parse(format!("Invalid IP address in database: {}", e)))?;

            let port: i64 = row.get(1);
            let state_str: String = row.get(3);
            let state = match state_str.as_str() {
                "open" => PortState::Open,
                "closed" => PortState::Closed,
                "filtered" => PortState::Filtered,
                _ => PortState::Unknown,
            };

            hosts.push(HostInfo {
                target_ip,
                port: port as u16,
                service: row.get(2),
                version: None,
                state,
            });
        }

        Ok(hosts)
    }

    /// Compare two scans to identify changes
    ///
    /// Analyzes differences between two scans:
    /// - New open ports
    /// - Closed ports
    /// - Changed service versions
    /// - New/disappeared hosts
    ///
    /// # Arguments
    ///
    /// * `scan_id1` - First scan ID (baseline)
    /// * `scan_id2` - Second scan ID (comparison)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # let reader = prtip_scanner::DbReader::new(":memory:").await?;
    /// let comparison = reader.compare_scans(1, 2).await?;
    /// println!("{} new open ports", comparison.new_open_ports.len());
    /// println!("{} closed ports", comparison.closed_ports.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn compare_scans(&self, scan_id1: i64, scan_id2: i64) -> Result<ScanComparison> {
        // Get results from both scans
        let results1 = self.storage.get_scan_results(scan_id1).await?;
        let results2 = self.storage.get_scan_results(scan_id2).await?;

        // Build lookup maps (key: "ip:port")
        let mut map1: HashMap<String, &ScanResult> = HashMap::new();
        let mut map2: HashMap<String, &ScanResult> = HashMap::new();

        for result in &results1 {
            let key = format!("{}:{}", result.target_ip, result.port);
            map1.insert(key, result);
        }

        for result in &results2 {
            let key = format!("{}:{}", result.target_ip, result.port);
            map2.insert(key, result);
        }

        // Find new open ports (in scan2, not in scan1, or changed from closed to open)
        let mut new_open_ports = Vec::new();
        for result in &results2 {
            if result.state != PortState::Open {
                continue;
            }

            let key = format!("{}:{}", result.target_ip, result.port);
            match map1.get(&key) {
                None => new_open_ports.push(result.clone()),
                Some(old_result) if old_result.state != PortState::Open => {
                    new_open_ports.push(result.clone())
                }
                _ => {}
            }
        }

        // Find closed ports (open in scan1, not open in scan2)
        let mut closed_ports = Vec::new();
        for result in &results1 {
            if result.state != PortState::Open {
                continue;
            }

            let key = format!("{}:{}", result.target_ip, result.port);
            match map2.get(&key) {
                None => closed_ports.push(result.clone()),
                Some(new_result) if new_result.state != PortState::Open => {
                    closed_ports.push(result.clone())
                }
                _ => {}
            }
        }

        // Find changed services (same port, different service/version)
        let mut changed_services = Vec::new();
        for result in &results2 {
            let key = format!("{}:{}", result.target_ip, result.port);
            if let Some(old_result) = map1.get(&key) {
                if old_result.service != result.service || old_result.version != result.version {
                    changed_services.push(((*old_result).clone(), result.clone()));
                }
            }
        }

        // Find new/disappeared hosts
        let mut hosts1: Vec<IpAddr> = results1.iter().map(|r| r.target_ip).collect();
        hosts1.sort();
        hosts1.dedup();

        let mut hosts2: Vec<IpAddr> = results2.iter().map(|r| r.target_ip).collect();
        hosts2.sort();
        hosts2.dedup();

        let new_hosts: Vec<IpAddr> = hosts2
            .iter()
            .filter(|ip| !hosts1.contains(ip))
            .copied()
            .collect();

        let disappeared_hosts: Vec<IpAddr> = hosts1
            .iter()
            .filter(|ip| !hosts2.contains(ip))
            .copied()
            .collect();

        Ok(ScanComparison {
            scan1_id: scan_id1,
            scan2_id: scan_id2,
            new_open_ports,
            closed_ports,
            changed_services,
            new_hosts,
            disappeared_hosts,
        })
    }

    /// Close the database connection
    pub async fn close(self) {
        self.storage.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::time::Duration;

    async fn create_test_db() -> DbReader {
        DbReader::new(":memory:").await.unwrap()
    }

    async fn populate_test_data(reader: &DbReader) -> (i64, i64) {
        // Create two scans
        let scan1 = reader
            .storage
            .create_scan(r#"{"target": "192.168.1.0/24"}"#)
            .await
            .unwrap();
        let scan2 = reader
            .storage
            .create_scan(r#"{"target": "192.168.1.0/24"}"#)
            .await
            .unwrap();

        // Add results to scan1
        let results1 = vec![
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                80,
                PortState::Open,
            )
            .with_service("http".to_string())
            .with_response_time(Duration::from_millis(100)),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                443,
                PortState::Open,
            )
            .with_service("https".to_string())
            .with_response_time(Duration::from_millis(150)),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
                22,
                PortState::Open,
            )
            .with_service("ssh".to_string())
            .with_response_time(Duration::from_millis(50)),
        ];
        reader
            .storage
            .store_results_batch(scan1, &results1)
            .await
            .unwrap();

        // Add results to scan2 (with some changes)
        let results2 = vec![
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                80,
                PortState::Open,
            )
            .with_service("http".to_string())
            .with_response_time(Duration::from_millis(120)),
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                443,
                PortState::Closed,
            )
            .with_response_time(Duration::from_millis(10)), // Port closed
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                8080,
                PortState::Open,
            )
            .with_service("http-alt".to_string())
            .with_response_time(Duration::from_millis(200)), // New port
            ScanResult::new(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)),
                80,
                PortState::Open,
            )
            .with_service("http".to_string())
            .with_response_time(Duration::from_millis(100)), // New host
        ];
        reader
            .storage
            .store_results_batch(scan2, &results2)
            .await
            .unwrap();

        (scan1, scan2)
    }

    #[tokio::test]
    async fn test_list_scans() {
        let reader = create_test_db().await;
        populate_test_data(&reader).await;

        let scans = reader.list_scans().await.unwrap();
        assert_eq!(scans.len(), 2);
        assert_eq!(scans[0].result_count, 4); // scan2 has 4 results
        assert_eq!(scans[1].result_count, 3); // scan1 has 3 results
    }

    #[tokio::test]
    async fn test_query_open_ports() {
        let reader = create_test_db().await;
        populate_test_data(&reader).await;

        let ports = reader.query_open_ports("192.168.1.1").await.unwrap();
        // Ports 80 (both scans), 443 (scan1), 8080 (scan2) - all open across history
        assert_eq!(ports.len(), 3);

        assert!(ports.iter().any(|p| p.port == 80));
        assert!(ports.iter().any(|p| p.port == 443)); // Was open in scan1
        assert!(ports.iter().any(|p| p.port == 8080));
    }

    #[tokio::test]
    async fn test_query_by_port() {
        let reader = create_test_db().await;
        populate_test_data(&reader).await;

        let hosts = reader.query_by_port(80).await.unwrap();
        assert_eq!(hosts.len(), 2); // 192.168.1.1 and 192.168.1.3

        let ips: Vec<String> = hosts.iter().map(|h| h.target_ip.to_string()).collect();
        assert!(ips.contains(&"192.168.1.1".to_string()));
        assert!(ips.contains(&"192.168.1.3".to_string()));
    }

    #[tokio::test]
    async fn test_query_by_service() {
        let reader = create_test_db().await;
        populate_test_data(&reader).await;

        let hosts = reader.query_by_service("http").await.unwrap();
        assert!(hosts.len() >= 2); // At least 192.168.1.1:80 and 192.168.1.3:80

        // Should match "http" and "http-alt"
        let services: Vec<String> = hosts.iter().filter_map(|h| h.service.clone()).collect();
        assert!(services.iter().any(|s| s.contains("http")));
    }

    #[tokio::test]
    async fn test_compare_scans() {
        let reader = create_test_db().await;
        let (scan1, scan2) = populate_test_data(&reader).await;

        let comparison = reader.compare_scans(scan1, scan2).await.unwrap();

        // New open ports: 192.168.1.1:8080, 192.168.1.3:80
        assert_eq!(comparison.new_open_ports.len(), 2);

        // Closed ports: 192.168.1.1:443, 192.168.1.2:22
        assert_eq!(comparison.closed_ports.len(), 2);

        // New hosts: 192.168.1.3
        assert_eq!(comparison.new_hosts.len(), 1);
        assert!(comparison
            .new_hosts
            .contains(&"192.168.1.3".parse().unwrap()));

        // Disappeared hosts: 192.168.1.2
        assert_eq!(comparison.disappeared_hosts.len(), 1);
        assert!(comparison
            .disappeared_hosts
            .contains(&"192.168.1.2".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_get_scan_results() {
        let reader = create_test_db().await;
        let (scan1, _) = populate_test_data(&reader).await;

        let results = reader.get_scan_results(scan1).await.unwrap();
        assert_eq!(results.len(), 3);
    }
}
