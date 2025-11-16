# ProRT-IP vs ZMap

Comprehensive technical comparison between ProRT-IP and ZMap, the academic Internet measurement tool that transformed network research by scanning the entire IPv4 address space in under 45 minutes.

---

## Executive Summary

**ZMap revolutionized Internet measurement** through stateless scanning architecture achieving 1.44 million pps at gigabit speeds (97-98% theoretical maximum) and 14.23 million pps at 10 gigabit speeds. Developed at the University of Michigan in 2013, ZMap completes full IPv4 scans in 42-45 minutes (gigabit) or 4 minutes 29 seconds (10 gigabit), representing a **1,300-fold speedup over Nmap** for Internet-wide surveys.

**ProRT-IP balances speed with comprehensive detection**, achieving comparable stateless performance (10M+ pps, similar to ZMap gigabit) while maintaining 85-90% service detection accuracy through modern Rust async I/O architecture. ProRT-IP's stateful mode (50K+ pps) adds service version detection (500+ services), OS fingerprinting (2,600+ signatures), and TLS certificate analysis unavailable in ZMap's core.

**The fundamental tradeoff**: ZMap optimizes exclusively for horizontal scanning (many hosts, single port) through single-probe methodology and zero per-connection state, making it the gold standard for Internet-wide research but requiring separate tools (ZGrab2, LZR) for application-layer detection. ProRT-IP achieves comparable stateless speed (10M+ pps) while integrating comprehensive detection in a single tool, though ZMap reaches higher maximum speeds (14.23 Mpps) with specialized 10 gigabit hardware.

---

## Quick Comparison

| Dimension | ZMap | ProRT-IP |
|-----------|------|----------|
| **First Released** | 2013 (University of Michigan) | 2024 (new project) |
| **Language** | C (kernel bypass optimizations) | Rust (memory-safe) |
| **Speed (Gigabit)** | 1.44 Mpps (97-98% theoretical max) | 10M+ pps stateless |
| **Speed (10 Gigabit)** | 14.23 Mpps (96% theoretical max) | 10M+ pps (hardware-limited) |
| **IPv4 Full Scan** | 42-45 minutes (gigabit), 4m 29s (10G) | ~15 minutes (stateless, 10M+ pps) |
| **Service Detection** | None (requires ZGrab2) | 85-90% accuracy (500+ services) |
| **OS Fingerprinting** | None | Full support (2,600+ signatures) |
| **Scan Types** | TCP SYN, ICMP, UDP | 8 types (SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle) |
| **Methodology** | Single probe per target | Single probe (stateless) or adaptive (stateful) |
| **Coverage** | 98% (accepts 2% packet loss) | 99%+ (stateful retries) |
| **Memory Footprint** | ~500MB (full dedup) or minimal (window) | Minimal (stateless) or moderate (stateful) |
| **IPv6 Support** | Limited (ZMapv6, requires target lists) | Full support (all scan types) |
| **Stateless Mode** | ✅ Core design | ✅ Optional mode (10M+ pps) |
| **Banner Grabbing** | ❌ (requires ZGrab2) | ✅ Built-in |
| **TLS Certificate** | ❌ (requires ZGrab2/ZLint) | ✅ X.509v3 analysis |
| **Memory Safety** | ❌ Manual C memory management | ✅ Rust compile-time guarantees |
| **Async Architecture** | ⚠️ Custom threads (send/receive/monitor) | ✅ Tokio runtime (industry-standard) |
| **Scripting** | ❌ Modular probe/output, no scripting | ⚠️ Lua plugin system (5.4) |
| **Database Storage** | ❌ (CSV/JSON output only) | ✅ SQLite with change detection |
| **Primary Use Case** | Internet-wide research surveys | Production security assessments |
| **Ecosystem** | ZGrab2, ZDNS, LZR, ZLint, Censys | Integrated single-tool solution |
| **Documentation** | Comprehensive academic papers | Professional production-ready |
| **Community** | 500+ academic papers, 33% scan traffic | New project, growing adoption |

---

## When to Use Each Tool

### Use ZMap When:

✅ **Internet-wide research surveys are the primary goal**
- Academic network measurement studies (TLS certificates, protocol adoption)
- Full IPv4 scans in 42-45 minutes (gigabit) or 4 minutes 29 seconds (10 gigabit)
- Horizontal scanning (many hosts, single port) optimization
- Statistical sampling with mathematically rigorous randomization

✅ **Maximum speed with 10 gigabit hardware is available**
- 14.23 million pps (96% of theoretical 10 GigE maximum)
- PF_RING Zero Copy kernel bypass for ultimate performance
- Specialized scanning infrastructure with optimized configuration

