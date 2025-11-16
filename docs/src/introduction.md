# ProRT-IP WarScan

**Protocol/Port Real-Time War Scanner for IP Networks**

[![Build](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-2,111%20passing-success)](https://github.com/doublegate/ProRT-IP/actions)
[![Coverage](https://img.shields.io/badge/coverage-54.92%25-yellow)](https://codecov.io/gh/doublegate/ProRT-IP)
[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

**Modern network scanner combining Masscan/ZMap speed with Nmap detection depth.**

## What is ProRT-IP?

**ProRT-IP WarScan** (Protocol/Port Real-Time IP War Scanner) is a modern equivalent of classic 1980s/1990s war dialers—reimagined for IP networks. Where tools like ToneLoc and THC-Scan systematically dialed phone numbers to find modems/BBSs, WarScan systematically scans IP address ranges, ports, and protocols to discover active hosts and services.

WarScan consolidates and advances the best of today's network scanning and analysis tools, delivering a comprehensive, high-performance, stealth-focused toolkit for penetration testers and red teams.

## Key Features

- **Speed:** 10M+ packets/second stateless scanning (comparable to Masscan/ZMap)
- **Depth:** Comprehensive service detection and OS fingerprinting (like Nmap)
- **Safety:** Memory-safe Rust implementation prevents entire vulnerability classes
- **Stealth:** Advanced evasion techniques (timing, decoys, fragmentation, TTL manipulation, idle scans)
- **Modern TUI:** Real-time dashboard with 60 FPS rendering, 4-tab interface, 8 production widgets
- **Extensibility:** Plugin system with Lua 5.4 sandboxed execution

## At a Glance

- **Multi-Protocol Scanning:** TCP (SYN, Connect, FIN, NULL, Xmas, ACK, Idle/Zombie), UDP, ICMP/ICMPv6, NDP
- **IPv6 Support:** ✅ **Complete IPv6 support (all 8 scanners)** - Full dual-stack implementation
- **Service Detection:** 187 embedded protocol probes + 5 protocol-specific parsers (HTTP, SSH, SMB, MySQL, PostgreSQL) + SSL/TLS handshake (85-90% detection rate)
- **OS Fingerprinting:** 2,600+ signatures using 16-probe technique
- **Evasion Techniques:** IP fragmentation (-f, --mtu), TTL manipulation (--ttl), bad checksums (--badsum), decoy scanning (-D RND:N), idle/zombie scan (-sI)
- **High Performance:** Asynchronous I/O with lock-free coordination, zero-copy packet building, adaptive rate limiting (-1.8% overhead)
- **Cross-Platform:** Linux, Windows, macOS, FreeBSD support with NUMA optimization
- **Multiple Interfaces:** CLI (production-ready), TUI (60 FPS real-time dashboard), Web UI (planned), GUI (planned)

## Quick Start

```bash
# Download latest release
wget https://github.com/doublegate/ProRT-IP/releases/latest/download/prtip-linux-x86_64
chmod +x prtip-linux-x86_64
sudo mv prtip-linux-x86_64 /usr/local/bin/prtip

# SYN scan (requires privileges)
prtip -sS -p 80,443 192.168.1.0/24

# Fast scan (top 100 ports)
prtip -F 192.168.1.1

# Service detection
prtip -sV -p 1-1000 scanme.nmap.org

# TUI mode with real-time dashboard
prtip --tui -sS -p 1-1000 192.168.1.0/24
```

## Documentation Navigation

### New Users
- [Installation & Setup](./getting-started/installation.md) - Get started in 5 minutes
- [Quick Start Guide](./getting-started/quick-start.md) - Your first scan
- [Tutorial: Your First Scan](./getting-started/tutorials.md) - Step-by-step walkthrough

### Experienced Users
- [User Guide](./user-guide/basic-usage.md) - Comprehensive usage documentation
- [Scan Types](./user-guide/scan-types.md) - TCP, UDP, stealth scans
- [Nmap Compatibility](./features/nmap-compatibility.md) - Drop-in nmap replacement

### Advanced Topics
- [Performance Tuning](./advanced/performance-tuning.md) - Optimize for speed
- [TUI Architecture](./advanced/tui-architecture.md) - Real-time dashboard internals
- [Plugin System](./features/plugin-system.md) - Extend ProRT-IP with Lua

### Developers
- [Architecture Overview](./development/architecture.md) - System design
- [Contributing Guidelines](./development/contributing.md) - How to contribute
- [Testing Strategy](./development/testing.md) - Quality assurance

## Current Status

**Version:** v0.5.2 (Released 2025-11-14)
**Phase:** 6 Sprint 6.3 PARTIAL (3/8 sprints, 38%)
**Tests:** 2,111 passing (100%)
**Coverage:** 54.92%

### Recent Achievements

**Sprint 6.2 COMPLETE (2025-11-14):** Live Dashboard & Real-Time Metrics
- 4-tab dashboard interface (Port Table, Service Table, Metrics, Network Graph)
- Real-time port discovery and service detection visualization
- Performance metrics with 5-second rolling averages
- Network activity time-series chart (60-second sliding window)
- 175 tests passing (150 unit + 25 integration)

**Sprint 6.3 PARTIAL (3/6 task areas):**
- CDN IP Deduplication (30-70% target reduction)
- Adaptive Batch Sizing (20-40% throughput improvement)
- Integration Testing (comprehensive test coverage)

## Links

- **GitHub:** <https://github.com/doublegate/ProRT-IP>
- **Issues:** <https://github.com/doublegate/ProRT-IP/issues>
- **Security:** <https://github.com/doublegate/ProRT-IP/security/advisories>
- **Changelog:** [../CHANGELOG.md](../CHANGELOG.md)

## License

This project is licensed under the GNU General Public License v3.0.

**GPLv3 allows you to:**
- ✅ Use the software for any purpose
- ✅ Study and modify the source code
- ✅ Distribute copies
- ✅ Distribute modified versions

**Under the conditions:**
- ⚠️ Disclose source code of modifications
- ⚠️ License modifications under GPLv3
- ⚠️ State changes made to the code
- ⚠️ Include copyright and license notices

---

⚠️ **IMPORTANT:** Only scan networks you own or have explicit written permission to test. Unauthorized scanning may violate laws (CFAA, CMA, etc.).

**Last Updated:** 2025-11-15
