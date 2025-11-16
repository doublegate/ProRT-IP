# Troubleshooting Guide

Comprehensive troubleshooting procedures for ProRT-IP issues across platforms, performance problems, and common errors.

## Common Issues

### Permission Denied Errors

**Symptoms:**
```
Error: Permission denied (os error 13)
Error: Operation not permitted (os error 1)
Error: Failed to create raw socket
```

**Cause:** Raw sockets require elevated privileges on most operating systems. This is a security measure to prevent unauthorized packet manipulation.

**Solutions:**

**1. Run with sudo (testing):**
```bash
sudo prtip -sS -p 80,443 192.168.1.1
```

**2. Set capabilities (Linux production):**
```bash
# Build release binary
cargo build --release

# Grant raw socket capability
sudo setcap cap_net_raw,cap_net_admin+ep ./target/release/prtip

# Run without sudo
./target/release/prtip -sS -p 80,443 192.168.1.1
```

**3. Use TCP Connect scan (no privileges required):**
```bash
# Connect scan works without elevated privileges
prtip -sT -p 80,443 192.168.1.1
# Note: Slower and more detectable than SYN scan
```

**4. Add user to specific group (Linux):**
```bash
sudo usermod -a -G netdev $USER
# Log out and back in for group membership to take effect
```

**Verification:**
```bash
# Check capabilities (Linux)
getcap ./target/release/prtip
# Expected: cap_net_admin,cap_net_raw+ep
```

---

### Packet Capture Failures

**Symptoms:**
```
Error: No suitable device found
Error: Failed to open capture device
Error: Device does not exist
PCAPNG capture failed: Interface not found
```

**Causes:**
- Network interface doesn't exist
- Interface name is incorrect
- Missing packet capture drivers (Windows/macOS)
- Permission issues

**Solutions:**

**1. List available interfaces:**
```bash
# Linux
ip link show
ip addr show

# macOS
ifconfig
networksetup -listallhardwareports

# Windows
ipconfig /all
```

**2. Specify interface explicitly:**
```bash
# Linux
prtip -e eth0 -sS 192.168.1.1

# macOS
prtip -e en0 -sS 192.168.1.1

# Windows
prtip -e "Ethernet" -sS 192.168.1.1
```

**3. Install packet capture drivers (Windows):**
```bash
# Download Npcap from https://npcap.com/
# Choose "WinPcap API-compatible mode" during installation
```

**4. Install ChmodBPF (macOS):**
```bash
# Install ChmodBPF for non-root packet capture
brew install --cask wireshark

# Or manually:
sudo chown $USER:admin /dev/bpf*
sudo chmod 600 /dev/bpf*
```

**5. Check interface status:**
```bash
# Ensure interface is UP
sudo ip link set eth0 up

# Verify interface has IP address
ip addr show eth0
```

**Common Interface Names:**

| Platform | Common Names | Notes |
|----------|--------------|-------|
| Linux | `eth0`, `ens33`, `enp3s0`, `wlan0` | Modern systemd uses predictable names |
| macOS | `en0`, `en1`, `lo0` | en0 is usually primary interface |
| Windows | `Ethernet`, `Wi-Fi`, `Local Area Connection` | Use full name with quotes |

---

### Network Timeout Issues

**Symptoms:**
```
Error: Operation timed out
Scan completed but no results
Warning: High timeout rate (>50%)
```

**Causes:**
- Target is down or blocking probes
- Network congestion
- Firewall dropping packets
- Timeout value too low
- Rate limiting too aggressive

**Solutions:**

**1. Increase timeout:**
```bash
# Use paranoid timing for slow/unreliable networks
prtip -T0 -p 80,443 192.168.1.1

# Or specify custom timeout (milliseconds)
prtip --timeout 5000 -p 80,443 192.168.1.1
```

**2. Adjust timing template:**
```bash
# T0 = Paranoid (5 min timeout, very slow)
# T1 = Sneaky (15 sec timeout, slow)
# T2 = Polite (1 sec timeout, medium)
# T3 = Normal (1 sec timeout, default)
# T4 = Aggressive (500ms timeout, fast)
# T5 = Insane (100ms timeout, very fast)

prtip -T2 -p 80,443 192.168.1.1
```

**3. Reduce scan rate:**
```bash
# Limit to 1000 packets/second
prtip --max-rate 1000 -sS 192.168.1.0/24

# Very slow scan (100 pps)
prtip --max-rate 100 -sS 192.168.1.0/24
```

