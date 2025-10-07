//! CLI argument parsing

use clap::{Parser, ValueEnum};
use prtip_core::{
    Config, NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, PortRange, ScanConfig,
    ScanType, TimingTemplate,
};
use std::path::PathBuf;

/// ProRT-IP WarScan - Modern Network Scanner
///
/// A high-performance network scanner written in Rust, combining the speed
/// of Masscan with the depth of Nmap.
///
/// # Examples
///
/// Basic scan:
///   prtip -p 80,443 192.168.1.1
///
/// Scan with JSON output:
///   prtip -p 1-1000 -o json --output-file=results.json 192.168.1.0/24
///
/// Aggressive timing:
///   prtip -T 4 -p 1-65535 target.com
#[derive(Parser, Debug)]
#[command(name = "prtip")]
#[command(version, about, long_about = None)]
#[command(author = "ProRT-IP Contributors")]
pub struct Args {
    /// Target specification (IP, CIDR, hostname)
    ///
    /// Examples: 192.168.1.1, 10.0.0.0/24, example.com
    #[arg(value_name = "TARGET", required = true)]
    pub targets: Vec<String>,

    /// Port specification
    ///
    /// Examples: 80, 1-1000, 80,443,8080
    #[arg(short = 'p', long, value_name = "PORTS", default_value = "1-1000")]
    pub ports: String,

    /// Scan type
    #[arg(
        short = 's',
        long,
        value_name = "TYPE",
        value_enum,
        default_value = "connect"
    )]
    pub scan_type: ScanTypeArg,

    /// Timing template (0-5)
    ///
    /// 0 = Paranoid, 1 = Sneaky, 2 = Polite, 3 = Normal, 4 = Aggressive, 5 = Insane
    #[arg(short = 'T', long, value_name = "0-5", value_parser = parse_timing, default_value = "3")]
    pub timing: u8,

    /// Connection timeout in milliseconds
    #[arg(long, value_name = "MS", default_value = "3000")]
    pub timeout: u64,

    /// Max packets per second
    #[arg(long, value_name = "RATE")]
    pub max_rate: Option<u32>,

    /// Maximum concurrent connections
    #[arg(long, value_name = "NUM")]
    pub max_concurrent: Option<usize>,

    /// Output format
    #[arg(
        short = 'o',
        long,
        value_name = "FORMAT",
        value_enum,
        default_value = "text"
    )]
    pub output_format: OutputFormatArg,

    /// Output file (defaults to stdout)
    #[arg(long, value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// SQLite database for results
    #[arg(long, value_name = "FILE", default_value = "scan_results.db")]
    pub database: String,

    /// Enable host discovery before scanning
    #[arg(short = 'P', long)]
    pub host_discovery: bool,

    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Network interface to use
    #[arg(long, value_name = "IFACE")]
    pub interface: Option<String>,

    /// Number of scan retries
    #[arg(long, value_name = "NUM", default_value = "0")]
    pub retries: u32,

    /// Scan delay in milliseconds between probes
    #[arg(long, value_name = "MS", default_value = "0")]
    pub scan_delay: u64,
}

