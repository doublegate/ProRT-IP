# Scan Templates

Scan templates provide pre-configured scanning scenarios for common use cases. Templates combine port specifications, scan types, timing settings, and detection options into reusable configurations accessible via the `--template` flag.

## Quick Start

```bash
# List all available templates
prtip --list-templates

# Use a built-in template
prtip --template web-servers 192.168.1.0/24

# Show template details before using
prtip --show-template stealth

# Override template settings with CLI flags
prtip --template stealth -T4 192.168.1.1  # Use stealth template but with T4 timing
```

## Built-in Templates

ProRT-IP includes 10 built-in templates optimized for common scanning scenarios:

### web-servers

Scan common web server ports with service and TLS certificate detection.

| Setting | Value |
|---------|-------|
| **Ports** | 80, 443, 8080, 8443, 3000, 5000, 8000, 8888 |
| **Scan Type** | SYN |
| **Service Detection** | Enabled |
| **TLS Analysis** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** Discovering web applications, identifying web frameworks, analyzing SSL/TLS configurations.

```bash
# Basic web server scan
prtip --template web-servers 10.0.0.0/24

# Web scan with verbose output
prtip --template web-servers -v 192.168.1.0/24

# Web scan with JSON output
prtip --template web-servers -oJ results.json target.com
```

### databases

Scan common database ports including MySQL, PostgreSQL, MongoDB, Redis, MSSQL, CouchDB, and Cassandra.

| Setting | Value |
|---------|-------|
| **Ports** | 3306, 5432, 27017, 6379, 1433, 5984, 9042 |
| **Scan Type** | Connect |
| **Service Detection** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** Database discovery, identifying exposed database services, inventory auditing.

```bash
# Database discovery scan
prtip --template databases 192.168.1.0/24

# Database scan with service version detection
prtip --template databases -sV db-servers.txt
```

**Port Reference:**

| Port | Service |
|------|---------|
| 3306 | MySQL |
| 5432 | PostgreSQL |
| 27017 | MongoDB |
| 6379 | Redis |
| 1433 | Microsoft SQL Server |
| 5984 | CouchDB |
| 9042 | Cassandra |

### quick

Fast scan of top 100 most common ports without service detection.

| Setting | Value |
|---------|-------|
| **Ports** | Top 100 (via -F flag) |
| **Scan Type** | SYN |
| **Service Detection** | Disabled |
| **Timing** | T4 (Aggressive) |

**Use Case:** Rapid network reconnaissance, initial host enumeration, large network sweeps.

```bash
# Quick scan of a network
prtip --template quick 10.0.0.0/8

# Quick scan with output to file
prtip --template quick -oN quick-results.txt 192.168.0.0/16
```

### thorough

Comprehensive scan of all 65,535 ports with service and OS detection.

| Setting | Value |
|---------|-------|
| **Ports** | All 65,535 (via -p-) |
| **Scan Type** | SYN |
| **Service Detection** | Enabled |
| **OS Detection** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** Complete host analysis, penetration testing, comprehensive security assessments.

```bash
# Thorough scan of a single host
prtip --template thorough target.com

# Thorough scan with all output formats
prtip --template thorough -oA full-scan target.com
```

**Warning:** Thorough scans take significantly longer. For a single host, expect 10-30 minutes depending on network conditions.

### stealth

Evasive scanning to minimize detection using FIN scan, slow timing, randomization, and packet fragmentation.

| Setting | Value |
|---------|-------|
| **Scan Type** | FIN |
| **Timing** | T1 (Sneaky) |
| **Max Rate** | 100 pps |
| **Randomization** | Enabled |
| **Fragmentation** | Enabled |

**Use Case:** Penetration testing, IDS/IPS evasion testing, stealth reconnaissance.

```bash
# Stealth scan
prtip --template stealth -p 22,80,443 target.com

# Stealth scan with decoys
prtip --template stealth -D RND:5 target.com
```

**Note:** FIN scans may not work against all systems. Windows hosts and some firewalls don't respond to FIN packets as expected per RFC 793.

### discovery

Host discovery only using ICMP ping without port scanning.

| Setting | Value |
|---------|-------|
| **Mode** | Discovery only (no port scan) |
| **Timing** | T4 (Aggressive) |

**Use Case:** Network mapping, identifying live hosts, pre-scan reconnaissance.

```bash
# Discover live hosts
prtip --template discovery 192.168.1.0/24

# Discovery with specific output
prtip --template discovery -oG live-hosts.gnmap 10.0.0.0/8
```

### ssl-only

