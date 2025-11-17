# API Reference

Complete API documentation for ProRT-IP's public interfaces.

**Version:** 2.0
**Last Updated:** November 2025

---

## Core Scanner API

### Scanner

Main entry point for executing network scans.

```rust
pub struct Scanner { /* private fields */ }
```

#### Constructor

```rust
impl Scanner {
    /// Create a new scanner with configuration
    ///
    /// # Arguments
    /// * `config` - Scan configuration
    ///
    /// # Returns
    /// * `Result<Self>` - Scanner instance or error
    ///
    /// # Errors
    /// * `Error::InvalidTarget` - Invalid target specification
    /// * `Error::InvalidPortRange` - Invalid port range
    /// * `Error::PermissionDenied` - Insufficient privileges for raw sockets
    ///
    /// # Example
    /// ```rust
    /// use prtip_core::{Scanner, ScanConfig, ScanType};
    ///
    /// let config = ScanConfig {
    ///     targets: vec!["192.168.1.0/24".parse()?],
    ///     ports: PortRange::new(1, 1000),
    ///     scan_type: ScanType::Syn,
    ///     ..Default::default()
    /// };
    ///
    /// let scanner = Scanner::new(config)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(config: ScanConfig) -> Result<Self>
}
```

#### Execution Methods

```rust
impl Scanner {
    /// Execute the scan asynchronously
    ///
    /// Runs the scan with default progress tracking.
    ///
    /// # Returns
    /// * `Result<ScanReport>` - Complete scan report with all results
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::{Scanner, ScanConfig};
    /// # let scanner = Scanner::new(ScanConfig::default())?;
    /// let report = scanner.execute().await?;
    /// println!("Scanned {} hosts, found {} open ports",
    ///     report.stats.hosts_scanned,
    ///     report.stats.ports_open);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn execute(&self) -> Result<ScanReport>

    /// Execute with progress callback
    ///
    /// Provides real-time progress updates during scanning.
    ///
    /// # Arguments
    /// * `callback` - Function called periodically with progress updates
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::{Scanner, ScanConfig, ScanProgress};
    /// # let scanner = Scanner::new(ScanConfig::default())?;
    /// let report = scanner.execute_with_progress(|progress| {
    ///     println!("Progress: {:.1}% ({}/{} ports)",
    ///         progress.percentage(),
    ///         progress.completed,
    ///         progress.total);
    /// }).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn execute_with_progress<F>(&self, callback: F) -> Result<ScanReport>
    where
        F: Fn(ScanProgress) + Send + 'static

    /// Execute with event stream
    ///
    /// Returns event receiver for real-time scan events.
    ///
    /// # Returns
    /// * Tuple of (ScanReport, EventReceiver)
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::Scanner;
    /// # let scanner = Scanner::new(Default::default())?;
    /// let (report, mut events) = scanner.execute_with_events().await?;
    ///
    /// while let Some(event) = events.recv().await {
    ///     match event {
    ///         ScanEvent::PortFound { target, port, state } => {
    ///             println!("Found: {}:{} ({:?})", target, port, state);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn execute_with_events(&self) -> Result<(ScanReport, EventReceiver)>
}
```

#### Control Methods

```rust
impl Scanner {
    /// Pause the scan
    ///
    /// Suspends packet transmission while preserving state.
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::Scanner;
    /// # let scanner = Scanner::new(Default::default())?;
    /// scanner.pause()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn pause(&self) -> Result<()>

    /// Resume a paused scan
    ///
    /// Resumes packet transmission from paused state.
    pub fn resume(&self) -> Result<()>

