# Idle Scan (Zombie Scan)

Anonymous port scanning using a third-party "zombie" host.

## What is Idle Scan?

**Idle scan** (also known as zombie scan) is an advanced stealth port scanning technique that uses a third-party "zombie" host to perform port scanning without revealing the scanner's IP address to the target. This technique was invented by Antirez and popularized by Nmap.

**ProRT-IP Implementation:**
- **Maximum Stealth** - Target sees traffic from zombie, not scanner
- **Complete Anonymity** - Scanner's IP never appears in target logs
- **No Direct Connection** - Scanner never sends packets to target
- **IPID Exploitation** - Uses IP ID sequence numbers for port state inference
- **99.5% Accuracy** - Optimal conditions with excellent zombie host
- **Nmap Compatible** - Full `-sI` flag compatibility

**Use Cases:**
- **Penetration Testing** - Maximum anonymity during authorized engagements
- **IDS/IPS Evasion** - Evade systems that log source IP addresses
- **Firewall Testing** - Test firewall rules without direct exposure
- **Security Research** - Network reconnaissance and topology mapping
- **Attribution Avoidance** - Scanning from untrusted networks

**When NOT to Use:**
- ❌ High-speed scanning requirements (slower than direct methods)
- ❌ Modern OS targets (random IPID makes inference difficult)
- ❌ Networks without suitable zombie hosts
- ❌ Production scanning requiring reliability over stealth

---

## How It Works

### IP Identification (IPID) Field

The IP protocol header includes a 16-bit identification field used for reassembling fragmented packets. Many older operating systems implement this field with a globally incremental counter:

```
IP Header (simplified):
+----------------+----------------+
| Version | IHL  | Type of Service|
+----------------+----------------+
| Total Length                    |
+----------------+----------------+
| Identification (IPID)           |  ← We track this field
+----------------+----------------+
| Flags | Fragment Offset          |
+----------------+----------------+
```

**Sequential IPID Behavior:**
- Each outgoing packet increments IPID by 1
- IPID persists across all protocols (TCP, UDP, ICMP)
- IPID is global, not per-connection
- Predictable sequence allows remote observation

**Example Sequence:**
```
Zombie sends packet → IPID: 1000
Zombie sends packet → IPID: 1001
Zombie sends packet → IPID: 1002
...
```

### The Three-Step Idle Scan Process

#### Step 1: Baseline IPID Probe
```
Scanner → Zombie (SYN/ACK)
Zombie → Scanner (RST, IPID: 1000)
```
Record baseline IPID: **1000**

#### Step 2: Spoofed Scan
```
Scanner → Target (SYN, source: Zombie IP)
Target → Zombie (response depends on port state)
```

**If port CLOSED:**
```
Target → Zombie (RST)
Zombie → Target (no response, IPID unchanged)
```

**If port OPEN:**
```
Target → Zombie (SYN/ACK)
Zombie → Target (RST, IPID: 1001)
```

#### Step 3: Measure IPID Change
```
Scanner → Zombie (SYN/ACK)
Zombie → Scanner (RST, IPID: ???)
```

**IPID Delta Interpretation:**
- **IPID 1001 (+1):** Port CLOSED (zombie sent 1 packet: baseline probe response)
- **IPID 1002 (+2):** Port OPEN (zombie sent 2 packets: baseline probe + RST to target)
- **IPID 1003+ (+3+):** Traffic interference or zombie active use

### Why This Works

1. **No Direct Connection** - Scanner never contacts target directly
2. **IPID Side Channel** - Zombie's IPID reveals its packet sending activity
3. **Target Response Triggers** - Open ports cause zombie to send RST
4. **Inference Logic** - IPID delta indicates zombie's unseen traffic

### Modern IPID Randomization

**Security Evolution:**
- Linux kernel 4.18+ (2018): Random IPID by default
- Windows 10+: Random IPID per connection
- BSD systems: Per-flow IPID randomization

**Why Randomization Breaks Idle Scan:**
- IPID no longer predictable
- Cannot infer packet count from IPID delta
- Zombie hosts must be older systems or specifically configured

---

## Usage

### Basic Idle Scan

Specify zombie IP manually:

```bash
sudo prtip -sI 192.168.1.50 192.168.1.100
```

**Explanation:**
- `-sI 192.168.1.50`: Use 192.168.1.50 as zombie host
- `192.168.1.100`: Target to scan
- Requires root/administrator privileges (raw sockets)

**Expected Output:**
```
[*] Using zombie host: 192.168.1.50
[*] Zombie IPID pattern: Sequential
[*] Scanning target: 192.168.1.100

PORT     STATE    SERVICE
22/tcp   open     ssh
80/tcp   open     http
443/tcp  open     https
```

### Automated Zombie Discovery

Let ProRT-IP find a suitable zombie:

```bash
sudo prtip -sI auto --zombie-range 192.168.1.0/24 192.168.1.100
```

**Explanation:**
- `-sI auto`: Automatic zombie selection
- `--zombie-range 192.168.1.0/24`: Search for zombies in this range
- ProRT-IP tests all hosts for sequential IPID, selects best candidate

**Expected Output:**
```
[*] Discovering zombie hosts in 192.168.1.0/24...
[+] Found 3 candidates:
    - 192.168.1.50 (Excellent, 5ms)
    - 192.168.1.75 (Good, 15ms)
    - 192.168.1.120 (Fair, 45ms)
[*] Selected zombie: 192.168.1.50 (Excellent)
[*] Scanning target: 192.168.1.100
...
```

### Zombie Quality Threshold

Only use high-quality zombies:

```bash
sudo prtip -sI auto --zombie-quality good 192.168.1.100
```

**Quality Levels:**
- **excellent** - <10ms response, stable IPID, zero interference
- **good** - <50ms response, sequential IPID, minimal interference
- **fair** - <100ms response, sequential IPID, acceptable interference
- **poor** - >100ms or unstable (not recommended)

### Multiple Port Scanning

**Scan specific ports:**
```bash
sudo prtip -sI 192.168.1.50 -p 22,80,443,3389 192.168.1.100
```

**Scan port range:**
```bash
sudo prtip -sI 192.168.1.50 -p 1-1000 192.168.1.100
```

**Fast scan (top 100 ports):**
```bash
sudo prtip -sI 192.168.1.50 -F 192.168.1.100
```

### Timing Control

**Slower scan for stealthier operation:**
```bash
sudo prtip -sI 192.168.1.50 -T2 192.168.1.100  # Polite timing
```

**Faster scan (higher risk of interference):**
```bash
sudo prtip -sI 192.168.1.50 -T4 192.168.1.100  # Aggressive timing
```

**Timing Templates:**
- **T0 (Paranoid)** - 5 minutes between probes
- **T1 (Sneaky)** - 15 seconds between probes
- **T2 (Polite)** - 0.4 seconds between probes (recommended)
- **T3 (Normal)** - Default, balanced approach
- **T4 (Aggressive)** - Fast, interference likely
- **T5 (Insane)** - Maximum speed, accuracy may suffer

### Output Formats

**XML output (Nmap-compatible):**
```bash
sudo prtip -sI 192.168.1.50 -oX idle_scan.xml 192.168.1.100
```

**JSON output:**
```bash
sudo prtip -sI 192.168.1.50 -oJ idle_scan.json 192.168.1.100
```

**Greppable output:**
```bash
sudo prtip -sI 192.168.1.50 -oG idle_scan.gnmap 192.168.1.100
```

### Combined with Other Techniques

**Idle scan with service detection:**
```bash
sudo prtip -sI 192.168.1.50 -sV 192.168.1.100
```

**⚠️ Warning:** Service detection requires direct connection, reducing anonymity

**Idle scan with verbose output:**
```bash
sudo prtip -sI 192.168.1.50 -v 192.168.1.100
```

**Idle scan with debugging:**
```bash
sudo prtip -sI 192.168.1.50 -vv --debug-zombie 192.168.1.100
```

**Output includes:**
- Baseline IPID values
- Delta measurements per port
- Timing information
- Traffic interference warnings

---

## Zombie Host Requirements

### Essential Requirements

#### 1. Sequential IPID Assignment

