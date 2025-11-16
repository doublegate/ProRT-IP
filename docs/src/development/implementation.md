# Implementation Guide

Comprehensive guide to ProRT-IP's implementation patterns, code organization, and best practices for contributors.

## Overview

ProRT-IP follows a **workspace-based architecture** with clear separation of concerns across multiple crates. This guide covers the practical implementation details you'll encounter when working with the codebase.

**Key Principles:**
- **Workspace Organization**: Multiple crates with well-defined responsibilities
- **Builder Pattern**: Complex types constructed via fluent APIs
- **Type State Pattern**: Compile-time state machine enforcement
- **Async-First**: All I/O operations use async/await with Tokio runtime
- **Zero-Copy**: Memory-mapped I/O and borrowed data where possible

## Workspace Structure

### Crate Layout

```
ProRT-IP/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── prtip-core/               # Shared types, errors, utilities
│   ├── prtip-network/            # Packet crafting, raw sockets
│   ├── prtip-scanner/            # Scan implementations
│   ├── prtip-detection/          # Service/OS detection
│   ├── prtip-plugins/            # Plugin system & Lua integration
│   ├── prtip-storage/            # Database storage
│   ├── prtip-tui/                # Terminal UI (ratatui)
│   └── prtip-cli/                # CLI binary
├── tests/                        # Integration tests
└── benches/                      # Performance benchmarks
```

### Crate Dependencies

**Dependency Graph:**

```
prtip-cli
    ├─> prtip-scanner
    │   ├─> prtip-network
    │   │   └─> prtip-core
    │   ├─> prtip-detection
    │   │   └─> prtip-network
    │   └─> prtip-core
    ├─> prtip-storage
    │   └─> prtip-core
    ├─> prtip-plugins
    │   └─> prtip-core
    └─> prtip-tui
        └─> prtip-core
```

**Design Rules:**
- `prtip-core` has **no internal dependencies** (foundational types only)
- `prtip-network` depends only on `prtip-core` (low-level networking)
- `prtip-scanner` orchestrates network + detection (high-level logic)
- `prtip-cli` is the only binary crate (entry point)

### Workspace Configuration

**Root `Cargo.toml`:**

```toml
[workspace]
members = [
    "crates/prtip-core",
    "crates/prtip-network",
    "crates/prtip-scanner",
    "crates/prtip-detection",
    "crates/prtip-plugins",
    "crates/prtip-storage",
    "crates/prtip-tui",
    "crates/prtip-cli",
]

resolver = "2"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# Networking
pnet = "0.34"
socket2 = "0.5"
pcap = "1.1"

# Concurrency
crossbeam = "0.8"
parking_lot = "0.12"
dashmap = "5.5"

# CLI & TUI
clap = { version = "4.4", features = ["derive"] }
ratatui = "0.29"
crossterm = "0.28"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

## Core Module (`prtip-core`)

### Purpose

Provides foundational types, error handling, and utilities shared across all crates.

**Contents:**
- `errors.rs` - Custom error types with `thiserror`
- `types.rs` - Common types (TargetSpec, PortRange, ScanConfig)
- `utils.rs` - Helper functions
- `constants.rs` - System constants

### Error Handling

**File:** `crates/prtip-core/src/errors.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrtipError {
    #[error("Invalid target specification: {0}")]
    InvalidTarget(String),

    #[error("Invalid port range: {0}")]
    InvalidPortRange(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network I/O error: {0}")]
    NetworkIo(#[from] std::io::Error),

    #[error("Packet construction error: {0}")]
    PacketError(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Detection error: {0}")]
    Detection(String),
}

pub type Result<T> = std::result::Result<T, PrtipError>;
```

**Design Pattern:**
- Use `thiserror` for declarative error definitions
- Implement `From` trait for automatic error conversion
- Provide context-rich error messages
- Avoid panics in library code (return `Result` instead)

### Common Types

**File:** `crates/prtip-core/src/types.rs`

```rust
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// Target specification (IP, CIDR, hostname)
#[derive(Debug, Clone)]
pub enum TargetSpec {
    Single(IpAddr),
    Range(IpAddr, IpAddr),
    Cidr(ipnetwork::IpNetwork),
    Hostname(String),
    File(PathBuf),
}

/// Port specification
#[derive(Debug, Clone)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    pub fn single(port: u16) -> Self {
        Self { start: port, end: port }
    }

    pub fn range(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    pub fn iter(&self) -> impl Iterator<Item = u16> {
        self.start..=self.end
    }
}

/// Scan type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    TcpSyn,
    TcpConnect,
    TcpFin,
    TcpNull,
    TcpXmas,
    TcpAck,
    Udp,
    Idle,
}

/// Port state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortState {
    Unknown,
    Filtered,
    Closed,
    Open,
}
```

## Network Module (`prtip-network`)

### Purpose

Low-level packet construction, raw socket abstraction, and packet capture.

**Contents:**
- `packet/` - Packet builders (TCP, UDP, ICMP, ICMPv6)
- `rawsock.rs` - Platform-specific raw socket abstraction
- `capture.rs` - Packet capture (libpcap wrapper)
- `checksum.rs` - Checksum calculation utilities

### TCP Packet Builder

**File:** `crates/prtip-network/src/packet/tcp.rs`

```rust
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub struct TcpPacketBuilder {
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: u8,
    window: u16,
    options: Vec<TcpOption>,
}

