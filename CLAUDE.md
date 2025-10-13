# CLAUDE.md

Guidance for Claude Code (claude.ai/code) working with ProRT-IP.

## Project Overview

**ProRT-IP WarScan**: Modern network scanner combining Masscan/ZMap speed with Nmap detection depth.

**Status**: Phase 4 COMPLETE + **v0.3.7 Testing Infrastructure**. 789 tests (100%), 61.92% coverage (15,397/24,814 lines, exceeds 60% target), 67 integration tests, OS fingerprinting, service detection, 20+ nmap-compatible flags, 7 scan types, greppable output. CI/CD 7/7 passing, 8/8 release targets (100%).

**Repository**: <https://github.com/doublegate/ProRT-IP>
**License**: GPL-3.0
**Updated**: 2025-10-13

## Architecture

### Hybrid Scanning Approach

1. **Fast Discovery**: Stateless scanning at Masscan speeds
2. **Deep Enumeration**: Stateful connections with Nmap-style detection

**Performance:** 10M+ pps stateless, internet-scale IPv4 sweep, stream to disk

**Key Decisions:**
- **Async:** Tokio multi-threaded, CPU-core workers, lock-free crossbeam
- **Packets:** Cross-platform raw sockets (AF_PACKET/Npcap/BPF), pnet crate
- **Privileges:** Create elevated → drop immediately → run unprivileged

## Scanning Capabilities

**TCP**: SYN (default), Connect, FIN/NULL/Xmas (stealth), ACK (firewall), Idle (anonymity)
**UDP**: Protocol payloads (DNS, SNMP, NetBIOS), ICMP interpretation, ~10-100x slower
**Detection**: Service (nmap-service-probes, 187 probes), OS fingerprinting (16-probe, 2,600+ DB)
**Timing**: T0-T5 (paranoid→insane), packet fragmentation, decoy scanning, randomization

## Implementation Status

| Phase | Status | Tests | Key Features |
|-------|--------|-------|--------------|
| 1-3 | ✅ COMPLETE | 391 | Core scanning, protocols, detection |
| **4: Performance** | **✅ COMPLETE** | **789** | **Testing infra, lock-free, sendmmsg, CDN, adaptive** |
| 5: Advanced | PLANNED | - | Idle scan, Lua plugins, fragmentation, TUI/GUI |

**Custom Commands**: /rust-check, /bench-compare, /sprint-*, /perf-profile, /module-create, /doc-update, /test-quick, /ci-status, /bug-report

## Critical Dependencies

```toml
tokio = "1.35"     # Async runtime
pnet = "0.34"      # Packets
clap = "4.4"       # CLI
sqlx = "0.7"       # Async SQL
```

**System**: Linux 4.15+, Windows 10+, macOS 11.0+ | Memory 4GB min (16GB rec)
**Build**: opt-level=3, lto="fat", codegen-units=1

## Security Requirements

### Input Validation
- `IpAddr::parse()` for IPs, `ipnetwork` for CIDR
- Allowlist at API boundaries
- **Never** construct shell commands from user input

### Privilege Pattern
```rust
let socket = create_raw_socket()?;      // 1. Create privileged
drop_privileges("scanner", "scanner")?; // 2. Drop IMMEDIATELY
run_scan_engine(socket)?;               // 3. Run unprivileged
```

### Packet Parsing
- Use pnet/etherparse bounds checking (never panic)
- Return Option/Result on malformed packets
- Resource limits: max concurrent, per-target rates, timeouts

### DoS Prevention
- Bound via `tokio::sync::Semaphore`
- Stream results to disk immediately
- Adaptive rate limiting

## Database Schema

SQLite/PostgreSQL storage: scans (metadata), scan_results (port/service/banner). WAL mode, batch inserts (1K-10K/tx), indexes on scan_id/target_ip/port.

## CLI Design

**Binary**: `prtip`
**Nmap Compatibility**: 20+ flags (`-sS`, `-sT`, `-sU`, `-p`, `-F`, `-oN`, `-oX`, `-oG`, `-v`, `-A`)

```bash
prtip -sS -p 1-1000 10.0.0.0/24          # SYN scan
prtip -F 192.168.1.1                      # Fast (top 100)
prtip -A -p 80,443 target.com             # Aggressive
```

**Output Formats**: Text (colorized), JSON, XML (nmap), Greppable, PCAPNG, SQL

## Documentation Structure

**Root-Level**: README (overview), ROADMAP (phases), CONTRIBUTING (guidelines), SECURITY (reporting), SUPPORT (help index), AUTHORS, CHANGELOG

**Technical Docs** (`docs/`):
- 00-ARCHITECTURE: System design
- 01-ROADMAP: 8 phases, 20 weeks
- 03-DEV-SETUP: Environment setup
- 04-IMPLEMENTATION-GUIDE: Code structure
- 06-TESTING: 5 test levels, coverage
- 08-SECURITY: Audit checklist
- 10-PROJECT-STATUS: Task tracking

**Reference** (`ref-docs/`): Technical specifications (36KB-190KB comprehensive guides)

**Quick Start**: 00-ARCHITECTURE → 03-DEV-SETUP → 10-PROJECT-STATUS → 04-IMPLEMENTATION-GUIDE → 08-SECURITY

### Local Memory (`CLAUDE.local.md`)

Living document: Current status, recent sessions (5-7 days), decisions, next actions, quick commands

## Important Notes

**Security**: Penetration testing/red team tool. Include: user confirmation (internet-scale), audit logging, rate limiting (prevent DoS)

**Performance**: Zero-copy >10KB, sendmmsg/recvmmsg @1M+ pps, NUMA penalties need IRQ affinity

**Cross-Platform**: Windows Npcap init (90s loss old versions), macOS ChmodBPF/root, FIN/NULL/Xmas fail Windows/Cisco, UDP 10-100x slower (ICMP limiting)
