//! CLI argument parsing

use clap::{Parser, ValueEnum};
use prtip_core::{
    Config, DecoyConfig, EvasionConfig, NetworkConfig, OutputConfig, OutputFormat,
    PerformanceConfig, PortRange, ScanConfig, ScanType, ServiceDetectionConfig, TimingTemplate,
};
use std::net::Ipv4Addr;
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
    long_about = "ProRT-IP WarScan - High-performance network scanner\n\n\
                  Combines Masscan speed (1M+ pps) with Nmap detection depth.\n\n\
                  🚀 PERFORMANCE: 3-48x faster than nmap while maintaining accuracy\n\
                  🔄 NMAP-COMPATIBLE: Supports 50+ nmap-style flags for familiar operation\n\
                  ✅ PRODUCTION-READY: 868 tests passing, cross-platform support\n\n\
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
    # IPv6-only scan (force IPv6)\n\
    prtip -6 -p 80,443 2001:db8::1\n\n\
    # IPv4-only scan (explicit)\n\
    prtip -4 -p 80,443 192.168.1.1\n\n\
    # Dual-stack scan (both IPv4 and IPv6)\n\
    prtip --dual-stack -p 80 192.168.1.1 2001:db8::1\n\n\
    # IPv6 link-local scan\n\
    prtip -6 -p 22,80,443 fe80::1%eth0\n\n\
    # Idle scan (zombie scan) - ultimate stealth\n\
    prtip -sI 192.168.1.100 -p 80,443 target.com\n\n\
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
        required_unless_present_any = ["list_templates", "show_template"],
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

    /// Maximum packet rate (packets per second)
    ///
    /// Enforces rate limiting to prevent network flooding and respect target capacity.
    /// ProRT-IP uses an optimized adaptive rate limiter with -1.8% average overhead
    /// (faster than no rate limiting due to system-wide optimizations).
    ///
    /// Recommended rates:
    ///   - Local networks: 50,000-100,000 pps
    ///   - Internet scans: 10,000-50,000 pps
    ///   - Quiet scanning: 1,000-10,000 pps
    ///
    /// Examples:
    ///   prtip --max-rate 100000 -sS 192.168.1.0/24
    ///   prtip -T4 192.168.1.1  # T4 timing includes rate limiting
    ///
    /// Nmap compatible: -T templates automatically set appropriate rates.
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

    /// Packet batch size for sendmmsg/recvmmsg (Linux only, 16-1024)
    ///
    /// Controls the number of packets sent/received in a single syscall on Linux.
    /// Higher values reduce syscall overhead at high packet rates (>100K pps).
    /// Typical values: 16 (low rate), 64 (balanced, default), 128 (high rate).
    /// Auto-detected based on --max-rate if not specified.
    ///
    /// Example: prtip --mmsg-batch-size 128 --max-rate 1000000 \<target\>
    #[arg(long, value_name = "SIZE", value_parser = clap::value_parser!(u16).range(16..=1024), help_heading = "TIMING AND PERFORMANCE")]
    pub mmsg_batch_size: Option<u16>,

    /// Adjust file descriptor limit (Unix only)
    #[arg(long, value_name = "LIMIT", help_heading = "TIMING AND PERFORMANCE")]
    pub ulimit: Option<u64>,

    /// Enable NUMA optimization for multi-socket systems (Linux only)
    ///
    /// Pins threads to CPU cores based on NUMA topology to reduce cross-socket
    /// memory access latency. Provides 20-30% throughput improvement on dual-socket
    /// systems. Requires CAP_SYS_NICE capability.
    ///
    /// Example: sudo setcap cap_sys_nice+ep /usr/bin/prtip
    #[arg(long, help_heading = "TIMING AND PERFORMANCE")]
    pub numa: bool,

    /// Explicitly disable NUMA optimization (even if available)
    #[arg(long, help_heading = "TIMING AND PERFORMANCE")]
    pub no_numa: bool,

    // ============================================================================
    // ADVANCED RATE LIMITING (Sprint 5.4)
    // ============================================================================
    /// Enable adaptive rate limiting with ICMP error monitoring
    ///
    /// Dynamically adjusts scan rate based on ICMP Type 3 Code 13 (admin prohibited)
    /// error messages from target hosts. Implements per-target exponential backoff
    /// (1s → 2s → 4s → 8s → 16s max) when errors are detected.
    ///
    /// This prevents triggering rate-limiting on target networks and reduces
    /// detection risk by adapting to network conditions in real-time.
    ///
    /// Example: prtip --adaptive-rate -sS -p 1-1000 192.168.1.0/24
    #[arg(long = "adaptive-rate", help_heading = "TIMING AND PERFORMANCE")]
    pub adaptive_rate: bool,

    /// Maximum number of concurrent target hosts (Nmap --max-hostgroup)
    ///
    /// Limits how many hosts are scanned simultaneously. Smaller values reduce
    /// network load and memory usage, larger values increase scan speed.
    ///
    /// Recommended values:
    /// - 16: Conservative (low bandwidth, high latency networks)
    /// - 64: Balanced (default, works well for most networks)
    /// - 256: Aggressive (fast networks, high bandwidth)
    ///
    /// Example: prtip --max-hostgroup 32 -sS -p 80,443 192.168.1.0/24
    #[arg(
        long = "max-hostgroup",
        value_name = "N",
        default_value = "64",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub max_hostgroup: usize,

    /// Minimum number of concurrent target hosts (Nmap --min-hostgroup)
    ///
    /// Ensures at least this many hosts are scanned concurrently, even if
    /// adaptive algorithms suggest fewer. Prevents excessive slowdown.
    ///
    /// Example: prtip --min-hostgroup 8 --max-hostgroup 64 -sS 192.168.1.0/24
    #[arg(
        long = "min-hostgroup",
        value_name = "N",
        default_value = "1",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub min_hostgroup: usize,

    /// Alias for --max-hostgroup (Nmap --max-parallelism)
    ///
    /// Nmap compatibility alias. Sets the same limit as --max-hostgroup.
    /// If both are specified, --max-parallelism takes precedence.
    ///
    /// Example: prtip --max-parallelism 128 -sS 10.0.0.0/24
    #[arg(
        long = "max-parallelism",
        value_name = "N",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub max_parallelism: Option<usize>,

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

    /// Disable TLS/SSL service detection (faster, but misses HTTPS/SMTPS/IMAPS/etc.)
    #[arg(long, help_heading = "DETECTION")]
    pub no_tls: bool,

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

    // ============================================================================
    // HOST DISCOVERY FLAGS (nmap-compatible)
    // ============================================================================
    /// Only do host discovery, don't port scan
    #[arg(long, help_heading = "HOST DISCOVERY")]
    pub ping_only: bool,

    /// ARP ping discovery (local network only)
    #[arg(short = 'R', long = "arp-ping", help_heading = "HOST DISCOVERY")]
    pub arp_ping: bool,

    /// TCP SYN ping on specified ports (default: 80)
    #[arg(long = "ps", value_name = "portlist", help_heading = "HOST DISCOVERY")]
    pub tcp_syn_ping: Option<String>,

    /// TCP ACK ping on specified ports (default: 80)
    #[arg(long = "pa", value_name = "portlist", help_heading = "HOST DISCOVERY")]
    pub tcp_ack_ping: Option<String>,

    /// UDP ping on specified ports (default: 40125)
    #[arg(long = "pu", value_name = "portlist", help_heading = "HOST DISCOVERY")]
    pub udp_ping: Option<String>,

    /// ICMP echo request ping
    #[arg(long = "pe", help_heading = "HOST DISCOVERY")]
    pub icmp_echo_ping: bool,

    /// ICMP timestamp ping
    #[arg(long = "pp", help_heading = "HOST DISCOVERY")]
    pub icmp_timestamp_ping: bool,

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

    // ============================================================================
    // TIMING FLAGS (nmap-compatible)
    // ============================================================================
    /// Maximum probe retransmissions (nmap --max-retries \<N\>)
    ///
    /// Cap on number of retransmissions for unresponsive probes.
    /// Lower values speed up scans but may miss hosts on lossy networks.
    ///
    /// Example: prtip --max-retries 5 \<target\>
    #[arg(
        long = "max-retries",
        value_name = "N",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub max_retries: Option<u32>,

    /// Give up on host after this time (nmap --host-timeout \<time\>)
    ///
    /// Timeout for individual hosts. Prevents wasting time on unresponsive targets.
    /// Accepts time units: 100ms, 5s, 10m, 1h
    ///
    /// Example: prtip --host-timeout 30m \<target\>
    #[arg(
        long = "host-timeout",
        value_name = "time",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub host_timeout: Option<String>,

    /// Maximum delay between probes (nmap --max-scan-delay \<time\>)
    ///
    /// Cap on probe delay to prevent excessive slowdown.
    /// Accepts time units: 100ms, 1s, etc.
    ///
    /// Example: prtip --max-scan-delay 500ms \<target\>
    #[arg(
        long = "max-scan-delay",
        value_name = "time",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub max_scan_delay: Option<String>,

    /// Minimum packets per second (nmap --min-rate \<N\>)
    ///
    /// Ensure minimum scan rate regardless of network conditions.
    /// Useful for maintaining scan speed on slow networks.
    ///
    /// Example: prtip --min-rate 100 \<target\>
    #[arg(
        long = "min-rate",
        value_name = "N",
        help_heading = "TIMING AND PERFORMANCE"
    )]
    pub min_rate: Option<u32>,

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

    /// Enable packet capture to PCAPNG file (Wireshark-compatible)
    ///
    /// Captures all sent and received packets for forensic analysis.
    /// Creates indexed files (e.g., scan-001.pcapng, scan-002.pcapng) with
    /// automatic rotation at 1GB to prevent single large files.
    ///
    /// Files can be analyzed with: tshark -r scan-001.pcapng
    ///
    /// Performance impact: <5% overhead with buffered async writes.
    ///
    /// Example: prtip --packet-capture scan.pcapng -sS -p 80,443 192.168.1.0/24
    #[arg(long, value_name = "FILE", help_heading = "OUTPUT")]
    pub packet_capture: Option<PathBuf>,

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

    /// Skip all confirmations (assume 'yes' to all prompts)
    ///
    /// Automatically confirms all dangerous operation prompts without asking.
    /// Useful for automation and CI/CD environments.
    ///
    /// Example: prtip --yes -sS -p- 8.8.8.8
    #[arg(short = 'y', long, help_heading = "OUTPUT")]
    pub yes: bool,

    /// Disable ASCII art banner (show compact version)
    #[arg(long, help_heading = "OUTPUT")]
    pub compact_banner: bool,

    /// Show progress bar during scan
    #[arg(long, help_heading = "OUTPUT")]
    pub progress: bool,

    /// Disable progress output
    #[arg(long, help_heading = "OUTPUT")]
    pub no_progress: bool,

    /// Progress display style (compact/detailed/bars)
    #[arg(
        long,
        value_name = "STYLE",
        default_value = "compact",
        help_heading = "OUTPUT"
    )]
    pub progress_style: String,

    /// Progress update interval in seconds
    #[arg(
        long,
        value_name = "SECS",
        default_value = "1",
        help_heading = "OUTPUT"
    )]
    pub progress_interval: u64,

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

    /// Capture raw service responses for debugging (increases memory usage)
    ///
    /// Stores the raw bytes returned by services when probed. Useful for diagnosing
    /// service detection issues and understanding why patterns match or fail.
    /// Increases memory usage proportional to number of detected services.
    ///
    /// Example: prtip --capture-raw-responses -sV -p 80,443 192.168.1.1
    #[arg(long, help_heading = "OUTPUT")]
    pub capture_raw_responses: bool,

    // ============================================================================
    // OUTPUT FILTERING AND DISPLAY FLAGS (nmap-compatible)
    // ============================================================================
    /// Show only open (or possibly open) ports (nmap --open)
    ///
    /// Filter output to display only interesting results. Dramatically reduces
    /// output size for large scans by hiding closed and filtered ports.
    ///
    /// Example: prtip --open -p- \<target\>
    #[arg(long, help_heading = "OUTPUT")]
    pub open: bool,

    /// Show all packets sent and received (nmap --packet-trace)
    ///
    /// Very verbose packet-level tracing. Useful for debugging and understanding
    /// scan behavior. Shows every packet transmitted and received.
    ///
    /// Example: prtip --packet-trace \<target\>
    #[arg(long, help_heading = "OUTPUT")]
    pub packet_trace: bool,

    /// Display reason for port state (nmap --reason)
    ///
    /// Show why each port is in its current state (syn-ack, rst, timeout, etc.).
    /// Useful for understanding firewall behavior and troubleshooting.
    ///
    /// Example: prtip --reason \<target\>
    #[arg(long, help_heading = "OUTPUT")]
    pub reason: bool,

    /// Print scan statistics every N seconds (nmap --stats-every \<time\>)
    ///
    /// Display periodic statistics during long-running scans.
    /// Accepts time units: 1s, 30s, 5m, etc.
    ///
    /// Example: prtip --stats-every 5s \<target\>
    #[arg(long, value_name = "time", help_heading = "OUTPUT")]
    pub stats_every: Option<String>,

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

    /// Idle scan using zombie host (nmap -sI \<zombie\>) - Ultimate stealth
    ///
    /// Perform completely anonymous port scanning via third-party zombie host.
    /// Preprocessed from -sI flag to --nmap-idle for internal use.
    ///
    /// Example: prtip -sI 192.168.1.100 -p 80,443 target.com
    #[arg(
        long = "nmap-idle",
        value_name = "ZOMBIE_HOST",
        hide = true,
        conflicts_with = "scan_type",
        help_heading = "NMAP-COMPATIBLE SCAN TYPES"
    )]
    pub nmap_idle: Option<String>,

    /// Normal text output (nmap -oN \<file\>) - Human-readable text format
    ///
    /// Writes scan results in plain text format similar to terminal output.
    /// Includes all scan details, banners, and service information.
    ///
    /// Example: prtip -sS -p 80,443 -oN scan.txt 192.168.1.0/24
    #[arg(long = "output-normal", value_name = "FILE", hide = true,
          conflicts_with_all = ["output_format", "output_file"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_normal: Option<PathBuf>,

    /// XML output (nmap -oX \<file\>) - Machine-parseable XML format
    ///
    /// Generates nmap-compatible XML output for integration with tools like
    /// Metasploit, Nessus, or custom parsers. Preserves all scan metadata.
    ///
    /// Example: prtip -sV -O -oX scan.xml 192.168.1.0/24
    #[arg(long = "output-xml", value_name = "FILE", hide = true,
          conflicts_with_all = ["output_format", "output_file"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_xml: Option<PathBuf>,

    /// Greppable output (nmap -oG \<file\>) - Greppable line-based format
    ///
    /// Each host occupies one line, making it easy to grep, awk, or sed.
    /// Format: Host: \<ip\> (\<hostname\>) Ports: \<port\>/\<state\>/\<protocol\>/\<service\>
    ///
    /// Example: prtip -sS -p 1-1000 -oG scan.gnmap 10.0.0.0/24
    #[arg(long = "output-greppable", value_name = "FILE", hide = true,
          conflicts_with_all = ["output_format", "output_file"],
          help_heading = "NMAP-COMPATIBLE OUTPUT")]
    pub output_greppable: Option<PathBuf>,

    /// All output formats (nmap -oA \<basename\>) - Creates .txt, .xml, .gnmap files
    ///
    /// Generates all three output formats (normal, XML, greppable) with the given
    /// basename. Creates: \<basename\>.txt, \<basename\>.xml, \<basename\>.gnmap
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

    /// Scan top N most common ports (nmap --top-ports \<N\>)
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

    /// Don't randomize port scan order (nmap -r)
    ///
    /// Scan ports in sequential order (1, 2, 3...) instead of random order.
    /// Slightly faster but more detectable by IDS/IPS systems.
    ///
    /// Example: prtip -r \<target\>
    #[arg(
        short = 'r',
        long = "no-randomize",
        help_heading = "NMAP-COMPATIBLE PORT SPECIFICATION"
    )]
    pub no_randomize: bool,

    /// Scan ports more common than specified ratio (nmap --port-ratio \<ratio\>)
    ///
    /// Scan ports more common than the given ratio (0.0-1.0).
    /// Advanced option for fine-grained port selection based on frequency.
    ///
    /// Example: prtip --port-ratio 0.5 \<target\>
    #[arg(
        long = "port-ratio",
        value_name = "ratio",
        conflicts_with = "ports",
        help_heading = "NMAP-COMPATIBLE PORT SPECIFICATION"
    )]
    pub port_ratio: Option<f32>,

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

    // ============================================================================
    // FIREWALL/IDS EVASION AND SPOOFING (nmap-compatible)
    // ============================================================================
    /// Fragment IP packets (nmap -f) - Send 8-byte fragments after IP header
    ///
    /// Split packets into small fragments to evade firewalls and IDS that don't
    /// properly reassemble fragments. Each fragment is 8 bytes of data after the
    /// IP header (minimum fragmentation, maximum stealth).
    ///
    /// Note: Some firewalls and IDS systems may drop fragmented packets or trigger alerts.
    /// Use --mtu for less aggressive fragmentation.
    ///
    /// Example: prtip -sS -f -p 80,443 target.com
    #[arg(
        short = 'f',
        long = "fragment",
        help_heading = "FIREWALL/IDS EVASION AND SPOOFING"
    )]
    pub fragment: bool,

    /// Custom MTU for packet fragmentation (nmap --mtu) - Must be multiple of 8
    ///
    /// Set a custom Maximum Transmission Unit for packet fragmentation. The MTU
    /// must be a multiple of 8 and at least 68 bytes (20 IP header + 8 fragment min).
    /// Larger MTU values create larger fragments (less fragmentation, faster scans).
    ///
    /// Common values:
    /// - 28: Extremely aggressive (20 IP + 8 data, similar to -f)
    /// - 200: Moderate fragmentation
    /// - 1500: Standard Ethernet MTU (no fragmentation)
    ///
    /// Example: prtip -sS --mtu 200 -p 80,443 target.com
    #[arg(
        long,
        value_name = "SIZE",
        help_heading = "FIREWALL/IDS EVASION AND SPOOFING"
    )]
    pub mtu: Option<usize>,

    /// Set IP Time-To-Live field (nmap --ttl) - Range: 1-255
    ///
    /// Set a custom TTL value for outgoing packets. Lower values can be used to
    /// discover firewalls in the network path (packets expire before reaching target).
    /// Higher values maximize packet lifetime.
    ///
    /// Default TTL varies by OS:
    /// - Linux: 64
    /// - Windows: 128
    /// - Cisco: 255
    ///
    /// Example: prtip -sS --ttl 32 -p 80,443 target.com
    #[arg(
        long,
        value_name = "VALUE",
        help_heading = "FIREWALL/IDS EVASION AND SPOOFING"
    )]
    pub ttl: Option<u8>,

    /// Cloak scan with decoys (nmap -D) - Format: RND:N or IP,IP,ME,IP
    ///
    /// Hide the real source IP among decoy IPs to make it harder to trace the scan origin.
    /// The scanner mixes real probes with spoofed-source decoy probes sent in random order.
    ///
    /// Formats:
    /// - RND:10          - Use 10 random decoy IPs
    /// - 1.2.3.4,ME,5.6.7.8 - Use specific decoys (ME = your real IP)
    /// - RND:5,10.0.0.1  - Mix random and specific decoys
    ///
    /// Note: ME token specifies position of real source IP (default: random position)
    /// Effective decoy count: 5-10 decoys (more may trigger rate limiting)
    ///
    /// Example: prtip -sS -D RND:10 -p 80,443 target.com
    /// Example: prtip -sS -D 1.2.3.4,ME,5.6.7.8 -p 80 target.com
    #[arg(
        short = 'D',
        long = "decoys",
        value_name = "decoy1,decoy2,...",
        help_heading = "FIREWALL/IDS EVASION AND SPOOFING"
    )]
    pub decoys: Option<String>,

    /// Use bad TCP/IP checksums (nmap --badsum) - Testing/debugging only
    ///
    /// Generate packets with intentionally incorrect checksums. This is used to test
    /// whether firewalls and IDS properly validate checksums before processing packets.
    ///
    /// ⚠️  WARNING: Most targets will drop these packets immediately. This flag is only
    /// useful for testing intermediate devices (firewalls, IDS) to verify they check
    /// checksums. Expect zero responses from the target.
    ///
    /// Example: prtip -sS --badsum -p 80,443 target.com
    #[arg(long, help_heading = "FIREWALL/IDS EVASION AND SPOOFING")]
    pub badsum: bool,

    /// Idle scan using zombie host (nmap -sI) - Ultimate stealth
    ///
    /// Perform completely anonymous port scanning by bouncing probes off a "zombie" host.
    /// Your real IP never contacts the target directly. This is the stealthiest scan
    /// technique available, as all probes appear to come from the zombie.
    ///
    /// Requirements:
    /// - Zombie must have predictable IPID (increments by 1 per packet)
    /// - Zombie must be idle (minimal network traffic)
    /// - Zombie must respond to unsolicited SYN/ACK with RST
    ///
    /// Good zombie candidates:
    /// - Printers and IoT devices (simple network stacks)
    /// - Idle workstations (Windows XP/2003, Linux <4.18)
    /// - Legacy systems with sequential IPID
    ///
    /// ⚠️  NOTE: Modern systems (Linux 4.18+, Windows 10+) use random IPID.
    ///
    /// Example: prtip -sI 192.168.1.100 -p 80,443 target.com
    /// Example: prtip -sI zombie.local --zombie-quality 0.8 -p 1-1000 target.com
    #[arg(
        short = 'I',
        long = "idle-scan",
        value_name = "ZOMBIE_HOST",
        help_heading = "FIREWALL/IDS EVASION AND SPOOFING"
    )]
    pub idle_scan: Option<String>,

    /// Minimum zombie quality score for idle scan (0.0-1.0, default: 0.7)
    ///
    /// When using idle scan, only accept zombies with quality score above this threshold.
    /// Higher values = more selective (better zombies, longer discovery time).
    ///
    /// Quality factors:
    /// - IPID pattern (Sequential=0.5, Broken256=0.4, Random=0.0)
    /// - Consistency (perfect=0.3, variable=0.0-0.3)
    /// - Latency (<50ms=0.2, >1000ms=0.0)
    ///
    /// Example: prtip -sI zombie.local --zombie-quality 0.9 -p 80,443 target.com
    #[arg(
        long,
        value_name = "0.0-1.0",
        default_value = "0.7",
        help_heading = "FIREWALL/IDS EVASION AND SPOOFING"
    )]
    pub zombie_quality: f32,

    // ============================================================================
    // MISCELLANEOUS FLAGS (nmap-compatible)
    // ============================================================================
    /// List network interfaces and routes (nmap --iflist)
    ///
    /// Display all available network interfaces with IP addresses, MAC addresses,
    /// and interface status. Useful for selecting the correct interface.
    ///
    /// Example: prtip --iflist
    #[arg(long, help_heading = "MISCELLANEOUS")]
    pub iflist: bool,

    /// Send using raw ethernet frames (nmap --send-eth)
    ///
    /// Force use of raw ethernet frames instead of IP packets.
    /// Advanced option for low-level packet crafting.
    ///
    /// Example: prtip --send-eth \<target\>
    #[arg(long, help_heading = "MISCELLANEOUS", group = "send_method")]
    pub send_eth: bool,

    /// Send using IP packets (nmap --send-ip)
    ///
    /// Force use of IP packets instead of raw ethernet frames.
    /// Default behavior for most scans.
    ///
    /// Example: prtip --send-ip \<target\>
    #[arg(long, help_heading = "MISCELLANEOUS", group = "send_method")]
    pub send_ip: bool,

    /// Assume user is privileged (nmap --privileged)
    ///
    /// Skip privilege checks and assume raw socket access.
    /// Use when running as root/Administrator.
    ///
    /// Example: prtip --privileged \<target\>
    #[arg(long, help_heading = "MISCELLANEOUS", group = "privilege")]
    pub privileged: bool,

    /// Assume user is unprivileged (nmap --unprivileged)
    ///
    /// Force TCP connect scan mode without privilege checks.
    /// Mutually exclusive with --privileged.
    ///
    /// Example: prtip --unprivileged \<target\>
    #[arg(long, help_heading = "MISCELLANEOUS", group = "privilege")]
    pub unprivileged: bool,

    /// Never perform DNS resolution (nmap -n)
    ///
    /// Disable DNS lookups for faster scanning. Only IP addresses in output.
    /// Reduces scan time but loses hostname information.
    ///
    /// Example: prtip -n \<target\>
    #[arg(short = 'n', long = "no-dns", help_heading = "MISCELLANEOUS")]
    pub no_dns: bool,

    // ============================================================================
    // IPv6 PROTOCOL SELECTION FLAGS
    // ============================================================================
    /// Enable IPv6 scanning only (nmap -6)
    ///
    /// Force IPv6-only scanning mode. Rejects IPv4 targets with clear error message.
    /// DNS resolution returns only AAAA records. All scanners use IPv6 packet building.
    /// Mutually exclusive with -4 and --dual-stack.
    ///
    /// Example: prtip -6 -p 80,443 2001:db8::1
    /// Example: prtip -6 -sS -p 1-1000 fe80::1%eth0
    #[arg(
        short = '6',
        long = "ipv6",
        help_heading = "IPv6 OPTIONS",
        group = "ip_version"
    )]
    pub ipv6: bool,

    /// Enable IPv4 scanning only (nmap -4)
    ///
    /// Force IPv4-only scanning mode (default behavior, made explicit).
    /// Rejects IPv6 targets with clear error message.
    /// DNS resolution returns only A records. All scanners use IPv4 packet building.
    /// Mutually exclusive with -6 and --dual-stack.
    ///
    /// Example: prtip -4 -p 80,443 192.168.1.1
    #[arg(
        short = '4',
        long = "ipv4",
        help_heading = "IPv6 OPTIONS",
        group = "ip_version"
    )]
    pub ipv4: bool,

    /// Allow both IPv4 and IPv6 targets (dual-stack mode)
    ///
    /// Accept mixed IPv4/IPv6 targets. DNS resolution returns both A and AAAA records.
    /// Scanners auto-detect protocol per target. This is the default behavior if no
    /// protocol flags are specified. Mutually exclusive with -4 and -6.
    ///
    /// Example: prtip --dual-stack -p 80 192.168.1.1 2001:db8::1
    #[arg(
        long = "dual-stack",
        help_heading = "IPv6 OPTIONS",
        group = "ip_version"
    )]
    pub dual_stack: bool,

    // ============================================================================
    // SCAN TEMPLATES
    // ============================================================================
    /// Use a predefined scan template
    ///
    /// Templates provide pre-configured scan settings for common scenarios.
    /// CLI flags override template values.
    ///
    /// Built-in templates:
    ///   - web-servers: Common web ports (80, 443, 8080, etc.)
    ///   - databases: Database ports (MySQL, PostgreSQL, MongoDB, Redis)
    ///   - quick: Fast scan of top 100 ports
    ///   - thorough: All 65,535 ports with service/OS detection
    ///   - stealth: Minimal detection (FIN scan, slow timing)
    ///   - discovery: Host discovery only (no port scan)
    ///   - ssl-only: HTTPS ports with certificate analysis
    ///   - admin-panels: Remote admin ports (SSH, RDP, VNC)
    ///   - mail-servers: Email ports (SMTP, IMAP, POP3)
    ///   - file-shares: File sharing protocols (SMB, NFS, FTP)
    ///
    /// Custom templates can be defined in ~/.prtip/templates.toml
    ///
    /// Examples:
    ///   prtip --template web-servers 192.168.1.0/24
    ///   prtip --template stealth -T4 target.com  # Override timing
    #[arg(long, value_name = "NAME", help_heading = "SCAN TEMPLATES")]
    pub template: Option<String>,

    /// List all available scan templates
    ///
    /// Shows built-in and custom templates with descriptions.
    ///
    /// Example: prtip --list-templates
    #[arg(long, help_heading = "SCAN TEMPLATES")]
    pub list_templates: bool,

    /// Show details for a specific template
    ///
    /// Displays the configuration and settings for a template.
    ///
    /// Example: prtip --show-template web-servers
    #[arg(long, value_name = "NAME", help_heading = "SCAN TEMPLATES")]
    pub show_template: Option<String>,
}