**4. Check target reachability:**
```bash
# Ping target first
ping -c 4 192.168.1.1

# Traceroute to identify routing issues
traceroute 192.168.1.1

# Check if specific ports are filtered
telnet 192.168.1.1 80
```

**5. Verify no firewall interference:**
```bash
# Temporarily disable local firewall (Linux)
sudo ufw disable

# Check iptables rules
sudo iptables -L -v -n

# Windows Firewall
netsh advfirewall show allprofiles
```

**Timeout Recommendations:**

| Scenario | Template | Timeout | Rate |
|----------|----------|---------|------|
| Local network (LAN) | T4-T5 | 100-500ms | 10K-100K pps |
| Remote network (WAN) | T3 | 1000ms | 1K-10K pps |
| Internet scanning | T2-T3 | 1000-2000ms | 100-1K pps |
| Unreliable network | T0-T1 | 5000-15000ms | 10-100 pps |
| IDS/IPS evasion | T0 | 300000ms | 1-10 pps |

---

### Service Detection Problems

**Symptoms:**
```
Port 80: open (service: unknown)
Port 443: open (service: unknown)
Low service detection rate (<30%)
```

**Causes:**
- Service using non-standard port
- Service requires specific handshake
- Service is SSL/TLS wrapped
- Insufficient timeout for service probe
- Service detection disabled

**Solutions:**

**1. Enable service detection:**
```bash
# Basic service detection
prtip -sV -p 80,443 192.168.1.1

# Aggressive service detection
prtip -A -p 80,443 192.168.1.1

# Higher intensity (0-9, default 7)
prtip -sV --version-intensity 9 -p 80,443 192.168.1.1
```

**2. Increase service probe timeout:**
```bash
# Allow more time for service responses
prtip -sV --timeout 5000 -p 80,443 192.168.1.1
```

**3. Enable SSL/TLS detection:**
```bash
# TLS handshake enabled by default in v0.4.0+
prtip -sV -p 443 192.168.1.1

# Disable TLS for performance
prtip -sV --no-tls -p 443 192.168.1.1
```

**4. Manual service verification:**
```bash
# Connect manually and send HTTP request
echo -e "GET / HTTP/1.0\r\n\r\n" | nc 192.168.1.1 80

# SSL/TLS connection
openssl s_client -connect 192.168.1.1:443 -showcerts
```

**5. Check service probe database:**
```bash
# List available probes
prtip --list-probes | grep -i http
# ProRT-IP uses 187 embedded probes by default
```

**Expected Detection Rates:**

| Service Type | Detection Rate | Notes |
|--------------|----------------|-------|
| HTTP/HTTPS | 95-100% | Excellent with TLS support |
| SSH | 90-95% | Banner typically sent immediately |
| FTP | 85-90% | Banner on connection |
| SMTP | 85-90% | Standard greeting |
| DNS | 80-85% | Requires specific queries |
| Database (MySQL, PostgreSQL) | 75-85% | May require authentication |
| Custom/Proprietary | 20-50% | Limited probe coverage |

---

### OS Fingerprinting Issues

**Symptoms:**
```
OS fingerprint: Unknown
OS detection confidence: Low (<30%)
No OS matches found
```

**Causes:**
- Target has strict firewall rules
- Not enough open ports for fingerprinting
- OS not in fingerprint database
- Unusual network stack behavior
- Virtual machine or container

**Solutions:**

**1. Enable OS detection:**
```bash
# Basic OS detection
prtip -O -p 80,443 192.168.1.1

# Aggressive OS detection
prtip -A -p 80,443 192.168.1.1
```

**2. Scan more ports:**
```bash
# OS detection works best with multiple open ports
prtip -O -p- 192.168.1.1

# At minimum, scan common ports
prtip -O -F 192.168.1.1
```

**3. Ensure target is responsive:**
```bash
# Combine with service detection
prtip -A -p 22,80,443 192.168.1.1

# Verify target responds to probes
prtip -sS -p 22,80,443 192.168.1.1
```

**4. Check OS fingerprint database:**
```bash
# ProRT-IP uses 2600+ signatures
# Coverage: Windows, Linux, BSD, macOS, network devices

# Manual OS identification via TTL
# TTL 64 = Linux/Unix
# TTL 128 = Windows
# TTL 255 = Network device (Cisco, etc.)
```

**OS Detection Confidence Levels:**

