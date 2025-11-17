# Glossary

Comprehensive glossary of networking, security, and ProRT-IP-specific terms used throughout this documentation.

---

## Quick Navigation

**By Category:**
- Network Scanning (35 terms)
- TCP/IP Protocols (28 terms)
- Security & Evasion (22 terms)
- Performance & Optimization (18 terms)
- ProRT-IP Specific (15 terms)
- General Networking (20 terms)

**Alphabetical:** A | B | C | D | E | F | G | H | I | J | K | L | M | N | O | P | Q | R | S | T | U | V | W | X | Y | Z

---

## A

### ACK Scan
**Category:** Network Scanning

TCP ACK scan (`-sA`) sends packets with the ACK flag set to determine firewall rule sets. Does not determine open/closed ports, but rather filtered/unfiltered. Firewalls that block SYN packets typically allow ACK packets through, revealing firewall configuration.

**Example:**
```bash
prtip -sA -p 80,443 target.com
```

**Related Terms:** SYN Scan, Firewall Detection, Stateful Firewall

---

### Adaptive Batch Sizing
**Category:** Performance & Optimization

ProRT-IP feature that dynamically adjusts batch sizes based on system performance metrics (CPU usage, memory pressure, network throughput). Optimizes resource utilization by increasing batch sizes when resources are available and decreasing when constrained.

**Implementation:** PerformanceMonitor + AdaptiveBatchSizer in `prtip-network/src/adaptive_batch.rs`

**Related Terms:** Batch I/O, sendmmsg, Performance Tuning

---

### ARP (Address Resolution Protocol)
**Category:** TCP/IP Protocols

Protocol for mapping IP addresses to MAC addresses on local networks. ProRT-IP uses ARP for host discovery on Ethernet networks via ARP ping (`-PR`).

**Packet Structure:**
- Hardware Type (2 bytes): Ethernet = 1
- Protocol Type (2 bytes): IPv4 = 0x0800
- Hardware Address Length (1 byte): 6 for MAC
- Protocol Address Length (1 byte): 4 for IPv4
- Operation (2 bytes): Request = 1, Reply = 2

**Related Terms:** Host Discovery, MAC Address, Ethernet

---

### Asynchronous I/O
**Category:** Performance & Optimization

I/O operations that don't block the calling thread, allowing concurrent execution of multiple operations. ProRT-IP uses Tokio async runtime for non-blocking network I/O, enabling high concurrency with low resource overhead.

**Benefits:**
- **High Concurrency:** 10,000+ simultaneous connections on single thread
- **Low Overhead:** ~2KB per task vs ~8MB per OS thread
- **Efficient:** Event-driven architecture with epoll/kqueue/IOCP

**Related Terms:** Tokio, Event Loop, Futures

---

### Audit Log
**Category:** ProRT-IP Specific

JSON-formatted log of all ProRT-IP operations for security monitoring and compliance. Records scan parameters, privilege changes, target validation, and security events.

**Enable:**
```bash
prtip -sS -p 80 target.com --audit-log /var/log/prtip/audit.log
```

