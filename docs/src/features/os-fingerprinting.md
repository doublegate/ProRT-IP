# OS Fingerprinting

Detect operating systems through TCP/IP stack behavior analysis.

## What is OS Fingerprinting?

**OS Fingerprinting** determines a target's operating system by analyzing unique characteristics in its TCP/IP stack implementation. Different operating systems implement TCP/IP slightly differently—these subtle variations create distinctive "fingerprints" that can be used for identification.

**ProRT-IP Implementation:**
- **2,600+ OS signatures** (Linux, Windows, BSD, macOS, embedded devices, IoT)
- **16-probe sequence** inspired by Nmap's methodology
- **95% accuracy** on well-known operating systems
- **Passive and active** detection modes

**Use Cases:**
- **Security Auditing**: Identify unpatched or end-of-life operating systems
- **Network Inventory**: Catalog OS versions across infrastructure
- **Vulnerability Assessment**: Match OS versions to CVE databases
- **Compliance**: Verify approved OS versions in production

---

## How It Works

### 16-Probe Sequence

ProRT-IP sends a carefully crafted sequence of 16 probes to elicit distinctive responses from the target's TCP/IP stack.

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

**Timing Strategy:**
- Probes 1-6 spaced 100ms apart (ISN sequence analysis)
- Probes 7-16 sent in rapid succession
- Total duration: 600-800ms typical

### Fingerprint Attributes

The 16 probes collect specific attributes that form the OS fingerprint:

#### TCP ISN Analysis

- **GCD (Greatest Common Divisor)**: GCD of ISN deltas between probes 1-6
  - Linux: Often divisible by large prime (e.g., 64000)
  - Windows: Often 1 (random ISN)
  - BSD: Typically 64000

- **ISR (ISN Counter Rate)**: Increments per second
  - Linux 2.4-3.x: ~800,000/sec
  - Windows XP-10: Random (0)
  - FreeBSD: ~64,000/sec

- **SP (Sequence Predictability)**: 0-255 scale (0 = unpredictable)
  - Modern OS: 0-3 (highly random)
  - Legacy OS: 100-255 (predictable)

#### TCP Options

- **O (Option Ordering)**: Order TCP options appear
  - **M** = MSS (Maximum Segment Size)
  - **W** = Window Scale
  - **T** = Timestamp
  - **S** = SACK (Selective Acknowledgment)

- **Examples:**
  - Linux: "MWTS" (MSS, Window, Timestamp, SACK)
  - Windows: "MWST" (different order)
  - BSD: "MWT" (no SACK)

#### IP ID Generation

- **TI (TCP IP ID)**: IP ID generation for TCP responses
  - **I** = Incremental (e.g., Linux)
  - **RI** = Random Incremental (e.g., Windows)
  - **Z** = Zero (e.g., OpenBSD)

- **CI (Closed IP ID)**: IP ID for closed port responses
- **II (ICMP IP ID)**: IP ID for ICMP responses

#### Window Sizes

- **W**: TCP window sizes for each probe
- **Format**: "W=4000|8000" means 16384 or 32768 bytes
- **Variation**: Different probes may elicit different window sizes

**Example Fingerprint:**
```
OS: Linux 5.15-6.1 (Ubuntu 22.04)
GCD: 1
ISR: 800000
SP: 0-3
O: MWTS
TI: I
CI: I
II: I
W: 8000|8000|8000|8000|8000|8000
```

---

## Usage

### Basic OS Detection

Scan with OS fingerprinting enabled:

```bash
sudo prtip -sS -O -p 1-1000 192.168.1.10
```

**Explanation:**
- `-O`: Enable OS detection
- Requires **at least one open** and **one closed port** for accuracy
- Sends 16-probe sequence after port scan completes

### Combined with Service Detection

Get comprehensive system information:

```bash
sudo prtip -sS -sV -O -p 1-1000 192.168.1.10 -oN system-audit.txt
```

