# Port Specification

Comprehensive guide to port specification syntax in ProRT-IP.

## Overview

**Port specification** defines which ports to scan on target hosts. ProRT-IP supports flexible port specification syntax compatible with Nmap, allowing single ports, ranges, lists, and combinations.

**Key Capabilities:**
- **Flexible Syntax:** Single ports, ranges, comma-separated lists, mixed formats
- **Top Ports:** Fast scanning of most common ports (top 100, top 1000, or custom)
- **All Ports:** Full 65,535 port scan with `-p-` syntax
- **Port Filtering:** Exclude or include specific ports
- **Validation:** Automatic validation of port numbers (1-65535)
- **Service Names:** Optional service name resolution (http → 80, https → 443)

**Common Use Cases:**
- **Quick Scan:** `-F` or `--top-ports 100` (most common 100 ports)
- **Web Services:** `-p 80,443,8080,8443` (HTTP/HTTPS variants)
- **Full Scan:** `-p-` or `-p 1-65535` (all ports, slow)
- **Custom Range:** `-p 1-1000` (well-known and registered ports)
- **Exclude Ports:** `--exclude-ports 80,443` (skip specific ports)

---

## Basic Port Specification Syntax

### Single Port

Scan a single port:

```bash
prtip -sS -p 80 192.168.1.1
```

**Explanation:**
- `-p 80`: Scan only port 80 (HTTP)
- Fastest option for specific service checks
- Common for service availability verification

**Example Output:**
```
PORT    STATE   SERVICE
80/tcp  open    http
```

**Use Cases:**
- Verify specific service is running
- Check HTTP server availability: `-p 80`
- Check HTTPS: `-p 443`
- Check SSH: `-p 22`
- Check database: `-p 3306` (MySQL), `-p 5432` (PostgreSQL)

### Port Range

Scan a continuous range of ports:

```bash
prtip -sS -p 1-1000 192.168.1.1
```

**Explanation:**
- `-p 1-1000`: Scan ports 1 through 1000 (inclusive)
- Hyphen notation: `start-end`
- Both start and end ports are included
- Range must be ascending (start ≤ end)

**Common Ranges:**
```bash
# Well-known ports (system services)
prtip -sS -p 1-1023 TARGET

# Well-known + registered ports
prtip -sS -p 1-49151 TARGET

# Custom range for specific services
prtip -sS -p 8000-9000 TARGET
```

**Performance Note:**
- Larger ranges take longer to scan
- Use timing templates (`-T0` to `-T5`) to control speed
- Consider `--top-ports` for faster reconnaissance

### Port List

Scan multiple specific ports (comma-separated):

```bash
prtip -sS -p 22,80,443,3389 192.168.1.1
```

**Explanation:**
- `-p 22,80,443,3389`: Scan exactly these 4 ports
- Comma-separated list
- No spaces allowed (use quotes if needed: `-p "22, 80"`)
- Order doesn't matter

**Common Port Lists:**

```bash
# Web services
prtip -sS -p 80,443,8080,8443 TARGET

# Remote access
prtip -sS -p 22,23,3389,5900 TARGET

# Databases
prtip -sS -p 3306,5432,1433,27017 TARGET

# Mail servers
prtip -sS -p 25,110,143,587,993,995 TARGET

# File sharing
prtip -sS -p 21,22,445,139 TARGET
```

**Whitespace Handling:**
- **Recommended:** No spaces (`-p 22,80,443`)
- **With spaces:** Use quotes (`-p "22, 80, 443"`)
- Whitespace is trimmed during parsing

### Mixed Format

Combine ranges and lists in single specification:

```bash
prtip -sS -p 22,80-85,443,8080-8090 192.168.1.1
```

**Explanation:**
- `-p 22,80-85,443,8080-8090`: Scans ports 22, 80-85, 443, 8080-8090
- Total ports: 1 + 6 + 1 + 11 = 19 ports
- Efficient for scanning related service groups

**Example Breakdown:**
```
Input: 22,80-85,443,8080-8090
Ports: 22, 80, 81, 82, 83, 84, 85, 443, 8080, 8081, ..., 8090
Count: 19 ports
```

**Advanced Examples:**

```bash
# Development environment (SSH + web services + databases)
prtip -sS -p 22,80,443,3000-3010,5432,27017 TARGET

# Security audit (common services + high ports)
prtip -sS -p 21-25,80,110,143,443-445,3389,8000-9000 TARGET

# IoT device scan
prtip -sS -p 22,23,80,443,1883,5000-5010,8080-8090 TARGET
```

---

## Special Port Specifications