impl Args {
    /// Validate arguments
    ///
    /// Ensures all arguments are within valid ranges and combinations.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.targets.is_empty() {
            anyhow::bail!("At least one target must be specified");
        }

        // Validate port specification
        PortRange::parse(&self.ports)
            .map_err(|e| anyhow::anyhow!("Invalid port specification '{}': {}", self.ports, e))?;

        if self.timeout == 0 {
            anyhow::bail!("Timeout must be greater than 0");
        }

        if self.timeout > 3_600_000 {
            anyhow::bail!("Timeout cannot exceed 1 hour (3600000 ms)");
        }

        if let Some(rate) = self.max_rate {
            if rate == 0 {
                anyhow::bail!("Max rate must be greater than 0");
            }
            if rate > 100_000_000 {
                anyhow::bail!("Max rate cannot exceed 100M pps");
            }
        }

        if let Some(concurrent) = self.max_concurrent {
            if concurrent == 0 {
                anyhow::bail!("Max concurrent must be greater than 0");
            }
            if concurrent > 100_000 {
                anyhow::bail!("Max concurrent cannot exceed 100,000");
            }
        }

        if self.retries > 10 {
            anyhow::bail!("Retries cannot exceed 10");
        }

        if self.timing > 5 {
            anyhow::bail!("Timing template must be 0-5");
        }

        Ok(())
    }

    /// Convert arguments to Config structure
    ///
    /// Transforms CLI arguments into the internal configuration format
    /// used by the scanner engine.
    pub fn to_config(&self) -> Config {
        let timing = match self.timing {
            0 => TimingTemplate::Paranoid,
            1 => TimingTemplate::Sneaky,
            2 => TimingTemplate::Polite,
            3 => TimingTemplate::Normal,
            4 => TimingTemplate::Aggressive,
            5 => TimingTemplate::Insane,
            _ => TimingTemplate::Normal,
        };

        let scan_type = match self.scan_type {
            ScanTypeArg::Connect => ScanType::Connect,
            ScanTypeArg::Syn => ScanType::Syn,
            ScanTypeArg::Fin => ScanType::Fin,
            ScanTypeArg::Null => ScanType::Null,
            ScanTypeArg::Xmas => ScanType::Xmas,
            ScanTypeArg::Ack => ScanType::Ack,
            ScanTypeArg::Udp => ScanType::Udp,
        };

        let output_format = match self.output_format {
            OutputFormatArg::Text => OutputFormat::Text,
            OutputFormatArg::Json => OutputFormat::Json,
            OutputFormatArg::Xml => OutputFormat::Xml,
        };

        // Determine parallelism
        let parallelism = self
            .max_concurrent
            .unwrap_or_else(|| num_cpus::get().max(1));

        Config {
            scan: ScanConfig {
                scan_type,
                timing_template: timing,
                timeout_ms: self.timeout,
                retries: self.retries,
                scan_delay_ms: self.scan_delay,
            },
            network: NetworkConfig {
                interface: self.interface.clone(),
                source_port: None, // TODO: Add CLI arg for source port in future
            },
            output: OutputConfig {
                format: output_format,
                file: self.output_file.clone(),
                verbose: self.verbose,
            },
            performance: PerformanceConfig {
                max_rate: self.max_rate,
                parallelism,
            },
        }
    }
}

/// Scan type argument
#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum ScanTypeArg {
    /// TCP connect scan (full 3-way handshake)
    Connect,
    /// TCP SYN scan (half-open, requires privileges)
    Syn,
    /// TCP FIN scan (stealth)
    Fin,
    /// TCP NULL scan (no flags set)
    Null,
    /// TCP Xmas scan (FIN, PSH, URG flags)
    Xmas,
    /// TCP ACK scan (firewall detection)
    Ack,
    /// UDP scan
    Udp,
}

/// Output format argument
#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum OutputFormatArg {
    /// Human-readable text with color
    Text,
    /// JSON format for machine parsing
    Json,
    /// XML format (Nmap-compatible)
    Xml,
}