**Expected Output:**
```
PORT     STATE  SERVICE     VERSION
22/tcp   open   ssh         OpenSSH 8.9p1 Ubuntu 3ubuntu0.1
80/tcp   open   http        Apache httpd 2.4.52 ((Ubuntu))
3306/tcp closed mysql

OS Detection Results:
OS: Linux 5.15 - 6.1 (Ubuntu 22.04)
Confidence: 95%
CPE: cpe:/o:canonical:ubuntu_linux:22.04

Scan complete: 1000 ports scanned, 2 open, 1 closed
Duration: 12.3s
```

### Aggressive Scan (All Features)

Enable OS detection, service detection, and default scripts:

```bash
sudo prtip -A -p 1-1000 192.168.1.10
```

**Equivalent to:**
```bash
sudo prtip -sV -O -sC --traceroute -p 1-1000 192.168.1.10
```

### Network-Wide OS Inventory

Scan entire subnet and identify OS versions:

```bash
sudo prtip -sS -O -p 22,80,443 192.168.1.0/24 -oJ os-inventory.json
```

**Post-Processing (extract OS data):**
```bash
cat os-inventory.json | jq '.hosts[] | {ip: .address, os: .os.name, confidence: .os.accuracy}'
```

---

## Understanding Results

### Output Format

```
OS Detection Results:
OS: Linux 5.15 - 6.1 (Ubuntu 22.04)
Confidence: 95%
CPE: cpe:/o:canonical:ubuntu_linux:22.04
```

**Fields:**
- **OS**: Operating system name and version range
- **Confidence**: Accuracy percentage (0-100%)
- **CPE**: Common Platform Enumeration identifier (for CVE matching)

### Confidence Levels

| Confidence | Meaning | Action |
|------------|---------|--------|
| 95-100% | Very high confidence | Trust result |
| 85-94% | High confidence | Likely accurate |
| 70-84% | Medium confidence | Review manually |
| 50-69% | Low confidence | Multiple possibilities |
| <50% | Very low confidence | Insufficient data |

### Example Results Analysis

#### Example 1: Linux Server

```
OS: Linux 5.15 - 6.1 (Ubuntu 22.04)
Confidence: 95%
CPE: cpe:/o:canonical:ubuntu_linux:22.04

Fingerprint Details:
- GCD: 1 (random ISN)
- ISR: ~800,000/sec (typical Linux)
- TCP Options: MWTS (Linux signature)
- IP ID: Incremental (Linux default)
```

**Analysis:**
- **High confidence** (95%) indicates reliable detection
- **Kernel range 5.15-6.1** matches Ubuntu 22.04 LTS
- **CPE identifier** can be used for CVE database queries
- **Action**: Verify kernel version with `uname -r` on host

#### Example 2: Windows Desktop

```
OS: Microsoft Windows 10 Build 19041-19044 (version 21H1-22H2)
Confidence: 92%
CPE: cpe:/o:microsoft:windows_10

Fingerprint Details:
- GCD: 1 (random ISN)
- ISR: 0 (random, not counter-based)
- TCP Options: MWST (Windows order)
- IP ID: Random Incremental
- Window: 8192 (Windows default)
```

**Analysis:**
- **Build range** narrows down Windows 10 update version
- **TCP option order "MWST"** is Windows-specific
- **Random ISN** (ISR=0) typical of modern Windows
- **Action**: Check for security patches

#### Example 3: Embedded Device (IoT)

```
OS: Embedded Linux (likely OpenWrt or DD-WRT)
Confidence: 78%

Fingerprint Details:
- GCD: 64000 (fixed increment)
- Limited TCP options support
- Unusual window size (512 bytes)
```

**Analysis:**
- **Medium confidence** (78%) suggests custom implementation
- **Small window size** indicates resource-constrained device
- **Limited options** typical of lightweight TCP/IP stacks
- **Action**: Manual verification recommended

#### Example 4: Firewall Blocking

```
OS: Detection failed - insufficient responses
Confidence: 0%

Reason: No open ports found, or firewall blocking probes
```