/// IP Version selection for scanning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IpVersion {
    /// IPv4-only mode (-4 flag)
    V4Only,
    /// IPv6-only mode (-6 flag)
    V6Only,
    /// Dual-stack mode (--dual-stack or default)
    #[default]
    DualStack,
}

impl Args {
    /// Get IP version from command-line flags
    ///
    /// Returns the selected IP version based on -4, -6, or --dual-stack flags.
    /// Default is DualStack if no flags are specified.
    pub fn get_ip_version(&self) -> IpVersion {
        if self.ipv6 {
            IpVersion::V6Only
        } else if self.ipv4 {
            IpVersion::V4Only
        } else {
            IpVersion::DualStack // default
        }
    }

    /// Validate target IP addresses against the selected IP version
    ///
    /// Checks that all parsed targets match the protocol mode (-4, -6, or --dual-stack).
    /// Returns an error if there's a protocol mismatch.
    ///
    /// # Arguments
    ///
    /// * `targets` - Slice of parsed ScanTarget structures
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - IPv4 target specified with -6 (IPv6-only) flag
    /// - IPv6 target specified with -4 (IPv4-only) flag
    pub fn validate_target_protocols(
        &self,
        targets: &[prtip_core::ScanTarget],
    ) -> anyhow::Result<()> {
        use std::net::IpAddr;

        let ip_version = self.get_ip_version();

        for target in targets {
            let ip = target.network.ip();

            match (ip, ip_version) {
                (IpAddr::V4(addr), IpVersion::V6Only) => {
                    anyhow::bail!(
                        "Error: IPv4 target '{}' specified with -6 (IPv6-only mode)\n\
                        Hint: Remove -6 flag to scan IPv4 targets, or use --dual-stack for mixed scanning",
                        addr
                    );
                }
                (IpAddr::V6(addr), IpVersion::V4Only) => {
                    anyhow::bail!(
                        "Error: IPv6 target '{}' specified with -4 (IPv4-only mode)\n\
                        Hint: Remove -4 flag to scan IPv6 targets, or use --dual-stack for mixed scanning",
                        addr
                    );
                }
                (_, IpVersion::DualStack) => {
                    // Accept all targets in dual-stack mode
                }
                _ => {
                    // Correct protocol for mode
                }
            }
        }

        Ok(())
    }

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

