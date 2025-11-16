# CLI Reference

Complete command-line interface reference for ProRT-IP.

## Synopsis

```bash
prtip [OPTIONS] <target>...
```

## Target Specification

### IP Addresses

```bash
prtip 192.168.1.1                    # Single IP
prtip 192.168.1.1 192.168.1.10       # Multiple IPs
prtip 192.168.1.0/24                 # CIDR notation
prtip 10.0.0.0/8                     # Large subnet
```

### IPv6 Addresses

```bash
prtip 2001:db8::1                    # IPv6 literal
prtip 2001:db8::/64                  # IPv6 CIDR
prtip -6 example.com                 # Force IPv6 resolution
```

### Hostnames

```bash
prtip example.com                    # Single hostname
prtip example.com target.local       # Multiple hostnames
```

## Port Specification

### Basic Port Syntax

```bash
-p, --ports <PORTS>                  # Specify ports to scan
```

**Examples:**

```bash
prtip -p 80 target.com               # Single port
prtip -p 80,443,8080 target.com      # Port list
prtip -p 1-1000 target.com           # Port range
prtip -p 22,80,443,8000-9000 target.com  # Mixed
prtip -p- target.com                 # All 65535 ports
```

### Top Ports

```bash
-F                                   # Fast scan (top 100 ports)
--top-ports <N>                      # Scan top N ports
```

**Examples:**

```bash
prtip -F target.com                  # Top 100 ports
prtip --top-ports 1000 target.com    # Top 1000 ports
```

## Scan Types

### TCP Scans

```bash
-sS, --scan-type syn                 # TCP SYN scan (default with sudo)
-sT, --scan-type connect             # TCP Connect scan (default)
-sF, --scan-type fin                 # TCP FIN scan
-sN, --scan-type null                # TCP NULL scan
-sX, --scan-type xmas                # TCP Xmas scan
-sA, --scan-type ack                 # TCP ACK scan (firewall detection)
```

### UDP Scans

```bash
-sU, --scan-type udp                 # UDP scan
```

### Idle Scan

```bash
-sI, --scan-type idle --zombie <IP>  # Idle/zombie scan
```

**Examples:**

```bash
sudo prtip -sS -p 80,443 target.com
prtip -sT -p 22-25 target.com
sudo prtip -sU -p 53,161 target.com
sudo prtip -sI --zombie 192.168.1.5 -p 80 target.com
```

## Detection Options

### Service Detection

```bash
-sV, --service-detection             # Enable service detection
--version-intensity <0-9>            # Detection intensity (default: 5)
```

**Examples:**

```bash
prtip -sV -p 22,80,443 target.com
prtip -sV --version-intensity 9 target.com  # Maximum intensity
```

### OS Fingerprinting

```bash
-O, --os-detect                      # Enable OS detection
```

**Example:**

```bash
sudo prtip -O target.com
```

### TLS Certificate Analysis

```bash
--tls-cert                           # Analyze TLS certificates
--sni <hostname>                     # SNI hostname for TLS
```

**Example:**

```bash
prtip --tls-cert -p 443 target.com
```

### Aggressive Mode

```bash
-A                                   # Enable all detection (-O -sV --progress)
```

**Example:**

```bash
sudo prtip -A target.com
```

## Timing Options

### Timing Templates

```bash
-T<0-5>                              # Timing template
```

**Templates:**

| Template | Name | Description |
|----------|------|-------------|
| -T0 | Paranoid | Slowest, for IDS evasion |
| -T1 | Sneaky | Slow, stealthy |
| -T2 | Polite | Minimal bandwidth |
| -T3 | Normal | Nmap default |
| -T4 | Aggressive | ProRT-IP default (fast) |
| -T5 | Insane | Maximum speed |

**Examples:**

```bash
prtip -T0 target.com                 # Paranoid mode
prtip -T4 target.com                 # Aggressive (default)
prtip -T5 target.com                 # Insane speed
```

### Performance Options

```bash
--timeout <MS>                       # Connection timeout (ms)
--max-concurrent <N>                 # Maximum concurrent connections
--host-delay <MS>                    # Delay between probes (ms)
```

**Examples:**

```bash
prtip --timeout 5000 target.com
prtip --max-concurrent 1000 target.com
prtip --host-delay 100 target.com
```

### Rate Limiting

```bash
--rate-limit <PPS>                   # Maximum packets per second
--burst <N>                          # Burst size (default: 100)
```

**Examples:**

```bash
prtip --rate-limit 1000 target.com
prtip --rate-limit 500 --burst 50 target.com
```

## Output Options

### Output Formats

```bash
-oN <FILE>                           # Normal text output
-oX <FILE>                           # XML output
-oG <FILE>                           # Greppable output
-oA <BASENAME>                       # All formats
--output <FORMAT>                    # Manual format specification
--output-file <FILE>                 # Output file path
```

**Formats:**
- `text` - Human-readable text
- `json` - JSON format
- `xml` - XML format (nmap-compatible)
- `greppable` - Greppable format

**Examples:**

