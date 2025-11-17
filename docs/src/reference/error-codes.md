# Error Codes

Comprehensive error handling reference for ProRT-IP WarScan.

## Overview

**Error handling** in ProRT-IP provides clear, actionable error messages with recovery suggestions. All errors follow a consistent format with colored output, error chains, and platform-specific guidance.

**Key Features:**
- **User-Friendly Messages:** No stack traces, debug formatting, or technical jargon
- **Error Categories:** Fatal, Warning, Info, and Tip severity levels
- **Recovery Suggestions:** 90%+ error coverage with actionable advice
- **Error Chains:** Full context path showing root causes
- **Colored Output:** Red for errors, yellow for warnings, cyan for suggestions
- **Platform-Specific:** Windows vs Unix-specific guidance

**Error Format Example:**
```
🔴 Error: Scanner operation failed: Resource exhausted: file descriptors (current: 1024, limit: 1024)

Caused by:
  → I/O error: Too many open files (os error 24)

💡 Suggestion: Reduce parallelism from 1024 to 512 with --max-parallelism
```

---

## Error Categories

ProRT-IP classifies errors into 4 severity categories with distinct icons:

| Category | Icon | Description | Retriable | Exit Code |
|----------|------|-------------|-----------|-----------|
| **Fatal** | 🔴 | Scan cannot proceed | No | 1 |
| **Warning** | ⚠️ | Scan degraded but continues | Yes | 0 |
| **Info** | ℹ️ | Informational message | - | 0 |
| **Tip** | 💡 | Optimization suggestion | - | 0 |

**Fatal Errors:**
- Permission denied
- Invalid target/configuration
- No valid targets
- Parse errors

**Warning Errors (Retriable):**
- Timeout
- Rate limit exceeded
- Too many open files
- Connection refused

---

## Core Error Types

### Network Errors

#### Network Unreachable

**Error Message:**
```
🔴 Error: Network unreachable: 192.168.1.1:80
```

**Causes:**
- Network interface down
- No route to target network
- Firewall blocking all traffic
- Incorrect routing configuration

**Recovery Suggestion:**
```
💡 Suggestion: Check network connectivity and routing. Try: ping <target> or traceroute <target>
```

**Example Fix:**
```bash
# Check connectivity
ping 192.168.1.1

# Check routing
traceroute 192.168.1.1

# Verify interface
ip addr show
ip route show
```

---

#### Host Unreachable

**Error Message:**
```
⚠️ Warning: Host unreachable: 10.0.0.1
```

**Causes:**
- Target host offline
- ICMP Echo (ping) blocked
- Intermediate firewall blocking
- Invalid ARP entry

**Recovery Suggestion:**
```
💡 Suggestion: Target host may be offline or blocking ICMP. Try: --skip-ping (-Pn) to bypass host discovery
```

**Example Fix:**
```bash
# Skip ping discovery
sudo prtip -Pn -sS -p 80,443 10.0.0.1

# Or use different discovery method
sudo prtip -PS -p 80,443 10.0.0.1  # SYN discovery
```

---

#### Connection Refused

**Error Message:**
```
ℹ️ Info: Connection refused: 192.168.100.50:22
```

**Causes:**
- Port closed (no service listening)
- Firewall blocking port
- Service not running
- Expected behavior for closed ports

**Recovery Suggestion:**
```
💡 Suggestion: All ports closed or firewall blocking. This is expected behavior for closed ports
```

**Interpretation:**
- **Port state: closed** - This is normal and expected
- Not an error - indicates successful scan of closed port
- Firewall may be dropping packets (results in "filtered" state instead)

---

#### Connection Timeout

**Error Message:**
```
⚠️ Warning: Timeout scanning 10.0.0.1:443 (5s elapsed)
```

**Causes:**
- Network latency too high
- Packet loss
- Firewall silently dropping packets
- Target overloaded

**Recovery Suggestion:**
```
💡 Suggestion: Increase timeout: --timeout 5000, or use faster timing (-T3, -T4) for better retries
```

