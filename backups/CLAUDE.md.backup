# CLAUDE.md

Guidance for Claude Code (claude.ai/code) working with ProRT-IP.

## Project Overview

**ProRT-IP WarScan**: Modern network scanner combining Masscan/ZMap speed (1M+ pps stateless) with Nmap detection depth.

**Status**: Phase 4 COMPLETE (Sprint 4.1-4.14) + v0.3.0 Production. 643 tests (100%), OS fingerprinting, service detection, 7 scan types, cyber-punk CLI, sendmmsg batching, CDN detection, decoy scanning, adaptive parallelism, 10 custom commands. CI/CD 7/7 passing, 9 targets (5 working). Zero TODOs/stubs/debt.

**Repository**: <https://github.com/doublegate/ProRT-IP>
**License**: GPL-3.0
**Updated**: 2025-10-11

## Architecture

### Hybrid Scanning Approach
1. **Fast Discovery**: Stateless scanning at Masscan speeds for rapid enumeration
2. **Deep Enumeration**: Stateful connections with Nmap-style service detection

Mirrors RustScan: scan all 65,535 ports ~3s, pipe to detailed analysis.

### Performance Targets
- **Stateless**: 10M+ pps async event-driven I/O
- **Internet-scale**: Full IPv4 sweep with adaptive rate control
- **Memory**: Stream to disk, avoid RAM accumulation

### Key Technical Decisions

**Async**: Tokio multi-threaded, worker threads = CPU cores (not hyperthreads), lock-free crossbeam, zero-copy >10KB payloads

**Packets**: Cross-platform raw sockets (Linux AF_PACKET/CAP_NET_RAW, Windows Npcap/Admin, macOS/BSD BPF/dev/bpf*), pnet crate

**Privileges**: Create raw sockets/captures elevated → drop immediately to unprivileged → never run scan logic as root

## Scanning Capabilities

### Multi-Protocol Support

**TCP**: SYN (half-open, default), Connect (fallback), FIN/NULL/Xmas (RFC 793 stealth), ACK (firewall state), Idle/zombie (anonymity)

**UDP**: Protocol payloads (DNS 53, SNMP 161, NetBIOS 137), ICMP unreachable interpretation, ~10-100x slower (ICMP rate limiting)

**Discovery**: ICMP echo/timestamp/netmask, TCP SYN/ACK pings, UDP common services, ARP local networks

### Detection Engines

**Service**: nmap-service-probes format, intensity 0-9, SSL/TLS handshake, NULL probes first

**OS Fingerprinting**: 16-probe sequence, weighted scoring vs nmap-os-db (2,600+), key discriminators (ISN, TCP timestamps, IP ID, TCP options)

### Stealth & Evasion

**Timing (T0-T5)**: T0 paranoid (5min delays), T2 polite (0.4s), T3 normal, T4 aggressive (production), T5 insane (max speed)

**Evasion**: Packet fragmentation (8-byte/custom MTU), decoy scanning (spoofed sources), source port trust (20/53/80/88), randomized order, idle/zombie (IP ID exploitation)

## Implementation Status

| Phase | Weeks | Status | Tests | Key Deliverables | Doc Ref |
|-------|-------|--------|-------|------------------|---------|
| 1: Core | 1-3 | ✅ COMPLETE | 215 | Packet capture, TCP connect, privilege mgmt, CLI | docs/10-PROJECT-STATUS.md |
| 2: Advanced | 4-6 | ✅ COMPLETE | 278 | SYN/UDP/stealth scans, timing, rate limiting, connection pool | docs/01-ROADMAP.md |
| 3: Detection | 7-10 | ✅ COMPLETE | 391 | OS fingerprinting, service detection, banner grabbing | Phase 3 complete |
| **4: Performance** | **11-13** | **✅ COMPLETE** | **643** | **Lock-free, sendmmsg, CDN detect, decoy, adaptive parallelism** | **Sprint 4.1-4.14** |
| 5: Advanced | 14-16 | PLANNED | - | Idle scanning, plugin system (Lua), fragmentation, TUI/GUI | Next phase |

**Enhancement Cycles 1-8**: COMPLETE (SipHash, Blackrock, resource limits, progress tracking, error categorization, batch sending, CDN/WAF detection, decoy scanning)

**Custom Commands**: 10 commands for development workflow automation (/rust-check, /bench-compare, /sprint-start, /sprint-complete, /perf-profile, /module-create, /doc-update, /test-quick, /ci-status, /bug-report)

## Critical Dependencies

```toml
tokio = "1.35"     # Async runtime
pnet = "0.34"      # Packet capture/manipulation
socket2 = "0.5"    # Raw socket API
clap = "4.4"       # CLI parsing
sqlx = "0.7"       # Async SQL
crossbeam = "0.8"  # Lock-free structures
governor = "0.6"   # Rate limiting
mlua = "0.9"       # Lua plugins (Phase 5)
```

**System Requirements**: Linux 4.15+ (libpcap 1.9+, setcap), Windows 10+ (Npcap 1.70+, Admin), macOS 11.0+ (ChmodBPF/root), Memory 4GB min (16GB recommended), SSD for results

**Build Profile**: opt-level=3, lto="fat", codegen-units=1, panic="abort"

## Security Requirements

### Input Validation
- `IpAddr::parse()` for IPs, `ipnetwork` for CIDR
- Allowlist at API boundaries
- **Never** construct shell commands from user input (use `std::process::Command`)

