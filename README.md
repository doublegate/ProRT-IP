# ProRT-IP WarScan

**Modern Network Scanner and War Dialer for IP Networks**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-pre--development-yellow.svg)]()

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

Complete documentation is available in the [`docs/`](docs/) directory:

| Document | Description |
|----------|-------------|
| [00-ARCHITECTURE](docs/00-ARCHITECTURE.md) | System architecture and design patterns |
| [01-ROADMAP](docs/01-ROADMAP.md) | Development phases and timeline |
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

### Phase 1: Core Infrastructure (Weeks 1-3)
- Packet capture abstraction (Linux/Windows/macOS)
- Basic TCP connect scanning
- CLI argument parsing
- Privilege management

### Phase 2: Advanced Scanning (Weeks 4-6)
- TCP SYN scanning with raw sockets
- UDP scanning with protocol-specific probes
- Stealth scans (FIN, NULL, Xmas)
- Timing templates and rate control

### Phase 3: Detection Systems (Weeks 7-10)
- OS fingerprinting (16-probe sequence)
- Service version detection
- Banner grabbing with SSL/TLS

### Phase 4: Performance Optimization (Weeks 11-13)
- Lock-free architecture
- Stateless scanning mode
- NUMA optimization

### Phase 5: Advanced Features (Weeks 14-16)
- Idle (zombie) scanning
- Packet fragmentation and decoys
- Lua plugin system

### Phase 6-7: UI and Release (Weeks 17-20)
- TUI interface
- Documentation completion
- v1.0 release

**See [docs/01-ROADMAP.md](docs/01-ROADMAP.md) for complete timeline.**

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
git clone https://github.com/YOUR_ORG/prtip-warscan.git
cd prtip-warscan

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

Contributions are welcome! This project is in early development.

### Getting Started

1. Read [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md) - understand the design
2. Review [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md) - find available tasks
3. Follow [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md) - set up your environment
4. Check [docs/08-SECURITY.md](docs/08-SECURITY.md) - follow security practices
5. See [docs/06-TESTING.md](docs/06-TESTING.md) - write tests for your code

### Development Process

- **Code style:** `cargo fmt` and `cargo clippy`
- **Testing:** All PRs must include tests
- **Security:** Follow secure coding guidelines
- **Documentation:** Update docs for new features

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

## Acknowledgments

This project draws inspiration from:

- **Nmap** - The gold standard for network scanning (https://nmap.org)
- **Masscan** - Ultra-fast asynchronous scanner (https://github.com/robertdavidgraham/masscan)
- **ZMap** - Internet-scale network scanner (https://zmap.io)
- **RustScan** - Fast Rust port scanner (https://github.com/RustScan/RustScan)
- **Unicornscan** - Asynchronous stateless scanner

Special thanks to the Rust community for excellent networking libraries (pnet, tokio, etc.).

---

## Contact

- **Issues:** https://github.com/YOUR_ORG/prtip-warscan/issues
- **Discussions:** https://github.com/YOUR_ORG/prtip-warscan/discussions
- **Security:** See [SECURITY.md](SECURITY.md) for reporting vulnerabilities

---

## Project Statistics

- **Documentation:** 237 KB across 13 documents
- **Planned Phases:** 8 phases over 20 weeks
- **Tracked Tasks:** 122+ implementation tasks
- **Target Performance:** 1M+ packets/second (stateless)
- **Code Coverage Goal:** >80% overall, >90% core modules

---

**Status:** 📝 Documentation Complete | 🚧 Implementation Starting Soon

For the latest updates, see [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md).
