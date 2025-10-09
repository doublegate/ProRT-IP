# ProRT-IP WarScan v0.3.0 - Production Release

**Release Date:** 2025-10-08
**Version:** v0.3.0
**Repository:** https://github.com/doublegate/ProRT-IP
**License:** GPL-3.0

---

## 🚀 Major Features

### Scanning Capabilities
- **7 scan types:** TCP Connect, SYN, UDP, FIN, NULL, Xmas, ACK
- **Protocol payloads:** 8 protocol-specific UDP payloads (DNS, NTP, NetBIOS, SNMP, RPC, IKE, SSDP, mDNS)
- **Timing templates:** T0-T5 (Paranoid to Insane) with RTT estimation

### Detection Systems
- **OS fingerprinting:** 16-probe sequence (6 TCP SYN, 2 ICMP, 1 ECN, 6 unusual TCP, 1 UDP)
- **Service detection:** nmap-service-probes format with 500+ probes
- **Banner grabbing:** 6 protocols + TLS support (HTTP, HTTPS, FTP, SSH, SMTP, DNS, SNMP)

### Performance & Stealth
- **Batch packet sending:** sendmmsg syscall (30-50% improvement at 1M+ pps)
- **Adaptive rate limiting:** Masscan-inspired circular buffer with dynamic batching
- **Connection pooling:** RustScan pattern with FuturesUnordered
- **Decoy scanning:** Up to 256 decoys for stealth attribution hiding
- **CDN/WAF detection:** 8 major providers with O(log n) lookup

### Infrastructure
- **Network interface detection:** Automatic routing and source IP selection
- **Resource management:** ulimit detection and batch size optimization
- **Privilege management:** Immediate drop after socket creation
- **Cross-platform:** Linux/Windows/macOS support

### User Experience
- **Professional CLI:** Cyber-punk ASCII banner with gradient colors
- **Progress tracking:** Real-time statistics with ETA estimation
- **Error categorization:** 7 categories with actionable suggestions
- **Multiple output formats:** JSON, XML, Text, SQLite

---

## 📊 Statistics

- **Tests:** 551 (100% pass rate)
- **Code:** 10,000+ lines across 4 crates (40+ modules)
- **Dependencies:** Production-ready with security audits passing
- **Platforms:** Linux, Windows, macOS
- **MSRV:** Rust 1.85+

---

## 🆕 What's New in v0.3.0

### Enhancement Cycle 8
1. **Batch Packet Sending (sendmmsg)**
   - Linux kernel syscall batching for 30-50% performance boost
   - Cross-platform fallback support
   - 656 lines of optimized code

2. **CDN/WAF Detection**
   - 8 major providers: Cloudflare, Akamai, Fastly, CloudFront, Google Cloud, Azure, Imperva, Sucuri
   - O(log n) binary search on sorted CIDR ranges
   - 455 lines of detection logic

3. **Decoy Scanning**
   - Up to 256 decoys for stealth
   - Fisher-Yates shuffle for randomization
   - Reserved IP avoidance (RFC 1918, multicast, etc.)
   - 505 lines of stealth implementation

### CI/CD Infrastructure
- GitHub Actions workflows for automated testing
- Multi-platform CI: Linux, Windows, macOS
- Security scanning: CodeQL, dependency review, cargo audit
- Automated release builds for 4 targets
- MSRV verification (Rust 1.82+)

### Quality Improvements
- Fixed 4 previously ignored doc-tests
- Zero clippy warnings (strict -D warnings mode)
- 100% test success rate (551 tests)
- Comprehensive documentation audit

---

## 📥 Installation

### Download Pre-built Binaries

Choose the appropriate binary for your platform:

- **Linux (glibc):** `prtip-0.3.0-x86_64-unknown-linux-gnu.tar.gz`
- **Linux (musl/static):** `prtip-0.3.0-x86_64-unknown-linux-musl.tar.gz`
- **Windows:** `prtip-0.3.0-x86_64-pc-windows-msvc.zip`
- **macOS:** `prtip-0.3.0-x86_64-apple-darwin.tar.gz`

### Linux / macOS
```bash
# Extract
tar xzf prtip-0.3.0-x86_64-unknown-linux-gnu.tar.gz

# Make executable
chmod +x prtip

# Run
./prtip --help
```

### Windows
```cmd
# Extract the zip file
# Then run:
prtip.exe --help
```

### Build from Source
```bash
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release
./target/release/prtip --help
```

---

## 🔧 Usage Examples

### Basic SYN Scan
```bash
prtip -sS -p 1-1000 192.168.1.0/24
```

### OS Detection + Service Detection
```bash
prtip -sS -O -sV -p 1-1000 10.0.0.1
```

### Stealth Scan with Decoys
```bash
prtip -sF -D RND:10 -p 80,443 target.com
```

### Fast Scan with Progress
```bash
prtip -T4 -p- --progress 192.168.1.1
```

### CDN Detection
```bash
prtip -sS -p 80,443 --detect-cdn example.com
```

### All Features Combined
```bash
prtip -sS -O -sV -D RND:5 -T4 --progress --stats-file scan.json -p 1-1000 target.com
```

---

## 📚 Documentation

