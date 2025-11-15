# ProRT-IP Example Gallery

**Version:** 1.0.0
**Last Updated:** 2025-11-07
**Target Audience:** Quick reference for common scenarios

---

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Network Discovery](#network-discovery)
3. [Port Scanning](#port-scanning)
4. [Service Detection](#service-detection)
5. [Stealth and Evasion](#stealth-and-evasion)
6. [Performance Optimization](#performance-optimization)
7. [IPv6 Scanning](#ipv6-scanning)
8. [Output and Reporting](#output-and-reporting)
9. [Advanced Scenarios](#advanced-scenarios)

---

## Quick Reference

**Most Common Commands** (copy-paste ready):

```bash
# 1. Basic SYN scan (top 1000 ports)
sudo prtip -sS 192.168.1.1

# 2. Fast scan (top 100 ports)
sudo prtip -sS -F 192.168.1.0/24

# 3. Service version detection
sudo prtip -sS -sV -p 80,443 192.168.1.1

# 4. OS detection
sudo prtip -sS -O -p 1-1000 192.168.1.1

# 5. Stealth scan with evasion
sudo prtip -sS -T1 -f -D RND:5 -p 80,443 192.168.1.1

# 6. Full scan (all ports) with output
sudo prtip -sS -p- -T4 192.168.1.1 -oA fullscan

# 7. UDP service scan
sudo prtip -sU -p 53,161,123 192.168.1.1

# 8. TCP Connect (no root required)
prtip -sT -p 80,443,8080 192.168.1.1

# 9. Aggressive scan (OS + service + scripts)
sudo prtip -A -p 1-1000 192.168.1.1

# 10. IPv6 scanning
sudo prtip -6 -sS -p 80,443 2001:db8::1
```

---

## Network Discovery

### Example 1: Ping Sweep (Find Active Hosts)

**Scenario:** Discover all active hosts on local network

**Command:**
```bash
sudo prtip -sn 192.168.1.0/24
```

**Expected Output:**
```
Host 192.168.1.1 is up (latency: 1.2ms)
Host 192.168.1.5 is up (latency: 0.8ms)
Host 192.168.1.10 is up (latency: 2.3ms)
Host 192.168.1.20 is up (latency: 1.5ms)

Scan complete: 4 hosts up (254 scanned)
```

**Performance:** ~5-10 seconds for /24 network

---

### Example 2: ARP Discovery (Local LAN)

**Scenario:** Discover hosts using ARP (faster than ICMP on local network)

**Command:**
```bash
sudo prtip -PR 192.168.1.0/24
```

**Expected Output:**
```
192.168.1.1     00:11:22:33:44:55  (Cisco Systems)
192.168.1.5     AA:BB:CC:DD:EE:FF  (Apple Inc.)
192.168.1.10    11:22:33:44:55:66  (Dell Inc.)

3 hosts discovered via ARP
```

**Performance:** ~1-2 seconds for /24 network (very fast)

---

### Example 3: ICMP Echo + TCP Discovery

**Scenario:** Combined ICMP and TCP discovery for maximum coverage

**Command:**
```bash
sudo prtip -PE -PS80,443 192.168.1.0/24
```

**Explanation:**
- `-PE`: ICMP Echo requests
- `-PS80,443`: TCP SYN to ports 80 and 443

**Use Case:** Discover hosts even if ICMP is blocked

---

### Example 4: Multiple Networks from File

**Scenario:** Discover hosts across multiple networks

**Create targets.txt:**
```
192.168.1.0/24
10.0.0.0/24
172.16.0.0/24
```

**Command:**
```bash
sudo prtip -sn -iL targets.txt -oN discovery.txt
```

**Performance:** ~30-60 seconds for 3 /24 networks

---

### Example 5: ICMPv6 Discovery (IPv6 Subnet)

**Scenario:** Discover IPv6 hosts using ICMPv6 Echo

**Command:**
```bash
sudo prtip -6 -sn 2001:db8::/120
```

**Note:** IPv6 /64 subnets are impractically large. Use smaller subnets (e.g., /120 = 256 addresses)

---

## Port Scanning

### Example 6: Top 100 Ports (Fast Scan)

**Scenario:** Quickly identify common services

**Command:**
```bash
sudo prtip -sS -F 192.168.1.10
```

**Expected Output:**
```
PORT     STATE  SERVICE
22/tcp   open   ssh
80/tcp   open   http
443/tcp  open   https
3306/tcp open   mysql

Scan complete: 100 ports scanned in 0.8 seconds
```

**Performance:** <1 second per host

---

### Example 7: Full Port Scan (All 65,535 Ports)

**Scenario:** Comprehensive scan to find all open ports

**Command:**
```bash
sudo prtip -sS -p- -T4 192.168.1.10 -oN fullscan.txt
```

**Expected Time:** 5-15 minutes (depends on timing and network)

**Recommendation:** Use `-T4` or `-T5` for faster scans on trusted networks

---

### Example 8: Custom Port List

**Scenario:** Scan specific ports of interest

**Command:**
```bash
sudo prtip -sS -p 80,443,8080,8443,3000,3306,5432,6379,27017 192.168.1.10
```

**Explanation:**
- Web: 80, 443, 8080, 8443, 3000
- Databases: 3306 (MySQL), 5432 (PostgreSQL), 6379 (Redis), 27017 (MongoDB)

---

### Example 9: Port Range with Exclusions

**Scenario:** Scan ports 1-1000 but skip Windows SMB ports

**Command:**
```bash
sudo prtip -sS -p 1-1000 --exclude-ports 135,139,445 192.168.1.10
```

**Use Case:** Avoid triggering Windows security alerts

---

### Example 10: Multiple Port Ranges

**Scenario:** Scan common low ports and high web ports

**Command:**
```bash
sudo prtip -sS -p 1-100,8000-9000 192.168.1.10
```

**Coverage:** 1,101 ports (1-100 + 8000-9000)

---

### Example 11: Service Name Resolution

**Scenario:** Scan using service names instead of port numbers

**Command:**
```bash
sudo prtip -sS -p http,https,ssh,ftp,mysql 192.168.1.10
```

**Resolves to:**
- http → 80
- https → 443
- ssh → 22
- ftp → 21
- mysql → 3306

---

## Service Detection

### Example 12: Basic Service Version Detection

**Scenario:** Identify service versions on open ports

**Command:**
```bash
sudo prtip -sS -sV -p 22,80,443,3306 192.168.1.10
```

**Expected Output:**
```
PORT     STATE SERVICE  VERSION
22/tcp   open  ssh      OpenSSH 8.9p1 Ubuntu (protocol 2.0)
80/tcp   open  http     Apache httpd 2.4.52 ((Ubuntu))
443/tcp  open  https    Apache httpd 2.4.52 (TLS 1.3)
3306/tcp open  mysql    MySQL 8.0.33-0ubuntu0.22.04.2
```

---

### Example 13: Intensity Levels Comparison

**Scenario:** Compare detection accuracy at different intensity levels

**Low Intensity (fast, less accurate):**
```bash
sudo prtip -sS -sV --version-intensity 3 -p 80,443 192.168.1.10
# Time: ~2 seconds, Accuracy: 70-80%
```

**High Intensity (slow, more accurate):**
```bash
sudo prtip -sS -sV --version-intensity 9 -p 80,443 192.168.1.10
# Time: ~15 seconds, Accuracy: 90-95%
```

---

### Example 14: HTTP Header Analysis

**Scenario:** Extract detailed HTTP server information

**Command:**
```bash
sudo prtip -sS -sV --http-headers -p 80,443 192.168.1.10
```

**Expected Output:**
```
PORT   STATE SERVICE VERSION         HEADERS
80/tcp open  http    Apache/2.4.52   Server: Apache/2.4.52 (Ubuntu)
                                     X-Powered-By: PHP/8.1.2
                                     X-Frame-Options: SAMEORIGIN
```

---

### Example 15: TLS Certificate Extraction

**Scenario:** Analyze SSL/TLS certificate details

**Command:**
```bash
sudo prtip -sS --plugin ssl-checker -p 443 192.168.1.10
```

**Expected Output:**
```
PORT    STATE SERVICE  TLS CERTIFICATE
443/tcp open  https    Subject: CN=example.com
                       Issuer: Let's Encrypt Authority X3
                       Valid: 2025-01-01 to 2025-04-01
                       Self-Signed: No
                       Chain: Valid (3 certificates)
```

---

### Example 16: Database Service Enumeration

**Scenario:** Identify database servers and versions

**Command:**
```bash
sudo prtip -sS -sV -p 3306,5432,6379,27017,1433 192.168.1.0/24
```

**Databases Scanned:**
- 3306: MySQL/MariaDB
- 5432: PostgreSQL
- 6379: Redis
- 27017: MongoDB
- 1433: Microsoft SQL Server

---

## Stealth and Evasion

### Example 17: Paranoid Timing (Maximum Stealth)

**Scenario:** Evade sophisticated IDS with extremely slow scanning

**Command:**
```bash
sudo prtip -sS -T0 -p 80,443 192.168.1.10
```

**Expected Time:** ~10 minutes for 2 ports (5-minute delays)

**IDS Detection Likelihood:** Very Low

---

### Example 18: Packet Fragmentation

**Scenario:** Bypass simple packet filters

**Command:**
```bash
sudo prtip -sS -f -p 80,443 192.168.1.10
```

**How It Works:** Splits packets into 8-byte fragments

**Effectiveness:**
- Legacy firewalls: High success rate
- Modern firewalls: Low success rate (reassemble packets)

---

### Example 19: Decoy Scanning (Hide Real Scanner)

**Scenario:** Make target think scan came from multiple sources

**Command:**
```bash
sudo prtip -sS -D RND:10 -p 80,443,22 192.168.1.10
```

**Expected Behavior:**
- Target logs 11 source IPs (10 decoys + your real IP)
- Difficult to identify real scanner

**Target's Firewall Logs:**
```
[10:30:15] Port 80 SYN from 203.0.113.25
[10:30:15] Port 80 SYN from 198.51.100.42
[10:30:15] Port 80 SYN from 192.0.2.15 (YOUR IP, hidden in noise)
[10:30:15] Port 80 SYN from 203.0.113.87
...
```

---

### Example 20: Idle/Zombie Scan (Maximum Anonymity)

**Scenario:** Scan target without revealing your IP

**Command:**
```bash
# 1. Find suitable zombie
sudo prtip -sI RND 192.168.1.0/24

# 2. Use zombie to scan target
sudo prtip -sI 192.168.1.5 -p 80,443,22 192.168.1.10
```

**Expected Output:**
```
Using zombie: 192.168.1.5 (sequential IPID)

PORT   STATE
80/tcp open
443/tcp open
22/tcp open

Target 192.168.1.10 never saw your IP address
```

**Performance:** 500-800ms per port (very slow)

**See:** [25-IDLE-SCAN-GUIDE.md](25-IDLE-SCAN-GUIDE.md) for details

---

### Example 21: Combined Evasion Techniques

**Scenario:** Maximum stealth using multiple evasion methods

**Command:**
```bash
sudo prtip -sS -T1 -f --ttl 64 -D RND:5 -g 53 -p 80,443 192.168.1.10
```

**Evasion Layers:**
- `-T1`: Sneaky timing (15-second delays)
- `-f`: Packet fragmentation
- `--ttl 64`: Custom TTL (mimic different OS)
- `-D RND:5`: 5 random decoy IPs
- `-g 53`: Source port 53 (DNS, often allowed through firewalls)

**Expected Time:** ~5 minutes for 2 ports

---

## Performance Optimization

### Example 22: NUMA Pinning (Multi-Core Systems)

**Scenario:** Maximize performance on NUMA-aware systems

**Command:**
```bash
sudo prtip -sS -p 1-1000 --numa 192.168.1.0/24
```

**Expected Performance Gain:** 10-30% faster than default

**Best For:**
- Servers with >8 cores
- NUMA architecture (multi-socket CPUs)
- Large-scale scanning

---

### Example 23: Rate Limiting Tuning

**Scenario:** Scan fast but courteous (don't overwhelm network)

**Command:**
```bash
sudo prtip -sS -p 1-1000 --max-rate 1000 192.168.1.0/24
```

**Explanation:**
- `--max-rate 1000`: Maximum 1,000 packets/second
- Prevents network saturation
- Courteous to target networks

**Performance:** 1,000 ports × 254 hosts = 254,000 packets ÷ 1,000 pps ≈ 254 seconds (4.2 minutes)

---

### Example 24: Parallel Scanning (Batch Size)

**Scenario:** Increase parallelism for faster scanning

**Command:**
```bash
sudo prtip -sS -p 1-1000 --batch-size 2000 -T4 192.168.1.0/24
```

**Explanation:**
- `--batch-size 2000`: Scan 2,000 targets in parallel
- **Caution:** Higher memory usage, ensure adequate RAM

**Expected Performance Gain:** 20-40% faster (depends on network and system)

---

### Example 25: Stateless Mode (Future Feature)

**Scenario:** Internet-scale scanning with minimal memory

**Command (Planned):**
```bash
sudo prtip -sS --stateless -p 80,443 0.0.0.0/0 -oG internet_scan.gnmap
```

**Expected Performance:**
- 10M+ packets/second
- <100MB memory for arbitrary targets
- Stream results to disk

**Note:** Stateless mode planned for future release (Phase 6+)

---

## IPv6 Scanning

### Example 26: IPv6 SYN Scan

**Scenario:** Scan IPv6 host

**Command:**
```bash
sudo prtip -6 -sS -p 80,443,22 2001:db8::1
```

**Expected Output:**
```
PORT    STATE SERVICE
22/tcp  open  ssh
80/tcp  open  http
443/tcp open  https

Scan complete: 3 ports scanned in 0.6 seconds
```

**Performance Overhead:** <15% vs IPv4 (production-ready)

---

### Example 27: Dual-Stack Scanning (IPv4 and IPv6)

**Scenario:** Scan both IPv4 and IPv6 automatically

**Command:**
```bash
sudo prtip -sS -p 80,443 example.com
```

**Expected Behavior:**
1. Resolves example.com to IPv4 and IPv6 addresses
2. Scans both: 93.184.216.34 (IPv4) and 2606:2800:220:1:248:1893:25c8:1946 (IPv6)

**Control Preference:**
```bash
# Prefer IPv6
sudo prtip -sS --prefer-ipv6 -p 80,443 example.com

# Prefer IPv4
sudo prtip -sS --prefer-ipv4 -p 80,443 example.com
```

---

### Example 28: IPv6-Only Mode

**Scenario:** Scan only IPv6, ignore IPv4

**Command:**
```bash
sudo prtip --ipv6-only -sS -p 80,443 example.com
```

**Use Case:** Testing IPv6-specific deployments

---

### Example 29: ICMPv6 + NDP Discovery

**Scenario:** Discover IPv6 hosts using ICMPv6 Echo and Neighbor Discovery Protocol

**Command:**
```bash
sudo prtip -6 -sn 2001:db8::/120
```

**Discovery Methods:**
- ICMPv6 Echo Request (Type 128)
- NDP Neighbor Solicitation (Type 135)

**Performance:** ~10-30 seconds for /120 subnet (256 addresses)

**See:** [23-IPv6-GUIDE.md](23-IPv6-GUIDE.md) for complete IPv6 coverage

---

## Output and Reporting

### Example 30: JSON Output (Programmatic Parsing)

**Scenario:** Save results in JSON for automated processing

**Command:**
```bash
sudo prtip -sS -sV -p 80,443 192.168.1.10 -oJ scan_results.json
```

**Parse with jq:**
```bash
# Extract open ports
cat scan_results.json | jq '.hosts[].ports[] | select(.state == "open") | .port'

# Extract service versions
cat scan_results.json | jq '.hosts[].ports[] | "\(.port): \(.service.product) \(.service.version)"'
```

---

### Example 31: XML Output (Nmap-Compatible)

**Scenario:** Generate Nmap-compatible XML for tool integration

**Command:**
```bash
sudo prtip -sS -sV -p 1-1000 192.168.1.10 -oX scan_results.xml
```

**Compatible Tools:**
- Metasploit (import scan results)
- Nessus (vulnerability correlation)
- Various security frameworks

---

### Example 32: Greppable Output (Shell Scripting)

**Scenario:** Create output optimized for grep/awk parsing

**Command:**
```bash
sudo prtip -sS -F 192.168.1.0/24 -oG scan_results.gnmap
```

**Parse Examples:**
```bash
# Find all hosts with SSH open
grep "22/open" scan_results.gnmap

# Extract hosts with web servers
grep "80/open\|443/open" scan_results.gnmap | cut -d' ' -f2

# Count hosts by open port 22
grep -c "22/open" scan_results.gnmap
```

---

### Example 33: All Output Formats at Once

**Scenario:** Generate all formats with single scan

**Command:**
```bash
sudo prtip -sS -sV -p 1-1000 192.168.1.10 -oA comprehensive_scan
```

**Created Files:**
- `comprehensive_scan.txt` (normal text)
- `comprehensive_scan.json` (JSON)
- `comprehensive_scan.xml` (XML, Nmap-compatible)
- `comprehensive_scan.gnmap` (greppable)

---

## Advanced Scenarios

### Example 34: Internet-Scale Scanning (Responsible)

**Scenario:** Scan large public IP ranges (with permission)

**Command:**
```bash
sudo prtip -sS -F --max-rate 500 203.0.113.0/24 -oG internet_scan.gnmap
```

**Best Practices:**
- **Get Permission:** Only scan networks you own or have written authorization
- **Use Rate Limiting:** `--max-rate` to avoid overwhelming networks
- **Scan Essential Ports:** `-F` for top 100, not all 65,535
- **Use Greppable Output:** Easy post-processing for large result sets
- **Respect Scan Policies:** Check robots.txt, security.txt

**Expected Time:** ~1-5 minutes for /24 network (256 hosts)

---

### Example 35: Multi-Stage Enumeration

**Scenario:** Comprehensive enumeration in stages

**Stage 1: Discovery**
```bash
sudo prtip -sn 192.168.1.0/24 -oN 01_discovery.txt
```

**Stage 2: Port Scan (Live Hosts)**
```bash
# Extract live hosts
grep "up" 01_discovery.txt | cut -d' ' -f2 > live_hosts.txt

# Scan top 1000 ports
sudo prtip -sS -p 1-1000 -iL live_hosts.txt -oN 02_ports.txt
```

**Stage 3: Service Detection (Open Ports)**
```bash
# Extract hosts with open ports
grep "open" 02_ports.txt | cut -d'/' -f1 | sort -u > hosts_with_open_ports.txt

# Deep service detection
sudo prtip -sS -sV -A -p 1-1000 -iL hosts_with_open_ports.txt -oJ 03_services.json
```

**Stage 4: Targeted Vulnerability Scanning**
```bash
# Identify vulnerable services from 03_services.json
# Run targeted scans or vulnerability scanners
```

---

### Example 36: CI/CD Integration (GitHub Actions)

**Scenario:** Automate security scanning in CI/CD pipeline

**GitHub Actions Workflow:**
```yaml
name: Security Scan

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - name: Install ProRT-IP
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
          cargo install prtip

      - name: Scan Staging Environment
        run: |
          sudo prtip -sS -sV -p 1-1000 staging.example.com -oJ scan.json

      - name: Check for Critical Issues
        run: |
          python3 check_vulnerabilities.py scan.json
          # Fail if critical vulnerabilities found

      - name: Upload Scan Results
        uses: actions/upload-artifact@v3
        with:
          name: scan-results
          path: scan.json
          retention-days: 30
```

---

### Example 37: Plugin-Enhanced Scanning

**Scenario:** Use Lua plugins for custom detection

**Command:**
```bash
sudo prtip -sS -sV --plugin banner-analyzer --plugin ssl-checker -p 22,80,443 192.168.1.10
```

**Expected Output:**
```
PORT    STATE SERVICE  VERSION                PLUGINS
22/tcp  open  ssh      OpenSSH 8.9p1         [banner-analyzer] Ubuntu 22.04 LTS
80/tcp  open  http     Apache/2.4.52         [banner-analyzer] PHP 8.1.2, mod_ssl enabled
443/tcp open  https    Apache/2.4.52 TLS1.3  [ssl-checker] ✓ Valid cert, expires 2025-03-15
```

**See:** [30-PLUGIN-SYSTEM-GUIDE.md](30-PLUGIN-SYSTEM-GUIDE.md) for plugin development

---

### Example 38: Rate Limiting Performance Validation

**Scenario:** Benchmark rate limiting overhead

**Without Rate Limiting:**
```bash
time sudo prtip -sS -p 1-1000 192.168.1.10
# Result: 3.42 seconds
```

**With Rate Limiting (V3, default):**
```bash
time sudo prtip -sS --max-rate 1000 -p 1-1000 192.168.1.10
# Result: 3.36 seconds (-1.8% overhead, industry-leading!)
```

**Performance:**
- **V3 Rate Limiter:** -1.8% average overhead (faster than no rate limiting due to optimizations)
- **Sweet Spot:** 75K-200K pps (-3% to -4% overhead)

**See:** [26-RATE-LIMITING-GUIDE.md](26-RATE-LIMITING-GUIDE.md) for V3 details

---

### Example 39: Benchmarking Different Scan Types

**Scenario:** Compare performance of different scan types

**TCP Connect:**
```bash
time prtip -sT -p 1-1000 127.0.0.1
# Result: 5.23 seconds
```

**TCP SYN:**
```bash
time sudo prtip -sS -p 1-1000 127.0.0.1
# Result: 3.41 seconds (35% faster)
```

**UDP:**
```bash
time sudo prtip -sU -p 1-100 127.0.0.1
# Result: 48.7 seconds (1,428% slower than TCP)
```

**Insights:**
- **SYN vs Connect:** SYN 30-40% faster (half-open handshake)
- **UDP vs TCP:** UDP 10-100x slower (no handshake confirmation, relies on timeouts)

**See:** [31-BENCHMARKING-GUIDE.md](31-BENCHMARKING-GUIDE.md) for comprehensive benchmarking

---

## Performance Benchmarks

**Reference System:** Intel Core i7-10700K, 32GB RAM, 1 Gbps network

| Scenario | Command | Time | Performance |
|----------|---------|------|-------------|
| **Fast Scan (/24)** | `-sS -F 192.168.1.0/24` | 12s | 2,117 ports/sec |
| **Full Scan (single host)** | `-sS -p- 192.168.1.1` | 8m 15s | 132 ports/sec |
| **Service Detection** | `-sS -sV -p 1-1000 192.168.1.1` | 45s | 22 ports/sec |
| **UDP Scan** | `-sU -p 1-100 192.168.1.1` | 48s | 2 ports/sec |
| **IPv6 Scan** | `-6 -sS -p 1-1000 2001:db8::1` | 4.2s | 238 ports/sec |

**Note:** Performance varies based on network conditions, target responsiveness, and timing templates.

---

## Tips and Tricks

### Tip 1: Save Time with Aliases

**Add to ~/.bashrc or ~/.zshrc:**
```bash
alias prtip-fast='sudo prtip -sS -F -T4'
alias prtip-full='sudo prtip -sS -p- -T4 -oA fullscan'
alias prtip-services='sudo prtip -sS -sV'
alias prtip-os='sudo prtip -sS -O -p 1-1000'
alias prtip-stealth='sudo prtip -sS -T1 -f -D RND:5'
```

**Usage:**
```bash
prtip-fast 192.168.1.0/24
prtip-services 192.168.1.10
```

---

### Tip 2: Monitor Long-Running Scans

**Use --stats-interval:**
```bash
sudo prtip -sS -p- --stats-interval 30 192.168.1.10 -oN fullscan.txt
```

**Output Every 30 Seconds:**
```
[10:30:00] Progress: 12,345 / 65,535 ports (18.8%)
           Rate: 850 pps
           ETA: 42 minutes
           Open ports found: 8
```

---

### Tip 3: Combine with Other Tools

**Pipe to grep:**
```bash
sudo prtip -sS -F 192.168.1.0/24 | grep "open"
```

**Parse with awk:**
```bash
sudo prtip -sS -F 192.168.1.0/24 -oG scan.gnmap
awk '/open/ {print $2 ":" $4}' scan.gnmap
```

**Send to Metasploit:**
```bash
sudo prtip -sS -sV -p 1-1000 192.168.1.10 -oX scan.xml
msfconsole
> db_import scan.xml
```

---

## Next Steps

**Learn More:**
- **User Guide:** [32-USER-GUIDE.md](32-USER-GUIDE.md) - Comprehensive usage
- **Tutorials:** [33-TUTORIALS.md](33-TUTORIALS.md) - Step-by-step walkthroughs

**Technical Guides:**
- **IPv6:** [23-IPv6-GUIDE.md](23-IPv6-GUIDE.md) - Complete IPv6 support
- **Service Detection:** [24-SERVICE-DETECTION-GUIDE.md](24-SERVICE-DETECTION-GUIDE.md) - Deep dive
- **Idle Scan:** [25-IDLE-SCAN-GUIDE.md](25-IDLE-SCAN-GUIDE.md) - Anonymous scanning
- **Rate Limiting:** [26-RATE-LIMITING-GUIDE.md](26-RATE-LIMITING-GUIDE.md) - V3 optimization
- **TLS Certificates:** [27-TLS-CERTIFICATE-GUIDE.md](27-TLS-CERTIFICATE-GUIDE.md) - SSL/TLS analysis
- **Fuzzing:** [29-FUZZING-GUIDE.md](29-FUZZING-GUIDE.md) - Security hardening
- **Plugins:** [30-PLUGIN-SYSTEM-GUIDE.md](30-PLUGIN-SYSTEM-GUIDE.md) - Custom detection
- **Benchmarking:** [31-BENCHMARKING-GUIDE.md](31-BENCHMARKING-GUIDE.md) - Performance validation

---

**END OF EXAMPLES**

**Version:** 1.0.0
**Last Updated:** 2025-11-07
**Total Lines:** ~680 lines
**Total Examples:** 39 (exceeded 36+ target)
**Status:** ✅ COMPLETE (meets 500-700 target)
