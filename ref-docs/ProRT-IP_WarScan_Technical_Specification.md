# ProRT-IP WarScan: Complete Software Specification and Implementation Document

**Version:** 1.0  
**Date:** October 6, 2025  
**Document Type:** Technical Specification & Implementation Guide  
**Project Status:** Pre-Development (Genesis Phase)

---

## Executive Summary

ProRT-IP WarScan represents the next evolution of network reconnaissance tools—combining the comprehensive feature set of Nmap, the blistering speed of Masscan, the modern architecture of RustScan, and innovative adaptive learning capabilities. Written entirely in Rust for memory safety and performance, WarScan will deliver internet-scale scanning capabilities with advanced stealth features, extensible architecture, and a user-centric design philosophy.

**Core Value Proposition:**

- **Speed:** Internet-scale scanning (full IPv4 sweep in <6 minutes on appropriate hardware)
- **Safety:** Memory-safe implementation eliminating entire categories of vulnerabilities
- **Stealth:** Advanced evasion techniques including timing controls, decoys, fragmentation, and idle scanning
- **Completeness:** Full-featured from host discovery through OS/service detection
- **Extensibility:** Plugin architecture and scripting engine for custom workflows
- **Accessibility:** Progressive interfaces from CLI → TUI → Web → GUI

This document provides the complete technical foundation needed to architect, implement, test, and deploy ProRT-IP WarScan from conception through production release.

---

## Table of Contents