```bash
prtip -oN results.txt target.com
prtip -oX results.xml target.com
prtip -oG results.gnmap target.com
prtip -oA scan-results target.com   # Creates .txt, .xml, .gnmap
prtip --output json --output-file results.json target.com
```

### Database Storage

```bash
--db <PATH>                          # SQLite database path
```

**Example:**

```bash
prtip --db scans.db target.com
```

### PCAP Output

```bash
--pcap <FILE>                        # PCAPNG packet capture
```

**Example:**

```bash
sudo prtip --pcap capture.pcapng -sS target.com
```

## Verbosity & Progress

### Verbosity Levels

```bash
-v                                   # Increase verbosity (info)
-vv                                  # More verbosity (debug)
-vvv                                 # Maximum verbosity (trace)
-q, --quiet                          # Quiet mode (errors only)
```

### Progress Display

```bash
--progress                           # Show progress bar
--live                               # Live TUI dashboard
```

**Examples:**

```bash
prtip -v -p 80,443 target.com
prtip --progress -p- target.com
prtip --live -p 1-10000 target.com/24
```

## Evasion Techniques

### Packet Fragmentation

```bash
-f                                   # Fragment packets (8-byte)
--mtu <SIZE>                         # Custom MTU size
```

### Decoy Scanning

```bash
-D, --decoys <LIST>                  # Decoy IP addresses
```

**Example:**

```bash
sudo prtip -D 192.168.1.2,192.168.1.3,ME target.com
```

### Source Port

```bash
-g, --source-port <PORT>             # Spoof source port
```

**Example:**

```bash
sudo prtip -g 53 target.com          # Use DNS source port
```

### TTL Manipulation

```bash
--ttl <VALUE>                        # Set packet TTL
```

**Example:**

```bash
sudo prtip --ttl 32 target.com
```

### Bad Checksum

```bash
--badsum                             # Send packets with invalid checksums
```

**Example:**

```bash
sudo prtip --badsum target.com
```

## Host Discovery

### Skip Ping

```bash
-Pn, --no-ping                       # Skip host discovery
```

**Example:**

```bash
prtip -Pn -p 80,443 target.com
```

## IPv6 Options

```bash
-6, --ipv6                           # Force IPv6
-4, --ipv4                           # Force IPv4
--prefer-ipv6                        # Prefer IPv6, fallback IPv4
--prefer-ipv4                        # Prefer IPv4, fallback IPv6
--ipv6-only                          # Strict IPv6 mode
--ipv4-only                          # Strict IPv4 mode
```

**Examples:**

```bash
prtip -6 example.com                 # Force IPv6
prtip --prefer-ipv6 example.com      # Prefer IPv6
prtip 2001:db8::1                    # IPv6 literal
```

## Plugin System

```bash
--plugin <PATH>                      # Load Lua plugin
--plugin-arg <KEY=VALUE>             # Plugin argument
```

**Example:**

```bash
prtip --plugin custom-banner.lua --plugin-arg verbose=true target.com
```

## Miscellaneous

### Configuration

```bash
--config <FILE>                      # Load configuration file
--template <NAME>                    # Load scan template
```

**Examples:**

```bash
prtip --config custom.toml target.com
prtip --template aggressive target.com
```

### Help & Version

```bash
-h, --help                           # Show help message
-V, --version                        # Show version
```

## Common Command Patterns

### Quick Network Scan

```bash
prtip -F 192.168.1.0/24
```

### Comprehensive Single Host

```bash
sudo prtip -A -p- target.com
```

### Stealth Scan

```bash
sudo prtip -sS -T2 --host-delay 100 -p 80,443 target.com
```

### Service Detection

```bash
prtip -sV --version-intensity 9 -p 1-10000 target.com
```

### Large-Scale Scan

```bash
sudo prtip -sS -p 80,443,8080 --rate-limit 10000 10.0.0.0/8
```

### IPv6 Network Discovery

```bash
prtip -6 -F 2001:db8::/64
```

### Database Storage

```bash
prtip -sV -p- --db scans.db --output json --output-file results.json target.com
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PRTIP_CONFIG` | Default configuration file | `~/.prtip/config.toml` |
| `PRTIP_DB` | Default database path | `~/.prtip/scans.db` |
| `PRTIP_THREADS` | Number of worker threads | CPU cores |
| `PRTIP_LOG` | Log level (error, warn, info, debug, trace) | `info` |
| `PRTIP_DISABLE_HISTORY` | Disable command history (testing) | `false` |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Permission denied (needs sudo) |
| 4 | Network error |
| 5 | Timeout |

## See Also

- [Basic Usage](./basic-usage.md) - Getting started guide
- [Scan Types](./scan-types.md) - Detailed scan type documentation
- [Output Formats](./output-formats.md) - Output format specifications
- [Timing & Performance](./timing-performance.md) - Performance tuning guide
- [Nmap Compatibility](../features/nmap-compatibility.md) - Nmap flag reference
- [Command Reference](../reference/command-reference.md) - Complete flag listing
