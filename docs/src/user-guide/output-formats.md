# Output Formats

Learn how to save and process ProRT-IP scan results in multiple formats.

## Overview

ProRT-IP supports 5 output formats designed for different use cases:

| Format | Extension | Best For | Parseable |
|--------|-----------|----------|-----------|
| **Text** | `.txt` | Human reading, terminal output | Manual review |
| **JSON** | `.json` | APIs, modern tooling, scripting | ✅ Very easy (jq) |
| **XML** | `.xml` | Nmap compatibility, legacy tools | ✅ Moderate (xmllint) |
| **Greppable** | `.gnmap` | Shell scripting, grep/awk | ✅ Easy (line-based) |
| **PCAPNG** | `.pcapng` | Packet analysis, Wireshark | ✅ Specialized tools |

---

## Text Format (Default)

**Purpose:** Human-readable terminal output with colorization

**Command:**
```bash
prtip -p 80,443 192.168.1.1
# Output directly to terminal (default)
```

**Save to File:**
```bash
prtip -p 80,443 192.168.1.1 -oN scan_results.txt
```

**Example Output:**
```
ProRT-IP v0.5.2 - Network Scanner
Starting scan at 2025-11-15 10:30:15

Scanning 192.168.1.1...
PORT     STATE  SERVICE
80/tcp   open   http
443/tcp  open   https

Scan complete: 2 ports scanned in 0.15 seconds
1 host up, 2 ports open
```

**Features:**
- Color-coded output (terminal only)
- Human-readable formatting
- Progress indicators
- Summary statistics

**Use Cases:**
- Interactive terminal sessions
- Quick manual review
- Sharing with non-technical users
- Documentation screenshots

---

## JSON Format

**Purpose:** Structured data for APIs, modern tooling, and scripting

**Command:**
```bash
prtip -p 80,443 192.168.1.1 -oJ scan_results.json
```

**Example Output:**
```json
{
  "scan_metadata": {
    "scanner": "ProRT-IP",
    "version": "0.5.2",
    "start_time": "2025-11-15T10:30:15Z",
    "end_time": "2025-11-15T10:30:16Z",
    "duration_seconds": 0.15,
    "command_line": "prtip -p 80,443 192.168.1.1 -oJ scan_results.json"
  },
  "hosts": [
    {
      "address": "192.168.1.1",
      "state": "up",
      "latency_ms": 2.3,
      "ports": [
        {
          "port": 80,
          "protocol": "tcp",
          "state": "open",
          "service": "http",
          "version": null
        },
        {
          "port": 443,
          "protocol": "tcp",
          "state": "open",
          "service": "https",
          "version": null
        }
      ]
    }
  ],
  "summary": {
    "total_hosts_scanned": 1,
    "hosts_up": 1,
    "total_ports_scanned": 2,
    "ports_open": 2,
    "ports_closed": 0,
    "ports_filtered": 0
  }
}
```

**Parsing with jq:**

```bash
# Extract all IP addresses with open ports
cat results.json | jq '.hosts[] | select(.state == "up") | .address'
# Output: "192.168.1.1"

# List all open ports
cat results.json | jq '.hosts[].ports[] | select(.state == "open") | .port'
# Output: 80, 443

# Get scan duration
cat results.json | jq '.scan_metadata.duration_seconds'
# Output: 0.15

# Complex query: hosts with SSH (port 22) open
cat results.json | jq '.hosts[] | select(.ports[] | select(.port == 22 and .state == "open")) | .address'

# Export to CSV
cat results.json | jq -r '.hosts[].ports[] | [.port, .protocol, .state, .service] | @csv'
```

**Use Cases:**
- API integrations
- CI/CD pipelines
- Automated analysis
- Database imports
- Modern scripting (Python, Node.js)

**Advantages:**
- Easy to parse (jq, Python json module)
- Structured data (no regex needed)
- Rich metadata
- Widely supported