    /// Stop the scan gracefully
    ///
    /// Waits for in-flight probes before terminating.
    pub fn stop(&self) -> Result<()>
}
```

---

### ScanConfig

Configuration for scan execution.

```rust
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Target hosts/networks to scan
    pub targets: Vec<Target>,

    /// Ports to scan
    pub ports: PortRange,

    /// Type of scan to perform
    pub scan_type: ScanType,

    /// Skip host discovery (assume all hosts are up)
    pub skip_discovery: bool,

    /// Enable service version detection
    pub service_detection: bool,

    /// Service detection intensity (0-9, default 7)
    pub service_intensity: u8,

    /// Enable OS fingerprinting
    pub os_detection: bool,

    /// Timing template (T0-T5)
    pub timing: TimingTemplate,

    /// Maximum packet rate (packets/second)
    pub max_rate: Option<u32>,

    /// Minimum packet rate (packets/second)
    pub min_rate: Option<u32>,

    /// Maximum retransmissions per probe
    pub max_retries: u8,

    /// Maximum scan duration
    pub max_duration: Option<Duration>,

    /// Output configuration
    pub output: OutputConfig,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            ports: PortRange::new(1, 1000),
            scan_type: ScanType::Syn,
            skip_discovery: false,
            service_detection: false,
            service_intensity: 7,
            os_detection: false,
            timing: TimingTemplate::Normal,
            max_rate: Some(100_000),  // 100K pps
            min_rate: None,
            max_retries: 2,
            max_duration: None,
            output: OutputConfig::default(),
        }
    }
}
```

**Field Descriptions:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `targets` | `Vec<Target>` | `[]` | Target hosts/networks (IP, CIDR, hostname, range) |
| `ports` | `PortRange` | `1-1000` | Ports to scan (individual or ranges) |
| `scan_type` | `ScanType` | `Syn` | Scan technique (SYN, Connect, UDP, etc.) |
| `skip_discovery` | `bool` | `false` | Assume all hosts up (skip ping) |
| `service_detection` | `bool` | `false` | Enable version detection |
| `service_intensity` | `u8` | `7` | Probe intensity 0-9 (higher = more probes) |
| `os_detection` | `bool` | `false` | Enable OS fingerprinting |
| `timing` | `TimingTemplate` | `Normal` | Timing template T0-T5 |
| `max_rate` | `Option<u32>` | `100000` | Maximum packets/second (None = unlimited) |
| `min_rate` | `Option<u32>` | `None` | Minimum packets/second |
| `max_retries` | `u8` | `2` | Retransmissions per probe |
| `max_duration` | `Option<Duration>` | `None` | Maximum scan time (None = no limit) |
| `output` | `OutputConfig` | `default` | Output formats and destinations |

---

### ScanType

Supported scan techniques.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    /// TCP SYN scan (half-open, stealth)
    ///
    /// Most common scan type. Sends SYN packets without completing
    /// the TCP handshake (never sends final ACK).
    Syn,

    /// TCP Connect scan (full connection)
    ///
    /// Completes full TCP handshake. More detectable but works
    /// without raw socket privileges.
    Connect,

    /// UDP scan
    ///
    /// Sends UDP probes with protocol-specific payloads. 10-100x
    /// slower than TCP due to ICMP rate limiting.
    Udp,

    /// TCP FIN scan (firewall evasion)
    ///
    /// Sends FIN packets. Open ports ignore, closed ports send RST.
    /// May bypass simple firewalls.
    Fin,

    /// TCP NULL scan (no flags)
    ///
    /// Sends packets with no TCP flags set. Similar to FIN scan
    /// for firewall evasion.
    Null,

    /// TCP Xmas scan (FIN+PSH+URG)
    ///
    /// Sends packets with FIN, PSH, and URG flags (lights up like
    /// a Christmas tree). Evasion technique.
    Xmas,

    /// TCP ACK scan (firewall detection)
    ///
    /// Sends ACK packets to detect firewall rules. Distinguishes
    /// between filtered and unfiltered ports.
    Ack,

    /// TCP Window scan (advanced)
    ///
    /// Examines TCP window field in RST responses to determine
    /// port state. More reliable than ACK scan.
    Window,

    /// Idle scan (zombie, maximum anonymity)
    ///
    /// Uses third-party "zombie" host to scan target. Attacker's
    /// IP never directly contacts target.
    ///
    /// # Requirements
    /// - Zombie host must be idle (predictable IPID)
    /// - Zombie must use incremental IPID globally
    /// - Zombie must respond to unsolicited SYN/ACK with RST
    Idle {
        /// Zombie host IP address
        zombie: IpAddr,
    },
}
```

**Scan Type Comparison:**

