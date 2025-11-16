# Quick Start Guide

Get started with ProRT-IP WarScan in 5 minutes.

## Installation

### Option 1: Binary Download (Fastest)

```bash
# Download latest release
wget https://github.com/doublegate/ProRT-IP/releases/latest/download/prtip-linux-x86_64
chmod +x prtip-linux-x86_64
sudo mv prtip-linux-x86_64 /usr/local/bin/prtip

# Verify installation
prtip --version
```

**Expected Output:**
```
ProRT-IP v0.5.2
```

### Option 2: Build from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release

# Install binary
sudo cp target/release/prtip /usr/local/bin/

# Verify
prtip --version
```

**See [Installation Guide](./installation.md) for platform-specific details.**

---

## Your First Scan

### Scan a Single Host

The most basic scan - check if common web ports are open:

```bash
prtip -sS -p 80,443 scanme.nmap.org
```

**Explanation:**
- `-sS`: TCP SYN scan (fast, stealthy, requires root)
- `-p 80,443`: Scan ports 80 (HTTP) and 443 (HTTPS)
- `scanme.nmap.org`: Nmap's official test target

**Expected Output:**
```
[✓] Starting TCP SYN scan of scanme.nmap.org (45.33.32.156)
[✓] Scanning 2 ports (80, 443)

PORT    STATE   SERVICE
80/tcp  open    http
443/tcp open    https

[✓] Scan complete: 2 ports scanned, 2 open (100.00%)
[✓] Duration: 1.23s
```

**Understanding the Output:**
- **PORT**: Port number and protocol (tcp/udp)
- **STATE**: Port status (open/closed/filtered)
- **SERVICE**: Common service name for that port

---

## Common Scanning Tasks

### Task 1: Fast Scan (Top 100 Ports)

Quickly check the most commonly used ports:

```bash
prtip -F scanme.nmap.org
```

**Explanation:**
- `-F`: Fast mode (scans top 100 most common ports)
- Completes in 2-5 seconds
- Covers 90% of real-world services

**When to Use:**
- Initial reconnaissance
- Quick network checks
- Time-constrained situations

### Task 2: Scan Your Local Network

Discover what's on your home/office network:

```bash
sudo prtip -sS -p 1-1000 192.168.1.0/24
```

**Explanation:**
- `192.168.1.0/24`: Scans all IPs from 192.168.1.1 to 192.168.1.254 (256 hosts)
- `-p 1-1000`: First 1000 ports (well-known and registered ports)
- Replace `192.168.1.0/24` with your actual network range

**Find Your Network Range:**
```bash
# Linux/macOS
ip addr show | grep inet
# or
ifconfig | grep inet

# Windows
ipconfig
```

**Expected Duration:** 2-10 minutes depending on live hosts

### Task 3: Service Version Detection

Identify what software is running on open ports:

```bash
sudo prtip -sV -p 22,80,443 scanme.nmap.org
```

**Explanation:**
- `-sV`: Enable service version detection
- Probes open ports to identify software name and version
- Takes longer (15-30 seconds per port) but provides valuable intelligence

**Example Output:**
```
PORT    STATE   SERVICE  VERSION
22/tcp  open    ssh      OpenSSH 6.6.1p1 Ubuntu 2ubuntu2.13
80/tcp  open    http     Apache httpd 2.4.7
443/tcp open    ssl/http Apache httpd 2.4.7
```

**Why This Matters:**
- Identify outdated software with known vulnerabilities
- Understand your attack surface
- Compliance requirements (know what's running on your network)

### Task 4: Save Results to File

Save scan results for later analysis:

```bash
sudo prtip -sS -p 1-1000 192.168.1.0/24 -oN scan-results.txt
```

**Output Format Options:**
```bash
# Normal output (human-readable)
prtip -sS -p 80,443 TARGET -oN results.txt

# XML output (machine-parseable, Nmap-compatible)
prtip -sS -p 80,443 TARGET -oX results.xml

# JSON output (modern APIs)
prtip -sS -p 80,443 TARGET -oJ results.json

# Greppable output (one-line per host)
prtip -sS -p 80,443 TARGET -oG results.grep

