# OS Fingerprints

Technical reference for ProRT-IP's operating system fingerprinting signature database.

## Overview

**OS Fingerprints** are distinctive patterns in TCP/IP stack behavior used to identify operating systems. ProRT-IP uses a comprehensive database of **2,600+ OS signatures** covering major operating systems, embedded devices, and network equipment.

**Key Features:**
- **2,600+ signatures** - Linux, Windows, BSD, macOS, IoT devices, network equipment
- **16-probe sequence** - Inspired by Nmap's OS detection methodology
- **95% accuracy** - High confidence on well-known operating systems
- **Passive and active modes** - Banner analysis and active probing
- **CPE identifiers** - Common Platform Enumeration for CVE matching

**Use Cases:**
- **Security Auditing**: Identify unpatched or end-of-life systems
- **Network Inventory**: Catalog OS versions across infrastructure
- **Vulnerability Assessment**: Match OS versions to CVE databases
- **Compliance**: Verify approved OS versions in production environments

---

## Fingerprint Database

### Database Statistics

ProRT-IP's OS fingerprint database includes signatures for:

| Category | Count | Examples |
|----------|-------|----------|
| **Linux Distributions** | 800+ | Ubuntu, Debian, RHEL, CentOS, Fedora, Arch, Gentoo |
| **Windows** | 450+ | XP, Vista, 7, 8, 8.1, 10, 11, Server 2008-2022 |
| **BSD Variants** | 300+ | FreeBSD, OpenBSD, NetBSD, DragonFly BSD |
| **macOS** | 250+ | 10.x (Yosemite-Monterey), 11.x (Big Sur), 12.x+ |
| **Embedded Systems** | 400+ | OpenWrt, DD-WRT, Tomato, pfSense, OPNsense |
| **IoT Devices** | 200+ | Raspberry Pi OS, Armbian, Yocto-based |
| **Network Equipment** | 200+ | Cisco IOS, Juniper Junos, MikroTik RouterOS |
| **Virtualization** | 100+ | VMware ESXi, Citrix XenServer, Proxmox |

**Total:** 2,600+ unique signatures

### Signature Coverage

**Operating System Families:**
- **Linux**: Kernels 2.4 through 6.x (2002-2024)
- **Windows**: XP through 11, Server 2003-2022
- **BSD**: FreeBSD 8.x-14.x, OpenBSD 5.x-7.x, NetBSD 7.x-10.x
- **macOS**: OS X 10.6 (Snow Leopard) through macOS 14 (Sonoma)
- **Embedded**: Custom Linux kernels, RTOS, stripped TCP/IP stacks

**Update Frequency:**
- Database updated quarterly with new OS releases
- Community contributions for rare/specialized systems
- Nmap database synchronization for compatibility

---

## Fingerprint Format

### Signature Structure

OS fingerprints consist of multiple test results from the 16-probe sequence:

```
Fingerprint Linux 5.15 - 6.1 (Ubuntu 22.04)
Class Linux | Linux | 5.15 | general purpose
CPE cpe:/o:canonical:ubuntu_linux:22.04
SEQ(SP=0-5%GCD=1%ISR=80000-90000%TI=I%II=I%SS=S%TS=U)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000|8000%W2=8000|8000%W3=8000|8000%W4=8000|8000%W5=8000|8000%W6=8000|8000)
ECN(R=Y%DF=Y%T=40%TG=40%W=8018%O=MWTS%CC=Y%Q=)
T1(R=Y%DF=Y%T=40%TG=40%S=O%A=S+%F=AS%RD=0%Q=)
T2(R=N)
T3(R=N)
T4(R=Y%DF=Y%T=40%TG=40%W=0%S=A%A=Z%F=R%Q=)
T5(R=Y%DF=Y%T=40%TG=40%W=0%S=Z%A=S+%F=AR%Q=)
T6(R=Y%DF=Y%T=40%TG=40%W=0%S=A%A=Z%F=R%Q=)
T7(R=Y%DF=Y%T=40%TG=40%W=0%S=Z%A=S%F=AR%Q=)
IE(R=Y%DFI=N%T=40%TG=40%CD=S)
U1(R=Y%DF=N%T=40%TG=40%IPL=164%UN=0%RIPL=G%RID=G%RIPCK=G%RUCK=G%RUD=G)
```

### Directive Reference

#### Fingerprint Line

**Format:** `Fingerprint <OS Name> [<Version Range>] [<Additional Info>]`

**Components:**
- **OS Name**: Operating system family (e.g., "Linux", "Windows", "FreeBSD")
- **Version Range**: Kernel/OS version range (e.g., "5.15 - 6.1", "10 Build 19041-19044")
- **Additional Info**: Distribution, edition, or device type (e.g., "Ubuntu 22.04", "Enterprise")

**Examples:**
```
Fingerprint Linux 5.15 - 6.1 (Ubuntu 22.04)
Fingerprint Microsoft Windows 10 Build 19041-19044
Fingerprint FreeBSD 13.0 - 14.0
Fingerprint Cisco IOS 15.2-15.9
```

#### Class Line

**Format:** `Class <Vendor> | <OS Family> | <Version> | <Device Type>`