**Format:** See [secure-configuration.md](../security/secure-configuration.md#security-monitoring)

**Related Terms:** Compliance, GDPR, Incident Response

---

## B

### Backscatter
**Category:** Network Scanning

Unsolicited responses received from hosts that were not directly scanned. Often indicates spoofed source addresses or reflection attacks. ProRT-IP filters backscatter in stateful mode but may encounter it in stateless mode.

**Causes:**
- Spoofed source IPs
- Reflection/amplification attacks
- Misconfigured firewalls

**Related Terms:** Stateless Scan, IP Spoofing, Firewall

---

### Banner Grabbing
**Category:** Network Scanning

Technique for retrieving service identification information by connecting to a port and reading the initial response. ProRT-IP performs banner grabbing during service detection (`-sV`).

**Example Banners:**
- SSH: `SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.5`
- HTTP: `Server: Apache/2.4.41 (Ubuntu)`
- SMTP: `220 mail.example.com ESMTP Postfix`

**Related Terms:** Service Detection, Version Detection, Fingerprinting

---

### Batch I/O
**Category:** Performance & Optimization

Sending or receiving multiple packets in a single system call. ProRT-IP uses `sendmmsg()` and `recvmmsg()` on Linux for 20-40% throughput improvement over single-packet `send()`/`recv()`.

**Benefits:**
- Reduced syscall overhead (1 call per 100 packets vs 100 calls)
- Better cache locality
- Improved throughput (50K pps → 70K pps)

**Related Terms:** sendmmsg, recvmmsg, Adaptive Batch Sizing

---

### BPF (Berkeley Packet Filter)
**Category:** TCP/IP Protocols

Kernel-level packet filtering mechanism used by libpcap. ProRT-IP uses BPF on macOS and BSD for raw packet capture. Modern systems use eBPF (extended BPF) for enhanced capabilities.

**Access:** macOS users must join `access_bpf` group for non-root packet capture.

**Related Terms:** libpcap, Packet Capture, Raw Sockets

---

### Burst Size
**Category:** ProRT-IP Specific

Maximum number of packets sent in rapid succession before applying rate limiting. ProRT-IP's Rate Limiter V3 uses `burst_size=100` for optimal balance between responsiveness and network courtesy.

**Configuration:**
```toml
[rate_limiter]
max_rate = 100000    # 100K pps
burst_size = 100     # Allow 100-packet bursts
```

**Trade-off:** Higher burst = faster initial scans, but higher risk of packet loss

**Related Terms:** Rate Limiting, Token Bucket, Packet Loss

---

## C

### CDN (Content Delivery Network)
**Category:** General Networking

Distributed network of servers that deliver content based on geographic location. ProRT-IP's CDN IP Deduplication feature detects and optionally filters CDN IPs (Cloudflare, Akamai, Fastly, etc.) to reduce scan targets by 30-70%.

**Detection Methods:**
- ASN matching
- IP range databases
- Reverse DNS patterns

**Related Terms:** IP Deduplication, ASN, Reverse DNS

---

### CIDR (Classless Inter-Domain Routing)
**Category:** General Networking

Method for allocating IP addresses and routing using prefix notation. ProRT-IP supports CIDR notation for target specification.

**Examples:**
- `192.168.1.0/24` = 192.168.1.0 - 192.168.1.255 (256 addresses)
- `10.0.0.0/8` = 10.0.0.0 - 10.255.255.255 (16,777,216 addresses)
- `2001:db8::/32` = IPv6 network with 2^96 addresses

**Calculation:** `/24` = 32-24 = 8 host bits = 2^8 = 256 addresses

**Related Terms:** Subnet Mask, IP Address, Target Specification

---

### Connect Scan
**Category:** Network Scanning

TCP Connect scan (`-sT`) completes the full three-way handshake using the operating system's `connect()` call. Does not require elevated privileges but is slower (1,000-5,000 pps) and more easily detected than SYN scan.

**Process:**
1. Send SYN
2. Receive SYN-ACK (open) or RST (closed)
3. Send ACK (completes handshake)
4. Send RST (tears down connection)

**When to Use:**
- No root/sudo access
- Targets with IDS that detect half-open scans
- Windows (SYN scan requires Npcap)

**Related Terms:** SYN Scan, Three-Way Handshake, Privileges

---

### Coverage (Code Coverage)
**Category:** ProRT-IP Specific

Percentage of source code executed by tests. ProRT-IP maintains **54.92% coverage** (2,361 tests) with automated CI/CD tracking via cargo-tarpaulin and Codecov.

**Coverage Types:**
- **Line Coverage:** 54.92% (lines executed)
- **Branch Coverage:** Not tracked (Rust limitation)
- **Function Coverage:** 60%+ (core functions)

**Improvement:** Sprint 5.6 increased coverage from 37% → 54.92% (+17.66pp)

**Related Terms:** Testing, Fuzzing, CI/CD

---

## D

### Decoy Scanning
**Category:** Security & Evasion

Evasion technique that sends packets from spoofed source IPs alongside the real scanner IP. Makes it difficult for defenders to identify the actual scanning source.

**Usage:**
```bash
# Use specific decoys
prtip -sS -D 192.168.1.10,192.168.1.20,ME,192.168.1.30 target.com

# Generate 10 random decoys
prtip -sS -D RND:10 target.com
```

**Limitations:**
- Requires spoofing capability
- SYN-ACK responses go to decoy IPs (stateless mode only)
- Some ISPs block spoofed packets

**Related Terms:** IP Spoofing, Stateless Scan, IDS Evasion

---

### DHCP (Dynamic Host Configuration Protocol)
**Category:** TCP/IP Protocols

Protocol for automatically assigning IP addresses and network configuration. Not directly used by ProRT-IP but relevant for understanding network topology.

**Ports:**
- UDP 67 (server)
- UDP 68 (client)

**Related Terms:** UDP, Network Discovery, IP Address

---

### DNS (Domain Name System)
**Category:** TCP/IP Protocols

Hierarchical naming system that translates domain names to IP addresses. ProRT-IP performs DNS resolution for target specification and reverse DNS for hostname identification.

**Record Types:**
- **A:** IPv4 address
- **AAAA:** IPv6 address
- **PTR:** Reverse DNS (IP to hostname)
- **MX:** Mail server
- **TXT:** Text records

**Ports:** UDP/TCP 53

**Related Terms:** Reverse DNS, IPv4, IPv6

---

### DoS (Denial of Service)
**Category:** Security & Evasion

Attack that makes a system unavailable by overwhelming it with traffic or resource exhaustion. ProRT-IP includes DoS prevention mechanisms:

**Prevention:**
- Resource limits (max_concurrent, max_memory_mb, max_duration_sec)
- Rate limiting (default 100K pps)
- Target confirmation for internet scans
- Automatic privilege drop

**Related Terms:** Rate Limiting, Resource Limits, Firewall

---

## E

### Ethernet
**Category:** TCP/IP Protocols

Data link layer protocol for local area networks. ProRT-IP constructs Ethernet frames for raw packet transmission.

**Frame Structure:**
- Destination MAC (6 bytes)
- Source MAC (6 bytes)
- EtherType (2 bytes): 0x0800 for IPv4, 0x86DD for IPv6
- Payload (46-1500 bytes)
- FCS checksum (4 bytes)

**Related Terms:** MAC Address, ARP, Raw Sockets

---

### Event System
**Category:** ProRT-IP Specific

Publish-subscribe architecture for real-time scan event notification. ProRT-IP's Event System (Sprint 5.5.3) provides 18 event types with -4.1% overhead.

**Event Types:**
- **Discovery:** PortDiscovered, ServiceDetected, HostDiscovered
- **Progress:** ScanStarted, ScanCompleted, TargetProgress
- **Statistics:** ThroughputUpdate, ResourceUsage

**Architecture:** EventBus + ProgressCollector + TUI integration

**Related Terms:** TUI, Progress Monitoring, Metrics

---

### Evasion Techniques
**Category:** Security & Evasion

Methods to avoid detection by firewalls, IDS/IPS, and security monitoring systems. ProRT-IP implements 6 evasion techniques:

1. **Packet Fragmentation** (`-f`, `--mtu`): Split packets to evade pattern matching
2. **Decoy Scanning** (`-D`): Spoof source IPs alongside real scanner
3. **Timing Randomization** (`--scan-delay`, `--max-scan-delay`): Variable delays between packets
4. **Source Port Manipulation** (`-g`, `--source-port`): Use common source ports (53, 80)
5. **TTL Manipulation** (`--ttl`): Set specific TTL values
6. **Bad Checksums** (`--badsum`): Invalid checksums (IDS testing)

**Related Terms:** IDS, IPS, Firewall

---

## F

### Filtered Port
**Category:** Network Scanning

Port state where packets are being blocked by a firewall or packet filter. ProRT-IP cannot determine if the port is open or closed.

**Detection:**
- No response after retries
- ICMP unreachable messages (type 3, codes 1/2/3/9/10/13)
- TCP RST from firewall

**Output:**
```
PORT    STATE      SERVICE
80/tcp  filtered   http
```

**Related Terms:** Open Port, Closed Port, Firewall

---

### FIN Scan
**Category:** Network Scanning

Stealth scan (`-sF`) that sends packets with only the FIN flag set. Exploits RFC 793 requirement that closed ports respond with RST, while open ports drop the packet.

**Advantages:**
- Bypasses some non-stateful firewalls
- Less likely to be logged than SYN scans

**Limitations:**
- Fails against Windows (RST for open and closed)
- Fails against Cisco IOS (drops all FIN packets)
- Open|filtered ambiguity (both produce no response)

**Related Terms:** NULL Scan, Xmas Scan, Stealth Scanning

---

### Fingerprinting
**Category:** Network Scanning

Technique for identifying systems, services, or protocols based on their unique characteristics. ProRT-IP performs two types:

**OS Fingerprinting (`-O`):**
- TCP/IP stack analysis (16-probe sequence)
- Nmap OS database (2,600+ signatures)
- Requires open + closed port

**Service Fingerprinting (`-sV`):**
- Banner analysis
- Probe matching (187 service probes)
- 85-90% accuracy

**Related Terms:** OS Detection, Service Detection, Banner Grabbing

---

### Firewall
**Category:** Security & Evasion

Network security device that monitors and controls traffic based on rules. ProRT-IP encounters three types:

**Stateless Firewall:**
- Filters based on packet headers only
- No connection tracking
- Vulnerable to ACK scan, fragmentation

**Stateful Firewall:**
- Tracks TCP connection state
- Drops unexpected packets (e.g., ACK without SYN)
- More secure than stateless

**Application Firewall:**
- Deep packet inspection
- Protocol-aware filtering
- Hardest to evade

**Detection:** Use `-sA` (ACK scan) to map firewall rules

**Related Terms:** Filtered Port, IDS, Evasion Techniques

---

### Fragmentation
**Category:** Security & Evasion

Splitting IP packets into smaller fragments. ProRT-IP uses fragmentation (`-f`, `--mtu`) to evade pattern-matching IDS/IPS.

**IP Fragmentation:**
- Splits large packets into MTU-sized fragments
- Reassembled by destination
- Evades IDS that don't reassemble

**Example:**
```bash
# Use default 8-byte fragments
prtip -sS -f -p 80 target.com

# Use custom MTU
prtip -sS --mtu 24 -p 80 target.com
```

**Limitations:**
- Some firewalls block fragmented packets
- Can reduce scan speed
- Not all IDS systems are vulnerable

**Related Terms:** MTU, IDS Evasion, Packet

---

### Fuzzing
**Category:** ProRT-IP Specific

Automated testing with random/invalid inputs to discover crashes and undefined behavior. ProRT-IP's fuzz testing (Sprint 5.7) achieved **230M+ executions with 0 crashes**.

**Fuzz Targets:**
1. `fuzz_target_ipv4_packet` - IPv4 packet parsing
2. `fuzz_target_ipv6_packet` - IPv6 packet parsing
3. `fuzz_target_tcp_packet` - TCP packet parsing
4. `fuzz_target_icmp_packet` - ICMP message parsing
5. `fuzz_target_service_probe` - Service detection

**Tool:** cargo-fuzz with libFuzzer

**Related Terms:** Testing, Coverage, Property-Based Testing

---

## G

### GDPR (General Data Protection Regulation)
**Category:** ProRT-IP Specific

European data protection regulation. ProRT-IP provides GDPR-compliant features:

**Data Minimization:**
```toml
[output]
include_timestamps = false
include_hostnames = false
include_banners = false
```

**Data Subject Rights:**
- Article 15 (Access): `--export-scan-data`
- Article 17 (Erasure): `--delete-scan-data`
- Article 16 (Rectification): `--update-scan-data`

**Retention:**
- Default: 30 days scan data, 90 days audit logs
- Configurable via `retention_days`

**Related Terms:** Audit Log, Compliance, PCI DSS

---

## H

### Half-Open Scan
**Category:** Network Scanning

Another term for SYN scan. Called "half-open" because it doesn't complete the three-way handshake.

**Process:**
1. Send SYN
2. Receive SYN-ACK (open) or RST (closed)
3. Send RST (abort handshake)

**Synonym:** SYN Scan

**Related Terms:** Three-Way Handshake, Stealth Scanning

---

### Host Discovery
**Category:** Network Scanning

Process of identifying active hosts on a network before port scanning. ProRT-IP supports 5 host discovery methods:

1. **ICMP Echo** (`-PE`): Ping
2. **ICMP Timestamp** (`-PP`): Timestamp request
3. **ICMP Netmask** (`-PM`): Netmask request
4. **TCP SYN** (`-PS`): SYN to common ports
5. **TCP ACK** (`-PA`): ACK to common ports
6. **ARP** (`-PR`): ARP ping (local network only)

**Disable:** `-Pn` (skip host discovery)

**Related Terms:** Ping, ICMP, ARP

---

## I

### ICMP (Internet Control Message Protocol)
**Category:** TCP/IP Protocols

Protocol for diagnostic and error messages. ProRT-IP uses ICMP for host discovery and interprets ICMP unreachable messages during UDP scanning.

**Message Types:**
- Type 0: Echo Reply (pong)
- Type 3: Destination Unreachable
  - Code 0: Network unreachable
  - Code 1: Host unreachable
  - Code 3: Port unreachable (UDP scanning)
- Type 8: Echo Request (ping)
- Type 11: Time Exceeded (traceroute)

**ICMPv6:** IPv6 equivalent with additional functionality (Neighbor Discovery)

**Related Terms:** Ping, UDP Scan, Host Discovery

---

### Idle Scan
**Category:** Network Scanning

Advanced stealth scan (`-sI`) that uses a "zombie" host to scan targets without revealing the scanner's IP. ProRT-IP's implementation (Sprint 5.3) achieves 99.5% accuracy.

**Process:**
1. Probe zombie's IP ID (baseline)
2. Spoof packet from zombie to target
3. Probe zombie's IP ID again
4. Analyze IP ID increment:
   - +1: Port closed (zombie sent RST)
   - +2: Port open (zombie sent RST after receiving SYN-ACK)

**Requirements:**
- Zombie with predictable IP ID sequence
- Zombie idle (no other traffic)
- Spoofing capability

**Related Terms:** IP Spoofing, IP ID, Stealth Scanning

---

### IDS (Intrusion Detection System)
**Category:** Security & Evasion

System that monitors network traffic for malicious activity and policy violations. ProRT-IP's evasion techniques can bypass some IDS systems.

**Types:**
- **NIDS (Network IDS):** Monitors network traffic (e.g., Snort, Suricata)
- **HIDS (Host IDS):** Monitors host activity (e.g., OSSEC)

**Detection Methods:**
- Signature-based (pattern matching)
- Anomaly-based (statistical analysis)
- Stateful protocol analysis

**Evasion:** `-f` (fragmentation), `-D` (decoys), `--scan-delay` (slow scan)

**Related Terms:** IPS, Firewall, Evasion Techniques

---

### IP Address
**Category:** General Networking

Numerical label assigned to devices on a network. ProRT-IP supports both IPv4 and IPv6.

**IPv4:**
- 32-bit address (4 octets)
- Example: `192.168.1.1`
- Range: 0.0.0.0 - 255.255.255.255
- Total: 4,294,967,296 addresses

**IPv6:**
- 128-bit address (8 hextets)
- Example: `2001:db8::1`
- Range: `::` - `ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff`
- Total: 340 undecillion addresses

**Related Terms:** IPv4, IPv6, CIDR

---

### IP Deduplication
**Category:** ProRT-IP Specific

Feature that removes duplicate IPs from scan targets to improve efficiency. ProRT-IP's CDN IP Deduplication (Sprint 6.3) achieves 30-70% target reduction.

**Methods:**
- HashSet-based deduplication (O(1) lookup)
- CDN IP detection and filtering
- Anycast IP identification

**Benefits:**
- Reduced scan time
- Lower bandwidth usage
- Fewer duplicate results

**Related Terms:** CDN, Anycast, Target Specification

---

### IP ID
**Category:** TCP/IP Protocols

16-bit identification field in IPv4 header used for fragment reassembly. Also exploited for idle scanning.

**IP ID Behavior:**
- **Sequential:** Increments by 1 per packet (ideal for idle scan zombie)
- **Random:** Randomized per packet (unusable as zombie)
- **Per-Destination:** Sequential per destination (BSD, macOS)

**Idle Scan:** ProRT-IP analyzes IP ID increments to infer port states

**Related Terms:** Idle Scan, IP Spoofing, IPv4

---

### IP Spoofing
**Category:** Security & Evasion

Forging packet source IP addresses to hide the scanner's identity. Required for idle scan and decoy scanning.

**Challenges:**
- Many ISPs block spoofed packets (BCP 38)
- Stateful firewalls track connections
- Response packets go to spoofed IP (stateless mode only)

**Usage:**
```bash
# Idle scan (spoofs zombie IP)
prtip -sI zombie.com -p 80 target.com

# Decoy scanning
prtip -sS -D 10.0.0.1,10.0.0.2,ME target.com
```

**Related Terms:** Idle Scan, Decoy Scanning, BCP 38

---

### IPS (Intrusion Prevention System)
**Category:** Security & Evasion

Active security device that detects AND blocks malicious traffic. More aggressive than IDS.

**Blocking Methods:**
- Drop packets matching signatures
- Reset TCP connections
- Rate limiting
- IP blacklisting

**Evasion:** Requires multiple techniques (fragmentation + timing + decoys)

**Related Terms:** IDS, Firewall, Rate Limiting

---

### IPv4
**Category:** TCP/IP Protocols

Internet Protocol version 4. 32-bit addresses, most widely deployed protocol.

**Header Fields (20 bytes minimum):**
- Version (4 bits): Always 4
- IHL (4 bits): Header length in 32-bit words
- Total Length (16 bits): Packet size including header
- Identification (16 bits): IP ID for fragmentation
- Flags (3 bits): Don't Fragment, More Fragments
- Fragment Offset (13 bits)
- TTL (8 bits): Time to live (hop count)
- Protocol (8 bits): 6=TCP, 17=UDP, 1=ICMP
- Checksum (16 bits)
- Source IP (32 bits)
- Destination IP (32 bits)

**Related Terms:** IPv6, TCP, UDP

---

### IPv6
**Category:** TCP/IP Protocols

Internet Protocol version 6. 128-bit addresses, successor to IPv4. ProRT-IP achieved **100% IPv6 coverage** in Sprint 5.1.

**Header Fields (40 bytes fixed):**
- Version (4 bits): Always 6
- Traffic Class (8 bits): QoS priority
- Flow Label (20 bits): QoS flow identification
- Payload Length (16 bits): Payload size (not including header)
- Next Header (8 bits): Type of next header (6=TCP, 17=UDP, 58=ICMPv6)
- Hop Limit (8 bits): Equivalent to IPv4 TTL
- Source Address (128 bits)
- Destination Address (128 bits)

**Address Types:**
- **Unicast:** Single destination (global, link-local, unique local)
- **Multicast:** Multiple destinations (ff00::/8)
- **Anycast:** Nearest of multiple destinations

**Overhead:** -1.9% vs IPv4 (exceeds +15% documented expectation)

**Related Terms:** IPv4, ICMPv6, NDP

---

## J

### JSON
**Category:** ProRT-IP Specific

Output format for structured scan results. ProRT-IP supports JSON output via `-oJ` flag.

**Example:**
```json
{
  "scan_id": "scan-2025-11-15-123456",
  "targets": ["scanme.nmap.org"],
  "ports": [
    {
      "port": 80,
      "protocol": "tcp",
      "state": "open",
      "service": "http",
      "version": "Apache/2.4.41"
    }
  ]
}
```

**Benefits:**
- Machine-readable
- Integration-friendly
- Structured querying (jq)

**Related Terms:** Output Formats, XML, Greppable

---

## K

### Kernel Bypass
**Category:** Performance & Optimization

Technique for moving packet processing from kernel to userspace, reducing context switches. ProRT-IP uses standard kernel networking (libpcap) but could adopt DPDK or XDP for >10M pps.

**Technologies:**
- **DPDK (Data Plane Development Kit):** Userspace packet processing
- **XDP (eXpress Data Path):** In-kernel fast path
- **AF_XDP:** XDP sockets for userspace

**Trade-off:** Complexity vs performance (future consideration)

**Related Terms:** Performance Tuning, Zero-Copy, libpcap

---

## L

### libpcap
**Category:** TCP/IP Protocols

Cross-platform library for packet capture. ProRT-IP uses libpcap (via pnet crate) for raw packet access on all platforms.

**Platform Variants:**
- **Linux:** libpcap with AF_PACKET sockets
- **Windows:** Npcap (WinPcap successor)
- **macOS/BSD:** libpcap with BPF

**Functions:**
- `pcap_open_live()`: Open capture interface
- `pcap_compile()`: Compile BPF filter
- `pcap_setfilter()`: Apply filter
- `pcap_next_ex()`: Read packet

**Related Terms:** BPF, Npcap, Raw Sockets

---

### Lua
**Category:** ProRT-IP Specific

Scripting language for ProRT-IP plugins. Plugin System (Sprint 5.8) uses Lua 5.4 with sandboxing and capabilities.

**Plugin Types:**
1. **Service Detection:** Custom service probes
2. **Post-Processing:** Result transformation
3. **Custom Scans:** User-defined scan logic

**Sandboxing:**
- Max instructions: 1,000,000
- Max memory: 10 MB
- Restricted I/O access
- No dangerous functions (os.execute, io.popen)

**Related Terms:** Plugin System, Sandboxing, NSE

---

## M

### MAC Address
**Category:** General Networking

Media Access Control address. 48-bit hardware identifier for network interfaces.

**Format:** 6 octets in hexadecimal (e.g., `00:1A:2B:3C:4D:5E`)

**Components:**
- **OUI (Organizationally Unique Identifier):** First 3 octets (vendor)
- **NIC (Network Interface Controller):** Last 3 octets (device)

**Usage:** ARP protocol, local network host discovery

**Related Terms:** ARP, Ethernet, Host Discovery

---

### Masscan
**Category:** General Networking

High-speed port scanner (6M+ pps) that inspired ProRT-IP's stateless mode. Uses custom TCP/IP stack for maximum performance.

**Comparison to ProRT-IP:**
- **Speed:** Masscan faster (6M pps vs 1M pps stateless)
- **Features:** ProRT-IP more comprehensive (OS detection, service detection, IPv6)
- **Safety:** ProRT-IP memory-safe (Rust vs C)

**Related Terms:** Stateless Scan, Performance Tuning, Nmap

---

### MTU (Maximum Transmission Unit)
**Category:** TCP/IP Protocols

Largest packet size that can be transmitted without fragmentation.

**Common MTU Values:**
- Ethernet: 1500 bytes
- PPPoE: 1492 bytes
- IPv6 minimum: 1280 bytes

**Fragmentation:** ProRT-IP uses `--mtu` flag to set custom MTU for IDS evasion

**Related Terms:** Fragmentation, Ethernet, Packet

---

## N

### NDP (Neighbor Discovery Protocol)
**Category:** TCP/IP Protocols

IPv6 equivalent of ARP. ProRT-IP uses NDP for IPv6 host discovery and link-layer address resolution.

**Functions:**
- **Router Discovery:** Find local routers
- **Prefix Discovery:** Determine address prefixes
- **Address Resolution:** IPv6 to MAC mapping (replaces ARP)
- **Duplicate Address Detection:** Ensure address uniqueness

**ICMPv6 Message Types:**
- Type 133: Router Solicitation
- Type 134: Router Advertisement
- Type 135: Neighbor Solicitation (ARP equivalent)
- Type 136: Neighbor Advertisement

**Related Terms:** IPv6, ICMPv6, ARP

---

### Nmap
**Category:** General Networking

Industry-standard network scanner (25+ years). ProRT-IP aims for Nmap-compatible CLI and 50%+ feature parity.

**Nmap Advantages:**
- 1,000+ service signatures
- 2,600+ OS signatures
- 600+ NSE scripts
- Decades of fingerprints

**ProRT-IP Advantages:**
- 10-100x faster (stateless mode)
- Memory-safe (Rust vs C)
- Modern async I/O
- Lower resource usage

**Compatibility:** ProRT-IP supports 50+ Nmap flags (`-sS`, `-sT`, `-sU`, `-O`, `-sV`, `-p`, `-T0-T5`, `-oN`, `-oX`, etc.)

**Related Terms:** NSE, Service Detection, OS Detection

---

### Npcap
**Category:** TCP/IP Protocols

Packet capture library for Windows (WinPcap successor). Required for ProRT-IP on Windows.

**Features:**
- Loopback capture
- Raw 802.11 capture
- WinPcap API compatibility

**Installation:** Download from https://npcap.com/, enable "WinPcap API-compatible mode"

**Related Terms:** libpcap, Windows, Packet Capture

---

### NSE (Nmap Scripting Engine)
**Category:** General Networking

Lua-based scripting engine in Nmap with 600+ scripts. ProRT-IP's Plugin System provides similar functionality with sandboxing and capabilities.

**Script Categories:**
- **Auth:** Authentication testing
- **Brute:** Password brute-forcing
- **Discovery:** Service/host discovery
- **Exploit:** Vulnerability exploitation
- **Vuln:** Vulnerability detection

**ProRT-IP Equivalent:** Lua plugin system with sandboxing (Sprint 5.8)

**Related Terms:** Nmap, Lua, Plugin System

---

### NULL Scan
**Category:** Network Scanning

Stealth scan (`-sN`) that sends packets with no flags set. Exploits RFC 793 requirement that closed ports respond with RST.

**Advantages:**
- Bypasses some non-stateful firewalls
- Stealthier than SYN scan

**Limitations:**
- Same as FIN scan (Windows, Cisco incompatibility)
- Open|filtered ambiguity

**Related Terms:** FIN Scan, Xmas Scan, Stealth Scanning

---

## O

### Open Port
**Category:** Network Scanning

Port state indicating an application is accepting connections.

**Detection:**
- **TCP:** SYN-ACK response to SYN
- **UDP:** No response (service listening but silent) OR application response

**Output:**
```
PORT    STATE  SERVICE
80/tcp  open   http
```

**Related Terms:** Closed Port, Filtered Port, Port Scanning

---

### OS Detection
**Category:** Network Scanning

Identifying the operating system of a target by analyzing TCP/IP stack behavior. ProRT-IP's OS fingerprinting (`-O`) uses Nmap's database (2,600+ signatures).

**Probes (16 total):**
1. **SEQ:** TCP Sequence number prediction
2. **OPS:** TCP Options ordering
3. **WIN:** TCP Window size
4. **ECN:** Explicit Congestion Notification
5. **T1-T7:** Various TCP tests
6. **U1:** UDP test
7. **IE:** ICMP Echo

**Requirements:**
- At least 1 open port
- At least 1 closed port
- Root/capabilities

**Accuracy:** 70-90% depending on signature coverage

**Related Terms:** Fingerprinting, Service Detection, Nmap

---

### Output Formats
**Category:** ProRT-IP Specific

ProRT-IP supports 5 output formats:

1. **Normal (`-oN`):** Human-readable text
2. **XML (`-oX`):** Nmap-compatible XML
3. **Greppable (`-oG`):** Line-based format
4. **JSON (`-oJ`):** Structured JSON
5. **All (`-oA`):** All formats simultaneously

**Database Storage:** SQLite via `--with-db` flag

**Related Terms:** JSON, XML, Greppable

---

## P

### Packet
**Category:** TCP/IP Protocols

Unit of data transmitted over a network. ProRT-IP constructs raw packets for scanning.

**Layers:**
1. **Ethernet:** MAC addresses (14 bytes)
2. **IP:** Source/dest IPs (20 bytes IPv4, 40 bytes IPv6)
3. **TCP/UDP:** Ports and flags (20+ bytes TCP, 8 bytes UDP)
4. **Payload:** Application data (optional)

**Maximum Sizes:**
- Ethernet frame: 1518 bytes (including FCS)
- IPv4 packet: 65,535 bytes (theoretical)
- IPv6 packet: 65,575 bytes (including extension headers)

**Related Terms:** Ethernet, IPv4, TCP

---

### Packet Capture
**Category:** TCP/IP Protocols

Recording network packets for analysis. ProRT-IP uses libpcap for packet capture.

**Formats:**
- **PCAP:** Standard packet capture format
- **PCAPNG:** Next-generation PCAP with metadata

**ProRT-IP:** Supports PCAPNG export via `--capture-raw-responses` flag

**Related Terms:** libpcap, PCAPNG, Wireshark

---

### PCI DSS (Payment Card Industry Data Security Standard)
**Category:** ProRT-IP Specific

Security standard for organizations handling credit card data. ProRT-IP supports PCI DSS compliance scanning:

**Requirement 11.2:** Quarterly external vulnerability scans
```bash
prtip -sS -sV -p 1-65535 --output-format pci-dss --scan-type quarterly-external target.com
```

**Requirement 11.3:** Annual penetration testing
```bash
prtip -sS -sV -O -A -p- --scan-type annual-pentest target.com
```

**Related Terms:** Compliance, GDPR, NIST CSF

---

### Performance Tuning
**Category:** Performance & Optimization

Optimizing ProRT-IP for maximum speed and efficiency. Key parameters:

**Timing Templates (`-T0` to `-T5`):**
- T0 (Paranoid): 1-10 pps, IDS evasion
- T3 (Normal): 1K-10K pps, default
- T5 (Insane): 10K-100K pps, LANs only

**Rate Limiting:**
- `--max-rate`: Maximum packets per second (default 100K)
- `--min-rate`: Minimum rate guarantee

**Parallelism:**
- `--max-parallelism`: Concurrent targets
- Adaptive parallelism based on CPU cores

**Resource Limits:**
- `--max-retries`: Retry attempts (default 3)
- `--timeout`: Per-probe timeout

**Related Terms:** Rate Limiting, Timing Templates, Adaptive Parallelism

---

### Ping
**Category:** Network Scanning

ICMP Echo Request used to test host reachability. ProRT-IP uses ping for host discovery (`-PE`).

**Process:**
1. Send ICMP Echo Request (type 8)
2. Receive ICMP Echo Reply (type 0) if host is up

**Limitations:**
- Many firewalls block ICMP
- Not reliable for host discovery alone

**Alternatives:** TCP SYN ping (`-PS`), TCP ACK ping (`-PA`)

**Related Terms:** ICMP, Host Discovery, Firewall

---

### Plugin System
**Category:** ProRT-IP Specific

Lua-based extension system (Sprint 5.8) for custom scanning logic. Supports 3 plugin types with sandboxing.

**Security:**
- Instruction limit: 1,000,000
- Memory limit: 10 MB
- Restricted I/O
- Capability-based access control

**Example Plugin:**
```lua
-- service_detector.lua
function detect(response)
    if response:match("SSH%-2%.0") then
        return {
            service = "ssh",
            version = response:match("SSH%-2%.0%-([%w%.]+)")
        }
    end
end
```

**Related Terms:** Lua, NSE, Sandboxing

---

### Port
**Category:** General Networking

16-bit number identifying an application endpoint (0-65535).

**Port Ranges:**
- **Well-Known (0-1023):** System services (HTTP 80, HTTPS 443, SSH 22)
- **Registered (1024-49151):** User applications (MySQL 3306, PostgreSQL 5432)
- **Dynamic (49152-65535):** Ephemeral ports

**Port States:**
- **Open:** Application accepting connections
- **Closed:** No application listening
- **Filtered:** Firewall blocking access

**Related Terms:** TCP, UDP, Open Port

---

### Port Scanning
**Category:** Network Scanning

Process of probing ports to determine state (open/closed/filtered). ProRT-IP's core functionality.

**Scan Types (8 total):**
1. **SYN Scan (`-sS`):** Default, fast, stealthy (50K+ pps)
2. **Connect Scan (`-sT`):** Full connection, no root required (1K-5K pps)
3. **UDP Scan (`-sU`):** UDP protocol, slow (100-500 pps)
4. **FIN Scan (`-sF`):** Stealth, bypasses some firewalls
5. **NULL Scan (`-sN`):** No flags set, stealth
6. **Xmas Scan (`-sX`):** FIN+PSH+URG flags, stealth
7. **ACK Scan (`-sA`):** Firewall detection
8. **Idle Scan (`-sI`):** Maximum anonymity via zombie

**Related Terms:** SYN Scan, TCP, UDP

---

### Privileges
**Category:** ProRT-IP Specific

Access rights required for raw packet operations. ProRT-IP requires elevated privileges for SYN/UDP/stealth scans.

**Linux:**
- **Option 1:** Run as root (`sudo prtip`)
- **Option 2:** Grant capabilities (`setcap cap_net_raw,cap_net_admin=eip prtip`)
- **Option 3:** Use Connect scan (`-sT`, no privileges required)

**macOS:**
- Add user to `access_bpf` group
- Or run with `sudo`

**Windows:**
- Run as Administrator (only option)

**Security:** ProRT-IP drops privileges immediately after socket creation

**Related Terms:** Capabilities, Privilege Drop, Connect Scan

---

## Q

### QoS (Quality of Service)
**Category:** TCP/IP Protocols

Network mechanism for prioritizing traffic. Relevant for understanding scan impact on production networks.

**IPv4 Fields:**
- **TOS (Type of Service):** 8-bit field for QoS
- **DSCP (Differentiated Services Code Point):** Modern replacement for TOS

**IPv6 Fields:**
- **Traffic Class:** 8-bit QoS priority
- **Flow Label:** 20-bit flow identification

**ProRT-IP:** Does not manipulate QoS fields (future consideration)

**Related Terms:** IPv4, IPv6, Traffic Shaping

---

## R

### Rate Limiting
**Category:** ProRT-IP Specific

Controlling packet transmission rate to prevent network congestion and avoid triggering IDS/IPS. ProRT-IP's Rate Limiter V3 achieves **-1.8% overhead** (industry-leading).

**Algorithm:** Token Bucket
- **Capacity:** `burst_size` tokens (default 100)
- **Refill Rate:** `max_rate` tokens/second (default 100K pps)
- **Adaptive:** Adjusts based on packet loss

**Configuration:**
```toml
[rate_limiter]
max_rate = 100000      # 100K pps
burst_size = 100       # Allow 100-packet bursts
adaptive = true        # Adjust on packet loss
```

**Related Terms:** Token Bucket, Burst Size, Adaptive Rate Limiting

---

### Raw Sockets
**Category:** TCP/IP Protocols

Low-level socket API for constructing custom packets. ProRT-IP uses raw sockets for SYN/UDP/stealth scans.

**Capabilities:**
- Construct arbitrary packets (custom headers)
- Send without kernel TCP/IP stack
- Receive all packets (promiscuous mode)

**Requirements:**
- Root/administrator privileges
- Or capabilities (Linux: `CAP_NET_RAW`, `CAP_NET_ADMIN`)

**Platform APIs:**
- Linux: `AF_PACKET` sockets
- Windows: Winsock raw sockets (via Npcap)
- macOS/BSD: BPF (Berkeley Packet Filter)

**Related Terms:** libpcap, Privileges, BPF

---

### Reconnaissance
**Category:** Network Scanning

Information gathering phase of security assessment. ProRT-IP provides comprehensive reconnaissance capabilities:

**Passive Reconnaissance:**
- DNS lookups
- WHOIS queries
- Public database searches

**Active Reconnaissance:**
- Port scanning
- Service detection
- OS fingerprinting
- Banner grabbing

**Related Terms:** Port Scanning, Service Detection, OS Detection

---

### Reverse DNS
**Category:** General Networking

DNS PTR record lookup to obtain hostname from IP address.

**Process:**
1. Reverse IP octets: `192.168.1.1` → `1.1.168.192.in-addr.arpa`
2. Query PTR record
3. Receive hostname (e.g., `host1.example.com`)

**ProRT-IP:** Performs reverse DNS during scanning for hostname identification

**Related Terms:** DNS, PTR Record, Hostname

---

## S

### Sandboxing
**Category:** ProRT-IP Specific

Isolating plugin execution to prevent malicious behavior. ProRT-IP's Plugin System uses Lua sandboxing:

**Restrictions:**
- Instruction limit: 1,000,000
- Memory limit: 10 MB
- No dangerous functions (`os.execute`, `io.popen`, `require`, `dofile`)
- Read-only access to scan results
- No network access
- No filesystem access

**Capabilities:** Explicit permissions for specific operations

**Related Terms:** Plugin System, Lua, Security

---

### sendmmsg
**Category:** Performance & Optimization

Linux syscall for sending multiple packets in a single call. ProRT-IP uses `sendmmsg()` for 20-40% throughput improvement.

**Signature:**
```c
int sendmmsg(int sockfd, struct mmsghdr *msgvec,
             unsigned int vlen, int flags);
```

**Benefits:**
- 1 syscall for 100 packets (vs 100 calls)
- Reduced context switches
- Better CPU cache locality
- 50K pps → 70K pps (40% improvement)

**Platform:** Linux only (macOS/Windows use single-packet `send()`)

**Related Terms:** recvmmsg, Batch I/O, Performance Tuning

---

### Service Detection
**Category:** Network Scanning

Identifying application protocols and versions on open ports. ProRT-IP's service detection (`-sV`) achieves **85-90% accuracy**.

**Methods:**
1. **Banner Grabbing:** Read initial response
2. **Probe Matching:** Send probes from nmap-service-probes (187 probes)
3. **Protocol Analysis:** Interpret responses

**Example:**
```bash
prtip -sS -sV -p 1-1000 target.com
```

**Output:**
```
PORT    STATE  SERVICE  VERSION
22/tcp  open   ssh      OpenSSH 8.2p1 Ubuntu-4ubuntu0.5
80/tcp  open   http     Apache/2.4.41 (Ubuntu)
3306/tcp open  mysql    MySQL 8.0.32
```

**Intensity Levels (`--version-intensity 0-9`):**
- 0: Banner grabbing only
- 7: Default (balance)
- 9: All probes (slowest, most accurate)

**Related Terms:** Banner Grabbing, Version Detection, nmap-service-probes

---

### SYN Scan
**Category:** Network Scanning

Default TCP scan method (`-sS`) that sends SYN packets without completing handshake. Fast (50K+ pps), stealthy, requires root/capabilities.

**Process:**
1. Send SYN (synchronize)
2. Receive SYN-ACK (open) or RST (closed)
3. Send RST (reset, abort handshake)

**Advantages:**
- Fast (50K-100K pps stateful)
- Stealthy (doesn't complete connection)
- Firewall-friendly (less likely to be logged)

**Requirements:**
- Root/sudo privileges
- Or capabilities (Linux)

**Related Terms:** Half-Open Scan, Three-Way Handshake, Connect Scan

---

## T

### TCP (Transmission Control Protocol)
**Category:** TCP/IP Protocols

Connection-oriented transport protocol with reliability, ordering, and error checking.

**Header Fields (20 bytes minimum):**
- Source Port (16 bits)
- Destination Port (16 bits)
- Sequence Number (32 bits): Byte offset
- Acknowledgment Number (32 bits): Next expected byte
- Data Offset (4 bits): Header length in 32-bit words
- Flags (9 bits): CWR, ECE, URG, ACK, PSH, RST, SYN, FIN
- Window Size (16 bits): Receive buffer size
- Checksum (16 bits)
- Urgent Pointer (16 bits)
- Options (0-40 bytes): MSS, Window Scale, SACK, Timestamps

**Three-Way Handshake:**
1. Client: SYN
2. Server: SYN-ACK
3. Client: ACK

**Related Terms:** SYN Scan, Three-Way Handshake, UDP

---

### Three-Way Handshake
**Category:** TCP/IP Protocols

TCP connection establishment process.

**Steps:**
1. **Client → Server: SYN**
   - Sequence number: X (random)
   - Flags: SYN

2. **Server → Client: SYN-ACK**
   - Sequence number: Y (random)
   - Acknowledgment: X + 1
   - Flags: SYN, ACK

3. **Client → Server: ACK**
   - Sequence number: X + 1
   - Acknowledgment: Y + 1
   - Flags: ACK

**Connection Established:** Both sides ready for data transfer

**SYN Scan:** Aborts after step 2 by sending RST

**Related Terms:** TCP, SYN Scan, Connect Scan

---

### Timing Templates
**Category:** ProRT-IP Specific

Predefined timing configurations (`-T0` to `-T5`) for different scanning scenarios.

**Templates:**

| Template | Name | Timeout | pps | Use Case |
|----------|------|---------|-----|----------|
| **-T0** | Paranoid | 5 min | 1-10 | IDS evasion |
| **-T1** | Sneaky | 15 sec | 10-100 | Unreliable networks |
| **-T2** | Polite | 1 sec | 100-1K | Internet scanning |
| **-T3** | Normal | 1 sec | 1K-10K | **Default** |
| **-T4** | Aggressive | 500ms | 10K-100K | LANs |
| **-T5** | Insane | 100ms | 10K-100K | Fast LANs |

**Trade-offs:**
- **Slower (T0-T2):** More accurate, less detected, slower
- **Faster (T4-T5):** Less accurate, more detected, faster

**Related Terms:** Rate Limiting, Performance Tuning, IDS Evasion

---

### TLS (Transport Layer Security)
**Category:** TCP/IP Protocols

Cryptographic protocol for secure communication. ProRT-IP's TLS Certificate Analysis (Sprint 5.5) parses X.509v3 certificates in **1.33μs**.

**Certificate Fields:**
- Subject/Issuer DN
- Serial number
- Validity period (not before/after)
- Public key algorithm and bits
- Signature algorithm
- Extensions (SAN, Key Usage, etc.)

**Analysis:**
```bash
prtip -sS -sV --tls-cert-analysis -p 443 target.com
```

**SNI (Server Name Indication):** Supported for virtual hosts

**Related Terms:** X.509, SNI, HTTPS

---

### Tokio
**Category:** Performance & Optimization

Asynchronous runtime for Rust. ProRT-IP uses Tokio for all async I/O operations.

**Features:**
- Multi-threaded work-stealing scheduler
- Async I/O with epoll (Linux), kqueue (macOS/BSD), IOCP (Windows)
- Timers, channels, synchronization primitives

**Benefits:**
- 10,000+ concurrent tasks on single thread
- ~2KB per task (vs ~8MB per OS thread)
- Efficient CPU utilization

**ProRT-IP Usage:** All scanners, network I/O, event system

**Related Terms:** Asynchronous I/O, Event Loop, Futures

---

### Token Bucket
**Category:** Performance & Optimization

Rate limiting algorithm that allows bursts while enforcing average rate.

**Metaphor:**
- Bucket holds tokens (max = burst_size)
- Tokens added at fixed rate (max_rate)
- Send packet = consume token
- No tokens = wait for refill

**ProRT-IP Implementation:**
```rust
capacity: 100 tokens (burst_size)
refill_rate: 100,000 tokens/sec (max_rate)
```

**Advantages:**
- Allows short bursts (responsive)
- Enforces long-term rate (courteous)
- Simple and efficient (-1.8% overhead)

**Related Terms:** Rate Limiting, Burst Size, Leaky Bucket

---

### TTL (Time To Live)
**Category:** TCP/IP Protocols

8-bit field in IP header that limits packet lifetime by hop count.

**Behavior:**
- Decremented by 1 at each router
- Packet dropped when TTL reaches 0
- ICMP Time Exceeded sent to source (traceroute basis)

**Default Values:**
- Linux: 64
- Windows: 128
- Cisco: 255

**ProRT-IP:** `--ttl` flag for custom TTL (evasion technique)

**Related Terms:** IPv4, Traceroute, Evasion Techniques

---

### TUI (Terminal User Interface)
**Category:** ProRT-IP Specific

Real-time terminal-based dashboard for scan monitoring. ProRT-IP's TUI Framework (Sprint 6.1) achieves **60 FPS** rendering with **10K+ events/sec** throughput.

**Features:**
- 4-tab dashboard (Port Table, Service Table, Metrics Dashboard, Network Graph)
- Keyboard navigation (Tab, Arrow keys, Q)
- Live metrics (throughput, ETA, progress)
- Event log with filtering

**Architecture:**
- ratatui 0.29 + crossterm 0.28
- Event-driven (tokio::select!)
- Thread-safe state (Arc<RwLock<ScanState>>)

**Usage:**
```bash
prtip --live -sS -p- target.com
```

**Related Terms:** Event System, Progress Monitoring, ratatui

---

## U

### UDP (User Datagram Protocol)
**Category:** TCP/IP Protocols

Connectionless transport protocol. Faster than TCP but unreliable (no ordering, no retransmission).

**Header Fields (8 bytes):**
- Source Port (16 bits)
- Destination Port (16 bits)
- Length (16 bits): Header + payload
- Checksum (16 bits): Optional in IPv4, mandatory in IPv6

**UDP Scan (`-sU`):**
- Send UDP packet to port
- No response → open|filtered (ambiguous)
- ICMP Port Unreachable (type 3 code 3) → closed
- Application response → open

**Challenges:**
- Very slow (100-500 pps due to ICMP rate limiting)
- Open vs filtered ambiguity
- Many services silent (don't respond to random data)

**Related Terms:** TCP, ICMP, UDP Scan

---

## V

### Version Detection
**Category:** Network Scanning

Synonym for service detection. Determining application versions on open ports.

**Intensity Levels:**
- **Light (0-2):** Banner grabbing only, fast
- **Default (7):** Balanced probes and speed
- **Aggressive (9):** All probes, slow but thorough

**Example:**
```bash
# Light detection
prtip -sV --version-intensity 2 -p 80 target.com

# Aggressive detection
prtip -sV --version-intensity 9 -p 80 target.com
```

**Related Terms:** Service Detection, Banner Grabbing, nmap-service-probes

---

## W

### Wireshark
**Category:** General Networking

Network protocol analyzer. Can import ProRT-IP's PCAPNG captures for detailed packet analysis.

**ProRT-IP Integration:**
```bash
# Capture packets during scan
prtip -sS -p 80,443 --capture-raw-responses target.com

# Open in Wireshark
wireshark scan-12345.pcapng
```

**Related Terms:** PCAPNG, Packet Capture, libpcap

---

## X

### X.509
**Category:** TCP/IP Protocols

Standard for public key certificates used in TLS/SSL. ProRT-IP parses X.509v3 certificates during TLS certificate analysis.

**Certificate Structure:**
- **Version:** X.509v3 (most common)
- **Serial Number:** Unique identifier
- **Signature Algorithm:** RSA, ECDSA, EdDSA
- **Issuer:** Certificate authority DN
- **Validity:** Not before/after dates
- **Subject:** Entity DN
- **Public Key:** Algorithm + key bits
- **Extensions:** SAN, Key Usage, Basic Constraints, etc.

**Parsing:** ProRT-IP uses `x509-parser` crate for 1.33μs parsing speed

**Related Terms:** TLS, SNI, Certificate Authority

---

### Xmas Scan
**Category:** Network Scanning

Stealth scan (`-sX`) that sends packets with FIN, PSH, and URG flags set (packet "lit up like a Christmas tree").

**Process:**
- Send FIN+PSH+URG
- Closed port → RST response
- Open port → no response (open|filtered)

**Advantages:**
- Bypasses some non-stateful firewalls
- Stealthier than SYN scan

**Limitations:**
- Windows/Cisco incompatibility (same as FIN/NULL)
- Open|filtered ambiguity

**Performance:** Fastest scan type in ProRT-IP benchmarks (9.7ms for 65K ports, 103K pps)

**Related Terms:** FIN Scan, NULL Scan, Stealth Scanning

---

### XML
**Category:** ProRT-IP Specific

Output format for scan results. ProRT-IP's XML output (`-oX`) is Nmap-compatible.

**Structure:**
```xml
<?xml version="1.0"?>
<nmaprun>
  <scaninfo type="syn" protocol="tcp" numservices="2" services="80,443"/>
  <host>
    <address addr="93.184.216.34" addrtype="ipv4"/>
    <ports>
      <port protocol="tcp" portid="80">
        <state state="open"/>
        <service name="http" product="Apache" version="2.4.41"/>
      </port>
    </ports>
  </host>
</nmaprun>
```

**Use Cases:**
- Automated parsing
- Tool integration (Metasploit, Burp Suite)
- Historical comparison

**Related Terms:** JSON, Nmap, Output Formats

---

## Z

### Zero-Copy
**Category:** Performance & Optimization

Avoiding data copies between userspace and kernel space for improved performance.

**Techniques:**
- **mmap():** Memory-mapped I/O
- **sendfile():** Direct kernel-to-kernel transfer
- **splice():** Pipe-based zero-copy

**ProRT-IP:** Uses zero-copy for packets >10KB during result streaming

**Future:** AF_XDP sockets for userspace packet processing without kernel copies

**Related Terms:** Performance Tuning, Kernel Bypass, mmap

---

### Zombie Host
**Category:** Network Scanning

Idle system used as intermediary for idle scanning. Must have:

1. **Predictable IP ID:** Sequential IP ID increments
2. **Idle:** No other network traffic
3. **Reachable:** Responds to probes

**ProRT-IP Idle Scan:**
```bash
prtip -sI zombie.com -p 80,443 target.com
```

**Finding Zombies:**
- Printers, plotters
- Network appliances
- Idle servers
- Test systems

**Related Terms:** Idle Scan, IP ID, IP Spoofing

---

## Additional Resources

**See Also:**
- [Technical Specification v2.0](../reference/tech-spec-v2.md) - Complete technical reference
- [FAQ](../reference/faq.md) - Frequently asked questions
- [User Guide](../user-guide/index.md) - Comprehensive usage guide
- [API Reference](../reference/api-reference.md) - Programming interface documentation

**External Resources:**
- [RFC 791 - IPv4](https://www.rfc-editor.org/rfc/rfc791.html)
- [RFC 793 - TCP](https://www.rfc-editor.org/rfc/rfc793.html)
- [RFC 8200 - IPv6](https://www.rfc-editor.org/rfc/rfc8200.html)
- [Nmap Reference Guide](https://nmap.org/book/man.html)

---

**Last Updated:** 2025-11-15
**Version:** 1.0.0
