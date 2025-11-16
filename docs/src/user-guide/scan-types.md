# Scan Types

ProRT-IP supports 8 scan types covering TCP, UDP, stealth scanning, and advanced anonymity techniques.

## Overview

| Flag | Scan Type | Description | Privilege | Speed | Use Case |
|------|-----------|-------------|-----------|-------|----------|
| `-sT` | TCP Connect | Full TCP handshake | User | Medium | No root access, 100% accuracy |
| `-sS` | TCP SYN | Half-open scan (stealth) | Root | Fast | Default, balanced stealth/speed |
| `-sU` | UDP | UDP port scan | Root | Slow | DNS, SNMP, NTP services |
| `-sF` | FIN | Stealth FIN scan | Root | Fast | Firewall evasion |
| `-sN` | NULL | Stealth NULL scan | Root | Fast | Firewall evasion |
| `-sX` | Xmas | Stealth Xmas scan | Root | Fast | Firewall evasion |
| `-sA` | ACK | Firewall detection | Root | Fast | Identify firewall rules |
| `-sI` | Idle/Zombie | Anonymous scan via zombie | Root | Very Slow | Maximum anonymity |

---

## TCP Connect Scan (`-sT`)

### How It Works

1. Completes full TCP three-way handshake
2. Establishes real connection
3. Immediately closes connection

**Diagram:**
```
Scanner                  Target
   |                        |
   |-------- SYN --------->|
   |<------- SYN-ACK ------|  (port open)
   |-------- ACK --------->|
   |-------- RST --------->|  (close connection)
```

### Advantages

- **No Privileges Required:** Works without root/administrator access
- **100% Accuracy:** Real connection test (vs 95% for SYN scan)
- **Universal Compatibility:** Works on all platforms
- **Reliable:** Not affected by packet filtering

### Disadvantages

- **Slower:** Full handshake overhead (vs half-open SYN)
- **Always Logged:** Target always logs connection attempts
- **Easier Detection:** Firewall/IDS easily identify connection patterns
- **More Overhead:** More packets exchanged per port

### Usage

```bash
# Basic TCP Connect scan (no root required)
prtip -sT -p 80,443 192.168.1.1

# Scan multiple ports
prtip -sT -p 1-1000 example.com

# From file without root
prtip -sT -p 22,80,443 -iL targets.txt
```

### When to Use

- You don't have root/administrator access
- Need 100% accurate results
- Testing application-layer availability
- Network policy prohibits raw packet manipulation
- Target firewall blocks SYN scans

**Expected Output:**
```
PORT    STATE  SERVICE
22/tcp  open   ssh
80/tcp  open   http
443/tcp open   https
3306/tcp closed mysql
```

---

## TCP SYN Scan (`-sS`)

### How It Works