| Scan Type | Speed | Stealth | Privileges | Firewall Evasion |
|-----------|-------|---------|------------|------------------|
| SYN | ⚡⚡⚡ Fast | 🔒 Medium | Root/Admin | Low |
| Connect | ⚡⚡ Medium | 🔓 Low | None | None |
| UDP | ⚡ Slow | 🔒 Medium | Root/Admin | Low |
| FIN/NULL/Xmas | ⚡⚡ Medium | 🔒🔒 High | Root/Admin | High |
| ACK | ⚡⚡⚡ Fast | 🔒 Medium | Root/Admin | N/A (firewall test) |
| Idle | ⚡ Slow | 🔒🔒🔒 Maximum | Root/Admin | Maximum |

---

### TimingTemplate

Predefined timing configurations (T0-T5).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingTemplate {
    /// Paranoid (T0): IDS evasion, 5-minute delays
    Paranoid,

    /// Sneaky (T1): Slow IDS evasion, 15-second delays
    Sneaky,

    /// Polite (T2): Less bandwidth/target load, 0.4-second delays
    Polite,

    /// Normal (T3): Default balanced scanning
    Normal,

    /// Aggressive (T4): Fast networks, assumes good connectivity
    Aggressive,

    /// Insane (T5): Maximum speed, may overwhelm targets
    Insane,
}

impl TimingTemplate {
    /// Get timing parameters for template
    pub fn params(&self) -> TimingParams {
        match self {
            TimingTemplate::Paranoid => TimingParams {
                initial_timeout: Duration::from_secs(300),
                max_timeout: Duration::from_secs(300),
                max_retries: 5,
                scan_delay: Some(Duration::from_secs(300)),
            },
            TimingTemplate::Sneaky => TimingParams {
                initial_timeout: Duration::from_secs(15),
                max_timeout: Duration::from_secs(15),
                max_retries: 5,
                scan_delay: Some(Duration::from_secs(15)),
            },
            TimingTemplate::Polite => TimingParams {
                initial_timeout: Duration::from_secs(1),
                max_timeout: Duration::from_secs(10),
                max_retries: 5,
                scan_delay: Some(Duration::from_millis(400)),
            },
            TimingTemplate::Normal => TimingParams {
                initial_timeout: Duration::from_secs(1),
                max_timeout: Duration::from_secs(10),
                max_retries: 2,
                scan_delay: None,
            },
            TimingTemplate::Aggressive => TimingParams {
                initial_timeout: Duration::from_millis(500),
                max_timeout: Duration::from_millis(1250),
                max_retries: 6,
                scan_delay: None,
            },
            TimingTemplate::Insane => TimingParams {
                initial_timeout: Duration::from_millis(250),
                max_timeout: Duration::from_millis(300),
                max_retries: 2,
                scan_delay: None,
            },
        }
    }
}
```

**Timing Template Parameters:**

| Template | Initial Timeout | Max Timeout | Max Retries | Scan Delay | Use Case |
|----------|----------------|-------------|-------------|------------|----------|
| T0 (Paranoid) | 300 sec | 300 sec | 5 | 5 min | IDS evasion, ultra-stealth |
| T1 (Sneaky) | 15 sec | 15 sec | 5 | 15 sec | Slow stealth scanning |
| T2 (Polite) | 1 sec | 10 sec | 5 | 0.4 sec | Bandwidth-limited |
| T3 (Normal) | 1 sec | 10 sec | 2 | 0 | Default balanced |
| T4 (Aggressive) | 500 ms | 1250 ms | 6 | 0 | Fast reliable networks |
| T5 (Insane) | 250 ms | 300 ms | 2 | 0 | Maximum speed (risky) |

---

## Network Protocol API

### TcpPacketBuilder

Builder for constructing TCP packets with options.

```rust
pub struct TcpPacketBuilder { /* private fields */ }
```

#### Methods

```rust
impl TcpPacketBuilder {
    /// Create new TCP packet builder
    pub fn new() -> Self

    /// Set source IP and port
    ///
    /// # Example
    /// ```rust
    /// use prtip_net::TcpPacketBuilder;
    /// use std::net::Ipv4Addr;
    ///
    /// let packet = TcpPacketBuilder::new()
    ///     .source(Ipv4Addr::new(10, 0, 0, 1), 12345)
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn source(self, ip: Ipv4Addr, port: u16) -> Self