**Example Fix:**
```bash
# Increase timeout to 10 seconds
sudo prtip -sS -p 80,443 --timeout 10000 target.com

# Use faster timing with more aggressive retries
sudo prtip -sS -p 80,443 -T4 target.com

# Combine both
sudo prtip -sS -p 80,443 -T4 --timeout 10000 target.com
```

---

#### Connection Reset

**Error Message:**
```
⚠️ Warning: Connection reset by peer: 10.1.1.1:8080
```

**Causes:**
- Firewall RST injection
- Load balancer rejecting connection
- Service crash during handshake
- Anti-scanning protection (e.g., fail2ban)

**Recovery Suggestion:**
```
💡 Suggestion: Target may be filtering connections. Try slower timing (-T0, -T1) or stealth scan (-sF, -sN, -sX)
```

**Retriable:** Yes (transient error)

**Example Fix:**
```bash
# Slower timing to avoid detection
sudo prtip -sS -p 8080 -T1 10.1.1.1

# Stealth scan techniques
sudo prtip -sF -p 8080 -T0 10.1.1.1  # FIN scan
sudo prtip -sN -p 8080 -T0 10.1.1.1  # NULL scan
```

---

#### DNS Resolution Failed

**Error Message:**
```
🔴 Error: DNS resolution failed for target.example.com
```

**Causes:**
- DNS server unreachable
- Invalid hostname
- DNS timeout
- /etc/resolv.conf misconfigured

**Recovery Suggestion:**
```
💡 Suggestion: DNS resolution failed. Use IP address directly, or check DNS settings: nslookup <hostname>
```

**Example Fix:**
```bash
# Verify DNS resolution
nslookup target.example.com

# Use IP address directly
sudo prtip -sS -p 80,443 192.168.1.1

# Specify DNS server
dig @8.8.8.8 target.example.com

# Check DNS configuration
cat /etc/resolv.conf
```

---

#### Network Interface Not Found

**Error Message:**
```
🔴 Error: Network interface 'eth0' not found
```

**Causes:**
- Interface name incorrect
- Interface doesn't exist
- Permissions issue
- Interface down

**Recovery Suggestion:**
```
💡 Suggestion: Network interface not found. List available interfaces with: prtip --iflist <target>
```

**Example Fix:**
```bash
# List all interfaces
ip addr show

# List interfaces with prtip
prtip --iflist

# Use auto-detect (default)
sudo prtip -sS -p 80,443 TARGET  # Auto-selects interface

# Specify correct interface
sudo prtip -e wlan0 -sS -p 80,443 TARGET
```

---

### Permission Errors

#### Insufficient Privileges (Raw Sockets)

**Error Message:**
```
🔴 Error: Insufficient privileges for SYN scan (raw sockets require elevated privileges)
```

**Causes:**
- Not running as root/Administrator
- Missing CAP_NET_RAW capability
- Raw socket creation failed

**Recovery Suggestion (Linux/macOS):**
```
💡 Suggestion: Run with sudo, set CAP_NET_RAW: sudo setcap cap_net_raw+ep $(which prtip), or use TCP Connect (-sT)
```

**Recovery Suggestion (Windows):**
```
💡 Suggestion: Run as Administrator, or use TCP Connect scan (-sT) which doesn't require elevated privileges
```

**Example Fix:**

**Option 1: Run with elevated privileges**
```bash
# Linux/macOS
sudo prtip -sS -p 80,443 TARGET

# Windows (PowerShell as Administrator)
prtip.exe -sS -p 80,443 TARGET
```

**Option 2: Set capabilities (Linux only)**
```bash
# One-time setup
sudo setcap cap_net_raw+ep $(which prtip)

# Now can run without sudo
prtip -sS -p 80,443 TARGET
```

**Option 3: Use TCP Connect scan (no privileges required)**
```bash
# Works without sudo/Administrator
prtip -sT -p 80,443 TARGET  # Slower but doesn't need raw sockets
```

---

#### CAP_NET_RAW Required

**Error Message:**
```
🔴 Error: Raw socket creation failed: Operation not permitted
```

