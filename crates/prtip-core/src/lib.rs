//! ProRT-IP Core Library
//!
//! This crate provides the core types, error handling, and configuration
//! for the ProRT-IP WarScan network scanner.
//!
//! # Examples
//!
//! ```
//! use prtip_core::{PortRange, ScanTarget, Config};
//!
//! // Parse port ranges
//! let ports = PortRange::parse("80,443,8080-8090").unwrap();
//! assert_eq!(ports.count(), 13); // 1 + 1 + 11 = 13
//!
//! // Parse scan targets
//! let target = ScanTarget::parse("192.168.1.0/24").unwrap();
//! assert!(!target.is_single_host());
//!
//! // Create default configuration
//! let config = Config::default();
//! assert!(config.validate().is_ok());
//! ```

pub mod cdn_detector;
pub mod config;
pub mod crypto;
pub mod error;
pub mod errors;
pub mod os_db;
pub mod progress;
pub mod resource_limits;
pub mod service_db;
pub mod top_ports;
pub mod types;

// Re-export commonly used types
pub use cdn_detector::{CdnDetector, CdnProvider, Ipv4Cidr};
pub use config::{
    Config, NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, ScanConfig,
    ServiceDetectionConfig,
};
pub use error::{Error, Result};
pub use errors::{ScanError, ScanErrorKind};
pub use os_db::{OsFingerprint, OsFingerprintDb, ProbeResults};
pub use progress::{ErrorCategory, ScanProgress};
pub use service_db::{ServiceMatch, ServiceProbe, ServiceProbeDb};
pub use types::{PortRange, PortState, Protocol, ScanResult, ScanTarget, ScanType, TimingTemplate};