| Confidence | Meaning | Action |
|------------|---------|--------|
| High (80-100%) | Strong match, reliable | Accept result |
| Medium (50-79%) | Likely match, some uncertainty | Verify with other methods |
| Low (30-49%) | Weak match, multiple possibilities | Manual verification needed |
| Unknown (<30%) | Insufficient data | Scan more ports, check firewall |

---

## Platform-Specific Issues

### Linux

#### AppArmor/SELinux blocking raw sockets

**Symptoms:**
```
Error: Permission denied even with sudo
Error: SELinux is preventing prtip from using raw sockets
```

**Solutions:**
```bash
# Check SELinux status
getenforce

# Temporarily disable (testing only)
sudo setenforce 0

# Create SELinux policy (production)
sudo semanage permissive -a prtip_t

# AppArmor (Ubuntu/Debian)
sudo aa-complain /path/to/prtip
```

#### iptables interfering with scans

**Symptoms:**
```
Unexpected RST packets
Scan results inconsistent
Local firewall blocking responses
```

**Solutions:**
```bash
# Check iptables rules
sudo iptables -L -v -n

# Temporarily disable (testing only)
sudo iptables -P INPUT ACCEPT
sudo iptables -P OUTPUT ACCEPT
sudo iptables -P FORWARD ACCEPT
sudo iptables -F

# Or create exception for prtip
sudo iptables -A OUTPUT -m owner --uid-owner $(id -u) -j ACCEPT
```

#### Socket buffer limits

**Symptoms:**
```
Error: Cannot allocate memory
Warning: Socket buffer size limit reached
High packet loss at high rates
```

**Solutions:**
```bash
# Check current limits
sysctl net.core.rmem_max
sysctl net.core.wmem_max

# Increase socket buffers (requires root)
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728

# Make persistent
echo "net.core.rmem_max=134217728" | sudo tee -a /etc/sysctl.conf
echo "net.core.wmem_max=134217728" | sudo tee -a /etc/sysctl.conf
```

---

### Windows

#### Npcap not installed or outdated

**Symptoms:**
```
Error: The NPF driver isn't running
Error: Failed to open the adapter
PCAPNG capture not working
```

**Solutions:**
1. Download Npcap from https://npcap.com/
2. Run installer as Administrator
3. Choose "Install Npcap in WinPcap API-compatible Mode"
4. Reboot if prompted
5. Verify installation:
```cmd
sc query npcap
```

#### Windows Firewall blocking scans

**Symptoms:**
```
No responses from local targets
Scan hangs or times out
Windows Security alerts
```

**Solutions:**
```powershell
# Check firewall status
netsh advfirewall show allprofiles

# Create exception for prtip
netsh advfirewall firewall add rule name="ProRT-IP" dir=out action=allow program="C:\path\to\prtip.exe"

# Or temporarily disable (testing only)
netsh advfirewall set allprofiles state off
```

#### SYN scan tests fail on loopback

**Symptoms:**
```
4 SYN discovery tests fail on Windows loopback
Test test_discovery_syn_ipv4 ... FAILED
```

**Cause:** This is **expected behavior** on Windows. The Windows network stack doesn't support SYN scanning on loopback (127.0.0.1) due to architectural limitations.

**Solutions:**
```cmd
# Use TCP Connect scan on loopback (works)
prtip -sT -p 80 127.0.0.1

# Use real network interface for SYN scans
prtip -sS -p 80 192.168.1.1

# This is documented and not a bug
```

---

### macOS

#### ChmodBPF not configured

**Symptoms:**
```
Error: You don't have permission to capture on that device
Error: No suitable device found
```

**Solutions:**
```bash
# Install ChmodBPF (easiest via Wireshark)
brew install --cask wireshark

# Or manual configuration
cd /Library/LaunchDaemons
sudo curl -O https://raw.githubusercontent.com/Homebrew/homebrew-cask/master/Casks/wireshark-chmodbpf.rb
sudo launchctl load /Library/LaunchDaemons/ChmodBPF.plist

# Reboot for changes to take effect
sudo reboot
```

#### FIN/NULL/Xmas scans don't work

**Symptoms:**
```
All ports show as open|filtered
No definitive open/closed results
```

**Cause:** macOS and some BSD-based network stacks don't respond to stealth scans as expected. This is a limitation of the OS, not ProRT-IP.

**Solutions:**
```bash
# Use SYN scan instead
prtip -sS -p 80,443 192.168.1.1

# Or TCP Connect scan
prtip -sT -p 80,443 192.168.1.1
```

#### System Integrity Protection (SIP) interference

**Symptoms:**
```
Error: Operation not permitted
Error: Cannot modify network stack
```