### All Ports

Scan all 65,535 TCP ports:

```bash
prtip -sS -p- 192.168.1.1
```

**Explanation:**
- `-p-`: Shorthand for `-p 1-65535`
- Scans every possible TCP port
- **Very slow** (can take hours depending on timing)
- Use aggressive timing (`-T4`) for local networks

**Equivalent Commands:**
```bash
prtip -sS -p-              # Shorthand (recommended)
prtip -sS -p 1-65535       # Explicit range (equivalent)
prtip -sS --all-ports      # Verbose flag (if supported)
```

**Performance Characteristics:**

| Network Type | Timing | Duration | Recommended? |
|--------------|--------|----------|--------------|
| Localhost | T4 | 5-10 min | ✅ Yes |
| Local LAN | T4 | 30-60 min | ✅ Yes (with patience) |
| Internet | T3 | 4-8 hours | ⚠️ Use with caution |
| Internet | T0 | 2-5 days | ❌ Not practical |

**Best Practices:**

```bash
# Fast local scan
sudo prtip -sS -p- -T4 192.168.1.1 -oN fullscan.txt

# Internet scan (moderate speed)
sudo prtip -sS -p- -T3 target.com --max-retries 2

# Save results to avoid rescans
sudo prtip -sS -p- -T4 TARGET -oA fullscan_$(date +%Y%m%d)
```

**When to Use:**
- ✅ Security audits requiring complete coverage
- ✅ Penetration testing engagements
- ✅ Finding hidden services on known hosts
- ✅ Compliance scans requiring full port inventory
- ❌ Routine reconnaissance (use `-F` or `--top-ports` instead)
- ❌ Time-constrained scans

### Top Ports (Fast Scan)

Scan the N most common ports based on nmap-services frequency:

```bash
# Top 100 ports (default fast scan)
prtip -F 192.168.1.1

# Equivalent to:
prtip --top-ports 100 192.168.1.1
```

**Top Ports Database:**

ProRT-IP includes a curated database of common ports based on real-world service prevalence:

- **Top 100 Ports:** Most common services (covers ~90% of real-world open ports)
- **Top 1000 Ports:** Comprehensive coverage (covers ~99% of services)

**Common Top-N Values:**

```bash
# Ultra-fast reconnaissance (top 10)
prtip --top-ports 10 TARGET
# Ports: 80, 23, 443, 21, 22, 25, 3389, 110, 445, 139

# Quick scan (top 100) - DEFAULT -F behavior
prtip -F TARGET
prtip --top-ports 100 TARGET
# Scans top 100 most common ports (3-5 seconds typical)

# Thorough scan (top 1000)
prtip --top-ports 1000 TARGET
# Scans top 1000 ports (30-60 seconds typical)
```

**Top 100 Ports List:**

The following ports are included in `-F` (fast scan):

**Essential Services (Top 20):**
```
21   FTP          | File Transfer Protocol
22   SSH          | Secure Shell
23   Telnet       | Unencrypted remote access
25   SMTP         | Email (sending)
53   DNS          | Domain Name System
80   HTTP         | Web traffic
110  POP3         | Email (receiving)
111  RPCbind      | RPC port mapper
135  MSRPC        | Microsoft RPC
139  NetBIOS      | Windows file sharing
143  IMAP         | Email (IMAP)
443  HTTPS        | Secure web traffic
445  SMB          | Windows file sharing (modern)
993  IMAPS        | Secure IMAP
995  POP3S        | Secure POP3
1433 MS-SQL       | Microsoft SQL Server
3306 MySQL        | MySQL database
3389 RDP          | Remote Desktop Protocol
5432 PostgreSQL   | PostgreSQL database
8080 HTTP-Alt     | Alternative HTTP
```

**Full list:** See `crates/prtip-core/src/top_ports.rs` for complete TOP_100_PORTS and TOP_1000_PORTS arrays.

**Performance Comparison:**

| Specification | Ports Scanned | Typical Duration | Coverage |
|---------------|---------------|------------------|----------|
| `--top-ports 10` | 10 | 1-2 seconds | ~50% services |
| `-F` (top 100) | 100 | 3-5 seconds | ~90% services |
| `--top-ports 1000` | 1,000 | 30-60 seconds | ~99% services |
| `-p 1-1000` | 1,000 | 30-60 seconds | Well-known + registered |
| `-p-` (all) | 65,535 | Hours | 100% coverage |

**When to Use:**

