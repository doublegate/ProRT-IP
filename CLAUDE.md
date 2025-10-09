# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ProRT-IP WarScan** is a modern network scanner and "war dialer" for IP networks, implemented in **Rust**. The project aims to combine the speed of Masscan/ZMap (1M+ packets/second stateless) with the depth of Nmap's service detection and OS fingerprinting.

**Current Status:** Phase 3 COMPLETE (v0.3.0). All detection systems fully implemented with 391 passing tests (100% success rate). Complete OS fingerprinting, service detection, banner grabbing, 7 scan types, and professional cyber-punk CLI. Zero TODOs, stubs, or incomplete code. Production-ready with zero technical debt.

**Repository:** https://github.com/doublegate/ProRT-IP

**License:** GPL-3.0 (LICENSE file in repository)

**Last Updated:** 2025-10-08

## Architecture and Design Philosophy

### Hybrid Scanning Approach

The core architectural decision is a **two-phase hybrid model**:

1. **Fast Discovery Phase:** Stateless scanning at Masscan speeds for rapid port enumeration across large address spaces
2. **Deep Enumeration Phase:** Stateful connections with Nmap-style service detection and version identification

This mirrors RustScan's successful design: scan all 65,535 ports in ~3 seconds, then pipe discovered services to detailed analysis.

### Performance Targets

- **Stateless scanning:** 10+ million packets/second using asynchronous event-driven I/O
- **Internet-scale capability:** Full IPv4 sweep feasibility with adaptive rate control
- **Memory efficiency:** Stream results to disk; avoid accumulating large datasets in RAM

### Key Technical Decisions

**Async Runtime:** Tokio with multi-threaded scheduler
- Worker threads matching physical CPU cores (not hyperthreads)
- Lock-free coordination using crossbeam for scan state
- Zero-copy techniques where applicable (>10KB payloads only)

**Packet Handling:** Cross-platform raw socket abstraction
- Linux: AF_PACKET sockets with CAP_NET_RAW capability
- Windows: Npcap (requires Administrator privileges)
- macOS/BSD: BPF devices via /dev/bpf*
- Use `pnet` crate for cross-platform compatibility

**Privilege Management:** Immediate drop after resource creation
- Create raw sockets/capture handles with elevated privileges
- Drop to unprivileged user via setuid/setgid (Linux capabilities preferred)
- Never run scanning logic with root privileges

## Scanning Capabilities

### Multi-Protocol Support

**TCP Scanning:**
- SYN (half-open) - default stealth technique
- Connect scans (OS sockets fallback)
- FIN/NULL/Xmas stealth scans (exploit RFC 793 loopholes)
- ACK scans for firewall state differentiation
- Idle (zombie) scanning for complete anonymity

**UDP Scanning:**
- Protocol-specific payloads for DNS (53), SNMP (161), NetBIOS (137)
- ICMP port unreachable interpretation for closed port detection
- ~10-100x slower than TCP due to ICMP rate limiting

**Host Discovery:**
- ICMP echo/timestamp/netmask requests
- TCP SYN/ACK pings
- UDP pings to common services
- ARP on local networks

### Detection Engines

**Service Version Detection:**
- Use nmap-service-probes database format
- Intensity levels 0-9 (balance coverage vs speed)
- SSL/TLS handshake for encrypted services
- NULL probes first (many services self-announce)

**OS Fingerprinting:**
- 16-probe sequence exploiting TCP/IP stack implementation differences
- Weighted scoring system against nmap-os-db database
- Key discriminators: ISN patterns, TCP timestamps, IP ID generation, TCP options ordering

### Stealth and Evasion

**Timing Templates (T0-T5):**
- T0 (Paranoid): 5-minute probe delays
- T2 (Polite): 0.4-second delays for bandwidth reduction
- T3 (Normal): Default balanced behavior
- T4 (Aggressive): Fast/reliable networks (recommended for production)
- T5 (Insane): Maximum speed, sacrifices accuracy

