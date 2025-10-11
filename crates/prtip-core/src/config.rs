//! Configuration management for ProRT-IP

use crate::error::{Error, Result};
use crate::types::{ScanType, TimingTemplate};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Scan-specific configuration
    pub scan: ScanConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Output configuration
    pub output: OutputConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file {:?}: {}", path, e)))?;

        Self::load_from_str(&contents)
    }

    /// Load configuration from a TOML string
    pub fn load_from_str(contents: &str) -> Result<Self> {
        let config: Config = toml::from_str(contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)
            .map_err(|e| Error::Config(format!("Failed to write config file {:?}: {}", path, e)))?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate scan config
        if self.scan.timeout_ms == 0 {
            return Err(Error::Config(
                "timeout_ms must be greater than 0".to_string(),
            ));
        }

        if self.scan.timeout_ms > 3_600_000 {
            return Err(Error::Config("timeout_ms cannot exceed 1 hour".to_string()));
        }

        if self.scan.retries > 10 {
            return Err(Error::Config("retries cannot exceed 10".to_string()));
        }

        // Validate performance config
        // parallelism == 0 is allowed (means use adaptive parallelism)
        // Values > 0 are explicit user settings

        if self.performance.parallelism > 100_000 {
            return Err(Error::Config(
                "parallelism cannot exceed 100,000".to_string(),
            ));
        }

        if let Some(max_rate) = self.performance.max_rate {
            if max_rate == 0 {
                return Err(Error::Config("max_rate must be greater than 0".to_string()));
            }
            if max_rate > 100_000_000 {
                return Err(Error::Config("max_rate cannot exceed 100M pps".to_string()));
            }
        }

        Ok(())
    }

    /// Get reference to timing template
    pub fn timing(&self) -> &TimingTemplate {
        &self.scan.timing_template
    }
}

/// Scan-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Type of scan to perform
    pub scan_type: ScanType,
    /// Timing template
    pub timing_template: TimingTemplate,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of retries for failed probes
    pub retries: u32,
    /// Scan delay in milliseconds
    #[serde(default)]
    pub scan_delay_ms: u64,
    /// Service detection configuration
    #[serde(default)]
    pub service_detection: ServiceDetectionConfig,
    /// Enable progress bar display
    #[serde(default)]
    pub progress: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            scan_type: ScanType::Connect,
            timing_template: TimingTemplate::Normal,
            timeout_ms: 3000,
            retries: 0,
            scan_delay_ms: 0,
            service_detection: ServiceDetectionConfig::default(),
            progress: false,
        }
    }
}

/// Service detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetectionConfig {
    /// Enable service detection
    pub enabled: bool,
    /// Detection intensity (0-9, higher = more thorough)
    pub intensity: u8,
    /// Enable banner grabbing
    pub banner_grab: bool,
    /// Custom probe database file path
    #[serde(default)]
    pub probe_db_path: Option<String>,
}

impl Default for ServiceDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 7,
            banner_grab: false,
            probe_db_path: None,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network interface to use (None = auto-detect)
    pub interface: Option<String>,
    /// Source port to use (None = random)
    pub source_port: Option<u16>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format
    pub format: OutputFormat,
    /// Output file path (None = stdout)
    pub file: Option<PathBuf>,
    /// Verbosity level (0-3)
    pub verbose: u8,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            file: None,
            verbose: 0,
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Human-readable text
    Text,
    /// JSON format
    Json,
    /// XML format (Nmap-compatible)
    Xml,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Xml => write!(f, "xml"),
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum packets per second (None = unlimited)
    pub max_rate: Option<u32>,
    /// Parallelism level (concurrent connections)
    pub parallelism: usize,
    /// Batch size for connection pooling (None = auto-calculate)
    #[serde(default)]
    pub batch_size: Option<usize>,
    /// Requested ulimit value for file descriptors (None = use current)
    #[serde(default)]
    pub requested_ulimit: Option<u64>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        // Default to number of CPU cores, or 1 if detection fails
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self {
            max_rate: None,
            parallelism,
            batch_size: None,
            requested_ulimit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.scan.scan_type, ScanType::Connect);
        assert_eq!(config.scan.timing_template, TimingTemplate::Normal);
        assert_eq!(config.scan.timeout_ms, 3000);
        assert_eq!(config.scan.retries, 0);
        assert_eq!(config.output.format, OutputFormat::Text);
        assert!(config.network.interface.is_none());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let mut config = Config::default();
        config.scan.timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_excessive_timeout() {
        let mut config = Config::default();
        config.scan.timeout_ms = 4_000_000; // > 1 hour
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_excessive_retries() {
        let mut config = Config::default();
        config.scan.retries = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_parallelism() {
        let mut config = Config::default();
        config.performance.parallelism = 0;
        // 0 is now allowed (means adaptive parallelism)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_excessive_parallelism() {
        let mut config = Config::default();
        config.performance.parallelism = 200_000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_max_rate() {
        let mut config = Config::default();
        config.performance.max_rate = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_toml_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("scan_type"));
        assert!(toml_str.contains("timing_template"));
    }

    #[test]
    fn test_config_toml_deserialization() {
        let toml_str = r#"
            [scan]
            scan_type = "Connect"
            timing_template = "Aggressive"
            timeout_ms = 1000
            retries = 2
            scan_delay_ms = 0

            [network]

            [output]
            format = "Json"
            verbose = 2

            [performance]
            parallelism = 100
        "#;

        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.scan.scan_type, ScanType::Connect);
        assert_eq!(config.scan.timing_template, TimingTemplate::Aggressive);
        assert_eq!(config.scan.timeout_ms, 1000);
        assert_eq!(config.scan.retries, 2);
        assert_eq!(config.output.format, OutputFormat::Json);
        assert_eq!(config.output.verbose, 2);
        assert_eq!(config.performance.parallelism, 100);
    }

    #[test]
    fn test_config_load_from_str_invalid() {
        let toml_str = r#"
            [scan]
            timeout_ms = 0
        "#;

        let result = Config::load_from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Text.to_string(), "text");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Xml.to_string(), "xml");
    }

    #[test]
    fn test_config_with_interface() {
        let toml_str = r#"
            [scan]
            scan_type = "Syn"
            timing_template = "Normal"
            timeout_ms = 3000
            retries = 0
            scan_delay_ms = 0

            [network]
            interface = "eth0"
            source_port = 53

            [output]
            format = "Text"
            verbose = 0

            [performance]
            max_rate = 100000
            parallelism = 1000
        "#;

        let config = Config::load_from_str(toml_str).unwrap();
        assert_eq!(config.network.interface, Some("eth0".to_string()));
        assert_eq!(config.network.source_port, Some(53));
        assert_eq!(config.performance.max_rate, Some(100000));
    }
}