/// Parse timing template (0-5)
fn parse_timing(s: &str) -> Result<u8, String> {
    let t: u8 = s
        .parse()
        .map_err(|_| format!("invalid timing value '{}'", s))?;
    if t <= 5 {
        Ok(t)
    } else {
        Err("timing must be 0-5".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_args() {
        let args = Args::parse_from(["prtip", "192.168.1.1"]);
        assert_eq!(args.targets, vec!["192.168.1.1"]);
        assert_eq!(args.ports, "1-1000");
        assert_eq!(args.timing, 3);
    }

    #[test]
    fn test_parse_with_ports() {
        let args = Args::parse_from(["prtip", "-p", "80,443", "example.com"]);
        assert_eq!(args.targets, vec!["example.com"]);
        assert_eq!(args.ports, "80,443");
    }

    #[test]
    fn test_parse_with_timing() {
        let args = Args::parse_from(["prtip", "-T", "4", "192.168.1.1"]);
        assert_eq!(args.timing, 4);
    }

    #[test]
    fn test_parse_with_output_format() {
        let args = Args::parse_from(["prtip", "-o", "json", "192.168.1.1"]);
        assert!(matches!(args.output_format, OutputFormatArg::Json));
    }

    #[test]
    fn test_parse_with_verbose() {
        let args = Args::parse_from(["prtip", "-vv", "192.168.1.1"]);
        assert_eq!(args.verbose, 2);
    }

    #[test]
    fn test_parse_multiple_targets() {
        let args = Args::parse_from(["prtip", "192.168.1.1", "10.0.0.1", "example.com"]);
        assert_eq!(args.targets.len(), 3);
        assert_eq!(args.targets[0], "192.168.1.1");
        assert_eq!(args.targets[1], "10.0.0.1");
        assert_eq!(args.targets[2], "example.com");
    }

    #[test]
    fn test_validate_valid() {
        let args = Args::parse_from(["prtip", "192.168.1.1"]);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_ports() {
        let args = Args::parse_from(["prtip", "-p", "invalid", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let args = Args::parse_from(["prtip", "--timeout", "0", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_excessive_timeout() {
        let args = Args::parse_from(["prtip", "--timeout", "4000000", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_zero_max_rate() {
        let args = Args::parse_from(["prtip", "--max-rate", "0", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_excessive_max_rate() {
        let args = Args::parse_from(["prtip", "--max-rate", "200000000", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_zero_max_concurrent() {
        let args = Args::parse_from(["prtip", "--max-concurrent", "0", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_excessive_retries() {
        let args = Args::parse_from(["prtip", "--retries", "20", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_to_config() {
        let args = Args::parse_from(["prtip", "-p", "80", "-T", "4", "192.168.1.1"]);
        let config = args.to_config();

        assert_eq!(config.scan.scan_type, ScanType::Connect);
        assert_eq!(config.scan.timing_template, TimingTemplate::Aggressive);
        assert_eq!(config.scan.timeout_ms, 3000);
        assert_eq!(config.output.format, OutputFormat::Text);
    }

    #[test]
    fn test_to_config_with_options() {
        let args = Args::parse_from([
            "prtip",
            "-s",
            "syn",
            "-T",
            "5",
            "-o",
            "json",
            "--timeout",
            "5000",
            "--retries",
            "3",
            "--max-rate",
            "10000",
            "--max-concurrent",
            "500",
            "192.168.1.1",
        ]);
        let config = args.to_config();

        assert_eq!(config.scan.scan_type, ScanType::Syn);
        assert_eq!(config.scan.timing_template, TimingTemplate::Insane);
        assert_eq!(config.scan.timeout_ms, 5000);
        assert_eq!(config.scan.retries, 3);
        assert_eq!(config.output.format, OutputFormat::Json);
        assert_eq!(config.performance.max_rate, Some(10000));
        assert_eq!(config.performance.parallelism, 500);
    }

    #[test]
    fn test_parse_timing_valid() {
        for i in 0..=5 {
            let result = parse_timing(&i.to_string());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), i);
        }
    }

    #[test]
    fn test_parse_timing_invalid() {
        assert!(parse_timing("6").is_err());
        assert!(parse_timing("100").is_err());
        assert!(parse_timing("abc").is_err());
        assert!(parse_timing("-1").is_err());
    }

    #[test]
    fn test_scan_type_variants() {
        let args = Args::parse_from(["prtip", "-s", "connect", "192.168.1.1"]);
        assert!(matches!(args.scan_type, ScanTypeArg::Connect));

        let args = Args::parse_from(["prtip", "-s", "syn", "192.168.1.1"]);
        assert!(matches!(args.scan_type, ScanTypeArg::Syn));

        let args = Args::parse_from(["prtip", "-s", "udp", "192.168.1.1"]);
        assert!(matches!(args.scan_type, ScanTypeArg::Udp));
    }

    #[test]
    fn test_output_format_variants() {
        let args = Args::parse_from(["prtip", "-o", "text", "192.168.1.1"]);
        assert!(matches!(args.output_format, OutputFormatArg::Text));

        let args = Args::parse_from(["prtip", "-o", "json", "192.168.1.1"]);
        assert!(matches!(args.output_format, OutputFormatArg::Json));

        let args = Args::parse_from(["prtip", "-o", "xml", "192.168.1.1"]);
        assert!(matches!(args.output_format, OutputFormatArg::Xml));
    }

    #[test]
    fn test_host_discovery_flag() {
        let args = Args::parse_from(["prtip", "-P", "192.168.1.0/24"]);
        assert!(args.host_discovery);

        let args = Args::parse_from(["prtip", "192.168.1.0/24"]);
        assert!(!args.host_discovery);
    }

    #[test]
    fn test_interface_option() {
        let args = Args::parse_from(["prtip", "--interface", "eth0", "192.168.1.1"]);
        assert_eq!(args.interface, Some("eth0".to_string()));
    }

    #[test]
    fn test_database_option() {
        let args = Args::parse_from(["prtip", "--database", "custom.db", "192.168.1.1"]);
        assert_eq!(args.database, "custom.db");

        let args = Args::parse_from(["prtip", "192.168.1.1"]);
        assert_eq!(args.database, "scan_results.db");
    }

    #[test]
    fn test_scan_delay_option() {
        let args = Args::parse_from(["prtip", "--scan-delay", "500", "192.168.1.1"]);
        assert_eq!(args.scan_delay, 500);
    }
}