**Evasion Techniques:**
- Packet fragmentation (8-byte or custom MTU)
- Decoy scanning (intermix real probes with spoofed sources)
- Source port manipulation (trust common ports: 20, 53, 80, 88)
- Randomized scan order to disrupt sequential detection
- Idle (zombie) scanning via predictable IP ID exploitation

## Implementation Roadmap

### Phase 1: Core Infrastructure (Weeks 1-3) - COMPLETE ✅
- Cross-platform packet capture using `pnet`
- Basic TCP connect scanning with Tokio async/await
- Privilege management (capabilities/setuid)
- Configuration loading (TOML via serde)
- Result storage (SQLite with indexing)
- 215 tests passing (100% success rate)
- Full CLI implementation with multiple output formats

### Phase 2: Advanced Scanning (Weeks 4-6) - COMPLETE ✅
- TCP SYN scanning with raw sockets
- UDP scanning with protocol-specific payloads (8 protocols: DNS, NTP, NetBIOS, SNMP, RPC, IKE, SSDP, mDNS)
- Stealth scan variants (FIN/NULL/Xmas/ACK)
- Timing templates (T0-T5 with RTT estimation)
- Adaptive rate limiting (Masscan-inspired, 256-bucket circular buffer)
- Connection pool optimization (RustScan FuturesUnordered pattern)

### Phase 3: Detection Systems (Weeks 7-10) - COMPLETE ✅
- OS fingerprinting (16-probe Nmap sequence with weighted scoring)
- Banner grabbing and application-level identification
- Service version detection (nmap-service-probes format parser)
- Protocol-specific banner extraction (HTTP, FTP, SSH, SMTP, DNS, SNMP)
- Progress reporting with real-time statistics (rate, ETA, JSON export)
- Error categorization with actionable suggestions (7 categories)
- Resource limits and interface detection
- Professional cyber-punk CLI banner

### Phase 4: Performance Optimization (Weeks 11-13)
- Lock-free data structures (crossbeam)
- Adaptive rate limiting based on response feedback
- sendmmsg/recvmmsg batching on Linux
- NUMA-aware thread placement and IRQ affinity
- Profiling with perf and flamegraph analysis

### Phase 5: Advanced Features (Weeks 14-16)
- Idle (zombie) scanning
- Decoy scanning with configurable placement
- Packet fragmentation support
- Plugin system (Lua via mlua)
- Audit logging and error recovery

## Critical Dependencies

**Core Crates:**
```toml
tokio = "1.35"           # Async runtime
pnet = "0.34"            # Packet capture/manipulation
socket2 = "0.5"          # Raw socket API
etherparse = "0.14"      # Zero-allocation parsing
clap = "4.4"             # CLI argument parsing
serde = "1.0"            # Serialization
sqlx = "0.7"             # Async SQL with compile-time checking
tracing = "0.1"          # Structured logging
crossbeam = "0.8"        # Lock-free data structures
governor = "0.6"         # Token bucket rate limiting
mlua = "0.9"             # Lua scripting (optional feature)
```

**System Requirements:**
- Linux: Kernel 4.15+, libpcap 1.9+, setcap for capabilities
- Windows: Windows 10+, Npcap 1.70+, Administrator privileges
- macOS: 11.0+, ChmodBPF or root for packet capture
- Memory: 4GB minimum, 16GB recommended for large scans
- Storage: SSD recommended for result database

## Build Configuration

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

[features]
default = ["async", "ssl"]
async = ["tokio/full"]
ssl = ["openssl"]
lua-plugins = ["mlua"]
```

**Development Build with Profiling:**
```bash
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release
perf record --call-graph dwarf -F 997 ./target/release/prtip
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

## Security Requirements

### Input Validation
- Use `IpAddr::parse()` for IP address validation
- Use `ipnetwork` crate for CIDR notation parsing
- Allowlist validation at API boundaries
- **Never** construct shell commands from user input - use `std::process::Command` directly

