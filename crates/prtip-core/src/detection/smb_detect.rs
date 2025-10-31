//! SMB dialect negotiation for Windows version detection
//!
//! This module analyzes SMB protocol responses to determine the SMB dialect
//! and infer the Windows version based on supported dialects.
//!
//! # SMB Dialect Mapping
//!
//! | Dialect Code | SMB Version | Windows Version |
//! |--------------|-------------|-----------------|
//! | 0x0202       | SMB 1.0     | Windows XP/2003 |
//! | 0x02FF       | SMB 2.002   | Windows Vista/2008 |
//! | 0x0210       | SMB 2.1     | Windows 7/2008 R2 |
//! | 0x0300       | SMB 3.0     | Windows 8/2012 |
//! | 0x0302       | SMB 3.02    | Windows 8.1/2012 R2 |
//! | 0x0311       | SMB 3.11    | Windows 10/2016+ |
//!
//! # Examples
//!
//! ```
//! use prtip_core::detection::smb_detect::SmbDetect;
//! use prtip_core::detection::ProtocolDetector;
//!
//! // SMB3.11 response (Windows 10/2016+)
//! let response = &[
//!     0xFE, 0x53, 0x4D, 0x42, // SMB2/3 magic "\xFESMB"
//!     // ... dialect negotiation response ...
//! ];
//!
//! let detector = SmbDetect::new();
//! if let Ok(Some(info)) = detector.detect(response) {
//!     assert_eq!(info.service, "microsoft-ds");
//!     // Windows version inferred from dialect
//! }
//! ```

use super::{ProtocolDetector, ServiceInfo};
use crate::Error;

/// SMB dialect detector
#[derive(Debug, Clone)]
pub struct SmbDetect {
    priority: u8,
}

impl SmbDetect {
    /// Create new SMB detector
    pub fn new() -> Self {
        Self { priority: 3 }
    }

    /// Check if response is SMB2/3 protocol
    ///
    /// SMB2/3 responses start with magic bytes: 0xFE 'S' 'M' 'B'
    fn is_smb2_response(response: &[u8]) -> bool {
        response.len() >= 4
            && response[0] == 0xFE
            && response[1] == b'S'
            && response[2] == b'M'
            && response[3] == b'B'
    }

    /// Check if response is SMB1 protocol
    ///
    /// SMB1 responses start with magic bytes: 0xFF 'S' 'M' 'B'
    fn is_smb1_response(response: &[u8]) -> bool {
        response.len() >= 4
            && response[0] == 0xFF
            && response[1] == b'S'
            && response[2] == b'M'
            && response[3] == b'B'
    }

    /// Extract SMB2/3 dialect from negotiate response
    ///
    /// Dialect code is at offset 0x48 (72 bytes) in SMB2 Negotiate Response
    /// Returns dialect code as u16 (little-endian)
    fn extract_smb2_dialect(response: &[u8]) -> Option<u16> {
        // SMB2 Negotiate Response structure (simplified):
        // 0x00: Header (64 bytes)
        // 0x40: StructureSize (2 bytes, should be 0x0041)
        // 0x42: SecurityMode (2 bytes)
        // 0x44: DialectRevision (2 bytes) <-- what we want
        const DIALECT_OFFSET: usize = 0x44;

        if response.len() < DIALECT_OFFSET + 2 {
            return None;
        }

        // Read dialect as little-endian u16
        let dialect = u16::from_le_bytes([response[DIALECT_OFFSET], response[DIALECT_OFFSET + 1]]);

        Some(dialect)
    }

