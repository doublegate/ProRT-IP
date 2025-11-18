# Tutorial: Your First Scan

Step-by-step tutorials to master ProRT-IP WarScan from basic to advanced usage.

## Tutorial Path

- **Beginner** (Tutorial 1-3): Basic scanning, scan types, service detection
- **Intermediate** (Tutorial 4-5): Advanced service detection, stealth scanning
- **Advanced** (Tutorial 6-7): Large-scale scanning, plugin development

## Tutorial 1: Your First Scan

**Objective:** Complete a basic port scan and understand the output

**Prerequisites:**
- ProRT-IP installed
- Terminal access
- Internet connection

### Step 1: Verify Installation

**Command:**
```bash
prtip --version
```

**Expected Output:**
```
ProRT-IP v0.5.2
```

**Verification:** Version should be 0.5.0 or higher. If not, see [Installation Guide](./installation.md).

### Step 2: Scan a Single Host

**Command:**
```bash
prtip -sS -p 80,443 scanme.nmap.org
```

**Explanation:**
- `-sS`: TCP SYN scan (requires root/admin privileges)
- `-p 80,443`: Scan ports 80 (HTTP) and 443 (HTTPS)
- `scanme.nmap.org`: Target host (Nmap's public scan target)

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

### Step 3: Understand the Output

**Port Column:**
- Format: `PORT/PROTOCOL`
- Example: `80/tcp` = Port 80 using TCP protocol

**State Column:**
- `open`: Service accepting connections
- `closed`: Port accessible but no service
- `filtered`: Blocked by firewall

**Service Column:**
- Common service name (HTTP, HTTPS, SSH, etc.)
- Based on port number (not version detection)

### Step 4: Save Results

**Command:**
```bash
prtip -sS -p 80,443 scanme.nmap.org -oN scan-results.txt
```

**Explanation:**
- `-oN scan-results.txt`: Normal output to file

**Output Formats:**
- `-oN`: Normal (human-readable)
- `-oX`: XML (machine-parseable)
- `-oJ`: JSON (modern APIs)
- `-oG`: Greppable (one-line per host)

### Step 5: Practice Exercise

**Task:** Scan `scanme.nmap.org` for common web ports (80, 443, 8080, 8443)

**Your Command:**
```bash
# Write your command here
prtip -sS -p 80,443,8080,8443 scanme.nmap.org
```

**Expected Result:**
- 2-4 open ports (80 and 443 typically open)
- Scan duration: 1-3 seconds

---

## Tutorial 2: Understanding Scan Types

**Objective:** Learn different scan types and when to use them

### Scan Type Overview

| Scan Type | Command | Privileges | Stealth | Speed | Accuracy |
|-----------|---------|------------|---------|-------|----------|
| SYN Scan | `-sS` | Root | High | Fast | 95% |
| Connect Scan | `-sT` | User | Low | Medium | 99% |
| UDP Scan | `-sU` | Root | Medium | Slow | 80% |
| FIN Scan | `-sF` | Root | Very High | Fast | 60% |
| Xmas Scan | `-sX` | Root | Very High | Fast | 60% |
| NULL Scan | `-sN` | Root | Very High | Fast | 60% |
| ACK Scan | `-sA` | Root | Medium | Fast | Firewall only |
| Idle Scan | `-sI` | Root | Maximum | Slow | 95% |

### Exercise 2.1: SYN Scan vs Connect Scan

**SYN Scan (requires root):**
```bash
sudo prtip -sS -p 1-1000 192.168.1.1
```

**Connect Scan (no root needed):**
```bash
prtip -sT -p 1-1000 192.168.1.1
```

**Comparison:**
- **SYN**: Faster (half-open connection), stealthier (no full TCP handshake)
- **Connect**: Slower (full connection), logged by target, works without privileges

**When to Use:**
- **SYN**: Default choice for privileged scanning (95% of use cases)
- **Connect**: When you don't have root access

### Exercise 2.2: UDP Scan

**Command:**
```bash
sudo prtip -sU -p 53,161,123 192.168.1.1
```

**UDP Services:**
- Port 53: DNS
- Port 161: SNMP
- Port 123: NTP

**Why UDP is Slower:**
- No ACK response from open ports
- Requires waiting for timeout
- ICMP Port Unreachable needed to confirm closed

**Expected Duration:** 10-60 seconds for 3 ports (vs 1-2 seconds for TCP)

### Exercise 2.3: Stealth Scans

**FIN Scan:**
```bash
sudo prtip -sF -p 80,443 scanme.nmap.org
```

**How it Works:**
- Sends FIN packet (normally used to close connection)
- Open ports: No response
- Closed ports: RST response

**Limitations:**
- Windows/Cisco devices respond incorrectly (false positives)
- Less accurate than SYN (60% vs 95%)

**When to Use:**
- Evading simple packet filters
- Testing firewall rules
- When extreme stealth is required

### Practice Exercise

**Task:** Compare SYN scan vs FIN scan on the same target

```bash
# SYN Scan
sudo prtip -sS -p 80,443 scanme.nmap.org -oN syn-scan.txt

# FIN Scan
sudo prtip -sF -p 80,443 scanme.nmap.org -oN fin-scan.txt

# Compare results
diff syn-scan.txt fin-scan.txt
```

**Expected Differences:**
- SYN: Both ports "open"
- FIN: Both ports "open|filtered" (less certain)

---

## Tutorial 3: Service Detection

**Objective:** Identify service versions running on open ports

### Basic Service Detection

**Command:**
```bash
sudo prtip -sV -p 80,443,22 scanme.nmap.org
```

**Explanation:**
- `-sV`: Enable service version detection
- Probes open ports to identify software and version

**Expected Output:**
```
PORT    STATE   SERVICE  VERSION
22/tcp  open    ssh      OpenSSH 6.6.1p1 Ubuntu 2ubuntu2.13
80/tcp  open    http     Apache httpd 2.4.7
443/tcp open    ssl/http Apache httpd 2.4.7
```

### Service Detection Intensity

**Intensity Levels (0-9):**
```bash
# Light detection (intensity 2, faster but less accurate)
sudo prtip -sV --version-intensity 2 -p 80 scanme.nmap.org

# Default detection (intensity 7, balanced)
sudo prtip -sV -p 80 scanme.nmap.org

# Aggressive detection (intensity 9, slower but comprehensive)
sudo prtip -sV --version-intensity 9 -p 80 scanme.nmap.org
```

**Trade-offs:**
- **Low intensity (2):** 5-10 seconds per port, 70% accuracy
- **Default (7):** 15-30 seconds per port, 85-90% accuracy
- **High intensity (9):** 30-60 seconds per port, 95% accuracy

### Protocol-Specific Detection

**HTTP Service:**
```bash
sudo prtip -sV -p 80 --script=http-title scanme.nmap.org
```

**SSH Service:**
```bash
sudo prtip -sV -p 22 scanme.nmap.org
```

**Database Services:**
```bash
sudo prtip -sV -p 3306,5432,1433 192.168.1.100
```

### TLS Certificate Analysis

**Command:**
```bash
sudo prtip -sV -p 443 --script=ssl-cert scanme.nmap.org
```

**Certificate Information Extracted:**
- Subject (domain name)
- Issuer (Certificate Authority)
- Validity period (not before/after dates)
- Subject Alternative Names (SANs)
- Signature algorithm
- Public key algorithm

**Example Output:**
```
PORT     STATE SERVICE   VERSION
443/tcp  open  ssl/http  Apache httpd 2.4.7
| ssl-cert: Subject: commonName=scanme.nmap.org
| Issuer: commonName=Let's Encrypt Authority X3
| Not valid before: 2024-01-15T00:00:00
| Not valid after:  2024-04-15T23:59:59
| SANs: scanme.nmap.org, www.scanme.nmap.org
```

### Practice Exercise

**Task:** Identify all services on common ports of a local device

```bash
# Scan common service ports with version detection
sudo prtip -sV -p 21,22,23,25,80,110,143,443,445,3389 192.168.1.1
```

**Questions to Answer:**
1. What web server version is running (if any)?
2. Is SSH enabled? What version?
3. Are there any outdated services with known vulnerabilities?

**Vulnerability Research:**
```bash
# Search for known vulnerabilities
# Example: If you find "Apache 2.2.8"
searchsploit "Apache 2.2.8"
```

---

## Tutorial 4: Advanced Service Detection

**Objective:** Master advanced service detection techniques

### HTTP-Specific Detection

**Title Extraction:**
```bash
sudo prtip -sV -p 80,443,8080,8443 --script=http-title 192.168.1.0/24
```

**Server Headers:**
```bash
sudo prtip -sV -p 80 --script=http-headers scanme.nmap.org
```

**Example Output:**
```
PORT   STATE SERVICE VERSION
80/tcp open  http    Apache httpd 2.4.7
| http-headers:
|   Server: Apache/2.4.7 (Ubuntu)
|   X-Powered-By: PHP/5.5.9-1ubuntu4.29
|   Content-Type: text/html; charset=UTF-8
```

### Database Service Detection

**MySQL:**
```bash
sudo prtip -sV -p 3306 192.168.1.100
```

**PostgreSQL:**
```bash
sudo prtip -sV -p 5432 192.168.1.100
```

**Expected Output:**
```
PORT     STATE SERVICE  VERSION
3306/tcp open  mysql    MySQL 5.7.32-0ubuntu0.16.04.1
5432/tcp open  postgresql PostgreSQL 12.2
```

### Multi-Protocol Detection

**Scan all common services:**
```bash
sudo prtip -sV -p 21,22,23,25,53,80,110,143,443,445,3306,3389,5432,8080 192.168.1.1
```

**Service Categories:**
- **Remote Access:** 22 (SSH), 23 (Telnet), 3389 (RDP)
- **Web Services:** 80 (HTTP), 443 (HTTPS), 8080 (HTTP-Alt)
- **Mail Services:** 25 (SMTP), 110 (POP3), 143 (IMAP)
- **Database Services:** 3306 (MySQL), 5432 (PostgreSQL)
- **File Sharing:** 445 (SMB)
- **DNS:** 53

### Practice Exercise

**Task:** Create a comprehensive service inventory of your local network

```bash
# Step 1: Discover live hosts
sudo prtip -sn 192.168.1.0/24 -oN live-hosts.txt

# Step 2: Extract IP addresses
grep "is up" live-hosts.txt | awk '{print $2}' > targets.txt

# Step 3: Service detection on all targets
sudo prtip -sV -p 1-1000 -iL targets.txt -oN service-inventory.txt

# Step 4: Analyze results
grep "open" service-inventory.txt | sort | uniq -c
```

**Expected Deliverable:**
- Complete list of all services on your network
- Version information for each service
- Potential security concerns (outdated versions)

---

## Tutorial 5: Stealth Scanning Techniques

**Objective:** Evade detection while gathering intelligence

### Timing Templates

**Paranoid (T0):**
```bash
sudo prtip -sS -T0 -p 80,443 scanme.nmap.org
```

**Configuration:**
- 5 minutes between probes
- Single probe at a time
- Minimal footprint
- Use case: Evading IDS

**Sneaky (T1):**
```bash
sudo prtip -sS -T1 -p 80,443 scanme.nmap.org
```

**Configuration:**
- 15 seconds between probes
- Use case: Slow scan to avoid detection

**Polite (T2):**
```bash
sudo prtip -sS -T2 -p 80,443 scanme.nmap.org
```

**Configuration:**
- 0.4 seconds between probes
- Reduces bandwidth usage
- Use case: Scanning production systems

**Normal (T3) - Default:**
```bash
sudo prtip -sS -p 80,443 scanme.nmap.org
```

**Aggressive (T4):**
```bash
sudo prtip -sS -T4 -p 80,443 scanme.nmap.org
```

**Configuration:**
- 5ms probe delay
- 1 second timeout
- Use case: Fast local network scanning

**Insane (T5):**
```bash
sudo prtip -sS -T5 -p 80,443 scanme.nmap.org
```

**Configuration:**
- No probe delay
- 0.3 second timeout
- Use case: Very fast networks only
- **Warning:** May miss results due to timeouts

### Decoy Scanning

**Basic Decoy:**
```bash
sudo prtip -sS -D RND:5 -p 80,443 scanme.nmap.org
```

**Explanation:**
- `-D RND:5`: Use 5 random decoy IP addresses
- Target sees scans from 6 IPs (5 decoys + your real IP)
- Makes it harder to identify the true source

**Manual Decoy IPs:**
```bash
sudo prtip -sS -D 192.168.1.10,192.168.1.20,ME,192.168.1.30 -p 80 scanme.nmap.org
```

**Explanation:**
- `ME`: Your real IP position in the decoy list
- Other IPs: Decoy addresses

**Best Practices:**
- Use IPs that are active on the network
- Place ME in a random position (not always first/last)
- Use 3-10 decoys (too many is suspicious)

### IP Fragmentation

**Fragment Packets:**
```bash
sudo prtip -sS -f -p 80,443 scanme.nmap.org
```

**Explanation:**
- `-f`: Fragment IP packets into 8-byte chunks
- Evades some packet filters and firewalls
- May bypass simple IDS

**Custom MTU:**
```bash
sudo prtip -sS --mtu 16 -p 80,443 scanme.nmap.org
```

**MTU Values:**
- Must be multiple of 8
- Common values: 8, 16, 24, 32
- Smaller = more fragments = harder to reassemble

### TTL Manipulation

**Custom TTL:**
```bash
sudo prtip -sS --ttl 32 -p 80,443 scanme.nmap.org
```

**Use Cases:**
- Bypass simple packet filters checking for unusual TTL
- Evade traceroute-based detection

### Combined Stealth Techniques

**Maximum Stealth:**
```bash
sudo prtip -sF -T0 -D RND:10 -f --ttl 64 --source-port 53 -p 80,443 scanme.nmap.org
```

**Explanation:**
- `-sF`: FIN scan (stealthy scan type)
- `-T0`: Paranoid timing (very slow)
- `-D RND:10`: 10 random decoys
- `-f`: IP fragmentation
- `--ttl 64`: Normal TTL value (less suspicious)
- `--source-port 53`: Spoof source port as DNS (often allowed through firewalls)

**Expected Duration:** 30-60 minutes for 2 ports

**When to Use:**
- Highly monitored networks
- IDS/IPS evasion required
- Time is not a constraint
- Legal testing only

### Practice Exercise

**Task:** Test firewall evasion on a test network

```bash
# Step 1: Normal scan (baseline)
sudo prtip -sS -p 80,443 192.168.1.1 -oN normal-scan.txt

# Step 2: Stealth scan
sudo prtip -sF -T1 -D RND:5 -f -p 80,443 192.168.1.1 -oN stealth-scan.txt

# Step 3: Compare results
diff normal-scan.txt stealth-scan.txt

# Step 4: Check firewall logs (if accessible)
# Did the stealth scan generate fewer log entries?
```

**Questions:**
1. Did both scans detect the same open ports?
2. What was the time difference?
3. Were there fewer firewall log entries for the stealth scan?

---

## Tutorial 6: Large-Scale Network Scanning

**Objective:** Efficiently scan entire networks

### Subnet Scanning

**Class C Network (256 hosts):**
```bash
sudo prtip -sS -p 80,443 192.168.1.0/24
```

**Expected Duration:**
- 2-5 minutes for 256 hosts × 2 ports
- ~512 total port scans

**Class B Network (65,536 hosts):**
```bash
sudo prtip -sS -p 80,443 192.168.0.0/16 -T4
```

**Expected Duration:**
- 2-4 hours for 65,536 hosts × 2 ports
- ~131,072 total port scans

**Optimization:**
- Use `-T4` or `-T5` for faster scanning
- Limit port range (`-p 80,443` vs `-p 1-65535`)
- Use `--top-ports 100` for most common ports

### Top Ports Scanning

**Fast Scan (Top 100):**
```bash
sudo prtip -F 192.168.1.0/24
```

**Explanation:**
- `-F`: Fast mode (scans top 100 ports)
- Equivalent to `--top-ports 100`

**Top Ports Lists:**
```bash
# Top 10 ports
sudo prtip --top-ports 10 192.168.1.0/24

# Top 1000 ports
sudo prtip --top-ports 1000 192.168.1.0/24
```

### Host Discovery Before Scanning

**Two-Phase Approach:**
```bash
# Phase 1: Discover live hosts (fast)
sudo prtip -sn 192.168.1.0/24 -oN live-hosts.txt

# Phase 2: Port scan only live hosts
sudo prtip -sS -p 1-1000 -iL live-hosts.txt -oN port-scan.txt
```

**Time Savings:**
- If 20 out of 256 hosts are live: 92% reduction in scan time
- Phase 1: 1-2 minutes
- Phase 2: 5-10 minutes (vs 60-120 minutes scanning all 256 hosts)

### Rate Limiting

**Limit Packet Rate:**
```bash
sudo prtip -sS -p 80,443 192.168.1.0/24 --max-rate 1000
```

**Explanation:**
- `--max-rate 1000`: Maximum 1,000 packets per second
- Prevents overwhelming the network
- Required for some production networks

**Minimum Rate:**
```bash
sudo prtip -sS -p 80,443 192.168.1.0/24 --min-rate 100
```

**Explanation:**
- `--min-rate 100`: Minimum 100 packets per second
- Ensures scan doesn't slow down too much
- Useful for large scans with timing constraints

### Parallel Scanning

**Scan Multiple Targets Simultaneously:**
```bash
# Create target list
echo "192.168.1.0/24" > targets.txt
echo "10.0.0.0/24" >> targets.txt

# Scan all targets
sudo prtip -sS -p 80,443 -iL targets.txt -oN parallel-scan.txt
```

### Practice Exercise

**Task:** Scan a large network and generate a comprehensive report

```bash
# Step 1: Define scope
NETWORK="192.168.0.0/16"

# Step 2: Host discovery
sudo prtip -sn $NETWORK -oN hosts.txt

# Step 3: Extract live IPs
grep "is up" hosts.txt | awk '{print $2}' > live.txt

# Step 4: Count live hosts
wc -l live.txt

# Step 5: Fast port scan (top 100 ports)
sudo prtip -sS -F -iL live.txt -oN ports.txt

# Step 6: Service detection on open ports
sudo prtip -sV -p 80,443,22,3389 -iL live.txt -oN services.txt

# Step 7: Generate summary
echo "=== Network Scan Summary ===" > summary.txt
echo "Total hosts scanned: $(wc -l < live.txt)" >> summary.txt
echo "Open ports found: $(grep -c 'open' ports.txt)" >> summary.txt
echo "Services identified: $(grep -c 'open' services.txt)" >> summary.txt
```

**Expected Deliverables:**
- `hosts.txt`: All live hosts
- `ports.txt`: Open ports on all hosts
- `services.txt`: Service versions
- `summary.txt`: High-level statistics

---

## Tutorial 7: Plugin Development

**Objective:** Extend ProRT-IP with custom Lua plugins

### Plugin Basics

**Plugin Structure:**
```lua
-- my-plugin.lua
return {
    name = "My Custom Plugin",
    version = "1.0.0",
    description = "Description of what this plugin does",

    -- Initialize plugin
    init = function(config)
        print("Plugin initialized")
    end,

    -- Process scan result
    process = function(result)
        -- result contains: ip, port, state, service
        if result.state == "open" then
            print(string.format("Found open port: %s:%d", result.ip, result.port))
        end
    end,

    -- Cleanup
    cleanup = function()
        print("Plugin cleanup")
    end
}
```

### Example 1: HTTP Title Checker

**Plugin:** `http-title-checker.lua`
```lua
return {
    name = "HTTP Title Checker",
    version = "1.0.0",
    description = "Extracts HTML titles from HTTP responses",

    process = function(result)
        if result.service == "http" and result.state == "open" then
            -- Make HTTP request
            local response = prtip.http.get(result.ip, result.port, "/")

            -- Extract title
            local title = response.body:match("<title>(.-)</title>")
            if title then
                print(string.format("[%s:%d] Title: %s", result.ip, result.port, title))
            end
        end
    end
}
```

**Usage:**
```bash
sudo prtip -sS -p 80,443 192.168.1.0/24 --plugin http-title-checker.lua
```

### Example 2: Vulnerability Scanner

**Plugin:** `vuln-scanner.lua`
```lua
local vulns = {
    ["Apache 2.2.8"] = "CVE-2011-3192 (Range DoS)",
    ["OpenSSH 6.6"] = "CVE-2016-0777 (Info leak)",
    ["MySQL 5.5.59"] = "CVE-2018-2562 (Privilege escalation)"
}

return {
    name = "Simple Vulnerability Scanner",
    version = "1.0.0",
    description = "Checks for known vulnerable versions",

    process = function(result)
        if result.version then
            local vuln = vulns[result.version]
            if vuln then
                print(string.format("[VULN] %s:%d %s - %s",
                    result.ip, result.port, result.version, vuln))
            end
        end
    end
}
```

**Usage:**
```bash
sudo prtip -sV -p 1-1000 192.168.1.0/24 --plugin vuln-scanner.lua
```

### Example 3: Custom Logger

**Plugin:** `custom-logger.lua`
```lua
local log_file = nil

return {
    name = "Custom Logger",
    version = "1.0.0",
    description = "Logs results to custom format",

    init = function(config)
        log_file = io.open("scan-log.csv", "w")
        log_file:write("Timestamp,IP,Port,State,Service,Version\n")
    end,

    process = function(result)
        local timestamp = os.date("%Y-%m-%d %H:%M:%S")
        log_file:write(string.format("%s,%s,%d,%s,%s,%s\n",
            timestamp,
            result.ip,
            result.port,
            result.state,
            result.service or "",
            result.version or ""))
        log_file:flush()
    end,

    cleanup = function()
        if log_file then
            log_file:close()
        end
    end
}
```

**Usage:**
```bash
sudo prtip -sS -p 1-1000 192.168.1.0/24 --plugin custom-logger.lua
```

### Plugin API Reference

**Available Functions:**
```lua
-- HTTP requests
prtip.http.get(ip, port, path)
prtip.http.post(ip, port, path, data)

-- DNS lookups
prtip.dns.resolve(hostname)
prtip.dns.reverse(ip)

-- Port state checks
prtip.port.is_open(ip, port)

-- Service detection
prtip.service.detect(ip, port)

-- Banner grabbing
prtip.banner.grab(ip, port)
```

### Practice Exercise

**Task:** Create a plugin that identifies web servers with directory listing enabled

```lua
-- directory-listing-checker.lua
return {
    name = "Directory Listing Checker",
    version = "1.0.0",
    description = "Checks for directory listing vulnerability",

    process = function(result)
        if result.service == "http" and result.state == "open" then
            local response = prtip.http.get(result.ip, result.port, "/")

            -- Check for common directory listing indicators
            if response.body:match("Index of /") or
               response.body:match("Directory listing") or
               response.body:match("Parent Directory") then
                print(string.format("[VULN] %s:%d - Directory listing enabled",
                    result.ip, result.port))
            end
        end
    end
}
```

**Test:**
```bash
sudo prtip -sS -p 80,8080 192.168.1.0/24 --plugin directory-listing-checker.lua
```

---

## Practice Exercises

### Exercise 1: Basic Network Mapping

**Objective:** Map all services on your local network

**Steps:**
1. Discover live hosts on your network
2. Scan top 1000 ports on all live hosts
3. Run service detection on open ports
4. Create a network diagram showing all services

**Commands:**
```bash
# Replace 192.168.1.0/24 with your network
sudo prtip -sn 192.168.1.0/24 -oN hosts.txt
sudo prtip -sS --top-ports 1000 -iL hosts.txt -oN ports.txt
sudo prtip -sV -iL hosts.txt -oN services.txt
```

**Deliverable:** Document showing all devices, open ports, and running services

### Exercise 2: Firewall Testing

**Objective:** Test firewall rules on a test system

**Steps:**
1. Perform normal SYN scan
2. Perform stealth scans (FIN, NULL, Xmas)
3. Try fragmentation and decoys
4. Compare results

**Commands:**
```bash
sudo prtip -sS -p 1-1000 TARGET -oN syn.txt
sudo prtip -sF -p 1-1000 TARGET -oN fin.txt
sudo prtip -sN -p 1-1000 TARGET -oN null.txt
sudo prtip -sX -p 1-1000 TARGET -oN xmas.txt
sudo prtip -sS -f -D RND:10 -p 1-1000 TARGET -oN stealth.txt
```

**Deliverable:** Analysis showing which scans were successful and which were blocked

### Exercise 3: Service Inventory

**Objective:** Create comprehensive service inventory

**Steps:**
1. Scan all hosts with service detection
2. Extract all unique services
3. Identify outdated versions
4. Research known vulnerabilities

**Commands:**
```bash
sudo prtip -sV -p 1-10000 192.168.1.0/24 -oN inventory.txt
grep "open" inventory.txt | awk '{print $3, $4, $5}' | sort | uniq > services.txt
```

**Deliverable:** Spreadsheet showing all services, versions, and known vulnerabilities

### Exercise 4: Performance Testing

**Objective:** Test scanning performance on different network types

**Test Cases:**
1. Localhost (127.0.0.1)
2. Local network (192.168.1.0/24)
3. Internet host (scanme.nmap.org)

**Commands:**
```bash
# Localhost
time sudo prtip -sS -p 1-65535 127.0.0.1

# Local network
time sudo prtip -sS -p 1-1000 192.168.1.1

# Internet host
time sudo prtip -sS -p 1-1000 scanme.nmap.org
```

**Deliverable:** Performance report showing scan times and packets per second

### Exercise 5: Custom Plugin Development

**Objective:** Create a plugin for a specific detection need

**Requirements:**
- Detect specific service (e.g., Redis, MongoDB, Elasticsearch)
- Check for default credentials
- Log findings to custom format

**Template:**
```lua
return {
    name = "Your Plugin Name",
    version = "1.0.0",
    description = "Your description",

    init = function(config)
        -- Initialize plugin
    end,

    process = function(result)
        -- Process each scan result
    end,

    cleanup = function()
        -- Cleanup
    end
}
```

### Exercise 6: Scan Script Automation

**Objective:** Create automated scanning workflow

**Requirements:**
1. Daily network scan
2. Email notification if new hosts/services discovered
3. Log all changes

**Bash Script Template:**
```bash
#!/bin/bash
NETWORK="192.168.1.0/24"
DATE=$(date +%Y-%m-%d)
LOGDIR="/var/log/scans"

# Create log directory
mkdir -p $LOGDIR

# Scan network
sudo prtip -sS -p 1-1000 $NETWORK -oN "$LOGDIR/scan-$DATE.txt"

# Compare with previous scan
if [ -f "$LOGDIR/scan-previous.txt" ]; then
    diff "$LOGDIR/scan-previous.txt" "$LOGDIR/scan-$DATE.txt" > "$LOGDIR/changes-$DATE.txt"

    if [ -s "$LOGDIR/changes-$DATE.txt" ]; then
        # Changes detected, send email
        mail -s "Network Changes Detected" admin@example.com < "$LOGDIR/changes-$DATE.txt"
    fi
fi

# Update previous scan
cp "$LOGDIR/scan-$DATE.txt" "$LOGDIR/scan-previous.txt"
```

### Exercise 7: IPv6 Scanning

**Objective:** Scan IPv6 networks

**Steps:**
1. Discover IPv6 hosts on local network
2. Scan common IPv6 ports
3. Compare with IPv4 scan results

**Commands:**
```bash
# IPv6 scan
sudo prtip -6 -sS -p 80,443 fe80::/10

# IPv4 scan (for comparison)
sudo prtip -sS -p 80,443 192.168.1.0/24
```

### Exercise 8: Idle Scan

**Objective:** Perform anonymous scanning using idle scan technique

**Requirements:**
- Identify zombie host (low-traffic host)
- Perform idle scan through zombie
- Verify results

**Commands:**
```bash
# Find potential zombie hosts
sudo prtip -sI --find-zombies 192.168.1.0/24

# Perform idle scan
sudo prtip -sI ZOMBIE_IP -p 80,443 TARGET_IP
```

### Exercise 9: Large-Scale Internet Scanning

**Objective:** Scan a large IP range efficiently

**Requirements:**
- Scan at least /16 network
- Use appropriate timing
- Generate comprehensive report

**Commands:**
```bash
# Phase 1: Host discovery
sudo prtip -sn 10.0.0.0/16 -oN hosts.txt --max-rate 1000

# Phase 2: Port scan
sudo prtip -sS --top-ports 100 -iL hosts.txt -oN ports.txt -T4

# Phase 3: Service detection
sudo prtip -sV -iL hosts.txt -oN services.txt
```

**Expected Duration:** 2-6 hours depending on network size and timing

---

## Common Pitfalls

### Pitfall 1: Insufficient Privileges

**Error:**
```
Error: You need root privileges to run SYN scan (-sS)
```

**Solution:**
```bash
# Use sudo
sudo prtip -sS -p 80,443 TARGET

# OR use Connect scan (no privileges needed)
prtip -sT -p 80,443 TARGET
```

### Pitfall 2: Firewall Blocking

**Symptom:** All ports show as "filtered"

**Diagnosis:**
```bash
# Try different scan types
sudo prtip -sS -p 80 TARGET  # SYN scan
sudo prtip -sA -p 80 TARGET  # ACK scan (firewall mapping)
```

**Solution:**
- Use fragmentation: `-f`
- Use decoys: `-D RND:10`
- Source port spoofing: `--source-port 53`

### Pitfall 3: Slow Scans

**Problem:** Scan takes hours for small network

**Diagnosis:**
```bash
# Check if using slow timing
prtip -v -sS TARGET  # Shows timing being used
```

**Solutions:**
```bash
# Increase timing
sudo prtip -sS -T4 TARGET

# Limit port range
sudo prtip -sS --top-ports 100 TARGET

# Increase max rate
sudo prtip -sS --max-rate 1000 TARGET
```

### Pitfall 4: Incorrect Port Ranges

**Error:**
```bash
# Wrong: Port 0 doesn't exist
prtip -sS -p 0-1000 TARGET

# Correct: Start from port 1
prtip -sS -p 1-1000 TARGET
```

**Common Ranges:**
- Well-known ports: `1-1023`
- Registered ports: `1024-49151`
- Dynamic ports: `49152-65535`
- All ports: `1-65535` or `-p-`

---

## Next Steps

**After completing these tutorials:**

1. **Read the User Guide**: [../user-guide/basic-usage.md](../user-guide/basic-usage.md)
2. **Explore Feature Guides**: [../features/](../features/)
3. **Review Examples**: [./examples.md](./examples.md)
4. **Advanced Topics**: [../advanced/](../advanced/)

**Additional Resources:**

- [Scan Types Reference](../user-guide/scan-types.md)
- [Service Detection Guide](../features/service-detection.md)
- [Stealth Techniques](../features/stealth-scanning.md)
- [Performance Tuning](../advanced/performance-tuning.md)
- [Plugin Development Guide](../features/plugin-system.md)

**Practice Labs:**

- [scanme.nmap.org](http://scanme.nmap.org) - Official Nmap test target
- [HackTheBox](https://www.hackthebox.eu) - Penetration testing labs
- TryHackMe (tryhackme.com) - Security training platform

**Community:**

- GitHub Issues: [https://github.com/doublegate/ProRT-IP/issues](https://github.com/doublegate/ProRT-IP/issues)
- Discussions: [https://github.com/doublegate/ProRT-IP/discussions](https://github.com/doublegate/ProRT-IP/discussions)
