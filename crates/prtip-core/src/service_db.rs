//! Service detection probe database parser
//!
//! This module implements parsing for nmap-service-probes format.
//! It handles probe definitions, match rules, and version extraction.
//!
//! # Format
//!
//! The nmap-service-probes format consists of:
//! - Probe definitions with protocol, name, and payload
//! - Match rules with regex patterns and version extraction
//! - Softmatch rules for partial matches
//!
//! # Example
//!
//! ```
//! use prtip_core::service_db::ServiceProbeDb;
//! use prtip_core::Protocol;
//!
//! let db_content = r#"
//! Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
//! ports 80,443,8080
//! rarity 1
//! match http m|^HTTP/1\.[01]| p/HTTP/
//! "#;
//! let db = ServiceProbeDb::parse(db_content)?;
//! let probes = db.probes_for_port(80, Protocol::Tcp);
//! assert!(!probes.is_empty());
//! # Ok::<(), prtip_core::Error>(())
//! ```

use crate::{Error, Protocol};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

// Embed nmap-service-probes at compile time
const EMBEDDED_SERVICE_PROBES: &str = include_str!("../data/nmap-service-probes");

/// Service probe database
#[derive(Debug, Clone)]
pub struct ServiceProbeDb {
    /// All probes indexed for quick lookup
    probes: Vec<ServiceProbe>,
    /// Port to probe mapping for optimization
    port_index: HashMap<u16, Vec<usize>>,
}

/// Individual service probe
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceProbe {
    /// Protocol (TCP/UDP)
    pub protocol: Protocol,
    /// Probe name (e.g., "GetRequest", "NULL")
    pub name: String,
    /// Probe payload to send
    pub probe_string: Vec<u8>,
    /// Ports this probe commonly applies to
    pub ports: Vec<u16>,
    /// SSL ports where this probe should use encryption
    pub ssl_ports: Vec<u16>,
    /// Rarity (1=common, 9=rare) - affects intensity levels
    pub rarity: u8,
    /// Match rules for this probe
    pub matches: Vec<ServiceMatch>,
    /// Soft match rules (less specific)
    pub soft_matches: Vec<ServiceMatch>,
}

/// Service match rule
#[derive(Debug, Clone)]
pub struct ServiceMatch {
    /// Service name (e.g., "http", "ssh", "ftp")
    pub service: String,
    /// Regex pattern to match response
    pub pattern: Regex,
    /// Product name extraction (from regex groups)
    pub product: Option<String>,
    /// Version extraction (from regex groups)
    pub version: Option<String>,
    /// Extra info extraction (from regex groups)
    pub info: Option<String>,
    /// Hostname extraction (from regex groups)
    pub hostname: Option<String>,
    /// OS type extraction (from regex groups)
    pub os_type: Option<String>,
    /// Device type extraction (from regex groups)
    pub device_type: Option<String>,
    /// CPE identifiers
    pub cpe: Vec<String>,
}

// Manual PartialEq implementation (Regex doesn't implement PartialEq)
impl PartialEq for ServiceMatch {
    fn eq(&self, other: &Self) -> bool {
        self.service == other.service
            && self.pattern.as_str() == other.pattern.as_str()
            && self.product == other.product
            && self.version == other.version
            && self.info == other.info
            && self.hostname == other.hostname
            && self.os_type == other.os_type
            && self.device_type == other.device_type
            && self.cpe == other.cpe
    }
}

impl ServiceProbeDb {
    /// Create empty database
    pub fn new() -> Self {
        Self {
            probes: Vec::new(),
            port_index: HashMap::new(),
        }
    }