**Components:**
- **Vendor**: OS vendor or manufacturer (e.g., "Linux", "Microsoft", "Cisco")
- **OS Family**: Operating system family (e.g., "Linux", "Windows", "IOS")
- **Version**: Version number or range (e.g., "5.15", "10", "15.x")
- **Device Type**: general purpose, router, switch, firewall, specialized, printer, webcam, etc.

**Examples:**
```
Class Linux | Linux | 5.15 | general purpose
Class Microsoft | Windows | 10 | general purpose
Class Cisco | IOS | 15 | router
Class embedded | Linux | 3.10 | webcam
```

#### CPE Line

**Format:** `CPE cpe:/o:<vendor>:<product>:<version>[:<update>:<edition>:<language>]`

**Common Platform Enumeration (CPE) for CVE matching:**
- **cpe:/o:** = Operating system CPE URI
- **vendor**: Vendor name (lowercase, hyphenated)
- **product**: Product name (lowercase, hyphenated)
- **version**: Version number (dotted notation)

**Examples:**
```
CPE cpe:/o:canonical:ubuntu_linux:22.04
CPE cpe:/o:microsoft:windows_10:1909
CPE cpe:/o:freebsd:freebsd:13.0
CPE cpe:/o:cisco:ios:15.2
```

---

## 16-Probe Sequence

### Probe Overview

ProRT-IP sends 16 carefully crafted probes to elicit distinctive responses from the target's TCP/IP stack:

| Probe | Type | Target | Flags | Purpose | Timing |
|-------|------|--------|-------|---------|--------|
| **1** | TCP | Open port | SYN | Initial sequence number (ISN), TCP options, window size | T+0ms |
| **2** | TCP | Open port | SYN | ISN delta (GCD calculation) | T+100ms |
| **3** | TCP | Open port | SYN | ISN delta | T+200ms |
| **4** | TCP | Open port | SYN | ISN delta | T+300ms |
| **5** | TCP | Open port | SYN | ISN delta | T+400ms |
| **6** | TCP | Open port | SYN | ISN delta | T+500ms |
| **7** | ICMP | Any | Echo (TOS=0, code=0) | ICMP response behavior | T+600ms |
| **8** | ICMP | Any | Echo (TOS=4, code=9) | ICMP error handling | T+650ms |
| **9** | TCP | Open port | ECN, SYN, CWR, ECE | Explicit Congestion Notification support | T+700ms |
| **10** | TCP | Closed port | NULL (no flags) | Response to unusual TCP packet | T+750ms |
| **11** | TCP | Closed port | SYN+FIN+URG+PSH | Response to invalid flag combinations | T+800ms |
| **12** | TCP | Closed port | ACK | Window size in RST response | T+850ms |
| **13** | TCP | Closed port | ACK (window=128) | Firewall detection | T+900ms |
| **14** | TCP | Closed port | ACK (window=256) | Firewall detection | T+950ms |
| **15** | TCP | Open port | SYN (varied options) | TCP option handling | T+1000ms |
| **16** | UDP | Closed port | Empty packet | ICMP Port Unreachable response | T+1050ms |

**Timing Strategy:**
- **Probes 1-6**: Spaced 100ms apart for ISN sequence analysis
- **Probes 7-16**: Sent in rapid succession (50ms intervals)
- **Total duration**: ~1.1 seconds typical (600-1200ms range)

### Probe Requirements

**Minimum Requirements for Successful Fingerprinting:**
- ✅ **At least 1 open TCP port** (for probes 1-6, 9, 15)
- ✅ **At least 1 closed TCP port** (for probes 10-14)
- ✅ **ICMP Echo permitted** (for probes 7-8, optional but recommended)
- ✅ **UDP accessible** (for probe 16, optional but recommended)

**Failure Modes:**
- ❌ **All ports filtered**: Cannot fingerprint (no responses)
- ❌ **Only open ports**: Reduced accuracy (no closed port tests)
- ❌ **Only closed ports**: Reduced accuracy (no open port tests)
- ⚠️ **ICMP blocked**: Still works with TCP-only tests (lower confidence)

---

## Fingerprint Attributes

### SEQ (Sequence Generation)

**Tests:** Probes 1-6 (six SYN packets spaced 100ms apart)

**Attributes:**

#### SP (Sequence Predictability)

**Format:** `SP=<0-255>`

**Meaning:** Measures how predictable the ISN (Initial Sequence Number) sequence is.

**Values:**
- **0-3**: Highly random (modern secure OS)
- **4-50**: Some predictability (acceptable)
- **51-100**: Moderate predictability (older OS)
- **101-255**: High predictability (very old OS, security risk)

**Examples:**
- Linux 5.x: `SP=0-3` (random)
- Windows 10: `SP=0-5` (random)
- Legacy systems: `SP=100-255` (predictable, vulnerable to hijacking)

#### GCD (Greatest Common Divisor)

**Format:** `GCD=<number>`

**Meaning:** GCD of ISN deltas between the 6 probes. Reveals ISN generation algorithm.

**Common Values:**
- **GCD=1**: Random ISN (most modern OS)
- **GCD=64000**: Fixed increment (BSD, some embedded systems)
- **GCD=800000**: Large prime divisor (older Linux kernels)

