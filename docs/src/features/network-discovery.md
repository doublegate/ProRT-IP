# Network Discovery

Quickly identify active hosts on networks before port scanning.

## What is Network Discovery?

**Network Discovery** (also called **Host Discovery** or **Ping Scanning**) identifies which hosts are alive on a network **without** performing full port scans. This is the critical first step in network reconnaissance—find active targets before investing time in comprehensive port scanning.

**ProRT-IP Discovery Capabilities:**
- **Multi-probe techniques**: ICMP Echo, TCP SYN/ACK, ARP (local networks)
- **Subnet-scale scanning**: Scan /8 to /32 networks efficiently
- **Speed**: 10-100x faster than port scanning
- **Stealth options**: Minimal footprint, various probe combinations
- **Output integration**: Save live hosts for follow-up scanning

**Why Network Discovery Matters:**
- **Time Savings**: Scan only active hosts (92% faster if 20/256 hosts are live)
- **Reduced Noise**: Fewer packets, less intrusion detection alerts
- **Resource Efficiency**: Focus port scanning on reachable targets
- **Network Inventory**: Catalog all devices on subnets
- **Change Detection**: Identify new/removed devices over time

**Common Scenarios:**
- Internal network audits (find all employee devices)
- Server inventory (identify active servers in data center)
- IoT device discovery (locate smart devices, printers, cameras)
- DHCP monitoring (detect unauthorized DHCP clients)
- Network health checks (verify expected hosts are reachable)

---

## How It Works

### Discovery Process

ProRT-IP uses multiple probe techniques to determine if a host is alive:

**1. ICMP Echo (Ping)**
```
Scanner                    Target
   |                          |
   |--- ICMP Echo Request --->|
   |                          |
   |<-- ICMP Echo Reply ------|
   |                          |
Result: Host is UP           |
```

**2. TCP SYN Probe**
```
Scanner                    Target
   |                          |
   |--- TCP SYN (Port 80) --->|
   |                          |
   |<-- TCP SYN-ACK ----------|
   |                          |
Result: Host is UP           |
```

**3. ARP Scan (Local Networks)**
```
Scanner                    Target (Same Subnet)
   |                          |
   |--- ARP Who-has? -------->|
   |                          |
   |<-- ARP Reply ------------|
   |                          |
Result: Host is UP           |
```

### Default Behavior (`-sn` Flag)

When using `-sn` (ping scan only), ProRT-IP:

1. **Checks reachability** using ICMP/TCP/ARP probes
2. **Does NOT** scan any ports (no SYN/Connect scans)
3. **Reports** which hosts are up with response times
4. **Saves** live host list for follow-up scanning

**Automatic Technique Selection:**
- **Local networks** (same subnet): Prefer ARP (fastest, most reliable)
- **Internet targets**: Use ICMP Echo + TCP SYN probes
- **Firewalled networks**: TCP SYN to common ports (80, 443)

---

## Usage

### Basic Network Discovery

Discover active hosts on local subnet:

```bash
sudo prtip -sn 192.168.1.0/24
```

**Explanation:**
- `-sn`: Ping scan only (no port scan)
- `192.168.1.0/24`: Scan all IPs 192.168.1.1-254 (256 hosts)

**Expected Output:**
```
[✓] Starting network discovery of 192.168.1.0/24
[✓] Scanning 256 hosts

Host 192.168.1.1 is up (latency: 2.3ms)   # Router
Host 192.168.1.5 is up (latency: 1.8ms)   # Desktop
Host 192.168.1.10 is up (latency: 3.1ms)  # Laptop
Host 192.168.1.15 is up (latency: 4.2ms)  # Printer
Host 192.168.1.20 is up (latency: 2.9ms)  # NAS

[✓] Scan complete: 5 hosts up (256 scanned)
[✓] Duration: 8.2s
```

**Performance:** ~8 seconds for 256 hosts vs 5-30 minutes for full port scan

### Save Live Hosts for Follow-Up

Two-stage scanning workflow (recommended):

```bash
# Stage 1: Discover live hosts
sudo prtip -sn 192.168.1.0/24 -oN live-hosts.txt

# Stage 2: Review results
cat live-hosts.txt

# Stage 3: Port scan only live hosts
sudo prtip -sS -p 1-1000 -iL live-hosts.txt
```

