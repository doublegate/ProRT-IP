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

## IPv6 Implementation

### Overview

ProRT-IP provides full IPv6 support across all scanning modes (Sprint 5.1). All scanners use runtime dispatch to handle both IPv4 and IPv6 packets transparently.

### IPv6 Packet Building

**File:** `crates/prtip-net/src/ipv6_packet.rs`

```rust
use pnet::packet::ipv6::{MutableIpv6Packet, Ipv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use std::net::Ipv6Addr;

pub struct Ipv6PacketBuilder {
    src: Ipv6Addr,
    dst: Ipv6Addr,
    next_header: u8,
    hop_limit: u8,
    payload: Vec<u8>,
}

impl Ipv6PacketBuilder {
    pub fn new() -> Self {
        Self {
            src: Ipv6Addr::UNSPECIFIED,
            dst: Ipv6Addr::UNSPECIFIED,
            next_header: IpNextHeaderProtocols::Tcp.0,
            hop_limit: 64,
            payload: Vec::new(),
        }
    }

    pub fn source(mut self, addr: Ipv6Addr) -> Self {
        self.src = addr;
        self
    }

    pub fn destination(mut self, addr: Ipv6Addr) -> Self {
        self.dst = addr;
        self
    }

    pub fn next_header(mut self, protocol: u8) -> Self {
        self.next_header = protocol;
        self
    }

    pub fn payload(mut self, data: Vec<u8>) -> Self {
        self.payload = data;
        self
    }

    pub fn build(self) -> Result<Vec<u8>> {
        let total_len = 40 + self.payload.len(); // IPv6 header is always 40 bytes
        let mut buffer = vec![0u8; total_len];

        {
            let mut packet = MutableIpv6Packet::new(&mut buffer)
                .ok_or(Error::PacketTooSmall)?;

            packet.set_version(6);
            packet.set_traffic_class(0);
            packet.set_flow_label(0);
            packet.set_payload_length(self.payload.len() as u16);
            packet.set_next_header(self.next_header);
            packet.set_hop_limit(self.hop_limit);
            packet.set_source(self.src);
            packet.set_destination(self.dst);
            packet.set_payload(&self.payload);
        }

        Ok(buffer)
    }
}
```

### TCP Over IPv6

**File:** `crates/prtip-net/src/packet/tcp.rs` (IPv6 additions)

```rust
use std::net::{IpAddr, Ipv6Addr};

impl TcpPacketBuilder {
    /// Build TCP packet for IPv6
    pub fn build_ipv6(self) -> Result<Vec<u8>> {
        let (src_ipv6, dst_ipv6) = match (self.src_ip, self.dst_ip) {
            (IpAddr::V6(src), IpAddr::V6(dst)) => (src, dst),
            _ => return Err(Error::InvalidAddressType),
        };

        // Build TCP segment
        let tcp_segment = self.build_tcp_segment()?;

        // Calculate IPv6 TCP checksum
        let checksum = calculate_tcp_checksum_ipv6(
            src_ipv6,
            dst_ipv6,
            &tcp_segment,
        );

        // Update checksum in TCP segment
        let mut tcp_segment = tcp_segment;
        tcp_segment[16] = (checksum >> 8) as u8;
        tcp_segment[17] = checksum as u8;

        // Build IPv6 packet
        Ipv6PacketBuilder::new()
            .source(src_ipv6)
            .destination(dst_ipv6)
            .next_header(IpNextHeaderProtocols::Tcp.0)
            .payload(tcp_segment)
            .build()
    }
}

/// Calculate TCP checksum for IPv6
fn calculate_tcp_checksum_ipv6(
    src: Ipv6Addr,
    dst: Ipv6Addr,
    tcp_segment: &[u8],
) -> u16 {
    let mut sum: u32 = 0;

    // Add source address (128 bits = 16 bytes = 8 words)
    for chunk in src.octets().chunks(2) {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }

    // Add destination address (128 bits = 16 bytes = 8 words)
    for chunk in dst.octets().chunks(2) {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }

    // Add TCP length (32 bits, split into two 16-bit words)
    let tcp_len = tcp_segment.len() as u32;
    sum += (tcp_len >> 16) & 0xFFFF;
    sum += tcp_len & 0xFFFF;

    // Add next header (TCP = 6, padded to 16 bits)
    sum += 6;

    // Add TCP segment (16-bit words)
    for chunk in tcp_segment.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], 0])
        };
        sum += word as u32;
    }

    // Fold 32-bit sum to 16 bits
    while (sum >> 16) > 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16
}
```

### ICMPv6 Implementation

**File:** `crates/prtip-net/src/icmpv6.rs`

