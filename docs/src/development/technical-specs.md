# Technical Specifications

Comprehensive technical specifications for ProRT-IP developers covering system requirements, protocol details, packet formats, performance characteristics, and platform-specific implementation details.

---

## Overview

ProRT-IP is a high-performance network scanner built with Rust, implementing multiple scanning techniques across TCP, UDP, and ICMP protocols with support for both IPv4 and IPv6. This document provides the technical foundation necessary for understanding and contributing to the implementation.

**Key Characteristics:**

- **Language:** Rust (Edition 2024, MSRV 1.85+)
- **Architecture:** Multi-crate workspace with async/await runtime (Tokio)
- **Performance:** 10M+ pps theoretical, 72K+ pps stateful (achieved)
- **Platform Support:** 5 production targets (Linux, Windows, macOS Intel/ARM64, FreeBSD)
- **Memory Safety:** Zero-cost abstractions with compile-time guarantees

---

## System Requirements

### Hardware Requirements

**Minimum Configuration (Small Networks):**

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

**Recommended Configuration (Medium Networks):**

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

**High-Performance Configuration (Internet-Scale):**

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

**Operating Systems:**

**Linux (Primary Platform):**

Supported Distributions:
- Ubuntu 20.04+ LTS / 22.04+ LTS
- Debian 11+ (Bullseye) / 12+ (Bookworm)
- Fedora 35+ / 38+
- RHEL 8+ / 9+ (Red Hat Enterprise Linux)
- Arch Linux (rolling release)
- CentOS Stream 8+ / 9+

Kernel Requirements:
- **Minimum:** 4.15+ (for sendmmsg/recvmmsg syscalls)
- **Recommended:** 5.x+ (for eBPF/XDP support)
- **Optimal:** 6.x+ (latest performance improvements)

System Packages:

```bash
# Debian/Ubuntu
sudo apt install libpcap-dev pkg-config libssl-dev

# Fedora/RHEL/CentOS
sudo dnf install libpcap-devel pkgconfig openssl-devel

# Arch Linux
sudo pacman -S libpcap pkg-config openssl
```

Runtime Libraries:
- libpcap 1.9+ (packet capture)
- OpenSSL 1.1+ or 3.x (TLS certificate analysis)
- glibc 2.27+ (standard C library)

**Windows:**

Supported Versions:
- Windows 10 (version 1809+)
- Windows 11 (all versions)
- Windows Server 2016+, 2019+, 2022+