impl TcpPacketBuilder {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            src_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            dst_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            src_port: rng.gen_range(1024..65535),
            dst_port: 0,
            seq: rng.gen(),
            ack: 0,
            flags: 0,
            window: 65535,
            options: Vec::new(),
        }
    }

    // Fluent API methods
    pub fn source(mut self, ip: IpAddr, port: u16) -> Self {
        self.src_ip = ip;
        self.src_port = port;
        self
    }

    pub fn destination(mut self, ip: IpAddr, port: u16) -> Self {
        self.dst_ip = ip;
        self.dst_port = port;
        self
    }

    pub fn sequence(mut self, seq: u32) -> Self {
        self.seq = seq;
        self
    }

    pub fn flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    pub fn tcp_option(mut self, option: TcpOption) -> Self {
        self.options.push(option);
        self
    }

    /// Build IPv4 or IPv6 packet based on src_ip type
    pub fn build(self) -> Result<Vec<u8>> {
        match (self.src_ip, self.dst_ip) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => self.build_ipv4(src, dst),
            (IpAddr::V6(src), IpAddr::V6(dst)) => self.build_ipv6(src, dst),
            _ => Err(PrtipError::PacketError("IP version mismatch".into())),
        }
    }

    fn build_ipv4(self, src: Ipv4Addr, dst: Ipv4Addr) -> Result<Vec<u8>> {
        // Calculate packet sizes
        let options_len = self.calculate_options_length();
        let tcp_header_len = 20 + options_len;
        let total_len = 20 + tcp_header_len; // IP header + TCP header

        let mut buffer = vec![0u8; total_len];

        // Build IPv4 header (20 bytes)
        self.build_ipv4_header(&mut buffer[0..20], src, dst, tcp_header_len)?;

        // Build TCP segment
        self.build_tcp_segment(&mut buffer[20..], src, dst)?;

        Ok(buffer)
    }

    fn build_ipv6(self, src: Ipv6Addr, dst: Ipv6Addr) -> Result<Vec<u8>> {
        let options_len = self.calculate_options_length();
        let tcp_segment_len = 20 + options_len;
        let total_len = 40 + tcp_segment_len; // IPv6 header (40) + TCP

        let mut buffer = vec![0u8; total_len];

        // Build IPv6 header (40 bytes)
        self.build_ipv6_header(&mut buffer[0..40], src, dst, tcp_segment_len)?;

        // Build TCP segment with IPv6 pseudo-header checksum
        self.build_tcp_segment_ipv6(&mut buffer[40..], src, dst)?;

        Ok(buffer)
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
```

**Usage Example:**

```rust
let packet = TcpPacketBuilder::new()
    .source(local_ip, random_port())
    .destination(target_ip, target_port)
    .sequence(random_seq())
    .flags(TcpFlags::SYN)
    .tcp_option(TcpOption::Mss(1460))
    .tcp_option(TcpOption::WindowScale(7))
    .tcp_option(TcpOption::SackPermitted)
    .tcp_option(TcpOption::Timestamp {
        tsval: now_timestamp(),
        tsecr: 0,
    })
    .build()?;

raw_socket.send(&packet).await?;
```

### Raw Socket Abstraction

**File:** `crates/prtip-network/src/rawsock.rs`

```rust
use socket2::{Socket, Domain, Type, Protocol};

pub struct RawSocket {
    socket: Socket,
}

impl RawSocket {
    /// Create IPv4 raw socket
    pub fn new_ipv4() -> Result<Self> {
        let socket = Socket::new(
            Domain::IPV4,
            Type::RAW,
            Some(Protocol::TCP),
        )?;

        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;

        Ok(Self { socket })
    }

    /// Create IPv6 raw socket
    pub fn new_ipv6() -> Result<Self> {
        let socket = Socket::new(
            Domain::IPV6,
            Type::RAW,
            Some(Protocol::TCP),
        )?;

        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;

        Ok(Self { socket })
    }

    /// Send raw packet
    pub async fn send(&self, packet: &[u8]) -> Result<usize> {
        self.socket.send(packet)
            .map_err(|e| PrtipError::NetworkIo(e))
    }

    /// Receive raw packet (async wrapper)
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match self.socket.recv(buf) {
                Ok(n) => return Ok(n),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    tokio::task::yield_now().await;
                }
                Err(e) => return Err(PrtipError::NetworkIo(e)),
            }
        }
    }
}
```

### Packet Capture

**File:** `crates/prtip-network/src/capture.rs`

```rust
use pcap::{Capture, Device, Active};