**Time Savings:**
- **Scenario**: 20 out of 256 hosts are live
- **Traditional**: Scan all 256 hosts (30 minutes)
- **Two-stage**: Discover 256 hosts (8s) + scan 20 hosts (2.3 minutes) = **2.4 minutes total**
- **Savings**: 92% faster (30min → 2.4min)

### Large Subnet Discovery

Scan entire Class C network:

```bash
sudo prtip -sn 10.0.0.0/16 -oN corporate-network.txt
```

**Explanation:**
- `10.0.0.0/16`: 65,536 hosts (10.0.0.1 - 10.0.255.254)
- Completes in 8-12 minutes (vs 24-72 hours for full port scan)

**Output Filtering (Live Hosts Only):**
```bash
# Extract only live hosts from output
grep "is up" corporate-network.txt > live-only.txt

# Count live hosts
grep -c "is up" corporate-network.txt
```

### Specific Probe Techniques

**ICMP Echo Only:**
```bash
# Use only ICMP Echo requests
sudo prtip -PE -sn 192.168.1.0/24
```

**TCP SYN Probes to Common Ports:**
```bash
# Probe ports 80 and 443 to detect hosts
sudo prtip -PS80,443 -sn 192.168.1.0/24
```

**ARP Scan (Local Networks):**
```bash
# Use ARP for fast local network discovery
sudo prtip -PR -sn 192.168.1.0/24
```

**Note:** ProRT-IP automatically selects appropriate techniques if no specific probe is specified.

### Combined with Port Scanning

Discover and port scan in single command:

```bash
# Network discovery THEN port scan live hosts
sudo prtip -sS -p 1-1000 192.168.1.0/24
```

**How It Works:**
1. Implicit network discovery (identifies live hosts)
2. Port scan only on live hosts (automatic optimization)
3. Report both discovery and port scan results

**Compared to `-sn` (Discovery Only):**
- `-sn`: Discovery only, no port scan (8s for 256 hosts)
- `-sS -p 1-1000`: Discovery + port scan (2-10 min for 20 live hosts)

---

## Understanding Results

### Output Format

**Standard Output:**
```
Host 192.168.1.1 is up (latency: 2.3ms)
Host 192.168.1.5 is up (latency: 1.8ms)
Host 192.168.1.10 is up (latency: 3.1ms)

Scan complete: 3 hosts up (254 scanned)
```

**Fields:**
- **Host IP**: Target IP address
- **Status**: "is up" (reachable) or "is down" (unreachable)
- **Latency**: Round-trip time in milliseconds

### Host States

**up**
- Host responded to at least one probe
- Reachable for port scanning
- Action: Include in port scan targets

**down**
- Host did not respond to any probes
- May be powered off, firewalled, or non-existent
- Action: Skip port scanning

**Ambiguous Cases:**
- Some hosts drop ICMP but respond to TCP probes
- Firewalls may block discovery probes
- Use multiple probe techniques (`-PE -PS80,443`)

### Example Analysis

**Home Network Discovery:**
```bash
sudo prtip -sn 192.168.1.0/24
```

**Results:**
```
Host 192.168.1.1 is up (latency: 2.3ms)     # Router (low latency)
Host 192.168.1.5 is up (latency: 1.8ms)     # Desktop (very low latency)
Host 192.168.1.10 is up (latency: 45.2ms)   # Laptop (WiFi - higher latency)
Host 192.168.1.15 is up (latency: 4.2ms)    # Printer (wired)
Host 192.168.1.20 is up (latency: 2.9ms)    # NAS (wired)

Scan complete: 5 hosts up (254 scanned)
Duration: 8.2s
```

**Insights:**
- **Low latency (1-5ms)**: Wired connections (Ethernet)
- **Higher latency (30-60ms)**: Wireless connections (WiFi)
- **Consistent IPs**: Static IP assignments (router, NAS, printer)
- **Variable IPs**: DHCP clients (laptops, desktops)

**Action Items:**
1. Port scan .1 (router) to check web interface security
2. Investigate .10 (high latency) for WiFi interference
3. Verify .15 (printer) has no unnecessary services
4. Baseline .20 (NAS) for change detection

---

## Best Practices

### 1. Always Start with Discovery

Before comprehensive port scanning:

```bash
# STEP 1: Discover live hosts (8 seconds)
sudo prtip -sn 192.168.1.0/24 -oN live-hosts.txt

# STEP 2: Review live hosts
cat live-hosts.txt

# STEP 3: Port scan only live hosts (2-10 minutes)
sudo prtip -sS -p 1-1000 -iL live-hosts.txt
```

