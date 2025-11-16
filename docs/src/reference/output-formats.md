# Output Formats

ProRT-IP supports four output formats for scan results: Text, JSON, XML, and Greppable. Each format is optimized for different use cases, from human readability to machine parsing.

## What are Output Formats?

**Output formats** control how ProRT-IP presents scan results. Different formats serve different purposes:

- **Text (-oN)**: Human-readable format with optional terminal colors
- **JSON (-oJ)**: Machine-parseable format for API integration and data processing
- **XML (-oX)**: Nmap-compatible format for tool interoperability
- **Greppable (-oG)**: One-line-per-host format optimized for grep/awk/sed

You can output to multiple formats simultaneously using the `-oA` flag.

---

## Text Format (-oN)

Human-readable format designed for terminal display and log files.

### Basic Usage

```bash
prtip -sS -p 80,443 192.168.1.0/24 -oN scan_results.txt
```

### Features

**Terminal Colors:**
- Open ports: Green + Bold
- Closed ports: Red
- Filtered ports: Yellow
- Unknown ports: White
- IP addresses: Bright Blue + Bold
- Numbers: Cyan + Bold

**Verbosity Levels:**
- **Level 0** (default): Only open ports shown
- **Level 1** (`-v`): Open + filtered ports shown
- **Level 2** (`-vv`): Open + filtered + closed ports shown

**Output Grouping:**
- Results grouped by host (alphabetically sorted)
- Only hosts with open ports shown (default)
- Each port shows: state, protocol, service, version (if detected)

### Example Output

```
Scan Results for 192.168.1.10:

PORT     STATE  SERVICE     VERSION
22/tcp   open   ssh         OpenSSH 8.9p1 Ubuntu 3ubuntu0.1
80/tcp   open   http        Apache httpd 2.4.52
443/tcp  open   https       Apache httpd 2.4.52
3306/tcp closed mysql

Response time: 12.3 ms
Banner: SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1

---

Scan Results for 192.168.1.20:

PORT     STATE  SERVICE
22/tcp   open   ssh
80/tcp   open   http
```

### Color Scheme

```rust
// Open ports: green + bold
PortState::Open => "open".green().bold()

// Closed ports: red
PortState::Closed => "closed".red()

// Filtered ports: yellow
PortState::Filtered => "filtered".yellow()

// IP addresses: bright blue + bold
format!("{}", ip).bright_blue().bold()

// Port numbers: cyan + bold
format!("{}", port).cyan().bold()
```

### Banner Display

- Banners truncated at 70 characters
- Full banner available in verbose mode
- Special characters displayed as-is

---

## JSON Format (-oJ)

Machine-parseable JSON format for API integration and automated processing.

### Basic Usage

```bash
prtip -sS -p 80,443 192.168.1.0/24 -oJ scan_results.json
```

### Schema Specification

#### Top-Level Structure

```json
{
  "scan_time": "2025-01-15T10:30:45.123456Z",
  "scan_type": "Connect",
  "timing_template": "Normal",
  "timeout_ms": 3000,
  "total_results": 256,
  "statistics": {
    "hosts_scanned": 256,
    "ports_open": 42,
    "ports_closed": 200,
    "ports_filtered": 14,
    "ports_unknown": 0
  },
  "results": [
    {
      "target_ip": "192.168.1.10",
      "port": 80,
      "state": "Open",
      "service": "http",
      "banner": "Apache/2.4.52 (Ubuntu)",
      "timestamp": "2025-01-15T10:30:45.234567Z",
      "response_time_ms": 12.3
    }
  ]
}
```

#### Field Descriptions

**Scan Metadata:**
- `scan_time`: ISO 8601 timestamp when scan started
- `scan_type`: Scan technique used (Connect, Syn, Udp, Fin, Null, Xmas, Ack, Idle)
- `timing_template`: Timing template (Paranoid, Sneaky, Polite, Normal, Aggressive, Insane)
- `timeout_ms`: Connection timeout in milliseconds
- `total_results`: Total number of port results

