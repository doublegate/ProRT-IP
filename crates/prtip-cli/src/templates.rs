//! Scan template management for ProRT-IP
//!
//! Provides predefined and custom scan templates for common scanning scenarios.
//! Templates encapsulate common flag combinations into named presets.
//!
//! # Built-in Templates
//!
//! - `web-servers`: Scan common web server ports (80, 443, 8080, etc.)
//! - `databases`: Scan database ports (MySQL, PostgreSQL, MongoDB, Redis)
//! - `quick`: Fast scan of top 100 ports
//! - `thorough`: Comprehensive scan of all 65,535 ports
//! - `stealth`: Evasive scanning with minimal detection
//! - `discovery`: Host discovery only (no port scan)
//! - `ssl-only`: HTTPS ports with certificate analysis
//! - `admin-panels`: Remote administration ports (SSH, RDP, VNC)
//! - `mail-servers`: Email server ports (SMTP, IMAP, POP3)
//! - `file-shares`: File sharing protocols (SMB, NFS, FTP)
//!
//! # Custom Templates
//!
//! Users can define custom templates in `~/.prtip/templates.toml`:
//!
//! ```toml
//! [web-servers]
//! description = "Scan common web server ports"
//! ports = [80, 443, 8080, 8443, 3000, 5000, 8000]
//! scan_type = "SYN"
//! service_detection = true
//! timing = "T3"
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Use built-in template
//! prtip --template web-servers 192.168.1.0/24
//!
//! # Override template values
//! prtip --template stealth -T4  # Override timing
//!
//! # List available templates
//! prtip --list-templates
//!
//! # Show template details
//! prtip --show-template web-servers
//! ```

use anyhow::{Context, Result};
use prtip_core::config::Config;
use prtip_core::types::{ScanType, TimingTemplate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A scan template encapsulating common scanning configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTemplate {
    /// Template name (e.g., "web-servers")
    #[serde(skip)]
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Optional port list to scan
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<u16>>,
    /// Optional scan type (SYN, Connect, UDP, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_type: Option<String>,
    /// Enable service detection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_detection: Option<bool>,
    /// Enable OS detection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_detection: Option<bool>,
    /// Timing template (T0-T5)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing: Option<String>,
    /// Maximum packet rate (pps)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_rate: Option<u32>,
    /// Randomize target/port order
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub randomize: Option<bool>,
    /// Enable packet fragmentation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment: Option<bool>,
    /// Enable TLS certificate analysis
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_analysis: Option<bool>,
    /// Host discovery only (no port scan)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovery_only: Option<bool>,
}

impl ScanTemplate {
    /// Create a new template with name and description
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ports: None,
            scan_type: None,
            service_detection: None,
            os_detection: None,
            timing: None,
            max_rate: None,
            randomize: None,
            fragment: None,
            tls_analysis: None,
            discovery_only: None,
        }
    }

    /// Validate template configuration
    pub fn validate(&self) -> Result<()> {
        // Validate ports
        if let Some(ref ports) = self.ports {
            for &port in ports {
                if port == 0 {
                    return Err(anyhow::anyhow!(
                        "Invalid port {} in template '{}': ports must be 1-65535",
                        port,
                        self.name
                    ));
                }
            }
        }

        // Validate scan type
        if let Some(ref scan_type) = self.scan_type {
            let valid_types = [
                "SYN", "Connect", "UDP", "FIN", "NULL", "Xmas", "ACK", "Idle",
            ];
            let scan_type_upper = scan_type.to_uppercase();
            if !valid_types.contains(&scan_type_upper.as_str()) {
                return Err(anyhow::anyhow!(
                    "Invalid scan_type '{}' in template '{}': must be one of {}",
                    scan_type,
                    self.name,
                    valid_types.join(", ")
                ));
            }
        }

        // Validate timing
        if let Some(ref timing) = self.timing {
            if !timing.starts_with('T') || timing.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid timing '{}' in template '{}': must be T0-T5",
                    timing,
                    self.name
                ));
            }
            let level = timing.chars().nth(1).unwrap();
            if !('0'..='5').contains(&level) {
                return Err(anyhow::anyhow!(
                    "Invalid timing '{}' in template '{}': must be T0-T5",
                    timing,
                    self.name
                ));
            }
        }

        // Validate max_rate
        if let Some(rate) = self.max_rate {
            if rate == 0 {
                return Err(anyhow::anyhow!(
                    "Invalid max_rate in template '{}': must be greater than 0",
                    self.name
                ));
            }
            if rate > 100_000_000 {
                return Err(anyhow::anyhow!(
                    "Invalid max_rate in template '{}': cannot exceed 100M pps",
                    self.name
                ));
            }
        }

        Ok(())
    }
}

