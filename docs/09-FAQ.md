# ProRT-IP WarScan: FAQ and Troubleshooting

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [General Questions](#general-questions)
2. [Installation and Setup](#installation-and-setup)
3. [Usage Questions](#usage-questions)
4. [Performance Questions](#performance-questions)
5. [Common Errors](#common-errors)
6. [Troubleshooting Guide](#troubleshooting-guide)

---

## General Questions

### What is ProRT-IP WarScan?

ProRT-IP WarScan is a modern network scanner written in Rust that combines the speed of tools like Masscan and ZMap with the comprehensive detection capabilities of Nmap. It's designed for penetration testers and security professionals who need fast, accurate network reconnaissance.

### Why another network scanner?

- **Speed:** 10-100x faster than traditional scanners while maintaining accuracy
- **Safety:** Memory-safe Rust implementation prevents entire classes of vulnerabilities
- **Modern:** Built from the ground up with async I/O, modern protocols, and current best practices
- **Open Source:** GPLv3 license encourages community contributions and transparency

### How does it compare to Nmap?

| Feature | Nmap | WarScan |
|---------|------|---------|
| **Speed** | ~300K pps max | 1M+ pps stateless, 50K+ pps stateful |
| **Memory Safety** | C (manual memory management) | Rust (compile-time guarantees) |
| **Service Detection** | 1000+ services | 500+ services (growing) |
| **OS Fingerprinting** | 2600+ signatures | Compatible with Nmap DB |
| **Maturity** | 25+ years | New project |

WarScan is **not** a replacement for Nmap in all scenarios. Nmap's NSE scripting engine and decades of fingerprints remain unmatched. WarScan excels at fast, large-scale scans and provides a modern, safe alternative.

### Is this legal to use?

**You must have explicit authorization to scan networks you do not own.** Unauthorized network scanning may be illegal in your jurisdiction and could violate computer fraud laws.

Legitimate use cases:
- Scanning your own networks and systems
- Authorized penetration testing engagements
- Bug bounty programs with explicit network scanning permission
- Security research on isolated lab environments

**Always obtain written permission before scanning networks.**

### What platforms are supported?

- **Linux:** Full support (recommended platform)
- **Windows:** Full support via Npcap (requires Administrator privileges)
- **macOS:** Full support via BPF (requires admin or group membership)
- **BSD:** Planned (FreeBSD, OpenBSD, NetBSD)

---

## Installation and Setup

### Q: I get "libpcap not found" during build

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
Download and install Npcap from https://npcap.com/dist/npcap-1.70.exe

### Q: Build fails with OpenSSL errors

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

### Q: How do I run without root/sudo?

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

### Q: "Permission denied" when creating raw socket

You need elevated privileges for raw packet access. See previous question for platform-specific solutions.

Alternatively, use TCP connect scan (slower but requires no privileges):
```bash
./prtip -sT -p 80,443 target.com
```

---

## Usage Questions

### Q: What's the fastest way to scan a /24 network?

```bash
# Fast stateless SYN scan of common ports
prtip -sS -p 80,443,22,21,25,3306,3389 --max-rate 100000 192.168.1.0/24

# Or use preset port list
prtip -sS --top-ports 100 --max-rate 100000 192.168.1.0/24
```

### Q: How do I scan all 65535 ports?

```bash
# Full port scan (will take time)
prtip -sS -p- 192.168.1.1

# Faster with aggressive timing
prtip -sS -p- -T4 192.168.1.1

# Even faster with stateless mode
prtip --stateless -p- 192.168.1.1
```

**Note:** Full port scans take 10-30 minutes depending on timing and network conditions.

### Q: How do I detect service versions?

```bash
# Service detection on discovered ports
prtip -sS -sV -p 1-1000 target.com

# Aggressive service detection (more probes)
prtip -sV --version-intensity 9 target.com
```

### Q: How do I perform OS fingerprinting?

```bash
# OS detection (requires root/capabilities)
prtip -sS -O target.com

# Aggressive OS detection with service versions
prtip -sS -O -sV -A target.com
```

### Q: Can I save results to a file?

```bash
# JSON output
prtip -sS -p 80,443 target.com -oJ results.json

# XML output (Nmap-compatible)
prtip -sS -p 80,443 target.com -oX results.xml

# All output formats
prtip -sS -p 80,443 target.com -oA results
# Creates: results.txt, results.json, results.xml
```

### Q: How do I resume an interrupted scan?

```bash
# Save scan state periodically
prtip -sS -p- --resume-file /tmp/scan.state target.com

# If interrupted, resume from last checkpoint
prtip --resume /tmp/scan.state
```

---

## Performance Questions

### Q: Why is my scan slow?

Common causes:
1. **Timing template too conservative:** Try `-T4` or `-T5`
2. **No privileges (using connect scan):** Use sudo or grant capabilities
3. **Network latency:** Increase `--max-rtt-timeout`
4. **Rate limiting:** Increase `--max-rate` (default: 100K pps)
5. **Single target:** Concurrent scanning helps; scan multiple targets

Example optimization:
```bash
# Slow (default conservative settings)
prtip -sS -p 1-1000 target.com

# Fast (aggressive settings)
prtip -sS -p 1-1000 -T5 --max-rate 500000 target.com
```

### Q: How many packets per second can WarScan achieve?

Depends on mode and hardware:
- **Stateless mode:** 1,000,000+ pps on modern hardware (10GbE + 16+ cores)
- **Stateful SYN scan:** 50,000-100,000 pps
- **TCP connect scan:** 1,000-5,000 pps (limited by OS)
- **Service detection:** 100-500 ports/second (probe-dependent)

### Q: Does scanning faster improve performance?

**Not always!** Excessive rates cause:
- **Packet loss:** Network congestion drops packets, requiring retransmissions
- **IDS/IPS blocking:** Security devices may rate-limit or block
- **Incomplete results:** Slow servers may not respond to burst traffic

**Recommendation:** Start with default rates, increase gradually while monitoring accuracy.

### Q: Can I distribute scanning across multiple machines?

**Currently:** No built-in support (planned for future release)

**Workaround:** Manually split targets:
```bash
# Machine 1
prtip -sS -p- 10.0.0.0/25

# Machine 2
prtip -sS -p- 10.0.128.0/25
```

---

## Common Errors

### Error: "Address already in use"

**Cause:** Another scan or process is using the same source port

**Solution:**
```bash
# Let WarScan choose random source ports (default)
prtip -sS -p 80 target.com

# Or specify different source port range
prtip -sS --source-port 50000-60000 -p 80 target.com
```

### Error: "Too many open files"

**Cause:** OS file descriptor limit too low for large scans

**Solution:**
```bash
# Check current limit
ulimit -n

# Increase temporarily (until reboot)
ulimit -n 65535

# Increase permanently (Linux)
echo "* soft nofile 65535" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65535" | sudo tee -a /etc/security/limits.conf
```

### Error: "Cannot create raw socket: Operation not permitted"

**Cause:** Insufficient privileges for raw packet access

**Solution:** See ["How do I run without root/sudo?"](#q-how-do-i-run-without-rootsudo) above

### Error: "Npcap not found" (Windows)

**Cause:** Npcap not installed or not in API-compatible mode

**Solution:**
1. Download Npcap: https://npcap.com/dist/npcap-1.70.exe
2. During installation, check "Install Npcap in WinPcap API-compatible mode"
3. Restart terminal/IDE after installation

### Error: "No route to host"

**Cause:** Target is unreachable (network configuration issue)

**Troubleshooting:**
```bash
# Verify connectivity
ping target.com

# Check routing
traceroute target.com

# Try different scan type
prtip -sT -Pn -p 80 target.com  # Skip ping, use connect scan
```

---

## Troubleshooting Guide

### Debug Mode

Enable verbose logging to diagnose issues:

```bash
# Basic debug info
RUST_LOG=info prtip -sS -p 80 target.com

# Detailed debug info
RUST_LOG=debug prtip -sS -p 80 target.com

# Maximum verbosity (very noisy)
RUST_LOG=trace prtip -sS -p 80 target.com
```

### Packet Capture Verification

Verify packets are actually being sent/received:

```bash
# On Linux
sudo tcpdump -i eth0 'tcp[tcpflags] & (tcp-syn) != 0 and dst host target.com'

# Run scan in another terminal
prtip -sS -p 80 target.com

# You should see SYN packets in tcpdump output
```

### Performance Profiling

If scans are unexpectedly slow:

```bash
# Monitor CPU usage
htop

# Check for errors
RUST_LOG=warn prtip -sS -p 80 target.com 2>&1 | grep -i error

# Network interface stats (Linux)
watch -n 1 'ifconfig eth0 | grep -E "(RX|TX) packets"'
```

### Memory Issues

Monitor memory usage during large scans:

```bash
# Monitor in real-time
watch -n 1 'ps aux | grep prtip'

# Or use time command
/usr/bin/time -v prtip --stateless -p 80 0.0.0.0/0
```

### False Positives/Negatives

If results seem incorrect:

1. **Verify with another tool:**
   ```bash
   # Cross-check with Nmap
   nmap -sS -p 80,443 target.com
   ```

2. **Try different scan type:**
   ```bash
   # SYN scan might be filtered, try ACK
   prtip -sA -p 80 target.com
   ```

3. **Disable optimizations:**
   ```bash
   # Slow but accurate
   prtip -sS -T0 --max-retries 5 -p 80 target.com
   ```

4. **Check for firewalls:**
   ```bash
   # Firewall detection
   prtip -sA -p 1-1000 target.com
   ```

### Getting Help

If you encounter issues not covered here:

1. **Check documentation:** `docs/` directory or online docs
2. **Search existing issues:** GitHub Issues
3. **Enable debug logging:** `RUST_LOG=debug` and include output in issue report
4. **Provide reproduction steps:** Exact command, target (if safe to share), error output
5. **System info:** OS, version, network setup

**Issue Template:**
```markdown
### Description
Brief description of the issue

### Command
```bash
prtip -sS -p 80 target.com
```

### Expected Behavior
What you expected to happen

### Actual Behavior
What actually happened

### Debug Output
```
RUST_LOG=debug prtip ... output here
```

### System Information
- OS: Ubuntu 22.04
- WarScan version: 1.0.0
- Rust version: 1.70.0
- Network: 10GbE, local network
```

---

## Best Practices

### 1. Start Small

```bash
# Test on single host first
prtip -sS -p 80 single-host.com

# Then expand to network
prtip -sS -p 80 192.168.1.0/24
```

### 2. Use Appropriate Timing

```bash
# Home network: fast
prtip -sS -T4 -p 80 192.168.1.0/24

# Corporate network: balanced
prtip -sS -T3 -p 80 10.0.0.0/24

# Internet scan: conservative (avoids triggering IDS)
prtip -sS -T2 -p 80 target.com
```

### 3. Save Results Incrementally

```bash
# Stream results to database (doesn't wait for scan completion)
prtip -sS -p- --output-db scans.db --stream-results target.com
```

### 4. Monitor Progress

```bash
# Enable progress indicator
prtip -sS -p- --progress target.com

# Or use TUI for real-time visualization
prtip --tui -sS -p- target.com
```

---

## Next Steps

- Review [Architecture](00-ARCHITECTURE.md) for system design
- See [Development Setup](03-DEV-SETUP.md) for build instructions
- Consult [Security Guide](08-SECURITY.md) for safe usage practices
- Check [Performance Guide](07-PERFORMANCE.md) for optimization tips