**Analysis:**
- **All ports filtered** or **stateful firewall** intercepting probes
- Requires **at least one open port** for fingerprinting
- **Action**: Try different ports or timing templates

---

## Accuracy and Confidence

### Factors Affecting Accuracy

**Positive Factors:**
- ✅ Multiple open and closed ports available
- ✅ Recent OS version with known fingerprint
- ✅ Default TCP/IP stack configuration
- ✅ Direct network path (no proxies/NAT)

**Negative Factors:**
- ❌ All ports filtered by firewall
- ❌ Custom TCP/IP stack tuning
- ❌ Network address translation (NAT)
- ❌ Load balancers or proxies

### Signature Database

ProRT-IP includes **2,600+ OS signatures** covering:

**Major Operating Systems:**
- **Linux**: Ubuntu, Debian, RHEL, CentOS, Fedora, Arch, Gentoo
- **Windows**: XP, Vista, 7, 8, 8.1, 10, 11, Server 2008-2022
- **BSD**: FreeBSD, OpenBSD, NetBSD, DragonFly BSD
- **macOS**: 10.x (Yosemite - Monterey), 11.x (Big Sur), 12.x (Monterey)

**Specialized Systems:**
- **Embedded**: OpenWrt, DD-WRT, Tomato, pfSense, OPNsense
- **IoT Devices**: Raspberry Pi OS, Armbian, Yocto-based
- **Network Equipment**: Cisco IOS, Juniper Junos, MikroTik RouterOS
- **Virtualization**: VMware ESXi, Citrix XenServer, Proxmox

### Fingerprint Matching Algorithm

1. **Collect Responses**: Send 16-probe sequence, record all responses
2. **Extract Attributes**: Parse ISN, options, IP ID, window sizes
3. **Calculate Similarity**: Compare against 2,600+ signatures
4. **Rank Matches**: Sort by similarity score (0-100%)
5. **Return Best Match**: Highest scoring match with confidence level

---

## Best Practices

### 1. Ensure Open and Closed Ports

OS fingerprinting requires:
- **At least 1 open port** (for TCP handshake analysis)
- **At least 1 closed port** (for RST packet analysis)

**Pre-scan to identify ports:**
```bash
# First: Quick port scan
sudo prtip -F 192.168.1.10

# Then: OS detection on known open/closed ports
sudo prtip -O -p 22,80,443,8080 192.168.1.10
```

### 2. Use Appropriate Timing

OS detection is sensitive to timing variations:

```bash
# Fast local network
sudo prtip -sS -O -T4 -p 1-1000 192.168.1.10

# Internet target (allow retries)
sudo prtip -sS -O -T3 -p 1-1000 target.example.com

# Slow/unstable connection
sudo prtip -sS -O -T2 -p 1-1000 remote-host.com
```

### 3. Combine with Service Detection

Service banners provide additional OS hints:

```bash
sudo prtip -sS -sV -O -p 1-1000 192.168.1.10
```

**Example: Cross-validation**
```
Service: OpenSSH 8.9p1 Ubuntu 3ubuntu0.1
OS: Linux 5.15-6.1 (Ubuntu 22.04)
✅ Consistent: SSH banner confirms Ubuntu 22.04
```

### 4. Handle Firewalls Gracefully

If OS detection fails due to filtering:

```bash
# Try different ports
sudo prtip -O -p 80,443,8080,22,23,21,25,3389 target.com

# Use aggressive timing (more retries)
sudo prtip -O -T4 --max-retries 3 -p 1-1000 target.com

# Combine with other scans
sudo prtip -A -p 1-1000 target.com
```

### 5. Regular Signature Updates

Keep OS signatures up-to-date:

```bash
# Update ProRT-IP (includes latest signatures)
cargo install prtip --force

# Check signature database version
prtip --version --verbose
```

---

## Troubleshooting

### Issue 1: "OS Detection Failed - Insufficient Responses"