1. [Project Vision and Architecture](#1-project-vision-and-architecture)
2. [Technical Stack and Dependencies](#2-technical-stack-and-dependencies)
3. [Core Scanning Engine Architecture](#3-core-scanning-engine-architecture)
4. [Network Protocol Implementation](#4-network-protocol-implementation)
5. [Scanning Techniques Specifications](#5-scanning-techniques-specifications)
6. [Host Discovery Module](#6-host-discovery-module)
7. [OS Fingerprinting System](#7-os-fingerprinting-system)
8. [Service Detection and Banner Grabbing](#8-service-detection-and-banner-grabbing)
9. [Stealth and Evasion Techniques](#9-stealth-and-evasion-techniques)
10. [Performance and Optimization Strategy](#10-performance-and-optimization-strategy)
11. [Output and Reporting Systems](#11-output-and-reporting-systems)
12. [Extensibility and Plugin Architecture](#12-extensibility-and-plugin-architecture)
13. [User Interfaces](#13-user-interfaces)
14. [Cross-Platform Implementation](#14-cross-platform-implementation)
15. [Security and Safety Considerations](#15-security-and-safety-considerations)
16. [Testing Strategy](#16-testing-strategy)
17. [Development Roadmap](#17-development-roadmap)
18. [Project Structure and Organization](#18-project-structure-and-organization)

---

## 1. Project Vision and Architecture

### 1.1 Design Philosophy

ProRT-IP WarScan adheres to the following architectural principles:

1. **Modular Design:** Each scanning technique, protocol handler, and output formatter exists as an independent, testable module
2. **Asynchronous by Default:** All I/O operations use Tokio's async runtime for maximum concurrency
3. **Zero-Copy Where Possible:** Minimize memory allocations and copies in hot paths
4. **Type Safety:** Leverage Rust's type system to prevent invalid state transitions
5. **Progressive Enhancement:** Core functionality works without privileges; raw packet features enhance capabilities when available
6. **Fail-Safe Defaults:** Conservative defaults that prevent accidental network disruption

### 1.2 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      User Interface Layer                        │
│  (CLI Args Parser, TUI Dashboard, Web API, Desktop GUI)         │
└────────────────────────┬─────────────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────────────┐
│                    Orchestration Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │   Scanner    │  │   Rate       │  │   Result     │           │
│  │   Scheduler  │  │   Controller │  │   Aggregator │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
└────────────────────────┬─────────────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────────────┐
│                     Scanning Engine Layer                         │
│  ┌───────────────┐ ┌────────────────┐ ┌───────────────┐         │
│  │ Host Discovery│ │ Port Scanner   │ │ Service Det.  │         │
│  │  (ICMP/ARP)   │ │ (TCP/UDP/SCTP) │ │ (Banners/Probes)        │
│  └───────────────┘ └────────────────┘ └───────────────┘         │
│  ┌───────────────┐ ┌────────────────┐ ┌───────────────┐         │
│  │ OS Fingerprint│ │ Stealth Module │ │ Script Engine │         │
│  └───────────────┘ └────────────────┘ └───────────────┘         │
└────────────────────────┬─────────────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────────────┐
│                   Network Protocol Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │  Raw Packet  │  │  TCP Stack   │  │  Packet      │           │
│  │  Crafting    │  │  (Custom)    │  │  Capture     │           │
│  │  (pnet)      │  │              │  │  (libpcap)   │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
└────────────────────────┬─────────────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────────────┐
│                  Operating System Layer                           │
│  (Linux/Windows/macOS - Raw Sockets, BPF, Npcap, etc.)          │
└───────────────────────────────────────────────────────────────────┘
```

### 1.3 Core Components Overview

#### 1.3.1 Scanner Scheduler

- **Purpose:** Orchestrates scan jobs, manages target queues, distributes work across threads
- **Responsibilities:**
  - Parse and expand target specifications (CIDR, ranges, lists)
  - Randomize target order (configurable)
  - Shard targets across worker pools
  - Coordinate multi-phase scans (discovery → enumeration → deep inspection)
  
#### 1.3.2 Rate Controller

- **Purpose:** Adaptive rate limiting to prevent network saturation and detection
- **Responsibilities:**
  - Track packet transmission rates
  - Monitor response rates and timeouts
  - Implement congestion control (inspired by TCP RFC 2581)
  - Apply user-specified rate caps
  - Dynamic adjustment based on network conditions

#### 1.3.3 Result Aggregator

- **Purpose:** Collect, deduplicate, and merge scan results from multiple workers
- **Responsibilities:**
  - Thread-safe result collection
  - Merge partial results for the same host/port
  - Maintain result state (open/closed/filtered)
  - Stream results to output formatters

#### 1.3.4 Packet Crafting Engine

- **Purpose:** Generate raw network packets for all scan types
- **Responsibilities:**
  - Build Ethernet/IP/TCP/UDP/ICMP packets
  - Apply stealth transformations (fragments, TTL manipulation, etc.)
  - Checksum calculation
  - Source address/port spoofing

#### 1.3.5 Packet Capture Engine

- **Purpose:** Receive and parse network responses
- **Responsibilities:**
  - Configure BPF filters for efficient capture
  - Parse responses into structured data
  - Match responses to probes
  - Handle out-of-order packets

---

## 2. Technical Stack and Dependencies

### 2.1 Core Rust Crates

#### 2.1.1 Network Programming

```toml
[dependencies]
# Async Runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# Low-level networking and packet manipulation
pnet = "0.34"              # Cross-platform packet crafting
pnet_datalink = "0.34"     # Data link layer access
pnet_packet = "0.34"       # Packet parsing/building
socket2 = "0.5"            # Low-level socket operations
rawsock = "0.3"            # Raw socket wrapper

# Packet capture
pcap = "1.1"               # libpcap bindings
etherparse = "0.14"        # Fast packet parsing

# DNS
trust-dns-resolver = "0.23" # Async DNS resolution
```

#### 2.1.2 Concurrency and Performance

```toml
# Parallelism
rayon = "1.8"              # Data parallelism
crossbeam = "0.8"          # Lock-free structures
parking_lot = "0.12"       # Efficient mutex/rwlock

# Performance monitoring
criterion = "0.5"          # Benchmarking
perfcnt = "0.8"            # Hardware perf counters
```

#### 2.1.3 CLI and TUI

```toml
# Command-line parsing
clap = { version = "4.4", features = ["derive", "cargo"] }

# Terminal UI
ratatui = "0.25"           # Modern TUI framework
crossterm = "0.27"         # Terminal manipulation
```

#### 2.1.4 Output Formats

```toml
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
quick-xml = { version = "0.31", features = ["serialize"] }

# Database
rusqlite = { version = "0.30", features = ["bundled"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
```

#### 2.1.5 Scripting Engine

```toml
# Lua scripting
mlua = { version = "0.9", features = ["async", "vendored"] }

# Python scripting (optional)
pyo3 = { version = "0.20", features = ["auto-initialize"] }
```

#### 2.1.6 Cryptography and SSL

```toml
# TLS for banner grabbing
tokio-native-tls = "0.3"
native-tls = "0.2"
rustls = "0.22"

# Hashing for fingerprints
siphasher = "1.0"          # SipHash for masscan-style stateless
sha2 = "0.10"
```

#### 2.1.7 Utilities

```toml
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# IP address handling
ipnet = "2.9"
ipnetwork = "0.20"

# Random number generation
rand = "0.8"
rand_chacha = "0.3"        # Deterministic randomization

# Time handling
chrono = "0.4"
```

### 2.2 Platform-Specific Dependencies

#### 2.2.1 Windows

```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winsock2", "ws2def"] }
windows-sys = "0.52"
```

#### 2.2.2 Linux

```toml
[target.'cfg(target_os = "linux")'.dependencies]
nix = { version = "0.27", features = ["net"] }
libc = "0.2"
```

#### 2.2.3 macOS

```toml
[target.'cfg(target_os = "macos")'.dependencies]
nix = { version = "0.27", features = ["net"] }
```

### 2.3 Build Requirements

- **Rust:** 1.70+ (MSRV - Minimum Supported Rust Version)
- **libpcap:** Platform-specific installation required
  - Linux: `libpcap-dev`
  - Windows: WinPcap or Npcap (WinPcap-compatible mode)
  - macOS: Pre-installed
- **OpenSSL:** For TLS-enabled service detection
  - Can use `native-tls` for platform TLS or `rustls` for pure Rust

### 2.4 Optional Acceleration Libraries

```toml
[dependencies]
# Linux-specific high-performance options
# These are optional and detected at runtime
[target.'cfg(target_os = "linux")'.dependencies]
# PF_RING support (compile-time optional)
# DPDK support (compile-time optional)
```

---

## 3. Core Scanning Engine Architecture

### 3.1 Scanning Modes

WarScan implements multiple scanning paradigms optimized for different scenarios:

#### 3.1.1 Stateless Mode (Masscan-style)

- **Use Case:** Large-scale, initial discovery
- **Characteristics:**
  - No connection state maintained
  - SipHash-based response validation
  - Maximum transmission speed (10M+ pps capable)
  - Minimal memory footprint
  - Target randomization via custom indexing

**Implementation Pattern:**

```rust
/// Stateless scanner using SipHash for response validation
pub struct StatelessScanner {
    /// SipHash key for this scan session
    siphash_key: (u64, u64),
    
    /// Transmission rate limiter
    rate_limiter: RateLimiter,
    
    /// Raw packet sender
    packet_tx: PacketSender,
    
    /// Response validator
    response_rx: PacketReceiver,
}

impl StatelessScanner {
    /// Generate a stateless probe with SipHash sequence number
    fn craft_probe(&self, target_ip: Ipv4Addr, target_port: u16) -> TcpPacket {
        let mut packet = TcpPacket::new();
        packet.set_source(self.source_ip);
        packet.set_destination(target_ip);
        packet.set_dest_port(target_port);
        
        // Generate SipHash-based sequence number
        let seq = self.generate_seq_number(target_ip, target_port);
        packet.set_sequence(seq);
        packet.set_flags(TcpFlags::SYN);
        
        packet
    }
    
    /// Validate response using SipHash
    fn validate_response(&self, packet: &TcpPacket) -> Option<ScanResult> {
        // Recompute expected acknowledgment number
        let expected_ack = self.generate_seq_number(
            packet.get_destination(),
            packet.get_dest_port()
        ).wrapping_add(1);
        
        if packet.get_acknowledgment() == expected_ack {
            Some(ScanResult::Open)
        } else {
            None // Not our packet
        }
    }
    
    /// SipHash implementation for sequence number generation
    fn generate_seq_number(&self, ip: Ipv4Addr, port: u16) -> u32 {
        use siphasher::sip::SipHasher;
        use std::hash::Hasher;
        
        let mut hasher = SipHasher::new_with_keys(
            self.siphash_key.0,
            self.siphash_key.1
        );
        hasher.write(&ip.octets());
        hasher.write_u16(port);
        (hasher.finish() & 0xFFFFFFFF) as u32
    }
}
```

#### 3.1.2 Stateful Mode (Nmap-style)

- **Use Case:** Detailed enumeration, stealth scanning
- **Characteristics:**
  - Per-connection state tracking
  - Retransmission support
  - Congestion control
  - Multiple scan types (SYN, FIN, NULL, Xmas, etc.)
  - Deep packet inspection

**Implementation Pattern:**

```rust
/// Stateful connection tracker
pub struct ConnectionState {
    target: SocketAddr,
    scan_type: ScanType,
    probes_sent: Vec<ProbeInfo>,
    responses: Vec<ResponseInfo>,
    state: PortState,
    timeout: Instant,
}

/// Stateful scanner with connection tracking
pub struct StatefulScanner {
    /// Active connections being tracked
    connections: DashMap<ConnectionKey, ConnectionState>,
    
    /// Retransmission queue
    retransmit_queue: mpsc::Receiver<RetransmitRequest>,
    
    /// Congestion window (TCP-inspired)
    cwnd: AtomicUsize,
    
    /// Round-trip time estimator
    rtt_estimator: RttEstimator,
}

impl StatefulScanner {
    /// Send probe and track state
    async fn send_probe(&self, target: SocketAddr, scan_type: ScanType) -> Result<()> {
        let key = ConnectionKey::new(target, scan_type);
        
        let state = ConnectionState {
            target,
            scan_type,
            probes_sent: vec![ProbeInfo::new(Instant::now())],
            responses: Vec::new(),
            state: PortState::Unknown,
            timeout: Instant::now() + self.get_timeout(),
        };
        
        self.connections.insert(key, state);
        
        // Send the actual probe
        self.transmit_packet(target, scan_type).await?;
        
        Ok(())
    }
    
    /// Process received response
    fn handle_response(&self, packet: ResponsePacket) -> Option<ScanResult> {
        let key = self.extract_key(&packet)?;
        let mut connection = self.connections.get_mut(&key)?;
        
        connection.responses.push(ResponseInfo::from_packet(&packet));
        connection.state = self.determine_state(&connection, &packet);
        
        // Update RTT estimate
        if let Some(probe_time) = connection.probes_sent.first() {
            self.rtt_estimator.update(probe_time.sent_at.elapsed());
        }
        
        Some(ScanResult {
            target: connection.target,
            state: connection.state,
            scan_type: connection.scan_type,
        })
    }
    
    /// Congestion control (RFC 2581-inspired)
    fn adjust_congestion_window(&self, event: NetworkEvent) {
        match event {
            NetworkEvent::Ack => {
                // Additive increase
                self.cwnd.fetch_add(1, Ordering::SeqCst);
            }
            NetworkEvent::Timeout | NetworkEvent::Loss => {
                // Multiplicative decrease
                let current = self.cwnd.load(Ordering::SeqCst);
                self.cwnd.store(current / 2, Ordering::SeqCst);
            }
        }
    }
}
```

#### 3.1.3 Hybrid Mode

- **Use Case:** Balanced speed and depth
- **Characteristics:**
  - Fast stateless initial sweep
  - Stateful follow-up on responsive hosts
  - Configurable threshold for state tracking
  - Automatic mode selection based on network conditions

### 3.2 Target Specification and Randomization

#### 3.2.1 Target Parser

```rust
/// Target specification parser supporting multiple formats
pub enum TargetSpec {
    /// Single IP: 192.168.1.1
    SingleIp(IpAddr),
    
    /// CIDR notation: 10.0.0.0/24
    Cidr(IpNetwork),
    
    /// IP range: 10.0.0.1-254
    Range { start: IpAddr, end: IpAddr },
    
    /// Hostname (requires DNS resolution)
    Hostname(String),
    
    /// File containing targets (one per line)
    File(PathBuf),
}

impl TargetSpec {
    /// Expand into iterator of IP addresses
    pub fn expand(&self) -> Result<impl Iterator<Item = IpAddr>> {
        match self {
            TargetSpec::SingleIp(ip) => {
                Ok(Box::new(std::iter::once(*ip)))
            }
            TargetSpec::Cidr(network) => {
                Ok(Box::new(network.iter()))
            }
            TargetSpec::Range { start, end } => {
                // Create range iterator
                Ok(Box::new(IpRangeIterator::new(*start, *end)))
            }
            // ... other variants
        }
    }
}
```

#### 3.2.2 Target Randomization

Masscan-style randomization using a permutation index:

```rust
/// Target randomizer using permutation index
pub struct TargetRandomizer {
    total_targets: u64,
    current_index: AtomicU64,
    ranges: Vec<IpRange>,
}

impl TargetRandomizer {
    /// Get next target using permuted index
    pub fn next_target(&self) -> Option<IpAddr> {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst);
        
        if index >= self.total_targets {
            return None;
        }
        
        // Apply permutation function (e.g., multiplicative inverse modulo prime)
        let permuted = self.permute_index(index);
        
        // Map permuted index to IP address
        self.index_to_ip(permuted)
    }
    
    /// Permutation function for randomization
    fn permute_index(&self, index: u64) -> u64 {
        // Using multiplicative inverse in prime field
        // Example: index * multiplier % prime
        const MULTIPLIER: u64 = 2654435761; // Golden ratio prime
        (index.wrapping_mul(MULTIPLIER)) % self.total_targets
    }
    
    /// Map index to IP address across ranges
    fn index_to_ip(&self, mut index: u64) -> Option<IpAddr> {
        for range in &self.ranges {
            let range_size = range.size();
            if index < range_size {
                return Some(range.offset_ip(index));
            }
            index -= range_size;
        }
        None
    }
}
```

### 3.3 Worker Pool Architecture

```rust
/// Scanner worker pool for parallel execution
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_queue: Arc<SegQueue<ScanTask>>,
    result_queue: Arc<SegQueue<ScanResult>>,
    shutdown: Arc<AtomicBool>,
}

impl WorkerPool {
    pub fn new(num_workers: usize, config: WorkerConfig) -> Self {
        let task_queue = Arc::new(SegQueue::new());
        let result_queue = Arc::new(SegQueue::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        
        let workers = (0..num_workers)
            .map(|id| Worker::spawn(
                id,
                Arc::clone(&task_queue),
                Arc::clone(&result_queue),
                Arc::clone(&shutdown),
                config.clone(),
            ))
            .collect();
        
        Self {
            workers,
            task_queue,
            result_queue,
            shutdown,
        }
    }
    
    pub fn submit_task(&self, task: ScanTask) {
        self.task_queue.push(task);
    }
    
    pub async fn collect_results(&self) -> Vec<ScanResult> {
        let mut results = Vec::new();
        
        while let Some(result) = self.result_queue.pop() {
            results.push(result);
        }
        
        results
    }
}

/// Individual worker thread
struct Worker {
    id: usize,
    handle: JoinHandle<()>,
}

impl Worker {
    fn spawn(
        id: usize,
        tasks: Arc<SegQueue<ScanTask>>,
        results: Arc<SegQueue<ScanResult>>,
        shutdown: Arc<AtomicBool>,
        config: WorkerConfig,
    ) -> Self {
        let handle = tokio::spawn(async move {
            let mut scanner = create_scanner(&config);
            
            while !shutdown.load(Ordering::Acquire) {
                if let Some(task) = tasks.pop() {
                    match scanner.scan(task).await {
                        Ok(result) => results.push(result),
                        Err(e) => tracing::error!("Scan error: {}", e),
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        });
        
        Self { id, handle }
    }
}
```

---

## 4. Network Protocol Implementation

### 4.1 Custom TCP/IP Stack Overview

For maximum performance and control, WarScan implements its own userspace TCP/IP stack for crafting packets:

```rust
/// Custom TCP/IP packet builder
pub mod packet_builder {
    use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
    use pnet::packet::ip::{IpNextHeaderProtocols};
    use pnet::packet::ipv4::{MutableIpv4Packet, Ipv4Packet, checksum};
    use pnet::packet::tcp::{MutableTcpPacket, TcpPacket};
    
    /// Builder for TCP packets
    pub struct TcpPacketBuilder {
        source_ip: Ipv4Addr,
        dest_ip: Ipv4Addr,
        source_port: u16,
        dest_port: u16,
        seq: u32,
        ack: u32,
        flags: TcpFlags,
        window: u16,
        options: Vec<TcpOption>,
        payload: Vec<u8>,
    }
    
    impl TcpPacketBuilder {
        pub fn new() -> Self {
            Self {
                source_ip: Ipv4Addr::UNSPECIFIED,
                dest_ip: Ipv4Addr::UNSPECIFIED,
                source_port: 0,
                dest_port: 0,
                seq: 0,
                ack: 0,
                flags: TcpFlags::empty(),
                window: 65535,
                options: Vec::new(),
                payload: Vec::new(),
            }
        }
        
        pub fn source(mut self, ip: Ipv4Addr, port: u16) -> Self {
            self.source_ip = ip;
            self.source_port = port;
            self
        }
        
        pub fn destination(mut self, ip: Ipv4Addr, port: u16) -> Self {
            self.dest_ip = ip;
            self.dest_port = port;
            self
        }
        
        pub fn sequence(mut self, seq: u32) -> Self {
            self.seq = seq;
            self
        }
        
        pub fn acknowledgment(mut self, ack: u32) -> Self {
            self.ack = ack;
            self
        }
        
        pub fn flags(mut self, flags: TcpFlags) -> Self {
            self.flags = flags;
            self
        }
        
        pub fn window_size(mut self, window: u16) -> Self {
            self.window = window;
            self
        }
        
        pub fn tcp_option(mut self, option: TcpOption) -> Self {
            self.options.push(option);
            self
        }
        
        pub fn payload(mut self, data: Vec<u8>) -> Self {
            self.payload = data;
            self
        }
        
        /// Build the complete packet with Ethernet, IP, and TCP layers
        pub fn build(self) -> Result<Vec<u8>> {
            // Calculate sizes
            let tcp_options_len = self.calculate_options_length();
            let tcp_header_len = 20 + tcp_options_len;
            let tcp_total_len = tcp_header_len + self.payload.len();
            let ip_total_len = 20 + tcp_total_len;
            let eth_total_len = 14 + ip_total_len;
            
            let mut buffer = vec![0u8; eth_total_len];
            
            // Build Ethernet header
            {
                let mut eth_packet = MutableEthernetPacket::new(&mut buffer[..14])
                    .ok_or(Error::PacketTooSmall)?;
                
                eth_packet.set_source(self.source_mac);
                eth_packet.set_destination(self.dest_mac);
                eth_packet.set_ethertype(EtherTypes::Ipv4);
            }
            
            // Build IPv4 header
            {
                let mut ip_packet = MutableIpv4Packet::new(&mut buffer[14..])
                    .ok_or(Error::PacketTooSmall)?;
                
                ip_packet.set_version(4);
                ip_packet.set_header_length(5); // 20 bytes
                ip_packet.set_dscp(0);
                ip_packet.set_ecn(0);
                ip_packet.set_total_length(ip_total_len as u16);
                ip_packet.set_identification(rand::random());
                ip_packet.set_flags(Ipv4Flags::DontFragment);
                ip_packet.set_fragment_offset(0);
                ip_packet.set_ttl(64);
                ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
                ip_packet.set_source(self.source_ip);
                ip_packet.set_destination(self.dest_ip);
                
                // Calculate and set IP checksum
                let checksum = checksum(&ip_packet.to_immutable());
                ip_packet.set_checksum(checksum);
            }
            
            // Build TCP header
            {
                let tcp_start = 14 + 20;
                let mut tcp_packet = MutableTcpPacket::new(&mut buffer[tcp_start..])
                    .ok_or(Error::PacketTooSmall)?;
                
                tcp_packet.set_source(self.source_port);
                tcp_packet.set_destination(self.dest_port);
                tcp_packet.set_sequence(self.seq);
                tcp_packet.set_acknowledgement(self.ack);
                tcp_packet.set_data_offset((tcp_header_len / 4) as u8);
                tcp_packet.set_reserved(0);
                tcp_packet.set_flags(self.flags.bits());
                tcp_packet.set_window(self.window);
                tcp_packet.set_urgent_ptr(0);
                
                // Set TCP options
                if !self.options.is_empty() {
                    let options_bytes = self.serialize_options();
                    tcp_packet.set_options(&options_bytes);
                }
                
                // Set payload
                if !self.payload.is_empty() {
                    let payload_start = tcp_start + tcp_header_len;
                    buffer[payload_start..].copy_from_slice(&self.payload);
                }
                
                // Calculate and set TCP checksum
                let checksum = self.calculate_tcp_checksum(&tcp_packet);
                tcp_packet.set_checksum(checksum);
            }
            
            Ok(buffer)
        }
        
        /// Calculate TCP checksum including pseudo-header
        fn calculate_tcp_checksum(&self, tcp_packet: &MutableTcpPacket) -> u16 {
            use pnet::packet::tcp::ipv4_checksum;
            ipv4_checksum(&tcp_packet.to_immutable(), &self.source_ip, &self.dest_ip)
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
                bytes.push(1); // TCP option kind NOP
            }
            bytes
        }
    }
}
```

### 4.2 TCP Options Support

```rust
/// TCP options for fingerprinting and behavior control
#[derive(Debug, Clone)]
pub enum TcpOption {
    /// End of options list
    EndOfOptions,
    
    /// No operation (padding)
    Nop,
    
    /// Maximum segment size
    Mss(u16),
    
    /// Window scale factor
    WindowScale(u8),
    
    /// SACK permitted
    SackPermitted,
    
    /// SACK blocks
    Sack(Vec<(u32, u32)>),
    
    /// Timestamp
    Timestamp { tsval: u32, tsecr: u32 },
    
    /// Unknown/custom option
    Unknown { kind: u8, data: Vec<u8> },
}

impl TcpOption {
    pub fn length(&self) -> usize {
        match self {
            TcpOption::EndOfOptions | TcpOption::Nop => 1,
            TcpOption::Mss(_) => 4,
            TcpOption::WindowScale(_) => 3,
            TcpOption::SackPermitted => 2,
            TcpOption::Sack(blocks) => 2 + blocks.len() * 8,
            TcpOption::Timestamp { .. } => 10,
            TcpOption::Unknown { data, .. } => 2 + data.len(),
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            TcpOption::EndOfOptions => vec![0],
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
            // ... other options
            _ => Vec::new(),
        }
    }
}
```

### 4.3 UDP Packet Implementation

```rust
/// UDP packet builder
pub struct UdpPacketBuilder {
    source_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    source_port: u16,
    dest_port: u16,
    payload: Vec<u8>,
}

impl UdpPacketBuilder {
    pub fn build(self) -> Result<Vec<u8>> {
        let udp_len = 8 + self.payload.len();
        let ip_len = 20 + udp_len;
        let total_len = 14 + ip_len;
        
        let mut buffer = vec![0u8; total_len];
        
        // Build Ethernet header
        // ... (similar to TCP)
        
        // Build IP header
        // ... (similar to TCP but with UDP protocol)
        
        // Build UDP header
        {
            let udp_start = 14 + 20;
            let mut udp_packet = MutableUdpPacket::new(&mut buffer[udp_start..])
                .ok_or(Error::PacketTooSmall)?;
            
            udp_packet.set_source(self.source_port);
            udp_packet.set_destination(self.dest_port);
            udp_packet.set_length(udp_len as u16);
            
            // Copy payload
            if !self.payload.is_empty() {
                let payload_start = udp_start + 8;
                buffer[payload_start..].copy_from_slice(&self.payload);
            }
            
            // Calculate checksum
            let checksum = self.calculate_udp_checksum(&udp_packet);
            udp_packet.set_checksum(checksum);
        }
        
        Ok(buffer)
    }
}
```

### 4.4 ICMP Implementation

```rust
/// ICMP packet types for host discovery
pub enum IcmpProbeType {
    EchoRequest,
    TimestampRequest,
    NetmaskRequest,
    AddressMask,
}

pub struct IcmpPacketBuilder {
    source_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    probe_type: IcmpProbeType,
    identifier: u16,
    sequence: u16,
    payload: Vec<u8>,
}

impl IcmpPacketBuilder {
    pub fn build(self) -> Result<Vec<u8>> {
        let (icmp_type, icmp_code) = match self.probe_type {
            IcmpProbeType::EchoRequest => (8, 0),
            IcmpProbeType::TimestampRequest => (13, 0),
            IcmpProbeType::NetmaskRequest => (17, 0),
            IcmpProbeType::AddressMask => (18, 0),
        };
        
        let icmp_len = 8 + self.payload.len();
        let ip_len = 20 + icmp_len;
        let total_len = 14 + ip_len;
        
        let mut buffer = vec![0u8; total_len];
        
        // Build layers...
        {
            let icmp_start = 14 + 20;
            let mut icmp_packet = MutableIcmpPacket::new(&mut buffer[icmp_start..])
                .ok_or(Error::PacketTooSmall)?;
            
            icmp_packet.set_icmp_type(IcmpType::new(icmp_type));
            icmp_packet.set_icmp_code(IcmpCode::new(icmp_code));
            
            // Set identifier and sequence
            buffer[icmp_start + 4..icmp_start + 6].copy_from_slice(&self.identifier.to_be_bytes());
            buffer[icmp_start + 6..icmp_start + 8].copy_from_slice(&self.sequence.to_be_bytes());
            
            // Copy payload
            if !self.payload.is_empty() {
                buffer[icmp_start + 8..].copy_from_slice(&self.payload);
            }
            
            // Calculate checksum
            let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
            icmp_packet.set_checksum(checksum);
        }
        
        Ok(buffer)
    }
}
```

### 4.5 Packet Transmission

```rust
/// Low-level packet transmitter
pub struct PacketTransmitter {
    /// Network interface for transmission
    interface: NetworkInterface,
    
    /// Raw socket sender
    tx: Box<dyn DataLinkSender>,
    
    /// Transmission statistics
    stats: Arc<TransmitStats>,
}

impl PacketTransmitter {
    pub fn new(interface_name: &str) -> Result<Self> {
        let interface = pnet::datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == interface_name)
            .ok_or(Error::InterfaceNotFound)?;
        
        let config = Config {
            write_buffer_size: 4096 * 1024, // 4MB write buffer
            read_buffer_size: 4096 * 1024,  // 4MB read buffer
            read_timeout: Some(Duration::from_millis(100)),
            write_timeout: None,
            channel_type: ChannelType::Layer2,
            bpf_fd_attempts: 1000,
            linux_fanout: None,
            promiscuous: false,
        };
        
        let (tx, _rx) = match pnet::datalink::channel(&interface, config) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err(Error::UnsupportedChannel),
            Err(e) => return Err(Error::ChannelCreation(e)),
        };
        
        Ok(Self {
            interface,
            tx,
            stats: Arc::new(TransmitStats::default()),
        })
    }
    
    /// Send a raw packet
    pub async fn send(&mut self, packet: &[u8]) -> Result<()> {
        self.tx.send_to(packet, None)
            .ok_or(Error::SendFailed)
            .map(|_| {
                self.stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                self.stats.bytes_sent.fetch_add(packet.len(), Ordering::Relaxed);
            })
    }
    
    /// Send multiple packets in batch
    pub async fn send_batch(&mut self, packets: &[Vec<u8>]) -> Result<usize> {
        let mut sent = 0;
        for packet in packets {
            if self.send(packet).await.is_ok() {
                sent += 1;
            }
        }
        Ok(sent)
    }
}
```

### 4.6 Packet Reception and Parsing

```rust
/// Packet receiver with BPF filtering
pub struct PacketReceiver {
    /// Capture handle
    capture: Capture<Active>,
    
    /// BPF filter for this scan
    filter: String,
    
    /// Reception statistics
    stats: Arc<ReceiveStats>,
}

impl PacketReceiver {
    pub fn new(interface: &str, filter: &str) -> Result<Self> {
        let mut capture = Capture::from_device(interface)?
            .promisc(true)
            .snaplen(65535)
            .buffer_size(4096 * 1024) // 4MB
            .timeout(100)
            .open()?;
        
        capture.filter(filter, true)?;
        
        Ok(Self {
            capture,
            filter: filter.to_string(),
            stats: Arc::new(ReceiveStats::default()),
        })
    }
    
    /// Receive next packet (async wrapper)
    pub async fn recv(&mut self) -> Result<PacketData> {
        tokio::task::spawn_blocking(move || {
            match self.capture.next_packet() {
                Ok(packet) => {
                    self.stats.packets_received.fetch_add(1, Ordering::Relaxed);
                    Ok(PacketData::from_slice(packet.data))
                }
                Err(e) => Err(Error::CaptureError(e)),
            }
        }).await?
    }
    
    /// Receive packets in batch
    pub async fn recv_batch(&mut self, max_count: usize) -> Vec<PacketData> {
        let mut packets = Vec::with_capacity(max_count);
        
        for _ in 0..max_count {
            match self.recv().await {
                Ok(packet) => packets.push(packet),
                Err(_) => break,
            }
        }
        
        packets
    }
}

/// Parsed packet data
#[derive(Debug, Clone)]
pub struct PacketData {
    /// Ethernet layer
    pub ethernet: EthernetHeader,
    
    /// IP layer (v4 or v6)
    pub ip: IpHeader,
    
    /// Transport layer
    pub transport: TransportHeader,
    
    /// Payload
    pub payload: Vec<u8>,
    
    /// Capture timestamp
    pub timestamp: Instant,
}

impl PacketData {
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        // Parse Ethernet frame
        let eth_packet = EthernetPacket::new(data)
            .ok_or(Error::InvalidPacket)?;
        
        let ethernet = EthernetHeader {
            source: eth_packet.get_source(),
            destination: eth_packet.get_destination(),
            ethertype: eth_packet.get_ethertype(),
        };
        
        // Parse IP layer
        let ip = match eth_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                let ip_packet = Ipv4Packet::new(eth_packet.payload())
                    .ok_or(Error::InvalidPacket)?;
                IpHeader::V4(Ipv4Header::from_packet(&ip_packet))
            }
            EtherTypes::Ipv6 => {
                let ip_packet = Ipv6Packet::new(eth_packet.payload())
                    .ok_or(Error::InvalidPacket)?;
                IpHeader::V6(Ipv6Header::from_packet(&ip_packet))
            }
            _ => return Err(Error::UnsupportedEthertype),
        };
        
        // Parse transport layer
        let transport = match ip.next_protocol() {
            IpNextHeaderProtocols::Tcp => {
                let tcp_packet = TcpPacket::new(ip.payload())
                    .ok_or(Error::InvalidPacket)?;
                TransportHeader::Tcp(TcpHeader::from_packet(&tcp_packet))
            }
            IpNextHeaderProtocols::Udp => {
                let udp_packet = UdpPacket::new(ip.payload())
                    .ok_or(Error::InvalidPacket)?;
                TransportHeader::Udp(UdpHeader::from_packet(&udp_packet))
            }
            IpNextHeaderProtocols::Icmp => {
                let icmp_packet = IcmpPacket::new(ip.payload())
                    .ok_or(Error::InvalidPacket)?;
                TransportHeader::Icmp(IcmpHeader::from_packet(&icmp_packet))
            }
            _ => TransportHeader::Unknown,
        };
        
        Ok(Self {
            ethernet,
            ip,
            transport,
            payload: transport.get_payload().to_vec(),
            timestamp: Instant::now(),
        })
    }
}
```

---

## 5. Scanning Techniques Specifications

### 5.1 TCP Scan Types

#### 5.1.1 TCP SYN Scan (Half-Open)

**Description:** The default and most popular scan type. Sends SYN packets without completing the three-way handshake.

**Advantages:**

- Fast and efficient
- Relatively stealthy (no full connection)
- Works against any compliant TCP stack
- Clear differentiation between open/closed/filtered

**Implementation:**

```rust
/// TCP SYN scanner implementation
pub struct SynScanner {
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl SynScanner {
    pub async fn scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
        // Craft SYN packet
        let packet = TcpPacketBuilder::new()
            .source(self.local_ip, self.local_port)
            .destination(target.ip(), target.port())
            .sequence(rand::random())
            .flags(TcpFlags::SYN)
            .window_size(65535)
            .tcp_option(TcpOption::Mss(1460))
            .tcp_option(TcpOption::WindowScale(7))
            .tcp_option(TcpOption::SackPermitted)
            .build()?;
        
        // Send SYN
        self.transmitter.send(&packet).await?;
        
        // Wait for response with timeout
        let start = Instant::now();
        while start.elapsed() < self.timeout {
            if let Ok(response) = self.receiver.recv().await {
                // Check if response is for our probe
                if !self.matches_probe(&response, &target) {
                    continue;
                }
                
                // Analyze response
                return Ok(self.analyze_syn_response(&response));
            }
        }
        
        // Timeout - port is filtered
        Ok(PortState::Filtered)
    }
    
    fn analyze_syn_response(&self, packet: &PacketData) -> PortState {
        match &packet.transport {
            TransportHeader::Tcp(tcp) => {
                if tcp.flags.contains(TcpFlags::SYN | TcpFlags::ACK) {
                    // SYN+ACK = Open
                    // Send RST to tear down connection
                    self.send_rst(packet);
                    PortState::Open
                } else if tcp.flags.contains(TcpFlags::RST) {
                    // RST = Closed
                    PortState::Closed
                } else {
                    PortState::Filtered
                }
            }
            TransportHeader::Icmp(icmp) => {
                // ICMP error = Filtered
                if self.is_port_unreachable_icmp(icmp) {
                    PortState::Filtered
                } else {
                    PortState::Unknown
                }
            }
            _ => PortState::Unknown,
        }
    }
    
    fn send_rst(&self, original_packet: &PacketData) {
        // Send RST to tear down half-open connection
        if let TransportHeader::Tcp(tcp) = &original_packet.transport {
            let rst_packet = TcpPacketBuilder::new()
                .source(self.local_ip, self.local_port)
                .destination(original_packet.ip.source(), tcp.source_port)
                .sequence(tcp.ack_number)
                .flags(TcpFlags::RST)
                .build();
            
            if let Ok(packet) = rst_packet {
                let _ = self.transmitter.send(&packet);
            }
        }
    }
}
```

**Detection Characteristics:**

- Logged by most IDSs as incomplete connections
- Appears in system logs on target
- Firewall-friendly (many allow SYN but track state)

#### 5.1.2 TCP Connect Scan

**Description:** Uses OS socket connect() system call. Fallback when raw socket privileges unavailable.

**Implementation:**

```rust
/// TCP Connect scanner using OS sockets
pub struct ConnectScanner {
    timeout: Duration,
    max_concurrent: usize,
}

impl ConnectScanner {
    pub async fn scan_port(&self, target: SocketAddr) -> Result<PortState> {
        match tokio::time::timeout(
            self.timeout,
            TcpStream::connect(target)
        ).await {
            Ok(Ok(_stream)) => {
                // Connection succeeded = Open
                // Stream automatically closed on drop
                Ok(PortState::Open)
            }
            Ok(Err(e)) => {
                // Connection refused = Closed
                if e.kind() == std::io::ErrorKind::ConnectionRefused {
                    Ok(PortState::Closed)
                } else {
                    Ok(PortState::Filtered)
                }
            }
            Err(_) => {
                // Timeout = Filtered
                Ok(PortState::Filtered)
            }
        }
    }
    
    pub async fn scan_ports_batch(&self, targets: Vec<SocketAddr>) -> Vec<ScanResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let results = Arc::new(Mutex::new(Vec::new()));
        
        let mut handles = Vec::new();
        
        for target in targets {
            let sem = Arc::clone(&semaphore);
            let res = Arc::clone(&results);
            let timeout = self.timeout;
            
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await;
                
                let state = match tokio::time::timeout(
                    timeout,
                    TcpStream::connect(target)
                ).await {
                    Ok(Ok(_)) => PortState::Open,
                    Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                        PortState::Closed
                    }
                    _ => PortState::Filtered,
                };
                
                res.lock().await.push(ScanResult { target, state });
            });
            
            handles.push(handle);
        }
        
        // Wait for all scans
        futures::future::join_all(handles).await;
        
        Arc::try_unwrap(results).unwrap().into_inner()
    }
}
```

**Advantages:**

- No special privileges required
- Works on all platforms
- Reliable results

**Disadvantages:**

- Slower than raw packet scans
- Creates full connections (more logs)
- More easily detected

#### 5.1.3 TCP FIN Scan

**Description:** Sends FIN packet to closed port (should get RST), open port should not respond per RFC 793.

**Implementation:**

```rust
pub struct FinScanner {
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl FinScanner {
    pub async fn scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
        // Craft FIN packet
        let packet = TcpPacketBuilder::new()
            .source(self.local_ip, self.local_port)
            .destination(target.ip(), target.port())
            .sequence(rand::random())
            .flags(TcpFlags::FIN)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        // Wait for response
        let start = Instant::now();
        while start.elapsed() < self.timeout {
            if let Ok(response) = self.receiver.recv().await {
                if !self.matches_probe(&response, &target) {
                    continue;
                }
                
                // RST response = Closed
                if let TransportHeader::Tcp(tcp) = &response.transport {
                    if tcp.flags.contains(TcpFlags::RST) {
                        return Ok(PortState::Closed);
                    }
                }
                
                // ICMP unreachable = Filtered
                if let TransportHeader::Icmp(_) = &response.transport {
                    return Ok(PortState::Filtered);
                }
            }
        }
        
        // No response = Open or Filtered (RFC 793 compliant stacks)
        Ok(PortState::OpenFiltered)
    }
}
```

**Stealth Characteristics:**

- Can bypass simple stateless firewalls
- Not logged as connection attempt
- Effective against non-stateful filters

**Limitations:**

- Not all OS stacks are RFC 793 compliant
- Windows, Cisco, BSDI, HP/UX may respond to FIN on open ports
- Result ambiguity (open|filtered)

#### 5.1.4 TCP NULL Scan

**Description:** TCP packet with no flags set.

```rust
pub async fn null_scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
    let packet = TcpPacketBuilder::new()
        .source(self.local_ip, self.local_port)
        .destination(target.ip(), target.port())
        .sequence(rand::random())
        .flags(TcpFlags::empty()) // No flags
        .build()?;
    
    // Same response analysis as FIN scan
    self.send_and_analyze(packet, target).await
}
```

#### 5.1.5 TCP Xmas Scan

**Description:** TCP packet with FIN, PSH, and URG flags set ("lights up like a Christmas tree").

```rust
pub async fn xmas_scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
    let packet = TcpPacketBuilder::new()
        .source(self.local_ip, self.local_port)
        .destination(target.ip(), target.port())
        .sequence(rand::random())
        .flags(TcpFlags::FIN | TcpFlags::PSH | TcpFlags::URG)
        .build()?;
    
    self.send_and_analyze(packet, target).await
}
```

#### 5.1.6 TCP ACK Scan

**Description:** Sends ACK packets to determine firewall rules. Used to map firewall rulesets rather than determine open ports.

```rust
/// ACK scanner for firewall mapping
pub struct AckScanner {
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl AckScanner {
    pub async fn scan_port(&mut self, target: SocketAddr) -> Result<FirewallState> {
        let packet = TcpPacketBuilder::new()
            .source(self.local_ip, self.local_port)
            .destination(target.ip(), target.port())
            .sequence(rand::random())
            .acknowledgment(rand::random())
            .flags(TcpFlags::ACK)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        let start = Instant::now();
        while start.elapsed() < self.timeout {
            if let Ok(response) = self.receiver.recv().await {
                if !self.matches_probe(&response, &target) {
                    continue;
                }
                
                match &response.transport {
                    TransportHeader::Tcp(tcp) if tcp.flags.contains(TcpFlags::RST) => {
                        // RST received = Unfiltered
                        return Ok(FirewallState::Unfiltered);
                    }
                    TransportHeader::Icmp(icmp) if self.is_port_unreachable_icmp(icmp) => {
                        // ICMP unreachable = Filtered
                        return Ok(FirewallState::Filtered);
                    }
                    _ => {}
                }
            }
        }
        
        // No response = Filtered
        Ok(FirewallState::Filtered)
    }
}
```

**Use Case:** Determine which ports are filtered by firewalls vs simply closed.

#### 5.1.7 TCP Window Scan

**Description:** Similar to ACK scan but examines TCP window size in RST packets.

```rust
pub async fn window_scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
    // Send ACK packet
    let packet = TcpPacketBuilder::new()
        .source(self.local_ip, self.local_port)
        .destination(target.ip(), target.port())
        .sequence(rand::random())
        .acknowledgment(rand::random())
        .flags(TcpFlags::ACK)
        .build()?;
    
    self.transmitter.send(&packet).await?;
    
    if let Ok(response) = self.recv_with_timeout(self.timeout).await {
        if let TransportHeader::Tcp(tcp) = &response.transport {
            if tcp.flags.contains(TcpFlags::RST) {
                // Check window size
                if tcp.window > 0 {
                    return Ok(PortState::Open);
                } else {
                    return Ok(PortState::Closed);
                }
            }
        }
    }
    
    Ok(PortState::Filtered)
}
```

**Note:** Less reliable than other methods, depends on OS-specific behavior.

#### 5.1.8 TCP Maimon Scan

**Description:** Sends FIN/ACK packets. Named after Uriel Maimon who discovered this technique.

```rust
pub async fn maimon_scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
    let packet = TcpPacketBuilder::new()
        .source(self.local_ip, self.local_port)
        .destination(target.ip(), target.port())
        .sequence(rand::random())
        .acknowledgment(rand::random())
        .flags(TcpFlags::FIN | TcpFlags::ACK)
        .build()?;
    
    self.transmitter.send(&packet).await?;
    
    // Most systems: no response = open|filtered, RST = closed
    // Some BSD systems: RST for open ports
    self.analyze_stealth_response(target).await
}
```

### 5.2 UDP Scanning

**Challenge:** UDP is connectionless; many services don't respond to empty packets.

#### 5.2.1 Basic UDP Scan

```rust
/// UDP scanner with protocol-specific payloads
pub struct UdpScanner {
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
    probe_database: ProbeDatabase,
}

impl UdpScanner {
    pub async fn scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
        // Get protocol-specific payload
        let payload = self.probe_database
            .get_udp_payload(target.port())
            .unwrap_or_default();
        
        // Build UDP packet
        let packet = UdpPacketBuilder::new()
            .source(self.local_ip, self.local_port)
            .destination(target.ip(), target.port())
            .payload(payload)
            .build()?;
        
        // Send probe
        self.transmitter.send(&packet).await?;
        
        // Analyze response
        let start = Instant::now();
        while start.elapsed() < self.timeout {
            if let Ok(response) = self.receiver.recv().await {
                match &response.transport {
                    TransportHeader::Udp(_) => {
                        // UDP response = Open
                        return Ok(PortState::Open);
                    }
                    TransportHeader::Icmp(icmp) => {
                        // ICMP Port Unreachable (Type 3, Code 3) = Closed
                        if icmp.icmp_type == IcmpTypes::DestinationUnreachable
                            && icmp.icmp_code == 3 {
                            return Ok(PortState::Closed);
                        }
                        // Other ICMP unreachables = Filtered
                        if self.is_filtered_icmp(icmp) {
                            return Ok(PortState::Filtered);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // No response = Open or Filtered (ambiguous)
        Ok(PortState::OpenFiltered)
    }
}
```

#### 5.2.2 UDP Protocol-Specific Probes

```rust
/// Database of UDP protocol-specific payloads
pub struct ProbeDatabase {
    probes: HashMap<u16, Vec<u8>>,
}

impl ProbeDatabase {
    pub fn new() -> Self {
        let mut probes = HashMap::new();
        
        // DNS query (port 53)
        probes.insert(53, vec![
            0x00, 0x00, // Transaction ID
            0x01, 0x00, // Flags: Standard query
            0x00, 0x01, // Questions: 1
            0x00, 0x00, // Answer RRs: 0
            0x00, 0x00, // Authority RRs: 0
            0x00, 0x00, // Additional RRs: 0
            // Query for version.bind TXT CH
            0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e,
            0x04, 0x62, 0x69, 0x6e, 0x64,
            0x00, // Null terminator
            0x00, 0x10, // Type: TXT
            0x00, 0x03, // Class: CH (Chaos)
        ]);
        
        // SNMP GetRequest (port 161)
        probes.insert(161, vec![
            0x30, 0x26, // SEQUENCE
            0x02, 0x01, 0x01, // Version: 1
            0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, // Community: public
            0xa0, 0x19, // GetRequest PDU
            0x02, 0x01, 0x00, // Request ID
            0x02, 0x01, 0x00, // Error status
            0x02, 0x01, 0x00, // Error index
            0x30, 0x0e, // Variable bindings
            0x30, 0x0c, // Variable binding
            0x06, 0x08, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x01, 0x00, // OID: sysDescr
            0x05, 0x00, // NULL
        ]);
        
        // NTP Version 3 request (port 123)
        probes.insert(123, vec![
            0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        
        // Add more protocol-specific probes...
        // NetBIOS (137), DHCP (67/68), SIP (5060), etc.
        
        Self { probes }
    }
    
    pub fn get_udp_payload(&self, port: u16) -> Option<Vec<u8>> {
        self.probes.get(&port).cloned()
    }
}
```

### 5.3 SCTP Scanning

```rust
/// SCTP INIT scan (similar to TCP SYN)
pub struct SctpScanner {
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl SctpScanner {
    pub async fn init_scan(&mut self, target: SocketAddr) -> Result<PortState> {
        // Build SCTP INIT chunk
        let packet = SctpPacketBuilder::new()
            .source(self.local_ip, self.local_port)
            .destination(target.ip(), target.port())
            .chunk(SctpChunk::Init {
                initiate_tag: rand::random(),
                a_rwnd: 65535,
                num_outbound_streams: 10,
                num_inbound_streams: 10,
                initial_tsn: rand::random(),
            })
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        // Wait for INIT-ACK or ABORT
        if let Ok(response) = self.recv_with_timeout(self.timeout).await {
            if let TransportHeader::Sctp(sctp) = &response.transport {
                match &sctp.chunk_type {
                    SctpChunkType::InitAck => return Ok(PortState::Open),
                    SctpChunkType::Abort => return Ok(PortState::Closed),
                    _ => {}
                }
            }
        }
        
        Ok(PortState::Filtered)
    }
    
    pub async fn cookie_echo_scan(&mut self, target: SocketAddr) -> Result<PortState> {
        // COOKIE ECHO scan - more stealthy
        // Open ports silently drop, closed ports send ABORT
        let packet = SctpPacketBuilder::new()
            .source(self.local_ip, self.local_port)
            .destination(target.ip(), target.port())
            .chunk(SctpChunk::CookieEcho {
                cookie: vec![0; 32], // Dummy cookie
            })
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        if let Ok(response) = self.recv_with_timeout(self.timeout).await {
            if let TransportHeader::Sctp(sctp) = &response.transport {
                if matches!(sctp.chunk_type, SctpChunkType::Abort) {
                    return Ok(PortState::Closed);
                }
            }
        }
        
        // No response = open|filtered
        Ok(PortState::OpenFiltered)
    }
}
```

---

## 6. Host Discovery Module

### 6.1 ICMP-based Discovery

```rust
/// ICMP-based host discovery
pub struct IcmpDiscovery {
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
    probes: Vec<IcmpProbeType>,
}

impl IcmpDiscovery {
    pub async fn discover_host(&mut self, target: IpAddr) -> Result<HostStatus> {
        let mut responses = Vec::new();
        
        for probe_type in &self.probes {
            let packet = IcmpPacketBuilder::new()
                .source(self.local_ip)
                .destination(target)
                .probe_type(*probe_type)
                .identifier(rand::random())
                .sequence(rand::random())
                .build()?;
            
            self.transmitter.send(&packet).await?;
            
            // Wait for response
            let start = Instant::now();
            while start.elapsed() < self.timeout {
                if let Ok(response) = self.receiver.recv().await {
                    if self.is_response_to_probe(&response, target) {
                        responses.push((probe_type, response));
                        break;
                    }
                }
            }
        }
        
        if responses.is_empty() {
            Ok(HostStatus::Down)
        } else {
            Ok(HostStatus::Up {
                response_time: responses[0].1.timestamp.elapsed(),
                icmp_types: responses.iter()
                    .map(|(t, _)| *t)
                    .collect(),
            })
        }
    }
}
```

### 6.2 ARP-based Discovery (LAN)

```rust
/// ARP-based host discovery for local networks
pub struct ArpDiscovery {
    interface: NetworkInterface,
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl ArpDiscovery {
    pub async fn discover_host(&mut self, target: Ipv4Addr) -> Result<HostStatus> {
        // Build ARP request
        let packet = ArpPacketBuilder::new()
            .operation(ArpOperation::Request)
            .sender_hw_addr(self.interface.mac.unwrap())
            .sender_proto_addr(self.local_ip)
            .target_hw_addr(MacAddr::zero())
            .target_proto_addr(target)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        // Wait for ARP reply
        let start = Instant::now();
        while start.elapsed() < self.timeout {
            if let Ok(response) = self.receiver.recv().await {
                if let Some(arp_reply) = self.parse_arp_reply(&response, target) {
                    return Ok(HostStatus::Up {
                        response_time: start.elapsed(),
                        mac_address: Some(arp_reply.sender_hw_addr),
                        mac_vendor: self.lookup_mac_vendor(&arp_reply.sender_hw_addr),
                    });
                }
            }
        }
        
        Ok(HostStatus::Down)
    }
    
    /// Sweep entire subnet using ARP
    pub async fn sweep_subnet(&mut self, network: Ipv4Network) -> Vec<HostInfo> {
        let hosts: Vec<_> = network.iter().collect();
        let mut results = Vec::new();
        
        // Send all ARP requests
        for ip in &hosts {
            let packet = self.build_arp_request(*ip)?;
            self.transmitter.send(&packet).await?;
        }
        
        // Collect responses
        let deadline = Instant::now() + self.timeout;
        while Instant::now() < deadline {
            if let Ok(response) = self.receiver.recv().await {
                if let Some(host_info) = self.extract_host_info(&response) {
                    results.push(host_info);
                }
            }
        }
        
        results
    }
    
    fn lookup_mac_vendor(&self, mac: &MacAddr) -> Option<String> {
        // Look up OUI in vendor database
        // Database loaded from IEEE OUI list
        self.vendor_db.lookup(mac)
    }
}
```

### 6.3 TCP/UDP Ping

```rust
/// TCP SYN ping (alternative to ICMP)
pub struct TcpPing {
    ports: Vec<u16>,
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl TcpPing {
    pub async fn ping(&mut self, target: IpAddr) -> Result<HostStatus> {
        // Try multiple common ports
        for &port in &self.ports {
            let packet = TcpPacketBuilder::new()
                .source(self.local_ip, rand::random())
                .destination(target, port)
                .sequence(rand::random())
                .flags(TcpFlags::SYN)
                .build()?;
            
            self.transmitter.send(&packet).await?;
            
            // Any TCP response (SYN/ACK or RST) indicates host is up
            let start = Instant::now();
            while start.elapsed() < self.timeout {
                if let Ok(response) = self.receiver.recv().await {
                    if self.is_tcp_response(&response, target, port) {
                        return Ok(HostStatus::Up {
                            response_time: start.elapsed(),
                            responding_port: Some(port),
                        });
                    }
                }
            }
        }
        
        Ok(HostStatus::Down)
    }
}

/// UDP ping
pub struct UdpPing {
    ports: Vec<u16>,
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
    timeout: Duration,
}

impl UdpPing {
    pub async fn ping(&mut self, target: IpAddr) -> Result<HostStatus> {
        for &port in &self.ports {
            let payload = self.get_protocol_payload(port);
            
            let packet = UdpPacketBuilder::new()
                .source(self.local_ip, rand::random())
                .destination(target, port)
                .payload(payload)
                .build()?;
            
            self.transmitter.send(&packet).await?;
            
            // UDP response or ICMP port unreachable indicates host is up
            if let Ok(response) = self.recv_with_timeout(self.timeout).await {
                if self.indicates_host_up(&response, target) {
                    return Ok(HostStatus::Up {
                        response_time: response.timestamp.elapsed(),
                    });
                }
            }
        }
        
        Ok(HostStatus::Down)
    }
}
```

### 6.4 Comprehensive Host Discovery

```rust
/// Multi-method host discovery orchestrator
pub struct HostDiscovery {
    icmp: Option<IcmpDiscovery>,
    arp: Option<ArpDiscovery>,
    tcp_ping: Option<TcpPing>,
    udp_ping: Option<UdpPing>,
    parallel_probes: bool,
}

impl HostDiscovery {
    pub async fn discover(&mut self, target: IpAddr) -> Result<HostStatus> {
        if self.parallel_probes {
            // Run all methods in parallel
            let mut handles = Vec::new();
            
            if let Some(ref mut icmp) = self.icmp {
                handles.push(tokio::spawn(async move {
                    icmp.discover_host(target).await
                }));
            }
            
            if let Some(ref mut arp) = self.arp {
                if target.is_ipv4() {
                    handles.push(tokio::spawn(async move {
                        arp.discover_host(target).await
                    }));
                }
            }
            
            // Wait for first positive response
            // or all methods to complete
            for handle in handles {
                if let Ok(Ok(HostStatus::Up { .. })) = handle.await {
                    return Ok(HostStatus::Up { .. });
                }
            }
            
            Ok(HostStatus::Down)
        } else {
            // Sequential probing with early exit
            if let Some(ref mut icmp) = self.icmp {
                if matches!(icmp.discover_host(target).await?, HostStatus::Up { .. }) {
                    return Ok(HostStatus::Up { .. });
                }
            }
            
            // Try other methods...
            
            Ok(HostStatus::Down)
        }
    }
}
```

---

## 7. OS Fingerprinting System

### 7.1 Fingerprint Database Structure

```rust
/// OS fingerprint from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsFingerprint {
    /// OS name and version
    pub os_name: String,
    
    /// CPE (Common Platform Enumeration)
    pub cpe: Option<String>,
    
    /// OS class (e.g., "Windows", "Linux", "BSD")
    pub os_class: String,
    
    /// Device type (e.g., "general purpose", "router", "WAP")
    pub device_type: String,
    
    /// Fingerprint tests and expected values
    pub tests: HashMap<String, TestValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestValues {
    /// Expected values for this test
    pub values: Vec<String>,
    
    /// Weight/confidence of this test
    pub weight: f64,
}

/// Subject fingerprint (captured from target)
#[derive(Debug, Clone)]
pub struct SubjectFingerprint {
    pub target: IpAddr,
    pub timestamp: SystemTime,
    pub tests: HashMap<String, String>,
}
```

### 7.2 TCP/IP Stack Fingerprinting Tests

```rust
/// OS fingerprinting engine
pub struct OsFingerprinter {
    /// Database of known fingerprints
    fingerprint_db: FingerprintDatabase,
    
    /// Packet transmitter
    transmitter: PacketTransmitter,
    
    /// Packet receiver
    receiver: PacketReceiver,
    
    /// Open port (required for some tests)
    open_port: Option<u16>,
    
    /// Closed port (required for some tests)
    closed_port: Option<u16>,
}

impl OsFingerprinter {
    pub async fn fingerprint(&mut self, target: IpAddr) -> Result<OsMatch> {
        // Find at least one open and one closed port
        self.find_test_ports(target).await?;
        
        // Run fingerprinting tests
        let mut subject = SubjectFingerprint::new(target);
        
        // SEQ - TCP ISN sequence generation
        subject.add_test("SEQ", self.tcp_isn_test(target).await?);
        
        // OPS - TCP options support and ordering
        subject.add_test("OPS", self.tcp_options_test(target).await?);
        
        // WIN - TCP window sizes
        subject.add_test("WIN", self.tcp_window_test(target).await?);
        
        // ECN - Explicit Congestion Notification
        subject.add_test("ECN", self.ecn_test(target).await?);
        
        // T1-T7 - TCP probes to open and closed ports
        for i in 1..=7 {
            let test_name = format!("T{}", i);
            subject.add_test(&test_name, self.tcp_probe_test(target, i).await?);
        }
        
        // IE - ICMP echo
        subject.add_test("IE", self.icmp_echo_test(target).await?);
        
        // U1 - UDP probe to closed port
        subject.add_test("U1", self.udp_probe_test(target).await?);
        
        // Match against database
        self.match_fingerprint(&subject)
    }
    
    /// TCP ISN (Initial Sequence Number) generation test
    async fn tcp_isn_test(&mut self, target: IpAddr) -> Result<String> {
        let mut sequence_numbers = Vec::new();
        let mut timestamps = Vec::new();
        
        // Send 6 SYN probes spaced 100ms apart
        for _ in 0..6 {
            let packet = TcpPacketBuilder::new()
                .source(self.local_ip, rand::random())
                .destination(target, self.open_port.unwrap())
                .sequence(0) // We're interested in their ISN
                .flags(TcpFlags::SYN)
                .tcp_option(TcpOption::Mss(1460))
                .build()?;
            
            let send_time = Instant::now();
            self.transmitter.send(&packet).await?;
            
            // Wait for SYN/ACK
            if let Ok(response) = self.recv_syn_ack(Duration::from_secs(2)).await {
                if let TransportHeader::Tcp(tcp) = &response.transport {
                    sequence_numbers.push(tcp.sequence);
                    timestamps.push(send_time.elapsed());
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Analyze ISN sequence
        self.analyze_isn_sequence(&sequence_numbers, &timestamps)
    }
    
    fn analyze_isn_sequence(&self, sequences: &[u32], timestamps: &[Duration]) -> Result<String> {
        if sequences.len() < 2 {
            return Ok("U".to_string()); // Unknown
        }
        
        // Calculate differences (GCD analysis)
        let diffs: Vec<i64> = sequences.windows(2)
            .map(|w| w[1] as i64 - w[0] as i64)
            .collect();
        
        let gcd = self.calculate_gcd(&diffs);
        
        // Classify based on GCD and variance
        if gcd == 0 {
            Ok("R".to_string()) // Random
        } else if gcd < 9 {
            Ok("RI".to_string()) // Random incremental
        } else if diffs.iter().all(|&d| d > 0 && d < 65536) {
            Ok(format!("I{}", gcd)) // Incremental with rate
        } else {
            // Check for time-based
            if self.is_time_dependent(sequences, timestamps) {
                Ok("T".to_string()) // Time dependent
            } else {
                Ok("U".to_string()) // Unknown pattern
            }
        }
    }
    
    /// TCP options test
    async fn tcp_options_test(&mut self, target: IpAddr) -> Result<String> {
        let packet = TcpPacketBuilder::new()
            .source(self.local_ip, rand::random())
            .destination(target, self.open_port.unwrap())
            .sequence(rand::random())
            .flags(TcpFlags::SYN)
            .tcp_option(TcpOption::Mss(265)) // Unusual MSS to test echo behavior
            .tcp_option(TcpOption::WindowScale(10))
            .tcp_option(TcpOption::SackPermitted)
            .tcp_option(TcpOption::Timestamp { tsval: 0xFFFFFFFF, tsecr: 0 })
            .tcp_option(TcpOption::EndOfOptions)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        if let Ok(response) = self.recv_syn_ack(Duration::from_secs(2)).await {
            if let TransportHeader::Tcp(tcp) = &response.transport {
                // Parse and encode options
                return Ok(self.encode_tcp_options(&tcp.options));
            }
        }
        
        Ok("".to_string())
    }
    
    fn encode_tcp_options(&self, options: &[TcpOption]) -> String {
        // Encode options in Nmap format
        // M = MSS, W = Window Scale, S = SACK permitted
        // T = Timestamp, E = End of Options, N = NOP
        let mut encoded = String::new();
        
        for opt in options {
            match opt {
                TcpOption::Mss(mss) => encoded.push_str(&format!("M{:X}", mss)),
                TcpOption::WindowScale(s) => encoded.push_str(&format!("W{}", s)),
                TcpOption::SackPermitted => encoded.push('S'),
                TcpOption::Timestamp { .. } => encoded.push('T'),
                TcpOption::Nop => encoded.push('N'),
                TcpOption::EndOfOptions => encoded.push('E'),
                _ => {}
            }
        }
        
        encoded
    }
    
    /// TCP window size test
    async fn tcp_window_test(&mut self, target: IpAddr) -> Result<String> {
        let mut windows = Vec::new();
        
        // Send probes with different MSS values
        for mss in [265, 640, 1400] {
            let packet = TcpPacketBuilder::new()
                .source(self.local_ip, rand::random())
                .destination(target, self.open_port.unwrap())
                .sequence(rand::random())
                .flags(TcpFlags::SYN)
                .tcp_option(TcpOption::Mss(mss))
                .build()?;
            
            self.transmitter.send(&packet).await?;
            
            if let Ok(response) = self.recv_syn_ack(Duration::from_secs(1)).await {
                if let TransportHeader::Tcp(tcp) = &response.transport {
                    windows.push(tcp.window);
                }
            }
        }
        
        // Encode window sizes
        Ok(windows.iter()
            .map(|w| format!("{:X}", w))
            .collect::<Vec<_>>()
            .join(","))
    }
    
    /// ECN test
    async fn ecn_test(&mut self, target: IpAddr) -> Result<String> {
        // Send SYN with ECN flags
        let packet = TcpPacketBuilder::new()
            .source(self.local_ip, rand::random())
            .destination(target, self.open_port.unwrap())
            .sequence(rand::random())
            .flags(TcpFlags::SYN | TcpFlags::CWR | TcpFlags::ECE)
            .tcp_option(TcpOption::Mss(1460))
            .tcp_option(TcpOption::WindowScale(7))
            .tcp_option(TcpOption::SackPermitted)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        if let Ok(response) = self.recv_syn_ack(Duration::from_secs(2)).await {
            return Ok(self.analyze_ecn_response(&response));
        }
        
        Ok("N".to_string())
    }
    
    /// T1-T7 TCP probes
    async fn tcp_probe_test(&mut self, target: IpAddr, probe_num: u8) -> Result<String> {
        let (port, flags, options) = match probe_num {
            1 => (self.open_port.unwrap(), TcpFlags::empty(), vec![]),
            2 => (self.open_port.unwrap(), TcpFlags::empty(), vec![
                TcpOption::Mss(1460),
                TcpOption::WindowScale(10),
                TcpOption::SackPermitted,
            ]),
            3 => (self.open_port.unwrap(), TcpFlags::SYN | TcpFlags::FIN | TcpFlags::URG | TcpFlags::PSH, vec![]),
            4 => (self.open_port.unwrap(), TcpFlags::ACK, vec![]),
            5 => (self.closed_port.unwrap(), TcpFlags::SYN, vec![]),
            6 => (self.closed_port.unwrap(), TcpFlags::ACK, vec![]),
            7 => (self.closed_port.unwrap(), TcpFlags::FIN | TcpFlags::PSH | TcpFlags::URG, vec![]),
            _ => return Err(Error::InvalidProbe),
        };
        
        let packet = TcpPacketBuilder::new()
            .source(self.local_ip, rand::random())
            .destination(target, port)
            .sequence(rand::random())
            .flags(flags)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        if let Ok(response) = self.recv_with_timeout(Duration::from_secs(2)).await {
            return Ok(self.encode_probe_response(&response, probe_num));
        }
        
        Ok("R=N".to_string()) // No response
    }
    
    /// ICMP echo test
    async fn icmp_echo_test(&mut self, target: IpAddr) -> Result<String> {
        let packet = IcmpPacketBuilder::new()
            .source(self.local_ip)
            .destination(target)
            .probe_type(IcmpProbeType::EchoRequest)
            .identifier(rand::random())
            .sequence(rand::random())
            .payload(vec![0x00; 120]) // 120 bytes of nulls
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        if let Ok(response) = self.recv_with_timeout(Duration::from_secs(2)).await {
            return Ok(self.encode_icmp_response(&response));
        }
        
        Ok("R=N".to_string())
    }
    
    /// UDP probe to closed port
    async fn udp_probe_test(&mut self, target: IpAddr) -> Result<String> {
        let packet = UdpPacketBuilder::new()
            .source(self.local_ip, rand::random())
            .destination(target, self.closed_port.unwrap())
            .payload(vec![0x43; 300]) // 300 bytes of 'C'
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        if let Ok(response) = self.recv_with_timeout(Duration::from_secs(2)).await {
            return Ok(self.encode_udp_response(&response));
        }
        
        Ok("R=N".to_string())
    }
    
    /// Match fingerprint against database
    fn match_fingerprint(&self, subject: &SubjectFingerprint) -> Result<OsMatch> {
        let mut matches = Vec::new();
        
        for fp in self.fingerprint_db.iter() {
            let score = self.calculate_match_score(subject, fp);
            if score > 0.85 {
                matches.push(OsMatch {
                    os_name: fp.os_name.clone(),
                    accuracy: score,
                    cpe: fp.cpe.clone(),
                });
            }
        }
        
        // Sort by accuracy
        matches.sort_by(|a, b| b.accuracy.partial_cmp(&a.accuracy).unwrap());
        
        if matches.is_empty() {
            Ok(OsMatch {
                os_name: "Unknown".to_string(),
                accuracy: 0.0,
                cpe: None,
            })
        } else {
            Ok(matches[0].clone())
        }
    }
    
    fn calculate_match_score(&self, subject: &SubjectFingerprint, reference: &OsFingerprint) -> f64 {
        let mut total_weight = 0.0;
        let mut matched_weight = 0.0;
        
        for (test_name, test_values) in &reference.tests {
            if let Some(subject_value) = subject.tests.get(test_name) {
                total_weight += test_values.weight;
                
                // Check if subject value matches any expected value
                if test_values.values.iter().any(|v| v == subject_value) {
                    matched_weight += test_values.weight;
                } else {
                    // Partial match scoring
                    let partial = self.partial_match_score(subject_value, &test_values.values);
                    matched_weight += test_values.weight * partial;
                }
            }
        }
        
        if total_weight == 0.0 {
            0.0
        } else {
            matched_weight / total_weight
        }
    }
}
```

### 7.3 Passive OS Fingerprinting

```rust
/// Passive OS fingerprinting (from captured traffic)
pub struct PassiveOsFingerprinter {
    fingerprint_db: PassiveFingerprintDatabase,
    syn_cache: DashMap<IpAddr, SynFingerprint>,
}

#[derive(Debug, Clone)]
pub struct SynFingerprint {
    ttl: u8,
    window_size: u16,
    tcp_options: Vec<TcpOption>,
    mss: Option<u16>,
    window_scale: Option<u8>,
    timestamp: bool,
}

impl PassiveOsFingerprinter {
    pub fn analyze_syn(&self, packet: &PacketData) -> Option<OsMatch> {
        if let (IpHeader::V4(ip), TransportHeader::Tcp(tcp)) = (&packet.ip, &packet.transport) {
            if tcp.flags == TcpFlags::SYN {
                let fingerprint = SynFingerprint {
                    ttl: ip.ttl,
                    window_size: tcp.window,
                    tcp_options: tcp.options.clone(),
                    mss: tcp.options.iter().find_map(|o| {
                        if let TcpOption::Mss(mss) = o { Some(*mss) } else { None }
                    }),
                    window_scale: tcp.options.iter().find_map(|o| {
                        if let TcpOption::WindowScale(ws) = o { Some(*ws) } else { None }
                    }),
                    timestamp: tcp.options.iter().any(|o| {
                        matches!(o, TcpOption::Timestamp { .. })
                    }),
                };
                
                return self.match_passive_fingerprint(&fingerprint);
            }
        }
        
        None
    }
}
```

---

## 8. Service Detection and Banner Grabbing

### 8.1 Service Probe Database

```rust
/// Service detection probe database
#[derive(Debug, Clone)]
pub struct ServiceProbe {
    /// Protocol (TCP/UDP)
    pub protocol: Protocol,
    
    /// Probe name
    pub name: String,
    
    /// Ports this probe applies to
    pub ports: Vec<u16>,
    
    /// SSL/TLS ports
    pub ssl_ports: Vec<u16>,
    
    /// Probe payload to send
    pub probe_string: Vec<u8>,
    
    /// Match patterns
    pub matches: Vec<ServiceMatch>,
    
    /// Softmatch patterns (less specific)
    pub soft_matches: Vec<ServiceMatch>,
    
    /// Rarity (1-9, how often to use this probe)
    pub rarity: u8,
    
    /// Timeout for this specific probe
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct ServiceMatch {
    /// Service name
    pub service: String,
    
    /// Regex pattern to match response
    pub pattern: Regex,
    
    /// Version extraction groups
    pub version_info: Option<VersionInfo>,
    
    /// CPE template
    pub cpe: Option<String>,
    
    /// OS info (if service reveals it)
    pub os_info: Option<String>,
    
    /// Hostname extraction
    pub hostname: Option<String>,
    
    /// Device type
    pub device_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub product: Option<String>,
    pub version: Option<String>,
    pub info: Option<String>,
}
```

### 8.2 Service Detection Engine

```rust
/// Service detection engine
pub struct ServiceDetector {
    /// Probe database
    probe_db: ProbeDatabase,
    
    /// TCP connection pool
    tcp_pool: TcpConnectionPool,
    
    /// UDP socket
    udp_socket: UdpSocket,
    
    /// SSL/TLS context
    tls_connector: TlsConnector,
    
    /// Timeout configuration
    timeouts: TimeoutConfig,
}

impl ServiceDetector {
    pub async fn detect_service(&mut self, target: SocketAddr, port_state: PortState) -> Result<ServiceInfo> {
        // NULL probe first (just connect and read banner)
        if let Some(banner) = self.null_probe(target).await? {
            if let Some(service) = self.match_banner(&banner) {
                return Ok(service);
            }
        }
        
        // Try protocol-specific probes
        let probes = self.probe_db.get_probes_for_port(target.port());
        
        for probe in probes {
            // Check if we should try SSL/TLS
            let use_ssl = probe.ssl_ports.contains(&target.port());
            
            match self.send_probe(target, probe, use_ssl).await {
                Ok(Some(service)) => return Ok(service),
                Ok(None) => continue,
                Err(e) => {
                    tracing::debug!("Probe {} failed: {}", probe.name, e);
                    continue;
                }
            }
        }
        
        Ok(ServiceInfo::unknown())
    }
    
    /// NULL probe - just connect and read initial banner
    async fn null_probe(&mut self, target: SocketAddr) -> Result<Option<Vec<u8>>> {
        let mut stream = tokio::time::timeout(
            self.timeouts.connect,
            TcpStream::connect(target)
        ).await??;
        
        // Set read timeout
        stream.set_read_timeout(Some(self.timeouts.read))?;
        
        // Read banner (up to 4KB)
        let mut buffer = vec![0u8; 4096];
        
        match tokio::time::timeout(
            self.timeouts.read,
            stream.read(&mut buffer)
        ).await {
            Ok(Ok(n)) if n > 0 => {
                buffer.truncate(n);
                Ok(Some(buffer))
            }
            _ => Ok(None),
        }
    }
    
    /// Send a specific probe
    async fn send_probe(
        &mut self,
        target: SocketAddr,
        probe: &ServiceProbe,
        use_ssl: bool,
    ) -> Result<Option<ServiceInfo>> {
        match probe.protocol {
            Protocol::Tcp => self.send_tcp_probe(target, probe, use_ssl).await,
            Protocol::Udp => self.send_udp_probe(target, probe).await,
        }
    }
    
    async fn send_tcp_probe(
        &mut self,
        target: SocketAddr,
        probe: &ServiceProbe,
        use_ssl: bool,
    ) -> Result<Option<ServiceInfo>> {
        if use_ssl {
            self.send_tls_probe(target, probe).await
        } else {
            self.send_plain_tcp_probe(target, probe).await
        }
    }
    
    async fn send_plain_tcp_probe(
        &mut self,
        target: SocketAddr,
        probe: &ServiceProbe,
    ) -> Result<Option<ServiceInfo>> {
        let mut stream = tokio::time::timeout(
            self.timeouts.connect,
            TcpStream::connect(target)
        ).await??;
        
        // Send probe
        stream.write_all(&probe.probe_string).await?;
        stream.flush().await?;
        
        // Read response
        let mut buffer = vec![0u8; 8192];
        let n = tokio::time::timeout(
            probe.timeout.unwrap_or(self.timeouts.read),
            stream.read(&mut buffer)
        ).await??;
        
        buffer.truncate(n);
        
        // Match response
        self.match_probe_response(&buffer, probe)
    }
    
    async fn send_tls_probe(
        &mut self,
        target: SocketAddr,
        probe: &ServiceProbe,
    ) -> Result<Option<ServiceInfo>> {
        let stream = tokio::time::timeout(
            self.timeouts.connect,
            TcpStream::connect(target)
        ).await??;
        
        // Wrap in TLS
        let mut tls_stream = tokio::time::timeout(
            self.timeouts.ssl_handshake,
            self.tls_connector.connect(target.ip().to_string(), stream)
        ).await??;
        
        // Send probe over TLS
        tls_stream.write_all(&probe.probe_string).await?;
        tls_stream.flush().await?;
        
        // Read response
        let mut buffer = vec![0u8; 8192];
        let n = tokio::time::timeout(
            probe.timeout.unwrap_or(self.timeouts.read),
            tls_stream.read(&mut buffer)
        ).await??;
        
        buffer.truncate(n);
        
        // Extract TLS info
        let tls_info = self.extract_tls_info(&tls_stream);
        
        // Match response
        let mut service_info = self.match_probe_response(&buffer, probe)?;
        
        if let Some(ref mut info) = service_info {
            info.tls_info = tls_info;
        }
        
        Ok(service_info)
    }
    
    async fn send_udp_probe(
        &mut self,
        target: SocketAddr,
        probe: &ServiceProbe,
    ) -> Result<Option<ServiceInfo>> {
        // Send UDP probe
        self.udp_socket.send_to(&probe.probe_string, target).await?;
        
        // Wait for response
        let mut buffer = vec![0u8; 8192];
        
        match tokio::time::timeout(
            probe.timeout.unwrap_or(self.timeouts.udp),
            self.udp_socket.recv_from(&mut buffer)
        ).await {
            Ok(Ok((n, _addr))) => {
                buffer.truncate(n);
                self.match_probe_response(&buffer, probe)
            }
            _ => Ok(None),
        }
    }
    
    /// Match probe response against patterns
    fn match_probe_response(
        &self,
        response: &[u8],
        probe: &ServiceProbe,
    ) -> Result<Option<ServiceInfo>> {
        // Try hard matches first
        for pattern in &probe.matches {
            if let Some(service_info) = self.try_match(response, pattern) {
                return Ok(Some(service_info));
            }
        }
        
        // Try soft matches
        for pattern in &probe.soft_matches {
            if let Some(service_info) = self.try_match(response, pattern) {
                return Ok(Some(service_info));
            }
        }
        
        Ok(None)
    }
    
    fn try_match(&self, response: &[u8], pattern: &ServiceMatch) -> Option<ServiceInfo> {
        // Convert response to string (lossy)
        let response_str = String::from_utf8_lossy(response);
        
        if let Some(captures) = pattern.pattern.captures(&response_str) {
            let mut service_info = ServiceInfo {
                service: pattern.service.clone(),
                product: None,
                version: None,
                extra_info: None,
                hostname: None,
                os_info: pattern.os_info.clone(),
                device_type: pattern.device_type.clone(),
                cpe: pattern.cpe.clone(),
                tls_info: None,
            };
            
            // Extract version information
            if let Some(ref version_info) = pattern.version_info {
                if let Some(ref product_template) = version_info.product {
                    service_info.product = Some(self.apply_template(product_template, &captures));
                }
                if let Some(ref version_template) = version_info.version {
                    service_info.version = Some(self.apply_template(version_template, &captures));
                }
                if let Some(ref info_template) = version_info.info {
                    service_info.extra_info = Some(self.apply_template(info_template, &captures));
                }
            }
            
            // Extract hostname if pattern specifies
            if let Some(ref hostname_template) = pattern.hostname {
                service_info.hostname = Some(self.apply_template(hostname_template, &captures));
            }
            
            return Some(service_info);
        }
        
        None
    }
    
    /// Apply template with regex capture groups
    fn apply_template(&self, template: &str, captures: &Captures) -> String {
        let mut result = template.to_string();
        
        // Replace $1, $2, etc. with captured groups
        for i in 0..captures.len() {
            if let Some(capture) = captures.get(i) {
                result = result.replace(&format!("${}", i), capture.as_str());
            }
        }
        
        result
    }
    
    fn extract_tls_info(&self, stream: &TlsStream<TcpStream>) -> Option<TlsInfo> {
        // Extract certificate info, cipher suite, protocol version
        // Implementation depends on TLS library used
        None
    }
}
```

### 8.3 Common Service Probes

```rust
impl ProbeDatabase {
    pub fn load_default_probes() -> Self {
        let mut probes = Vec::new();
        
        // NULL probe
        probes.push(ServiceProbe {
            protocol: Protocol::Tcp,
            name: "NULL".to_string(),
            ports: vec![], // All ports
            ssl_ports: vec![443, 465, 993, 995],
            probe_string: vec![],
            matches: vec![
                // SSH banner
                ServiceMatch {
                    service: "ssh".to_string(),
                    pattern: Regex::new(r"^SSH-([0-9.]+)-(.+)").unwrap(),
                    version_info: Some(VersionInfo {
                        product: Some("$2".to_string()),
                        version: Some("$1".to_string()),
                        info: None,
                    }),
                    ..Default::default()
                },
                // HTTP banner
                ServiceMatch {
                    service: "http".to_string(),
                    pattern: Regex::new(r"HTTP/1\.[01] \d+").unwrap(),
                    ..Default::default()
                },
                // FTP banner
                ServiceMatch {
                    service: "ftp".to_string(),
                    pattern: Regex::new(r"^220[- ]").unwrap(),
                    ..Default::default()
                },
                // SMTP banner
                ServiceMatch {
                    service: "smtp".to_string(),
                    pattern: Regex::new(r"^220[- ].*SMTP").unwrap(),
                    ..Default::default()
                },
                // POP3 banner
                ServiceMatch {
                    service: "pop3".to_string(),
                    pattern: Regex::new(r"^\+OK").unwrap(),
                    ..Default::default()
                },
                // IMAP banner
                ServiceMatch {
                    service: "imap".to_string(),
                    pattern: Regex::new(r"^\* OK.*IMAP").unwrap(),
                    ..Default::default()
                },
            ],
            soft_matches: vec![],
            rarity: 1,
            timeout: Some(Duration::from_secs(5)),
        });
        
        // HTTP GET request
        probes.push(ServiceProbe {
            protocol: Protocol::Tcp,
            name: "GetRequest".to_string(),
            ports: vec![80, 8080, 8000, 8888],
            ssl_ports: vec![443, 8443],
            probe_string: b"GET / HTTP/1.0\r\n\r\n".to_vec(),
            matches: vec![
                ServiceMatch {
                    service: "http".to_string(),
                    pattern: Regex::new(r"HTTP/1\.[01] \d+ .+\r\nServer: (.+)").unwrap(),
                    version_info: Some(VersionInfo {
                        product: Some("$1".to_string()),
                        version: None,
                        info: None,
                    }),
                    ..Default::default()
                },
            ],
            soft_matches: vec![],
            rarity: 1,
            timeout: Some(Duration::from_secs(7)),
        });
        
        // DNS Status Request
        probes.push(ServiceProbe {
            protocol: Protocol::Udp,
            name: "DNSStatusRequest".to_string(),
            ports: vec![53],
            ssl_ports: vec![],
            probe_string: vec![
                0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
            matches: vec![
                ServiceMatch {
                    service: "domain".to_string(),
                    pattern: Regex::new(r"^.{2}\x90\x04").unwrap(),
                    ..Default::default()
                },
            ],
            soft_matches: vec![],
            rarity: 1,
            timeout: Some(Duration::from_secs(5)),
        });
        
        // SNMP GET Request
        probes.push(ServiceProbe {
            protocol: Protocol::Udp,
            name: "SNMPv1public".to_string(),
            ports: vec![161],
            ssl_ports: vec![],
            probe_string: vec![
                0x30, 0x26, 0x02, 0x01, 0x01, // SNMP version 1
                0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, // Community: public
                0xa0, 0x19, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00,
                0x02, 0x01, 0x00, 0x30, 0x0e, 0x30, 0x0c, 0x06,
                0x08, 0x2b, 0x06, 0x01, 0x02, 0x01, 0x01, 0x01, 0x00,
                0x05, 0x00,
            ],
            matches: vec![
                ServiceMatch {
                    service: "snmp".to_string(),
                    pattern: Regex::new(r"^\x30").unwrap(),
                    ..Default::default()
                },
            ],
            soft_matches: vec![],
            rarity: 1,
            timeout: Some(Duration::from_secs(5)),
        });
        
        // Add more probes for: NTP, NetBIOS, DHCP, SIP, RTSP, etc.
        
        Self { probes }
    }
}
```

### 8.4 Banner Extraction and Analysis

```rust
/// Banner grabber for simple text protocols
pub struct BannerGrabber {
    timeout: Duration,
}

impl BannerGrabber {
    pub async fn grab_banner(&self, target: SocketAddr) -> Result<Banner> {
        let mut stream = tokio::time::timeout(
            self.timeout,
            TcpStream::connect(target)
        ).await??;
        
        // Read initial banner
        let mut buffer = vec![0u8; 4096];
        let n = tokio::time::timeout(
            self.timeout,
            stream.read(&mut buffer)
        ).await??;
        
        buffer.truncate(n);
        
        Ok(Banner {
            data: buffer,
            sanitized: self.sanitize_banner(&buffer),
        })
    }
    
    fn sanitize_banner(&self, data: &[u8]) -> String {
        // Convert to string, replace non-printable characters
        data.iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b.is_ascii_whitespace() {
                    b as char
                } else {
                    '.'
                }
            })
            .collect()
    }
}

/// HTTP-specific banner grabbing
pub async fn grab_http_banner(target: SocketAddr, use_https: bool) -> Result<HttpBanner> {
    let request = format!(
        "GET / HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: Mozilla/5.0\r\n\
         Accept: */*\r\n\
         Connection: close\r\n\
         \r\n",
        target.ip()
    );
    
    let stream = TcpStream::connect(target).await?;
    
    let mut stream: Box<dyn AsyncReadExt + AsyncWriteExt + Unpin> = if use_https {
        Box::new(/* TLS wrap */)
    } else {
        Box::new(stream)
    };
    
    stream.write_all(request.as_bytes()).await?;
    
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    
    parse_http_response(&response)
}

fn parse_http_response(data: &[u8]) -> Result<HttpBanner> {
    let response_str = String::from_utf8_lossy(data);
    
    // Parse status line
    let mut lines = response_str.lines();
    let status_line = lines.next().ok_or(Error::InvalidHttp)?;
    
    // Parse headers
    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break; // End of headers
        }
        
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }
    
    Ok(HttpBanner {
        status_line: status_line.to_string(),
        server: headers.get("server").cloned(),
        headers,
    })
}
```

---

## 9. Stealth and Evasion Techniques

### 9.1 Timing Controls

```rust
/// Timing template (Nmap-style T0-T5)
#[derive(Debug, Clone, Copy)]
pub enum TimingTemplate {
    Paranoid,    // T0: Very slow, 5 minutes between probes
    Sneaky,      // T1: 15 seconds between probes
    Polite,      // T2: 0.4 seconds between probes
    Normal,      // T3: Default, parallel scanning
    Aggressive,  // T4: Fast, assumes fast/reliable network
    Insane,      // T5: Maximum speed, may sacrifice accuracy
}

impl TimingTemplate {
    pub fn to_config(&self) -> TimingConfig {
        match self {
            TimingTemplate::Paranoid => TimingConfig {
                min_rtt_timeout: Duration::from_secs(100),
                max_rtt_timeout: Duration::from_secs(300),
                initial_rtt_timeout: Duration::from_secs(300),
                scan_delay: Duration::from_secs(300),
                max_parallelism: 1,
                max_retries: 1,
                host_timeout: Duration::from_secs(900),
            },
            TimingTemplate::Sneaky => TimingConfig {
                min_rtt_timeout: Duration::from_secs(2),
                max_rtt_timeout: Duration::from_secs(10),
                initial_rtt_timeout: Duration::from_secs(5),
                scan_delay: Duration::from_secs(15),
                max_parallelism: 1,
                max_retries: 2,
                host_timeout: Duration::from_secs(900),
            },
            TimingTemplate::Polite => TimingConfig {
                min_rtt_timeout: Duration::from_millis(400),
                max_rtt_timeout: Duration::from_secs(10),
                initial_rtt_timeout: Duration::from_secs(1),
                scan_delay: Duration::from_millis(400),
                max_parallelism: 1,
                max_retries: 2,
                host_timeout: Duration::from_secs(900),
            },
            TimingTemplate::Normal => TimingConfig {
                min_rtt_timeout: Duration::from_millis(100),
                max_rtt_timeout: Duration::from_secs(10),
                initial_rtt_timeout: Duration::from_secs(1),
                scan_delay: Duration::from_millis(0),
                max_parallelism: 100,
                max_retries: 3,
                host_timeout: Duration::from_secs(900),
            },
            TimingTemplate::Aggressive => TimingConfig {
                min_rtt_timeout: Duration::from_millis(50),
                max_rtt_timeout: Duration::from_secs(1),
                initial_rtt_timeout: Duration::from_millis(250),
                scan_delay: Duration::from_millis(0),
                max_parallelism: 1000,
                max_retries: 2,
                host_timeout: Duration::from_secs(300),
            },
            TimingTemplate::Insane => TimingConfig {
                min_rtt_timeout: Duration::from_millis(5),
                max_rtt_timeout: Duration::from_millis(75),
                initial_rtt_timeout: Duration::from_millis(20),
                scan_delay: Duration::from_millis(0),
                max_parallelism: 5000,
                max_retries: 1,
                host_timeout: Duration::from_secs(75),
            },
        }
    }
}

/// Timing configuration
#[derive(Debug, Clone)]
pub struct TimingConfig {
    pub min_rtt_timeout: Duration,
    pub max_rtt_timeout: Duration,
    pub initial_rtt_timeout: Duration,
    pub scan_delay: Duration,
    pub max_parallelism: usize,
    pub max_retries: u32,
    pub host_timeout: Duration,
}

/// Jitter/dithering for timing randomization
pub struct TimingJitter {
    jitter_percent: f64,
    rng: ChaCha8Rng,
}

impl TimingJitter {
    pub fn new(jitter_percent: f64) -> Self {
        Self {
            jitter_percent,
            rng: ChaCha8Rng::from_entropy(),
        }
    }
    
    pub fn apply_jitter(&mut self, base_delay: Duration) -> Duration {
        let jitter_range = base_delay.as_millis() as f64 * self.jitter_percent;
        let jitter = self.rng.gen_range(-jitter_range..=jitter_range);
        
        let adjusted = base_delay.as_millis() as f64 + jitter;
        Duration::from_millis(adjusted.max(0.0) as u64)
    }
}
```

### 9.2 Packet Fragmentation

```rust
/// IP packet fragmenter
pub struct PacketFragmenter {
    fragment_size: usize,
    fragment_offset_step: u16,
}

impl PacketFragmenter {
    pub fn new(mtu: usize) -> Self {
        Self {
            fragment_size: mtu - 20, // Subtract IP header
            fragment_offset_step: 8, // Must be multiple of 8
        }
    }
    
    /// Fragment a packet into multiple IP fragments
    pub fn fragment_packet(&self, original: &[u8]) -> Result<Vec<Vec<u8>>> {
        if original.len() <= self.fragment_size {
            return Ok(vec![original.to_vec()]);
        }
        
        let mut fragments = Vec::new();
        let ip_packet = Ipv4Packet::new(original)
            .ok_or(Error::InvalidPacket)?;
        
        let payload = ip_packet.payload();
        let mut offset = 0;
        let id = rand::random::<u16>();
        
        while offset < payload.len() {
            let chunk_size = std::cmp::min(
                self.fragment_size,
                payload.len() - offset
            );
            
            let chunk = &payload[offset..offset + chunk_size];
            let more_fragments = offset + chunk_size < payload.len();
            
            // Build fragment
            let fragment = self.build_fragment(
                &ip_packet,
                id,
                chunk,
                offset,
                more_fragments,
            )?;
            
            fragments.push(fragment);
            offset += chunk_size;
        }
        
        Ok(fragments)
    }
    
    fn build_fragment(
        &self,
        original: &Ipv4Packet,
        id: u16,
        payload: &[u8],
        offset: usize,
        more_fragments: bool,
    ) -> Result<Vec<u8>> {
        let total_len = 20 + payload.len();
        let mut buffer = vec![0u8; total_len];
        
        let mut frag_packet = MutableIpv4Packet::new(&mut buffer)
            .ok_or(Error::PacketTooSmall)?;
        
        // Copy IP header fields
        frag_packet.set_version(4);
        frag_packet.set_header_length(5);
        frag_packet.set_dscp(original.get_dscp());
        frag_packet.set_ecn(original.get_ecn());
        frag_packet.set_total_length(total_len as u16);
        frag_packet.set_identification(id);
        
        // Set fragment flags and offset
        let mut flags = Ipv4Flags::empty();
        if more_fragments {
            flags |= Ipv4Flags::MoreFragments;
        }
        if original.get_flags() & Ipv4Flags::DontFragment != 0 {
            // Preserve DF flag (though fragmenting with DF set is unusual)
        }
        frag_packet.set_flags(flags);
        frag_packet.set_fragment_offset((offset / 8) as u16);
        
        frag_packet.set_ttl(original.get_ttl());
        frag_packet.set_next_level_protocol(original.get_next_level_protocol());
        frag_packet.set_source(original.get_source());
        frag_packet.set_destination(original.get_destination());
        
        // Copy payload
        frag_packet.payload_mut().copy_from_slice(payload);
        
        // Calculate checksum
        let checksum = checksum(&frag_packet.to_immutable());
        frag_packet.set_checksum(checksum);
        
        Ok(buffer)
    }
}

/// Tiny fragment attack (overlapping fragments)
pub struct TinyFragmenter {
    first_fragment_size: usize,
}

impl TinyFragmenter {
    pub fn fragment_tcp_packet(&self, packet: &[u8]) -> Result<Vec<Vec<u8>>> {
        // Split TCP header across multiple fragments
        // First fragment: IP header + first 8 bytes of TCP header
        // Second fragment: Rest of TCP header + payload
        
        // This can evade some simple packet filters that only
        // inspect the first fragment
        
        todo!("Implement tiny fragmentation")
    }
}
```

### 9.3 Decoy Scanning

```rust
/// Decoy scanner to mask real source
pub struct DecoyScanner {
    real_source: IpAddr,
    decoys: Vec<IpAddr>,
    randomize_order: bool,
}

impl DecoyScanner {
    pub fn new(real_source: IpAddr) -> Self {
        Self {
            real_source,
            decoys: Vec::new(),
            randomize_order: true,
        }
    }
    
    pub fn add_decoy(&mut self, decoy: IpAddr) {
        self.decoys.push(decoy);
    }
    
    pub fn generate_random_decoys(&mut self, count: usize) {
        for _ in 0..count {
            // Generate random IP (avoiding reserved ranges)
            let ip = self.generate_random_public_ip();
            self.decoys.push(ip);
        }
    }
    
    pub async fn scan_with_decoys(
        &mut self,
        target: SocketAddr,
        scan_type: ScanType,
    ) -> Result<PortState> {
        let mut sources = self.decoys.clone();
        sources.push(self.real_source);
        
        if self.randomize_order {
            sources.shuffle(&mut rand::thread_rng());
        }
        
        // Send probe from each source
        for source in sources {
            let packet = self.build_probe(source, target, scan_type)?;
            self.transmitter.send(&packet).await?;
            
            // Small delay between decoy probes
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Only process response to our real probe
        self.wait_for_real_response(target).await
    }
    
    fn generate_random_public_ip(&self) -> Ipv4Addr {
        loop {
            let ip = Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            );
            
            // Skip reserved/private ranges
            if !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && !ip.is_documentation()
            {
                return ip;
            }
        }
    }
}
```

### 9.4 Idle (Zombie) Scanning

```rust
/// Idle scan implementation
pub struct IdleScanner {
    zombie: IpAddr,
    real_source: IpAddr,
    transmitter: PacketTransmitter,
    receiver: PacketReceiver,
}

impl IdleScanner {
    pub async fn scan_port(&mut self, target: SocketAddr) -> Result<PortState> {
        // 1. Probe zombie to get baseline IP ID
        let baseline_id = self.probe_zombie_ipid().await?;
        
        // 2. Spoof SYN to target from zombie
        self.send_spoofed_syn(target).await?;
        
        // Small delay for target to respond
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 3. Probe zombie again to check IP ID increment
        let new_id = self.probe_zombie_ipid().await?;
        
        // 4. Analyze IP ID difference
        let diff = new_id.wrapping_sub(baseline_id);
        
        match diff {
            0 | 1 => {
                // No increment or single increment from our probe
                // = Target port closed (sent RST to zombie, zombie didn't respond)
                Ok(PortState::Closed)
            }
            2 => {
                // Two increments: our probe + zombie's SYN/ACK to target
                // = Target port open (sent SYN/ACK to zombie)
                Ok(PortState::Open)
            }
            _ => {
                // Unexpected increment - zombie may not be good candidate
                Ok(PortState::Unknown)
            }
        }
    }
    
    /// Probe zombie's IP ID
    async fn probe_zombie_ipid(&mut self) -> Result<u16> {
        // Send SYN/ACK to zombie (zombie will respond with RST)
        let packet = TcpPacketBuilder::new()
            .source(self.real_source, rand::random())
            .destination(self.zombie, rand::random::<u16>())
            .sequence(rand::random())
            .acknowledgment(rand::random())
            .flags(TcpFlags::SYN | TcpFlags::ACK)
            .build()?;
        
        self.transmitter.send(&packet).await?;
        
        // Wait for RST response
        let response = self.recv_with_timeout(Duration::from_secs(2)).await?;
        
        if let IpHeader::V4(ip) = &response.ip {
            Ok(ip.identification)
        } else {
            Err(Error::NoResponse)
        }
    }
    
    /// Send spoofed SYN from zombie to target
    async fn send_spoofed_syn(&mut self, target: SocketAddr) -> Result<()> {
        let packet = TcpPacketBuilder::new()
            .source(self.zombie, rand::random())
            .destination(target.ip(), target.port())
            .sequence(rand::random())
            .flags(TcpFlags::SYN)
            .build()?;
        
        self.transmitter.send(&packet).await
    }
    
    /// Find suitable zombie host
    pub async fn find_zombie_candidates(&mut self, network: IpNetwork) -> Vec<IpAddr> {
        let mut candidates = Vec::new();
        
        for ip in network.iter() {
            if self.is_good_zombie(ip).await {
                candidates.push(ip);
            }
        }
        
        candidates
    }
    
    async fn is_good_zombie(&mut self, ip: IpAddr) -> bool {
        // Good zombie characteristics:
        // 1. Uses incremental IP ID
        // 2. Idle (not generating much traffic)
        // 3. Responsive to our probes
        
        let mut ids = Vec::new();
        
        // Probe multiple times
        for _ in 0..5 {
            if let Ok(id) = self.probe_specific_zombie_ipid(ip).await {
                ids.push(id);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        if ids.len() < 5 {
            return false; // Not responsive enough
        }
        
        // Check if incremental
        let diffs: Vec<u16> = ids.windows(2)
            .map(|w| w[1].wrapping_sub(w[0]))
            .collect();
        
        // Should have small, consistent increments
        diffs.iter().all(|&d| d > 0 && d < 10)
    }
}
```

### 9.5 Source Port Manipulation

```rust
/// Source port manipulation for firewall evasion
pub struct SourcePortManipulator {
    strategy: SourcePortStrategy,
}

#[derive(Debug, Clone)]
pub enum SourcePortStrategy {
    /// Use specific source port (e.g., 53 for DNS, 20 for FTP-DATA)
    Fixed(u16),
    
    /// Randomize source port
    Random,
    
    /// Use common trusted ports (53, 80, 443)
    TrustedPorts(Vec<u16>),
}

impl SourcePortManipulator {
    pub fn select_source_port(&self) -> u16 {
        match &self.strategy {
            SourcePortStrategy::Fixed(port) => *port,
            SourcePortStrategy::Random => rand::random::<u16>() | 1024,
            SourcePortStrategy::TrustedPorts(ports) => {
                let idx = rand::random::<usize>() % ports.len();
                ports[idx]
            }
        }
    }
}
```

### 9.6 MAC Address Spoofing

```rust
/// MAC address spoofer for Ethernet layer
pub struct MacSpoofer {
    real_mac: MacAddr,
    spoofed_mac: Option<MacAddr>,
}

impl MacSpoofer {
    pub fn spoof_mac(&mut self, mac: MacAddr) {
        self.spoofed_mac = Some(mac);
    }
    
    pub fn generate_random_mac(&mut self) {
        // Generate random MAC (set locally-administered bit)
        let mut bytes = [0u8; 6];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes[0] |= 0x02; // Set locally-administered bit
        bytes[0] &= 0xFE; // Clear multicast bit
        
        self.spoofed_mac = Some(MacAddr::new(
            bytes[0], bytes[1], bytes[2],
            bytes[3], bytes[4], bytes[5],
        ));
    }
    
    pub fn get_source_mac(&self) -> MacAddr {
        self.spoofed_mac.unwrap_or(self.real_mac)
    }
}
```

### 9.7 TTL Manipulation

```rust
/// TTL manipulation for stealth and traceroute evasion
pub struct TtlManipulator {
    strategy: TtlStrategy,
}

#[derive(Debug, Clone)]
pub enum TtlStrategy {
    /// Use specific TTL
    Fixed(u8),
    
    /// Randomize TTL in range
    Random(u8, u8),
    
    /// Mimic specific OS TTL defaults
    MimicOs(OsType),
}

#[derive(Debug, Clone, Copy)]
pub enum OsType {
    Linux,   // TTL 64
    Windows, // TTL 128
    Cisco,   // TTL 255
}

impl TtlManipulator {
    pub fn select_ttl(&self) -> u8 {
        match &self.strategy {
            TtlStrategy::Fixed(ttl) => *ttl,
            TtlStrategy::Random(min, max) => {
                rand::thread_rng().gen_range(*min..=*max)
            }
            TtlStrategy::MimicOs(os_type) => match os_type {
                OsType::Linux => 64,
                OsType::Windows => 128,
                OsType::Cisco => 255,
            },
        }
    }
}
```

---

## 10. Performance and Optimization Strategy

### 10.1 Rate Limiting and Congestion Control

```rust
/// Adaptive rate limiter inspired by TCP congestion control
pub struct AdaptiveRateLimiter {
    /// Current transmission rate (packets per second)
    current_rate: Arc<AtomicU64>,
    
    /// Maximum allowed rate
    max_rate: u64,
    
    /// Congestion window (number of outstanding packets)
    cwnd: Arc<AtomicUsize>,
    
    /// Slow start threshold
    ssthresh: Arc<AtomicUsize>,
    
    /// RTT estimator
    rtt_estimator: Arc<Mutex<RttEstimator>>,
    
    /// Packet loss detector
    loss_detector: Arc<Mutex<LossDetector>>,
    
    /// State: SlowStart, CongestionAvoidance, FastRecovery
    state: Arc<Mutex<CongestionState>>,
}

#[derive(Debug, Clone, Copy)]
enum CongestionState {
    SlowStart,
    CongestionAvoidance,
    FastRecovery,
}

impl AdaptiveRateLimiter {
    pub fn new(initial_rate: u64, max_rate: u64) -> Self {
        Self {
            current_rate: Arc::new(AtomicU64::new(initial_rate)),
            max_rate,
            cwnd: Arc::new(AtomicUsize::new(1)),
            ssthresh: Arc::new(AtomicUsize::new(usize::MAX)),
            rtt_estimator: Arc::new(Mutex::new(RttEstimator::new())),
            loss_detector: Arc::new(Mutex::new(LossDetector::new())),
            state: Arc::new(Mutex::new(CongestionState::SlowStart)),
        }
    }
    
    pub async fn acquire_permit(&self) -> RateLimitPermit {
        let cwnd = self.cwnd.load(Ordering::Relaxed);
        
        // Calculate delay based on current rate
        let rate = self.current_rate.load(Ordering::Relaxed);
        if rate > 0 {
            let delay = Duration::from_micros(1_000_000 / rate);
            tokio::time::sleep(delay).await;
        }
        
        RateLimitPermit {
            limiter: self.clone(),
            sent_at: Instant::now(),
        }
    }
    
    pub async fn on_ack(&self, rtt: Duration) {
        let mut rtt_est = self.rtt_estimator.lock().await;
        rtt_est.update(rtt);
        
        let state = *self.state.lock().await;
        let cwnd = self.cwnd.load(Ordering::Relaxed);
        let ssthresh = self.ssthresh.load(Ordering::Relaxed);
        
        match state {
            CongestionState::SlowStart => {
                // Exponential increase
                self.cwnd.fetch_add(1, Ordering::Relaxed);
                
                if cwnd >= ssthresh {
                    *self.state.lock().await = CongestionState::CongestionAvoidance;
                }
            }
            CongestionState::CongestionAvoidance => {
                // Additive increase: cwnd += 1/cwnd
                if rand::random::<f64>() < 1.0 / cwnd as f64 {
                    self.cwnd.fetch_add(1, Ordering::Relaxed);
                }
            }
            CongestionState::FastRecovery => {
                // Inflate cwnd for each duplicate ACK
                self.cwnd.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // Update rate based on cwnd and RTT
        self.update_rate().await;
    }
    
    pub async fn on_timeout(&self) {
        // Multiplicative decrease
        let cwnd = self.cwnd.load(Ordering::Relaxed);
        let new_ssthresh = std::cmp::max(cwnd / 2, 2);
        
        self.ssthresh.store(new_ssthresh, Ordering::Relaxed);
        self.cwnd.store(1, Ordering::Relaxed);
        *self.state.lock().await = CongestionState::SlowStart;
        
        self.update_rate().await;
    }
    
    pub async fn on_packet_loss(&self) {
        let mut loss_det = self.loss_detector.lock().await;
        loss_det.record_loss();
        
        let loss_rate = loss_det.estimate_loss_rate();
        
        if loss_rate > 0.05 {
            // Significant packet loss - reduce rate aggressively
            self.on_timeout().await;
        }
    }
    
    async fn update_rate(&self) {
        let cwnd = self.cwnd.load(Ordering::Relaxed);
        let rtt_est = self.rtt_estimator.lock().await;
        let avg_rtt = rtt_est.smoothed_rtt();
        
        if avg_rtt.as_secs_f64() > 0.0 {
            let rate = (cwnd as f64 / avg_rtt.as_secs_f64()) as u64;
            let capped_rate = std::cmp::min(rate, self.max_rate);
            self.current_rate.store(capped_rate, Ordering::Relaxed);
        }
    }
}

/// RTT estimator using exponentially-weighted moving average
pub struct RttEstimator {
    srtt: Option<Duration>,  // Smoothed RTT
    rttvar: Duration,        // RTT variation
    alpha: f64,              // Smoothing factor for SRTT (0.125)
    beta: f64,               // Smoothing factor for RTTVAR (0.25)
}

impl RttEstimator {
    pub fn new() -> Self {
        Self {
            srtt: None,
            rttvar: Duration::from_millis(100),
            alpha: 0.125,
            beta: 0.25,
        }
    }
    
    pub fn update(&mut self, sample: Duration) {
        match self.srtt {
            None => {
                // First measurement
                self.srtt = Some(sample);
                self.rttvar = sample / 2;
            }
            Some(srtt) => {
                // RFC 6298 algorithm
                let diff = if sample > srtt {
                    sample - srtt
                } else {
                    srtt - sample
                };
                
                self.rttvar = Duration::from_secs_f64(
                    (1.0 - self.beta) * self.rttvar.as_secs_f64()
                        + self.beta * diff.as_secs_f64()
                );
                
                self.srtt = Some(Duration::from_secs_f64(
                    (1.0 - self.alpha) * srtt.as_secs_f64()
                        + self.alpha * sample.as_secs_f64()
                ));
            }
        }
    }
    
    pub fn smoothed_rtt(&self) -> Duration {
        self.srtt.unwrap_or(Duration::from_millis(100))
    }
    
    pub fn rto(&self) -> Duration {
        // Retransmission timeout = SRTT + 4 * RTTVAR
        let srtt = self.smoothed_rtt();
        srtt + self.rttvar * 4
    }
}

/// Packet loss detector
pub struct LossDetector {
    sent_packets: VecDeque<Instant>,
    lost_packets: usize,
    window_size: usize,
}

impl LossDetector {
    pub fn new() -> Self {
        Self {
            sent_packets: VecDeque::new(),
            lost_packets: 0,
            window_size: 100,
        }
    }
    
    pub fn record_loss(&mut self) {
        self.lost_packets += 1;
        
        // Keep window size constant
        if self.sent_packets.len() >= self.window_size {
            self.sent_packets.pop_front();
        }
    }
    
    pub fn estimate_loss_rate(&self) -> f64 {
        if self.sent_packets.is_empty() {
            0.0
        } else {
            self.lost_packets as f64 / self.sent_packets.len() as f64
        }
    }
}
```

### 10.2 Zero-Copy Optimizations

```rust
/// Zero-copy buffer management
pub struct ZeroCopyBuffer {
    backing_store: Vec<u8>,
    regions: Vec<BufferRegion>,
}

#[derive(Debug, Clone)]
struct BufferRegion {
    offset: usize,
    length: usize,
    in_use: bool,
}

impl ZeroCopyBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            backing_store: vec![0u8; size],
            regions: vec![BufferRegion {
                offset: 0,
                length: size,
                in_use: false,
            }],
        }
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<&mut [u8]> {
        // Find free region
        for region in &mut self.regions {
            if !region.in_use && region.length >= size {
                region.in_use = true;
                let slice = &mut self.backing_store[region.offset..region.offset + size];
                return Some(slice);
            }
        }
        None
    }
    
    pub fn release(&mut self, slice: &[u8]) {
        let ptr = slice.as_ptr() as usize;
        let base_ptr = self.backing_store.as_ptr() as usize;
        let offset = ptr - base_ptr;
        
        for region in &mut self.regions {
            if region.offset == offset {
                region.in_use = false;
                break;
            }
        }
        
        // Merge adjacent free regions
        self.coalesce_regions();
    }
    
    fn coalesce_regions(&mut self) {
        let mut i = 0;
        while i < self.regions.len() - 1 {
            if !self.regions[i].in_use && !self.regions[i + 1].in_use {
                self.regions[i].length += self.regions[i + 1].length;
                self.regions.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}
```

### 10.3 Batch Processing

```rust
/// Batch processor for amortizing overhead
pub struct BatchProcessor<T> {
    batch_size: usize,
    batch: Vec<T>,
    processor: Box<dyn Fn(Vec<T>) + Send>,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize, processor: Box<dyn Fn(Vec<T>) + Send>) -> Self {
        Self {
            batch_size,
            batch: Vec::with_capacity(batch_size),
            processor,
        }
    }
    
    pub fn add(&mut self, item: T) {
        self.batch.push(item);
        
        if self.batch.len() >= self.batch_size {
            self.flush();
        }
    }
    
    pub fn flush(&mut self) {
        if !self.batch.is_empty() {
            let batch = std::mem::replace(
                &mut self.batch,
                Vec::with_capacity(self.batch_size)
            );
            (self.processor)(batch);
        }
    }
}
```

### 10.4 Lock-Free Data Structures

```rust
use crossbeam::queue::SegQueue;
use crossbeam::channel::{bounded, Sender, Receiver};

/// Lock-free work queue
pub struct LockFreeWorkQueue<T> {
    queue: Arc<SegQueue<T>>,
    workers: usize,
}

impl<T> LockFreeWorkQueue<T> {
    pub fn new(workers: usize) -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
            workers,
        }
    }
    
    pub fn push(&self, item: T) {
        self.queue.push(item);
    }
    
    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Lock-free result collector
pub struct LockFreeResultCollector<T> {
    results: Arc<SegQueue<T>>,
}

impl<T> LockFreeResultCollector<T> {
    pub fn new() -> Self {
        Self {
            results: Arc::new(SegQueue::new()),
        }
    }
    
    pub fn collect(&self, result: T) {
        self.results.push(result);
    }
    
    pub fn drain(&self) -> Vec<T> {
        let mut collected = Vec::new();
        while let Some(result) = self.results.pop() {
            collected.push(result);
        }
        collected
    }
}
```

---

## 11. Output and Reporting Systems

### 11.1 Output Formats

```rust
/// Output format specification
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,        // Human-readable text
    Json,        // JSON
    JsonLines,   // JSONL (one object per line)
    Xml,         // XML (Nmap-compatible)
    Csv,         // CSV
    Grepable,    // Greppable format
    Binary,      // Binary format (custom)
}

/// Output manager
pub struct OutputManager {
    formats: Vec<OutputFormat>,
    outputs: HashMap<OutputFormat, Box<dyn OutputWriter>>,
}

impl OutputManager {
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
            outputs: HashMap::new(),
        }
    }
    
    pub fn add_output(&mut self, format: OutputFormat, writer: Box<dyn OutputWriter>) {
        self.formats.push(format);
        self.outputs.insert(format, writer);
    }
    
    pub fn write_result(&mut self, result: &ScanResult) -> Result<()> {
        for format in &self.formats {
            if let Some(writer) = self.outputs.get_mut(format) {
                writer.write_result(result)?;
            }
        }
        Ok(())
    }
    
    pub fn finalize(&mut self) -> Result<()> {
        for writer in self.outputs.values_mut() {
            writer.finalize()?;
        }
        Ok(())
    }
}

/// Output writer trait
pub trait OutputWriter: Send {
    fn write_result(&mut self, result: &ScanResult) -> Result<()>;
    fn finalize(&mut self) -> Result<()>;
}
```

### 11.2 Text Output

```rust
/// Text output writer
pub struct TextOutputWriter {
    writer: Box<dyn Write + Send>,
    verbose: bool,
}

impl OutputWriter for TextOutputWriter {
    fn write_result(&mut self, result: &ScanResult) -> Result<()> {
        writeln!(
            self.writer,
            "{} {} {} {}",
            result.target,
            result.port,
            result.state,
            result.service.as_ref().map(|s| s.name.as_str()).unwrap_or("unknown")
        )?;
        
        if self.verbose {
            if let Some(ref service) = result.service {
                if let Some(ref version) = service.version {
                    writeln!(self.writer, "  Version: {}", version)?;
                }
                if let Some(ref os) = service.os_info {
                    writeln!(self.writer, "  OS: {}", os)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
```

### 11.3 JSON Output

```rust
use serde::{Serialize, Deserialize};

/// JSON output writer
pub struct JsonOutputWriter {
    writer: Box<dyn Write + Send>,
    results: Vec<ScanResult>,
    pretty: bool,
}

impl JsonOutputWriter {
    pub fn new(writer: Box<dyn Write + Send>, pretty: bool) -> Self {
        Self {
            writer,
            results: Vec::new(),
            pretty,
        }
    }
}

impl OutputWriter for JsonOutputWriter {
    fn write_result(&mut self, result: &ScanResult) -> Result<()> {
        self.results.push(result.clone());
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<()> {
        let json = if self.pretty {
            serde_json::to_string_pretty(&self.results)?
        } else {
            serde_json::to_string(&self.results)?
        };
        
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()?;
        Ok(())
    }
}

/// JSONL (JSON Lines) output writer
pub struct JsonLinesOutputWriter {
    writer: Box<dyn Write + Send>,
}

impl OutputWriter for JsonLinesOutputWriter {
    fn write_result(&mut self, result: &ScanResult) -> Result<()> {
        let json = serde_json::to_string(result)?;
        writeln!(self.writer, "{}", json)?;
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
```

### 11.4 XML Output (Nmap-compatible)

```rust
use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};

/// XML output writer (Nmap-compatible format)
pub struct XmlOutputWriter {
    writer: Writer<Box<dyn Write + Send>>,
    scan_info: ScanInfo,
}

impl XmlOutputWriter {
    pub fn new(writer: Box<dyn Write + Send>, scan_info: ScanInfo) -> Self {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);
        
        // Write XML declaration
        xml_writer.write_event(Event::Decl(BytesDecl::new(
            b"1.0",
            Some(b"UTF-8"),
            None,
        ))).unwrap();
        
        // Write DOCTYPE
        xml_writer.write_event(Event::DocType(BytesText::from_plain_str(
            "nmaprun SYSTEM \"https://nmap.org/nmap.dtd\""
        ))).unwrap();
        
        // Open root element
        let mut root = BytesStart::borrowed_name(b"nmaprun");
        root.push_attribute(("scanner", "warscan"));
        root.push_attribute(("version", env!("CARGO_PKG_VERSION")));
        root.push_attribute(("start", &scan_info.start_time.to_string()));
        xml_writer.write_event(Event::Start(root)).unwrap();
        
        Self {
            writer: xml_writer,
            scan_info,
        }
    }
}

impl OutputWriter for XmlOutputWriter {
    fn write_result(&mut self, result: &ScanResult) -> Result<()> {
        // <host>
        let mut host = BytesStart::borrowed_name(b"host");
        self.writer.write_event(Event::Start(host.to_borrowed()))?;
        
        // <address>
        let mut addr = BytesStart::borrowed_name(b"address");
        addr.push_attribute(("addr", result.target.to_string().as_str()));
        addr.push_attribute(("addrtype", "ipv4"));
        self.writer.write_event(Event::Empty(addr))?;
        
        // <ports>
        self.writer.write_event(Event::Start(BytesStart::borrowed_name(b"ports")))?;
        
        // <port>
        let mut port = BytesStart::borrowed_name(b"port");
        port.push_attribute(("protocol", "tcp"));
        port.push_attribute(("portid", result.port.to_string().as_str()));
        self.writer.write_event(Event::Start(port.to_borrowed()))?;
        
        // <state>
        let mut state = BytesStart::borrowed_name(b"state");
        state.push_attribute(("state", result.state.to_string().as_str()));
        self.writer.write_event(Event::Empty(state))?;
        
        // <service> (if detected)
        if let Some(ref service) = result.service {
            let mut svc = BytesStart::borrowed_name(b"service");
            svc.push_attribute(("name", service.name.as_str()));
            if let Some(ref product) = service.product {
                svc.push_attribute(("product", product.as_str()));
            }
            if let Some(ref version) = service.version {
                svc.push_attribute(("version", version.as_str()));
            }
            self.writer.write_event(Event::Empty(svc))?;
        }
        
        // </port>
        self.writer.write_event(Event::End(BytesEnd::borrowed(b"port")))?;
        
        // </ports>
        self.writer.write_event(Event::End(BytesEnd::borrowed(b"ports")))?;
        
        // </host>
        self.writer.write_event(Event::End(BytesEnd::borrowed(b"host")))?;
        
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<()> {
        // Write scan statistics
        let mut runstats = BytesStart::borrowed_name(b"runstats");
        self.writer.write_event(Event::Start(runstats))?;
        
        // <finished>
        let mut finished = BytesStart::borrowed_name(b"finished");
        finished.push_attribute(("time", &Instant::now().elapsed().as_secs().to_string()));
        self.writer.write_event(Event::Empty(finished))?;
        
        // </runstats>
        self.writer.write_event(Event::End(BytesEnd::borrowed(b"runstats")))?;
        
        // Close root element
        self.writer.write_event(Event::End(BytesEnd::borrowed(b"nmaprun")))?;
        
        Ok(())
    }
}
```

### 11.5 Database Output

```rust
/// SQLite database output
pub struct SqliteOutputWriter {
    conn: Connection,
}

impl SqliteOutputWriter {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Create tables
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scans (
                id INTEGER PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT,
                command TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS hosts (
                id INTEGER PRIMARY KEY,
                scan_id INTEGER NOT NULL,
                ip_address TEXT NOT NULL,
                hostname TEXT,
                status TEXT NOT NULL,
                FOREIGN KEY (scan_id) REFERENCES scans(id)
            );
            
            CREATE TABLE IF NOT EXISTS ports (
                id INTEGER PRIMARY KEY,
                host_id INTEGER NOT NULL,
                port INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                state TEXT NOT NULL,
                service TEXT,
                version TEXT,
                FOREIGN KEY (host_id) REFERENCES hosts(id)
            );
            
            CREATE INDEX IF NOT EXISTS idx_hosts_scan_id ON hosts(scan_id);
            CREATE INDEX IF NOT EXISTS idx_ports_host_id ON ports(host_id);"
        )?;
        
        Ok(Self { conn })
    }
}

impl OutputWriter for SqliteOutputWriter {
    fn write_result(&mut self, result: &ScanResult) -> Result<()> {
        // Insert or update host
        let host_id: i64 = self.conn.query_row(
            "INSERT INTO hosts (scan_id, ip_address, status) VALUES (?1, ?2, ?3)
             ON CONFLICT(scan_id, ip_address) DO UPDATE SET status = ?3
             RETURNING id",
            params![1, result.target.to_string(), "up"],
            |row| row.get(0),
        )?;
        
        // Insert port result
        self.conn.execute(
            "INSERT INTO ports (host_id, port, protocol, state, service, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                host_id,
                result.port,
                "tcp",
                result.state.to_string(),
                result.service.as_ref().map(|s| s.name.as_str()),
                result.service.as_ref().and_then(|s| s.version.as_deref()),
            ],
        )?;
        
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<()> {
        // Commit any pending transactions
        Ok(())
    }
}
```

### 11.6 PCAP Output

```rust
/// PCAP file writer for packet logging
pub struct PcapOutputWriter {
    pcap_writer: pcap::Savefile,
}

impl PcapOutputWriter {
    pub fn new(path: &str) -> Result<Self> {
        let pcap_writer = pcap::Savefile::new(path)?;
        Ok(Self { pcap_writer })
    }
    
    pub fn write_packet(&mut self, packet: &[u8]) -> Result<()> {
        self.pcap_writer.write(packet)?;
        Ok(())
    }
}
```

---

## 12. Extensibility and Plugin Architecture

### 12.1 Plugin System

```rust
/// Plugin trait for extensibility
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str;
    
    /// Initialize plugin
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    
    /// Called before scan starts
    fn on_scan_start(&mut self, scan_info: &ScanInfo) -> Result<()> {
        Ok(())
    }
    
    /// Called for each discovered host
    fn on_host_discovered(&mut self, host: &HostInfo) -> Result<()> {
        Ok(())
    }
    
    /// Called for each port scan result
    fn on_port_result(&mut self, result: &ScanResult) -> Result<()> {
        Ok(())
    }
    
    /// Called after scan completes
    fn on_scan_complete(&mut self, results: &ScanResults) -> Result<()> {
        Ok(())
    }
    
    /// Cleanup
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    pub fn initialize_all(&mut self, config: &PluginConfig) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.initialize(config)?;
        }
        Ok(())
    }
    
    pub fn notify_scan_start(&mut self, scan_info: &ScanInfo) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.on_scan_start(scan_info)?;
        }
        Ok(())
    }
    
    pub fn notify_port_result(&mut self, result: &ScanResult) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.on_port_result(result)?;
        }
        Ok(())
    }
    
    // ... other notification methods
}
```

### 12.2 Scripting Engine (Lua)

```rust
use mlua::prelude::*;

/// Lua scripting engine
pub struct LuaScriptEngine {
    lua: Lua,
    scripts: Vec<LuaScript>,
}

#[derive(Clone)]
pub struct LuaScript {
    name: String,
    code: String,
    categories: Vec<String>,
}

impl LuaScriptEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        
        // Register API functions
        Self::register_api(&lua)?;
        
        Ok(Self {
            lua,
            scripts: Vec::new(),
        })
    }
    
    fn register_api(lua: &Lua) -> Result<()> {
        let globals = lua.globals();
        
        // Register warscan table
        let warscan = lua.create_table()?;
        
        // warscan.tcp_connect(host, port)
        let tcp_connect = lua.create_function(|_, (host, port): (String, u16)| {
            // Implementation
            Ok(true)
        })?;
        warscan.set("tcp_connect", tcp_connect)?;
        
        // warscan.http_get(url)
        let http_get = lua.create_function(|_, url: String| {
            // Implementation
            Ok("HTTP response".to_string())
        })?;
        warscan.set("http_get", http_get)?;
        
        // warscan.log(message)
        let log = lua.create_function(|_, message: String| {
            tracing::info!("[LUA] {}", message);
            Ok(())
        })?;
        warscan.set("log", log)?;
        
        globals.set("warscan", warscan)?;
        
        Ok(())
    }
    
    pub fn load_script(&mut self, script: LuaScript) -> Result<()> {
        // Validate script
        self.lua.load(&script.code).eval::<()>()?;
        self.scripts.push(script);
        Ok(())
    }
    
    pub async fn run_script(
        &self,
        script_name: &str,
        target: &ScanResult,
    ) -> Result<ScriptResult> {
        let script = self.scripts.iter()
            .find(|s| s.name == script_name)
            .ok_or(Error::ScriptNotFound)?;
        
        // Execute script
        let result: LuaValue = self.lua.load(&script.code)
            .call_async((target.target.to_string(), target.port))
            .await?;
        
        Ok(ScriptResult {
            script_name: script_name.to_string(),
            output: format!("{:?}", result),
        })
    }
}

/// Example Lua script
const HTTP_TITLE_SCRIPT: &str = r#"
function run(host, port)
    local response = warscan.http_get("http://" .. host .. ":" .. port .. "/")
    local title = string.match(response, "<title>(.-)</title>")
    
    if title then
        warscan.log("Found title: " .. title)
        return {title = title}
    end
    
    return nil
end
"#;
```

### 12.3 Custom Scan Modules

```rust
/// Custom scan module trait
pub trait ScanModule: Send + Sync {
    /// Module name
    fn name(&self) -> &str;
    
    /// Scan a target
    fn scan(&mut self, target: SocketAddr) -> Result<ModuleResult>;
    
    /// Supported port/service types
    fn supported_services(&self) -> Vec<&str>;
}

/// Example: HTTP vulnerability scanner module
pub struct HttpVulnScanner {
    user_agent: String,
}

impl ScanModule for HttpVulnScanner {
    fn name(&self) -> &str {
        "http-vuln-scanner"
    }
    
    fn scan(&mut self, target: SocketAddr) -> Result<ModuleResult> {
        // Check for common HTTP vulnerabilities
        let mut findings = Vec::new();
        
        // Check for directory listing
        if self.check_directory_listing(target)? {
            findings.push("Directory listing enabled".to_string());
        }
        
        // Check for common files
        if self.check_common_files(target)? {
            findings.push("Sensitive files accessible".to_string());
        }
        
        Ok(ModuleResult {
            findings,
            severity: Severity::Medium,
        })
    }
    
    fn supported_services(&self) -> Vec<&str> {
        vec!["http", "https"]
    }
}

impl HttpVulnScanner {
    fn check_directory_listing(&self, target: SocketAddr) -> Result<bool> {
        // Implementation
        Ok(false)
    }
    
    fn check_common_files(&self, target: SocketAddr) -> Result<bool> {
        // Check for: robots.txt, .git/, .env, etc.
        Ok(false)
    }
}
```

---

## 13. User Interfaces

### 13.1 CLI (Command-Line Interface)

```rust
use clap::{Parser, Subcommand, ValueEnum};

/// ProRT-IP WarScan - Modern Network Scanner
#[derive(Parser)]
#[command(name = "prtip")]
#[command(about = "Advanced network reconnaissance tool", long_about = None)]
#[command(version)]
struct Cli {
    /// Target specification (IP, CIDR, range)
    #[arg(required = true)]
    targets: Vec<String>,
    
    /// Scan technique
    #[command(flatten)]
    scan_type: ScanTypeArgs,
    
    /// Port specification
    #[arg(short = 'p', long, default_value = "1-1000")]
    ports: String,
    
    /// Timing template (0-5)
    #[arg(short = 'T', long, value_enum, default_value = "normal")]
    timing: TimingTemplate,
    
    /// Maximum scan rate (packets per second)
    #[arg(long)]
    max_rate: Option<u64>,
    
    /// Enable service version detection
    #[arg(short = 's', long)]
    service_detection: bool,
    
    /// Enable OS detection
    #[arg(short = 'O', long)]
    os_detection: bool,
    
    /// Output format
    #[arg(short = 'o', long, value_enum, default_value = "text")]
    output_format: OutputFormat,
    
    /// Output file
    #[arg(long)]
    output_file: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Enable debugging
    #[arg(long)]
    debug: bool,
    
    /// Network interface
    #[arg(short = 'i', long)]
    interface: Option<String>,
    
    /// Source IP address
    #[arg(short = 'S', long)]
    source_ip: Option<IpAddr>,
    
    /// Stealth options
    #[command(flatten)]
    stealth: StealthArgs,
    
    /// Script options
    #[command(flatten)]
    scripts: ScriptArgs,
}

#[derive(clap::Args)]
#[group(multiple = true)]
struct ScanTypeArgs {
    /// TCP SYN scan
    #[arg(long)]
    syn: bool,
    
    /// TCP connect scan
    #[arg(long)]
    connect: bool,
    
    /// TCP FIN scan
    #[arg(long)]
    fin: bool,
    
    /// TCP NULL scan
    #[arg(long)]
    null: bool,
    
    /// TCP Xmas scan
    #[arg(long)]
    xmas: bool,
    
    /// TCP ACK scan
    #[arg(long)]
    ack: bool,
    
    /// UDP scan
    #[arg(short = 'U', long)]
    udp: bool,
}

#[derive(clap::Args)]
struct StealthArgs {
    /// Use decoy scanning
    #[arg(short = 'D', long)]
    decoys: Option<String>,
    
    /// Spoof source address
    #[arg(long)]
    spoof_source: Option<IpAddr>,
    
    /// Fragment packets
    #[arg(short = 'f', long)]
    fragment: bool,
    
    /// Source port
    #[arg(long)]
    source_port: Option<u16>,
    
    /// Randomize target order
    #[arg(long)]
    randomize: bool,
}

#[derive(clap::Args)]
struct ScriptArgs {
    /// Run NSE scripts
    #[arg(long)]
    script: Option<String>,
    
    /// Script categories to run
    #[arg(long)]
    script_categories: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(cli.verbose, cli.debug)?;
    
    // Build scanner configuration
    let config = build_config(&cli)?;
    
    // Create scanner
    let mut scanner = Scanner::new(config)?;
    
    // Run scan
    let results = scanner.scan().await?;
    
    // Output results
    output_results(&cli, &results)?;
    
    Ok(())
}

fn build_config(cli: &Cli) -> Result<ScanConfig> {
    let mut config = ScanConfig::default();
    
    // Parse targets
    for target_spec in &cli.targets {
        config.add_target(TargetSpec::parse(target_spec)?);
    }
    
    // Parse ports
    config.ports = parse_port_spec(&cli.ports)?;
    
    // Scan types
    if cli.scan_type.syn {
        config.scan_types.push(ScanType::Syn);
    }
    if cli.scan_type.connect {
        config.scan_types.push(ScanType::Connect);
    }
    // ... other scan types
    
    // Timing
    config.timing = cli.timing.to_config();
    if let Some(max_rate) = cli.max_rate {
        config.timing.max_rate = max_rate;
    }
    
    // Features
    config.service_detection = cli.service_detection;
    config.os_detection = cli.os_detection;
    
    // Stealth
    if let Some(ref decoys) = cli.stealth.decoys {
        config.decoys = parse_decoys(decoys)?;
    }
    config.fragment = cli.stealth.fragment;
    config.randomize_targets = cli.stealth.randomize;
    
    Ok(config)
}
```

### 13.2 TUI (Terminal User Interface)

```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Table, Row},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// TUI application state
pub struct TuiApp {
    scanner: Arc<Mutex<Scanner>>,
    results: Arc<Mutex<Vec<ScanResult>>>,
    selected_tab: usize,
    scroll_offset: usize,
}

impl TuiApp {
    pub async fn run(mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Start scan in background
        let scanner = Arc::clone(&self.scanner);
        let results = Arc::clone(&self.results);
        tokio::spawn(async move {
            Self::run_scan(scanner, results).await
        });
        
        // Main event loop
        loop {
            // Draw UI
            terminal.draw(|f| self.draw(f))?;
            
            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => self.next_tab(),
                        KeyCode::Up => self.scroll_up(),
                        KeyCode::Down => self.scroll_down(),
                        _ => {}
                    }
                }
            }
        }
        
        // Cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        
        Ok(())
    }
    
    fn draw(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Footer
            ])
            .split(f.size());
        
        // Header
        self.draw_header(f, chunks[0]);
        
        // Content (tabs)
        match self.selected_tab {
            0 => self.draw_overview(f, chunks[1]),
            1 => self.draw_results(f, chunks[1]),
            2 => self.draw_stats(f, chunks[1]),
            _ => {}
        }
        
        // Footer
        self.draw_footer(f, chunks[2]);
    }
    
    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new("ProRT-IP WarScan v1.0")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, area);
    }
    
    fn draw_overview(&self, f: &mut Frame, area: Rect) {
        let scanner = self.scanner.lock().unwrap();
        let stats = scanner.get_stats();
        
        let text = vec![
            Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    format!("{:?}", stats.status),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Progress: "),
                Span::raw(format!("{}/{}", stats.completed, stats.total)),
            ]),
            Line::from(vec![
                Span::raw("Rate: "),
                Span::raw(format!("{} pps", stats.current_rate)),
            ]),
        ];
        
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Overview").borders(Borders::ALL));
        
        f.render_widget(paragraph, area);
    }
    
    fn draw_results(&self, f: &mut Frame, area: Rect) {
        let results = self.results.lock().unwrap();
        
        let items: Vec<ListItem> = results.iter()
            .skip(self.scroll_offset)
            .take(area.height as usize - 2)
            .map(|r| {
                let state_color = match r.state {
                    PortState::Open => Color::Green,
                    PortState::Closed => Color::Red,
                    PortState::Filtered => Color::Yellow,
                    _ => Color::Gray,
                };
                
                ListItem::new(format!(
                    "{:15} {:5} {:10} {}",
                    r.target,
                    r.port,
                    r.state,
                    r.service.as_ref().map(|s| s.name.as_str()).unwrap_or(""),
                ))
                .style(Style::default().fg(state_color))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().title("Results").borders(Borders::ALL));
        
        f.render_widget(list, area);
    }
    
    fn draw_stats(&self, f: &mut Frame, area: Rect) {
        let scanner = self.scanner.lock().unwrap();
        let stats = scanner.get_stats();
        
        let table = Table::new(vec![
            Row::new(vec!["Total Hosts", &stats.total_hosts.to_string()]),
            Row::new(vec!["Hosts Up", &stats.hosts_up.to_string()]),
            Row::new(vec!["Open Ports", &stats.open_ports.to_string()]),
            Row::new(vec!["Closed Ports", &stats.closed_ports.to_string()]),
            Row::new(vec!["Filtered Ports", &stats.filtered_ports.to_string()]),
        ])
        .header(Row::new(vec!["Metric", "Value"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().title("Statistics").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
        
        f.render_widget(table, area);
    }
    
    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new("Tab: Switch | ↑/↓: Scroll | q: Quit")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, area);
    }
    
    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 3;
    }
    
    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
    
    fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }
}
```

---

## 14. Cross-Platform Implementation

### 14.1 Platform Detection

```rust
/// Platform-specific code organization
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

/// Platform-specific raw socket implementation
pub trait RawSocketImpl: Send + Sync {
    fn create_raw_socket(&self, protocol: Protocol) -> Result<RawSocket>;
    fn send_packet(&self, socket: &RawSocket, packet: &[u8]) -> Result<()>;
    fn recv_packet(&self, socket: &RawSocket, buffer: &mut [u8]) -> Result<usize>;
}

/// Get platform-specific implementation
pub fn get_platform_impl() -> Box<dyn RawSocketImpl> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxRawSocket::new())
    }
    
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsRawSocket::new())
    }
    
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsRawSocket::new())
    }
}
```

### 14.2 Linux Implementation

```rust
#[cfg(target_os = "linux")]
pub mod linux {
    use nix::sys::socket::*;
    
