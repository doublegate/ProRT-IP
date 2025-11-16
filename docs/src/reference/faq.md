# Frequently Asked Questions

Common questions and answers about ProRT-IP usage, troubleshooting, and best practices.

## General Questions

### What is ProRT-IP?

ProRT-IP is a modern network scanner written in Rust that combines the speed of tools like Masscan and ZMap with the comprehensive detection capabilities of Nmap. It's designed for penetration testers and security professionals who need fast, accurate network reconnaissance.

**Key Features:**
- **Speed:** 1M+ pps stateless scanning, 50K+ pps stateful
- **Safety:** Memory-safe Rust implementation
- **Detection:** Service version detection, OS fingerprinting
- **Modern:** Async I/O, modern protocols, current best practices
- **Open Source:** GPLv3 license

### How does ProRT-IP compare to Nmap?

| Feature | Nmap | ProRT-IP |
|---------|------|----------|
| **Speed** | ~300K pps max | 1M+ pps stateless, 50K+ pps stateful |
| **Memory Safety** | C (manual memory) | Rust (compile-time guarantees) |
| **Service Detection** | 1000+ services | 500+ services (growing) |
| **OS Fingerprinting** | 2600+ signatures | Compatible with Nmap DB |
| **Maturity** | 25+ years | New project |
| **Scripting** | NSE (Lua) | Lua plugin system |

ProRT-IP excels at fast, large-scale scans and provides a modern, safe alternative. Nmap's NSE scripting engine and decades of fingerprints remain unmatched for deep inspection.

### Is network scanning legal?

**You must have explicit authorization to scan networks you do not own.** Unauthorized network scanning may be illegal in your jurisdiction and could violate computer fraud laws.

**Legitimate use cases:**
- Scanning your own networks and systems
- Authorized penetration testing engagements
- Bug bounty programs with explicit network scanning permission
- Security research on isolated lab environments

**Always obtain written permission before scanning networks.**

### What platforms are supported?

| Platform | Support Level | Notes |
|----------|--------------|-------|
| **Linux** | Full support | Recommended platform |
| **Windows** | Full support | Requires Npcap + Administrator privileges |
| **macOS** | Full support | Requires admin or BPF group membership |
| **BSD** | Planned | FreeBSD, OpenBSD, NetBSD |

See [Platform Support](../features/platform-support.md) for detailed installation instructions.

### Why another network scanner?

**Modern Architecture:**
- Async I/O with Tokio runtime
- Zero-copy packet processing
- Lock-free concurrent data structures

**Safety First:**
- Memory-safe Rust prevents buffer overflows, use-after-free, data races
- Compile-time guarantees eliminate entire vulnerability classes
- Comprehensive test suite (2,111 tests, 54.92% coverage)

**Performance:**
- 10-100x faster than traditional scanners for large-scale scans
- Adaptive parallelism scales with available hardware
- Stream-to-disk results prevent memory exhaustion

## Installation and Setup

### "libpcap not found" during build

Install the platform-specific libpcap development package:

**Linux:**
```bash
# Debian/Ubuntu
sudo apt install libpcap-dev

# Fedora/RHEL
sudo dnf install libpcap-devel

# Arch
sudo pacman -S libpcap
```

**macOS:**
```bash
brew install libpcap
```

**Windows:**
Download and install Npcap from https://npcap.com/

### Build fails with OpenSSL errors

**Linux:**
```bash
sudo apt install libssl-dev pkg-config  # Debian/Ubuntu
sudo dnf install openssl-devel          # Fedora
```

**macOS:**
```bash
brew install openssl@3
export PKG_CONFIG_PATH="/usr/local/opt/openssl@3/lib/pkgconfig"
```

**Windows:**
Use `rustls` feature instead:
```bash
cargo build --no-default-features --features rustls
```

### How do I run without root/sudo?

**Linux (Recommended):**
```bash
# Grant capabilities to binary
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Now run without sudo
./target/release/prtip [args]
```