- ✅ **Initial reconnaissance:** `-F` provides 90% service coverage in seconds
- ✅ **Time-constrained scans:** Quick network inventory
- ✅ **Large network scans:** 192.168.0.0/16 with `-F` = manageable duration
- ✅ **Continuous monitoring:** Periodic scans of critical services
- ❌ **Compliance audits:** May require full port scan (`-p-`)
- ❌ **Exhaustive security testing:** Hidden services on non-standard ports

**Recommendation:**
Start with `-F` for reconnaissance, then use targeted full scans (`-p-`) on interesting hosts.

---

## Port Validation Rules

ProRT-IP enforces strict port validation to prevent errors and ensure compatibility:

### Valid Port Range

**Allowed:** Ports 1-65535 (inclusive)
**Forbidden:** Port 0 (reserved, invalid)

```bash
# Valid ports
prtip -sS -p 1 TARGET        # ✅ Port 1 (valid)
prtip -sS -p 80 TARGET       # ✅ Port 80 (valid)
prtip -sS -p 65535 TARGET    # ✅ Port 65535 (max valid port)

# Invalid ports
prtip -sS -p 0 TARGET        # ❌ Port 0 (invalid, error)
prtip -sS -p 65536 TARGET    # ❌ Port 65536 (exceeds u16::MAX, error)
prtip -sS -p -1 TARGET       # ❌ Negative port (invalid, error)
prtip -sS -p 99999 TARGET    # ❌ Port 99999 (exceeds 65535, error)
```

**Error Messages:**

```
# Port 0
Error: invalid port specification: port 0 is invalid

# Port too high
Error: invalid port number: 65536 (exceeds maximum 65535)

# Negative port
Error: invalid port number: -1
```

**Rationale:**
- Port 0 is reserved by IANA (cannot be assigned)
- TCP/UDP ports are 16-bit unsigned integers (0-65535 range)
- Port 0 has special meaning in socket programming (OS-assigned port)

### Range Order Validation

Ranges must be specified in ascending order:

```bash
# Valid ranges
prtip -sS -p 80-85 TARGET    # ✅ Ascending (80 → 85)
prtip -sS -p 1-65535 TARGET  # ✅ Full range
prtip -sS -p 8080-8080 TARGET # ✅ Single port as range (equivalent to -p 8080)

# Invalid ranges
prtip -sS -p 85-80 TARGET    # ❌ Descending (end < start)
prtip -sS -p 1000-100 TARGET # ❌ Reversed
```

**Error Message:**
```
Error: invalid port range: end port 80 < start port 85
```

**Why Required:**
- Prevents ambiguous specifications
- Ensures consistent behavior
- Range iteration always ascending

**Note:** Some tools auto-reverse descending ranges. ProRT-IP returns an error for clarity.

### Empty Specification Validation

Port specification cannot be empty:

```bash
# Invalid
prtip -sS -p "" TARGET       # ❌ Empty string
prtip -sS -p , TARGET        # ❌ Empty list (only comma)
prtip -sS -p ,,, TARGET      # ❌ Multiple commas, no ports

# Valid
prtip -sS TARGET             # ✅ Uses default: -p 1-1000
```

**Error Message:**
```
Error: invalid port specification: empty port specification
```

**Default Behavior:**
If `-p` is omitted entirely, ProRT-IP uses default port range `1-1000` (well-known and registered ports).

### Parsing Algorithm

ProRT-IP's port specification parser follows this logic:

```rust
// Simplified parsing algorithm (from crates/prtip-core/src/types.rs)
fn parse(input: &str) -> Result<PortRange> {
    // Step 1: Empty check
    if input.is_empty() {
        return Err("empty port specification");
    }

    // Step 2: Check for comma-separated list
    if input.contains(',') {
        // Recursively parse each part
        let parts = input.split(',').map(|s| parse(s.trim()));
        return Ok(PortRange::List(parts));
    }

    // Step 3: Check for range (hyphen)
    if input.contains('-') {
        let parts: Vec<&str> = input.split('-').collect();
        let start = parts[0].parse::<u16>()?;
        let end = parts[1].parse::<u16>()?;

        // Validate port 0 and range order
        if start == 0 || end == 0 {
            return Err("port 0 is invalid");
        }
        if end < start {
            return Err("end port < start port");
        }

        return Ok(PortRange::Range(start, end));
    }

    // Step 4: Single port
    let port = input.parse::<u16>()?;
    if port == 0 {
        return Err("port 0 is invalid");
    }
    Ok(PortRange::Single(port))
}
```

**Parse Order:**
1. **Empty check** → Error if empty
2. **Comma detection** → Recursively parse list
3. **Hyphen detection** → Parse range with validation
4. **Default** → Parse as single port

