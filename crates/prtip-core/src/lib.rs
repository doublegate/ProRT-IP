//! ProRT-IP Core Library
//!
//! This crate provides the core types, error handling, and configuration
//! for the ProRT-IP WarScan network scanner.
//!
//! # Overview
//!
//! The `prtip-core` crate serves as the foundation for ProRT-IP WarScan, providing:
//!
//! - **Type System**: Port ranges, scan targets, protocols, scan types
//! - **Configuration**: Scan parameters, timing templates, output formats
//! - **Error Handling**: Comprehensive error types with context
//! - **Detection**: Protocol detectors for HTTP, MySQL, PostgreSQL, SMB, SSH
//! - **Utilities**: CDN detection, circuit breakers, retry mechanisms
//!
//! # Quick Start
//!
//! ```
//! use prtip_core::{PortRange, ScanTarget, Config, ScanType, TimingTemplate};
//!
//! // Parse port ranges (various formats supported)
//! let ports = PortRange::parse("80,443,8080-8090").unwrap();
//! assert_eq!(ports.count(), 13); // 1 + 1 + 11 = 13
//!
//! // Parse scan targets (IP, CIDR, hostname)
//! let target = ScanTarget::parse("192.168.1.0/24").unwrap();
//! assert!(!target.is_single_host());
//!
//! // Create configuration with timing template
//! let mut config = Config::default();
//! config.scan.scan_type = ScanType::Syn;
//! config.scan.timing_template = TimingTemplate::Aggressive;
//! assert!(config.validate().is_ok());
//! ```
//!
//! # Common Patterns
//!
//! ## Parsing Targets and Ports
//!
//! ```
//! use prtip_core::{ScanTarget, PortRange};
//!
//! // Multiple target formats
//! let single = ScanTarget::parse("192.168.1.1").unwrap();
//! let cidr = ScanTarget::parse("10.0.0.0/8").unwrap();
//! let range = ScanTarget::parse("172.16.0.0/24").unwrap();
//!
//! // Port range formats
//! let common = PortRange::parse("22,80,443").unwrap();
//! let range_ports = PortRange::parse("1-1000").unwrap();
//! let mixed = PortRange::parse("22,80,443,8000-9000").unwrap();
//! ```
//!
//! ## Service Detection
//!
//! ```
//! use prtip_core::ServiceInfo;
//!
//! // Service information with confidence
//! let service = ServiceInfo {
//!     service: "http".to_string(),
//!     product: Some("nginx".to_string()),
//!     version: Some("1.18.0".to_string()),
//!     info: Some("Ubuntu".to_string()),
//!     os_type: None,
//!     confidence: 0.95,
//! };
//! ```
//!
//! ## Error Handling
//!
//! ```
//! use prtip_core::{Result, Error};
//!
//! fn validate_port(port: u16) -> Result<u16> {
//!     if port == 0 {
//!         Err(Error::Config("Port cannot be 0".to_string()))
//!     } else {
//!         Ok(port)
//!     }
//! }
//!
//! assert!(validate_port(80).is_ok());
//! assert!(validate_port(0).is_err());
//! ```
//!
//! # Features
//!
//! All features are enabled by default. See the `Cargo.toml` for platform-specific features.

pub mod cdn_detector;
pub mod circuit_breaker;
pub mod config;
pub mod crypto;
pub mod detection;
pub mod error;
pub mod errors;
pub mod event_bus;
pub mod event_logger;
pub mod events;
pub mod os_db;
pub mod progress;
pub mod resource_limits;
pub mod resource_monitor;
pub mod retry;
pub mod service_db;
pub mod top_ports;
pub mod types;

// Re-export commonly used types
pub use cdn_detector::{CdnDetector, CdnProvider, Ipv4Cidr};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitStats};
pub use config::{
    Config, DecoyConfig, EvasionConfig, NetworkConfig, OutputConfig, OutputFormat,
    PerformanceConfig, ScanConfig, ServiceDetectionConfig,
};
pub use detection::{
    http_fingerprint::HttpFingerprint, mysql_detect::MysqlDetect,
    postgresql_detect::PostgresqlDetect, smb_detect::SmbDetect, ssh_banner::SshBanner,
    ProtocolDetector, ServiceInfo,
};
pub use error::{Error, Result};
pub use errors::{ScanError, ScanErrorKind};
pub use event_bus::{EventBus, EventFilter};
pub use event_logger::{EventLogger, EventLoggerConfig};
pub use events::{
    DiscoveryMethod, MetricType, PauseReason, ScanEvent, ScanEventType, ScanStage, Throughput,
    ValidationError, WarningSeverity,
};
pub use os_db::{OsFingerprint, OsFingerprintDb, ProbeResults};
pub use progress::{ErrorCategory, ScanProgress};
pub use resource_monitor::{
    AdaptiveConfig, ResourceMonitor, ResourceMonitorConfig, ResourceStatus,
};
pub use retry::{retry_with_backoff, RetryConfig};
pub use service_db::{ServiceMatch, ServiceProbe, ServiceProbeDb};
pub use types::{PortRange, PortState, Protocol, ScanResult, ScanTarget, ScanType, TimingTemplate};