**macOS:**
```bash
# Add yourself to access_bpf group
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf

# Logout and login again for group membership to take effect
```

**Windows:**
Must run terminal as Administrator (no alternative for raw packet access).

**Alternative:**
Use TCP connect scan (slower but requires no privileges):
```bash
./prtip -sT -p 80,443 target.com
```

### "Permission denied" when creating raw socket

You need elevated privileges for raw packet access. See previous question for platform-specific solutions.

TCP connect scan (`-sT`) does not require elevated privileges but is slower:
```bash
prtip -sT -p 80,443 target.com
```

## Usage Questions

### What's the fastest way to scan a /24 network?

**Common ports (fast):**
```bash
prtip -sS -p 80,443,22,21,25,3306,3389 --max-rate 100000 192.168.1.0/24
```

**Top 100 ports:**
```bash
prtip -sS --top-ports 100 --max-rate 100000 192.168.1.0/24
```

**Balanced (common ports + service detection):**
```bash
prtip -sS -sV --top-ports 100 -T4 192.168.1.0/24
```

### How do I scan all 65535 ports?

**Default (balanced):**
```bash
prtip -sS -p- 192.168.1.1
```

**Fast (aggressive timing):**
```bash
prtip -sS -p- -T4 192.168.1.1
```

**Fastest (stateless mode):**
```bash
prtip --stateless -p- 192.168.1.1
```

**Note:** Full port scans take 10-30 minutes depending on timing and network conditions.

### How do I detect service versions?

**Basic service detection:**
```bash
prtip -sS -sV -p 1-1000 target.com
```

**Aggressive service detection (more probes):**
```bash
prtip -sV --version-intensity 9 target.com
```

**Light service detection (faster):**
```bash
prtip -sV --version-intensity 2 target.com
```

See [Service Detection](../features/service-detection.md) for details on probe intensity levels.

### How do I perform OS fingerprinting?

**Basic OS detection:**
```bash
prtip -sS -O target.com
```

**Aggressive (OS + service versions):**
```bash
prtip -sS -O -sV -A target.com
```

**Requires:**
- At least one open port
- At least one closed port
- Elevated privileges (root/capabilities)

See [OS Fingerprinting](../features/os-detection.md) for detailed information.

### Can I save results to a file?

**JSON output:**
```bash
prtip -sS -p 80,443 target.com -oJ results.json
```

**XML output (Nmap-compatible):**
```bash
prtip -sS -p 80,443 target.com -oX results.xml
```

**All output formats:**
```bash
prtip -sS -p 80,443 target.com -oA results
# Creates: results.txt, results.json, results.xml
```

**Database storage:**
```bash
prtip -sS -p 80,443 target.com --with-db --database scans.db
```

See [Output Formats](../user-guide/output-formats.md) and [Database Storage](../features/database-storage.md).

### How do I resume an interrupted scan?

**Save scan state periodically:**
```bash
prtip -sS -p- --resume-file /tmp/scan.state target.com
```

**Resume from last checkpoint:**
```bash
prtip --resume /tmp/scan.state
```

**Note:** Resume feature is available for SYN, Connect, and UDP scans. Service detection and OS fingerprinting states are not preserved.

## Performance Questions

### Why is my scan slow?

**Common causes and solutions:**

| Cause | Solution |
|-------|----------|
| Timing too conservative | Try `-T4` or `-T5` |
| No privileges (connect scan) | Use sudo or grant capabilities |
| Network latency | Increase `--max-rtt-timeout` |
| Rate limiting | Increase `--max-rate` (default: 100K pps) |
| Single target | Scan multiple targets concurrently |

**Example optimization:**
```bash
# Slow (default conservative settings)
prtip -sS -p 1-1000 target.com

# Fast (aggressive settings)
prtip -sS -p 1-1000 -T5 --max-rate 500000 target.com
```

See [Performance Tuning](../advanced/performance-tuning.md) for comprehensive optimization guide.

### How many packets per second can ProRT-IP achieve?

