# ProRT-IP vs Naabu

Comprehensive technical comparison between ProRT-IP and Naabu, the Go-based port scanner by ProjectDiscovery that achieves 3-7x faster scanning than traditional tools through goroutine-based concurrency, automatic IP deduplication, and seamless integration with modern bug bounty reconnaissance workflows.

---

## Executive Summary

**Naabu transformed bug bounty reconnaissance** through intelligent engineering choices that prioritize workflow efficiency over raw speed. Built by ProjectDiscovery (a funded company with $25M Series A in 2021 and 100,000+ engineers), Naabu scans the **top 100 ports by default at 1000 packets per second** using either SYN scanning (with root privileges) or CONNECT scanning (without). What makes Naabu unique is not maximum speed—RustScan and Masscan outpace it in certain scenarios—but rather its **workflow optimizations**: automatic IP deduplication (reduces scan time by 80% on subdomain lists), built-in CDN/WAF detection, seamless ProjectDiscovery toolchain integration (Subfinder → Naabu → httpx → Nuclei), and clean handoff to Nmap for detailed service enumeration.

**ProRT-IP provides comparable speed with integrated detection**, achieving 10M+ pps stateless (exceeding Naabu's optimized 7000 pps) and 50K+ pps stateful with 85-90% service detection accuracy—eliminating the need for two-tool workflows in most scenarios.

**The fundamental difference**: Naabu optimizes for **bug bounty domain-based reconnaissance** with IP deduplication and ProjectDiscovery ecosystem integration, making it ideal for scanning hundreds of subdomains that resolve to shared infrastructure. ProRT-IP balances comparable stateless speed (10M+ pps) with **integrated comprehensive detection** (service + OS + TLS in single tool), eliminating Nmap dependency and providing database storage for historical tracking.

**Key Architecture Contrast**: Naabu's **Go goroutine model** (25 lightweight workers by default, configurable to 100+) with gopacket/libpcap packet handling optimizes for cloud VPS deployment and pipeline integration. ProRT-IP's **Tokio multi-threaded runtime** with adaptive parallelism enables comprehensive detection at high throughput. Naabu's microservices philosophy ("do one thing well, integrate cleanly") contrasts with ProRT-IP's single-pass comprehensive assessment model.

**Performance Reality**: Benchmarks show Naabu at default settings (1000 pps, 25 workers) completing scans in **28-32 seconds**, while optimized Naabu (7000 pps, 100 workers, 250ms timeout) achieves **10-11 seconds**. ProRT-IP stateless mode delivers **6-10 seconds** (comparable to optimized Naabu) with option for integrated stateful detection (**2-5 minutes** comprehensive single-pass vs Naabu+Nmap **13-23 seconds** two-phase when few ports open).

---

## Quick Comparison

| Dimension | Naabu | ProRT-IP |
|-----------|-------|----------|
| **First Released** | 2020 (ProjectDiscovery) | 2024 (Phase 1-5 complete) |
| **Language** | Go | Rust |
| **Speed (Top 100 Ports)** | 7-11 seconds (optimized 7000 pps) | 3-5 seconds stateless |
| **Speed (65K Ports)** | 10-11 seconds (optimized, discovery only) | 6-10 seconds stateless, 15-30 min comprehensive |
| **Detection Method** | None (requires Nmap integration) | Integrated (187 probes, 500+ services) |
| **Architecture** | Goroutines (25 default, 100+ configurable) | Tokio multi-threaded async |
| **Service Detection** | None (Nmap via `-nmap` flag) | 85-90% accuracy, version extraction, CPE |
| **OS Fingerprinting** | None (Nmap via `-nmap` flag) | Native (Nmap-compatible, 2,600+ signatures) |
| **Scan Types** | 3 (SYN, CONNECT, UDP) | 8 (SYN, Connect, FIN, NULL, Xmas, ACK, UDP, Idle) |
| **Primary Use Case** | Bug bounty reconnaissance, web app testing | Comprehensive security assessment |
| **Unique Feature** | IP deduplication (80% time reduction on subdomains) | Single-pass comprehensive (service+OS+TLS) |
| **CDN/WAF Detection** | Built-in (`-exclude-cdn` flag) | None |
| **Privileges Required** | Root for SYN (CONNECT fallback without) | Root/capabilities for raw sockets |
| **Default Behavior** | Top 100 ports, 1000 pps | All common ports, adaptive rate |
| **Concurrency Model** | Goroutines (lightweight threads) | Tokio work-stealing scheduler |
| **Memory Safety** | Go runtime garbage collection | Rust ownership model (zero-cost) |
| **Platform Support** | Linux, macOS (limited), Windows (Docker only) | Linux, macOS, Windows, FreeBSD (native) |
| **libpcap Dependency** | Required (gopacket wrapper) | Required (pnet wrapper) |
| **Rate Limiting** | Manual (`-rate` flag, 7000 pps optimal) | Adaptive (-1.8% overhead, burst management) |
| **IPv6 Support** | Yes (`-ip-version 6` or `4,6`) | 100% (all scan types) |
| **TLS Certificate** | None | X.509v3, SNI, chain validation, 1.33μs |
| **Database Storage** | JSON/CSV output only | SQLite, historical tracking, queries |
| **Scripting/Plugin** | None (delegate to Nmap NSE) | Lua 5.4 plugin system |
| **Output Formats** | Text, JSON (JSON Lines), CSV | Text, JSON, XML (Nmap), Greppable, PCAPNG |
| **Nmap Integration** | Seamless (`-nmap` flag, auto pipe) | Manual or database export to XML |
| **Metrics/Observability** | HTTP endpoint (localhost:63636) | Event system + TUI (60 FPS, 10K+ events/sec) |
| **GitHub Stars** | 4,900+ (as of Feb 2025) | New project |
| **Maturity** | Production (v2.3.3 stable, v2.3.4 regression) | Production (Phase 5 complete, v0.5.2) |
| **Community** | ProjectDiscovery ecosystem (100K+ engineers) | Growing |
| **Organization** | ProjectDiscovery (funded, $25M Series A) | Open source |

---

## When to Use Each Tool

### Use Naabu When:

✅ **Bug bounty reconnaissance with domain-based scoping** (IP deduplication 80% time reduction)
✅ **ProjectDiscovery workflow integration** (Subfinder → Naabu → httpx → Nuclei)
✅ **CDN/WAF-heavy environments** (automatic exclusion for Cloudflare/Akamai/Incapsula/Sucuri)
✅ **Pipeline automation with clean output** (silent mode, JSON Lines format)
✅ **Unprivileged execution acceptable** (CONNECT scan fallback without root)
✅ **Cloud VPS deployment** (lightweight, Docker support, metrics endpoint)

### Use ProRT-IP When:

✅ **Single-pass comprehensive assessment** required (service + OS + TLS in one tool)
✅ **Detection capabilities critical** (85-90% service accuracy, no Nmap dependency)
✅ **Advanced scan types needed** (8 types including stealth FIN/NULL/Xmas, Idle)
✅ **Database storage and historical tracking** valuable (SQLite queries, change detection)
✅ **Cross-platform native executables** matter (Windows/FreeBSD native, no Docker)
✅ **Real-time monitoring with TUI** (live dashboard, 60 FPS, interactive tables)
✅ **TLS certificate analysis** important (X.509v3, chain validation, SNI support)

---

## Speed Comparison

### Benchmark Results (Top 100 Ports - Bug Bounty Typical)

| Scanner | Mode | Configuration | Speed (pps) | Time | Ratio |
|---------|------|---------------|-------------|------|-------|
| **ProRT-IP** | Stateless | 10M+ pps maximum | 10M+ | **~3-5 seconds** | **1.0x baseline** |
| **Naabu** | Optimized | 7000 pps, 100 workers, 250ms timeout | 7,000 | **~10-11 seconds** | **2.2-3.7x slower** |
| **Naabu** | Default | 1000 pps, 25 workers | 1,000 | **~28-32 seconds** | **5.6-10.7x slower** |
| **ProRT-IP** | Stateful SYN (T4) | Integrated detection | 50K+ | **~2-5 minutes** | **24-100x slower but comprehensive** |

### Benchmark Results (All 65,535 Ports - Comprehensive Scan)

| Scanner | Mode | Configuration | Time | Detection | Notes |
|---------|------|---------------|------|-----------|-------|
| **ProRT-IP** | Stateless | 10M+ pps | **~6-10 seconds** | None | Discovery only |
| **Naabu** | Optimized | 7000 pps, 100 workers | **~10-11 seconds** | None | Discovery only, requires Nmap |
| **RustScan** | Default | 4500 batch, 1500ms timeout | **~8 seconds** | None | Discovery only, auto-Nmap |
| **Naabu** | Default | 1000 pps, 25 workers | **~488 seconds (8+ min)** | None | Unoptimized |
| **ProRT-IP** | Stateful SYN (T5) | Integrated detection | **~15-30 minutes** | 85-90% service, OS, TLS | Single-pass comprehensive |
| **Nmap** | Full (-p- -A -T5) | Integrated detection | **~17 minutes** | ~95% service, OS, scripts | Single-pass comprehensive |

### Naabu Configuration Impact

| Configuration | Rate (pps) | Workers | Timeout | Scan Time | Accuracy | Use Case |
|--------------|------------|---------|---------|-----------|----------|----------|
| **Default** | 1,000 | 25 | 2000ms | ~30 seconds | 100% | Conservative |
| **Recommended** | 7,000 | 100 | 250ms | ~10 seconds | 100% | **Optimal balance** |
| **Aggressive** | 10,000 | 100 | 100ms | ~7 seconds | 95% | High-bandwidth cloud |
| **Conservative** | 3,000 | 50 | 1000ms | ~18 seconds | 100% | IDS evasion |
| **Maximum** | 15,000 | 100 | 50ms | ~5 seconds | 80% | **Not recommended (packet loss)** |

**Strategic Insight**: Naabu's **optimal sweet spot is 7000 pps with 100 workers** (100% accuracy maintained). Above 8000 pps, packet loss degrades accuracy significantly. ProRT-IP's adaptive rate limiting (-1.8% overhead) automatically adjusts to network conditions without manual tuning.

### Total Time for Comprehensive Assessment

When **service detection and OS fingerprinting** are required goals:

| Workflow | Discovery Time | Enumeration Time | Total Time | Coverage |
|----------|----------------|------------------|------------|----------|
| **Naabu + Nmap (few ports)** | 10s | 3-13s | **13-23 seconds** | Service + OS via Nmap |
| **Naabu + Nmap (many ports)** | 10s | 5-15 min | **5-15 minutes** | Service + OS via Nmap |
| **ProRT-IP Stateless + Nmap** | 6-10s | 5-15 min | **5-15 minutes** | Service + OS via Nmap |
| **ProRT-IP Hybrid** | 6-10s | 2-5 min (targeted) | **2-5 minutes** | Service + OS + TLS integrated |
| **ProRT-IP Stateful** | N/A (single-pass) | N/A (single-pass) | **15-30 minutes** | Service + OS + TLS + PCAPNG comprehensive |
| **RustScan + Nmap** | 8s | 5-15 min | **5-15 minutes** | Service + OS via Nmap |

**Key Insight**: For **bug bounty rapid reconnaissance** with few open ports expected, Naabu+Nmap achieves **13-23 second total time** (optimal). For **comprehensive enterprise assessment**, ProRT-IP single-pass **15-30 minutes** provides service+OS+TLS+database+PCAPNG without tool switching. ProRT-IP hybrid approach (**2-5 minutes**) balances speed and depth.

---

## Detection Capabilities

### Service Version Detection

| Scanner | Capability | Method | Database | Detection Rate | Integration |
|---------|------------|--------|----------|----------------|-------------|
| **Naabu** | None (core) | N/A | N/A | N/A | Requires Nmap via `-nmap` flag |
| **Naabu Workflow** | Via Nmap | Signature matching | 1,000+ (Nmap DB) | ~95% | Two-phase (Naabu discovery → Nmap enumeration) |
| **ProRT-IP** | Integrated | Signature matching | 500+ (growing) | 85-90% | Single-pass (187 probes, version extraction, CPE) |

**Naabu Workflow Example**:
```bash
# Phase 1: Rapid port discovery with Naabu
naabu -host target.com -p - -verify -rate 7000 -silent -o ports.txt

# Phase 2: Service detection with Nmap
nmap -iL ports.txt -sV -sC -oA services
```

**ProRT-IP Workflow Example**:
```bash
# Single-pass comprehensive (no tool switching)
prtip -sS -sV -p- target.com -oJ results.json --with-db
```

### OS Fingerprinting

| Scanner | Capability | Method | Database | Accuracy | Requirements |
|---------|------------|--------|----------|----------|--------------|
| **Naabu** | None (core) | N/A | N/A | N/A | Requires Nmap |
| **Naabu + Nmap** | Full support (via Nmap) | 16-probe | 2,600+ | Comparable to Nmap | Two-phase workflow |
| **ProRT-IP** | Native support | 16-probe | 2,600+ (Nmap DB) | Comparable to Nmap | Integrated single-pass |

**Naabu OS Detection Example**:
```bash
# Naabu discovers ports, Nmap performs OS detection
naabu -host target.com -p - -verify -silent |
nmap -iL - -O -oA os-detection
```

**ProRT-IP OS Detection Example**:
```bash
# Integrated OS detection (no Nmap needed)
prtip -sS -O -p- target.com -oA scan-results
```

### TLS Certificate Analysis

| Scanner | Capability | Certificate Parsing | Chain Validation | SNI Support |
|---------|------------|---------------------|------------------|-------------|
| **Naabu** | None | N/A | N/A | N/A |
| **Naabu + Nmap** | Via Nmap scripts | Limited (ssl-cert NSE) | No | Limited |
| **ProRT-IP** | Native integrated | Full X.509v3 (1.33μs) | Yes | Yes |

**ProRT-IP TLS Example**:
```bash
# Integrated TLS certificate extraction
prtip -sS -sV --tls-cert -p 443,8443 target.com -oJ tls-results.json

# Results include: subject, issuer, validity, SANs, chain, algorithms
```

---

## Feature Comparison

### Scan Types

| Scan Type | Naabu | ProRT-IP | Notes |
|-----------|-------|----------|-------|
| **TCP SYN** | ✅ Default (with root) | ✅ Default | Half-open scanning, stealth |
| **TCP Connect** | ✅ Fallback (no root) | ✅ Available | Full three-way handshake |
| **TCP FIN** | ❌ Not supported | ✅ Supported | Stealth scan, bypasses some firewalls |
| **TCP NULL** | ❌ Not supported | ✅ Supported | Stealth scan, no flags set |
| **TCP Xmas** | ❌ Not supported | ✅ Supported | Stealth scan, FIN+PSH+URG flags |
| **TCP ACK** | ❌ Not supported | ✅ Supported | Firewall rule mapping |
| **TCP Window** | ❌ Not supported | ❌ Planned (Phase 7) | Advanced firewall mapping |
| **UDP** | ✅ Limited (`u:53` syntax) | ✅ Full support | Protocol payloads, ICMP interpretation |
| **Idle Scan** | ❌ Not supported | ✅ Supported | Maximum anonymity, zombie host |

### Advanced Features

| Feature | Naabu | ProRT-IP |
|---------|-------|----------|
| **Service Detection** | ❌ (requires Nmap) | ✅ 85-90% accuracy, 187 probes, CPE |
| **OS Fingerprinting** | ❌ (requires Nmap) | ✅ Nmap-compatible, 2,600+ signatures |
| **TLS Certificate** | ❌ (limited Nmap NSE) | ✅ X.509v3, SNI, chain validation |
| **IP Deduplication** | ✅ **Automatic (hash-based tracking)** | ❌ Not applicable (IP-based scanning) |
| **CDN/WAF Detection** | ✅ **Built-in (Cloudflare/Akamai/Incapsula/Sucuri)** | ❌ Not specialized |
| **Host Discovery** | ✅ ARP/ICMP/TCP/IPv6 neighbor | ✅ ICMP/ARP, configurable |
| **Rate Limiting** | Manual (`-rate` flag, 7000 pps optimal) | ✅ Adaptive (-1.8% overhead) |
| **Packet Fragmentation** | ❌ Not supported | ✅ `-f` flag, MTU control |
| **Decoy Scanning** | ❌ Not supported | ✅ `-D` flag, RND generation |
| **Source Port Spoofing** | ❌ Limited (platform-dependent) | ✅ `-g` flag |
| **TTL Manipulation** | ❌ Not supported | ✅ `--ttl` flag |
| **Timing Templates** | ❌ Manual rate/timeout | ✅ T0-T5 (paranoid → insane) |
| **Retry Logic** | ✅ 3 default attempts | ✅ Configurable (`--max-retries`) |
| **Database Storage** | ❌ JSON/CSV output only | ✅ SQLite, historical tracking, queries |
| **Real-Time TUI** | ❌ Metrics endpoint (localhost:63636) | ✅ Interactive dashboard (60 FPS, 4 tabs) |
| **PCAPNG Capture** | ❌ Not supported | ✅ Full packet capture for forensic analysis |
| **Resume Capability** | ❌ Not supported | ✅ `--resume` flag (SYN/Connect/UDP) |
| **Lua Plugins** | ❌ Not supported | ✅ Lua 5.4, sandboxing, capabilities |
| **Nmap Integration** | ✅ **Seamless (`-nmap` flag, auto pipe)** | Manual (database export to XML) |
| **ProjectDiscovery Integration** | ✅ **Native (Subfinder/httpx/Nuclei)** | ❌ Not applicable |

---

## Architecture Comparison

### Naabu's Architecture

**Language**: Go
**Core Design**: Goroutine-based concurrency with gopacket/libpcap packet handling and ProjectDiscovery ecosystem integration

**Key Innovations**:

1. **Goroutine-Based Concurrency** (25 lightweight workers by default, configurable to 100+)
   - Go's goroutines provide massive parallelism without memory overhead (unlike OS threads)
   - Each goroutine scans multiple ports/hosts simultaneously
   - Successful deployments run 100+ concurrent workers on cloud VPS instances

2. **Automatic IP Deduplication** (hash-based tracking, 80% time reduction)
   - Modern infrastructure: dozens of subdomains → shared IP addresses (CDN, load balancers, containers)
   - Naabu resolves all domains → maintains hash set → scans each unique IP once
   - Critical for bug bounty workflows with large subdomain lists

3. **CDN/WAF Detection and Exclusion** (`-exclude-cdn` flag)
   - Recognizes Cloudflare, Akamai, Incapsula, Sucuri infrastructure
   - Limits CDN IPs to ports 80/443 only (prevents hours of wasted scanning)
   - Prevents triggering rate limiting or security alerts from edge providers

4. **Metrics Endpoint** (localhost:63636 HTTP observability)
   - JSON metrics during scan execution for monitoring integration
   - Prometheus, Grafana, Datadog compatible
   - Tracks scan progress, port counts, error rates, performance characteristics

5. **ProjectDiscovery Ecosystem Integration** (microservices pattern)
   - Unix philosophy: focused tools with minimal overlap
   - Clean pipeline composition: Subfinder → Naabu → httpx → Nuclei
   - Silent mode strips informational messages for piping
   - JSON Lines output (one valid JSON object per line) for jq filtering

**Packet Handling**:
- gopacket library (Go wrapper around libpcap)
- SYN scans: manually build Ethernet/IP/TCP layers with checksums, transmit via raw sockets (AF_PACKET, SOCK_RAW on Linux)
- Response capture: libpcap with BPF rules (minimize kernel↔user context switches)
- Shared packet capture handlers globally (v2.3.0+) prevent resource leaks

**Strengths**:
- Workflow optimizations for bug bounty reconnaissance (IP deduplication, CDN awareness)
- Seamless ProjectDiscovery toolchain integration (standardized pipelines)
- Lightweight resource footprint (<100MB RAM at default settings)
- Excellent observability (metrics endpoint for monitoring stacks)
- Automatic privilege fallback (SYN → CONNECT gracefully)

**Weaknesses**:
- No service detection or OS fingerprinting (requires Nmap dependency)
- Limited scan types (SYN, CONNECT, UDP only—no FIN/NULL/Xmas/ACK/Idle)
- Platform constraints (Windows requires Docker, macOS limited by ulimit 255)
- Version stability issues (v2.3.4 regression: CPU <1%, scans hours instead of minutes)
- Manual rate tuning required (no adaptive rate limiting)

---

### ProRT-IP's Architecture

**Language**: Rust
**Core Design**: Hybrid stateful/stateless scanning with integrated comprehensive detection and event-driven architecture

**Key Innovations**:

1. **Tokio Multi-Threaded Async Runtime** (work-stealing scheduler, adaptive parallelism)
   - Distributes workload across CPU cores dynamically
   - Scales from embedded systems to NUMA servers
   - Zero-copy packet processing for >10KB payloads

2. **Hybrid Scanning Modes** (stateless 10M+ pps, stateful 50K+ pps with detection)
   - Stateless: Masscan-style speed for rapid discovery
   - Stateful: Comprehensive detection without Nmap dependency
   - User chooses tradeoff based on reconnaissance goals

3. **Integrated Detection Pipeline** (service 187 probes, OS 16-probe, TLS X.509v3)
   - Single-pass comprehensive assessment (no tool switching)
   - 85-90% service detection accuracy
   - Nmap-compatible OS fingerprinting (2,600+ signatures)
   - TLS certificate extraction (1.33μs parsing, chain validation, SNI)

4. **Event-Driven Architecture** (pub-sub system, -4.1% overhead, 18 event types)
   - 10K+ events/sec throughput
   - Real-time progress tracking for TUI
   - Event logging to SQLite (queries, replay capabilities)

5. **Rate Limiting V3** (-1.8% overhead, adaptive burst management)
   - Industry-leading efficiency (vs Nmap 10-20%, Masscan 5-10%)
   - Automatic adjustment to network conditions
   - Token bucket + leaky bucket hybrid algorithm

**Strengths**:
- Comprehensive detection in single tool (no Nmap dependency)
- 8 scan types (including stealth FIN/NULL/Xmas and Idle anonymity)
- Cross-platform native executables (Windows/FreeBSD/macOS/Linux, no Docker)
- Database storage with historical tracking and change detection
- Real-time TUI monitoring (60 FPS, 4 tabs: Port/Service/Metrics/Network)
- Memory safety (Rust ownership model, zero-cost abstractions)

**Weaknesses**:
- Stateless speed slightly slower than optimized Naabu (6-10s vs 10-11s)
- No IP deduplication feature (not workflow-optimized for subdomain lists)
- Requires elevated privileges for raw sockets (no CONNECT fallback to standard sockets)
- No CDN/WAF detection (not specialized for bug bounty workflows)

---

## Use Cases

### Naabu Use Cases

#### 1. **Bug Bounty Reconnaissance at Scale** (IP Deduplication Critical)

**Scenario**: Bug bounty program with 500+ subdomains resolving to ~50 unique IPs (shared CDN/load balancer infrastructure).

**Why Naabu**: IP deduplication reduces scan time **80%** (4 hours → 45 minutes) while maintaining identical coverage. CDN exclusion prevents wasting time on Cloudflare/Akamai edge servers.

```bash
# Comprehensive bug bounty reconnaissance pipeline
subfinder -d target.com -all -silent | \
dnsx -silent -resp-only | \
naabu -p - -verify -exclude-cdn -rate 7000 -c 100 -timeout 250 -silent | \
httpx -silent -title -tech-detect -screenshot | \
nuclei -t cves/,exposures/ -severity critical,high -json | \
jq -r 'select(.info.severity=="critical")' | \
notify -provider telegram

# Total time: ~30-60 minutes for comprehensive pipeline
# Without IP deduplication: ~3-5 hours for same coverage
```

**Key Benefits**:
- Automatic IP deduplication (hash-based tracking)
- CDN/WAF exclusion (Cloudflare/Akamai/Incapsula/Sucuri limited to 80/443)
- Seamless ProjectDiscovery integration (Subfinder → Naabu → httpx → Nuclei)
- Clean JSON Lines output for jq filtering and notify alerting

---

#### 2. **ProjectDiscovery Ecosystem Workflows** (Native Integration)

**Scenario**: DevSecOps team needs continuous security monitoring with standardized toolchain.

**Why Naabu**: Native integration with ProjectDiscovery tools (Subfinder, httpx, Nuclei, Notify, CloudList) creates standardized, reproducible workflows.

```bash
# Multi-cloud asset discovery and vulnerability scanning
cloudlist -providers aws,gcp,azure -silent | \
naabu -p 22,80,443,3306,5432,8080,8443 -verify -rate 5000 -silent | \
httpx -silent -title -tech-detect -status-code | \
nuclei -t cloud/,cves/ -severity critical,high -silent | \
notify -provider slack

# GitHub Actions scheduled CI/CD scan
name: Security Scan
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - run: |
          naabu -list production-hosts.txt -p - -verify -silent -json -o scan-$(date +%Y%m%d).json
          # Compare against baseline, alert on changes
```

**Key Benefits**:
- Standardized toolchain (bug bounty community consensus)
- Silent mode for clean piping
- JSON Lines format for jq processing
- Metrics endpoint (localhost:63636) for Prometheus/Grafana

---

#### 3. **Two-Phase Penetration Testing** (Rapid Discovery + Detailed Enumeration)

**Scenario**: Penetration testing engagement with 100-host scope, need to identify attack surface quickly before detailed enumeration.

**Why Naabu**: Completes initial discovery **60-70% faster** than Nmap-only workflows, allowing more time for exploitation and analysis.

```bash
# Phase 1: Rapid port discovery with Naabu (10-15 seconds per host)
naabu -list scope.txt -p - -verify -rate 7000 -c 100 -exclude-cdn -silent -o discovered-ports.txt

# Phase 2: Detailed enumeration with Nmap (targeted, 5-10 minutes)
nmap -iL discovered-ports.txt -sV -sC -O --script vuln -oA detailed-scan

# Total time: ~15 minutes discovery + ~10-30 minutes enumeration = ~25-45 minutes
# vs Nmap-only: ~60-90 minutes for equivalent coverage
```

**Key Benefits**:
- 3-5x faster port discovery than Nmap
- Automatic Nmap integration via `-nmap` flag (optional)
- Clean handoff with `host:port` format
- Verify flag (`-verify`) establishes full TCP connections to reduce false positives

---

#### 4. **VPS-Optimized Cloud Deployment** (Lightweight, Observable)

**Scenario**: Managed Security Service Provider (MSSP) needs continuous scanning from cloud VPS instances with minimal resource consumption.

**Why Naabu**: Lightweight footprint (<100MB RAM), Docker support, metrics endpoint for observability.

```bash
# Docker deployment with resource limits
docker run -it --rm \
  --cpus="2" --memory="200m" \
  -v $(pwd):/output \
  projectdiscovery/naabu:latest \
  -list /output/targets.txt -p - -verify -rate 7000 -json -o /output/scan.json

# Metrics monitoring (Prometheus integration)
curl http://localhost:63636/metrics
# Returns JSON: scan_progress, ports_checked, errors, throughput

# Kubernetes CronJob for scheduled scanning
apiVersion: batch/v1
kind: CronJob
metadata:
  name: naabu-scan
spec:
  schedule: "0 */6 * * *"  # Every 6 hours
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: naabu
            image: projectdiscovery/naabu:2.3.3  # Stable version (avoid 2.3.4 regression)
            args: ["-list", "/config/targets.txt", "-p", "-", "-verify", "-json"]
```

**Key Benefits**:
- Lightweight resource footprint (100MB RAM, 2 CPU cores sufficient)
- Docker support (consistent deployment across environments)
- Metrics endpoint (localhost:63636) for Prometheus/Grafana/Datadog
- No libpcap installation required (Docker image includes dependencies)

---

#### 5. **Network Reconnaissance with Conservative Settings** (IDS Evasion)

**Scenario**: Internal penetration testing in enterprise network with IDS/IPS monitoring, need to avoid triggering security alerts.

**Why Naabu**: Configurable rate limiting and timing parameters enable conservative scanning that evades detection.

```bash
# Conservative enterprise network scan (IDS evasion)
naabu -list internal-network.txt \
  -rate 500 \           # Low packet rate (vs 7000 aggressive)
  -c 25 \               # Moderate concurrency (vs 100 aggressive)
  -retries 5 \          # Multiple retry attempts
  -timeout 3000 \       # Long timeouts (3 seconds)
  -verify \             # Verify open ports (full TCP connections)
  -warm-up-time 5s \    # Gradual scan startup
  -json -o audit-$(date +%Y%m%d).json

# Host discovery only (no port scanning)
naabu -list internal-network.txt -sn -json -o active-hosts.json
```

**Key Benefits**:
- Granular rate control (500-1000 pps for stealth)
- Host discovery features (ARP ping for local subnets, TCP SYN ping for remote)
- Multiple retry attempts reduce scan noise
- Long timeouts accommodate slow networks and IDS rate limiting

---

### ProRT-IP Use Cases

#### 1. **Single-Pass Comprehensive Security Assessment** (No Tool Switching)

**Scenario**: Security audit requiring service detection, OS fingerprinting, and TLS certificate analysis—need complete results without managing multiple tool outputs.

**Why ProRT-IP**: Integrated detection eliminates Nmap dependency and provides service+OS+TLS in single execution with database storage.

```bash
# Comprehensive single-pass assessment
prtip -sS -sV -O --tls-cert -p- target.com \
  --with-db --database security-audit.db \
  -oJ results.json -oX nmap-format.xml

# Query results from database
prtip db query security-audit.db --target 192.168.1.100 --open
prtip db query security-audit.db --service apache
prtip db query security-audit.db --port 443  # TLS certificate details included

# Total time: 15-30 minutes
# vs Naabu+Nmap: 10s discovery + 15-30 min enumeration (similar, but two tools)
```

**Key Benefits**:
- Single tool execution (no pipeline management)
- Database storage with historical tracking
- Integrated TLS certificate extraction (X.509v3, chain validation, SNI)
- OS fingerprinting without Nmap dependency
- Multiple output formats simultaneously (JSON, XML, Greppable, Text)

---

#### 2. **Hybrid Approach for Rapid Comprehensive Reconnaissance** (Speed + Depth)

**Scenario**: Time-sensitive security assessment needing balance between rapid discovery and comprehensive detection.

**Why ProRT-IP**: Hybrid mode combines stateless discovery (6-10 seconds) with targeted stateful enumeration (2-5 minutes total).

```bash
# Phase 1: Stateless rapid discovery (6-10 seconds)
prtip --stateless -p- target.com -oG open-ports.gnmap

# Phase 2: Targeted stateful enumeration on discovered ports (2-5 minutes)
PORTS=$(grep -oP '\d+/open' open-ports.gnmap | cut -d'/' -f1 | paste -sd,)
prtip -sS -sV -O --tls-cert -p $PORTS target.com --with-db -oJ results.json

# Total time: 2-5 minutes comprehensive
# vs Naabu+Nmap: 13-23 seconds (few ports) or 5-15 minutes (many ports)
# vs RustScan+Nmap: 5-15 minutes (similar)
```

**Key Benefits**:
- Balances speed and comprehensiveness (2-5 min total)
- Stateless mode for rapid port discovery (comparable to Naabu/RustScan)
- Stateful mode with integrated detection (no Nmap dependency)
- Database storage for historical tracking and change detection

---

#### 3. **Advanced Scan Types for Firewall Mapping** (8 Scan Types Available)

**Scenario**: Network security assessment requiring firewall rule analysis and stealth reconnaissance.

**Why ProRT-IP**: 8 scan types (vs Naabu's 3) enable comprehensive firewall mapping and stealth techniques.

```bash
# Firewall rule mapping with multiple scan types

# 1. ACK scan to map firewall rules (stateful vs stateless detection)
prtip -sA -p 1-1000 target.com -oG firewall-acl.gnmap

# 2. Stealth FIN/NULL/Xmas scans bypass some firewalls
prtip -sF -p 80,443,8080,8443 target.com  # FIN scan
prtip -sN -p 80,443,8080,8443 target.com  # NULL scan
prtip -sX -p 80,443,8080,8443 target.com  # Xmas scan

# 3. Idle scan for maximum anonymity (zombie host required)
prtip --idle-scan zombie.host.com -p- target.com -oJ idle-results.json

# 4. UDP scan with protocol payloads
prtip -sU -p 53,161,123,500 target.com  # DNS, SNMP, NTP, IKE
```

**Key Benefits**:
- 8 scan types vs Naabu's 3 (SYN/CONNECT/UDP only)
- Stealth scans (FIN/NULL/Xmas) bypass some stateless firewalls
- ACK scan for firewall rule mapping
- Idle scan for maximum anonymity (no packets from attacker IP)
- UDP scanning with protocol-specific payloads

---

#### 4. **Real-Time Monitoring with TUI Dashboard** (Live Visualization)

**Scenario**: Large-scale network scan requiring real-time progress monitoring and interactive result exploration.

**Why ProRT-IP**: Interactive TUI with 60 FPS rendering, 4 tabs (Port/Service/Metrics/Network), live updates.

```bash
# Launch real-time TUI for interactive scanning
prtip --live -sS -sV -p- 192.168.1.0/24 --with-db --database live-scan.db

# TUI Features:
# - Tab 1 (Port Table): Interactive port list with sorting (port/state/service)
# - Tab 2 (Service Table): Service detection results with version/CPE
# - Tab 3 (Metrics Dashboard): Real-time throughput, progress, ETA
# - Tab 4 (Network Graph): Time-series chart (60-second sliding window)
#
# Keyboard Navigation:
# - Tab/Shift+Tab: Switch between tabs
# - Up/Down: Navigate tables
# - s: Sort by service, p: Sort by port
# - q: Quit TUI, Ctrl+C: Abort scan

# Query results after scan completes
prtip db list live-scan.db
prtip db query live-scan.db --scan-id 1 --open
```

**Key Benefits**:
- 60 FPS rendering with <5ms frame time (responsive UI)
- 10K+ events/sec throughput (real-time updates)
- 4-tab dashboard system (Port/Service/Metrics/Network)
- Interactive tables with sorting and filtering
- Event-driven architecture (-4.1% overhead)

---

#### 5. **PCAPNG Forensic Capture for Evidence Preservation** (Offline Analysis)

**Scenario**: Security incident investigation requiring full packet capture for forensic analysis and legal evidence.

**Why ProRT-IP**: PCAPNG packet capture with offline analysis capabilities.

```bash
# Capture all packets during scan for forensic analysis
prtip -sS -sV -p- target.com --pcapng scan-evidence.pcapng -oJ metadata.json

# Offline analysis with Wireshark/tshark
wireshark scan-evidence.pcapng  # GUI analysis
tshark -r scan-evidence.pcapng -Y "tcp.flags.syn==1 && tcp.flags.ack==1" | head -20

# Extract specific protocol conversations
tshark -r scan-evidence.pcapng -Y "http" -T fields -e http.request.uri
tshark -r scan-evidence.pcapng -Y "ssl.handshake.type == 1" -T fields -e ssl.handshake.extensions_server_name

# Timeline reconstruction
tshark -r scan-evidence.pcapng -T fields -e frame.time -e ip.src -e tcp.dstport | sort
```

**Key Benefits**:
- Full packet capture for forensic analysis
- Offline analysis with Wireshark/tshark (no need to rescan)
- Legal evidence preservation (immutable PCAPNG format)
- Protocol-specific filtering and extraction
- Timeline reconstruction for incident response

---

## Migration Guide

### Migrating from Naabu to ProRT-IP

#### What You Gain

**Integrated Detection** (eliminate Nmap dependency for most use cases)
- Service version detection (85-90% accuracy, 187 probes, CPE identifiers)
- OS fingerprinting (Nmap-compatible, 2,600+ signatures, 16-probe sequence)
- TLS certificate analysis (X.509v3, chain validation, SNI support, 1.33μs parsing)

**Advanced Scan Types** (8 types vs Naabu's 3)
- Stealth scans (FIN, NULL, Xmas) bypass some stateless firewalls
- ACK scan for firewall rule mapping
- Idle scan for maximum anonymity
- Full UDP support with protocol payloads

**Database Storage** (historical tracking and queries)
- SQLite storage with comprehensive indexes
- Change detection between scans (compare scan results)
- Queries by scan ID, target, port, service
- Export to JSON/CSV/XML/text

**Cross-Platform Native Executables** (no Docker requirement)
- Windows native support (vs Docker-only for Naabu)
- FreeBSD support
- macOS native (no ulimit 255 constraint)

**Real-Time Monitoring** (TUI dashboard)
- 60 FPS rendering, 4 tabs (Port/Service/Metrics/Network)
- Interactive tables with sorting and filtering
- Event-driven architecture with 10K+ events/sec throughput

**Memory Safety** (both tools benefit, but ProRT-IP adds production features)
- Rust ownership model (compile-time guarantees)
- Zero-cost abstractions
- No garbage collection pauses

#### What You Keep

**High-Speed Port Discovery** (comparable stateless performance)
- ProRT-IP stateless: 10M+ pps (exceeds Naabu's 7000 pps optimal)
- ProRT-IP stateful: 50K+ pps with integrated detection
- Both tools fast enough for practical reconnaissance

**Memory Safety** (both Rust and Go provide memory safety)
- Naabu: Go runtime garbage collection
- ProRT-IP: Rust ownership model (zero-cost)

**Minimal Memory Footprint** (stateless mode negligible overhead)
- Both tools efficient for rapid port discovery
- ProRT-IP stateless: ~4MB + ports × 1.0 KB
- Naabu: <100MB RAM at default settings

#### What Changes

**Speed Trade-off** (slightly slower stateless discovery, but integrated detection option)
- Naabu optimized: 10-11 seconds (65K ports, discovery only)
- ProRT-IP stateless: 6-10 seconds (65K ports, discovery only)
- ProRT-IP stateful: 15-30 minutes (65K ports, comprehensive single-pass)
- Total time with detection: Naabu+Nmap 13-23s (few ports) vs ProRT-IP hybrid 2-5 min

**Workflow Methodology** (single tool vs microservices pipeline)
- Naabu: Specialized for bug bounty workflows (IP deduplication, CDN exclusion, ProjectDiscovery integration)
- ProRT-IP: Single-pass comprehensive assessment (service+OS+TLS in one tool)
- Choose based on use case: bug bounty (Naabu) vs enterprise assessment (ProRT-IP)

**Privilege Requirements** (both require root for SYN, but Naabu has graceful fallback)
- Naabu: Automatic fallback to CONNECT scan without root
- ProRT-IP: Requires root/capabilities for raw sockets (no CONNECT fallback)
- Both support unprivileged TCP CONNECT scanning (`-sT` for ProRT-IP)

**IP Deduplication** (Naabu feature not in ProRT-IP)
- Naabu: Automatic IP deduplication (80% time reduction on subdomain lists)
- ProRT-IP: Not workflow-optimized for subdomain scanning (IP-based scanning)
- Workaround: Pre-process subdomain lists with dnsx, deduplicate IPs manually

**CDN/WAF Detection** (Naabu specialized feature)
- Naabu: Built-in CDN/WAF exclusion (Cloudflare/Akamai/Incapsula/Sucuri)
- ProRT-IP: No CDN-specific features (general-purpose scanner)

#### Migration Steps

**Step 1: Assess Your Workflow**

Determine if you benefit from Naabu's specialized features:
- **Bug bounty with subdomain lists**: Keep Naabu for IP deduplication
- **Comprehensive security assessment**: Migrate to ProRT-IP for single-pass
- **Hybrid approach**: Use both tools appropriately

**Step 2: Adapt Reconnaissance Scripts**

```bash
# Naabu reconnaissance pipeline
subfinder -d target.com -silent | \
naabu -p - -verify -exclude-cdn -rate 7000 -silent | \
httpx -silent | \
nuclei -t cves/

# ProRT-IP equivalent (if migrating away from ProjectDiscovery)
# (Note: ProRT-IP not optimized for this workflow—Naabu better choice)
subfinder -d target.com -silent | \
dnsx -silent -resp-only | \
sort -u > ips.txt  # Manual IP deduplication
prtip -sS -sV -iL ips.txt -p 80,443,8080,8443 --with-db -oJ results.json
jq -r 'select(.state=="open") | "\(.ip):\(.port)"' results.json | \
httpx -silent | \
nuclei -t cves/
```

**Recommendation**: For bug bounty workflows with subdomain lists, **keep using Naabu** (specialized IP deduplication and CDN exclusion features).

**Step 3: Migrate Comprehensive Assessments**

```bash
# Naabu + Nmap workflow (two tools)
naabu -host target.com -p - -verify -rate 7000 -silent -o ports.txt
nmap -iL ports.txt -sV -sC -O -oA detailed-scan

# ProRT-IP equivalent (single tool)
prtip -sS -sV -O --tls-cert -p- target.com --with-db -oJ results.json -oX nmap-format.xml
```

**Step 4: Adapt CI/CD Pipelines**

```yaml
# GitHub Actions: Naabu security scan
- name: Port Scan
  run: |
    naabu -list production-hosts.txt -p - -verify -silent -json -o scan.json

# GitHub Actions: ProRT-IP equivalent
- name: Port Scan
  run: |
    prtip -sS -sV -iL production-hosts.txt --with-db --database scan.db -oJ scan.json
    prtip db compare scan.db 1 2  # Compare against baseline
```

**Step 5: Database Integration**

```bash
# ProRT-IP database capabilities (not available in Naabu)

# Store results in SQLite
prtip -sS -sV -p- target.com --with-db --database security-audit.db

# Query by target
prtip db query security-audit.db --target 192.168.1.100 --open

# Query by service
prtip db query security-audit.db --service apache

# Compare scans for change detection
prtip db compare security-audit.db 1 2

# Export to multiple formats
prtip db export security-audit.db --scan-id 1 --format json -o export.json
prtip db export security-audit.db --scan-id 1 --format xml -o nmap-format.xml
```

---

## Command Comparison

### Basic Scanning

| Task | Naabu | ProRT-IP |
|------|-------|----------|
| **Scan default ports** | `naabu -host target.com` | `prtip -sS target.com` |
| **Scan specific port** | `naabu -host target.com -p 80` | `prtip -sS -p 80 target.com` |
| **Scan port range** | `naabu -host target.com -p 1-1000` | `prtip -sS -p 1-1000 target.com` |
| **Scan all ports** | `naabu -host target.com -p -` | `prtip -sS -p- target.com` |
| **Scan multiple hosts** | `naabu -list hosts.txt -p -` | `prtip -sS -p- -iL hosts.txt` |
| **Scan top 100 ports** | `naabu -host target.com` (default) | `prtip -sS --top-ports 100 target.com` |
| **Scan with verification** | `naabu -host target.com -verify` | `prtip -sS -p- target.com` (integrated) |
| **Unprivileged scan** | `naabu -host target.com` (auto fallback) | `prtip -sT -p- target.com` |

### Performance Tuning

| Task | Naabu | ProRT-IP |
|------|-------|----------|
| **Aggressive timing** | `naabu -rate 7000 -c 100 -timeout 250` | `prtip -sS -T5 -p- target.com` |
| **Conservative timing** | `naabu -rate 500 -c 25 -timeout 3000` | `prtip -sS -T1 -p- target.com` |
| **Custom packet rate** | `naabu -rate 5000` | `prtip --max-rate 50000` |
| **Increase concurrency** | `naabu -c 100` | (Adaptive parallelism automatic) |
| **Custom timeout** | `naabu -timeout 2000` (milliseconds) | `prtip --max-rtt-timeout 2000` |
| **Retry attempts** | `naabu -retries 5` | `prtip --max-retries 5` |
| **Disable host discovery** | `naabu -Pn` | `prtip -Pn` |

### Detection and Enumeration

| Task | Naabu | ProRT-IP |
|------|-------|----------|
| **Service detection** | `naabu -nmap-cli 'nmap -sV'` (via Nmap) | `prtip -sS -sV -p- target.com` |
| **OS fingerprinting** | `naabu -nmap-cli 'nmap -O'` (via Nmap) | `prtip -sS -O -p- target.com` |
| **TLS certificate** | Not supported | `prtip -sS -sV --tls-cert -p 443,8443 target.com` |
| **Aggressive detection** | `naabu -nmap-cli 'nmap -A'` (via Nmap) | `prtip -sS -A -p- target.com` |
| **Version intensity** | `naabu -nmap-cli 'nmap --version-intensity 9'` | `prtip -sV --version-intensity 9 target.com` |
| **Stealth scan** | Not supported (SYN only) | `prtip -sF -p- target.com` (FIN/NULL/Xmas) |
| **Idle scan** | Not supported | `prtip --idle-scan zombie.host.com -p- target.com` |

### Output Formats

| Task | Naabu | ProRT-IP |
|------|-------|----------|
| **Normal output** | `naabu -host target.com` (default stdout) | `prtip -sS -p- target.com` (default stdout) |
| **JSON output** | `naabu -json -o results.json` | `prtip -sS -p- -oJ results.json target.com` |
| **CSV output** | `naabu -csv -o results.csv` | `prtip db export scan.db --format csv -o results.csv` |
| **XML output** | `naabu -nmap-cli 'nmap -oX results.xml'` | `prtip -sS -p- -oX results.xml target.com` |
| **Silent mode** | `naabu -silent` | `prtip -sS -p- target.com > /dev/null 2>&1` |
| **Multiple formats** | (Requires multiple runs) | `prtip -sS -p- -oA results target.com` (all formats) |
| **Database storage** | Not supported (JSON/CSV only) | `prtip --with-db --database scan.db target.com` |

### Bug Bounty Workflows

| Task | Naabu | ProRT-IP |
|------|-------|----------|
| **IP deduplication** | `naabu -list domains.txt -p -` (automatic) | (Manual dnsx + sort -u required) |
| **CDN exclusion** | `naabu -exclude-cdn` | (No CDN-specific features) |
| **Subdomain pipeline** | `subfinder \| naabu \| httpx \| nuclei` | (Not workflow-optimized) |
| **Metrics endpoint** | `curl http://localhost:63636/metrics` | `prtip --live` (TUI with real-time metrics) |
| **JSON Lines output** | `naabu -json` (one JSON per line) | `prtip -oJ` (standard JSON array) |

---

## Integration Workflows

### Naabu Workflows

#### Multi-Tool Bug Bounty Pipeline (ProjectDiscovery Ecosystem)

```bash
#!/bin/bash
# Comprehensive bug bounty reconnaissance pipeline
# Phase 1: Asset Discovery → Phase 2: Port Scanning → Phase 3: HTTP Probing → Phase 4: Vulnerability Scanning

TARGET="target.com"
OUTPUT_DIR="recon-$(date +%Y%m%d)"
mkdir -p $OUTPUT_DIR

# Phase 1: Subdomain enumeration (Subfinder)
echo "[*] Phase 1: Subdomain enumeration..."
subfinder -d $TARGET -all -silent > $OUTPUT_DIR/subdomains.txt

# DNS resolution and deduplication (dnsx)
cat $OUTPUT_DIR/subdomains.txt | \
dnsx -silent -resp-only | \
sort -u > $OUTPUT_DIR/ips.txt

# Phase 2: Port scanning with IP deduplication (Naabu)
echo "[*] Phase 2: Port scanning (Naabu with IP deduplication)..."
naabu -list $OUTPUT_DIR/subdomains.txt \
  -p - \                      # All ports
  -verify \                   # Verify open ports (full TCP connections)
  -exclude-cdn \              # Skip Cloudflare/Akamai/Incapsula/Sucuri
  -rate 7000 \                # Optimal balance (100% accuracy)
  -c 100 \                    # 100 concurrent workers
  -timeout 250 \              # 250ms timeout
  -retries 3 \                # 3 retry attempts
  -silent \                   # Clean output for piping
  -json -o $OUTPUT_DIR/ports.json

# Phase 3: HTTP service probing (httpx)
echo "[*] Phase 3: HTTP service probing..."
cat $OUTPUT_DIR/ports.json | \
jq -r '"\(.ip):\(.port)"' | \
httpx -silent \
  -title \                    # Extract page titles
  -tech-detect \              # Detect technologies (Wappalyzer)
  -status-code \              # HTTP status codes
  -screenshot \               # Take screenshots
  -json -o $OUTPUT_DIR/http.json

# Phase 4: Vulnerability scanning (Nuclei)
echo "[*] Phase 4: Vulnerability scanning..."
cat $OUTPUT_DIR/http.json | \
jq -r '.url' | \
nuclei -t cves/,exposures/,vulnerabilities/ \
  -severity critical,high \
  -silent \
  -json -o $OUTPUT_DIR/vulns.json

# Alert on critical findings (Notify)
cat $OUTPUT_DIR/vulns.json | \
jq -r 'select(.info.severity=="critical")' | \
notify -provider telegram

echo "[+] Pipeline complete! Total time: ~30-60 minutes"
echo "[+] Results: $OUTPUT_DIR/"
```

**Time Breakdown**:
- Subfinder: 5-10 minutes
- Naabu: 10-20 minutes (IP deduplication saves hours)
- httpx: 5-15 minutes
- Nuclei: 10-20 minutes
- **Total: ~30-60 minutes** for comprehensive pipeline

---

#### CI/CD Security Scanning (GitHub Actions)

```yaml
name: Naabu Security Scan
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
  workflow_dispatch:

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Setup Naabu
        run: |
          go install -v github.com/projectdiscovery/naabu/v2/cmd/naabu@v2.3.3
          # Use v2.3.3 (stable) - avoid v2.3.4 regression

      - name: Port Scan
        run: |
          naabu -list production-hosts.txt \
            -p - \
            -verify \
            -rate 5000 \
            -c 100 \
            -silent \
            -json -o scan-$(date +%Y%m%d).json

      - name: Compare with Baseline
        run: |
          # Compare current scan with yesterday's baseline
          PREV=$(ls scan-*.json | tail -2 | head -1)
          CURR=$(ls scan-*.json | tail -1)
          diff <(jq -S . $PREV) <(jq -S . $CURR) > changes.txt || true

      - name: Alert on Changes
        if: success()
        run: |
          if [ -s changes.txt ]; then
            echo "New services detected!" | notify -provider slack
            cat changes.txt | notify -provider slack
          fi

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: scan-results
          path: scan-*.json
```

**Key Benefits**:
- Daily automated scanning
- Baseline comparison for change detection
- Slack/Discord/Telegram alerting via Notify
- Artifact retention for compliance/auditing

---

### ProRT-IP Workflows

#### Single-Pass Comprehensive Assessment with Database

```bash
#!/bin/bash
# ProRT-IP comprehensive security assessment with database storage

TARGET="target.com"
OUTPUT_DIR="assessment-$(date +%Y%m%d)"
mkdir -p $OUTPUT_DIR

# Single-pass comprehensive scan (service + OS + TLS + database)
prtip -sS -sV -O --tls-cert -p- $TARGET \
  --with-db --database $OUTPUT_DIR/scan.db \
  -oJ $OUTPUT_DIR/results.json \
  -oX $OUTPUT_DIR/nmap-format.xml \
  -oG $OUTPUT_DIR/greppable.gnmap \
  --pcapng $OUTPUT_DIR/packets.pcapng

# Query results from database
echo "[*] Open ports:"
prtip db query $OUTPUT_DIR/scan.db --target $TARGET --open

echo "[*] Services detected:"
prtip db query $OUTPUT_DIR/scan.db --service apache
prtip db query $OUTPUT_DIR/scan.db --service nginx

echo "[*] TLS certificates:"
prtip db query $OUTPUT_DIR/scan.db --port 443

# Export to CSV for reporting
prtip db export $OUTPUT_DIR/scan.db --scan-id 1 --format csv -o $OUTPUT_DIR/report.csv

# Compare with previous scan for change detection
PREV_DB=$(ls assessment-*/scan.db | tail -2 | head -1)
if [ -f "$PREV_DB" ]; then
  echo "[*] Changes since last scan:"
  prtip db compare $PREV_DB $OUTPUT_DIR/scan.db
fi

echo "[+] Assessment complete! Total time: 15-30 minutes"
echo "[+] Results: $OUTPUT_DIR/"
```

**Key Benefits**:
- Single tool execution (no pipeline management)
- Database storage with historical tracking
- Multiple output formats simultaneously
- Change detection between scans
- Full packet capture for forensic analysis

---

#### Hybrid Approach (Rapid Discovery + Targeted Enumeration)

```bash
#!/bin/bash
# ProRT-IP hybrid workflow: stateless discovery + targeted stateful enumeration

TARGET="192.168.1.0/24"
OUTPUT_DIR="hybrid-scan-$(date +%Y%m%d)"
mkdir -p $OUTPUT_DIR

# Phase 1: Stateless rapid discovery (6-10 seconds for /24)
echo "[*] Phase 1: Stateless port discovery..."
prtip --stateless -p- $TARGET -oG $OUTPUT_DIR/open-ports.gnmap

# Extract discovered ports
PORTS=$(grep -oP '\d+/open' $OUTPUT_DIR/open-ports.gnmap | cut -d'/' -f1 | sort -u | paste -sd,)
echo "[+] Discovered ports: $PORTS"

# Phase 2: Targeted stateful enumeration (2-5 minutes)
echo "[*] Phase 2: Stateful enumeration on discovered ports..."
prtip -sS -sV -O --tls-cert -p $PORTS $TARGET \
  --with-db --database $OUTPUT_DIR/scan.db \
  -oJ $OUTPUT_DIR/results.json \
  -oX $OUTPUT_DIR/nmap-format.xml

# Query interesting results
echo "[*] High-risk services:"
prtip db query $OUTPUT_DIR/scan.db --service telnet
prtip db query $OUTPUT_DIR/scan.db --service ftp
prtip db query $OUTPUT_DIR/scan.db --port 3389  # RDP

echo "[+] Hybrid scan complete! Total time: 2-5 minutes"
echo "[+] Phase 1 (discovery): 6-10 seconds"
echo "[+] Phase 2 (enumeration): 2-5 minutes"
```

**Key Benefits**:
- Balances speed and comprehensiveness (2-5 min total)
- Stateless mode comparable to Naabu/RustScan (6-10 seconds)
- Stateful mode with integrated detection (no Nmap dependency)
- Database storage for historical tracking

---

#### Real-Time TUI Monitoring with Live Dashboard

```bash
#!/bin/bash
# ProRT-IP real-time TUI monitoring for interactive scanning

TARGET="10.0.0.0/16"  # Large network scan
DATABASE="live-scan-$(date +%Y%m%d).db"

# Launch interactive TUI with live dashboard
prtip --live -sS -sV -p- $TARGET --with-db --database $DATABASE

# TUI features during scan:
#
# Tab 1 (Port Table):
#   - Interactive port list with sorting (port/state/service)
#   - Up/Down: Navigate table
#   - s: Sort by service, p: Sort by port
#   - f: Filter by open/closed/filtered
#
# Tab 2 (Service Table):
#   - Service detection results with version/CPE
#   - Sorting by service name, version, product
#   - Color-coded severity (green=safe, yellow=caution, red=critical)
#
# Tab 3 (Metrics Dashboard):
#   - Real-time throughput (packets per second, 5-second average)
#   - Progress indicator (% complete, ports scanned, ETA)
#   - Statistics (total ports, open/closed/filtered counts)
#
# Tab 4 (Network Graph):
#   - Time-series chart (60-second sliding window)
#   - Throughput over time, packet loss rates
#   - Color-coded status (green=healthy, yellow=degraded, red=issues)
#
# Keyboard Navigation:
#   - Tab/Shift+Tab: Switch between tabs
#   - q: Quit TUI (scan continues in background)
#   - Ctrl+C: Abort scan
#   - s/p/f: Sorting and filtering (context-dependent)

# After scan completes, query results from database
echo "[*] Scan complete! Querying results from database..."
prtip db list $DATABASE
prtip db query $DATABASE --scan-id 1 --open

# Export for reporting
prtip db export $DATABASE --scan-id 1 --format json -o results.json
prtip db export $DATABASE --scan-id 1 --format csv -o report.csv
```

**Key Benefits**:
- 60 FPS rendering with <5ms frame time (responsive UI)
- 4-tab dashboard system (Port/Service/Metrics/Network)
- Interactive tables with sorting and filtering
- Real-time progress monitoring (throughput, ETA, statistics)
- Event-driven architecture with 10K+ events/sec throughput

---

#### PCAPNG Forensic Capture for Evidence Preservation

```bash
#!/bin/bash
# ProRT-IP forensic packet capture for incident investigation

TARGET="compromised-server.example.com"
EVIDENCE_DIR="incident-$(date +%Y%m%d-%H%M%S)"
mkdir -p $EVIDENCE_DIR

# Full packet capture during scan
echo "[*] Starting forensic scan with full packet capture..."
prtip -sS -sV -O --tls-cert -p- $TARGET \
  --pcapng $EVIDENCE_DIR/packets.pcapng \
  -oJ $EVIDENCE_DIR/metadata.json \
  --with-db --database $EVIDENCE_DIR/scan.db

# Calculate checksums for evidence integrity
sha256sum $EVIDENCE_DIR/packets.pcapng > $EVIDENCE_DIR/checksums.txt
sha256sum $EVIDENCE_DIR/metadata.json >> $EVIDENCE_DIR/checksums.txt

# Offline analysis with tshark/Wireshark
echo "[*] Extracting protocol conversations..."

# TCP conversations
tshark -r $EVIDENCE_DIR/packets.pcapng -z conv,tcp -q > $EVIDENCE_DIR/tcp-conversations.txt

# HTTP requests
tshark -r $EVIDENCE_DIR/packets.pcapng -Y "http" -T fields -e http.request.uri > $EVIDENCE_DIR/http-requests.txt

# TLS certificates
tshark -r $EVIDENCE_DIR/packets.pcapng -Y "ssl.handshake.type == 1" \
  -T fields -e ssl.handshake.extensions_server_name > $EVIDENCE_DIR/tls-sni.txt

# Timeline reconstruction
tshark -r $EVIDENCE_DIR/packets.pcapng -T fields -e frame.time -e ip.src -e tcp.dstport | \
  sort > $EVIDENCE_DIR/timeline.txt

# Create evidence package
tar -czf $EVIDENCE_DIR.tar.gz $EVIDENCE_DIR/
sha256sum $EVIDENCE_DIR.tar.gz > $EVIDENCE_DIR.tar.gz.sha256

echo "[+] Forensic capture complete!"
echo "[+] Evidence package: $EVIDENCE_DIR.tar.gz"
echo "[+] Checksums: $EVIDENCE_DIR.tar.gz.sha256"
```

**Key Benefits**:
- Full packet capture for forensic analysis
- Immutable PCAPNG format for legal evidence
- Offline analysis with Wireshark/tshark (no need to rescan)
- Protocol-specific filtering and extraction
- Timeline reconstruction for incident response
- Checksum verification for evidence integrity

---

## Summary and Recommendations

### Choose Naabu If:

✅ **Bug bounty reconnaissance with domain-based scoping** (IP deduplication 80% time reduction on subdomain lists)
✅ **ProjectDiscovery workflow integration** (standardized Subfinder → Naabu → httpx → Nuclei pipeline)
✅ **CDN/WAF-heavy environments** (automatic exclusion for Cloudflare/Akamai/Incapsula/Sucuri)
✅ **Pipeline automation with clean output** (silent mode, JSON Lines format for jq filtering)
✅ **Unprivileged execution acceptable** (CONNECT scan fallback without root privileges)
✅ **Cloud VPS deployment** (lightweight <100MB RAM, Docker support, metrics endpoint)
✅ **Microservices philosophy** (focused tools with minimal overlap, clean integration)

### Choose ProRT-IP If:

✅ **Single-pass comprehensive assessment** required (service + OS + TLS in one tool without Nmap dependency)
✅ **Detection capabilities critical** (85-90% service accuracy, 187 probes, version extraction, CPE identifiers)
✅ **Advanced scan types needed** (8 types including stealth FIN/NULL/Xmas and Idle anonymity)
✅ **Database storage and historical tracking** valuable (SQLite queries, change detection between scans)
✅ **Cross-platform native executables** matter (Windows/FreeBSD/macOS/Linux native, no Docker requirement)
✅ **Real-time monitoring with TUI** (interactive dashboard, 60 FPS, 4 tabs: Port/Service/Metrics/Network)
✅ **TLS certificate analysis** important (X.509v3, chain validation, SNI support, 1.33μs parsing)
✅ **PCAPNG packet capture** for forensic analysis (full packet capture, offline analysis, legal evidence)

### Hybrid Approach

Many security professionals use **both tools appropriately** based on reconnaissance context:

**Scenario 1: Bug Bounty with Large Subdomain List**
- **Use Naabu** for IP deduplication (80% time reduction) and CDN exclusion
- ProjectDiscovery pipeline: Subfinder → Naabu → httpx → Nuclei
- Total time: ~30-60 minutes for comprehensive pipeline

**Scenario 2: Enterprise Security Assessment**
- **Use ProRT-IP** for single-pass comprehensive assessment
- Integrated detection eliminates Nmap dependency
- Database storage for historical tracking and change detection
- Total time: 15-30 minutes for service+OS+TLS+database

**Scenario 3: Penetration Testing Engagement**
- **Phase 1**: Naabu rapid discovery (10-15 seconds) or ProRT-IP stateless (6-10 seconds)
- **Phase 2**: ProRT-IP targeted stateful enumeration (2-5 minutes comprehensive)
- Total time: ~2-5 minutes for balanced speed and depth

### Key Insights

**Architecture Philosophy**:
- **Naabu**: "Microservices pattern—do one thing exceptionally well, integrate cleanly"
- **ProRT-IP**: "Single-pass comprehensive assessment—balance speed with integrated detection"

**Speed Comparison**:
- **Naabu optimized**: 10-11 seconds (65K ports, discovery only, 7000 pps, 100 workers)
- **ProRT-IP stateless**: 6-10 seconds (65K ports, discovery only, 10M+ pps)
- **ProRT-IP stateful**: 15-30 minutes (65K ports, comprehensive single-pass)

**Total Time for Comprehensive Assessment**:
- **Naabu + Nmap**: 13-23 seconds (few open ports) or 5-15 minutes (many ports)
- **ProRT-IP stateful**: 15-30 minutes (single-pass comprehensive)
- **ProRT-IP hybrid**: 2-5 minutes (rapid discovery + targeted enumeration)

**Platform Considerations**:
- **Naabu**: Linux native, macOS limited (ulimit 255), Windows Docker-only
- **ProRT-IP**: Cross-platform native (Linux/macOS/Windows/FreeBSD, no Docker)

**Use Case Alignment**:
- **Naabu**: Bug bounty reconnaissance (IP deduplication, CDN exclusion, ProjectDiscovery integration)
- **ProRT-IP**: Comprehensive security assessment (service+OS+TLS, database, TUI, PCAPNG)

**Community and Maturity**:
- **Naabu**: ProjectDiscovery ecosystem (100K+ engineers, $25M Series A), 4,900+ GitHub stars, production (v2.3.3 stable)
- **ProRT-IP**: New project, growing community, production (Phase 5 complete v0.5.2)

---

## See Also

- [Overview: ProRT-IP vs Network Scanners](./overview.md) - Comparison landscape and decision framework
- [ProRT-IP vs Nmap](./nmap.md) - Traditional comprehensive scanner comparison
- [ProRT-IP vs Masscan](./masscan.md) - Internet-scale speed comparison
- [ProRT-IP vs ZMap](./zmap.md) - Research-oriented scanner comparison
- [ProRT-IP vs RustScan](./rustscan.md) - Modern rapid scanner comparison
- [User Guide](../../user-guide/index.md) - Step-by-step usage instructions
- [Platform Support](../../features/platform-support.md) - Installation and platform-specific guidance
- [Database Storage](../../features/database-storage.md) - Historical tracking and queries
- [Performance Tuning](../../advanced/performance-tuning.md) - Optimization strategies