✅ **Single-probe methodology is acceptable**
- 98% coverage sufficient (2% packet loss tolerated)
- Speed priority over perfect accuracy
- Time-critical Internet measurement requiring rapid completion

✅ **Two-phase workflow with ZGrab2 is acceptable**
- Layer 4 discovery (ZMap) + Layer 7 interrogation (ZGrab2) separation
- Ecosystem integration (ZDNS, LZR, ZLint, ZAnnotate) valuable
- Pipeline approach: `zmap -p 443 | ztee results.csv | zgrab2 http`

### Use ProRT-IP When:

✅ **Single-pass comprehensive assessment is required**
- Service detection + OS fingerprinting + TLS certificates in one tool
- 10M+ pps stateless for rapid discovery (ZMap gigabit-class)
- 50K+ pps stateful with 85-90% detection accuracy
- No multi-tool pipeline orchestration needed

✅ **Detection capabilities are critical**
- Service version identification (500+ services, growing database)
- OS fingerprinting (Nmap-compatible, 2,600+ signatures)
- TLS certificate analysis (X.509v3, chain validation, SNI support)
- Banner grabbing for application-layer identification

✅ **Production security operations require reliability**
- Memory safety (Rust compile-time guarantees vs C manual memory)
- Comprehensive error handling (detailed actionable messages)
- Database storage with change detection over time
- Event-driven architecture for real-time monitoring

✅ **Cross-platform consistency matters**
- 10M+ pps stateless on Linux, Windows, macOS, FreeBSD (consistent)
- No platform-specific optimizations required
- Single binary deployment across diverse environments

✅ **Multiple scan types needed**
- 8 scan types (SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle) vs ZMap's basic SYN/ICMP/UDP
- Firewall detection (ACK scan)
- Stealth techniques (FIN/NULL/Xmas)
- Anonymity (Idle scan via zombie hosts)

---

## Speed Comparison

### Benchmark Results (65,535-Port SYN Scan)

| Scanner | Mode | Speed (pps) | Time | Ratio |
|---------|------|-------------|------|-------|
| **ZMap** | 10 GigE (PF_RING ZC) | 14.23M | ~4.6 seconds | **1.0x baseline** |
| **ProRT-IP** | Stateless | 10M+ | ~6.5 seconds | **1.4x slower** |
| **ZMap** | Gigabit (standard) | 1.44M | ~45 seconds | **9.8x slower** |
| **ProRT-IP** | Stateful T5 | 50K+ | ~21 minutes | **274x slower** |
| **ZMap** | Conservative (10K pps) | 10K | ~109 minutes | **1,422x slower** |

**Notes:**
- ZMap 10 GigE requires specialized hardware (Intel X540-AT2, PF_RING ZC kernel bypass)
- ProRT-IP stateless mode (10M+ pps) comparable to ZMap gigabit (1.44 Mpps)
- ProRT-IP stateful mode adds detection capabilities unavailable in ZMap core

### Internet-Scale Scanning (IPv4 Single-Port)

| Scanner | Configuration | Time | Notes |
|---------|---------------|------|-------|
| **ZMap** | 14.23 Mpps (10 GigE + PF_RING ZC) | ~4 minutes 29 seconds | Entire IPv4 (3.7B addresses), academic record |
| **ProRT-IP** | 10M+ pps (stateless) | ~6-7 minutes | Entire IPv4 (3.7B addresses), port 443 |
| **ZMap** | 1.44 Mpps (gigabit standard) | ~42-45 minutes | Standard configuration, no kernel bypass |
| **ProRT-IP** | 50K+ pps (stateful + detection) | ~20 hours | With service detection, OS fingerprinting, TLS |
| **Nmap** | Optimized (-T5, 2 probes max) | ~62.5 days | 1,300x slower than ZMap gigabit |

**ZMap vs Nmap Empirical Testing** (1M hosts, TCP port 443):
- ZMap: ~10 seconds, 98.7% coverage (single probe)
- Nmap -T5 (max 2 probes): 45 minutes, 97.8% coverage
- Nmap (single probe): 24 minutes, 81.4% coverage

**ZMap vs Masscan** (10 GigE hardware):
- ZMap: 14.1 Mpps (94.6% line rate), single receive queue
- Masscan: 7.4 Mpps (49.6% line rate), dual receive-side scaling queues

---

## Detection Capabilities

### Service Version Detection

