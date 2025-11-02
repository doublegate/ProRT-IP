# ProRT-IP WarScan: Architecture Overview

**Version:** 3.0
**Last Updated:** 2025-11-01
**Status:** Phase 5 IN PROGRESS (40% Complete) - v0.4.3 Advanced Features (IPv6 100%, Idle Scan, Rate Limiting)

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
- **Rate Limiting:** Three-layer system (ICMP monitoring, hostgroup control, adaptive rate limiting) prevents network disruption
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

### 2. Three-Layer Rate Limiting System (Sprint 5.4)

**Purpose:** Responsible scanning with precise control over network load, target concurrency, and DoS prevention

ProRT-IP implements a sophisticated three-layer rate limiting architecture combining Nmap-compatible hostgroup control, Masscan-inspired adaptive rate limiting, and ICMP-based automatic backoff:

#### Layer 1: ICMP Type 3 Code 13 Detection (Automatic Backoff)

**Purpose:** Detect and respond to administrative prohibitions from firewalls/routers

**Key Responsibilities:**
- Monitor for ICMP Type 3, Code 13 ("Communication Administratively Prohibited")
- Trigger automatic cooldown (exponential backoff: 1s, 2s, 4s, 8s, 16s max)
- Per-target tracking (one target's prohibition doesn't block others)
- Graceful recovery when network conditions improve

**Implementation:**
```rust
pub struct IcmpMonitor {
    prohibited_targets: DashMap<IpAddr, ProhibitionState>,
}

struct ProhibitionState {
    cooldown_until: Instant,
    backoff_level: u8,  // 0-4 (1s, 2s, 4s, 8s, 16s)
}
```

**Benefits:**
- Prevents network administrator complaints
- Automatic compliance with rate-limiting routers
- Graceful degradation (continues scanning other targets)

#### Layer 2: Hostgroup Limiting (Nmap-Compatible)

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

#### Layer 3: Adaptive Rate Limiting (Masscan-Inspired)

**Purpose:** Packet-per-second throttling with dynamic batch sizing (Masscan throttler algorithm)

**Key Responsibilities:**
- 256-bucket circular buffer tracks recent packet rates
- Convergence algorithm: `batch_size *= (target_rate / observed_rate)^0.5`
- Batch size range: 1.0 → 10,000.0 packets
- Handles system suspend/resume (reset on >1s gap)

**Implementation:**
```rust
pub struct AdaptiveRateLimiter {
    buckets: [AtomicU32; 256],
    bucket_index: AtomicU8,
    last_time: AtomicU64,
    batch_size: AtomicU64,  // f64 as u64 bits
    max_rate: u64,          // packets per second
}
```

**Performance Characteristics:**
- Low rates (<1K pps): Batch ~1, per-packet throttling
- Medium rates (1K-100K pps): Batch 2-100, reduced syscall overhead
- High rates (>100K pps): Batch 100-10,000, minimal overhead

**CLI Flags:**
- `--max-rate N` - Maximum packets per second

**Benefits:**
- Better high-rate performance than token bucket (batching)
- Self-tuning (no manual batch size configuration)
- Graceful system suspend handling

#### Integration Pattern

**Multi-Port Scanners:**
```rust
// Acquire hostgroup slot
let _permit = hostgroup_limiter.acquire().await;

// Scan all ports for this target
for port in ports {
    adaptive_limiter.wait().await;
    if icmp_monitor.is_prohibited(target_ip).await {
        backoff().await;
        continue;
    }
    send_packet(target_ip, port).await;
}
```

**Per-Port Scanners:**
```rust
// No hostgroup limiting (per-port iteration)
for (target_ip, port) in targets_x_ports {
    if icmp_monitor.is_prohibited(target_ip).await {
        backoff().await;
        continue;
    }
    send_packet(target_ip, port).await;
}
```

**For comprehensive usage examples and troubleshooting, see [docs/26-RATE-LIMITING-GUIDE.md](26-RATE-LIMITING-GUIDE.md).**

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

## Next Steps

- Review [Development Roadmap](01-ROADMAP.md) for implementation phases
- Consult [Technical Specifications](02-TECHNICAL-SPECS.md) for detailed component design
- See [Development Setup Guide](03-DEV-SETUP.md) for build environment configuration
- Examine [Testing Strategy](06-TESTING.md) for quality assurance approach