---

## Port Categories

Understanding port categories helps with security and scanning strategy:

### Well-Known Ports (1-1023)

**Definition:** Ports assigned by IANA for common services. Require root/admin privileges to bind on Unix systems.

**Common Well-Known Ports:**

```bash
# Scan well-known ports only
prtip -sS -p 1-1023 TARGET
```

| Port | Service | Description |
|------|---------|-------------|
| 20-21 | FTP | File Transfer Protocol (data/control) |
| 22 | SSH | Secure Shell |
| 23 | Telnet | Unencrypted remote access |
| 25 | SMTP | Simple Mail Transfer Protocol |
| 53 | DNS | Domain Name System |
| 67-68 | DHCP | Dynamic Host Configuration Protocol |
| 80 | HTTP | Hypertext Transfer Protocol |
| 110 | POP3 | Post Office Protocol v3 |
| 123 | NTP | Network Time Protocol |
| 143 | IMAP | Internet Message Access Protocol |
| 161-162 | SNMP | Simple Network Management Protocol |
| 389 | LDAP | Lightweight Directory Access Protocol |
| 443 | HTTPS | HTTP Secure (TLS/SSL) |
| 445 | SMB | Server Message Block (Windows) |
| 465 | SMTPS | SMTP Secure |
| 514 | Syslog | System logging |
| 587 | Submission | Email submission (SMTP) |
| 636 | LDAPS | LDAP Secure |
| 993 | IMAPS | IMAP Secure |
| 995 | POP3S | POP3 Secure |

**Security Note:**
- Most critical services run on well-known ports
- High-value targets for attackers
- Should be secured with firewall rules
- Monitor these ports closely in logs

### Registered Ports (1024-49151)

**Definition:** Ports registered with IANA for specific services. Can be bound by non-privileged users.

**Common Registered Ports:**

```bash
# Scan registered ports
prtip -sS -p 1024-49151 TARGET
```

| Port | Service | Description |
|------|---------|-------------|
| 1433 | MS-SQL | Microsoft SQL Server |
| 1521 | Oracle | Oracle database |
| 2049 | NFS | Network File System |
| 3306 | MySQL | MySQL database |
| 3389 | RDP | Remote Desktop Protocol |
| 5432 | PostgreSQL | PostgreSQL database |
| 5900 | VNC | Virtual Network Computing |
| 6379 | Redis | Redis database |
| 8080 | HTTP-Alt | Alternative HTTP (often proxy) |
| 8443 | HTTPS-Alt | Alternative HTTPS |
| 9200 | Elasticsearch | Elasticsearch REST API |
| 27017 | MongoDB | MongoDB database |

**Use Cases:**
- Application-specific services
- Databases and middleware
- Custom enterprise applications
- Development servers (8000, 3000, 4000, 5000)

### Dynamic/Private Ports (49152-65535)

**Definition:** Ephemeral ports assigned by operating system for client-side connections. Generally not used for listening services.

```bash
# Scan dynamic ports (uncommon)
prtip -sS -p 49152-65535 TARGET
```

**Characteristics:**
- **Client-side ports:** Used by OS for outbound connections
- **Temporary:** Recycled after connection closes
- **Rarely scanned:** Few listening services in this range
- **Exceptions:** Some P2P applications, custom services

**When to Scan:**
- ✅ P2P application discovery
- ✅ Custom enterprise services on non-standard ports
- ✅ Comprehensive security audits
- ❌ Routine reconnaissance (low value)

---

## Port Filtering

ProRT-IP supports inclusion and exclusion of ports from scan ranges:

### Exclude Specific Ports

Skip certain ports while scanning a range:

```bash
# Scan 1-1000 but exclude common web ports
prtip -sS -p 1-1000 --exclude-ports 80,443,8080 TARGET
```

**Explanation:**
- `--exclude-ports 80,443,8080`: Blacklist these ports
- Scans 997 ports (1000 total - 3 excluded)
- Useful for avoiding known services or rate-limited ports

**Common Exclusions:**

```bash
# Exclude web ports (already audited)
prtip -sS -p- --exclude-ports 80,443,8080,8443 TARGET

# Exclude Windows file sharing (SMB can trigger IDS)
prtip -sS -p 1-1000 --exclude-ports 135,137-139,445 TARGET

# Exclude mail ports (slow to respond)
prtip -sS -p 1-10000 --exclude-ports 25,110,143,587,993,995 TARGET

# Exclude database ports (separate scan planned)
prtip -sS -p- --exclude-ports 3306,5432,1433,27017,6379 TARGET
```