**Statistics:**
- `hosts_scanned`: Number of unique IP addresses scanned
- `ports_open`: Count of open ports
- `ports_closed`: Count of closed ports
- `ports_filtered`: Count of filtered ports
- `ports_unknown`: Count of ports in unknown state

**Result Fields:**
- `target_ip`: IPv4 or IPv6 address of target host
- `port`: Port number (1-65535)
- `state`: Port state (Open, Closed, Filtered, Unknown)
- `service`: Detected service name (optional, requires `-sV`)
- `banner`: Service banner text (optional, if retrieved)
- `timestamp`: ISO 8601 timestamp of port scan
- `response_time_ms`: Time to receive response in milliseconds (optional)

### Example Output

```json
{
  "scan_time": "2025-01-15T10:30:45.123456Z",
  "scan_type": "Syn",
  "timing_template": "Aggressive",
  "timeout_ms": 2000,
  "total_results": 5,
  "statistics": {
    "hosts_scanned": 2,
    "ports_open": 3,
    "ports_closed": 1,
    "ports_filtered": 1,
    "ports_unknown": 0
  },
  "results": [
    {
      "target_ip": "192.168.1.10",
      "port": 22,
      "state": "Open",
      "service": "ssh",
      "banner": "SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1",
      "timestamp": "2025-01-15T10:30:46.234567Z",
      "response_time_ms": 8.2
    },
    {
      "target_ip": "192.168.1.10",
      "port": 80,
      "state": "Open",
      "service": "http",
      "banner": "Apache/2.4.52 (Ubuntu)",
      "timestamp": "2025-01-15T10:30:46.345678Z",
      "response_time_ms": 12.5
    },
    {
      "target_ip": "192.168.1.10",
      "port": 3306,
      "state": "Closed",
      "timestamp": "2025-01-15T10:30:46.456789Z",
      "response_time_ms": 2.1
    },
    {
      "target_ip": "192.168.1.20",
      "port": 443,
      "state": "Open",
      "service": "https",
      "timestamp": "2025-01-15T10:30:47.567890Z",
      "response_time_ms": 15.8
    },
    {
      "target_ip": "192.168.1.20",
      "port": 8080,
      "state": "Filtered",
      "timestamp": "2025-01-15T10:30:50.678901Z"
    }
  ]
}
```

### Pretty Printing

JSON output is automatically formatted with 2-space indentation for readability:

```rust
let json = serde_json::to_string_pretty(&output)?;
```

---

## XML Format (-oX)

Nmap-compatible XML format for interoperability with Nmap-based tools.

### Basic Usage

```bash
prtip -sS -p 80,443 192.168.1.0/24 -oX scan_results.xml
```

### DTD Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE nmaprun>
<nmaprun scanner="prtip" version="0.5.2" start="1736938245">
  <scaninfo type="Syn" protocol="tcp" timeout="3000" />
  <verbose level="0" />

  <host>
    <address addr="192.168.1.10" addrtype="ipv4" />
    <status state="up" reason="syn-ack" />

    <ports>
      <port protocol="tcp" portid="80">
        <state state="open" reason="syn-ack" />
        <service name="http" />
      </port>
      <port protocol="tcp" portid="443">
        <state state="open" reason="syn-ack" />
        <service name="https" />
      </port>
    </ports>

    <times response="12" />
  </host>

  <runstats>
    <finished time="1736938250" elapsed="5.0" />
    <hosts total="256" up="42" />
  </runstats>
</nmaprun>
```

### Element Reference

**Root Element:**
```xml
<nmaprun scanner="prtip" version="0.5.2" start="unix_timestamp">
```

**Scan Information:**
```xml
<scaninfo type="Syn|Connect|Udp|Fin|Null|Xmas|Ack|Idle"
          protocol="tcp|udp"
          timeout="milliseconds" />
```

**Host Element:**
```xml
<host>
  <address addr="192.168.1.10" addrtype="ipv4|ipv6" />
  <status state="up" reason="syn-ack|reset|no-response|unknown" />
  <ports>...</ports>
  <times response="milliseconds" />
</host>
```

**Port Element:**
```xml
<port protocol="tcp|udp" portid="1-65535">
  <state state="open|closed|filtered|unknown" reason="syn-ack|reset|no-response" />
  <service name="http|ssh|https|..." />
  <script id="script_name" output="script_output" />