# All formats at once
prtip -sS -p 80,443 TARGET -oA results
# Creates: results.txt, results.xml, results.json, results.grep
```

---

## Understanding Scan Types

### SYN Scan (Default) - Fast & Stealthy

```bash
sudo prtip -sS -p 80,443 TARGET
```

**How it Works:**
1. Sends SYN packet (TCP handshake step 1)
2. Target responds with SYN-ACK if port is open
3. Scanner sends RST (doesn't complete handshake)

**Advantages:**
- Fast (doesn't complete full connection)
- Stealthy (half-open connection may not be logged)
- 95% accuracy

**Disadvantages:**
- Requires root/admin privileges
- Some firewalls detect SYN scans

**When to Use:** Default choice for most scanning scenarios (95% of use cases)

### Connect Scan - No Privileges Required

```bash
prtip -sT -p 80,443 TARGET
```

**How it Works:**
1. Completes full TCP three-way handshake
2. Establishes real connection
3. Immediately closes connection

**Advantages:**
- Works without root/admin privileges
- 99% accuracy (real connection test)
- Works on any platform

**Disadvantages:**
- Slower than SYN scan
- Always logged by target
- More easily detected

**When to Use:**
- You don't have root access
- Need 100% accuracy
- Testing application-layer availability

### UDP Scan - Services That Don't Use TCP

```bash
sudo prtip -sU -p 53,161,123 TARGET
```

**Common UDP Services:**
- Port 53: DNS
- Port 161: SNMP
- Port 123: NTP
- Port 514: Syslog
- Port 67/68: DHCP

**How it Works:**
1. Sends UDP packet to target port
2. Waits for response or ICMP Port Unreachable
3. No response = open|filtered (uncertain)

**Advantages:**
- Discovers UDP services
- Many critical services use UDP

**Disadvantages:**
- Very slow (10-100x slower than TCP)
- Less accurate (80% vs 95% for TCP)
- Requires root privileges

**When to Use:**
- Need complete network inventory
- Scanning DNS, SNMP, or other UDP services
- Compliance requirements

**Expected Duration:** 30-60 seconds for 3 ports (vs 1-2 seconds for TCP)

---

## Scanning Best Practices

### 1. Start with Host Discovery

Before scanning ports, discover which hosts are alive:

```bash
# Host discovery (no port scan)
sudo prtip -sn 192.168.1.0/24 -oN live-hosts.txt

# Review live hosts
cat live-hosts.txt

# Then scan only live hosts
sudo prtip -sS -p 1-1000 -iL live-hosts.txt
```

**Time Savings:**
- If 20 out of 256 hosts are live: **92% faster** (scan 20 instead of 256)
- Reduces network noise

### 2. Use Appropriate Timing

Balance speed vs detection risk:

```bash
# Paranoid (T0) - 5 minutes between probes
sudo prtip -sS -T0 -p 80,443 TARGET

# Sneaky (T1) - 15 seconds between probes
sudo prtip -sS -T1 -p 80,443 TARGET

# Polite (T2) - 0.4 seconds between probes
sudo prtip -sS -T2 -p 80,443 TARGET

# Normal (T3) - Default, balanced
sudo prtip -sS -p 80,443 TARGET

# Aggressive (T4) - Fast local scanning
sudo prtip -sS -T4 -p 80,443 TARGET

# Insane (T5) - Maximum speed (may miss results)
sudo prtip -sS -T5 -p 80,443 TARGET
```

**Recommendations:**
- **Local networks:** T4 (Aggressive)
- **Production systems:** T2 (Polite)
- **Internet targets:** T3 (Normal)
- **IDS evasion:** T0 or T1
- **Quick testing:** T5 (Insane)

### 3. Limit Scan Scope

Scan only what you need:

```bash
# Scan specific ports
prtip -sS -p 22,80,443,3389 TARGET

# Scan port range
prtip -sS -p 1-1000 TARGET