    pub struct LinuxRawSocket {
        capabilities: Capabilities,
    }
    
    impl LinuxRawSocket {
        pub fn new() -> Self {
            Self {
                capabilities: Capabilities::check(),
            }
        }
        
        fn has_raw_socket_capability(&self) -> bool {
            self.capabilities.has(Capability::CAP_NET_RAW)
        }
    }
    
    impl RawSocketImpl for LinuxRawSocket {
        fn create_raw_socket(&self, protocol: Protocol) -> Result<RawSocket> {
            if !self.has_raw_socket_capability() {
                return Err(Error::InsufficientPrivileges);
            }
            
            let socket_type = match protocol {
                Protocol::Tcp | Protocol::Udp => SockType::Raw,
                Protocol::Icmp => SockType::Raw,
            };
            
            let protocol = match protocol {
                Protocol::Tcp => SockProtocol::Tcp,
                Protocol::Udp => SockProtocol::Udp,
                Protocol::Icmp => SockProtocol::Icmp,
            };
            
            let socket = socket(
                AddressFamily::Inet,
                socket_type,
                SockFlag::empty(),
                protocol,
            )?;
            
            // Set socket options
            setsockopt(socket, sockopt::IpHdrIncl, &true)?;
            
            Ok(RawSocket { fd: socket })
        }
        