</port>
```

### Address Type Detection

ProRT-IP automatically detects IPv4 vs IPv6:

```rust
let addr_type = match host {
    IpAddr::V4(_) => "ipv4",
    IpAddr::V6(_) => "ipv6",
};
```

### Port State Reasons

```rust
let reason = match result.state {
    PortState::Open => "syn-ack",
    PortState::Closed => "reset",
    PortState::Filtered => "no-response",
    PortState::Unknown => "unknown",
};
```

### XML Special Character Escaping

Banners are automatically escaped for XML compatibility:

| Character | Escaped As |
|-----------|------------|
| `&` | `&amp;` |
| `<` | `&lt;` |
| `>` | `&gt;` |
| `"` | `&quot;` |
| `'` | `&apos;` |

---

## Greppable Format (-oG)

One-line-per-host format optimized for grep, awk, and sed parsing.

### Basic Usage

```bash
prtip -sS -p 80,443 192.168.1.0/24 -oG scan_results.gnmap
```

### Format Specification

#### Header

```
# Nmap-style greppable output (ProRT-IP v0.5.2)
# Scan started at: 2025-01-15 10:30:45 UTC
```

#### Host Lines

Each host appears on two lines:

```
Host: 192.168.1.10 ()	Status: Up
Ports: 80/open/tcp/http, 443/open/tcp/https, 22/closed/tcp/ssh
```

**Line 1 Format:**
```
Host: <ip_address> ()	Status: Up
```

**Line 2 Format:**
```
Ports: <port>/<state>/<protocol>/<service>, <port>/<state>/<protocol>/<service>, ...
```

**Port Entry Format:**
```
<port_number>/<state>/<protocol>/<service_name>
```

#### Footer

```
# Scan finished at: 2025-01-15 10:35:50 UTC
# Total hosts: 256, Ports scanned: 2
```

### Example Output

```
# Nmap-style greppable output (ProRT-IP v0.5.2)
# Scan started at: 2025-01-15 10:30:45 UTC
Host: 192.168.1.10 ()	Status: Up
Ports: 22/open/tcp/ssh, 80/open/tcp/http, 443/open/tcp/https, 3306/closed/tcp/mysql
Host: 192.168.1.20 ()	Status: Up
Ports: 80/open/tcp/http, 8080/filtered/tcp/http-proxy
Host: 192.168.1.30 ()	Status: Up
Ports: 22/open/tcp/ssh, 443/open/tcp/https
# Scan finished at: 2025-01-15 10:35:50 UTC
# Total hosts: 3, Ports scanned: 2
```

### Grep Examples

**Find all hosts with port 80 open:**
```bash
grep "80/open/tcp" scan_results.gnmap
```

**Find all hosts with SSH open:**
```bash
grep "22/open/tcp/ssh" scan_results.gnmap
```

**Extract IP addresses with filtered ports:**
```bash
grep "filtered" scan_results.gnmap | awk '{print $2}'
```

**Count open ports per host:**
```bash
grep "Ports:" scan_results.gnmap | awk -F',' '{print NF}'
```

---

## All Formats (-oA)

Output scan results to all four formats simultaneously.

### Basic Usage

```bash
prtip -sS -p 80,443 192.168.1.0/24 -oA scan_results
```

### Files Created

The `-oA` flag creates four files with different extensions:

| Format | Extension | File Name |
|--------|-----------|-----------|
| Text | `.txt` | `scan_results.txt` |
| JSON | `.json` | `scan_results.json` |
| XML | `.xml` | `scan_results.xml` |
| Greppable | `.gnmap` | `scan_results.gnmap` |

### Example

```bash
# Single command creates all formats
prtip -sS -p 1-1000 10.0.0.0/24 -oA network_audit

# Results in:
# - network_audit.txt
# - network_audit.json
# - network_audit.xml
# - network_audit.gnmap
```

### Use Cases

**Complete Documentation:**
- Text format for human review
- JSON for automated processing
- XML for Nmap tool integration
- Greppable for quick searches