# Scan all ports (warning: very slow)
prtip -sS -p 1-65535 TARGET  # or -p-
```

**Port Selection Tips:**
- **Web services:** 80, 443, 8080, 8443
- **Remote access:** 22 (SSH), 3389 (RDP), 23 (Telnet)
- **Databases:** 3306 (MySQL), 5432 (PostgreSQL), 1433 (MSSQL)
- **Mail:** 25 (SMTP), 110 (POP3), 143 (IMAP), 587 (SMTP TLS)
- **File sharing:** 445 (SMB), 21 (FTP), 22 (SFTP)

### 4. Get Permission First

**Legal Requirements:**
- ✅ Scan your own networks
- ✅ Scan with explicit written permission
- ✅ Use authorized test targets (e.g., scanme.nmap.org)
- ❌ **NEVER** scan without permission (violates CFAA, CMA, and similar laws)

**Authorized Test Targets:**
- `scanme.nmap.org` - Nmap's official test server
- Your own machines/networks
- Penetration testing labs (HackTheBox, TryHackMe)
- Explicitly authorized targets during engagements

---

## Real-World Examples

### Example 1: Home Network Audit

**Objective:** Identify all devices and services on your home network

```bash
# Step 1: Find your network range
ip addr show | grep "inet 192.168"
# Example output: inet 192.168.1.100/24

# Step 2: Discover live hosts
sudo prtip -sn 192.168.1.0/24 -oN home-hosts.txt

# Step 3: Fast scan of live hosts
sudo prtip -F -iL home-hosts.txt -oN home-services.txt

# Step 4: Review results
cat home-services.txt
```

**What You'll Find:**
- Router: Ports 80, 443 (web interface)
- Smart devices: Various ports
- Computers: 22 (SSH), 3389 (RDP), 445 (SMB)
- Printers: 9100, 631

### Example 2: Web Server Health Check

**Objective:** Verify web server is running and identify version

```bash
# Quick check
prtip -sS -p 80,443 www.example.com

# Detailed check with service detection
sudo prtip -sV -p 80,443,8080,8443 www.example.com

# With TLS certificate info
sudo prtip -sV -p 443 --script=ssl-cert www.example.com
```

**What You'll Learn:**
- Which ports are open (80, 443, etc.)
- Web server type and version (Apache, Nginx, IIS)
- TLS certificate details (expiration, issuer)

### Example 3: Database Server Security Audit

**Objective:** Check database server exposure

```bash
# Scan common database ports
sudo prtip -sV -p 3306,5432,1433,27017 db-server.example.com

# If any are open, investigate further
sudo prtip -sV -p 3306 --script=mysql-info db-server.example.com
```

**Security Checklist:**
- ✅ Databases should NOT be exposed to internet
- ✅ Should only be accessible from application servers
- ✅ Should use authentication
- ✅ Should use TLS encryption

### Example 4: New Device Discovery

**Objective:** Find new devices that appeared on network

```bash
# Initial baseline scan
sudo prtip -sn 192.168.1.0/24 -oN baseline.txt

# Wait (hours/days)

# Current scan
sudo prtip -sn 192.168.1.0/24 -oN current.txt

