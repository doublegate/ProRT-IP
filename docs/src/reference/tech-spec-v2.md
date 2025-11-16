# Technical Specifications v2.0

Comprehensive technical specifications for ProRT-IP WarScan network scanner. This reference documents system requirements, protocol specifications, packet formats, scanning techniques, detection engines, data structures, and file formats.

**Version:** 2.0
**Last Updated:** November 2025
**Status:** Production

---

## System Requirements

### Hardware Requirements

#### Minimum Configuration (Small Networks)

| Component | Requirement | Purpose |
|-----------|------------|---------|
| **CPU** | 2 cores @ 2.0 GHz | Basic scanning operations |
| **RAM** | 2 GB | Small network scans (<1,000 hosts) |
| **Storage** | 100 MB | Binary + dependencies |
| **Network** | 100 Mbps | Basic throughput (~10K pps) |

**Supported Workloads:**
- Single-target scans
- Port range: 1-1000 ports
- Network size: <1,000 hosts
- Scan types: TCP SYN, Connect
- No service detection

#### Recommended Configuration (Medium Networks)

| Component | Requirement | Purpose |
|-----------|------------|---------|
| **CPU** | 8+ cores @ 3.0 GHz | Parallel scanning, high throughput |
| **RAM** | 16 GB | Large network scans (100K+ hosts) |
| **Storage** | 1 GB SSD | Fast result database operations |
| **Network** | 1 Gbps+ | High-speed scanning (100K pps) |

**Supported Workloads:**
- Multi-target scans (100K+ hosts)
- All 65,535 ports
- Scan types: All 8 types (SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle)
- Service detection + OS fingerprinting
- Database storage

#### High-Performance Configuration (Internet-Scale)

| Component | Requirement | Purpose |
|-----------|------------|---------|
| **CPU** | 16+ cores @ 3.5+ GHz | Internet-scale scanning |
| **RAM** | 32+ GB | Stateful scanning of millions of targets |
| **Storage** | 10+ GB NVMe SSD | Massive result storage |
| **Network** | 10 Gbps+ | Maximum throughput (1M+ pps) |
| **NIC Features** | RSS, multi-queue, SR-IOV | Packet distribution across cores |

**Supported Workloads:**
- Internet-wide IPv4 scans (3.7B hosts)
- All protocols (TCP, UDP, ICMP, IPv6)
- Stateless scanning at 10M+ pps
- NUMA-optimized packet processing
- Real-time streaming to database

**NIC Requirements:**
- **RSS (Receive Side Scaling):** Distribute packets across CPU cores
- **Multi-Queue:** Multiple TX/RX queues (16+ recommended)
- **SR-IOV:** Direct NIC hardware access for VMs
- **Hardware Offloading:** TCP checksum, segmentation offload

### Software Requirements

#### Operating Systems

**Linux (Primary Platform):**

**Supported Distributions:**
- Ubuntu 20.04+ LTS / 22.04+ LTS
- Debian 11+ (Bullseye) / 12+ (Bookworm)
- Fedora 35+ / 38+
- RHEL 8+ / 9+ (Red Hat Enterprise Linux)
- Arch Linux (rolling release)
- CentOS Stream 8+ / 9+

**Kernel Requirements:**
- **Minimum:** 4.15+ (for sendmmsg/recvmmsg syscalls)
- **Recommended:** 5.x+ (for eBPF/XDP support)
- **Optimal:** 6.x+ (latest performance improvements)

**System Packages:**
```bash
# Debian/Ubuntu
sudo apt install libpcap-dev pkg-config libssl-dev

# Fedora/RHEL/CentOS
sudo dnf install libpcap-devel pkgconfig openssl-devel

# Arch Linux
sudo pacman -S libpcap pkg-config openssl
```

**Runtime Libraries:**
- libpcap 1.9+ (packet capture)
- OpenSSL 1.1+ or 3.x (TLS certificate analysis)
- glibc 2.27+ (standard C library)

---

**Windows:**

**Supported Versions:**
- Windows 10 (version 1809+)
- Windows 11 (all versions)
- Windows Server 2016+
- Windows Server 2019+
- Windows Server 2022+