        fn send_packet(&self, socket: &RawSocket, packet: &[u8]) -> Result<()> {
            use nix::sys::socket::send;
            send(socket.fd, packet, MsgFlags::empty())?;
            Ok(())
        }
        
        fn recv_packet(&self, socket: &RawSocket, buffer: &mut [u8]) -> Result<usize> {
            use nix::sys::socket::recv;
            let n = recv(socket.fd, buffer, MsgFlags::empty())?;
            Ok(n)
        }
    }
    
    /// Check for Linux capabilities
    struct Capabilities {
        has_net_raw: bool,
        has_net_admin: bool,
    }
    
    impl Capabilities {
        fn check() -> Self {
            // Check if running as root or with capabilities
            let euid = unsafe { libc::geteuid() };
            
            Self {
                has_net_raw: euid == 0, // Simplified check
                has_net_admin: euid == 0,
            }
        }
        
        fn has(&self, cap: Capability) -> bool {
            match cap {
                Capability::CAP_NET_RAW => self.has_net_raw,
                Capability::CAP_NET_ADMIN => self.has_net_admin,
            }
        }
    }
    
    enum Capability {
        CAP_NET_RAW,
        CAP_NET_ADMIN,
    }
}
```

### 14.3 Windows Implementation

```rust
#[cfg(target_os = "windows")]
pub mod windows {
    use winapi::um::winsock2::*;
    use winapi::shared::ws2def::*;
    