### Privilege Pattern
```rust
let socket = create_raw_socket()?;      // 1. Create privileged
drop_privileges("scanner", "scanner")?; // 2. Drop IMMEDIATELY
run_scan_engine(socket)?;               // 3. Run unprivileged
```

### Packet Parsing
- Use pnet/etherparse automatic bounds checking
- Return Option/Result (never panic on malformed packets)
- Validate data offset fields before indexing
- Resource limits: max concurrent, per-target rates, timeouts

### DoS Prevention
- Bound operations via `tokio::sync::Semaphore`
- Stream results to disk immediately
- Monitor FD usage and set limits
- Adaptive rate limiting prevents flooding

## Database Schema

```sql
CREATE TABLE scans (id INTEGER PRIMARY KEY, start_time TIMESTAMP, end_time TIMESTAMP, config_json TEXT);
CREATE TABLE scan_results (id INTEGER PRIMARY KEY, scan_id INTEGER REFERENCES scans(id), target_ip TEXT, port INTEGER, state TEXT, service TEXT, banner TEXT, response_time_ms INTEGER, timestamp TIMESTAMP);
CREATE INDEX idx_scan_id ON scan_results(scan_id);
CREATE INDEX idx_target_ip ON scan_results(target_ip);
CREATE INDEX idx_port ON scan_results(port);
```

**Performance**: SQLite WAL mode, batch inserts (1K-10K/transaction), PostgreSQL COPY (10-100x INSERT)

## CLI Design

**Binary**: `prtip`

**Examples**:
```bash
prtip -sS -p 1-1000 10.0.0.0/24                          # Basic SYN scan
prtip -sS -p 1-1000 -O -D RND:5 --output=json 10.0.0.0/24  # OS detect + 5 decoys + JSON
prtip -T4 -p- -sV 192.168.1.1                            # Aggressive timing, all ports, service detect
prtip -T2 -sF -p 80,443 --scan-delay 100ms target.com    # Stealth FIN scan with delays
```

**Output Formats**: Text (colorized), JSON (machine-readable), XML (nmap-compatible), Binary (masscan streaming), PCAPNG (packet capture), SQLite/PostgreSQL (direct export)

## Documentation Structure

### Root-Level Docs

| Document | Description | Use When |
|----------|-------------|----------|
| README.md (14KB) | Overview, quick start, navigation | First-time visitors |
| ROADMAP.md (8KB) | Phases, timelines, metrics | Project direction |
| CONTRIBUTING.md (10KB) | Guidelines, standards, PR process | Contributing code |
| SECURITY.md (9KB) | Vulnerability reporting, responsible use | Security researchers |
| SUPPORT.md (9KB) | Documentation index, quick starts | Getting help |
| AUTHORS.md (8KB) | Contributors, acknowledgments | Recognition |
| CHANGELOG.md | Version history, release notes | Tracking changes |

### Technical Docs (`docs/`)

| Document | Size | Description |
|----------|------|-------------|
| 00-ARCHITECTURE.md | 23KB | System design, patterns, components |
| 01-ROADMAP.md | 18KB | 8 phases, 20 weeks, 122+ tasks |
| 02-TECHNICAL-SPECS.md | 22KB | Protocol specs, packet formats |
| 03-DEV-SETUP.md | 14KB | Environment setup (Linux/Win/Mac) |
| 04-IMPLEMENTATION-GUIDE.md | 24KB | Code structure, 500+ lines examples |
| 05-API-REFERENCE.md | 20KB | 50+ APIs with examples |
| 06-TESTING.md | 17KB | 5 test levels, coverage targets |
| 07-PERFORMANCE.md | 17KB | Benchmarks, optimization |
| 08-SECURITY.md | 20KB | Implementation, audit checklist |
| 09-FAQ.md | 12KB | 30+ FAQs, troubleshooting |
| 10-PROJECT-STATUS.md | 19KB | Task tracking with checkboxes |

**Quick Start**: 1) 00-ARCHITECTURE, 2) 03-DEV-SETUP, 3) 10-PROJECT-STATUS, 4) 04-IMPLEMENTATION-GUIDE, 5) 08-SECURITY

### Reference Docs (`ref-docs/`)

- `ProRT-IP_Overview.md`: High-level blueprint
- `ProRT-IP_WarScan_Technical_Specification.md` (190KB): Comprehensive implementation
- `ProRT-IP_WarScan_Technical_Specification-v2.md` (36KB): Condensed guide

**Key Insights**: ZMap hit rates (97% @ 4Mpps, 63% @ 14.23Mpps), Tokio scheduler (10x via work-stealing), Nmap OS fingerprinting (2,600+ DB), Masscan stateless (SYN cookies, encrypted index)

### Local Memory (`CLAUDE.local.md`)

Living document: Current status, recent sessions, decisions, next actions, known issues, quick commands

## Important Notes

**Security Scope**: Defensive tool for penetration testing/red team. Must include: explicit user confirmation (internet-scale), audit logging, legal/ethical docs, rate limiting (prevent DoS)

**Performance**: Zero-copy >10KB payloads only, sendmmsg/recvmmsg largest gains @1M+ pps, NUMA penalties (10-30%) need IRQ affinity, lock contention bottleneck >1M pps

**Cross-Platform Caveats**: Windows Npcap init (90s network loss old versions, 0.993+ fixed), macOS ChmodBPF/root required, FIN/NULL/Xmas fail on Windows/Cisco (send RST always), UDP 10-100x slower (ICMP rate limiting, Linux 1 error/sec default)