/// Manages built-in and custom scan templates
pub struct TemplateManager {
    /// Built-in templates
    builtin_templates: HashMap<String, ScanTemplate>,
    /// User-defined custom templates
    custom_templates: HashMap<String, ScanTemplate>,
}

impl TemplateManager {
    /// Create a new template manager with built-in templates
    pub fn new() -> Self {
        let builtin_templates = Self::load_builtin_templates();

        Self {
            builtin_templates,
            custom_templates: HashMap::new(),
        }
    }

    /// Create template manager and load custom templates from config directory
    pub fn with_custom_templates() -> Result<Self> {
        let mut manager = Self::new();
        manager.load_custom_templates()?;
        Ok(manager)
    }

    /// Load all built-in templates
    fn load_builtin_templates() -> HashMap<String, ScanTemplate> {
        let mut templates = HashMap::new();

        // Template: web-servers
        let mut tmpl = ScanTemplate::new(
            "web-servers",
            "Scan common web server ports with service detection",
        );
        tmpl.ports = Some(vec![80, 443, 8080, 8443, 3000, 5000, 8000, 8888]);
        tmpl.scan_type = Some("SYN".to_string());
        tmpl.service_detection = Some(true);
        tmpl.tls_analysis = Some(true);
        tmpl.timing = Some("T3".to_string());
        templates.insert("web-servers".to_string(), tmpl);

        // Template: databases
        let mut tmpl = ScanTemplate::new(
            "databases",
            "Scan common database ports (MySQL, PostgreSQL, MongoDB, Redis, MSSQL)",
        );
        tmpl.ports = Some(vec![
            3306,  // MySQL
            5432,  // PostgreSQL
            27017, // MongoDB
            6379,  // Redis
            1433,  // MSSQL
            5984,  // CouchDB
            9042,  // Cassandra
        ]);
        tmpl.scan_type = Some("Connect".to_string());
        tmpl.service_detection = Some(true);
        tmpl.timing = Some("T3".to_string());
        templates.insert("databases".to_string(), tmpl);

        // Template: quick
        let mut tmpl = ScanTemplate::new("quick", "Fast scan of top 100 most common ports");
        tmpl.scan_type = Some("SYN".to_string());
        tmpl.service_detection = Some(false);
        tmpl.timing = Some("T4".to_string());
        // Note: Top 100 ports would be set via CLI flag -F or --top-ports 100
        templates.insert("quick".to_string(), tmpl);

        // Template: thorough
        let mut tmpl = ScanTemplate::new(
            "thorough",
            "Comprehensive scan of all 65,535 ports with service and OS detection",
        );
        tmpl.scan_type = Some("SYN".to_string());
        tmpl.service_detection = Some(true);
        tmpl.os_detection = Some(true);
        tmpl.timing = Some("T3".to_string());
        // Note: All ports (-p-) would be set via CLI integration
        templates.insert("thorough".to_string(), tmpl);

        // Template: stealth
        let mut tmpl = ScanTemplate::new(
            "stealth",
            "Evasive scanning to minimize detection (FIN scan, slow timing, randomization)",
        );
        tmpl.scan_type = Some("FIN".to_string());
        tmpl.timing = Some("T1".to_string());
        tmpl.randomize = Some(true);
        tmpl.fragment = Some(true);
        tmpl.max_rate = Some(100); // Very slow
        templates.insert("stealth".to_string(), tmpl);

        // Template: discovery
        let mut tmpl =
            ScanTemplate::new("discovery", "Host discovery only (ICMP ping, no port scan)");
        tmpl.discovery_only = Some(true);
        tmpl.timing = Some("T4".to_string());
        templates.insert("discovery".to_string(), tmpl);

        // Template: ssl-only
        let mut tmpl =
            ScanTemplate::new("ssl-only", "Scan HTTPS ports with TLS certificate analysis");
        tmpl.ports = Some(vec![
            443,  // HTTPS
            8443, // Alt HTTPS
            9443, // Alt HTTPS
            636,  // LDAPS
            993,  // IMAPS
            995,  // POP3S
            465,  // SMTPS
        ]);
        tmpl.scan_type = Some("SYN".to_string());
        tmpl.service_detection = Some(true);
        tmpl.tls_analysis = Some(true);
        tmpl.timing = Some("T3".to_string());
        templates.insert("ssl-only".to_string(), tmpl);

        // Template: admin-panels
        let mut tmpl = ScanTemplate::new(
            "admin-panels",
            "Scan remote administration ports (SSH, Telnet, RDP, VNC, etc.)",
        );
        tmpl.ports = Some(vec![
            22,    // SSH
            23,    // Telnet
            3389,  // RDP
            5900,  // VNC
            5901,  // VNC
            8291,  // MikroTik
            10000, // Webmin
        ]);
        tmpl.scan_type = Some("Connect".to_string());
        tmpl.service_detection = Some(true);
        tmpl.timing = Some("T3".to_string());
        templates.insert("admin-panels".to_string(), tmpl);

        // Template: mail-servers
        let mut tmpl = ScanTemplate::new(
            "mail-servers",
            "Scan email server ports (SMTP, IMAP, POP3, submission)",
        );
        tmpl.ports = Some(vec![
            25,  // SMTP
            110, // POP3
            143, // IMAP
            465, // SMTPS
            587, // Submission
            993, // IMAPS
            995, // POP3S
        ]);
        tmpl.scan_type = Some("Connect".to_string());
        tmpl.service_detection = Some(true);
        tmpl.timing = Some("T3".to_string());
        templates.insert("mail-servers".to_string(), tmpl);

        // Template: file-shares
        let mut tmpl = ScanTemplate::new(
            "file-shares",
            "Scan file sharing protocols (FTP, SFTP, SMB, NFS, rsync)",
        );
        tmpl.ports = Some(vec![
            21,   // FTP
            22,   // SFTP (SSH)
            139,  // NetBIOS
            445,  // SMB
            2049, // NFS
            873,  // rsync
        ]);
        tmpl.scan_type = Some("Connect".to_string());
        tmpl.service_detection = Some(true);
        tmpl.timing = Some("T3".to_string());
        templates.insert("file-shares".to_string(), tmpl);

        templates
    }

