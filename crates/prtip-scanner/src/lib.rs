//! ProRT-IP Scanner Engine
//!
//! This crate provides the core scanning functionality for ProRT-IP WarScan,
//! including TCP, UDP, stealth scanning, service detection, and TLS analysis.
//!
//! # Overview
//!
//! The scanner engine combines the speed of Masscan with the depth of Nmap,
//! providing:
//!
//! - **8 Scan Types**: SYN, Connect, UDP, FIN/NULL/Xmas, ACK, Idle/Zombie
//! - **Service Detection**: 85-90% accuracy with 187+ protocol probes
//! - **TLS Analysis**: X.509v3 certificate parsing, chain validation, cipher detection
//! - **Stealth Techniques**: Fragmentation, decoys, timing control, TTL manipulation
//! - **Rate Limiting**: Industry-leading -1.8% overhead with V3 algorithm
//! - **IPv6 Support**: Dual-stack with <15% performance overhead
//! - **Plugin System**: Lua-based extensibility with sandboxing
//!
//! # Architecture
//!
//! The scanner engine is organized into specialized modules:
//!
//! ## Core Scanners
//!
//! - [`tcp_connect`]: TCP connect scan using OS sockets (portable, no privileges)
//! - [`syn_scanner`]: TCP SYN scan (half-open) using raw packets (requires root)
//! - [`udp_scanner`]: UDP scan with protocol-specific payloads (DNS, SNMP, etc.)
//! - [`stealth_scanner`]: Stealth scans (FIN, NULL, Xmas, ACK) for firewall detection
//! - [`idle`]: Idle/Zombie scan for IP spoofing and anonymity
//!
//! ## Discovery & Detection
//!
//! - [`discovery`]: Host discovery via ICMP, ARP, TCP SYN pings, and NDP
//! - [`service_detector`]: Service fingerprinting with nmap-service-probes
//! - [`os_fingerprinter`]: OS detection with 2,600+ fingerprints
//! - [`tls_certificate`]: X.509v3 certificate parsing and chain validation
//! - [`banner_grabber`]: Application banner collection and analysis
//!
//! ## Performance & Control
//!
//! - [`adaptive_rate_limiter_v3`]: V3 rate limiter with -1.8% overhead
//! - [`timing`]: Timing templates (T0-T5) and adaptive rate control
//! - [`scheduler`]: High-level scan orchestration and coordination
//! - [`concurrent_scanner`]: Parallel scanning with bounded concurrency
//!
//! ## Storage & Output
//!
//! - [`storage`]: Async SQLite storage for scan results
//! - [`pcapng`]: PCAPNG packet capture format for Wireshark analysis
//! - [`memory_storage`]: In-memory storage for performance testing
//!
//! ## Advanced Features
//!
//! - [`decoy_scanner`]: Decoy scanning to obscure scan source
//! - [`plugin`]: Lua plugin system with sandboxing and capabilities
//! - [`icmp_monitor`]: ICMP monitoring for rate limit detection
//!
//! # Quick Start
//!
//! ## TCP Connect Scan (No Root Required)
//!
//! ```no_run
//! use prtip_scanner::{TcpConnectScanner, MemoryStorage, StorageBackend};
//! use prtip_core::{Config, ScanTarget, PortRange};
//! use std::sync::Arc;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create in-memory storage
//! let storage = Arc::new(MemoryStorage::new());
//! let storage_backend = Arc::new(
//!     StorageBackend::async_memory(storage, "192.168.1.1".parse()?)
//! );
//!
//! // Create scanner
//! let scanner = TcpConnectScanner::new(Config::default(), storage_backend);
//!
//! // Scan common ports
//! let target = ScanTarget::parse("192.168.1.1")?;
//! let ports = PortRange::parse("80,443")?;
//!
//! scanner.scan(&target, &ports).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## SYN Scan with Service Detection (Requires Root)
//!
//! ```no_run
//! use prtip_scanner::{SynScanner, ServiceDetector, ScanStorage, StorageBackend};
//! use prtip_core::{Config, ScanTarget, PortRange, ScanType};
//! use std::sync::Arc;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create storage
//! let storage = Arc::new(ScanStorage::new("scan.db").await?);
//! let storage_backend = Arc::new(
//!     StorageBackend::async_database(storage.clone(), ScanType::Syn, "192.168.1.1").await?
//! );
//!
//! // Create scanners
//! let syn_scanner = SynScanner::new(Config::default(), storage_backend.clone()).await?;
//! let service_detector = ServiceDetector::new(
//!     "/usr/share/nmap/nmap-service-probes",
//!     storage.clone()
//! ).await?;
//!
//! // Scan and detect services
//! let target = ScanTarget::parse("192.168.1.1")?;
//! let ports = PortRange::parse("1-1000")?;
//!
//! syn_scanner.scan(&target, &ports).await?;
//!
//! // Detect services on open ports
//! let open_ports = vec![80, 443, 22];
//! for port in open_ports {
//!     let service = service_detector.detect_service(&target.to_ip()?, port).await?;
//!     println!("Port {}: {:?}", port, service);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Stealth Scan with Decoys
//!
//! ```no_run
//! use prtip_scanner::{DecoyScanner, MemoryStorage, StorageBackend};
//! use prtip_core::{Config, ScanTarget, PortRange, DecoyConfig};
//! use std::sync::Arc;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let mut config = Config::default();
//! config.evasion_config.decoy_config = DecoyConfig {
//!     enabled: true,
//!     count: 5,
//!     placement: prtip_scanner::DecoyPlacement::Random,
//! };
//!
//! let storage = Arc::new(MemoryStorage::new());
//! let storage_backend = Arc::new(
//!     StorageBackend::async_memory(storage, "192.168.1.1".parse()?)
//! );
//!
//! let scanner = DecoyScanner::new(config, storage_backend).await?;
//!
//! let target = ScanTarget::parse("192.168.1.1")?;
//! let ports = PortRange::parse("80,443")?;
//!
//! scanner.scan(&target, &ports).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## TLS Certificate Analysis
//!
//! ```no_run
//! use prtip_scanner::tls_certificate::{parse_certificate, validate_chain};
//! use std::net::IpAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Analyze TLS certificate
//! let target_ip: IpAddr = "8.8.8.8".parse()?;
//! let cert_chain = parse_certificate(target_ip, 443, None).await?;
//!
//! println!("Subject: {}", cert_chain.subject);
//! println!("Issuer: {}", cert_chain.issuer);
//! println!("Valid from: {:?}", cert_chain.not_before);
//! println!("Valid to: {:?}", cert_chain.not_after);
//!
//! // Validate certificate chain
//! let validation = validate_chain(&cert_chain, "google.com");
//! println!("Validation: {:?}", validation);
//! # Ok(())
//! # }
//! ```
//!
//! ## Plugin System
//!
//! ```no_run
//! use prtip_scanner::{PluginManager, Capability};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut plugin_manager = PluginManager::new();
//!
//! // Load plugin with specific capabilities
//! plugin_manager.load_plugin(
//!     "banner-analyzer",
//!     "/path/to/plugin",
//!     vec![Capability::Network]
//! ).await?;
//!
//! // Plugins can hook into scan lifecycle
//! println!("Loaded {} plugins", plugin_manager.plugin_count());
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `vendored-openssl`: Statically link OpenSSL (required for musl builds)
//! - `numa`: Enable NUMA optimizations for multi-socket systems
//! - `network-tests`: Enable integration tests that require internet access
//!
//! # Performance
//!
//! - **SYN Scan**: 10M+ packets/second (stateless)
//! - **Service Detection**: 85-90% accuracy, ~500ms per service
//! - **Rate Limiting**: -1.8% overhead (V3 algorithm)
//! - **IPv6**: <15% overhead vs IPv4
//! - **TLS Parsing**: 1.33μs average per certificate
//!
//! # Platform Support
//!
//! - **Linux**: Full support (raw sockets, NUMA, all features)
//! - **macOS**: Full support (requires root for raw sockets)
//! - **Windows**: Full support (requires Npcap for raw sockets)
//! - **BSD**: Partial support (tested on FreeBSD 13+)

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