**Use Cases:**
- Skip previously scanned ports
- Avoid triggering specific IDS/IPS rules
- Exclude slow-responding services
- Reduce scan duration by skipping known-closed ports

### Include Only Specific Ports

Create a whitelist of allowed ports (if supported):

```bash
# Hypothetical syntax (check documentation for support)
prtip -sS --include-ports 22,80,443,3389 TARGET
```

**Explanation:**
- `--include-ports`: Whitelist mode (if supported)
- Scans only specified ports, ignores all others
- Equivalent to `-p 22,80,443,3389` for simple cases

**Note:** For basic inclusion, use `-p` flag directly:

```bash
# Recommended approach (standard -p flag)
prtip -sS -p 22,80,443,3389 TARGET
```

---

## Service Name Resolution

ProRT-IP may support resolving service names to port numbers:

### Common Service Names

```bash
# Using service names instead of port numbers
prtip -sS -p http,https,ssh TARGET
```

**Common Service Mappings:**

| Service Name | Port | Protocol |
|--------------|------|----------|
| `ftp` | 21 | TCP |
| `ssh` | 22 | TCP |
| `telnet` | 23 | TCP |
| `smtp` | 25 | TCP |
| `dns` | 53 | TCP/UDP |
| `http` | 80 | TCP |
| `pop3` | 110 | TCP |
| `imap` | 143 | TCP |
| `https` | 443 | TCP |
| `smb` | 445 | TCP |
| `submission` | 587 | TCP |
| `imaps` | 993 | TCP |
| `pop3s` | 995 | TCP |
| `mysql` | 3306 | TCP |
| `rdp` | 3389 | TCP |
| `postgres` | 5432 | TCP |
| `redis` | 6379 | TCP |
| `http-alt` | 8080 | TCP |
| `https-alt` | 8443 | TCP |
| `mongodb` | 27017 | TCP |

**Example Usage:**

```bash
# Web services
prtip -sS -p http,https,http-alt,https-alt TARGET

# Remote access
prtip -sS -p ssh,telnet,rdp TARGET

# Databases
prtip -sS -p mysql,postgres,mongodb,redis TARGET

# Mail servers
prtip -sS -p smtp,pop3,imap,submission TARGET
```

**Advantages:**
- **Readability:** `http,https` clearer than `80,443`
- **Maintainability:** Update `/etc/services` to change ports
- **Portability:** Same command works if service port changes

**Limitations:**
- Not all ProRT-IP versions may support service names
- Requires `/etc/services` file or internal mapping
- May fail if service name unrecognized

**Check Support:**
```bash
# Test if service names work
prtip -sS -p http 192.168.1.1

# If error, use numeric ports
prtip -sS -p 80 192.168.1.1
```

---

## Advanced Usage

### Combining with Other Flags

Port specification works with all scan types and options:

```bash
# SYN scan with service detection on top 100
sudo prtip -sS -sV -F 192.168.1.1

# UDP scan with specific ports
sudo prtip -sU -p 53,161,500 192.168.1.1

# Stealth FIN scan, all ports, aggressive timing
sudo prtip -sF -p- -T4 192.168.1.1

# OS fingerprinting with custom port range
sudo prtip -sS -O -p 1-5000 192.168.1.1

# Idle scan through zombie, top 1000 ports
sudo prtip -sI zombie.example.com -p --top-ports 1000 TARGET
```

### Batch Scanning Multiple Targets

Scan same ports across multiple targets:

```bash
# CIDR notation
prtip -sS -p 80,443 192.168.1.0/24

# Multiple targets
prtip -sS -p 22,3389 192.168.1.1 192.168.1.10 192.168.1.20

# Target file
prtip -sS -p- -iL targets.txt
```

**targets.txt:**
```
192.168.1.1
192.168.1.10
10.0.0.0/24
webserver.example.com
```

### Scan Optimization Strategies

**Strategy 1: Progressive Depth**

Start fast, go deeper as needed:

```bash
# Step 1: Quick reconnaissance (top 100)
prtip -F TARGET -oN quick.txt

# Step 2: If interesting, scan top 1000
prtip --top-ports 1000 TARGET -oN medium.txt

# Step 3: Full scan if critical target
prtip -p- -T4 TARGET -oN full.txt
```

**Strategy 2: Parallel Port Groups**

Scan different port groups in parallel for faster results:

```bash
# Terminal 1: Well-known ports
prtip -sS -p 1-1023 TARGET -oN well_known.txt &

# Terminal 2: Registered ports
prtip -sS -p 1024-49151 TARGET -oN registered.txt &

# Terminal 3: Dynamic ports
prtip -sS -p 49152-65535 TARGET -oN dynamic.txt &

# Wait for all
wait
```