    pub struct WindowsRawSocket {
        npcap_installed: bool,
    }
    
    impl WindowsRawSocket {
        pub fn new() -> Self {
            Self {
                npcap_installed: Self::check_npcap(),
            }
        }
        
        fn check_npcap() -> bool {
            // Check if Npcap or WinPcap is installed
            std::path::Path::new("C:\\Windows\\System32\\Npcap\\wpcap.dll").exists()
                || std::path::Path::new("C:\\Windows\\System32\\wpcap.dll").exists()
        }
    }
    
    impl RawSocketImpl for WindowsRawSocket {
        fn create_raw_socket(&self, protocol: Protocol) -> Result<RawSocket> {
            if !self.npcap_installed {
                return Err(Error::NpcapNotInstalled);
            }
            
            unsafe {
                // Initialize Winsock
                let mut wsadata: WSADATA = std::mem::zeroed();
                if WSAStartup(0x0202, &mut wsadata) != 0 {
                    return Err(Error::WinsockInitFailed);
                }
                
                // Create raw socket
                let socket_type = match protocol {
                    Protocol::Tcp => SOCK_RAW,
                    Protocol::Udp => SOCK_RAW,
                    Protocol::Icmp => SOCK_RAW,
                };
                
                let protocol_num = match protocol {
                    Protocol::Tcp => IPPROTO_TCP,
                    Protocol::Udp => IPPROTO_UDP,
                    Protocol::Icmp => IPPROTO_ICMP,
                };
                
                let socket = socket(AF_INET, socket_type, protocol_num);
                if socket == INVALID_SOCKET {
                    return Err(Error::SocketCreationFailed);
                }
                
                // Set socket options
                let optval: i32 = 1;
                setsockopt(
                    socket,
                    IPPROTO_IP,
                    IP_HDRINCL,
                    &optval as *const _ as *const i8,
                    std::mem::size_of::<i32>() as i32,
                );
                
                Ok(RawSocket { socket })
            }
        }
        
