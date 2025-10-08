# ProRT-IP WarScan

## Modern Network Scanner and War Dialer for IP Networks

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-Phase_2_Complete-brightgreen.svg)]
[![Tests](https://img.shields.io/badge/tests-278_passing-brightgreen.svg)]
[![GitHub](https://img.shields.io/badge/github-ProRT--IP-blue)](https://github.com/doublegate/ProRT-IP)

---

## Overview

**ProRT-IP WarScan** is a modern network scanner written in Rust that combines:

- **Speed:** 1M+ packets/second stateless scanning (comparable to Masscan/ZMap)
- **Depth:** Comprehensive service detection and OS fingerprinting (like Nmap)
- **Safety:** Memory-safe Rust implementation prevents entire vulnerability classes
- **Stealth:** Advanced evasion techniques (timing, decoys, fragmentation, idle scans)
- **Extensibility:** Plugin system with Lua scripting support

**At a glance:**

- **Multi-Protocol Scanning:** TCP (SYN, Connect, FIN, NULL, Xmas, ACK, Idle), UDP, ICMP
- **Service Detection:** 500+ protocol probes with version identification
- **OS Fingerprinting:** 2000+ signatures using 16-probe technique
- **High Performance:** Asynchronous I/O with lock-free coordination
- **Cross-Platform:** Linux, Windows, macOS support
- **Multiple Interfaces:** CLI (v1.0), TUI (planned), Web UI (planned), GUI (planned)

### Introduction

**ProRT-IP WarScan** (Protocol/Port Real-Time IP War Scanner) is a modern equivalent of classic 1980s/1990s war dialers—reimagined for IP networks. Where tools like ToneLoc and THC-Scan systematically dialed phone numbers to find modems/BBSs, WarScan systematically scans IP address ranges, ports, and protocols to discover active hosts and services.

WarScan consolidates and advances the best of today's network scanning and analysis tools, delivering a comprehensive, high-performance, stealth-focused toolkit for penetration testers and red teams. It is implemented in **Rust** for safety and performance, initially released as a **CLI** utility (`prtip`), with a roadmap for **TUI**, **web**, and **desktop GUI** interfaces.

**Key goals and characteristics:**

- **Extensive multi-layer scanning:** From host discovery (ARP/ICMP) up through TCP/UDP scans and application-layer banner grabbing
- **High performance & efficiency:** Internet-scale scanning inspired by the fastest modern scanners (1M+ packets/second stateless)
- **Advanced red-team features:** Stealth techniques (randomization, timing dithering, decoys, fragmentation, idle scans) to evade detection
- **Cross-platform & extensible:** Linux-first with Windows/macOS support via Rust portability; open-source (GPLv3)
- **Future UI enhancements:** TUI → web → GUI, expanding accessibility without sacrificing power

**In summary**, WarScan aims to be a one-stop, modern war-scanning solution—combining the thoroughness of classic mappers, the speed of internet-scale scanners, the usability of friendly GUIs, the deep packet insight of protocol analyzers, and the raw versatility of low-level network tools.

### Inspiration from Existing Tools

To design WarScan, we surveyed state-of-the-art tools widely used for networking, penetration testing, and packet analysis. Each contributes valuable features and lessons:

- **Nmap (Network Mapper):** The gold standard for discovery, versatile port scan techniques, service/version detection, OS fingerprinting, a powerful scripting engine, and numerous stealth/evasion capabilities. WarScan incorporates multiple scan types (connect, SYN, FIN/NULL/Xmas, UDP), service/OS detection, and similar evasion features in a modernized implementation.

- **Masscan:** Ultra high-speed, asynchronous/stateless internet-scale scanning at extreme packet rates. WarScan borrows the speed/scalability model—highly parallelized, stateless fast modes—then enables deeper follow-up scans on responders.

- **ZMap:** Internet-scale, single-packet rapid scans across huge IP ranges. WarScan includes a comparable "fast scan mode" for breadth-first discovery followed by depth on responsive hosts.

- **RustScan:** Demonstrates Rust's advantages: fast full-port sweeps, adaptive performance learning, and extensibility/scripting. WarScan mirrors this split-phase strategy (fast discovery → detailed enumeration) and evaluates an embedded scripting layer.

- **Unicornscan:** Pioneered asynchronous/stateless techniques and userland TCP/IP stack control for advanced packet crafting, banner grabbing, protocol-specific UDP probes, and OS/app fingerprinting. WarScan builds similar packet-crafting flexibility and export to PCAP/DB.

- **Wireshark:** The model for protocol depth and parsing. WarScan parses responses (e.g., HTTP headers, TLS certs), logs to PCAP, and emphasizes robust protocol coverage.

- **Angry IP Scanner:** Highlights usability, speed via multithreading, cross-platform reach, simple exports, and plugins. WarScan's roadmap includes a friendly TUI/GUI and enriched host info (reverse DNS, ARP/MAC/vendor, NetBIONS/mDNS where possible).

- **Netcat/Ncat:** The "Swiss Army knife" for quick banner grabs and interactive tests. WarScan supports custom payloads and optional interactive follow-ups to validate findings.

### Feature Comparison

![Network Scanner Feature Comparison](images/scanner_comparison.jpg)

*Comparison of ProRT-IP WarScan with leading network scanning tools (Nmap, Masscan, ZMap, RustScan)*

---

## Table of Contents

- [Project Status](#project-status)
- [Documentation](#documentation)
- [Quick Start](#quick-start)
- [Planned Usage](#planned-usage)
- [Development Roadmap](#development-roadmap)
- [Technical Specifications](#technical-specifications)
- [Building from Source](#building-from-source)
- [Contributing](#contributing)
- [Support](#support)
- [Security](#security)
- [License](#license)
- [Authors & Acknowledgments](#authors--acknowledgments)
- [Legal Notice](#legal-notice)

---

## Project Status

**Current Phase:** Phase 2 COMPLETE ✅ | Enhancement Cycles COMPLETE ✅ | **Phase 3 Ready**

**Latest Version:** 0.2.0 (Advanced Scanning + Production Enhancements)

**Test Coverage:** 391 tests passing (100% success rate)

**Recent Accomplishments:**
- ✅ Phase 1: Core Infrastructure (weeks 1-3)
- ✅ Phase 2: Advanced Scanning (weeks 4-6)
- ✅ Enhancement Cycle 1-5: Reference implementation optimizations
  - Cryptographic foundation (SipHash, Blackrock)
  - Concurrent scanning patterns (FuturesUnordered)
  - Port filtering infrastructure
  - Resource management (ulimit detection, interface selection)
  - Progress tracking and error categorization

**Enhancement Impact:**
- Tests: 100 → 391 (+291, +291% growth)
- Lines: ~2,930 added across 5 enhancement cycles
- Modules: 6 new production modules (crypto, concurrent_scanner, port_filter, resource_limits, interface, progress, errors)
- Dependencies: +2 (rlimit, indicatif)

**Next Phase:** Phase 3 - Detection Systems (OS fingerprinting, service detection, banner grabbing)

---

## Documentation

### Root Documentation

| Document | Description |
|----------|-------------|
| **[Roadmap](ROADMAP.md)** | High-level development roadmap and vision |
| **[Contributing](CONTRIBUTING.md)** | Contribution guidelines and development process |
| **[Security](SECURITY.md)** | Security policy and vulnerability reporting |
| **[Support](SUPPORT.md)** | Support resources and help |
| **[Authors](AUTHORS.md)** | Contributors and acknowledgments |
| **[Changelog](CHANGELOG.md)** | Version history and release notes |

### Technical Documentation (`docs/`)

Complete technical documentation is available in the [`docs/`](docs/) directory:

| Document | Description |
|----------|-------------|
| [Architecture](docs/00-ARCHITECTURE.md) | System architecture and design patterns |
| [Roadmap](docs/01-ROADMAP.md) | Detailed development phases and timeline |
| [Technical Specs](docs/02-TECHNICAL-SPECS.md) | Protocol specifications and data formats |
| [Dev Setup](docs/03-DEV-SETUP.md) | Development environment setup |
| [Implementation Guide](docs/04-IMPLEMENTATION-GUIDE.md) | Code structure and patterns |
| [API Reference](docs/05-API-REFERENCE.md) | Complete API documentation |
| [Testing](docs/06-TESTING.md) | Testing strategy and coverage |
| [Performance](docs/07-PERFORMANCE.md) | Benchmarks and optimization |
| [Security](docs/08-SECURITY.md) | Security implementation guide |
| [FAQ](docs/09-FAQ.md) | Frequently asked questions |
| [Project Status](docs/10-PROJECT-STATUS.md) | Current status and task tracking |

**Quick Start:** See [Documentation README](docs/README.md) for navigation guide.

---

## Quick Start

### For Users

1. **Check project status**: [Project Status](docs/10-PROJECT-STATUS.md)
2. **Read FAQ**: [FAQ](docs/09-FAQ.md)
3. **Get support**: [Support](SUPPORT.md)

### For Developers

1. **Understand architecture**: [Architecture](docs/00-ARCHITECTURE.md)
2. **Set up environment**: [Dev Setup](docs/03-DEV-SETUP.md)
3. **Review roadmap**: [Roadmap](ROADMAP.md) and [Detailed Roadmap](docs/01-ROADMAP.md)
4. **Start contributing**: [Contributing](CONTRIBUTING.md)

### For Security Researchers

1. **Read security policy**: [Security](SECURITY.md)
2. **Review implementation**: [Security Implementation](docs/08-SECURITY.md)
3. **Report vulnerabilities**: See [Security Policy](SECURITY.md#reporting-security-vulnerabilities)

---

## Usage Examples

**Phase 2 Features Implemented:**

```bash
# TCP SYN scan (half-open)
prtip -sS -p 1-1000 192.168.1.0/24

# TCP connect scan (full 3-way handshake)
prtip -sT -p 80,443 target.com

# UDP scan with protocol-specific payloads
prtip -sU -p 53,123,161 target.com

# Stealth FIN scan
prtip -sF -p 1-1000 192.168.1.0/24

# NULL scan (no flags)
prtip -sN -p 80,443 target.com

# Xmas scan (FIN, PSH, URG flags)
prtip -sX -p 1-1000 192.168.1.0/24

# ACK scan (firewall detection)
prtip -sA -p 80,443 target.com

# Aggressive timing template
prtip -sS -T4 -p 1-1000 192.168.1.0/24

# Paranoid timing (stealth)
prtip -sS -T0 -p 80,443 target.com

# Output formats
prtip -sS -p 80,443 --output json target.com  # JSON
prtip -sS -p 80,443 --output xml target.com   # XML
prtip -sS -p 80,443 --output text target.com  # Text (default)
```

**Planned Features (Phase 3+):**

```bash
# Service version detection (Phase 3)
prtip -sS -sV -p 80,443 target.com

# OS fingerprinting (Phase 3)
prtip -sS -O -T4 target.com

# Full scan with all features (Phase 3+)
prtip -sS -sV -O -p- --output json target.com
```

---

## Development Roadmap

### 8 Phases | 20 Weeks | 122+ Tasks

### Quick Overview

| Phase | Timeline | Focus Area | Status |
|-------|----------|------------|--------|
| **Phase 1** | Weeks 1-3 | Core Infrastructure | ✅ Complete |
| **Phase 2** | Weeks 4-6 | Advanced Scanning | ✅ Complete |
| **Phase 3** | Weeks 7-10 | Detection Systems | Ready to begin |
| **Phase 4** | Weeks 11-13 | Performance Optimization | Planned |
| **Phase 5** | Weeks 14-16 | Advanced Features | Planned |
| **Phase 6** | Weeks 17-18 | User Interfaces | Planned |
| **Phase 7** | Weeks 19-20 | Release Preparation | Planned |
| **Phase 8** | Beyond | Post-Release Features | Future |

### Key Milestones

- **M0**: Documentation Complete ✅ (2025-10-07)
- **M1**: Basic Scanning Capability ✅ (2025-10-07)
- **M2**: Advanced Scanning Complete ✅ (2025-10-08)
- **M3**: Comprehensive Detection (Phase 3)
- **M4**: High-Performance Scanning (Phase 4)
- **M5**: Enterprise Features (Phase 5)
- **M6**: Enhanced Usability (Phase 6)
- **M7**: Version 1.0 Release (Phase 7)

**Full Details**: See [Roadmap](ROADMAP.md) and [Detailed Roadmap](docs/01-ROADMAP.md)

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

**High-Performance:**

- CPU: 16+ cores @ 3.5+ GHz
- RAM: 32+ GB
- Storage: 10+ GB NVMe SSD
- Network: 10 Gbps+ with multi-queue NIC

### Supported Platforms

- **Linux:** Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch, RHEL 8+ (kernel 4.15+)
- **Windows:** Windows 10 (1809+), Windows 11 (requires Npcap)
- **macOS:** 11.0 (Big Sur) or later

---

## Building from Source

**Prerequisites:**

- Rust 1.70 or later
- libpcap (Linux/macOS) or Npcap (Windows)
- OpenSSL development libraries

**Linux:**

```bash
# Install dependencies
sudo apt install libpcap-dev libssl-dev pkg-config  # Debian/Ubuntu
sudo dnf install libpcap-devel openssl-devel        # Fedora

# Clone repository
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP

# Build
cargo build --release

# Grant capabilities (instead of root)
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Run
./target/release/prtip --help
```

**See [Dev Setup](docs/03-DEV-SETUP.md) for platform-specific instructions.**

---

## Contributing

We welcome contributions of all kinds! ProRT-IP WarScan is in early development and there are many opportunities to contribute.

### How to Contribute

- 🐛 **Report Bugs**: [Open an issue](https://github.com/doublegate/ProRT-IP/issues)
- 💡 **Suggest Features**: [Start a discussion](https://github.com/doublegate/ProRT-IP/discussions)
- 📖 **Improve Documentation**: Submit PRs for typos, clarifications, examples
- 💻 **Write Code**: Check [good first issues](https://github.com/doublegate/ProRT-IP/labels/good-first-issue)
- 🧪 **Write Tests**: Help us reach >90% coverage
- 🔍 **Review Code**: Help review pull requests

### Getting Started

1. Read [Contributing](CONTRIBUTING.md) for detailed guidelines
2. Review [Architecture](docs/00-ARCHITECTURE.md) for system design
3. Check [Project Status](docs/10-PROJECT-STATUS.md) for available tasks
4. Set up your environment: [Dev Setup](docs/03-DEV-SETUP.md)

### Development Standards

- **Code Quality**: Run `cargo fmt` and `cargo clippy -- -D warnings`
- **Testing**: All PRs must include tests (>80% coverage)
- **Security**: Follow [Security Implementation](docs/08-SECURITY.md) guidelines
- **Documentation**: Update docs for new features
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) format

See [Contributing](CONTRIBUTING.md) for complete details.

---

## Support

Need help? We're here to assist!

### Documentation Resources

- **FAQ**: [FAQ](docs/09-FAQ.md)
- **Troubleshooting**: [Dev Setup](docs/03-DEV-SETUP.md)
- **Full Docs**: [Documentation README](docs/README.md)

### Community

- **Questions**: [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)
- **Bug Reports**: [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions/categories/ideas)

See [Support](SUPPORT.md) for comprehensive support resources.

---

## Security

### Security Policy

ProRT-IP WarScan is a **defensive security tool** for authorized penetration testing. We take security seriously.

### Reporting Vulnerabilities

🔒 **DO NOT** create public issues for security vulnerabilities.

- **Private Reporting**: Use [GitHub Security Advisories](https://github.com/doublegate/ProRT-IP/security/advisories)
- **Email**: Contact maintainers privately (see [Security](SECURITY.md))

### Responsible Use

⚠️ **IMPORTANT**: Only scan networks you own or have explicit written permission to test.

- Unauthorized scanning may violate laws (CFAA, CMA, etc.)
- Always obtain authorization before testing
- Use for legitimate security research only

See [Security](SECURITY.md) for full security policy and best practices.

---

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

**GPLv3** allows you to:

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

### Contributors

ProRT-IP WarScan is developed and maintained by security researchers and Rust developers passionate about creating safe, high-performance security tools.

See [Authors](AUTHORS.md) for:

- Complete contributor list
- Acknowledgments to inspirational projects
- Recognition of Rust ecosystem contributors

### Inspirations

This project builds on the pioneering work of:

- **[Nmap](https://nmap.org/)** - Gordon "Fyodor" Lyon
- **[Masscan](https://github.com/robertdavidgraham/masscan)** - Robert Graham
- **[RustScan](https://github.com/RustScan/RustScan)** - RustScan Community
- **[ZMap](https://zmap.io/)** - University of Michigan

Special thanks to the Rust community for excellent libraries (Tokio, pnet, etherparse, clap, and many others).

**Want to be listed?** See [Contributing](CONTRIBUTING.md) to start contributing!

---

## Legal Notice

**IMPORTANT:** This tool is for authorized security testing only.

**You must have explicit permission to scan networks you do not own.** Unauthorized network scanning may violate:

- Computer Fraud and Abuse Act (US)
- Computer Misuse Act (UK)
- Similar laws in your jurisdiction

**Legitimate use cases:**

- Your own networks and systems
- Authorized penetration testing engagements
- Bug bounty programs (with explicit network scanning permission)
- Security research in isolated lab environments

**Always obtain written authorization before scanning networks.**

---

## Project Statistics

- **Total Documentation:** 478 KB (237 KB technical docs + 241 KB reference specs)
- **Root Documents:** 6 files (ROADMAP, CONTRIBUTING, SECURITY, SUPPORT, AUTHORS, CHANGELOG)
- **Technical Documents:** 12 files in docs/ directory
- **Development Phases:** 8 phases over 20 weeks (Phase 2 + Enhancements complete)
- **Implementation Progress:** Phase 2/8 + Enhancements complete (30%)
- **Test Suite:** 391 tests passing (87 core + 35 network + 114 scanner + 63 cli + 37 integration + 55 doctests)
- **Crates Implemented:** 4 (prtip-core, prtip-network, prtip-scanner, prtip-cli)
- **Total Code Added:** 6,481 lines (Phase 2: 3,551 + Enhancements: 2,930)
- **Enhancement Cycles:** 5 complete (cryptographic foundation, filtering, resource management, CLI integration, user feedback)
- **New Modules:** 13 production modules (6 Phase 2 + 7 enhancements)
- **Scan Types:** 7 implemented (Connect, SYN, UDP, FIN, NULL, Xmas, ACK)
- **Protocol Payloads:** 8 (DNS, NTP, NetBIOS, SNMP, RPC, IKE, SSDP, mDNS)
- **Timing Templates:** 6 (T0-T5 paranoid to insane)
- **CLI Version:** 0.2.0 (advanced scanning + production enhancements)
- **Dependencies:** 2 new (rlimit 0.10.2, indicatif 0.17)
- **Target Performance:** 1M+ packets/second (stateless), 50K+ pps (stateful)
- **Code Coverage:** 391/391 tests (100% pass rate)

---

## Links

- **GitHub Repository**: <https://github.com/doublegate/ProRT-IP>
- **Issues**: <https://github.com/doublegate/ProRT-IP/issues>
- **Discussions**: <https://github.com/doublegate/ProRT-IP/discussions>
- **Security Advisories**: <https://github.com/doublegate/ProRT-IP/security/advisories>

---

**Current Status**: ✅ Phase 2 Complete | ✅ Enhancement Cycles Complete | 🚀 Phase 3 Ready | 391 Tests Passing | 6,481 Lines Added

**Last Updated**: 2025-10-08

For the latest project status, see [Project Status](docs/10-PROJECT-STATUS.md) and [Changelog](CHANGELOG.md).