        // Validate progress style
        if !["compact", "detailed", "bars"].contains(&self.progress_style.as_str()) {
            anyhow::bail!("Progress style must be one of: compact, detailed, bars");
        }

        if self.progress_interval == 0 {
            anyhow::bail!("Progress interval must be greater than 0");
        }

        if self.progress_interval > 60 {
            anyhow::bail!("Progress interval cannot exceed 1 minute (60 seconds)");
        }

        if self.stats_interval == 0 {
            anyhow::bail!("Stats interval must be greater than 0");
        }

        if self.stats_interval > 3600 {
            anyhow::bail!("Stats interval cannot exceed 1 hour (3600 seconds)");
        }

        // Validate new flags
        if let Some(ratio) = self.port_ratio {
            if !(0.0..=1.0).contains(&ratio) {
                anyhow::bail!("Port ratio must be between 0.0 and 1.0");
            }
        }

        if let Some(retries) = self.max_retries {
            if retries > 20 {
                anyhow::bail!("Max retries cannot exceed 20");
            }
        }

        if let Some(rate) = self.min_rate {
            if rate == 0 {
                anyhow::bail!("Min rate must be greater than 0");
            }
            if rate > 100_000_000 {
                anyhow::bail!("Min rate cannot exceed 100M pps");
            }
        }