**Time Comparison:**

| Approach | Discovery Time | Port Scan Time | Total Time | Efficiency |
|----------|----------------|----------------|------------|------------|
| **Direct Port Scan** | 0s (implicit) | 30 min (256 hosts) | 30 min | Baseline |
| **Two-Stage** | 8s (explicit) | 2.4 min (20 hosts) | 2.5 min | **92% faster** |

### 2. Use Appropriate Probe Combinations

**Local Networks (Same Subnet):**
```bash
# ARP is fastest and most reliable
sudo prtip -PR -sn 192.168.1.0/24
```

**Internet Targets:**
```bash
# Combine ICMP and TCP probes
sudo prtip -PE -PS80,443 -sn TARGET
```

**Heavily Firewalled:**
```bash
# TCP SYN to common web ports
sudo prtip -PS80,443,8080,8443 -sn TARGET
```

### 3. Adjust Timing for Network Conditions

**Fast Local Networks:**
```bash
# Aggressive timing (T4)
sudo prtip -sn -T4 192.168.1.0/24
```

**Slow/Unreliable Networks:**
```bash
# Polite timing (T2)
sudo prtip -sn -T2 10.0.0.0/16
```

**Stealth (Avoid Detection):**
```bash
# Slow timing (T1)
sudo prtip -sn -T1 TARGET.com
```

### 4. Save Results for Change Detection

Baseline and monitor network changes:

```bash
# Initial baseline (Day 1)
sudo prtip -sn 192.168.1.0/24 -oN baseline-2025-01-15.txt

# Daily scans (ongoing)
sudo prtip -sn 192.168.1.0/24 -oN daily-2025-01-16.txt

# Compare for new/removed devices
diff baseline-2025-01-15.txt daily-2025-01-16.txt
```

**Use Cases:**
- Detect rogue devices on network
- Identify new IoT devices
- Monitor DHCP lease assignments
- Verify expected hosts are online

### 5. Combine Output Formats

Save results in multiple formats for analysis:

```bash
# All formats (text, JSON, XML, greppable)
sudo prtip -sn 192.168.1.0/24 -oA network-discovery

# Creates:
#   network-discovery.txt   (human-readable)
#   network-discovery.json  (API/scripts)
#   network-discovery.xml   (nmap-compatible)
#   network-discovery.gnmap (grep-friendly)
```

**JSON Analysis:**
```bash
# Extract live host IPs from JSON
cat network-discovery.json | jq '.hosts[] | select(.status=="up") | .address'
```

---

## Performance Characteristics

### Discovery Speed

| Network Size | Hosts | Discovery Time (T3) | Port Scan Time (1-1000) | Time Saved |
|--------------|-------|---------------------|-------------------------|------------|
| /30 | 4 | 1s | 12s | N/A (small) |
| /24 | 256 | 8s | 5-30 min | 92-96% |
| /20 | 4,096 | 2 min | 8-24 hours | 98-99% |
| /16 | 65,536 | 10 min | 5-30 days | 99%+ |

**Factors Affecting Speed:**
- **Probe count**: More probe types = slower but more reliable
- **Timing template**: T0-T5 (paranoid to insane)
- **Network latency**: Local (1-5ms) vs Internet (50-200ms)
- **Firewall filtering**: May require retries

### Probe Technique Comparison

| Technique | Speed | Reliability | Stealth | Works Through Firewall? |
|-----------|-------|-------------|---------|-------------------------|
| **ARP** | ⚡⚡⚡ Fastest | 99% (local) | Medium | ❌ Local only |
| **ICMP Echo** | ⚡⚡ Fast | 80% | Low | ❌ Often blocked |
| **TCP SYN (80)** | ⚡ Medium | 85% | Medium | ✅ Usually allowed |
| **TCP ACK** | ⚡ Medium | 75% | High | ✅ Firewall evasion |

**Recommendation:** Combine techniques for best results:
```bash
sudo prtip -PE -PS80,443 -PA80,443 -sn TARGET
```

### Overhead Analysis

**Discovery Overhead vs Direct Port Scanning:**

**Scenario:** 20 live hosts out of 256 in 192.168.1.0/24

