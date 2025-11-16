# IPv6 Support

Complete IPv6 support across all scan types with dual-stack scanning capabilities.

## What is IPv6?

**IPv6** (Internet Protocol version 6) is the next-generation internet protocol designed to replace IPv4. With 340 undecillion addresses (2^128), IPv6 solves IPv4 address exhaustion while providing enhanced features for modern networks.

**ProRT-IP IPv6 Capabilities:**
- **100% Scanner Coverage:** All 8 scan types support both IPv4 and IPv6
- **Dual-Stack Resolution:** Automatic hostname resolution to both protocols
- **Protocol Preference:** User-controlled IPv4/IPv6 preference with fallback
- **CIDR Support:** Full IPv6 CIDR notation (/64, /48, etc.) for subnet scanning
- **ICMPv6 & NDP:** Native support for IPv6 discovery protocols
- **Performance Parity:** IPv6 scans match or exceed IPv4 performance

**Version History:**
- Sprint 4.21: TCP Connect IPv6 foundation ✅
- Sprint 5.1 Phase 1: TCP Connect + SYN IPv6 ✅
- Sprint 5.1 Phase 2: UDP + Stealth IPv6 ✅
- Sprint 5.1 Phase 3: Discovery + Decoy IPv6 ✅
- Sprint 5.1 Phase 4: CLI flags, documentation ✅

---

## IPv6 Addressing

### Address Types

#### 1. Global Unicast (2000::/3)

**Internet-routable addresses** (equivalent to public IPv4)

**Format:** `2001:db8:85a3::8a2e:370:7334`

**Usage:**
```bash
# Scan a single global address
prtip -sS -p 80,443 2001:4860:4860::8888

# Scan a /64 subnet (256 addresses)
prtip -sS -p 80,443 2001:db8::0/120
```

**Characteristics:**
- Routable on public internet
- First 48 bits: Global routing prefix
- Next 16 bits: Subnet ID
- Last 64 bits: Interface Identifier (IID)

#### 2. Link-Local (fe80::/10)

**Single network segment** communication (equivalent to APIPA in IPv4)

**Format:** `fe80::1234:5678:90ab:cdef`

**Usage:**
```bash
# Requires interface specification (zone ID)
prtip -sS -p 80,443 fe80::1%eth0        # Linux
prtip -sS -p 80,443 fe80::1%en0         # macOS
prtip -sS -p 80,443 fe80::1%12          # Windows (interface index)
```

**Characteristics:**
- Not routable beyond local link
- Auto-configured on all IPv6 interfaces
- Always start with `fe80::`
- Common for device-to-device communication

#### 3. Unique Local (fc00::/7)

**Private IPv6 networks** (equivalent to RFC 1918 in IPv4)

**Format:** `fd00:1234:5678:90ab::1`

**Usage:**
```bash
# Scan ULA address
prtip -sS -p 22,80,443 fd12:3456:789a:1::1

# Scan /48 organization network
prtip -sS -p 22,80,443 fd00:1234:5678::/48
```

**Characteristics:**
- Not routable on public internet
- Unique within organization
- fc00::/7 range (fd00::/8 for locally assigned)

#### 4. Multicast (ff00::/8)

**One-to-many** communication

**Common Addresses:**
- `ff02::1` - All nodes on local link
- `ff02::2` - All routers on local link
- `ff02::1:ffXX:XXXX` - Solicited-node multicast (NDP)

**Usage:**
```bash
# Scan all nodes on local link (may be blocked)
prtip -sS -p 80,443 ff02::1
```

#### 5. Loopback (::1/128)

**Local host testing** (equivalent to 127.0.0.1 in IPv4)

**Format:** `::1`

**Usage:**
```bash
# Test local services
prtip -sS -p 80,443 ::1

# Service detection on loopback
prtip -sT -sV -p 22,80,443,3306,5432 ::1
```

**Characteristics:**
- Single address (not a subnet)
- Always refers to local system
- Ideal for scanner validation tests

### Address Notation

#### Full Format
```
2001:0db8:85a3:0000:0000:8a2e:0370:7334
```

#### Compressed Format (Recommended)
```
2001:db8:85a3::8a2e:370:7334
```

**Compression Rules:**
- Leading zeros can be omitted: `0db8` → `db8`
- Consecutive zero blocks become `::` (only once): `0000:0000` → `::`
- Use lowercase hexadecimal (convention)

#### CIDR Notation