**See Also:**
- [jq manual](https://stedolan.github.io/jq/manual/)
- [jq playground](https://jqplay.org/)

---

## XML Format (Nmap-Compatible)

**Purpose:** Nmap compatibility and legacy tool integration

**Command:**
```bash
prtip -p 80,443 192.168.1.1 -oX scan_results.xml
```

**Example Output:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<nmaprun scanner="ProRT-IP" version="0.5.2" start="1700048415" startstr="2025-11-15 10:30:15">
  <scaninfo type="syn" protocol="tcp" numservices="2" services="80,443"/>
  <host starttime="1700048415" endtime="1700048416">
    <status state="up" reason="syn-ack"/>
    <address addr="192.168.1.1" addrtype="ipv4"/>
    <ports>
      <port protocol="tcp" portid="80">
        <state state="open" reason="syn-ack"/>
        <service name="http" method="table" conf="3"/>
      </port>
      <port protocol="tcp" portid="443">
        <state state="open" reason="syn-ack"/>
        <service name="https" method="table" conf="3"/>
      </port>
    </ports>
    <times srtt="2300" rttvar="500"/>
  </host>
  <runstats>
    <finished time="1700048416" timestr="2025-11-15 10:30:16" elapsed="0.15"/>
    <hosts up="1" down="0" total="1"/>
  </runstats>
</nmaprun>
```

**Parsing with xmllint:**

```bash
# Extract host IPs
xmllint --xpath '//host/address/@addr' scan_results.xml

# Extract open ports
xmllint --xpath '//port[@protocol="tcp"]/state[@state="open"]/../@portid' scan_results.xml

# Get service names
xmllint --xpath '//port/service/@name' scan_results.xml
```

**Nmap Compatibility:**

ProRT-IP's XML format is **fully compatible** with Nmap XML tools:

```bash
# Use with Nmap scripts
prtip -sS -p 1-1000 192.168.1.1 -oX results.xml
nmap --script vuln results.xml  # Analyze with Nmap scripts

# Convert to HTML report
xsltproc scan_results.xml > report.html

# Import into Metasploit
db_import scan_results.xml
```

**Use Cases:**
- Nmap tool integration
- XSLT transformations
- Legacy system compatibility
- Metasploit/Burp Suite imports

---

## Greppable Format

**Purpose:** Shell scripting and line-based processing

**Command:**
```bash
prtip -p 1-1000 192.168.1.0/24 -oG results.gnmap
```

**Example Output:**
```
# ProRT-IP 0.5.2 scan initiated 2025-11-15 10:30:15
Host: 192.168.1.1 ()	Status: Up
Host: 192.168.1.1 ()	Ports: 22/open/tcp//ssh///, 80/open/tcp//http///, 443/open/tcp//https///
Host: 192.168.1.5 ()	Status: Up
Host: 192.168.1.5 ()	Ports: 3306/open/tcp//mysql///, 8080/closed/tcp//http-proxy///
# ProRT-IP done at 2025-11-15 10:30:16 -- 256 IP addresses (2 hosts up) scanned in 0.85 seconds
```

**Format Specification:**

Each line follows this structure:
```
Host: <IP> (<hostname>)  Ports: <port>/<state>/<protocol>//<service>///[, ...]
```

**Parsing with grep:**

```bash
# Find all hosts with port 22 (SSH) open
grep "22/open" results.gnmap
# Output: Host: 192.168.1.1 ()	Ports: 22/open/tcp//ssh///...

# Count hosts with SSH
grep -c "22/open" results.gnmap
# Output: 1

# Extract only IP addresses with SSH
grep "22/open" results.gnmap | awk '{print $2}'
# Output: 192.168.1.1

# Find hosts with MySQL (port 3306)
grep "3306/open" results.gnmap | awk '{print $2}'

# List all unique open ports
grep "Ports:" results.gnmap | grep -oP '\d+/open' | cut -d'/' -f1 | sort -n | uniq
```

**Parsing with awk:**

```bash
# Extract IP and open ports
awk '/Ports:/ {
  ip=$2;
  for(i=4; i<=NF; i++) {
    if($i ~ /open/) {
      split($i, port, "/");
      print ip, port[1], port[5];
    }
  }
}' results.gnmap

# Count open ports per host
awk '/Ports:/ {
  ip=$2;
  count=0;
  for(i=4; i<=NF; i++) {
    if($i ~ /open/) count++;
  }
  print ip, count;
}' results.gnmap
```

**Use Cases:**
- Shell scripting (bash, zsh)
- Quick grep/awk processing
- Legacy Nmap workflows
- Log analysis
- Simple automation

**Advantages:**
- One line per host (easy to grep)
- Fast processing (no parsing libraries needed)
- Portable (works on any Unix system)
- Compact format

---

## PCAPNG Format (Packet Capture)

**Purpose:** Detailed packet analysis and forensics

**Command:**
```bash
sudo prtip -sS -p 80,443 192.168.1.1 --pcap capture.pcapng
```

**Example Output:**

PCAPNG files contain raw packet data captured during the scan. Open with Wireshark:

```bash
# View with Wireshark
wireshark capture.pcapng

# Command-line analysis with tshark
tshark -r capture.pcapng

# Filter SYN packets
tshark -r capture.pcapng -Y "tcp.flags.syn == 1"

# Extract HTTP requests
tshark -r capture.pcapng -Y "http.request"

# Statistics
capinfos capture.pcapng
```

**Wireshark Filters:**

```
# SYN-ACK responses (open ports)
tcp.flags == 0x012

# RST responses (closed ports)
tcp.flags.reset == 1

# ICMP unreachable (filtered)
icmp.type == 3 && icmp.code == 13