Scan HTTPS and other TLS-enabled ports with certificate analysis.

| Setting | Value |
|---------|-------|
| **Ports** | 443, 8443, 9443, 636, 993, 995, 465 |
| **Scan Type** | SYN |
| **Service Detection** | Enabled |
| **TLS Analysis** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** SSL/TLS security assessments, certificate inventory, encryption compliance audits.

```bash
# SSL certificate scan
prtip --template ssl-only target.com

# SSL scan with verbose certificate details
prtip --template ssl-only -v --tls-details target.com
```

**Port Reference:**

| Port | Service |
|------|---------|
| 443 | HTTPS |
| 8443 | Alternative HTTPS |
| 9443 | Alternative HTTPS |
| 636 | LDAPS |
| 993 | IMAPS |
| 995 | POP3S |
| 465 | SMTPS |

### admin-panels

Scan remote administration ports including SSH, Telnet, RDP, VNC, and management interfaces.

| Setting | Value |
|---------|-------|
| **Ports** | 22, 23, 3389, 5900, 5901, 8291, 10000 |
| **Scan Type** | Connect |
| **Service Detection** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** Administrative access discovery, remote management auditing, attack surface assessment.

```bash
# Admin panel discovery
prtip --template admin-panels 192.168.1.0/24

# Admin scan with service version detection
prtip --template admin-panels -sV internal-servers.txt
```

**Port Reference:**

| Port | Service |
|------|---------|
| 22 | SSH |
| 23 | Telnet |
| 3389 | RDP (Remote Desktop) |
| 5900 | VNC |
| 5901 | VNC (display :1) |
| 8291 | MikroTik WinBox |
| 10000 | Webmin |

### mail-servers

Scan email server ports including SMTP, IMAP, POP3, and their secure variants.

| Setting | Value |
|---------|-------|
| **Ports** | 25, 110, 143, 465, 587, 993, 995 |
| **Scan Type** | Connect |
| **Service Detection** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** Email infrastructure discovery, mail server inventory, email security assessments.

```bash
# Mail server discovery
prtip --template mail-servers mx-records.txt

# Mail scan with verbose output
prtip --template mail-servers -v mail.example.com
```

**Port Reference:**

| Port | Service |
|------|---------|
| 25 | SMTP |
| 110 | POP3 |
| 143 | IMAP |
| 465 | SMTPS |
| 587 | Submission |
| 993 | IMAPS |
| 995 | POP3S |

### file-shares

Scan file sharing protocols including FTP, SFTP, SMB, NFS, and rsync.

| Setting | Value |
|---------|-------|
| **Ports** | 21, 22, 139, 445, 2049, 873 |
| **Scan Type** | Connect |
| **Service Detection** | Enabled |
| **Timing** | T3 (Normal) |

**Use Case:** File share discovery, network storage auditing, data exfiltration risk assessment.

```bash
# File share discovery
prtip --template file-shares 192.168.1.0/24

# File share scan with greppable output
prtip --template file-shares -oG shares.gnmap internal-network.txt
```

**Port Reference:**

| Port | Service |
|------|---------|
| 21 | FTP |
| 22 | SFTP (SSH) |
| 139 | NetBIOS Session Service |
| 445 | SMB (Direct) |
| 2049 | NFS |
| 873 | rsync |

## Custom Templates

Create custom templates in `~/.prtip/templates.toml` to define reusable scanning configurations tailored to your environment.

### Template Configuration Format

```toml
[my-template-name]
description = "Human-readable description of the template"
ports = [80, 443, 8080]           # Optional: specific ports to scan
scan_type = "SYN"                 # Optional: SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle
service_detection = true          # Optional: enable service detection
os_detection = false              # Optional: enable OS fingerprinting
timing = "T3"                     # Optional: T0-T5 timing template
max_rate = 1000                   # Optional: maximum packets per second
randomize = false                 # Optional: randomize port/target order
fragment = false                  # Optional: enable packet fragmentation
tls_analysis = false              # Optional: enable TLS certificate analysis
discovery_only = false            # Optional: host discovery only (no port scan)
```

### Example Custom Templates

