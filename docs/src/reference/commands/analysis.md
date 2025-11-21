# Command Analysis

This document provides a detailed analysis of ProRT-IP's command-line interface patterns and options.

## Command Structure

ProRT-IP follows nmap-compatible CLI conventions where possible:

```
prtip [SCAN TYPE] [OPTIONS] [TARGETS]
```

## Scan Type Flags

### TCP Scans

| Flag | Name | Description | Privileges |
|------|------|-------------|------------|
| `-sS` | SYN Scan | Half-open stealth scan | Root |
| `-sT` | Connect Scan | Full TCP connection | User |
| `-sF` | FIN Scan | Stealth via FIN flag | Root |
| `-sN` | NULL Scan | No flags set | Root |
| `-sX` | Xmas Scan | FIN+PSH+URG flags | Root |
| `-sA` | ACK Scan | Firewall detection | Root |
| `-sI` | Idle Scan | Anonymous via zombie | Root |

### UDP Scans

| Flag | Name | Description | Privileges |
|------|------|-------------|------------|
| `-sU` | UDP Scan | Protocol payloads | Root |

## Port Specification

| Option | Example | Description |
|--------|---------|-------------|
| `-p` | `-p 80` | Single port |
| `-p` | `-p 1-1000` | Port range |
| `-p` | `-p 80,443,8080` | Port list |
| `-p` | `-p-` | All 65535 ports |
| `-F` | `-F` | Fast (top 100) |
| `--top-ports` | `--top-ports 1000` | Most common N ports |

## Target Specification

| Format | Example | Description |
|--------|---------|-------------|
| Single IP | `192.168.1.1` | One host |
| CIDR | `192.168.1.0/24` | Network range |
| Range | `192.168.1.1-254` | IP range |
| Hostname | `example.com` | DNS resolution |
| List | `-iL targets.txt` | File input |
| IPv6 | `::1` or `2001:db8::1` | IPv6 addresses |

## Output Options

| Option | Format | Description |
|--------|--------|-------------|
| `-oN` | Normal | Human-readable text |
| `-oX` | XML | Nmap-compatible XML |
| `-oG` | Greppable | Grep-friendly format |
| `-oJ` | JSON | Structured JSON |
| `-oP` | PCAPNG | Packet capture |
| `-oA` | All | All formats at once |

## Timing Templates

| Flag | Name | Behavior |
|------|------|----------|
| `-T0` | Paranoid | 5 min between probes |
| `-T1` | Sneaky | 15 sec between probes |
| `-T2` | Polite | 400ms between probes |
| `-T3` | Normal | Default balanced |
| `-T4` | Aggressive | Faster, assumes good network |
| `-T5` | Insane | Maximum speed |

## Evasion Options

| Option | Description |
|--------|-------------|
| `-f` | Fragment packets (8 bytes) |
| `--mtu N` | Custom fragment size |
| `-D decoys` | Decoy scanning |
| `-S addr` | Spoof source address |
| `-g port` | Source port spoofing |
| `--ttl N` | Custom TTL value |
| `--badsum` | Invalid checksum |

## Service Detection

| Option | Description |
|--------|-------------|
| `-sV` | Version detection |
| `--version-intensity N` | Probe aggressiveness (0-9) |
| `-A` | Aggressive (OS + version + scripts) |

## Performance Options

| Option | Description |
|--------|-------------|
| `--min-rate N` | Minimum packets/second |
| `--max-rate N` | Maximum packets/second |
| `--adaptive-batch` | Enable adaptive batching |
| `--min-batch-size N` | Minimum batch size |
| `--max-batch-size N` | Maximum batch size |

## CDN Options

| Option | Description |
|--------|-------------|
| `--skip-cdn` | Skip all CDN IPs |
| `--cdn-whitelist` | Scan only CDN IPs |
| `--cdn-blacklist` | Exclude specific CDNs |

## Verbosity

| Option | Description |
|--------|-------------|
| `-v` | Increase verbosity |
| `-vv` | More verbose |
| `-vvv` | Debug level |
| `-q` | Quiet mode |

## Option Compatibility Matrix

| Option | SYN | Connect | UDP | Stealth | Idle |
|--------|-----|---------|-----|---------|------|
| `-sV` | Yes | Yes | Yes | No | No |
| `-f` | Yes | No | Yes | Yes | Yes |
| `-D` | Yes | No | Yes | Yes | Yes |
| `-T0-T5` | Yes | Yes | Yes | Yes | Yes |

## Common Patterns

### Quick Network Discovery
```bash
prtip -sS -F 192.168.1.0/24
```

### Full Service Scan
```bash
prtip -sS -sV -p- target.com
```

### Stealth Assessment
```bash
prtip -sS -f -D RND:5 -g 53 target.com
```

### Maximum Speed
```bash
prtip -sS -T5 --max-rate 100000 -p- target.com
```

## See Also

- [Command Overview](./overview.md)
- [CLI Specification](../../reference/cli.md)
- [Quick Start](../../getting-started/quick-start.md)

