# Command Reference

Complete reference for all ProRT-IP command-line options and flags.

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

### `<TARGET>`

**Description:** One or more targets to scan (IP addresses, CIDR ranges, hostnames, or file input).

**Formats:**

| Format | Description | Example |
|--------|-------------|---------|
| **Single IP** | IPv4 or IPv6 address | `192.168.1.1`, `2001:db8::1` |
| **CIDR** | Network range in CIDR notation | `192.168.1.0/24`, `10.0.0.0/16` |
| **IP Range** | Dash-separated range | `192.168.1.1-50` |
| **Hostname** | DNS resolvable hostname | `example.com`, `scanme.nmap.org` |
| **Multiple** | Space-separated targets | `192.168.1.1 10.0.0.1/24 example.com` |
| **File Input** | Read targets from file | `-iL targets.txt` |

**Examples:**
```bash
# Single IP
prtip 192.168.1.1

# CIDR range
prtip 192.168.1.0/24

# Multiple targets
prtip 192.168.1.1 192.168.1.2 example.com

# From file
prtip -iL targets.txt
```

**See Also:**
- [Basic Usage: Target Specification](../user-guide/basic-usage.md#target-specification)
- [IPv6 Guide](../features/ipv6.md)

---

## Port Specification

### `-p, --ports <PORTS>`

**Description:** Specify ports to scan.

**Default:** `1-1000` (first 1,000 ports)

**Formats:**

| Format | Description | Example |
|--------|-------------|---------|
| **Single Port** | Individual port | `-p 80` |
| **Port List** | Comma-separated | `-p 80,443,8080` |
| **Port Range** | Dash-separated range | `-p 1-1000`, `-p 20-25` |
| **All Ports** | Scan all 65,535 ports | `-p-` or `-p 1-65535` |
| **Service Names** | Use service names | `-p http,https,ssh` |
| **Mixed** | Combine formats | `-p 22,80,443,1000-2000` |

**Examples:**
```bash
# Specific ports
prtip -p 80,443,8080 192.168.1.1

# Port range
prtip -p 1-1000 192.168.1.1

# All ports
prtip -p- 192.168.1.1

# Service names
prtip -p http,https,ssh 192.168.1.1
```

### `--exclude-ports <PORTS>`

**Description:** Exclude specific ports from scan.

**Format:** Same as `--ports` (comma-separated, ranges)

**Example:**
```bash
# Scan ports 1-1000 except Windows file sharing ports
prtip -p 1-1000 --exclude-ports 135,139,445 192.168.1.1
```

**See Also:**
- [Port Specification Guide](./port-specification.md)
- [Basic Usage: Port Specification](../user-guide/basic-usage.md#port-specification)

---

## Scan Techniques

### `-s, --scan-type <TYPE>`

**Description:** Scan technique to use.

**Default:** `connect` (unprivileged) or `syn` (privileged)

**Options:**

| Type | Description | Privileges | Stealth | Speed |
|------|-------------|------------|---------|-------|
| `syn` | TCP SYN scan (half-open) | Root required | High | Fast |
| `connect` | TCP Connect scan (full handshake) | None | Low | Medium |
| `udp` | UDP scan | Root required | Medium | Slow |
| `fin` | TCP FIN scan (no flags) | Root required | Very High | Fast |
| `null` | TCP NULL scan (all flags off) | Root required | Very High | Fast |
| `xmas` | TCP Xmas scan (FIN+PSH+URG) | Root required | Very High | Fast |
| `ack` | TCP ACK scan (firewall mapping) | Root required | High | Fast |
| `idle` | Idle scan via zombie host | Root required | Ultimate | Slow |

**Examples:**
```bash
# TCP SYN scan (default if privileged)
sudo prtip -s syn -p 1-1000 192.168.1.1

# TCP Connect scan (default if unprivileged)
prtip -s connect -p 80,443 192.168.1.1

# UDP scan
sudo prtip -s udp -p 53,161,514 192.168.1.1

# Stealth FIN scan
sudo prtip -s fin -p 1-1000 192.168.1.1

# Idle scan (anonymous)
sudo prtip -s idle -p 80,443 --idle-zombie 192.168.1.5 192.168.1.1
```

**See Also:**
- [Scan Types Guide](../user-guide/scan-types.md)
- [Idle Scan Guide](../features/idle-scan.md)

---

## Timing and Performance

### `-T <0-5>` (Timing Template)

**Description:** Timing template for scan speed and stealth.

**Default:** `T3` (Normal)

**Templates:**

| Level | Name | Description | Use Case |
|-------|------|-------------|----------|
| `T0` | Paranoid | 5 minutes between probes | Maximum stealth, IDS evasion |
| `T1` | Sneaky | 15 seconds between probes | Slow stealth scanning |
| `T2` | Polite | 0.4 seconds between probes | Production systems |
| `T3` | Normal | Balanced speed/accuracy | Default, most use cases |
| `T4` | Aggressive | Fast local scanning | LAN scanning |
| `T5` | Insane | Maximum speed (may miss results) | Quick testing only |

**Examples:**
```bash
# Paranoid (maximum stealth)
sudo prtip -T0 -p 80,443 target.com

# Aggressive (fast local scanning)
sudo prtip -T4 -p 1-1000 192.168.1.0/24

# Normal (default, balanced)
sudo prtip -T3 -p 1-1000 target.com
```

### `--timeout <MILLISECONDS>`

**Description:** Timeout for each probe in milliseconds.

**Default:** `1000` (1 second)

**Range:** `1-3600000` (1ms to 1 hour)

**Example:**
```bash
# 5 second timeout for slow networks
prtip --timeout 5000 -p 80,443 slow-target.com
```

### `--max-rate <PACKETS_PER_SECOND>`

**Description:** Maximum packets per second to send.

**Default:** Unlimited

**Range:** `1-100000000` (1 to 100 million pps)

**Example:**
```bash
# Limit to 1000 packets/second (courtesy scan)
sudo prtip --max-rate 1000 -p 1-1000 192.168.1.0/24
```

### `--adaptive-rate`

**Description:** Enable adaptive rate limiting with ICMP error monitoring. Dynamically adjusts scan rate based on ICMP Type 3 Code 13 (admin prohibited) errors.

**Behavior:**
- Monitors ICMP "Communication Administratively Prohibited" errors
- Implements per-target exponential backoff: 1s → 2s → 4s → 8s → 16s (max)
- Reduces detection risk by adapting to network conditions

**Example:**
```bash
# Adaptive rate limiting (responds to target rate limits)
sudo prtip --adaptive-rate -p 1-1000 192.168.1.0/24
```

**See Also:** [Rate Limiting Guide](../features/rate-limiting.md)

### `--adaptive-batch`

**Description:** Enable adaptive batch sizing for sendmmsg/recvmmsg operations (Linux only). Dynamically adjusts packet batch sizes (1-1024) based on network performance.

**Behavior:**
- Increases batch size when success rate ≥95%
- Decreases batch size when success rate <85%
- Memory-aware sizing (respects available resources)

**Related Flags:**
- `--min-batch-size <1-1024>` (default: 1)
- `--max-batch-size <1-1024>` (default: 1024)

**Example:**
```bash
# Adaptive batching with custom limits
sudo prtip --adaptive-batch --min-batch-size 10 --max-batch-size 512 -p 1-1000 192.168.1.0/24
```

**See Also:** [Performance Tuning Guide](../advanced/performance-analysis.md)

### `--max-concurrent <COUNT>`

**Description:** Maximum concurrent scans (targets × ports).

**Default:** `10000`

**Range:** `1-1000000`

**Example:**
```bash
# Limit concurrency for resource-constrained systems
prtip --max-concurrent 500 -p 1-1000 192.168.1.0/24
```

### `--batch-size <SIZE>`

**Description:** Batch size for packet operations.

**Default:** `3000`

**Example:**
```bash
# Smaller batch for low-memory systems
sudo prtip --batch-size 1000 -p 1-1000 192.168.1.0/24
```

### `--numa`

**Description:** Enable NUMA (Non-Uniform Memory Access) optimization for multi-socket systems. Pins worker threads to CPU cores based on topology.

**Benefits:**
- 20-30% throughput improvement on dual-socket systems
- Reduces memory latency
- Better cache utilization

**Example:**
```bash
# Enable NUMA optimization (multi-socket servers)
sudo prtip --numa -p 1-65535 192.168.1.0/24
```

**See Also:** [Performance Tuning Guide](../advanced/performance-analysis.md#numa-optimization)

### `--max-hostgroup <SIZE>`

**Description:** Maximum number of hosts to scan in parallel (Nmap-compatible).

**Default:** `64`

**Alias:** `--max-parallelism`

**Example:**
```bash
# Scan 128 hosts in parallel
sudo prtip --max-hostgroup 128 -p 80,443 192.168.1.0/24
```

### `--min-hostgroup <SIZE>`

**Description:** Minimum number of hosts to scan in parallel (Nmap-compatible).

**Default:** `1`

**Example:**
```bash
# Maintain at least 32 hosts in parallel
sudo prtip --min-hostgroup 32 --max-hostgroup 128 -p 80,443 192.168.1.0/24
```

### `--max-retries <COUNT>`

**Description:** Maximum number of retries for each port.

**Default:** `3`

**Range:** `0-10`

**Example:**
```bash
# No retries (fast but may miss results)
prtip --max-retries 0 -p 80,443 192.168.1.1

# More retries for unreliable networks
prtip --max-retries 5 -p 80,443 slow-target.com
```

### `--host-timeout <MILLISECONDS>`

**Description:** Maximum time to wait for a single host to complete.

**Default:** `300000` (5 minutes)

**Example:**
```bash
# 10 minute timeout for slow hosts
sudo prtip --host-timeout 600000 -p 1-1000 192.168.1.0/24
```

### `--scan-delay <MILLISECONDS>`

**Description:** Delay between sending packets to the same host.

**Default:** `0` (no delay)

**Example:**
```bash
# 100ms delay between packets (polite scanning)
sudo prtip --scan-delay 100 -p 1-1000 192.168.1.1
```

### `--max-scan-delay <MILLISECONDS>`

**Description:** Maximum delay between packets (for adaptive timing).

**Default:** `1000` (1 second)

**Example:**
```bash
# Cap adaptive delay at 500ms
sudo prtip --max-scan-delay 500 -p 1-1000 192.168.1.1
```

### `--min-rate <PACKETS_PER_SECOND>`

**Description:** Minimum packets per second (ensures minimum scan speed).

**Default:** None

**Example:**
```bash
# Ensure at least 100 packets/second
sudo prtip --min-rate 100 -p 1-1000 192.168.1.0/24
```

**See Also:**
- [Timing Templates Guide](./timing-templates.md)
- [Timing & Performance Guide](../user-guide/timing-performance.md)

---

## Network Options

### `--interface <NAME>`

**Description:** Network interface to use for scanning.

**Default:** Auto-selected based on routing table

**Example:**
```bash
# Use specific interface
sudo prtip --interface eth0 -p 80,443 192.168.1.1
```

### `--source-port <PORT>`

**Description:** Source port to use for scanning (firewall evasion).

**Default:** Random ephemeral port

**Common Values:** `53` (DNS), `80` (HTTP), `443` (HTTPS)

**Example:**
```bash
# Use source port 53 (may bypass firewalls expecting DNS)
sudo prtip --source-port 53 -p 1-1000 192.168.1.1
```

### `--skip-cdn`

**Description:** Skip scanning CDN IP addresses entirely. Reduces scan time by 30-70% when targeting origin servers behind CDNs.

**Detected CDN Providers:**
- Cloudflare
- AWS CloudFront
- Azure CDN
- Akamai
- Fastly
- Google Cloud CDN

**Example:**
```bash
# Skip all CDN IPs
prtip --skip-cdn -p 80,443 example.com
```

### `--cdn-whitelist <PROVIDERS>`

**Description:** Only skip specific CDN providers (comma-separated).

**Providers:** `cloudflare`, `aws`, `azure`, `akamai`, `fastly`, `google`

**Example:**
```bash
# Only skip Cloudflare and AWS CloudFront
prtip --cdn-whitelist cloudflare,aws -p 80,443 example.com
```

### `--cdn-blacklist <PROVIDERS>`

**Description:** Never skip specific CDN providers (comma-separated).

**Example:**
```bash
# Skip all CDNs except Cloudflare
prtip --skip-cdn --cdn-blacklist cloudflare -p 80,443 example.com
```

**See Also:** [CDN Detection Guide](../features/index.md)

---

## Detection

### `-O, --os-detection`

**Description:** Enable OS fingerprinting via TCP/IP stack analysis.

**Requires:** At least one open port and one closed port for accuracy

**Accuracy:** 95% on well-known operating systems

**Example:**
```bash
# OS detection with service detection
sudo prtip -O -sV -p 1-1000 192.168.1.10
```

**See Also:** [OS Fingerprinting Guide](../features/os-fingerprinting.md)

### `--sV, --service-detection`

**Description:** Enable service version detection.

**Method:** Sends protocol-specific probes to identify software name and version

**Accuracy:** 85-90% detection rate

**Example:**
```bash
# Service detection on web ports
sudo prtip --sV -p 80,443,8080,8443 192.168.1.10
```

**See Also:** [Service Detection Guide](../features/service-detection.md)

### `--version-intensity <0-9>`

**Description:** Service detection intensity level (more probes = higher accuracy but slower).

**Default:** `7`

**Range:** `0-9`
- `0`: Light probes only (fast, less accurate)
- `9`: All probes (slow, most accurate)

**Example:**
```bash
# Maximum intensity (most accurate)
sudo prtip --sV --version-intensity 9 -p 80,443 192.168.1.10
```

### `--banner-grab`

**Description:** Enable banner grabbing for open ports (quick service identification).

**Example:**
```bash
# Banner grabbing only (faster than full service detection)
prtip --banner-grab -p 21,22,25,80,443 192.168.1.10
```

### `--probe-db <PATH>`

**Description:** Path to custom service detection probe database.

**Default:** Built-in nmap-service-probes database

**Example:**
```bash
# Use custom probe database
sudo prtip --sV --probe-db /path/to/custom-probes.txt -p 1-1000 192.168.1.10
```

**See Also:** [Service Probes Reference](./service-probes.md)

---

## Host Discovery

### `--ping-only` (alias: `-sn`)

**Description:** Host discovery only (no port scan). Determines which hosts are alive.

**Example:**
```bash
# Find live hosts on network
sudo prtip --ping-only 192.168.1.0/24
```

### `--arp-ping`

**Description:** Use ARP ping for host discovery (local network only, most reliable).

**Example:**
```bash
# ARP discovery on local network
sudo prtip --arp-ping --ping-only 192.168.1.0/24
```

### `--ps <PORTS>` (TCP SYN Ping)

**Description:** TCP SYN ping to specified ports for host discovery.

**Default Ports:** `80,443`

**Example:**
```bash
# TCP SYN ping to web ports
sudo prtip --ps 80,443 --ping-only 192.168.1.0/24
```

### `--pa <PORTS>` (TCP ACK Ping)

**Description:** TCP ACK ping to specified ports (may bypass stateless firewalls).

**Example:**
```bash
# TCP ACK ping (firewall bypass)
sudo prtip --pa 80,443 --ping-only 192.168.1.0/24
```

### `--pu <PORTS>` (UDP Ping)

**Description:** UDP ping to specified ports for host discovery.

**Default Ports:** `53,161`

**Example:**
```bash
# UDP ping to DNS and SNMP
sudo prtip --pu 53,161 --ping-only 192.168.1.0/24
```

### `--pe` (ICMP Echo Ping)

**Description:** ICMP Echo Request (traditional ping) for host discovery.

**Example:**
```bash
# ICMP echo ping
sudo prtip --pe --ping-only 192.168.1.0/24
```

### `--pp` (ICMP Timestamp Ping)

**Description:** ICMP Timestamp Request for host discovery (may bypass ICMP Echo filters).

**Example:**
```bash
# ICMP timestamp ping
sudo prtip --pp --ping-only 192.168.1.0/24
```

**See Also:** [Host Discovery Guide](../user-guide/scan-types.md#host-discovery)

---

## Output Options

### `-o, --output-format <FORMAT>`

**Description:** Output format for scan results.

**Options:**
- `text` - Human-readable text (default)
- `json` - JSON format (machine-parseable)
- `xml` - XML format (Nmap-compatible)
- `greppable` - Greppable format (one line per host)

**Example:**
```bash
# JSON output
prtip -o json -p 80,443 192.168.1.1
```

### `--output-file <PATH>`

**Description:** Write results to file.

**Example:**
```bash
# Save to file
prtip --output-file scan-results.txt -p 80,443 192.168.1.1
```

### `--with-db`

**Description:** Enable SQLite database storage for results.

**Database:** `~/.prtip/scans.db`

**Performance:** ~40-50ms overhead for 10K ports vs memory-only

**Example:**
```bash
# Store results in database
sudo prtip --with-db -p 1-1000 192.168.1.0/24
```

**See Also:** [Database Schema Reference](./database-schema.md)

### `--packet-capture <PATH>`

**Description:** Capture packets to PCAPNG file (Wireshark-compatible).

**Rotation:** Automatic 1GB file rotation

**Example:**
```bash
# Packet capture for analysis
sudo prtip --packet-capture scan.pcapng -p 80,443 192.168.1.1
```

### `-v, --verbose`

**Description:** Increase verbosity level (can be repeated: `-v`, `-vv`, `-vvv`).

**Levels:**
- `-v`: Basic progress information
- `-vv`: Detailed scan progress
- `-vvv`: Debug-level information

**Example:**
```bash
# Verbose output
prtip -vv -p 80,443 192.168.1.1
```

### `-q, --quiet`

**Description:** Suppress all output except errors.

**Example:**
```bash
# Quiet mode (errors only)
prtip -q -p 80,443 192.168.1.1
```

### `--yes`

**Description:** Answer "yes" to all confirmation prompts (use with caution).

**Example:**
```bash
# Skip confirmations for internet-scale scans
sudo prtip --yes -p 80,443 0.0.0.0/0
```

### `--progress`

**Description:** Show real-time progress indicators.

**Default:** Enabled for interactive terminals

**Example:**
```bash
# Force progress display
prtip --progress -p 1-1000 192.168.1.0/24
```

### `--no-progress`

**Description:** Disable progress indicators (useful for scripting).

**Example:**
```bash
# No progress for scripting
prtip --no-progress -p 1-1000 192.168.1.0/24 > results.txt
```

### `--progress-style <STYLE>`

**Description:** Progress bar style.

**Options:** `bar`, `spinner`, `simple`, `minimal`

**Default:** `bar`

**Example:**
```bash
# Spinner style progress
prtip --progress-style spinner -p 1-1000 192.168.1.1
```

### `--stats-interval <SECONDS>`

**Description:** Interval for printing scan statistics.

**Default:** `10` seconds

**Example:**
```bash
# Print stats every 5 seconds
prtip --stats-interval 5 -p 1-1000 192.168.1.0/24
```

### `--open`

**Description:** Show only open ports in output.

**Example:**
```bash
# Display open ports only
prtip --open -p 1-1000 192.168.1.1
```

### `--reason`

**Description:** Display reason for port state (SYN-ACK, RST, timeout, etc.).

**Example:**
```bash
# Show port state reasons
prtip --reason -p 80,443 192.168.1.1
```

**See Also:** [Output Formats Guide](./output-formats.md)

---

## Nmap-Compatible Flags

ProRT-IP supports 50+ Nmap-compatible flags for familiar operation. These flags are preprocessed before argument parsing to map to ProRT-IP's native options.

### Scan Types

#### `-sS` (TCP SYN Scan)

**Description:** TCP SYN scan (half-open, stealthy, requires root).

**Equivalent:** `--scan-type syn`

**Example:**
```bash
sudo prtip -sS -p 1-1000 192.168.1.1
```

#### `-sT` (TCP Connect Scan)

**Description:** TCP Connect scan (full handshake, no root required).

**Equivalent:** `--scan-type connect`

**Example:**
```bash
prtip -sT -p 80,443 192.168.1.1
```

#### `-sU` (UDP Scan)

**Description:** UDP scan (requires root, slower than TCP).

**Equivalent:** `--scan-type udp`

**Example:**
```bash
sudo prtip -sU -p 53,161,514 192.168.1.1
```

#### `-sN` (TCP NULL Scan)

**Description:** TCP NULL scan (stealth, all flags off, requires root).

**Equivalent:** `--scan-type null`

**Example:**
```bash
sudo prtip -sN -p 1-1000 192.168.1.1
```

#### `-sF` (TCP FIN Scan)

**Description:** TCP FIN scan (stealth, FIN flag only, requires root).

**Equivalent:** `--scan-type fin`

**Example:**
```bash
sudo prtip -sF -p 1-1000 192.168.1.1
```

#### `-sX` (TCP Xmas Scan)

**Description:** TCP Xmas scan (stealth, FIN+PSH+URG flags, requires root).

**Equivalent:** `--scan-type xmas`

**Example:**
```bash
sudo prtip -sX -p 1-1000 192.168.1.1
```

#### `-sA` (TCP ACK Scan)

**Description:** TCP ACK scan (firewall rule mapping, requires root).

**Equivalent:** `--scan-type ack`

**Example:**
```bash
sudo prtip -sA -p 1-1000 192.168.1.1
```

#### `-sI <ZOMBIE>` (Idle Scan)

**Description:** Idle scan via zombie host (completely anonymous, requires root).

**Equivalent:** `--scan-type idle --idle-zombie <ZOMBIE>`

**Example:**
```bash
sudo prtip -sI 192.168.1.5 -p 80,443 192.168.1.10
```

**See Also:** [Idle Scan Guide](../features/idle-scan.md)

### Output Formats

#### `-oN <FILE>` (Normal Output)

**Description:** Normal text output to file.

**Equivalent:** `--output-format text --output-file <FILE>`

**Example:**
```bash
prtip -sS -p 80,443 192.168.1.1 -oN scan.txt
```

#### `-oX <FILE>` (XML Output)

**Description:** XML output to file (Nmap-compatible).

**Equivalent:** `--output-format xml --output-file <FILE>`

**Example:**
```bash
prtip -sS -p 80,443 192.168.1.1 -oX scan.xml
```

#### `-oG <FILE>` (Greppable Output)

**Description:** Greppable output to file (one line per host).

**Equivalent:** `--output-format greppable --output-file <FILE>`

**Example:**
```bash
prtip -sS -p 80,443 192.168.1.1 -oG scan.gnmap
```

#### `-oA <BASENAME>` (All Formats)

**Description:** Output in all formats (text, XML, greppable).

**Creates:** `<BASENAME>.txt`, `<BASENAME>.xml`, `<BASENAME>.gnmap`

**Example:**
```bash
prtip -sS -p 80,443 192.168.1.1 -oA scan-results
# Creates: scan-results.txt, scan-results.xml, scan-results.gnmap
```

### Port Specification

#### `-F` (Fast Scan)

**Description:** Fast scan (top 100 most common ports).

**Equivalent:** `--fast-scan`

**Example:**
```bash
prtip -F 192.168.1.1
```

#### `--top-ports <N>`

**Description:** Scan N most common ports.

**Example:**
```bash
prtip --top-ports 500 192.168.1.1
```

#### `-r` (No Randomize)

**Description:** Don't randomize port scan order.

**Equivalent:** `--no-randomize`

**Example:**
```bash
prtip -r -p 1-1000 192.168.1.1
```

### Detection

#### `-A` (Aggressive Scan)

**Description:** Enable OS detection, service detection, default scripts, and traceroute.

**Equivalent:** `--aggressive`

**Includes:** `-O`, `--sV`, `-sC`, `--traceroute`

**Example:**
```bash
sudo prtip -A -p 1-1000 192.168.1.1
```

#### `-Pn` (No Ping)

**Description:** Skip host discovery (treat all hosts as online).

**Equivalent:** `--skip-ping`

**Example:**
```bash
prtip -Pn -p 80,443 192.168.1.1
```

**See Also:** [Nmap Compatibility Guide](../14-NMAP-COMPATIBILITY.md)

---

## Firewall/IDS Evasion

### `-f, --fragment`

**Description:** Fragment packets into 8-byte chunks (evade packet inspection).

**Requires:** Root privileges

**Example:**
```bash
sudo prtip -f -p 80,443 192.168.1.1
```

### `--mtu <SIZE>`

**Description:** Custom MTU for packet fragmentation.

**Range:** `≥68`, multiple of 8, `≤65535`

**Example:**
```bash
# 24-byte fragments
sudo prtip --mtu 24 -p 80,443 192.168.1.1
```

### `--ttl <VALUE>`

**Description:** Set IP Time-To-Live field.

**Range:** `1-255`

**Use Case:** Evade distance-based filtering, traceroute obfuscation

**Example:**
```bash
# Set TTL to 64 (common Linux default)
sudo prtip --ttl 64 -p 80,443 192.168.1.1
```

### `-D, --decoys <DECOY_LIST>`

**Description:** Decoy scanning to hide real source IP.

**Formats:**
- `RND:<N>` - N random decoys
- `IP1,ME,IP2` - Specific decoys (ME = real source)

**Example:**
```bash
# 10 random decoys
sudo prtip -D RND:10 -p 80,443 192.168.1.1

# Specific decoys
sudo prtip -D 1.2.3.4,ME,5.6.7.8 -p 80,443 192.168.1.1
```

### `--badsum`

**Description:** Send packets with bad TCP/UDP checksums (firewall/IDS testing).

**Use Case:** Detect firewalls (real hosts drop bad checksums, firewalls may respond)

**Example:**
```bash
sudo prtip --badsum -p 80,443 192.168.1.1
```

### `-I, --idle-scan <ZOMBIE>`

**Description:** Idle scan using zombie host (completely anonymous scanning).

**Requires:** Zombie host with predictable IP ID generation

**Example:**
```bash
# Idle scan via zombie
sudo prtip -I 192.168.1.5 -p 80,443 192.168.1.10
```

### `--zombie-quality`

**Description:** Test zombie host quality for idle scanning (IP ID predictability).

**Example:**
```bash
# Test zombie quality
sudo prtip --zombie-quality 192.168.1.5
```

**See Also:**
- [Evasion Techniques Guide](../advanced/evasion-techniques.md)
- [Idle Scan Guide](../features/idle-scan.md)

---

## IPv6 Options

### `-6, --ipv6`

**Description:** Enable IPv6 scanning only. Only accepts IPv6 targets and returns AAAA DNS records.

**Equivalent:** `--ip-version v6`

**Example:**
```bash
# IPv6-only scan
prtip -6 -p 80,443 2001:db8::1
```

### `-4, --ipv4`

**Description:** Enable IPv4 scanning only. Only accepts IPv4 targets and returns A DNS records.

**Equivalent:** `--ip-version v4`

**Example:**
```bash
# IPv4-only scan
prtip -4 -p 80,443 192.168.1.1
```

### `--dual-stack`

**Description:** Allow both IPv4 and IPv6 targets (default behavior).

**Example:**
```bash
# Dual-stack scanning
prtip --dual-stack -p 80,443 example.com
# Scans both IPv4 and IPv6 addresses of example.com
```

**Validation:**
- `-6` with IPv4 target → Error with hint to remove `-6` or use IPv6 address
- `-4` with IPv6 target → Error with hint to remove `-4` or use IPv4 address
- `--dual-stack` allows both

**See Also:** [IPv6 Guide](../features/ipv6.md)

---

## Scan Templates

### `--template <NAME>`

**Description:** Use predefined scan template.

**Built-in Templates:**
- `web-servers` - Scan common web ports (80, 443, 8080, 8443, 3000)
- `databases` - Scan database ports (3306, 5432, 1433, 27017, 6379)
- `quick` - Fast scan of top 100 ports
- `thorough` - Comprehensive scan of all 65,535 ports
- `stealth` - Stealthy scan with evasion techniques
- `discovery` - Host discovery only (no port scan)
- `ssl-only` - SSL/TLS ports only (443, 8443, 993, 995, 465)
- `admin-panels` - Common admin panel ports (8080, 8443, 8888, 9090)
- `mail-servers` - Email server ports (25, 110, 143, 587, 993, 995)
- `file-shares` - File sharing ports (21, 22, 445, 139, 2049)

**Example:**
```bash
# Use web-servers template
prtip --template web-servers 192.168.1.0/24

# Use databases template
prtip --template databases 192.168.1.10
```

### `--list-templates`

**Description:** List all available scan templates.

**Example:**
```bash
prtip --list-templates
```

### `--show-template <NAME>`

**Description:** Show configuration for a specific template.

**Example:**
```bash
prtip --show-template web-servers
```

**Custom Templates:**
Custom templates can be defined in `~/.prtip/templates.toml`

**See Also:** [Configuration Files Reference](./config-files.md)

---

## Miscellaneous

### `--iflist`

**Description:** List available network interfaces and exit.

**Example:**
```bash
prtip --iflist
```

### `--privileged`

**Description:** Force privileged mode (use raw sockets even if unprivileged).

**Example:**
```bash
sudo prtip --privileged -p 80,443 192.168.1.1
```

### `--unprivileged`

**Description:** Force unprivileged mode (use Connect scan even if root).

**Example:**
```bash
sudo prtip --unprivileged -p 80,443 192.168.1.1
```

### `-n, --no-dns`

**Description:** Never perform DNS resolution.

**Use Case:** Faster scanning, privacy (no DNS queries)

**Example:**
```bash
prtip -n -p 80,443 192.168.1.1
```

---

## Event Logging

### `--event-log <PATH>`

**Description:** Enable event logging to SQLite database (scan progress, discoveries, errors).

**Database Schema:** 18 event types (ScanStarted, PortDiscovered, ServiceDetected, etc.)

**Example:**
```bash
# Log events to database
sudo prtip --event-log scan-events.db -p 1-1000 192.168.1.0/24
```

### `--live-results`

**Description:** Display scan results in real-time as ports are discovered (event-driven output).

**Example:**
```bash
# Real-time result display
sudo prtip --live-results -p 1-1000 192.168.1.0/24
```

**See Also:** [Event System Guide](../advanced/index.md)

---

## Examples

### Basic Scans

```bash
# Quick scan of common ports
prtip -F 192.168.1.1

# Scan specific ports
prtip -p 80,443,8080 192.168.1.1

# Scan port range
prtip -p 1-1000 192.168.1.1

# Scan all ports
prtip -p- 192.168.1.1
```

### Network Scans

```bash
# Scan entire subnet
sudo prtip -sS -p 1-1000 192.168.1.0/24

# Scan multiple targets
prtip -p 80,443 192.168.1.1 192.168.1.2 example.com

# Scan targets from file
prtip -iL targets.txt -p 80,443
```

### Service Detection

```bash
# Basic service detection
sudo prtip --sV -p 22,80,443 192.168.1.10

# Aggressive scan (OS + service + scripts)
sudo prtip -A -p 1-1000 192.168.1.10

# OS detection only
sudo prtip -O -p 1-1000 192.168.1.10
```

### Output Options

```bash
# Save to text file
prtip -p 80,443 192.168.1.1 -oN scan.txt

# Save to all formats
prtip -p 80,443 192.168.1.1 -oA scan-results

# JSON output
prtip -o json -p 80,443 192.168.1.1 > results.json
```

### Performance Tuning

```bash
# Fast local scan
sudo prtip -T4 -p 1-1000 192.168.1.0/24

# Slow stealthy scan
sudo prtip -T1 -p 80,443 target.com

# Rate limiting
sudo prtip --max-rate 1000 -p 1-1000 192.168.1.0/24

# NUMA optimization (multi-socket servers)
sudo prtip --numa -p 1-65535 192.168.1.0/24
```

### Evasion Techniques

```bash
# Packet fragmentation
sudo prtip -f -p 80,443 192.168.1.1

# Decoy scanning
sudo prtip -D RND:10 -p 80,443 192.168.1.1

# Idle scan (anonymous)
sudo prtip -sI 192.168.1.5 -p 80,443 192.168.1.10

# Custom TTL
sudo prtip --ttl 64 -p 80,443 192.168.1.1
```

### IPv6 Scanning

```bash
# IPv6 scan
prtip -6 -p 80,443 2001:db8::1

# IPv6 subnet scan
prtip -6 -p 1-1000 2001:db8::/64

# Dual-stack scan
prtip --dual-stack -p 80,443 example.com
```

---

## See Also

- [Basic Usage Guide](../user-guide/basic-usage.md) - Common command patterns and workflows
- [Scan Types Guide](../user-guide/scan-types.md) - Detailed scan technique documentation
- [Timing Templates](./timing-templates.md) - T0-T5 timing specifications
- [Output Formats](./output-formats.md) - JSON/XML/Greppable format details
- [Port Specification](./port-specification.md) - Port syntax reference
- [OS Fingerprinting](../features/os-fingerprinting.md) - OS detection details
- [Service Detection](../features/service-detection.md) - Service detection details
- [Idle Scan](../features/idle-scan.md) - Anonymous scanning technique
- [Rate Limiting](../features/rate-limiting.md) - Adaptive rate limiting
- [IPv6 Guide](../features/ipv6.md) - IPv6 scanning capabilities
- [Evasion Techniques](../advanced/evasion-techniques.md) - Firewall/IDS evasion
- [Performance Tuning](../advanced/performance-analysis.md) - Optimization techniques

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
