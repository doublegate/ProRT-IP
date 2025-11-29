# CLAUDE.md

ProRT-IP project guidance for Claude Code.

## Project Overview

**ProRT-IP WarScan**: Network scanner combining Masscan/ZMap speed with Nmap detection depth.

**Status**: Phase 6 COMPLETE (v0.5.9, 2,557 tests, 51.40% coverage, 8 scan types, production-ready TUI)

**Repository**: <https://github.com/doublegate/ProRT-IP> | **License**: GPL-3.0 | **Updated**: 2025-11-28

## Architecture

**Hybrid Approach**: Fast stateless discovery (Masscan speed) + Deep stateful enumeration (Nmap detection)
**Performance**: 10M+ pps stateless, internet-scale IPv4/IPv6, stream to disk

**Key Decisions**:
- **Async**: Tokio multi-threaded, CPU-core workers, lock-free crossbeam
- **Packets**: Cross-platform raw sockets (AF_PACKET/Npcap/BPF), pnet crate
- **Privileges**: Create elevated → drop immediately → run unprivileged

## Capabilities

| Category | Details |
|----------|---------|
| **TCP** | SYN (default), Connect, FIN/NULL/Xmas (stealth), ACK (firewall), Idle (anonymity) |
| **UDP** | Protocol payloads (DNS, SNMP, NetBIOS), ICMP interpretation, ~10-100x slower |
| **Detection** | Service (187 probes, 85-90%), OS fingerprinting (16-probe, 2,600+ DB) |
| **Timing** | T0-T5 (paranoid→insane), packet fragmentation, decoy scanning |
| **TUI** | 60 FPS, 4-tab dashboard, 10K+ events/sec, real-time metrics |

## Implementation Status

| Phase | Status | Tests | Key Features |
|-------|--------|-------|--------------|
| 1-3 | ✅ | 391 | Core scanning, protocols, detection |
| 4 | ✅ | 1,166 | Zero-copy, NUMA, PCAPNG, evasion, IPv6 foundation |
| 5 | ✅ | 868 | IPv6 100%, Idle scan, Service detection, Rate limiting -1.8%, TLS |
| **6** | **✅** | **2,557** | **TUI (60 FPS), Dashboard, CDN filtering, Network optimizations, Buffer Pool, Interactive Widgets, Event Flow Fixes** |

## Critical Dependencies

```toml
tokio = "1.35"     # Async runtime
pnet = "0.34"      # Packets
clap = "4.4"       # CLI
sqlx = "0.7"       # Async SQL
ratatui = "0.29"   # TUI
```

**System**: Linux 4.15+, Windows 10+, macOS 11.0+ | Memory 4GB min (16GB rec)

## Security Requirements

**Input Validation**: `IpAddr::parse()` for IPs, `ipnetwork` for CIDR, allowlist at boundaries
**Privilege Pattern**: Create elevated → drop immediately → run unprivileged
**Packet Parsing**: pnet/etherparse bounds checking, return Option/Result on malformed
**DoS Prevention**: `tokio::sync::Semaphore` bounds, stream to disk, adaptive rate limiting

## CLI Design

**Binary**: `prtip` | **Nmap Compatibility**: 50+ flags (`-sS`, `-sT`, `-sU`, `-p`, `-F`, `-oN`, `-oX`, `-oG`, `-v`, `-A`, `-f`, `--mtu`, `--ttl`, `-D`, `--badsum`, `-g`, etc.)

```bash
prtip -sS -p 1-1000 10.0.0.0/24          # SYN scan
prtip -F 192.168.1.1                      # Fast (top 100)
prtip -A -p 80,443 target.com             # Aggressive
```

**Output**: Text (colorized), JSON, XML (nmap), Greppable, PCAPNG, SQL

## Documentation Structure

**Root**: README, ROADMAP, CONTRIBUTING, SECURITY, SUPPORT, AUTHORS, CHANGELOG

**Technical Docs** (`docs/`):
- 00-ARCHITECTURE: System design
- 01-ROADMAP: 8 phases, 20 weeks
- 03-DEV-SETUP: Environment
- 04-IMPLEMENTATION-GUIDE: Code structure
- 06-TESTING: 5 test levels, coverage
- 08-SECURITY: Audit checklist
- 10-PROJECT-STATUS: Task tracking
- TUI-ARCHITECTURE: Event-driven TUI

**Reference** (`ref-docs/`): Technical specifications (36KB-190KB guides)

**Quick Start**: 00-ARCHITECTURE → 03-DEV-SETUP → 10-PROJECT-STATUS → 04-IMPLEMENTATION-GUIDE

## Important Notes

**Security**: Penetration testing/red team tool. User confirmation (internet-scale), audit logging, rate limiting (prevent DoS)
**Performance**: Zero-copy >10KB, sendmmsg/recvmmsg @1M+ pps, NUMA penalties need IRQ affinity
**Cross-Platform**: Windows Npcap init (90s loss old versions), macOS ChmodBPF/root, FIN/NULL/Xmas fail Windows/Cisco

## Release Standards

**Tag Messages** (100-150 lines): Executive summary, features, performance metrics, technical details, files changed, testing, strategic value
**GitHub Releases** (150-200 lines): All tag content + links, installation, platform matrix, known issues, asset downloads
**Process**: Read /tmp/ProRT-IP/RELEASE-NOTES-v*.md → Read SPRINT-*-COMPLETE.md → Review commits → Create tag → Create release → Verify → Push
**Reference**: v0.3.7-v0.3.9, v0.4.0-v0.4.5 (extensive, technically detailed)

## Maintenance

- Update CLAUDE.local.md after sessions | Sync 10-PROJECT-STATUS.md | Update CHANGELOG per release
- cargo fmt + clippy before commits | Maintain >60% coverage (>90% core) | Document public APIs
- Review 08-SECURITY.md before releases | Weekly cargo audit | Fuzz input validation

## Historical Decisions

Architectural decisions before Oct 2025. See CLAUDE.local.md "Recent Decisions" for last 30 days.

| Date | Decision | Rationale |
|------|----------|-----------|
| 10-23 | Raw response capture opt-in | Memory safety by default (--capture-raw-responses flag) |
| 10-14 | Extensive release notes | Quality standard: 100-200 lines, technical depth |
| 10-13 | Document Windows loopback failures | 4 SYN discovery tests (expected behavior) |
| 10-07 | Rate limiter burst=10 | Balance responsiveness + courtesy |
| 10-07 | Test timeouts 5s | CI variability, prevent false failures |
| 10-07 | License GPL-3.0 | Derivative works open, security community |
