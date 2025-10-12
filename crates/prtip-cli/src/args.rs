//! CLI argument parsing

use clap::{Parser, ValueEnum};
use prtip_core::{
    Config, NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, PortRange, ScanConfig,
    ScanType, ServiceDetectionConfig, TimingTemplate,
};
use std::path::PathBuf;

/// ProRT-IP WarScan - Modern Network Scanner
///
/// A high-performance network scanner written in Rust, combining the speed
/// of Masscan with the depth of Nmap.
#[derive(Parser, Debug)]
#[command(
    name = "prtip",
    version,
    about = "Protocol/Port Real-Time War Scanner",
    long_about = "ProRT-IP WarScan v0.3.5 - High-performance network scanner\n\n\
                  Combines Masscan speed (1M+ pps) with Nmap detection depth.\n\n\
                  🚀 PERFORMANCE: 3-48x faster than nmap while maintaining accuracy\n\
                  🔄 NMAP-COMPATIBLE: Supports 20+ nmap-style flags for familiar operation\n\
                  ✅ PRODUCTION-READY: 677 tests passing, cross-platform support\n\n\
                  Both nmap and ProRT-IP syntaxes are fully supported - mix and match freely!",
    after_help = "EXAMPLES:\n\
    # Quick scan (nmap syntax - top 100 ports)\n\
    prtip -F 192.168.1.1\n\n\
    # Service detection on specific ports\n\
    prtip -sV -p 22,80,443 target.com\n\n\
    # Aggressive scan with XML output\n\
    prtip -A -oX scan.xml 192.168.1.0/24\n\n\
    # Full port scan (all 65535 ports)\n\
    prtip -p- 192.168.1.1\n\n\
    # Stealth FIN scan with top 1000 ports\n\
    prtip -sF --top-ports 1000 target.com\n\n\
    # OS fingerprinting with SYN scan\n\
    prtip -sS -O -p 1-1000 192.168.1.0/24\n\n\
    # UDP scan with service detection\n\
    prtip -sU -sV -p 53,161,500 192.168.1.1\n\n\
    # Multiple output formats\n\
    prtip -sS -p 80,443 -oA scan-results 10.0.0.0/24\n\n\
    # Original ProRT-IP syntax (still supported)\n\
    prtip -s syn --ports 1-1000 --output json target.com\n\n\
    # Mix nmap and ProRT-IP syntax freely\n\
    prtip -sS --ports 1-1000 -oX scan.xml 192.168.1.1\n\n\
COMPATIBILITY:\n\
    Both nmap and ProRT-IP syntaxes are supported and can be mixed freely.\n\
    ProRT-IP accepts familiar nmap flags like -sS, -sV, -O, -oN, -oX, etc.\n\
    See docs/NMAP_COMPATIBILITY.md for comprehensive compatibility guide.\n\n\
PERFORMANCE:\n\
    ProRT-IP is 3-48x faster than nmap while maintaining 100% accuracy:\n\
    • 1K ports:    66ms (nmap: 3.2s)  → 48x faster\n\
    • Services:   2.3s (nmap: 8.1s)  → 3.5x faster\n\
    • OS detect:  1.8s (nmap: 5.4s)  → 3x faster\n\n\
DOCUMENTATION:\n\
    Repository:       https://github.com/doublegate/ProRT-IP\n\
    Nmap Compat:      docs/NMAP_COMPATIBILITY.md\n\
    Architecture:     docs/00-ARCHITECTURE.md\n\
    API Reference:    docs/05-API-REFERENCE.md\n\
    Getting Started:  README.md",
    author = "ProRT-IP Contributors"
)]
pub struct Args {
    /// Target specification (IP, CIDR, hostname)
    ///
    /// Examples: 192.168.1.1, 10.0.0.0/24, example.com
    #[arg(
        value_name = "TARGET",
        required = true,
        help_heading = "TARGET SPECIFICATION"
    )]
    pub targets: Vec<String>,

    /// Port specification
    ///
    /// Examples: 80, 1-1000, 80,443,8080, - (all ports)
    #[arg(
        short = 'p',
        long,
        value_name = "PORTS",
        default_value = "1-1000",
        help_heading = "PORT SPECIFICATION"
    )]
    pub ports: String,

    /// Scan type
    #[arg(
        short = 's',
        long,
        value_name = "TYPE",
        value_enum,
        default_value = "connect",
        help_heading = "SCAN TECHNIQUES"
    )]
    pub scan_type: ScanTypeArg,

    /// Timing template (0-5): 0=paranoid, 3=normal, 5=insane
    #[arg(short = 'T', long, value_name = "0-5", value_parser = parse_timing, default_value = "3", help_heading = "TIMING AND PERFORMANCE")]
    pub timing: u8,

    /// Connection timeout in milliseconds
    #[arg(
        long,
        value_name = "MS",
        default_value = "1000",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub timeout: u64,

    /// Max packets per second
    #[arg(long, value_name = "RATE", help_heading = "TIMING AND PERFORMANCE")]
    pub max_rate: Option<u32>,

    /// Maximum concurrent connections
    #[arg(long, value_name = "NUM", help_heading = "TIMING AND PERFORMANCE")]
    pub max_concurrent: Option<usize>,

    /// Batch size for connection pooling
    #[arg(
        short = 'b',
        long,
        value_name = "SIZE",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub batch_size: Option<usize>,

    /// Adjust file descriptor limit (Unix only)
    #[arg(long, value_name = "LIMIT", help_heading = "TIMING AND PERFORMANCE")]
    pub ulimit: Option<u64>,

    /// List available network interfaces and exit
    #[arg(long, help_heading = "NETWORK")]
    pub interface_list: bool,

    /// Source port to use for scanning (for firewall evasion)
    #[arg(short = 'g', long, value_name = "PORT", help_heading = "NETWORK")]
    pub source_port: Option<u16>,

    /// Enable OS detection (requires open and closed ports)
    #[arg(short = 'O', long, help_heading = "DETECTION")]
    pub os_detection: bool,

    /// Enable service version detection
    #[arg(long = "sV", help_heading = "DETECTION")]
    pub service_detection: bool,

    /// Service detection intensity (0-9, default: 7)
    #[arg(
        long,
        value_name = "0-9",
        default_value = "7",
        help_heading = "DETECTION"
    )]
    pub version_intensity: u8,

    /// Only OS detect hosts with at least one open port
    #[arg(long, help_heading = "DETECTION")]
    pub osscan_limit: bool,

    /// Enable banner grabbing for open ports
    #[arg(long, help_heading = "DETECTION")]
    pub banner_grab: bool,

    /// Custom service probe database file
    #[arg(
        long,
        value_name = "FILE",
        help_heading = "DETECTION",
        help = "Load service probes from custom file (default: embedded nmap-service-probes)"
    )]
    pub probe_db: Option<String>,

    /// Enable host discovery before scanning
    #[arg(short = 'P', long, help_heading = "DETECTION")]
    pub host_discovery: bool,

    /// Network interface to use
    #[arg(long, value_name = "IFACE", help_heading = "NETWORK")]
    pub interface: Option<String>,

    /// Number of scan retries
    #[arg(
        long,
        value_name = "NUM",
        default_value = "0",
        help_heading = "SCAN OPTIONS"
    )]
    pub retries: u32,

    /// Scan delay in milliseconds between probes
    #[arg(
        long,
        value_name = "MS",
        default_value = "0",
        help_heading = "SCAN OPTIONS"
    )]
    pub scan_delay: u64,

    /// Delay between hosts (milliseconds) - helps avoid network rate limiting
    #[arg(
        long,
        value_name = "MS",
        default_value = "0",
        help_heading = "SCAN OPTIONS",
        help = "Add delay after completing each host (useful for avoiding IDS/IPS detection)"
    )]
    pub host_delay: u64,

    /// Output format: text, json, xml
    #[arg(
        short = 'o',
        long,
        value_name = "FORMAT",
        value_enum,
        default_value = "text",
        help_heading = "OUTPUT"
    )]
    pub output_format: OutputFormatArg,

    /// Output file (defaults to stdout)
    #[arg(long, value_name = "FILE", help_heading = "OUTPUT")]
    pub output_file: Option<PathBuf>,

    /// Enable SQLite database storage (optional, async worker mode)
    ///
    /// By default, ProRT-IP stores results only in memory for maximum performance
    /// (~37ms for 10K ports). Use this flag to enable persistent SQLite storage
    /// with async worker (~40-50ms for 10K ports, non-blocking writes).
    ///
    /// The async worker writes results to disk in the background without blocking
    /// the scanning threads, providing near-memory performance with persistence.
    #[arg(long, help_heading = "OUTPUT")]
    pub with_db: bool,

    /// SQLite database file path (used with --with-db)
    ///
    /// Defaults to "scan_results.db" in the current directory.
    /// Only used when --with-db flag is specified.
    #[arg(
        long,
        value_name = "FILE",
        default_value = "scan_results.db",
        help_heading = "OUTPUT"
    )]
    pub database: String,

    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, help_heading = "OUTPUT")]
    pub verbose: u8,

    /// Quiet mode (suppress banner and non-essential output)
    #[arg(short = 'q', long, help_heading = "OUTPUT")]
    pub quiet: bool,

    /// Disable ASCII art banner (show compact version)
    #[arg(long, help_heading = "OUTPUT")]
    pub compact_banner: bool,

    /// Show progress bar during scan
    #[arg(long, help_heading = "OUTPUT")]
    pub progress: bool,

    /// Disable progress output
    #[arg(long, help_heading = "OUTPUT")]
    pub no_progress: bool,

    /// Statistics update interval in seconds
    #[arg(
        long,
        value_name = "SECS",
        default_value = "1",
        help_heading = "OUTPUT"
    )]
    pub stats_interval: u64,

    /// Write final statistics to JSON file
    #[arg(long, value_name = "FILE", help_heading = "OUTPUT")]
    pub stats_file: Option<PathBuf>,

    // ============================================================================
    // NMAP-COMPATIBLE FLAGS (v0.3.1+)
    // Processed via argv preprocessor in main.rs: -sS → --nmap-syn, etc.
    // Now visible in help to showcase nmap compatibility
    // ============================================================================
    /// TCP SYN scan (nmap -sS) - Half-open scan, default if privileged
    ///
    /// Sends SYN packets and analyzes responses without completing the handshake.
    /// Requires raw socket privileges. Fast and stealthy, leaves no connection logs.
    ///
    /// Example: prtip -sS -p 80,443 192.168.1.0/24
    #[arg(
        long = "nmap-syn",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_syn: bool,

    /// TCP Connect scan (nmap -sT) - Full 3-way handshake, default if unprivileged
    ///
    /// Uses OS's connect() syscall to establish full TCP connections.
    /// No raw socket privileges required. More detectable but universally compatible.
    ///
    /// Example: prtip -sT -p 1-1000 target.com
    #[arg(
        long = "nmap-connect",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_connect: bool,

    /// UDP scan (nmap -sU) - Probe UDP services
    ///
    /// Sends UDP packets to discover UDP services. Slower due to ICMP rate limiting.
    /// Best combined with version detection (-sV) for accurate service identification.
    ///
    /// Example: prtip -sU -sV -p 53,161,500 192.168.1.1
    #[arg(
        long = "nmap-udp",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_udp: bool,

    /// NULL scan (nmap -sN) - Stealth scan with no TCP flags set
    ///
    /// RFC 793 stealth technique. No flags set → closed ports respond with RST.
    /// May bypass some firewalls but fails on Windows/Cisco (they send RST always).
    ///
    /// Example: prtip -sN -p 80,443 10.0.0.0/24
    #[arg(
        long = "nmap-null",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_null: bool,

    /// FIN scan (nmap -sF) - Stealth scan with FIN flag
    ///
    /// RFC 793 stealth technique. FIN flag set → closed ports respond with RST.
    /// May bypass some firewalls but fails on Windows/Cisco.
    ///
    /// Example: prtip -sF --top-ports 1000 target.com
    #[arg(
        long = "nmap-fin",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_fin: bool,

    /// Xmas scan (nmap -sX) - Stealth scan with FIN, PSH, URG flags (lights up like a tree)
    ///
    /// RFC 793 stealth technique. FIN+PSH+URG flags → closed ports respond with RST.
    /// Named "Xmas" because flags light up like Christmas tree. Fails on Windows/Cisco.
    ///
    /// Example: prtip -sX -p 1-1000 192.168.1.0/24
    #[arg(
        long = "nmap-xmas",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_xmas: bool,

    /// ACK scan (nmap -sA) - Firewall rule mapping
    ///
    /// Sends ACK packets to map firewall rules. Used to determine if ports are filtered.
    /// Doesn't determine open/closed state, only filtered/unfiltered.
    ///
    /// Example: prtip -sA -p 80,443 target.com
    #[arg(
        long = "nmap-ack",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_ack: bool,

    /// Normal text output (nmap -oN <file>) - Human-readable text format
    ///
    /// Writes scan results in plain text format similar to terminal output.
    /// Includes all scan details, banners, and service information.
    ///
    /// Example: prtip -sS -p 80,443 -oN scan.txt 192.168.1.0/24
    #[arg(long = "output-normal", value_name = "FILE", hide = true,
          conflicts_with_all = ["output_format", "output_file"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_normal: Option<PathBuf>,

    /// XML output (nmap -oX <file>) - Machine-parseable XML format
    ///
    /// Generates nmap-compatible XML output for integration with tools like
    /// Metasploit, Nessus, or custom parsers. Preserves all scan metadata.
    ///
    /// Example: prtip -sV -O -oX scan.xml 192.168.1.0/24
    #[arg(long = "output-xml", value_name = "FILE", hide = true,
          conflicts_with_all = ["output_format", "output_file"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_xml: Option<PathBuf>,

    /// Greppable output (nmap -oG <file>) - Greppable line-based format
    ///
    /// Each host occupies one line, making it easy to grep, awk, or sed.
    /// Format: Host: <ip> (<hostname>) Ports: <port>/<state>/<protocol>/<service>
    ///
    /// Example: prtip -sS -p 1-1000 -oG scan.gnmap 10.0.0.0/24
    #[arg(long = "output-greppable", value_name = "FILE", hide = true,
          conflicts_with_all = ["output_format", "output_file"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_greppable: Option<PathBuf>,

    /// All output formats (nmap -oA <basename>) - Creates .txt, .xml, .gnmap files
    ///
    /// Generates all three output formats (normal, XML, greppable) with the given
    /// basename. Creates: <basename>.txt, <basename>.xml, <basename>.gnmap
    ///
    /// Example: prtip -sS -p 80,443 -oA scan-results 192.168.1.0/24
    #[arg(long = "output-all-formats", value_name = "BASENAME", hide = true,
          conflicts_with_all = ["output_format", "output_file",
                               "output_normal", "output_xml", "output_greppable"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_all: Option<String>,

    /// Fast scan (nmap -F) - Scan top 100 most common ports
    ///
    /// Scans only the 100 most frequently used ports based on nmap-services
    /// frequency database. Dramatically faster than default 1-1000 range.
    ///
    /// Example: prtip -F 192.168.1.1
    #[arg(
        short = 'F',
        long = "fast-scan",
        conflicts_with = "ports",
        help_heading = "NMAP-COMPATIBLE PORT SPECIFICATION"
    )]
    pub fast_scan: bool,

    /// Scan top N most common ports (nmap --top-ports <N>)
    ///
    /// Scans the N most common ports based on nmap-services frequency database.
    /// Useful for quick scans: --top-ports 10 for quickest, --top-ports 1000 for thorough.
    ///
    /// Example: prtip --top-ports 1000 target.com
    #[arg(
        long = "top-ports",
        value_name = "N",
        conflicts_with = "ports",
        help_heading = "NMAP-COMPATIBLE PORT SPECIFICATION"
    )]
    pub top_ports: Option<usize>,

    /// Aggressive scan mode (nmap -A) - Enables OS detect, service detect, progress
    ///
    /// Combines multiple detection techniques for comprehensive results. Equivalent to:
    /// -O (OS detection), -sV (service detection), --progress (real-time progress bar)
    ///
    /// Example: prtip -A -p 1-1000 192.168.1.0/24
    #[arg(
        short = 'A',
        long = "aggressive",
        help_heading = "NMAP-COMPATIBLE DETECTION"
    )]
    pub aggressive: bool,

    /// Skip host discovery (nmap -Pn) - Treat all hosts as online
    ///
    /// Bypasses ping-based host discovery and treats all targets as online.
    /// Useful when hosts don't respond to ping but have open ports.
    ///
    /// Example: prtip -Pn -sS -p 80,443 192.168.1.0/24
    #[arg(
        long = "skip-ping",
        hide = true,
        help_heading = "NMAP-COMPATIBLE DISCOVERY"
    )]
    pub skip_ping: bool,
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

        if let Some(batch) = self.batch_size {
            if batch == 0 {
                anyhow::bail!("Batch size must be greater than 0");
            }
            if batch > 100_000 {
                anyhow::bail!("Batch size cannot exceed 100,000");
            }
        }

        if self.version_intensity > 9 {
            anyhow::bail!("Version intensity must be 0-9");
        }

        if let Some(ulimit) = self.ulimit {
            if ulimit < 100 {
                anyhow::bail!("Ulimit must be at least 100");
            }
        }

        if self.progress && self.no_progress {
            anyhow::bail!("Cannot specify both --progress and --no-progress");
        }

        if self.stats_interval == 0 {
            anyhow::bail!("Stats interval must be greater than 0");
        }

        if self.stats_interval > 3600 {
            anyhow::bail!("Stats interval cannot exceed 1 hour (3600 seconds)");
        }

        Ok(())
    }

    /// Get effective port specification (handles -F and --top-ports)
    ///
    /// Returns the ports string to be parsed, considering fast scan and top ports flags.
    pub fn get_effective_ports(&self) -> String {
        use prtip_core::top_ports::{get_top_ports, ports_to_spec};

        if self.fast_scan {
            // Fast scan: top 100 ports
            ports_to_spec(&get_top_ports(100))
        } else if let Some(n) = self.top_ports {
            // Top N ports
            ports_to_spec(&get_top_ports(n))
        } else {
            // Use specified ports
            self.ports.clone()
        }
    }

    /// Convert arguments to Config structure
    ///
    /// Transforms CLI arguments into the internal configuration format
    /// used by the scanner engine. Handles both original ProRT-IP flags
    /// and nmap-compatible aliases.
    ///
    /// # Adaptive Parallelism
    ///
    /// If `--max-concurrent` is not specified, parallelism will be calculated
    /// adaptively based on port count during scan execution. This provides optimal
    /// performance without requiring manual tuning:
    /// - Small scans (≤1K ports): 20 concurrent
    /// - Medium scans (1K-10K ports): 100 concurrent
    /// - Large scans (>10K ports): 500-1000 concurrent
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

        // Determine scan type (nmap aliases take precedence for explicitness)
        let scan_type = if self.nmap_syn {
            ScanType::Syn
        } else if self.nmap_connect {
            ScanType::Connect
        } else if self.nmap_udp {
            ScanType::Udp
        } else if self.nmap_null {
            ScanType::Null
        } else if self.nmap_fin {
            ScanType::Fin
        } else if self.nmap_xmas {
            ScanType::Xmas
        } else if self.nmap_ack {
            ScanType::Ack
        } else {
            // Fall back to standard scan_type flag
            match self.scan_type {
                ScanTypeArg::Connect => ScanType::Connect,
                ScanTypeArg::Syn => ScanType::Syn,
                ScanTypeArg::Fin => ScanType::Fin,
                ScanTypeArg::Null => ScanType::Null,
                ScanTypeArg::Xmas => ScanType::Xmas,
                ScanTypeArg::Ack => ScanType::Ack,
                ScanTypeArg::Udp => ScanType::Udp,
            }
        };

        // Determine output format and file (nmap aliases take precedence)
        let (output_format, output_file) = if let Some(file) = &self.output_normal {
            (OutputFormat::Text, Some(file.clone()))
        } else if let Some(file) = &self.output_xml {
            (OutputFormat::Xml, Some(file.clone()))
        } else if let Some(file) = &self.output_greppable {
            (OutputFormat::Greppable, Some(file.clone()))
        } else if let Some(_base) = &self.output_all {
            // -oA handled separately in main.rs (generates multiple files)
            // For now, default to text (main.rs will override this)
            (OutputFormat::Text, self.output_file.clone())
        } else {
            // Fall back to standard flags
            let format = match self.output_format {
                OutputFormatArg::Text => OutputFormat::Text,
                OutputFormatArg::Json => OutputFormat::Json,
                OutputFormatArg::Xml => OutputFormat::Xml,
            };
            (format, self.output_file.clone())
        };

        // Determine if service detection should be enabled (aggressive mode enables it)
        let service_detection_enabled = self.service_detection || self.aggressive;

        // Determine if progress should be shown (aggressive mode enables it)
        let show_progress = (self.progress || self.aggressive) && !self.no_progress;

        // Determine parallelism
        // If user specified --max-concurrent, use it directly
        // Otherwise, use a placeholder (0) to signal adaptive parallelism
        // should be used during scan execution based on actual port count
        let parallelism = self.max_concurrent.unwrap_or(0);

        Config {
            scan: ScanConfig {
                scan_type,
                timing_template: timing,
                timeout_ms: self.timeout,
                retries: self.retries,
                scan_delay_ms: self.scan_delay,
                host_delay_ms: self.host_delay,
                service_detection: ServiceDetectionConfig {
                    enabled: service_detection_enabled,
                    intensity: self.version_intensity,
                    banner_grab: self.banner_grab,
                    probe_db_path: self.probe_db.clone(),
                },
                progress: show_progress,
            },
            network: NetworkConfig {
                interface: self.interface.clone(),
                source_port: self.source_port,
            },
            output: OutputConfig {
                format: output_format,
                file: output_file,
                verbose: self.verbose,
            },
            performance: PerformanceConfig {
                max_rate: self.max_rate,
                parallelism,
                batch_size: self.batch_size,
                requested_ulimit: self.ulimit,
            },
        }
    }

    /// Check if host discovery should be performed
    ///
    /// Considers both the --host-discovery flag and the nmap -Pn (skip-ping) alias.
    pub fn should_perform_host_discovery(&self) -> bool {
        self.host_discovery && !self.skip_ping
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
        assert_eq!(config.scan.timeout_ms, 1000); // Changed from 3000ms to 1000ms (new default)
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

    #[test]
    fn test_batch_size_option() {
        let args = Args::parse_from(["prtip", "-b", "2000", "192.168.1.1"]);
        assert_eq!(args.batch_size, Some(2000));

        let args = Args::parse_from(["prtip", "--batch-size", "5000", "192.168.1.1"]);
        assert_eq!(args.batch_size, Some(5000));
    }

    #[test]
    fn test_ulimit_option() {
        let args = Args::parse_from(["prtip", "--ulimit", "10000", "192.168.1.1"]);
        assert_eq!(args.ulimit, Some(10000));
    }

    #[test]
    fn test_interface_list_flag() {
        let args = Args::parse_from(["prtip", "--interface-list", "192.168.1.1"]);
        assert!(args.interface_list);

        let args = Args::parse_from(["prtip", "192.168.1.1"]);
        assert!(!args.interface_list);
    }

    #[test]
    fn test_validate_batch_size_zero() {
        let args = Args::parse_from(["prtip", "-b", "0", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_batch_size_excessive() {
        let args = Args::parse_from(["prtip", "-b", "200000", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_ulimit_too_low() {
        let args = Args::parse_from(["prtip", "--ulimit", "50", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_to_config_with_batch_and_ulimit() {
        let args = Args::parse_from(["prtip", "-b", "3000", "--ulimit", "8000", "192.168.1.1"]);
        let config = args.to_config();

        assert_eq!(config.performance.batch_size, Some(3000));
        assert_eq!(config.performance.requested_ulimit, Some(8000));
    }

    #[test]
    fn test_progress_flag() {
        let args = Args::parse_from(["prtip", "--progress", "192.168.1.1"]);
        assert!(args.progress);
        assert!(!args.no_progress);
    }

    #[test]
    fn test_no_progress_flag() {
        let args = Args::parse_from(["prtip", "--no-progress", "192.168.1.1"]);
        assert!(!args.progress);
        assert!(args.no_progress);
    }

    #[test]
    fn test_stats_interval() {
        let args = Args::parse_from(["prtip", "--stats-interval", "5", "192.168.1.1"]);
        assert_eq!(args.stats_interval, 5);
    }

    #[test]
    fn test_stats_file() {
        let args = Args::parse_from(["prtip", "--stats-file", "stats.json", "192.168.1.1"]);
        assert_eq!(args.stats_file, Some(PathBuf::from("stats.json")));
    }

    #[test]
    fn test_validate_conflicting_progress() {
        let args = Args::parse_from(["prtip", "--progress", "--no-progress", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_stats_interval_zero() {
        let args = Args::parse_from(["prtip", "--stats-interval", "0", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_stats_interval_excessive() {
        let args = Args::parse_from(["prtip", "--stats-interval", "5000", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }
}