**Examples:**
- Linux 2.4-3.x: `GCD=64000` (fixed increment)
- Linux 4.x+: `GCD=1` (random)
- Windows XP-11: `GCD=1` (random)
- FreeBSD: `GCD=64000` (64,000 increments per second)

#### ISR (ISN Counter Rate)

**Format:** `ISR=<increments per second>`

**Meaning:** Average rate of ISN increments per second (if using counter-based ISN).

**Common Values:**
- **ISR=0**: Random ISN (no counter)
- **ISR=64000**: 64,000 increments/sec (typical BSD)
- **ISR=80000-90000**: ~800,000 increments/sec (older Linux)

**Examples:**
- Linux 2.4-3.x: `ISR=80000-90000` (~800,000/sec)
- Windows XP-11: `ISR=0` (random, no counter)
- FreeBSD: `ISR=64000` (64,000/sec)

#### TI (TCP IP ID)

**Format:** `TI=Z|I|RI|BI|RD|<hex>`

**Meaning:** IP ID generation behavior for TCP packets.

**Values:**
- **Z**: All zeros (e.g., OpenBSD)
- **I**: Incremental (sequential, e.g., Linux)
- **RI**: Random Incremental (random but incrementing, e.g., Windows)
- **BI**: Broken Incremental (inconsistent pattern)
- **RD**: Random (completely random)

**Examples:**
- Linux: `TI=I` (sequential IP IDs)
- Windows 10: `TI=RI` (random incremental)
- OpenBSD: `TI=Z` (all zeros for privacy)

#### II (ICMP IP ID)

**Format:** `II=Z|I|RI|<hex>`

**Meaning:** IP ID generation behavior for ICMP packets.

**Values:** Same as TI (Z, I, RI, BI, RD)

**Examples:**
- Linux: `II=I` (incremental)
- Windows: `II=I` (incremental)
- BSD: `II=RI` (random incremental)

#### SS (Shared Sequence)

**Format:** `SS=S|O`

**Meaning:** Whether TCP and ICMP share IP ID sequence.

**Values:**
- **S**: Shared (same counter for TCP and ICMP)
- **O**: Own (separate counters)

**Examples:**
- Linux: `SS=S` (shared)
- Windows: `SS=O` (separate)

#### TS (Timestamp Option)

**Format:** `TS=U|0|1|7|8`

**Meaning:** TCP timestamp option behavior.

**Values:**
- **U**: Unsupported (no timestamp option)
- **0**: Timestamp present, value 0
- **1**: Timestamp present, reasonable value
- **7**: Timestamp present, value ~100Hz
- **8**: Timestamp present, value ~1000Hz

**Examples:**
- Linux: `TS=U` or `TS=1` (depends on sysctl)
- Windows: `TS=1` (supported)

### OPS (TCP Options)

**Tests:** Probes 1-6, 9, 15 (TCP packets to open ports)

**Format:** `OPS(O1=<options>%O2=<options>%...%O6=<options>)`

**TCP Option Ordering:**
- **M**: MSS (Maximum Segment Size)
- **W**: Window Scale
- **T**: Timestamp
- **S**: SACK (Selective Acknowledgment)
- **E**: EOL (End of Option List)
- **N**: NOP (No Operation)

**Common Patterns:**
- **Linux**: `MWTS` (MSS, Window, Timestamp, SACK)
- **Windows**: `MWST` (different order)
- **BSD**: `MWT` (no SACK support)
- **Embedded**: `M` (MSS only, limited options)

**Examples:**
```
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)  # Linux
OPS(O1=MWST%O2=MWST%O3=MWST%O4=MWST%O5=MWST%O6=MWST)  # Windows
OPS(O1=MWT%O2=MWT%O3=MWT%O4=MWT%O5=MWT%O6=MWT)        # BSD
```

### WIN (Window Sizes)

**Tests:** Probes 1-6, 9, 12-15 (various TCP packets)

**Format:** `WIN(W1=<size>%W2=<size>%...%W6=<size>)`

**Window Size Notation:**
- Values in hexadecimal (e.g., `8000` = 32,768 bytes)
- Pipe separator `|` indicates multiple possible values
- Format: `W1=8000|4000` means 32,768 or 16,384 bytes

**Common Window Sizes:**
- **Linux**: `W=8000` (32,768 bytes default)
- **Windows**: `W=2000` or `W=4000` (8,192 or 16,384 bytes)
- **BSD**: `W=4000` (16,384 bytes)
- **Embedded**: `W=200` or smaller (512 bytes, limited resources)

**Examples:**
```
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)  # Linux
WIN(W1=2000%W2=2000%W3=2000%W4=2000%W5=2000%W6=2000)  # Windows
WIN(W1=200%W2=200%W3=200%W4=200%W5=200%W6=200)        # Embedded
```

### ECN (Explicit Congestion Notification)

**Test:** Probe 9 (TCP packet with ECN flags set)

**Format:** `ECN(R=Y|N%DF=Y|N%T=<TTL>%W=<window>%O=<options>%CC=Y|N%Q=)`