# SSL/TLS handshakes
ssl.handshake
```

**Use Cases:**
- Deep packet inspection
- Protocol analysis
- Troubleshooting scan issues
- IDS/IPS signature development
- Security research
- Forensic investigation

**Advantages:**
- Complete packet capture
- Protocol-level analysis
- Timestamp precision (μs)
- Wireshark compatibility

**Limitations:**
- Large file sizes (1GB+ for extensive scans)
- Requires packet capture privileges (root)
- Not suitable for automation (binary format)

---

## Multiple Output Formats

**Save All Formats Simultaneously:**

```bash
sudo prtip -sS -p 80,443 192.168.1.1 -oA scan_results
```

**Creates:**
- `scan_results.txt` (normal text)
- `scan_results.json` (JSON)
- `scan_results.xml` (Nmap XML)
- `scan_results.gnmap` (greppable)

**Use Case:**
- Archive complete scan results
- Support multiple analysis workflows
- Share with different teams (JSON for devs, text for management)

---

## Output Processing Examples

### Example 1: Extract Web Servers (JSON)

**Goal:** Find all hosts with HTTP/HTTPS services

```bash
# Scan network
sudo prtip -sS -p 80,443,8080,8443 192.168.1.0/24 -oJ scan.json

# Parse JSON: extract IPs with any web port open
cat scan.json | jq -r '
  .hosts[] |
  select(.ports[] | select(
    .port == 80 or .port == 443 or .port == 8080 or .port == 8443
  ) | .state == "open") |
  .address
'
# Output: 192.168.1.1, 192.168.1.10, 192.168.1.20
```

### Example 2: Generate CSV Report (JSON)

**Goal:** Create spreadsheet-friendly CSV

```bash
# Extract: IP, Port, State, Service
cat scan.json | jq -r '
  .hosts[] as $host |
  $host.ports[] |
  [$host.address, .port, .state, .service] |
  @csv
' > report.csv

# Import to spreadsheet (Excel, LibreOffice)
```

### Example 3: Count Open Ports by Service (Greppable)

**Goal:** Inventory services across network

```bash
# Scan
sudo prtip -sS -p 1-1000 192.168.1.0/24 -oG scan.gnmap

# Count by service
grep "Ports:" scan.gnmap | \
  grep -oP '\d+/open/\w+//\w+' | \
  cut -d'/' -f5 | \
  sort | uniq -c | sort -nr

# Output:
# 45 http
# 23 ssh
# 12 https
# 8 mysql
# 3 smtp
```

### Example 4: Compare Two Scans (JSON)

**Goal:** Find new/closed ports between scans

```bash
# Baseline scan
sudo prtip -sS -p 1-1000 192.168.1.1 -oJ baseline.json

# Current scan
sudo prtip -sS -p 1-1000 192.168.1.1 -oJ current.json

# Find new open ports (jq)
diff \
  <(cat baseline.json | jq '.hosts[].ports[] | select(.state == "open") | .port' | sort) \
  <(cat current.json | jq '.hosts[].ports[] | select(.state == "open") | .port' | sort)

# Output: > 3306  (MySQL newly opened)
```

### Example 5: Filter by Port State (XML)

**Goal:** Extract only filtered ports (firewalled)

```bash
# Scan
sudo prtip -sA -p 1-1000 192.168.1.1 -oX scan.xml

# Extract filtered ports
xmllint --xpath '//port/state[@state="filtered"]/../@portid' scan.xml | \
  grep -oP '\d+' | \
  sort -n
```

### Example 6: Automated Vulnerability Check (JSON)

**Goal:** Alert on outdated service versions

```bash
# Scan with service detection
sudo prtip -sS -sV -p 22,80,443 192.168.1.1 -oJ scan.json

# Check for vulnerable versions (example: OpenSSH < 8.0)
cat scan.json | jq '
  .hosts[].ports[] |
  select(.service == "ssh" and .version != null) |
  select(.version | test("OpenSSH [0-7]\\.[0-9]")) |
  "⚠️ Vulnerable SSH: " + .version