        fn send_packet(&self, socket: &RawSocket, packet: &[u8]) -> Result<()> {
            unsafe {
                let result = send(
                    socket.socket,
                    packet.as_ptr() as *const i8,
                    packet.len() as i32,
                    0,
                );
                
                if result == SOCKET_ERROR {
                    return Err(Error::SendFailed);
                }
            }
            Ok(())
        }
        
        fn recv_packet(&self, socket: &RawSocket, buffer: &mut [u8]) -> Result<usize> {
            unsafe {
                let result = recv(
                    socket.socket,
                    buffer.as_mut_ptr() as *mut i8,
                    buffer.len() as i32,
                    0,
                );
                
                if result == SOCKET_ERROR {
                    return Err(Error::RecvFailed);
                }
                
                Ok(result as usize)
            }
        }
    }
}
```

### 14.4 macOS Implementation

```rust
#[cfg(target_os = "macos")]
pub mod macos {
    use nix::sys::socket::*;
    
    pub struct MacOsRawSocket {}
    
    impl MacOsRawSocket {
        pub fn new() -> Self {
            Self {}
        }
        
        fn check_privileges() -> bool {
            unsafe { libc::geteuid() == 0 }
        }
    }
    
    impl RawSocketImpl for MacOsRawSocket {
        fn create_raw_socket(&self, protocol: Protocol) -> Result<RawSocket> {
            if !Self::check_privileges() {
                return Err(Error::InsufficientPrivileges);
            }
            
            // macOS raw socket creation (similar to Linux)
            let socket = socket(
                AddressFamily::Inet,
                SockType::Raw,
                SockFlag::empty(),
                match protocol {
                    Protocol::Tcp => SockProtocol::Tcp,
                    Protocol::Udp => SockProtocol::Udp,
                    Protocol::Icmp => SockProtocol::Icmp,
                },
            )?;
            
            Ok(RawSocket { fd: socket })
        }
        