    /// Parse database from string (nmap-service-probes format)
    pub fn parse(content: &str) -> Result<Self, Error> {
        let mut db = Self::new();
        let mut current_probe: Option<ServiceProbe> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse Probe line
            if line.starts_with("Probe ") {
                // Save previous probe if exists
                if let Some(probe) = current_probe.take() {
                    db.add_probe(probe);
                }

                if let Some((protocol_name, rest)) =
                    line.strip_prefix("Probe ").unwrap_or("").split_once(' ')
                {
                    let protocol = match protocol_name {
                        "TCP" => Protocol::Tcp,
                        "UDP" => Protocol::Udp,
                        _ => Protocol::Tcp,
                    };

                    if let Some((name, probe_str)) = rest.split_once(" q|") {
                        let probe_string = if let Some(data) = probe_str.strip_suffix('|') {
                            Self::parse_probe_string(data)
                        } else {
                            Vec::new()
                        };

                        current_probe = Some(ServiceProbe {
                            protocol,
                            name: name.to_string(),
                            probe_string,
                            ports: Vec::new(),
                            ssl_ports: Vec::new(),
                            rarity: 5, // Default medium rarity
                            matches: Vec::new(),
                            soft_matches: Vec::new(),
                        });
                    }
                }
                continue;
            }

            if let Some(ref mut probe) = current_probe {
                // Parse ports line
                if line.starts_with("ports ") {
                    let ports_str = line.strip_prefix("ports ").unwrap_or("");
                    probe.ports = Self::parse_port_list(ports_str);
                    continue;
                }

                // Parse sslports line
                if line.starts_with("sslports ") {
                    let ports_str = line.strip_prefix("sslports ").unwrap_or("");
                    probe.ssl_ports = Self::parse_port_list(ports_str);
                    continue;
                }

                // Parse rarity line
                if line.starts_with("rarity ") {
                    if let Ok(r) = line.strip_prefix("rarity ").unwrap_or("5").parse() {
                        probe.rarity = r;
                    }
                    continue;
                }

                // Parse match line
                if line.starts_with("match ") {
                    if let Some(service_match) = Self::parse_match_line(line, false) {
                        probe.matches.push(service_match);
                    }
                    continue;
                }

                // Parse softmatch line
                if line.starts_with("softmatch ") {
                    if let Some(service_match) = Self::parse_match_line(line, true) {
                        probe.soft_matches.push(service_match);
                    }
                    continue;
                }
            }
        }

        // Save last probe
        if let Some(probe) = current_probe {
            db.add_probe(probe);
        }