| Scanner | Capability | Method | Database | Detection Rate | Notes |
|---------|------------|--------|----------|----------------|-------|
| **ZMap** | None (core) | N/A | N/A | N/A | Requires ZGrab2 for application-layer |
| **ZMap + ZGrab2** | Application-layer | Stateful handshakes | 12 protocols | Protocol-specific | HTTP, HTTPS, SSH, Telnet, FTP, SMTP, POP3, IMAP, Modbus, BACNET, S7, Fox |
| **ZMap + LZR** | Protocol identification | 5 handshakes | 99% accurate | Multi-protocol | Addresses Layer 4/Layer 7 gap |
| **ProRT-IP** | Comprehensive detection | Signature matching | 500+ services | 85-90% accuracy | 187 probes, version extraction, CPE identifiers |

### OS Fingerprinting

| Scanner | Capability | Method | Database | Accuracy |
|---------|------------|--------|----------|----------|
| **ZMap** | None | N/A (architectural limitation) | N/A | N/A |
| **ProRT-IP** | Full support | 16-probe sequence | 2,600+ signatures (Nmap DB) | Comparable to Nmap |

**Key Difference**: ZMap's stateless architecture fundamentally precludes OS fingerprinting (requires multiple probes and response correlation). ZGrab2 provides application-layer data but not OS detection. ProRT-IP integrates OS fingerprinting directly.

---

## Feature Comparison

### Scan Types

| Feature | ZMap | ProRT-IP |
|---------|------|----------|
| **TCP SYN** | ✅ Primary mode (tcp_synscan) | ✅ Default (-sS) |
| **TCP Connect** | ❌ Not supported | ✅ Supported (-sT) |
| **FIN Scan** | ❌ Not supported | ✅ Stealth mode (-sF) |
| **NULL Scan** | ❌ Not supported | ✅ Stealth mode (-sN) |
| **Xmas Scan** | ❌ Not supported | ✅ Stealth mode (-sX) |
| **ACK Scan** | ❌ Not supported | ✅ Firewall detection (-sA) |
| **UDP Scan** | ✅ Via probe module (payload templating) | ✅ Protocol payloads (-sU) |
| **Idle Scan** | ❌ Not supported | ✅ Maximum anonymity (-sI) |
| **ICMP Scan** | ✅ icmp_echoscan, icmp_echo_time modules | ⚠️ Limited (host discovery only) |

### Advanced Features

| Feature | ZMap | ProRT-IP |
|---------|------|----------|
| **Stateless Scanning** | ✅ Core design (zero per-connection state) | ✅ Optional mode (10M+ pps) |
| **Stateful Scanning** | ❌ Architectural limitation | ✅ Primary mode (50K+ pps with detection) |
| **Address Randomization** | ✅ Cyclic multiplicative groups (mathematically rigorous) | ✅ Adaptive randomization |
| **Pause/Resume** | ⚠️ Via seed + sharding (complex) | ✅ Checkpoint-based state preservation |
| **Sharding** | ✅ Built-in (--shards, --shard, --seed) | ⚠️ Manual (target list splitting) |
| **Banner Grabbing** | ❌ Requires ZGrab2 | ✅ Built-in (all protocols) |
| **TLS Certificate** | ❌ Requires ZGrab2 + ZLint | ✅ X.509v3 analysis, chain validation, SNI |
| **Rate Limiting** | ✅ Packet rate (-r) or bandwidth (-B) | ✅ Industry-leading -1.8% overhead |
| **Output Formats** | ✅ CSV (default), JSON (compile flag) | ✅ Text, JSON, XML (Nmap), Greppable, PCAPNG |
| **Database Storage** | ❌ File output only | ✅ SQLite with change detection |
| **IPv6 Support** | ⚠️ Limited (ZMapv6, requires target lists) | ✅ Full support (100% coverage, all scan types) |
| **Blacklist/Allowlist** | ✅ Radix tree (complex, efficient at 14+ Mpps) | ✅ CIDR notation (standard, simple) |
| **Kernel Bypass** | ✅ PF_RING Zero Copy (10 GigE) | ❌ Standard async I/O |
| **Memory Safety** | ❌ C manual memory | ✅ Rust compile-time guarantees |

---

## Architecture Comparison

### ZMap's Architecture

**Language**: C (highly optimized, kernel bypass options)
**Core Design**: Stateless asynchronous scanning with mathematically rigorous randomization

**Key Innovations**:

1. **Cyclic Multiplicative Groups for Address Permutation**:
   - Multiplicative group (Z/pZ)× modulo p where p = 2³² + 15 (smallest prime > 2³²)
   - Sequence a(i+1) = g × a(i) mod p produces complete random permutation
   - Requires storing only 3 integers: primitive root g, first address a₀, current address a(i)
   - Mathematically rigorous randomization suitable for statistical sampling

