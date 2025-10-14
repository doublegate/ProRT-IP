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
//! use prtip_scanner::{TcpConnectScanner, ScanStorage, ScanScheduler, StorageBackend};
//! use prtip_core::{Config, ScanTarget, ScanType};
//! use std::sync::Arc;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create storage backend
//! let storage = Arc::new(ScanStorage::new("scan_results.db").await?);
//! let storage_backend = Arc::new(
//!     StorageBackend::async_database(storage, ScanType::Connect, "192.168.1.1").await?
//! );
//!
//! // Create configuration
//! let config = Config::default();
//!
//! // Create scheduler
//! let scheduler = ScanScheduler::new(config, storage_backend).await?;
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

pub mod adaptive_parallelism;
pub mod adaptive_rate_limiter;
pub mod async_storage;
pub mod banner_grabber;
pub mod concurrent_scanner;
pub mod connection_pool;
pub mod decoy_scanner;
pub mod discovery;
pub mod lockfree_aggregator;
pub mod memory_storage;
pub mod os_fingerprinter;
pub mod os_probe;
pub mod pcapng;
pub mod progress_bar;
pub mod rate_limiter;
pub mod scheduler;
pub mod service_detector;
pub mod stealth_scanner;
pub mod storage;
pub mod storage_backend;
pub mod syn_scanner;
pub mod tcp_connect;
pub mod timing;
pub mod tls_handshake;
pub mod udp_scanner;

pub use adaptive_rate_limiter::{AdaptiveRateLimiter as AdaptiveRateLimiterV2, RateLimiterStats};
pub use async_storage::async_storage_worker;
pub use banner_grabber::{BannerGrabber, BannerParser};
pub use concurrent_scanner::ConcurrentScanner;
pub use connection_pool::ConnectionPool;
pub use decoy_scanner::{DecoyPlacement, DecoyScanner, MAX_DECOYS};
pub use discovery::{DiscoveryEngine, DiscoveryMethod};
pub use lockfree_aggregator::LockFreeAggregator;
pub use memory_storage::MemoryStorage;
pub use os_fingerprinter::{OsDetectionResult, OsFingerprinter};
pub use os_probe::OsProbeEngine;
pub use pcapng::{Direction, PcapngWriter};
pub use progress_bar::ScanProgressBar;
pub use rate_limiter::RateLimiter;
pub use scheduler::ScanScheduler;
pub use service_detector::{ServiceDetector, ServiceInfo};
pub use stealth_scanner::{StealthScanType, StealthScanner};
pub use storage::ScanStorage;
pub use storage_backend::StorageBackend;
pub use syn_scanner::SynScanner;
pub use tcp_connect::TcpConnectScanner;
pub use timing::{AdaptiveRateLimiter, TimingConfig};
pub use tls_handshake::{ServerInfo as TlsServerInfo, TlsHandshake};
pub use udp_scanner::UdpScanner;
