# Stealth Scanning

Bypass firewalls and IDS using advanced packet manipulation techniques.

## What is Stealth Scanning?

**Stealth Scanning** (also called **Firewall/IDS Evasion**) uses packet manipulation techniques to bypass security controls that monitor or block network traffic. These techniques are essential for:

- **Penetration Testing**: Assessing security defenses by simulating attacker behaviors
- **Red Team Operations**: Testing blue team detection capabilities
- **Security Research**: Understanding how malicious actors evade detection
- **Network Troubleshooting**: Diagnosing firewall/IDS misconfigurations

**ProRT-IP Implementation:**
- **5 evasion techniques**: Fragmentation, MTU control, TTL manipulation, decoy scanning, bad checksums
- **Nmap-compatible**: All flags match nmap syntax
- **Combinable**: Techniques can be layered for maximum stealth
- **Performance-tuned**: Optimized implementations minimize overhead

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
```

#### Example 3: Combined with Service Detection

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
│  TTL: 64 (Linux/macOS)  ← OS Default   │
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

#### Example 1: Simulate Windows Host

```bash
# Windows TTL signature (128)
prtip -sS --ttl 128 -p 80,443,445 target.com

# Makes traffic look like Windows origin
# May bypass OS-specific firewall rules
```

#### Example 2: Combined with Fragmentation

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

#### 2. Manual Decoys (`IP,IP,ME,IP`)

**Format**: Comma-separated IP list with `ME` for your real IP

```bash
# Specific decoys with your IP in the middle
prtip -sS -D 1.2.3.4,5.6.7.8,ME,9.10.11.12 -p 80,443 target.com

# ProRT-IP sends in order:
# 1. Probe from 1.2.3.4 (decoy)
# 2. Probe from 5.6.7.8 (decoy)
# 3. Probe from <YOUR_IP> (real, listens for response)
# 4. Probe from 9.10.11.12 (decoy)
```

**Important**: `ME` is your real IP - only this probe gets responses.

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

#### Example 3: Combined Evasion (Decoys + Fragmentation + TTL)

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
│  Checksum: 0x4a3c  ← Correct IP checksum       │
├────────────────────────────────────────────────┤
│ TCP Header                                     │
│  Checksum: 0x9f21  ← Correct TCP checksum      │
│  Flags: SYN                                    │
└────────────────────────────────────────────────┘
✅ Firewall validates, packet passes

Bad Checksum Packet (--badsum):
┌────────────────────────────────────────────────┐
│ IP Header                                      │
│  Checksum: 0xFFFF  ← Intentionally wrong       │
├────────────────────────────────────────────────┤
│ TCP Header                                     │
│  Checksum: 0x0000  ← Intentionally wrong       │
│  Flags: SYN                                    │
└────────────────────────────────────────────────┘
❌ Stateless firewall may pass (no validation)
✅ Stateful firewall drops (validates checksums)
```

### CLI Flag

#### `--badsum` - Send Bad TCP/IP Checksums

**Purpose**: Intentionally corrupt checksums for testing

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

---

## Performance Impact Analysis

### Benchmark Results

Tests performed on:
- **Hardware**: Intel i7-12700K, 32GB RAM, 10GbE NIC
- **Target**: Local network, 1000 ports, 256 hosts
- **Baseline**: ProRT-IP `-sS` (no evasion)

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

---

## Troubleshooting

### Issue 1: "Permission denied" Error

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
```

### Issue 2: "MTU must be multiple of 8" Error

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

### Issue 3: No Responses with Fragmentation

**Symptom:**
```
prtip -sS -f -p 1-1000 target.com
# Result: All ports show as "filtered" or timeout
```

**Possible Causes:**
1. Network blocks fragmented packets (BCP 38)
2. Target drops fragmented packets
3. MTU too small (packets can't reach target)

**Solutions:**
```bash
# Solution 1: Use larger MTU (less obvious)
prtip -sS --mtu 576 -p 1-1000 target.com

# Solution 2: Skip fragmentation, use other evasion
prtip -sS --ttl 32 -D RND:5 -p 1-1000 target.com
```

---

## See Also

- **[User Guide: Stealth Scanning](../user-guide/scan-types.md)** - Basic stealth scan usage
- **[Nmap Compatibility](./nmap-compatibility.md)** - Evasion flag mappings to nmap
- **[Performance Tuning](../21-PERFORMANCE-GUIDE.md)** - Optimizing scan speed with evasion
- **[Architecture: Evasion Implementation](../00-ARCHITECTURE.md)** - How evasion techniques are implemented
- **[Troubleshooting Guide](../../TROUBLESHOOTING.md)** - Common evasion issues and fixes

**External Resources:**
- **RFC 791**: Internet Protocol (IP) - Fragment specification
- **RFC 793**: Transmission Control Protocol (TCP) - Checksum definition
- **Nmap Firewall Evasion**: https://nmap.org/book/man-bypass-firewalls-ids.html

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