**Option A: Direct Port Scan (No Discovery)**
```
Duration: 30 minutes (scan all 256 hosts × 1,000 ports)
Wasted: 236 hosts × 7 seconds = 27.5 minutes scanning dead hosts
```

**Option B: Two-Stage (Discovery + Port Scan)**
```
Discovery: 8 seconds (256 hosts)
Port Scan: 2.3 minutes (20 hosts × 1,000 ports)
Total: 2.4 minutes
Savings: 92% (30 min → 2.4 min)
```

**Overhead:** 8 seconds for 27.5 minutes savings = **ROI 20,600%**

---

## Troubleshooting

### Issue 1: No Hosts Detected (All Show as Down)

**Cause:** Firewall blocking discovery probes

**Solutions:**

```bash
# 1. Try TCP probes to common ports
sudo prtip -PS80,443,22 -sn 192.168.1.0/24

# 2. Use multiple probe types
sudo prtip -PE -PS80,443 -PA80,443 -sn 192.168.1.0/24

# 3. Test specific known-live host first
sudo prtip -sn 192.168.1.1  # Router
```

**Verify:**
```bash
# Manual ping test
ping -c 1 192.168.1.1

# If ping fails but you know host is up:
# → Firewall is blocking ICMP
# → Use TCP probes instead
```

### Issue 2: Discovery Too Slow

**Cause:** Conservative timing template (T0-T2)

**Solutions:**

```bash
# 1. Use aggressive timing (local networks)
sudo prtip -sn -T4 192.168.1.0/24

# 2. Limit probe types
sudo prtip -PR -sn 192.168.1.0/24  # ARP only (local)

# 3. Reduce parallelism for unstable networks
sudo prtip -sn -T2 --max-parallelism 10 192.168.1.0/24
```

**Benchmark:**
```bash
# Test timing difference
time sudo prtip -sn -T2 192.168.1.0/24
time sudo prtip -sn -T4 192.168.1.0/24
```

### Issue 3: Inconsistent Results (Hosts Up/Down Varies)

**Cause:** Network instability or rate limiting

**Solutions:**

```bash
# 1. Use slower timing with retries
sudo prtip -sn -T2 --max-retries 3 192.168.1.0/24

# 2. Run multiple scans and compare
sudo prtip -sn 192.168.1.0/24 -oN scan1.txt
sudo prtip -sn 192.168.1.0/24 -oN scan2.txt
diff scan1.txt scan2.txt

# 3. Increase timeout for slow hosts
sudo prtip -sn --host-timeout 5000ms 192.168.1.0/24
```

### Issue 4: ARP Scan Not Working

**Cause:** ARP only works on local subnet

**Symptoms:**
```bash
sudo prtip -PR -sn 10.0.0.0/24
# No hosts detected despite known live hosts
```

**Solutions:**

```bash
# 1. Verify you're on the same subnet
ip addr show | grep "inet 10.0.0"
# If not on 10.0.0.x, ARP won't work

# 2. Use ICMP/TCP instead for remote networks
sudo prtip -PE -PS80,443 -sn 10.0.0.0/24

# 3. For local networks, verify interface
sudo prtip -PR -sn -e eth0 192.168.1.0/24
```

**ARP Limitation:** Only works within same broadcast domain (same subnet, no routers in between)

---

## Security Considerations

### 1. Discovery Footprint

Network discovery generates network traffic that may trigger alerts:

**Logged Events:**
- Firewall logs: Multiple ICMP/TCP probes from single source
- IDS alerts: Network scanning patterns detected
- Router logs: ARP requests for entire subnet

**Minimize Footprint:**
```bash
# Slow timing (T0-T1)
sudo prtip -sn -T1 TARGET

# Limit probe types
sudo prtip -PS80 -sn TARGET  # Only probe port 80

# Spaced intervals (not implemented, manual workaround)
for subnet in 192.168.{1..10}.0/24; do
  sudo prtip -sn $subnet
  sleep 300  # 5 minutes between subnets
done
```

### 2. ARP Spoofing Detection

ARP scans can be confused with ARP spoofing attacks:

**Legitimate ARP Scan:**
```
ARP Request: Who has 192.168.1.1? Tell 192.168.1.100
ARP Request: Who has 192.168.1.2? Tell 192.168.1.100
ARP Request: Who has 192.168.1.3? Tell 192.168.1.100
...
```

**Mitigation:**
- Use `-T2` or slower timing
- Schedule scans during maintenance windows
- Notify network security team before large-scale discovery