    /// Set destination IP and port
    pub fn destination(self, ip: Ipv4Addr, port: u16) -> Self

    /// Set sequence number (random for SYN, SipHash-derived for stateless)
    pub fn sequence(self, seq: u32) -> Self

    /// Set acknowledgment number
    pub fn acknowledgment(self, ack: u32) -> Self

    /// Set TCP flags
    ///
    /// # Example
    /// ```rust
    /// use prtip_net::{TcpPacketBuilder, TcpFlags};
    ///
    /// let packet = TcpPacketBuilder::new()
    ///     .flags(TcpFlags::SYN)
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn flags(self, flags: TcpFlags) -> Self

    /// Set window size (default 65535)
    pub fn window_size(self, window: u16) -> Self

    /// Add TCP option
    ///
    /// # Example
    /// ```rust
    /// use prtip_net::{TcpPacketBuilder, TcpOption};
    ///
    /// let packet = TcpPacketBuilder::new()
    ///     .tcp_option(TcpOption::Mss(1460))
    ///     .tcp_option(TcpOption::WindowScale(7))
    ///     .tcp_option(TcpOption::SackPermitted)
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn tcp_option(self, option: TcpOption) -> Self

    /// Build the packet
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - Complete IP+TCP packet bytes
    ///
    /// # Errors
    /// * `Error::InvalidAddress` - Source or destination not set
    /// * `Error::PacketTooLarge` - Options exceed 40-byte maximum
    pub fn build(self) -> Result<Vec<u8>>
}
```

---

### TcpFlags

TCP control flags (bitflags).

```rust
bitflags::bitflags! {
    pub struct TcpFlags: u8 {
        const FIN = 0b00000001;  // Finish connection
        const SYN = 0b00000010;  // Synchronize sequence numbers
        const RST = 0b00000100;  // Reset connection
        const PSH = 0b00001000;  // Push buffered data
        const ACK = 0b00010000;  // Acknowledgment
        const URG = 0b00100000;  // Urgent pointer valid
        const ECE = 0b01000000;  // ECN echo
        const CWR = 0b10000000;  // Congestion window reduced
    }
}
```

**Common Flag Combinations:**

```rust
// SYN scan
TcpFlags::SYN

// SYN/ACK response
TcpFlags::SYN | TcpFlags::ACK

// FIN scan
TcpFlags::FIN

// Xmas scan
TcpFlags::FIN | TcpFlags::PSH | TcpFlags::URG

// NULL scan
TcpFlags::empty()

// ACK scan
TcpFlags::ACK
```

---

### TcpOption

TCP options for packet customization.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TcpOption {
    /// Maximum Segment Size (MSS)
    ///
    /// Typical values: 1460 (Ethernet), 1440 (PPPoE), 536 (dial-up)
    Mss(u16),

    /// Window Scale factor (0-14)
    ///
    /// Multiplier for TCP window field: actual_window = window << scale
    WindowScale(u8),

    /// SACK Permitted
    ///
    /// Enables Selective Acknowledgment
    SackPermitted,

    /// Timestamp (RFC 7323)
    ///
    /// Used for RTT measurement and PAWS (Protection Against Wrapped Sequences)
    Timestamp {
        /// Timestamp value
        tsval: u32,
        /// Timestamp echo reply
        tsecr: u32,
    },

    /// No Operation (padding)
    Nop,

    /// End of Options
    Eol,
}

impl TcpOption {
    /// Get option length in bytes
    pub fn length(&self) -> usize

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8>

    /// Parse from bytes
    ///
    /// # Returns
    /// * Tuple of (TcpOption, bytes_consumed)
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize)>
}
```

---

### PacketCapture

Packet capture interface (libpcap/Npcap/BPF wrapper).