**Requirements:**
- **Npcap 1.70+** (packet capture driver) - [Download](https://npcap.com/)
- **Visual C++ Redistributable 2019+** (runtime libraries)
- **Administrator privileges** (required for raw packet access)

**Installation:**
```powershell
# Download and install Npcap
# Enable "WinPcap API-compatible Mode" during installation
# Restart computer after Npcap installation

# Verify installation
prtip --version
```

**Known Limitations:**
- FIN/NULL/Xmas scans not supported (Windows TCP/IP stack limitation)
- Administrator privileges required (no capability-based alternative)
- SYN discovery tests fail on loopback (127.0.0.1) - expected Npcap behavior

---

**macOS:**

**Supported Versions:**
- macOS 11.0+ (Big Sur) - Intel & Apple Silicon
- macOS 12.0+ (Monterey) - M1/M2 chips
- macOS 13.0+ (Ventura) - M1/M2/M3 chips
- macOS 14.0+ (Sonoma) - M1/M2/M3/M4 chips

**Requirements:**
- **Xcode Command Line Tools** (clang compiler)
- **libpcap** (pre-installed on macOS)
- **Root privileges OR access_bpf group membership**

**Setup BPF Access (Recommended):**
```bash
# Grant user BPF device access (avoids sudo)
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf

# Verify group membership
dseditgroup -o checkmember -m $(whoami) access_bpf

# Logout and login for changes to take effect
```

**Installation:**
```bash
# Remove quarantine attribute (macOS Gatekeeper)
xattr -d com.apple.quarantine /usr/local/bin/prtip

# Verify installation
prtip --version
```

---

#### Runtime Dependencies

**Rust Dependency Tree** (from `Cargo.toml`):

```toml
[dependencies]
# Core runtime (required)
tokio = { version = "1.35", features = ["full"] }
pnet = "0.34"                  # Packet manipulation
pcap = "1.1"                   # Packet capture (libpcap wrapper)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"             # JSON serialization

# Networking
socket2 = "0.5"                # Low-level socket operations
etherparse = "0.13"            # Ethernet/IP/TCP/UDP parsing

# Async utilities
tokio-util = "0.7"
futures = "0.3"
crossbeam = "0.8"              # Lock-free data structures

# CLI
clap = { version = "4.4", features = ["derive", "cargo"] }
colored = "2.0"                # Terminal colors

# Database (optional features)
rusqlite = { version = "0.30", optional = true }
sqlx = { version = "0.7", features = ["sqlite", "postgres"], optional = true }

# Plugin system (optional)
mlua = { version = "0.9", features = ["lua54", "send"], optional = true }

# Cryptography
ring = "0.17"                  # SipHash for stateless cookies
x509-parser = "0.15"           # TLS certificate parsing

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Feature Flags:**

```bash
# Default build (SQLite + plugins)
cargo build --release

# Minimal build (no database, no plugins)
cargo build --release --no-default-features

# PostgreSQL support
cargo build --release --features postgres

# All features
cargo build --release --all-features
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

| Field | Size | Description | Common Values |
|-------|------|-------------|---------------|
| **Destination MAC** | 6 bytes | Target MAC address | `FF:FF:FF:FF:FF:FF` (broadcast) |
| **Source MAC** | 6 bytes | Scanner's MAC address | Interface MAC |
| **EtherType** | 2 bytes | Protocol identifier | `0x0800` (IPv4), `0x0806` (ARP), `0x86DD` (IPv6) |

**ProRT-IP Implementation:**
- Automatically discovers gateway MAC via ARP for remote targets
- Uses broadcast MAC for LAN scans
- Supports VLAN tagging (802.1Q) when `--vlan` flag specified

---

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

| Field | Size | Description | ProRT-IP Default |
|-------|------|-------------|------------------|
| **Version** | 4 bits | IP version | `4` (IPv4) |
| **IHL** | 4 bits | Header length in 32-bit words | `5` (20 bytes, no options) |
| **ToS/DSCP** | 8 bits | Type of Service | `0` (default, configurable with `--tos`) |
| **Total Length** | 16 bits | Entire packet size | Variable (header + TCP/UDP) |
| **Identification** | 16 bits | Fragment identification | Random (per packet) |
| **Flags** | 3 bits | DF, MF, Reserved | `DF=1` (Don't Fragment) |
| **Fragment Offset** | 13 bits | Fragment position | `0` (no fragmentation) |
| **TTL** | 8 bits | Time To Live | `64` (Linux default), configurable with `--ttl` |
| **Protocol** | 8 bits | Upper layer protocol | `6` (TCP), `17` (UDP), `1` (ICMP) |
| **Header Checksum** | 16 bits | One's complement checksum | Calculated automatically |
| **Source IP** | 32 bits | Scanner's IP address | Interface IP (configurable with `-S`) |
| **Destination IP** | 32 bits | Target IP address | User-specified target |

**Fragmentation Support:**

ProRT-IP supports IP fragmentation for firewall evasion (`-f` flag):

```bash
# Fragment packets into 8-byte segments
prtip -f -sS -p 80,443 192.168.1.1

# Custom MTU (Maximum Transmission Unit)
prtip --mtu 16 -sS -p 80,443 192.168.1.1
```

---

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

| Field | Size | Description | ProRT-IP Default |
|-------|------|-------------|------------------|
| **Source Port** | 16 bits | Scanner's source port | Random 1024-65535 (configurable with `-g`) |
| **Destination Port** | 16 bits | Target port being scanned | User-specified (`-p` flag) |
| **Sequence Number** | 32 bits | Initial sequence number | Random (SYN scan), SipHash-derived (stateless) |
| **Acknowledgment Number** | 32 bits | ACK number | `0` (SYN scan), varies (Connect scan) |
| **Data Offset** | 4 bits | Header length in 32-bit words | `5` (20 bytes) or `6` (24 bytes with MSS) |
| **Flags** | 8 bits | CWR, ECE, URG, ACK, PSH, RST, SYN, FIN | Scan-type dependent |
| **Window** | 16 bits | Receive window size | `64240` (typical), `65535` (max) |
| **Checksum** | 16 bits | TCP checksum (includes pseudo-header) | Calculated automatically |
| **Urgent Pointer** | 16 bits | Urgent data pointer | `0` (not used in scanning) |

**TCP Flag Combinations by Scan Type:**

| Scan Type | SYN | FIN | RST | ACK | PSH | URG | Use Case |
|-----------|-----|-----|-----|-----|-----|-----|----------|
| SYN (`-sS`) | 1 | 0 | 0 | 0 | 0 | 0 | Stealth, most common |
| Connect (`-sT`) | 1 | 0 | 0 | 0 | 0 | 0 | Full TCP handshake |
| FIN (`-sF`) | 0 | 1 | 0 | 0 | 0 | 0 | Firewall evasion |
| NULL (`-sN`) | 0 | 0 | 0 | 0 | 0 | 0 | Stealth scan |
| Xmas (`-sX`) | 0 | 1 | 0 | 0 | 1 | 1 | Named for "lit up" flags |
| ACK (`-sA`) | 0 | 0 | 0 | 1 | 0 | 0 | Firewall rule detection |

#### TCP Options

**Common Options Used in Scanning:**

| Option | Kind | Length | Data | Purpose |
|--------|------|--------|------|---------|
| **EOL** (End of Option List) | 0 | 1 | - | Terminates option list |
| **NOP** (No Operation) | 1 | 1 | - | Padding for alignment |
| **MSS** (Maximum Segment Size) | 2 | 4 | 2 bytes | Maximum segment size (typical: 1460) |
| **Window Scale** | 3 | 3 | 1 byte | Window scaling factor (0-14) |
| **SACK Permitted** | 4 | 2 | - | Selective ACK support |
| **Timestamp** | 8 | 10 | 8 bytes | Timestamps (TSval, TSecr) |

**Standard Option Ordering (for OS fingerprinting):**

```
MSS, NOP, Window Scale, NOP, NOP, Timestamp, SACK Permitted, EOL
```

**Example (24-byte TCP header with MSS option):**

```
Data Offset: 6 (24 bytes)
Options:
  - MSS: Kind=2, Length=4, Value=1460
  - EOL: Kind=0
```

---

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

| Field | Size | Description | ProRT-IP Default |
|-------|------|-------------|------------------|
| **Source Port** | 16 bits | Scanner's source port | Random 1024-65535 |
| **Destination Port** | 16 bits | Target UDP port | User-specified (`-p`) |
| **Length** | 16 bits | Header + payload length | Variable (8 + payload_len) |
| **Checksum** | 16 bits | UDP checksum (optional) | Calculated (0 if disabled) |

**UDP Scan Challenges:**

UDP scanning is **10-100x slower** than TCP due to:
1. **No handshake:** Cannot determine "open" without application response
2. **ICMP rate limiting:** Many firewalls/routers rate-limit ICMP unreachable messages
3. **Stateless:** Requires protocol-specific payloads to elicit responses

**Protocol-Specific Payloads:**

ProRT-IP includes built-in payloads for common UDP services:

| Port | Service | Payload Type | Expected Response |
|------|---------|--------------|-------------------|
| 53 | DNS | Standard DNS A query | DNS response or ICMP unreachable |
| 161 | SNMP | GetRequest (community: public) | GetResponse or ICMP unreachable |
| 123 | NTP | NTP version 3 query | NTP response or ICMP unreachable |
| 137 | NetBIOS | NBNS name query | Name response or ICMP unreachable |
| 111 | RPC (Portmapper) | NULL procedure call | RPC response or ICMP unreachable |
| 500 | ISAKMP (IKE) | IKE SA INIT | IKE response or ICMP unreachable |
| 1900 | UPnP (SSDP) | M-SEARCH discovery | SSDP response or ICMP unreachable |

---

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

| Type | Code | Meaning | Use in ProRT-IP |
|------|------|---------|-----------------|
| 0 | 0 | Echo Reply | Host discovery confirmation |
| 3 | 0 | Network Unreachable | Target network filtered |
| 3 | 1 | Host Unreachable | Target host filtered |
| 3 | 3 | Port Unreachable | **UDP scan: port closed** |
| 3 | 9 | Network Prohibited | Firewall blocking |
| 3 | 10 | Host Prohibited | Firewall blocking |
| 3 | 13 | Admin Prohibited | **Rate limiting triggered** |
| 8 | 0 | Echo Request | Host discovery probe |
| 11 | 0 | Time Exceeded | Traceroute (TTL=0) |
| 13 | 0 | Timestamp Request | OS fingerprinting probe |
| 17 | 0 | Address Mask Request | OS fingerprinting probe |

**ICMP Rate Limiting Detection:**

ProRT-IP includes adaptive rate limiting based on ICMP Type 3 Code 13 responses:

```bash
# Enable adaptive rate limiting (monitors ICMP unreachable messages)
prtip -sS -p 1-1000 --adaptive-rate 192.168.1.0/24
```

**Backoff Levels:**
- Level 0: No backoff (initial state)
- Level 1: 2 seconds backoff
- Level 2: 4 seconds backoff
- Level 3: 8 seconds backoff
- Level 4: 16 seconds backoff (maximum)

---

## Packet Format Specifications

### TCP SYN Scan Packet (Complete Structure)

**Full packet: 58 bytes (Ethernet + IPv4 + TCP with MSS)**

```rust
// Ethernet Header (14 bytes)
[
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55,  // Destination MAC (target or gateway)
    0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,  // Source MAC (scanner's interface)
    0x08, 0x00,                          // EtherType: IPv4 (0x0800)
]

// IPv4 Header (20 bytes, no options)
[
    0x45,              // Version (4) + IHL (5 = 20 bytes)
    0x00,              // DSCP (0) + ECN (0)
    0x00, 0x2C,        // Total Length: 44 bytes (20 IP + 24 TCP)
    0x12, 0x34,        // Identification: random (e.g., 0x1234)
    0x40, 0x00,        // Flags: DF (0x4000) + Fragment Offset (0)
    0x40,              // TTL: 64 (Linux default)
    0x06,              // Protocol: TCP (6)
    0x00, 0x00,        // Header Checksum (calculated, placeholder here)
    0x0A, 0x00, 0x00, 0x01,  // Source IP: 10.0.0.1
    0x0A, 0x00, 0x00, 0x02,  // Destination IP: 10.0.0.2
]

// TCP Header with MSS Option (24 bytes)
[
    0x30, 0x39,        // Source Port: 12345 (random 1024-65535)
    0x00, 0x50,        // Destination Port: 80 (HTTP)
    0xAB, 0xCD, 0xEF, 0x12,  // Sequence Number: random or SipHash-derived
    0x00, 0x00, 0x00, 0x00,  // Acknowledgment: 0 (not ACK flag)
    0x60,              // Data Offset: 6 (24 bytes) + Reserved (0)
    0x02,              // Flags: SYN (0x02)
    0xFF, 0xFF,        // Window: 65535 (maximum)
    0x00, 0x00,        // Checksum (calculated, placeholder here)
    0x00, 0x00,        // Urgent Pointer: 0 (not urgent)

    // TCP Options (4 bytes)
    0x02, 0x04,        // MSS: Kind=2, Length=4
    0x05, 0xB4,        // MSS Value: 1460 (typical Ethernet MTU 1500 - 40)
]
```

**Checksum Calculation:**

**IPv4 Checksum:**
```rust
// One's complement sum of 16-bit words
let mut sum: u32 = 0;
for chunk in header.chunks(2) {
    sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
}
while (sum >> 16) > 0 {
    sum = (sum & 0xFFFF) + (sum >> 16);
}
let checksum = !(sum as u16);
```

**TCP Checksum (includes pseudo-header):**
```rust
// Pseudo-header: Source IP (4) + Dest IP (4) + Zero (1) + Protocol (1) + TCP Length (2)
let pseudo_header = [
    src_ip[0], src_ip[1], src_ip[2], src_ip[3],
    dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3],
    0x00,
    0x06,  // Protocol: TCP
    (tcp_len >> 8) as u8, tcp_len as u8,
];
// Then checksum pseudo_header + TCP header + payload
```

---

### UDP Scan Packet with DNS Payload

**Full packet: 56 bytes (Ethernet + IPv4 + UDP + DNS)**

```rust
// Ethernet Header (14 bytes) - same as above

// IPv4 Header (20 bytes)
[
    0x45,              // Version + IHL
    0x00,              // DSCP + ECN
    0x00, 0x2A,        // Total Length: 42 bytes (20 IP + 8 UDP + 14 DNS)
    0x56, 0x78,        // Identification: random
    0x00, 0x00,        // Flags: no DF + Fragment Offset: 0
    0x40,              // TTL: 64
    0x11,              // Protocol: UDP (17)
    0x00, 0x00,        // Checksum (calculated)
    0x0A, 0x00, 0x00, 0x01,  // Source IP
    0x0A, 0x00, 0x00, 0x02,  // Destination IP
]

// UDP Header (8 bytes)
[
    0x30, 0x39,        // Source Port: 12345
    0x00, 0x35,        // Destination Port: 53 (DNS)
    0x00, 0x16,        // Length: 22 bytes (8 UDP + 14 DNS)
    0x00, 0x00,        // Checksum: 0 (optional for IPv4)
]

// DNS Query Payload (14 bytes)
[
    0x12, 0x34,        // Transaction ID: random
    0x01, 0x00,        // Flags: Standard query, recursion desired
    0x00, 0x01,        // Questions: 1
    0x00, 0x00,        // Answer RRs: 0
    0x00, 0x00,        // Authority RRs: 0
    0x00, 0x00,        // Additional RRs: 0

    // Query for "." (DNS root)
    0x00,              // Name: root (zero-length label)
    0x00, 0x01,        // Type: A (host address)
    0x00, 0x01,        // Class: IN (Internet)
]
```

---

## Scanning Technique Specifications

### TCP SYN Scan (-sS)

**Packet Sequence Diagram:**

```
Scanner                           Target
   |                                 |
   |-------- SYN ------------------>|  (1) Probe: SYN flag set
   |                                 |
   |<------- SYN/ACK --------------|  (2a) OPEN: Responds with SYN/ACK
   |-------- RST ------------------>|  (3a) Reset connection (stealth)
   |                                 |
   |<------- RST ------------------|  (2b) CLOSED: Responds with RST
   |                                 |
   |         (timeout)               |  (2c) FILTERED: No response
   |                                 |
   |<------- ICMP Unreachable -----|  (2d) FILTERED: ICMP Type 3
```

**State Determination Logic:**

| Response | Port State | Flags | Code |
|----------|-----------|-------|------|
| **SYN/ACK received** | **Open** | TCP: SYN+ACK | - |
| **RST received** | **Closed** | TCP: RST | - |
| **ICMP Type 3 Code 1/2/3/9/10/13** | **Filtered** | - | ICMP unreachable |
| **No response after timeout + retries** | **Filtered** | - | - |

**Timing Parameters by Template:**

| Template | Initial Timeout | Max Timeout | Max Retries | Scan Delay |
|----------|----------------|-------------|-------------|------------|
| **T0 (Paranoid)** | 300 sec | 300 sec | 5 | 5 min |
| **T1 (Sneaky)** | 15 sec | 15 sec | 5 | 15 sec |
| **T2 (Polite)** | 1 sec | 10 sec | 5 | 0.4 sec |
| **T3 (Normal)** | 1 sec | 10 sec | 2 | 0 |
| **T4 (Aggressive)** | 500 ms | 1250 ms | 6 | 0 |
| **T5 (Insane)** | 250 ms | 300 ms | 2 | 0 |

**Example:**

```bash
# Normal SYN scan (T3)
prtip -sS -p 80,443 192.168.1.1

# Aggressive scan (T4 - faster)
prtip -T4 -sS -p 1-10000 192.168.1.0/24

# Paranoid scan (T0 - stealth)
prtip -T0 -sS -p 22,23,3389 target.com
```

---

### UDP Scan (-sU)

**Packet Sequence Diagram:**

```
Scanner                           Target
   |                                 |
   |-------- UDP ------------------>|  (1) Probe: UDP packet (with/without payload)
   |                                 |
   |<------- UDP Response ---------|  (2a) OPEN: Application responds
   |                                 |
   |<------- ICMP Type 3 Code 3 ---|  (2b) CLOSED: Port unreachable
   |                                 |
   |<------- ICMP Type 3 Other -----|  (2c) FILTERED: Other unreachable codes
   |                                 |
   |         (timeout)               |  (2d) OPEN|FILTERED: No response
```

**State Determination Logic:**

| Response | Port State |
|----------|-----------|
| **UDP response received** | **Open** |
| **ICMP Type 3 Code 3 (Port Unreachable)** | **Closed** |
| **ICMP Type 3 Code 1/2/9/10/13** | **Filtered** |
| **No response after timeout** | **Open\|Filtered** (indeterminate) |

**UDP Scan Optimization:**

ProRT-IP uses protocol-specific payloads to increase accuracy:

```bash
# UDP scan with protocol-specific probes
prtip -sU -p 53,161,123,137,111,500 192.168.1.1
```

**Known Limitations:**
- **10-100x slower than TCP:** ICMP rate limiting on routers/firewalls
- **Open\|Filtered:** Cannot distinguish without application response
- **Firewall Detection:** Many firewalls silently drop UDP packets

---

### Idle Scan (-sI zombie_host)

**Packet Sequence Diagram:**

```
Scanner          Zombie (Idle Host)        Target
   |               |                       |
   |-- SYN/ACK -->|                       |  (1) Probe zombie IPID
   |<--- RST -----|                       |
   | (IPID: 1000) |                       |
   |               |                       |
   |               |<------ SYN ----------|  (2) Spoof SYN from zombie to target
   |               |                       |
   |               |------- SYN/ACK ----->|  (3a) If port OPEN: Target sends SYN/ACK
   |               |<------ RST ----------|  (4a) Zombie responds with RST (IPID increments)
   |               |                       |
   |               |------- RST ---------->|  (3b) If port CLOSED: Target sends RST
   |               |       (no response)   |  (4b) Zombie does nothing (IPID unchanged)
   |               |                       |
   |-- SYN/ACK -->|                       |  (5) Re-probe zombie IPID
   |<--- RST -----|                       |
   | (IPID: 1002) |                       |  (IPID increased by 2 = PORT OPEN)
   | (IPID: 1001) |                       |  (IPID increased by 1 = PORT CLOSED/FILTERED)
```

**IPID Interpretation:**

| IPID Delta | Port State | Explanation |
|------------|-----------|-------------|
| **+2** | **Open** | Zombie sent RST in response to target's SYN/ACK (incremented by 1), plus scanner's probe (+1) |
| **+1** | **Closed** or **Filtered** | Only scanner's probe incremented IPID (no traffic from zombie) |
| **>+2** | **Indeterminate** | Zombie is receiving other traffic (not idle) |

**Zombie Host Requirements:**

1. **Idle:** Little to no network traffic (predictable IPID sequence)
2. **Incremental IPID:** IP ID increments globally (not per-connection)
3. **Unfiltered:** Responds to unsolicited SYN/ACK with RST

**Zombie Suitability Test:**

```bash
# Test if host is suitable as zombie
prtip --idle-scan-test potential_zombie_host

# Example output:
# Zombie Analysis: 192.168.1.100
#   IPID Generation: Incremental (GOOD)
#   Traffic Level: <5 pps (IDLE)
#   Responds to SYN/ACK: Yes (SUITABLE)
#   Recommendation: SUITABLE for idle scan
```

**Idle Scan Usage:**

```bash
# Perform idle scan using zombie host
prtip -sI 192.168.1.100 -p 80,443 target.com
```

**Advantages:**
- **Maximum anonymity:** Target logs zombie's IP, not scanner's
- **Firewall bypass:** Bypasses source IP-based filtering
- **No packets from scanner to target:** Ultimate stealth

**Disadvantages:**
- **Requires idle zombie host:** Difficult to find suitable zombies
- **Slower:** Multiple probes per port (zombie probe → spoof → zombie probe)
- **99.5% accuracy:** Not 100% due to network timing variations

---

## Detection Engine Specifications

### OS Fingerprinting

#### 16-Probe Sequence

ProRT-IP implements Nmap-compatible OS fingerprinting with 16 distinct probes:

| Probe # | Type | Target Port | Flags | Purpose | Key Attributes |
|---------|------|------------|-------|---------|----------------|
| **1** | TCP | Open port | SYN | Initial SYN probe | ISN, TCP options, window size |
| **2** | TCP | Open port | SYN | ISN probe (100ms later) | ISN delta calculation |
| **3** | TCP | Open port | SYN | ISN probe (100ms later) | ISN delta calculation |
| **4** | TCP | Open port | SYN | ISN probe (100ms later) | ISN delta calculation |
| **5** | TCP | Open port | SYN | ISN probe (100ms later) | ISN delta calculation |
| **6** | TCP | Open port | SYN | ISN probe (100ms later) | ISN delta (GCD calculation) |
| **7** | ICMP | Any | Echo (TOS=0, code=0) | ICMP echo response | DF flag, TTL, TOS handling |
| **8** | ICMP | Any | Echo (TOS=4, code=9) | ICMP error handling | Non-standard code handling |
| **9** | TCP | Open port | ECN, SYN, CWR, ECE | ECN support test | ECN echo, option handling |
| **10** | TCP | Closed port | NULL | No flags set | Response to NULL scan |
| **11** | TCP | Closed port | SYN+FIN+URG+PSH | Unusual flags | Unusual flags handling |
| **12** | TCP | Closed port | ACK | ACK probe | Window value in RST |
| **13** | TCP | Closed port | ACK (window=128) | Firewall detection | Window scaling detection |
| **14** | TCP | Closed port | ACK (window=256) | Firewall detection | Window scaling detection |
| **15** | TCP | Open port | SYN (options vary) | Option handling | Option ordering, values |
| **16** | UDP | Closed port | Empty UDP packet | ICMP unreachable | ICMP response analysis |

#### Fingerprint Attributes Analyzed

**TCP Initial Sequence Number (ISN) Analysis:**

| Attribute | Description | Calculation |
|-----------|-------------|-------------|
| **GCD** | Greatest common divisor of ISN deltas | `gcd(Δ1, Δ2, Δ3, Δ4, Δ5)` where Δn = ISN(n+1) - ISN(n) |
| **ISR** | ISN counter rate (increments per second) | `avg(Δ1, Δ2, Δ3, Δ4, Δ5) / 0.1s` |
| **SP** | Sequence predictability index | Variance in ISN deltas (0-255, 0=random, 255=sequential) |

**Example:**
```
Probe 1: ISN = 1000000
Probe 2: ISN = 1001250  (Δ1 = 1250)
Probe 3: ISN = 1002500  (Δ2 = 1250)
Probe 4: ISN = 1003750  (Δ3 = 1250)
Probe 5: ISN = 1005000  (Δ4 = 1250)
Probe 6: ISN = 1006250  (Δ5 = 1250)

GCD = 1250
ISR = 1250 / 0.1s = 12,500 increments/sec
SP = 0 (no variance, highly predictable)
```

**TCP Options Encoding:**

ProRT-IP records the exact ordering and values of TCP options:

| Code | Option | Example |
|------|--------|---------|
| **M** | MSS (Maximum Segment Size) | `M1460` (MSS value 1460) |
| **W** | Window Scale | `W7` (scale factor 7) |
| **T** | Timestamp | `T` (timestamp present) |
| **S** | SACK Permitted | `S` (SACK supported) |
| **E** | EOL (End of Option List) | `E` |
| **N** | NOP (No Operation) | `N` |

**Example Option String:**
```
Options: MNWNNTS
Breakdown:
  M = MSS (1460)
  N = NOP (padding)
  W = Window Scale (7)
  N = NOP (padding)
  N = NOP (padding)
  T = Timestamp
  S = SACK Permitted
```

**IP ID Generation Patterns:**

| Pattern | Code | Description | Example OSes |
|---------|------|-------------|--------------|
| **Incremental** | `I` | Globally incremental IP ID | Windows, older Linux |
| **Random Incremental** | `RI` | Random but incremental | Some BSD variants |
| **Zero** | `Z` | Always 0 | Some embedded systems |
| **Broken Increment** | `BI` | Incremental with wrap issues | Rare |

**Example Fingerprint:**

```
OS: Linux 5.x
GCD: 1
ISR: 12800
SP: 0-5
TI: I  (TCP IPID incremental)
CI: I  (Closed port IPID incremental)
II: I  (ICMP IPID incremental)
SS: S  (SYN scan IPID sequence)
TS: 100HZ  (TCP timestamp frequency)
Options: MWNNTS
Window: 5840  (typical Linux)
```

#### Fingerprint Database

ProRT-IP includes a comprehensive OS fingerprint database:

```rust
// Location: crates/prtip-core/src/os_db.rs
pub struct OsDatabase {
    fingerprints: Vec<OsFingerprint>,  // 2,600+ fingerprints
    index: HashMap<String, Vec<usize>>,  // Fast lookup by attribute
}

pub struct OsFingerprint {
    pub name: String,              // "Linux 5.10-5.15"
    pub class: OsClass,            // OS family, vendor, type
    pub cpe: Vec<String>,          // CPE identifiers
    pub tests: FingerprintTests,   // All 16 probe results
}
```

**Database Statistics:**
- **Total Fingerprints:** 2,600+
- **OS Families:** 15+ (Linux, Windows, BSD, macOS, iOS, Android, etc.)
- **Vendors:** 200+ (Microsoft, Apple, Cisco, Juniper, etc.)
- **Match Accuracy:** 85-95% for common OSes

---

### Service Version Detection

#### Probe Intensity Levels

ProRT-IP supports configurable probe intensity (0-9):

| Level | Probes Sent | Duration | Use Case |
|-------|-------------|----------|----------|
| **0** | Registered probes only | <1 sec | Expected service (e.g., HTTP on port 80) |
| **1** | Registered + NULL probe | ~2 sec | Quick check with null probe fallback |
| **2-6** | Incremental | 3-8 sec | Balanced (increasingly thorough) |
| **7** | Common + comprehensive | ~10 sec | **Default recommended** |
| **8** | Nearly all probes | ~20 sec | Thorough detection |
| **9** | All 187 probes | ~30 sec | Exhaustive (slow) |

**Example:**

```bash
# Default intensity (level 7)
prtip -sV -p 80,443 192.168.1.1

# Minimal intensity (level 0)
prtip -sV --version-intensity 0 -p 80,443 192.168.1.1

# Exhaustive intensity (level 9)
prtip -sV --version-intensity 9 -p 1-1000 192.168.1.1
```

#### nmap-service-probes Format

ProRT-IP uses Nmap-compatible service probe definitions:

```
Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
rarity 1
ports 80,443,8080,8443,8000,8888,9000

match http m|^HTTP/1\.[01] (\d\d\d)| p/HTTP/ v/$1/
match http m|^Server: ([^\r\n]+)| p/$1/
match http m|^Server: Apache/([^\s]+)| p/Apache httpd/ v/$1/
match nginx m|^Server: nginx/([^\s]+)| p/nginx/ v/$1/

Probe TCP TLSSessionReq q|\x16\x03\x00\x00S\x01\x00\x00O\x03\x00|
rarity 2
ports 443,8443,8444,9443,4443,10443,12443,18091,18092

match ssl m|^\x16\x03[\x00\x01\x02\x03]|s p/SSL/ v/TLSv1/
```

**Probe Components:**

| Component | Description | Example |
|-----------|-------------|---------|
| **Probe** | Protocol + Name | `TCP GetRequest` |
| **q\|...\|** | Query payload (hex or string) | `q|GET / HTTP/1.0\r\n\r\n|` |
| **rarity** | Probe frequency (1=common, 9=rare) | `rarity 1` |
| **ports** | Target ports | `ports 80,443,8080` |
| **match** | Regex pattern | `m|^HTTP/1\.[01] (\d\d\d)|` |
| **p/** | Product name | `p/Apache httpd/` |
| **v/** | Version | `v/$1/` (from capture group) |

**Probe Database:**

- **Total Probes:** 187
- **Protocols Supported:** HTTP, HTTPS, FTP, SSH, SMTP, POP3, IMAP, Telnet, RDP, VNC, MySQL, PostgreSQL, MongoDB, Redis, and 50+ more
- **Match Patterns:** 1,200+ regex patterns

**Detection Accuracy:**
- **Common Services:** 85-90% (HTTP, HTTPS, SSH, FTP)
- **Databases:** 80-85% (MySQL, PostgreSQL, MongoDB)
- **Proprietary Protocols:** 60-70% (vendor-specific)

---

## Data Structures

### ScanResult

Primary result structure for individual port scan results:

```rust
pub struct ScanResult {
    /// Target socket address (IP:port)
    pub target: SocketAddr,

    /// Port number (1-65535)
    pub port: u16,

    /// Protocol (TCP, UDP, SCTP)
    pub protocol: Protocol,

    /// Port state (Open, Closed, Filtered, etc.)
    pub state: PortState,

    /// Detected service information (if -sV used)
    pub service: Option<ServiceInfo>,

    /// Banner grabbed from service (if available)
    pub banner: Option<String>,

    /// Response time (latency)
    pub response_time: Duration,

    /// Timestamp of scan
    pub timestamp: SystemTime,
}

pub enum Protocol {
    Tcp,
    Udp,
    Sctp,
}

pub enum PortState {
    Open,           // Port accepting connections
    Closed,         // Port actively rejecting connections (RST)
    Filtered,       // Firewall/filter blocking access
    OpenFiltered,   // UDP scan: could be open or filtered
    ClosedFiltered, // Rare: IPID idle scan
    Unknown,        // Unexpected response
}

pub struct ServiceInfo {
    /// Service name (e.g., "http", "ssh", "mysql")
    pub name: String,

    /// Service version (e.g., "2.4.52")
    pub version: Option<String>,

    /// Product name (e.g., "Apache httpd", "OpenSSH")
    pub product: Option<String>,

    /// CPE identifier (if available)
    pub cpe: Option<String>,

    /// OS hint from service banner
    pub os_hint: Option<String>,
}
```

**Example JSON Serialization:**

```json
{
  "target": "192.168.1.100:80",
  "port": 80,
  "protocol": "Tcp",
  "state": "Open",
  "service": {
    "name": "http",
    "version": "2.4.52",
    "product": "Apache httpd",
    "cpe": "cpe:/a:apache:http_server:2.4.52",
    "os_hint": "Ubuntu"
  },
  "banner": "Apache/2.4.52 (Ubuntu)",
  "response_time_ms": 12,
  "timestamp": "2025-11-15T10:30:00Z"
}
```

---

### OsFingerprint

OS fingerprinting data structure:

```rust
pub struct OsFingerprint {
    /// OS name (e.g., "Linux 5.10-5.15")
    pub name: String,

    /// OS classification
    pub class: OsClass,

    /// CPE identifiers
    pub cpe: Vec<String>,

    /// All fingerprint test results
    pub tests: FingerprintTests,
}

pub struct OsClass {
    /// OS family (Linux, Windows, BSD, etc.)
    pub family: String,

    /// Vendor (Microsoft, Apple, Red Hat, etc.)
    pub vendor: String,

    /// Device type (general purpose, router, firewall, etc.)
    pub device_type: String,

    /// Generation (e.g., "5.x", "Windows 10", "iOS 14")
    pub generation: String,
}

pub struct FingerprintTests {
    /// Sequence generation (ISN analysis)
    pub seq: SequenceGeneration,

    /// TCP options from probes
    pub ops: TcpOptions,

    /// Window sizes from probes
    pub win: WindowSizes,

    /// ECN response (probe 9)
    pub ecn: EcnResponse,

    /// TCP tests (probes 1-6, 10-15)
    pub t1_t7: TcpTests,

    /// UDP test (probe 16)
    pub u1: UdpTest,

    /// ICMP echo tests (probes 7-8)
    pub ie: IcmpEchoTests,
}

pub struct SequenceGeneration {
    /// Greatest common divisor of ISN deltas
    pub gcd: u32,

    /// ISN counter rate (increments/sec)
    pub isr: u32,

    /// Sequence predictability (0-255)
    pub sp: u8,

    /// TCP IPID sequence type
    pub ti: IpIdType,

    /// Closed port IPID sequence
    pub ci: IpIdType,

    /// ICMP IPID sequence
    pub ii: IpIdType,

    /// SYN scan IPID sequence
    pub ss: IpIdType,

    /// TCP timestamp frequency
    pub ts: TimestampFrequency,
}

pub enum IpIdType {
    Incremental,
    RandomIncremental,
    Zero,
    BrokenIncrement,
}
```

---

## File Formats

### JSON Output Format

ProRT-IP JSON output follows this schema:

```json
{
  "scan_info": {
    "version": "0.5.0",
    "start_time": "2025-11-15T10:00:00Z",
    "end_time": "2025-11-15T10:05:30Z",
    "scan_type": ["SYN", "SERVICE"],
    "targets": ["192.168.1.0/24"],
    "ports": "1-1000",
    "timing_template": "Normal",
    "max_rate": 100000
  },
  "results": [
    {
      "ip": "192.168.1.100",
      "hostname": "server1.example.com",
      "state": "up",
      "latency_ms": 2,
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
        },
        {
          "port": 443,
          "protocol": "tcp",
          "state": "open",
          "service": {
            "name": "https",
            "product": "nginx",
            "version": "1.21.6",
            "ssl": true
          },
          "tls_certificate": {
            "subject": "CN=server1.example.com",
            "issuer": "CN=Let's Encrypt",
            "valid_from": "2025-10-01T00:00:00Z",
            "valid_to": "2026-01-01T00:00:00Z",
            "san": ["server1.example.com", "www.server1.example.com"]
          }
        }
      ],
      "os": {
        "name": "Linux 5.15-5.19",
        "family": "Linux",
        "vendor": "Linux",
        "accuracy": 95,
        "cpe": ["cpe:/o:linux:linux_kernel:5.15"]
      }
    }
  ],
  "statistics": {
    "total_hosts": 256,
    "hosts_up": 42,
    "hosts_down": 214,
    "total_ports_scanned": 42000,
    "ports_open": 156,
    "ports_closed": 89,
    "ports_filtered": 41755,
    "scan_duration_sec": 330,
    "packets_sent": 84312,
    "packets_received": 245,
    "throughput_pps": 255
  }
}
```

**Usage:**

```bash
# JSON output
prtip -sS -p 1-1000 192.168.1.0/24 -oJ scan_results.json

# JSON with service detection
prtip -sV -p 80,443 targets.txt -oJ results_with_services.json

# Parse with jq
jq '.results[] | select(.ports[].state == "open")' scan_results.json
```

---

### SQLite Schema

**Database:** `scans.db` (default location: `./scans.db`)

```sql
-- Scan metadata table
CREATE TABLE scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    scan_type TEXT NOT NULL,
    targets TEXT NOT NULL,
    ports TEXT NOT NULL,
    timing_template TEXT,
    max_rate INTEGER,
    config_json TEXT
);

-- Host discovery results
CREATE TABLE hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    ip TEXT NOT NULL,
    hostname TEXT,
    state TEXT NOT NULL,
    latency_ms INTEGER,
    os_name TEXT,
    os_family TEXT,
    os_accuracy INTEGER,
    os_cpe TEXT,
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

-- Port scan results
CREATE TABLE ports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    host_id INTEGER NOT NULL,
    port INTEGER NOT NULL,
    protocol TEXT NOT NULL,
    state TEXT NOT NULL,
    service_name TEXT,
    service_product TEXT,
    service_version TEXT,
    service_cpe TEXT,
    banner TEXT,
    response_time_ms INTEGER,
    timestamp TIMESTAMP NOT NULL,
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE
);

-- TLS certificates (optional)
CREATE TABLE tls_certificates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    port_id INTEGER NOT NULL,
    subject TEXT,
    issuer TEXT,
    serial_number TEXT,
    valid_from TIMESTAMP,
    valid_to TIMESTAMP,
    san TEXT,  -- Subject Alternative Names (JSON array)
    fingerprint_sha256 TEXT,
    FOREIGN KEY (port_id) REFERENCES ports(id) ON DELETE CASCADE
);

-- Indexes for fast queries
CREATE INDEX idx_scan_id ON hosts(scan_id);
CREATE INDEX idx_host_id ON ports(host_id);
CREATE INDEX idx_port ON ports(port);
CREATE INDEX idx_state ON ports(state);
CREATE INDEX idx_service_name ON ports(service_name);
CREATE INDEX idx_ip ON hosts(ip);
```

**Usage:**

```bash
# Enable database storage
prtip -sS -p 1-1000 192.168.1.0/24 --with-db

# Custom database location
prtip -sS -p 1-1000 192.168.1.0/24 --with-db --database /path/to/results.db

# Query results
prtip db query results.db --scan-id 1
prtip db query results.db --target 192.168.1.100
prtip db query results.db --port 22 --open

# Export from database
prtip db export results.db --scan-id 1 --format json -o scan1.json
```

---

## API Specifications

### Core Scanner API

Primary scanning interface:

```rust
use prtip_core::{Scanner, ScanConfig, ScanReport};

pub struct Scanner {
    config: ScanConfig,
    runtime: Runtime,
}

impl Scanner {
    /// Create new scanner with configuration
    pub fn new(config: ScanConfig) -> Result<Self> {
        // Validates configuration
        // Initializes runtime environment
        // Drops privileges after initialization
    }

    /// Execute scan and return complete report
    pub async fn execute(&self) -> Result<ScanReport> {
        // Runs scan based on config
        // Returns complete results
    }

    /// Execute scan with real-time progress callback
    pub async fn execute_with_progress<F>(&self, callback: F) -> Result<ScanReport>
    where
        F: Fn(ScanProgress) + Send + 'static
    {
        // Calls callback periodically with progress updates
        // Returns complete results when done
    }

    /// Execute scan with event stream
    pub async fn execute_with_events(&self) -> Result<(ScanReport, EventReceiver)> {
        // Returns results + event stream for real-time monitoring
    }
}

pub struct ScanConfig {
    /// Target specifications (IPs, CIDRs, hostnames)
    pub targets: Vec<Target>,

    /// Port range to scan
    pub ports: PortRange,

    /// Scan technique (SYN, Connect, UDP, etc.)
    pub scan_type: ScanType,

    /// Timing template (T0-T5)
    pub timing: TimingTemplate,

    /// Maximum packets per second (rate limiting)
    pub max_rate: Option<u32>,

    /// Output configuration
    pub output: OutputConfig,

    /// Enable service detection
    pub service_detection: bool,

    /// Service detection intensity (0-9)
    pub version_intensity: u8,

    /// Enable OS fingerprinting
    pub os_detection: bool,

    /// Database storage
    pub database: Option<PathBuf>,
}

pub struct ScanReport {
    /// All scan results
    pub results: Vec<ScanResult>,

    /// Scan statistics
    pub statistics: ScanStatistics,

    /// Scan metadata
    pub metadata: ScanMetadata,
}

pub struct ScanProgress {
    /// Percentage complete (0.0-100.0)
    pub percentage: f64,

    /// Estimated time remaining
    pub eta_seconds: Option<u64>,

    /// Throughput (packets per second)
    pub throughput_pps: u64,

    /// Number of results so far
    pub results_count: usize,
}
```

**Example Usage:**

```rust
use prtip_core::{Scanner, ScanConfig, ScanType, TimingTemplate, PortRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ScanConfig {
        targets: vec!["192.168.1.0/24".parse()?],
        ports: PortRange::parse("80,443")?,
        scan_type: ScanType::Syn,
        timing: TimingTemplate::Normal,
        max_rate: Some(100_000),
        service_detection: true,
        version_intensity: 7,
        os_detection: false,
        ..Default::default()
    };

    let scanner = Scanner::new(config)?;

    // With progress callback
    let report = scanner.execute_with_progress(|progress| {
        println!("Progress: {:.1}% | ETA: {:?}s",
                 progress.percentage,
                 progress.eta_seconds);
    }).await?;

    println!("Scan complete: {} results", report.results.len());
    Ok(())
}
```

---

### Plugin API

Extensible plugin interface for custom scanning logic:

```rust
pub trait Plugin: Send + Sync {
    /// Plugin name (unique identifier)
    fn name(&self) -> &str;

    /// Initialize plugin with configuration
    fn init(&mut self, config: &PluginConfig) -> Result<()>;

    /// Called for each discovered port
    fn on_port_discovered(&mut self, result: &ScanResult) -> Result<()>;

    /// Called when service is detected
    fn on_service_detected(&mut self, result: &ScanResult, service: &ServiceInfo) -> Result<()>;

    /// Called at scan completion
    fn on_scan_complete(&mut self, report: &ScanReport) -> Result<()>;

    /// Cleanup resources
    fn cleanup(&mut self) -> Result<()>;
}

pub struct PluginConfig {
    /// Plugin-specific configuration (JSON)
    pub config: serde_json::Value,

    /// Plugin capabilities (read-only, network, filesystem)
    pub capabilities: PluginCapabilities,
}

pub struct PluginCapabilities {
    /// Read-only mode (no modifications)
    pub read_only: bool,

    /// Network access allowed
    pub network_access: bool,

    /// Filesystem access allowed
    pub filesystem_access: bool,
}
```

**Example Plugin (Lua):**

```lua
-- vulnerability_scanner.lua
plugin = {
    name = "VulnerabilityScanner",
    version = "1.0"
}

function plugin:on_service_detected(result, service)
    -- Check for known vulnerable versions
    if service.product == "Apache httpd" and service.version == "2.4.49" then
        log("WARNING: CVE-2021-41773 detected on " .. result.target)
    end
end

function plugin:on_scan_complete(report)
    log("Scan complete: " .. #report.results .. " results")
end
```

**Load Plugin:**

```bash
prtip -sV -p 80,443 --plugin vulnerability_scanner.lua 192.168.1.0/24
```

---

## See Also

- [Architecture](../development/architecture.md) - System architecture and design decisions
- [API Reference](./api-reference.md) - Complete public API documentation
- [Performance Characteristics](../advanced/performance-characteristics.md) - Performance metrics and KPIs
- [Nmap Compatibility](./comparisons/nmap.md) - Nmap compatibility guide
- [FAQ](./faq.md) - Frequently asked questions
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