**Causes:**
- Missing CAP_NET_RAW capability
- Not running as root
- SELinux/AppArmor blocking

**Recovery Suggestion:**
```
💡 Suggestion: Raw sockets require elevated privileges. Use sudo or setcap, or switch to -sT (TCP Connect) scan
```

**Example Fix:**
```bash
# Check current capabilities
getcap $(which prtip)

# Grant CAP_NET_RAW
sudo setcap cap_net_raw+ep $(which prtip)

# Verify
getcap $(which prtip)
# Output: /usr/local/bin/prtip = cap_net_raw+ep

# Alternative: Run with sudo
sudo prtip -sS -p 80,443 TARGET
```

---

### Resource Limit Errors

#### Too Many Open Files

**Error Message:**
```
⚠️ Warning: Resource exhausted: file descriptors (current: 1024, limit: 1024)

Caused by:
  → I/O error: Too many open files (os error 24)
```

**Causes:**
- File descriptor limit (ulimit) too low
- High parallelism setting
- Many targets/ports being scanned simultaneously
- Leaked file descriptors (bug)

**Recovery Suggestion:**
```
💡 Suggestion: Reduce parallelism with --max-parallelism or increase system file descriptor limit: ulimit -n 10000
```

**Example Fix:**

**Option 1: Reduce parallelism**
```bash
# Reduce from default 1024 to 512
sudo prtip -sS -p 80,443 --max-parallelism 512 192.168.1.0/24

# Or reduce batch size
sudo prtip -sS -p 80,443 -b 500 192.168.1.0/24
```

**Option 2: Increase ulimit (temporary)**
```bash
# Increase limit for current shell session
ulimit -n 10000

# Verify
ulimit -n
# Output: 10000

# Now run scan
sudo prtip -sS -p 80,443 192.168.1.0/24
```

**Option 3: Increase ulimit (permanent)**
```bash
# Edit /etc/security/limits.conf
sudo vim /etc/security/limits.conf

# Add lines:
* soft nofile 10000
* hard nofile 10000

# Logout and login again, verify:
ulimit -n
```

**Retriable:** Yes (can free resources)

---

#### Memory Exhausted

**Error Message:**
```
⚠️ Warning: Resource exhausted: memory (current: 2048 MB, limit: 2048 MB)
```

**Causes:**
- Insufficient RAM
- Memory leak (bug)
- Scanning too many targets simultaneously
- Service detection on thousands of ports

**Recovery Suggestion:**
```
💡 Suggestion: Reduce memory usage: --max-parallelism 100 -b 500, or scan in smaller batches
```

**Example Fix:**
```bash
# Reduce parallelism and batch size
sudo prtip -sS -p 80,443 --max-parallelism 100 -b 500 10.0.0.0/16

# Scan in smaller CIDR blocks
sudo prtip -sS -p 80,443 10.0.0.0/24   # /24 instead of /16
sudo prtip -sS -p 80,443 10.0.1.0/24
...

# Disable memory-intensive features
sudo prtip -sS -p 80,443 10.0.0.0/16  # No -sV (service detection)
```

**Retriable:** Yes

---

#### File Descriptor Limit (ulimit)

**Error Message:**
```
⚠️ Warning: File descriptor limit reached (ulimit: 1024)
```

**Recovery Suggestion:**
```
💡 Suggestion: Increase file descriptor limit with --ulimit 10000, or reduce batch size with -b 500
```

**Example Fix:**
```bash
# Specify ulimit directly via ProRT-IP
sudo prtip -sS -p 80,443 --ulimit 10000 192.168.1.0/24

# Or use batch size reduction
sudo prtip -sS -p 80,443 -b 500 192.168.1.0/24
```

---

### Input Validation Errors

#### Invalid Target Format

**Error Message:**
```
🔴 Error: Invalid target: '999.999.999.999' (parse error: Invalid IP address)
```

**Causes:**
- Malformed IP address
- Invalid CIDR notation
- Hostname that can't be resolved
- Empty target specification

**Recovery Suggestion:**
```
💡 Suggestion: Invalid target format. Use: IP (192.168.1.1), CIDR (10.0.0.0/24), or hostname (example.com)
```