### 3. Reconnaissance Phase Awareness

Network discovery is typically the first phase of reconnaissance:

**Attack Kill Chain:**
1. **Network Discovery** (this feature) ← We are here
2. Port scanning (identify services)
3. Service enumeration (detect versions)
4. Exploitation (attack vulnerabilities)

**Defense Perspective:**
- Network discovery alone is not an attack
- Combine with port scanning patterns for threat scoring
- Investigate persistent discovery from external IPs

**Ethical Use:**
- Only discover networks you own or have permission to audit
- Document authorized scopes in penetration testing engagements
- Use `--reason` flag to log scan justification (future feature)

### 4. Privacy Considerations

Discovery reveals network topology and device presence:

**Sensitive Information Disclosed:**
- IP address assignments
- Number of devices on network
- Device response times (wired vs WiFi)
- Network architecture (subnets, VLANs)

**Best Practices:**
- Limit discovery scopes to authorized networks
- Do not share discovery results externally
- Secure output files (contain network inventory)
- Use encrypted storage for long-term baselines

---

## Integration with Scanning Workflow

### Recommended Multi-Stage Workflow

**Stage 1: Network Discovery**
```bash
sudo prtip -sn 192.168.1.0/24 -oN 01-discovery.txt
```

**Stage 2: Fast Port Scan (Top 100 Ports)**
```bash
sudo prtip -F -iL 01-discovery.txt -oN 02-fast-scan.txt
```

**Stage 3: Service Detection (Open Ports Only)**
```bash
# Extract open ports from Stage 2
grep "open" 02-fast-scan.txt > open-ports.txt

# Service detection on open ports
sudo prtip -sV -iL open-ports.txt -oN 03-services.txt
```

**Stage 4: Deep Port Scan (Specific Hosts)**
```bash
# Scan all 65,535 ports on critical servers
sudo prtip -sS -p- 192.168.1.10 -oN 04-deep-scan.txt
```

**Time Comparison:**

| Approach | Total Time | Thoroughness |
|----------|------------|--------------|
| **Direct Full Scan** | 24-72 hours | 100% (all ports, all hosts) |
| **Multi-Stage** | 15-45 minutes | 95% (targeted deep scans) |
| **Savings** | 97-99% | Minimal loss |

### Automation Example

Automate daily network monitoring:

```bash
#!/bin/bash
# daily-discovery.sh

DATE=$(date +%Y-%m-%d)
SUBNET="192.168.1.0/24"
OUTPUT_DIR="/var/log/network-scans"

# Daily discovery
sudo prtip -sn $SUBNET -oN "$OUTPUT_DIR/discovery-$DATE.txt"

# Compare with previous day
PREV_DATE=$(date -d "yesterday" +%Y-%m-%d)
diff "$OUTPUT_DIR/discovery-$PREV_DATE.txt" "$OUTPUT_DIR/discovery-$DATE.txt" > "$OUTPUT_DIR/changes-$DATE.txt"

# Alert on new hosts
NEW_HOSTS=$(grep "^>" "$OUTPUT_DIR/changes-$DATE.txt" | wc -l)
if [ $NEW_HOSTS -gt 0 ]; then
  echo "Alert: $NEW_HOSTS new host(s) detected on $DATE" | mail -s "Network Discovery Alert" admin@example.com
fi
```

**Schedule with Cron:**
```bash
# Run daily at 2 AM
0 2 * * * /usr/local/bin/daily-discovery.sh
```

---

## See Also

- **[Scan Types](../user-guide/scan-types.md)** - Complete scan type reference including `-sn` flag
- **[Basic Usage](../user-guide/basic-usage.md)** - Network discovery usage examples
- **[Quick Start](../getting-started/quick-start.md)** - Getting started with network discovery
- **[Timing & Performance](../user-guide/timing-performance.md)** - Optimize discovery speed
- **[Output Formats](../user-guide/output-formats.md)** - Save and analyze discovery results
- **[Nmap Compatibility](./nmap-compatibility.md)** - Nmap `-sn` flag compatibility

**External Resources:**
- **Nmap Network Scanning**: Original network discovery techniques
- **ARP Protocol** (RFC 826): Address Resolution Protocol specification
- **ICMP Protocol** (RFC 792): Internet Control Message Protocol specification
- **Network Reconnaissance**: Best practices for network enumeration

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