```bash
# Common prefix lengths
2001:db8::1/128          # Single host
2001:db8::/64            # Single subnet (18.4 quintillion addresses)
2001:db8::/48            # Medium organization (65,536 subnets)
2001:db8::/32            # Large organization or ISP
```

**Scanning Guidelines:**
- **/128:** Single host
- **/120:** 256 hosts (manageable)
- **/112:** 65,536 hosts (slow but feasible)
- **/64:** 18.4 quintillion hosts (**NEVER scan fully**)

---

## CLI Flags

### Primary Flags

#### `-6` / `--ipv6` - Force IPv6

**Prefer IPv6** addresses when resolving hostnames

```bash
# Force IPv6 resolution
prtip -sS -6 -p 80,443 example.com

# Mixed targets (hostname→IPv6, literals unchanged)
prtip -sS -6 -p 80,443 example.com 192.168.1.1 2001:db8::1
```

**Behavior:**
- Hostnames resolve to AAAA records (IPv6)
- IPv4/IPv6 literals remain unchanged
- Falls back to IPv4 if no AAAA record

**Nmap Compatible:** ✅ Equivalent to `nmap -6`

#### `-4` / `--ipv4` - Force IPv4

**Prefer IPv4** addresses when resolving hostnames

```bash
# Force IPv4 resolution
prtip -sS -4 -p 80,443 example.com
```

**Behavior:**
- Hostnames resolve to A records (IPv4)
- IPv4/IPv6 literals remain unchanged
- Falls back to IPv6 if no A record

**Nmap Compatible:** ✅ Equivalent to `nmap -4`

### Advanced Flags

#### `--prefer-ipv6` - Prefer with Fallback

**Use IPv6 when available**, fall back to IPv4

```bash
# Prefer IPv6, graceful degradation
prtip -sS --prefer-ipv6 -p 80,443 dual-stack.example.com
```

**Use Case:** Testing IPv6 connectivity before IPv6-only deployment

#### `--prefer-ipv4` - Prefer with Fallback

**Use IPv4 when available**, fall back to IPv6

```bash
# Prefer IPv4 (default behavior)
prtip -sS --prefer-ipv4 -p 80,443 example.com
```

**Use Case:** Legacy networks, gradual IPv6 migration

#### `--ipv6-only` - Strict IPv6 Mode

**Reject all IPv4 addresses**, IPv6 only

```bash
# IPv6-only scan (error on IPv4 targets)
prtip -sS --ipv6-only -p 80,443 2001:db8::/64

# Error: IPv4 address in IPv6-only mode
prtip -sS --ipv6-only -p 80,443 192.168.1.1
# Error: Target 192.168.1.1 is IPv4, but --ipv6-only specified
```

**Use Case:** IPv6-only networks, security assessments requiring IPv6 purity

#### `--ipv4-only` - Strict IPv4 Mode

**Reject all IPv6 addresses**, IPv4 only

```bash
# IPv4-only scan (error on IPv6 targets)
prtip -sS --ipv4-only -p 80,443 192.168.1.0/24
```

**Use Case:** Legacy networks, IPv4-only security assessments

### Flag Conflicts

**Invalid Combinations:**
```bash
# Error: Cannot specify both -6 and -4
prtip -sS -6 -4 -p 80,443 example.com

# Error: Conflicting preferences
prtip -sS --ipv6-only --prefer-ipv4 -p 80,443 example.com
```

**Valid Combinations:**
```bash
# OK: Preference flags are compatible
prtip -sS -6 --prefer-ipv6 -p 80,443 example.com
```

---

## Scanner-Specific Behavior

### 1. TCP Connect (`-sT`)

**IPv6 Support:** ✅ Full dual-stack (Sprint 5.1 Phase 1)

**Description:** Full TCP three-way handshake using OS TCP stack

```bash
# IPv6 single host
prtip -sT -p 80,443 2001:db8::1

# IPv6 CIDR
prtip -sT -p 22,80,443 2001:db8::/120

# Dual-stack target list
prtip -sT -p 80,443 192.168.1.1 2001:db8::1 example.com
```

**Behavior:**
- Uses kernel TCP stack (no raw sockets)
- No root privileges required
- Automatic IPv4/IPv6 socket creation
- Full connection establishment

**Performance:**
- IPv6 overhead: <5% vs IPv4
- Loopback: ~5ms for 6 ports
- LAN: ~20-50ms depending on RTT

**Port States:**
- **Open:** SYN → SYN+ACK → ACK completed
- **Closed:** SYN → RST received
- **Filtered:** SYN timed out