**Example Fix:**
```bash
# Valid formats:
sudo prtip -sS -p 80,443 192.168.1.1                # ✅ IPv4
sudo prtip -sS -p 80,443 10.0.0.0/24                # ✅ CIDR
sudo prtip -sS -p 80,443 example.com                # ✅ Hostname
sudo prtip -sS -p 80,443 2001:db8::1                # ✅ IPv6
sudo prtip -sS -p 80,443 2001:db8::/64              # ✅ IPv6 CIDR

# Invalid formats:
sudo prtip -sS -p 80,443 999.999.999.999            # ❌ Invalid IP
sudo prtip -sS -p 80,443 10.0.0.0/33                # ❌ Invalid CIDR
sudo prtip -sS -p 80,443 ""                         # ❌ Empty
```

**Non-Retriable:** Configuration error, must fix input

---

#### Invalid Port Range

**Error Message:**
```
🔴 Error: Invalid port range: '80-79' (end port 79 < start port 80)
```

**Causes:**
- Reversed port range (end < start)
- Port 0 specified (invalid)
- Port > 65535 (exceeds u16::MAX)
- Malformed port syntax

**Recovery Suggestion:**
```
💡 Suggestion: Port must be 1-65535. Use: single (80), range (1-1000), list (80,443), or all (-p-)
```

**Example Fix:**
```bash
# Valid port specifications:
sudo prtip -sS -p 80 TARGET                  # ✅ Single port
sudo prtip -sS -p 1-1000 TARGET              # ✅ Range
sudo prtip -sS -p 80,443,8080 TARGET         # ✅ List
sudo prtip -sS -p 80-85,443,8080-8082 TARGET # ✅ Mixed
sudo prtip -sS -p- TARGET                    # ✅ All ports (1-65535)

# Invalid port specifications:
sudo prtip -sS -p 0 TARGET                   # ❌ Port 0 invalid
sudo prtip -sS -p 80-79 TARGET               # ❌ Reversed range
sudo prtip -sS -p 65536 TARGET               # ❌ Exceeds max
sudo prtip -sS -p abc TARGET                 # ❌ Non-numeric
```

**See:** [Port Specification](./port-specification.md) for complete syntax

**Non-Retriable:** Configuration error

---

#### Invalid CIDR Notation

**Error Message:**
```
🔴 Error: Invalid CIDR: '10.0.0.0/33' (prefix length must be 0-32 for IPv4)
```

**Causes:**
- CIDR prefix out of valid range (0-32 for IPv4, 0-128 for IPv6)
- Missing prefix (e.g., `10.0.0.0`)
- Invalid IP address
- Incorrect syntax

**Recovery Suggestion:**
```
💡 Suggestion: Invalid CIDR notation. Use: /24 for 255.255.255.0, /16 for 255.255.0.0, etc. (IPv4: /0-32, IPv6: /0-128)
```

**Example Fix:**
```bash
# Valid CIDR notation:
sudo prtip -sS -p 80,443 10.0.0.0/24         # ✅ /24 = 256 hosts
sudo prtip -sS -p 80,443 192.168.0.0/16      # ✅ /16 = 65,536 hosts
sudo prtip -sS -p 80,443 172.16.0.0/12       # ✅ /12 = 1,048,576 hosts
sudo prtip -sS -p 80,443 2001:db8::/32       # ✅ IPv6 /32

# Invalid CIDR notation:
sudo prtip -sS -p 80,443 10.0.0.0/33         # ❌ IPv4 max is /32
sudo prtip -sS -p 80,443 10.0.0.0            # ❌ Missing prefix
sudo prtip -sS -p 80,443 10.0.0.0/           # ❌ Missing prefix value
sudo prtip -sS -p 80,443 2001:db8::/129      # ❌ IPv6 max is /128
```

**Non-Retriable:** Configuration error

---

#### No Valid Targets

**Error Message:**
```
🔴 Error: No valid targets specified
```

**Causes:**
- Missing target argument
- All targets invalid
- Empty target file (-iL)
- Target specification filtered out all IPs