        Ok(db)
    }

    /// Add probe to database and update indexes
    fn add_probe(&mut self, probe: ServiceProbe) {
        let probe_idx = self.probes.len();

        // Index by ports
        for &port in &probe.ports {
            self.port_index.entry(port).or_default().push(probe_idx);
        }

        self.probes.push(probe);
    }

    /// Parse probe string with escape sequences
    fn parse_probe_string(s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    chars.next(); // Consume the next char
                    match next {
                        'r' => result.push(b'\r'),
                        'n' => result.push(b'\n'),
                        't' => result.push(b'\t'),
                        '0' => result.push(b'\0'),
                        'x' => {
                            // Hex escape \xHH
                            let hex: String = chars.by_ref().take(2).collect();
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                result.push(byte);
                            }
                        }
                        '\\' => result.push(b'\\'),
                        _ => result.push(next as u8),
                    }
                } else {
                    result.push(b'\\');
                }
            } else {
                result.push(c as u8);
            }
        }

        result
    }

    /// Parse comma-separated port list (supports ranges like "80-85")
    fn parse_port_list(s: &str) -> Vec<u16> {
        let mut ports = Vec::new();

        for part in s.split(',') {
            let part = part.trim();

            if part.contains('-') {
                // Handle port range (e.g., "80-85")
                if let Some((start_str, end_str)) = part.split_once('-') {
                    if let (Ok(start), Ok(end)) = (
                        start_str.trim().parse::<u16>(),
                        end_str.trim().parse::<u16>(),
                    ) {
                        for port in start..=end {
                            ports.push(port);
                        }
                    }
                }
            } else {
                // Handle single port
                if let Ok(port) = part.parse() {
                    ports.push(port);
                }
            }
        }

        ports
    }

    /// Parse match or softmatch line
    fn parse_match_line(line: &str, is_soft: bool) -> Option<ServiceMatch> {
        let prefix = if is_soft { "softmatch " } else { "match " };
        let rest = line.strip_prefix(prefix)?;

        // Parse: service m|pattern| [p/product/] [v/version/] [i/info/] ...
        let mut parts = rest.splitn(2, ' ');
        let service = parts.next()?.to_string();
        let remaining = parts.next()?;

        // Extract pattern (between m| and |)
        if let Some(pattern_start) = remaining.find("m|") {
            let pattern_content = &remaining[pattern_start + 2..];
            if let Some(pattern_end) = pattern_content.find('|') {
                let pattern_str = &pattern_content[..pattern_end];

                // Compile regex (handle errors gracefully)
                let pattern = match Regex::new(pattern_str) {
                    Ok(p) => p,
                    Err(_) => return None, // Skip invalid patterns
                };

                let rest = &pattern_content[pattern_end + 1..];

                // Extract version info fields
                let mut product = None;
                let mut version = None;
                let mut info = None;
                let mut hostname = None;
                let mut os_type = None;
                let mut device_type = None;
                let mut cpe = Vec::new();

                // Simple extraction of p/.../ v/.../ etc.
                for field in rest.split_whitespace() {
                    if field.starts_with("p/") {
                        product = field
                            .strip_prefix("p/")
                            .and_then(|s| s.strip_suffix('/'))
                            .map(String::from);
                    } else if field.starts_with("v/") {
                        version = field
                            .strip_prefix("v/")
                            .and_then(|s| s.strip_suffix('/'))
                            .map(String::from);
                    } else if field.starts_with("i/") {
                        info = field
                            .strip_prefix("i/")
                            .and_then(|s| s.strip_suffix('/'))
                            .map(String::from);
                    } else if field.starts_with("h/") {
                        hostname = field
                            .strip_prefix("h/")
                            .and_then(|s| s.strip_suffix('/'))
                            .map(String::from);
                    } else if field.starts_with("o/") {
                        os_type = field
                            .strip_prefix("o/")
                            .and_then(|s| s.strip_suffix('/'))
                            .map(String::from);
                    } else if field.starts_with("d/") {
                        device_type = field
                            .strip_prefix("d/")
                            .and_then(|s| s.strip_suffix('/'))
                            .map(String::from);
                    } else if field.starts_with("cpe:") {
                        cpe.push(field.to_string());
                    }
                }

                return Some(ServiceMatch {
                    service,
                    pattern,
                    product,
                    version,
                    info,
                    hostname,
                    os_type,
                    device_type,
                    cpe,
                });
            }
        }

        None
    }

    /// Get probes suitable for a specific port
    ///
    /// Returns probes ordered by rarity (common first)
    pub fn probes_for_port(&self, port: u16, protocol: Protocol) -> Vec<&ServiceProbe> {
        let mut probes = Vec::new();

        // Get indexed probes for this port
        if let Some(indices) = self.port_index.get(&port) {
            for &idx in indices {
                if self.probes[idx].protocol == protocol {
                    probes.push(&self.probes[idx]);
                }
            }
        }

        // Add NULL probe and common probes
        for probe in &self.probes {
            if probe.protocol == protocol && probe.ports.is_empty() {
                // Generic probes (no specific ports)
                probes.push(probe);
            }
        }

        // If no port-specific probes found, add common probes as fallback
        // This handles non-standard ports (e.g., 2021 instead of 21)
        if probes.len() <= 1 {
            // Only NULL probe or empty
            for probe in &self.probes {
                if probe.protocol == protocol && probe.rarity <= 3 && !probe.ports.is_empty() {
                    // Add common probes (rarity 1-3) regardless of port match
                    probes.push(probe);
                }
            }
        }

        // Sort by rarity (common first)
        probes.sort_by_key(|p| p.rarity);
        probes.dedup();

        probes
    }

    /// Get all probes for a protocol up to a certain intensity level
    pub fn probes_for_intensity(&self, protocol: Protocol, intensity: u8) -> Vec<&ServiceProbe> {
        self.probes
            .iter()
            .filter(|p| p.protocol == protocol && p.rarity <= intensity)
            .collect()
    }

    /// Get number of probes in database
    pub fn len(&self) -> usize {
        self.probes.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.probes.is_empty()
    }

    /// Create database with embedded probes
    pub fn with_embedded_probes() -> Result<Self, Error> {
        Self::parse(EMBEDDED_SERVICE_PROBES)
    }

    /// Load from standard nmap locations
    pub fn load_from_system() -> Result<Self, Error> {
        let paths = [
            "/usr/share/nmap/nmap-service-probes",                // Linux
            "/usr/local/share/nmap/nmap-service-probes",          // BSD/macOS Homebrew
            "/opt/nmap/share/nmap-service-probes",                // Alternative
            "C:\\Program Files\\Nmap\\nmap-service-probes",       // Windows
            "C:\\Program Files (x86)\\Nmap\\nmap-service-probes", // Windows 32-bit
        ];

        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                return Self::parse(&content);
            }
        }

        Err(Error::Config(
            "nmap-service-probes not found in standard locations".to_string(),
        ))
    }

    /// Load from custom file path
    pub fn load_from_file(path: &str) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read {}: {}", path, e)))?;
        Self::parse(&content)
    }

    /// Create database with best available source (hybrid approach)
    pub fn load_default() -> Result<Self, Error> {
        // 1. Try embedded probes (always available)
        if let Ok(db) = Self::with_embedded_probes() {
            eprintln!("Service detection: Using embedded nmap-service-probes");
            return Ok(db);
        }

        // 2. Try system installation
        if let Ok(db) = Self::load_from_system() {
            eprintln!("Service detection: Using system nmap-service-probes");
            return Ok(db);
        }

        // 3. Return empty with warning
        eprintln!("Warning: No service probes available");
        eprintln!("Service detection disabled. Install nmap or use --probe-db <file>");
        Ok(Self::new())
    }
}

impl Default for ServiceProbeDb {
    fn default() -> Self {
        Self::load_default().unwrap_or_else(|_| {
            eprintln!("Error: Failed to load service probes");
            Self::new()
        })
    }
}

impl FromStr for ServiceProbeDb {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_db() {
        let db = ServiceProbeDb::new();
        assert_eq!(db.len(), 0);
        assert!(db.is_empty());
    }

