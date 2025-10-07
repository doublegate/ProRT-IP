# ProRT-IP WarScan: Implementation Guide

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Core Module Implementation](#core-module-implementation)
4. [Networking Module Implementation](#networking-module-implementation)
5. [Detection Module Implementation](#detection-module-implementation)
6. [CLI Implementation](#cli-implementation)
7. [Error Handling Patterns](#error-handling-patterns)
8. [Best Practices](#best-practices)

---

## Getting Started

### Initial Project Setup

```bash
# Create workspace structure
cargo new --lib prtip-warscan
cd prtip-warscan

# Create workspace layout
mkdir -p crates/{core,net,detect,plugins,cli}

# Initialize each crate
cargo new --lib crates/core
cargo new --lib crates/net
cargo new --lib crates/detect
cargo new --lib crates/plugins
cargo new --bin crates/cli
```

### Workspace Configuration

**Root `Cargo.toml`:**

```toml
[workspace]
members = [
    "crates/core",
    "crates/net",
    "crates/detect",
    "crates/plugins",
    "crates/cli",
]

resolver = "2"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# Networking
pnet = "0.34"
pnet_datalink = "0.34"
pnet_packet = "0.34"
socket2 = "0.5"
pcap = "1.1"
etherparse = "0.14"

# Concurrency
crossbeam = "0.8"
parking_lot = "0.12"
rayon = "1.8"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# CLI
clap = { version = "4.4", features = ["derive", "cargo"] }

# Database
rusqlite = { version = "0.30", features = ["bundled"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Utilities
ipnetwork = "0.20"
rand = "0.8"
chrono = "0.4"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true
```

---

## Project Structure

### Directory Layout

```
prtip-warscan/
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock                    # Dependency lock
├── README.md                     # Project README
├── LICENSE                       # GPLv3 license
├── CHANGELOG.md                  # Version history
│
├── crates/
│   ├── core/                     # Core scanning engine
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── scanner.rs        # Main scanner orchestrator
│   │   │   ├── scheduler.rs      # Target scheduling
│   │   │   ├── rate_limiter.rs   # Rate control
│   │   │   ├── result.rs         # Result aggregation
│   │   │   └── config.rs         # Configuration
│   │   └── tests/
│   │
│   ├── net/                      # Network protocol layer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── packet/           # Packet construction
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tcp.rs
│   │   │   │   ├── udp.rs
│   │   │   │   └── icmp.rs
│   │   │   ├── capture.rs        # Packet capture
│   │   │   ├── checksum.rs       # Checksum calculation
│   │   │   └── rawsock.rs        # Raw socket abstraction
│   │   └── tests/
│   │
│   ├── detect/                   # Detection engines
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── os_fingerprint.rs # OS detection
│   │   │   ├── service.rs        # Service detection
│   │   │   ├── banner.rs         # Banner grabbing
│   │   │   └── probes.rs         # Probe database
│   │   └── tests/
│   │
│   ├── plugins/                  # Plugin system
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── api.rs            # Plugin API
│   │   │   ├── lua.rs            # Lua integration
│   │   │   └── loader.rs         # Plugin loading
│   │   └── examples/
│   │       └── http_enum.lua     # Example plugin
│   │
│   └── cli/                      # Command-line interface
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── args.rs           # Argument parsing
│       │   ├── output.rs         # Output formatters
│       │   └── ui.rs             # TUI (future)
│       └── tests/
│
├── tests/                        # Integration tests
│   ├── integration_syn_scan.rs
│   ├── integration_service_detect.rs
│   └── fixtures/
│       ├── docker-compose.yml
│       └── pcaps/
│
├── benches/                      # Performance benchmarks
│   ├── packet_crafting.rs
│   └── scan_throughput.rs
│
├── docs/                         # Documentation
│   └── *.md
│
└── scripts/                      # Utility scripts
    ├── build.sh
    ├── test.sh
    └── benchmark.sh
```

---

## Core Module Implementation

### Scanner Orchestrator

**File:** `crates/core/src/scanner.rs`

```rust
use tokio::runtime::Runtime;
use crossbeam::queue::SegQueue;
use std::sync::Arc;

pub struct Scanner {
    config: ScanConfig,
    runtime: Runtime,
    target_scheduler: TargetScheduler,
    rate_limiter: RateLimiter,
    result_aggregator: ResultAggregator,
}

impl Scanner {
    pub fn new(config: ScanConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Create async runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(num_cpus::get_physical())
            .thread_name("prtip-worker")
            .enable_all()
            .build()?;

        // Initialize components
        let target_scheduler = TargetScheduler::new(&config.targets)?;
        let rate_limiter = RateLimiter::new(config.max_rate);
        let result_aggregator = ResultAggregator::new(config.output.clone());

        Ok(Self {
            config,
            runtime,
            target_scheduler,
            rate_limiter,
            result_aggregator,
        })
    }

    pub async fn execute(&self) -> Result<ScanReport> {
        tracing::info!("Starting scan with config: {:?}", self.config);

        // Phase 1: Host discovery (if enabled)
        let live_hosts = if self.config.skip_discovery {
            self.target_scheduler.all_targets()
        } else {
            self.discover_hosts().await?
        };

        // Phase 2: Port scanning
        let open_ports = self.scan_ports(&live_hosts).await?;

        // Phase 3: Service detection (if enabled)
        let results = if self.config.service_detection {
            self.detect_services(&open_ports).await?
        } else {
            open_ports
        };

        // Phase 4: OS fingerprinting (if enabled)
        let final_results = if self.config.os_detection {
            self.detect_os(&results).await?
        } else {
            results
        };

        // Generate report
        let report = self.result_aggregator.generate_report(final_results)?;

        tracing::info!("Scan complete: {} hosts, {} ports",
            report.hosts_scanned, report.ports_open);

        Ok(report)
    }

    async fn scan_ports(&self, targets: &[Target]) -> Result<Vec<ScanResult>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10000);
        let task_queue = Arc::new(SegQueue::new());

        // Populate task queue
        for target in targets {
            for port in self.config.ports.iter() {
                task_queue.push(ScanTask {
                    target: target.clone(),
                    port,
                    scan_type: self.config.scan_type,
                });
            }
        }

        // Spawn worker pool
        let worker_count = num_cpus::get_physical();
        let mut workers = Vec::new();

        for _ in 0..worker_count {
            let queue = Arc::clone(&task_queue);
            let tx = tx.clone();
            let rate_limiter = self.rate_limiter.clone();

            let worker = tokio::spawn(async move {
                while let Some(task) = queue.pop() {
                    // Wait for rate limiter
                    rate_limiter.wait().await;

                    // Execute scan
                    match scan_port(&task).await {
                        Ok(result) => {
                            tx.send(result).await.ok();
                        }
                        Err(e) => {
                            tracing::warn!("Scan error: {}", e);
                        }
                    }
                }
            });

            workers.push(worker);
        }

        drop(tx); // Close sender so rx knows when to stop

        // Collect results
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        // Wait for workers
        for worker in workers {
            worker.await?;
        }

        Ok(results)
    }
}

async fn scan_port(task: &ScanTask) -> Result<ScanResult> {
    match task.scan_type {
        ScanType::Syn => syn_scan(task).await,
        ScanType::Connect => connect_scan(task).await,
        ScanType::Udp => udp_scan(task).await,
        // ... other scan types
    }
}
```

### Rate Limiter

**File:** `crates/core/src/rate_limiter.rs`

```rust
use governor::{Quota, RateLimiter as GovRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovRateLimiter<governor::clock::DefaultClock>>,
}

impl RateLimiter {
    pub fn new(packets_per_second: u32) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(packets_per_second).unwrap()
        );
        let limiter = Arc::new(GovRateLimiter::direct(quota));

        Self { limiter }
    }

    pub async fn wait(&self) {
        self.limiter.until_ready().await;
    }

    pub fn try_acquire(&self) -> bool {
        self.limiter.check().is_ok()
    }
}
```

---

## Networking Module Implementation

### TCP Packet Builder

**File:** `crates/net/src/packet/tcp.rs`

```rust
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::ip::IpNextHeaderProtocols;
use std::net::Ipv4Addr;

pub struct TcpPacketBuilder {
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: TcpFlags,
    window: u16,
    options: Vec<TcpOption>,
}

impl TcpPacketBuilder {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            src_ip: Ipv4Addr::UNSPECIFIED,
            dst_ip: Ipv4Addr::UNSPECIFIED,
            src_port: rng.gen_range(1024..65535),
            dst_port: 0,
            seq: rng.gen(),
            ack: 0,
            flags: TcpFlags::empty(),
            window: 65535,
            options: Vec::new(),
        }
    }

    pub fn source(mut self, ip: Ipv4Addr, port: u16) -> Self {
        self.src_ip = ip;
        self.src_port = port;
        self
    }

    pub fn destination(mut self, ip: Ipv4Addr, port: u16) -> Self {
        self.dst_ip = ip;
        self.dst_port = port;
        self
    }

    pub fn sequence(mut self, seq: u32) -> Self {
        self.seq = seq;
        self
    }

    pub fn flags(mut self, flags: TcpFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn tcp_option(mut self, option: TcpOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn build(self) -> Result<Vec<u8>> {
        // Calculate sizes
        let options_len = self.calculate_options_length();
        let tcp_header_len = 20 + options_len;
        let ip_total_len = 20 + tcp_header_len;

        // Build complete packet
        let mut buffer = vec![0u8; ip_total_len];

        // Build IPv4 header
        {
            use pnet::packet::ipv4::MutableIpv4Packet;

            let mut ip_packet = MutableIpv4Packet::new(&mut buffer[..20])
                .ok_or(Error::PacketTooSmall)?;

            ip_packet.set_version(4);
            ip_packet.set_header_length(5);
            ip_packet.set_total_length(ip_total_len as u16);
            ip_packet.set_identification(rand::random());
            ip_packet.set_ttl(64);
            ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
            ip_packet.set_source(self.src_ip);
            ip_packet.set_destination(self.dst_ip);

            // Calculate IP checksum
            let checksum = pnet::packet::ipv4::checksum(&ip_packet.to_immutable());
            ip_packet.set_checksum(checksum);
        }

        // Build TCP header
        {
            let mut tcp_packet = MutableTcpPacket::new(&mut buffer[20..])
                .ok_or(Error::PacketTooSmall)?;

            tcp_packet.set_source(self.src_port);
            tcp_packet.set_destination(self.dst_port);
            tcp_packet.set_sequence(self.seq);
            tcp_packet.set_acknowledgement(self.ack);
            tcp_packet.set_data_offset((tcp_header_len / 4) as u8);
            tcp_packet.set_flags(self.flags.bits());
            tcp_packet.set_window(self.window);

            // Set options
            if !self.options.is_empty() {
                let options_bytes = self.serialize_options();
                tcp_packet.set_options(&options_bytes);
            }

            // Calculate TCP checksum
            let checksum = pnet::packet::tcp::ipv4_checksum(
                &tcp_packet.to_immutable(),
                &self.src_ip,
                &self.dst_ip
            );
            tcp_packet.set_checksum(checksum);
        }

        Ok(buffer)
    }

    fn calculate_options_length(&self) -> usize {
        let mut len = 0;
        for opt in &self.options {
            len += opt.length();
        }
        // Pad to 4-byte boundary
        (len + 3) & !3
    }

    fn serialize_options(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for opt in &self.options {
            bytes.extend_from_slice(&opt.to_bytes());
        }
        // Pad with NOPs
        while bytes.len() % 4 != 0 {
            bytes.push(1); // NOP
        }
        bytes
    }
}

#[derive(Debug, Clone)]
pub enum TcpOption {
    Mss(u16),
    WindowScale(u8),
    SackPermitted,
    Timestamp { tsval: u32, tsecr: u32 },
    Nop,
}

impl TcpOption {
    pub fn length(&self) -> usize {
        match self {
            TcpOption::Nop => 1,
            TcpOption::Mss(_) => 4,
            TcpOption::WindowScale(_) => 3,
            TcpOption::SackPermitted => 2,
            TcpOption::Timestamp { .. } => 10,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            TcpOption::Nop => vec![1],
            TcpOption::Mss(mss) => {
                vec![2, 4, (mss >> 8) as u8, *mss as u8]
            }
            TcpOption::WindowScale(scale) => vec![3, 3, *scale],
            TcpOption::SackPermitted => vec![4, 2],
            TcpOption::Timestamp { tsval, tsecr } => {
                let mut bytes = vec![8, 10];
                bytes.extend_from_slice(&tsval.to_be_bytes());
                bytes.extend_from_slice(&tsecr.to_be_bytes());
                bytes
            }
        }
    }
}
```

### Packet Capture

**File:** `crates/net/src/capture.rs`

```rust
use pcap::{Capture, Device, Active};
use pnet::packet::ethernet::EthernetPacket;

pub struct PacketCapture {
    handle: Capture<Active>,
}

impl PacketCapture {
    pub fn new(interface: &str) -> Result<Self> {
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == interface)
            .ok_or(Error::InterfaceNotFound)?;

        let mut handle = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(100)
            .open()?;

        // Set BPF filter to reduce captured traffic
        handle.filter("tcp or udp or icmp", true)?;

        Ok(Self { handle })
    }

    pub fn set_filter(&mut self, filter: &str) -> Result<()> {
        self.handle.filter(filter, true)?;
        Ok(())
    }

    pub fn next_packet(&mut self) -> Result<Option<Vec<u8>>> {
        match self.handle.next_packet() {
            Ok(packet) => Ok(Some(packet.data.to_vec())),
            Err(pcap::Error::TimeoutExpired) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn recv_async(&mut self) -> Result<Vec<u8>> {
        loop {
            if let Some(packet) = self.next_packet()? {
                return Ok(packet);
            }
            tokio::task::yield_now().await;
        }
    }
}
```

---

## Detection Module Implementation

### Service Detection

**File:** `crates/detect/src/service.rs`

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

pub struct ServiceDetector {
    probes: Vec<ServiceProbe>,
    intensity: u8,
}

impl ServiceDetector {
    pub async fn detect(&self, target: SocketAddr) -> Result<Option<ServiceInfo>> {
        // Try NULL probe first (wait for banner)
        if let Some(info) = self.null_probe(target).await? {
            return Ok(Some(info));
        }

        // Try registered probes for this port
        let port = target.port();
        for probe in &self.probes {
            if !probe.ports.contains(&port) {
                continue;
            }

            if probe.rarity > self.intensity {
                continue;
            }

            if let Some(info) = self.execute_probe(target, probe).await? {
                return Ok(Some(info));
            }
        }

        Ok(None)
    }

    async fn null_probe(&self, target: SocketAddr) -> Result<Option<ServiceInfo>> {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(target)
        ).await??;

        // Wait for banner
        let mut banner = vec![0u8; 1024];
        let n = tokio::time::timeout(
            Duration::from_secs(2),
            stream.read(&mut banner)
        ).await??;

        if n > 0 {
            let banner_str = String::from_utf8_lossy(&banner[..n]);
            Ok(self.match_banner(&banner_str))
        } else {
            Ok(None)
        }
    }

    async fn execute_probe(
        &self,
        target: SocketAddr,
        probe: &ServiceProbe
    ) -> Result<Option<ServiceInfo>> {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(target)
        ).await??;

        // Send probe
        stream.write_all(&probe.payload).await?;

        // Read response
        let mut response = vec![0u8; 4096];
        let n = tokio::time::timeout(
            Duration::from_secs(2),
            stream.read(&mut response)
        ).await??;

        if n > 0 {
            let response_str = String::from_utf8_lossy(&response[..n]);
            Ok(self.match_response(&response_str, &probe.matches))
        } else {
            Ok(None)
        }
    }

    fn match_banner(&self, banner: &str) -> Option<ServiceInfo> {
        // Simple pattern matching
        if banner.starts_with("SSH-") {
            Some(ServiceInfo {
                name: "ssh".to_string(),
                product: Some(extract_ssh_version(banner)),
                version: None,
                cpe: None,
            })
        } else if banner.starts_with("220 ") && banner.contains("FTP") {
            Some(ServiceInfo {
                name: "ftp".to_string(),
                product: Some(extract_ftp_server(banner)),
                version: None,
                cpe: None,
            })
        } else {
            None
        }
    }
}

pub struct ServiceProbe {
    name: String,
    payload: Vec<u8>,
    ports: Vec<u16>,
    rarity: u8,
    matches: Vec<ServiceMatch>,
}

pub struct ServiceMatch {
    pattern: regex::Regex,
    service: String,
    product: Option<String>,
    version: Option<String>,
}
```

---

## CLI Implementation

### Argument Parsing

**File:** `crates/cli/src/args.rs`

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "prtip")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Target specification (IP, CIDR, hostname, file)
    #[arg(value_name = "TARGETS")]
    pub targets: Vec<String>,

    /// Port specification (-p 80,443 or -p 1-1000)
    #[arg(short = 'p', long, default_value = "1-1000")]
    pub ports: String,

    /// Scan type
    #[arg(short = 's', long, value_enum, default_value = "syn")]
    pub scan_type: ScanTypeArg,

    /// Enable service version detection
    #[arg(short = 'V', long)]
    pub service_detection: bool,

    /// Enable OS detection
    #[arg(short = 'O', long)]
    pub os_detection: bool,

    /// Timing template (0-5)
    #[arg(short = 'T', long, value_parser = parse_timing)]
    pub timing: Option<u8>,

    /// Maximum packets per second
    #[arg(long)]
    pub max_rate: Option<u32>,

    /// Output format
    #[arg(short = 'o', long, value_enum)]
    pub output: Option<OutputFormat>,

    /// Output file
    #[arg(long)]
    pub output_file: Option<String>,

    /// Verbosity level
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ScanTypeArg {
    #[value(name = "S")]
    Syn,
    #[value(name = "T")]
    Connect,
    #[value(name = "U")]
    Udp,
    #[value(name = "F")]
    Fin,
    #[value(name = "N")]
    Null,
    #[value(name = "X")]
    Xmas,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Xml,
}

fn parse_timing(s: &str) -> Result<u8, String> {
    let t: u8 = s.parse().map_err(|_| "invalid timing value")?;
    if t <= 5 {
        Ok(t)
    } else {
        Err("timing must be 0-5".to_string())
    }
}
```

---

## Error Handling Patterns

### Custom Error Types

**File:** `crates/core/src/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid target specification: {0}")]
    InvalidTarget(String),

    #[error("Invalid port range: {0}")]
    InvalidPortRange(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    #[error("Packet error: {0}")]
    Packet(String),

    #[error("Timeout")]
    Timeout,

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Best Practices

### 1. Always Use Builder Pattern for Complex Types

```rust
let packet = TcpPacketBuilder::new()
    .source(local_ip, local_port)
    .destination(target_ip, target_port)
    .flags(TcpFlags::SYN)
    .tcp_option(TcpOption::Mss(1460))
    .build()?;
```

### 2. Prefer Type State Pattern for State Machines

```rust
struct Scanner<S> {
    state: PhantomData<S>,
    // ...
}

struct Configured;
struct Running;

impl Scanner<Configured> {
    fn start(self) -> Scanner<Running> {
        // Can only start if configured
    }
}
```

### 3. Use Channels for Inter-Thread Communication

```rust
let (tx, rx) = tokio::sync::mpsc::channel(10000);

// Producer
tokio::spawn(async move {
    tx.send(result).await.ok();
});

// Consumer
while let Some(result) = rx.recv().await {
    process(result);
}
```

### 4. Implement Display and Debug for Custom Types

```rust
#[derive(Debug)]
pub struct ScanResult {
    // ...
}

impl Display for ScanResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{} - {}", self.ip, self.port, self.state)
    }
}
```

---

## Next Steps

- Review [Architecture](00-ARCHITECTURE.md) for system design
- Consult [Technical Specs](02-TECHNICAL-SPECS.md) for protocol details
- See [API Reference](05-API-REFERENCE.md) for complete API
- Check [Testing Strategy](06-TESTING.md) for test guidelines