**Attributes:**
- **R**: Response received (Y/N)
- **DF**: Don't Fragment flag set (Y/N)
- **T**: Time to Live value (hex)
- **W**: Window size (hex)
- **O**: TCP options (same notation as OPS)
- **CC**: Congestion Control supported (Y/N)
- **Q**: Reserved for future use

**Examples:**
```
ECN(R=Y%DF=Y%T=40%W=8018%O=MWTS%CC=Y%Q=)  # Linux with ECN support
ECN(R=N)                                    # No ECN support
```

### T1-T7 (TCP Tests)

**Tests:** Probes 1-6, 9-15 (various TCP packets)

**Format:** `T<n>(R=Y|N%DF=Y|N%T=<TTL>%S=O|Z%A=S|S+|Z%F=<flags>%RD=<retrans>%Q=)`

**Attributes:**
- **R**: Response received (Y/N)
- **DF**: Don't Fragment flag set (Y/N)
- **T**: Time to Live value (hex)
- **S**: Sequence number (O=original, Z=zero)
- **A**: Acknowledgment number (S=incremented, S+=incremented by >1, Z=zero)
- **F**: TCP flags (A=ACK, S=SYN, R=RST, F=FIN, P=PSH, U=URG)
- **RD**: Retransmission delay
- **Q**: Reserved

**Test Purposes:**
- **T1**: Response to SYN on open port (SYN-ACK expected)
- **T2-T6**: Various unusual packets (NULL, SYN+FIN, etc.)
- **T7**: Response to ICMP Echo

**Examples:**
```
T1(R=Y%DF=Y%T=40%S=O%A=S+%F=AS%RD=0%Q=)   # Normal SYN-ACK
T4(R=Y%DF=Y%T=40%W=0%S=A%A=Z%F=R%Q=)      # RST to closed port
```

### IE (ICMP Echo)

**Tests:** Probes 7-8 (ICMP Echo requests)

**Format:** `IE(R=Y|N%DFI=Y|N%T=<TTL>%CD=S|Z|O)`

**Attributes:**
- **R**: Response received (Y/N)
- **DFI**: Don't Fragment flag in response (Y/N)
- **T**: Time to Live in response (hex)
- **CD**: Code in ICMP response (S=same, Z=zero, O=other)

**Examples:**
```
IE(R=Y%DFI=N%T=40%CD=S)  # Normal ICMP Echo Reply
IE(R=N)                   # ICMP blocked
```

### U1 (UDP Test)

**Test:** Probe 16 (UDP packet to closed port)

**Format:** `U1(R=Y|N%DF=Y|N%T=<TTL>%IPL=<length>%UN=<code>%RIPL=G|Z%RID=G|Z%RIPCK=G|Z%RUCK=G|Z%RUD=G|Z)`

**Attributes:**
- **R**: ICMP Port Unreachable received (Y/N)
- **DF**: Don't Fragment flag in ICMP response (Y/N)
- **T**: Time to Live in ICMP response (hex)
- **IPL**: IP total length in ICMP response (decimal)
- **UN**: ICMP unreachable code (0-15)
- **RIPL**: Returned IP length (G=good, Z=zero)
- **RID**: Returned IP ID (G=good, Z=zero)
- **RIPCK**: Returned IP checksum (G=good, Z=zero, I=incorrect)
- **RUCK**: Returned UDP checksum (G=good, Z=zero)
- **RUD**: Returned UDP data (G=good, Z=zero)

**Examples:**
```
U1(R=Y%DF=N%T=40%IPL=164%UN=0%RIPL=G%RID=G%RIPCK=G%RUCK=G%RUD=G)  # Linux
U1(R=N)                                                              # UDP filtered
```

---

## Detection Algorithm

### Fingerprint Matching Process

**Step-by-step algorithm:**

**1. Probe Sequence Execution:**
- Send probes 1-6 (SYN packets, 100ms apart)
- Send probes 7-16 (ICMP, unusual TCP, UDP)
- Total duration: ~1.1 seconds

**2. Response Collection:**
- Capture all responses (SYN-ACK, RST, ICMP Echo Reply, ICMP Unreachable)
- Record timestamps for ISN analysis
- Extract TCP options, window sizes, flags

**3. Attribute Extraction:**
- Calculate ISN deltas (differences between probes 1-6)
- Compute GCD of ISN deltas
- Calculate ISR (ISN increments per second)
- Compute SP (sequence predictability score)
- Extract TCP option ordering
- Record IP ID generation patterns
- Measure window sizes

**4. Signature Comparison:**
- Compare extracted attributes against 2,600+ signatures
- Calculate similarity score for each signature (0-100%)
- Weight different attributes:
  - High weight: SEQ attributes (GCD, ISR, SP) - 30%
  - High weight: TCP options ordering - 25%
  - Medium weight: Window sizes - 20%
  - Medium weight: ECN behavior - 15%
  - Low weight: IP ID patterns - 10%

**5. Similarity Scoring:**

For each signature, compute:
```
score = Σ (attribute_match * attribute_weight) / total_weight
```

