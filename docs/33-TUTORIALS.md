# ProRT-IP Tutorials

**Version:** 2.0.0 (Enhanced)
**Last Updated:** 2025-11-07
**Target Audience:** Progressive learning (beginner → advanced)

---

## Table of Contents

1. [Introduction](#introduction)
2. [Beginner Tutorials](#beginner-tutorials)
3. [Intermediate Tutorials](#intermediate-tutorials)
4. [Advanced Tutorials](#advanced-tutorials)
5. [Practice Exercises](#practice-exercises)
6. [Common Pitfalls](#common-pitfalls)

---

## Introduction

### How to Use These Tutorials

**Progressive Learning Path:**
1. **Beginners:** Start with Tutorial 1, work through sequentially
2. **Intermediate Users:** Jump to intermediate tutorials after basics
3. **Advanced Users:** Focus on advanced tutorials and exercises

**Tutorial Format:**
- **Objective:** Clear goal for the tutorial
- **Prerequisites:** What you need before starting
- **Estimated Time:** How long it takes
- **Steps:** Detailed walkthrough
- **Expected Output:** What you should see
- **Troubleshooting:** Common issues and solutions
- **Next Steps:** What to learn next

**Hands-On Practice:**
All tutorials are hands-on. Type commands, observe results, experiment!

---

## Beginner Tutorials

### Tutorial 1: Your First Scan

**Objective:** Complete a basic port scan and understand the output

**Prerequisites:**
- ProRT-IP installed (see [32-USER-GUIDE.md](32-USER-GUIDE.md))
- Terminal access
- Internet connection (for test target)

**Estimated Time:** 15 minutes

---

#### Step 1: Verify Installation

**Command:**
```bash
prtip --version
```

**Expected Output:**
```
ProRT-IP v0.5.0
```

**Troubleshooting:**
- **Command not found:** Check installation, add to PATH
- **Wrong version:** Update to latest release

---

#### Step 2: Understanding the Command

**Basic Syntax:**
```bash
prtip [OPTIONS] <TARGET>
```

**Example:**
```bash
prtip -sT -p 80,443 scanme.nmap.org
```

**Breakdown:**
- `prtip`: Program name
- `-sT`: TCP Connect scan (safe, no root required)
- `-p 80,443`: Scan ports 80 and 443
- `scanme.nmap.org`: Target (Nmap's test server)

---

#### Step 3: Run Your First Scan

**Command:**
```bash
prtip -sT -p 80,443 scanme.nmap.org
```

**Expected Output:**
```
ProRT-IP v0.5.0 - Network Scanner
Starting scan at 2025-11-07 10:30:15

Resolving scanme.nmap.org... 45.33.32.156

Scanning 45.33.32.156...
PORT    STATE  SERVICE
80/tcp  open   http
443/tcp open   https

Scan complete: 2 ports scanned in 0.42 seconds
1 hosts up, 2 ports open
```

**What Happened:**
1. **DNS Resolution:** scanme.nmap.org → IP address
2. **Port Probing:** Attempted connections to ports 80 and 443
3. **State Detection:** Both ports responded (open)
4. **Service Guess:** Port 80 = HTTP, 443 = HTTPS

---

#### Step 4: Save Results to File

**Command:**
```bash
prtip -sT -p 80,443 scanme.nmap.org -oN first_scan.txt
```

**Expected Output:**
```
...
Results saved to: first_scan.txt
```

**View File:**
```bash
cat first_scan.txt
```

**What's in the File:**
- Scan metadata (time, target, command)
- Port scan results (same as terminal output)
- Summary statistics

---

#### Step 5: Scan Localhost

**Command:**
```bash
prtip -sT -p 22,80,443,3306 127.0.0.1
```

**Expected Output (will vary based on your services):**
```
Scanning 127.0.0.1...
PORT     STATE  SERVICE
22/tcp   open   ssh          # If SSH running
80/tcp   closed http         # If no web server
443/tcp  closed https
3306/tcp open   mysql        # If MySQL running

Scan complete: 4 ports scanned in 0.05 seconds
```

**Interpretation:**
- **open:** Service is running and accepting connections
- **closed:** Port is accessible but no service listening
- **filtered:** Firewall blocking (not shown in this example)

---

#### Next Steps

- **Tutorial 2:** Learn different scan types (SYN, UDP, stealth)
- **User Guide:** Explore [32-USER-GUIDE.md](32-USER-GUIDE.md) for more options
- **Practice:** Try scanning different ports: `-p 1-100`

> **See Also:**
> - [User Guide - Getting Started](32-USER-GUIDE.md#getting-started) - Installation and first steps
> - [Architecture Guide](00-ARCHITECTURE.md) - How ProRT-IP works internally
> - [Example: Basic SYN Scan](34-EXAMPLES-GALLERY.md#common_basic_syn_scan) - Copy-paste ready code

---

### Tutorial 2: Understanding Scan Types

**Objective:** Learn when and how to use different scan types

**Prerequisites:**
- Completed Tutorial 1
- Root/administrator access (for SYN/UDP scans)

**Estimated Time:** 20 minutes

---

#### Part 1: TCP Connect Scan (-sT)

**When to Use:**
- No root/admin privileges available
- Scanning localhost safely
- Learning and testing

**How It Works:**
- Full TCP three-way handshake (SYN → SYN/ACK → ACK)
- Target logs full connection
- Slower than SYN scan

**Command:**
```bash
prtip -sT -p 80,443,22 scanme.nmap.org
```

**Expected Time:** ~0.5 seconds for 3 ports

---

#### Part 2: TCP SYN Scan (-sS)

**When to Use:**
- Root/admin privileges available
- Stealth required (less logging)
- Faster scanning needed

**How It Works:**
- Half-open scan (SYN → SYN/ACK → RST)
- Doesn't complete handshake (stealthier)
- Faster than Connect scan

**Command:**
```bash
sudo prtip -sS -p 80,443,22 scanme.nmap.org
```

**Expected Time:** ~0.3 seconds for 3 ports (30-40% faster)

**Comparison:**
```
TCP Connect (-sT):  SYN → SYN/ACK → ACK → RST
TCP SYN (-sS):      SYN → SYN/ACK → RST (half-open)
```

---

#### Part 3: UDP Scan (-sU)

**When to Use:**
- Scanning UDP services (DNS, SNMP, NTP)
- Comprehensive network mapping

**How It Works:**
- Sends UDP packets with protocol-specific payloads
- ICMP unreachable = closed
- No response = open|filtered (timeout)

**Command:**
```bash
sudo prtip -sU -p 53,161,123,137 scanme.nmap.org
```

**Expected Time:** ~5-10 seconds (much slower than TCP)

**Why Slower:**
- UDP is connectionless (no SYN/ACK confirmation)
- Relies on timeouts or ICMP responses
- Some firewalls rate-limit ICMP unreachable messages

---

#### Part 4: Stealth Scans (FIN/NULL/Xmas)

**When to Use:**
- Evading basic firewall rules
- Testing firewall effectiveness
- Red team engagements

**How They Work:**
- **FIN (-sF):** Sends FIN flag (connection termination)
- **NULL (-sN):** No flags set
- **Xmas (-sX):** FIN+PSH+URG flags (packet "lit up like a Christmas tree")

**Commands:**
```bash
sudo prtip -sF -p 80,443 scanme.nmap.org  # FIN scan
sudo prtip -sN -p 80,443 scanme.nmap.org  # NULL scan
sudo prtip -sX -p 80,443 scanme.nmap.org  # Xmas scan
```

**Expected Behavior:**
- **Closed ports:** Respond with RST
- **Open ports:** No response (timeout) → indicates open|filtered
- **Filtered:** Firewalls may drop unusual packets

**Limitations:**
- Doesn't work on Windows targets (RFC violation)
- Some firewalls detect and block these

---

#### Part 5: ACK Scan (-sA)

**When to Use:**
- Mapping firewall rulesets
- Determining filtered vs unfiltered ports

**How It Works:**
- Sends ACK packets (mid-connection acknowledgment)
- Unfiltered ports respond with RST
- Filtered ports drop packet or send ICMP unreachable

**Command:**
```bash
sudo prtip -sA -p 22,80,443,25 scanme.nmap.org
```

**Expected Output:**
```
PORT   STATE
22/tcp unfiltered  # Firewall allows SSH
80/tcp unfiltered  # Firewall allows HTTP
443/tcp unfiltered # Firewall allows HTTPS
25/tcp filtered    # Firewall blocks SMTP
```

**Use Case:** Firewall rule enumeration

---

#### Comparison Table

| Scan Type | Flag | Speed | Stealth | Root Required | Best For |
|-----------|------|-------|---------|---------------|----------|
| Connect   | -sT  | Medium| Low     | No            | Learning, no-root scenarios |
| SYN       | -sS  | Fast  | Medium  | Yes           | General scanning |
| UDP       | -sU  | Slow  | Low     | Yes           | DNS, SNMP, NTP services |
| FIN       | -sF  | Fast  | High    | Yes           | Firewall evasion |
| NULL      | -sN  | Fast  | High    | Yes           | Firewall evasion |
| Xmas      | -sX  | Fast  | High    | Yes           | Firewall evasion |
| ACK       | -sA  | Fast  | Medium  | Yes           | Firewall mapping |

---

#### Next Steps

- **Tutorial 3:** Learn service detection to identify versions
- **User Guide Section 3.2:** Deep dive into scan types

---

### Tutorial 3: Service Detection Basics

**Objective:** Identify services and versions running on open ports

**Prerequisites:**
- Completed Tutorial 2
- Root/admin access

**Estimated Time:** 25 minutes

---

#### Step 1: Basic Service Detection

**Command:**
```bash
sudo prtip -sS -sV -p 22,80,443,3306 scanme.nmap.org
```

**Expected Output:**
```
PORT     STATE  SERVICE  VERSION
22/tcp   open   ssh      OpenSSH 7.9p1 Debian (protocol 2.0)
80/tcp   open   http     Apache httpd 2.4.41 ((Debian))
443/tcp  open   https    Apache httpd 2.4.41 ((Debian))
3306/tcp closed mysql
```

**What Happened:**
1. **Port Scanning:** Identified open ports (22, 80, 443)
2. **Service Probing:** Sent protocol-specific requests
3. **Banner Grabbing:** Captured service responses
4. **Version Parsing:** Extracted service names and versions

---

#### Step 2: Understanding Detection Intensity

**Intensity Levels (0-9):**
- **0:** Lightest (fewest probes, fastest, least accurate)
- **7 (default):** Balanced (good accuracy, reasonable speed)
- **9:** Most thorough (all probes, slowest, highest accuracy)

**Low Intensity (Fast, Less Accurate):**
```bash
sudo prtip -sS -sV --version-intensity 3 -p 80,443 scanme.nmap.org
```

**High Intensity (Slow, More Accurate):**
```bash
sudo prtip -sS -sV --version-intensity 9 -p 80,443 scanme.nmap.org
```

**Comparison:**
- **Intensity 3:** ~1-2 seconds, 70-80% accuracy
- **Intensity 7:** ~3-5 seconds, 85-90% accuracy
- **Intensity 9:** ~10-15 seconds, 90-95% accuracy

---

#### Step 3: Banner Grabbing

**What is a Banner?**
Servers often announce themselves with service banners.

**Example HTTP Banner:**
```bash
sudo prtip -sS -sV -p 80 scanme.nmap.org
```

**Captured Banner:**
```
Server: Apache/2.4.41 (Debian)
X-Powered-By: PHP/7.4.3
```

**Use Cases:**
- **Version Identification:** Apache 2.4.41
- **OS Hint:** Debian Linux
- **Technology Stack:** PHP 7.4.3

---

#### Step 4: SSL/TLS Service Detection

**Command:**
```bash
sudo prtip -sS -sV -p 443 scanme.nmap.org
```

**Expected Output:**
```
PORT    STATE SERVICE  VERSION
443/tcp open  https    Apache httpd 2.4.41 (TLS certificate: CN=scanme.nmap.org)
```

**Additional TLS Info:**
- Certificate common name (CN)
- Issuer (Let's Encrypt, etc.)
- Expiration date

**See:** [27-TLS-CERTIFICATE-GUIDE.md](27-TLS-CERTIFICATE-GUIDE.md) for detailed TLS analysis

---

#### Step 5: Database Service Detection

**MySQL Example:**
```bash
sudo prtip -sS -sV -p 3306 192.168.1.10
```

**Expected Output:**
```
PORT     STATE SERVICE VERSION
3306/tcp open  mysql   MySQL 8.0.33-0ubuntu0.22.04.2
```

**PostgreSQL Example:**
```bash
sudo prtip -sS -sV -p 5432 192.168.1.10
```

**Expected Output:**
```
PORT     STATE SERVICE    VERSION
5432/tcp open  postgresql PostgreSQL 14.8 (Ubuntu 14.8-0ubuntu0.22.04.1)
```

---

#### Next Steps

- **Tutorial 4:** Advanced service detection with plugins
- **User Guide Use Case 3:** More service detection examples

> **See Also:**
> - [Service Detection Guide](24-SERVICE-DETECTION.md) - Deep-dive into detection algorithms
> - [TLS Certificate Analysis](27-TLS-CERTIFICATE-GUIDE.md) - X.509v3 parsing details
> - [Example: Service Fingerprinting](34-EXAMPLES-GALLERY.md#common_service_detection) - Production examples

---

## Intermediate Tutorials

### Tutorial 4: Advanced Service Detection

**Objective:** Use plugins and custom probes for enhanced detection

**Prerequisites:**
- Completed Tutorial 3
- Plugin system enabled (v0.4.8+)

**Estimated Time:** 30 minutes

---

#### Step 1: Enable Plugin System

**Verify Plugins Available:**
```bash
prtip --list-plugins
```

**Expected Output:**
```
Available Plugins:
- banner-analyzer (v1.0.0): Enhanced banner parsing for 8 services
- ssl-checker (v1.0.0): TLS certificate validation and analysis
```

---

#### Step 2: Use banner-analyzer Plugin

**Command:**
```bash
sudo prtip -sS -sV --plugin banner-analyzer -p 22,80,443 scanme.nmap.org
```

**Expected Output:**
```
PORT    STATE SERVICE  VERSION                           PLUGIN ANALYSIS
22/tcp  open  ssh      OpenSSH 7.9p1 Debian             [banner-analyzer] Ubuntu 20.04 detected
80/tcp  open  http     Apache/2.4.41 (Debian)           [banner-analyzer] PHP 7.4.3 backend
443/tcp open  https    Apache/2.4.41 (TLS 1.3)          [banner-analyzer] TLS 1.3, strong ciphers
```

**Plugin Benefits:**
- **Enhanced Parsing:** Extracts OS versions from package strings
- **Technology Detection:** Identifies backend frameworks (PHP, Node.js)
- **Security Analysis:** Flags outdated versions

---

#### Step 3: TLS Certificate Analysis

**Command:**
```bash
sudo prtip -sS --plugin ssl-checker -p 443 scanme.nmap.org
```

**Expected Output:**
```
PORT    STATE SERVICE  TLS ANALYSIS
443/tcp open  https    [ssl-checker] ✓ Valid certificate
                       Issuer: Let's Encrypt Authority X3
                       Valid: 2025-01-01 to 2025-04-01
                       CN: scanme.nmap.org
                       SAN: scanme.nmap.org, www.scanme.nmap.org
                       ✓ Chain valid, ✓ Not self-signed
```

---

#### Step 4: Custom HTTP Header Analysis

**Command:**
```bash
sudo prtip -sS -sV -p 80 --http-headers scanme.nmap.org
```

**Captured Headers:**
```
HTTP/1.1 200 OK
Server: Apache/2.4.41 (Debian)
X-Powered-By: PHP/7.4.3
X-Frame-Options: SAMEORIGIN
Content-Security-Policy: default-src 'self'
```

**Security Insights:**
- **X-Frame-Options:** Clickjacking protection
- **CSP:** Content Security Policy enabled
- **Server Version:** Apache 2.4.41 (check for vulnerabilities)

---

#### Step 5: SMB Dialect Enumeration

**Command:**
```bash
sudo prtip -sS -sV -p 445 192.168.1.10
```

**Expected Output:**
```
PORT    STATE SERVICE VERSION
445/tcp open  smb     SMB2/SMB3 (Windows Server 2019)
                      Dialects: SMB 2.02, 2.10, 3.00, 3.02, 3.11
```

**Windows Version Mapping:**
- SMB 3.11 → Windows Server 2019 / Windows 10 1809+
- SMB 3.02 → Windows Server 2012 R2 / Windows 8.1
- SMB 2.10 → Windows Server 2012 / Windows 8

**See:** [24-SERVICE-DETECTION.md](24-SERVICE-DETECTION.md) for SMB details

---

#### Next Steps

- **Tutorial 5:** Learn stealth scanning techniques
- **Plugin Development:** [30-PLUGIN-SYSTEM-GUIDE.md](30-PLUGIN-SYSTEM-GUIDE.md)

---

### Tutorial 5: Stealth Scanning Techniques

**Objective:** Evade intrusion detection systems (IDS) and firewalls

**Prerequisites:**
- Completed Tutorial 4
- Understanding of network security

**Estimated Time:** 35 minutes

**Warning:** Use only on networks you own or have written permission to test.

---

#### Step 1: Timing Control

**Paranoid Timing (T0):**
```bash
sudo prtip -sS -T0 -p 80,443 192.168.1.10
```

**Expected Time:** ~10 minutes for 2 ports (5-minute delays between probes)

**When to Use:**
- Evading sophisticated IDS
- Penetration testing in highly monitored environments
- Maximum stealth required

---

**Normal Timing (T3, default):**
```bash
sudo prtip -sS -T3 -p 80,443 192.168.1.10
```

**Expected Time:** ~0.5 seconds for 2 ports

---

**Comparison:**
| Template | Delay Between Probes | IDS Detection Likelihood |
|----------|----------------------|--------------------------|
| T0       | 5 minutes            | Very Low |
| T1       | 15 seconds           | Low |
| T2       | 400ms                | Medium |
| T3       | 100ms (default)      | High |
| T4       | 10ms                 | Very High |
| T5       | 0ms                  | Certain |

---

#### Step 2: Packet Fragmentation

**Command:**
```bash
sudo prtip -sS -f -p 80,443 192.168.1.10
```

**How It Works:**
- Splits TCP packets into smaller fragments
- Each fragment ≤8 bytes
- Evades simple packet inspection

**Expected Behavior:**
- **Modern Firewalls:** May reassemble and detect
- **Legacy Firewalls:** May allow fragmented packets through

**Custom MTU:**
```bash
sudo prtip -sS --mtu 16 -p 80,443 192.168.1.10
```

- MTU (Maximum Transmission Unit) sets fragment size
- Must be multiple of 8

---

#### Step 3: TTL Manipulation

**Command:**
```bash
sudo prtip -sS --ttl 64 -p 80,443 192.168.1.10
```

**Purpose:**
- Mimic different operating systems
- Bypass TTL-based filters

**Common TTL Values:**
- **Linux:** 64
- **Windows:** 128
- **Cisco:** 255

**Example: Mimic Windows from Linux:**
```bash
sudo prtip -sS --ttl 128 -p 80,443 192.168.1.10
```

---

#### Step 4: Decoy Scanning

**Random Decoys:**
```bash
sudo prtip -sS -D RND:10 -p 80,443 192.168.1.10
```

**How It Works:**
- Scanner uses 10 random IP addresses as decoys
- Target sees scans from 11 sources (10 decoys + your IP)
- Difficult to identify real scanner

**Manual Decoys:**
```bash
sudo prtip -sS -D 192.168.1.5,192.168.1.7,ME,192.168.1.9 -p 80,443 192.168.1.10
```

**Explanation:**
- `192.168.1.5, 192.168.1.7`: Decoy IPs
- `ME`: Your real IP (position matters for stealth)
- `192.168.1.9`: Another decoy

**Target's Logs:**
```
[2025-11-07 10:30:15] Port 80 scan from 192.168.1.5
[2025-11-07 10:30:15] Port 80 scan from 192.168.1.7
[2025-11-07 10:30:15] Port 80 scan from 192.168.1.2 (YOUR IP)
[2025-11-07 10:30:15] Port 80 scan from 192.168.1.9
```

**Best Practice:** Place `ME` in middle of decoy list for better anonymity

---

#### Step 5: Idle/Zombie Scan

**Command:**
```bash
# 1. Find zombie host (sequential IPID)
sudo prtip -sI RND 192.168.1.0/24

# 2. Use discovered zombie
sudo prtip -sI 192.168.1.5 -p 80,443 192.168.1.10
```

**How It Works:**
1. Scan zombie's IPID (before)
2. Spoof SYN packet to target (source = zombie IP)
3. Target responds to zombie
4. Scan zombie's IPID (after)
5. IPID increment reveals port state

**IPID Delta Interpretation:**
- **+0:** Filtered (no response from target)
- **+1:** Closed (target sent RST to zombie)
- **+2:** Open (target sent SYN/ACK to zombie, zombie sent RST)

**Advantages:**
- **Maximum Anonymity:** Target never sees your IP
- **Evades Logging:** Only zombie IP logged

**Disadvantages:**
- **Very Slow:** 500-800ms per port
- **Requires Zombie:** Suitable zombie hosts rare
- **Complex:** Multi-step process

**See:** [25-IDLE-SCAN-GUIDE.md](25-IDLE-SCAN-GUIDE.md) for complete guide

---

#### Step 6: Combined Evasion Techniques

**Maximum Stealth:**
```bash
sudo prtip -sS -T1 -f --ttl 64 -D RND:5 -g 53 -p 80,443 192.168.1.10
```

**Breakdown:**
- `-T1`: Sneaky timing (15s delays)
- `-f`: Fragmentation
- `--ttl 64`: Custom TTL
- `-D RND:5`: 5 random decoys
- `-g 53`: Source port 53 (DNS, often allowed through firewalls)

**Expected Time:** ~3-5 minutes for 2 ports

**Effectiveness:**
- **Basic IDS:** High evasion likelihood
- **Modern IDS (Snort, Suricata):** Medium evasion likelihood
- **Advanced IDS (Palo Alto, Fortinet):** Low evasion likelihood

---

#### Next Steps

- **Tutorial 6:** Large-scale scanning for internet-wide enumeration
- **Evasion Guide:** [19-EVASION-GUIDE.md](19-EVASION-GUIDE.md)

> **See Also:**
> - [Evasion Guide](19-EVASION-GUIDE.md) - Complete stealth techniques reference
> - [Idle Scan Guide](25-IDLE-SCAN-GUIDE.md) - Maximum anonymity scanning
> - [Example: Stealth Scanning](34-EXAMPLES-GALLERY.md#common_stealth_scan) - FIN/NULL/Xmas combinations

---

## Advanced Tutorials

### Tutorial 6: Large-Scale Scanning

**Objective:** Scan thousands of hosts efficiently and responsibly

**Prerequisites:**
- Completed Tutorial 5
- Understanding of network capacity
- **Written permission** for target networks

**Estimated Time:** 45 minutes

**Warning:** Large-scale scanning can disrupt networks. Use responsibly.

---

#### Step 1: Planning the Scan

**Questions to Answer:**
1. **Scope:** How many hosts? (100? 10,000? 1,000,000?)
2. **Timing:** How long can scan take? (minutes? hours? days?)
3. **Ports:** All ports or specific subset?
4. **Network Capacity:** What's the bandwidth limit?
5. **Courtesy:** What rate won't overwhelm targets?

**Example Scenario:**
- **Scope:** 10,000 hosts (10.0.0.0/16 subnet, partial)
- **Timing:** 30 minutes max
- **Ports:** Top 100 (common services)
- **Network:** 1 Gbps corporate LAN
- **Courtesy:** 1,000 packets/second max

---

#### Step 2: Calculate Required Rate

**Math:**
```
Hosts: 10,000
Ports per host: 100
Total probes: 10,000 × 100 = 1,000,000
Time limit: 30 minutes = 1,800 seconds
Required rate: 1,000,000 ÷ 1,800 ≈ 555 packets/second
```

**Add 20% buffer:** 555 × 1.2 ≈ 666 pps

**Command:**
```bash
sudo prtip -sS -F --max-rate 666 10.0.0.0/16 -oG large_scan.gnmap
```

---

#### Step 3: Performance Tuning

**Enable NUMA (Linux):**
```bash
sudo prtip -sS -F --max-rate 1000 --numa 10.0.0.0/16 -oG scan.gnmap
```

**Benefits:**
- 10-30% performance improvement on NUMA systems
- Better CPU core utilization

**Increase Batch Size:**
```bash
sudo prtip -sS -F --max-rate 1000 --batch-size 2000 10.0.0.0/16 -oG scan.gnmap
```

**Benefits:**
- More parallel connections
- **Warning:** Higher memory usage, ensure adequate RAM

---

#### Step 4: Output Management

**Greppable Output (Best for Large Scans):**
```bash
sudo prtip -sS -F --max-rate 1000 10.0.0.0/16 -oG scan.gnmap
```

**Why Greppable:**
- One line per host (easy to parse with grep/awk)
- Smaller file size vs XML/JSON
- Streaming-friendly

**Post-Processing:**
```bash
# Extract hosts with port 22 open
grep "22/open" scan.gnmap > ssh_hosts.txt

# Count hosts by open ports
grep -o "[0-9]*/open" scan.gnmap | cut -d'/' -f1 | sort | uniq -c
```

---

#### Step 5: Monitor Progress

**Real-Time Stats:**
```bash
sudo prtip -sS -F --max-rate 1000 --stats-interval 10 10.0.0.0/16 -oG scan.gnmap
```

**Output Every 10 Seconds:**
```
[2025-11-07 10:30:00] Progress: 1,234 / 10,000 hosts (12.34%)
                      Rate: 850 pps
                      ETA: 22 minutes 15 seconds
                      Open ports found: 456
```

---

#### Step 6: Distributed Scanning (Future)

**Concept:** Split scan across multiple machines

**Example (Manual):**
```bash
# Machine 1: Scan 10.0.0.0/17 (first half)
sudo prtip -sS -F --max-rate 1000 10.0.0.0/17 -oG scan1.gnmap

# Machine 2: Scan 10.0.128.0/17 (second half)
sudo prtip -sS -F --max-rate 1000 10.0.128.0/17 -oG scan2.gnmap

# Merge results
cat scan1.gnmap scan2.gnmap > merged.gnmap
```

**Note:** Distributed scanning feature planned for future release

---

#### Next Steps

- **Tutorial 7:** Custom plugin development for specialized detection
- **Benchmarking:** [31-BENCHMARKING-GUIDE.md](31-BENCHMARKING-GUIDE.md)

> **See Also:**
> - [Benchmarking Guide](31-BENCHMARKING-GUIDE.md) - Performance optimization techniques
> - [Rate Limiting Guide](26-RATE-LIMITING-GUIDE.md) - V3 algorithm deep-dive
> - [NUMA Optimization](00-ARCHITECTURE.md#numa-optimization) - Thread pinning best practices

---

### Tutorial 7: Custom Plugin Development

**Objective:** Create a Lua plugin for custom service detection

**Prerequisites:**
- Completed Tutorial 6
- Basic Lua programming knowledge
- Plugin system enabled (v0.4.8+)

**Estimated Time:** 60 minutes

---

#### Step 1: Plugin System Overview

**Plugin Types:**
1. **DetectionPlugin:** Analyze service banners
2. **OutputPlugin:** Custom output formats
3. **ScanPlugin:** Custom scan logic

**This Tutorial:** DetectionPlugin for Tomcat version detection

---

#### Step 2: Create Plugin Directory

**Commands:**
```bash
mkdir -p ~/.prtip/plugins/tomcat-detector
cd ~/.prtip/plugins/tomcat-detector
```

---

#### Step 3: Create plugin.toml Metadata

**File:** `~/.prtip/plugins/tomcat-detector/plugin.toml`

**Content:**
```toml
[plugin]
name = "tomcat-detector"
version = "1.0.0"
author = "Your Name"
description = "Enhanced Apache Tomcat version detection"
type = "DetectionPlugin"

[capabilities]
network = true      # Needs network access
filesystem = false  # No file system access
system = false      # No system calls
database = false    # No database access

[config]
default_ports = [8080, 8443]
timeout = 5000  # milliseconds
```

---

#### Step 4: Create init.lua Plugin Logic

**File:** `~/.prtip/plugins/tomcat-detector/init.lua`

**Content:**
```lua
-- Tomcat Detector Plugin v1.0.0

function on_load()
    prtip.log("INFO", "Tomcat Detector plugin loaded")
    return true
end

function analyze_banner(target, port, banner)
    -- Check if banner contains Tomcat signature
    if not banner:match("Apache%-Tomcat") and not banner:match("Server: Apache Tomcat") then
        return nil  -- Not Tomcat
    end

    -- Extract version
    local version = banner:match("Apache%-Tomcat/([%d%.]+)")
    if not version then
        version = banner:match("Apache Tomcat/([%d%.]+)")
    end

    -- Map version to CVE vulnerabilities (example)
    local vulnerabilities = {}
    if version then
        local major = tonumber(version:match("^(%d+)"))
        if major and major < 9 then
            table.insert(vulnerabilities, "CVE-2020-1938 (Ghostcat)")
        end
    end

    -- Create ServiceInfo result
    local info = {
        service = "http",
        product = "Apache Tomcat",
        version = version or "unknown",
        os_hint = "Cross-platform (Java)",
        cpe = version and ("cpe:/a:apache:tomcat:" .. version) or nil,
        extra = {
            vulnerabilities = vulnerabilities,
            technology = "Java Servlet Container"
        }
    }

    return info
end

function on_unload()
    prtip.log("INFO", "Tomcat Detector plugin unloaded")
    return true
end
```

---

#### Step 5: Test Plugin

**Command:**
```bash
sudo prtip -sS -sV --plugin tomcat-detector -p 8080 192.168.1.10
```

**Expected Output:**
```
PORT     STATE SERVICE VERSION
8080/tcp open  http    Apache Tomcat/9.0.65
                       [tomcat-detector] ✓ Tomcat 9.0.65 detected
                       Technology: Java Servlet Container
                       No known CVEs for this version
```

---

#### Step 6: Debugging Plugin

**Enable Debug Logs:**
```bash
RUST_LOG=debug sudo -E prtip -sS -sV --plugin tomcat-detector -p 8080 192.168.1.10
```

**Check Plugin Loading:**
```
[2025-11-07 10:30:00] DEBUG: Loading plugin: tomcat-detector
[2025-11-07 10:30:00] INFO: Tomcat Detector plugin loaded
[2025-11-07 10:30:05] DEBUG: Analyzing banner for 192.168.1.10:8080
[2025-11-07 10:30:05] INFO: Tomcat 9.0.65 detected on 192.168.1.10:8080
```

---

#### Step 7: Share Plugin

**Package Plugin:**
```bash
cd ~/.prtip/plugins
tar -czf tomcat-detector-v1.0.0.tar.gz tomcat-detector/
```

**Distribution:**
- Upload to GitHub
- Share in ProRT-IP community
- Submit to official plugin repository (future)

---

#### Next Steps

- **Plugin System Guide:** [30-PLUGIN-SYSTEM-GUIDE.md](30-PLUGIN-SYSTEM-GUIDE.md) for advanced features
- **Practice Exercises:** Try the exercises below

> **See Also:**
> - [Plugin System Guide](30-PLUGIN-SYSTEM-GUIDE.md) - Complete API reference
> - [Example Plugins](30-PLUGIN-SYSTEM-GUIDE.md#example-plugins) - banner-analyzer, ssl-checker
> - [Security Model](30-PLUGIN-SYSTEM-GUIDE.md#security-model) - Sandboxing and capabilities

---

## Practice Exercises

### Exercise 1: Network Mapping

**Objective:** Map your local network and document the topology

**Tasks:**
1. Discover all active hosts on your LAN (e.g., 192.168.1.0/24)
2. Identify operating systems of discovered hosts
3. Enumerate running services (top 100 ports)
4. Create network diagram showing hosts and services

**Hints:**
```bash
# Step 1: Discovery
sudo prtip -sn 192.168.1.0/24 -oN discovery.txt

# Step 2: OS detection on live hosts
sudo prtip -sS -O -p 1-1000 -iL live_hosts.txt -oN os_detection.txt

# Step 3: Service enumeration
sudo prtip -sS -sV -F -iL live_hosts.txt -oJ services.json
```

**Solution:**
See solution at end of document.

---

### Exercise 2: Service Enumeration on Web Server

**Objective:** Thoroughly enumerate a web server

**Target:** scanme.nmap.org (safe, legal target)

**Tasks:**
1. Identify all open ports (1-65535)
2. Detect web server version (Apache/Nginx/IIS)
3. Analyze TLS certificate
4. Check for HTTP security headers
5. Document findings

**Hints:**
```bash
# Full port scan
sudo prtip -sS -p- -T4 scanme.nmap.org -oN fullscan.txt

# Service detection on open ports
sudo prtip -sS -sV -p <open_ports> scanme.nmap.org

# TLS analysis
sudo prtip -sS --plugin ssl-checker -p 443 scanme.nmap.org
```

**Solution:**
See solution at end of document.

---

### Exercise 3: Firewall Rule Testing

**Objective:** Test firewall effectiveness

**Setup:** Configure iptables rules on test system

**Tasks:**
1. Allow ports 22, 80, 443
2. Block port 25 (SMTP)
3. Verify rules with ACK scan
4. Attempt evasion with fragmentation
5. Document results

**Hints:**
```bash
# Setup (on test system)
sudo iptables -A INPUT -p tcp --dport 22 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 25 -j DROP

# Test from scanner
sudo prtip -sA -p 22,25,80,443 <test_system_ip>
sudo prtip -sS -f -p 22,25,80,443 <test_system_ip>
```

**Solution:**
See solution at end of document.

---

### Exercise 4: Scan Type Speed Comparison

**Objective:** Benchmark different scan types

**Target:** localhost (safe)

**Tasks:**
1. Time TCP Connect scan (ports 1-1000)
2. Time SYN scan (ports 1-1000)
3. Time UDP scan (ports 1-100)
4. Calculate speed ratios
5. Explain performance differences

**Hints:**
```bash
time prtip -sT -p 1-1000 127.0.0.1
time sudo prtip -sS -p 1-1000 127.0.0.1
time sudo prtip -sU -p 1-100 127.0.0.1
```

**Expected Ratios:**
- SYN scan: ~30-40% faster than Connect
- UDP scan: ~10-100x slower than TCP

**Solution:**
See solution at end of document.

---

### Exercise 5: Write Custom Lua Plugin

**Objective:** Create plugin to detect Node.js version

**Tasks:**
1. Research Node.js server banners (Express, Koa, etc.)
2. Create plugin structure (plugin.toml + init.lua)
3. Implement banner parsing logic
4. Test on local Node.js server
5. Document usage

**Hints:**
- Look for "X-Powered-By: Express" header
- Extract version from "Server: Express/4.18.2"
- Handle variations (Express, Koa, Fastify)

**Solution:**
See solution at end of document.

---

### Exercise 6: Discover All Web Servers in Subnet

**Objective:** Find all HTTP/HTTPS servers in 192.168.1.0/24

**Prerequisites:**
- ProRT-IP installed
- Access to 192.168.1.0/24 network
- Root/sudo access

**Estimated Time:** 15-20 minutes

**Tasks:**
1. Scan subnet for open HTTP/HTTPS ports (80, 443, 8080, 8443)
2. Identify web server versions
3. Extract HTTP headers
4. Create inventory report

**Hints:**
```bash
# Step 1: Quick scan for web ports
sudo prtip -sS -p 80,443,8080,8443 192.168.1.0/24 -oN webscan.txt

# Step 2: Service detection on discovered hosts
# Extract live IPs first, then detect versions

# Step 3: Banner grabbing for HTTP headers
sudo prtip -sS -sV --http-headers -p 80,443 <discovered_hosts>
```

**Expected Output:**
```
192.168.1.10:80    - Apache 2.4.41 (Ubuntu)
192.168.1.10:443   - Apache 2.4.41 (TLS 1.3)
192.168.1.15:8080  - Nginx 1.18.0
192.168.1.20:443   - IIS 10.0 (Windows Server 2019)
```

**Solution:** See Solution 6 below

---

### Exercise 7: Detect SSH Versions on Network

**Objective:** Identify all SSH servers and their versions on remote network

**Prerequisites:**
- Access to target network
- Root/sudo access
- Permission to scan network

**Estimated Time:** 15-20 minutes

**Tasks:**
1. Discover hosts with SSH (port 22)
2. Extract SSH banner and version
3. Identify OS distributions from SSH package strings
4. Flag outdated versions (security audit)

**Hints:**
```bash
# Step 1: Find SSH servers
sudo prtip -sS -p 22 10.0.0.0/24 -oG ssh_hosts.gnmap

# Step 2: Service detection
sudo prtip -sS -sV -p 22 <discovered_hosts>

# Step 3: Look for version patterns
# OpenSSH 7.x = Ubuntu 18.04 / Debian 9
# OpenSSH 8.x = Ubuntu 20.04+ / Debian 10+
```

**Expected Output:**
```
10.0.0.5:22   - OpenSSH 7.9p1 Debian (Ubuntu 18.04)
10.0.0.10:22  - OpenSSH 8.9p1 Ubuntu (Ubuntu 22.04)
10.0.0.15:22  - OpenSSH 7.4 (CentOS 7)
10.0.0.20:22  - Dropbear sshd 2019.78 (embedded device)
```

**Security Check:**
- OpenSSH <7.4: VULNERABLE to CVE-2016-10009
- OpenSSH <8.3: VULNERABLE to CVE-2020-15778
- Dropbear <2020.81: VULNERABLE to CVE-2021-36369

**Solution:** See Solution 7 below

---

### Exercise 8: Write Custom Node.js Detection Plugin

**Objective:** Create Lua plugin to detect Node.js frameworks from HTTP headers

**Prerequisites:**
- Plugin system enabled (v0.4.8+)
- Basic Lua programming knowledge
- Local Node.js/Express server for testing

**Estimated Time:** 30-40 minutes

**Tasks:**
1. Create plugin directory structure
2. Write plugin.toml metadata
3. Implement banner analysis logic
4. Test on local Node.js server
5. Handle edge cases (Koa, Fastify, missing headers)

**Hints:**
- Look for "X-Powered-By: Express" header
- Extract version from "Server: Express/4.18.2"
- Handle variations: Express, Koa, Fastify, Hapi

**Template:**
```lua
function analyze_banner(target, port, banner)
    -- Check for Node.js frameworks
    local frameworks = {
        Express = banner:match("Express/([%d%.]+)"),
        Koa = banner:match("Koa/([%d%.]+)"),
        Fastify = banner:match("Fastify/([%d%.]+)")
    }

    -- Return ServiceInfo if detected
    -- Return nil if not Node.js
end
```

**Expected Detection:**
```
192.168.1.10:3000 - Express 4.18.2 (Node.js)
192.168.1.15:8080 - Fastify 4.10.2 (Node.js)
192.168.1.20:5000 - Koa 2.14.1 (Node.js)
```

**Solution:** See Solution 8 below

---

### Exercise 9: Optimize Scan Speed for 10K Hosts

**Objective:** Reduce scan time from 30min to <5min for 10,000 hosts

**Prerequisites:**
- 10.0.0.0/16 test network (or use localhost with simulated hosts)
- Benchmark tools (time, hyperfine)
- Understanding of timing templates

**Estimated Time:** 25-30 minutes

**Tasks:**
1. Baseline: Measure default scan time
2. Tune timing template (T0 → T4)
3. Optimize rate limiting
4. Adjust batch size for memory/speed
5. Compare results and document

**Hints:**
```bash
# Baseline (default T3)
time sudo prtip -sS -p 80,443 10.0.0.0/18 -oN baseline.txt

# Optimization 1: Aggressive timing
time sudo prtip -sS -T4 -p 80,443 10.0.0.0/18 -oN t4.txt

# Optimization 2: Higher rate limit
time sudo prtip -sS -T4 --max-rate 50000 -p 80,443 10.0.0.0/18 -oN rate.txt

# Optimization 3: Batch size tuning
time sudo prtip -sS -T4 --max-rate 50000 --batch-size 2000 -p 80,443 10.0.0.0/18 -oN batch.txt
```

**Metrics to Track:**
- Scan duration (total seconds)
- Throughput (packets per second)
- Memory usage (MB)
- Accuracy (% ports detected vs baseline)

**Expected Results:**
```
Baseline (T3):            28m 45s | 850 pps  | 128 MB | 100% accuracy
T4 timing:                12m 20s | 1,950 pps | 145 MB | 99.8% accuracy
+ max-rate 50K:            8m 35s | 3,100 pps | 142 MB | 99.5% accuracy
+ batch-size 2000:         4m 50s | 5,200 pps | 385 MB | 99.2% accuracy

Improvement: 83% faster (28m → 4m 50s)
Tradeoff: 0.8% accuracy loss, 3x memory usage
```

**Solution:** See Solution 9 below

---

## Solutions

### Solution 1: Network Mapping

**Discovery:**
```bash
sudo prtip -sn 192.168.1.0/24 -oN discovery.txt
```

**OS Detection:**
```bash
# Extract live IPs
grep "up" discovery.txt | cut -d' ' -f2 > live_hosts.txt

# OS scan
sudo prtip -sS -O -p 1-1000 -iL live_hosts.txt -oN os_detection.txt
```

**Service Enumeration:**
```bash
sudo prtip -sS -sV -F -iL live_hosts.txt -oJ services.json
```

**Network Diagram (example):**
```
192.168.1.1 (Router)        - Linux 5.15 - Services: 22/ssh, 80/http
192.168.1.10 (Web Server)   - Ubuntu 22.04 - Services: 22/ssh, 80/http, 443/https, 3306/mysql
192.168.1.20 (Database)     - Debian 11 - Services: 22/ssh, 5432/postgresql
192.168.1.30 (Desktop)      - Windows 11 - Services: 135/rpc, 139/smb, 445/smb, 3389/rdp
```

---

### Solution 2: Service Enumeration

**Full Port Scan:**
```bash
sudo prtip -sS -p- -T4 scanme.nmap.org -oN fullscan.txt
# Found: 22, 80 (others filtered/closed)
```

**Service Detection:**
```bash
sudo prtip -sS -sV -p 22,80 scanme.nmap.org
# 22/tcp: OpenSSH 7.9p1 Debian
# 80/tcp: Apache httpd 2.4.41
```

**TLS Analysis:**
```bash
sudo prtip -sS --plugin ssl-checker -p 443 scanme.nmap.org
# (If 443 is open, otherwise N/A for scanme.nmap.org)
```

---

### Solution 3: Firewall Testing

**ACK Scan Results:**
```bash
sudo prtip -sA -p 22,25,80,443 192.168.1.10
# 22/tcp: unfiltered
# 25/tcp: filtered
# 80/tcp: unfiltered
# 443/tcp: unfiltered
```

**Interpretation:**
- Ports 22, 80, 443: Firewall allows (unfiltered)
- Port 25: Firewall blocks (filtered)

**Fragmentation Test:**
```bash
sudo prtip -sS -f -p 25 192.168.1.10
# Result: Still filtered (modern firewalls reassemble fragments)
```

---

### Solution 4: Speed Comparison

**Results (example):**
```
TCP Connect: 5.23 seconds
TCP SYN:     3.41 seconds (35% faster)
UDP:         48.7 seconds (1,428% slower)
```

**Explanation:**
- **SYN faster:** Half-open handshake (SYN → SYN/ACK → RST vs SYN → SYN/ACK → ACK → FIN)
- **UDP much slower:** No handshake confirmation, relies on timeouts or ICMP unreachable

---

### Solution 5: Node.js Plugin

**plugin.toml:**
```toml
[plugin]
name = "nodejs-detector"
version = "1.0.0"
author = "Your Name"
description = "Detect Node.js and popular frameworks"
type = "DetectionPlugin"

[capabilities]
network = true
```

**init.lua:**
```lua
function analyze_banner(target, port, banner)
    local frameworks = {
        Express = banner:match("Express/([%d%.]+)"),
        Koa = banner:match("Koa/([%d%.]+)"),
        Fastify = banner:match("Fastify/([%d%.]+)")
    }

    for framework, version in pairs(frameworks) do
        if version then
            return {
                service = "http",
                product = framework,
                version = version,
                extra = { technology = "Node.js" }
            }
        end
    end

    return nil
end
```

---

### Solution 6: Web Server Discovery

**Step 1: Scan subnet**
```bash
sudo prtip -sS -p 80,443,8080,8443 192.168.1.0/24 -oG webscan.gnmap
```

**Step 2: Extract live web servers**
```bash
grep "80/open\|443/open\|8080/open\|8443/open" webscan.gnmap | \
  cut -d' ' -f2 > web_hosts.txt
```

**Step 3: Service detection**
```bash
sudo prtip -sS -sV --http-headers -p 80,443,8080,8443 -iL web_hosts.txt -oN web_inventory.txt
```

**Step 4: Parse results**
```bash
# Apache servers
grep "Apache" web_inventory.txt

# Nginx servers
grep "nginx" web_inventory.txt

# IIS servers
grep "IIS" web_inventory.txt
```

**Inventory Report:**
```
Total web servers found: 8
- Apache: 3 (37.5%)
- Nginx: 2 (25%)
- IIS: 2 (25%)
- Other: 1 (12.5%)
```

---

### Solution 7: SSH Version Detection

**Step 1: Find SSH servers**
```bash
sudo prtip -sS -p 22 10.0.0.0/24 -oG ssh.gnmap
grep "22/open" ssh.gnmap | cut -d' ' -f2 > ssh_hosts.txt
```

**Step 2: Version detection**
```bash
sudo prtip -sS -sV -p 22 -iL ssh_hosts.txt -oN ssh_versions.txt
```

**Step 3: Parse and categorize**
```bash
# OpenSSH versions
grep "OpenSSH" ssh_versions.txt | awk '{print $1, $5, $6}'

# Vulnerable versions (OpenSSH <7.4)
grep -E "OpenSSH [0-6]\." ssh_versions.txt
grep -E "OpenSSH 7\.[0-3]" ssh_versions.txt
```

**Security Audit:**
```
Total SSH servers: 12
- OpenSSH 8.x (secure): 6 (50%)
- OpenSSH 7.4-7.9 (secure): 4 (33%)
- OpenSSH <7.4 (VULNERABLE): 2 (17%) ⚠️

Action Required: Upgrade 2 servers (10.0.0.5, 10.0.0.15)
```

---

### Solution 8: Node.js Detection Plugin

**plugin.toml:**
```toml
[plugin]
name = "nodejs-detector"
version = "1.0.0"
author = "Your Name"
description = "Detect Node.js frameworks (Express, Koa, Fastify)"
type = "DetectionPlugin"

[capabilities]
network = true
```

**init.lua:**
```lua
function on_load()
    prtip.log("INFO", "Node.js Detector plugin loaded")
    return true
end

function analyze_banner(target, port, banner)
    -- Framework detection patterns
    local frameworks = {
        Express = banner:match("Express/([%d%.]+)"),
        Koa = banner:match("Koa/([%d%.]+)"),
        Fastify = banner:match("Fastify/([%d%.]+)"),
        Hapi = banner:match("hapi/([%d%.]+)")
    }

    -- Check X-Powered-By header
    if not banner:match("Express") and not banner:match("Koa") then
        local powered_by = banner:match("X%-Powered%-By: ([^\r\n]+)")
        if powered_by and powered_by:match("Express") then
            frameworks.Express = powered_by:match("Express/([%d%.]+)") or "unknown"
        end
    end

    -- Return first match
    for framework, version in pairs(frameworks) do
        if version then
            return {
                service = "http",
                product = framework,
                version = version,
                extra = {
                    technology = "Node.js",
                    framework = framework
                }
            }
        end
    end

    return nil  -- Not Node.js
end

function on_unload()
    prtip.log("INFO", "Node.js Detector plugin unloaded")
    return true
end
```

**Testing:**
```bash
# Test on local Express server
sudo prtip -sS -sV --plugin nodejs-detector -p 3000 127.0.0.1

# Expected output:
# 3000/tcp open http Express 4.18.2 (Node.js framework: Express)
```

---

### Solution 9: Performance Optimization

**Baseline Measurement:**
```bash
time sudo prtip -sS -p 80,443 10.0.0.0/18 -oN baseline.txt
# Result: 28m 45s (1,725 seconds)
```

**Optimization Steps:**

**Step 1: Timing template (T3 → T4)**
```bash
time sudo prtip -sS -T4 -p 80,443 10.0.0.0/18 -oN t4.txt
# Result: 12m 20s (740s) - 57% faster
```

**Step 2: Rate limiting**
```bash
time sudo prtip -sS -T4 --max-rate 50000 -p 80,443 10.0.0.0/18 -oN rate.txt
# Result: 8m 35s (515s) - 70% faster than baseline
```

**Step 3: Batch size**
```bash
time sudo prtip -sS -T4 --max-rate 50000 --batch-size 2000 -p 80,443 10.0.0.0/18 -oN batch.txt
# Result: 4m 50s (290s) - 83% faster than baseline
```

**Final Configuration:**
```bash
sudo prtip -sS -T4 --max-rate 50000 --batch-size 2000 -p 80,443 10.0.0.0/18
```

**Performance Summary:**
| Configuration | Time | Throughput | Memory | Speedup |
|---------------|------|------------|--------|---------|
| Baseline (T3) | 28m 45s | 850 pps | 128 MB | 1.0x |
| + T4 | 12m 20s | 1,950 pps | 145 MB | 2.3x |
| + max-rate | 8m 35s | 3,100 pps | 142 MB | 3.4x |
| + batch-size | 4m 50s | 5,200 pps | 385 MB | 5.9x |

**Recommendation:** Use optimized config for time-critical scans, baseline for accuracy-critical.

---

## Common Pitfalls

Learn from common mistakes and avoid frustration.

---

### Pitfall 1: Permission Denied - CAP_NET_RAW Missing

**Symptom:**
- Error: "Operation not permitted" or "Permission denied"
- Scan fails immediately after starting
- Only affects SYN/UDP/stealth scans (TCP Connect works)

**Example Error:**
```
Error: Failed to create raw socket
Caused by: Operation not permitted (os error 1)
```

**Root Cause:**
Raw socket creation requires elevated privileges (CAP_NET_RAW capability or root).
Most scan types (SYN, UDP, FIN, NULL, Xmas, Idle) need direct packet manipulation.

**Step-by-Step Fix:**

**Option 1: Run with sudo** (quick but less secure)
```bash
sudo prtip -sS -p 80,443 192.168.1.10
```

**Option 2: Grant CAP_NET_RAW capability** (recommended)
```bash
# One-time setup
sudo setcap cap_net_raw+ep $(which prtip)

# Now works without sudo
prtip -sS -p 80,443 192.168.1.10
```

**Option 3: Use TCP Connect scan** (no privileges needed)
```bash
# Slower but works without root
prtip -sT -p 80,443 192.168.1.10
```

**Prevention Strategy:**
1. Set up CAP_NET_RAW after installation (one-time)
2. Document privilege requirements in team runbooks
3. Use TCP Connect (-sT) for non-privileged environments
4. Create wrapper scripts with sudo for automation

---

### Pitfall 2: Misinterpreting Filtered Ports

**Symptom:**
- All ports show as "filtered" instead of open/closed
- Scan takes very long (timeout for each port)
- Expected open ports not detected

**Example Output:**
```
PORT    STATE
80/tcp  filtered
443/tcp filtered
22/tcp  filtered
```

**Root Cause:**
Firewall is dropping packets (no response). Scanner waits for timeout to determine state.
Filtered ≠ Closed. Filtered = Firewall blocking, Closed = No service listening.

**Understanding Port States:**
- **open:** Service is listening and accepts connections
- **closed:** Port is reachable but no service (target sends RST)
- **filtered:** Firewall drops packets (no response, timeout)
- **open|filtered:** Likely open but firewall may be filtering (UDP scans)

**Step-by-Step Fix:**

**Fix 1: Try different scan type**
```bash
# If SYN scan filtered, try ACK scan to map firewall
sudo prtip -sA -p 80,443 192.168.1.10

# ACK scan shows unfiltered vs filtered
```

**Fix 2: Use evasion techniques**
```bash
# Fragmentation
sudo prtip -sS -f -p 80,443 192.168.1.10

# Source port spoofing (port 53 often allowed)
sudo prtip -sS -g 53 -p 80,443 192.168.1.10

# Decoy scanning
sudo prtip -sS -D RND:5 -p 80,443 192.168.1.10
```

**Fix 3: Reduce timeout (faster results)**
```bash
# Default timeout may be too long
sudo prtip -sS --max-rtt-timeout 2000 -p 80,443 192.168.1.10
```

**Prevention Strategy:**
1. Start with ACK scan (-sA) to map firewall rules
2. Combine multiple scan types for comprehensive results
3. Document expected firewall behavior for target networks
4. Use stealth techniques only when needed (ethical/legal)

---

### Pitfall 3: "Too Many Open Files" - ulimit Errors

**Symptom:**
- Error: "Too many open files" or "Cannot allocate memory"
- Scan stops midway through large target set
- Performance degrades with many concurrent connections

**Example Error:**
```
Error: Failed to open file descriptor
Caused by: Too many open files (os error 24)
Current limit: 1024
Recommended: 65536
```

**Root Cause:**
Operating system limits number of open file descriptors (includes sockets).
Large scans need thousands of simultaneous connections.

**Step-by-Step Fix:**

**Fix 1: Check current limit**
```bash
ulimit -n
# Output: 1024 (too low for large scans)
```

**Fix 2: Increase limit (temporary)**
```bash
# For current session only
ulimit -n 65536

# Verify
ulimit -n
# Output: 65536

# Now run scan
sudo prtip -sS -p- 192.168.1.0/24
```

**Fix 3: Increase limit (permanent)**
```bash
# Edit /etc/security/limits.conf
sudo nano /etc/security/limits.conf

# Add these lines:
* soft nofile 65536
* hard nofile 65536

# Logout and login again, then verify
ulimit -n
```

**Fix 4: Reduce batch size (workaround)**
```bash
# If can't increase ulimit, reduce parallel connections
sudo prtip -sS --batch-size 500 -p- 192.168.1.0/24
```

**Prevention Strategy:**
1. Set ulimit to 65536+ before large scans
2. Add ulimit check to scan automation scripts
3. Monitor resource usage during scans
4. Use `--batch-size` to control concurrency

---

### Pitfall 4: Confusing Rate Limiter with Target Throttling

**Symptom:**
- Scan slower than expected despite high --max-rate
- "Backed off" messages for all targets
- Throughput much lower than configured

**Example Output:**
```
[2025-11-07 10:30:00] Warning: All targets backed off due to rate limiting
Configured rate: 100,000 pps
Actual throughput: 5,000 pps (5% of configured)
```

**Root Cause:**
ProRT-IP has TWO rate limiting mechanisms:
1. **Local rate limiter:** Limits outgoing packets (--max-rate, -T0-T5)
2. **Adaptive ICMP monitoring:** Backs off if target's network is overwhelmed (--adaptive-rate)

Confusion: Target throttling ≠ Your rate limit setting.

**Understanding the Difference:**
- **--max-rate 100000:** "Don't exceed 100K pps outgoing"
- **Adaptive backing off:** "Target's firewall/router is rate limiting, slow down"

**Step-by-Step Fix:**

**Fix 1: Check if adaptive rate is enabled**
```bash
# Disable adaptive ICMP monitoring if not needed
sudo prtip -sS --max-rate 50000 --no-adaptive-rate -p- 192.168.1.0/24
```

**Fix 2: Reduce target rate (not max-rate)**
```bash
# If target genuinely can't handle load, reduce overall rate
sudo prtip -sS -T2 -p- 192.168.1.0/24
# T2 = Polite timing (slower but target-friendly)
```

**Fix 3: Check ICMP unreachable messages**
```bash
# Run with verbose logging
RUST_LOG=debug sudo -E prtip -sS --max-rate 50000 -p 80,443 192.168.1.10

# Look for: "Received ICMP Type 3 Code 13 (Admin prohibited)"
```

**Fix 4: Increase minimum hostgroup**
```bash
# If scanning many hosts, increase parallel targets
sudo prtip -sS --max-rate 50000 --min-hostgroup 64 -p- 192.168.1.0/24
```

**Prevention Strategy:**
1. Understand difference between local rate limit and target throttling
2. Start with conservative rates (T2-T3) for new networks
3. Disable adaptive rate if scanning controlled networks
4. Monitor ICMP messages for actual rate limit indicators
5. See [26-RATE-LIMITING-GUIDE.md](26-RATE-LIMITING-GUIDE.md) for deep-dive

> **See Also:**
> - [User Guide - Troubleshooting](32-USER-GUIDE.md#troubleshooting) - Common errors and fixes
> - [Security Guide](08-SECURITY.md) - Privilege handling best practices
> - [Performance Tuning](31-BENCHMARKING-GUIDE.md#performance-optimization-tips) - Resource limits
> - [Rate Limiting Guide](26-RATE-LIMITING-GUIDE.md) - Comprehensive rate limiting reference

---

**END OF TUTORIALS**

**Version:** 2.0.0 (Sprint 5.5.1 Task 4 - Enhanced)
**Last Updated:** 2025-11-07
**Total Lines:** 2,074 lines (+1,314 from v1.0.0, 173% growth)
**Enhancements:**
- ✅ 4 new exercises (6-9) with complete solutions
- ✅ Common Pitfalls section (4 pitfalls documented)
- ✅ 6 cross-reference "See Also" boxes for navigation
- ✅ Complete beginner→intermediate→advanced learning path
**Status:** ✅ COMPLETE (exceeds 1,200-1,500 target)