**Recovery Suggestion:**
```
💡 Suggestion: Specify at least one target: prtip [OPTIONS] <TARGET> (e.g., prtip -sS 192.168.1.1)
```

**Example Fix:**
```bash
# Correct usage:
sudo prtip -sS -p 80,443 192.168.1.1         # ✅ Target specified
sudo prtip -sS -p 80,443 -iL targets.txt     # ✅ Target file

# Missing target:
sudo prtip -sS -p 80,443                     # ❌ No target
```

**Non-Retriable:** Configuration error

---

#### Conflicting Options

**Error Message:**
```
🔴 Error: Invalid configuration: Conflicting options: --syn and --connect cannot be used together
```

**Causes:**
- Multiple mutually exclusive scan types specified
- Conflicting timing options
- Invalid option combinations

**Recovery Suggestion:**
```
💡 Suggestion: Remove conflicting options. Use either --syn (-sS) or --connect (-sT), not both
```

**Common Conflicts:**
- `-sS` and `-sT` (SYN vs Connect scan)
- `-sU` and TCP-only flags
- Multiple `-T` timing templates
- `--min-rate` and `--max-rate` with invalid ranges

**Example Fix:**
```bash
# Conflicting:
sudo prtip -sS -sT -p 80,443 TARGET          # ❌ Can't do both

# Valid:
sudo prtip -sS -p 80,443 TARGET              # ✅ SYN scan only
sudo prtip -sT -p 80,443 TARGET              # ✅ Connect scan only
```

---

### Rate Limiting Errors

#### Rate Limit Exceeded

**Error Message:**
```
⚠️ Warning: Rate limit exceeded: 150,000 pps (max: 100,000 pps)
```

**Causes:**
- Scanning too fast for network capacity
- Exceeds user-specified --max-rate
- Triggering target's rate limiting
- Network congestion

**Recovery Suggestion:**
```
💡 Suggestion: Reduce scan rate: slower timing (-T0 to -T3), or explicit --max-rate 1000
```

**Example Fix:**
```bash
# Use slower timing template
sudo prtip -sS -p 80,443 -T0 TARGET          # Paranoid (very slow)
sudo prtip -sS -p 80,443 -T1 TARGET          # Sneaky
sudo prtip -sS -p 80,443 -T2 TARGET          # Polite
sudo prtip -sS -p 80,443 -T3 TARGET          # Normal (default)

# Or explicit rate limit
sudo prtip -sS -p 80,443 --max-rate 1000 TARGET  # 1000 pps max

# Combine with other options
sudo prtip -sS -p 80,443 -T2 --max-rate 5000 TARGET
```

**Retriable:** Yes (transient constraint)

**See:** [Timing Templates](./timing-templates.md) for timing options

---

### File/Output Errors

#### Output File Already Exists

**Error Message:**
```
🔴 Error: Output file already exists: 'results.txt'
```

**Causes:**
- File from previous scan exists
- Protection against accidental overwrite
- No `--force` flag specified

**Recovery Suggestion:**
```
💡 Suggestion: File already exists. Use different filename, or add --force flag to overwrite
```

**Example Fix:**
```bash
# Use different filename
sudo prtip -sS -p 80,443 TARGET -oN results-2025-11-15.txt

# Or force overwrite
sudo prtip -sS -p 80,443 TARGET -oN results.txt --force

# Or remove existing file
rm results.txt
sudo prtip -sS -p 80,443 TARGET -oN results.txt
```

---

#### File Not Found

**Error Message:**
```
🔴 Error: File not found: '/path/to/targets.txt' (No such file or directory)
```

**Causes:**
- Incorrect path
- File doesn't exist
- Permission denied
- Typo in filename

**Recovery Suggestion:**
```
💡 Suggestion: File not found. Check path and permissions: ls -la $(dirname <path>)
```

**Example Fix:**
```bash
# Verify file exists
ls -la /path/to/targets.txt

# Check directory permissions
ls -ld /path/to/

# Use absolute path
sudo prtip -sS -p 80,443 -iL /home/user/targets.txt

# Verify path
realpath targets.txt
```