pub struct PacketCapture {
    handle: Capture<Active>,
}

impl PacketCapture {
    pub fn new(interface: &str) -> Result<Self> {
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == interface)
            .ok_or(PrtipError::Config("Interface not found".into()))?;

        let handle = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(100)
            .open()?;

        Ok(Self { handle })
    }

    pub fn set_filter(&mut self, filter: &str) -> Result<()> {
        self.handle.filter(filter, true)?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        loop {
            match self.handle.next_packet() {
                Ok(packet) => return Ok(packet.data.to_vec()),
                Err(pcap::Error::TimeoutExpired) => {
                    tokio::task::yield_now().await;
                }
                Err(e) => return Err(PrtipError::NetworkIo(e.into())),
            }
        }
    }
}
```

## Scanner Module (`prtip-scanner`)

### Purpose

High-level scan orchestration, scan type implementations, and result aggregation.

**Contents:**
- `scheduler.rs` - Target scheduling and worker pool management
- `syn_scanner.rs` - TCP SYN scan implementation
- `connect_scanner.rs` - TCP Connect scan
- `udp_scanner.rs` - UDP scan
- `stealth_scanner.rs` - FIN/NULL/Xmas scans
- `idle_scanner.rs` - Idle (zombie) scan
- `result_aggregator.rs` - Result merging and deduplication

### Scanner Scheduler

**File:** `crates/prtip-scanner/src/scheduler.rs`

```rust
use tokio::sync::mpsc;
use crossbeam::queue::SegQueue;
use std::sync::Arc;

pub struct ScanScheduler {
    config: ScanConfig,
    target_queue: Arc<SegQueue<ScanTask>>,
    rate_limiter: Arc<AdaptiveRateLimiterV3>,
    result_tx: mpsc::Sender<ScanResult>,
}

impl ScanScheduler {
    pub fn new(config: ScanConfig, result_tx: mpsc::Sender<ScanResult>) -> Self {
        let target_queue = Arc::new(SegQueue::new());
        let rate_limiter = Arc::new(AdaptiveRateLimiterV3::new(config.max_rate));

        Self {
            config,
            target_queue,
            rate_limiter,
            result_tx,
        }
    }

    pub async fn execute(&mut self) -> Result<()> {
        // Phase 1: Populate task queue
        for target in &self.config.targets {
            for port in self.config.ports.iter() {
                self.target_queue.push(ScanTask {
                    target: target.clone(),
                    port,
                    scan_type: self.config.scan_type,
                });
            }
        }

        // Phase 2: Spawn worker pool
        let worker_count = num_cpus::get_physical();
        let mut workers = Vec::new();

        for worker_id in 0..worker_count {
            let queue = Arc::clone(&self.target_queue);
            let rate_limiter = Arc::clone(&self.rate_limiter);
            let result_tx = self.result_tx.clone();
            let config = self.config.clone();

            let worker = tokio::spawn(async move {
                Self::worker_loop(worker_id, queue, rate_limiter, result_tx, config).await
            });

            workers.push(worker);
        }

        // Phase 3: Wait for completion
        for worker in workers {
            worker.await??;
        }

        Ok(())
    }

