# Basic Usage

Learn the fundamentals of ProRT-IP WarScan command-line interface.

## Command Syntax

**General Format:**
```bash
prtip [OPTIONS] <TARGET>
```

**Examples:**
```bash
prtip 192.168.1.1                    # Basic scan (default ports)
prtip -p 80,443 example.com          # Specific ports
prtip -sS -p 1-1000 10.0.0.0/24      # SYN scan, port range, CIDR
```

---

## Target Specification

### Single IP

```bash
prtip 192.168.1.1
prtip example.com
```

### CIDR Notation

```bash
prtip 192.168.1.0/24        # Scan 192.168.1.1-254
prtip 10.0.0.0/16           # Scan 10.0.0.1-10.0.255.254
```

### IP Range

```bash
prtip 192.168.1.1-50        # Scan 192.168.1.1 to 192.168.1.50
prtip 192.168.1-10.1        # Scan 192.168.1.1 to 192.168.10.1
```

### Multiple Targets

```bash
prtip 192.168.1.1 192.168.1.2 192.168.1.3
prtip 192.168.1.1/24 10.0.0.1/24
```

### From File

```bash
prtip -iL targets.txt
```

**targets.txt content:**
```
192.168.1.1
10.0.0.0/24
example.com
```

### IPv6

```bash
prtip -6 2001:db8::1
prtip -6 2001:db8::/64
```

---

## Port Specification

### Specific Ports

```bash
prtip -p 80,443,8080 TARGET
```

### Port Range

```bash
prtip -p 1-100 TARGET          # Ports 1-100
prtip -p- TARGET               # All ports (1-65535)
```

### Common Ports (Fast)

```bash
prtip -F TARGET                # Top 100 ports
```

### Exclude Ports

```bash
prtip -p 1-1000 --exclude-ports 135,139,445 TARGET
```

### Service Names

```bash
prtip -p http,https,ssh TARGET   # Resolves to 80,443,22
```

---

## Common Use Cases

### Network Discovery

**Goal:** Find all active hosts on local network

**Command:**
```bash
sudo prtip -sn 192.168.1.0/24
```

**Explanation:**
- `-sn`: Ping scan only (no port scan)
- `192.168.1.0/24`: Scan entire /24 subnet (192.168.1.1-254)

**Expected Output:**
```
Host 192.168.1.1 is up (latency: 2.3ms)
Host 192.168.1.5 is up (latency: 1.8ms)
Host 192.168.1.10 is up (latency: 3.1ms)
...
Scan complete: 3 hosts up (254 scanned)
```

### Port Scanning

#### Common Ports (Fast)

**Goal:** Quickly identify common services

**Command:**
```bash
sudo prtip -sS -F 192.168.1.10
```

**Explanation:**
- `-F`: Fast scan (top 100 ports)
- Completes in seconds

**Expected Output:**
```
PORT    STATE  SERVICE
22/tcp  open   ssh
80/tcp  open   http
443/tcp open   https
3306/tcp open  mysql
```

#### Full Port Scan

**Goal:** Comprehensive scan of all 65,535 ports

**Command:**
```bash
sudo prtip -sS -p- -T4 192.168.1.10 -oN fullscan.txt
```

**Explanation:**
- `-p-`: All ports (1-65535)
- `-T4`: Aggressive timing (faster)
- `-oN fullscan.txt`: Save results

**Note:** Full scan can take 5-30 minutes depending on network and timing template.

#### Custom Port List

**Goal:** Scan specific ports of interest

**Command:**
```bash
sudo prtip -sS -p 80,443,8080,8443,3000,3306 192.168.1.10
```

**Explanation:**
- Web ports: 80, 443, 8080, 8443, 3000
- Database port: 3306 (MySQL)

---

## Service Detection

**Goal:** Identify services and versions running on open ports

**Command:**
```bash
sudo prtip -sS -sV -p 1-1000 192.168.1.10
```

**Expected Output:**
```
PORT    STATE  SERVICE  VERSION
22/tcp  open   ssh      OpenSSH 8.9p1 Ubuntu 3ubuntu0.1 (Ubuntu Linux; protocol 2.0)
80/tcp  open   http     Apache httpd 2.4.52 ((Ubuntu))
443/tcp open   https    Apache httpd 2.4.52 ((Ubuntu))
3306/tcp open  mysql    MySQL 8.0.33-0ubuntu0.22.04.2
```

**Interpretation:**
- **OpenSSH 8.9p1:** SSH server version
- **Apache 2.4.52:** Web server version
- **MySQL 8.0.33:** Database version
- **Ubuntu Linux:** Operating system hint

**Use Case:**
- Vulnerability assessment (check for outdated versions)
- Inventory management (document server configurations)

### Intensity Levels

**Basic Service Detection:**
```bash
sudo prtip -sS -sV -p 1-1000 192.168.1.1
```