2. **Stateless Scanning with UMAC Validation**:
   - Zero per-connection state (eliminates memory overhead for billions of addresses)
   - UMAC (Universal Message Authentication Code) encodes validation in probe packets
   - Source port and sequence number contain cryptographic validation
   - Receiver independently validates responses without sender coordination

3. **Asynchronous Send/Receive Threading**:
   - Minimal shared state (independent sender and receiver threads)
   - Sender operates in tight loop at maximum NIC capacity
   - Receiver independently captures and validates via libpcap
   - Monitor thread tracks progress without synchronization overhead

4. **Direct Ethernet Frame Generation**:
   - Bypasses kernel TCP/IP stack entirely via raw sockets
   - Eliminates routing lookups, ARP cache checks, netfilter processing
   - PF_RING Zero Copy (10 GigE) provides direct userspace-to-NIC communication
   - Pre-caches static packet content, updates only host-specific fields

5. **Constraint Tree Optimization**:
   - Hybrid radix tree + /20 prefix array for complex blacklist processing
   - Enables 1,000+ blacklist entries without performance impact at 14+ Mpps
   - O(log n) recursive procedures map permutation index to allowed addresses

**Strengths**:
- Absolute maximum speed for horizontal scanning (14.23 Mpps at 10 GigE)
- Perfect randomization with mathematical proof (suitable for research sampling)
- Minimal memory footprint (~500MB full dedup or negligible with window method)
- 97-98% of theoretical network capacity utilization
- Proven at Internet scale (500+ academic papers, 33% of scan traffic)

**Weaknesses**:
- No service detection or OS fingerprinting (architectural limitation)
- Single-probe methodology (98% coverage, accepts 2% packet loss)
- IPv4-only design (IPv6 requires separate ZMapv6 with target generation)
- Manual memory management risks (C buffer overflows, use-after-free)
- Layer 4/Layer 7 gap (TCP liveness ≠ service presence)

---

### ProRT-IP's Architecture

**Language**: Rust (memory-safe, zero-cost abstractions)
**Core Design**: Hybrid stateful/stateless with async I/O and comprehensive detection

**Key Innovations**:

1. **Tokio Async Runtime**: Industry-standard non-blocking I/O, proven scalability
2. **Hybrid Scanning Modes**: Stateless (10M+ pps) for speed + Stateful (50K+ pps) for detection
3. **Memory Safety**: Compile-time guarantees (no buffer overflows, no use-after-free)
4. **Event-Driven Architecture**: Pub-sub system with -4.1% overhead
5. **Rate Limiting V3**: Industry-leading -1.8% overhead (bucket algorithm + adaptive throttling)

**Strengths**:
- Memory safety without performance penalty (Rust guarantees)
- Comprehensive detection (service versions, OS, TLS certificates) in single tool
- 8 scan types (flexibility for different scenarios)
- Cross-platform consistency (10M+ pps on Linux/Windows/macOS)
- Modern features (database storage, TUI, event system, plugin system)

