# ProRT-IP WarScan v0.3.0 - Production Release

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
- **Cross-platform:** Linux/Windows/macOS/FreeBSD support
- **CI/CD:** GitHub Actions with multi-platform testing and automated releases

### User Experience
- **Professional CLI:** Cyber-punk ASCII banner with gradient colors
- **Progress tracking:** Real-time statistics with ETA estimation
- **Error categorization:** 7 categories with actionable suggestions
- **Multiple output formats:** JSON, XML, Text, SQLite

## 📊 Statistics

- **Tests:** 551 (100% pass rate)
- **Code:** 10,000+ lines across 4 crates (40+ modules)
- **Dependencies:** Production-ready with security audits passing
- **Platforms:** Linux, Windows, macOS, FreeBSD
- **MSRV:** Rust 1.70+

## 🆕 What's New in v0.3.0

### Quality Improvements
- Fixed 4 previously ignored doc-tests (now 551 tests total, 100% passing)
- Zero clippy warnings (strict -D warnings mode)
- Self-contained doc-test examples requiring no external files
- Production-ready code snippets in all module documentation

### CI/CD Infrastructure
- **5 GitHub Actions workflows** for automated testing and releases
- **Multi-platform CI:** Linux, Windows, macOS with 3-tier cargo caching
- **Security scanning:** CodeQL, dependency review, cargo audit
- **Automated releases:** Multi-platform binary builds on git tags
- **MSRV verification:** Rust 1.70+ enforced in pipeline

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

## 📥 Installation

### Available Platforms (5 Architectures)

Pre-built binaries are available for the following platforms:

| Platform | Architecture | File | Notes |
|----------|--------------|------|-------|
| **Linux** | x86_64 (glibc) | `prtip-0.3.0-x86_64-unknown-linux-gnu.tar.gz` | Ubuntu, Debian, Fedora, RHEL, etc. |
| **Windows** | x86_64 (Intel/AMD) | `prtip-0.3.0-x86_64-pc-windows-msvc.zip` | Windows 10/11 (Requires [Npcap](https://npcap.com/)) |
| **macOS** | x86_64 (Intel) | `prtip-0.3.0-x86_64-apple-darwin.tar.gz` | Intel-based Macs (macOS 11+) |
| **macOS** | aarch64 (Apple Silicon) | `prtip-0.3.0-aarch64-apple-darwin.tar.gz` | M1/M2/M3/M4 Macs (macOS 11+) |
| **FreeBSD** | x86_64 | `prtip-0.3.0-x86_64-unknown-freebsd.tar.gz` | FreeBSD 12+, pfSense, OPNsense |

### Linux (x86_64)
```bash
# Download
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.3.0/prtip-0.3.0-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar xzf prtip-0.3.0-x86_64-unknown-linux-gnu.tar.gz

# Install (optional)
sudo mv prtip /usr/local/bin/
chmod +x /usr/local/bin/prtip

# Run
prtip --help
```

### Windows (x86_64)
```powershell
# Download
curl -L -o prtip-0.3.0-x86_64-pc-windows-msvc.zip https://github.com/doublegate/ProRT-IP/releases/download/v0.3.0/prtip-0.3.0-x86_64-pc-windows-msvc.zip

# Extract
Expand-Archive prtip-0.3.0-x86_64-pc-windows-msvc.zip

# Install Npcap (required for packet capture)
# Download from: https://npcap.com/

# Run
.\prtip.exe --help
```

### macOS (Intel x86_64)
```bash
# Download
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.3.0/prtip-0.3.0-x86_64-apple-darwin.tar.gz

# Extract
tar xzf prtip-0.3.0-x86_64-apple-darwin.tar.gz

# Install (optional)
sudo mv prtip /usr/local/bin/
chmod +x /usr/local/bin/prtip

# Run with elevated privileges (required for raw sockets)
sudo prtip --help
```

### macOS (Apple Silicon M1/M2/M3/M4)
```bash
# Download
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.3.0/prtip-0.3.0-aarch64-apple-darwin.tar.gz

# Extract
tar xzf prtip-0.3.0-aarch64-apple-darwin.tar.gz

# Install (optional)
sudo mv prtip /usr/local/bin/
chmod +x /usr/local/bin/prtip

# Run with elevated privileges (required for raw sockets)
sudo prtip --help
```

### FreeBSD (x86_64)
```bash
# Download
fetch https://github.com/doublegate/ProRT-IP/releases/download/v0.3.0/prtip-0.3.0-x86_64-unknown-freebsd.tar.gz

# Extract
tar xzf prtip-0.3.0-x86_64-unknown-freebsd.tar.gz

# Install (optional)
sudo mv prtip /usr/local/bin/
chmod +x /usr/local/bin/prtip

# Run with elevated privileges (required for raw sockets)
sudo prtip --help
```

### Build from Source (All Platforms)
```bash
# Clone repository
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
git checkout v0.3.0

# Build release binary
cargo build --release

# Binary location
./target/release/prtip --help
```

**Requirements:**
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- libpcap development headers (Linux: `libpcap-dev`, macOS: included, Windows: [Npcap SDK](https://npcap.com/#download))
- pkg-config (Linux: `pkg-config`, macOS: `pkgconf`)

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

### Full Port Range Scan (All 65,535 ports)
```bash
prtip -sS -p- scanme.nmap.org
```

## 📚 Documentation

- [README](https://github.com/doublegate/ProRT-IP/blob/main/README.md)
- [Architecture](https://github.com/doublegate/ProRT-IP/blob/main/docs/00-ARCHITECTURE.md)
- [Implementation Guide](https://github.com/doublegate/ProRT-IP/blob/main/docs/04-IMPLEMENTATION-GUIDE.md)
- [API Reference](https://github.com/doublegate/ProRT-IP/blob/main/docs/05-API-REFERENCE.md)
- [Performance Guide](https://github.com/doublegate/ProRT-IP/blob/main/docs/07-PERFORMANCE.md)
- [CI/CD Workflows](https://github.com/doublegate/ProRT-IP/blob/main/.github/workflows/README.md)

## 🔒 Security

This is a **security research tool** intended for:
- Penetration testing
- Network security auditing
- Educational purposes
- Red team operations

**Always obtain proper authorization before scanning networks.**

See [SECURITY.md](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md) for responsible use guidelines.

## 📝 Full Changelog

See [CHANGELOG.md](https://github.com/doublegate/ProRT-IP/blob/main/CHANGELOG.md) for complete version history.

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](https://github.com/doublegate/ProRT-IP/blob/main/CONTRIBUTING.md) for guidelines.

All pull requests must pass:
- Format check (cargo fmt)
- Linting (cargo clippy)
- Tests on Linux, Windows, macOS
- Security audit (cargo audit)
- MSRV check (Rust 1.70+)

## 📄 License

GPL-3.0 - See [LICENSE](https://github.com/doublegate/ProRT-IP/blob/main/LICENSE)

---

**Repository:** https://github.com/doublegate/ProRT-IP  
**CI/CD:** [GitHub Actions](https://github.com/doublegate/ProRT-IP/actions)  
**Issues:** https://github.com/doublegate/ProRT-IP/issues  
**Discussions:** https://github.com/doublegate/ProRT-IP/discussions

🤖 Built with GitHub Actions