```rust
pub struct PacketCapture { /* private fields */ }

impl PacketCapture {
    /// Create new packet capture on interface
    ///
    /// # Arguments
    /// * `interface` - Network interface name (e.g., "eth0", "\\Device\\NPF_{GUID}")
    ///
    /// # Example
    /// ```no_run
    /// use prtip_net::PacketCapture;
    ///
    /// let mut capture = PacketCapture::new("eth0")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(interface: &str) -> Result<Self>

    /// Set BPF filter for packet filtering
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_net::PacketCapture;
    /// # let mut capture = PacketCapture::new("eth0")?;
    /// // Capture only TCP traffic to port 80
    /// capture.set_filter("tcp and dst port 80")?;
    ///
    /// // Capture SYN/ACK responses from 192.168.1.0/24
    /// capture.set_filter("tcp[tcpflags] & (tcp-syn|tcp-ack) != 0 and src net 192.168.1.0/24")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_filter(&mut self, filter: &str) -> Result<()>

    /// Get next packet (blocking)
    ///
    /// Returns None if timeout expires without packet.
    pub fn next_packet(&mut self) -> Result<Option<Vec<u8>>>

    /// Get next packet asynchronously
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_net::PacketCapture;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut capture = PacketCapture::new("eth0")?;
    /// let packet = capture.recv_async().await?;
    /// println!("Received {} bytes", packet.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recv_async(&mut self) -> Result<Vec<u8>>
}
```

---

## Detection Engine API

### ServiceDetector

Service version detection engine.

```rust
pub struct ServiceDetector { /* private fields */ }

impl ServiceDetector {
    /// Create new service detector
    ///
    /// # Arguments
    /// * `intensity` - Detection intensity (0-9)
    ///   - 0: Registered ports only
    ///   - 7: Recommended default (common + comprehensive)
    ///   - 9: All 187 probes (exhaustive)
    ///
    /// # Example
    /// ```rust
    /// use prtip_detect::ServiceDetector;
    ///
    /// let detector = ServiceDetector::new(7);
    /// ```
    pub fn new(intensity: u8) -> Self

    /// Load probe database from file
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_detect::ServiceDetector;
    /// let mut detector = ServiceDetector::new(7);
    /// detector.load_probes("probes/nmap-service-probes")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_probes(&mut self, path: &str) -> Result<()>

    /// Detect service on target port
    ///
    /// Sends probes and matches responses against database.
    ///
    /// # Returns
    /// * `Option<ServiceInfo>` - Detected service or None if unrecognized
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_detect::ServiceDetector;
    /// # use std::net::SocketAddr;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let detector = ServiceDetector::new(7);
    /// let target: SocketAddr = "192.168.1.1:80".parse()?;
    /// if let Some(service) = detector.detect(target).await? {
    ///     println!("Service: {} {} ({})",
    ///         service.name,
    ///         service.version.unwrap_or_default(),
    ///         service.product.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect(&self, target: SocketAddr) -> Result<Option<ServiceInfo>>
}
```

---

### ServiceInfo

Detected service information.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    /// Service name (e.g., "http", "ssh", "mysql")
    pub name: String,

    /// Product name (e.g., "nginx", "OpenSSH", "MySQL")
    pub product: Option<String>,

    /// Version string (e.g., "1.21.6", "8.9p1", "8.0.32")
    pub version: Option<String>,

    /// Extra info (e.g., "Ubuntu Linux; protocol 2.0")
    pub extra_info: Option<String>,

    /// CPE identifier (e.g., "cpe:/a:openbsd:openssh:8.9p1")
    pub cpe: Option<String>,

    /// OS hint from service banner (e.g., "Ubuntu", "Windows")
    pub os_hint: Option<String>,
}
```

**Example ServiceInfo:**

```rust
ServiceInfo {
    name: "http".to_string(),
    product: Some("nginx".to_string()),
    version: Some("1.21.6".to_string()),
    extra_info: Some("Ubuntu".to_string()),
    cpe: Some("cpe:/a:igor_sysoev:nginx:1.21.6".to_string()),
    os_hint: Some("Linux".to_string()),
}
```

---

### OsDetector

OS fingerprinting engine.