    async fn worker_loop(
        worker_id: usize,
        queue: Arc<SegQueue<ScanTask>>,
        rate_limiter: Arc<AdaptiveRateLimiterV3>,
        result_tx: mpsc::Sender<ScanResult>,
        config: ScanConfig,
    ) -> Result<()> {
        while let Some(task) = queue.pop() {
            // Wait for rate limiter
            rate_limiter.wait().await;

            // Execute scan
            match Self::execute_scan(&task, &config).await {
                Ok(result) => {
                    result_tx.send(result).await.ok();
                }
                Err(e) => {
                    tracing::warn!("Worker {}: Scan error: {}", worker_id, e);
                }
            }
        }

        Ok(())
    }

    async fn execute_scan(task: &ScanTask, config: &ScanConfig) -> Result<ScanResult> {
        match task.scan_type {
            ScanType::TcpSyn => syn_scan(task, config).await,
            ScanType::TcpConnect => connect_scan(task, config).await,
            ScanType::Udp => udp_scan(task, config).await,
            ScanType::TcpFin => fin_scan(task, config).await,
            ScanType::TcpNull => null_scan(task, config).await,
            ScanType::TcpXmas => xmas_scan(task, config).await,
            ScanType::TcpAck => ack_scan(task, config).await,
            ScanType::Idle => idle_scan(task, config).await,
        }
    }
}
```

### SYN Scanner Implementation

**File:** `crates/prtip-scanner/src/syn_scanner.rs`

```rust
use prtip_network::TcpPacketBuilder;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct SynScanner {
    socket: RawSocket,
    capture: PacketCapture,
    pending: Arc<DashMap<u16, PendingPort>>,
}

struct PendingPort {
    target: IpAddr,
    port: u16,
    sent_at: Instant,
}

impl SynScanner {
    pub async fn scan_port(
        &self,
        target: IpAddr,
        port: u16,
    ) -> Result<PortState> {
        // Generate random source port
        let src_port = rand::random::<u16>() | 0x8000; // Ensure high bit set

        // Store pending state
        self.pending.insert(src_port, PendingPort {
            target,
            port,
            sent_at: Instant::now(),
        });

        // Send SYN packet
        let packet = TcpPacketBuilder::new()
            .source(get_local_ip()?, src_port)
            .destination(target, port)
            .sequence(rand::random())
            .flags(TcpFlags::SYN)
            .tcp_option(TcpOption::Mss(1460))
            .tcp_option(TcpOption::WindowScale(7))
            .tcp_option(TcpOption::SackPermitted)
            .build()?;

        self.socket.send(&packet).await?;

        // Wait for response with timeout
        tokio::time::timeout(
            Duration::from_secs(2),
            self.wait_for_response(src_port)
        ).await?
    }

    async fn wait_for_response(&self, src_port: u16) -> Result<PortState> {
        loop {
            let mut buf = vec![0u8; 65535];
            let n = self.capture.recv(&mut buf).await?;

            if let Some(state) = self.parse_response(&buf[..n], src_port)? {
                self.pending.remove(&src_port);
                return Ok(state);
            }

            // Check timeout
            if let Some((_, pending)) = self.pending.get(&src_port) {
                if pending.sent_at.elapsed() > Duration::from_secs(2) {
                    self.pending.remove(&src_port);
                    return Ok(PortState::Filtered);
                }
            }
        }
    }

    fn parse_response(&self, packet: &[u8], expected_src_port: u16) -> Result<Option<PortState>> {
        // Parse Ethernet + IP + TCP headers
        let tcp_packet = parse_tcp_packet(packet)?;

        if tcp_packet.destination() != expected_src_port {
            return Ok(None); // Not for us
        }

        // Check flags
        if tcp_packet.flags() & TcpFlags::SYN != 0 && tcp_packet.flags() & TcpFlags::ACK != 0 {
            // SYN/ACK received - port is open
            // Send RST to close connection
            self.send_rst(tcp_packet.source(), tcp_packet.destination()).await?;
            Ok(Some(PortState::Open))
        } else if tcp_packet.flags() & TcpFlags::RST != 0 {
            // RST received - port is closed
            Ok(Some(PortState::Closed))
        } else {
            Ok(None) // Unknown response
        }
    }

