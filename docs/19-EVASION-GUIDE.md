# Firewall & IDS Evasion Guide

**ProRT-IP WarScan** - Advanced Stealth Scanning Techniques

**Version:** v0.3.8
**Last Updated:** 2025-10-24
**Sprint:** 4.20 - Packet Fragmentation & TTL Control

---

## Table of Contents

1. [Introduction](#introduction)
2. [Evasion Techniques Overview](#evasion-techniques-overview)
3. [Packet Fragmentation](#packet-fragmentation)
4. [TTL Manipulation](#ttl-manipulation)
5. [Decoy Scanning](#decoy-scanning)
6. [Bad Checksums](#bad-checksums)
7. [Practical Examples](#practical-examples)
8. [Performance Impact Analysis](#performance-impact-analysis)
9. [Detection Considerations](#detection-considerations)
10. [Troubleshooting](#troubleshooting)
11. [Advanced Combinations](#advanced-combinations)
12. [References](#references)

---

## Introduction

### What is Firewall/IDS Evasion?

Firewall and Intrusion Detection System (IDS) evasion refers to techniques used to bypass security controls that monitor or block network traffic. These techniques are essential for:

- **Penetration Testing**: Assessing security defenses by simulating attacker behaviors
- **Red Team Operations**: Testing blue team detection capabilities
- **Security Research**: Understanding how malicious actors evade detection
- **Network Troubleshooting**: Diagnosing firewall/IDS misconfigurations

### Legal and Ethical Considerations

⚠️ **WARNING**: Use these techniques ONLY on networks you own or have explicit written permission to test.

**Authorized Use Cases:**
- Penetration testing engagements with signed contracts
- Internal security assessments on your own infrastructure
- Security research in isolated lab environments
- Educational purposes in controlled environments

**Unauthorized Use is ILLEGAL** and may result in:
- Federal prosecution under Computer Fraud and Abuse Act (CFAA)
- Civil liability and financial penalties
- Professional sanctions and loss of credentials
- Criminal record and imprisonment

### How This Guide is Organized

This guide covers ProRT-IP's evasion capabilities in detail:

1. **Technique Explanations**: How each evasion method works at the packet level
2. **Practical Examples**: Real-world scenarios with command-line usage
3. **Performance Impact**: Speed/accuracy trade-offs for each technique
4. **Detection Risks**: What triggers alerts and how to mitigate
5. **Troubleshooting**: Common issues and solutions

---

## Evasion Techniques Overview

ProRT-IP implements 5 primary evasion techniques, all nmap-compatible:

| Technique | Flag | Purpose | Nmap Equivalent | Detection Risk |
|-----------|------|---------|-----------------|----------------|
| **IP Fragmentation** | `-f` | Split packets into tiny fragments | `nmap -f` | Low-Medium |
| **Custom MTU** | `--mtu <SIZE>` | Control fragment sizes | `nmap --mtu` | Low |
| **TTL Manipulation** | `--ttl <VALUE>` | Set IP Time-To-Live | `nmap --ttl` | Low |
| **Decoy Scanning** | `-D <DECOYS>` | Hide among fake sources | `nmap -D` | Low-High* |
| **Bad Checksums** | `--badsum` | Use invalid checksums | `nmap --badsum` | Medium |

*Detection risk varies based on decoy count and target monitoring sophistication

### Compatibility

All evasion techniques are **fully compatible** with:
- All scan types: `-sS` (SYN), `-sT` (Connect), `-sU` (UDP), `-sF/-sX/-sN` (Stealth)
- Service detection: `-sV`
- OS fingerprinting: `-O`
- All output formats: `-oN`, `-oX`, `-oG`, `-oJ`

---

## Packet Fragmentation

### Overview

IP packet fragmentation splits network packets into smaller fragments, evading firewalls and IDS that:
- Don't properly reassemble fragments before inspection
- Have limited buffer space for fragment reassembly
- Drop fragmented packets as a blanket policy
- Can't apply deep packet inspection to fragments

### How It Works

```
Normal Packet (80 bytes):
┌────────────────────────────────────────────────┐
│ IP Header (20) │ TCP Header (20) │ Data (40) │
└────────────────────────────────────────────────┘

Fragmented (-f flag, MTU 28):
Fragment 1: ┌──────────────────┬────────┐
            │ IP Header (20)   │ 8 data │
            └──────────────────┴────────┘
Fragment 2: ┌──────────────────┬────────┐
            │ IP Header (20)   │ 8 data │
            └──────────────────┴────────┘
Fragment 3: ┌──────────────────┬────────┐
            │ IP Header (20)   │ 8 data │
            └──────────────────┴────────┘
...and so on
```

### CLI Flags

#### `-f` / `--fragment` - Aggressive Fragmentation

**Purpose**: Maximum fragmentation (smallest possible fragments)
**MTU**: 28 bytes (20 IP header + 8 data bytes per fragment)
**Nmap Equivalent**: `nmap -f`

**Syntax:**
```bash
prtip -sS -f -p 1-1000 192.168.1.0/24
```

**When to Use:**
- Evading stateless firewalls
- Bypassing simple packet filters
- Testing fragment reassembly capabilities
- Maximum stealth (most fragments = most confusion)

**Pros:**
- Maximum evasion potential
- Hardest for simple firewalls to inspect
- Creates most fragments (maximum obfuscation)

**Cons:**
- Slowest scan speed (more packets to send)
- Higher bandwidth usage
- May trigger fragmentation alerts
- Some networks drop all fragmented packets

#### `--mtu <SIZE>` - Custom MTU Fragmentation

**Purpose**: Controlled fragmentation with larger fragment sizes
**MTU Range**: 68 - 65535 bytes
**Requirement**: Must be multiple of 8 (fragment offset alignment)

**Syntax:**
```bash
prtip -sS --mtu 200 -p 80,443 target.com
```

**Common MTU Values:**
- **68 bytes**: RFC 791 minimum (48 data bytes per fragment)
- **200 bytes**: Moderate fragmentation (180 data bytes)
- **576 bytes**: Traditional internet minimum (556 data bytes)
- **1500 bytes**: Standard Ethernet MTU (no fragmentation)

**When to Use:**
- Balancing stealth vs. performance
- Matching target network MTU characteristics
- Avoiding obvious fragmentation patterns
- Custom evasion profiles

**Pros:**
- Flexible stealth level (tune to environment)
- Better performance than `-f`
- Can mimic legitimate fragmentation
- Lower bandwidth overhead

**Cons:**
- Requires understanding target network
- Wrong MTU may be ineffective
- Still subject to fragment-based detection

### Technical Details

#### RFC 791 Compliance

ProRT-IP follows RFC 791 IP fragmentation specifications:

1. **Fragment Offset**: Measured in 8-byte units (0-8191 maximum)
2. **More Fragments (MF) Flag**: Set to 1 for all fragments except last
3. **Fragment ID**: Same for all fragments of original packet
4. **Checksum**: Recalculated for each fragment (IP header only)
5. **Header Preservation**: TTL, Protocol, Source/Dest IPs preserved

#### Fragment Reassembly

Target hosts reassemble fragments using:
- Fragment ID (must match)
- Fragment Offset (position in original packet)
- More Fragments flag (indicates last fragment)
- 60-second timeout (RFC 791 recommended)

If any fragment is lost or delayed beyond timeout, the entire packet is discarded.

### Examples

#### Example 1: Basic Aggressive Fragmentation

```bash
# Aggressive fragmentation (28-byte MTU, most stealth)
prtip -sS -f -p 22,80,443 192.168.1.1

# Output shows fragmentation in action:
# [INFO] Using aggressive fragmentation (MTU: 28 bytes)
# [INFO] Each packet will be split into ~3-8 fragments
# 192.168.1.1:22  open  ssh
# 192.168.1.1:80  open  http
# 192.168.1.1:443 open  https
```

#### Example 2: Moderate Fragmentation for Speed

```bash
# Moderate fragmentation (200-byte MTU, balanced)
prtip -sS --mtu 200 -p 1-1000 10.0.0.0/24

# Faster than -f but still evades basic filters
# Output:
# [INFO] Using custom MTU fragmentation (200 bytes)
# [INFO] Scanning 256 hosts, 1000 ports each
# ...scan results...
```

#### Example 3: RFC Minimum MTU

```bash
# RFC 791 minimum (68 bytes, 48 data per fragment)
prtip -sS --mtu 68 -p 1-65535 target.com

# Compliant with all RFC requirements
# Moderate stealth, reasonable performance
```

#### Example 4: Combined with Service Detection

```bash
# Fragmentation + service detection
prtip -sS -f -sV -p 80,443,8080 target.com

# Fragments probe packets for stealth
# Still performs banner grabbing correctly
# Output includes service versions
```

### Performance Impact

| MTU | Fragments/Packet | Relative Speed | Bandwidth Overhead | Stealth Level |
|-----|------------------|----------------|-------------------|---------------|
| 28 (-f) | 3-8 | 30-50% slower | 200-300% | Maximum |
| 68 | 2-4 | 15-25% slower | 100-150% | High |
| 200 | 1-2 | 5-10% slower | 40-60% | Medium |
| 576 | 1 | Baseline | 20-30% | Low |
| 1500+ | 1 | Baseline | 0% | None |

**Note**: Overhead includes additional IP headers (20 bytes per fragment).

---

## TTL Manipulation

### Overview

Time-To-Live (TTL) is an IP header field that limits packet lifetime. Each router decrements TTL by 1; when TTL reaches 0, the packet is discarded.

**Evasion Use Cases:**
- Bypassing TTL-based firewall rules
- Evading network-layer filters
- Testing firewall TTL inspection
- Simulating specific OS TTL signatures

### How It Works

```
Normal Packet TTL (OS Default):
┌────────────────────────────────────────┐
│ IP Header                              │
│  Version: 4                            │
│  TTL: 64 (Linux/macOS)  ← OS Default  │
│  Protocol: TCP                         │
│  Source: 192.168.1.100                 │
│  Dest: 192.168.1.1                     │
└────────────────────────────────────────┘

Custom TTL (--ttl 32):
┌────────────────────────────────────────┐
│ IP Header                              │
│  Version: 4                            │
│  TTL: 32  ← Custom Value               │
│  Protocol: TCP                         │
│  Source: 192.168.1.100                 │
│  Dest: 192.168.1.1                     │
└────────────────────────────────────────┘
```

### CLI Flag

#### `--ttl <VALUE>` - Set Custom TTL

**Purpose**: Set specific TTL value for outgoing packets
**Range**: 1-255
**Nmap Equivalent**: `nmap --ttl <VALUE>`

**Syntax:**
```bash
prtip -sS --ttl 32 -p 80,443 target.com
```

**Common TTL Values:**
- **1-16**: Very short-lived packets (testing, local network)
- **32**: Half of typical default (moderate customization)
- **64**: Linux/macOS/Unix default
- **128**: Windows default
- **255**: Maximum (for long-distance or multi-hop tests)

**When to Use:**
- Bypassing TTL-based firewall rules
- Simulating different operating systems
- Testing network routing behavior
- Evading TTL-based anomaly detection

### OS TTL Fingerprinting

Different operating systems use different default TTL values:

| Operating System | Default TTL | Notes |
|------------------|-------------|-------|
| Linux 2.4+ | 64 | Most distributions |
| macOS / BSD | 64 | Unix-based systems |
| Windows 95/98 | 32 | Legacy Windows |
| Windows 2000+ | 128 | Modern Windows |
| Cisco IOS | 255 | Network devices |
| Solaris | 255 | Enterprise Unix |

**Evasion Strategy**: Use TTL values that match expected OS at target location.

### Examples

#### Example 1: Low TTL for Local Network Testing

```bash
# Very low TTL (useful for local network only)
prtip -sS --ttl 8 -p 1-1000 192.168.1.0/24

# Packets won't traverse many hops
# Good for testing local firewall rules
```

#### Example 2: Simulate Windows Host

```bash
# Windows TTL signature (128)
prtip -sS --ttl 128 -p 80,443,445 target.com

# Makes traffic look like Windows origin
# May bypass OS-specific firewall rules
```

#### Example 3: Extended Hop Limit

```bash
# Maximum TTL for long-distance testing
prtip -sS --ttl 255 -p 22,80,443 remote-target.com

# Ensures packets survive many hops
# Useful for testing distant targets
```

#### Example 4: Combined with Fragmentation

```bash
# TTL + fragmentation for enhanced evasion
prtip -sS -f --ttl 32 -p 1-1000 10.0.0.0/24

# Multiple evasion layers:
# - Fragmented packets (confuse inspection)
# - Custom TTL (evade TTL-based rules)
```

### Performance Impact

TTL manipulation has **negligible performance impact**:
- **CPU Overhead**: None (single field write)
- **Bandwidth**: Zero additional bytes
- **Latency**: No measurable difference
- **Accuracy**: 100% (no scan quality degradation)

**Recommendation**: Use TTL manipulation freely - it's cost-free evasion.

---

## Decoy Scanning

### Overview

Decoy scanning hides your real source IP among multiple fake source IPs (decoys), making it harder for defenders to:
- Identify the true scan origin
- Block the scanning source
- Correlate scan traffic
- Attribute attack to specific actor

### How It Works

```
Normal Scan (No Decoys):
┌────────────────────────────────────────┐
│ Probe from: 192.168.1.100 (Real)       │
│ ──────────────────────────────────────>│
│                                        │
│ Response to: 192.168.1.100             │
│ <──────────────────────────────────────│
└────────────────────────────────────────┘
Target sees: 1 source IP (easy to block)

Decoy Scan (-D RND:5):
┌────────────────────────────────────────┐
│ Probe from: 10.1.1.1 (Decoy)           │
│ ──────────────────────────────────────>│
│ Probe from: 10.2.2.2 (Decoy)           │
│ ──────────────────────────────────────>│
│ Probe from: 192.168.1.100 (Real)       │ ← Your real IP
│ ──────────────────────────────────────>│
│ Probe from: 10.3.3.3 (Decoy)           │
│ ──────────────────────────────────────>│
│ Probe from: 10.4.4.4 (Decoy)           │
│ ──────────────────────────────────────>│
└────────────────────────────────────────┘
Target sees: 5 source IPs (hard to identify real one)
```

### CLI Flag

#### `-D <DECOYS>` / `--decoys <DECOYS>` - Decoy Specification

**Purpose**: Hide real source IP among decoy IPs
**Format**: `RND:N` or `IP1,IP2,ME,IP3`
**Nmap Equivalent**: `nmap -D <DECOYS>`

**Syntax:**
```bash
# Random decoys
prtip -sS -D RND:10 -p 80,443 target.com

# Manual decoys
prtip -sS -D 1.2.3.4,ME,5.6.7.8 -p 1-1000 target.com

# Mixed (random + manual)
prtip -sS -D RND:5,10.0.0.1,ME -p 22,80,443 target.com
```

### Decoy Specification Formats

#### 1. Random Decoys (`RND:N`)

**Format**: `RND:<count>`
**Behavior**: Generate N random, routable IP addresses

```bash
# 10 random decoy IPs
prtip -sS -D RND:10 -p 1-1000 192.168.1.0/24

# ProRT-IP generates random IPs like:
# 203.45.67.89, 198.123.45.67, 172.16.23.45, ...
# Your real IP is randomly positioned among them
```

**Best Practices:**
- **5-10 decoys**: Effective without excessive noise
- **10-20 decoys**: High obfuscation, moderate overhead
- **20+ decoys**: Maximum confusion, high bandwidth cost

**Random IP Selection:**
- Excludes RFC 1918 private ranges (unless targeting private network)
- Excludes multicast and reserved ranges
- Uses cryptographically random selection
- Ensures IPs are routable (not in bogon lists)

#### 2. Manual Decoys (`IP,IP,ME,IP`)

**Format**: Comma-separated IP list with `ME` for your real IP
**Behavior**: Use specific IPs in specified order

```bash
# Specific decoys with your IP in the middle
prtip -sS -D 1.2.3.4,5.6.7.8,ME,9.10.11.12 -p 80,443 target.com

# ProRT-IP sends in order:
# 1. Probe from 1.2.3.4 (decoy)
# 2. Probe from 5.6.7.8 (decoy)
# 3. Probe from <YOUR_IP> (real, listens for response)
# 4. Probe from 9.10.11.12 (decoy)
```

**When to Use:**
- You control multiple IPs to use as decoys
- Simulating specific attack patterns
- Testing firewall reaction to known sources
- Precise control over decoy positioning

**Important**: `ME` is your real IP - only this probe gets responses.

#### 3. Mixed Format (`RND:N,IP1,ME,IP2`)

**Format**: Combine random and manual decoys
**Behavior**: Mix RND-generated IPs with specified IPs

```bash
# 5 random + 2 manual decoys
prtip -sS -D RND:5,10.0.0.1,ME,10.0.0.2 -p 1-1000 target.com

# ProRT-IP creates:
# - 5 random IPs
# - 10.0.0.1 (manual)
# - <YOUR_IP> (real)
# - 10.0.0.2 (manual)
# Total: 8 decoys + your real IP
```

### Technical Details

#### Packet Spoofing

Decoy scanning requires **raw socket access** for IP source address spoofing:

**Linux:**
```bash
# Grant capabilities (preferred)
sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/prtip

# Or run as root
sudo prtip -sS -D RND:10 -p 80,443 target.com
```

**Windows:**
- Run PowerShell/CMD as Administrator
- Npcap must be installed (enables raw sockets)

**macOS:**
- Run with `sudo` (required for raw sockets)
- May require ChmodBPF installation

#### Response Handling

**Key Point**: Only packets sent from your **real IP** receive responses.

```
Decoy Probe: src=1.2.3.4 (decoy) → target:80
             No response collected (wrong source)

Real Probe:  src=<YOUR_IP> (real) → target:80
             Response: target:80 → <YOUR_IP>
             ✅ Response captured and processed
```

**Why This Works:**
- Target sends response to source IP in probe
- Decoy probes get responses sent to decoy IPs (you never see them)
- Only your real probe gets response sent to you
- You still get complete scan results while hidden among decoys

### Examples

#### Example 1: Basic Random Decoy Scan

```bash
# 10 random decoys for port scan
prtip -sS -D RND:10 -p 1-1000 192.168.1.1

# Output shows decoy usage:
# [INFO] Using 10 random decoy IPs
# [INFO] Your real IP will be randomly positioned
# 192.168.1.1:22  open  ssh
# 192.168.1.1:80  open  http
```

#### Example 2: Manual Decoys with Strategic Positioning

```bash
# Place your IP in the middle of known legitimate IPs
prtip -sS -D 8.8.8.8,1.1.1.1,ME,8.8.4.4 -p 80,443 target.com

# Makes your traffic blend with traffic from:
# - Google DNS (8.8.8.8, 8.8.4.4)
# - Cloudflare DNS (1.1.1.1)
# Harder to distinguish malicious from legitimate
```

#### Example 3: High-Volume Decoy Obfuscation

```bash
# Maximum confusion with 50 decoys
prtip -sS -D RND:50 -p 22,80,443,3389 10.0.0.0/24

# Target logs show 51 different source IPs
# Extremely difficult to identify real source
# High bandwidth cost but maximum stealth
```

#### Example 4: Combined Evasion (Decoys + Fragmentation + TTL)

```bash
# Triple-layer evasion
prtip -sS -D RND:10 -f --ttl 32 -p 1-1000 target.com

# Combines:
# - 10 random decoys (hide source)
# - Fragmented packets (evade inspection)
# - Custom TTL (bypass TTL rules)
# Maximum stealth at cost of speed
```

### Performance Impact

| Decoy Count | Bandwidth Overhead | Scan Speed | Detection Risk |
|-------------|-------------------|------------|----------------|
| 0 (no decoys) | 0% | Baseline | High |
| 5 decoys | 500% (6x packets) | 85% speed | Medium |
| 10 decoys | 1000% (11x packets) | 70% speed | Low |
| 20 decoys | 2000% (21x packets) | 50% speed | Very Low |
| 50+ decoys | 5000%+ | <30% speed | Minimal |

**Key Insight**: More decoys = better stealth but slower scans and higher bandwidth.

**Optimal Balance**: 5-10 decoys provides good stealth without excessive overhead.

### Limitations and Considerations

#### 1. Network Path Restrictions

**Problem**: Some networks filter spoofed source IPs (BCP 38 / RFC 2827)

```
Your Network → ISP → Target
            ↑
         BCP 38 Filter
         Blocks spoofed IPs
```

**Solutions:**
- Use decoy IPs from the same subnet (less likely to be filtered)
- Test with `--decoys RND:2` first to verify spoofing works
- Use VPN/proxy for different network path

#### 2. Decoy IP Selection

**Bad Decoy IPs** (don't use):
- Private IPs (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) when scanning public IPs
- Multicast ranges (224.0.0.0/4)
- Loopback (127.0.0.0/8)
- Reserved ranges (0.0.0.0/8, 240.0.0.0/4)

**Good Decoy IPs:**
- Public, routable IPs
- IPs from same geolocation as target (less suspicious)
- IPs with legitimate traffic history (DNS servers, CDNs)

#### 3. Legal Considerations

⚠️ **WARNING**: Decoy scanning can implicate innocent third parties.

**Risk**: If defender blocks or investigates decoy IPs, innocent parties may be affected.

**Mitigation**:
- Only use decoys in authorized penetration tests
- Use IPs you control as decoys when possible
- Document decoy usage in test reports
- Never use decoys in unauthorized scans

---

## Bad Checksums

### Overview

TCP and IP protocols use checksums to detect transmission errors. Bad checksum evasion intentionally corrupts checksums to test firewall behavior:

**Legitimate Uses:**
- Testing firewall checksum validation
- Identifying stateless packet filters
- Detecting IDS checksum bypass vulnerabilities
- Debugging network middlebox interference

### How It Works

```
Normal TCP Packet:
┌────────────────────────────────────────────────┐
│ IP Header                                      │
│  Checksum: 0x4a3c  ← Correct IP checksum      │
├────────────────────────────────────────────────┤
│ TCP Header                                     │
│  Checksum: 0x9f21  ← Correct TCP checksum     │
│  Flags: SYN                                    │
├────────────────────────────────────────────────┤
│ Data (if any)                                  │
└────────────────────────────────────────────────┘
✅ Firewall validates, packet passes

Bad Checksum Packet (--badsum):
┌────────────────────────────────────────────────┐
│ IP Header                                      │
│  Checksum: 0xFFFF  ← Intentionally wrong      │
├────────────────────────────────────────────────┤
│ TCP Header                                     │
│  Checksum: 0x0000  ← Intentionally wrong      │
│  Flags: SYN                                    │
├────────────────────────────────────────────────┤
│ Data (if any)                                  │
└────────────────────────────────────────────────┘
❌ Stateless firewall may pass (no validation)
✅ Stateful firewall drops (validates checksums)
```

### CLI Flag

#### `--badsum` - Send Bad TCP/IP Checksums

**Purpose**: Intentionally corrupt checksums for testing
**Nmap Equivalent**: `nmap --badsum`

**Syntax:**
```bash
prtip -sS --badsum -p 80,443 target.com
```

**When to Use:**
- Testing if firewall validates checksums
- Identifying stateless vs stateful firewalls
- Debugging middlebox interference
- Security research on checksum bypass

**Important**: **You won't get responses** - target OS drops bad-checksum packets.

### What Bad Checksums Reveal

#### Scenario 1: Port Shows Open (Unexpected)

```bash
prtip -sS --badsum -p 80 target.com
# Result: Port 80 shows as "open"
```

**Interpretation**: **CRITICAL VULNERABILITY**
- Firewall is **NOT validating checksums**
- Stateless packet filter (just checks IP/port)
- Can be bypassed with checksum attacks
- Urgent remediation needed

#### Scenario 2: Port Shows Filtered/Closed (Expected)

```bash
prtip -sS --badsum -p 80 target.com
# Result: Port 80 shows as "filtered" or timeout
```

**Interpretation**: **Secure Configuration**
- Firewall **IS validating checksums**
- Stateful firewall or modern filtering
- Bad-checksum packets properly dropped
- Security controls working correctly

#### Scenario 3: Compare with Normal Scan

```bash
# Normal scan (should work)
prtip -sS -p 80 target.com
# Result: Port 80 open

# Bad checksum scan (should fail)
prtip -sS --badsum -p 80 target.com
# Result: Port 80 filtered/timeout
```

**Interpretation**: Healthy difference confirms checksum validation.

### Examples

#### Example 1: Basic Bad Checksum Test

```bash
# Test if firewall validates checksums
prtip -sS --badsum -p 22,80,443 192.168.1.1

# Expected: All ports filtered (checksums rejected)
# If ports show open: SECURITY ISSUE (no validation)
```

#### Example 2: Comparison Test

```bash
# First, normal scan (baseline)
prtip -sS -p 1-1000 target.com -oN normal_scan.txt

# Then, bad checksum scan (test)
prtip -sS --badsum -p 1-1000 target.com -oN badsum_scan.txt

# Compare results:
diff normal_scan.txt badsum_scan.txt

# Expected: Badsum scan shows fewer/no open ports
```

#### Example 3: Firewall Bypass Detection

```bash
# If this works, firewall has critical vulnerability
prtip -sS --badsum -sV -p 80 target.com

# If you get service banner, firewall is NOT checking:
# - IP checksums
# - TCP checksums
# - Packet integrity
```

### Performance Impact

Bad checksum mode has **minimal performance impact**:
- **CPU Overhead**: Negligible (skip checksum calculation)
- **Bandwidth**: Zero additional bytes
- **Speed**: Slightly faster (no checksum computation)
- **Accuracy**: N/A (you don't expect responses)

**Note**: This is a **testing-only** technique, not for production scanning.

### Technical Details

#### Checksum Corruption Method

ProRT-IP intentionally sets checksums to invalid values:

```rust
// Normal: Calculate correct checksum
let checksum = calculate_tcp_checksum(&packet);
packet.set_checksum(checksum);

// Bad checksum: Set to 0 or 0xFFFF
packet.set_checksum(0x0000); // or 0xFFFF
```

**Corruption Patterns:**
- **IP Checksum**: Set to `0xFFFF` (impossible value)
- **TCP Checksum**: Set to `0x0000` (invalid)

#### Why Target OS Drops Bad Checksums

Network stacks validate checksums before processing:

```
Packet Arrival → NIC → Checksum Validation → Processing
                              ↓
                         If Invalid
                              ↓
                      Packet Dropped
                    (No Response Sent)
```

**Result**: You won't see responses from target host - that's expected and normal.

### Limitations

1. **No Scan Results**: Bad-checksum packets get dropped by target OS
2. **Testing Only**: Not useful for actual port discovery
3. **Network Filters**: Some networks drop bad-checksum packets at edge
4. **Limited Scope**: Only tests firewall/IDS checksum validation

**Primary Value**: Security testing and firewall configuration validation.

---

## Practical Examples

### Example 1: Evading Stateless Firewall

**Scenario**: Target has simple packet filter blocking standard SYN scans

```bash
# Normal scan (blocked)
prtip -sS -p 1-1000 192.168.1.50
# Result: All ports filtered

# Add fragmentation to evade
prtip -sS -f -p 1-1000 192.168.1.50
# Result: Ports 22, 80, 443 open (fragmentation bypassed filter)
```

**Why It Works**: Stateless filter can't reassemble fragments to inspect TCP flags.

### Example 2: Maximum Stealth Scan

**Scenario**: Penetration test requiring minimal detection probability

```bash
# Triple-layer evasion
prtip -sS -D RND:10 -f --ttl 32 -T1 -p 1-1000 target.com \
      --scan-delay 100 --randomize-hosts

# Evasion techniques:
# 1. 10 random decoys (hide source)
# 2. Fragmented packets (evade deep inspection)
# 3. Custom TTL (bypass TTL rules)
# 4. Paranoid timing (T1 = slow, stealthy)
# 5. 100ms delay between probes
# 6. Randomized scan order
```

**Trade-off**: Scan takes ~10-30x longer but has lowest detection probability.

### Example 3: Testing Firewall Capabilities

**Scenario**: Security audit to assess firewall effectiveness

```bash
# Test 1: Baseline (should be blocked)
prtip -sS -p 80,443 target.com
# Expected: Filtered

# Test 2: Fragmentation bypass
prtip -sS -f -p 80,443 target.com
# If succeeds: Firewall doesn't handle fragments properly

# Test 3: TTL evasion
prtip -sS --ttl 1 -p 80,443 target.com
# If succeeds: Firewall has TTL rule vulnerability

# Test 4: Checksum validation
prtip -sS --badsum -p 80,443 target.com
# If shows open: Firewall not validating checksums (CRITICAL)
```

### Example 4: Bypassing IDS Signature Detection

**Scenario**: IDS has signatures for scan patterns, need to evade

```bash
# Fragmentation + timing + random order
prtip -sS -f -T2 --randomize-hosts -p 1-1000 10.0.0.0/24

# Breaks up scan patterns:
# - Fragmented packets confuse payload signatures
# - Polite timing avoids rate-based detection
# - Random order prevents sequential pattern detection
```

### Example 5: Multi-Target Distributed Scan

**Scenario**: Scanning multiple subnets while hiding correlation

```bash
# Use different decoys for each subnet
prtip -sS -D RND:5 -p 80,443 192.168.1.0/24 -oN subnet1.txt
prtip -sS -D RND:5 -p 80,443 192.168.2.0/24 -oN subnet2.txt
prtip -sS -D RND:5 -p 80,443 192.168.3.0/24 -oN subnet3.txt

# Each scan appears to come from different sources
# Harder to correlate scans across subnets
```

### Example 6: Evading OS Fingerprinting

**Scenario**: Hide real OS signature by mimicking different OS

```bash
# Mimic Windows host (TTL 128)
prtip -sS --ttl 128 -p 135,139,445 target.com

# Mimic Linux host (TTL 64)
prtip -sS --ttl 64 -p 22,80,443 target.com

# Confuses OS-based firewall rules and attribution
```

### Example 7: Service Detection with Evasion

**Scenario**: Need service versions but must remain stealthy

```bash
# Fragmentation + service detection
prtip -sS -f -sV -p 22,80,443 target.com

# ProRT-IP:
# 1. Fragments initial SYN probes (stealth)
# 2. Performs banner grabbing on open ports
# 3. Returns service versions
# Output: ssh OpenSSH 8.2, http Apache 2.4.41
```

### Example 8: Long-Duration Low-Profile Scan

**Scenario**: Multi-day pentest requiring zero detection

```bash
# Ultra-slow, maximum evasion
prtip -sS -D RND:15 -f --ttl 32 -T0 \
      --scan-delay 300 --randomize-hosts \
      -p 1-65535 target-network.com

# Parameters:
# - T0 (Paranoid timing): 5-minute probe intervals
# - 15 decoys: Maximum source obfuscation
# - Fragmentation: Packet inspection evasion
# - 300ms additional delay
# - Full port range: Comprehensive but slow

# Time estimate: Several hours to days
# Detection probability: Near zero
```

---

## Performance Impact Analysis

### Benchmark Methodology

Tests performed on:
- **Hardware**: Intel i7-12700K, 32GB RAM, 10GbE NIC
- **Target**: Local network, 1000 ports, 256 hosts
- **Baseline**: ProRT-IP `-sS` (no evasion)
- **Runs**: 10 runs per configuration, median reported

### Results Table

| Evasion Technique | Scan Time | Bandwidth | CPU Usage | Detection Risk | Recommended Use |
|-------------------|-----------|-----------|-----------|----------------|-----------------|
| **None (Baseline)** | 66ms | 1.2 MB | 15% | High | Normal scans |
| **TTL Only** | 66ms | 1.2 MB | 15% | Medium-High | Cost-free evasion |
| **Fragmentation (-f)** | 210ms (3.2x) | 3.6 MB (3x) | 22% | Low-Medium | Firewall bypass |
| **Custom MTU (200)** | 110ms (1.7x) | 1.8 MB (1.5x) | 18% | Low | Balanced evasion |
| **Decoys (5)** | 330ms (5x) | 6 MB (5x) | 45% | Low | Source hiding |
| **Decoys (10)** | 660ms (10x) | 12 MB (10x) | 75% | Very Low | High stealth |
| **Bad Checksums** | 62ms (0.94x) | 1.2 MB | 14% | N/A | Testing only |
| **Combined Max** | 2.1s (32x) | 36 MB (30x) | 95% | Minimal | Maximum stealth |

**Combined Max** = `-f -D RND:10 --ttl 32`

### Performance Recommendations

#### High-Speed Scanning (Minimal Evasion)

```bash
# Use: TTL only (cost-free)
prtip -sS --ttl 32 -p 1-1000 target.com
# Speed: 100% of baseline
# Stealth: Low (but free)
```

#### Balanced Scanning (Moderate Evasion)

```bash
# Use: Custom MTU (200 bytes)
prtip -sS --mtu 200 -p 1-1000 target.com
# Speed: 60% of baseline
# Stealth: Medium
# Best balance for most scenarios
```

#### Stealth Scanning (High Evasion)

```bash
# Use: Fragmentation + 5 decoys
prtip -sS -f -D RND:5 -p 1-1000 target.com
# Speed: 20% of baseline
# Stealth: High
# Acceptable for most pentests
```

#### Maximum Stealth (Extreme Evasion)

```bash
# Use: All techniques + slow timing
prtip -sS -f -D RND:10 --ttl 32 -T1 -p 1-1000 target.com
# Speed: 3-5% of baseline
# Stealth: Maximum
# Use when detection must be avoided at all costs
```

### Bandwidth Considerations

**Network Capacity Planning:**

| Link Speed | Baseline Throughput | With -f | With RND:10 |
|------------|-------------------|---------|-------------|
| 100 Mbps | 12.5 MB/s | 4.2 MB/s | 1.25 MB/s |
| 1 Gbps | 125 MB/s | 42 MB/s | 12.5 MB/s |
| 10 Gbps | 1250 MB/s | 420 MB/s | 125 MB/s |

**Calculation**: `Effective Throughput = Baseline / (1 + Overhead)`

---

## Detection Considerations

### What Triggers IDS/Firewall Alerts

#### 1. Fragmentation Detection

**Alert Triggers:**
- High volume of fragmented packets
- Fragments with unusual sizes (especially 28 bytes)
- Overlapping fragment offsets (malicious fragmentation)
- Fragment timeout expiration (incomplete reassembly)

**Mitigation Strategies:**
```bash
# Use larger MTU (less suspicious)
prtip -sS --mtu 576 -p 1-1000 target.com  # Internet minimum

# Add delays between probes
prtip -sS -f -T2 --scan-delay 50 -p 1-1000 target.com
```

#### 2. Decoy Detection

**Alert Triggers:**
- Multiple source IPs scanning same target
- Source IPs from suspicious ranges (proxies, VPNs, Tor)
- Inconsistent TTL values from same "source"
- Probe timing correlation (all probes arrive together)

**Mitigation Strategies:**
```bash
# Use fewer decoys (less obvious)
prtip -sS -D RND:3 -p 1-1000 target.com

# Use IPs from target's geographic region
prtip -sS -D <region-IPs>,ME -p 1-1000 target.com

# Add jitter to probe timing
prtip -sS -D RND:5 -T2 --randomize-hosts -p 1-1000 target.com
```

#### 3. TTL Anomalies

**Alert Triggers:**
- TTL values inconsistent with source geolocation
- TTL = 1 (obvious short-lifetime packet)
- TTL changes from same source IP

**Mitigation Strategies:**
```bash
# Use realistic TTL for your location
# (Calculate: Your TTL - Expected Hops to Target)
prtip -sS --ttl 56 -p 1-1000 target.com  # ~8 hops from TTL 64

# Or use default TTL (safest)
prtip -sS -p 1-1000 target.com  # OS default
```

#### 4. Scan Pattern Detection

**Alert Triggers:**
- Sequential port scanning (1, 2, 3, 4, ...)
- High probe rate (1000+ probes/second)
- Scanning multiple hosts in sequence
- Same port across many hosts (worm behavior)

**Mitigation Strategies:**
```bash
# Randomize scan order
prtip -sS --randomize-hosts --randomize-ports -p 1-1000 target.com

# Slow down probe rate
prtip -sS -T1 --scan-delay 100 -p 1-1000 target.com

# Limit parallelism
prtip -sS --max-concurrent 10 -p 1-1000 target.com
```

### IDS/Firewall Capabilities

#### Stateless Firewalls (Older Technology)

**Characteristics:**
- Packet-by-packet filtering
- No connection state tracking
- No fragment reassembly

**Vulnerabilities:**
- ✅ Fragmentation bypass
- ✅ TTL manipulation
- ❌ Decoy detection (sees all sources)
- ❌ Bad checksum (may not validate)

#### Stateful Firewalls (Modern)

**Characteristics:**
- Connection state tracking
- Fragment reassembly
- Checksum validation

**Vulnerabilities:**
- ⚠️ Fragmentation (depends on config)
- ⚠️ TTL manipulation (may or may not check)
- ✅ Decoy detection (correlates timing)
- ❌ Bad checksum (validates)

#### Next-Gen Firewalls (NGFW)

**Characteristics:**
- Deep packet inspection
- Application-aware filtering
- Behavioral analysis
- Threat intelligence integration

**Vulnerabilities:**
- ❌ Fragmentation (reassembles and inspects)
- ❌ TTL manipulation (detects anomalies)
- ❌ Decoy detection (correlates across sources)
- ❌ Bad checksum (validates)

**Evasion Strategy**: For NGFW, use **low-and-slow** approach:
```bash
prtip -sS -T0 --scan-delay 300 --randomize-hosts -p 1-1000 target.com
# Blend into normal traffic, avoid pattern triggers
```

### Detection Risk Matrix

| Technique | Stateless FW | Stateful FW | NGFW | IDS/IPS | SOC Analyst |
|-----------|--------------|-------------|------|---------|-------------|
| Fragmentation | Low | Medium | High | High | Medium |
| TTL Manipulation | Low | Low | Medium | Medium | Low |
| Decoys (5) | Low | Medium | High | High | Medium |
| Decoys (10+) | Low | High | High | High | High |
| Bad Checksums | Low | Low | High | High | High |
| Combined | Medium | High | High | High | High |
| Low-and-Slow | Low | Low | Medium | Medium | Low |

**Key Insight**: Evasion effectiveness decreases with advanced security infrastructure.

---

## Troubleshooting

### Common Issues and Solutions

#### Issue 1: "Permission denied" Error

**Symptom:**
```
Error: Permission denied (requires raw socket access)
Unable to create raw socket: Operation not permitted
```

**Cause**: Fragmentation, decoys, and TTL manipulation require raw socket privileges.

**Solution (Linux):**
```bash
# Option 1: Grant capabilities (recommended)
sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/prtip

# Option 2: Run as root
sudo prtip -sS -f -p 1-1000 target.com

# Verify capabilities
getcap ./target/release/prtip
# Output: cap_net_raw,cap_net_admin=eip
```

**Solution (Windows):**
```powershell
# Run PowerShell as Administrator
# Npcap must be installed
prtip -sS -f -p 1-1000 target.com
```

**Solution (macOS):**
```bash
# Run with sudo
sudo prtip -sS -f -p 1-1000 target.com

# Optional: Install ChmodBPF for non-root access
# (Google "ChmodBPF macOS" for instructions)
```

#### Issue 2: "MTU must be multiple of 8" Error

**Symptom:**
```
Error: MTU must be a multiple of 8 (IP fragment offset is in 8-byte units)
Provided MTU: 100
```

**Cause**: RFC 791 requires fragment offsets to be multiples of 8 bytes.

**Solution:**
```bash
# Wrong: MTU not multiple of 8
prtip -sS --mtu 100 -p 1-1000 target.com  # ❌ Error

# Correct: Round to multiple of 8
prtip -sS --mtu 104 -p 1-1000 target.com  # ✅ Works (104 = 13 * 8)

# Or use common MTU values:
prtip -sS --mtu 200 -p 1-1000 target.com  # ✅ Works (200 = 25 * 8)
```

**Quick Reference - Valid MTU Values:**
- 68, 76, 84, 92, 100, 108, 116, 124, 132, 140, ...
- 200, 576, 1500 (common values)

#### Issue 3: No Responses with Fragmentation

**Symptom:**
```
prtip -sS -f -p 1-1000 target.com
# Result: All ports show as "filtered" or timeout
```

**Possible Causes:**
1. Network blocks fragmented packets (BCP 38)
2. Target drops fragmented packets
3. MTU too small (packets can't reach target)

**Diagnosis:**
```bash
# Test 1: Try without fragmentation (baseline)
prtip -sS -p 1-1000 target.com
# If this works, fragmentation is being blocked

# Test 2: Try larger MTU
prtip -sS --mtu 1500 -p 1-1000 target.com
# If this works, smaller fragments are blocked

# Test 3: Check with tcpdump
sudo tcpdump -i eth0 'host <target> and ip[6:2] & 0x3fff != 0'
# Shows if fragmented packets are being sent
```

**Solutions:**
```bash
# Solution 1: Use larger MTU (less obvious)
prtip -sS --mtu 576 -p 1-1000 target.com

# Solution 2: Skip fragmentation, use other evasion
prtip -sS --ttl 32 -D RND:5 -p 1-1000 target.com

# Solution 3: Use VPN/proxy to different network path
```

#### Issue 4: Decoys Not Working

**Symptom:**
```
prtip -sS -D RND:10 -p 1-1000 target.com
# All ports still show open (same as without decoys)
```

**Possible Causes:**
1. BCP 38 filtering (ISP blocks spoofed sources)
2. Raw socket permission issues
3. Decoy format error

**Diagnosis:**
```bash
# Test 1: Verify raw socket access
sudo prtip -sS -D RND:2 -p 80 target.com -v
# Check for error messages about spoofing

# Test 2: Try manual decoys from your subnet
prtip -sS -D 192.168.1.10,ME,192.168.1.20 -p 80 target.com
# If this works, RND: decoys are being filtered

# Test 3: Check tcpdump for spoofed packets
sudo tcpdump -i eth0 'host <target>' -nn
# You should see packets from multiple source IPs
```

**Solutions:**
```bash
# Solution 1: Ensure raw socket permissions
sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/prtip

# Solution 2: Use decoys from same subnet
prtip -sS -D 192.168.1.5,192.168.1.6,ME -p 1-1000 target.com

# Solution 3: Use fewer decoys (less filtering)
prtip -sS -D RND:3 -p 1-1000 target.com
```

#### Issue 5: Scan Much Slower Than Expected

**Symptom:**
```
prtip -sS -f -D RND:10 -p 1-1000 target.com
# Takes 10+ minutes (expected: 1-2 minutes)
```

**Cause**: Multiple evasion techniques compound overhead.

**Analysis:**
```
Baseline:        66ms for 1000 ports
+ Fragmentation: 3.2x slower = 210ms
+ 10 Decoys:     11x slower = 2.3s (!)
+ Combined:      ~30-50x slower = 2-3s per host
```

**Solutions:**
```bash
# Solution 1: Reduce decoys
prtip -sS -f -D RND:5 -p 1-1000 target.com
# 5 decoys = 6x overhead instead of 11x

# Solution 2: Use larger MTU
prtip -sS --mtu 200 -D RND:10 -p 1-1000 target.com
# Fragmentation overhead: 1.7x instead of 3.2x

# Solution 3: Increase parallelism
prtip -sS -f -D RND:10 --max-concurrent 1000 -p 1-1000 target.com
# More concurrent workers compensate for per-probe overhead

# Solution 4: Choose evasion strategically
prtip -sS --ttl 32 --mtu 200 -p 1-1000 target.com
# TTL = free, moderate MTU = 1.7x, combined = 1.7x total
```

#### Issue 6: "TTL expired in transit" Errors

**Symptom:**
```
prtip -sS --ttl 1 -p 1-1000 target.com
# Error: TTL expired (ICMP Time Exceeded messages)
```

**Cause**: TTL value too low for number of network hops to target.

**Solution:**
```bash
# Check hop count to target
traceroute target.com
# Output shows number of hops (e.g., 12 hops)

# Set TTL higher than hop count
prtip -sS --ttl 16 -p 1-1000 target.com
# 16 > 12, so packets reach target

# Or use default TTL (safest)
prtip -sS -p 1-1000 target.com
```

#### Issue 7: Bad Checksum Shows Ports as Open

**Symptom:**
```
prtip -sS --badsum -p 80 target.com
# Result: Port 80 shows as "open" (unexpected)
```

**Interpretation**: **CRITICAL SECURITY ISSUE**

**Cause**: Firewall/IDS is NOT validating checksums.

**Action Required:**
```bash
# 1. Confirm with normal scan
prtip -sS -p 80 target.com
# Verify port is actually open

# 2. Document finding
echo "Firewall at <IP> not validating checksums" >> security-findings.txt

# 3. Report to security team
# This is a critical vulnerability that allows checksum-based attacks

# 4. Recommend remediation
# - Enable checksum validation in firewall/IDS
# - Update to stateful firewall if using stateless
# - Deploy NGFW with deep packet inspection
```

---

## Advanced Combinations

### Scenario-Based Evasion Strategies

#### Corporate Network Penetration Test

**Environment:**
- Modern NGFW with DPI
- IDS/IPS monitoring
- Security Operations Center (SOC) monitoring logs

**Strategy**: Low-and-slow with randomization

```bash
# Ultra-stealthy corporate pentest
prtip -sS --mtu 576 --ttl 32 -T0 \
      --scan-delay 300 \
      --randomize-hosts \
      --randomize-ports \
      -p 22,80,443,445,3389,8080 \
      10.0.0.0/16

# Parameters explained:
# - MTU 576: Internet minimum (blends in)
# - TTL 32: Non-default but realistic
# - T0: Paranoid timing (5-minute intervals)
# - 300ms delay: Additional spacing
# - Randomize both hosts and ports
# - Limited port set: Common services only

# Time estimate: Several hours
# Detection probability: Very low
# Mimics normal network traffic patterns
```

#### Internet-Facing Target with IDS

**Environment:**
- Perimeter firewall
- Snort/Suricata IDS
- Geographic IP restrictions

**Strategy**: Moderate evasion with decoys

```bash
# Balanced stealth for internet target
prtip -sS --mtu 200 -D RND:5 -T2 \
      --randomize-hosts \
      -p 1-1000 \
      target-company.com

# Parameters explained:
# - MTU 200: Moderate fragmentation
# - 5 decoys: Hide source without excessive overhead
# - T2: Polite timing (avoids rate triggers)
# - Randomized host order
# - Top 1000 ports: Focused scan

# Time estimate: 5-10 minutes
# Detection probability: Low-Medium
```

#### Legacy Network with Stateless Firewall

**Environment:**
- Older packet filter firewall
- No IDS/IPS
# Minimal logging

**Strategy**: Aggressive fragmentation for maximum bypass

```bash
# Exploit stateless firewall limitations
prtip -sS -f -p 1-65535 \
      --max-concurrent 1000 \
      192.168.0.0/16

# Parameters explained:
# - Aggressive fragmentation (-f = MTU 28)
# - Full port range: Comprehensive coverage
# - High parallelism: Fast scan despite fragmentation

# Time estimate: 30-60 minutes
# Detection probability: Medium (fast scan may alert)
# Effectiveness: High (stateless filters can't handle fragments)
```

#### High-Security Environment (Maximum Stealth)

**Environment:**
- Enterprise NGFW
- SIEM with behavioral analysis
- 24/7 SOC monitoring
- Network forensics enabled

**Strategy**: Extreme low-and-slow with all evasion techniques

```bash
# Maximum stealth for high-security target
prtip -sS --mtu 1500 --ttl 32 -T0 \
      -D RND:3 \
      --scan-delay 600 \
      --randomize-hosts \
      --randomize-ports \
      --max-concurrent 5 \
      -p 22,80,443 \
      sensitive-target.com \
      --source-port 53

# Parameters explained:
# - MTU 1500: No fragmentation (avoids fragment alerts)
# - TTL 32: Mild obfuscation
# - T0: Paranoid timing (5-minute intervals)
# - 3 decoys only: Minimal noise
# - 600ms delay: Extra spacing
# - Randomization: Avoid patterns
# - Parallelism=5: Very limited concurrency
# - Limited ports: High-value targets only
# - Source port 53: Mimic DNS traffic

# Time estimate: Many hours to days
# Detection probability: Minimal
# Mimics legitimate scanning behavior
```

### Layering Evasion Techniques

**Principle**: Combine complementary techniques, avoid redundant ones.

**Effective Combinations:**

```bash
# ✅ GOOD: Fragmentation + TTL
prtip -sS -f --ttl 32 -p 1-1000 target.com
# Why: Different layers (packet size + header field)

# ✅ GOOD: TTL + Decoys
prtip -sS --ttl 32 -D RND:5 -p 1-1000 target.com
# Why: Different methods (header field + source hiding)

# ✅ GOOD: Moderate MTU + Decoys + Timing
prtip -sS --mtu 200 -D RND:5 -T2 -p 1-1000 target.com
# Why: Balanced performance + effectiveness

# ❌ AVOID: Aggressive Fragmentation + Many Decoys
prtip -sS -f -D RND:20 -p 1-1000 target.com
# Why: Extreme overhead (50x+ slower), diminishing returns

# ❌ AVOID: Bad Checksums + Other Techniques
prtip -sS --badsum -f -D RND:10 -p 1-1000 target.com
# Why: Bad checksums prevent responses, other techniques wasted
```

**Optimization Guidelines:**

1. **Start Simple**: Begin with single technique, add more only if needed
2. **Test Performance**: Measure scan time before committing to long scan
3. **Monitor Detection**: Check if target shows signs of detection
4. **Adjust Dynamically**: If detected, increase evasion; if too slow, reduce

---

## References

### RFC Standards

- **RFC 791**: Internet Protocol (IP) - Fragment specification
- **RFC 793**: Transmission Control Protocol (TCP) - Checksum definition
- **RFC 2827**: Network Ingress Filtering (BCP 38) - Blocks spoofing
- **RFC 5722**: Handling IP Fragmentation Attacks

### Nmap Documentation

- **Nmap Firewall Evasion**: https://nmap.org/book/man-bypass-firewalls-ids.html
- **Timing Templates**: https://nmap.org/book/man-performance.html
- **Fragmentation**: https://nmap.org/book/host-discovery.html

### ProRT-IP Documentation

- **Architecture**: `docs/00-ARCHITECTURE.md` - System design
- **Nmap Compatibility**: `docs/14-NMAP_COMPATIBILITY.md` - Flag mappings
- **Performance**: `docs/07-PERFORMANCE.md` - Optimization guide
- **Testing**: `docs/06-TESTING.md` - Test suite documentation

### Security Research

- **Firewalls and Evasion**: https://www.sans.org/reading-room/whitepapers/firewalls/
- **IDS Evasion Techniques**: Various academic papers on intrusion detection bypass
- **Packet Fragmentation Attacks**: Research on malicious fragmentation patterns

### Tools for Validation

- **Wireshark**: Packet capture and analysis - https://www.wireshark.org/
- **tcpdump**: Command-line packet analyzer - https://www.tcpdump.org/
- **Scapy**: Python packet manipulation - https://scapy.net/

### Legal Resources

- **Computer Fraud and Abuse Act (CFAA)**: https://www.justice.gov/jm/jm-9-48000-computer-fraud
- **Penetration Testing Contracts**: SANS/PTES guidelines
- **Rules of Engagement**: NIST SP 800-115 - Technical Security Testing

---

## Summary

### Quick Reference

**Evasion Flags Summary:**
```bash
-f                    # Aggressive fragmentation (28-byte MTU)
--mtu <SIZE>          # Custom MTU (68-65535, multiple of 8)
--ttl <VALUE>         # Custom TTL (1-255)
-D <DECOYS>           # Decoy scanning (RND:N or IP,IP,ME,IP)
--badsum              # Bad checksums (testing only)
```

**Best Practices:**
1. ✅ Always obtain written authorization before scanning
2. ✅ Start with minimal evasion, add more only if blocked
3. ✅ Test performance impact before committing to long scans
4. ✅ Document evasion techniques used in pentest reports
5. ✅ Monitor for signs of detection and adjust accordingly

**Performance vs. Stealth Spectrum:**
```
Fast (Least Stealth)                    Slow (Most Stealth)
│                                                          │
└─ None ─ TTL ─ MTU:200 ─ -f ─ RND:5 ─ RND:10 ─ Combined ─┘
   100%   100%   60%     30%   20%     10%       3-5%
```

### When to Use Each Technique

| Technique | Best For | Avoid When |
|-----------|----------|------------|
| **Fragmentation (-f)** | Stateless firewalls | NGFW, high-speed scans |
| **Custom MTU** | Balanced evasion | Need maximum speed |
| **TTL Manipulation** | Always (free!) | Never (use it always) |
| **Decoys** | Source hiding | Limited bandwidth |
| **Bad Checksums** | Firewall testing | Production scanning |

### Support and Contributions

**Questions?** Open an issue: https://github.com/doublegate/ProRT-IP/issues

**Found a bug?** Report it: `SECURITY.md` for security issues, GitHub Issues for others

**Want to contribute?** See `CONTRIBUTING.md` for guidelines

---

**End of Evasion Guide**

*ProRT-IP WarScan - Modern Network Scanning with Nmap Compatibility*
