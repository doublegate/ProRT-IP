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
//! - [`discovery`]: Host discovery via ICMP, ARP, and TCP SYN pings
//! - [`rate_limiter`]: Token bucket rate limiting to control scan speed
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

pub mod discovery;
pub mod rate_limiter;
pub mod scheduler;
pub mod storage;
pub mod tcp_connect;

pub use discovery::{DiscoveryEngine, DiscoveryMethod};
pub use rate_limiter::RateLimiter;
pub use scheduler::ScanScheduler;
pub use storage::ScanStorage;
pub use tcp_connect::TcpConnectScanner;