**Intensity Levels (0-9):**
```bash
sudo prtip -sS -sV --version-intensity 5 -p 80,443 192.168.1.1
# Higher intensity = more probes, more accurate, slower
```

**Aggressive Detection (OS + Service + Scripts):**
```bash
sudo prtip -A -p 1-1000 192.168.1.1
# Equivalent to: -sV -O -sC --traceroute
```

---

## OS Fingerprinting

**Goal:** Determine operating system of target

**Command:**
```bash
sudo prtip -sS -O -p 1-1000 192.168.1.10
```

**Expected Output:**
```
OS Detection Results:
OS: Linux 5.15 - 6.1 (Ubuntu 22.04)
Confidence: 95%
CPE: cpe:/o:canonical:ubuntu_linux:22.04
```

**Interpretation:**
- **OS:** Linux kernel 5.15-6.1
- **Distribution:** Ubuntu 22.04
- **Confidence:** 95% (high confidence)

**Use Case:**
- Network inventory
- Vulnerability scanning (OS-specific exploits)
- Compliance checks

---

## Batch Scanning

**Goal:** Scan multiple targets from file

**Command:**
```bash
sudo prtip -sS -p 80,443 -iL targets.txt -oA batch_results
```

**targets.txt:**
```
192.168.1.10
192.168.1.20
10.0.0.0/24
example.com
```

**Output:**
- `batch_results.txt` (normal output)
- `batch_results.json` (JSON)
- `batch_results.xml` (XML)
- `batch_results.gnmap` (greppable)

---

## Best Practices

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

### 2. Limit Scan Scope

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

### 3. Get Permission First

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

## Common Mistakes

### Mistake 1: Forgetting sudo for SYN Scan

**Wrong:**
```bash
prtip -sS -p 80,443 192.168.1.1
# Error: Permission denied
```

**Correct:**
```bash
sudo prtip -sS -p 80,443 192.168.1.1
```

### Mistake 2: Scanning Without Permission

**Wrong:**
```bash
sudo prtip -sS -p 1-65535 8.8.8.8
# Illegal: Scanning Google DNS without permission
```

**Correct:**
```bash
# Only scan networks you own or have written permission to test
sudo prtip -sS -p 1-1000 scanme.nmap.org  # Nmap provides this for testing
```

### Mistake 3: Using Wrong Port Syntax

**Wrong:**
```bash
sudo prtip -sS -p 80-443 192.168.1.1
# This scans ports 80 to 443 (364 ports), not just 80 and 443
```

**Correct:**
```bash
sudo prtip -sS -p 80,443 192.168.1.1
# Scan only ports 80 and 443
```

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

## Quick Reference

### Essential Commands

```bash
# Basic Scans
prtip -sT -p 80,443 TARGET          # TCP Connect (no root)
sudo prtip -sS -p 1-1000 TARGET     # SYN scan (stealth)
sudo prtip -sU -p 53,161 TARGET     # UDP scan

# Service Detection
sudo prtip -sS -sV -p 1-1000 TARGET              # Version detection
sudo prtip -sS -O -p 1-1000 TARGET               # OS detection
sudo prtip -A -p 1-1000 TARGET                   # Aggressive (all)

# Output
sudo prtip -sS -p 80,443 TARGET -oN results.txt  # Normal
sudo prtip -sS -p 80,443 TARGET -oJ results.json # JSON
sudo prtip -sS -p 80,443 TARGET -oA results      # All formats
```

### Common Port Reference

| Port | Service | Description |
|------|---------|-------------|
| 20/21 | FTP | File Transfer Protocol |
| 22 | SSH | Secure Shell |
| 23 | Telnet | Unencrypted text |
| 25 | SMTP | Email (sending) |
| 53 | DNS | Domain Name System |
| 80 | HTTP | Web traffic |
| 110 | POP3 | Email (receiving) |
| 143 | IMAP | Email (receiving) |
| 443 | HTTPS | Secure web traffic |
| 3306 | MySQL | MySQL database |
| 3389 | RDP | Remote Desktop Protocol |
| 5432 | PostgreSQL | PostgreSQL database |
| 8080 | HTTP-Alt | Alternative HTTP |

---

## Next Steps

- **[Learn Scan Types](./scan-types.md)** - TCP, UDP, stealth scanning techniques
- **[Master Timing & Performance](./timing-performance.md)** - Optimize scan speed
- **[Explore Output Formats](./output-formats.md)** - JSON, XML, Greppable formats
- **[Advanced Usage](./advanced-usage.md)** - IPv6, plugins, automation

**See Also:**
- [Quick Start Guide](../getting-started/quick-start.md)
- [Tutorials](../getting-started/tutorials.md)
- [Examples Gallery](../getting-started/examples.md)