**Compliance Requirements:**
- Multiple format redundancy
- Tool-agnostic archival
- Human + machine readability

**Team Collaboration:**
- Analysts use text format
- Developers use JSON format
- Security tools use XML format
- Shell scripters use greppable format

---

## Format Comparison

| Feature | Text (-oN) | JSON (-oJ) | XML (-oX) | Greppable (-oG) |
|---------|------------|------------|-----------|-----------------|
| **Human Readable** | ✅ Excellent | ⚠️ Good | ❌ Poor | ⚠️ Moderate |
| **Machine Parseable** | ❌ Poor | ✅ Excellent | ✅ Excellent | ✅ Good |
| **Terminal Colors** | ✅ Yes | ❌ No | ❌ No | ❌ No |
| **Nmap Compatible** | ❌ No | ❌ No | ✅ Yes | ✅ Yes |
| **File Size** | Medium | Large | Largest | Smallest |
| **Parsing Speed** | Slow | Fast | Medium | Fastest |
| **Structured Data** | ❌ No | ✅ Yes | ✅ Yes | ⚠️ Limited |
| **Grep Friendly** | ❌ No | ❌ No | ❌ No | ✅ Yes |
| **API Integration** | ❌ Poor | ✅ Excellent | ✅ Good | ❌ Poor |
| **Recommended For** | Humans | APIs | Tools | Scripts |

---

## Parsing Examples

### Python (JSON Format)

```python
import json

# Load JSON results
with open('scan_results.json', 'r') as f:
    data = json.load(f)

# Extract statistics
stats = data['statistics']
print(f"Hosts scanned: {stats['hosts_scanned']}")
print(f"Open ports: {stats['ports_open']}")

# Find all web servers
web_servers = [
    r for r in data['results']
    if r['port'] in [80, 443, 8080, 8443] and r['state'] == 'Open'
]

for server in web_servers:
    print(f"{server['target_ip']}:{server['port']} - {server.get('service', 'unknown')}")

# Filter by response time
fast_responses = [
    r for r in data['results']
    if r.get('response_time_ms', 999999) < 10.0
]
```

### jq (JSON Format)

```bash
# Extract all open ports
jq '.results[] | select(.state == "Open") | {ip: .target_ip, port: .port}' scan_results.json

# Get statistics summary
jq '.statistics' scan_results.json

# Find SSH servers
jq '.results[] | select(.service == "ssh")' scan_results.json

# Count results by state
jq '.results | group_by(.state) | map({state: .[0].state, count: length})' scan_results.json

# Extract IPs with open web ports
jq -r '.results[] | select(.port == 80 or .port == 443) | select(.state == "Open") | .target_ip' scan_results.json
```

### xmllint (XML Format)

```bash
# Extract all open ports
xmllint --xpath "//port[state/@state='open']/@portid" scan_results.xml

# Get scan start time
xmllint --xpath "string(//nmaprun/@start)" scan_results.xml

# Find all hosts with SSH open
xmllint --xpath "//host[ports/port[@portid='22']/state[@state='open']]/address/@addr" scan_results.xml

# Count total ports scanned
xmllint --xpath "count(//port)" scan_results.xml

# Extract service names
xmllint --xpath "//service/@name" scan_results.xml
```

### awk (Greppable Format)

```bash
# Extract IP addresses
awk '/^Host:/ {print $2}' scan_results.gnmap

# Count open ports per host
awk -F',' '/^Ports:/ {print NF}' scan_results.gnmap

# Find hosts with specific service
awk '/80\/open\/tcp\/http/ {print $2}' scan_results.gnmap

# Extract port numbers
awk -F'/' '/^Ports:/ {for(i=1;i<=NF;i++) print $1}' scan_results.gnmap
```

### grep (Greppable Format)

```bash
# Find hosts with SSH open
grep "22/open/tcp/ssh" scan_results.gnmap

# Find all filtered ports
grep "filtered" scan_results.gnmap

# Count hosts with web servers
grep -c "80/open/tcp" scan_results.gnmap

# Extract unique service names
grep -oP '\d+/open/tcp/\K\w+' scan_results.gnmap | sort -u
```