Requirements:
- **Npcap 1.70+** (packet capture driver) - [Download](https://npcap.com/)
- **Visual C++ Redistributable 2019+** (runtime libraries)
- **Administrator privileges** (required for raw packet access)

Known Limitations:
- FIN/NULL/Xmas scans not supported (Windows TCP/IP stack limitation)
- Administrator privileges required (no capability-based alternative)
- SYN discovery tests fail on loopback (127.0.0.1) - expected Npcap behavior

**macOS:**

Supported Versions:
- macOS 11.0+ (Big Sur) - Intel & Apple Silicon
- macOS 12.0+ (Monterey) - M1/M2 chips
- macOS 13.0+ (Ventura) - M1/M2/M3 chips
- macOS 14.0+ (Sonoma) - M1/M2/M3/M4 chips

Requirements:
- **Xcode Command Line Tools** (clang compiler)
- **libpcap** (pre-installed on macOS)
- **Root privileges OR access_bpf group membership**

Setup BPF Access (Recommended):

```bash
# Grant user BPF device access (avoids sudo)
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf

# Verify group membership
dseditgroup -o checkmember -m $(whoami) access_bpf

# Logout and login for changes to take effect
```

---

## Protocol Specifications

### Ethernet (Layer 2)

**Frame Format:**

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

### IPv4 (Layer 3)

**Header Format:**

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
# Fragment packets into 8-byte segments (28-byte MTU)
prtip -f -sS -p 80,443 192.168.1.1

# Custom MTU (Maximum Transmission Unit, must be ≥68 and multiple of 8)
prtip --mtu 200 -sS -p 80,443 192.168.1.1
```

### IPv6 (Layer 3)

**Header Format:**

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|Version| Traffic Class |           Flow Label                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Payload Length        |  Next Header  |   Hop Limit   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
+                                                               +
|                                                               |
+                         Source Address                        +
|                                                               |
+                                                               +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
+                                                               +
|                                                               |
+                      Destination Address                      +
|                                                               |
+                                                               +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Field Specifications:**

| Field | Size | Description | ProRT-IP Default |
|-------|------|-------------|------------------|
| **Version** | 4 bits | IP version | `6` (IPv6) |
| **Traffic Class** | 8 bits | QoS/DSCP | `0` (default) |
| **Flow Label** | 20 bits | Flow identification | `0` (not used) |
| **Payload Length** | 16 bits | Payload size (excluding header) | Variable |
| **Next Header** | 8 bits | Protocol identifier | `6` (TCP), `17` (UDP), `58` (ICMPv6) |
| **Hop Limit** | 8 bits | Equivalent to IPv4 TTL | `64` (default) |
| **Source Address** | 128 bits | Scanner's IPv6 address | Interface IPv6 |
| **Destination Address** | 128 bits | Target IPv6 address | User-specified |

**IPv6 Address Types:**

- **Global Unicast:** `2000::/3` (Internet routable)
- **Link-Local:** `fe80::/10` (local network only)
- **Unique Local Address (ULA):** `fd00::/8` (private networks)
- **Multicast:** `ff00::/8` (group communication)

**ProRT-IP IPv6 Support:**
- 100% scanner coverage (all 8 scan types)
- ICMPv6 Echo (Type 128/129) for discovery
- NDP (Neighbor Discovery Protocol) support
- Dual-stack automatic detection
- Random Interface Identifier generation for decoy scanning

### TCP (Layer 4)

**Header Format:**

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

**TCP Options:**

Common options used in scanning:

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

### UDP (Layer 4)

**Header Format:**

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

### ICMP (Layer 3/4)

**Echo Request/Reply Format:**

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

---

## Performance Specifications

### Throughput Characteristics

**Achieved Performance:**

| Mode | Packets/Second | Notes |
|------|----------------|-------|
| **Stateless** | 1,000,000+ pps | 10GbE + 16+ cores (theoretical) |
| **Stateful SYN** | 72,000+ pps | Localhost scan (achieved) |
| **TCP Connect** | 1,000-5,000 pps | OS limit |
| **Service Detection** | 100-500 ports/sec | Probe-dependent |
| **OS Fingerprinting** | 50-100 hosts/min | 16-probe sequence |

**Scan Speed Benchmarks:**

| Scenario | Duration | Throughput | Speedup vs Baseline |
|----------|----------|------------|---------------------|
| 65K ports SYN scan | 0.91s | 72K pps | 198x faster |
| 1K ports SYN scan | 66ms | ~15K pps | 48x faster |
| Service detection | 2.3s | ~434 ports/sec | 3.5x faster |
| OS fingerprinting | 1.8s | ~33 hosts/min | 3x faster |

**Rate Limiting Performance:**

| Rate (pps) | Overhead | Status |
|------------|----------|--------|
| 10K        | **-8.2%** | ✅ Faster than no limiting |
| 50K        | **-1.8%** | ✅ Faster than no limiting |
| 75K-200K   | **-3% to -4%** | ✅ Sweet spot |
| 500K-1M    | **+0% to +3%** | ✅ Near-zero overhead |

### Memory Characteristics

**Memory Scaling Formula:**

```
Memory = 2 MB (baseline) + ports × 1.0 KB
```

**Examples:**
- 1,000 ports: ~3 MB
- 10,000 ports: ~12 MB
- 65,535 ports: ~68 MB

**Service Detection Memory:**
- Baseline: 2.7 MB
- With detection: 1.97 GB (730x increase)
- **Recommendation:** Limit service detection to 10-20 ports

### CPU Characteristics

**CPU Utilization:**

- **Futex Contention:** 77-88% CPU time (Phase 6.1 optimization target)
- **Network I/O:** 0.9-1.6% (industry-leading efficiency)
- **Packet Construction:** 58.8ns (zero-copy optimization)

**Performance Targets (Phase 6):**

- **Futex Reduction:** 30-50% CPU savings (QW-1 priority)
- **Memory Pool:** 60% brk reduction + 50% memory savings (QW-2 priority)
- **Vector Preallocation:** 10-15% memory reduction (QW-3 priority)

---

## Platform Specifications

### Build Targets

**Production Platforms (5 targets, ~95% user base):**

| Platform | Target Triple | Status | Notes |
|----------|--------------|--------|-------|
| Linux x86_64 (glibc) | `x86_64-unknown-linux-gnu` | ✅ Production | Recommended platform |
| Windows x86_64 | `x86_64-pc-windows-msvc` | ✅ Production | Requires Npcap + Administrator |
| macOS Intel | `x86_64-apple-darwin` | ✅ Production | macOS 10.13+ |
| macOS Apple Silicon | `aarch64-apple-darwin` | ✅ Production | M1/M2/M3/M4 chips, 110% baseline performance |
| FreeBSD x86_64 | `x86_64-unknown-freebsd` | ✅ Production | FreeBSD 12+ |

**Experimental Platforms (4 targets, known limitations):**

| Platform | Target Triple | Status | Known Issues |
|----------|--------------|--------|--------------|
| Linux x86_64 (musl) | `x86_64-unknown-linux-musl` | ⚠️ Experimental | Type mismatch issues |
| Linux ARM64 (glibc) | `aarch64-unknown-linux-gnu` | ⚠️ Experimental | OpenSSL cross-compilation issues |
| Linux ARM64 (musl) | `aarch64-unknown-linux-musl` | ⚠️ Experimental | Multiple compilation issues |
| Windows ARM64 | `aarch64-pc-windows-msvc` | ⚠️ Removed | Toolchain unavailable in CI |

### Platform Performance Comparison

Performance and characteristics relative to Linux x86_64 baseline:

| Platform | Binary Size | Startup Time | Performance | Package Manager |
|----------|-------------|--------------|-------------|-----------------| | Linux x86_64 (glibc) | ~8MB | <50ms | 100% (baseline) | apt, dnf, pacman |
| Linux x86_64 (musl) | ~6MB | <30ms | 95% | apk |
| Linux ARM64 | ~8MB | <60ms | 85% | apt, dnf |
| Windows x86_64 | ~9MB | <100ms | 90% | chocolatey, winget |
| macOS Intel | ~8MB | <70ms | 95% | brew |
| **macOS ARM64** | **~7MB** | **<40ms** | **110%** | **brew** |
| FreeBSD x86_64 | ~8MB | <60ms | 90% | pkg |

**Notes:**
- macOS ARM64 is fastest platform (110% baseline, native optimization)
- musl builds are smallest and fastest startup
- Performance measured with 65,535-port SYN scan baseline

---

## See Also

- [Architecture](./architecture.md) - System architecture and component design
- [Implementation](./implementation.md) - Code organization and key components
- [API Reference](../reference/api-reference.md) - Complete public API documentation
- [Performance Characteristics](../../../34-PERFORMANCE-CHARACTERISTICS.md) - Performance metrics and KPIs
- [Platform Support](../features/platform-support.md) - Platform-specific installation
