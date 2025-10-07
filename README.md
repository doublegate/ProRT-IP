# ProRT-IP WarScan

**Modern Network Scanner and War Dialer for IP Networks**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-pre--development-yellow.svg)]()
[![GitHub](https://img.shields.io/badge/github-ProRT--IP-blue)](https://github.com/doublegate/ProRT-IP)

---

## Table of Contents

- [Overview](#overview)
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

## Overview

ProRT-IP WarScan (Protocol/Port Real-Time IP War Scanner) is a modern network scanner written in Rust that combines:

- **Speed:** 1M+ packets/second stateless scanning (comparable to Masscan/ZMap)
- **Depth:** Comprehensive service detection and OS fingerprinting (like Nmap)
- **Safety:** Memory-safe Rust implementation prevents entire vulnerability classes
- **Stealth:** Advanced evasion techniques (timing, decoys, fragmentation, idle scans)
- **Extensibility:** Plugin system with Lua scripting support

### Key Features

- **Multi-Protocol Scanning:** TCP (SYN, Connect, FIN, NULL, Xmas, ACK, Idle), UDP, ICMP
- **Service Detection:** 500+ protocol probes with version identification
- **OS Fingerprinting:** 2000+ signatures using 16-probe technique
- **High Performance:** Asynchronous I/O with lock-free coordination
- **Cross-Platform:** Linux, Windows, macOS support
- **Multiple Interfaces:** CLI (v1.0), TUI (planned), Web UI (planned), GUI (planned)

---

## Project Status

**Current Phase:** Pre-Development (Genesis)

This project is in the planning and specification phase. Comprehensive documentation is complete and development is ready to begin.

**Progress:**
- ✅ Architecture design complete
- ✅ Technical specifications complete
- ✅ Development roadmap established (8 phases, 20 weeks)
- ✅ Testing strategy defined
- ✅ Security requirements documented
- ⏳ Implementation pending (Phase 1 starting)

---

## Documentation

### Root Documentation

| Document | Description |
|----------|-------------|
| **[ROADMAP.md](ROADMAP.md)** | High-level development roadmap and vision |
| **[CONTRIBUTING.md](CONTRIBUTING.md)** | Contribution guidelines and development process |
| **[SECURITY.md](SECURITY.md)** | Security policy and vulnerability reporting |
| **[SUPPORT.md](SUPPORT.md)** | Support resources and help |
| **[AUTHORS.md](AUTHORS.md)** | Contributors and acknowledgments |
| **[CHANGELOG.md](CHANGELOG.md)** | Version history and release notes |

### Technical Documentation (`docs/`)

Complete technical documentation is available in the [`docs/`](docs/) directory:

| Document | Description |
|----------|-------------|
| [00-ARCHITECTURE](docs/00-ARCHITECTURE.md) | System architecture and design patterns |
| [01-ROADMAP](docs/01-ROADMAP.md) | Detailed development phases and timeline |
| [02-TECHNICAL-SPECS](docs/02-TECHNICAL-SPECS.md) | Protocol specifications and data formats |
| [03-DEV-SETUP](docs/03-DEV-SETUP.md) | Development environment setup |
| [04-IMPLEMENTATION-GUIDE](docs/04-IMPLEMENTATION-GUIDE.md) | Code structure and patterns |
| [05-API-REFERENCE](docs/05-API-REFERENCE.md) | Complete API documentation |
| [06-TESTING](docs/06-TESTING.md) | Testing strategy and coverage |
| [07-PERFORMANCE](docs/07-PERFORMANCE.md) | Benchmarks and optimization |
| [08-SECURITY](docs/08-SECURITY.md) | Security implementation guide |
| [09-FAQ](docs/09-FAQ.md) | Frequently asked questions |
| [10-PROJECT-STATUS](docs/10-PROJECT-STATUS.md) | Current status and task tracking |

**Quick Start:** See [docs/README.md](docs/README.md) for navigation guide.

---

## Quick Start

### For Users

1. **Check project status**: [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md)
2. **Read FAQ**: [docs/09-FAQ.md](docs/09-FAQ.md)
3. **Get support**: [SUPPORT.md](SUPPORT.md)

### For Developers

1. **Understand architecture**: [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md)
2. **Set up environment**: [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md)
3. **Review roadmap**: [ROADMAP.md](ROADMAP.md) and [docs/01-ROADMAP.md](docs/01-ROADMAP.md)
4. **Start contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)

### For Security Researchers

1. **Read security policy**: [SECURITY.md](SECURITY.md)
2. **Review implementation**: [docs/08-SECURITY.md](docs/08-SECURITY.md)
3. **Report vulnerabilities**: See [SECURITY.md](SECURITY.md#reporting-security-vulnerabilities)

---

## Planned Usage

**Note:** Software not yet implemented. This shows intended usage.

```bash
# Basic SYN scan
prtip -sS -p 1-1000 192.168.1.0/24

# Service version detection
prtip -sS -sV -p 80,443 target.com

# OS fingerprinting with aggressive timing
prtip -sS -O -T4 target.com

# Stealth scan with decoys
prtip -sS -p 80 -D RND:5 -T2 target.com

# Full scan with all features
prtip -sS -sV -O -p- --output json target.com
```

---

## Development Roadmap

**8 Phases | 20 Weeks | 122+ Tasks**

### Quick Overview

| Phase | Timeline | Focus Area | Status |
|-------|----------|------------|--------|
| **Phase 1** | Weeks 1-3 | Core Infrastructure | Ready to begin |
| **Phase 2** | Weeks 4-6 | Advanced Scanning | Planned |
| **Phase 3** | Weeks 7-10 | Detection Systems | Planned |
| **Phase 4** | Weeks 11-13 | Performance Optimization | Planned |
| **Phase 5** | Weeks 14-16 | Advanced Features | Planned |
| **Phase 6** | Weeks 17-18 | User Interfaces | Planned |
| **Phase 7** | Weeks 19-20 | Release Preparation | Planned |
| **Phase 8** | Beyond | Post-Release Features | Future |

### Key Milestones

- **M0**: Documentation Complete ✅ (2025-10-07)
- **M1**: Basic Scanning Capability (Phase 1)
- **M2**: Production-Ready Scanning (Phase 2)
- **M3**: Comprehensive Detection (Phase 3)
- **M4**: High-Performance Scanning (Phase 4)
- **M5**: Enterprise Features (Phase 5)
- **M6**: Enhanced Usability (Phase 6)
- **M7**: Version 1.0 Release (Phase 7)

**Full Details**: See [ROADMAP.md](ROADMAP.md) and [docs/01-ROADMAP.md](docs/01-ROADMAP.md)

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

**See [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md) for platform-specific instructions.**

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

1. Read [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines
2. Review [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md) for system design
3. Check [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md) for available tasks
4. Set up your environment: [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md)

### Development Standards

- **Code Quality**: Run `cargo fmt` and `cargo clippy -- -D warnings`
- **Testing**: All PRs must include tests (>80% coverage)
- **Security**: Follow [docs/08-SECURITY.md](docs/08-SECURITY.md) guidelines
- **Documentation**: Update docs for new features
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) format

See [CONTRIBUTING.md](CONTRIBUTING.md) for complete details.

---

## Support

Need help? We're here to assist!

### Documentation

- **FAQ**: [docs/09-FAQ.md](docs/09-FAQ.md)
- **Troubleshooting**: [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md)
- **Full Docs**: [docs/README.md](docs/README.md)

### Community

- **Questions**: [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)
- **Bug Reports**: [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions/categories/ideas)

See [SUPPORT.md](SUPPORT.md) for comprehensive support resources.

---

## Security

### Security Policy

ProRT-IP WarScan is a **defensive security tool** for authorized penetration testing. We take security seriously.

### Reporting Vulnerabilities

🔒 **DO NOT** create public issues for security vulnerabilities.

- **Private Reporting**: Use [GitHub Security Advisories](https://github.com/doublegate/ProRT-IP/security/advisories)
- **Email**: Contact maintainers privately (see [SECURITY.md](SECURITY.md))

### Responsible Use

⚠️ **IMPORTANT**: Only scan networks you own or have explicit written permission to test.

- Unauthorized scanning may violate laws (CFAA, CMA, etc.)
- Always obtain authorization before testing
- Use for legitimate security research only

See [SECURITY.md](SECURITY.md) for full security policy and best practices.

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

See [AUTHORS.md](AUTHORS.md) for:
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

**Want to be listed?** See [CONTRIBUTING.md](CONTRIBUTING.md) to start contributing!

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
- **Planned Phases:** 8 phases over 20 weeks
- **Tracked Tasks:** 122+ implementation tasks
- **Target Performance:** 1M+ packets/second (stateless), 50K+ pps (stateful)
- **Code Coverage Goal:** >80% overall, >90% core modules

---

## Links

- **GitHub Repository**: https://github.com/doublegate/ProRT-IP
- **Issues**: https://github.com/doublegate/ProRT-IP/issues
- **Discussions**: https://github.com/doublegate/ProRT-IP/discussions
- **Security Advisories**: https://github.com/doublegate/ProRT-IP/security/advisories

---

**Current Status**: 📝 Documentation Complete ✅ | 🚧 Phase 1 Ready to Begin

**Last Updated**: 2025-10-07

For the latest project status, see [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md) and [CHANGELOG.md](CHANGELOG.md).