### 2. SYN Scanner (`-sS`)

**IPv6 Support:** ✅ Full dual-stack (Sprint 5.1 Phase 1)

**Description:** Half-open scanning (SYN without completing handshake)

**Requires:** Root/administrator (raw socket access)

```bash
# IPv6 SYN scan
sudo prtip -sS -p 80,443 2001:db8::1

# IPv6 subnet
sudo prtip -sS -p 1-1000 2001:db8::/120

# Dual-stack with IPv6 preference
sudo prtip -sS -6 -p 80,443 example.com
```

**Behavior:**
- Sends SYN, waits for SYN+ACK or RST
- Sends RST to abort (no full handshake)
- Stealthier than Connect scan
- Automatic IPv6 pseudo-header checksum

**Performance:**
- IPv6 overhead: <10% vs IPv4
- Loopback: ~10ms for 6 ports
- LAN: ~15-40ms

**IPv6 Considerations:**
- IPv6 header: 40 bytes (vs 20 bytes IPv4)
- TCP checksum includes IPv6 addresses
- No fragmentation by default

### 3. UDP Scanner (`-sU`)

**IPv6 Support:** ✅ Full dual-stack (Sprint 5.1 Phase 2)

**Description:** UDP datagrams with protocol-specific payloads

**Requires:** Root/administrator (raw ICMP socket)

```bash
# IPv6 UDP scan (common services)
sudo prtip -sU -p 53,123,161 2001:db8::1

# IPv6 subnet DNS scan
sudo prtip -sU -p 53 2001:db8::/120
```

**Behavior:**
- Sends UDP datagrams to target ports
- Waits for UDP response or ICMPv6 Port Unreachable
- Protocol-specific payloads (DNS, SNMP, NTP, mDNS, DHCPv6)
- Interprets ICMPv6 Type 1 Code 4 as "closed"

**Performance:**
- **Slower than TCP:** 10-100x due to stateless nature
- IPv6 overhead: <5% vs IPv4
- Timeout-dependent (use T4 or T5)

**Protocol Payloads (IPv6-compatible):**
- DNS (53): version.bind TXT query
- SNMP (161): GetRequest for sysDescr.0
- NTP (123): Mode 3 client request
- mDNS (5353): _services._dns-sd._udp.local PTR
- DHCPv6 (547): SOLICIT message

**Port States:**
- **Open:** UDP response received
- **Closed:** ICMPv6 Port Unreachable
- **Open|Filtered:** No response (timeout)
- **Filtered:** ICMPv6 Administratively Prohibited

### 4. Stealth Scanners (`-sF`, `-sN`, `-sX`, `-sA`)

**IPv6 Support:** ✅ Full dual-stack (Sprint 5.1 Phase 2)

**Description:** Unusual TCP flag combinations to evade firewalls

**Requires:** Root/administrator (raw sockets)

#### FIN Scan (`-sF`)
```bash
sudo prtip -sF -p 80,443 2001:db8::1
```
- Sends FIN flag only
- Open: No response | Closed: RST

#### NULL Scan (`-sN`)
```bash
sudo prtip -sN -p 80,443 2001:db8::1
```
- No flags set
- Open: No response | Closed: RST

#### Xmas Scan (`-sX`)
```bash
sudo prtip -sX -p 80,443 2001:db8::1
```
- FIN+PSH+URG flags ("lit up like Christmas")
- Open: No response | Closed: RST

#### ACK Scan (`-sA`)
```bash
sudo prtip -sA -p 80,443 2001:db8::1
```
- ACK flag only (firewall detection)
- Unfiltered: RST | Filtered: No response

**Port States:**
- **Open|Filtered:** No response (timeout)
- **Closed:** RST received
- **Filtered:** ICMP unreachable

**IPv6 Considerations:**
- IPv6 firewalls may behave differently
- Stateful firewalls often block these scans
- Windows doesn't follow RFC 793 for closed ports

### 5. Discovery Engine (`--discovery`)

**IPv6 Support:** ✅ Full ICMPv6 & NDP (Sprint 5.1 Phase 3)

**Description:** Host discovery using ICMP Echo and NDP

**Requires:** Root/administrator (raw ICMP socket)

```bash
# IPv6 host discovery
sudo prtip --discovery 2001:db8::/120

# Dual-stack discovery
sudo prtip --discovery 192.168.1.0/24 2001:db8::/120

# Discovery then scan
sudo prtip --discovery --discovery-then-scan -p 80,443 2001:db8::/120
```