Performance depends on mode and hardware:

| Mode | Packets/Second | Notes |
|------|----------------|-------|
| **Stateless** | 1,000,000+ pps | 10GbE + 16+ cores |
| **Stateful SYN** | 50,000-100,000 pps | Adaptive parallelism |
| **TCP Connect** | 1,000-5,000 pps | OS limit |
| **Service Detection** | 100-500 ports/sec | Probe-dependent |
| **OS Fingerprinting** | 50-100 hosts/min | 16-probe sequence |

See [Performance Characteristics](../advanced/performance-characteristics.md) for detailed benchmarks.

### Does scanning faster improve performance?

**Not always!** Excessive rates cause:

**Problems:**
- **Packet loss:** Network congestion drops packets, requiring retransmissions
- **IDS/IPS blocking:** Security devices may rate-limit or block
- **Incomplete results:** Slow servers may not respond to burst traffic
- **Firewall rate limiting:** Many firewalls drop excess packets

**Recommendation:**
1. Start with default rates (100K pps)
2. Increase gradually while monitoring accuracy
3. Compare results with conservative timing (`-T2`)
4. Use `--max-retries` to handle packet loss

### Can I distribute scanning across multiple machines?

**Currently:** No built-in support (planned for future release)

**Workaround:** Manually split targets:
```bash
# Machine 1
prtip -sS -p- 10.0.0.0/25

# Machine 2
prtip -sS -p- 10.0.128.0/25
```

**Future:** Distributed scanning coordinator with automatic target distribution, result aggregation, and failure recovery.

## Common Errors

### "Address already in use"

**Cause:** Another scan or process is using the same source port

**Solution:**
```bash
# Let ProRT-IP choose random source ports (default)
prtip -sS -p 80 target.com

# Or specify different source port range
prtip -sS --source-port 50000-60000 -p 80 target.com
```

### "Too many open files"

**Cause:** OS file descriptor limit too low for large scans

**Check current limit:**
```bash
ulimit -n
```

**Increase temporarily (until reboot):**
```bash
ulimit -n 65535
```

**Increase permanently (Linux):**
```bash
echo "* soft nofile 65535" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65535" | sudo tee -a /etc/security/limits.conf
# Logout and login for changes to take effect
```

### "Cannot create raw socket: Operation not permitted"

**Cause:** Insufficient privileges for raw packet access