```rust
pub struct OsDetector { /* private fields */ }

impl OsDetector {
    /// Create new OS detector
    pub fn new() -> Self

    /// Load fingerprint database
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_detect::OsDetector;
    /// let mut detector = OsDetector::new();
    /// detector.load_fingerprints("fingerprints/nmap-os-db")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_fingerprints(&mut self, path: &str) -> Result<()>

    /// Detect OS of target
    ///
    /// Sends 16-probe sequence (6 TCP SYN ISN, 2 ICMP, 7 TCP misc, 1 UDP).
    ///
    /// # Arguments
    /// * `target` - Target IP address
    /// * `open_port` - Known open TCP port
    /// * `closed_port` - Known closed TCP port
    ///
    /// # Returns
    /// * `Vec<OsMatch>` - Possible OS matches sorted by confidence (highest first)
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_detect::OsDetector;
    /// # use std::net::Ipv4Addr;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let detector = OsDetector::new();
    /// let target = Ipv4Addr::new(192, 168, 1, 1);
    /// let matches = detector.detect(target, 80, 12345).await?;
    ///
    /// if let Some(best) = matches.first() {
    ///     println!("OS: {} ({}% confidence)", best.name, best.accuracy);
    ///     println!("Class: {} {} {}",
    ///         best.class.vendor,
    ///         best.class.os_family,
    ///         best.class.device_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect(
        &self,
        target: Ipv4Addr,
        open_port: u16,
        closed_port: u16
    ) -> Result<Vec<OsMatch>>
}
```

---

### OsMatch

OS detection match result.

```rust
#[derive(Debug, Clone)]
pub struct OsMatch {
    /// OS name (e.g., "Linux 5.15", "Windows 10 or 11")
    pub name: String,

    /// OS classification
    pub class: OsClass,

    /// Match accuracy (0-100)
    pub accuracy: u8,

    /// CPE identifiers (e.g., ["cpe:/o:linux:linux_kernel:5.15"])
    pub cpe: Vec<String>,

    /// Additional info
    pub info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsClass {
    /// Vendor (e.g., "Linux", "Microsoft", "Apple")
    pub vendor: String,

    /// OS family (e.g., "Linux", "Windows", "embedded")
    pub os_family: String,

    /// OS generation (e.g., "5.x", "10", "11")
    pub os_generation: Option<String>,

    /// Device type (e.g., "general purpose", "router", "firewall")
    pub device_type: String,
}
```

**Example OsMatch:**

```rust
OsMatch {
    name: "Linux 5.15".to_string(),
    class: OsClass {
        vendor: "Linux".to_string(),
        os_family: "Linux".to_string(),
        os_generation: Some("5.x".to_string()),
        device_type: "general purpose".to_string(),
    },
    accuracy: 95,
    cpe: vec!["cpe:/o:linux:linux_kernel:5.15".to_string()],
    info: None,
}
```

---

## Plugin API

### Plugin Trait

Interface for extending scanner functionality.