'
```

---

## Format Selection Guide

### When to Use Each Format

**Text (-oN):**
- ✅ Interactive terminal sessions
- ✅ Quick manual review
- ✅ Non-technical stakeholders
- ❌ Automated processing

**JSON (-oJ):**
- ✅ API integrations
- ✅ Modern scripting (Python, Node.js)
- ✅ CI/CD pipelines
- ✅ Database imports
- ❌ Human reading (too verbose)

**XML (-oX):**
- ✅ Nmap tool compatibility
- ✅ Metasploit/Burp imports
- ✅ XSLT transformations
- ❌ Modern APIs (JSON preferred)

**Greppable (-oG):**
- ✅ Shell scripting
- ✅ Quick grep/awk analysis
- ✅ Legacy workflows
- ❌ Complex data structures

**PCAPNG (--pcap):**
- ✅ Protocol analysis
- ✅ Troubleshooting
- ✅ Security research
- ❌ General reporting (too low-level)

---

## Performance Considerations

### File Sizes

Approximate sizes for 1,000 ports scanned on 100 hosts:

| Format | Typical Size | Notes |
|--------|--------------|-------|
| Text | 50-100 KB | Human-readable, compact |
| JSON | 200-500 KB | Structured, verbose |
| XML | 300-600 KB | Most verbose |
| Greppable | 100-200 KB | One line per host |
| PCAPNG | 10-100 MB | Packet-level data |

### I/O Optimization

**Large Scans (10K+ hosts):**

```bash
# Stream to file (avoid memory buffering)
sudo prtip -sS -p 80,443 10.0.0.0/16 -oJ results.json &
tail -f results.json | jq .  # Monitor in real-time

# Compress output
sudo prtip -sS -p- 192.168.1.0/24 -oJ scan.json
gzip scan.json  # 60-80% size reduction
```

**Network Shares:**

```bash
# Avoid writing to network shares during scan (slow)
# Write locally, then copy
sudo prtip -sS -p 1-1000 192.168.1.0/24 -oA /tmp/scan
rsync -avz /tmp/scan.* server:/backup/
```

---

## Best Practices

### 1. Always Save Results

```bash
# Bad: No output saved
sudo prtip -sS -p 80,443 192.168.1.1

# Good: Save for later analysis
sudo prtip -sS -p 80,443 192.168.1.1 -oA scan-$(date +%Y%m%d)
```

### 2. Use Descriptive Filenames

```bash
# Include: date, target, purpose
sudo prtip -sS -p 1-1000 web-server.example.com \
  -oA webserver-audit-$(date +%Y%m%d-%H%M)
```

### 3. Combine Formats for Different Audiences

```bash
# JSON for automation, text for stakeholders
sudo prtip -sS -p 1-1000 192.168.1.0/24 -oA scan
cat scan.txt | mail -s "Scan Report" manager@example.com
python3 process_scan.py scan.json  # Automated analysis
```

### 4. Validate JSON Output

```bash
# Check JSON syntax after scan
cat scan.json | jq . > /dev/null
echo $?  # 0 = valid JSON, non-zero = error
```

### 5. Archive with Metadata

```bash
# Create scan archive
mkdir scan-archive-$(date +%Y%m%d)
sudo prtip -sS -p 1-1000 192.168.1.0/24 -oA scan-archive-$(date +%Y%m%d)/scan
echo "Scan: 192.168.1.0/24, Ports: 1-1000, Date: $(date)" > scan-archive-$(date +%Y%m%d)/README.txt
tar czf scan-archive-$(date +%Y%m%d).tar.gz scan-archive-$(date +%Y%m%d)/
```

---

## Troubleshooting

### Issue: JSON Parsing Errors

**Error:**
```
parse error: Expected separator between values at line 45
```

**Cause:** Incomplete JSON (scan interrupted)

**Solution:**
```bash
# Ensure scan completes
sudo prtip -sS -p 80,443 192.168.1.1 -oJ scan.json
echo $?  # Check exit code (0 = success)

# Validate JSON
cat scan.json | jq . > /dev/null
```

### Issue: Large Output Files

**Problem:** PCAPNG file 10GB+

**Solution:**
```bash
# Limit packet capture
sudo prtip -sS -p 80,443 192.168.1.0/24 --pcap scan.pcapng --snaplen 96
# snaplen=96: Capture only headers (no payload)

# Disable PCAPNG for large scans
sudo prtip -sS -p 80,443 192.168.1.0/24 -oJ scan.json
# Use JSON/XML/Greppable for large networks
```

### Issue: Permission Denied Writing Output

**Error:**
```
Error: Permission denied writing to /var/log/scan.json
```

**Solution:**
```bash
# Write to user-writable directory
sudo prtip -sS -p 80,443 192.168.1.1 -oJ ~/scans/scan.json

# Or use sudo for privileged paths
sudo prtip -sS -p 80,443 192.168.1.1 -oJ /var/log/scan.json
```

---

## Next Steps

- **[Advanced Usage](./advanced-usage.md)** - Complex scanning scenarios
- **[Examples Gallery](../getting-started/examples.md)** - 65 runnable examples
- **[Tutorials](../getting-started/tutorials.md)** - Interactive walkthroughs

**See Also:**
- [Output Formats (32-USER-GUIDE.md)](../../32-USER-GUIDE.md#35-output-formats)
- [Output Processing (32-USER-GUIDE.md)](../../32-USER-GUIDE.md#use-case-12-output-processing)
- [Batch Scanning](../../32-USER-GUIDE.md#use-case-11-batch-scanning)
