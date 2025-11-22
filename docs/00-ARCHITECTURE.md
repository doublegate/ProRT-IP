# ProRT-IP WarScan: Architecture Overview

**Version:** 3.4
**Last Updated:** 2025-11-22
**Status:** Phase 6 IN PROGRESS (Sprint 6.5 COMPLETE) - v0.5.5 TUI + Interactive Widgets (~76% Overall Progress, 2,246 tests, 54.92% coverage)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Design Philosophy](#design-philosophy)
3. [High-Level Architecture](#high-level-architecture)
4. [Core Components](#core-components)
5. [Scanning Modes](#scanning-modes)
6. [Data Flow](#data-flow)
7. [Technology Stack](#technology-stack)
8. [Design Patterns](#design-patterns)

---

## Executive Summary

ProRT-IP WarScan is a modern, high-performance network reconnaissance tool written in Rust. It combines the speed of Masscan (10M+ packets/second), the depth of Nmap's service detection, and the safety of memory-safe implementation.

### Core Value Proposition

- **Speed:** Internet-scale scanning (full IPv4 sweep in <6 minutes on appropriate hardware)
- **Safety:** Memory-safe Rust implementation eliminating buffer overflows and use-after-free vulnerabilities
- **Stealth:** Six evasion techniques including timing controls, decoys, fragmentation, TTL manipulation, bad checksums, and idle (zombie) scanning
- **Completeness:** Full-featured from host discovery through OS/service detection (85-90% detection rate, 187 Nmap probes)
- **IPv6 Support:** 100% coverage across all 6 scanner types (TCP SYN, TCP Connect, UDP, Stealth, Discovery, Decoy)
- **Rate Limiting:** Two-tier system (hostgroup control + AdaptiveRateLimiterV3) with -1.8% average overhead (faster than no limiting!)
- **Extensibility:** Plugin architecture and scripting engine for custom workflows
- **Accessibility:** Progressive interfaces from CLI → TUI → Web → GUI

### Current Capabilities (v0.4.3)

**Scan Types:** 8 total
- TCP SYN (default, requires privileges)
- TCP Connect (fallback, no privileges)
- UDP (protocol-specific payloads)
- Stealth (FIN/NULL/Xmas/ACK) (firewall detection)
- Discovery (ICMP/ICMPv6/NDP, host enumeration)
- Decoy (source address spoofing for anonymity)
- Idle/Zombie (maximum anonymity via third-party relay)

**Detection:** 85-90% accuracy
- Service detection (187 Nmap probes embedded)
- Protocol parsers (HTTP, SSH, SMB, MySQL, PostgreSQL)
- Ubuntu/Debian/RHEL version mapping from banners
- OS fingerprinting (16 probes, 2,600+ signatures)

**Performance:** Production-Ready
- Common ports: 5.15ms (29x faster than Nmap)
- Full 65K ports: 259ms (146x faster than Phase 3 baseline)
- Idle scan: 500-800ms/port (stealth tradeoff, 300x slower)
- Zero-copy packet building (<1% overhead)
- NUMA optimization support (enterprise-ready)

### Architecture Goals

1. **Modularity:** Independent, testable components with clear interfaces
2. **Performance:** Asynchronous I/O with zero-copy optimizations in hot paths
3. **Type Safety:** Leverage Rust's type system to prevent invalid states
4. **Progressive Enhancement:** Core functionality works without privileges; raw packets enhance capabilities
5. **Fail-Safe Defaults:** Conservative settings prevent accidental network disruption

---

## Design Philosophy

### 1. Modular Design

Each scanning technique, protocol handler, and output formatter exists as an independent, testable module. This enables:

- **Unit testing** of individual components in isolation
- **Feature flags** for conditional compilation (e.g., Lua plugins, Python bindings)
- **Code reuse** across different scanning modes
- **Parallel development** by multiple contributors

### 2. Asynchronous by Default

All I/O operations use Tokio's async runtime for maximum concurrency:

- **Non-blocking I/O** prevents thread starvation
- **Work-stealing scheduler** optimizes CPU utilization across cores
- **Backpressure handling** prevents memory exhaustion during large scans
- **Graceful degradation** under resource constraints

### 3. Zero-Copy Where Possible

Minimize memory allocations and copies in hot paths:

- **Memory-mapped I/O** for large result files
- **Borrowed data** throughout the packet processing pipeline
- **Pre-allocated buffers** for packet crafting
- **Lock-free data structures** for inter-thread communication

### 4. Type Safety

Leverage Rust's type system to prevent invalid state transitions:

```rust
// Example: Type-safe scan state machine
enum ScanState {
    Pending,
    Probing { attempts: u8, last_sent: Instant },
    Responded { packet: ResponsePacket },
    Timeout,
    Filtered,
}

// Compiler enforces state transitions
impl ScanState {
    fn on_response(self, packet: ResponsePacket) -> Self {
        match self {
            ScanState::Probing { .. } => ScanState::Responded { packet },
            _ => self, // Invalid transition ignored
        }
    }
}
```

---

## High-Level Architecture

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

### Layer Responsibilities

#### 1. User Interface Layer

- Parse command-line arguments and configuration files
- Present real-time progress and results
- Handle user interrupts and control signals
- Format output for human consumption

#### 2. Orchestration Layer

- Coordinate multi-phase scans (discovery → enumeration → deep inspection)
- Distribute work across worker threads
- Implement adaptive rate limiting and congestion control
- Aggregate and deduplicate results from multiple workers

#### 3. Scanning Engine Layer

- Implement specific scan techniques (SYN, UDP, ICMP, etc.)
- Perform service version detection and OS fingerprinting
- Execute stealth transformations (fragmentation, decoys, timing)
- Run plugin scripts for custom logic

#### 4. Network Protocol Layer

- Craft raw packets at Ethernet/IP/TCP/UDP layers
- Capture and parse network responses
- Implement custom TCP/IP stack for stateless operation
- Apply BPF filters for efficient packet capture

#### 5. Operating System Layer

- Platform-specific packet injection (AF_PACKET, BPF, Npcap)
- Privilege management (capabilities, setuid)
- Network interface enumeration and configuration

---

## Core Components

### 1. Scanner Scheduler

**Purpose:** Orchestrates scan jobs, manages target queues, distributes work across threads

**Key Responsibilities:**

- Parse and expand target specifications (CIDR, ranges, hostname lists)
- Randomize target order using permutation functions
- Shard targets across worker pools for parallel execution
- Coordinate multi-phase scans with dependency management

**Implementation Pattern:**

```rust
pub struct ScannerScheduler {
    targets: TargetRandomizer,
    workers: WorkerPool,
    phases: Vec<ScanPhase>,
    config: ScanConfig,
}

impl ScannerScheduler {
    pub async fn execute(&mut self) -> Result<ScanReport> {
        for phase in &self.phases {
            match phase {
                ScanPhase::Discovery => self.run_discovery().await?,
                ScanPhase::Enumeration => self.run_enumeration().await?,
                ScanPhase::DeepInspection => self.run_deep_inspection().await?,
            }
        }
        Ok(self.generate_report())
    }
}
```

**CDN Filtering Integration (Sprint 6.3 Bug Fix):**

ProRT-IP supports intelligent filtering of CDN infrastructure IPs to reduce scan targets by 30-70% for internet-scale operations.

**Critical Bug Fixed (2025-11-16):**

The scanner scheduler has two entry points for executing scans:

1. `scan_ports()` - Internal method with full CDN filtering logic (lines 271-314)
2. `execute_scan_ports()` - CLI entry point that LACKED filtering logic

**Problem:**

The CLI called `execute_scan_ports()` (via `crates/prtip-cli/src/main.rs:557`), which meant the `--skip-cdn` flag was non-functional in production despite filtering logic existing in `scan_ports()`.

**Root Cause:**

Architectural mismatch between two scan execution methods. CDN filtering was implemented in the internal method but not in the CLI-facing method.

**Fix (commit 19ba706):**

Added 38 lines of CDN filtering logic to `execute_scan_ports()` at line 658, matching the pattern from `scan_ports()`:

```rust
// In execute_scan_ports() - now includes CDN filtering
for target in targets {
    let original_hosts = target.expand_hosts();

    // Filter CDN IPs if enabled
    let hosts = if let Some(ref detector) = self.cdn_detector {
        let mut filtered = Vec::new();
        let mut skipped = 0;
        let mut provider_counts: std::collections::HashMap<CdnProvider, usize> =
            std::collections::HashMap::new();

        for host in original_hosts {
            if let Some(provider) = detector.detect(&host) {
                *provider_counts.entry(provider).or_insert(0) += 1;
                skipped += 1;
                debug!("Skipping CDN IP {}: {:?}", host, provider);
            } else {
                filtered.push(host);
            }
        }

        if skipped > 0 {
            let total = filtered.len() + skipped;
            let reduction_pct = (skipped * 100) / total;
            info!("Filtered {} CDN IPs ({}% reduction): {:?}",
                  skipped, reduction_pct, provider_counts);
        }

        if filtered.is_empty() {
            debug!("All hosts filtered (CDN detection), continuing to next target");
            continue;
        }

        debug!("Scanning {} hosts after CDN filtering", filtered.len());
        filtered
    } else {
        original_hosts
    };

    // Continue with filtered hosts...
}
```

**Verification:**

- 100% filtering rate confirmed across Cloudflare, AWS, Azure, Akamai, Fastly, Google Cloud
- Statistics logging working correctly
- Performance overhead: +37.5% for skip-all mode, **-22.8% for whitelist mode** (faster than baseline)

**Supported CDN Providers:**

- Cloudflare: 104.16.0.0/13 and 36 other ranges
- AWS CloudFront: 52.84.0.0/15 and 18 other ranges
- Azure CDN: 13.107.0.0/16 and 12 other ranges
- Akamai: 23.0.0.0/8 and 25 other ranges
- Fastly: 151.101.0.0/16 and 8 other ranges
- Google Cloud CDN: 35.186.0.0/16 and 15 other ranges

### 2. Two-Tier Rate Limiting System (Sprint 5.X, V3 Default)

**Purpose:** Responsible scanning with precise control over network load and target concurrency

ProRT-IP implements a two-tier rate limiting architecture combining Nmap-compatible hostgroup control with industry-leading AdaptiveRateLimiterV3 achieving **-1.8% average overhead** (faster than no rate limiting!):

#### Tier 1: Hostgroup Limiting (Nmap-Compatible)

**Purpose:** Control concurrent target-level parallelism (Nmap `--max-hostgroup` / `--min-hostgroup` compatibility)

**Key Responsibilities:**
- Semaphore-based concurrent target limiting
- Applies to "multi-port" scanners (TCP SYN, TCP Connect, Concurrent)
- Separate from packet-per-second rate limiting
- Dynamic adjustment based on scan size

**Implementation:**
```rust
pub struct HostgroupLimiter {
    semaphore: Arc<Semaphore>,
    max_hostgroup: usize,
    min_hostgroup: usize,
}
```

**CLI Flags:**
- `--max-hostgroup N` - Maximum concurrent targets (default: 100)
- `--min-hostgroup N` - Minimum concurrent targets (default: 1)

**Scanner Categories:**

**Multi-Port Scanners** (3): Hostgroup limiting applied
- ConcurrentScanner (adaptive parallelism)
- TcpConnectScanner (kernel stack)
- SynScanner (raw sockets)

**Per-Port Scanners** (4): No hostgroup limiting (per-port iteration)
- UdpScanner
- StealthScanner (FIN/NULL/Xmas/ACK)
- IdleScanner (zombie relay)
- DecoyScanner (source spoofing)

**Benefits:**
- Prevents overwhelming single targets
- Reduces memory usage (bounded concurrency)
- Nmap workflow compatibility

#### Tier 2: AdaptiveRateLimiterV3 (Default, Production-Ready)

**Status:** ✅ **Default Rate Limiter** (promoted 2025-11-02) achieving **-1.8% average overhead**

**Purpose:** Packet-per-second throttling with two-tier convergence and Relaxed memory ordering

**Key Innovations:**
- **Relaxed Memory Ordering:** Eliminates memory barriers (10-30ns savings per operation)
- **Two-Tier Convergence:** Hostgroup-level aggregate + per-target batch scheduling
- **Self-Correction:** Convergence compensates for stale atomic reads: `batch *= sqrt(target/observed)`
- **Batch Range:** 1.0 → 10,000.0 packets/batch

**Implementation:**
```rust
pub struct AdaptiveRateLimiterV3 {
    // Hostgroup-level tracking
    hostgroup_rate: Arc<AtomicU64>,
    hostgroup_last_time: Arc<AtomicU64>,

    // Per-target state
    batch_size: AtomicU64,  // f64 as u64 bits
    max_rate: u64,          // packets per second
}

pub type RateLimiter = AdaptiveRateLimiterV3;  // Type alias for backward compatibility
```

**Performance Achievement (Sprint 5.X):**

| Rate (pps) | Baseline (ms) | With V3 (ms) | Overhead | Performance Grade |
|------------|---------------|--------------|----------|-------------------|
| 10K        | 8.9 ± 1.4     | 8.2 ± 0.4    | **-8.2%** | ✅ Best Case |
| 50K        | 7.3 ± 0.3     | 7.2 ± 0.3    | **-1.8%** | ✅ Typical |
| 75K-200K   | 7.2-7.4       | 7.0-7.2      | **-3% to -4%** | ✅ Sweet Spot |
| 500K-1M    | 7.2-7.4       | 7.2-7.6      | **+0% to +3.1%** | ✅ Minimal |

**Average Overhead:** **-1.8%** (weighted by typical usage patterns)

**CLI Flags:**
- `--max-rate N` - Maximum packets per second (auto-uses V3)
- `-T0` through `-T5` - Timing templates (paranoid → insane)

**Benefits:**
- **Industry-leading performance** (faster than no rate limiting!)
- **15.2 percentage points** improvement over previous implementation
- **34% variance reduction** (more consistent timing)
- **Self-tuning** (no manual configuration needed)
- **Production-ready** (comprehensive testing, 1,466 tests passing)

#### Integration Pattern

**Multi-Port Scanners:**
```rust
// Acquire hostgroup slot (Tier 1)
let _permit = hostgroup_limiter.acquire().await;

// Scan all ports for this target
for port in ports {
    // Adaptive rate limiting (Tier 2: V3)
    rate_limiter.wait().await;
    send_packet(target_ip, port).await;
}
```

**Per-Port Scanners:**
```rust
// No hostgroup limiting (per-port iteration)
for (target_ip, port) in targets_x_ports {
    // Adaptive rate limiting (Tier 2: V3)
    rate_limiter.wait().await;
    send_packet(target_ip, port).await;
}
```

**Historical Context:**

Prior to Sprint 5.X (2025-11-02), ProRT-IP used a three-layer system:
1. **ICMP Type 3 Code 13 Detection** - Automatic backoff (removed, better handled at firewall level)
2. **Hostgroup Limiting** - Concurrent target control (retained as Tier 1)
3. **Governor Token Bucket** - Rate limiting with 40% overhead (replaced by V3)

The V3 promotion (Sprint 5.X Phase 5) consolidated to a cleaner two-tier architecture, achieving 15.2 percentage point overhead reduction (40% → -1.8%) while maintaining full Nmap compatibility.

**For comprehensive usage examples, CLI flags, performance tuning, and troubleshooting, see [docs/26-RATE-LIMITING-GUIDE.md](26-RATE-LIMITING-GUIDE.md).**

### 3. Result Aggregator

**Purpose:** Collect, deduplicate, and merge scan results from multiple workers

**Key Responsibilities:**

- Thread-safe result collection using lock-free queues
- Merge partial results for the same host/port (e.g., from retransmissions)
- Maintain canonical port state (open/closed/filtered)
- Stream results to output formatters without buffering entire dataset
- Handle out-of-order results from parallel workers

**Result Merging Logic:**

```rust
pub struct ResultAggregator {
    results: DashMap<TargetKey, TargetResult>,
    output_tx: mpsc::Sender<ScanResult>,
}

impl ResultAggregator {
    pub fn merge_result(&self, new_result: ScanResult) {
        self.results.entry(new_result.key())
            .and_modify(|existing| {
                // Merge logic: open > closed > filtered > unknown
                if new_result.state > existing.state {
                    existing.state = new_result.state;
                }
                existing.banners.extend(new_result.banners);
            })
            .or_insert(new_result.clone().into());
    }
}
```

### 4. Packet Crafting Engine

**Purpose:** Generate raw network packets for all scan types

**Key Responsibilities:**

- Build complete packets from Ethernet layer upward
- Apply stealth transformations (fragmentation, TTL manipulation, decoys)
- Calculate checksums including pseudo-headers
- Support source address/port spoofing

**Builder Pattern:**

```rust
let packet = TcpPacketBuilder::new()
    .source(local_ip, random_port())
    .destination(target_ip, target_port)
    .sequence(random_seq())
    .flags(TcpFlags::SYN)
    .window_size(65535)
    .tcp_option(TcpOption::Mss(1460))
    .tcp_option(TcpOption::WindowScale(7))
    .tcp_option(TcpOption::SackPermitted)
    .tcp_option(TcpOption::Timestamp { tsval: now(), tsecr: 0 })
    .build()?;
```

### 5. Packet Capture Engine

**Purpose:** Receive and parse network responses efficiently

**Key Responsibilities:**

- Configure BPF filters to reduce captured traffic (e.g., only TCP/UDP/ICMP to scanner)
- Parse responses into structured data with zero-copy where possible
- Match responses to probes using connection tracking or stateless validation
- Handle out-of-order packets and duplicates

**BPF Filter Example:**

```rust
// Capture only packets destined to our scanner
let filter = format!(
    "((tcp or udp) and dst host {}) or (icmp and host {})",
    local_ip, local_ip
);

pcap_handle.filter(&filter, true)?;
```

---

## IPv6 Dual-Stack Architecture

### Overview

ProRT-IP provides full IPv6 support across all scanning modes (Sprint 5.1, 100% scanner coverage). The architecture uses runtime protocol dispatch to handle both IPv4 and IPv6 transparently.

### Protocol Dispatch Pattern

```rust
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

// All scanners use this pattern
pub async fn scan_target(addr: SocketAddr) -> Result<PortState> {
    match addr.ip() {
        IpAddr::V4(ipv4) => scan_ipv4(ipv4, addr.port()).await,
        IpAddr::V6(ipv6) => scan_ipv6(ipv6, addr.port()).await,
    }
}
```

### IPv6 Packet Structure

**IPv6 Header (40 bytes, fixed size):**
- Version (4 bits): Always 6
- Traffic Class (8 bits): QoS/priority
- Flow Label (20 bits): Flow identification
- Payload Length (16 bits): Length of payload (excluding header)
- Next Header (8 bits): Protocol (TCP=6, UDP=17, ICMPv6=58)
- Hop Limit (8 bits): TTL equivalent
- Source Address (128 bits): IPv6 source
- Destination Address (128 bits): IPv6 destination

**Key Differences from IPv4:**
- No fragmentation in router (sender-only)
- No header checksum (delegated to link layer)
- No options in main header (use extension headers)
- Minimum MTU: 1280 bytes (vs 68 bytes IPv4)

### ICMPv6 & NDP Support

**ICMPv6 Message Types:**
- Type 1: Destination Unreachable (UDP port closed indication)
- Type 3: Time Exceeded (firewall drop indication)
- Type 128: Echo Request (ping6 equivalent)
- Type 129: Echo Reply (ping6 response)
- Type 135: Neighbor Solicitation (ARP equivalent)
- Type 136: Neighbor Advertisement (ARP reply equivalent)

**Neighbor Discovery Protocol (NDP):**
```
Target: 2001:db8::1234:5678
Solicited-Node Multicast: ff02::1:ff34:5678
                                    ^^^^^^^^
                                    Last 24 bits
```

NDP provides:
- Address resolution (ARP equivalent)
- Router discovery
- Neighbor unreachability detection
- Duplicate address detection

### Scanner-Specific IPv6 Handling

#### TCP Connect Scanner
- Uses kernel TCP stack (AF_INET6)
- No raw sockets required
- Full three-way handshake
- Automatic IPv4/IPv6 socket creation

#### SYN Scanner
- Raw socket (AF_PACKET/Npcap)
- IPv6 pseudo-header for TCP checksum
- 40-byte IPv6 header + 20-byte TCP header
- Requires root/administrator privileges

#### UDP Scanner
- Raw socket for receiving ICMPv6 responses
- ICMPv6 Type 1, Code 4: Port Unreachable (closed)
- Protocol-specific payloads (DNS, SNMP, etc.)
- Slower than TCP (10-100x) due to timeouts

#### Stealth Scanners (FIN/NULL/Xmas/ACK)
- Raw packet crafting with unusual flag combinations
- IPv6 firewalls may behave differently than IPv4
- Stateful firewalls often block these scans
- Useful for firewall detection

#### Discovery Engine
- ICMPv6 Echo Request/Reply (Type 128/129)
- NDP Neighbor Solicitation/Advertisement (Type 135/136)
- Solicited-node multicast for efficient discovery
- Link-local and global unicast support

#### Decoy Scanner
- Random IPv6 Interface Identifier (IID) generation
- Subnet-aware /64 decoy placement
- Avoids reserved IPv6 ranges (loopback, multicast, link-local, etc.)
- Source address spoofing (requires privileges)

### Performance Considerations

**IPv6 Overhead:**
- Header size: +100% (40 bytes vs 20 bytes)
- Checksum calculation: -50% CPU (no IP checksum, TCP/UDP only)
- Latency: +0-25% (depending on network)
- Throughput: -3% at 1Gbps (negligible)

**Optimization Strategies:**
- Zero-copy packet building (reuse buffers)
- Batched syscalls (sendmmsg/recvmmsg)
- Parallel processing (multi-threaded runtime)
- Adaptive concurrency (scale with scan size)

For comprehensive IPv6 usage examples, CLI flags, and troubleshooting, see [docs/23-IPv6-GUIDE.md](23-IPv6-GUIDE.md).

---

## Scanning Modes

### 1. Stateless Mode (Masscan-Style)

**Use Case:** Large-scale initial discovery (internet-wide sweeps)

**Characteristics:**

- No connection state maintained per target
- SipHash-based sequence numbers encode target identity
- Maximum transmission speed (10M+ packets/second capable)
- Minimal memory footprint (O(1) state regardless of target count)
- Target randomization via permutation functions

**Validation Without State:**

```rust
// Encode target in sequence number
fn generate_seq(ip: Ipv4Addr, port: u16, key: (u64, u64)) -> u32 {
    let mut hasher = SipHasher::new_with_keys(key.0, key.1);
    hasher.write(&ip.octets());
    hasher.write_u16(port);
    (hasher.finish() & 0xFFFFFFFF) as u32
}

// Validate response without lookup
fn validate_response(packet: &TcpPacket, key: (u64, u64)) -> Option<TargetId> {
    let expected_ack = generate_seq(
        packet.dest_ip(),
        packet.dest_port(),
        key
    ).wrapping_add(1);

    if packet.ack() == expected_ack {
        Some(TargetId { ip: packet.dest_ip(), port: packet.dest_port() })
    } else {
        None
    }
}
```

### 2. Stateful Mode (Nmap-Style)

**Use Case:** Detailed enumeration, stealth scanning, service detection

**Characteristics:**

- Per-connection state tracking in hash map
- Retransmission support with exponential backoff
- Congestion control based on RTT estimates
- Multiple scan types (SYN, FIN, NULL, Xmas, ACK, Window, Maimon)
- Deep packet inspection for OS fingerprinting

**State Machine:**

```rust
enum ConnectionState {
    Pending,
    SynSent { sent_at: Instant, seq: u32 },
    SynAckReceived { rtt: Duration },
    RstReceived,
    Timeout { attempts: u8 },
}
```

### 3. Hybrid Mode

**Use Case:** Balanced speed and depth (recommended for most scans)

**Workflow:**

1. **Fast Discovery:** Stateless SYN sweep across all targets (Phase 1)
2. **Filter Responsive:** Identify hosts/ports that responded (Phase 2)
3. **Deep Enumeration:** Stateful service detection on responsive targets only (Phase 3)

**Benefits:**

- 90%+ time reduction vs. full stateful scan
- Maintains accuracy for service detection
- Automatic fallback to stateful if response rate is low

### 4. Idle Scan Mode (Sprint 5.3, Zombie Scan)

**Use Case:** Maximum anonymity - scan target without revealing scanner's IP address

**Architecture:** Three-party relay system using "zombie" host as intermediary

**Key Components:**

1. **IP ID Tracking** - Monitor zombie's IP ID sequence (must be sequential)
2. **Spoofed SYN/ACK** - Send spoofed packets appearing to come from zombie
3. **Port State Inference** - Deduce target port state from IP ID increments

**How It Works:**

```
Scanner (Hidden)    Zombie (Relay)        Target
     │                   │                   │
     │  1. Probe         │                   │
     ├──────SYN/ACK───────>│                   │
     │                   │<──RST (IP ID=100)─┤
     │                   │                   │
     │  2. Spoof (src=Zombie, dst=Target:80)  │
     │───────────────────────────SYN────────────>│
     │                   │                   │
     │                   │<──SYN/ACK (open)──┤  If port open
     │                   ├──RST──────────────>│  (zombie responds)
     │                   │   (IP ID inc by 2) │
     │                   │                   │
     │  3. Probe again   │                   │
     ├──────SYN/ACK───────>│                   │
     │                   │<──RST (IP ID=102)─┤
     │                   │                   │
     │  Port OPEN (IP ID increased by 2)      │
```

**IP ID Pattern Analysis:**

- **+0:** Port filtered (no response from target)
- **+1:** Port closed (target sent RST to zombie, zombie ignored)
- **+2:** Port open (target sent SYN/ACK, zombie sent RST back)

**Zombie Requirements:**

1. **Sequential IP ID:** Must use incrementing IP ID (not random)
2. **Low Traffic:** Minimal background traffic (stable IP ID)
3. **Connectivity:** Reachable from scanner and target
4. **OS Compatibility:** Most Linux/BSD (not Windows/modern macOS)

**Implementation:**

```rust
pub struct IdleScanner {
    zombie_addr: IpAddr,
    target_addr: IpAddr,
    ipid_tracker: IpIdTracker,
    spoof_engine: SpoofEngine,
}

pub enum IpIdPattern {
    Sequential,      // +1 per packet (good zombie)
    Random,          // Unpredictable (bad zombie)
    Global,          // Shared counter (acceptable)
    PerDestination,  // Per-target counter (poor)
}

impl IdleScanner {
    pub async fn scan_port(&mut self, port: u16) -> Result<PortState> {
        // 1. Probe zombie (baseline IP ID)
        let ipid_before = self.probe_zombie().await?;

        // 2. Spoof SYN from zombie to target
        self.send_spoofed_syn(self.target_addr, port).await?;
        sleep(Duration::from_millis(500)).await;

        // 3. Probe zombie again (measure IP ID change)
        let ipid_after = self.probe_zombie().await?;

        // 4. Infer port state from IP ID delta
        match ipid_after.wrapping_sub(ipid_before) {
            0 => Ok(PortState::Filtered),
            1 => Ok(PortState::Closed),
            2 => Ok(PortState::Open),
            _ => Err(Error::ZombieUnstable),
        }
    }
}
```

**Performance Characteristics:**

- **Speed:** 500-800ms per port (300x slower than direct SYN scan)
- **Accuracy:** 99.5% (when zombie requirements met)
- **Anonymity:** Maximum (target logs only zombie IP, not scanner)
- **Detection:** Extremely difficult (no direct connection to target)

**Nmap Compatibility:**

ProRT-IP provides full Nmap `-sI` flag parity:

- Automatic zombie discovery (`-sI RND` equivalent)
- Manual zombie specification (`-sI <zombie_ip>`)
- Timing control (`-T0` to `-T5`)
- Port range support (single, ranges, lists)
- Verbose progress reporting
- Zombie suitability testing

**Limitations:**

- **IPv6:** Not yet implemented (planned for future sprint)
- **Firewalls:** Stateful firewalls may block spoofed packets
- **Speed:** 300x slower than direct scans (patience required)
- **Zombie Finding:** Requires sequential IP ID hosts (increasingly rare)

**For comprehensive usage examples, zombie discovery, troubleshooting, and security considerations, see [docs/25-IDLE-SCAN-GUIDE.md](25-IDLE-SCAN-GUIDE.md).**

---

## Data Flow

### Scan Execution Pipeline

```
[Target Specification]
         │
         ▼
[Target Parser & Expander]
         │
         ▼
[Target Randomizer] ──────────┐
         │                     │
         ▼                     │
[Worker Pool Distribution]    │ (Permutation index)
         │                     │
         ├──────┬──────┬───────┘
         ▼      ▼      ▼
    [Worker] [Worker] [Worker]
         │      │      │
         ▼      ▼      ▼
   [Packet Crafting Engine]
         │      │      │
         ▼      ▼      ▼
    [Raw Socket Transmission]
         │
         ▼
    [Network]
         │
         ▼
    [Packet Capture]
         │
         ▼
    [Response Parser]
         │
         ▼
    [Response Validator]
         │
         ▼
    [Result Aggregator]
         │
         ▼
    [Output Formatters]
         │
         ▼
    [Files/DB/Screen]
```

### Multi-Phase Scan Flow

```
Phase 1: Discovery
  ├─ ICMP ping sweep
  ├─ TCP SYN to common ports (top 100)
  ├─ UDP to common services
  └─ ARP scan (local networks)
         │
         ▼
  [Responsive Hosts List]
         │
         ▼
Phase 2: Enumeration
  ├─ TCP SYN to all 65535 ports (responsive hosts)
  ├─ UDP to expanded port list
  └─ Protocol-specific probes
         │
         ▼
  [Open Ports List]
         │
         ▼
Phase 3: Deep Inspection
  ├─ Service version detection
  ├─ OS fingerprinting
  ├─ Banner grabbing
  ├─ SSL/TLS certificate analysis
  └─ Script execution (NSE-like)
         │
         ▼
  [Complete Scan Report]
```

---

## Technology Stack

### Core Language

- **Rust 1.70+** (MSRV - Minimum Supported Rust Version)
  - Memory safety without garbage collection
  - Zero-cost abstractions
  - Fearless concurrency
  - Excellent cross-platform support

### Async Runtime

- **Tokio 1.35+** with multi-threaded scheduler
  - Work-stealing task scheduler
  - Efficient I/O event loop (epoll/kqueue/IOCP)
  - Semaphores and channels for coordination
  - Timer wheels for timeout management

### Networking

- **pnet 0.34+** for packet crafting and parsing
- **pcap 1.1+** for libpcap bindings
- **socket2 0.5+** for low-level socket operations
- **etherparse 0.14+** for fast zero-copy packet parsing

### Concurrency

- **crossbeam 0.8+** for lock-free data structures (queues, deques)
- **parking_lot 0.12+** for efficient mutexes (when locks are necessary)
- **rayon 1.8+** for data parallelism in analysis phases

### Data Storage

- **rusqlite 0.30+** for SQLite backend (default)
- **sqlx 0.7+** for PostgreSQL support (optional)
- **serde 1.0+** for JSON/TOML/XML serialization

### Platform-Specific

- **Linux:** `nix` crate for capabilities, `libc` for syscalls
- **Windows:** `winapi` for Winsock2, Npcap SDK
- **macOS:** `nix` crate for BPF device access

---

## Design Patterns

### 1. Builder Pattern

Used extensively for packet construction:

```rust
TcpPacketBuilder::new()
    .source(ip, port)
    .destination(target_ip, target_port)
    .flags(TcpFlags::SYN)
    .build()?
```

### 2. Strategy Pattern

Scan type selection:

```rust
trait ScanStrategy {
    async fn execute(&self, target: SocketAddr) -> Result<PortState>;
}

struct SynScan;
struct FinScan;
struct UdpScan;

// Each implements ScanStrategy with different logic
```

### 3. Observer Pattern

Result streaming:

```rust
trait ResultObserver {
    fn on_result(&mut self, result: ScanResult);
}

struct FileWriter { /* ... */ }
struct DatabaseWriter { /* ... */ }
struct TerminalPrinter { /* ... */ }

// Aggregator notifies all registered observers
```

### 4. Command Pattern

CLI argument handling:

```rust
enum ScanCommand {
    PortScan { targets: Vec<Target>, ports: PortRange },
    HostDiscovery { network: IpNetwork },
    ServiceDetection { targets: Vec<SocketAddr> },
}

impl ScanCommand {
    fn execute(&self) -> Result<()> {
        match self {
            ScanCommand::PortScan { .. } => { /* ... */ }
            // ...
        }
    }
}
```

### 5. Type State Pattern

Compile-time state enforcement:

```rust
struct Scanner<S> {
    state: PhantomData<S>,
    // ...
}

struct Unconfigured;
struct Configured;
struct Running;

impl Scanner<Unconfigured> {
    fn configure(self, config: ScanConfig) -> Scanner<Configured> {
        // ...
    }
}

impl Scanner<Configured> {
    fn start(self) -> Scanner<Running> {
        // Can only call start() if configured
        // ...
    }
}
```

---

## Architecture Benefits

### Performance

- **Async I/O** prevents blocking on slow network operations
- **Lock-free queues** eliminate contention in hot paths
- **Zero-copy parsing** reduces memory bandwidth requirements
- **NUMA awareness** keeps data local to processing cores

### Safety

- **Memory safety** prevents buffer overflows and use-after-free
- **Type safety** catches logic errors at compile time
- **Error handling** forces explicit handling of failures
- **Bounds checking** prevents array overruns (with negligible overhead)

### Maintainability

- **Modular design** enables independent testing and development
- **Clear interfaces** reduce coupling between components
- **Comprehensive logging** aids debugging and troubleshooting
- **Documentation tests** keep examples synchronized with code

### Extensibility

- **Plugin architecture** supports custom scan logic
- **Scripting engine** enables rapid prototyping
- **Output formatters** are independent and pluggable
- **Scan strategies** can be added without core changes

---

## Plugin System Architecture

The plugin system provides extensibility through sandboxed Lua plugins, enabling community-driven scanner enhancements.

### Core Components

**PluginManager** (`src/plugin/plugin_manager.rs`, 399 lines)
- Plugin discovery (scan `~/.prtip/plugins/` directory)
- Plugin loading (create Lua VM, parse metadata)
- Lifecycle management (on_load → execute → on_unload)
- Hot reload support (Arc<Mutex<Lua>> for thread safety)

**Plugin API** (`src/plugin/plugin_api.rs`, 522 lines)
- Trait-based design for type safety
- **ScanPlugin**: Pre-scan setup, post-scan cleanup hooks
  * `on_load()` - Initialize plugin
  * `pre_scan(target)` - Execute before scan
  * `post_scan(target, results)` - Execute after scan
  * `on_unload()` - Cleanup
- **OutputPlugin**: Custom result formatting
  * `on_load()` - Initialize plugin
  * `format_results(results)` - Format scan results
  * `on_unload()` - Cleanup
- **DetectionPlugin**: Service detection
  * `on_load()` - Initialize plugin
  * `analyze_banner(banner, port, protocol)` - Passive detection
  * `probe_service(target, port)` - Active detection
  * `on_unload()` - Cleanup

**Lua API** (`src/plugin/lua_api.rs`, 388 lines)
- Sandboxed Lua VM (mlua 0.11.3 with Lua 5.4)
- ProRT-IP API exposed to plugins:
  * `prtip.log(level, message)` - Logging
  * `prtip.get_target()` - Access scan target
  * `prtip.connect(host, port)` - Network operations (requires Network capability)
  * `prtip.send(socket, data)` - Send data
  * `prtip.receive(socket)` - Receive data
  * `prtip.close(socket)` - Close connection
  * `prtip.add_result(data)` - Add scan result

**Security Layer** (`src/plugin/security.rs`, 320 lines)
- Capabilities-based access control:
  * **Network**: Allow/deny network operations
  * **Filesystem**: Allow/deny file access
  * **System**: Allow/deny system calls
  * **Database**: Allow/deny database operations
- Lua VM sandboxing:
  * Remove `io` library (prevent arbitrary file access)
  * Remove `os` library (prevent command execution)
  * Remove `debug` library (prevent introspection)
  * Keep `string`, `table`, `math` (safe libraries)
- Resource limits:
  * Memory: 100MB per plugin
  * CPU: 5 seconds execution time
  * Instructions: 1 million Lua instructions maximum

**Metadata Parser** (`src/plugin/plugin_metadata.rs`, 272 lines)
- TOML parsing (plugin.toml files)
- Version validation (semver compatibility)
- Capability parsing (Network/Filesystem/System/Database)
- Dependency tracking

### Integration Flow

```
Scanner Core
    ↓
PluginManager.discover_plugins()
    ↓
PluginManager.load_plugin(path)
    ├── Parse plugin.toml (metadata)
    ├── Create sandboxed Lua VM
    ├── Register ProRT-IP API (prtip.*)
    ├── Load main.lua
    └── Call on_load() hook
    ↓
Plugin Execution
    ├── ScanPlugin: pre_scan() → scan → post_scan()
    ├── OutputPlugin: format_results()
    └── DetectionPlugin: analyze_banner() or probe_service()
    ↓
Results → Output System
    ↓
PluginManager.unload_plugin()
    └── Call on_unload() hook
```

### Security Model

**Deny-by-Default Capabilities:**
Plugins must explicitly request capabilities in plugin.toml:
```toml
[plugin.capabilities]
network = false      # Deny network access by default
filesystem = false   # Deny filesystem access by default
system = false       # Deny system calls by default
database = false     # Deny database access by default
```

**Sandboxing Enforcement:**
- Lua VM created with restricted standard library
- Dangerous libraries removed before plugin execution
- Resource limits enforced at runtime
- Thread-safe implementation (Arc<Mutex<Lua>>)

**Example Plugins:**
- **banner-analyzer** (DetectionPlugin): Passive analysis, no capabilities required
- **ssl-checker** (DetectionPlugin): Active probing, network capability required

### Performance

- Plugin overhead: <2% per plugin
- 5 plugins overhead: <10% total
- Plugin loading: <100ms
- Hot reload: Zero downtime

See [Plugin System Guide](30-PLUGIN-SYSTEM-GUIDE.md) for complete documentation.

---

## Next Steps

- Review [Development Roadmap](01-ROADMAP.md) for implementation phases
- Consult [Technical Specifications](02-TECHNICAL-SPECS.md) for detailed component design
- See [Development Setup Guide](03-DEV-SETUP.md) for build environment configuration
- Examine [Testing Strategy](06-TESTING.md) for quality assurance approach