---

## Best Practices

### Format Selection Guide

**Use Text Format (-oN) when:**
- Manual review and analysis required
- Creating human-readable reports
- Terminal-based workflows
- Quick visual inspection needed

**Use JSON Format (-oJ) when:**
- Integrating with APIs or web services
- Processing with Python, JavaScript, or Go
- Storing in document databases (MongoDB, Elasticsearch)
- Building dashboards or visualizations

**Use XML Format (-oX) when:**
- Integrating with Nmap-based tools (Metasploit, OpenVAS)
- Using XML processing pipelines (XSLT, XPath)
- Compliance requires Nmap-compatible output
- Exchanging data with security tools

**Use Greppable Format (-oG) when:**
- Shell scripting with grep, awk, sed
- Quick command-line searches needed
- Minimal storage space required
- One-liner analysis preferred

### Storage Considerations

**File Size Comparison (1,000 results):**
- Text: ~150 KB
- JSON: ~250 KB
- XML: ~400 KB
- Greppable: ~100 KB

**Compression Recommendations:**
```bash
# Compress JSON (best compression ratio)
gzip scan_results.json  # 250 KB → 25 KB (90% reduction)

# Compress XML (good compression)
gzip scan_results.xml   # 400 KB → 50 KB (87.5% reduction)

# Compress greppable (already compact)
gzip scan_results.gnmap # 100 KB → 15 KB (85% reduction)
```

### Integration Patterns

**REST API Integration (JSON):**
```python
import requests
import json

# Load scan results
with open('scan_results.json', 'r') as f:
    data = json.load(f)

# Post to API endpoint
response = requests.post(
    'https://api.example.com/scans',
    json=data,
    headers={'Authorization': 'Bearer token'}
)
```

**Database Import (JSON):**
```python
import json
import sqlite3

# Load results
with open('scan_results.json', 'r') as f:
    data = json.load(f)

# Insert into database
conn = sqlite3.connect('scans.db')
cursor = conn.cursor()

for result in data['results']:
    cursor.execute('''
        INSERT INTO scan_results
        (ip, port, state, service, banner, timestamp)
        VALUES (?, ?, ?, ?, ?, ?)
    ''', (
        result['target_ip'],
        result['port'],
        result['state'],
        result.get('service'),
        result.get('banner'),
        result['timestamp']
    ))

conn.commit()
```

**Log Aggregation (Greppable):**
```bash
# Stream to centralized logging
cat scan_results.gnmap | logger -t prtip -p local0.info

# Append to daily log
cat scan_results.gnmap >> /var/log/prtip/$(date +%Y-%m-%d).log

# Send to syslog server
cat scan_results.gnmap | nc syslog.example.com 514
```

### Performance Tips

**Large Scans (>100,000 results):**
- Use `-oG` for fastest parsing (one-line-per-host)
- Avoid `-oX` (XML overhead)
- Stream JSON to disk with jq: `prtip ... | jq -c > results.jsonl`

**Memory-Constrained Environments:**
- Use `-oG` (smallest file size)
- Process greppable format line-by-line
- Avoid loading full JSON/XML into memory

**Real-Time Processing:**
- Monitor output file with `tail -f scan_results.txt`
- Use named pipes: `mkfifo scan_pipe && prtip ... -oN scan_pipe`
- Stream greppable format to processing pipeline

---

## See Also

- **[Command Reference](./command-reference.md)** - Complete CLI flag reference including output flags
- **[Basic Usage](../user-guide/basic-usage.md)** - Output format usage examples
- **[Database Schema](./database-schema.md)** - SQLite database storage (alternative to file output)
- **[Scan Types](../user-guide/scan-types.md)** - Different scan techniques that generate results
- **[Service Detection](../features/service-detection.md)** - Service detection adds service/version to output
- **[Automation](../advanced/automation.md)** - Automated processing of output formats

**External Resources:**
- **Nmap Output Formats**: Original format specifications
- **JSON Schema**: Validating JSON output
- **XPath/XSLT**: Processing XML output
- **jq Manual**: Advanced JSON querying

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