    #[test]
    fn test_parse_simple_probe() {
        let content = r#"
# Test service probes
Probe TCP NULL q||
ports 21,22,23
rarity 1

match ftp m|^220.*FTP| p/FTP/ v/1.0/
"#;

        let db = ServiceProbeDb::parse(content).unwrap();
        assert_eq!(db.len(), 1);

        let probe = &db.probes[0];
        assert_eq!(probe.protocol, Protocol::Tcp);
        assert_eq!(probe.name, "NULL");
        assert_eq!(probe.ports, vec![21, 22, 23]);
        assert_eq!(probe.rarity, 1);
        assert_eq!(probe.matches.len(), 1);
    }

    #[test]
    fn test_parse_probe_string() {
        let result = ServiceProbeDb::parse_probe_string("GET / HTTP/1.0\\r\\n\\r\\n");
        assert_eq!(result, b"GET / HTTP/1.0\r\n\r\n");

        let result = ServiceProbeDb::parse_probe_string("\\x00\\x01\\x02");
        assert_eq!(result, vec![0x00, 0x01, 0x02]);
    }

    #[test]
    fn test_parse_port_list() {
        let ports = ServiceProbeDb::parse_port_list("80,443,8080");
        assert_eq!(ports, vec![80, 443, 8080]);

        // Test port ranges
        let ports = ServiceProbeDb::parse_port_list("80-85");
        assert_eq!(ports, vec![80, 81, 82, 83, 84, 85]);

        // Test mixed
        let ports = ServiceProbeDb::parse_port_list("22,80-82,443");
        assert_eq!(ports, vec![22, 80, 81, 82, 443]);
    }

    #[test]
    fn test_parse_match_line() {
        let line = "match http m|^HTTP/1\\.[01]| p/HTTP/ v/$1/";
        let service_match = ServiceProbeDb::parse_match_line(line, false).unwrap();

        assert_eq!(service_match.service, "http");
        assert_eq!(service_match.product, Some("HTTP".to_string()));
        assert_eq!(service_match.version, Some("$1".to_string()));
    }

    #[test]
    fn test_probes_for_port() {
        let content = r#"
Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
ports 80,8080
rarity 1
match http m|^HTTP| p/HTTP/

Probe TCP NULL q||
rarity 1
match ftp m|^220| p/FTP/
"#;

        let db = ServiceProbeDb::parse(content).unwrap();
        let probes = db.probes_for_port(80, Protocol::Tcp);

        assert!(!probes.is_empty());
        assert!(probes.iter().any(|p| p.name == "GetRequest"));
    }

    #[test]
    fn test_probes_for_intensity() {
        let content = r#"
Probe TCP Common q|test|
rarity 1

Probe TCP Rare q|test|
rarity 9
"#;

        let db = ServiceProbeDb::parse(content).unwrap();

        let probes = db.probes_for_intensity(Protocol::Tcp, 5);
        assert_eq!(probes.len(), 1); // Only rarity 1

        let probes = db.probes_for_intensity(Protocol::Tcp, 9);
        assert_eq!(probes.len(), 2); // Both rarity 1 and 9
    }

    #[test]
    fn test_softmatch_parsing() {
        let content = r#"
Probe TCP Test q|test|
softmatch http m|^HTTP|
"#;

        let db = ServiceProbeDb::parse(content).unwrap();
        assert_eq!(db.probes[0].soft_matches.len(), 1);
    }

    #[test]
    fn test_embedded_probes_exist() {
        let db = ServiceProbeDb::default();
        assert!(!db.is_empty(), "Probe database should not be empty");
        assert!(
            db.probes.len() > 100,
            "Should have >100 probes, got {}",
            db.probes.len()
        );
        eprintln!("Loaded {} service probes", db.probes.len());
    }

    #[test]
    fn test_http_probe_exists() {
        let db = ServiceProbeDb::default();
        let http_probes = db
            .probes
            .iter()
            .filter(|p| p.protocol == Protocol::Tcp && p.name.contains("GetRequest"))
            .count();
        assert!(http_probes > 0, "Should have HTTP probes");
    }

    #[test]
    fn test_load_from_file() {
        // Test that load_from_file works with a valid path
        let content = r#"
Probe TCP Test q|test|
match http m|^HTTP|
"#;
        // Use cross-platform temp directory (Windows: %TEMP%, Unix: /tmp)
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("prtip-test-probes.txt");
        std::fs::write(&temp_path, content).unwrap();

        let result = ServiceProbeDb::load_from_file(temp_path.to_str().unwrap());
        assert!(result.is_ok());
        let db = result.unwrap();
        assert_eq!(db.len(), 1);

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_load_from_file_invalid_path() {
        let result = ServiceProbeDb::load_from_file("/nonexistent/path/to/probes.txt");
        assert!(result.is_err());
    }
}