**Protocols:**

#### ICMPv6 Echo Request/Reply
- Type 128: Echo Request (IPv6 ICMP Type 8 equivalent)
- Type 129: Echo Reply (IPv6 ICMP Type 0 equivalent)
- Basic host liveness check

#### NDP Neighbor Discovery (RFC 4861)
- Type 135: Neighbor Solicitation (NS)
- Type 136: Neighbor Advertisement (NA)
- Link-layer address resolution + host discovery
- More reliable than Echo on local links

**Solicited-Node Multicast:**
```
Target Address: 2001:db8::1234:5678
Solicited-Node: ff02::1:ff34:5678
                          ^^^^^^^^
                          Last 24 bits
```

**Performance:**
- ICMPv6 Echo: ~20-50ms per host
- NDP: ~10-30ms on local link (faster)
- Combined: ~50-100ms per host
- Scales linearly with CPU cores

### 6. Decoy Scanner (`-D`)

**IPv6 Support:** ✅ Full dual-stack with /64-aware generation (Sprint 5.1 Phase 3)

**Description:** Obscure source by generating traffic from multiple IPs

**Requires:** Root/administrator (source spoofing)

```bash
# IPv6 decoy scan (5 random decoys)
sudo prtip -sS -D RND:5 -p 80,443 2001:db8::1

# Manual decoy list
sudo prtip -sS -D 2001:db8::10,2001:db8::20,ME,2001:db8::30 \
    -p 80,443 2001:db8::1

# Subnet scan with decoys
sudo prtip -sS -D RND:10 -p 80,443 2001:db8::/120
```

**Behavior:**
- Sends packets from real IP + decoy IPs
- Decoy IPs are spoofed (source manipulation)
- Target sees traffic from N+1 sources
- ME keyword specifies real IP position