### Privilege Management Pattern
```rust
// 1. Create privileged resources (raw sockets, capture handles)
let socket = create_raw_socket()?;
let capture = open_pcap_handle()?;

// 2. Drop privileges IMMEDIATELY
drop_privileges("scanner", "scanner")?;

// 3. Run all scanning logic as unprivileged user
run_scan_engine(socket, capture)?;
```

### Packet Parsing Safety
- Use `pnet` or `etherparse` with automatic bounds checking
- Return `Option`/`Result` for parsing operations
- **Never** use `panic!` in packet parsing - malformed packets are expected
- Validate data offset fields before indexing
- Implement resource limits: max concurrent scans, per-target rate limits, scan duration timeouts

### DoS Prevention
- Bound concurrent operations via `tokio::sync::Semaphore`
- Stream results to disk immediately (don't accumulate in memory)
- Monitor file descriptor usage and set limits
- Implement adaptive rate limiting to prevent network flooding

## Database Schema

```sql
CREATE TABLE scans (
    id INTEGER PRIMARY KEY,
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    config_json TEXT
);

CREATE TABLE scan_results (
    id INTEGER PRIMARY KEY,
    scan_id INTEGER REFERENCES scans(id),
    target_ip TEXT,
    port INTEGER,
    state TEXT,
    service TEXT,
    banner TEXT,
    response_time_ms INTEGER,
    timestamp TIMESTAMP
);

CREATE INDEX idx_scan_id ON scan_results(scan_id);
CREATE INDEX idx_target_ip ON scan_results(target_ip);
CREATE INDEX idx_port ON scan_results(port);
```

**Performance Tips:**
- SQLite: Enable WAL mode (`PRAGMA journal_mode=WAL`)
- Batch inserts in transactions (1000-10000 per transaction)
- PostgreSQL: Use COPY for bulk loading (10-100x faster than INSERT)

## CLI Design

**Binary Name:** `prtip`

**Example Commands:**
```bash
# Basic SYN scan of common ports
prtip -sS -p 1-1000 10.0.0.0/24

# Full scan with OS detection, 5 random decoys, JSON output
prtip -sS -p 1-1000 -O -D RND:5 --output=json 10.0.0.0/24

# Aggressive timing, all ports, service detection
prtip -T4 -p- -sV 192.168.1.1

# Stealth scan with timing delays
prtip -T2 -sF -p 80,443 --scan-delay 100ms target.com
```

## Output Formats

- **Text:** Human-readable terminal output (colorized)
- **JSON:** Machine-readable structured data
- **XML:** Nmap-compatible schema for tool integration
- **Binary:** Masscan-style streaming format for large scans
- **PCAPNG:** Full packet capture with metadata
- **SQLite/PostgreSQL:** Direct database export

## Documentation Structure

### Root-Level Documentation

The project includes comprehensive GitHub community health files:

| Document | Description | Use When |
|----------|-------------|----------|
| **README.md** (14 KB) | Project overview, quick start, navigation | First-time visitors, project introduction |
| **ROADMAP.md** (8 KB) | Development phases, timelines, success metrics | Understanding project direction, planning |
| **CONTRIBUTING.md** (10 KB) | Contribution guidelines, code standards, PR process | Contributing code, submitting issues |
| **SECURITY.md** (9 KB) | Vulnerability reporting, responsible use, hardening | Security researchers, ethical usage |
| **SUPPORT.md** (9 KB) | Documentation index, quick starts, community channels | Getting help, finding resources |
| **AUTHORS.md** (8 KB) | Contributors, acknowledgments, Rust ecosystem credits | Recognition, attribution |
| **CHANGELOG.md** | Version history, release notes | Tracking changes, release information |

### Comprehensive Documentation Suite (`docs/`)

The project has complete technical documentation (237 KB across 12 documents):

| Document | Description | Use When |
|----------|-------------|----------|
| **00-ARCHITECTURE.md** (23 KB) | System architecture, design patterns, component overview | Understanding system design, planning features |
| **01-ROADMAP.md** (18 KB) | 8 phases, 20 weeks, 122+ tracked tasks | Sprint planning, timeline estimation |
| **02-TECHNICAL-SPECS.md** (22 KB) | Protocol specs, packet formats, data structures | Implementing networking code, packet parsing |
| **03-DEV-SETUP.md** (14 KB) | Environment setup for Linux/Windows/macOS | Initial setup, troubleshooting builds |
| **04-IMPLEMENTATION-GUIDE.md** (24 KB) | Code structure, 500+ lines of examples | Writing new modules, following patterns |
| **05-API-REFERENCE.md** (20 KB) | 50+ documented APIs with examples | Using scanner APIs, writing plugins |
| **06-TESTING.md** (17 KB) | Testing strategy, 5 test levels, coverage targets | Writing tests (TDD), setting up CI |
| **07-PERFORMANCE.md** (17 KB) | Benchmarks, optimization techniques | Profiling, optimizing hot paths |
| **08-SECURITY.md** (20 KB) | Security implementation, audit checklist | Privilege handling, input validation |
| **09-FAQ.md** (12 KB) | 30+ FAQs, troubleshooting | Common issues, user questions |
| **10-PROJECT-STATUS.md** (19 KB) | Task tracking with checkboxes | Finding next task, tracking progress |
| **README.md** (14 KB) | Documentation navigation guide | Finding relevant documentation |

**Quick Start for Development:**
1. Read `00-ARCHITECTURE.md` - understand the system design
2. Follow `03-DEV-SETUP.md` - set up environment
3. Check `10-PROJECT-STATUS.md` - find next task in current sprint
4. Consult `04-IMPLEMENTATION-GUIDE.md` - see code patterns and examples
5. Review `08-SECURITY.md` - ensure secure implementation

### Reference Documentation (`ref-docs/`)

Original technical specifications (241 KB total):
- `ProRT-IP_Overview.md`: High-level feature blueprint and project goals
- `ProRT-IP_WarScan_Technical_Specification.md` (190 KB): Comprehensive implementation details
- `ProRT-IP_WarScan_Technical_Specification-v2.md` (36 KB): Condensed technical guide

**Key Insights from References:**
- ZMap hit rates: 97% at 4Mpps, 63% at 14.23Mpps (network congestion limits)
- Tokio scheduler redesign: 10x performance improvement via work-stealing and LIFO slots
- Nmap's weighted OS fingerprinting: 2,600+ fingerprint database
- Masscan's stateless design: SYN cookies in source ports, encrypted index for randomization

### Local Memory Bank

**CLAUDE.local.md** - Living document tracking:
- Current development status and phase
- Recent session summaries
- Decision log with rationales
- Next immediate actions
- Known issues and blockers
- Quick reference commands

## Important Notes

**Security Scope:** This is a **defensive security tool** for penetration testing and red team operations. Implementation must include:
- Explicit user confirmation for internet-scale scans
- Audit logging of all scan activity
- Clear documentation of legal/ethical usage requirements
- Rate limiting to prevent unintentional DoS

**Performance Considerations:**
- Zero-copy benefits only apply to >10KB payloads (not typical 40-100 byte scan packets)
- sendmmsg/recvmmsg syscall batching provides largest performance gains at >1M pps
- NUMA penalties (10-30%) require IRQ affinity and thread pinning
- Lock contention becomes dominant bottleneck above 1M pps

**Cross-Platform Caveats:**
- Windows: Npcap initialization causes 90-second network loss on old versions (0.993+ fixed)
- macOS: Requires ChmodBPF launch daemon or root access
- FIN/NULL/Xmas scans fail on Windows and Cisco devices (send RST regardless of port state)
- UDP scanning 10-100x slower due to ICMP rate limiting (Linux default: 1 error/second)