**Solution:** See ["How do I run without root/sudo?"](#how-do-i-run-without-rootsudo) above.

**Quick fix:**
```bash
# Linux
sudo setcap cap_net_raw,cap_net_admin=eip ./prtip

# macOS
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf

# Windows
# Right-click terminal → "Run as Administrator"
```

### "Npcap not found" (Windows)

**Cause:** Npcap not installed or not in API-compatible mode

**Solution:**
1. Download Npcap: https://npcap.com/
2. During installation, check "Install Npcap in WinPcap API-compatible mode"
3. Restart terminal/IDE after installation

**Verify installation:**
```powershell
# Check if Npcap DLLs are in PATH
where wpcap.dll
where Packet.dll
```

### "No route to host"

**Cause:** Target is unreachable (network configuration issue)

**Troubleshooting:**
```bash
# Verify connectivity
ping target.com

# Check routing
traceroute target.com  # Linux/macOS
tracert target.com     # Windows

# Try different scan type
prtip -sT -Pn -p 80 target.com  # Skip ping, use connect scan
```

**Common causes:**
- Firewall blocking ICMP
- No route to target network
- Target is down
- Incorrect network configuration

## Troubleshooting

### Enable Debug Logging

**Basic debug info:**
```bash
RUST_LOG=info prtip -sS -p 80 target.com
```

**Detailed debug info:**
```bash
RUST_LOG=debug prtip -sS -p 80 target.com
```

**Maximum verbosity (very noisy):**
```bash
RUST_LOG=trace prtip -sS -p 80 target.com
```

**Module-specific logging:**
```bash
RUST_LOG=prtip_scanner=debug,prtip_network=info prtip -sS -p 80 target.com
```

### Verify Packet Transmission

**Linux:**
```bash
# Capture outgoing SYN packets
sudo tcpdump -i eth0 'tcp[tcpflags] & (tcp-syn) != 0 and dst host target.com'

# Run scan in another terminal
prtip -sS -p 80 target.com
```

**macOS:**
```bash
sudo tcpdump -i en0 'tcp[tcpflags] & (tcp-syn) != 0 and dst host target.com'
```

**Windows (Npcap):**
```powershell
# Use Wireshark or tcpdump equivalent
```

### Performance Profiling

**Monitor CPU usage:**
```bash
htop  # or top on macOS
```

**Check for errors:**
```bash
RUST_LOG=warn prtip -sS -p 80 target.com 2>&1 | grep -i error
```

**Network interface stats (Linux):**
```bash
watch -n 1 'ifconfig eth0 | grep -E "(RX|TX) packets"'
```

**Measure memory usage:**
```bash
/usr/bin/time -v prtip --stateless -p 80 0.0.0.0/0
```

### Validate Results

**Cross-check with Nmap:**
```bash
nmap -sS -p 80,443 target.com
```

**Try different scan type:**
```bash
# SYN scan might be filtered, try ACK
prtip -sA -p 80 target.com
```

**Slow but accurate:**
```bash
prtip -sS -T0 --max-retries 5 -p 80 target.com
```

**Firewall detection:**
```bash
prtip -sA -p 1-1000 target.com
```

### Getting Help

**Before opening an issue:**
1. Check [Troubleshooting Guide](troubleshooting.md)
2. Search [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
3. Review documentation for your use case

**When reporting issues:**
1. Enable debug logging: `RUST_LOG=debug`
2. Provide exact command and output
3. Include system information (OS, version, network setup)
4. Describe expected vs actual behavior

**Issue template:**
```markdown
### Description
Brief description of the issue

### Command
\```bash
prtip -sS -p 80 target.com
\```

### Expected Behavior
What you expected to happen

### Actual Behavior
What actually happened

### Debug Output
\```
RUST_LOG=debug prtip ... output here
\```

### System Information
- OS: Ubuntu 22.04
- ProRT-IP version: 0.5.0
- Rust version: 1.70.0
- Network: 10GbE, local network
```

## Best Practices

### Start Small

Test on single host before scanning large networks:

```bash
# Test on single host first
prtip -sS -p 80 single-host.com

# Then expand to network
prtip -sS -p 80 192.168.1.0/24
```

### Use Appropriate Timing

Match timing template to environment:

```bash
# Home network: fast
prtip -sS -T4 -p 80 192.168.1.0/24

# Corporate network: balanced
prtip -sS -T3 -p 80 10.0.0.0/24

# Internet scan: conservative (avoids triggering IDS)
prtip -sS -T2 -p 80 target.com
```

See [Timing Templates](../user-guide/cli-reference.md#timing-templates) for details on T0-T5.

### Save Results Incrementally

Stream results to database during scan:

```bash
prtip -sS -p- --with-db --database scans.db target.com
```

**Benefits:**
- Results preserved if scan is interrupted
- Real-time analysis of discovered ports
- No memory exhaustion on large scans
- Historical tracking of network changes

### Monitor Progress

**Progress indicator:**
```bash
prtip -sS -p- --progress target.com
```

**TUI for real-time visualization:**
```bash
prtip --live -sS -p- target.com
```

**Verbose output:**
```bash
prtip -v -sS -p- target.com
```

## See Also

- [CLI Reference](../user-guide/cli-reference.md) - Complete command-line reference
- [Troubleshooting Guide](troubleshooting.md) - Detailed troubleshooting procedures
- [Performance Tuning](../advanced/performance-tuning.md) - Optimization strategies
- [Platform Support](../features/platform-support.md) - Platform-specific installation
- [Output Formats](../user-guide/output-formats.md) - Result export formats
