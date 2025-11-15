# ProRT-IP WarScan

[![Build](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-2,111%20passing-success)](https://github.com/doublegate/ProRT-IP/actions)
[![Coverage](https://img.shields.io/badge/coverage-54.92%25-yellow)](https://codecov.io/gh/doublegate/ProRT-IP)
[![License](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

**Modern network scanner combining Masscan/ZMap speed with Nmap detection depth.**

---

## Overview

ProRT-IP WarScan is a production-ready network scanner built in Rust that delivers industry-leading performance without sacrificing detection accuracy. Designed for penetration testers and security researchers, it provides:

- **Blazing Speed:** 10M+ packets/second stateless scanning (theoretical), 72K+ pps stateful (achieved)
- **Deep Detection:** 85-90% service detection accuracy, OS fingerprinting with 2,600+ signatures
- **Advanced Evasion:** 8 scan types, 6 evasion techniques, decoy scanning, packet fragmentation
- **Modern TUI:** Real-time dashboard with 60 FPS rendering, 4-tab interface, 8 production widgets
- **Production Ready:** 2,111 tests (100% passing), 54.92% coverage, 230M+ fuzz executions (0 crashes)

**Inspired by:** Nmap (detection depth) + Masscan (raw speed) + RustScan (modern async)
**Built with:** Tokio async runtime, pnet packet library, ratatui TUI framework

---

## Table of Contents

- [Project Status](#project-status)
- [Quick Start](#quick-start)
- [Features](#features)
- [Terminal User Interface (TUI)](#terminal-user-interface-tui)
- [Usage Examples](#usage-examples)
- [Plugin System](#plugin-system)
- [Documentation](#documentation)
- [Development Roadmap](#development-roadmap)
- [Technical Specifications](#technical-specifications)
- [Contributing](#contributing)
- [License](#license)

---

## Project Status

**Current:** Phase 6 Sprint 6.3 PARTIAL (3/8 sprints, 38%)
**Version:** v0.5.2 (Released 2025-11-14)
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
- Remaining: Batch I/O Implementation, Scheduler Integration, Production Benchmarks

### Phase 5 + 5.5 Archive

ProRT-IP completed Phase 5 (Advanced Features) and Phase 5.5 (Pre-TUI Enhancements) encompassing 16 comprehensive sprints over 26 days (~240 hours development effort). These phases established production-ready foundations including IPv6 100% coverage, service detection (85-90% accuracy), idle scanning, rate limiting (-1.8% overhead), TLS certificate analysis, code coverage automation, fuzz testing (230M+ executions), plugin system (Lua), benchmarking framework, and complete documentation polish.

**For complete Phase 5/5.5 history, see:** [docs/archive/PHASE-5-README-ARCHIVE.md](docs/archive/PHASE-5-README-ARCHIVE.md) (1,862 lines comprehensive archive)

---

## Quick Start

### Installation

**Pre-built binaries** (5 production platforms):
```bash
# Download latest release
wget https://github.com/doublegate/ProRT-IP/releases/latest/download/prtip-linux-x86_64
chmod +x prtip-linux-x86_64
sudo mv prtip-linux-x86_64 /usr/local/bin/prtip
```

**Build from source:**
```bash
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip
```

### Basic Usage

```bash
# SYN scan (requires privileges)
prtip -sS -p 80,443 192.168.1.0/24

# Fast scan (top 100 ports)
prtip -F 192.168.1.1

# Service detection
prtip -sV -p 1-1000 scanme.nmap.org

# TUI mode with real-time dashboard
prtip --tui -sS -p 1-1000 192.168.1.0/24
```

---

## Features

### Core Capabilities

- **8 Scan Types:** SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle (zombie)
- **Protocol Support:** IPv4/IPv6 dual-stack (100% coverage), 8 UDP protocol payloads
- **Detection:** Service detection (85-90% accuracy), OS fingerprinting (2,600+ signatures), TLS certificate analysis
- **Performance:** 10M+ pps theoretical, 72K+ pps stateful (localhost), adaptive parallelism, rate limiting (-1.8% overhead)
- **Evasion:** Packet fragmentation (-f, --mtu), TTL control, bad checksums, decoy scanning, timing templates (T0-T5)
- **Output:** Text, JSON, XML, Greppable, PCAPNG capture, SQLite database storage
- **TUI:** Real-time dashboard with 60 FPS, 4-tab interface, 8 widgets, event-driven updates (10K+ events/sec)
- **Plugin System:** Lua 5.4 sandboxed execution, capabilities-based security, hot reload

### Network Optimizations (Sprint 6.3)

- **CDN Deduplication:** Intelligent filtering of CDN infrastructure IPs (30-70% reduction target)
- **Adaptive Batching:** Dynamic batch size adjustment based on network performance (20-40% throughput improvement)
- **Batch I/O:** sendmmsg/recvmmsg support for high-performance packet transmission (pending implementation)

### Quality Assurance

- **Testing:** 2,111 tests (100% passing), 54.92% code coverage
- **Fuzz Testing:** 230M+ executions across 5 fuzzers, 0 crashes
- **CI/CD:** 9/9 GitHub Actions workflows, multi-platform matrix (Linux/macOS/Windows)
- **Benchmarking:** Automated performance regression detection (5%/10% thresholds)

---

## Terminal User Interface (TUI)

### Overview

ProRT-IP includes a production-ready TUI for real-time scan visualization and monitoring. Built with ratatui 0.29 and crossterm 0.28, the interface provides instant feedback at 60 FPS with <5ms render times.

### Key Features

- **Real-Time Visualization:** Live scan progress with 60 FPS rendering
- **Event-Driven Updates:** Integrated with EventBus for instant result updates (10K+ events/sec)
- **8 Production Widgets:** StatusBar, MainWidget, LogWidget, HelpWidget + 4 Dashboard Widgets
- **Tabbed Interface:** 4 real-time dashboards with Tab key switching
- **Thread-Safe State:** Shared `Arc<RwLock<ScanState>>` for scanner integration
- **Responsive Design:** Immediate mode rendering with <5ms frame time per widget

### Dashboard Widgets (Sprint 6.2)

**1. Port Table Widget**
- Real-time port discovery visualization (1,000-entry ringbuffer)
- Sortable 6-column table: Timestamp, IP, Port, State, Protocol, Scan Type
- Triple filtering: State, Protocol, Search (IP/Port)
- Color-coded states: Open=Green, Filtered=Yellow, Closed=Red
- Keyboard: t/i/p/s/r/c (sort), a (auto-scroll), f/d (filters)

**2. Service Table Widget**
- Real-time service detection visualization (500-entry ringbuffer)
- Sortable 6-column table: Timestamp, IP, Port, Service, Version, Confidence
- Confidence-based color coding: Green ≥90%, Yellow 50-89%, Red <50%
- Multi-level filtering: All, Low (≥50%), Medium (≥75%), High (≥90%)
- Keyboard: 1-6 (sort by column), c (cycle filter), a (auto-scroll)

**3. Metrics Dashboard Widget**
- Real-time performance metrics in 3-column layout
- Progress: Scan percentage, completed/total, ETA calculation
- Throughput: Current/average/peak ports/sec, packets/sec (5-second rolling average)
- Statistics: Open ports, services, errors, scan duration, status indicator
- Human-readable formatting with color-coded status

**4. Network Graph Widget**
- Real-time network activity time-series chart (60-second sliding window)
- Three data series: packets sent (cyan), packets received (green), ports discovered (yellow)
- Auto-scaling Y-axis bounds with 10% headroom
- 1 sample/second with derivative computation for "ports/sec"

### Global Keyboard Shortcuts

- `q` / `Ctrl+C` - Quit TUI
- `?` - Toggle help screen
- `Tab` / `Shift+Tab` - Switch dashboards (Port Table ↔ Service Table ↔ Metrics ↔ Network Graph)

### Performance Metrics

- **Rendering:** 60 FPS (16.67ms frame budget, <5ms actual)
- **Event Throughput:** 10,000+ events/second with aggregation
- **Memory Overhead:** <10 MB (TUI framework + buffers)
- **Tests:** 175 passing (150 unit + 25 integration)

### Launching TUI

```bash
# Start TUI with default scan
prtip --tui -p 80,443 192.168.1.0/24

# TUI with service detection
prtip --tui -sV -p 1-1000 scanme.nmap.org

# TUI with all features
prtip --tui -sS -sV -O -p- 192.168.1.1
```

**See also:** [TUI Architecture Guide](docs/TUI-ARCHITECTURE.md) (891 lines comprehensive documentation)

---

## Usage Examples

### Basic Scanning

```bash
# SYN scan (default, requires privileges)
prtip -sS -p 80,443 192.168.1.0/24

# TCP Connect scan (no privileges required)
prtip -sT -p 1-1000 192.168.1.1

# UDP scan
prtip -sU -p 53,161,500 192.168.1.0/24

# Fast scan (top 100 ports)
prtip -F 192.168.1.1

# All ports
prtip -p- 192.168.1.1
```

### Advanced Detection

```bash
# Service detection
prtip -sV -p 22,80,443 192.168.1.1

# OS fingerprinting
prtip -O -p 1-1000 192.168.1.1

# Aggressive scan (OS + service + scripts)
prtip -A -p 80,443 target.com

# TLS certificate analysis
prtip -sV --tls-cert -p 443 example.com
```

### Stealth & Evasion

```bash
# Stealth scans (FIN/NULL/Xmas)
prtip -sF -p 1-1000 192.168.1.1
prtip -sN -p 1-1000 192.168.1.1
prtip -sX -p 1-1000 192.168.1.1

# Packet fragmentation
prtip -sS -f -p 80,443 target.com

# Custom MTU fragmentation
prtip -sS --mtu 200 --ttl 32 target.com

# Decoy scanning (5 random decoys)
prtip -sS -D RND:5 -p 80,443 target.com

# Source port spoofing
prtip -sS -g 53 -p 80,443 target.com

# Timing templates (T0=paranoid, T5=insane)
prtip -sS -T0 -p 80 target.com
```

### Output Formats

```bash
# Normal text output
prtip -p 22 192.168.1.1 -oN scan.txt

# XML format (nmap-compatible)
prtip -p 22 192.168.1.1 -oX scan.xml

# Greppable output
prtip -p 22 192.168.1.1 -oG scan.gnmap

# JSON output
prtip -p 22 192.168.1.1 --output json --output-file scan.json

# PCAPNG packet capture
prtip -p 22 192.168.1.1 --packet-capture scan.pcapng
```

### Performance Tuning

```bash
# Timing templates
prtip -T4 -p 1-1000 192.168.1.0/24  # Aggressive
prtip -T2 -p 1-1000 192.168.1.0/24  # Polite

# Rate limiting
prtip --rate-limit 1000 -p 1-1000 192.168.1.0/24

# Custom parallelism
prtip --parallelism 500 -p 1-65535 192.168.1.0/24

# Adaptive batch sizing (Sprint 6.3)
prtip --adaptive-batch --min-batch-size 100 --max-batch-size 2000 -p 1-65535 192.168.1.0/24
```

**More examples:** [docs/34-EXAMPLES-GALLERY.md](docs/34-EXAMPLES-GALLERY.md) (65 comprehensive scenarios)

---

## Plugin System

ProRT-IP v0.4.8+ includes a Lua-based plugin system for extending scanner functionality with custom detection logic, output formats, and scan lifecycle hooks.

### Plugin Types

1. **Detection Plugins** - Enhanced service detection and banner analysis
2. **Output Plugins** - Custom result formatting and export
3. **Scan Plugins** - Lifecycle hooks for scan coordination

### Features

- **Sandboxed Execution:** Lua VM with resource limits (100MB memory, 5s CPU, 1M instructions)
- **Capabilities-Based Security:** Explicit Network/Filesystem/System/Database permissions (deny-by-default)
- **Hot Reload:** Load/unload plugins without restarting scanner
- **Plugin API:** `prtip.*` table with logging, network, and result manipulation functions

### Quick Examples

```bash
# List available plugins
prtip plugin list

# Load a specific plugin
prtip plugin load banner-analyzer

# Run scan with plugins
prtip -sS -p 80,443 --plugin banner-analyzer,ssl-checker 192.168.1.0/24
```

### Example Plugins

**banner-analyzer** (Detection Plugin)
- Detects: HTTP, SSH, FTP, SMTP, MySQL, PostgreSQL, Redis, MongoDB
- 85-90% detection accuracy
- No capabilities required (passive analysis)

**ssl-checker** (Detection Plugin)
- SSL/TLS protocol detection
- Certificate validation
- Requires: `network` capability

**Documentation:** [docs/30-PLUGIN-SYSTEM-GUIDE.md](docs/30-PLUGIN-SYSTEM-GUIDE.md) (784 lines comprehensive guide)

---

## Documentation

### Core Documentation

- [Architecture](docs/00-ARCHITECTURE.md) - System design and module relationships
- [Documentation Index](docs/00-DOCUMENTATION-INDEX.md) - Complete catalog of all docs (1,070 lines)
- [Roadmap](docs/01-ROADMAP.md) - Development phases and sprint planning
- [Project Status](docs/10-PROJECT-STATUS.md) - Current progress and metrics
- [TUI Architecture](docs/TUI-ARCHITECTURE.md) - TUI framework comprehensive guide (891 lines)

### Guides & References

- [User Guide](docs/32-USER-GUIDE.md) - Comprehensive usage documentation (2,448 lines)
- [Plugin System Guide](docs/30-PLUGIN-SYSTEM-GUIDE.md) - Plugin development (784 lines)
- [Benchmarking Guide](docs/31-BENCHMARKING-GUIDE.md) - Performance testing (1,044 lines)
- [Examples Gallery](docs/34-EXAMPLES-GALLERY.md) - 65 practical scenarios
- [Nmap Compatibility](docs/14-NMAP_COMPATIBILITY.md) - Flag compatibility matrix

### Technical References

- [IPv6 Guide](docs/23-IPv6-IMPLEMENTATION.md) - IPv6 100% implementation
- [Service Detection](docs/24-SERVICE-DETECTION.md) - Detection engine details
- [Idle Scan](docs/25-IDLE-SCAN-GUIDE.md) - Zombie scanning implementation
- [Rate Limiting](docs/26-RATE-LIMITING.md) - Adaptive rate control
- [TLS Certificate](docs/27-TLS-CERTIFICATE-ANALYSIS.md) - X.509v3 parsing

### Historical Archive

- [Phase 5 Archive](docs/archive/PHASE-5-README-ARCHIVE.md) - Complete Phase 5/5.5 history (1,862 lines)

---

## Development Roadmap

### Current Focus: Phase 6 - TUI Interface + Network Optimizations

**Progress:** 2.5/8 sprints (31.25%)

| Sprint | Status | Description |
|--------|--------|-------------|
| 6.1 | ✅ Complete | TUI Framework (60 FPS, 4 widgets, event integration) |
| 6.2 | ✅ Complete | Live Dashboard (4-tab interface, 8 widgets total) |
| 6.3 | 🔄 Partial | Network Optimizations (CDN dedup, adaptive batching) |
| 6.4 | 📋 Planned | Zero-Copy Optimizations |
| 6.5 | 📋 Planned | Interactive Target Selection |
| 6.6 | 📋 Planned | TUI Polish & UX |
| 6.7 | 📋 Planned | Configuration Profiles |
| 6.8 | 📋 Planned | Help System & Tooltips |

### Overall Progress: 8 Phases

| Phase | Timeline | Focus | Status |
|-------|----------|-------|--------|
| Phase 1-3 | Weeks 1-10 | Core Infrastructure, Advanced Scanning, Detection | ✅ Complete |
| Phase 4 | Weeks 11-13 | Performance Optimization | ✅ Complete |
| Phase 5 + 5.5 | Weeks 14-18 | Advanced Features + Pre-TUI | ✅ Complete |
| **Phase 6** | **Weeks 19-20** | **TUI Interface** | **🔄 31% (2.5/8 sprints)** |
| Phase 7 | Weeks 21-22 | Release Preparation | 📋 Planned |
| Phase 8 | Beyond | Post-Release Features | 📋 Future |

**Full roadmap:** [docs/01-ROADMAP.md](docs/01-ROADMAP.md) (comprehensive sprint breakdown)

---

## Technical Specifications

### System Requirements

**Minimum:**
- CPU: 2 cores @ 2.0 GHz
- RAM: 2 GB
- Storage: 100 MB
- Network: 100 Mbps

**Recommended:**
- CPU: 8+ cores @ 3.0 GHz
- RAM: 16 GB
- Storage: 1 GB SSD
- Network: 1 Gbps+

### Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 (glibc) | ✅ Production | Debian, Ubuntu, Fedora, Arch, CentOS (NUMA-optimized) |
| Windows x86_64 | ✅ Production | Windows 10+, Server 2016+ (requires Npcap) |
| macOS Intel (x86_64) | ✅ Production | macOS 10.13+ (High Sierra and later) |
| macOS Apple Silicon (ARM64) | ✅ Production | M1/M2/M3/M4 chips (native binary) |
| FreeBSD x86_64 | ✅ Production | FreeBSD 12+ |

**Platform Coverage:** 5 production platforms covering ~95% of target user base

### Performance Characteristics

- **Throughput:** 10M+ packets/second (stateless, theoretical), 72K+ pps (stateful, achieved)
- **Scan Speed:** 65K ports in 0.91s (198x faster than baseline)
- **Detection Accuracy:** Service detection 85-90%, OS fingerprinting 2,600+ signatures
- **Rate Limiting Overhead:** -1.8% (industry-leading efficiency)
- **TUI Performance:** 60 FPS rendering, <5ms frame time, 10K+ events/sec

### Build Requirements

- Rust 1.85 or later (MSRV for edition 2024)
- libpcap (Linux/macOS) or Npcap (Windows)
- OpenSSL development libraries
- (Optional) hwloc for NUMA optimization (Linux only)

---

## Contributing

We welcome contributions of all kinds! ProRT-IP is in active development with many opportunities to contribute.

### Ways to Contribute

- 🐛 **Report Bugs:** [Bug Report Template](https://github.com/doublegate/ProRT-IP/issues/new?template=bug_report.yml)
- 💡 **Suggest Features:** [Feature Request](https://github.com/doublegate/ProRT-IP/issues/new?template=feature_request.yml)
- ⚡ **Performance Issues:** [Performance Template](https://github.com/doublegate/ProRT-IP/issues/new?template=performance.yml)
- 📖 **Improve Documentation:** [Documentation Template](https://github.com/doublegate/ProRT-IP/issues/new?template=documentation.yml)
- 💻 **Write Code:** Check [good first issues](https://github.com/doublegate/ProRT-IP/labels/good-first-issue)
- 🧪 **Write Tests:** Help us reach >90% coverage

### Development Standards

- **Code Quality:** Run `cargo fmt` and `cargo clippy -- -D warnings`
- **Testing:** All PRs must include tests (>80% coverage)
- **Security:** Follow [Security Implementation](docs/08-SECURITY.md) guidelines
- **Documentation:** Update docs for new features
- **Commits:** Use [Conventional Commits](https://www.conventionalcommits.org/) format

**See:** [CONTRIBUTING.md](CONTRIBUTING.md) for complete guidelines

---

## License

This project is licensed under the GNU General Public License v3.0 - see [LICENSE](LICENSE) for details.

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

## Authors & Acknowledgments

ProRT-IP WarScan is developed by security researchers and Rust developers passionate about creating safe, high-performance security tools.

### Inspirations

This project builds on the pioneering work of:
- **[Nmap](https://nmap.org/)** - Gordon "Fyodor" Lyon
- **[Masscan](https://github.com/robertdavidgraham/masscan)** - Robert Graham
- **[RustScan](https://github.com/RustScan/RustScan)** - RustScan Community
- **[ZMap](https://zmap.io/)** - University of Michigan

**See:** [AUTHORS.md](AUTHORS.md) for complete contributor list

---

## Security & Legal

### Security Policy

🔒 **Private Reporting:** [GitHub Security Advisories](https://github.com/doublegate/ProRT-IP/security/advisories)

**See:** [SECURITY.md](SECURITY.md) for full security policy

### Responsible Use

⚠️ **IMPORTANT:** Only scan networks you own or have explicit written permission to test.

- Unauthorized scanning may violate laws (CFAA, CMA, etc.)
- Always obtain authorization before testing
- Use for legitimate security research only

---

## Links

- **GitHub:** <https://github.com/doublegate/ProRT-IP>
- **Issues:** <https://github.com/doublegate/ProRT-IP/issues>
- **Discussions:** <https://github.com/doublegate/ProRT-IP/discussions>
- **Security:** <https://github.com/doublegate/ProRT-IP/security/advisories>
- **Changelog:** [CHANGELOG.md](CHANGELOG.md)

---

**Last Updated:** 2025-11-15
**Current Version:** v0.5.2
**Phase:** 6 Sprint 6.3 PARTIAL (3/8 sprints, 38%)