**Strategy 3: Service-Specific Scanning**

Target specific service categories:

```bash
# Web services only
prtip -sS -p 80,443,8000-9000 -sV TARGET

# Database audit
prtip -sS -p 1433,3306,5432,27017,6379 -sV TARGET

# Remote access review
prtip -sS -p 22,23,3389,5900 -sV TARGET
```

---

## Performance Considerations

### Scan Duration Estimates

Approximate scan times (T3 normal timing, good network):

| Port Specification | Port Count | Localhost | LAN | Internet |
|--------------------|------------|-----------|-----|----------|
| `-p 80` | 1 | <1s | <1s | 1-2s |
| `-p 80,443,8080` | 3 | <1s | <1s | 3-5s |
| `-F` (top 100) | 100 | 1-2s | 3-5s | 30s-1min |
| `--top-ports 1000` | 1,000 | 5-10s | 30-60s | 5-10min |
| `-p 1-1000` | 1,000 | 5-10s | 30-60s | 5-10min |
| `-p 1-10000` | 10,000 | 1-2min | 5-10min | 1-2hr |
| `-p-` (all 65535) | 65,535 | 5-10min | 30-60min | 4-8hr |

**Variables Affecting Duration:**
- **Timing template:** T0 (slowest) to T5 (fastest)
- **Network latency:** Internet scans 10-100x slower than LAN
- **Parallelism:** Higher concurrency = faster scans (up to network limits)
- **Target responsiveness:** Firewall/IDS may slow responses
- **Open port count:** More open ports = longer service detection

### Timing Template Impact

Same scan (`-p 1-1000`) with different timing:

| Timing | Delays | Parallelism | Typical Duration | Use Case |
|--------|--------|-------------|------------------|----------|
| T0 Paranoid | 5min | 1 | ~83 hours | Maximum stealth |
| T1 Sneaky | 15s | 10 | ~25 minutes | IDS evasion |
| T2 Polite | 0.4s | 100 | ~6 minutes | Production networks |
| T3 Normal | 0s | 1000 | ~30-60s | Default scanning |
| T4 Aggressive | 0s | 5000 | ~10-20s | Fast LANs |
| T5 Insane | 0s | 10000 | ~5-10s | Very fast LANs |

**Recommendation:**
- **LAN scans:** Use T4 for speed
- **Internet scans:** Use T3 (default) for reliability
- **IDS evasion:** Use T1 or T2
- **Production systems:** Use T2 (polite)

### Memory Usage

Port specification affects memory consumption:

**Memory Formula:**
```
Memory (MB) = Base (2 MB) + Ports × Target_Count × Result_Size (1 KB)
```

**Examples:**

| Specification | Targets | Ports | Memory |
|---------------|---------|-------|--------|
| `-p 80` | 1 | 1 | ~2 MB |
| `-F` (top 100) | 1 | 100 | ~2.1 MB |
| `-p 1-1000` | 1 | 1,000 | ~3 MB |
| `-p-` | 1 | 65,535 | ~67 MB |
| `-p 1-1000` | 256 (/24) | 1,000 | ~258 MB |
| `-p-` | 256 (/24) | 65,535 | ~16 GB |

**Memory Optimization Tips:**
- Use `--top-ports` instead of `-p-` for large networks
- Stream results to disk with `-oN` flag
- Scan in batches for very large target lists
- Limit port range for /16 or larger networks

---

## Common Patterns and Examples

### Example 1: Web Server Audit

**Goal:** Check if web server is running and identify version

```bash
# Quick check
prtip -sS -p 80,443 webserver.example.com

# With service detection
sudo prtip -sS -sV -p 80,443,8080,8443 webserver.example.com
```

**Expected Output:**
```
PORT     STATE   SERVICE  VERSION
80/tcp   open    http     nginx 1.18.0
443/tcp  open    https    nginx 1.18.0 (TLS 1.3)
8080/tcp closed  http-alt
8443/tcp closed  https-alt
```

### Example 2: Database Security Scan

**Goal:** Find exposed databases on network

```bash
# Scan common database ports across subnet
sudo prtip -sS -sV -p 3306,5432,1433,27017,6379 192.168.1.0/24
```

**Security Checklist:**
- ✅ Databases should NOT be accessible from internet
- ✅ Should bind to localhost only (127.0.0.1)
- ✅ Firewall should block external access
- ⚠️ If open: Verify authentication, encryption, and firewall rules

### Example 3: Initial Network Reconnaissance