**Solutions:**
```bash
# Check SIP status
csrutil status

# SIP must be enabled for security
# Solution: Run with sudo or use TCP Connect scan
sudo prtip -sS -p 80,443 192.168.1.1
```

---

## Performance Issues

### Slow Scanning

**Symptoms:**
- Scan takes much longer than expected
- Progress bar moves very slowly
- Low packet rate (<1000 pps)

**Diagnosis:**
```bash
# Run with verbose output
prtip -sS -vv -p 80,443 192.168.1.0/24

# Check timing template
prtip -T5 -p 80,443 192.168.1.0/24  # Fastest

# Monitor system resources
top  # Linux/macOS
taskmgr  # Windows
```

**Solutions:**

**1. Increase parallelism:**
```bash
# Override default parallelism (num_cpus * 2)
prtip --parallelism 100 -sS 192.168.1.0/24
```

**2. Adjust timing template:**
```bash
# T5 = Insane (fastest, least stealthy)
prtip -T5 -p 80,443 192.168.1.0/24

# Or custom rate
prtip --max-rate 100000 -sS 192.168.1.0/24
```

**3. Disable unnecessary features:**
```bash
# Disable service detection
prtip -sS -p 80,443 192.168.1.0/24  # No -sV

# Disable OS detection
prtip -sS -p 80,443 192.168.1.0/24  # No -O

# Disable TLS handshake
prtip -sV --no-tls -p 443 192.168.1.0/24
```

**4. Use NUMA optimization (multi-socket systems):**
```bash
# Enable NUMA-aware thread pinning
prtip --numa -sS 192.168.1.0/24
# Can provide 30%+ improvement on dual-socket servers
```

**5. Reduce target scope:**
```bash
# Scan fewer ports
prtip -F 192.168.1.0/24  # Top 100 instead of all 65535

# Scan smaller ranges
prtip -sS -p 80,443 192.168.1.0/28  # /28 instead of /24
```

---

### High Memory Usage

**Symptoms:**
```
Warning: Memory usage above 80%
Error: Cannot allocate memory
System becoming unresponsive
OOM killer terminating process
```

**Diagnosis:**
```bash
# Check memory usage
free -h  # Linux
vm_stat  # macOS

# Monitor prtip memory
ps aux | grep prtip
top -p $(pgrep prtip)
```

**Solutions:**

**1. Reduce parallelism:**
```bash
# Lower concurrent operations
prtip --parallelism 10 -sS 192.168.1.0/24
```

**2. Disable PCAPNG capture:**
```bash
# Packet capture uses significant memory
prtip -sS 192.168.1.0/24  # Don't use --packet-capture
```

**3. Stream results to disk:**
```bash
# Don't buffer all results in memory
prtip -sS -oN results.txt 192.168.1.0/24

# Use database export for large scans
prtip -sS --with-db --database results.db 192.168.1.0/24
```

**4. Scan in smaller batches:**
```bash
# Break large scans into chunks
for i in {1..255}; do
  prtip -sS -p 80,443 192.168.1.$i
done
```

**5. Resource monitoring triggers automatic degradation (v0.4.0+):**
```bash
# ProRT-IP automatically reduces memory usage when >80% utilized
# Manual configuration:
prtip --memory-limit 80 -sS 192.168.1.0/24
```

---

### CPU Bottlenecks

**Symptoms:**
- CPU usage at 100%
- Scan slower than network capacity
- High context switching

**Diagnosis:**
```bash
# Check CPU usage
mpstat 1 10  # Linux
top  # macOS
perfmon  # Windows

# Check context switches
vmstat 1 10  # Linux
```

**Solutions:**

**1. Adjust thread count:**
```bash
# Match CPU core count
prtip --threads $(nproc) -sS 192.168.1.0/24

# Or explicitly set
prtip --threads 8 -sS 192.168.1.0/24
```

**2. Enable NUMA optimization:**
```bash
# Pin threads to specific cores
prtip --numa -sS 192.168.1.0/24
```

**3. Reduce packet processing overhead:**
```bash
# Disable service detection
prtip -sS 192.168.1.0/24  # No -sV

# Use SYN scan instead of Connect
prtip -sS 192.168.1.0/24  # Faster than -sT
```

**4. Build with release optimizations:**
```bash
# Ensure using release build
cargo build --release
./target/release/prtip -sS 192.168.1.0/24

# Debug builds are 10-100x slower
```

---

## Output & Export Issues

### Greppable Output Not Parsing

