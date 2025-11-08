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
//! use prtip_scanner::TcpConnectScanner;
//! use std::time::Duration;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create scanner with 2s timeout, 1 retry
//! let scanner = TcpConnectScanner::new(Duration::from_secs(2), 1);
//!
//! // Scan common ports on target
//! let target = "192.168.1.1".parse().unwrap();
//! let ports = vec![80, 443, 8080];
//! let results = scanner.scan_ports(target, ports, 10).await?;
//!
//! for result in results {
//!     println!("Port {}: {:?}", result.port, result.state);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## SYN Scan with Service Detection (Requires Root)
//!
//! ```no_run
//! use prtip_scanner::{SynScanner, service_detector::ServiceDetector};
//! use prtip_core::{Config, ServiceProbeDb};
//! use std::net::{IpAddr, SocketAddr};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create SYN scanner
//! let scanner = SynScanner::new(Config::default())?;
//!
//! // Load service detection database
//! let probe_data = std::fs::read_to_string("/usr/share/nmap/nmap-service-probes")?;
//! let db = ServiceProbeDb::parse(&probe_data)?;
//! let service_detector = ServiceDetector::new(db, 7);
//!
//! // Scan ports
//! let target: IpAddr = "192.168.1.1".parse()?;
//! let ports = vec![80, 443, 22];
//! let results = scanner.scan_ports(target, ports.clone()).await?;
//!
//! // Detect services on open ports
//! for result in results.iter().filter(|r| r.state == prtip_core::PortState::Open) {
//!     let addr = SocketAddr::new(target, result.port);
//!     let service = service_detector.detect_service(addr).await?;
//!     let version = service.version.as_deref().unwrap_or("unknown");
//!     println!("Port {}: {} ({})", result.port, service.service, version);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Stealth Scan with Decoys
//!
//! ```no_run
//! use prtip_scanner::DecoyScanner;
//! use prtip_core::{Config, ScanTarget};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create decoy scanner and configure decoys
//! let mut scanner = DecoyScanner::new(Config::default());
//! scanner.set_random_decoys(5); // Generate 5 random decoy IPs
//!
//! // Scan target port (packets sent from decoy IPs)
//! let target = ScanTarget::parse("192.168.1.1")?;
//! let result = scanner.scan_with_decoys(target, 80).await?;
//!
//! println!("Port 80: {:?} (scanned via decoys)", result.state);
//! # Ok(())
//! # }
//! ```
//!
//! ## TLS Certificate Analysis
//!
//! ```no_run
//! use prtip_scanner::tls_handshake::TlsHandshake;
//! use prtip_scanner::tls_certificate::parse_certificate;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Perform TLS handshake and get server info
//! let tls_handler = TlsHandshake::new();
//! let server_info = tls_handler.connect("example.com", 443).await?;
//!
//! println!("Server: {}", server_info.common_name);
//! println!("Issuer: {}", server_info.issuer);
//! println!("TLS Version: {}", server_info.tls_version);
//! println!("Expiry: {:?}", server_info.expiry);
//!
//! // Parse leaf certificate for detailed analysis
//! if let Some(leaf_cert) = server_info.raw_cert_chain.first() {
//!     let cert_info = parse_certificate(leaf_cert)?;
//!     println!("Key Usage: {:?}", cert_info.key_usage);
//!     println!("SAN: {:?}", cert_info.san_categorized.dns_names);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Plugin System
//!
//! ```no_run
//! use prtip_scanner::plugin::PluginManager;
//!
//! # fn main() -> prtip_core::Result<()> {
//! // Create plugin manager with default directory (~/.prtip/plugins/)
//! let mut manager = PluginManager::with_default_dir()?;
//!
//! // Discover all available plugins
//! manager.discover_plugins()?;
//!
//! // Load specific plugin
//! manager.load_plugin("banner-analyzer")?;
//!
//! // List loaded plugins
//! for name in manager.list_loaded() {
//!     println!("Loaded plugin: {}", name);
//! }
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