**Goal:** Quickly identify active services on unknown target

```bash
# Phase 1: Fast scan (top 100 ports)
prtip -F TARGET -oN recon_quick.txt

# Phase 2: Review results, identify interesting services

# Phase 3: Targeted deep scan
prtip -sS -sV -p 22,80,443,3306,8080 TARGET -oN recon_detailed.txt
```

**Progressive Approach:**
1. **Quick scan** (`-F`): 90% service coverage in seconds
2. **Analyze results**: Identify critical services
3. **Targeted scan**: Deep dive on interesting ports with `-sV`
4. **Full scan** (if needed): `-p-` for comprehensive audit

### Example 4: Penetration Testing

**Goal:** Comprehensive port scan for security assessment

```bash
# Full port scan with service detection
sudo prtip -sS -sV -p- -T4 TARGET -oA pentest_fullscan

# Parse results
cat pentest_fullscan.txt | grep open
```

**Penetration Test Port Strategy:**
1. **Initial:** `-F` for quick service identification
2. **Full scan:** `-p-` for comprehensive coverage
3. **Service detection:** `-sV` on all open ports
4. **OS fingerprinting:** `-O` for OS identification
5. **Vulnerability scanning:** Match versions to CVE database

### Example 5: Continuous Monitoring

**Goal:** Monitor critical services availability

```bash
# Monitor script (cron every 5 minutes)
#!/bin/bash
prtip -sS -p 22,80,443,3306 critical-server.example.com -oN /tmp/monitor.txt

# Alert if any port closed
if grep -q closed /tmp/monitor.txt; then
    mail -s "ALERT: Port closed on critical-server" admin@example.com < /tmp/monitor.txt
fi
```

**Monitoring Checklist:**
- Keep port list small (faster scans)
- Use `-T4` for speed
- Save results with timestamps
- Alert on state changes (open → closed)

---

## Troubleshooting

### Issue 1: "Port 0 is Invalid" Error

**Problem:**
```
$ prtip -sS -p 0 TARGET
Error: invalid port specification: port 0 is invalid
```

**Cause:** Port 0 is reserved and cannot be scanned.

**Solution:**
Use valid ports (1-65535):
```bash
prtip -sS -p 1 TARGET   # Start from port 1
```

---

### Issue 2: "End Port < Start Port" Error

**Problem:**
```
$ prtip -sS -p 85-80 TARGET
Error: invalid port range: end port 80 < start port 85
```

**Cause:** Range specified in descending order.

**Solution:**
Reverse the range to ascending order:
```bash
prtip -sS -p 80-85 TARGET   # Correct: start < end
```

---

### Issue 3: Empty Port Specification

**Problem:**
```
$ prtip -sS -p "" TARGET
Error: invalid port specification: empty port specification
```

**Cause:** Port specification is empty string.

**Solution:**
Specify ports or omit `-p` to use default:
```bash
prtip -sS TARGET              # Uses default: -p 1-1000
prtip -sS -p 80,443 TARGET    # Explicit ports
```

---

### Issue 4: Service Name Not Recognized

**Problem:**
```
$ prtip -sS -p http TARGET
Error: invalid port number: http
```

**Cause:** Service name resolution not supported or name not found.

**Solution:**
Use numeric port numbers:
```bash
prtip -sS -p 80 TARGET        # Use numeric port instead
```

**Workaround:**
Check `/etc/services` for service-to-port mappings:
```bash
grep http /etc/services
```

---

### Issue 5: Scan Takes Too Long

**Problem:** Full port scan (`-p-`) taking hours.

**Cause:** Scanning all 65,535 ports with default timing.

**Solutions:**

**1. Use faster timing:**
```bash
sudo prtip -sS -p- -T4 TARGET
```

**2. Use top ports instead:**
```bash
sudo prtip -F TARGET                    # Top 100 (fast)
sudo prtip --top-ports 1000 TARGET      # Top 1000 (thorough)
```

**3. Scan in parallel port groups:**
```bash
# Split into 3 ranges
sudo prtip -sS -p 1-20000 TARGET &
sudo prtip -sS -p 20001-40000 TARGET &
sudo prtip -sS -p 40001-65535 TARGET &
wait
```

**4. Increase parallelism:**
```bash
sudo prtip -sS -p- -T5 --max-parallelism 10000 TARGET
```

---

### Issue 6: Permission Denied for Low Ports

**Problem:**
```
$ prtip -sS -p 80 TARGET
Error: Permission denied (raw sockets require root)
```

**Cause:** SYN scan (`-sS`) requires raw socket access (root privileges).