1. Sends SYN packet (TCP handshake step 1)
2. Target responds with SYN-ACK if port is open
3. Scanner sends RST (doesn't complete handshake)

**Diagram:**
```
Scanner                  Target
   |                        |
   |-------- SYN --------->|
   |<------- SYN-ACK ------|  (port open, step 2)
   |-------- RST --------->|  (abort, don't complete)
   |                        |
```

### Advantages

- **Fast:** Half-open connection (no full handshake overhead)
- **Stealthy:** May not be logged by target (incomplete connection)
- **95% Accuracy:** Reliable for most scenarios
- **Default Choice:** Industry standard for network scanning
- **Lower Overhead:** Fewer packets than Connect scan

### Disadvantages

- **Requires Root:** Needs raw packet privileges
- **Some Firewalls Detect:** Modern IDS/IPS may identify SYN scans
- **Platform Issues:** Windows/Cisco firewalls may behave differently
- **Not 100% Accurate:** Some edge cases (stateful firewalls)

### Usage

```bash
# Basic SYN scan (requires root)
sudo prtip -sS -p 80,443 192.168.1.1

# Scan port range
sudo prtip -sS -p 1-1000 192.168.1.1

# Fast scan (top 100 ports)
sudo prtip -sS -F 192.168.1.1

# All ports
sudo prtip -sS -p- 192.168.1.1
```

### When to Use

- **Default choice** for 95% of scanning scenarios
- You have root/administrator access
- Need balance between speed and stealth
- Target doesn't have advanced IDS/IPS
- Large-scale network scanning

**Expected Output:**
```
PORT     STATE   SERVICE
22/tcp   open    ssh
80/tcp   open    http
443/tcp  open    https
3306/tcp closed  mysql
8080/tcp filtered http-proxy
```

---

## UDP Scan (`-sU`)

### How It Works

1. Sends UDP packet to target port
2. Waits for response or ICMP Port Unreachable
3. No response = `open|filtered` (uncertain)
4. Response = `open`
5. ICMP Port Unreachable = `closed`

**Diagram:**
```
Scanner                  Target
   |                        |
   |------- UDP Probe ---->|
   |                        |  (no response)
   |                        |
   (wait timeout)          |
   |                        |
Result: open|filtered      |
```

### Advantages

- **Discovers UDP Services:** Only way to find DNS, SNMP, NTP, etc.
- **Critical Services:** Many important services use UDP
- **Protocol Payloads:** ProRT-IP sends protocol-specific probes for accuracy

### Disadvantages

- **Very Slow:** 10-100x slower than TCP (ICMP rate limiting)
- **Less Accurate:** 80% vs 95% for TCP (many uncertain results)
- **Requires Root:** Raw packet privileges needed
- **Network Dependent:** Performance varies by network/firewall

### Usage

```bash
# Scan common UDP services
sudo prtip -sU -p 53,161,123 192.168.1.10

# Scan specific UDP ports
sudo prtip -sU -p 67,68,137,138,514 192.168.1.10

# Combined TCP + UDP scan
sudo prtip -sS -sU -p 1-100 192.168.1.10
```

### Common UDP Services

| Port | Service | Description |
|------|---------|-------------|
| 53 | DNS | Domain Name System |
| 67/68 | DHCP | Dynamic Host Configuration |
| 123 | NTP | Network Time Protocol |
| 137/138 | NetBIOS | Windows naming service |
| 161/162 | SNMP | Network management |
| 514 | Syslog | System logging |
| 1900 | UPnP | Universal Plug and Play |

### When to Use

- Need complete network inventory
- Scanning DNS, SNMP, or other UDP services
- Compliance requirements (must scan all protocols)
- Network troubleshooting (identify UDP services)

**Expected Output:**
```
PORT     STATE         SERVICE
53/udp   open          dns
161/udp  open          snmp
123/udp  open|filtered ntp
514/udp  open|filtered syslog
```

**Note:** UDP scans are slow. Port 53 scan may take 30-60 seconds vs 1-2 seconds for TCP.

---

## Stealth Scans (FIN, NULL, Xmas)

### Overview

Stealth scans exploit TCP RFC 793 to evade simple packet filters. They send unusual flag combinations:

| Scan Type | TCP Flags | Flag Bits |
|-----------|-----------|-----------|
| FIN (`-sF`) | FIN | 000001 |
| NULL (`-sN`) | None | 000000 |
| Xmas (`-sX`) | FIN, PSH, URG | 101001 |

**How They Work:**
- **Closed ports:** Should respond with RST
- **Open ports:** No response (RFC 793 behavior)
- **Filtered:** No response or ICMP unreachable

### FIN Scan (`-sF`)

Sends packets with only FIN flag set.

```bash
# FIN scan (evade simple firewalls)
sudo prtip -sF -p 80,443 192.168.1.10

# Combined with slow timing
sudo prtip -sF -T0 -p 80,443 192.168.1.10
```

**Expected Output:**
```
PORT    STATE         SERVICE
80/tcp  open|filtered http
443/tcp open|filtered https
22/tcp  closed        ssh
```

### NULL Scan (`-sN`)

Sends packets with no flags set (all zero).

```bash
# NULL scan
sudo prtip -sN -p 80,443 192.168.1.10
```

### Xmas Scan (`-sX`)

Sends packets with FIN, PSH, and URG flags set ("lit up like a Christmas tree").

```bash
# Xmas scan
sudo prtip -sX -p 80,443 192.168.1.10
```

### Advantages

- **Evade Simple Firewalls:** Some packet filters only check SYN flag
- **Stealthy:** Unusual traffic may bypass detection
- **RFC 793 Compliant:** Works against compliant TCP stacks

### Disadvantages

- **Unreliable on Windows:** Windows ignores these packets
- **Unreliable on Cisco:** Some Cisco devices don't follow RFC 793
- **Modern Firewalls Detect:** Stateful firewalls catch these easily
- **Less Accurate:** More `open|filtered` results (uncertain)

### When to Use

- **Not Recommended for Modern Networks:** Most firewalls now stateful
- Evading legacy firewall rules
- Penetration testing (demonstrate bypass)
- Academic/research purposes

**Note:** These scans are largely obsolete due to stateful firewalls. Use SYN scan for modern networks.

---

## ACK Scan (`-sA`)

### How It Works

Sends ACK packets (normally part of established connection). Used to map firewall rules, not discover open ports.

**Diagram:**
```
Scanner                  Target/Firewall
   |                        |
   |-------- ACK --------->|
   |<------- RST --------| (unfiltered)
   |                        |
   (no response = filtered)
```

### Usage

```bash
# Firewall rule mapping
sudo prtip -sA -p 80,443,22,25 192.168.1.10
```

**Expected Output:**
```
PORT   STATE
80/tcp unfiltered   # Firewall allows traffic
443/tcp unfiltered  # Firewall allows traffic
22/tcp filtered     # Firewall blocks SSH
25/tcp filtered     # Firewall blocks SMTP
```

### Interpretation

- **Unfiltered:** Port is accessible (firewall allows)
- **Filtered:** Port is blocked by firewall
- **Does NOT indicate open/closed:** Only shows firewall rules

### When to Use

- Mapping firewall rules
- Identifying which ports are filtered
- Understanding network security posture
- Compliance testing (verify firewall configuration)

**Use Case Example:**
```bash
# Test firewall allows web traffic
sudo prtip -sA -p 80,443 192.168.1.10

# If unfiltered, then test actual port state
sudo prtip -sS -p 80,443 192.168.1.10
```

---

## Idle/Zombie Scan (`-sI`)

### Overview

**Maximum anonymity scan:** Target never sees your IP address. Uses intermediary "zombie" host with predictable IP ID sequence.

### How It Works

1. **Find Zombie:** Discover host with incremental IP ID
2. **Baseline:** Check zombie's current IP ID
3. **Probe:** Spoof packet from zombie to target
4. **Check:** Measure zombie's IP ID increment
   - Increment +2: Port open (zombie received SYN-ACK from target)
   - Increment +1: Port closed (no response to zombie)

**Diagram:**
```
Your IP         Zombie Host         Target
   |                |                  |
   |-- Probe 1 ---->|                  |
   |<-- IPID 100 ---|                  |
   |                |                  |
   |-- Spoof ------>|                  |
   |                |-- SYN (spoofed)->|
   |                |<---- SYN-ACK ----|  (port open)
   |                |-- RST ---------->|
   |                |                  |
   |-- Probe 2 ---->|                  |
   |<-- IPID 102 ---|                  |
   |                |                  |
   (IPID +2 = port open)
```

### Usage

```bash
# Discover suitable zombie hosts
sudo prtip -sI RND 192.168.1.0/24

# Use specific zombie
sudo prtip -sI 192.168.1.5 -p 80,443 TARGET

# Idle scan with verbose output
sudo prtip -sI 192.168.1.5 -p 80,443 -v TARGET
```

### Finding Zombie Hosts

**Requirements:**
- Idle (low network traffic)
- Incremental IP ID sequence
- Not behind firewall that blocks spoofed packets

**Automatic Discovery:**
```bash
# Scan network for suitable zombies
sudo prtip -sI RND 192.168.1.0/24

# Output:
# [✓] Found zombie: 192.168.1.5 (idle, incremental IPID)
# [✓] Found zombie: 192.168.1.42 (idle, incremental IPID)
# [✗] Rejected: 192.168.1.10 (busy)
# [✗] Rejected: 192.168.1.15 (random IPID)
```

### Advantages

- **Maximum Anonymity:** Target never sees your IP
- **Bypass IP-based Filters:** Target logs zombie IP, not yours
- **Stealth:** No direct connection to target
- **Unique Technique:** Few scanners support this

### Disadvantages

- **Very Slow:** 500-800ms per port (vs 1-2ms for SYN)
- **Requires Suitable Zombie:** Not always available
- **Complex:** Requires understanding of IP ID behavior
- **99.5% Accuracy:** Slightly less accurate than SYN (rare edge cases)

### When to Use

- **Penetration Testing:** Demonstrate advanced stealth
- **Anonymity Required:** Hide your IP from target logs
- **Bypassing IP Filters:** Target blocks your IP
- **Research/Academic:** Study IP ID behavior

**Ethical Note:** Only use on authorized targets. Zombie host owner may be implicated.

**See Also:**
- [Idle Scan Guide](../features/idle-scan.md) - Complete technical reference
- [Tutorial: Advanced Stealth](../getting-started/tutorials.md#tutorial-5-stealth-scanning) - Hands-on walkthrough

---

## Port Scanning Techniques

### Common Ports (Fast)

**Goal:** Quickly identify common services

```bash
sudo prtip -sS -F 192.168.1.10
```

**Explanation:**
- `-F`: Fast scan (top 100 ports)
- Completes in 2-5 seconds
- Covers 90% of real-world services

**When to Use:**
- Initial reconnaissance
- Quick network checks
- Time-constrained situations

**Expected Output:**
```
PORT    STATE  SERVICE
22/tcp  open   ssh
80/tcp  open   http
443/tcp open   https
3306/tcp open  mysql
```

---

### Full Port Scan

**Goal:** Comprehensive scan of all 65,535 ports

```bash
sudo prtip -sS -p- -T4 192.168.1.10 -oN fullscan.txt
```

**Explanation:**
- `-p-`: All ports (1-65535)
- `-T4`: Aggressive timing (faster)
- `-oN fullscan.txt`: Save results

**Duration:** 5-30 minutes depending on network and timing template

**When to Use:**
- Security audit (find all services)
- Non-standard port discovery
- Complete inventory required
- Compliance requirements

---

### Custom Port List

**Goal:** Scan specific ports of interest

```bash
sudo prtip -sS -p 80,443,8080,8443,3000,3306 192.168.1.10
```

**Explanation:**
- Web ports: 80, 443, 8080, 8443, 3000
- Database port: 3306 (MySQL)

**Port Selection by Category:**

**Web Services:**
```bash
prtip -sS -p 80,443,8080,8443,3000,8000 TARGET
```

**Databases:**
```bash
prtip -sS -p 3306,5432,1433,27017,6379,1521 TARGET
```

**Remote Access:**
```bash
prtip -sS -p 22,23,3389,5900,5901 TARGET
```

**Mail Services:**
```bash
prtip -sS -p 25,110,143,465,587,993,995 TARGET
```

**File Sharing:**
```bash
prtip -sS -p 21,22,445,139,2049 TARGET
```

---

## Stealth Scanning Techniques

### Slow Timing (T0)

**Goal:** Evade intrusion detection systems (IDS)

```bash
sudo prtip -sS -T0 -p 80,443,22 192.168.1.10
```

**Explanation:**
- `-T0`: Paranoid timing (5-minute delays between packets)
- Very slow but stealthy
- Avoids rate-based IDS triggers

**Duration:** Hours for small port ranges

### Fragmentation

**Goal:** Evade simple packet filters

```bash
sudo prtip -sS -f -p 80,443 192.168.1.10
```

**Explanation:**
- `-f`: Fragment packets into small pieces
- Some firewalls can't reassemble/inspect fragments
- Modern stateful firewalls defeat this

### Decoy Scanning

**Goal:** Hide your real IP among fake sources

```bash
sudo prtip -sS -D RND:10 -p 80,443 192.168.1.10
```

**Explanation:**
- `-D RND:10`: Use 10 random decoy IPs
- Target sees scan from multiple sources
- Your real IP hidden in noise

**Expected Output:**
```
Using decoys: 203.0.113.15, 198.51.100.42, ..., YOUR_IP, ...
Scanning 192.168.1.10...
PORT    STATE  SERVICE
80/tcp  open   http
443/tcp open   https
```

### Combined Evasion

**Maximum stealth** - combine multiple techniques:

```bash
sudo prtip -sS -T1 -f --ttl 64 -D RND:5 -p 80,443 192.168.1.10
```

**Explanation:**
- `-T1`: Sneaky timing
- `-f`: Fragmentation
- `--ttl 64`: Custom TTL (mimic different OS)
- `-D RND:5`: 5 random decoy IPs

**Use Case:** Maximum stealth for penetration testing

---

## Multiple Scan Types

**Goal:** Combine TCP SYN and UDP scanning

```bash
sudo prtip -sS -sU -p 1-100 192.168.1.10
```

**Explanation:**
- Scans TCP ports 1-100 with SYN scan
- Scans UDP ports 1-100 with UDP scan
- Comprehensive coverage (all protocols)

**Duration:** UDP is slow (10-100x slower than TCP)

**When to Use:**
- Complete network inventory
- Compliance requirements (scan all protocols)
- Identify both TCP and UDP services

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

### 2. Choose Appropriate Scan Type

| Scenario | Recommended Scan | Command |
|----------|------------------|---------|
| No root access | TCP Connect | `prtip -sT -p 80,443 TARGET` |
| Default/balanced | TCP SYN | `sudo prtip -sS -p 1-1000 TARGET` |
| UDP services | UDP | `sudo prtip -sU -p 53,161 TARGET` |
| Firewall testing | ACK | `sudo prtip -sA -p 80,443 TARGET` |
| Maximum anonymity | Idle | `sudo prtip -sI ZOMBIE -p 80 TARGET` |
| Legacy firewall bypass | Stealth (FIN/NULL/Xmas) | `sudo prtip -sF -p 80,443 TARGET` |

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

### Mistake 2: Using Stealth Scans on Modern Networks

**Wrong:**
```bash
sudo prtip -sF -p 80,443 192.168.1.1
# Modern stateful firewall detects this
```

**Correct:**
```bash
sudo prtip -sS -p 80,443 192.168.1.1
# Use SYN scan for modern networks
```

### Mistake 3: Not Accounting for UDP Slowness

**Wrong:**
```bash
sudo prtip -sU -p- 192.168.1.1
# This will take DAYS
```

**Correct:**
```bash
sudo prtip -sU -p 53,161,123,514 192.168.1.1
# Scan only essential UDP ports
```

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
- Common with UDP scans and stealth scans
- May need additional probing

**Example Analysis:**
```
PORT     STATE         SERVICE
22/tcp   open          ssh         # ✅ SSH running
80/tcp   open          http        # ✅ Web server
443/tcp  open          https       # ✅ HTTPS server
3306/tcp closed        mysql       # ❌ MySQL not running
8080/tcp filtered      http-proxy  # 🔒 Firewall blocking
9200/tcp open|filtered http        # ❓ Uncertain (needs investigation)
```

---

## Quick Reference

### Essential Commands

```bash
# Basic Scans
prtip -sT -p 80,443 TARGET          # TCP Connect (no root)
sudo prtip -sS -p 1-1000 TARGET     # SYN scan (stealth)
sudo prtip -sU -p 53,161 TARGET     # UDP scan

# Stealth
sudo prtip -sF -p 80,443 TARGET     # FIN scan
sudo prtip -sN -p 80,443 TARGET     # NULL scan
sudo prtip -sX -p 80,443 TARGET     # Xmas scan
sudo prtip -sA -p 80,443 TARGET     # ACK scan (firewall)
sudo prtip -sI ZOMBIE -p 80 TARGET  # Idle scan (anonymous)

# Port Ranges
sudo prtip -sS -F TARGET            # Fast (top 100)
sudo prtip -sS -p- TARGET           # All ports
sudo prtip -sS -p 1-1000 TARGET     # Custom range
```

### Common Port Reference

| Port Range | Description | Example Command |
|------------|-------------|-----------------|
| 1-1023 | Well-known ports | `prtip -p 1-1023 TARGET` |
| 1024-49151 | Registered ports | `prtip -p 1024-49151 TARGET` |
| 49152-65535 | Dynamic/private | `prtip -p 49152-65535 TARGET` |

---

## Next Steps

- **[Timing & Performance](./timing-performance.md)** - Optimize scan speed
- **[Service Detection](../features/service-detection.md)** - Identify versions
- **[Advanced Usage](./advanced-usage.md)** - IPv6, plugins, automation
- **[Examples Gallery](../getting-started/examples.md)** - Runnable examples

**See Also:**
- [Tutorial: Understanding Scan Types](../getting-started/tutorials.md#tutorial-2-scan-types)
- [Stealth Scanning Guide](../features/stealth-scanning.md)
- [Idle Scan Technical Reference](../features/idle-scan.md)