    async fn send_rst(&self, dst_ip: IpAddr, dst_port: u16) -> Result<()> {
        let packet = TcpPacketBuilder::new()
            .source(get_local_ip()?, dst_port)
            .destination(dst_ip, dst_port)
            .flags(TcpFlags::RST)
            .build()?;

        self.socket.send(&packet).await?;
        Ok(())
    }
}
```

## Detection Module (`prtip-detection`)

### Purpose

Service detection, OS fingerprinting, and banner analysis.

**Contents:**
- `service.rs` - Service version detection (Nmap probes)
- `os_fingerprint.rs` - OS detection (TCP/IP stack fingerprinting)
- `banner.rs` - Banner grabbing and parsing
- `probes.rs` - Probe database loading

### Service Detection

**File:** `crates/prtip-detection/src/service.rs`

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
        // Phase 1: NULL probe (wait for banner)
        if let Some(info) = self.null_probe(target).await? {
            return Ok(Some(info));
        }

        // Phase 2: Try registered probes
        let port = target.port();
        for probe in &self.probes {
            if probe.rarity > self.intensity {
                continue;
            }

            if !probe.ports.is_empty() && !probe.ports.contains(&port) {
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

        // Wait for banner (2 second timeout)
        let mut banner = vec![0u8; 4096];
        let n = tokio::time::timeout(
            Duration::from_secs(2),
            stream.read(&mut banner)
        ).await.ok().and_then(|r| r.ok()).unwrap_or(0);

        if n > 0 {
            let banner_str = String::from_utf8_lossy(&banner[..n]);
            Ok(self.match_banner(&banner_str))
        } else {
            Ok(None)
        }
    }

    async fn execute_probe(&self, target: SocketAddr, probe: &ServiceProbe) -> Result<Option<ServiceInfo>> {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(target)
        ).await??;

        // Send probe
        stream.write_all(&probe.payload).await?;
        stream.flush().await?;

        // Read response
        let mut response = vec![0u8; 8192];
        let n = tokio::time::timeout(
            Duration::from_secs(2),
            stream.read(&mut response)
        ).await.ok().and_then(|r| r.ok()).unwrap_or(0);

        if n > 0 {
            let response_str = String::from_utf8_lossy(&response[..n]);
            Ok(self.match_response(&response_str, &probe.matches))
        } else {
            Ok(None)
        }
    }

    fn match_banner(&self, banner: &str) -> Option<ServiceInfo> {
        // SSH detection
        if banner.starts_with("SSH-") {
            return Some(ServiceInfo {
                name: "ssh".to_string(),
                product: Some(extract_ssh_version(banner)),
                version: None,
                cpe: None,
            });
        }

        // FTP detection
        if banner.starts_with("220 ") && banner.contains("FTP") {
            return Some(ServiceInfo {
                name: "ftp".to_string(),
                product: Some(extract_ftp_server(banner)),
                version: None,
                cpe: None,
            });
        }

        // HTTP detection
        if banner.starts_with("HTTP/") {
            return Some(ServiceInfo {
                name: "http".to_string(),
                product: Some(extract_http_server(banner)),
                version: None,
                cpe: None,
            });
        }

        None
    }
}
```

## Plugin System (`prtip-plugins`)

### Purpose

Lua-based plugin system for extensibility.

**Contents:**
- `manager.rs` - Plugin discovery and lifecycle management
- `api.rs` - Plugin trait definitions
- `lua.rs` - Lua VM integration (mlua)
- `sandbox.rs` - Capability-based security

### Plugin API

**File:** `crates/prtip-plugins/src/api.rs`

```rust
#[async_trait::async_trait]
pub trait ScanPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    async fn on_load(&mut self) -> Result<()> {
        Ok(())
    }

    async fn pre_scan(&mut self, config: &ScanConfig) -> Result<()> {
        Ok(())
    }

    async fn post_scan(&mut self, results: &[ScanResult]) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait OutputPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn format(&self, results: &[ScanResult]) -> Result<String>;
}

#[async_trait::async_trait]
pub trait DetectionPlugin: Send + Sync {
    fn name(&self) -> &str;
    async fn analyze_banner(&self, banner: &str) -> Result<Option<ServiceInfo>>;
    async fn probe_service(&self, target: SocketAddr) -> Result<Option<ServiceInfo>>;
}
```

## Design Patterns in Practice

### 1. Builder Pattern

**Used for:** Complex packet construction, configuration objects

```rust
let packet = TcpPacketBuilder::new()
    .source(local_ip, local_port)
    .destination(target_ip, target_port)
    .flags(TcpFlags::SYN)
    .tcp_option(TcpOption::Mss(1460))
    .build()?;
```

### 2. Type State Pattern

**Used for:** Compile-time state machine enforcement