**Symptoms:**
```
Output format is malformed
Cannot parse greppable results
Fields are missing or incorrect
```

**Solutions:**
```bash
# Verify greppable format
prtip -sS -oG results.txt 192.168.1.1
cat results.txt

# Expected format:
# Host: 192.168.1.1 () Status: Up
# Host: 192.168.1.1 () Ports: 80/open/tcp//http///, 443/open/tcp//https///

# Parse with awk
awk '/Ports:/ {print $2, $5}' results.txt
```

### XML Output Invalid

**Symptoms:**
```
XML parsing errors
Invalid XML structure
Missing closing tags
```

**Solutions:**
```bash
# Verify XML output
prtip -sS -oX results.xml 192.168.1.1

# Validate XML
xmllint --noout results.xml

# Common issues:
# - Special characters in banners (automatically escaped)
# - Incomplete scans (use Ctrl+C gracefully, not kill -9)
```

### Database Export Fails

**Symptoms:**
```
Error: Database locked
Error: Cannot create database file
SQLite error: disk I/O error
```

**Solutions:**
```bash
# Check file permissions
ls -la results.db
chmod 644 results.db

# Ensure directory is writable
mkdir -p /tmp/ProRT-IP
prtip -sS --with-db --database /tmp/ProRT-IP/results.db 192.168.1.1

# Check disk space
df -h /tmp

# Verify database is not locked by another process
lsof results.db
```

---

## Database Issues

### Cannot Query Database

**Symptoms:**
```
Error: No such table: scans
Error: Database file is encrypted or is not a database
```

**Solutions:**
```bash
# Verify database schema
sqlite3 results.db ".schema"

# Expected tables:
# - scans
# - scan_results

# Query manually
sqlite3 results.db "SELECT * FROM scans;"

# Use prtip db commands
prtip db list results.db
prtip db query results.db --scan-id 1
```

### Database Corruption

**Symptoms:**
```
Error: Database disk image is malformed
SQLite error: database corruption
```

**Solutions:**
```bash
# Attempt recovery
sqlite3 results.db ".dump" > dump.sql
sqlite3 recovered.db < dump.sql

# Verify integrity
sqlite3 results.db "PRAGMA integrity_check;"

# If corrupted beyond repair, re-run scan
prtip -sS --with-db --database new-results.db 192.168.1.0/24
```

---

## IPv6 Issues

### IPv6 Scans Not Working

**Symptoms:**
```
Error: IPv6 not supported for this scan type
Warning: IPv6 support is partial in v0.4.0
Only TCP Connect works with IPv6
```

**Cause:** IPv6 support is **partial** in v0.4.0. Only TCP Connect scanner supports IPv6 targets.

**Solutions:**
```bash
# Use TCP Connect scan for IPv6
prtip -sT -p 80,443 2001:db8::1

# IPv6 CIDR ranges supported
prtip -sT -p 80 2001:db8::/64

# Dual-stack scanning
prtip -sT -p 80,443 example.com  # Resolves both IPv4 and IPv6

# Full IPv6 support available in v0.5.0+ (Phase 5 complete)
# Now supported: SYN (-sS), UDP (-sU), FIN/NULL/Xmas, Discovery
```

### IPv6 Address Resolution

**Symptoms:**
```
Error: Cannot resolve IPv6 address
Error: Name resolution failed
```

**Solutions:**
```bash
# Ensure IPv6 is enabled
ping6 2001:db8::1

# Check DNS resolution
nslookup -type=AAAA example.com
dig AAAA example.com

# Specify IPv6 explicitly
prtip -sT -6 -p 80 example.com

# Or use direct IPv6 address
prtip -sT -p 80 2001:db8::1
```

---

## Advanced Troubleshooting

### Enable Debug Logging

```bash
# Set RUST_LOG environment variable
RUST_LOG=debug prtip -sS 192.168.1.1

# More verbose
RUST_LOG=trace prtip -sS 192.168.1.1

# Module-specific logging
RUST_LOG=prtip_scanner=debug prtip -sS 192.168.1.1

# Save debug output
RUST_LOG=debug prtip -sS 192.168.1.1 2> debug.log
```

### Packet Capture for Analysis

```bash
# Capture packets for analysis
prtip -sS --packet-capture -p 80,443 192.168.1.1
# Output: scan-TIMESTAMP.pcapng

# Analyze with Wireshark
wireshark scan-*.pcapng

# Or tcpdump
tcpdump -r scan-*.pcapng
```

### Network Trace