**Attribute matching:**
- **Exact match**: 100% (e.g., GCD=1 matches GCD=1)
- **Range match**: 80-100% (e.g., SP=0-5 matches SP=3)
- **Partial match**: 50-80% (e.g., similar window size)
- **No match**: 0% (e.g., completely different TCP options)

**6. Rank and Select:**
- Sort signatures by similarity score (highest first)
- Select top match if score ≥ 85% (high confidence)
- Return top 2-3 matches if highest score 70-84% (medium confidence)
- Return "insufficient data" if highest score < 70% (low confidence)

**7. Confidence Calculation:**

```
confidence = (top_score - second_best_score) + (top_score * 0.5)
```

**Ranges:**
- **95-100%**: Very high confidence (single clear match)
- **85-94%**: High confidence (likely accurate)
- **70-84%**: Medium confidence (manual verification recommended)
- **50-69%**: Low confidence (multiple possibilities)
- **<50%**: Very low confidence (insufficient data)

### Matching Strategy

**Tiered Matching Approach:**

**Tier 1: Exact Match (Score 95-100%)**
- All major attributes match exactly
- GCD, ISR, TCP options identical
- Window sizes within 5% tolerance
- Result: Single OS with high confidence

**Tier 2: Close Match (Score 85-94%)**
- Major attributes match, minor differences
- GCD matches, ISR within 10% tolerance
- TCP options match, window size varies
- Result: OS family with version range

**Tier 3: Partial Match (Score 70-84%)**
- Some attributes match, significant differences
- GCD pattern similar, ISR different
- TCP options partially match
- Result: Multiple OS possibilities

**Tier 4: Weak Match (Score 50-69%)**
- Few attributes match
- Basic TCP/IP stack behavior only
- Result: OS class (e.g., "Linux-like", "BSD-like")

**Tier 5: No Match (Score <50%)**
- Insufficient responses or unknown OS
- Result: "Unknown OS" or "Custom TCP/IP stack"

---

## Common Signatures

### Linux Examples

#### Ubuntu 22.04 (Linux 5.15-6.1)

```
Fingerprint Linux 5.15 - 6.1 (Ubuntu 22.04)
Class Linux | Linux | 5.15 | general purpose
CPE cpe:/o:canonical:ubuntu_linux:22.04
SEQ(SP=0-3%GCD=1%ISR=80000-90000%TI=I%II=I%SS=S%TS=U)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)
ECN(R=Y%DF=Y%T=40%W=8018%O=MWTS%CC=Y%Q=)
```

**Key Characteristics:**
- Random ISN (GCD=1, SP=0-3)
- ISR ~800,000/sec (older kernel legacy)
- Incremental IP ID (TI=I, II=I)
- TCP options: MWTS (MSS, Window, Timestamp, SACK)
- 32,768 byte window (W=8000)
- ECN support (CC=Y)

#### RHEL 8 / CentOS 8 (Linux 4.18)

```
Fingerprint Linux 4.18 (RHEL 8 / CentOS 8)
Class Linux | Linux | 4.18 | general purpose
CPE cpe:/o:redhat:enterprise_linux:8
SEQ(SP=0-5%GCD=1%ISR=80000-90000%TI=I%II=I%SS=S%TS=1)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)
```

**Differences from Ubuntu:**
- Timestamp enabled by default (TS=1 vs TS=U)
- Slightly wider SP range (0-5 vs 0-3)

### Windows Examples

#### Windows 10 Build 19041-19044 (21H1-22H2)

```
Fingerprint Microsoft Windows 10 Build 19041-19044
Class Microsoft | Windows | 10 | general purpose
CPE cpe:/o:microsoft:windows_10:1909
SEQ(SP=0-5%GCD=1%ISR=0%TI=RI%II=I%SS=O%TS=1)
OPS(O1=MWST%O2=MWST%O3=MWST%O4=MWST%O5=MWST%O6=MWST)
WIN(W1=2000%W2=2000%W3=2000%W4=2000%W5=2000%W6=2000)
ECN(R=Y%DF=Y%T=80%W=2000%O=MWST%CC=Y%Q=)
```

**Key Characteristics:**
- Random ISN (GCD=1, ISR=0 - no counter)
- Random Incremental IP ID for TCP (TI=RI)
- Separate TCP/ICMP counters (SS=O)
- TCP options: MWST (different order than Linux)
- 8,192 byte window (W=2000)
- Higher TTL (T=80 vs Linux T=40)

#### Windows Server 2019

```
Fingerprint Microsoft Windows Server 2019 (Build 17763)
Class Microsoft | Windows | 2019 | general purpose
CPE cpe:/o:microsoft:windows_server_2019
SEQ(SP=0-5%GCD=1%ISR=0%TI=RI%II=I%SS=O%TS=1)
OPS(O1=MWST%O2=MWST%O3=MWST%O4=MWST%O5=MWST%O6=MWST)
WIN(W1=2000%W2=2000%W3=2000%W4=2000%W5=2000%W6=2000)
```

**Similar to Windows 10:**
- Server editions share most TCP/IP stack characteristics
- Differentiated by service detection (SMB, RDP versions)

### BSD Examples

#### FreeBSD 13.0-14.0