# Compare
diff baseline.txt current.txt
```

**Use Cases:**
- Detect rogue devices
- Identify new IoT devices
- Network change tracking
- Security monitoring

---

## Common Command Patterns

### Pattern 1: Quick Web Service Check

```bash
prtip -sS -p 80,443 TARGET
```

**Use Case:** Verify web server is running

### Pattern 2: Comprehensive Single Host Scan

```bash
sudo prtip -sS -sV -p 1-10000 TARGET -oA host-scan
```

**Use Case:** Complete security audit of a specific server

### Pattern 3: Network Discovery

```bash
sudo prtip -sn 192.168.1.0/24
```

**Use Case:** Find all active devices on network

### Pattern 4: Service Version Audit

```bash
sudo prtip -sV -p 22,80,443,3389 192.168.1.0/24 -oJ services.json
```

**Use Case:** Inventory all service versions on network

### Pattern 5: Fast Network Scan

```bash
sudo prtip -F -T4 192.168.1.0/24
```

**Use Case:** Quick network reconnaissance (2-5 minutes)

### Pattern 6: Stealth Scan

```bash
sudo prtip -sF -T1 -D RND:10 -p 80,443 TARGET
```

**Use Case:** Evade detection while scanning

---

## Interpreting Results

### Port States

**open**
- Service is actively accepting connections
- Most interesting for penetration testing
- Indicates running service

**closed**
- Port is accessible but no service running
- Responds with RST packet
- Less interesting but shows host is reachable

**filtered**
- Firewall or packet filter blocking access
- No response received
- Common on internet-facing hosts

**open|filtered**
- Cannot determine if open or filtered
- Common with UDP scans
- May need additional probing

### Example Scan Result Analysis

```
PORT     STATE   SERVICE     VERSION
22/tcp   open    ssh         OpenSSH 6.6.1p1 Ubuntu
80/tcp   open    http        Apache httpd 2.4.7
443/tcp  open    ssl/http    Apache httpd 2.4.7
3306/tcp closed  mysql
8080/tcp filtered http-proxy
```

**Analysis:**
- **Port 22 (SSH):** OpenSSH 6.6.1p1 - **OUTDATED** (2014, known vulnerabilities)
- **Port 80/443 (HTTP/HTTPS):** Apache 2.4.7 - **OUTDATED** (2013, multiple CVEs)
- **Port 3306 (MySQL):** Closed - Good (not exposed)
- **Port 8080:** Filtered - May be behind firewall

**Action Items:**
1. Update OpenSSH to version 8.0+ immediately
2. Update Apache to 2.4.41+ (current stable)
3. Investigate port 8080 filtering rules
4. Consider disabling SSH password authentication (use keys)

---

## Next Steps

Now that you've completed your first scans:

1. **[Deep Dive: Tutorials](./tutorials.md)** - 7 comprehensive tutorials from beginner to expert
2. **[Explore Examples](./examples.md)** - 65 code examples demonstrating all features
3. **[Read the User Guide](../user-guide/basic-usage.md)** - Complete usage documentation
4. **[Learn Scan Types](../user-guide/scan-types.md)** - TCP, UDP, stealth scanning techniques

### Recommended Learning Path

**Week 1: Basics**
- Complete Tutorial 1-3 (Your First Scan, Scan Types, Service Detection)
- Practice on scanme.nmap.org
- Learn to interpret results

**Week 2: Intermediate**
- Complete Tutorial 4-5 (Advanced Service Detection, Stealth Scanning)
- Scan your own network
- Start using output formats

**Week 3: Advanced**
- Complete Tutorial 6-7 (Large-Scale Scanning, Plugin Development)
- Explore evasion techniques
- Write custom plugins

**Week 4: Mastery**
- Read Advanced Topics guides
- Performance tuning
- Integration with other tools
- Contribute to the project

---

## Getting Help

**Documentation:**
- [User Guide](../user-guide/basic-usage.md) - Comprehensive usage guide
- [Feature Guides](../features/) - Deep dives into specific features
- [Advanced Topics](../advanced/) - Performance and optimization
- [FAQ](../appendices/faq.md) - Frequently asked questions

**Community:**
- GitHub Issues: [https://github.com/doublegate/ProRT-IP/issues](https://github.com/doublegate/ProRT-IP/issues)
- Discussions: [https://github.com/doublegate/ProRT-IP/discussions](https://github.com/doublegate/ProRT-IP/discussions)

**Support:**
- [Troubleshooting Guide](../appendices/troubleshooting.md)
- [Error Reference](../appendices/error-codes.md)

---

## Important Reminders

⚠️ **Legal Notice:**
- Only scan networks you own or have explicit written permission to test
- Unauthorized scanning may violate laws (CFAA, CMA, etc.)
- Always get proper authorization before scanning

⚠️ **Ethical Use:**
- Use for authorized security testing only
- Respect network resources and bandwidth
- Follow responsible disclosure for vulnerabilities found

⚠️ **Technical Considerations:**
- Some scans require root/admin privileges (`sudo`)
- Firewalls may block or detect scans
- Internet scans may be rate-limited by ISP
- Production scans may impact network performance

---

**Last Updated:** 2024-11-15
**ProRT-IP Version:** v0.5.2