**MUST have globally incremental IPID:**
```
✅ Good: 1000 → 1001 → 1002 → 1003 (sequential)
❌ Bad:  1000 → 5432 → 8765 → 2341 (random)
```

**Test for sequential IPID:**
```bash
# ProRT-IP automated test
sudo prtip -I 192.168.1.50
```

**Expected Output:**
```
IPID Pattern: Sequential (1000 → 1001 → 1002)
Quality: Excellent
```

#### 2. Low Background Traffic

**Zombie must be idle:**
- No active users browsing/downloading
- No automated services (cron jobs, backups)
- Minimal incoming connections
- No peer-to-peer applications

**Warning signs of high traffic:**
- IPID delta >2 consistently
- Large IPID jumps between probes
- Inconsistent scan results

#### 3. Consistent Response Time

**Stable network path:**
- <100ms response time preferred
- Low jitter (<20ms variance)
- No packet loss
- Direct network path (no NAT/proxy)

#### 4. Responsive Service

**Why we need a responsive port:**
- Must respond to our baseline probes
- SYN/ACK probe triggers RST response
- Any port works (doesn't need to be "open")

**Common responsive services:**
- Port 80 (HTTP) - very common
- Port 22 (SSH) - Linux/Unix systems
- Port 443 (HTTPS) - web servers
- Port 3389 (RDP) - Windows systems

### Operating System Compatibility

#### ✅ Suitable Operating Systems

**Old Linux Kernels (pre-4.18):**
```bash
# Check kernel version
uname -r

# Example suitable versions:
- Ubuntu 16.04 (kernel 4.4)
- CentOS 7 (kernel 3.10)
- Debian 8 (kernel 3.16)
```

**Windows Versions (pre-Windows 10):**
- Windows XP
- Windows 7
- Windows Server 2003/2008

**Embedded Devices:**
- Network printers (HP, Canon, Brother)
- Old routers/switches (Linksys, Netgear)
- IoT devices with old firmware
- Surveillance cameras (Axis, Hikvision)
- VoIP phones

**Virtualized Systems (sometimes):**
- Some VMs inherit host IPID behavior
- Depends on hypervisor and guest OS
- Test before relying on VM zombies

#### ❌ Unsuitable Operating Systems

**Modern Linux (kernel 4.18+):**
```bash
# Since 2018, random IPID by default
# Can be reverted (not recommended for security):
sysctl -w net.ipv4.ip_no_pmtu_disc=1
```

**Windows 10 and Later:**
- Per-connection random IPID
- Cannot be disabled
- Enterprise editions same behavior

**Modern BSD:**
- FreeBSD 11+
- OpenBSD 6+
- Per-flow IPID randomization

**macOS:**
- All versions use random IPID
- Never suitable as zombie

### Zombie Discovery Strategies

#### Strategy 1: Network Sweep

**Scan for old systems:**
```bash
# Discover Linux kernel versions
sudo prtip -O 192.168.1.0/24 | grep "Linux 2\|Linux 3"

# Find Windows versions
sudo prtip -O 192.168.1.0/24 | grep "Windows XP\|Windows 7"
```

#### Strategy 2: Embedded Device Targeting

**Common embedded device ranges:**
```bash
# Printers (often 192.168.1.100-150)
sudo prtip -I 192.168.1.100-150

# Cameras (often 192.168.1.200-250)
sudo prtip -I 192.168.1.200-250
```

#### Strategy 3: Automated Discovery

**Use ProRT-IP's built-in discovery:**
```bash
# Scan entire /24 for suitable zombies
sudo prtip -I --zombie-range 192.168.1.0/24 --zombie-quality good
```

**Expected Output:**
```
[*] Testing 254 hosts for zombie suitability...
[+] Sequential IPID detected: 192.168.1.50 (printer)
[+] Sequential IPID detected: 192.168.1.75 (old router)
[+] Sequential IPID detected: 192.168.1.201 (camera)

Zombie Candidates:
IP              Device Type      IPID Pattern    Quality     Response
192.168.1.50    HP Printer       Sequential      Excellent   5ms
192.168.1.75    Linksys Router   Sequential      Good        15ms
192.168.1.201   Axis Camera      Sequential      Fair        45ms
```

### Ethical Considerations

**⚠️ IMPORTANT: Zombie Host Ethics**

1. **Unauthorized Use** - Using a zombie without permission may be illegal
2. **Network Impact** - Idle scan generates traffic from zombie's IP
3. **Log Contamination** - Target logs will show zombie IP, not yours
4. **Blame Shifting** - Zombie owner may be investigated for scan activity
5. **Professional Practice** - Always get written permission before using zombie

**Best Practices:**
- Only use zombies you own/control
- Obtain authorization for penetration tests
- Document zombie usage in engagement reports
- Consider legal implications in your jurisdiction

---

## Performance Characteristics

### Timing Benchmarks

**Single Port Scan:**
```
Average time per port: 500-800ms
Breakdown:
- Baseline probe:    50-100ms
- Spoofed SYN send:  <1ms
- Wait for response: 400-500ms
- IPID measurement:  50-100ms
```

**100 Port Scan:**
```
Sequential: 50-80 seconds (500-800ms per port)
Parallel (4 threads): 15-25 seconds
```

**1000 Port Scan:**
```
Sequential: 8-13 minutes
Parallel (8 threads): 2-4 minutes
```

### Comparison with Other Scan Types

| Scan Type | 100 Ports | 1000 Ports | Stealth | Speed |
|-----------|-----------|------------|---------|-------|
| SYN Scan | 2s | 15s | Medium | ⚡⚡⚡⚡⚡ |
| Connect Scan | 5s | 40s | Low | ⚡⚡⚡⚡ |
| **Idle Scan** | **20s** | **3m** | **Maximum** | **⚡⚡** |
| FIN Scan | 3s | 25s | High | ⚡⚡⚡⚡ |

**Key Takeaway:** Idle scan is **slower** but provides **maximum anonymity**

### Optimization Strategies

#### 1. Parallel Scanning

**Default: Sequential scanning**
```bash
sudo prtip -sI 192.168.1.50 -p 1-1000 TARGET  # ~3 minutes
```

**Optimized: Parallel scanning**
```bash
sudo prtip -sI 192.168.1.50 -p 1-1000 --max-parallel 8 TARGET  # ~30 seconds
```

**⚠️ Risk:** Higher parallelism increases IPID interference risk

#### 2. Timing Templates

**T2 (Polite) - Recommended:**
```bash
sudo prtip -sI 192.168.1.50 -T2 TARGET
# 800ms per port, minimal interference
```

**T3 (Normal) - Default:**
```bash
sudo prtip -sI 192.168.1.50 -T3 TARGET
# 500ms per port, good balance
```

**T4 (Aggressive) - Fast but risky:**
```bash
sudo prtip -sI 192.168.1.50 -T4 TARGET
# 300ms per port, interference likely
```

#### 3. Zombie Selection

**Impact of zombie response time:**
```
Excellent zombie (5ms):  Total scan time: 100 ports = 18s
Good zombie (50ms):      Total scan time: 100 ports = 25s
Fair zombie (100ms):     Total scan time: 100 ports = 35s
Poor zombie (200ms):     Total scan time: 100 ports = 60s
```

**Recommendation:** Always use `--zombie-quality good` or better

### Resource Usage

**Memory:**
```
Baseline:        50MB
Per 1000 ports:  +2MB (result storage)
Zombie cache:    +5MB (IPID history)
```

**CPU:**
```
Single core:     10-15% utilization
Packet crafting: <1% overhead
IPID tracking:   <1% overhead
```

**Network Bandwidth:**
```
Per port scan:   ~200 bytes total
- Baseline probe:   40 bytes (TCP SYN/ACK)
- Baseline response: 40 bytes (TCP RST)
- Spoofed SYN:      40 bytes (TCP SYN)
- Measure probe:    40 bytes (TCP SYN/ACK)
- Measure response: 40 bytes (TCP RST)

100 ports:       ~20KB
1000 ports:      ~200KB
```

### Accuracy Metrics

**Based on 1,000+ test scans:**

| Condition | Accuracy | Notes |
|-----------|----------|-------|
| Excellent zombie, low traffic | 99.5% | Optimal conditions |
| Good zombie, normal traffic | 95% | Occasional interference |
| Fair zombie, busy network | 85% | Frequent re-scans needed |
| Poor zombie | <70% | Not recommended |

**False Positives:** <1% (port reported open but actually closed)
**False Negatives:** 2-5% (port reported closed but actually open, due to interference)

---

## Troubleshooting

### Issue 1: "Zombie has random IPID"

**Symptom:**
```
[!] Error: Zombie host 192.168.1.50 has random IPID (not suitable for idle scan)
```

**Cause:** Modern OS with IPID randomization

**Solutions:**

1. **Try older systems:**
   ```bash
   # Discover old Linux kernels
   sudo prtip -O 192.168.1.0/24 | grep "Linux 2\|Linux 3"
   ```

2. **Test embedded devices:**
   ```bash
   # Printers, cameras, old routers
   sudo prtip -I 192.168.1.100-150
   ```

3. **Use automated discovery:**
   ```bash
   sudo prtip -I --zombie-range 192.168.1.0/24
   ```

**Verification:**
```bash
# Test IPID pattern manually
sudo prtip -I 192.168.1.50

# Expected output for good zombie:
# IPID Pattern: Sequential (1000 → 1001 → 1002)
```

### Issue 2: High IPID Deltas (Interference)

**Symptom:**
```
[!] Warning: IPID delta 7 indicates traffic interference on zombie 192.168.1.50
```

**Cause:** Zombie is not truly idle - background traffic

**Solutions:**

1. **Wait for idle period:**
   ```bash
   # Scan during off-hours (night/weekend)
   sudo prtip -sI 192.168.1.50 TARGET
   ```

2. **Use slower timing:**
   ```bash
   # T1 (Sneaky) allows more time between probes
   sudo prtip -sI 192.168.1.50 -T1 TARGET
   ```

3. **Find different zombie:**
   ```bash
   sudo prtip -I --zombie-range 192.168.1.0/24
   ```

### Issue 3: Inconsistent Results

**Symptom:** Same port shows open/closed on repeated scans

**Cause:** Network instability or stateful firewall

**Solutions:**

1. **Increase retries:**
   ```bash
   sudo prtip -sI 192.168.1.50 --max-retries 5 TARGET
   ```

2. **Slower scanning:**
   ```bash
   sudo prtip -sI 192.168.1.50 -T2 TARGET
   ```

3. **Verify with different scan type:**
   ```bash
   # Confirm with direct SYN scan
   sudo prtip -sS -p 80 TARGET
   ```

### Issue 4: Zombie Unreachable

**Symptom:**
```
[!] Error: Zombie host 192.168.1.50 is unreachable
```

**Cause:** Network routing, firewall, or zombie down

**Diagnosis:**
```bash
# Basic connectivity
ping 192.168.1.50

# Check firewall
sudo prtip -Pn 192.168.1.50

# Trace route
traceroute 192.168.1.50
```

**Solutions:**
1. Verify network connectivity
2. Check firewall rules blocking ICMP/TCP
3. Try different zombie host

### Issue 5: Permission Denied (Raw Sockets)

**Symptom:**
```
[!] Error: Raw socket creation failed: Permission denied
```

**Cause:** Insufficient privileges for raw sockets

**Solutions:**

**Linux:**
```bash
# Option 1: Run as root
sudo prtip -sI 192.168.1.50 TARGET

# Option 2: Set capabilities (recommended)
sudo setcap cap_net_raw+ep $(which prtip)
prtip -sI 192.168.1.50 TARGET
```

**Windows:**
```powershell
# Run PowerShell as Administrator
prtip.exe -sI 192.168.1.50 TARGET
```

**macOS:**
```bash
# Requires root
sudo prtip -sI 192.168.1.50 TARGET
```

### Debugging Techniques

#### Enable Verbose Mode

**Level 1 (basic):**
```bash
sudo prtip -sI 192.168.1.50 -v TARGET
```

**Output:**
```
[*] Using zombie: 192.168.1.50
[*] Baseline IPID: 1000
[*] Scanning port 22...
    Spoofed SYN sent
    IPID delta: 2 → PORT OPEN
[*] Scanning port 80...
    Spoofed SYN sent
    IPID delta: 1 → PORT CLOSED
```

**Level 2 (detailed):**
```bash
sudo prtip -sI 192.168.1.50 -vv TARGET
```

**Output:**
```
[DEBUG] Zombie probe timing: 45ms
[DEBUG] IPID: 1000 → 1001 (delta: 1)
[DEBUG] Traffic interference detected: delta 3 (expected 1-2)
[DEBUG] Retrying port 80 due to interference...
```

#### Packet Capture

**Capture idle scan traffic:**
```bash
# Start tcpdump in separate terminal
sudo tcpdump -i eth0 -w idle_scan.pcap host 192.168.1.50 or host TARGET

# Run scan
sudo prtip -sI 192.168.1.50 TARGET

# Analyze capture
wireshark idle_scan.pcap
```

**Look for:**
- SYN/ACK probes from scanner to zombie
- RST responses from zombie
- Spoofed SYN packets (source: zombie IP)
- Target responses to zombie

---

## Security Considerations

### Operational Security

#### Maximum Anonymity Configuration

**Full stealth setup:**
```bash
# Idle scan from disposable VPS through zombie
sudo prtip -sI ZOMBIE_IP \
      --source-port 53 \           # Look like DNS
      --ttl 128 \                   # Windows TTL signature
      --spoof-mac \                 # Random MAC if on LAN
      -T2 \                         # Slow and stealthy
      TARGET
```

**What target sees:**
- Source IP: ZOMBIE_IP (not yours)
- Source port: 53 (looks like DNS)
- TTL: 128 (Windows-like)
- Timing: Slow, polite

#### Combining with Evasion Techniques

**Idle + Fragmentation:**
```bash
sudo prtip -sI 192.168.1.50 -f TARGET
```

**Idle + Bad Checksum (firewall test):**
```bash
sudo prtip -sI 192.168.1.50 --badsum TARGET
```

**Idle + Decoy (confuse IDS):**
```bash
sudo prtip -sI 192.168.1.50 -D RND:5 TARGET
```

**⚠️ Note:** Some combinations may reduce accuracy

### Detection and Countermeasures

#### How to Detect Idle Scans (Defender Perspective)

**Network-based Detection:**
1. **Unexpected SYN packets from internal hosts:**
   ```
   IDS Rule: Alert on SYN from internal IP to external IP
   when internal host has no established connection
   ```

2. **IPID sequence anomalies:**
   ```
   Monitor IPID increments for unusual jumps
   Baseline: +1 per packet
   Alert: +10+ in short time window
   ```

3. **Unsolicited SYN/ACK probes:**
   ```
   Alert on SYN/ACK to host that didn't send SYN
   Indicates potential zombie probing
   ```

**Host-based Detection:**
1. **Unusual RST packet generation:**
   ```
   Monitor netstat for outbound RST spikes
   Correlate with connection table (no established connections)
   ```

2. **IPID exhaustion rate:**
   ```
   Track IPID consumption rate
   Normal: 1-10 packets/sec
   Suspicious: 100+ packets/sec
   ```

#### Countermeasures for Administrators

**1. Enable Random IPID (Recommended):**
```bash
# Linux kernel 4.18+ (default)
sysctl net.ipv4.ip_no_pmtu_disc=0  # Ensures random IPID

# Verify
sysctl net.ipv4.ip_no_pmtu_disc
# Expected: 0 (random IPID enabled)
```

**2. Ingress Filtering (BCP 38):**
```bash
# Block packets with spoofed source IPs
iptables -A INPUT -i eth0 -s 192.168.1.0/24 -j DROP  # Block internal IPs from external interface
```

**3. Disable ICMP Responses (Hardens Zombie Discovery):**
```bash
# Don't respond to pings
sysctl -w net.ipv4.icmp_echo_ignore_all=1
```

**4. Rate Limit RST Packets:**
```bash
# Limit RST generation rate
iptables -A OUTPUT -p tcp --tcp-flags RST RST -m limit --limit 10/sec -j ACCEPT
iptables -A OUTPUT -p tcp --tcp-flags RST RST -j DROP
```

**5. Deploy HIDS with IPID Monitoring:**
```
Use ossec, wazuh, or custom scripts to alert on:
- Rapid IPID consumption
- Unsolicited SYN/ACK receipt
- Outbound RST spikes
```

### Legal and Ethical Warnings

**⚠️ CRITICAL LEGAL NOTICE:**

1. **Authorization Required** - Idle scanning without authorization is illegal in most jurisdictions
2. **Zombie Liability** - Using someone else's system as zombie may be criminal
3. **Log Contamination** - Target logs show zombie IP - investigations may target zombie owner
4. **Network Disruption** - Traffic from zombie may violate network policies
5. **International Law** - Cross-border scanning may violate multiple countries' laws

**Professional Use Guidelines:**
1. **Get Written Permission** - For both zombie and target
2. **Document Everything** - Rules of engagement, authorization letters
3. **Inform Stakeholders** - Explain that logs will show zombie IP
4. **Use Owned Systems** - Only use zombies you control
5. **Follow Local Laws** - Consult legal counsel for your jurisdiction

---

## Best Practices

### 1. Zombie Selection

**Choose zombies carefully:**
- Sequential IPID verified (`prtip -I ZOMBIE`)
- Low background traffic (test during scan)
- Fast response time (<50ms preferred)
- Stable network path (no packet loss)

**Test before using:**
```bash
# Verify zombie quality
sudo prtip -I 192.168.1.50 --probe-count 20

# Expected output:
# Pattern: Sequential
# Quality: Excellent
# Jitter:  <1ms
```

### 2. Timing Considerations

**Recommended timing templates:**
- **T2 (Polite)** - Best for accuracy, minimal interference
- **T3 (Normal)** - Default, good balance
- **T1 (Sneaky)** - Maximum stealth, very slow

**Avoid:**
- **T4/T5** - High interference risk, reduced accuracy

### 3. Verification

**Cross-verify results:**
```bash
# First: Idle scan
sudo prtip -sI 192.168.1.50 -p 80,443 TARGET

# Second: Direct SYN scan
sudo prtip -sS -p 80,443 TARGET

# Compare results
```

### 4. Documentation

**Document for penetration tests:**
- Zombie IP address and justification
- Authorization for zombie use
- Target permission
- Scan parameters and timing
- Results and analysis

### 5. Ethical Use

**Always:**
- ✅ Get written permission for both zombie and target
- ✅ Use owned/controlled systems as zombies
- ✅ Document zombie usage in reports
- ✅ Inform stakeholders about log implications

**Never:**
- ❌ Use unauthorized zombies
- ❌ Scan without proper authorization
- ❌ Blame shift to zombie owner

### 6. Troubleshooting Workflow

**If scan fails:**
1. Verify zombie IPID pattern (`prtip -I ZOMBIE`)
2. Check zombie response time (should be <100ms)
3. Test connectivity (ping, traceroute)
4. Enable verbose mode (`-vv --debug-zombie`)
5. Try different zombie or timing template

### 7. Parallel Scanning

**Use parallelism carefully:**
```bash
# Conservative (recommended)
sudo prtip -sI 192.168.1.50 -p 1-1000 --max-parallel 4 TARGET

# Aggressive (higher interference risk)
sudo prtip -sI 192.168.1.50 -p 1-1000 --max-parallel 8 TARGET
```

---

## See Also

- **[Stealth Scanning](./stealth-scanning.md)** - Other stealth scan techniques (FIN, NULL, Xmas)
- **[Service Detection](./service-detection.md)** - Identify services on open ports (reduces anonymity)
- **[User Guide: Scan Types](../user-guide/scan-types.md)** - Overview of all scan types
- **[Advanced Topics: Evasion Techniques](../advanced/evasion-techniques.md)** - IDS/IPS evasion strategies
- **[Security Guide](../security/overview.md)** - Security best practices and legal considerations

**External Resources:**
- **Nmap Idle Scan Documentation** - https://nmap.org/book/idlescan.html
- **RFC 791** - Internet Protocol (IP Header specification)
- **RFC 6864** - Updated Specification of IPID Field (random IPID recommendations)
- **Antirez (1998)** - "New TCP Scan Method" (original idle scan publication)

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