```rust
struct Scanner<S> {
    state: PhantomData<S>,
    config: Option<ScanConfig>,
}

struct Unconfigured;
struct Configured;
struct Running;

impl Scanner<Unconfigured> {
    pub fn configure(self, config: ScanConfig) -> Scanner<Configured> {
        Scanner {
            state: PhantomData,
            config: Some(config),
        }
    }
}

impl Scanner<Configured> {
    pub async fn start(self) -> Result<Scanner<Running>> {
        // Can only call start() if configured
        // Compiler enforces this at compile time
        Ok(Scanner {
            state: PhantomData,
            config: self.config,
        })
    }
}
```

### 3. Strategy Pattern

**Used for:** Scan type selection

```rust
trait ScanStrategy {
    async fn scan_port(&self, target: SocketAddr) -> Result<PortState>;
}

struct SynScanStrategy;
struct ConnectScanStrategy;

impl ScanStrategy for SynScanStrategy {
    async fn scan_port(&self, target: SocketAddr) -> Result<PortState> {
        // SYN scan implementation
    }
}

// Scan executor uses strategy pattern
pub struct Scanner {
    strategy: Box<dyn ScanStrategy>,
}
```

### 4. Observer Pattern

**Used for:** Result streaming, event notifications

```rust
pub trait ScanObserver: Send {
    fn on_result(&mut self, result: ScanResult);
    fn on_error(&mut self, error: PrtipError);
    fn on_complete(&mut self);
}

pub struct Scanner {
    observers: Vec<Box<dyn ScanObserver>>,
}

impl Scanner {
    fn notify_result(&mut self, result: ScanResult) {
        for observer in &mut self.observers {
            observer.on_result(result.clone());
        }
    }
}
```

### 5. Command Pattern

**Used for:** CLI argument handling

```rust
#[derive(Parser)]
pub enum Command {
    Scan(ScanCommand),
    List(ListCommand),
    Export(ExportCommand),
}

impl Command {
    pub async fn execute(&self) -> Result<()> {
        match self {
            Command::Scan(cmd) => cmd.execute().await,
            Command::List(cmd) => cmd.execute().await,
            Command::Export(cmd) => cmd.execute().await,
        }
    }
}
```

## Best Practices

### 1. Async/Await

**Always use async for I/O operations:**

```rust
// ✅ Good
pub async fn scan_port(target: SocketAddr) -> Result<PortState> {
    let stream = TcpStream::connect(target).await?;
    // ...
}

// ❌ Bad (blocking I/O in async context)
pub async fn scan_port_bad(target: SocketAddr) -> Result<PortState> {
    let stream = std::net::TcpStream::connect(target)?; // Blocks tokio thread!
    // ...
}
```

### 2. Error Handling

**Use `?` operator with `Result` return types:**

```rust
pub async fn execute_scan(&self) -> Result<ScanReport> {
    let targets = self.parse_targets()?; // Early return on error
    let results = self.scan_targets(&targets).await?;
    let report = self.generate_report(results)?;
    Ok(report)
}
```

### 3. Resource Management

**Use RAII pattern for resource cleanup:**

```rust
pub struct ScanSession {
    socket: RawSocket,
    capture: PacketCapture,
}

impl Drop for ScanSession {
    fn drop(&mut self) {
        tracing::info!("Cleaning up scan session");
        // Automatic cleanup when ScanSession goes out of scope
    }
}
```

### 4. Concurrency

**Use channels for inter-thread communication:**

```rust
let (tx, mut rx) = mpsc::channel(10000);

// Producer
tokio::spawn(async move {
    for result in results {
        tx.send(result).await.ok();
    }
});

// Consumer
while let Some(result) = rx.recv().await {
    process(result);
}
```

### 5. Testing

**Write unit tests for public APIs:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_packet_builder() {
        let packet = TcpPacketBuilder::new()
            .source(Ipv4Addr::LOCALHOST.into(), 12345)
            .destination(Ipv4Addr::LOCALHOST.into(), 80)
            .flags(TcpFlags::SYN)
            .build()
            .unwrap();

        assert!(packet.len() >= 40); // IP + TCP headers
    }

    #[test]
    fn test_port_range_iter() {
        let range = PortRange::range(80, 82);
        let ports: Vec<u16> = range.iter().collect();
        assert_eq!(ports, vec![80, 81, 82]);
    }
}
```

## See Also

- [Architecture](architecture.md) - System design and component relationships
- [Testing](testing.md) - Testing strategy and infrastructure
- [CI/CD](ci-cd.md) - Build automation and release process
- [Contributing](contributing.md) - Contribution guidelines
