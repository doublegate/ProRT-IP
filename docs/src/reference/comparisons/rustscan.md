# ProRT-IP vs RustScan

Comprehensive technical comparison between ProRT-IP and RustScan, the modern port scanner that revolutionized reconnaissance by completing all 65,535 ports in 3-8 seconds—approximately 60-250 times faster than traditional Nmap port discovery.

---

## Executive Summary

**RustScan transformed network reconnaissance from a waiting game into an instant operation**. Created in 2020 by Autumn Skerritt as a three-day Rust learning project, this tool has evolved into a production-grade scanner with 18,200+ GitHub stars. RustScan scans all 65,535 ports in 3-8 seconds through single-threaded asynchronous I/O (async-std runtime, 4,500 concurrent connections), then automatically pipes discovered ports to Nmap for detailed enumeration. The hybrid approach achieves 60-250x speed advantage over Nmap's default port discovery while maintaining comprehensive analysis capabilities.

**ProRT-IP provides comparable speed with integrated detection**, achieving 10M+ pps stateless (similar to RustScan's rapid discovery) and 50K+ pps stateful with 85-90% service detection accuracy. Unlike RustScan's preprocessing-only design (requires Nmap for service enumeration), ProRT-IP integrates comprehensive detection in a single tool through Tokio multi-threaded async I/O and built-in service fingerprinting.

**The fundamental difference**: RustScan optimizes exclusively for fast port discovery (do one thing exceptionally well, delegate enumeration to Nmap), making it ideal for CTF competitions and bug bounties where seconds matter. ProRT-IP balances comparable stateless speed (10M+ pps) with integrated detection (service versions, OS fingerprinting, TLS certificates), eliminating multi-tool orchestration while maintaining single-pass comprehensive assessment capabilities.

**Key Architecture Contrast**: Both tools leverage Rust's memory safety and zero-cost abstractions, but use fundamentally different concurrency models. RustScan's single-threaded async-std (4,500 concurrent connections in one thread) optimizes for minimal resource overhead and predictable performance. ProRT-IP's Tokio multi-threaded runtime enables adaptive parallelism and comprehensive detection operations while maintaining 10M+ pps stateless throughput.

---

## Quick Comparison

| Dimension | RustScan | ProRT-IP |
|-----------|----------|----------|
| **First Released** | 2020 (3-day learning project) | 2024 (new project) |
| **Language** | Rust (single-threaded async-std) | Rust (multi-threaded Tokio) |
| **Speed (65K Ports)** | 3-8 seconds (60-250x faster than Nmap) | 6-10 seconds stateless, 15-30 min stateful |
| **Detection Method** | None (requires Nmap integration) | Integrated (500+ services, 85-90% accuracy) |
| **Architecture** | Single-threaded async I/O (4,500 concurrent) | Multi-threaded async I/O (adaptive parallelism) |
| **Service Detection** | Via Nmap only (automatic piping) | Native (187 probes, version extraction, CPE) |
| **OS Fingerprinting** | Via Nmap only | Native (2,600+ signatures, Nmap-compatible DB) |
| **Scan Types** | TCP Connect (full handshake), UDP (v2.3.0+) | 8 types (SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle) |
| **Primary Use Case** | Rapid port discovery + Nmap delegation | Single-pass comprehensive assessment |
| **Nmap Integration** | Automatic (core feature, preprocessing model) | Optional (compatibility layer, standalone capable) |
| **Scripting** | Python, Lua, Shell (RSE engine) | Lua 5.4 (plugin system) |
| **Privileges** | None required (standard sockets) | Required for raw sockets (SYN, FIN, etc.) |
| **Default Behavior** | All 65,535 ports scanned → pipe to Nmap | Top 1,000 ports (configurable) |
| **Concurrency Model** | 4,500 async tasks (single thread, batch-based) | Adaptive parallelism (CPU cores × workers) |
| **Memory Safety** | Compile-time guarantees (Rust ownership) | Compile-time guarantees (Rust ownership) |
| **Platform Support** | Linux (native), macOS/Windows (Docker only) | Linux, macOS, Windows, FreeBSD (full support) |
| **File Descriptor** | 4,500-65,535 required (ulimit challenges) | Adaptive (system-aware limits) |
| **Rate Limiting** | Timeout-based (batch size control) | Adaptive (-1.8% overhead, burst management) |
| **IPv6 Support** | Yes (less tested than IPv4) | Full support (all scan types, 100% coverage) |
| **TLS Certificate** | Via Nmap scripts | Native (X.509v3, SNI, chain validation, 1.33μs) |
| **Database Storage** | None (output to stdout/files) | Native (SQLite, historical tracking, queries) |
| **GitHub Stars** | 18,200+ | New project |
| **Maturity** | Production (50+ contributors, active development) | Production (Phase 5 complete, v0.5.0) |
| **Community** | Discord (489 members), GitHub, TryHackMe room | GitHub Discussions |

---

## When to Use Each Tool

### Use RustScan When:

✅ **CTF competitions where speed is paramount**
- 3-8 second full-range scans enable comprehensive reconnaissance
- Time saved translates to additional exploitation attempts
- Multiple CTF veterans report RustScan became essential infrastructure

✅ **Bug bounty initial reconnaissance across large scopes**
- Rapid service enumeration feeds nuclei, nikto, custom tools
- Example: `rustscan -a 10.20.30.0/24 -p 80,443,8080,8443 -b 4000 > web_services.txt`
- Identifies all HTTP/HTTPS services in seconds for subsequent testing

✅ **Single-host or small subnet scanning**
- Optimized for "scanning all ports on single hosts with maximum speed"
- Default 4,500 concurrent connections per host (batch-based)
- Not designed for scanning thousands of hosts (use Masscan/ZMap for Internet-scale)

✅ **Automatic Nmap integration valuable**
- Seamless transition from discovery to enumeration without orchestration
- RustScan finds ports (3-8 sec) → Nmap enumerates services (10-15 sec) = ~19 sec total
- Example: `rustscan -a TARGET -- -sV -sC` (service detection + default scripts)

✅ **Unprivileged execution required**
- Standard TCP sockets (no raw socket access needed)
- Full three-way handshakes provide reliable open/closed determination
- No sudo/root required (unlike SYN scanning)

### Use ProRT-IP When:

✅ **Single-pass comprehensive assessment required**
- Service detection + OS fingerprinting + TLS certificates in one tool
- 10M+ pps stateless for rapid discovery (comparable to RustScan)
- 50K+ pps stateful with 85-90% detection accuracy
- No multi-tool pipeline orchestration needed

✅ **Detection capabilities critical**
- Service version identification (500+ services, growing database)
- OS fingerprinting (Nmap-compatible, 2,600+ signatures, 16-probe sequence)
- TLS certificate analysis (X.509v3, chain validation, SNI support)
- Version extraction and CPE identifiers for vulnerability correlation

✅ **Advanced scan types needed**
- 8 scan types (SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle)
- Firewall/IDS evasion techniques (fragmentation, decoys, TTL manipulation)
- Idle scan for maximum anonymity (zombie host required)

✅ **Database storage and historical tracking valuable**
- SQLite integration (WAL mode, batch inserts, comprehensive indexes)
- Historical comparisons (detect new services, version changes)
- Query interface (search by port, service, target, scan ID)

✅ **Cross-platform consistency matters**
- 10M+ pps stateless on Linux, macOS, Windows (production binaries)
- FreeBSD support (x86_64)
- No Docker requirement (native executables, platform-optimized)

---

## Speed Comparison

### Benchmark Results (65,535-Port Full Scan)

| Scanner | Mode | Speed (pps) | Time | Ratio |
|---------|------|-------------|------|-------|
| **RustScan** | Default (batch 4,500, timeout 1,500ms) | 3,000-4,500 | **3-8 seconds** | **1.0x baseline** |
| **ProRT-IP** | Stateless (10M+ pps maximum) | 10M+ | **~6-10 seconds** | **1.3-2.2x slower** |
| **ProRT-IP** | Stateful SYN (T5 aggressive) | 50K+ | **~15-30 minutes** | **112-225x slower** |
| **Nmap** | Default (T3 Normal, top 1,000 ports) | 5K-10K | **~15 minutes** | **112-120x slower** |
| **Nmap** | Full range (-p-, T4 Aggressive) | 20K-30K | **~5-10 minutes** | **37-75x slower** |
| **Nmap** | Full range + aggressive (-p- -A -T5) | 30K-50K | **~17 minutes** | **127-212x slower** |

**Notes**:
- RustScan's "60-250x faster than Nmap" claim compares port discovery against Nmap `-A` (aggressive: version detection, OS detection, scripts, traceroute)
- Fair comparison (port discovery only): RustScan 3-8s vs Nmap `-sS -p-` 5-10 minutes (37-75x speed advantage)
- ProRT-IP stateless mode achieves comparable speed to RustScan (6-10s vs 3-8s, 1.3-2.2x difference)
- ProRT-IP's stateful mode adds detection overhead (service fingerprinting, OS probing) but provides integrated analysis

### RustScan Configuration Impact

| Configuration | Batch Size | Timeout | Expected Scan Time | Use Case |
|--------------|------------|---------|-------------------|----------|
| **Default** | 4,500 | 1,500ms | ~8 seconds | Balanced speed/reliability |
| **Fast** | 10,000 | 1,000ms | ~5 seconds | Local networks, high bandwidth |
| **Maximum** | 65,535 | 500ms | ~1-3 seconds (theoretical) | Requires `ulimit -n 70000`, aggressive |
| **Stealth** | 10-100 | 5,000ms | ~5 minutes | Reduced detection likelihood |
| **Conservative** | 500 | 3,000ms | ~30 seconds | High-latency connections, maximum accuracy |

**System Constraints**:
- Linux default ulimit (8,800): Supports batch sizes up to 8,000 comfortably
- macOS default ulimit (255): Severely constrains performance, Docker recommended
- Kali Linux default (~90,000): Enables maximum performance with batch size 65,535
- Windows WSL: Lacks ulimit support, requires Docker deployment

### ProRT-IP vs RustScan Speed Analysis

**Stateless Mode (Comparable)**:
- RustScan: 3-8 seconds (single-threaded async-std, 4,500 concurrent connections)
- ProRT-IP: 6-10 seconds (multi-threaded Tokio, 10M+ pps maximum)
- Difference: 1.3-2.2x (ProRT-IP slightly slower but provides integrated detection option)

**Detection Phase**:
- RustScan: Requires Nmap integration (automatic, adds 10-15 seconds for service detection on open ports only)
- ProRT-IP: Integrated service detection during scan (no separate phase, 85-90% accuracy, 187 probes)

**Total Time for Comprehensive Assessment**:
- RustScan + Nmap: 3-8s (discovery) + 10-15s (Nmap enumeration on open ports) = **~13-23 seconds**
- ProRT-IP stateful: 15-30 minutes (single-pass with integrated detection on all ports)
- ProRT-IP stateless + stateful: 6-10s (discovery) + 2-5 min (targeted enumeration) = **~2-5 minutes**

**Strategic Insight**: RustScan + Nmap workflow (13-23 seconds) is faster for scenarios where only a few ports are open. ProRT-IP stateful (15-30 minutes) provides comprehensive detection but longer runtime. ProRT-IP hybrid (stateless + targeted stateful) balances speed with integrated detection (2-5 minutes total).

---

## Detection Capabilities

### Service Version Detection

| Scanner | Capability | Method | Database | Detection Rate | Notes |
|---------|------------|--------|----------|----------------|-------|
| **RustScan** | None (core) | N/A | N/A | N/A | Requires Nmap integration for service detection |
| **RustScan + Nmap** | Comprehensive detection | Signature matching | 1,000+ services (Nmap DB) | ~95% (Nmap quality) | Automatic piping: `rustscan -a TARGET -- -sV` |
| **ProRT-IP** | Integrated detection | Signature matching | 500+ services (growing) | 85-90% accuracy | 187 probes, version extraction, CPE identifiers |

**RustScan Workflow**:
```bash
# Port discovery (3-8 seconds)
rustscan -a 192.168.1.100

# Automatic Nmap integration
nmap -Pn -vvv -p 22,80,443,3306 192.168.1.100

# Custom Nmap arguments
rustscan -a 192.168.1.100 -- -sV -sC  # Service detection + default scripts
```

**ProRT-IP Workflow**:
```bash
# Single-pass comprehensive (15-30 minutes, integrated detection)
prtip -sS -sV -p- 192.168.1.100

# Hybrid approach (faster)
prtip --stateless -p- 192.168.1.100 -oJ discovery.json  # 6-10 seconds
prtip -sS -sV -p 22,80,443,3306 192.168.1.100           # 2-5 minutes targeted
```

### OS Fingerprinting

| Scanner | Capability | Method | Database | Accuracy |
|---------|------------|--------|----------|----------|
| **RustScan** | None (core) | N/A | N/A | N/A |
| **RustScan + Nmap** | Full support (via Nmap) | 16-probe sequence | 2,600+ signatures | Comparable to Nmap |
| **ProRT-IP** | Native support | 16-probe sequence | 2,600+ signatures (Nmap DB) | Comparable to Nmap |

**RustScan OS Fingerprinting**:
```bash
# Requires Nmap integration
rustscan -a TARGET -- -O

# Aggressive scan (OS + service + scripts)
rustscan -a TARGET -- -A
```

**ProRT-IP OS Fingerprinting**:
```bash
# Native implementation
prtip -sS -O TARGET

# Comprehensive
prtip -sS -O -sV -A TARGET
```

### TLS Certificate Analysis

| Scanner | Capability | Method | Features |
|---------|------------|--------|----------|
| **RustScan** | None (core) | N/A | Requires Nmap SSL scripts |
| **RustScan + Nmap** | Via NSE scripts | `--script ssl-cert` | Certificate details, chains, validation |
| **ProRT-IP** | Native (Sprint 5.5) | X.509v3 parser | SNI support, chain validation, 1.33μs parsing, automatic HTTPS detection |

**Example Comparison**:
```bash
# RustScan + Nmap SSL
rustscan -a TARGET -- --script ssl-cert,ssl-enum-ciphers

# ProRT-IP native TLS
prtip -sS -sV -p 443 TARGET  # Automatic certificate extraction with SNI support
```

---

## Feature Comparison

### Scan Types

| Scan Type | RustScan | ProRT-IP |
|-----------|----------|----------|
| **TCP Connect** | ✅ Full handshake (default) | ✅ Full handshake |
| **TCP SYN** | ❌ (uses standard sockets only) | ✅ Default scan type |
| **TCP FIN** | ❌ | ✅ Stealth scanning |
| **TCP NULL** | ❌ | ✅ Stealth scanning |
| **TCP Xmas** | ❌ | ✅ Stealth scanning |
| **TCP ACK** | ❌ | ✅ Firewall mapping |
| **UDP** | ✅ v2.3.0+ (timeout-based, less reliable) | ✅ Protocol-specific payloads |
| **Idle Scan** | ❌ | ✅ Maximum anonymity (zombie host) |
| **ICMP** | ❌ | ✅ Host discovery |

### Advanced Features

| Feature | RustScan | ProRT-IP |
|---------|----------|----------|
| **Stateless Scanning** | ❌ (full handshakes only) | ✅ 10M+ pps maximum |
| **Stateful Scanning** | ✅ TCP Connect (4,500 concurrent) | ✅ 50K+ pps with detection |
| **Service Detection** | ❌ (requires Nmap) | ✅ Native (500+ services, 85-90%) |
| **OS Fingerprinting** | ❌ (requires Nmap) | ✅ Native (2,600+ signatures) |
| **TLS Certificate** | ❌ (requires Nmap scripts) | ✅ Native (X.509v3, SNI, 1.33μs) |
| **Nmap Integration** | ✅ Automatic piping (core feature) | ✅ Optional compatibility layer |
| **Scripting Engine** | ✅ RSE (Python, Lua, Shell) | ✅ Lua 5.4 plugin system |
| **Rate Limiting** | Timeout-based (batch size control) | ✅ Adaptive (-1.8% overhead) |
| **Adaptive Learning** | ✅ Basic maths (no bloated ML) | ✅ Performance monitoring |
| **Configuration Files** | ✅ TOML (`~/.rustscan.toml`) | ✅ TOML + CLI flags |
| **Output Formats** | Greppable, JSON, text | JSON, XML (Nmap-compatible), CSV, text |
| **Database Storage** | ❌ (stdout/files only) | ✅ SQLite (WAL, queries, historical) |
| **IPv6 Support** | ✅ (less tested than IPv4) | ✅ Full support (all scan types, 100%) |
| **Batch Processing** | ✅ 4,500 default (configurable to 65,535) | ✅ Adaptive parallelism |
| **Privilege Escalation** | ❌ Not required (standard sockets) | ✅ Required for raw sockets (SYN, FIN, etc.) |
| **Memory Safety** | ✅ Rust ownership model | ✅ Rust ownership model |
| **Zero-Cost Abstractions** | ✅ Compile-time optimizations | ✅ Compile-time optimizations |
| **Cross-Platform** | Linux (native), macOS/Windows (Docker) | Linux, macOS, Windows, FreeBSD (native) |
| **Accessibility** | ✅ `--accessible` (screen reader friendly) | Standard terminal output |

---

## Architecture Comparison

### RustScan's Architecture

**Language**: Rust (async-std runtime, single-threaded event loop)
**Core Design**: Batch-based asynchronous port probing with automatic Nmap integration

**Key Innovations**:

1. **Single-Threaded Asynchronous I/O**
   - async-std event loop reactor handles thousands of concurrent connections in one thread
   - Avoids context-switching overhead and reduces memory consumption
   - Leverages OS-level async I/O primitives (epoll on Linux, kqueue on BSD/macOS, IOCP on Windows)
   - Default 4,500 concurrent async tasks (configurable to 65,535 maximum)

2. **Batch-Based Port Probing**
   - Divides 65,535 ports into batches (default 4,500)
   - Scans each batch completely, then moves to next
   - Prevents file descriptor exhaustion (most systems have 8,000-8,800 ulimit)
   - Sweet spot: 4,000-10,000 batch size with 5,000+ ulimit

3. **Adaptive Learning System**
   - Automatically detects system file descriptor limits via rlimit crate
   - Adjusts batch sizes to system capabilities
   - Learns optimal timeout values over time
   - Stores patterns in `~/.rustscan.toml` (basic maths, no bloated ML)

4. **Preprocessing + Delegation Model**
   - Core philosophy: "Do one thing exceptionally well" (find open ports fast)
   - Automatic Nmap integration: Constructs `nmap -Pn -vvv -p $DISCOVERED_PORTS $TARGET`
   - Seamless transition from discovery to enumeration without manual orchestration

5. **Performance Regression Prevention**
   - Automated HyperFine benchmarking in CI (v2.4.1+)
   - Every pull request triggers benchmark runs
   - Significant performance degradation fails the build
   - Treats speed as first-class requirement alongside correctness/security

**Strengths**:
- Absolute maximum speed for single-host port discovery (3-8 seconds for 65K ports)
- Minimal resource overhead (single-threaded design eliminates synchronization)
- Automatic Nmap integration creates seamless workflows (speed + depth without orchestration)
- Memory safety (Rust ownership model prevents buffer overflows, use-after-free, data races)
- Zero-cost abstractions (expressive high-level code compiles to efficient machine code)

**Weaknesses**:
- No service detection or OS fingerprinting (architectural limitation, requires Nmap)
- Limited scan types (TCP Connect, UDP only—no SYN, FIN, NULL, Xmas, ACK, Idle)
- Platform constraints (Windows requires Docker due to rlimit incompatibility, macOS ulimit 255 default severely limits performance)
- High file descriptor requirements (4,500-65,535 for maximum speed)
- Not designed for multi-host scanning (focused on single hosts or small subnets)

---

### ProRT-IP's Architecture

**Language**: Rust (Tokio runtime, multi-threaded async I/O)
**Core Design**: Hybrid stateful/stateless scanning with integrated comprehensive detection

**Key Innovations**:

1. **Tokio Multi-Threaded Async Runtime**
   - Industry-standard async I/O with work-stealing scheduler
   - Adaptive parallelism (CPU cores × workers)
   - Multi-threaded event loop enables concurrent detection operations
   - Cross-platform consistency (10M+ pps on Linux/Windows/macOS)

2. **Hybrid Scanning Modes**
   - Stateless mode: 10M+ pps for rapid discovery (comparable to RustScan)
   - Stateful mode: 50K+ pps with integrated detection (service, OS, TLS)
   - Mode switching without tool change (seamless workflow)

3. **Integrated Detection Pipeline**
   - Service detection: 187 probes, 500+ service database, 85-90% accuracy
   - OS fingerprinting: 16-probe sequence, 2,600+ signatures (Nmap-compatible DB)
   - TLS certificate analysis: X.509v3 parser, SNI support, 1.33μs parsing
   - Single-pass comprehensive assessment (no multi-tool orchestration)

4. **Event-Driven Architecture**
   - Pub-sub event system (Sprint 5.5.3, -4.1% overhead)
   - 18 event types across 4 categories
   - Real-time metrics, progress tracking, ETAs
   - TUI foundation for live dashboard visualization

5. **Rate Limiting V3**
   - Industry-leading -1.8% overhead
   - Adaptive burst management (burst=100 optimal)
   - Token bucket algorithm with fixed-size queue
   - Prevents network congestion and target overload

**Strengths**:
- Comprehensive detection (service + OS + TLS) in single tool
- 8 scan types (SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle)
- Cross-platform native executables (no Docker requirement)
- Database storage with historical tracking and queries
- Memory safety (Rust compile-time guarantees)
- Modern features (event system, plugin system, TUI, PCAPNG capture)

**Weaknesses**:
- Stateless speed slightly slower than RustScan (6-10s vs 3-8s for 65K ports)
- Stateful mode slower due to detection overhead (15-30 minutes comprehensive)
- Requires elevated privileges for raw sockets (SYN, FIN, NULL, Xmas, ACK, Idle)
- No automatic Nmap integration (optional compatibility layer, standalone design)

---

## Use Cases

### RustScan Excels At:

#### **1. CTF Competition Reconnaissance**

Fast port discovery enables comprehensive reconnaissance in time-constrained scenarios:

```bash
# Full-range scan in 3-8 seconds
rustscan -a 10.10.10.100 -b 65535 -t 1000

# Discovered ports: 22, 80, 8080, 31337
# Automatic Nmap integration enumerates services in 10-15 seconds
# Total: ~13-23 seconds for complete reconnaissance

# Manual Nmap alternative: 15+ minutes (traditional workflow)
```

**CTF Benefits**:
- Time saved translates to additional exploitation attempts
- Finds services on unusual high ports (30000-40000 range) without manual guessing
- Multiple CTF veterans report RustScan became essential infrastructure

---

#### **2. Bug Bounty Initial Reconnaissance**

Rapid service enumeration across target scopes feeds subsequent testing:

```bash
# Find all HTTP/HTTPS services in seconds
rustscan -a 10.20.30.0/24 -p 80,443,8080,8443 -b 4000 > web_services.txt

# Feed to nuclei, nikto, or custom tools
cat web_services.txt | nuclei -t http/ -severity critical,high
```

**Benefits**:
- Broader scope coverage within bug bounty time constraints
- Clean output format (`<IP> -> [<ports>]`) for easy parsing
- Greppable mode (`-g`) enables automation

---

#### **3. Penetration Testing Hybrid Workflows**

Two-phase approach separates discovery from enumeration:

```bash
# Phase 1: Initial discovery (3-8 seconds)
rustscan -a TARGET -q > ports.txt

# Phase 2: Extract ports programmatically
PORTS=$(cat ports.txt | grep -oP '\d+' | paste -sd,)

# Phase 3: Detailed enumeration (10-15 seconds on open ports)
nmap -sV -sC -p $PORTS TARGET -oA results

# Total: ~13-23 seconds (vs 20+ minutes traditional full Nmap)
```

**Benefits**:
- Identical information to full Nmap scan in ~2% of the time
- Clean separation of phases enables custom analysis scripts
- Integration with security frameworks (Metasploit, custom Python tools)

---

#### **4. Network Mapping Across Subnets**

RustScan's speed enables comprehensive coverage previously impractical:

```bash
# Scan 10 Class C subnets in parallel
for subnet in 192.168.{1..10}.0; do
    rustscan -a $subnet/24 -p 22,80,443,3389 -b 4000 > subnet-$subnet.txt &
done

# Wait for completion
wait

# Aggregate results
cat subnet-*.txt | grep "Open" > all-services.txt

# Traditional Nmap alternative: Days of sequential scanning
```

**Benefits**:
- Inverted funnel (broad discovery → targeted depth)
- Prevents wasting enumeration effort on closed ports
- Hours instead of days for comprehensive subnet mapping

---

#### **5. Security Automation and CI/CD**

Docker integration enables consistent scanning across environments:

```bash
# GitHub Actions workflow
docker run -it --rm rustscan/rustscan:2.1.1 -a infrastructure.company.com

# GitLab CI security scan stage
rustscan-security-scan:
  image: rustscan/rustscan:2.1.1
  script:
    - rustscan -a $TARGET_INFRA -b 4000 > results.txt
    - if grep -q "unexpected_port" results.txt; then exit 1; fi

# Jenkins pipeline
pipeline {
    agent { docker 'rustscan/rustscan:2.1.1' }
    stages {
        stage('Scan') {
            steps {
                sh 'rustscan -a prod-servers.txt -p 22,80,443'
            }
        }
    }
}
```

**Benefits**:
- Containerized deployment eliminates environment dependencies
- Consistent performance regardless of runner configuration
- Rapid feedback in security pipelines

---

### ProRT-IP Excels At:

#### **1. Single-Pass Comprehensive Security Assessment**

Integrated detection eliminates multi-tool orchestration:

```bash
# Service detection + OS fingerprinting + TLS certificates in one tool
prtip -sS -sV -O -p- 192.168.1.0/24 \
  --with-db --database comprehensive.db \
  -oX scan.xml -oJ scan.json

# RustScan alternative requires:
# 1. rustscan -a 192.168.1.0/24 (discovery)
# 2. nmap -sV -O -p $PORTS (enumeration)
# 3. nmap --script ssl-cert (TLS analysis)
# 4. Manual result aggregation
```

**Benefits**:
- No pipeline orchestration complexity
- Database storage for historical tracking
- Multiple output formats for integration

---

#### **2. Production Security Operations with Change Detection**

Database-driven continuous monitoring detects unauthorized services:

```bash
#!/bin/bash
# Daily security scan with automatic alerting

DB="security-monitor.db"
TARGET="192.168.1.0/24"

# Run comprehensive scan
prtip -sS -sV -p 22,80,443,3306,3389 $TARGET \
  --with-db --database $DB

# Get last two scan IDs
SCAN1=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

# Compare scans and alert on changes
if prtip db compare $DB $SCAN1 $SCAN2 | grep -q "New Open Ports"; then
  echo "ALERT: Unauthorized services detected!" | \
    mail -s "Security Alert" soc@company.com
fi
```

**Benefits**:
- Automated change detection (new ports, version updates, closed services)
- Historical tracking for compliance audits
- Integrated database eliminates external storage

---

#### **3. Advanced Scan Types for Firewall Mapping**

8 scan types enable comprehensive security assessment:

```bash
# Firewall mapping with ACK scan
prtip -sA -p 1-1000 target.com

# Stealth scanning with FIN/NULL/Xmas
prtip -sF -sN -sX -p 80,443,8080 target.com

# Maximum anonymity with Idle scan (requires zombie host)
prtip -sI zombie.host.com target.com

# RustScan alternative: Only TCP Connect available
# Nmap required for advanced scan types
```

**Benefits**:
- Firewall/IDS evasion capabilities
- Idle scan for zero-attribution reconnaissance
- Combined evasion techniques (fragmentation, decoys, TTL manipulation)

---

#### **4. Real-Time Monitoring with TUI Dashboard**

Live visualization of scan progress and metrics (Sprint 6.2):

```bash
# Launch TUI with real-time updates
prtip --live -sS -sV -p- 192.168.1.0/24

# TUI Features:
# - Port Table: Interactive list with sorting/filtering (Tab navigation)
# - Service Table: Detected services with versions
# - Metrics Dashboard: Real-time throughput, progress, ETA
# - Network Graph: Time-series chart (60-second sliding window)
# - 60 FPS rendering, <5ms frame time, 10K+ events/sec throughput
```

**Benefits**:
- Professional-grade monitoring interface
- Immediate visibility into scan operations
- Keyboard navigation and multiple view modes

---

#### **5. PCAPNG Packet Capture for Forensic Analysis**

Full packet capture enables offline analysis and evidence preservation:

```bash
# Capture all packets during scan
prtip -sS -p- target.com --pcapng scan-evidence.pcapng

# Analyze with Wireshark
wireshark scan-evidence.pcapng

# Or tshark for scripting
tshark -r scan-evidence.pcapng -Y "tcp.flags.syn==1" -T fields -e ip.dst -e tcp.dstport
```

**Benefits**:
- Evidence preservation for security incidents
- Offline analysis with standard tools (Wireshark, tshark)
- Supports legal and compliance requirements

---

## Migration Guide

### RustScan → ProRT-IP

#### What You Gain

**Integrated Detection** (eliminate Nmap dependency for most use cases)
- Service version identification (500+ services, 85-90% accuracy, 187 probes)
- OS fingerprinting (Nmap-compatible, 2,600+ signatures)
- TLS certificate analysis (X.509v3, SNI support, chain validation)

**Advanced Scan Types** (8 types vs RustScan's TCP Connect only)
- SYN (default), FIN, NULL, Xmas (stealth)
- ACK (firewall mapping)
- Idle (maximum anonymity)
- UDP (protocol-specific payloads)

**Database Storage** (historical tracking and queries)
- SQLite integration (WAL mode, batch inserts, comprehensive indexes)
- Historical comparisons (detect new services, version changes, closed ports)
- Query interface (search by port, service, target, scan ID)

**Cross-Platform Native Executables** (no Docker requirement)
- Linux, macOS, Windows, FreeBSD (production binaries)
- 10M+ pps stateless on all platforms
- No ulimit configuration needed (adaptive system limits)

**Memory Safety** (both tools use Rust, but ProRT-IP adds production features)
- Compile-time guarantees (ownership model)
- Comprehensive test suite (2,102 tests, 54.92% coverage, 230M+ fuzz executions)
- Production-ready error handling and logging

---

#### What You Keep

**High-Speed Port Discovery** (comparable stateless performance)
- RustScan: 3-8 seconds for 65K ports (single-threaded async-std)
- ProRT-IP: 6-10 seconds stateless (multi-threaded Tokio, 10M+ pps)
- Difference: 1.3-2.2x (acceptable for integrated detection option)

**Rust Memory Safety** (both tools benefit from ownership model)
- Buffer overflow prevention
- Use-after-free prevention
- Data race prevention (compile-time guarantees)

**Minimal Memory Footprint** (stateless mode negligible overhead)
- RustScan: Single-threaded design, batch-based allocation
- ProRT-IP: Stream-to-disk results, adaptive parallelism

---

#### What Changes

**Speed Trade-off** (slightly slower for pure discovery, but integrated detection)
- RustScan: 3-8 seconds (port discovery only, requires Nmap for enumeration)
- ProRT-IP: 6-10 seconds stateless (comparable discovery), OR 15-30 minutes stateful (integrated detection)
- Hybrid approach: 6-10s stateless + 2-5 min targeted stateful = comprehensive assessment

**Workflow Methodology** (single tool vs preprocessing + delegation)
- RustScan: Find ports fast → pipe to Nmap → Nmap enumerates
- ProRT-IP: Single-pass comprehensive OR stateless discovery + targeted stateful
- Integration: ProRT-IP can output to Nmap format for compatibility

**Privilege Requirements** (raw sockets vs standard sockets)
- RustScan: No privileges required (standard TCP sockets, full handshakes)
- ProRT-IP: Elevated privileges for SYN/FIN/NULL/Xmas/ACK/Idle (raw sockets)
- Alternative: ProRT-IP `-sT` (TCP Connect) requires no privileges like RustScan

---

#### Migration Steps

**1. Install ProRT-IP**

```bash
# Linux (download from GitHub releases)
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz
tar xzf prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz
sudo mv prtip /usr/local/bin/
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip  # Grant capabilities
```

**2. Test Familiar RustScan-Style Commands**

```bash
# RustScan
rustscan -a 192.168.1.100 -p 22,80,443,8080

# ProRT-IP equivalent (stateless, fast discovery)
prtip --stateless -p 22,80,443,8080 192.168.1.100

# ProRT-IP equivalent (stateful with detection)
prtip -sS -sV -p 22,80,443,8080 192.168.1.100
```

**3. Leverage Integrated Detection**

```bash
# RustScan + Nmap workflow (2 tools, 2 phases)
rustscan -a TARGET -q > ports.txt
nmap -sV -sC -p $(cat ports.txt | grep -oP '\d+' | paste -sd,) TARGET

# ProRT-IP single-pass (1 tool, integrated)
prtip -sS -sV -p- TARGET -oA results
```

**4. Explore Database Features**

```bash
# Store results in database
prtip -sS -sV -p- 192.168.1.0/24 --with-db --database enterprise.db

# Query by service
prtip db query enterprise.db --service apache

# Query by port
prtip db query enterprise.db --port 22

# Compare scans for change detection
prtip db compare enterprise.db 1 2
```

**5. Integration Patterns**

```bash
# ProRT-IP in security pipeline
#!/bin/bash

# Phase 1: Rapid stateless discovery (6-10 seconds, RustScan-class speed)
prtip --stateless -p- 192.168.1.0/24 -oJ discovery.json

# Phase 2: Extract open ports
OPEN_PORTS=$(jq -r '.[] | select(.state=="Open") | .port' discovery.json | paste -sd,)

# Phase 3: Targeted stateful enumeration (2-5 minutes, integrated detection)
prtip -sS -sV -O -p $OPEN_PORTS 192.168.1.0/24 --with-db --database results.db

# Phase 4: Optional Nmap for NSE scripts
prtip -sS -sV -p $OPEN_PORTS 192.168.1.0/24 -- --script vuln
```

---

## Command Comparison

### Basic Scanning

| Task | RustScan | ProRT-IP |
|------|----------|----------|
| **SYN scan** | N/A (uses TCP Connect only) | `prtip -sS -p 80,443 192.168.1.1` |
| **TCP Connect** | `rustscan -a 192.168.1.1` (default) | `prtip -sT -p 80,443 192.168.1.1` |
| **All ports** | `rustscan -a 192.168.1.1` (default) | `prtip -sS -p- 192.168.1.1` |
| **Multiple ports** | `rustscan -a 192.168.1.1 -p 22,80,443` | `prtip -sS -p 22,80,443 192.168.1.1` |
| **Port ranges** | `rustscan -a 192.168.1.1 -r 1-1000` | `prtip -sS -p 1-1000 192.168.1.1` |
| **UDP scan** | `rustscan -a 192.168.1.1 --udp -p 53,161` | `prtip -sU -p 53,161 192.168.1.1` |
| **Target file** | `rustscan -a targets.txt` | `prtip -sS -p 80,443 -iL targets.txt` |
| **Exclude ports** | `rustscan -a 192.168.1.1 -e 22,3389` | `prtip -sS --exclude-ports 22,3389 192.168.1.1` |

### Performance Tuning

| Task | RustScan | ProRT-IP |
|------|----------|----------|
| **Aggressive speed** | `rustscan -a TARGET -b 65535 -t 500` | `prtip --stateless --max-rate 10000000 TARGET` |
| **Conservative** | `rustscan -a TARGET -b 500 -t 3000` | `prtip -sS -T2 TARGET` |
| **Timing template** | N/A (manual batch/timeout) | `prtip -sS -T4 TARGET` (T0-T5 profiles) |
| **Batch size** | `rustscan -a TARGET -b 10000` | Adaptive parallelism (CPU cores × workers) |
| **Timeout** | `rustscan -a TARGET -t 1500` | `prtip -sS --max-rtt-timeout 1500 TARGET` |
| **Rate limit** | N/A (batch size controls concurrency) | `prtip -sS --max-rate 100000 TARGET` |
| **Retry attempts** | `rustscan -a TARGET --tries 3` | `prtip -sS --max-retries 3 TARGET` |

### Detection and Enumeration

| Task | RustScan | ProRT-IP |
|------|----------|----------|
| **Service detection** | `rustscan -a TARGET -- -sV` | `prtip -sS -sV TARGET` |
| **OS fingerprinting** | `rustscan -a TARGET -- -O` | `prtip -sS -O TARGET` |
| **Aggressive scan** | `rustscan -a TARGET -- -A` | `prtip -sS -A TARGET` |
| **TLS certificates** | `rustscan -a TARGET -- --script ssl-cert` | `prtip -sS -sV -p 443 TARGET` (automatic) |
| **Version intensity** | `rustscan -a TARGET -- --version-intensity 9` | `prtip -sV --version-intensity 9 TARGET` |
| **Default scripts** | `rustscan -a TARGET -- -sC` | N/A (use Nmap integration) |
| **Vulnerability scan** | `rustscan -a TARGET -- --script vuln` | `prtip -sS -sV TARGET -- --script vuln` |

### Output Formats

| Task | RustScan | ProRT-IP |
|------|----------|----------|
| **JSON output** | `rustscan -a TARGET -g` (greppable only) | `prtip -sS -p 80,443 TARGET -oJ results.json` |
| **XML output** | N/A (Nmap integration only) | `prtip -sS -p 80,443 TARGET -oX results.xml` |
| **Normal text** | `rustscan -a TARGET` (default) | `prtip -sS -p 80,443 TARGET -oN results.txt` |
| **All formats** | N/A | `prtip -sS -p 80,443 TARGET -oA results` |
| **Database** | N/A | `prtip -sS -p 80,443 TARGET --with-db --database scan.db` |
| **Greppable** | `rustscan -a TARGET -g` | `prtip -sS -p 80,443 TARGET -oG results.gnmap` |
| **Quiet mode** | `rustscan -a TARGET -q` | `prtip -sS -p 80,443 TARGET -q` |

### Scripting and Customization

| Task | RustScan | ProRT-IP |
|------|----------|----------|
| **Python script** | RSE: `~/.rustscan_scripts.toml` + metadata | Lua plugin system: `~/.prtip/plugins/` |
| **Lua script** | RSE: Same as Python (multi-language) | `prtip --plugin custom-scan.lua TARGET` |
| **Shell script** | RSE: Same as Python (multi-language) | Lua integration or subprocess |
| **Nmap scripts** | `rustscan -a TARGET -- --script <script>` | `prtip -sS -sV TARGET -- --script <script>` |
| **Configuration** | `~/.rustscan.toml` (TOML format) | `~/.prtip/config.toml` + CLI flags |

---

## Integration Workflows

### RustScan Workflows

#### **Multi-Tool Security Pipeline**

Complete workflow combining RustScan's speed with comprehensive analysis:

```bash
#!/bin/bash
# Complete security pipeline: Discovery → Enumeration → Vulnerability Assessment

TARGET="192.168.1.0/24"
OUTPUT_DIR="security-assessment-$(date +%Y%m%d)"
mkdir -p $OUTPUT_DIR

echo "[*] Phase 1: RustScan port discovery (3-8 seconds per host)"
rustscan -a $TARGET -b 4000 -g > $OUTPUT_DIR/discovery.txt

echo "[*] Phase 2: Nmap service enumeration (30-60 seconds)"
HOSTS=$(cat $OUTPUT_DIR/discovery.txt | cut -d' ' -f1 | sort -u)
for host in $HOSTS; do
    PORTS=$(grep "^$host" $OUTPUT_DIR/discovery.txt | cut -d'[' -f2 | cut -d']' -f1)
    nmap -sV -sC -p $PORTS $host -oA $OUTPUT_DIR/nmap-$host
done

echo "[*] Phase 3: Nuclei vulnerability scanning (2-5 minutes)"
cat $OUTPUT_DIR/discovery.txt | grep ":80\|:443\|:8080\|:8443" | cut -d' ' -f1 | \
  nuclei -t http/ -severity critical,high -o $OUTPUT_DIR/nuclei-results.txt

echo "[*] Phase 4: Nikto web scanning (5-10 minutes per web server)"
cat $OUTPUT_DIR/discovery.txt | grep ":80\|:443\|:8080" | while read host_port; do
    HOST=$(echo $host_port | cut -d' ' -f1)
    PORT=$(echo $host_port | grep -oP '\d+')
    nikto -h $HOST -p $PORT -output $OUTPUT_DIR/nikto-$HOST-$PORT.txt
done

echo "[*] Complete! Total time: ~20 minutes (vs hours with traditional sequential approach)"
```

**Benefits**:
- Comprehensive vulnerability assessment in under 20 minutes
- Automated multi-tool orchestration
- Leverages each tool's strengths (RustScan speed, Nmap depth, Nuclei/Nikto vulnerabilities)

---

#### **CI/CD Security Scanning**

Automated infrastructure monitoring in continuous integration:

```yaml
# GitHub Actions workflow
name: Security Scan

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  workflow_dispatch:

jobs:
  rustscan-security:
    runs-on: ubuntu-latest
    steps:
      - name: Pull RustScan Docker image
        run: docker pull rustscan/rustscan:2.1.1

      - name: Scan infrastructure
        run: |
          docker run --rm rustscan/rustscan:2.1.1 \
            -a infrastructure.company.com \
            -b 4000 -g > scan-results.txt

      - name: Check for unexpected ports
        run: |
          if grep -qE ":(8080|3000|5000|6379)" scan-results.txt; then
            echo "::error::Unexpected development ports exposed"
            exit 1
          fi

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: rustscan-results
          path: scan-results.txt

      - name: Send alerts
        if: failure()
        run: |
          curl -X POST -H 'Content-type: application/json' \
            --data '{"text":"Security scan failed!"}' \
            ${{ secrets.SLACK_WEBHOOK }}
```

**Benefits**:
- Continuous security monitoring
- Automated alerting on unexpected ports
- Historical result tracking via artifacts

---

### ProRT-IP Workflows

#### **Single-Pass Comprehensive Assessment with Database**

Integrated detection eliminates multi-tool orchestration:

```bash
#!/bin/bash
# Comprehensive security assessment with historical tracking

DB="enterprise-security.db"
TARGET="192.168.1.0/24"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

echo "[*] Running comprehensive scan with integrated detection"
prtip -sS -sV -O -p- $TARGET \
  --with-db --database $DB \
  -oX scan-$TIMESTAMP.xml \
  -oJ scan-$TIMESTAMP.json \
  --progress

echo "[*] Querying high-risk services"
prtip db query $DB --service "telnet|ftp|rsh" --open

echo "[*] Comparing with previous scan for change detection"
SCAN1=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

prtip db compare $DB $SCAN1 $SCAN2 | tee changes-$TIMESTAMP.txt

echo "[*] Generating compliance report"
prtip db export $DB --scan-id $SCAN2 --format csv -o compliance-$TIMESTAMP.csv

echo "[*] Complete! Total time: ~15-30 minutes (single-pass with detection)"
```

**Benefits**:
- No multi-tool pipeline complexity
- Automatic change detection (new services, version updates, closed ports)
- Historical tracking for compliance audits
- Multiple output formats for different consumers

---

#### **Hybrid Approach (Stateless Discovery + Targeted Enumeration)**

Balance speed with comprehensive detection:

```bash
#!/bin/bash
# Hybrid workflow: Fast discovery → Targeted comprehensive enumeration

TARGET="192.168.1.0/24"
OUTPUT_DIR="hybrid-scan-$(date +%Y%m%d)"
mkdir -p $OUTPUT_DIR

echo "[*] Phase 1: Stateless rapid discovery (6-10 seconds, RustScan-class speed)"
prtip --stateless -p- $TARGET -oJ $OUTPUT_DIR/discovery.json --max-rate 10000000

echo "[*] Phase 2: Extract open ports"
OPEN_PORTS=$(jq -r '.[] | select(.state=="Open") | .port' $OUTPUT_DIR/discovery.json | \
  sort -n | uniq | paste -sd,)

echo "[*] Found open ports: $OPEN_PORTS"

echo "[*] Phase 3: Targeted stateful enumeration (2-5 minutes, integrated detection)"
prtip -sS -sV -O -p $OPEN_PORTS $TARGET \
  --with-db --database $OUTPUT_DIR/comprehensive.db \
  -oX $OUTPUT_DIR/enumeration.xml \
  --progress

echo "[*] Phase 4: Query results by service"
prtip db query $OUTPUT_DIR/comprehensive.db --service "http|https" > $OUTPUT_DIR/web-services.txt
prtip db query $OUTPUT_DIR/comprehensive.db --service "ssh" > $OUTPUT_DIR/ssh-services.txt

echo "[*] Complete! Total time: ~2-5 minutes (hybrid approach)"
```

**Benefits**:
- Combines RustScan-class discovery speed (6-10 seconds) with integrated detection (2-5 minutes)
- Single tool (no RustScan → Nmap transition)
- Database storage for queries and historical tracking
- Total time 2-5 minutes vs 15-30 minutes full stateful scan

---

#### **Real-Time TUI Monitoring**

Live visualization of scan progress and metrics:

```bash
# Launch interactive TUI dashboard (Sprint 6.2)
prtip --live -sS -sV -p- 192.168.1.0/24

# TUI Features:
# - Tab: Switch between Port Table / Service Table / Metrics / Network Graph
# - Arrow Keys: Navigate tables, scroll content
# - Enter: Select port/service for details
# - Esc: Return to previous view
# - Q: Quit TUI

# Performance:
# - 60 FPS rendering
# - <5ms frame time (16.67ms budget)
# - 10K+ events/sec throughput
# - Real-time metrics (throughput, progress, ETA)
# - Time-series network graph (60-second sliding window)
```

**Benefits**:
- Professional-grade monitoring interface
- Immediate visibility into scan operations
- Multiple view modes (Port Table, Service Table, Metrics Dashboard, Network Graph)
- Keyboard navigation and interactive filtering

---

#### **PCAPNG Forensic Capture**

Full packet capture for offline analysis:

```bash
#!/bin/bash
# Forensic evidence preservation

CASE_ID="incident-2025-01-15"
TARGET="compromised.server.com"
OUTPUT_DIR="evidence-$CASE_ID"
mkdir -p $OUTPUT_DIR

echo "[*] Capturing all packets during scan"
prtip -sS -sV -O -p- $TARGET \
  --pcapng $OUTPUT_DIR/scan-packets.pcapng \
  -oX $OUTPUT_DIR/scan-results.xml \
  --with-db --database $OUTPUT_DIR/evidence.db

echo "[*] Analyzing captured packets"
tshark -r $OUTPUT_DIR/scan-packets.pcapng -T fields \
  -e frame.number -e ip.src -e ip.dst -e tcp.srcport -e tcp.dstport -e tcp.flags \
  > $OUTPUT_DIR/packet-summary.txt

echo "[*] Extracting suspicious patterns"
tshark -r $OUTPUT_DIR/scan-packets.pcapng -Y "tcp.flags.syn==1 && tcp.flags.ack==0" \
  > $OUTPUT_DIR/syn-probes.txt

echo "[*] Creating evidence archive"
tar -czf $CASE_ID-evidence.tar.gz $OUTPUT_DIR/

echo "[*] Complete! Evidence preserved for forensic analysis"
```

**Benefits**:
- Complete packet capture for legal proceedings
- Offline analysis with standard tools (Wireshark, tshark)
- Evidence integrity and chain of custody
- Supports security incident response

---

## Summary and Recommendations

### Choose RustScan If:

✅ **CTF competitions where speed is paramount** (3-8 seconds for 65K ports)
✅ **Bug bounty initial reconnaissance** across large scopes (rapid service enumeration)
✅ **Automatic Nmap integration valuable** (seamless transition from discovery to enumeration)
✅ **Unprivileged execution required** (standard sockets, no root/sudo needed)
✅ **Single-host or small subnet scanning** (optimized for this use case)
✅ **Minimal resource overhead critical** (single-threaded design, 10MB binary, 50-100MB RAM)

### Choose ProRT-IP If:

✅ **Single-pass comprehensive assessment** required (service + OS + TLS in one tool)
✅ **Detection capabilities critical** (85-90% service accuracy, OS fingerprinting, TLS certificates)
✅ **Advanced scan types needed** (SYN, FIN, NULL, Xmas, ACK, UDP, Idle—8 total)
✅ **Database storage and historical tracking valuable** (SQLite, queries, change detection)
✅ **Cross-platform native executables** matter (Linux, macOS, Windows, FreeBSD—no Docker)
✅ **Real-time monitoring with TUI** (live dashboard, 60 FPS, interactive tables)

### Hybrid Approach

Many security professionals use **both tools appropriately**:

**Scenario 1: CTF Competition** (RustScan dominant)
1. RustScan rapid discovery (3-8 seconds)
2. Automatic Nmap enumeration (10-15 seconds on open ports)
3. Manual exploitation (time saved enables thorough testing)

**Scenario 2: Enterprise Security Assessment** (ProRT-IP dominant)
1. ProRT-IP stateless discovery (6-10 seconds, comparable to RustScan)
2. ProRT-IP stateful enumeration (2-5 minutes targeted, integrated detection)
3. ProRT-IP database queries and change detection (historical tracking)

**Scenario 3: Bug Bounty Reconnaissance** (Combined)
1. RustScan rapid web service discovery (seconds across large scopes)
2. ProRT-IP comprehensive assessment of discovered hosts (integrated TLS analysis)
3. ProRT-IP database storage for scope tracking (historical vulnerability correlation)

---

### Key Insights

**Architecture Philosophy**:
- RustScan: "Do one thing exceptionally well" (port discovery) → delegate enumeration to Nmap
- ProRT-IP: "Balance speed with integrated detection" (comparable stateless speed + comprehensive features)

**Speed Comparison**:
- RustScan: 3-8 seconds (single-threaded async-std, 4,500 concurrent connections)
- ProRT-IP: 6-10 seconds stateless (multi-threaded Tokio, 10M+ pps), 15-30 minutes stateful (integrated detection)
- Difference: 1.3-2.2x for pure discovery, but ProRT-IP eliminates Nmap dependency for most use cases

**Total Time for Comprehensive Assessment**:
- RustScan + Nmap: 3-8s (discovery) + 10-15s (Nmap enumeration) = **13-23 seconds** (few open ports)
- ProRT-IP stateful: **15-30 minutes** (single-pass comprehensive, all ports)
- ProRT-IP hybrid: 6-10s (stateless) + 2-5 min (targeted stateful) = **2-5 minutes** (balanced)

**Platform Considerations**:
- RustScan: Linux (native, best performance), macOS/Windows (Docker required due to ulimit/rlimit issues)
- ProRT-IP: Linux, macOS, Windows, FreeBSD (native executables, platform-optimized)

**Use Case Alignment**:
- RustScan: CTF competitions, bug bounties, penetration testing (time-sensitive scenarios)
- ProRT-IP: Enterprise security assessments, continuous monitoring, forensic analysis (comprehensive requirements)

**Community and Maturity**:
- RustScan: 18,200+ GitHub stars, 50+ contributors, active Discord (489 members), TryHackMe learning room
- ProRT-IP: New project (2024), Phase 5 complete (v0.5.0), production-ready (2,102 tests, 54.92% coverage)

Both tools leverage Rust's memory safety and zero-cost abstractions, making them reliable and performant alternatives to traditional C-based scanners. The choice depends on workflow priorities: pure speed with automatic Nmap integration (RustScan) or comprehensive single-tool assessment with integrated detection (ProRT-IP).

---

## See Also

- [ProRT-IP vs Nmap](./nmap.md) - Comparison with traditional comprehensive scanner
- [ProRT-IP vs Masscan](./masscan.md) - Comparison with Internet-scale scanner
- [ProRT-IP vs ZMap](./zmap.md) - Comparison with academic Internet measurement tool
- [ProRT-IP vs Naabu](./naabu.md) - Comparison with Project Discovery scanner
- [Scanner Comparison Overview](./overview.md) - All scanner comparisons in one place
- [CLI Reference](../user-guide/cli-reference.md) - Complete ProRT-IP command-line reference
- [Performance Characteristics](../../advanced/performance-characteristics.md) - Detailed benchmark data