```toml
# ~/.prtip/templates.toml

# Internal network scan with company-specific ports
[internal-services]
description = "Scan internal company services"
ports = [80, 443, 8080, 8443, 9200, 9300, 5601, 3000, 8081, 8082]
scan_type = "Connect"
service_detection = true
timing = "T4"

# IoT device discovery
[iot-devices]
description = "Scan common IoT and embedded device ports"
ports = [80, 443, 23, 8080, 8443, 554, 1883, 8883, 5683]
scan_type = "SYN"
service_detection = true
timing = "T3"
tls_analysis = true

# High-speed reconnaissance
[speed-scan]
description = "Maximum speed network sweep"
scan_type = "SYN"
service_detection = false
timing = "T5"
max_rate = 100000

# Ultra-stealth assessment
[ultra-stealth]
description = "Minimal footprint stealth scan"
scan_type = "FIN"
timing = "T0"
max_rate = 10
randomize = true
fragment = true
```

### Using Custom Templates

```bash
# Use custom template
prtip --template internal-services 10.0.0.0/8

# List all templates (built-in + custom)
prtip --list-templates

# Show custom template details
prtip --show-template internal-services
```

### Template Inheritance

Custom templates with the same name as built-in templates override the built-in version:

```toml
# Override the built-in web-servers template
[web-servers]
description = "Custom web servers scan with additional ports"
ports = [80, 443, 8080, 8443, 3000, 5000, 8000, 8888, 9000, 9443]
scan_type = "SYN"
service_detection = true
tls_analysis = true
timing = "T4"  # Faster than default T3
```

## Template Validation

Templates are validated when loaded. Invalid configurations will produce clear error messages:

```bash
$ prtip --template invalid-template
Error: Invalid custom template 'invalid-template' in ~/.prtip/templates.toml
  Caused by: Invalid scan_type 'INVALID': must be one of SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle
```

### Validation Rules

| Field | Constraints |
|-------|-------------|
| `ports` | Must be 1-65535 (port 0 is invalid) |
| `scan_type` | Must be: SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle |
| `timing` | Must be: T0, T1, T2, T3, T4, T5 |
| `max_rate` | Must be 1-100,000,000 pps |

### Testing Templates

Validate your custom templates before using them:

```bash
# Show template details (validates the template)
prtip --show-template my-custom-template

# Dry run with verbose output
prtip --template my-custom-template -v --dry-run 192.168.1.1
```

## Template Override Behavior

CLI flags override template settings. This allows fine-tuning template behavior:

```bash
# Use stealth template but with T3 timing instead of T1
prtip --template stealth -T3 target.com

# Use web-servers template but add additional ports
prtip --template web-servers -p 80,443,8080,9000,9443 target.com

# Use thorough template but with faster timing
prtip --template thorough -T4 --max-rate 10000 target.com
```

**Override Priority:** CLI flags > Custom templates > Built-in templates > Defaults

## Performance Characteristics

| Template | Ports | Approximate Time (Single Host) | Network Impact |
|----------|-------|-------------------------------|----------------|
| quick | 100 | 5-15 seconds | Low |
| web-servers | 8 | 2-5 seconds | Very Low |
| databases | 7 | 2-5 seconds | Very Low |
| ssl-only | 7 | 5-15 seconds (TLS handshake) | Low |
| admin-panels | 7 | 2-5 seconds | Very Low |
| mail-servers | 7 | 2-5 seconds | Very Low |
| file-shares | 6 | 2-5 seconds | Very Low |
| discovery | N/A | 1-3 seconds | Minimal |
| stealth | Varies | 10-60 minutes | Minimal |
| thorough | 65,535 | 10-30 minutes | Moderate |

**Note:** Times are approximate and depend on network conditions, target responsiveness, and system resources.

## CI/CD Integration

Templates integrate well with automated security pipelines:

```yaml
# GitHub Actions example
- name: Scan web servers
  run: |
    prtip --template web-servers -oJ results.json ${{ env.TARGET }}

- name: Check for critical findings
  run: |
    jq '.ports[] | select(.state == "open")' results.json
```

```bash
# Jenkins/Shell script example
#!/bin/bash
TARGETS="192.168.1.0/24"
prtip --template databases -oG databases.gnmap $TARGETS
prtip --template admin-panels -oG admin.gnmap $TARGETS
prtip --template file-shares -oG shares.gnmap $TARGETS

# Parse results
grep "open" *.gnmap > all-open-ports.txt
```

## See Also

- [Configuration Guide](./configuration.md) - Global configuration options
- [CLI Reference](../reference/command-reference.md) - Complete command-line reference
- [Basic Usage](./basic-usage.md) - Getting started with scanning
- [Timing Templates](../reference/timing-templates.md) - T0-T5 timing details
- [Evasion Techniques](../advanced/evasion-techniques.md) - Stealth scanning methods

---

**Last Updated:** 2025-11-21
**ProRT-IP Version:** v0.5.4