**Cause**: No open or closed ports found, or firewall blocking probes.

**Solutions:**
```bash
# 1. Verify ports are accessible
sudo prtip -sS -p 1-1000 192.168.1.10

# 2. Try more ports
sudo prtip -O -p 1-10000 192.168.1.10

# 3. Use different scan type
sudo prtip -sT -O -p 80,443 192.168.1.10
```

### Issue 2: Low Confidence (<70%)

**Cause**: Ambiguous fingerprint or custom TCP/IP stack.

**Solutions:**
```bash
# 1. Increase probe range
sudo prtip -O -p 1-5000 192.168.1.10

# 2. Combine with service detection for additional hints
sudo prtip -sV -O -p 1-1000 192.168.1.10

# 3. Use aggressive scan
sudo prtip -A -p 1-1000 192.168.1.10
```

### Issue 3: Multiple Possible Matches

**Output:**
```
OS: Linux 5.x (80%) OR Linux 6.x (75%)
Confidence: 80%
```

**Solutions:**
- Cross-check with service banners (SSH, HTTP Server headers)
- Manual verification via SSH: `uname -a`
- Check other network hosts for patterns

### Issue 4: NAT/Proxy Interference

**Cause**: OS fingerprint reflects intermediate device, not target.

**Indicators:**
- Generic "Linux firewall" result
- Low confidence despite many responses
- OS mismatch with service banners

**Solutions:**
```bash
# Disable OS detection, rely on service banners
sudo prtip -sV -p 1-1000 192.168.1.10

# Use traceroute to identify proxies
sudo prtip -A --traceroute -p 80 192.168.1.10
```

---

## Technical Details

### Data Structures

#### OsFingerprint

```rust
pub struct OsFingerprint {
    pub name: String,           // OS name (e.g., "Linux 5.15-6.1")
    pub class: OsClass,         // OS classification
    pub cpe: Vec<String>,       // CPE identifiers
    pub tests: FingerprintTests,// Test results
}
```

#### FingerprintTests

```rust
pub struct FingerprintTests {
    pub seq: SequenceGeneration,  // ISN analysis (GCD, ISR, SP)
    pub ops: TcpOptions,          // TCP option ordering
    pub win: WindowSizes,         // Window size patterns
    pub ecn: EcnResponse,         // ECN flag handling
    pub t1_t7: TcpTests,          // TCP probes 1-7 results
    pub u1: UdpTest,              // UDP probe result
    pub ie: IcmpEchoTests,        // ICMP echo results
}
```

### Integration with Service Detection

OS fingerprinting complements service detection:

1. **Service Detection** identifies software (e.g., "OpenSSH 8.9p1 Ubuntu")
2. **OS Fingerprinting** confirms OS (e.g., "Ubuntu 22.04")
3. **Cross-validation** increases overall confidence

**Example Workflow:**
```
1. Port Scan: Find open port 22
2. Service Detection: "OpenSSH 8.9p1 Ubuntu 3ubuntu0.1"
3. OS Fingerprinting: "Ubuntu 22.04 LTS"
4. Result: High confidence (service + OS agree)
```

---

## See Also

- **[Service Detection](./service-detection.md)** - Protocol-specific detection complementing OS fingerprinting
- **[User Guide: OS Fingerprinting](../user-guide/basic-usage.md#os-fingerprinting)** - Usage examples and interpretation
- **[Architecture: Scanning Engine](../../00-ARCHITECTURE.md)** - How OS fingerprinting fits into scan workflow
- **[Technical Specs: Data Structures](../../02-TECHNICAL-SPECS.md#data-structures)** - Complete OsFingerprint implementation details
- **[Performance Guide](../../21-PERFORMANCE-GUIDE.md)** - OS detection performance characteristics

**External Resources:**
- **Nmap OS Detection**: Original fingerprinting methodology
- **CVE Database**: Match CPE identifiers to known vulnerabilities
- **TCP/IP Illustrated**: Deep dive into TCP/IP stack implementations

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