- [README](https://github.com/doublegate/ProRT-IP/blob/main/README.md) - Project overview
- [Architecture](https://github.com/doublegate/ProRT-IP/blob/main/docs/00-ARCHITECTURE.md) - System design
- [Implementation Guide](https://github.com/doublegate/ProRT-IP/blob/main/docs/04-IMPLEMENTATION-GUIDE.md) - Code patterns
- [API Reference](https://github.com/doublegate/ProRT-IP/blob/main/docs/05-API-REFERENCE.md) - Function documentation
- [Performance Guide](https://github.com/doublegate/ProRT-IP/blob/main/docs/07-PERFORMANCE.md) - Optimization techniques
- [Security](https://github.com/doublegate/ProRT-IP/blob/main/docs/08-SECURITY.md) - Security implementation

---

## 🔒 Security

This is a **security research tool** intended for:
- Penetration testing
- Network security auditing
- Educational purposes
- Red team operations

**Always obtain proper authorization before scanning networks.**

### Responsible Use

- ✅ **DO:** Use on networks you own or have explicit permission to test
- ✅ **DO:** Use for educational and research purposes
- ✅ **DO:** Report vulnerabilities responsibly
- ❌ **DON'T:** Use on networks without authorization
- ❌ **DON'T:** Use for malicious purposes
- ❌ **DON'T:** Violate laws or regulations

See [SECURITY.md](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md) for complete responsible use guidelines.

---

## 📝 Full Changelog

### [0.3.0] - 2025-10-08

#### Added
- Batch packet sending with sendmmsg syscall (30-50% performance improvement at 1M+ pps)
- CDN/WAF detection for 8 major providers (Cloudflare, Akamai, Fastly, CloudFront, Google, Azure, Imperva, Sucuri)
- Decoy scanning support (up to 256 decoys for stealth attribution hiding)
- GitHub Actions CI/CD workflows (ci.yml, release.yml, dependency-review.yml, codeql.yml)
- Multi-platform automated release builds (Linux gnu/musl, Windows, macOS)
- CI/CD badges in README.md
- Workflow documentation (.github/workflows/README.md)

#### Fixed
- Activated and fixed 4 previously ignored doc-tests (now 551 tests total)
- Implemented Display trait for Ipv4Cidr (proper Rust idiom)
- Fixed bool comparison clippy warnings
- Resolved unused field warnings in batch_sender and decoy_scanner

#### Changed
- Updated all version references to v0.3.0 across codebase
- Enhanced documentation with CI/CD information
- Optimized workflow caching (3-tier: registry, index, build)

#### Performance
- 30-50% improvement at 1M+ pps with sendmmsg batching on Linux
- O(log n) CDN detection with binary search on sorted CIDR ranges
- Cargo caching: 50-80% faster CI/CD workflow runs

#### Testing
- Total tests: 551 (100% pass rate)
- Added tests for batch sending, CDN detection, decoy scanning
- MSRV verification (Rust 1.82+)
- Multi-platform CI: Linux, Windows, macOS

See [CHANGELOG.md](https://github.com/doublegate/ProRT-IP/blob/main/CHANGELOG.md) for complete version history.

---

## 🤝 Contributing

Contributions welcome! We're looking for:

- **Bug reports** - Found an issue? Report it!
- **Feature requests** - Have an idea? Share it!
- **Code contributions** - PRs welcome!
- **Documentation** - Improvements always appreciated

### Pull Request Requirements

All pull requests must pass:
- ✅ Format check (`cargo fmt`)
- ✅ Clippy lint (`cargo clippy -- -D warnings`)
- ✅ Tests on Linux, Windows, macOS
- ✅ Security audit (`cargo audit`)
- ✅ MSRV check (Rust 1.82+)

See [CONTRIBUTING.md](https://github.com/doublegate/ProRT-IP/blob/main/CONTRIBUTING.md) for complete guidelines.

---

## 📄 License

**GPL-3.0** - This project is licensed under the GNU General Public License v3.0.

See [LICENSE](https://github.com/doublegate/ProRT-IP/blob/main/LICENSE) for full license text.

### What This Means

- ✅ You can use, modify, and distribute this software
- ✅ You can use it for commercial purposes
- ✅ You must disclose source code for derivatives
- ✅ You must use the same license for derivatives
- ✅ You must state changes made to the code

---

## 🔗 Links

- **Repository:** https://github.com/doublegate/ProRT-IP
- **Issues:** https://github.com/doublegate/ProRT-IP/issues
- **Discussions:** https://github.com/doublegate/ProRT-IP/discussions
- **Releases:** https://github.com/doublegate/ProRT-IP/releases
- **Actions:** https://github.com/doublegate/ProRT-IP/actions
- **Security:** https://github.com/doublegate/ProRT-IP/security

---

## 🙏 Acknowledgments

### Reference Implementations

This project was inspired by and learned from:

- **Masscan** - Ultra-fast stateless scanning architecture
- **RustScan** - Modern Rust scanner with excellent async patterns
- **Nmap** - Comprehensive detection systems and probe databases
- **ZMap** - Internet-scale scanning techniques
- **naabu** - Fast SYN scanner and CDN detection

### Rust Ecosystem

Built with these excellent crates:

- **tokio** - Async runtime
- **pnet** - Packet manipulation
- **clap** - CLI parsing
- **sqlx** - Database layer
- **crossbeam** - Lock-free data structures
- **governor** - Rate limiting
- And many more - see [Cargo.toml](https://github.com/doublegate/ProRT-IP/blob/main/Cargo.toml)

---

## 📞 Support

Need help? Here's how to get support:

1. **Documentation** - Check the [docs/](https://github.com/doublegate/ProRT-IP/tree/main/docs) directory
2. **FAQ** - See [docs/09-FAQ.md](https://github.com/doublegate/ProRT-IP/blob/main/docs/09-FAQ.md)
3. **Issues** - Report bugs or request features on [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
4. **Discussions** - Ask questions in [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)

See [SUPPORT.md](https://github.com/doublegate/ProRT-IP/blob/main/SUPPORT.md) for complete support resources.

---

**ProRT-IP WarScan v0.3.0** - Modern network scanning at Masscan speed with Nmap detection depth. 🚀