    /// Load custom templates from ~/.prtip/templates.toml
    pub fn load_custom_templates(&mut self) -> Result<()> {
        let config_path = Self::get_templates_path()?;

        if !config_path.exists() {
            // No custom templates file - this is OK
            return Ok(());
        }

        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read templates file: {:?}", config_path))?;

        let templates: HashMap<String, ScanTemplate> = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse templates file: {:?}", config_path))?;

        // Validate and store custom templates
        for (name, mut template) in templates {
            template.name = name.clone();
            template.validate().with_context(|| {
                format!("Invalid custom template '{}' in {:?}", name, config_path)
            })?;
            self.custom_templates.insert(name.to_lowercase(), template);
        }

        Ok(())
    }

    /// Get the path to the templates configuration file
    fn get_templates_path() -> Result<PathBuf> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".prtip");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
        }

        Ok(config_dir.join("templates.toml"))
    }

    /// Get a template by name (case-insensitive)
    ///
    /// Custom templates override built-in templates with the same name.
    pub fn get_template(&self, name: &str) -> Option<&ScanTemplate> {
        let name_lower = name.to_lowercase();

        // Check custom templates first (they override built-ins)
        if let Some(template) = self.custom_templates.get(&name_lower) {
            return Some(template);
        }

        // Fall back to built-in templates
        self.builtin_templates.get(&name_lower)
    }

    /// List all available templates (built-in + custom)
    pub fn list_templates(&self) -> Vec<(&str, &ScanTemplate)> {
        let mut templates = Vec::new();

        // Add all built-in templates
        for (name, template) in &self.builtin_templates {
            // Skip if overridden by custom template
            if !self.custom_templates.contains_key(name) {
                templates.push((name.as_str(), template));
            }
        }

        // Add all custom templates (these may override built-ins)
        for (name, template) in &self.custom_templates {
            templates.push((name.as_str(), template));
        }

        // Sort by name for consistent output
        templates.sort_by_key(|(name, _)| *name);
        templates
    }

    /// Apply a template to a configuration
    ///
    /// Only sets fields that are specified in the template.
    /// Existing config values are preserved unless overridden.
    pub fn apply_template(&self, name: &str, config: &mut Config) -> Result<()> {
        let template = self
            .get_template(name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

        // Apply scan type
        if let Some(ref scan_type_str) = template.scan_type {
            config.scan.scan_type = Self::parse_scan_type(scan_type_str)?;
        }

        // Apply timing template
        if let Some(ref timing) = template.timing {
            config.scan.timing_template = Self::parse_timing(timing)?;
        }

        // Apply service detection
        if let Some(enabled) = template.service_detection {
            config.scan.service_detection.enabled = enabled;
        }

        // Apply OS detection
        // Note: OS detection would need to be added to Config if not present
        // For now, we'll store it as a flag or in a future OSDetectionConfig

        // Apply max rate
        if let Some(rate) = template.max_rate {
            config.performance.max_rate = Some(rate);
        }

        // Apply TLS analysis
        if let Some(enabled) = template.tls_analysis {
            config.scan.service_detection.enable_tls = enabled;
        }

        // Note: Ports, randomize, fragment, discovery_only would be handled
        // in CLI integration where we have access to the full args structure

        Ok(())
    }

    /// Parse scan type string to ScanType enum
    fn parse_scan_type(s: &str) -> Result<ScanType> {
        match s.to_uppercase().as_str() {
            "SYN" => Ok(ScanType::Syn),
            "CONNECT" => Ok(ScanType::Connect),
            "UDP" => Ok(ScanType::Udp),
            "FIN" => Ok(ScanType::Fin),
            "NULL" => Ok(ScanType::Null),
            "XMAS" => Ok(ScanType::Xmas),
            "ACK" => Ok(ScanType::Ack),
            "IDLE" => Ok(ScanType::Idle),
            _ => Err(anyhow::anyhow!("Invalid scan type: {}", s)),
        }
    }

    /// Parse timing template string to TimingTemplate enum
    fn parse_timing(s: &str) -> Result<TimingTemplate> {
        match s.to_uppercase().as_str() {
            "T0" => Ok(TimingTemplate::Paranoid),
            "T1" => Ok(TimingTemplate::Sneaky),
            "T2" => Ok(TimingTemplate::Polite),
            "T3" => Ok(TimingTemplate::Normal),
            "T4" => Ok(TimingTemplate::Aggressive),
            "T5" => Ok(TimingTemplate::Insane),
            _ => Err(anyhow::anyhow!("Invalid timing template: {}", s)),
        }
    }

    /// Get all built-in template names
    pub fn builtin_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.builtin_templates.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get all custom template names
    pub fn custom_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.custom_templates.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_validation_valid() {
        let mut tmpl = ScanTemplate::new("test", "Test template");
        tmpl.ports = Some(vec![80, 443, 8080]);
        tmpl.scan_type = Some("SYN".to_string());
        tmpl.timing = Some("T3".to_string());
        tmpl.max_rate = Some(1000);

        assert!(tmpl.validate().is_ok());
    }

    #[test]
    fn test_template_validation_invalid_port() {
        let mut tmpl = ScanTemplate::new("test", "Test template");
        tmpl.ports = Some(vec![80, 0]); // Invalid port (0 is not valid)

        let result = tmpl.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port"));
    }

    #[test]
    fn test_template_validation_invalid_scan_type() {
        let mut tmpl = ScanTemplate::new("test", "Test template");
        tmpl.scan_type = Some("INVALID".to_string());

        let result = tmpl.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("INVALID"));
    }

    #[test]
    fn test_template_validation_invalid_timing() {
        let mut tmpl = ScanTemplate::new("test", "Test template");
        tmpl.timing = Some("T6".to_string()); // Invalid

        let result = tmpl.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("T6"));
    }

    #[test]
    fn test_template_manager_builtin_templates() {
        let manager = TemplateManager::new();

        // Check all built-in templates exist
        assert!(manager.get_template("web-servers").is_some());
        assert!(manager.get_template("databases").is_some());
        assert!(manager.get_template("quick").is_some());
        assert!(manager.get_template("thorough").is_some());
        assert!(manager.get_template("stealth").is_some());
        assert!(manager.get_template("discovery").is_some());
        assert!(manager.get_template("ssl-only").is_some());
        assert!(manager.get_template("admin-panels").is_some());
        assert!(manager.get_template("mail-servers").is_some());
        assert!(manager.get_template("file-shares").is_some());
    }

    #[test]
    fn test_template_manager_case_insensitive() {
        let manager = TemplateManager::new();

        // Test case-insensitive lookup
        assert!(manager.get_template("WEB-SERVERS").is_some());
        assert!(manager.get_template("Web-Servers").is_some());
        assert!(manager.get_template("web-servers").is_some());
    }

    #[test]
    fn test_template_manager_nonexistent() {
        let manager = TemplateManager::new();
        assert!(manager.get_template("nonexistent").is_none());
    }

    #[test]
    fn test_apply_template_web_servers() {
        let manager = TemplateManager::new();
        let mut config = Config::default();

        manager.apply_template("web-servers", &mut config).unwrap();

        assert_eq!(config.scan.scan_type, ScanType::Syn);
        assert_eq!(config.scan.timing_template, TimingTemplate::Normal);
        assert!(config.scan.service_detection.enabled);
        assert!(config.scan.service_detection.enable_tls);
    }

    #[test]
    fn test_apply_template_stealth() {
        let manager = TemplateManager::new();
        let mut config = Config::default();

        manager.apply_template("stealth", &mut config).unwrap();

        assert_eq!(config.scan.scan_type, ScanType::Fin);
        assert_eq!(config.scan.timing_template, TimingTemplate::Sneaky);
        assert_eq!(config.performance.max_rate, Some(100));
    }

    #[test]
    fn test_apply_template_nonexistent() {
        let manager = TemplateManager::new();
        let mut config = Config::default();

        let result = manager.apply_template("nonexistent", &mut config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_list_templates() {
        let manager = TemplateManager::new();
        let templates = manager.list_templates();

        // Should have at least 10 built-in templates
        assert!(templates.len() >= 10);

        // Check they're sorted
        let names: Vec<_> = templates.iter().map(|(name, _)| *name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_parse_scan_type() {
        assert_eq!(
            TemplateManager::parse_scan_type("SYN").unwrap(),
            ScanType::Syn
        );
        assert_eq!(
            TemplateManager::parse_scan_type("connect").unwrap(),
            ScanType::Connect
        );
        assert_eq!(
            TemplateManager::parse_scan_type("UDP").unwrap(),
            ScanType::Udp
        );
        assert!(TemplateManager::parse_scan_type("INVALID").is_err());
    }

    #[test]
    fn test_parse_timing() {
        assert_eq!(
            TemplateManager::parse_timing("T0").unwrap(),
            TimingTemplate::Paranoid
        );
        assert_eq!(
            TemplateManager::parse_timing("T3").unwrap(),
            TimingTemplate::Normal
        );
        assert_eq!(
            TemplateManager::parse_timing("T5").unwrap(),
            TimingTemplate::Insane
        );
        assert!(TemplateManager::parse_timing("T6").is_err());
        assert!(TemplateManager::parse_timing("invalid").is_err());
    }

    #[test]
    fn test_builtin_template_descriptions() {
        let manager = TemplateManager::new();

        // Verify all templates have descriptions
        for (name, template) in manager.list_templates() {
            assert!(
                !template.description.is_empty(),
                "Template {} has no description",
                name
            );
        }
    }
}