**IPv6 Decoy Generation:**
- Random /64 Interface Identifiers
- Subnet-aware (uses target's network prefix)
- Avoids 7 reserved ranges:
  1. Loopback (::1/128)
  2. Multicast (ff00::/8)
  3. Link-local (fe80::/10)
  4. ULA (fc00::/7)
  5. Documentation (2001:db8::/32)
  6. IPv4-mapped (::ffff:0:0/96)
  7. Unspecified (::/128)

**IPv6 /64 Rationale:**
- Most IPv6 subnets are /64
- Decoys within same /64 more believable
- SLAAC uses /64 boundaries
- NDP operates within /64 scope

**Performance:**
- 2-5% overhead per decoy
- 5 decoys: ~10-25% total overhead
- 10 decoys: ~20-50% total overhead

**Limitations:**
- Egress filtering may block spoofed packets
- Return packets only reach real IP
- Modern IDS can correlate timing patterns

---

## Protocol Details

### ICMPv6 Message Types

| Type | Name | Purpose | Scanner |
|------|------|---------|---------|
| 1 | Destination Unreachable | Port closed indication | UDP, Stealth |
| 3 | Time Exceeded | Firewall/router drop | All |
| 128 | Echo Request | Host discovery | Discovery |
| 129 | Echo Reply | Host alive | Discovery |
| 135 | Neighbor Solicitation | NDP resolution | Discovery |
| 136 | Neighbor Advertisement | NDP response | Discovery |

#### Type 1: Destination Unreachable

**Codes:**
- **0:** No route to destination
- **1:** Communication administratively prohibited (filtered)
- **3:** Address unreachable (host down)
- **4:** Port unreachable (closed port)

**ProRT-IP Interpretation:**
```rust
// Code 4 = closed port
if icmpv6_type == 1 && icmpv6_code == 4 {
    port_state = PortState::Closed;
}

// Code 1 = firewall filtering
if icmpv6_type == 1 && icmpv6_code == 1 {
    port_state = PortState::Filtered;
}
```

#### Type 135/136: NDP

**Solicited-Node Multicast Example:**
```
Target: 2001:db8::1234:5678:9abc:def0
Multicast: ff02::1:ff9a:bcdef0
```

**ProRT-IP NDP Flow:**
1. Build NS packet with target IPv6
2. Calculate solicited-node multicast (ff02::1:ffXX:XXXX)
3. Send to multicast (all nodes on link process)
4. Wait for NA with target's link-layer address
5. Mark host alive if NA received

**Performance:**
- NDP faster than Echo on local links (~10-30ms vs 20-50ms)
- Bypasses ICMP filtering (NDP required for IPv6)
- Only works within L2 segment

### TCP Over IPv6

#### IPv6 Pseudo-Header for Checksum

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Source Address                        |
|                            (128 bits)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                      Destination Address                      |
|                            (128 bits)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                   TCP Length                  |     Zeros     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Key Differences from IPv4:**
- IPv6 addresses: 128 bits (16 bytes) each
- No IP header checksum (delegated to link layer)
- TCP checksum includes full IPv6 addresses
- Pseudo-header is 40 bytes (vs 12 bytes IPv4)

### UDP Over IPv6

**Checksum:**
- Same pseudo-header format as TCP
- UDP checksum **mandatory** in IPv6 (optional in IPv4)
- Zero checksum is invalid in IPv6

---

## Performance Characteristics

### IPv4 vs IPv6 Comparison

| Metric | IPv4 | IPv6 | Overhead |
|--------|------|------|----------|
| **Header Size** | 20 bytes | 40 bytes | +100% |
| **Checksum Calculation** | IP + TCP/UDP | TCP/UDP only | -50% CPU |
| **Address Resolution** | ARP (broadcast) | NDP (multicast) | -90% traffic |
| **Loopback Latency** | ~5ms | ~5-7ms | +0-40% |
| **LAN Latency** | ~20ms | ~20-25ms | +0-25% |
| **WAN Latency** | ~50ms | ~50-60ms | +0-20% |
| **Throughput (1Gbps)** | 95 Mbps | 92 Mbps | -3% |

**Conclusion:** IPv6 overhead negligible on modern hardware (<5-10% in most scenarios)

### Timeout Recommendations

| Scan Type | IPv4 Default | IPv6 Recommended | Reason |
|-----------|--------------|------------------|--------|
| TCP Connect | 2000ms | 2500ms | Slightly higher RTT |
| SYN Scan | 1000ms | 1500ms | ICMPv6 processing delay |
| UDP Scan | 3000ms | 3500ms | ICMPv6 unreachable path |
| Discovery | 500ms | 750ms | NDP multicast delay |
| Stealth | 2000ms | 2500ms | Firewall processing |

**Timing Template Adjustments:**
```bash
# T3 (Normal) - Increased timeouts for IPv6
prtip -sS -T3 -p 80,443 2001:db8::1  # 2.5s timeout

# T4 (Aggressive) - Default IPv4 timeouts OK
prtip -sS -T4 -p 80,443 2001:db8::1  # 1.5s timeout

# T5 (Insane) - Minimal timeout, may miss responses
prtip -sS -T5 -p 80,443 2001:db8::1  # 500ms (risky)
```

---

## Common Use Cases

### 1. Scanning IPv6 Loopback

**Purpose:** Local service enumeration, scanner testing

```bash
# TCP Connect (no privileges)
prtip -sT -p 22,80,443,3306,5432 ::1

# SYN scan (requires root)
sudo prtip -sS -p 1-1000 ::1

# Service detection
prtip -sT -sV -p 80,443 ::1
```

**Expected Output:**
```
Scanning ::1 (IPv6 loopback)...
PORT     STATE  SERVICE  VERSION
22/tcp   open   ssh      OpenSSH 8.9p1
80/tcp   open   http     nginx 1.18.0
443/tcp  open   https    nginx 1.18.0 (TLS 1.3)
3306/tcp open   mysql    MySQL 8.0.30
```

### 2. Scanning Link-Local Addresses

**Purpose:** Local network device discovery

```bash
# Link-local with interface (macOS/Linux)
prtip -sS -p 80,443 fe80::1%eth0

# Link-local subnet
prtip -sS -p 80,443 fe80::/64%eth0

# Discovery on link-local
sudo prtip --discovery fe80::/64%eth0
```

**Platform-Specific Zone IDs:**
- **Linux:** `%eth0`, `%ens33`, `%wlan0`
- **macOS:** `%en0`, `%en1`
- **Windows:** `%12`, `%3` (interface index)
- **FreeBSD:** `%em0`, `%re0`

### 3. Scanning Global Unicast

**Purpose:** Internet-facing service enumeration

```bash
# Single global address
prtip -sS -p 80,443 2001:4860:4860::8888

# Multiple hosts
prtip -sS -p 80,443 2001:db8::1 2606:2800:220:1:248:1893:25c8:1946

# With service detection
prtip -sS -sV -p 80,443 2001:4860:4860::8888
```

### 4. IPv6 CIDR Scanning

**Purpose:** Subnet enumeration (targeted, not full /64)

```bash
# /120 subnet (256 addresses - manageable)
prtip -sS -p 80,443 2001:db8::0/120

# Discovery then port scan (efficient)
sudo prtip --discovery --discovery-then-scan -p 80,443 2001:db8::0/120
```

**CIDR Guidelines:**
- **/120:** 256 hosts (manageable)
- **/112:** 65,536 hosts (slow but feasible)
- **/64:** 18.4 quintillion (**NEVER scan fully**)

### 5. Dual-Stack Hosts

**Purpose:** Test both IPv4 and IPv6 connectivity

```bash
# Prefer IPv6, fallback to IPv4
prtip -sS --prefer-ipv6 -p 80,443 example.com

# Prefer IPv4, fallback to IPv6
prtip -sS --prefer-ipv4 -p 80,443 example.com

# Scan both explicitly
prtip -sS -p 80,443 example.com 2606:2800:220:1:248:1893:25c8:1946
```

### 6. Mixed IPv4/IPv6 Targets

**Purpose:** Heterogeneous network scanning

```bash
# Mixed targets (auto-detect protocol)
prtip -sS -p 80,443 \
    192.168.1.1 \
    2001:db8::1 \
    example.com \
    10.0.0.0/24 \
    2001:db8::/120

# With protocol preference
prtip -sS -6 -p 80,443 \
    192.168.1.1 \      # IPv4 literal (unchanged)
    example.com \       # Resolves to IPv6
    2001:db8::1         # IPv6 literal (unchanged)
```

### 7. IPv6 Service Detection

**Purpose:** Identify services and versions

```bash
# Service detection
prtip -sT -sV -p 22,80,443 2001:db8::1

# Aggressive scan (OS + Service + Scripts)
prtip -sS -A -p- 2001:db8::1

# High intensity
prtip -sT -sV --version-intensity 9 -p 80,443 2001:db8::1
```

### 8. IPv6 Stealth Scanning

**Purpose:** Evade firewalls and IDS

```bash
# FIN scan with timing
sudo prtip -sF -T2 -p 80,443 2001:db8::1

# NULL scan with decoys
sudo prtip -sN -D RND:5 -p 80,443 2001:db8::1
```

### 9. IPv6 Decoy Scanning

**Purpose:** Obscure scan origin

```bash
# Random decoys
sudo prtip -sS -D RND:10 -p 80,443 2001:db8::1

# Manual decoys with ME positioning
sudo prtip -sS -D 2001:db8::10,2001:db8::20,ME,2001:db8::30 \
    -p 80,443 2001:db8::1
```

### 10. Hostname Resolution

**Purpose:** Resolve dual-stack hostnames

```bash
# Default: Prefer IPv4
prtip -sS -p 80,443 example.com

# Force IPv6
prtip -sS -6 -p 80,443 example.com

# Show DNS details
prtip -sS -6 -vvv -p 80,443 example.com
```

---

## Troubleshooting

### Common Issues

#### Issue 1: "IPv6 not supported"

**Error:**
```
Error: IPv6 not supported on this interface
```

**Causes:**
- IPv6 disabled in OS
- No IPv6 address on interface
- Kernel module not loaded

**Solutions:**
```bash
# Check IPv6 status (Linux)
ip -6 addr show
sysctl net.ipv6.conf.all.disable_ipv6

# Enable IPv6 (Linux)
sudo sysctl -w net.ipv6.conf.all.disable_ipv6=0

# Check IPv6 (macOS)
ifconfig | grep inet6

# Enable IPv6 (macOS)
sudo networksetup -setv6automatic Wi-Fi

# Check IPv6 (Windows)
netsh interface ipv6 show config

# Enable IPv6 (Windows)
netsh interface ipv6 install
```

#### Issue 2: NDP Timeouts

**Error:**
```
Warning: NDP timeout for fe80::1%eth0
```

**Causes:**
- Wrong interface
- Firewall blocking ICMPv6 Type 135/136
- Host not on local link

**Solutions:**
```bash
# List interfaces
ip link show  # Linux
ifconfig -a   # macOS

# Verify link-local addresses
ip -6 addr show eth0

# Test NDP manually
ping6 -c 1 -I eth0 ff02::1  # All nodes
```

#### Issue 3: ICMPv6 Unreachable Not Received

**Symptom:** All UDP ports show "open|filtered"

**Causes:**
- Firewall dropping ICMPv6
- Rate limiting on ICMPv6
- Packet loss

**Solutions:**
```bash
# Increase timeout
prtip -sU --timeout 5000 -p 53,123,161 2001:db8::1

# Aggressive timing
prtip -sU -T5 -p 53,123,161 2001:db8::1

# Test with known-closed port
prtip -sU -p 9999 2001:db8::1  # Should be "closed"
```

#### Issue 4: Link-Local Scope Issues

**Error:**
```
Error: Cannot connect to fe80::1: Invalid argument
```

**Cause:** Missing zone ID

**Solution:**
```bash
# WRONG: No zone ID
prtip -sS -p 80,443 fe80::1

# CORRECT: With zone ID
prtip -sS -p 80,443 fe80::1%eth0  # Linux
prtip -sS -p 80,443 fe80::1%en0   # macOS
prtip -sS -p 80,443 fe80::1%12    # Windows
```

#### Issue 5: Firewall Blocking ICMPv6 Echo

**Symptom:** No Echo response, but NDP works

**Cause:** Firewall allows NDP (required) but blocks Echo

**Solutions:**
```bash
# Use NDP-only discovery
sudo prtip --discovery --ndp-only 2001:db8::/120

# Check firewall (Linux)
sudo ip6tables -L -n | grep icmpv6

# Temporarily allow Echo (TESTING ONLY)
sudo ip6tables -I INPUT -p icmpv6 --icmpv6-type echo-request -j ACCEPT
```

### Platform-Specific

#### Linux
```bash
# Use sudo for raw sockets
sudo prtip -sS -p 80,443 2001:db8::1

# OR: Grant CAP_NET_RAW (persistent)
sudo setcap cap_net_raw=eip /path/to/prtip
```

#### macOS
```bash
# Use sudo (required)
sudo prtip -sS -p 80,443 2001:db8::1

# Verify BPF permissions
ls -l /dev/bpf*
```

#### Windows
```powershell
# Install Npcap
# https://npcap.com/

# Verify installation
sc query npcap

# Run as Administrator
```

#### FreeBSD
```bash
# Use sudo
sudo prtip -sS -p 80,443 2001:db8::1

# Verify IPv6 enabled
sysctl net.inet6.ip6.forwarding
```

---

## Best Practices

### 1. When to Use IPv6 vs IPv4

**Use IPv6 When:**
- Target is dual-stack or IPv6-only
- Testing IPv6-specific vulnerabilities
- Assessing IPv6 security posture
- Future-proofing assessments
- ISP/cloud is IPv6-native

**Use IPv4 When:**
- Legacy IPv4-only networks
- IPv6 firewall too restrictive
- Faster scan needed (slight edge)

**Use Both When:**
- Comprehensive security assessment
- Different firewall rules per protocol
- Comparing service availability

### 2. Protocol Preference Strategies

#### Default (Prefer IPv4)
```bash
# No flags = prefer IPv4
prtip -sS -p 80,443 example.com
```
**Use Case:** General scanning, legacy networks

#### Prefer IPv6
```bash
# Prefer IPv6, fallback IPv4
prtip -sS --prefer-ipv6 -p 80,443 example.com
```
**Use Case:** Modern networks, cloud, testing

#### Force IPv6 Only
```bash
# Strict IPv6 (error on IPv4)
prtip -sS --ipv6-only -p 80,443 2001:db8::/120
```
**Use Case:** IPv6-only networks, audits

#### Scan Both
```bash
# Explicit IPv4 + IPv6
prtip -sS -p 80,443 example.com \
    $(dig +short example.com A) \
    $(dig +short example.com AAAA)
```
**Use Case:** Compare protocol parity

### 3. Performance Optimization

#### Aggressive Timing
```bash
# T4/T5 for IPv6
prtip -sS -T4 -p 80,443 2001:db8::/120
```
**Rationale:** IPv6 slightly higher latency, aggressive timing compensates

#### Increase Parallelism
```bash
# High concurrency for /120
prtip -sS --max-concurrent 500 -p 80,443 2001:db8::/120
```
**Rationale:** IPv6 benefits from higher parallelism

#### Use NDP for Local Discovery
```bash
# 2-3x faster than Echo
sudo prtip --discovery --ndp-only fe80::/64%eth0
```
**Rationale:** NDP multicast more efficient than ICMP unicast

### 4. Security Considerations

#### IPv6-Specific Attack Surfaces

**Router Advertisement Spoofing:**
- Use RA Guard on switches
- Monitor for unexpected RAs

**NDP Exhaustion:**
- Implement NDP rate limiting
- Use ND Inspection

**Extension Header Abuse:**
- Drop excessive extension headers

**Tunneling (6to4, Teredo):**
- Scan for tunnel endpoints (UDP 3544)

#### Scanning Etiquette

**Rate Limiting:**
```bash
# Polite scan
prtip -sS -T2 --max-rate 100 -p 80,443 2001:db8::/120
```

**Avoid Full /64:**
```bash
# NEVER: Full /64
# prtip -sS -p 80,443 2001:db8::/64

# GOOD: Targeted /120
prtip -sS -p 80,443 2001:db8::/120
```

**Respect Firewall Responses:**
- ICMPv6 Administratively Prohibited = stop
- No response = timeout indicates firewall

---

## Advanced Topics

### 1. IPv6 Fragmentation

**Difference from IPv4:**
- IPv6 routers do NOT fragment (only sender)
- Path MTU Discovery mandatory
- Minimum MTU: 1280 bytes (vs 68 IPv4)

**Fragmentation Header:**
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Next Header  |   Reserved    |      Fragment Offset    |Res|M|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Identification                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Fields:**
- Next Header: Protocol after reassembly
- Fragment Offset: 13 bits (8-byte units)
- M flag: More fragments (1=more, 0=last)
- Identification: 32 bits (unique per packet)

### 2. Extension Headers

**Common Extension Headers:**
1. Hop-by-Hop Options (0)
2. Routing (43)
3. Fragment (44)
4. Destination Options (60)
5. Authentication (AH) (51)
6. ESP (50)

**Processing Order:**
```
IPv6 Header
  → Hop-by-Hop
    → Routing
      → Fragment
        → Destination Options
          → TCP/UDP/ICMP
```

### 3. Privacy Addresses (RFC 4941)

**Purpose:** Prevent address-based tracking

**Mechanism:**
- Temporary addresses from random IIDs
- Change every 1-7 days
- Original address still used for servers

**ProRT-IP Considerations:**
```bash
# Stable address (consistent)
prtip -sS -p 80,443 2001:db8::1234:5678:90ab:cdef

# Privacy address (may change)
prtip -sS -p 80,443 2001:db8::a3f1:2b4c:9d8e:7f61
```

### 4. Solicited-Node Multicast

**Purpose:** Efficient neighbor resolution

**Format:**
```
Target:  2001:0db8::1234:5678:9abc:def0
Multicast: ff02::1:ff9a:bcdef0
           ^^^^^^^^^^^^^^
           ff02::1:ff + last 24 bits
```

**Algorithm:**
```rust
fn solicited_node_multicast(target: Ipv6Addr) -> Ipv6Addr {
    let octets = target.octets();
    let last_24 = [octets[13], octets[14], octets[15]];

    Ipv6Addr::new(
        0xff02, 0, 0, 0, 0, 1,
        0xff00 | (last_24[0] as u16),
        ((last_24[1] as u16) << 8) | (last_24[2] as u16),
    )
}
```

### 5. DHCPv6 vs SLAAC

**SLAAC:**
- No DHCP server
- Address = Prefix + EUI-64/random
- Fast, automatic, stateless

**DHCPv6:**
- Centralized management
- Stateful (tracks leases)
- Provides DNS, NTP, etc.

**Scanning:**
```bash
# SLAAC (predictable EUI-64)
prtip -sS -p 80,443 2001:db8::211:22ff:fe33:4455

# DHCPv6 (query server for list)
prtip -sS -p 80,443 $(cat dhcpv6-leases.txt)
```

---

## See Also

- **[Stealth Scanning](./stealth-scanning.md)** - Evasion techniques work with IPv6
- **[User Guide: Basic Usage](../user-guide/basic-usage.md)** - IPv6 target specification
- **[Architecture](../../00-ARCHITECTURE.md)** - IPv6 packet building design
- **[Nmap Compatibility](./nmap-compatibility.md)** - IPv6 flag compatibility

**External Resources:**
- [Nmap IPv6 Documentation](https://nmap.org/book/ipv6.html)
- [IANA IPv6 Address Space](https://www.iana.org/assignments/ipv6-address-space/)
- [RFC 8200: IPv6 Specification](https://tools.ietf.org/html/rfc8200)
- [RFC 4861: NDP Protocol](https://tools.ietf.org/html/rfc4861)

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
**Sprint:** 5.1 (100% IPv6 Coverage)