        // Validate fragmentation/evasion flags
        if let Some(mtu) = self.mtu {
            if mtu < 68 {
                anyhow::bail!("MTU must be at least 68 bytes (20 IP header + 8 fragment minimum)");
            }
            if mtu % 8 != 0 {
                anyhow::bail!(
                    "MTU must be a multiple of 8 (IP fragment offset is in 8-byte units)"
                );
            }
            if mtu > 65535 {
                anyhow::bail!("MTU cannot exceed 65535 bytes");
            }
        }

        // If -f flag is used without --mtu, we'll use default of 28 bytes (handled in config)
        if self.fragment && self.mtu.is_some() {
            if let Some(mtu) = self.mtu {
                if mtu < 28 {
                    anyhow::bail!("When using -f with --mtu, MTU should be at least 28 bytes (20 IP + 8 data)");
                }
            }
        }

        // Validate hostgroup limits (Sprint 5.4)
        if self.max_hostgroup == 0 {
            anyhow::bail!("Max hostgroup must be greater than 0");
        }
        if self.max_hostgroup > 10_000 {
            anyhow::bail!("Max hostgroup cannot exceed 10,000");
        }
        if self.min_hostgroup == 0 {
            anyhow::bail!("Min hostgroup must be greater than 0");
        }
        if self.min_hostgroup > self.max_hostgroup {
            anyhow::bail!(
                "Min hostgroup ({}) cannot exceed max hostgroup ({})",
                self.min_hostgroup,
                self.max_hostgroup
            );
        }
        if let Some(parallelism) = self.max_parallelism {
            if parallelism == 0 {
                anyhow::bail!("Max parallelism must be greater than 0");
            }
            if parallelism > 10_000 {
                anyhow::bail!("Max parallelism cannot exceed 10,000");
            }
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

    /// Get effective maximum hostgroup (handles --max-parallelism alias)
    ///
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
    pub fn to_config(&self) -> prtip_core::Result<Config> {
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
        let scan_type = if self.nmap_idle.is_some() || self.idle_scan.is_some() {
            ScanType::Idle // -sI flag (preprocessed to --nmap-idle) or direct -I flag
        } else if self.nmap_syn {
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
                ScanTypeArg::Idle => {
                    // Idle scan requires zombie host
                    if self.idle_scan.is_none() && self.nmap_idle.is_none() {
                        return Err(prtip_core::Error::Config(
                            "Idle scan (-s idle) requires zombie host (-sI <zombie> or -I <zombie>)".into(),
                        ));
                    }
                    ScanType::Idle
                }
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

        Ok(Config {
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
                    enable_tls: !self.no_tls,
                    capture_raw: self.capture_raw_responses,
                },
                progress: show_progress,
                event_bus: None, // Event bus integration for TUI (Phase 6)
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
                numa_enabled: self.numa && !self.no_numa, // Enabled only if --numa and not --no-numa
            },
            evasion: EvasionConfig {
                fragment_packets: self.fragment,
                mtu: self.mtu.or({
                    // If -f flag is used without --mtu, default to aggressive 28-byte fragments
                    if self.fragment {
                        Some(28) // 20 IP header + 8 data (Nmap -f equivalent)
                    } else {
                        None
                    }
                }),
                ttl: self.ttl,
                decoys: if let Some(ref spec) = self.decoys {
                    Some(parse_decoy_spec(spec).map_err(|e| {
                        prtip_core::Error::Config(format!("Invalid -D/--decoy: {}", e))
                    })?)
                } else {
                    None
                },
                bad_checksums: self.badsum,
            },
        })
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
    /// Idle scan (zombie scan, requires zombie host with -sI)
    Idle,
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

/// Parse decoy specification from -D flag
///
/// Formats supported:
/// - RND:N - Generate N random decoy IPs (e.g., RND:10)
/// - ip1,ip2,ip3 - Manual list of IPs
/// - ip1,ME,ip2 - Manual list with ME positioning
///
/// Examples:
/// - parse_decoy_spec("RND:5") → Random { count: 5, me_position: None }
/// - parse_decoy_spec("192.168.1.1,ME,192.168.1.2") → Manual { ips: [1.1, 1.2], me_position: Some(1) }
fn parse_decoy_spec(spec: &str) -> Result<DecoyConfig, String> {
    // Handle RND:N format
    if spec.starts_with("RND:") || spec.starts_with("rnd:") {
        let count_str = &spec[4..];
        let count = count_str
            .parse::<usize>()
            .map_err(|_| format!("Invalid RND count: '{}'", count_str))?;

        if count == 0 {
            return Err("RND count must be at least 1".to_string());
        }
        if count > 1000 {
            return Err("RND count must not exceed 1000".to_string());
        }

        return Ok(DecoyConfig::Random {
            count,
            me_position: None, // ME appended by default for RND
        });
    }

    // Handle manual IP list (comma-separated)
    let parts: Vec<&str> = spec.split(',').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err("Decoy list cannot be empty".to_string());
    }

    let mut ips = Vec::new();
    let mut me_position = None;

    for (idx, part) in parts.iter().enumerate() {
        if part.eq_ignore_ascii_case("ME") {
            if me_position.is_some() {
                return Err("ME can only appear once in decoy list".to_string());
            }
            me_position = Some(idx);
        } else {
            // Parse IP address
            let ip = part
                .parse::<Ipv4Addr>()
                .map_err(|_| format!("Invalid IP address: '{}'", part))?;
            ips.push(ip);
        }
    }

    if ips.is_empty() {
        return Err("Decoy list must contain at least one IP address".to_string());
    }

    Ok(DecoyConfig::Manual { ips, me_position })
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
        let config = args.to_config().unwrap();

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
        let config = args.to_config().unwrap();

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
        let config = args.to_config().unwrap();

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
    fn test_progress_style_default() {
        let args = Args::parse_from(["prtip", "192.168.1.1"]);
        assert_eq!(args.progress_style, "compact");
    }

    #[test]
    fn test_progress_style_detailed() {
        let args = Args::parse_from(["prtip", "--progress-style", "detailed", "192.168.1.1"]);
        assert_eq!(args.progress_style, "detailed");
    }

    #[test]
    fn test_progress_style_bars() {
        let args = Args::parse_from(["prtip", "--progress-style", "bars", "192.168.1.1"]);
        assert_eq!(args.progress_style, "bars");
    }

    #[test]
    fn test_validate_invalid_progress_style() {
        let args = Args::parse_from(["prtip", "--progress-style", "invalid", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_progress_interval_default() {
        let args = Args::parse_from(["prtip", "192.168.1.1"]);
        assert_eq!(args.progress_interval, 1);
    }

    #[test]
    fn test_progress_interval_custom() {
        let args = Args::parse_from(["prtip", "--progress-interval", "5", "192.168.1.1"]);
        assert_eq!(args.progress_interval, 5);
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

    // ========================================================================
    // Tests for Sprint 4.16 new flags
    // ========================================================================

    #[test]
    fn test_host_discovery_ping_only() {
        let args = Args::parse_from(["prtip", "--ping-only", "192.168.1.0/24"]);
        assert!(args.ping_only);
    }

    #[test]
    fn test_host_discovery_arp_ping() {
        let args = Args::parse_from(["prtip", "-PR", "192.168.1.0/24"]);
        assert!(args.arp_ping);
    }

    #[test]
    fn test_host_discovery_tcp_syn_ping() {
        let args = Args::parse_from(["prtip", "--ps", "80,443", "192.168.1.1"]);
        assert_eq!(args.tcp_syn_ping, Some("80,443".to_string()));
    }

    #[test]
    fn test_host_discovery_tcp_ack_ping() {
        let args = Args::parse_from(["prtip", "--pa", "80", "192.168.1.1"]);
        assert_eq!(args.tcp_ack_ping, Some("80".to_string()));
    }

    #[test]
    fn test_host_discovery_udp_ping() {
        let args = Args::parse_from(["prtip", "--pu", "53", "192.168.1.1"]);
        assert_eq!(args.udp_ping, Some("53".to_string()));
    }

    #[test]
    fn test_host_discovery_icmp_echo() {
        let args = Args::parse_from(["prtip", "--pe", "192.168.1.1"]);
        assert!(args.icmp_echo_ping);
    }

    #[test]
    fn test_host_discovery_icmp_timestamp() {
        let args = Args::parse_from(["prtip", "--pp", "192.168.1.1"]);
        assert!(args.icmp_timestamp_ping);
    }

    #[test]
    fn test_port_spec_no_randomize() {
        let args = Args::parse_from(["prtip", "-r", "192.168.1.1"]);
        assert!(args.no_randomize);
    }

    #[test]
    fn test_port_spec_port_ratio() {
        let args = Args::parse_from(["prtip", "--port-ratio", "0.5", "192.168.1.1"]);
        assert_eq!(args.port_ratio, Some(0.5));
    }

    #[test]
    fn test_validate_port_ratio_valid() {
        let args = Args::parse_from(["prtip", "--port-ratio", "0.5", "192.168.1.1"]);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_port_ratio_invalid() {
        let args = Args::parse_from(["prtip", "--port-ratio", "1.5", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_timing_max_retries() {
        let args = Args::parse_from(["prtip", "--max-retries", "5", "192.168.1.1"]);
        assert_eq!(args.max_retries, Some(5));
    }

    #[test]
    fn test_timing_host_timeout() {
        let args = Args::parse_from(["prtip", "--host-timeout", "30m", "192.168.1.1"]);
        assert_eq!(args.host_timeout, Some("30m".to_string()));
    }

    #[test]
    fn test_timing_max_scan_delay() {
        let args = Args::parse_from(["prtip", "--max-scan-delay", "500ms", "192.168.1.1"]);
        assert_eq!(args.max_scan_delay, Some("500ms".to_string()));
    }

    #[test]
    fn test_timing_min_rate() {
        let args = Args::parse_from(["prtip", "--min-rate", "1000", "192.168.1.1"]);
        assert_eq!(args.min_rate, Some(1000));
    }

    #[test]
    fn test_validate_min_rate_zero() {
        let args = Args::parse_from(["prtip", "--min-rate", "0", "192.168.1.1"]);
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_output_open_flag() {
        let args = Args::parse_from(["prtip", "--open", "192.168.1.1"]);
        assert!(args.open);
    }

    #[test]
    fn test_output_packet_trace() {
        let args = Args::parse_from(["prtip", "--packet-trace", "192.168.1.1"]);
        assert!(args.packet_trace);
    }

    #[test]
    fn test_output_reason() {
        let args = Args::parse_from(["prtip", "--reason", "192.168.1.1"]);
        assert!(args.reason);
    }

    #[test]
    fn test_output_stats_every() {
        let args = Args::parse_from(["prtip", "--stats-every", "5s", "192.168.1.1"]);
        assert_eq!(args.stats_every, Some("5s".to_string()));
    }

    #[test]
    fn test_misc_iflist() {
        let args = Args::parse_from(["prtip", "--iflist", "192.168.1.1"]);
        assert!(args.iflist);
    }

    #[test]
    fn test_misc_send_eth() {
        let args = Args::parse_from(["prtip", "--send-eth", "192.168.1.1"]);
        assert!(args.send_eth);
        assert!(!args.send_ip);
    }

    #[test]
    fn test_misc_send_ip() {
        let args = Args::parse_from(["prtip", "--send-ip", "192.168.1.1"]);
        assert!(args.send_ip);
        assert!(!args.send_eth);
    }

    #[test]
    fn test_misc_privileged() {
        let args = Args::parse_from(["prtip", "--privileged", "192.168.1.1"]);
        assert!(args.privileged);
        assert!(!args.unprivileged);
    }

    #[test]
    fn test_misc_unprivileged() {
        let args = Args::parse_from(["prtip", "--unprivileged", "192.168.1.1"]);
        assert!(args.unprivileged);
        assert!(!args.privileged);
    }

    #[test]
    fn test_misc_privileged_unprivileged_mutual_exclusion() {
        // Should fail due to ArgGroup
        let result = Args::try_parse_from(vec![
            "prtip",
            "--privileged",
            "--unprivileged",
            "192.168.1.1",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_misc_no_dns() {
        let args = Args::parse_from(["prtip", "-n", "192.168.1.1"]);
        assert!(args.no_dns);
    }

    #[test]
    fn test_misc_no_dns_long() {
        let args = Args::parse_from(["prtip", "--no-dns", "192.168.1.1"]);
        assert!(args.no_dns);
    }

    // ========================================================================
    // Sprint 4.20 Phase 8: Decoy Flag (-D) Tests
    // ========================================================================

    #[test]
    fn test_decoy_flag_rnd_5() {
        let args = Args::parse_from(["prtip", "-D", "RND:5", "-p", "80", "127.0.0.1"]);
        assert!(args.decoys.is_some());
        assert_eq!(args.decoys.as_ref().unwrap(), "RND:5");

        // Verify config parsing
        let config = args.to_config().expect("Config should parse");
        assert!(config.evasion.decoys.is_some());
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Random { count, me_position } => {
                assert_eq!(count, 5);
                assert_eq!(me_position, None); // ME appended by default
            }
            _ => panic!("Expected Random variant"),
        }
    }

    #[test]
    fn test_decoy_flag_rnd_10() {
        let args = Args::parse_from(["prtip", "-D", "RND:10", "-p", "80", "127.0.0.1"]);
        let config = args.to_config().expect("Config should parse");
        assert!(config.evasion.decoys.is_some());
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Random { count, .. } => assert_eq!(count, 10),
            _ => panic!("Expected Random variant"),
        }
    }

    #[test]
    fn test_decoy_flag_manual_single_ip() {
        let args = Args::parse_from(["prtip", "-D", "192.168.1.5", "-p", "80", "127.0.0.1"]);
        let config = args.to_config().expect("Config should parse");
        assert!(config.evasion.decoys.is_some());
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Manual { ips, me_position } => {
                assert_eq!(ips.len(), 1);
                assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 5));
                assert_eq!(me_position, None);
            }
            _ => panic!("Expected Manual variant"),
        }
    }

    #[test]
    fn test_decoy_flag_manual_multiple_ips() {
        let args = Args::parse_from([
            "prtip",
            "-D",
            "192.168.1.5,192.168.1.10,192.168.1.15",
            "-p",
            "80",
            "127.0.0.1",
        ]);
        let config = args.to_config().expect("Config should parse");
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Manual { ips, .. } => {
                assert_eq!(ips.len(), 3);
                assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 5));
                assert_eq!(ips[1], Ipv4Addr::new(192, 168, 1, 10));
                assert_eq!(ips[2], Ipv4Addr::new(192, 168, 1, 15));
            }
            _ => panic!("Expected Manual variant"),
        }
    }

    #[test]
    fn test_decoy_flag_with_me_first() {
        let args = Args::parse_from([
            "prtip",
            "-D",
            "ME,192.168.1.5,192.168.1.10",
            "-p",
            "80",
            "127.0.0.1",
        ]);
        let config = args.to_config().expect("Config should parse");
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Manual { ips, me_position } => {
                assert_eq!(ips.len(), 2);
                assert_eq!(me_position, Some(0)); // ME at position 0
            }
            _ => panic!("Expected Manual variant"),
        }
    }

    #[test]
    fn test_decoy_flag_with_me_middle() {
        let args = Args::parse_from([
            "prtip",
            "-D",
            "192.168.1.5,ME,192.168.1.10",
            "-p",
            "80",
            "127.0.0.1",
        ]);
        let config = args.to_config().expect("Config should parse");
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Manual { ips, me_position } => {
                assert_eq!(ips.len(), 2);
                assert_eq!(me_position, Some(1)); // ME at position 1 (middle)
            }
            _ => panic!("Expected Manual variant"),
        }
    }

    #[test]
    fn test_decoy_flag_with_me_last() {
        let args = Args::parse_from([
            "prtip",
            "-D",
            "192.168.1.5,192.168.1.10,ME",
            "-p",
            "80",
            "127.0.0.1",
        ]);
        let config = args.to_config().expect("Config should parse");
        match config.evasion.decoys.unwrap() {
            DecoyConfig::Manual { ips, me_position } => {
                assert_eq!(ips.len(), 2);
                assert_eq!(me_position, Some(2)); // ME at position 2 (last)
            }
            _ => panic!("Expected Manual variant"),
        }
    }

    #[test]
    fn test_decoy_flag_with_scan_type() {
        let args = Args::parse_from([
            "prtip",
            "--scan-type",
            "syn",
            "-D",
            "RND:5",
            "-p",
            "80",
            "127.0.0.1",
        ]);
        assert!(args.decoys.is_some());
        let config = args.to_config().expect("Config should parse");
        assert!(config.evasion.decoys.is_some());
    }

    #[test]
    fn test_decoy_flag_combined_evasion() {
        let args = Args::parse_from([
            "prtip",
            "--scan-type",
            "syn",
            "-D",
            "RND:5",
            "-f",
            "--ttl",
            "32",
            "--badsum",
            "-p",
            "80",
            "127.0.0.1",
        ]);
        let config = args.to_config().expect("Config should parse");

        // Verify all evasion features enabled
        assert!(config.evasion.decoys.is_some());
        assert!(config.evasion.fragment_packets);
        assert_eq!(config.evasion.ttl, Some(32));
        assert!(config.evasion.bad_checksums);
    }

    #[test]
    fn test_decoy_flag_invalid_rnd_format() {
        let args = Args::parse_from([
            "prtip",
            "-D",
            "RND:abc", // Invalid: not a number
            "-p",
            "80",
            "127.0.0.1",
        ]);
        let result = args.to_config();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid RND count") || err.contains("Invalid -D"));
    }
}
