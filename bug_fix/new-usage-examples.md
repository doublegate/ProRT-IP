## Usage Examples

### Basic Scanning

```bash
# Scan common ports on single host
prtip --scan-type connect -p 80,443,8080 192.168.1.1

# Scan subnet (CIDR notation)
prtip --scan-type connect -p 1-1000 192.168.1.0/24

# Full port range (65535 ports in ~190ms on localhost!)
prtip --scan-type connect -p 1-65535 192.168.1.1
```

### Scan Types

```bash
# TCP Connect (no privileges required)
prtip --scan-type connect -p 1-1000 192.168.1.1

# SYN scan (stealth, requires root/CAP_NET_RAW)
prtip --scan-type syn -p 1-1000 192.168.1.1

# UDP scan (protocol-specific payloads: DNS, SNMP, NTP, etc.)
prtip --scan-type udp -p 53,161,123 192.168.1.1

# Stealth scans
prtip --scan-type fin -p 1-1000 192.168.1.1     # FIN scan
prtip --scan-type null -p 1-1000 192.168.1.1    # NULL scan (no flags)
prtip --scan-type xmas -p 1-1000 192.168.1.1    # Xmas scan (FIN+PSH+URG)
prtip --scan-type ack -p 1-1000 192.168.1.1     # ACK scan (firewall detection)
```

### Detection Features

```bash
# Service version detection
prtip --scan-type connect -p 1-1000 --sV 192.168.1.1

# Adjust detection intensity (0=light, 9=aggressive)
prtip --scan-type connect -p 22,80,443 --sV --version-intensity 9 192.168.1.1

# Banner grabbing
prtip --scan-type connect -p 22,80,443 --banner-grab 192.168.1.1

# Service detection + banner grabbing
prtip --scan-type connect -p 1-1000 --sV --banner-grab 192.168.1.1
```

### Timing & Performance

```bash
# Timing templates (T0-T5)
prtip --scan-type connect -p 1-1000 -T 0 192.168.1.1  # Paranoid (5min delays)
prtip --scan-type connect -p 1-1000 -T 2 192.168.1.1  # Polite (0.4s delays)
prtip --scan-type connect -p 1-1000 -T 3 192.168.1.1  # Normal (default)
prtip --scan-type connect -p 1-1000 -T 4 192.168.1.1  # Aggressive (fast)
prtip --scan-type connect -p 1-1000 -T 5 192.168.1.1  # Insane (maximum speed)

# Adaptive parallelism (automatic: 20 for small, 1000 for large scans)
prtip --scan-type connect -p 1-65535 192.168.1.1

# Manual parallelism override
prtip --scan-type connect -p 1-1000 --max-concurrent 500 192.168.1.1
```

### Storage & Output

```bash
# In-memory mode (default, fastest - 39ms for 10K ports)
prtip --scan-type connect -p 1-10000 192.168.1.1

# Database storage (async writes - 75ms for 10K ports)
prtip --scan-type connect -p 1-10000 --with-db 192.168.1.1

# Output formats
prtip --scan-type connect -p 1-1000 --output-format json 192.168.1.1 > results.json
prtip --scan-type connect -p 1-1000 --output-format xml 192.168.1.1 > results.xml
```

### Real-World Scenarios

```bash
# Web server reconnaissance
prtip --scan-type connect -p 80,443,8080,8443 --sV --banner-grab example.com

# Network inventory audit
prtip --scan-type connect -p 22,80,443,3389 --with-db --sV 192.168.0.0/16

# Quick security assessment
prtip --scan-type syn -p 1-65535 -T 4 --sV 192.168.1.1

# Stealth reconnaissance
prtip --scan-type syn -p 1-1000 -T 0 192.168.1.1
```

### Performance Benchmarks

```bash
# Localhost performance (CachyOS Linux, i9-10850K)
$ time prtip --scan-type connect -p 1-1000 127.0.0.1      # ~4.5ms
$ time prtip --scan-type connect -p 1-10000 127.0.0.1     # ~39ms
$ time prtip --scan-type connect -p 1-65535 127.0.0.1     # ~190ms

# With database storage
$ time prtip --scan-type connect -p 1-10000 --with-db 127.0.0.1  # ~75ms
```

---