        fn send_packet(&self, socket: &RawSocket, packet: &[u8]) -> Result<()> {
            use nix::sys::socket::send;
            send(socket.fd, packet, MsgFlags::empty())?;
            Ok(())
        }
        
        fn recv_packet(&self, socket: &RawSocket, buffer: &mut [u8]) -> Result<usize> {
            use nix::sys::socket::recv;
            let n = recv(socket.fd, buffer, MsgFlags::empty())?;
            Ok(n)
        }
    }
}
```

---

## 15. Security and Safety Considerations

### 15.1 Input Validation

```rust
/// Input validator for user-provided data
pub struct InputValidator;

impl InputValidator {
    /// Validate target specification
    pub fn validate_target(target: &str) -> Result<()> {
        // Check for valid IP, CIDR, or hostname
        if !Self::is_valid_ip(target)
            && !Self::is_valid_cidr(target)
            && !Self::is_valid_hostname(target)
        {
            return Err(Error::InvalidTarget(target.to_string()));
        }
        
        Ok(())
    }
    
    /// Validate port specification
    pub fn validate_ports(ports: &str) -> Result<()> {
        // Parse and validate port specification
        for part in ports.split(',') {
            if part.contains('-') {
                // Range
                let parts: Vec<&str> = part.split('-').collect();
                if parts.len() != 2 {
                    return Err(Error::InvalidPortSpec(ports.to_string()));
                }
                
                let start: u16 = parts[0].parse()
                    .map_err(|_| Error::InvalidPortSpec(ports.to_string()))?;
                let end: u16 = parts[1].parse()
                    .map_err(|_| Error::InvalidPortSpec(ports.to_string()))?;
                
                if start > end || end > 65535 {
                    return Err(Error::InvalidPortSpec(ports.to_string()));
                }
            } else {
                // Single port
                let port: u16 = part.parse()
                    .map_err(|_| Error::InvalidPortSpec(ports.to_string()))?;
                
                if port == 0 {
                    return Err(Error::InvalidPortSpec(ports.to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    fn is_valid_ip(s: &str) -> bool {
        s.parse::<IpAddr>().is_ok()
    }
    
    fn is_valid_cidr(s: &str) -> bool {
        s.parse::<IpNetwork>().is_ok()
    }
    
    fn is_valid_hostname(s: &str) -> bool {
        // Basic hostname validation
        !s.is_empty() && s.len() <= 255
            && s.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    }
}
```

### 15.2 Rate Limiting to Prevent DoS

```rust
/// Safety guard to prevent accidental DoS
pub struct DosPreventionGuard {
    max_rate_without_confirmation: u64,
    large_scan_threshold: usize,
}

impl DosPreventionGuard {
    pub fn new() -> Self {
        Self {
            max_rate_without_confirmation: 10000, // 10k pps
            large_scan_threshold: 100000,         // 100k targets
        }
    }
    
    pub fn check_scan_config(&self, config: &ScanConfig) -> Result<()> {
        // Check rate
        if config.max_rate > self.max_rate_without_confirmation {
            if !self.confirm_high_rate(config.max_rate)? {
                return Err(Error::RateTooHigh);
            }
        }
        
        // Check scale
        let total_targets = config.count_targets();
        if total_targets > self.large_scan_threshold {
            if !self.confirm_large_scan(total_targets)? {
                return Err(Error::ScanTooLarge);
            }
        }
        
        // Check for internet-wide scans
        if config.targets_include_internet() {
            if !self.confirm_internet_scan()? {
                return Err(Error::InternetScanDenied);
            }
        }
        
        Ok(())
    }
    
    fn confirm_high_rate(&self, rate: u64) -> Result<bool> {
        println!(
            "WARNING: High scan rate ({} pps) may cause network issues.",
            rate
        );
        println!("Are you sure you want to continue? (yes/no): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "yes")
    }
    
    fn confirm_large_scan(&self, count: usize) -> Result<bool> {
        println!(
            "WARNING: Large scan ({} targets) will take significant time.",
            count
        );
        println!("Are you sure you want to continue? (yes/no): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "yes")
    }
    
    fn confirm_internet_scan(&self) -> Result<bool> {
        println!("WARNING: Internet-wide scan detected!");
        println!("This will generate significant traffic and may be illegal in your jurisdiction.");
        println!("Do you have explicit authorization? (yes/no): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "yes")
    }
}
```

### 15.3 Privilege Management

```rust
/// Privilege manager for safe privilege dropping
pub struct PrivilegeManager {
    initial_uid: u32,
    initial_gid: u32,
}

impl PrivilegeManager {
    pub fn new() -> Self {
        Self {
            initial_uid: unsafe { libc::getuid() },
            initial_gid: unsafe { libc::getgid() },
        }
    }
    
    pub fn is_root(&self) -> bool {
        self.initial_uid == 0
    }
    
    pub fn drop_privileges(&self, uid: u32, gid: u32) -> Result<()> {
        if !self.is_root() {
            return Err(Error::NotRoot);
        }
        
        #[cfg(unix)]
        {
            use nix::unistd::{setgid, setuid, Gid, Uid};
            
            // Drop group privileges first
            setgid(Gid::from_raw(gid))?;
            
            // Drop user privileges
            setuid(Uid::from_raw(uid))?;
            
            // Verify we can't regain privileges
            if unsafe { libc::getuid() } == 0 {
                return Err(Error::PrivilegeDropFailed);
            }
        }
        
        Ok(())
    }
}
```

---

## 16. Testing Strategy

### 16.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_target_parsing() {
        // Test single IP
        let target = TargetSpec::parse("192.168.1.1").unwrap();
        assert!(matches!(target, TargetSpec::SingleIp(_)));
        
        // Test CIDR
        let target = TargetSpec::parse("10.0.0.0/24").unwrap();
        assert!(matches!(target, TargetSpec::Cidr(_)));
        
        // Test range
        let target = TargetSpec::parse("192.168.1.1-254").unwrap();
        assert!(matches!(target, TargetSpec::Range { .. }));
    }
    
    #[test]
    fn test_port_parsing() {
        let ports = parse_port_spec("80,443,8080-8090").unwrap();
        assert_eq!(ports.len(), 13); // 80, 443, + 11 ports in range
    }
    
    #[tokio::test]
    async fn test_syn_scan() {
        let mut scanner = SynScanner::new_test();
        // Mock network layer
        // ...
    }
    
    #[test]
    fn test_packet_building() {
        let packet = TcpPacketBuilder::new()
            .source(Ipv4Addr::new(192, 168, 1, 1), 12345)
            .destination(Ipv4Addr::new(192, 168, 1, 2), 80)
            .sequence(1000)
            .flags(TcpFlags::SYN)
            .build()
            .unwrap();
        
        // Verify packet structure
        assert!(packet.len() > 0);
    }
}
```

### 16.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_local_scan() {
        // Scan localhost
        let config = ScanConfig {
            targets: vec![TargetSpec::SingleIp("127.0.0.1".parse().unwrap())],
            ports: vec![80, 22, 443],
            scan_types: vec![ScanType::Connect],
            ..Default::default()
        };
        
        let mut scanner = Scanner::new(config).unwrap();
        let results = scanner.scan().await.unwrap();
        
        assert!(!results.is_empty());
    }
    
    #[tokio::test]
    async fn test_service_detection() {
        // Test against known service
        let mut detector = ServiceDetector::new();
        let result = detector.detect_service(
            "scanme.nmap.org:80".parse().unwrap(),
            PortState::Open,
        ).await.unwrap();
        
        assert_eq!(result.service, "http");
    }
}
```

### 16.3 Fuzzing

```rust
// Using cargo-fuzz

#[cfg(fuzzing)]
mod fuzz_targets {
    use libfuzzer_sys::fuzz_target;
    
    fuzz_target!(|data: &[u8]| {
        // Fuzz packet parser
        let _ = PacketData::from_slice(data);
    });
    
    fuzz_target!(|data: &[u8]| {
        // Fuzz service detection
        if let Ok(s) = String::from_utf8(data.to_vec()) {
            let _ = ServiceDetector::match_banner(&s);
        }
    });
}
```

### 16.4 Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_packet_building(c: &mut Criterion) {
    c.bench_function("tcp_packet_build", |b| {
        b.iter(|| {
            TcpPacketBuilder::new()
                .source(black_box(Ipv4Addr::new(192, 168, 1, 1)), 12345)
                .destination(black_box(Ipv4Addr::new(192, 168, 1, 2)), 80)
                .sequence(1000)
                .flags(TcpFlags::SYN)
                .build()
                .unwrap()
        });
    });
}

fn bench_target_randomization(c: &mut Criterion) {
    let mut randomizer = TargetRandomizer::new(100000);
    
    c.bench_function("target_randomize", |b| {
        b.iter(|| {
            randomizer.next_target()
        });
    });
}

criterion_group!(benches, bench_packet_building, bench_target_randomization);
criterion_main!(benches);
```

---

## 17. Development Roadmap

### Phase 1: Foundation (Months 1-2)

**Goal:** Core scanning engine with basic TCP scans

**Deliverables:**

- Project structure and build system
- Raw packet transmission/reception (pnet integration)
- Basic TCP SYN scan
- TCP Connect scan (fallback)
- Simple target parsing (single IP, CIDR)
- Text output format
- Unit test framework

**Success Criteria:**

- Can scan localhost successfully
- Proper packet crafting validated with Wireshark
- 1000+ ports scanned in <5 seconds on local network

---

### Phase 2: Advanced Scanning (Months 3-4)

**Goal:** Complete scan type coverage

**Deliverables:**

- FIN/NULL/Xmas scans
- ACK scan for firewall mapping
- UDP scanning with protocol-specific probes
- SCTP scanning (INIT and COOKIE ECHO)
- ICMP-based host discovery
- ARP discovery for LAN
- Port randomization
- Retransmission logic

**Success Criteria:**

- All major scan types implemented
- Can detect filtered vs closed ports
- Successful UDP service detection (DNS, SNMP)

---

### Phase 3: Intelligence Gathering (Months 5-6)

**Goal:** OS fingerprinting and service detection

**Deliverables:**

- OS fingerprint database (initial set: 100+ OS signatures)
- TCP/IP stack fingerprinting implementation
- Passive OS fingerprinting
- Service probe database (500+ service signatures)
- Banner grabbing for common protocols
- TLS/SSL service detection
- Version extraction and parsing

**Success Criteria:**

- 85%+ accuracy on OS detection (tested against known targets)
- Successful service version detection for top 20 services
- Banner grabbing works for HTTP/FTP/SSH/SMTP

---

### Phase 4: Performance Optimization (Month 7)

**Goal:** Internet-scale performance

**Deliverables:**

- Stateless scanning mode (Masscan-style)
- SipHash response validation
- Adaptive rate control with congestion avoidance
- Zero-copy optimizations
- Multi-core utilization (work stealing)
- Batch packet processing
- Performance benchmarks and profiling

**Success Criteria:**

- 500k+ pps on commodity hardware
- <5% packet loss at high rates
- Can complete /16 network scan in <10 minutes

---

### Phase 5: Stealth and Evasion (Month 8)

**Goal:** Advanced red-team capabilities

**Deliverables:**

- Timing templates (T0-T5)
- Packet fragmentation
- Decoy scanning
- Idle (zombie) scanning
- Source port manipulation
- TTL manipulation
- MAC address spoofing

**Success Criteria:**

- Paranoid mode (T0) evades basic IDS
- Idle scan works against predictable-IPID hosts
- Fragmentation bypasses simple filters

---

### Phase 6: Output and Integration (Month 9)

**Goal:** Professional reporting and data export

**Deliverables:**

- JSON/JSONL output
- XML output (Nmap-compatible)
- CSV output
- Grepable output
- SQLite database export
- PostgreSQL database export
- PCAP packet logging

**Success Criteria:**

- Output parseable by standard tools
- XML compatible with Nmap viewers
- Database schema supports efficient queries

---

### Phase 7: Extensibility (Month 10)

**Goal:** Plugin architecture and scripting

**Deliverables:**

- Plugin API and trait system
- Lua scripting engine with WarScan API
- 20+ example scripts (HTTP title, SSL cert, etc.)
- Script categories (discovery, auth, vuln)
- Custom scan module interface

**Success Criteria:**

- Third-party plugins can be loaded
- Lua scripts execute correctly
- Custom modules integrate seamlessly

---

### Phase 8: TUI (Month 11)

**Goal:** Interactive terminal interface

**Deliverables:**

- Real-time scan progress dashboard
- Interactive result browser
- Configuration wizard
- Live statistics display
- Keyboard navigation

**Success Criteria:**

- TUI renders correctly on Linux/macOS/Windows
- Responsive at high scan rates
- User-friendly for non-experts

---

### Phase 9: Web Interface (Month 12)

**Goal:** Browser-based dashboard

**Deliverables:**

- Local web server (actix-web/axum)
- REST API for scan control
- WebSocket for real-time updates
- Responsive web UI (HTML/CSS/JS)
- Authentication system
- Scan history and reports

**Success Criteria:**

- API documented with OpenAPI
- UI works on modern browsers
- Secure by default (TLS, auth)

---

### Phase 10: GUI Application (Month 13)

**Goal:** Native desktop application

**Deliverables:**

- Cross-platform GUI (Tauri or Iced)
- Visual target builder
- Scan template manager
- Results visualization (charts, graphs)
- Export functionality

**Success Criteria:**

- Native feel on Windows/Linux/macOS
- Installs via standard package managers
- Accessible to non-technical users

---

### Phase 11: Documentation and Polish (Month 14)

**Goal:** Production-ready release

**Deliverables:**

- Comprehensive user manual
- API documentation (rustdoc)
- Tutorial videos
- Example workflows
- Man pages
- Troubleshooting guide

**Success Criteria:**

- Documentation covers all features
- New users can perform basic scans from docs
- No critical bugs in issue tracker

---

### Phase 12: Community and Distribution (Month 15)

**Goal:** Open-source launch

**Deliverables:**

- GitHub repository public release
- Package for: Cargo, Debian/Ubuntu, Arch AUR, Homebrew, Chocolatey
- Docker images
- Contribution guidelines
- Code of conduct
- Security policy
- CI/CD pipeline

**Success Criteria:**

- GPLv3 licensing verified
- Available in major package repositories
- Automated builds for releases
- Community accepting contributions

---

## 18. Project Structure and Organization

### 18.1 Repository Layout

```
prtip-warscan/
├── Cargo.toml              # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE (GPLv3)
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── .github/
│   ├── workflows/
│   │   ├── ci.yml          # Continuous integration
│   │   ├── release.yml     # Release automation
│   │   └── security.yml    # Security scanning
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       └── feature_request.md
│
├── prtip-core/             # Core library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── scanner/
│   │   │   ├── mod.rs
│   │   │   ├── tcp.rs
│   │   │   ├── udp.rs
│   │   │   ├── icmp.rs
│   │   │   └── sctp.rs
│   │   ├── packet/
│   │   │   ├── mod.rs
│   │   │   ├── builder.rs
│   │   │   ├── parser.rs
│   │   │   └── fragments.rs
│   │   ├── network/
│   │   │   ├── mod.rs
│   │   │   ├── transmit.rs
│   │   │   ├── receive.rs
│   │   │   └── capture.rs
│   │   ├── host_discovery/
│   │   │   ├── mod.rs
│   │   │   ├── icmp.rs
│   │   │   ├── arp.rs
│   │   │   └── tcp_ping.rs
│   │   ├── fingerprint/
│   │   │   ├── mod.rs
│   │   │   ├── os.rs
│   │   │   ├── database.rs
│   │   │   └── passive.rs
│   │   ├── service_detection/
│   │   │   ├── mod.rs
│   │   │   ├── probes.rs
│   │   │   ├── matcher.rs
│   │   │   └── banner.rs
│   │   ├── stealth/
│   │   │   ├── mod.rs
│   │   │   ├── timing.rs
│   │   │   ├── fragmentation.rs
│   │   │   ├── decoys.rs
│   │   │   └── idle_scan.rs
│   │   ├── output/
│   │   │   ├── mod.rs
│   │   │   ├── json.rs
│   │   │   ├── xml.rs
│   │   │   ├── text.rs
│   │   │   └── database.rs
│   │   └── error.rs
│   └── tests/
│       ├── integration_tests.rs
│       └── fixtures/
│
├── prtip-cli/              # Command-line interface
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── args.rs
│   │   ├── config.rs
│   │   └── runner.rs
│   └── tests/
│
├── prtip-tui/              # Terminal UI
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   ├── ui/
│   │   │   ├── mod.rs
│   │   │   ├── dashboard.rs
│   │   │   ├── results.rs
│   │   │   └── stats.rs
│   │   └── events.rs
│   └── tests/
│
├── prtip-web/              # Web interface
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── scans.rs
│   │   │   └── results.rs
│   │   ├── websocket.rs
│   │   └── auth.rs
│   ├── static/             # Frontend assets
│   │   ├── index.html
│   │   ├── style.css
│   │   └── app.js
│   └── tests/
│
├── prtip-gui/              # Desktop GUI
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   └── widgets/
│   └── tests/
│
├── prtip-plugins/          # Plugin system
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── plugin.rs
│   │   ├── manager.rs
│   │   └── scripting/
│   │       ├── mod.rs
│   │       ├── lua.rs
│   │       └── python.rs
│   └── examples/
│       ├── example_plugin.rs
│       └── scripts/
│           ├── http_title.lua
│           └── ssl_cert.lua
│
├── prtip-platform/         # Platform-specific code
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── linux.rs
│       ├── windows.rs
│       └── macos.rs
│
├── data/                   # Data files
│   ├── nmap-os-db          # OS fingerprints
│   ├── nmap-service-probes # Service probes
│   ├── nmap-services       # Port/service mapping
│   └── oui.txt             # MAC vendor database
│
├── docs/                   # Documentation
│   ├── user-guide.md
│   ├── api-reference.md
│   ├── developer-guide.md
│   └── architecture.md
│
├── benches/                # Benchmarks
│   ├── packet_building.rs
│   └── scanning.rs
│
├── fuzz/                   # Fuzzing targets
│   └── fuzz_targets/
│       ├── packet_parser.rs
│       └── service_detector.rs
│
└── scripts/                # Build and utility scripts
    ├── build.sh
    ├── test.sh
    ├── package.sh
    └── generate_docs.sh
```

### 18.2 Module Organization

**Core Library (`prtip-core`):**

- Provides all scanning functionality as a library
- No CLI or UI dependencies
- Fully documented with rustdoc
- Extensive unit tests
- Can be used by other Rust projects

**CLI Application (`prtip-cli`):**

- Thin wrapper around core library
- Argument parsing and validation
- Progress display
- Error handling and user messages

**TUI Application (`prtip-tui`):**

- Interactive terminal interface
- Real-time updates
- Event-driven architecture

**Web Interface (`prtip-web`):**

- REST API for scan control
- WebSocket for live updates
- Static file serving
- Authentication

**GUI Application (`prtip-gui`):**

- Native desktop application
- Visual scan configuration
- Results visualization

**Plugin System (`prtip-plugins`):**

- Plugin trait definitions
- Lua/Python scripting engines
- Example plugins and scripts

**Platform Layer (`prtip-platform`):**

- Platform-specific implementations
- Raw socket abstractions
- Privilege management

### 18.3 Configuration Files

**User Configuration (~/.prtip/config.toml):**

```toml
[general]
default_timing = "Normal"
default_interface = "eth0"

[output]
default_format = "Text"
verbose = false

[stealth]
randomize_targets = true
randomize_ports = true

[performance]
max_parallelism = 1000
adaptive_rate = true

[plugins]
enabled = ["http-title", "ssl-cert"]
script_directory = "~/.prtip/scripts"
```

**Scan Profiles (~/.prtip/profiles/):**

```toml
# quick.toml
[scan]
name = "Quick Scan"
scan_types = ["SYN"]
ports = "1-1000"
timing = "Aggressive"
service_detection = false
os_detection = false

# stealth.toml
[scan]
name = "Stealth Scan"
scan_types = ["FIN", "NULL"]
ports = "1-65535"
timing = "Paranoid"
decoys = true
fragment = true
```

---

## Conclusion

This comprehensive specification document provides the complete technical foundation for implementing ProRT-IP WarScan. It covers:

1. **Architecture** - Modular, async-first design with clear separation of concerns
2. **Technical Stack** - Rust crates and dependencies for each component
3. **Scanning Techniques** - Detailed implementations of TCP/UDP/ICMP/SCTP scans
4. **Intelligence** - OS fingerprinting and service detection systems
5. **Stealth** - Advanced evasion techniques for red-team operations
6. **Performance** - Optimization strategies for internet-scale scanning
7. **Extensibility** - Plugin architecture and scripting engine
8. **Interfaces** - Progressive UI enhancement (CLI → TUI → Web → GUI)
9. **Cross-Platform** - Platform-specific implementations and abstractions
10. **Security** - Safe practices and privilege management
11. **Testing** - Comprehensive testing and quality assurance strategies
12. **Roadmap** - Phased development plan with clear milestones

**Next Steps:**

1. Review and validate technical approach
2. Set up development environment
3. Begin Phase 1 implementation
4. Establish CI/CD pipeline
5. Create initial project structure
6. Implement core packet crafting and basic SYN scan

This document should serve as the definitive reference throughout the development lifecycle, ensuring consistency, completeness, and technical excellence in the ProRT-IP WarScan project.

---

**Document Version:** 1.0  
**Last Updated:** October 6, 2025  
**Author:** ProRT-IP Development Team  
**License:** GPLv3