---

### Database Errors

#### SQLite Database Error

**Error Message:**
```
🔴 Error: Database error: unable to open database file
```

**Causes:**
- Disk full
- Permission denied on database file
- Corrupted database
- Invalid database path

**Recovery Suggestion:**
```
💡 Suggestion: Database error. Check file permissions and disk space. Try: rm <db_file> to recreate
```

**Example Fix:**
```bash
# Check disk space
df -h

# Check file permissions
ls -la ~/.prtip/scan.db

# Recreate database (WARNING: loses history)
rm ~/.prtip/scan.db
sudo prtip -sS -p 80,443 TARGET  # Creates new DB

# Check SQLite version
sqlite3 --version
```

---

## Exit Codes

ProRT-IP follows standard Unix exit code conventions:

| Exit Code | Meaning | Description |
|-----------|---------|-------------|
| 0 | Success | Scan completed successfully |
| 1 | Error | Fatal error occurred (see stderr) |
| 2 | Invalid usage | Invalid command-line arguments |
| 130 | SIGINT | User interrupted with Ctrl+C |

**Example:**
```bash
# Success (exit code 0)
sudo prtip -sS -p 80,443 192.168.1.1
echo $?  # Output: 0

# Error (exit code 1)
prtip -sS -p 80,443 192.168.1.1  # Missing sudo
echo $?  # Output: 1

# Interrupted (exit code 130)
sudo prtip -sS -p- 192.168.1.0/24
# Press Ctrl+C
echo $?  # Output: 130
```

---

## Error Display Format

All errors follow a consistent format with color-coded output:

### Standard Error Format

```
[Icon] Error: [Primary message]

Caused by:
  → [Intermediate cause 1]
  → [Root cause]

💡 Suggestion: [Recovery advice]
```

**Components:**

1. **Icon:** Visual indicator (🔴 Fatal, ⚠️ Warning, ℹ️ Info, 💡 Tip)
2. **Error Header:** "Error:" in red (or yellow/cyan for warnings/info)
3. **Primary Message:** User-facing description (no technical jargon)
4. **Error Chain:** Optional "Caused by:" with nested causes (→ arrows)
5. **Recovery Suggestion:** Optional actionable advice (90%+ coverage)

**Example:**
```
🔴 Error: Scanner operation failed: Resource exhausted: file descriptors (current: 1024, limit: 1024)

Caused by:
  → I/O error: Too many open files (os error 24)

💡 Suggestion: Reduce parallelism from 1024 to 512 with --max-parallelism
```

---

## Color Output

Error messages use ANSI color codes when stdout is a TTY:

| Element | Color | ANSI Code |
|---------|-------|-----------|
| Fatal errors | Red | `\x1b[31m` |
| Warnings | Yellow | `\x1b[33m` |
| Info messages | Cyan | `\x1b[36m` |
| Success messages | Green | `\x1b[32m` |
| Error chains | Bright black | `\x1b[90m` |
| Suggestions | Cyan | `\x1b[36m` |

**Disable Colors:**
```bash
# Auto-detect (colors when TTY, plain when piped)
sudo prtip -sS -p 80,443 TARGET

# Force disable colors
sudo prtip -sS -p 80,443 TARGET --no-color

# Pipe to file (auto-disables colors)
sudo prtip -sS -p 80,443 TARGET > results.txt
```

---

## Troubleshooting Guide

### "Permission denied" errors

**Symptom:** Raw socket creation fails

**Solutions:**
1. Run with `sudo` (temporary)
2. Set `CAP_NET_RAW` capability (permanent, Linux)
3. Use `-sT` TCP Connect scan (no privileges needed)

**Platform-specific:**
- **Linux:** `sudo setcap cap_net_raw+ep $(which prtip)`
- **macOS:** `sudo prtip` required (capabilities not supported)
- **Windows:** Run as Administrator

---

### "Too many open files" errors

**Symptom:** File descriptor limit reached