**Solutions:**

**1. Use sudo:**
```bash
sudo prtip -sS -p 80 TARGET
```

**2. Use Connect scan (no root):**
```bash
prtip -sT -p 80 TARGET
```

**3. Grant CAP_NET_RAW capability:**
```bash
sudo setcap cap_net_raw+ep /usr/local/bin/prtip
prtip -sS -p 80 TARGET  # Now works without sudo
```

---

## Best Practices

### 1. Start Fast, Go Deep

Use progressive scanning approach:

```bash
# Step 1: Quick reconnaissance (top 100)
prtip -F TARGET -oN quick.txt

# Step 2: Review results
cat quick.txt

# Step 3: Targeted full scan if critical
prtip -sS -p- -T4 TARGET -oN full.txt
```

**Rationale:**
- `-F` provides 90% service coverage in seconds
- Avoid wasting time on full scans for uninteresting hosts
- Full scan (`-p-`) only for confirmed targets

### 2. Save Results

Always save scan results for later analysis:

```bash
# Single format
prtip -sS -p- TARGET -oN results.txt

# All formats (recommended)
prtip -sS -p- TARGET -oA results_$(date +%Y%m%d_%H%M%S)
```

**Formats:**
- `-oN`: Normal text (human-readable)
- `-oJ`: JSON (machine-parseable)
- `-oX`: XML (Nmap-compatible)
- `-oG`: Greppable (easy parsing)
- `-oA`: All formats (recommended)

### 3. Use Appropriate Timing

Match timing template to environment:

| Environment | Recommended Timing | Rationale |
|-------------|-------------------|-----------|
| Localhost | T4 Aggressive | Maximum speed, zero latency |
| Local LAN | T4 Aggressive | Fast, reliable network |
| Production LAN | T2 Polite | Avoid network stress |
| Internet | T3 Normal | Balance speed and reliability |
| IDS evasion | T1 Sneaky | Minimize detection risk |

### 4. Limit Scope for Large Networks

Avoid full port scans on large networks:

```bash
# Bad: Full scan on /16 (could take weeks)
prtip -sS -p- 10.0.0.0/16

# Good: Top 100 ports on /16 (manageable)
prtip -sS -F 10.0.0.0/16

# Better: Progressive approach
prtip -sS -F 10.0.0.0/16 -oN quick.txt
# Review quick.txt, identify interesting /24 subnets
prtip -sS -p- -T4 10.0.1.0/24 -oN deep.txt
```

### 5. Document Port Selection

Explain why specific ports were chosen:

```bash
# Document in scan output
prtip -sS -p 80,443,8080 TARGET -oN web_scan.txt
echo "Rationale: Web service audit (HTTP/HTTPS/Alt-HTTP)" >> web_scan.txt

# Or in script comments
#!/bin/bash
# Scan database ports for security audit
# Ports: 3306 (MySQL), 5432 (PostgreSQL), 1433 (MS-SQL)
prtip -sS -sV -p 3306,5432,1433 TARGET
```

---

## See Also

**Related Reference Documentation:**
- **[Command Reference](./command-reference.md)** - Complete CLI flag reference
- **[Timing Templates](./timing-templates.md)** - Scan speed optimization (T0-T5)
- **[Output Formats](./output-formats.md)** - Result storage and parsing

**User Guides:**
- **[Basic Usage](../user-guide/basic-usage.md)** - Getting started with ProRT-IP
- **[Scan Types](../user-guide/scan-types.md)** - TCP SYN, Connect, UDP, stealth scans
- **[Advanced Usage](../user-guide/advanced-usage.md)** - Complex scanning scenarios

**Feature Guides:**
- **[Service Detection](../features/service-detection.md)** - Identifying services on open ports
- **[OS Fingerprinting](../features/os-fingerprinting.md)** - Operating system detection

**Technical Documentation:**
- **[Architecture](../00-ARCHITECTURE.md)** - System design and components
- **[Implementation Guide](../04-IMPLEMENTATION-GUIDE.md)** - Internal port handling

**External Resources:**
- **IANA Port Registry:** [https://www.iana.org/assignments/service-names-port-numbers](https://www.iana.org/assignments/service-names-port-numbers)
- **Nmap Port Scanning Guide:** [https://nmap.org/book/man-port-specification.html](https://nmap.org/book/man-port-specification.html)
- **RFC 6335:** Service Name and Port Number Procedures

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
**Related Files:** `crates/prtip-core/src/types.rs` (PortRange), `crates/prtip-core/src/top_ports.rs` (TOP_100_PORTS/TOP_1000_PORTS)