```rust
pub trait Plugin: Send + Sync {
    /// Plugin name (unique identifier)
    fn name(&self) -> &str;

    /// Plugin version (semantic versioning)
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Initialize plugin with configuration
    ///
    /// # Arguments
    /// * `config` - Plugin-specific configuration
    fn init(&mut self, config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    /// Called when scan starts
    ///
    /// # Arguments
    /// * `scan_info` - Scan metadata (targets, ports, scan type)
    fn on_scan_start(&mut self, _scan_info: &ScanInfo) -> Result<()> {
        Ok(())
    }

    /// Called for each discovered host
    fn on_host_discovered(&mut self, _host: &HostInfo) -> Result<()> {
        Ok(())
    }

    /// Called for each discovered port
    ///
    /// # Example
    /// ```rust
    /// # use prtip_plugins::Plugin;
    /// # struct AlertPlugin;
    /// # impl Plugin for AlertPlugin {
    /// #     fn name(&self) -> &str { "alert" }
    /// fn on_port_discovered(&mut self, result: &ScanResult) -> Result<()> {
    ///     if result.port == 22 && result.state == PortState::Open {
    ///         println!("Alert: SSH port open on {}", result.target);
    ///     }
    ///     Ok(())
    /// }
    /// # }
    /// ```
    fn on_port_discovered(&mut self, _result: &ScanResult) -> Result<()> {
        Ok(())
    }

    /// Called when service is detected
    fn on_service_detected(&mut self, _result: &ScanResult, _service: &ServiceInfo) -> Result<()> {
        Ok(())
    }

    /// Called when scan completes
    fn on_scan_complete(&mut self, _report: &ScanReport) -> Result<()> {
        Ok(())
    }

    /// Cleanup resources
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
```

---

### PluginManager

Manages plugin lifecycle and event dispatch.

```rust
pub struct PluginManager { /* private fields */ }

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self

    /// Register a plugin
    ///
    /// # Example
    /// ```rust
    /// # use prtip_plugins::{PluginManager, Plugin};
    /// # struct MyPlugin;
    /// # impl Plugin for MyPlugin {
    /// #     fn name(&self) -> &str { "my-plugin" }
    /// # }
    /// let mut manager = PluginManager::new();
    /// manager.register(Box::new(MyPlugin))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()>

    /// Load plugin from shared library file
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_plugins::PluginManager;
    /// # let mut manager = PluginManager::new();
    /// manager.load_from_file("plugins/alert.so")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_from_file(&mut self, path: &str) -> Result<()>

    /// Notify all plugins of scan start
    pub fn notify_scan_start(&mut self, scan_info: &ScanInfo) -> Result<()>

    /// Notify all plugins of port discovery
    pub fn notify_port_discovered(&mut self, result: &ScanResult) -> Result<()>

    /// Notify all plugins of service detection
    pub fn notify_service_detected(&mut self, result: &ScanResult, service: &ServiceInfo) -> Result<()>

    /// Notify all plugins of scan completion
    pub fn notify_scan_complete(&mut self, report: &ScanReport) -> Result<()>
}
```

---

## Configuration API

### Target

Target specification (IP, network, hostname, range).

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Single IP address
    Ip(IpAddr),

    /// Network in CIDR notation (e.g., "192.168.1.0/24")
    Network(IpNetwork),

    /// Hostname (requires DNS resolution)
    Hostname(String),

    /// IP range (e.g., "192.168.1.1-192.168.1.254")
    Range {
        start: IpAddr,
        end: IpAddr,
    },
}

impl Target {
    /// Parse target from string
    ///
    /// Supports:
    /// - Single IP: "192.168.1.1"
    /// - CIDR notation: "10.0.0.0/24"
    /// - Hostname: "example.com"
    /// - IP range: "192.168.1.1-192.168.1.254"
    ///
    /// # Example
    /// ```rust
    /// use prtip_core::Target;
    ///
    /// let t1: Target = "192.168.1.1".parse()?;
    /// let t2: Target = "10.0.0.0/24".parse()?;
    /// let t3: Target = "example.com".parse()?;
    /// let t4: Target = "192.168.1.1-192.168.1.254".parse()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(s: &str) -> Result<Self>

    /// Expand into IP addresses
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::Target;
    /// let target: Target = "192.168.1.0/30".parse()?;
    /// let ips = target.expand()?;
    /// assert_eq!(ips.len(), 4);  // 192.168.1.0, .1, .2, .3
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn expand(&self) -> Result<Vec<IpAddr>>
}
```

---

### PortRange

Port range specification (individual ports or ranges).

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRange {
    ranges: Vec<(u16, u16)>,
}

impl PortRange {
    /// Create new port range
    ///
    /// # Example
    /// ```rust
    /// use prtip_core::PortRange;
    ///
    /// let range = PortRange::new(1, 1000);
    /// assert_eq!(range.count(), 1000);
    /// ```
    pub fn new(start: u16, end: u16) -> Self

    /// Parse from string
    ///
    /// Supports:
    /// - Individual ports: "80,443,8080"
    /// - Ranges: "1-1000"
    /// - Mixed: "80,443,8000-9000"
    /// - Special: "-" or "all" for 1-65535
    ///
    /// # Example
    /// ```rust
    /// use prtip_core::PortRange;
    ///
    /// let p1: PortRange = "80,443".parse()?;
    /// let p2: PortRange = "1-1000".parse()?;
    /// let p3: PortRange = "80,443,8000-9000".parse()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(s: &str) -> Result<Self>

    /// Iterate over all ports
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::PortRange;
    /// let range: PortRange = "80,443,8000-8002".parse()?;
    /// let ports: Vec<u16> = range.iter().collect();
    /// assert_eq!(ports, vec![80, 443, 8000, 8001, 8002]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = u16>

    /// Count of ports in range
    pub fn count(&self) -> usize
}
```

---

## Result Types

### ScanReport

Complete scan report with all results.

```rust
#[derive(Debug, Clone)]
pub struct ScanReport {
    /// Scan configuration used
    pub config: ScanConfig,