**Solutions:**
1. Reduce `--max-parallelism` to 100-512
2. Increase `ulimit -n 10000` (temporary)
3. Edit `/etc/security/limits.conf` (permanent)
4. Reduce batch size with `-b 500`

**Verification:**
```bash
# Check current limit
ulimit -n

# Check hard limit
ulimit -Hn

# Monitor file descriptors during scan
watch -n 1 'lsof -p $(pgrep prtip) | wc -l'
```

---

### "Network unreachable" errors

**Symptom:** Cannot reach target network

**Solutions:**
1. Verify network interface: `ip addr show`
2. Check routing table: `ip route show`
3. Test connectivity: `ping <target>`
4. Verify firewall rules: `iptables -L` (Linux)

---

### "Timeout" errors

**Symptom:** Scans timing out frequently

**Solutions:**
1. Increase timeout: `--timeout 10000` (10 seconds)
2. Use faster timing: `-T4` (more aggressive retries)
3. Reduce parallelism: `--max-parallelism 100`
4. Check network latency: `ping <target>`

---

### "Invalid target" errors

**Symptom:** Target parsing fails

**Solutions:**
1. Use valid IP: `192.168.1.1` (not `999.999.999.999`)
2. Use valid CIDR: `10.0.0.0/24` (not `/33`)
3. Check hostname resolution: `nslookup <hostname>`
4. Use IPv6 brackets: `[2001:db8::1]` for literal IPv6

---

## Best Practices

### 1. Always Check Error Messages

Error messages contain actionable recovery suggestions:

```bash
# Bad: Ignore error and retry blindly
sudo prtip -sS -p 80,443 TARGET
# Error occurs
sudo prtip -sS -p 80,443 TARGET  # Same error

# Good: Read error message and apply suggestion
sudo prtip -sS -p 80,443 TARGET
# Error: "Run with sudo or use -sT"
sudo prtip -sT -p 80,443 TARGET  # Apply suggestion
```

---

### 2. Use Verbose Output for Debugging

Enable verbose output to see detailed error context:

```bash
# Standard output (minimal)
sudo prtip -sS -p 80,443 TARGET

# Verbose output (detailed errors)
sudo prtip -v -sS -p 80,443 TARGET

# Very verbose (debug level)
sudo prtip -vv -sS -p 80,443 TARGET
```

---

### 3. Save Error Logs

Redirect stderr to file for later analysis:

```bash
# Capture both stdout and stderr
sudo prtip -sS -p 80,443 TARGET > results.txt 2> errors.txt

# Or combine
sudo prtip -sS -p 80,443 TARGET &> combined.txt

# With tee (display and save)
sudo prtip -sS -p 80,443 TARGET 2>&1 | tee scan.log
```

---

### 4. Validate Input Before Scanning

Check targets and ports before running expensive scans:

```bash
# Validate CIDR
ipcalc 10.0.0.0/24

# Verify hostname resolution
nslookup example.com

# Test single port first
sudo prtip -sS -p 80 TARGET

# Then full scan
sudo prtip -sS -p 1-65535 TARGET
```

---

### 5. Handle Errors in Scripts

Properly handle exit codes in automation:

```bash
#!/bin/bash
# Scan with error handling

if sudo prtip -sS -p 80,443 192.168.1.0/24 -oN results.txt; then
    echo "Scan completed successfully"
    # Post-process results
    ./analyze-results.sh results.txt
else
    echo "Scan failed with exit code $?" >&2
    # Notify admin
    echo "ProRT-IP scan failed" | mail -s "Scan Error" admin@example.com
    exit 1
fi
```

---

## See Also

- **[Command Reference](./command-reference.md)** - Complete CLI reference with all flags
- **[Timing Templates](./timing-templates.md)** - Timing options for performance tuning
- **[Port Specification](./port-specification.md)** - Port syntax and validation rules
- **[Troubleshooting Guide](../reference/troubleshooting.md)** - Common issues and solutions
- **[FAQ](../reference/faq.md)** - Frequently asked questions

**External Resources:**
- **errno(3) man page** - Unix error codes
- **Windows Error Codes** - Microsoft error code reference

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
