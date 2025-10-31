//! Protocol-specific service detection modules
//!
//! This module provides specialized parsers for common protocols that go beyond
//! simple regex matching. Each sub-module implements protocol-aware detection
//! to extract detailed version, OS, and configuration information.
//!
//! # Supported Protocols
//!
//! - **HTTP**: Header parsing (Server, X-Powered-By, X-AspNet-Version)
//! - **SSH**: Banner parsing with OS hints
//! - **SMB**: Dialect negotiation for Windows version detection
//! - **MySQL**: Handshake response parsing
//! - **PostgreSQL**: Startup message parsing
//!
//! # Architecture
//!
//! Each protocol module implements the `ProtocolDetector` trait, providing:
//! - `detect()`: Parse response and extract service information
//! - `confidence()`: Return confidence score (0.0-1.0)
//! - `priority()`: Return detection priority (1=highest)
//!
//! # Fallback Chain
//!
//! Detection follows a three-tier fallback chain:
//! 1. **Protocol-specific detection** (highest confidence)
//! 2. **Regex pattern matching** (medium confidence)
//! 3. **Generic service identification** (low confidence)

pub mod http_fingerprint;
pub mod mysql_detect;
pub mod postgresql_detect;
pub mod smb_detect;
pub mod ssh_banner;

use crate::Error;

/// Service detection information
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceInfo {
    /// Service name (e.g., "http", "ssh", "mysql")
    pub service: String,
    /// Product name (e.g., "Apache", "OpenSSH", "MySQL")
    pub product: Option<String>,
    /// Version string (e.g., "2.4.41", "8.2p1", "8.0.27")
    pub version: Option<String>,
    /// Extra information (e.g., "Ubuntu", "SSL/TLS", "protocol 10")
    pub info: Option<String>,
    /// OS type (e.g., "Linux", "Windows", "FreeBSD")
    pub os_type: Option<String>,
    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
}

impl ServiceInfo {
    /// Create new ServiceInfo with just service name
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            product: None,
            version: None,
            info: None,
            os_type: None,
            confidence: 0.5, // Default medium confidence
        }
    }

    /// Set product name
    pub fn with_product(mut self, product: impl Into<String>) -> Self {
        self.product = Some(product.into());
        self
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set info
    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.info = Some(info.into());
        self
    }

    /// Set OS type
    pub fn with_os(mut self, os_type: impl Into<String>) -> Self {
        self.os_type = Some(os_type.into());
        self
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Protocol-specific detector trait
pub trait ProtocolDetector {
    /// Detect service from response bytes
    fn detect(&self, response: &[u8]) -> Result<Option<ServiceInfo>, Error>;

    /// Get detector confidence level (0.0-1.0)
    fn confidence(&self) -> f32;

    /// Get detector priority (1=highest, lower numbers run first)
    fn priority(&self) -> u8;
}