    /// Scan start time
    pub start_time: SystemTime,

    /// Scan end time
    pub end_time: SystemTime,

    /// Results per host
    pub hosts: Vec<HostResult>,

    /// Scan statistics
    pub stats: ScanStats,
}

impl ScanReport {
    /// Duration of scan
    pub fn duration(&self) -> Duration {
        self.end_time.duration_since(self.start_time).unwrap()
    }

    /// Total open ports across all hosts
    pub fn total_open_ports(&self) -> usize {
        self.hosts.iter()
            .flat_map(|h| &h.ports)
            .filter(|p| p.state == PortState::Open)
            .count()
    }

    /// Export to JSON
    ///
    /// # Example
    /// ```rust
    /// # use prtip_core::ScanReport;
    /// # let report = ScanReport::default();
    /// let json = report.to_json()?;
    /// std::fs::write("scan_results.json", json)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Export to Nmap-compatible XML
    pub fn to_xml(&self) -> Result<String>

    /// Save to SQLite database
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_core::ScanReport;
    /// # let report = ScanReport::default();
    /// report.save_to_db("scans.db")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn save_to_db(&self, db_path: &str) -> Result<()>
}
```

---

### HostResult

Results for a single host.

```rust
#[derive(Debug, Clone)]
pub struct HostResult {
    /// IP address
    pub ip: IpAddr,

    /// Resolved hostname (if available)
    pub hostname: Option<String>,

    /// Host state (Up/Down/Unknown)
    pub state: HostState,

    /// Port scan results
    pub ports: Vec<PortResult>,

    /// OS fingerprint match (if OS detection enabled)
    pub os: Option<OsMatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostState {
    /// Host is up (responded to probes)
    Up,

    /// Host is down (no response)
    Down,

    /// Unable to determine (skip_discovery=true)
    Unknown,
}
```

---

### PortResult

Results for a single port.

```rust
#[derive(Debug, Clone)]
pub struct PortResult {
    /// Port number (1-65535)
    pub port: u16,

    /// Protocol (TCP/UDP)
    pub protocol: Protocol,

    /// Port state
    pub state: PortState,

    /// Detected service (if service_detection enabled)
    pub service: Option<ServiceInfo>,

    /// Raw service banner (if captured)
    pub banner: Option<String>,

    /// Response time (RTT)
    pub response_time: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    /// Port is open (accepting connections)
    Open,

    /// Port is closed (actively rejecting with RST)
    Closed,

    /// Port is filtered (firewall blocking)
    Filtered,

    /// Open or filtered (UDP scan ambiguity)
    OpenFiltered,

    /// Closed or filtered (rare, IPID idle scan)
    ClosedFiltered,

    /// Unknown state (unexpected response)
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}
```

---

## Error Types

### Error

Main error type for all ProRT-IP operations.

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid target: {0}")]
    InvalidTarget(String),

    #[error("Invalid port range: {0}")]
    InvalidPortRange(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Invalid packet: {0}")]
    InvalidPacket(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Common Error Scenarios:**

| Error | Cause | Solution |
|-------|-------|----------|
| `PermissionDenied` | Raw socket access | Run with root/Administrator privileges |
| `InvalidTarget` | Malformed IP/CIDR | Check target syntax (e.g., "192.168.1.0/24") |
| `InvalidPortRange` | Invalid port specification | Valid range: 1-65535 |
| `Timeout` | No response from target | Increase timeout or retry count |
| `Network` | Network interface issue | Check interface name, connectivity |
| `Config` | Invalid configuration | Review ScanConfig fields |
| `Plugin` | Plugin initialization failed | Check plugin compatibility |

---

## See Also

- [Technical Specifications](tech-spec-v2.md) - System requirements, protocols, packet formats
- [User Guide](../user-guide/cli-reference.md) - CLI usage examples
- [Architecture](../development/architecture.md) - System design overview
- [Plugin Development](../features/plugin-system.md) - Creating custom plugins