```rust
use pnet::packet::icmpv6::{Icmpv6Types, MutableIcmpv6Packet};
use std::net::Ipv6Addr;

/// ICMPv6 Echo Request builder
pub struct Icmpv6EchoBuilder {
    identifier: u16,
    sequence: u16,
    payload: Vec<u8>,
}

impl Icmpv6EchoBuilder {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            identifier: rng.gen(),
            sequence: 0,
            payload: Vec::new(),
        }
    }

    pub fn identifier(mut self, id: u16) -> Self {
        self.identifier = id;
        self
    }

    pub fn sequence(mut self, seq: u16) -> Self {
        self.sequence = seq;
        self
    }

    pub fn build(self) -> Result<Vec<u8>> {
        let packet_len = 8 + self.payload.len(); // ICMPv6 header (8) + payload
        let mut buffer = vec![0u8; packet_len];

        {
            let mut packet = MutableIcmpv6Packet::new(&mut buffer)
                .ok_or(Error::PacketTooSmall)?;

            packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
            packet.set_icmpv6_code(0);

            // Set identifier and sequence in payload
            buffer[4..6].copy_from_slice(&self.identifier.to_be_bytes());
            buffer[6..8].copy_from_slice(&self.sequence.to_be_bytes());

            if !self.payload.is_empty() {
                buffer[8..].copy_from_slice(&self.payload);
            }

            // Calculate checksum
            let checksum = calculate_icmpv6_checksum(&buffer);
            packet.set_checksum(checksum);
        }

        Ok(buffer)
    }
}

/// NDP Neighbor Solicitation builder
pub struct NdpSolicitationBuilder {
    target: Ipv6Addr,
    src_link_layer: Option<[u8; 6]>,
}

impl NdpSolicitationBuilder {
    pub fn new(target: Ipv6Addr) -> Self {
        Self {
            target,
            src_link_layer: None,
        }
    }

    pub fn source_link_layer(mut self, mac: [u8; 6]) -> Self {
        self.src_link_layer = Some(mac);
        self
    }

    /// Calculate solicited-node multicast address
    pub fn solicited_node_multicast(&self) -> Ipv6Addr {
        let target_octets = self.target.octets();

        // ff02::1:ffXX:XXXX where XX:XXXX are last 24 bits of target
        Ipv6Addr::new(
            0xff02, 0, 0, 0,
            0, 1,
            0xff00 | (target_octets[13] as u16),
            ((target_octets[14] as u16) << 8) | (target_octets[15] as u16),
        )
    }

    pub fn build(self) -> Result<Vec<u8>> {
        // NS message: Type(1) + Code(1) + Checksum(2) + Reserved(4) + Target(16) + [Options]
        let option_len = if self.src_link_layer.is_some() { 8 } else { 0 };
        let packet_len = 24 + option_len;
        let mut buffer = vec![0u8; packet_len];

        // ICMPv6 Type 135 (Neighbor Solicitation)
        buffer[0] = 135;
        buffer[1] = 0; // Code

        // Reserved (4 bytes, zero)
        // buffer[4..8] already zero

        // Target address (16 bytes)
        buffer[8..24].copy_from_slice(&self.target.octets());

        // Source Link-Layer Address option (Type 1, Length 1)
        if let Some(mac) = self.src_link_layer {
            buffer[24] = 1; // Type: Source Link-Layer Address
            buffer[25] = 1; // Length: 1 (in units of 8 bytes)
            buffer[26..32].copy_from_slice(&mac);
        }

        // Calculate checksum (requires pseudo-header, done by caller)

        Ok(buffer)
    }
}
```

### Dual-Stack Scanner Integration

**File:** `crates/prtip-scanner/src/tcp_connect.rs` (example)

```rust
use std::net::{IpAddr, SocketAddr};

pub async fn tcp_connect_scan(
    target: SocketAddr,
    port: u16,
    timeout: Duration,
) -> Result<PortState> {
    // Automatic IPv4/IPv6 handling via SocketAddr
    let addr = SocketAddr::new(target.ip(), port);

    match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
        Ok(Ok(_stream)) => Ok(PortState::Open),
        Ok(Err(e)) if e.kind() == io::ErrorKind::ConnectionRefused => {
            Ok(PortState::Closed)
        }
        Ok(Err(_)) | Err(_) => Ok(PortState::Filtered),
    }
}
```

**File:** `crates/prtip-scanner/src/syn_scanner.rs` (example)

```rust
pub async fn send_syn_packet(
    socket: &RawSocket,
    target: SocketAddr,
) -> Result<()> {
    match target.ip() {
        IpAddr::V4(ipv4) => {
            let packet = TcpPacketBuilder::new()
                .source(get_local_ipv4()?, random_port())
                .destination(ipv4, target.port())
                .flags(TcpFlags::SYN)
                .build_ipv4()?;
            socket.send(&packet).await?;
        }
        IpAddr::V6(ipv6) => {
            let packet = TcpPacketBuilder::new()
                .source_v6(get_local_ipv6()?, random_port())
                .destination_v6(ipv6, target.port())
                .flags(TcpFlags::SYN)
                .build_ipv6()?;
            socket.send(&packet).await?;
        }
    }
    Ok(())
}
```

### Best Practices for IPv6 Implementation

1. **Use `IpAddr` enum for protocol dispatch:**
   ```rust
   match addr {
       IpAddr::V4(ipv4) => handle_ipv4(ipv4),
       IpAddr::V6(ipv6) => handle_ipv6(ipv6),
   }
   ```

2. **Always calculate checksums correctly:**
   - IPv6 TCP/UDP checksums are mandatory (unlike IPv4 UDP)
   - Include pseudo-header with full 128-bit addresses
   - No IP header checksum in IPv6 (delegated to link layer)

3. **Handle ICMPv6 responses:**
   - Type 1, Code 4: Port Unreachable (UDP closed)
   - Type 1, Code 1: Administratively Prohibited (filtered)
   - Type 129: Echo Reply (host alive)
   - Type 136: Neighbor Advertisement (NDP response)

4. **Test on multiple platforms:**
   - Linux: Use AF_INET6 raw sockets
   - Windows: Requires Npcap with IPv6 support
   - macOS: BPF device for raw packet access
   - FreeBSD: Native IPv6 raw socket support

For comprehensive IPv6 usage examples and protocol details, see [23-IPv6-GUIDE.md](23-IPv6-GUIDE.md).

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