```
Fingerprint FreeBSD 13.0 - 14.0
Class FreeBSD | FreeBSD | 13.0 | general purpose
CPE cpe:/o:freebsd:freebsd:13.0
SEQ(SP=0-5%GCD=64000%ISR=64000%TI=I%II=RI%SS=O%TS=1)
OPS(O1=MWT%O2=MWT%O3=MWT%O4=MWT%O5=MWT%O6=MWT)
WIN(W1=4000%W2=4000%W3=4000%W4=4000%W5=4000%W6=4000)
ECN(R=Y%DF=Y%T=40%W=4000%O=MWT%CC=Y%Q=)
```

**Key Characteristics:**
- Fixed ISN increment (GCD=64000, ISR=64000)
- Incremental TCP IP ID (TI=I)
- Random Incremental ICMP IP ID (II=RI)
- Separate counters (SS=O)
- No SACK support (TCP options: MWT only)
- 16,384 byte window (W=4000)

#### OpenBSD 7.0+

```
Fingerprint OpenBSD 7.0 - 7.3
Class OpenBSD | OpenBSD | 7.0 | general purpose
CPE cpe:/o:openbsd:openbsd:7.0
SEQ(SP=0-5%GCD=1%ISR=0%TI=Z%II=Z%SS=S%TS=U)
OPS(O1=MW%O2=MW%O3=MW%O4=MW%O5=MW%O6=MW)
WIN(W1=4000%W2=4000%W3=4000%W4=4000%W5=4000%W6=4000)
```

**Key Characteristics:**
- Random ISN (GCD=1, ISR=0)
- **Zero IP ID** (TI=Z, II=Z - privacy feature)
- Minimal TCP options (MW only - no timestamp, no SACK)
- 16,384 byte window

### macOS Examples

#### macOS 12 (Monterey) / macOS 13 (Ventura)

```
Fingerprint macOS 12.x - 13.x (Monterey / Ventura)
Class Apple | macOS | 12 | general purpose
CPE cpe:/o:apple:macos:12
SEQ(SP=0-5%GCD=1%ISR=0%TI=I%II=I%SS=S%TS=1)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)
ECN(R=Y%DF=Y%T=40%W=8000%O=MWTS%CC=Y%Q=)
```

**Key Characteristics:**
- Random ISN (modern security)
- Incremental IP ID (shared for TCP/ICMP)
- Linux-like TCP options (MWTS)
- Large window (32,768 bytes)
- ECN support

### Embedded / IoT Examples

#### Raspberry Pi OS (Debian-based)

```
Fingerprint Linux 5.10 - 6.1 (Raspberry Pi OS)
Class Linux | Linux | 5.10 | embedded
CPE cpe:/o:raspbian:raspbian:11
SEQ(SP=0-5%GCD=1%ISR=80000-90000%TI=I%II=I%SS=S%TS=U)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)
```

**Similar to desktop Linux:**
- Full TCP/IP stack (not stripped)
- Standard kernel configuration

#### OpenWrt / DD-WRT (Router Firmware)

```
Fingerprint Embedded Linux 4.x-5.x (OpenWrt / DD-WRT)
Class embedded | Linux | 4.x | router
SEQ(SP=5-20%GCD=64000%ISR=64000%TI=I%II=I%SS=S%TS=U)
OPS(O1=MW%O2=MW%O3=MW%O4=MW%O5=MW%O6=MW)
WIN(W1=400%W2=400%W3=400%W4=400%W5=400%W6=400)
```

**Key Characteristics:**
- Higher SP (less randomness, resource constraints)
- Fixed ISN increment (GCD=64000)
- Limited TCP options (MW only - no timestamp, no SACK)
- **Small window** (1,024 bytes - W=400)

#### IoT Camera (Custom Embedded Linux)

```
Fingerprint Embedded Linux 3.x (Custom IoT)
Class embedded | Linux | 3.x | webcam
SEQ(SP=50-100%GCD=64000%ISR=64000%TI=I%II=I%SS=S%TS=U)
OPS(O1=M%O2=M%O3=M%O4=M%O5=M%O6=M)
WIN(W1=200%W2=200%W3=200%W4=200%W5=200%W6=200)
```

**Key Characteristics:**
- **High SP** (predictable ISN - security concern)
- Fixed increment (old kernel)
- **Minimal TCP options** (MSS only)
- **Very small window** (512 bytes - W=200)

---

## Custom Fingerprints

### Adding New OS Signatures

**Step-by-step guide to creating custom fingerprints:**

#### Step 1: Capture Fingerprint Data

Run OS detection with verbose output:

```bash
prtip -sS -O -v --osscan-guess TARGET > fingerprint-capture.txt
```

**Look for "Unmatched fingerprint" section:**
```
OS detection results:
No exact OS matches for host. Closest match: Linux 5.x

Unmatched fingerprint:
SEQ(SP=0-5%GCD=1%ISR=80000-90000%TI=I%II=I%SS=S%TS=U)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)
ECN(R=Y%DF=Y%T=40%W=8018%O=MWTS%CC=Y%Q=)
T1(R=Y%DF=Y%T=40%S=O%A=S+%F=AS%RD=0%Q=)
...
```

