//! Complete OS fingerprinting implementation
//!
//! This module combines the OS probe engine with fingerprint matching
//! to provide complete OS detection capabilities.
//!
//! # Example
//!
//! ```ignore
//! use prtip_scanner::os_fingerprinter::OsFingerprinter;
//! use prtip_core::OsFingerprintDb;
//! use std::net::Ipv4Addr;
//!
//! # async fn example() -> Result<(), prtip_core::Error> {
//! // Load OS fingerprint database from file
//! let db_content = std::fs::read_to_string("data/nmap-os-db")?;
//! let db = OsFingerprintDb::parse(&db_content)?;
//! let fingerprinter = OsFingerprinter::new(db);
//!
//! let result = fingerprinter.fingerprint_os(
//!     Ipv4Addr::new(192, 168, 1, 1),
//!     80,   // open port
//!     9999  // closed port
//! ).await?;
//!
//! println!("Detected OS: {} ({}% confidence)", result.os_name, result.accuracy);
//! # Ok(())
//! # }
//! ```

use crate::os_probe::OsProbeEngine;
use prtip_core::{Error, OsFingerprintDb};
use std::net::Ipv4Addr;
use std::sync::Arc;

/// OS fingerprinting engine
pub struct OsFingerprinter {
    /// OS fingerprint database
    db: Arc<OsFingerprintDb>,
}

/// OS detection result
#[derive(Debug, Clone)]
pub struct OsDetectionResult {
    /// Detected OS name
    pub os_name: String,
    /// OS class information
    pub os_class: String,
    /// CPE identifiers
    pub cpe: Vec<String>,
    /// Detection accuracy (0-100)
    pub accuracy: u8,
    /// Alternative matches
    pub alternatives: Vec<(String, u8)>,
}

impl OsFingerprinter {
    /// Create new OS fingerprinter with database
    pub fn new(db: OsFingerprintDb) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Fingerprint target OS
    ///
    /// Sends 16-probe sequence and matches results against database
    pub async fn fingerprint_os(
        &self,
        target: Ipv4Addr,
        open_port: u16,
        closed_port: u16,
    ) -> Result<OsDetectionResult, Error> {
        // Send probes
        let probe_engine = OsProbeEngine::new(target, open_port, closed_port);
        let results = probe_engine.send_probes().await?;

        // Match against database
        let matches = self.db.match_fingerprint(&results);

        if matches.is_empty() {
            return Err(Error::Detection("No OS match found".to_string()));
        }

        // Get best match
        let (best_match, best_score) = &matches[0];

        // Get alternatives (top 5)
        let mut alternatives = Vec::new();
        for (fp, score) in matches.iter().skip(1).take(5) {
            alternatives.push((fp.name.clone(), *score as u8));
        }

        Ok(OsDetectionResult {
            os_name: best_match.name.clone(),
            os_class: format!(
                "{} | {} | {} | {}",
                best_match.class.vendor,
                best_match.class.os_family,
                best_match.class.os_gen,
                best_match.class.device_type
            ),
            cpe: best_match.cpe.clone(),
            accuracy: *best_score as u8,
            alternatives,
        })
    }

    /// Get database size
    pub fn db_size(&self) -> usize {
        self.db.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::OsFingerprintDb;

    #[test]
    fn test_create_fingerprinter() {
        let db = OsFingerprintDb::new();
        let fingerprinter = OsFingerprinter::new(db);
        assert_eq!(fingerprinter.db_size(), 0);
    }

    #[test]
    fn test_fingerprinter_with_db() {
        let content = r#"
Fingerprint Test Linux 5.x
Class Linux | Linux | 5.x | general purpose
CPE cpe:/o:linux:linux_kernel:5
SEQ(SP=5%GCD=1%ISR=9A%TI=I)
"#;
        let db = OsFingerprintDb::parse(content).unwrap();
        let fingerprinter = OsFingerprinter::new(db);
        assert_eq!(fingerprinter.db_size(), 1);
    }
}
