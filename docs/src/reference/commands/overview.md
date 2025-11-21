# Custom Commands Overview

This document provides an overview of ProRT-IP's CLI commands and usage patterns.

## Binary Name

The ProRT-IP command-line tool is invoked as `prtip`:

```bash
prtip [OPTIONS] [TARGETS]
```

## Quick Reference

### Essential Commands

```bash
# Basic SYN scan
prtip -sS -p 80,443 192.168.1.1

# Fast scan (top 100 ports)
prtip -F 192.168.1.0/24

# Full port scan with service detection
prtip -sS -sV -p- target.com

# Aggressive scan (OS + services)
prtip -A target.com
```

### Scan Types Summary

| Type | Flag | Use Case |
|------|------|----------|
| SYN | `-sS` | Default, fast, stealthy |
| Connect | `-sT` | No root required |
| UDP | `-sU` | UDP services |
| FIN | `-sF` | Firewall evasion |
| NULL | `-sN` | Firewall evasion |
| Xmas | `-sX` | Firewall evasion |
| ACK | `-sA` | Firewall mapping |
| Idle | `-sI` | Anonymous scanning |

## Command Categories

### Discovery Commands

```bash
# Ping sweep (host discovery)
prtip -sn 192.168.1.0/24

# Skip host discovery
prtip -Pn -p 80 target.com

# ARP discovery (local network)
prtip -PR 192.168.1.0/24
```

### Port Scanning Commands

```bash
# Single port
prtip -p 22 target.com

# Port range
prtip -p 1-1000 target.com

# Common ports
prtip -p 21,22,23,25,80,443 target.com

# All ports
prtip -p- target.com

# Top N ports
prtip --top-ports 1000 target.com
```

### Service Detection Commands

```bash
# Basic version detection
prtip -sV target.com

# Aggressive version detection
prtip -sV --version-intensity 9 target.com

# Light version detection
prtip -sV --version-light target.com
```

### Output Commands

```bash
# Normal output
prtip -oN scan.txt target.com

# XML output
prtip -oX scan.xml target.com

# JSON output
prtip -oJ scan.json target.com

# Greppable output
prtip -oG scan.grep target.com

# All formats
prtip -oA scan target.com

# PCAPNG capture
prtip -oP capture.pcapng target.com
```

### Performance Commands

```bash
# Maximum speed
prtip -T5 --max-rate 100000 target.com

# Polite scanning
prtip -T2 --max-rate 100 target.com

# Adaptive batching
prtip --adaptive-batch --min-batch-size 16 target.com
```

### Evasion Commands

```bash
# Packet fragmentation
prtip -sS -f target.com

# Custom MTU
prtip -sS --mtu 24 target.com

# Decoy scanning
prtip -sS -D 10.0.0.1,10.0.0.2,ME target.com

# Source port spoofing
prtip -sS -g 53 target.com

# TTL manipulation
prtip -sS --ttl 128 target.com
```

### CDN Filtering Commands

```bash
# Skip CDN IPs
prtip -sS --skip-cdn target.com

# Only scan CDN IPs
prtip -sS --cdn-whitelist target.com

# Exclude specific CDNs
prtip -sS --cdn-blacklist cloudflare,akamai target.com
```

## TUI Mode

Launch the interactive terminal user interface:

```bash
# Start TUI with scan
prtip --tui -sS target.com

# TUI with specific ports
prtip --tui -p 1-1000 target.com
```

## Help and Version

```bash
# Show help
prtip --help
prtip -h

# Show version
prtip --version
prtip -V

# Show specific help
prtip -sS --help
```

## Configuration Files

ProRT-IP supports configuration files:

```bash
# Use config file
prtip --config ~/.prtip/config.toml target.com

# Generate default config
prtip --generate-config > config.toml
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `PRTIP_CONFIG` | Default config path |
| `PRTIP_DISABLE_HISTORY` | Disable scan history |
| `NO_COLOR` | Disable colored output |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Permission denied |
| 4 | Network error |

## See Also

- [Command Analysis](./analysis.md)
- [Quick Start Guide](../../getting-started/quick-start.md)
- [Tutorials](../../getting-started/tutorials.md)