**Weaknesses**:
- Maximum stateless speed 10M+ pps (vs ZMap 14.23 Mpps with PF_RING)
- Newer project (less field testing than ZMap's 11+ years, 500+ papers)
- No kernel bypass optimizations (standard async I/O only)

---

## Use Cases

### ZMap Excels At:

**1. Internet-Wide TLS Certificate Surveys**

```bash
# Scan entire IPv4 for port 443 in 42-45 minutes
zmap -p 443 -B 1G -o https-hosts.csv
cat https-hosts.csv | zgrab2 tls | zlint

# Academic study: 158 scans over 1 year
# Result: 33.6M unique X.509 certificates
# Discoveries: 1,832 browser-trusted CAs, misissued certificates
```

**2. Vulnerability Assessment at Internet Scale**

```bash
# UPnP vulnerability scan (entire IPv4 in under 2 hours)
zmap -p 1900 | zgrab2 upnp -o upnp-devices.json

# Heartbleed monitoring (scans every few hours)
zmap -p 443 | zgrab2 tls --heartbleed -o heartbleed-check.json

# Result: 15.7M publicly accessible UPnP devices
# Result: 3.4M vulnerable systems identified
```

**3. Network Infrastructure Monitoring**

```bash
# Hurricane Sandy impact assessment (continuous scans during storm)
while true; do
  zmap -p 80 -B 500M -o hosts-$(date +%Y%m%d-%H%M).csv
  sleep 3600  # Hourly scans
done

# Geographic mapping of >30% decrease in listening hosts
# Near real-time infrastructure assessment during disaster
```

**4. Protocol Adoption Studies**

```bash
# Random 0.05% sample across TCP ports 0-9175
for port in $(seq 0 9175); do
  zmap -p $port -n 0.05% -o port-$port.csv
done

# Discoveries: HTTP 1.77%, CWMP 1.12%, HTTPS 0.93%
# Unexpected: Port 7547 (CWMP), 3479 (2Wire RPC)
```

**5. Distributed Internet Measurement**

```bash
# Machine 1 (Google Cloud, us-central1)
zmap --shards 3 --shard 0 --seed 1234 -p 443 -B 500M -o shard-0.csv

# Machine 2 (AWS EC2, us-east-1)
zmap --shards 3 --shard 1 --seed 1234 -p 443 -B 500M -o shard-1.csv

# Machine 3 (Azure, westus2)
zmap --shards 3 --shard 2 --seed 1234 -p 443 -B 500M -o shard-2.csv

# Combines to single complete scan with geographic distribution
```

---

### ProRT-IP Excels At:

**1. Single-Pass Comprehensive Assessment**

```bash
# Stateful scan with service detection + OS fingerprinting + TLS in one pass
prtip -sS -sV -O -p- 192.168.1.0/24 \
  --with-db --database comprehensive.db \
  -oX scan.xml -oJ scan.json

# No multi-tool pipeline needed (vs ZMap + ZGrab2 + LZR + ZLint)
```

**2. Production Security Operations**

```bash
#!/bin/bash
# Daily monitoring with change detection
DB="security-monitor.db"
TARGET="production.example.com"

prtip -sS -sV -p 22,80,443,3306,3389 $TARGET --with-db --database $DB

# Compare with previous scan
SCAN1=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

if prtip db compare $DB $SCAN1 $SCAN2 | grep -q "New Open Ports"; then
  echo "ALERT: New services detected!" | mail -s "Security Alert" soc@company.com
fi
```

**3. Cross-Platform Enterprise Scanning**

```bash
# Linux workstation
prtip -sS -sV -p- 192.168.1.0/24 --with-db --database linux-scan.db

# Windows workstation (same performance characteristics)
prtip.exe -sS -sV -p- 192.168.1.0/24 --with-db --database windows-scan.db

# macOS workstation (same performance characteristics)
prtip -sS -sV -p- 192.168.1.0/24 --with-db --database macos-scan.db

# Consistent 10M+ pps stateless on all platforms (vs ZMap platform variations)
```

**4. Real-Time TUI Monitoring**

```bash
# Interactive scan visualization at 60 FPS
prtip --live -sS -sV -p- 192.168.1.0/24

# TUI Features:
# - Port Table: Interactive list with sorting/filtering
# - Service Table: Detected services with versions
# - Metrics Dashboard: Real-time throughput, progress, ETA
# - Network Graph: Time-series visualization of activity
```

**5. Bug Bounty / Penetration Testing**

```bash
# Phase 1: Stateless rapid discovery (ZMap-class speed)
prtip --stateless -p 80,443,8080,8443 --max-rate 10000000 bug-bounty-scope.txt -oJ rapid.json

# Phase 2: Stateful enumeration with detection
prtip -sS -sV -A -p- discovered-hosts.txt --with-db --database pentest.db

# Phase 3: Query interesting services
prtip db query pentest.db --service apache
prtip db query pentest.db --port 8080
```

---

## Migration Guide

### Migrating from ZMap to ProRT-IP

**What You Gain:**

**Service Detection** (85-90% accuracy with 500+ service database)
**OS Fingerprinting** (Nmap database compatible, 2,600+ signatures)
**TLS Certificate Analysis** (X.509v3, chain validation, SNI support)
**Multiple Scan Types** (8 types vs ZMap's SYN/ICMP/UDP basic)
**Memory Safety** (Rust compile-time guarantees vs C manual memory)
**Modern Features** (database storage, TUI, event system, plugin system)
**Single-Tool Solution** (no ZGrab2/LZR/ZLint pipeline orchestration)

**What You Keep:**

**High-Speed Stateless Scanning** (10M+ pps, comparable to ZMap gigabit 1.44 Mpps)
**Randomized Address Order** (prevents network saturation)
**Minimal Memory Footprint** (stateless mode negligible overhead)
**Cross-Platform Support** (Linux, Windows, macOS, FreeBSD)
**Pause/Resume Capability** (checkpoint-based state preservation)

**What Changes:**

**Maximum Speed** (10M+ pps vs ZMap 14.23 Mpps with 10 GigE + PF_RING ZC)
**Methodology** (hybrid stateful/stateless vs pure stateless)
**Ecosystem** (single integrated tool vs ZMap + ZGrab2 + LZR pipeline)
**Sharding** (manual target splitting vs ZMap's built-in --shards/--shard/--seed)
**Research Focus** (production security vs academic Internet measurement)

**Migration Steps:**

**1. Install ProRT-IP**

```bash
# Linux
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-x86_64-linux.tar.gz
tar xzf prtip-x86_64-linux.tar.gz
sudo mv prtip /usr/local/bin/
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip
```

**2. Test Familiar ZMap-Style Commands**

```bash
# ZMap: Internet-wide port 443 scan
zmap -p 443 -B 1G -o https-hosts.csv

# ProRT-IP: Equivalent stateless scan
prtip --stateless -p 443 --max-rate 10000000 0.0.0.0/0 -oJ https-hosts.json
```

**3. Leverage Single-Tool Detection Advantage**

```bash
# ZMap: Two-phase workflow (Layer 4 + Layer 7)
zmap -p 443 | ztee results.csv | zgrab2 http > http-data.json

# ProRT-IP: Single-pass with integrated detection
prtip -sS -sV -p 443 0.0.0.0/0 --with-db --database https-scan.db
prtip db query https-scan.db --service apache
```

**4. Explore Database Features**

```bash
# Run daily scans with change detection
prtip -sS -sV -p 22,80,443 critical-infrastructure.txt \
  --with-db --database monitoring.db

# Compare scans over time
prtip db compare monitoring.db 1 2
prtip db export monitoring.db --scan-id 1 --format json -o scan1.json
```

**5. Integration Patterns**

```bash
# Phase 1: ProRT-IP stateless (ZMap-class speed)
prtip --stateless -p 80,443 --max-rate 5000000 targets.txt -oJ rapid.json

# Phase 2: ProRT-IP stateful (comprehensive detection)
prtip -sS -sV -O -p- discovered-hosts.txt --with-db --database detailed.db

# Phase 3: Nmap deep inspection (optional)
nmap -sS -sV -sC --script vuln -iL interesting-hosts.txt -oX nmap-vuln.xml
```

---

## Command Comparison

### Basic Scanning

| Operation | ZMap | ProRT-IP |
|-----------|------|----------|
| **SYN scan** | `zmap -p 80` | `prtip -sS -p 80 TARGET` |
| **All ports** | `zmap -p 1-65535` | `prtip -p- TARGET` |
| **Multiple ports** | `zmap -p 80,443,8080` | `prtip -p 80,443,8080 TARGET` |
| **Port ranges** | `zmap -p 1000-2000` | `prtip -p 1000-2000 TARGET` |
| **UDP scan** | `zmap --probe-module=udp --probe-args=text:payload` | `prtip -sU -p 53,161 TARGET` |
| **ICMP scan** | `zmap --probe-module=icmp_echoscan` | `prtip -PE TARGET` (host discovery) |
| **Target file** | `zmap -p 80 -I targets.txt` | `prtip -p 80 -iL targets.txt` |
| **Exclude list** | `zmap -p 80 -b exclude.txt` | `prtip -p 80 --exclude exclude.txt` |

### Performance Tuning

| Operation | ZMap | ProRT-IP |
|-----------|------|----------|
| **Set rate (pps)** | `zmap -p 80 -r 100000` | `prtip -p 80 --max-rate 100000 TARGET` |
| **Set bandwidth** | `zmap -p 80 -B 1G` | `prtip -p 80 --max-rate 1488000 TARGET` (1G ≈ 1.488M pps) |
| **Unlimited rate** | `zmap -p 80 -r 0` | `prtip -p 80 --max-rate 0 TARGET` |
| **Timing template** | N/A (explicit rate only) | `prtip -T5 -p 80 TARGET` (aggressive) |
| **Sender threads** | `zmap -p 80 -T 4` | N/A (automatic parallelism) |
| **Max targets** | `zmap -p 80 -n 1000000` | `prtip -p 80 TARGET --max-targets 1000000` |
| **Max runtime** | `zmap -p 80 -t 60` | `prtip -p 80 TARGET --max-runtime 60` |
| **Cooldown time** | `zmap -p 80 -c 10` | N/A (adaptive timeout) |

### Detection

| Operation | ZMap | ProRT-IP |
|-----------|------|----------|
| **Service detection** | `zmap -p 443 | zgrab2 http` | `prtip -sV -p 443 TARGET` |
| **Banner grabbing** | `zmap -p 22 | zgrab2 ssh` | `prtip -sV -p 22 TARGET` (built-in) |
| **TLS certificates** | `zmap -p 443 | zgrab2 tls | zlint` | `prtip -sV -p 443 TARGET` (X.509v3 analysis) |
| **OS fingerprinting** | N/A (not supported) | `prtip -O TARGET` |
| **Aggressive** | N/A | `prtip -A TARGET` (-sV -O -sC --traceroute) |

### Output Formats

| Operation | ZMap | ProRT-IP |
|-----------|------|----------|
| **CSV** | `zmap -p 80 -o results.csv` (default) | `prtip -p 80 TARGET -oG results.gnmap` |
| **JSON** | `zmap -p 80 -O json -o results.json` (compile flag) | `prtip -p 80 TARGET -oJ results.json` |
| **XML** | N/A | `prtip -p 80 TARGET -oX results.xml` (Nmap-compatible) |
| **Normal text** | N/A | `prtip -p 80 TARGET -oN results.txt` |
| **All formats** | N/A | `prtip -p 80 TARGET -oA results` |
| **Database** | N/A | `prtip -p 80 TARGET --with-db --database scan.db` |
| **Field selection** | `zmap -p 80 -f saddr,daddr,sport` | N/A (automatic based on scan type) |
| **Output filter** | `zmap -p 80 --output-filter "success=1"` | N/A (filtering via database queries) |

### Distributed Scanning

| Operation | ZMap | ProRT-IP |
|-----------|------|----------|
| **Sharding** | `zmap --shards 3 --shard 0 --seed 1234` | Manual (split target list into 3 files) |
| **Consistent seed** | `zmap --seed 1234` (all shards) | N/A (randomization automatic) |
| **Resume** | Complex (seed + shard + start index) | `prtip --resume /tmp/scan.state` |
| **Pause** | Ctrl+C (track index manually) | `prtip --resume-file /tmp/scan.state` (automatic) |

---

## Integration Workflows

### ZMap Workflows

**Internet-Wide TLS Survey with Analysis**:

```bash
# Phase 1: Layer 4 discovery (ZMap, 42-45 minutes)
zmap -p 443 -B 1G -o https-hosts.csv

# Phase 2: Layer 7 interrogation (ZGrab2, hours to days)
cat https-hosts.csv | zgrab2 tls --timeout 10s -o tls-handshakes.json

# Phase 3: Certificate analysis (ZLint)
cat tls-handshakes.json | zlint -o certificate-validation.json

# Phase 4: Enrichment (ZAnnotate)
cat https-hosts.csv | zannotate --geoip2 --whois -o enriched-hosts.json

# Phase 5: Analysis (custom scripts)
python analyze-certificates.py certificate-validation.json > report.txt
```

**Vulnerability Assessment Pipeline**:

```bash
# Rapid UPnP discovery (ZMap + ZGrab2)
zmap -p 1900 | zgrab2 upnp -o upnp-devices.json

# Parse results and identify vulnerable versions
cat upnp-devices.json | jq -r 'select(.data.upnp.vulnerable == true) | .ip' > vulnerable-upnp.txt

# Integrate with vulnerability scanner
nmap -sV -sC --script upnp-info -iL vulnerable-upnp.txt -oX upnp-detail.xml
```

**Continuous Monitoring with Censys**:

```bash
# ZMap infrastructure powers Censys (4.3B IPv4 daily)
# Public API access instead of running scans

import censys.search
h = censys.search.CensysHosts()
query = h.search("services.service_name:APACHE", per_page=100, pages=1)

for page in query:
    for host in page:
        print(f"{host['ip']} - {host['services'][0]['service_name']}")
```

---

### ProRT-IP Workflows

**Single-Pass Comprehensive Security Assessment**:

```bash
# Phase 1: Stateless rapid discovery (10M+ pps, ZMap-class)
prtip --stateless -p 80,443,8080,8443 --max-rate 10000000 \
  enterprise-network.txt -oJ rapid-discovery.json

# Phase 2: Stateful enumeration with detection (single tool)
prtip -sS -sV -O -p- discovered-hosts.txt \
  --with-db --database comprehensive.db \
  -oX scan.xml -oJ scan.json

# Phase 3: Query and analyze (built-in database)
prtip db query comprehensive.db --service apache
prtip db query comprehensive.db --port 8080 --open
prtip db export comprehensive.db --scan-id 1 --format csv -o report.csv
```

**Continuous Security Monitoring with Change Detection**:

```bash
#!/bin/bash
# Daily scans with automated alerting

DB="security-monitor.db"
TARGETS="critical-infrastructure.txt"

# Run comprehensive scan
prtip -sS -sV -p 22,23,80,443,3389 -iL $TARGETS \
  --with-db --database $DB

# Compare with previous scan
SCAN1=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

# Alert on changes
CHANGES=$(prtip db compare $DB $SCAN1 $SCAN2)
if echo "$CHANGES" | grep -q "New Open Ports"; then
  echo "$CHANGES" | mail -s "[ALERT] New Services Detected" soc@company.com
fi
```

**Real-Time TUI Monitoring**:

```bash
# Interactive scan visualization at 60 FPS
prtip --live -sS -sV -p- 192.168.1.0/24

# Keyboard shortcuts:
# Tab: Switch between Port Table, Service Table, Metrics, Network Graph
# ↑/↓: Navigate table rows
# s: Sort by column
# f: Filter results
# q: Quit
```

**PCAPNG Packet Capture for Forensics**:

```bash
# Capture all packets during scan for post-analysis
prtip -sS -sV -p- 192.168.1.0/24 \
  --pcapng scan-$(date +%Y%m%d-%H%M).pcapng \
  --with-db --database scan.db

# Analyze with Wireshark
wireshark scan-20250514-1230.pcapng

# Query database for correlation
prtip db query scan.db --target 192.168.1.100
```

---

## Summary and Recommendations

### Choose ZMap If:

✅ **Internet-wide research surveys are the primary goal** (42-45 min full IPv4 at gigabit)
✅ **Academic network measurement** (TLS certificate studies, protocol adoption, vulnerability tracking)
✅ **Maximum speed with specialized hardware** (14.23 Mpps at 10 GigE + PF_RING ZC)
✅ **Horizontal scanning optimization** (many hosts, single port) is the use case
✅ **Mathematically rigorous randomization** for statistical sampling required
✅ **Two-phase workflow acceptable** (ZMap Layer 4 + ZGrab2 Layer 7 separation)
✅ **Censys integration valuable** (4.3B IPv4 daily scans, public API access)

### Choose ProRT-IP If:

✅ **Single-pass comprehensive assessment** required (service + OS + TLS in one tool)
✅ **Detection capabilities critical** (85-90% service accuracy, OS fingerprinting, TLS certificates)
✅ **Production security operations** (memory safety, error handling, database storage)
✅ **Cross-platform consistency** matters (10M+ pps on Linux/Windows/macOS)
✅ **Multiple scan types needed** (8 types: SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle)
✅ **Memory safety mandatory** (Rust guarantees vs C manual memory)
✅ **Modern features valuable** (TUI, event system, plugin system, change detection)

### Hybrid Approach

**Phase 1: ProRT-IP Stateless Discovery** (10M+ pps, ZMap gigabit-class speed)

```bash
prtip --stateless -p 80,443 --max-rate 10000000 enterprise-network.txt -oJ rapid.json
```

**Phase 2: ProRT-IP Stateful Enumeration** (50K+ pps with 85-90% detection)

```bash
prtip -sS -sV -O -p- discovered-hosts.txt --with-db --database comprehensive.db
```

**Phase 3: Nmap Deep Inspection** (optional, 100% accuracy, vulnerability scripts)

```bash
nmap -sS -sV -sC --script vuln -iL interesting-hosts.txt -oX vuln-scan.xml
```

**Key Insight**: ZMap's maximum speed advantage (14.23 Mpps vs ProRT-IP 10M+ pps) requires specialized 10 gigabit hardware with PF_RING Zero Copy kernel bypass. For standard gigabit deployments, ZMap achieves 1.44 Mpps while ProRT-IP stateless reaches 10M+ pps (**~7x faster**). ProRT-IP's integrated detection eliminates multi-tool pipeline orchestration (ZMap + ZGrab2 + LZR + ZLint) while maintaining comparable gigabit-class speeds.

**Academic vs Production**: ZMap optimizes for Internet-wide research (500+ papers, 33% of scan traffic) with mathematically rigorous randomization and proven stateless architecture. ProRT-IP targets production security assessments with comprehensive detection, memory safety, and single-tool simplicity. Choose based on use case: academic measurement (ZMap), production security (ProRT-IP), or hybrid approach (ProRT-IP stateless + ProRT-IP stateful + optional Nmap).

---

## See Also

- [ProRT-IP vs Nmap](nmap.md) - Comparison with the gold standard scanner
- [ProRT-IP vs Masscan](masscan.md) - Comparison with the Internet-scale speed champion
- [Technical Specification](../tech-spec-v2.md) - ProRT-IP architecture details
- [Performance Characteristics](../../advanced/performance-characteristics.md) - Benchmarks and analysis
- [ZMap Project](https://zmap.io/) - Official ZMap documentation
- [Censys](https://search.censys.io/) - Public Internet search powered by ZMap infrastructure
