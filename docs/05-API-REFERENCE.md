# ProRT-IP WarScan: API Reference

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [Core Scanner API](#core-scanner-api)
2. [Network Protocol API](#network-protocol-api)
3. [Detection Engine API](#detection-engine-api)
4. [Plugin API](#plugin-api)
5. [Configuration API](#configuration-api)
6. [Result Types](#result-types)
7. [Error Types](#error-types)

---

## Core Scanner API

### Scanner

Main entry point for executing scans.

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
    /// * `Error::PermissionDenied` - Insufficient privileges
    ///
    /// # Example
    /// ```
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

#### Methods

```rust
impl Scanner {
    /// Execute the scan asynchronously
    ///
    /// # Returns
    /// * `Result<ScanReport>` - Complete scan report
    ///
    /// # Example
    /// ```
    /// # use prtip_core::{Scanner, ScanConfig};
    /// # let scanner = Scanner::new(ScanConfig::default())?;
    /// let report = scanner.execute().await?;
    /// println!("Scanned {} hosts", report.hosts_scanned);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn execute(&self) -> Result<ScanReport>

    /// Execute with progress callback
    ///
    /// # Arguments
    /// * `callback` - Function called with progress updates
    ///
    /// # Example
    /// ```
    /// # use prtip_core::{Scanner, ScanConfig, ScanProgress};
    /// # let scanner = Scanner::new(ScanConfig::default())?;
    /// let report = scanner.execute_with_progress(|progress| {
    ///     println!("Progress: {:.1}%", progress.percentage());
    /// }).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn execute_with_progress<F>(&self, callback: F) -> Result<ScanReport>
    where
        F: Fn(ScanProgress) + Send + 'static

    /// Pause the scan
    ///
    /// # Example
    /// ```
    /// # use prtip_core::Scanner;
    /// # let scanner = Scanner::new(Default::default())?;
    /// scanner.pause()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn pause(&self) -> Result<()>

    /// Resume a paused scan
    pub fn resume(&self) -> Result<()>

    /// Stop the scan gracefully
    pub fn stop(&self) -> Result<()>
}
```

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

    /// Skip host discovery
    pub skip_discovery: bool,

    /// Enable service version detection
    pub service_detection: bool,

    /// Service detection intensity (0-9)
    pub service_intensity: u8,

    /// Enable OS fingerprinting
    pub os_detection: bool,

    /// Timing template (0-5)
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
            max_rate: Some(100_000),
            min_rate: None,
            max_retries: 2,
            max_duration: None,
            output: OutputConfig::default(),
        }
    }
}
```

### ScanType

Types of scans supported.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    /// TCP SYN scan (half-open)
    Syn,

    /// TCP Connect scan (full connection)
    Connect,

    /// UDP scan
    Udp,

    /// TCP FIN scan
    Fin,

    /// TCP NULL scan (no flags)
    Null,

    /// TCP Xmas scan (FIN+PSH+URG)
    Xmas,

    /// TCP ACK scan (firewall detection)
    Ack,

    /// TCP Window scan
    Window,

    /// Idle scan (zombie)
    Idle {
        /// Zombie host IP
        zombie: IpAddr,
    },
}
```

### TimingTemplate

Predefined timing configurations.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingTemplate {
    /// Paranoid (T0): 5-minute delays
    Paranoid,

    /// Sneaky (T1): 15-second delays
    Sneaky,

    /// Polite (T2): 0.4-second delays
    Polite,

    /// Normal (T3): Default balanced
    Normal,

    /// Aggressive (T4): Fast networks
    Aggressive,

    /// Insane (T5): Maximum speed
    Insane,
}

impl TimingTemplate {
    /// Get timing parameters
    pub fn params(&self) -> TimingParams {
        match self {
            TimingTemplate::Paranoid => TimingParams {
                initial_timeout: Duration::from_secs(300),
                max_timeout: Duration::from_secs(300),
                max_retries: 5,
                scan_delay: Some(Duration::from_secs(300)),
            },
            // ... other templates
        }
    }
}
```

---

## Network Protocol API

### TcpPacketBuilder

Builder for constructing TCP packets.

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
    /// # Arguments
    /// * `ip` - Source IP address
    /// * `port` - Source port
    ///
    /// # Example
    /// ```
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

    /// Set sequence number
    pub fn sequence(self, seq: u32) -> Self

    /// Set acknowledgment number
    pub fn acknowledgment(self, ack: u32) -> Self

    /// Set TCP flags
    ///
    /// # Example
    /// ```
    /// use prtip_net::{TcpPacketBuilder, TcpFlags};
    ///
    /// let packet = TcpPacketBuilder::new()
    ///     .flags(TcpFlags::SYN)
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn flags(self, flags: TcpFlags) -> Self

    /// Set window size
    pub fn window_size(self, window: u16) -> Self

    /// Add TCP option
    ///
    /// # Example
    /// ```
    /// use prtip_net::{TcpPacketBuilder, TcpOption};
    ///
    /// let packet = TcpPacketBuilder::new()
    ///     .tcp_option(TcpOption::Mss(1460))
    ///     .tcp_option(TcpOption::WindowScale(7))
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn tcp_option(self, option: TcpOption) -> Self

    /// Build the packet
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - Complete packet bytes or error
    ///
    /// # Errors
    /// * `Error::InvalidAddress` - Source or destination not set
    /// * `Error::PacketTooLarge` - Options exceed maximum size
    pub fn build(self) -> Result<Vec<u8>>
}
```

### TcpFlags

TCP flags.

```rust
bitflags::bitflags! {
    pub struct TcpFlags: u8 {
        const FIN = 0b00000001;
        const SYN = 0b00000010;
        const RST = 0b00000100;
        const PSH = 0b00001000;
        const ACK = 0b00010000;
        const URG = 0b00100000;
        const ECE = 0b01000000;
        const CWR = 0b10000000;
    }
}
```

### TcpOption

TCP options.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TcpOption {
    /// Maximum Segment Size
    Mss(u16),

    /// Window Scale factor (0-14)
    WindowScale(u8),

    /// SACK Permitted
    SackPermitted,

    /// Timestamp
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
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize)>
}
```

### PacketCapture

Packet capture interface.

```rust
pub struct PacketCapture { /* private fields */ }

impl PacketCapture {
    /// Create new packet capture on interface
    ///
    /// # Arguments
    /// * `interface` - Network interface name
    ///
    /// # Example
    /// ```no_run
    /// use prtip_net::PacketCapture;
    ///
    /// let mut capture = PacketCapture::new("eth0")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(interface: &str) -> Result<Self>

    /// Set BPF filter
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_net::PacketCapture;
    /// # let mut capture = PacketCapture::new("eth0")?;
    /// capture.set_filter("tcp and dst port 80")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_filter(&mut self, filter: &str) -> Result<()>

    /// Get next packet (blocking)
    pub fn next_packet(&mut self) -> Result<Option<Vec<u8>>>

    /// Get next packet asynchronously
    pub async fn recv_async(&mut self) -> Result<Vec<u8>>
}
```

---

## Detection Engine API

### ServiceDetector

Service version detection.

```rust
pub struct ServiceDetector { /* private fields */ }

impl ServiceDetector {
    /// Create new service detector
    ///
    /// # Arguments
    /// * `intensity` - Detection intensity (0-9)
    ///
    /// # Example
    /// ```
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
    /// detector.load_probes("probes/service-probes.txt")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_probes(&mut self, path: &str) -> Result<()>

    /// Detect service on target port
    ///
    /// # Returns
    /// * `Option<ServiceInfo>` - Detected service or None
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_detect::ServiceDetector;
    /// # use std::net::SocketAddr;
    /// # let detector = ServiceDetector::new(7);
    /// let target: SocketAddr = "192.168.1.1:80".parse()?;
    /// if let Some(service) = detector.detect(target).await? {
    ///     println!("Found: {} {}", service.name, service.version.unwrap_or_default());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn detect(&self, target: SocketAddr) -> Result<Option<ServiceInfo>>
}
```

### ServiceInfo

Detected service information.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    /// Service name (e.g., "http", "ssh")
    pub name: String,

    /// Product name (e.g., "nginx", "OpenSSH")
    pub product: Option<String>,

    /// Version string
    pub version: Option<String>,

    /// Extra info
    pub extra_info: Option<String>,

    /// CPE identifier
    pub cpe: Option<String>,

    /// OS hint from service
    pub os_hint: Option<String>,
}
```

### OsDetector

OS fingerprinting engine.

```rust
pub struct OsDetector { /* private fields */ }

impl OsDetector {
    /// Create new OS detector
    pub fn new() -> Self

    /// Load fingerprint database
    pub fn load_fingerprints(&mut self, path: &str) -> Result<()>

    /// Detect OS of target
    ///
    /// # Arguments
    /// * `target` - Target IP address
    /// * `open_port` - Known open TCP port
    /// * `closed_port` - Known closed TCP port
    ///
    /// # Returns
    /// * `Vec<OsMatch>` - Possible OS matches sorted by confidence
    ///
    /// # Example
    /// ```no_run
    /// # use prtip_detect::OsDetector;
    /// # use std::net::Ipv4Addr;
    /// # let detector = OsDetector::new();
    /// let target = Ipv4Addr::new(192, 168, 1, 1);
    /// let matches = detector.detect(target, 80, 12345).await?;
    ///
    /// if let Some(best) = matches.first() {
    ///     println!("OS: {} ({}% confidence)", best.name, best.accuracy);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn detect(
        &self,
        target: Ipv4Addr,
        open_port: u16,
        closed_port: u16
    ) -> Result<Vec<OsMatch>>
}
```

### OsMatch

OS detection match.

```rust
#[derive(Debug, Clone)]
pub struct OsMatch {
    /// OS name
    pub name: String,

    /// OS class
    pub class: OsClass,

    /// Match accuracy (0-100)
    pub accuracy: u8,

    /// CPE identifiers
    pub cpe: Vec<String>,

    /// Additional info
    pub info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsClass {
    pub vendor: String,
    pub os_family: String,
    pub os_generation: Option<String>,
    pub device_type: String,
}
```

---

## Plugin API

### Plugin Trait

Interface for scanner plugins.

```rust
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Initialize plugin with configuration
    ///
    /// # Arguments
    /// * `config` - Plugin configuration
    fn init(&mut self, config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    /// Called when scan starts
    fn on_scan_start(&mut self, _scan_info: &ScanInfo) -> Result<()> {
        Ok(())
    }

    /// Called for each discovered host
    fn on_host_discovered(&mut self, _host: &HostInfo) -> Result<()> {
        Ok(())
    }

    /// Called for each discovered port
    fn on_port_discovered(&mut self, _result: &ScanResult) -> Result<()> {
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

### PluginManager

Manages plugin lifecycle.

```rust
pub struct PluginManager { /* private fields */ }

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self

    /// Register a plugin
    ///
    /// # Example
    /// ```
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

    /// Load plugin from file
    pub fn load_from_file(&mut self, path: &str) -> Result<()>

    /// Notify all plugins of event
    pub fn notify_port_discovered(&mut self, result: &ScanResult) -> Result<()>
}
```

---

## Configuration API

### Target

Target specification.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Single IP address
    Ip(IpAddr),

    /// Network in CIDR notation
    Network(IpNetwork),

    /// Hostname (requires DNS resolution)
    Hostname(String),

    /// IP range
    Range {
        start: IpAddr,
        end: IpAddr,
    },
}

impl Target {
    /// Parse target from string
    ///
    /// # Example
    /// ```
    /// use prtip_core::Target;
    ///
    /// let t1: Target = "192.168.1.1".parse()?;
    /// let t2: Target = "10.0.0.0/24".parse()?;
    /// let t3: Target = "example.com".parse()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(s: &str) -> Result<Self>

    /// Expand into IP addresses
    pub fn expand(&self) -> Result<Vec<IpAddr>>
}
```

### PortRange

Port range specification.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRange {
    ranges: Vec<(u16, u16)>,
}

impl PortRange {
    /// Create new port range
    pub fn new(start: u16, end: u16) -> Self

    /// Parse from string
    ///
    /// # Example
    /// ```
    /// use prtip_core::PortRange;
    ///
    /// let p1: PortRange = "80,443".parse()?;
    /// let p2: PortRange = "1-1000".parse()?;
    /// let p3: PortRange = "80,443,8000-9000".parse()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(s: &str) -> Result<Self>

    /// Iterate over all ports
    pub fn iter(&self) -> impl Iterator<Item = u16>

    /// Count of ports
    pub fn count(&self) -> usize
}
```

---

## Result Types

### ScanReport

Complete scan report.

```rust
#[derive(Debug, Clone)]
pub struct ScanReport {
    /// Scan configuration
    pub config: ScanConfig,

    /// Scan start time
    pub start_time: SystemTime,

    /// Scan end time
    pub end_time: SystemTime,

    /// Results per host
    pub hosts: Vec<HostResult>,

    /// Statistics
    pub stats: ScanStats,
}

impl ScanReport {
    /// Duration of scan
    pub fn duration(&self) -> Duration {
        self.end_time.duration_since(self.start_time).unwrap()
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Export to XML
    pub fn to_xml(&self) -> Result<String>

    /// Save to database
    pub fn save_to_db(&self, db_path: &str) -> Result<()>
}
```

### HostResult

Results for a single host.

```rust
#[derive(Debug, Clone)]
pub struct HostResult {
    pub ip: IpAddr,
    pub hostname: Option<String>,
    pub state: HostState,
    pub ports: Vec<PortResult>,
    pub os: Option<OsMatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostState {
    Up,
    Down,
    Unknown,
}
```

### PortResult

Results for a single port.

```rust
#[derive(Debug, Clone)]
pub struct PortResult {
    pub port: u16,
    pub protocol: Protocol,
    pub state: PortState,
    pub service: Option<ServiceInfo>,
    pub banner: Option<String>,
    pub response_time: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Closed,
    Filtered,
    OpenFiltered,
    ClosedFiltered,
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

Main error type.

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

    #[error("Timeout")]
    Timeout,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Next Steps

- Review [Architecture](00-ARCHITECTURE.md) for system overview
- Consult [Implementation Guide](04-IMPLEMENTATION-GUIDE.md) for usage examples
- See [Testing Strategy](06-TESTING.md) for testing APIs