    /// Map SMB dialect code to version string and Windows version
    ///
    /// Returns (smb_version, windows_version, confidence)
    fn map_dialect_to_version(dialect: u16) -> (String, String, f32) {
        match dialect {
            0x0202 => ("SMB 1.0".to_string(), "Windows XP/2003".to_string(), 0.75),
            0x02FF => (
                "SMB 2.002".to_string(),
                "Windows Vista/2008".to_string(),
                0.8,
            ),
            0x0210 => ("SMB 2.1".to_string(), "Windows 7/2008 R2".to_string(), 0.85),
            0x0300 => ("SMB 3.0".to_string(), "Windows 8/2012".to_string(), 0.9),
            0x0302 => (
                "SMB 3.02".to_string(),
                "Windows 8.1/2012 R2".to_string(),
                0.9,
            ),
            0x0311 => ("SMB 3.11".to_string(), "Windows 10/2016+".to_string(), 0.95),
            _ => (
                format!("SMB (dialect 0x{:04X})", dialect),
                "Windows (unknown version)".to_string(),
                0.6,
            ),
        }
    }
}

impl Default for SmbDetect {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolDetector for SmbDetect {
    fn detect(&self, response: &[u8]) -> Result<Option<ServiceInfo>, Error> {
        // Check for SMB2/3 response
        if Self::is_smb2_response(response) {
            // Extract dialect
            if let Some(dialect) = Self::extract_smb2_dialect(response) {
                let (smb_version, windows_version, confidence) =
                    Self::map_dialect_to_version(dialect);

                let mut service_info = ServiceInfo::new("microsoft-ds");
                service_info.product = Some("Samba/Windows SMB".to_string());
                service_info.version = Some(smb_version.clone());
                service_info.os_type = Some(windows_version.clone());
                service_info.info = Some(format!("{} ({})", smb_version, windows_version));
                service_info.confidence = confidence;

                return Ok(Some(service_info));
            }
        }

        // Check for SMB1 response (legacy)
        if Self::is_smb1_response(response) {
            let mut service_info = ServiceInfo::new("microsoft-ds");
            service_info.product = Some("SMB".to_string());
            service_info.version = Some("SMB 1.0".to_string());
            service_info.os_type = Some("Windows XP/2003 or Samba".to_string());
            service_info.info = Some("SMB 1.0 (legacy protocol)".to_string());
            service_info.confidence = 0.7;

            return Ok(Some(service_info));
        }

        // Not an SMB response
        Ok(None)
    }

    fn confidence(&self) -> f32 {
        0.8 // SMB responses are fairly reliable
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smb311_detection() {
        // Minimal SMB3.11 Negotiate Response
        let mut response = vec![0u8; 0x46];
        response[0..4].copy_from_slice(&[0xFE, b'S', b'M', b'B']); // SMB2/3 magic
        response[0x44..0x46].copy_from_slice(&[0x11, 0x03]); // Dialect 0x0311 (SMB 3.11)

        let detector = SmbDetect::new();
        let result = detector.detect(&response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "microsoft-ds");
        assert_eq!(info.version, Some("SMB 3.11".to_string()));
        assert_eq!(info.os_type, Some("Windows 10/2016+".to_string()));
        assert!(info.confidence >= 0.9);
    }

    #[test]
    fn test_smb21_detection() {
        // Minimal SMB 2.1 Negotiate Response
        let mut response = vec![0u8; 0x46];
        response[0..4].copy_from_slice(&[0xFE, b'S', b'M', b'B']);
        response[0x44..0x46].copy_from_slice(&[0x10, 0x02]); // Dialect 0x0210 (SMB 2.1)

        let detector = SmbDetect::new();
        let result = detector.detect(&response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "microsoft-ds");
        assert_eq!(info.version, Some("SMB 2.1".to_string()));
        assert_eq!(info.os_type, Some("Windows 7/2008 R2".to_string()));
        assert!(info.confidence >= 0.8);
    }

    #[test]
    fn test_smb1_detection() {
        // Minimal SMB1 response
        let mut response = vec![0u8; 64];
        response[0..4].copy_from_slice(&[0xFF, b'S', b'M', b'B']); // SMB1 magic

        let detector = SmbDetect::new();
        let result = detector.detect(&response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "microsoft-ds");
        assert_eq!(info.version, Some("SMB 1.0".to_string()));
        assert!(info.confidence >= 0.7);
    }
}
