# ProRT-IP WarScan: Technical Specifications

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Network Protocol Specifications](#network-protocol-specifications)
3. [Packet Format Specifications](#packet-format-specifications)
4. [Scanning Technique Specifications](#scanning-technique-specifications)
5. [Detection Engine Specifications](#detection-engine-specifications)
6. [Data Structures](#data-structures)
7. [File Formats](#file-formats)
8. [API Specifications](#api-specifications)

---

## System Requirements

### Hardware Requirements

#### Minimum Configuration

| Component | Requirement | Purpose |
|-----------|------------|---------|
| **CPU** | 2 cores @ 2.0 GHz | Basic scanning operations |
| **RAM** | 2 GB | Small network scans (<1000 hosts) |
| **Storage** | 100 MB | Binary + dependencies |
| **Network** | 100 Mbps | Basic throughput |

#### Recommended Configuration

| Component | Requirement | Purpose |
|-----------|------------|---------|
| **CPU** | 8+ cores @ 3.0 GHz | Parallel scanning, high throughput |
| **RAM** | 16 GB | Large network scans (100K+ hosts) |
| **Storage** | 1 GB SSD | Fast result database operations |
| **Network** | 1 Gbps+ | High-speed scanning |

#### High-Performance Configuration

| Component | Requirement | Purpose |
|-----------|------------|---------|
| **CPU** | 16+ cores @ 3.5+ GHz | Internet-scale scanning |
| **RAM** | 32+ GB | Stateful scanning of millions of targets |
| **Storage** | 10+ GB NVMe SSD | Massive result storage |
| **Network** | 10 Gbps+ | Maximum throughput (1M+ pps) |
| **NIC Features** | RSS, multi-queue, SR-IOV | Packet distribution across cores |

### Software Requirements

#### Operating Systems

**Linux (Primary Platform):**

- Kernel: 4.15+ (5.x+ recommended for eBPF/XDP)
- Distributions: Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch, RHEL 8+
- Packages: libpcap 1.9+, OpenSSL 1.1+, pkg-config

**Windows:**

- Version: Windows 10 (1809+) or Windows 11
- Requirements: Npcap 1.70+, Visual C++ Redistributable 2019+
- Privileges: Administrator required for raw packet access

**macOS:**

- Version: macOS 11.0 (Big Sur) or later
- Requirements: Xcode Command Line Tools, libpcap (included)
- Privileges: Root or access_bpf group membership

#### Runtime Dependencies

```toml
[dependencies]
# Core (required)
tokio = "1.35"              # Async runtime
pnet = "0.34"               # Packet manipulation
pcap = "1.1"                # Packet capture
serde = "1.0"               # Serialization

# Optional features
rusqlite = "0.30"           # SQLite support
sqlx = "0.7"                # PostgreSQL support (optional)
mlua = "0.9"                # Lua plugins (optional)
```

---

## Network Protocol Specifications

### Ethernet (Layer 2)

#### Frame Format

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Destination MAC Address                    |
+                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                               |                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               +
|                      Source MAC Address                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           EtherType           |          Payload...           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Field Specifications:**

- **Destination MAC:** 6 bytes - Target MAC address (or broadcast FF:FF:FF:FF:FF:FF)
- **Source MAC:** 6 bytes - Scanner's MAC address
- **EtherType:** 2 bytes - 0x0800 (IPv4), 0x0806 (ARP), 0x86DD (IPv6)

### IPv4 (Layer 3)

#### Header Format

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|Version|  IHL  |Type of Service|          Total Length         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Identification        |Flags|      Fragment Offset    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Time to Live |    Protocol   |         Header Checksum       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Source IP Address                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Destination IP Address                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Options (if IHL > 5)                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Field Specifications:**

- **Version:** 4 bits - Always 4 for IPv4
- **IHL:** 4 bits - Header length in 32-bit words (5-15, typically 5)
- **ToS/DSCP:** 8 bits - Type of Service / Differentiated Services
- **Total Length:** 16 bits - Entire packet size (header + data)
- **Identification:** 16 bits - Fragment identification
- **Flags:** 3 bits - DF (Don't Fragment), MF (More Fragments)
- **Fragment Offset:** 13 bits - Fragment position
- **TTL:** 8 bits - Time To Live (default: 64 for Linux, 128 for Windows)
- **Protocol:** 8 bits - 1 (ICMP), 6 (TCP), 17 (UDP)
- **Header Checksum:** 16 bits - One's complement checksum of header

### TCP (Layer 4)

#### Header Format

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |       Destination Port        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Sequence Number                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Acknowledgment Number                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Data |       |C|E|U|A|P|R|S|F|                               |
| Offset| Rsrvd |W|C|R|C|S|S|Y|I|            Window             |
|       |       |R|E|G|K|H|T|N|N|                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Checksum            |         Urgent Pointer        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Options (if Data Offset > 5)               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Field Specifications:**

- **Source Port:** 16 bits - Scanner's source port (typically random 1024-65535)
- **Destination Port:** 16 bits - Target port being scanned
- **Sequence Number:** 32 bits - Random for SYN, or SipHash-derived for stateless
- **Acknowledgment Number:** 32 bits - 0 for SYN scan
- **Data Offset:** 4 bits - Header length in 32-bit words (5-15)
- **Flags:** 8 bits - CWR, ECE, URG, ACK, PSH, RST, SYN, FIN
- **Window:** 16 bits - Receive window size (typical: 64240, 65535)
- **Checksum:** 16 bits - Checksum including pseudo-header
- **Urgent Pointer:** 16 bits - Usually 0

#### TCP Options

**Common Options Used in Scanning:**

| Option | Kind | Length | Purpose |
|--------|------|--------|---------|
| MSS | 2 | 4 | Maximum Segment Size (typical: 1460) |
| Window Scale | 3 | 3 | Window scaling factor (0-14) |
| SACK Permitted | 4 | 2 | Selective Acknowledgment support |
| Timestamp | 8 | 10 | Timestamps (TSval, TSecr) |
| NOP | 1 | 1 | Padding for alignment |
| EOL | 0 | 1 | End of options list |

**Standard Option Ordering (for OS fingerprinting):**

```
MSS, NOP, Window Scale, NOP, NOP, Timestamp, SACK Permitted, EOL
```

### UDP (Layer 4)

#### Header Format

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |       Destination Port        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|            Length             |           Checksum            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Payload...                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Field Specifications:**

- **Source Port:** 16 bits - Scanner's source port
- **Destination Port:** 16 bits - Target port
- **Length:** 16 bits - Header + payload length
- **Checksum:** 16 bits - Optional (0 if not computed)

### ICMP (Layer 3/4)

#### Echo Request/Reply Format

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Identifier          |        Sequence Number        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Payload...                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Type/Code Combinations:**

| Type | Code | Meaning |
|------|------|---------|
| 0 | 0 | Echo Reply |
| 3 | 0 | Network Unreachable |
| 3 | 1 | Host Unreachable |
| 3 | 3 | Port Unreachable (UDP scan) |
| 8 | 0 | Echo Request |
| 13 | 0 | Timestamp Request |
| 17 | 0 | Address Mask Request |

---

## Packet Format Specifications

### SYN Scan Packet

**Complete packet structure:**

```rust
// Ethernet Header (14 bytes)
[
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55,  // Dest MAC
    0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,  // Src MAC
    0x08, 0x00,                          // EtherType (IPv4)
]

// IPv4 Header (20 bytes)
[
    0x45,              // Version (4) + IHL (5)
    0x00,              // DSCP + ECN
    0x00, 0x2C,        // Total Length (44 bytes)
    0x12, 0x34,        // Identification (random)
    0x40, 0x00,        // Flags (DF) + Fragment Offset
    0x40,              // TTL (64)
    0x06,              // Protocol (TCP)
    0x00, 0x00,        // Checksum (calculated)
    0x0A, 0x00, 0x00, 0x01,  // Source IP (10.0.0.1)
    0x0A, 0x00, 0x00, 0x02,  // Dest IP (10.0.0.2)
]

// TCP Header with Options (24 bytes)
[
    0x30, 0x39,        // Source Port (12345)
    0x00, 0x50,        // Dest Port (80)
    0xAB, 0xCD, 0xEF, 0x12,  // Sequence (random/SipHash)
    0x00, 0x00, 0x00, 0x00,  // Acknowledgment (0)
    0x60,              // Data Offset (6 words = 24 bytes)
    0x02,              // Flags (SYN)
    0xFF, 0xFF,        // Window (65535)
    0x00, 0x00,        // Checksum (calculated)
    0x00, 0x00,        // Urgent Pointer (0)

    // Options (4 bytes)
    0x02, 0x04, 0x05, 0xB4,  // MSS: 1460
]
```

### UDP Scan Packet with DNS Payload

```rust
// Ethernet + IPv4 headers (same as above)

// UDP Header (8 bytes)
[
    0x30, 0x39,        // Source Port (12345)
    0x00, 0x35,        // Dest Port (53 - DNS)
    0x00, 0x1D,        // Length (29 bytes)
    0x00, 0x00,        // Checksum (0 or calculated)
]

// DNS Query Payload (21 bytes)
[
    0x12, 0x34,        // Transaction ID
    0x01, 0x00,        // Flags (standard query)
    0x00, 0x01,        // Questions: 1
    0x00, 0x00,        // Answer RRs: 0
    0x00, 0x00,        // Authority RRs: 0
    0x00, 0x00,        // Additional RRs: 0

    // Query: "." (root)
    0x00,              // Name length (root)
    0x00, 0x01,        // Type: A
    0x00, 0x01,        // Class: IN
]
```

---

## Scanning Technique Specifications

### TCP SYN Scan (-sS)

**Packet Sequence:**

```
Scanner                           Target
   |                                 |
   |-------- SYN ------------------>|  (1) Probe
   |                                 |
   |<------- SYN/ACK --------------|  (2a) Open
   |-------- RST ------------------>|  (3a) Reset
   |                                 |
   |<------- RST ------------------|  (2b) Closed
   |                                 |
   |         (timeout)               |  (2c) Filtered
```

**State Determination:**

- **Open:** Received SYN/ACK
- **Closed:** Received RST
- **Filtered:** No response after timeout + retries
- **Open|Filtered:** ICMP unreachable (type 3, code 1/2/3/9/10/13)

**Timing Parameters:**

| Template | Initial Timeout | Max Timeout | Max Retries | Scan Delay |
|----------|----------------|-------------|-------------|------------|
| T0 | 300 sec | 300 sec | 5 | 5 min |
| T1 | 15 sec | 15 sec | 5 | 15 sec |
| T2 | 1 sec | 10 sec | 5 | 0.4 sec |
| T3 | 1 sec | 10 sec | 2 | 0 |
| T4 | 500 ms | 1250 ms | 6 | 0 |
| T5 | 250 ms | 300 ms | 2 | 0 |

### UDP Scan (-sU)

**Packet Sequence:**

```
Scanner                           Target
   |                                 |
   |-------- UDP ------------------>|  (1) Probe
   |                                 |
   |<------- UDP Response ---------|  (2a) Open
   |                                 |
   |<------- ICMP Unreachable -----|  (2b) Closed (type 3, code 3)
   |                                 |
   |         (timeout)               |  (2c) Open|Filtered
```

**Protocol-Specific Payloads:**

| Port | Service | Payload | Expected Response |
|------|---------|---------|-------------------|
| 53 | DNS | Standard DNS query | DNS response |
| 161 | SNMP | GetRequest (community: public) | GetResponse |
| 123 | NTP | NTP version query | NTP response |
| 137 | NetBIOS | Name query | Name response |
| 111 | RPC | NULL procedure call | RPC response |

### Idle Scan (-sI)

**Packet Sequence:**

```
Scanner          Zombie                  Target
   |               |                       |
   |-- SYN/ACK -->|                       |  (1) Probe zombie IPID
   |<--- RST -----|                       |
   | (IPID: 1000) |                       |
   |               |                       |
   |               |<------ SYN ----------|  (2) Spoof SYN from zombie
   |               |                       |
   |               |------- SYN/ACK ----->|  (3a) If port open
   |               |<------ RST ----------|  (4a) Zombie responds
   |               |                       |
   |-- SYN/ACK -->|                       |  (5) Re-probe zombie
   |<--- RST -----|                       |
   | (IPID: 1002) |                       |  (IPID increased by 2 = OPEN)
```

**IPID Interpretation:**

- **+2:** Port open (zombie sent RST in response to SYN/ACK)
- **+1:** Port closed or filtered (no traffic from zombie)

---

## Detection Engine Specifications

### OS Fingerprinting

#### 16-Probe Sequence

| Probe # | Type | Target | Flags | Purpose |
|---------|------|--------|-------|---------|
| 1 | TCP | Open port | SYN | ISN, options, window |
| 2 | TCP | Open port | SYN (100ms later) | ISN delta (GCD calc) |
| 3 | TCP | Open port | SYN (100ms later) | ISN delta |
| 4 | TCP | Open port | SYN (100ms later) | ISN delta |
| 5 | TCP | Open port | SYN (100ms later) | ISN delta |
| 6 | TCP | Open port | SYN (100ms later) | ISN delta |
| 7 | ICMP | Any | Echo (TOS=0, code=0) | ICMP response |
| 8 | ICMP | Any | Echo (TOS=4, code=9) | ICMP error handling |
| 9 | TCP | Open port | ECN, SYN, CWR, ECE | ECN support |
| 10 | TCP | Closed port | NULL | Response to no flags |
| 11 | TCP | Closed port | SYN+FIN+URG+PSH | Unusual flags |
| 12 | TCP | Closed port | ACK | Window in RST |
| 13 | TCP | Closed port | ACK (window=128) | Firewall detection |
| 14 | TCP | Closed port | ACK (window=256) | Firewall detection |
| 15 | TCP | Open port | SYN (options vary) | Option handling |
| 16 | UDP | Closed port | Empty | ICMP unreachable |

#### Fingerprint Attributes

**TCP ISN Analysis:**

- **GCD:** Greatest common divisor of ISN deltas
- **ISR:** ISN counter rate (increments per second)
- **SP:** Sequence predictability index

**TCP Options:**

- **O:** Option ordering (M=MSS, W=WScale, T=Timestamp, S=SACK)
- **Example:** "MWTS" means MSS, Window Scale, Timestamp, SACK

**IP ID Generation:**

- **TI:** TCP IP ID generation (I=incremental, RI=random incremental, Z=zero)
- **CI:** Closed port IP ID generation
- **II:** ICMP IP ID generation

**Window Sizes:**

- **W:** TCP window sizes for each probe
- **Example:** "W=4000|8000" means 16384 or 32768

### Service Version Detection

#### Probe Intensity Levels

| Level | Probes Sent | Use Case |
|-------|-------------|----------|
| 0 | Registered only | Minimal (just check expected service) |
| 1 | Registered + NULL | Quick check |
| 2-6 | Incremental | Default (balanced) |
| 7 | Common + comprehensive | Default recommended |
| 8 | Nearly all | Thorough |
| 9 | All probes | Exhaustive |

#### Probe Format (nmap-service-probes style)

```
Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
rarity 1
ports 80,443,8080,8443

match http m|^HTTP/1\.[01] \d\d\d| p/HTTP/ v/$1/
match http m|^Server: ([^\r\n]+)| p/HTTP/ v/$1/
```

---

## Data Structures

### ScanResult

```rust
pub struct ScanResult {
    pub target: SocketAddr,
    pub port: u16,
    pub protocol: Protocol,
    pub state: PortState,
    pub service: Option<ServiceInfo>,
    pub banner: Option<String>,
    pub response_time: Duration,
    pub timestamp: SystemTime,
}

pub enum PortState {
    Open,
    Closed,
    Filtered,
    OpenFiltered,
    ClosedFiltered,
    Unknown,
}

pub struct ServiceInfo {
    pub name: String,
    pub version: Option<String>,
    pub product: Option<String>,
    pub cpe: Option<String>,
    pub os_hint: Option<String>,
}
```

### OsFingerprint

```rust
pub struct OsFingerprint {
    pub name: String,
    pub class: OsClass,
    pub cpe: Vec<String>,
    pub tests: FingerprintTests,
}

pub struct FingerprintTests {
    pub seq: SequenceGeneration,
    pub ops: TcpOptions,
    pub win: WindowSizes,
    pub ecn: EcnResponse,
    pub t1_t7: TcpTests,
    pub u1: UdpTest,
    pub ie: IcmpEchoTests,
}
```

---

## File Formats

### JSON Output Format

```json
{
  "scan_info": {
    "version": "1.0.0",
    "start_time": "2025-10-07T12:00:00Z",
    "end_time": "2025-10-07T12:05:30Z",
    "scan_type": ["SYN", "SERVICE"],
    "targets": ["10.0.0.0/24"],
    "ports": "1-1000"
  },
  "results": [
    {
      "ip": "10.0.0.1",
      "hostname": "server1.example.com",
      "state": "up",
      "ports": [
        {
          "port": 80,
          "protocol": "tcp",
          "state": "open",
          "service": {
            "name": "http",
            "product": "nginx",
            "version": "1.21.6",
            "cpe": "cpe:/a:nginx:nginx:1.21.6"
          },
          "banner": "nginx/1.21.6",
          "response_time_ms": 12
        }
      ],
      "os": {
        "name": "Linux 5.x",
        "accuracy": 95,
        "cpe": "cpe:/o:linux:linux_kernel:5"
      }
    }
  ],
  "statistics": {
    "total_hosts": 256,
    "hosts_up": 42,
    "total_ports_scanned": 42000,
    "ports_open": 156,
    "scan_duration_sec": 330
  }
}
```

### SQLite Schema

```sql
CREATE TABLE scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    scan_type TEXT NOT NULL,
    targets TEXT NOT NULL,
    ports TEXT NOT NULL,
    config JSON
);

CREATE TABLE hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    ip TEXT NOT NULL,
    hostname TEXT,
    state TEXT NOT NULL,
    os_name TEXT,
    os_accuracy INTEGER,
    FOREIGN KEY (scan_id) REFERENCES scans(id)
);

CREATE TABLE ports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    host_id INTEGER NOT NULL,
    port INTEGER NOT NULL,
    protocol TEXT NOT NULL,
    state TEXT NOT NULL,
    service_name TEXT,
    service_product TEXT,
    service_version TEXT,
    banner TEXT,
    response_time_ms INTEGER,
    FOREIGN KEY (host_id) REFERENCES hosts(id)
);

CREATE INDEX idx_scan_id ON hosts(scan_id);
CREATE INDEX idx_host_id ON ports(host_id);
CREATE INDEX idx_port ON ports(port);
CREATE INDEX idx_state ON ports(state);
```

---

## API Specifications

### Core Scanner API

```rust
pub struct Scanner {
    config: ScanConfig,
    runtime: Runtime,
}

impl Scanner {
    /// Create new scanner with configuration
    pub fn new(config: ScanConfig) -> Result<Self>;

    /// Execute scan and return results
    pub async fn execute(&self) -> Result<ScanReport>;

    /// Execute scan with progress callback
    pub async fn execute_with_progress<F>(&self, callback: F) -> Result<ScanReport>
    where
        F: Fn(ScanProgress) + Send + 'static;
}

pub struct ScanConfig {
    pub targets: Vec<Target>,
    pub ports: PortRange,
    pub scan_type: ScanType,
    pub timing: TimingTemplate,
    pub max_rate: Option<u32>,
    pub output: OutputConfig,
}
```

### Plugin API

```rust
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Initialize plugin
    fn init(&mut self, config: &PluginConfig) -> Result<()>;

    /// Called for each discovered port
    fn on_port_discovered(&mut self, result: &ScanResult) -> Result<()>;

    /// Called at scan completion
    fn on_scan_complete(&mut self, report: &ScanReport) -> Result<()>;

    /// Cleanup
    fn cleanup(&mut self) -> Result<()>;
}
```

---

## Next Steps

- Review [Architecture](00-ARCHITECTURE.md) for system design
- Consult [Implementation Guide](04-IMPLEMENTATION-GUIDE.md) for coding details
- See [API Reference](05-API-REFERENCE.md) for complete API documentation
