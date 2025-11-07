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
//! let results = scheduler.execute_scan(targets, None).await?;
//!
//! println!("Scan complete: {} results", results.len());
//! # Ok(())
//! # }
//! ```

pub mod adaptive_parallelism;
pub mod adaptive_rate_limiter; // Keep for ICMP backoff functionality (Sprint 5.4)
pub mod adaptive_rate_limiter_v3;
pub mod async_storage;
pub mod banner_grabber;
pub mod concurrent_scanner;
pub mod connection_pool;
pub mod db_reader;
pub mod decoy_scanner;
pub mod discovery;
pub mod error;
pub mod hostgroup_limiter;
pub mod icmp_monitor;
pub mod idle;
pub mod lockfree_aggregator;
pub mod memory_storage;
pub mod os_fingerprinter;
pub mod os_probe;
pub mod pcapng;
pub mod plugin;
pub mod progress_bar;
pub mod scheduler;
pub mod service_detector;
pub mod stealth_scanner;
pub mod storage;
pub mod storage_backend;
pub mod syn_scanner;
pub mod tcp_connect;
pub mod timing;
pub mod tls_certificate;
pub mod tls_handshake;
pub mod udp_scanner;

pub use adaptive_rate_limiter::{AdaptiveRateLimiter as AdaptiveRateLimiterV2, RateLimiterStats}; // ICMP backoff
pub use adaptive_rate_limiter_v3::AdaptiveRateLimiterV3;
// Type alias for backward compatibility - V3 is now the default rate limiter
pub type RateLimiter = AdaptiveRateLimiterV3;
pub use async_storage::async_storage_worker;
pub use banner_grabber::{BannerGrabber, BannerParser};
pub use concurrent_scanner::ConcurrentScanner;
pub use connection_pool::ConnectionPool;
pub use db_reader::{DbReader, HostInfo, PortInfo, ScanComparison, ScanInfo};
pub use decoy_scanner::{DecoyPlacement, DecoyScanner, MAX_DECOYS};
pub use discovery::{DiscoveryEngine, DiscoveryMethod};
pub use error::{ErrorCategory, ScannerError, ScannerResult};
pub use hostgroup_limiter::{HostgroupConfig, HostgroupLimiter, TargetPermit};
pub use icmp_monitor::{BackoffState, IcmpError, IcmpMonitor};
pub use idle::{
    DiscoveryConfig as ZombieDiscoveryConfig, IPIDMeasurement, IPIDPattern, IPIDTracker,
    IdleScanConfig, IdleScanResult, IdleScanner, ZombieCandidate, ZombieDiscovery,
};
pub use lockfree_aggregator::LockFreeAggregator;
pub use memory_storage::MemoryStorage;
pub use os_fingerprinter::{OsDetectionResult, OsFingerprinter};
pub use os_probe::OsProbeEngine;
pub use pcapng::{Direction, PcapngWriter};
pub use plugin::{
    Capability, DetectionPlugin, LuaContext, OutputPlugin, Plugin, PluginCapabilities,
    PluginManager, PluginType, ResourceLimits, ScanPlugin, SecurityError,
};
pub use progress_bar::ScanProgressBar;
pub use scheduler::ScanScheduler;
pub use service_detector::{ServiceDetector, ServiceInfo};
pub use stealth_scanner::{StealthScanType, StealthScanner};
pub use storage::ScanStorage;
pub use storage_backend::StorageBackend;
pub use syn_scanner::SynScanner;
pub use tcp_connect::TcpConnectScanner;
pub use timing::{AdaptiveRateLimiter, TimingConfig};
pub use tls_certificate::{
    categorize_chain, parse_certificate, parse_certificate_chain, validate_chain,
    validate_chain_comprehensive, CertificateChain, CertificateExtension, CertificateInfo,
    ChainCategories, CipherStrength, CipherSuite, ExtendedKeyUsage, KeyUsage, PublicKeyInfo,
    SecurityStrength, ServerHello, SignatureAlgorithm, SubjectAlternativeName, TlsAnalysisResult,
    TlsExtension, TlsExtensionData, TlsFingerprint, TlsVersion, ValidationResult,
};
pub use tls_handshake::{ServerInfo as TlsServerInfo, TlsHandshake};
pub use udp_scanner::UdpScanner;