#### Step 2: Verify OS Identity

**Manually verify the target OS:**
```bash
# SSH into target (if accessible)
ssh user@TARGET
uname -a  # Linux kernel version
cat /etc/os-release  # Distribution details

# Or use banner grabbing
prtip -sV -p 22,80 TARGET  # Service banners may reveal OS
```

#### Step 3: Create Signature Entry

**Create a new fingerprint entry in `nmap-os-db` format:**

```
Fingerprint Linux 6.5 (Custom Distribution X)
Class Linux | Linux | 6.5 | general purpose
CPE cpe:/o:vendor:product:6.5
SEQ(SP=0-5%GCD=1%ISR=80000-90000%TI=I%II=I%SS=S%TS=U)
OPS(O1=MWTS%O2=MWTS%O3=MWTS%O4=MWTS%O5=MWTS%O6=MWTS)
WIN(W1=8000%W2=8000%W3=8000%W4=8000%W5=8000%W6=8000)
ECN(R=Y%DF=Y%T=40%W=8018%O=MWTS%CC=Y%Q=)
T1(R=Y%DF=Y%T=40%S=O%A=S+%F=AS%RD=0%Q=)
T2(R=N)
T3(R=N)
T4(R=Y%DF=Y%T=40%W=0%S=A%A=Z%F=R%Q=)
T5(R=Y%DF=Y%T=40%W=0%S=Z%A=S+%F=AR%Q=)
T6(R=Y%DF=Y%T=40%W=0%S=A%A=Z%F=R%Q=)
T7(R=Y%DF=Y%T=40%W=0%S=Z%A=S%F=AR%Q=)
IE(R=Y%DFI=N%T=40%CD=S)
U1(R=Y%DF=N%T=40%IPL=164%UN=0%RIPL=G%RID=G%RIPCK=G%RUCK=G%RUD=G)
```

#### Step 4: Add to Database

**Option A: Local Database**

Add to `~/.prtip/os-fingerprints.db`:

```bash
echo "Fingerprint Linux 6.5 (Custom Distribution X)
Class Linux | Linux | 6.5 | general purpose
CPE cpe:/o:vendor:product:6.5
SEQ(...)
OPS(...)
..." >> ~/.prtip/os-fingerprints.db
```

**Option B: Embedded Database**

Submit to ProRT-IP repository:
1. Fork: https://github.com/doublegate/ProRT-IP
2. Edit: `crates/prtip-core/data/nmap-os-db`
3. Add fingerprint at end of file
4. Test with: `prtip -sS -O TARGET` (should now match)
5. Submit pull request

#### Step 5: Test Fingerprint

**Verify the new signature works:**

```bash
# Test against known target
prtip -sS -O -p 80,443 TARGET

# Should now show:
# OS: Linux 6.5 (Custom Distribution X)
# Confidence: 95%
```

**If confidence < 85%:**
- Capture multiple fingerprints from same OS
- Look for patterns (ranges in SP, ISR)
- Generalize signature to match variants

#### Step 6: Submit to Community

**Contributing to ProRT-IP fingerprint database:**

1. **Test on multiple targets:**
   - Verify signature works on 3+ instances of same OS
   - Test different versions (e.g., 6.5.0, 6.5.1, 6.5.2)

2. **Document OS details:**
   - Exact OS name and version
   - Kernel version: `uname -r`
   - Distribution: `cat /etc/os-release`
   - TCP/IP stack tuning: `sysctl net.ipv4`

3. **Create pull request:**
   - Title: "Add fingerprint: Linux 6.5 (Custom Distribution X)"
   - Description: Testing details, OS sources, sample targets
   - Include test results from 3+ targets

4. **Upstream synchronization:**
   - ProRT-IP syncs with Nmap database quarterly
   - Contribute to Nmap: https://nmap.org/submit/

---

## Examples

### Example 1: Linux Server Detection

**Command:**
```bash
prtip -sS -O -p 22,80,443 192.168.1.10
```

**Output:**
```
PORT     STATE  SERVICE
22/tcp   open   ssh
80/tcp   open   http
443/tcp  open   https

OS Detection Results:
OS: Linux 5.15 - 6.1 (Ubuntu 22.04)
Confidence: 95%
CPE: cpe:/o:canonical:ubuntu_linux:22.04

Fingerprint Details:
- GCD: 1 (random ISN)
- ISR: ~800,000/sec (typical Linux)
- TCP Options: MWTS (Linux signature)
- Window: 32,768 bytes (8000 hex)
- IP ID: Incremental (Linux default)
- ECN: Supported
```

**Analysis:**
- **High confidence** (95%) indicates reliable detection
- **Kernel range 5.15-6.1** matches Ubuntu 22.04 LTS lifecycle
- **CPE identifier** can be used for CVE database queries
- **TCP options MWTS** is Linux-specific ordering

**Action Items:**
1. Verify kernel version: `ssh user@192.168.1.10 'uname -r'`
2. Check for security patches: `apt list --upgradable`
3. Cross-validate with service banners (SSH, HTTP)

### Example 2: Windows Desktop Detection

**Command:**
```bash
prtip -sS -O -p 445,3389 10.0.0.50
```

