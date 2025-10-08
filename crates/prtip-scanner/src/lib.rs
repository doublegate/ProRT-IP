//! ProRT-IP Scanner Engine
//!
//! This crate provides the core scanning functionality for ProRT-IP WarScan,
//! including TCP connect scanning, host discovery, rate limiting, and result storage.
//!
//! # Architecture
//!
//! The scanner engine is organized into several specialized modules:
//!
//! - [`tcp_connect`]: TCP connect scan implementation using OS sockets
//! - [`syn_scanner`]: TCP SYN scan (half-open) using raw packets
//! - [`udp_scanner`]: UDP scan with protocol-specific payloads
//! - [`stealth_scanner`]: Stealth scans (FIN, NULL, Xmas, ACK)
//! - [`discovery`]: Host discovery via ICMP, ARP, and TCP SYN pings
//! - [`rate_limiter`]: Token bucket rate limiting to control scan speed
//! - [`timing`]: Advanced timing templates and adaptive rate control
//! - [`storage`]: Async SQLite storage for scan results
//! - [`scheduler`]: High-level scan orchestration and coordination
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::{TcpConnectScanner, ScanStorage, ScanScheduler};
//! use prtip_core::{Config, ScanTarget};
//! use std::time::Duration;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create storage
//! let storage = ScanStorage::new("scan_results.db").await?;
//!
//! // Create configuration
//! let config = Config::default();
//!
//! // Create scheduler
//! let scheduler = ScanScheduler::new(config, storage).await?;
//!
//! // Define targets
//! let targets = vec![ScanTarget::parse("192.168.1.1")?];
//!
//! // Execute scan
//! let results = scheduler.execute_scan(targets).await?;
//!
//! println!("Scan complete: {} results", results.len());
//! # Ok(())
//! # }
//! ```

pub mod adaptive_rate_limiter;
pub mod concurrent_scanner;
pub mod connection_pool;
pub mod discovery;
pub mod rate_limiter;
pub mod scheduler;
pub mod stealth_scanner;
pub mod storage;
pub mod syn_scanner;
pub mod tcp_connect;
pub mod timing;
pub mod udp_scanner;

pub use adaptive_rate_limiter::{AdaptiveRateLimiter as AdaptiveRateLimiterV2, RateLimiterStats};
pub use concurrent_scanner::ConcurrentScanner;
pub use connection_pool::ConnectionPool;
pub use discovery::{DiscoveryEngine, DiscoveryMethod};
pub use rate_limiter::RateLimiter;
pub use scheduler::ScanScheduler;
pub use stealth_scanner::{StealthScanType, StealthScanner};
pub use storage::ScanStorage;
pub use syn_scanner::SynScanner;
pub use tcp_connect::TcpConnectScanner;
pub use timing::{AdaptiveRateLimiter, TimingConfig};
pub use udp_scanner::UdpScanner;