```bash
# Linux: tcpdump
sudo tcpdump -i eth0 -w trace.pcap host 192.168.1.1

# Run scan in another terminal
prtip -sS -p 80,443 192.168.1.1

# Analyze trace
wireshark trace.pcap
```

### Strace/Dtrace for System Calls

```bash
# Linux: strace
sudo strace -e trace=network prtip -sS 192.168.1.1 2> strace.log

# macOS: dtrace
sudo dtruss -n prtip 2> dtruss.log
```

### Memory Profiling

```bash
# Use valgrind (Linux)
valgrind --leak-check=full prtip -sS 192.168.1.1

# Use heaptrack
heaptrack prtip -sS 192.168.1.1
heaptrack_gui heaptrack.prtip.*.gz
```

### Performance Profiling

```bash
# Linux: perf
sudo perf record --call-graph dwarf prtip -sS 192.168.1.1
sudo perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph -- -sS 192.168.1.1
```

---

## Getting Help

### Before Asking for Help

1. **Check this troubleshooting guide**
2. **Read the documentation** in [Documentation Index](../getting-started/index.md)
3. **Search existing issues** on [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
4. **Enable debug logging** and check output
5. **Verify you're using the latest version**: `prtip --version`

### Reporting Bugs

Create a GitHub issue with:

```markdown
## Environment
- ProRT-IP version: [output of `prtip --version`]
- OS: [output of `uname -a` (Linux/macOS) or `ver` (Windows)]
- Rust version: [output of `rustc --version`]
- Installation method: Binary/Source

## Description
[Clear description of the problem]

## Steps to Reproduce
1. Run: `prtip -sS -p 80 192.168.1.1`
2. Expected: [What you expected to happen]
3. Actual: [What actually happened]

## Error Output
```
[Paste error messages here]
```

## Debug Log
```
[Paste RUST_LOG=debug output]
```

## Additional Context
[Any other relevant information]
```

### Getting Support

- **GitHub Issues:** https://github.com/doublegate/ProRT-IP/issues
- **Documentation:** [Getting Started](../getting-started/index.md)
- **Security Issues:** See `SECURITY.md` for responsible disclosure

### Community Resources

- **FAQ:** [Frequently Asked Questions](faq.md)
- **Development Guide:** [Implementation Details](../development/implementation.md)
- **Architecture:** [System Architecture](../advanced/architecture.md)
- **Performance:** [Performance Tuning](../advanced/performance-tuning.md)

---

## Quick Reference

### Common Error Messages and Solutions

| Error | Quick Fix |
|-------|-----------|
| "Permission denied" | Run with `sudo` or set capabilities |
| "No suitable device found" | Specify interface with `-e eth0` |
| "Operation timed out" | Increase timeout with `-T2` or `--timeout 5000` |
| "Service: unknown" | Enable service detection with `-sV` |
| "Database locked" | Close other connections, check permissions |
| "IPv6 not supported" | Use TCP Connect scan `-sT` (v0.4.0) or upgrade to v0.5.0+ |
| "Too many open files" | Increase file descriptor limit: `ulimit -n 65535` |
| "Cannot allocate memory" | Increase socket buffers or reduce parallelism |
| "Npcap not found" | Install Npcap from https://npcap.com/ |

### Performance Optimization Checklist

- [ ] Use release build: `cargo build --release`
- [ ] Enable NUMA on multi-socket: `--numa`
- [ ] Adjust parallelism: `--parallelism 100`
- [ ] Use appropriate timing: `-T4` for LANs, `-T2` for WANs
- [ ] Disable unnecessary features: No `-sV` or `-O` if not needed
- [ ] Stream to disk: `-oN results.txt` or `--with-db`
- [ ] Scan in batches for large targets
- [ ] Increase socket buffers (Linux): `sudo sysctl -w net.core.rmem_max=134217728`

### Platform-Specific Quick Fixes

**Linux:**
```bash
sudo setcap cap_net_raw+ep ./target/release/prtip
```

**Windows:**
```powershell
# Install Npcap from https://npcap.com/
# Run as Administrator
```

**macOS:**
```bash
brew install --cask wireshark  # Installs ChmodBPF
sudo reboot
```

---

## See Also

- [FAQ](faq.md) - Frequently asked questions
- [CLI Reference](../user-guide/cli-reference.md) - Complete command-line reference
- [Platform Support](../features/platform-support.md) - Platform-specific installation
- [Performance Tuning](../advanced/performance-tuning.md) - Optimization strategies
- [Database Storage](../features/database-storage.md) - Database troubleshooting