**Output:**
```
PORT      STATE  SERVICE
445/tcp   open   microsoft-ds
3389/tcp  open   ms-wbt-server

OS Detection Results:
OS: Microsoft Windows 10 Build 19041-19044 (21H1-22H2)
Confidence: 92%
CPE: cpe:/o:microsoft:windows_10

Fingerprint Details:
- GCD: 1 (random ISN)
- ISR: 0 (random, not counter-based)
- TCP Options: MWST (Windows ordering)
- Window: 8,192 bytes (2000 hex)
- IP ID: Random Incremental (Windows pattern)
- TTL: 128 (Windows default)
```

**Analysis:**
- **Build range 19041-19044** narrows down update version
- **TCP option order "MWST"** is Windows-specific
- **Random ISN** (ISR=0) typical of modern Windows
- **Higher TTL** (128 vs Linux 64) is Windows characteristic

**Action Items:**
1. Check Windows Update status
2. Verify SMB version (445/tcp) for security patches
3. Ensure RDP (3389/tcp) uses NLA (Network Level Authentication)

### Example 3: Embedded Device (Router)

**Command:**
```bash
prtip -sS -O -p 80,443 192.168.1.1
```

**Output:**
```
PORT     STATE  SERVICE
80/tcp   open   http
443/tcp  open   https

OS Detection Results:
OS: Embedded Linux 4.x-5.x (OpenWrt / DD-WRT)
Confidence: 78%

Fingerprint Details:
- GCD: 64000 (fixed increment)
- ISR: 64,000/sec (BSD-like)
- TCP Options: MW (limited support)
- Window: 1,024 bytes (400 hex - small)
- SP: 5-20 (moderate predictability)
```

**Analysis:**
- **Medium confidence** (78%) suggests custom TCP/IP stack
- **Small window size** (1,024 bytes) indicates resource-constrained device
- **Limited TCP options** (MW only) typical of embedded systems
- **Fixed ISN increment** (GCD=64000) is older implementation

**Action Items:**
1. Manual verification: Check web interface for firmware version
2. Update firmware if available (security patches)
3. Review router security settings (disable WPS, enable WPA3)

### Example 4: Firewall Blocking Detection

**Command:**
```bash
prtip -sS -O -p 1-1000 firewall.example.com
```

**Output:**
```
All 1000 scanned ports on firewall.example.com are filtered

OS Detection Results:
OS: Detection failed - insufficient responses
Confidence: 0%

Reason: No open ports found, or firewall blocking OS detection probes
Recommendation: Try different ports or timing templates
```

**Analysis:**
- **All ports filtered** - stateful firewall or IPS blocking probes
- Requires **at least one open port** for fingerprinting
- **No ICMP responses** - ICMP Echo may be blocked

**Troubleshooting:**
```bash
# Try known open ports
prtip -sS -O -p 80,443,22,3389 firewall.example.com

# Use aggressive timing (more retries)
prtip -sS -O -T4 --max-retries 3 -p 80,443 firewall.example.com

# Combine with service detection for additional hints
prtip -sS -sV -O -p 80,443 firewall.example.com
```

---

## See Also

### Feature Guides

- **[OS Fingerprinting](../features/os-fingerprinting.md)** - Complete guide to OS detection
  - How OS fingerprinting works (16-probe sequence)
  - Usage examples and best practices
  - Accuracy and confidence levels
  - Troubleshooting detection failures

- **[Service Detection](../features/service-detection.md)** - Service version detection
  - Complements OS detection with application-level information
  - Cross-validation: service banners + OS fingerprints
  - Combined detection workflows

### User Guides

- **[Basic Usage](../user-guide/basic-usage.md#os-fingerprinting)** - OS detection examples
  - Command syntax: `prtip -O`
  - Interpreting results
  - Common use cases

- **[Advanced Usage](../user-guide/advanced-usage.md)** - Advanced OS detection techniques
  - Combining OS detection with service scanning
  - Handling firewalls and filtered ports
  - Batch OS inventory across networks

### Reference

- **[Command Reference](./command-reference.md)** - All OS detection flags
  - `-O` / `--osscan-guess` - Enable OS detection
  - `--osscan-limit` - Limit to promising targets
  - `-v` / `--verbose` - Show unmatched fingerprints

- **[Error Codes](./error-codes.md)** - OS detection error meanings
  - Insufficient responses
  - No open/closed ports
  - Fingerprint database errors

### External Resources

- **Nmap OS Detection**: Original methodology and database
  - https://nmap.org/book/osdetect.html
  - nmap-os-db format specification
  - Submitting new fingerprints

- **CPE Dictionary**: Common Platform Enumeration
  - https://nvd.nist.gov/products/cpe
  - CPE naming specification
  - OS identifier format

- **CVE Database**: Vulnerability matching
  - https://cve.mitre.org/
  - Match CPE identifiers to known vulnerabilities
  - Security advisories by OS version

- **TCP/IP Illustrated**: Deep dive into TCP/IP stack implementations
  - Volume 1: The Protocols (Stevens)
  - Volume 2: The Implementation (Wright, Stevens)
  - OS-specific TCP/IP behavior differences

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
