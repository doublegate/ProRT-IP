# ProRT-IP WarScan User Manual

**Version:** 1.0.0
**Last Updated:** 2025-01-25
**Target Audience:** Security professionals, penetration testers, network administrators, red teams

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Installation Guide](#2-installation-guide)
3. [Quick Start Guide](#3-quick-start-guide)
4. [CLI Reference](#4-cli-reference)
5. [Scan Types](#5-scan-types)
6. [Service Detection](#6-service-detection)
7. [OS Fingerprinting](#7-os-fingerprinting)
8. [Terminal User Interface (TUI)](#8-terminal-user-interface-tui)
9. [Output Formats](#9-output-formats)
10. [Evasion Techniques](#10-evasion-techniques)
11. [Troubleshooting](#11-troubleshooting)
12. [FAQ](#12-faq)
13. [Appendix](#appendix)

---

## 1. Introduction

### 1.1 What is ProRT-IP?

**ProRT-IP WarScan** (Protocol/Port Real-Time IP War Scanner) is a modern, high-performance network scanner written in Rust that combines:

- **Speed of Masscan/ZMap:** 10M+ packets/second stateless scanning
- **Depth of Nmap:** Comprehensive service detection (85-90% accuracy) and OS fingerprinting (2,600+ signatures)
- **Safety of Rust:** Memory-safe implementation preventing entire vulnerability classes
- **Stealth Capabilities:** Advanced evasion techniques including timing controls, decoys, fragmentation, TTL manipulation, and idle scanning

### 1.2 Key Features

| Feature | Description |
|---------|-------------|
| **Multi-Protocol** | TCP (SYN, Connect, FIN, NULL, Xmas, ACK, Idle), UDP, ICMP/ICMPv6, NDP |
| **IPv6 Support** | Full dual-stack implementation across all 8 scanner types |
| **Service Detection** | 187 embedded Nmap probes + 5 protocol parsers (HTTP, SSH, SMB, MySQL, PostgreSQL) |
| **OS Fingerprinting** | 16-probe technique with 2,600+ signatures |
| **Evasion** | Fragmentation (-f, --mtu), TTL manipulation, bad checksums, decoys (-D), idle scan (-sI) |
| **TUI Dashboard** | 60 FPS real-time visualization with 4-tab interface |
| **Plugin System** | Lua 5.4 sandboxed scripting engine |
| **Cross-Platform** | Linux, Windows, macOS, FreeBSD |

### 1.3 System Requirements

| Platform | Minimum | Recommended |
|----------|---------|-------------|
| **CPU** | 2 cores | 8+ cores |
| **RAM** | 4 GB | 16 GB |
| **Storage** | 100 MB | 1 GB |
| **OS** | Linux 4.15+, Windows 10+, macOS 11.0+, FreeBSD 12+ | Latest LTS |
| **Rust** | 1.85+ | Latest stable |

### 1.4 Legal Notice

ProRT-IP is a security research tool intended for:
- Penetration testing (with authorization)
- Network security auditing
- Educational purposes
- Red team operations

**ALWAYS obtain proper authorization before scanning networks you do not own.**

---

## 2. Installation Guide

### 2.1 Linux Installation

#### 2.1.1 Debian/Ubuntu

```bash
# Install dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libpcap-dev libssl-dev cmake

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release

# Install system-wide
sudo cp target/release/prtip /usr/local/bin/
sudo chmod +x /usr/local/bin/prtip

# Grant capabilities (recommended over running as root)
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip

# Verify installation
prtip --version
```

#### 2.1.2 Fedora/RHEL/CentOS

```bash
# Install dependencies
sudo dnf groupinstall "Development Tools"
sudo dnf install -y libpcap-devel openssl-devel cmake

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release

# Install and configure
sudo cp target/release/prtip /usr/local/bin/
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip
```

#### 2.1.3 Arch Linux

```bash
# Install dependencies
sudo pacman -S base-devel libpcap openssl cmake

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release

# Install
sudo cp target/release/prtip /usr/local/bin/
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip
```

### 2.2 Windows Installation

#### 2.2.1 Prerequisites

1. **Visual Studio Build Tools**
   - Download from: https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++"

2. **Npcap** (Required for raw packet access)
   - Download from: https://npcap.com/
   - Install with "WinPcap API-compatible mode" enabled
   - Download SDK: https://npcap.com/dist/npcap-sdk-1.13.zip
   - Extract to `C:\npcap-sdk`

3. **Environment Variables**
   ```powershell
   setx NPCAP_SDK "C:\npcap-sdk"
   ```

#### 2.2.2 Build Steps

```powershell
# Install Rust
winget install Rustlang.Rustup

# Clone repository
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP

# Configure build (create .cargo/config.toml)
@"
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/LIBPATH:C:\\npcap-sdk\\Lib\\x64"]
"@ | Out-File -Encoding utf8 .cargo/config.toml

# Build
cargo build --release

# Install
Copy-Item target\release\prtip.exe C:\Windows\System32\
```

**Note:** Run as Administrator for raw packet access on Windows.

### 2.3 macOS Installation

#### 2.3.1 Prerequisites

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not present)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install libpcap openssl cmake
```

#### 2.3.2 Build Steps

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release

# Install
sudo cp target/release/prtip /usr/local/bin/
sudo chmod +x /usr/local/bin/prtip
```

**Note:** Raw packet scanning on macOS requires either:
- Running as root (`sudo`)
- Using ChmodBPF (part of Wireshark installation)

### 2.4 Pre-built Binaries

Download pre-built binaries from the [GitHub Releases](https://github.com/doublegate/ProRT-IP/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 (glibc) | `prtip-x86_64-unknown-linux-gnu.tar.gz` |
| Linux x86_64 (musl) | `prtip-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (glibc) | `prtip-aarch64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `prtip-x86_64-pc-windows-msvc.zip` |
| macOS Intel | `prtip-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `prtip-aarch64-apple-darwin.tar.gz` |
| FreeBSD x86_64 | `prtip-x86_64-unknown-freebsd.tar.gz` |

### 2.5 Docker Installation

```bash
# Pull the official image
docker pull doublegate/prtip:latest

# Run a scan
docker run --rm --net=host --cap-add=NET_RAW doublegate/prtip -sS -p 80,443 192.168.1.1

# Interactive mode
docker run -it --rm --net=host --cap-add=NET_RAW doublegate/prtip --tui
```

---

## 3. Quick Start Guide

### 3.1 First Scan in 5 Minutes

#### Step 1: Verify Installation

```bash
prtip --version
# ProRT-IP 1.0.0
```

#### Step 2: Basic TCP Connect Scan

```bash
# No privileges required
prtip -sT -p 80,443,22 127.0.0.1
```

#### Step 3: SYN Scan (Recommended)

```bash
# Requires privileges
prtip -sS -p 80,443,22 192.168.1.1
```

#### Step 4: Fast Scan (Top 100 Ports)

```bash
prtip -F 192.168.1.1
```

#### Step 5: Service Detection

```bash
prtip -sS -sV -p 22,80,443 192.168.1.1
```

### 3.2 Common Scan Patterns

| Goal | Command |
|------|---------|
| Quick scan | `prtip -F TARGET` |
| Full port scan | `prtip -p- TARGET` |
| Aggressive scan | `prtip -A TARGET` |
| Service detection | `prtip -sV -p 1-1000 TARGET` |
| OS detection | `prtip -O -p 1-1000 TARGET` |
| Stealth scan | `prtip -sF -T2 TARGET` |
| IPv6 scan | `prtip -6 -p 80,443 TARGET` |

### 3.3 Understanding Output

```
ProRT-IP v1.0.0 - Network Scanner
Starting scan at 2025-01-25 10:00:00

Target: 192.168.1.1
PORT     STATE    SERVICE      VERSION
22/tcp   open     ssh          OpenSSH 8.9p1 Ubuntu
80/tcp   open     http         nginx 1.24.0
443/tcp  open     https        nginx 1.24.0
3306/tcp filtered mysql        -
8080/tcp closed   http-proxy   -

OS Detection: Linux 5.x (95% confidence)

Scan complete: 5 ports in 2.3 seconds
1 host up, 3 open, 1 filtered, 1 closed
```

**Port States:**
- **open**: Port is accepting connections
- **closed**: Port is reachable but no service
- **filtered**: Firewall blocking probes
- **open|filtered**: Cannot determine (UDP/stealth scans)

---

## 4. CLI Reference

### 4.1 Basic Syntax

```
prtip [OPTIONS] [TARGET...]
```

### 4.2 Target Specification

| Format | Example | Description |
|--------|---------|-------------|
| Single IP | `192.168.1.1` | One host |
| CIDR | `192.168.1.0/24` | Network range |
| Range | `192.168.1.1-100` | IP range |
| Hostname | `example.com` | DNS resolved |
| IPv6 | `2001:db8::1` | IPv6 address |
| File | `-iL targets.txt` | List from file |

### 4.3 Port Specification

| Option | Example | Description |
|--------|---------|-------------|
| `-p PORT` | `-p 80` | Single port |
| `-p RANGE` | `-p 1-1000` | Port range |
| `-p LIST` | `-p 22,80,443` | Port list |
| `-p-` | `-p-` | All 65535 ports |
| `-F` | `-F` | Top 100 ports |
| `--top-ports N` | `--top-ports 1000` | Top N ports |

### 4.4 Scan Types

| Flag | Type | Description |
|------|------|-------------|
| `-sS` | SYN | Half-open scan (default, requires privileges) |
| `-sT` | Connect | Full TCP handshake (no privileges) |
| `-sU` | UDP | UDP service discovery |
| `-sF` | FIN | Stealth (FIN flag only) |
| `-sN` | NULL | Stealth (no flags) |
| `-sX` | Xmas | Stealth (FIN+PSH+URG) |
| `-sA` | ACK | Firewall rule mapping |
| `-sI HOST` | Idle | Anonymous via zombie |

### 4.5 Detection Options

| Flag | Description |
|------|-------------|
| `-sV` | Enable service/version detection |
| `-O` | Enable OS fingerprinting |
| `-A` | Aggressive mode (-O -sV --progress) |
| `--version-intensity 0-9` | Detection thoroughness (default: 7) |
| `--banner-grab` | Capture service banners |
| `--no-tls` | Disable TLS/SSL detection |

### 4.6 Timing and Performance

| Flag | Description |
|------|-------------|
| `-T0` | Paranoid (5 min between probes) |
| `-T1` | Sneaky (15 sec between probes) |
| `-T2` | Polite (0.4 sec between probes) |
| `-T3` | Normal (default) |
| `-T4` | Aggressive (10ms parallelism) |
| `-T5` | Insane (5ms parallelism) |
| `--max-rate N` | Maximum packets/second |
| `--max-retries N` | Probe retransmission cap |
| `--host-timeout TIME` | Per-host timeout (e.g., 30m) |
| `--adaptive-rate` | Dynamic rate limiting with ICMP monitoring |

### 4.7 Evasion Options

| Flag | Description |
|------|-------------|
| `-f` | Fragment packets (8-byte fragments) |
| `--mtu SIZE` | Custom MTU (must be multiple of 8) |
| `--ttl VALUE` | Set IP Time-To-Live (1-255) |
| `-D DECOYS` | Decoy scanning (e.g., `RND:10`) |
| `--badsum` | Send packets with bad checksums |
| `-sI ZOMBIE` | Idle scan via zombie host |
| `-g PORT` | Use specific source port |

### 4.8 Output Options

| Flag | Description |
|------|-------------|
| `-o FORMAT` | Output format (text, json, xml) |
| `-oN FILE` | Normal text output |
| `-oX FILE` | XML output (Nmap compatible) |
| `-oG FILE` | Greppable output |
| `-oA BASE` | All formats (BASE.txt, BASE.xml, BASE.gnmap) |
| `--open` | Show only open ports |
| `-v` | Verbose (-vv, -vvv for more) |
| `-q` | Quiet mode |
| `--reason` | Show port state reasons |

### 4.9 IPv6 Options

| Flag | Description |
|------|-------------|
| `-6` | IPv6 only mode |
| `-4` | IPv4 only mode |
| `--dual-stack` | Both IPv4 and IPv6 |

### 4.10 TUI Options

| Flag | Description |
|------|-------------|
| `--tui` | Launch interactive TUI dashboard |

### 4.11 Miscellaneous

| Flag | Description |
|------|-------------|
| `--iflist` | List network interfaces |
| `--interface IFACE` | Use specific interface |
| `-n` | Never do DNS resolution |
| `-Pn` | Skip host discovery |
| `--skip-cdn` | Skip CDN/cloud provider IPs |
| `--with-db` | Enable SQLite storage |
| `--packet-capture FILE` | Save to PCAPNG |
| `-y` | Skip confirmations |

---

## 5. Scan Types

### 5.1 TCP SYN Scan (-sS)

**The default and recommended scan type.**

**How it works:**
1. Send SYN packet to target port
2. Analyze response:
   - SYN/ACK = open
   - RST = closed
   - No response = filtered
3. Send RST (never complete handshake)

**Advantages:**
- Fast (stateless)
- Stealthy (no connection logged)
- Reliable results

**Usage:**
```bash
prtip -sS -p 1-1000 192.168.1.0/24
```

### 5.2 TCP Connect Scan (-sT)

**Use when you don't have raw socket privileges.**

**How it works:**
1. Complete full TCP 3-way handshake
2. Analyze connection result

**Advantages:**
- No privileges required
- Works on any system

**Disadvantages:**
- Slower (full handshake)
- More detectable (logged by services)

**Usage:**
```bash
prtip -sT -p 80,443 target.com
```

### 5.3 UDP Scan (-sU)

**For discovering UDP services.**

**How it works:**
1. Send UDP packet (protocol-specific payload if known)
2. Analyze response:
   - Response = open
   - ICMP unreachable = closed
   - No response = open|filtered

**Protocol-Specific Payloads:**
- DNS (53)
- SNMP (161)
- NetBIOS (137-139)
- NTP (123)
- SSDP (1900)
- mDNS (5353)
- RPC (111)
- IKE (500)

**Usage:**
```bash
prtip -sU -p 53,161,500 192.168.1.1
```

### 5.4 Stealth Scans (-sF, -sN, -sX)

**For bypassing simple packet filters.**

**FIN Scan (-sF):**
- Sends FIN flag only
- Closed ports respond with RST
- Open ports don't respond

**NULL Scan (-sN):**
- Sends packet with no flags
- Same response pattern as FIN

**Xmas Scan (-sX):**
- Sends FIN+PSH+URG (lights up like Christmas tree)
- Same response pattern

**Limitations:**
- Don't work on Windows/Cisco (always RST)
- Cannot distinguish open from filtered

**Usage:**
```bash
prtip -sF -p 1-1000 target.com
prtip -sN --top-ports 100 192.168.1.1
prtip -sX -T2 -p 80,443 target.com
```

### 5.5 ACK Scan (-sA)

**For mapping firewall rules.**

**How it works:**
1. Send ACK packet to port
2. Analyze response:
   - RST = unfiltered (firewall allows)
   - No response = filtered

**Note:** Does not determine open/closed, only filtered/unfiltered.

**Usage:**
```bash
prtip -sA -p 1-1000 target.com
```

### 5.6 Idle Scan (-sI)

**Ultimate stealth - completely anonymous scanning.**

**How it works:**
1. Find a "zombie" host with predictable IP ID
2. Probe zombie's IP ID
3. Send SYN to target spoofed as zombie
4. Probe zombie's IP ID again
5. If ID increased by 2 = port open (zombie received RST)
6. If ID increased by 1 = port closed

**Requirements:**
- Zombie must have sequential IP ID
- Zombie must be idle (minimal traffic)
- Zombie must respond to SYN/ACK with RST

**Good Zombie Candidates:**
- Printers
- IoT devices
- Older Windows systems (XP, 2003)
- Legacy Linux (< 4.18)

**Usage:**
```bash
# Basic idle scan
prtip -sI 192.168.1.100 -p 80,443 target.com

# With quality threshold
prtip -sI zombie.local --zombie-quality 0.9 -p 1-1000 target.com
```

---

## 6. Service Detection

### 6.1 Overview

Service detection identifies what software and version is running on open ports.

**Enable with:**
```bash
prtip -sV -p 1-1000 target.com
```

### 6.2 Detection Accuracy

| Protocol | Accuracy | Method |
|----------|----------|--------|
| HTTP | 90%+ | Headers, response analysis |
| SSH | 95%+ | Version banner |
| MySQL | 90%+ | Handshake protocol |
| PostgreSQL | 90%+ | Startup message |
| SMB | 85%+ | Negotiate protocol |
| Generic | 85%+ | Nmap probe database |

### 6.3 Detection Intensity

Control thoroughness with `--version-intensity`:

| Level | Description |
|-------|-------------|
| 0 | Light (fast, basic probes only) |
| 1-3 | Low |
| 4-6 | Medium |
| 7 | Default (balanced) |
| 8-9 | High (comprehensive, slower) |

```bash
prtip -sV --version-intensity 9 -p 80,443 target.com
```

### 6.4 TLS/SSL Detection

ProRT-IP performs TLS handshakes to detect HTTPS, SMTPS, IMAPS, etc.

**Disable with:**
```bash
prtip -sV --no-tls -p 443 target.com
```

### 6.5 Output Example

```
PORT     STATE  SERVICE     VERSION
22/tcp   open   ssh         OpenSSH 8.9p1 Ubuntu 3ubuntu0.4
80/tcp   open   http        nginx 1.24.0
443/tcp  open   https       nginx 1.24.0 (TLS 1.3)
3306/tcp open   mysql       MySQL 8.0.35
5432/tcp open   postgresql  PostgreSQL 15.4
```

---

## 7. OS Fingerprinting

### 7.1 Overview

OS fingerprinting identifies the operating system based on TCP/IP stack behavior.

**Enable with:**
```bash
prtip -O -p 1-1000 target.com
```

### 7.2 How It Works

ProRT-IP uses a 16-probe sequence:
- 6 TCP SYN probes (various options)
- 2 ICMP probes
- 1 ECN probe
- 6 unusual TCP probes
- 1 UDP probe

Responses are compared against 2,600+ signatures.

### 7.3 Requirements

For accurate OS detection:
- At least 1 open port
- At least 1 closed port
- Target must respond to probes

### 7.4 Limiting Detection

Skip hosts without open ports:
```bash
prtip -O --osscan-limit -p 1-1000 192.168.1.0/24
```

### 7.5 Output Example

```
OS Detection for 192.168.1.1:
Running: Linux 5.15.0-91-generic
OS CPE: cpe:/o:linux:linux_kernel:5.15
Accuracy: 95%
OS Details: Ubuntu 22.04 LTS (Jammy Jellyfish)
```

---

## 8. Terminal User Interface (TUI)

### 8.1 Overview

ProRT-IP includes a real-time terminal dashboard for visual scan monitoring.

**Launch with:**
```bash
prtip --tui -sS -p 1-1000 192.168.1.0/24
```

### 8.2 Dashboard Tabs

| Tab | Description |
|-----|-------------|
| **Ports** | Real-time discovered ports table |
| **Services** | Detected services with versions |
| **Metrics** | Scan progress, speed, statistics |
| **Network** | Time-series throughput graph |

### 8.3 Keyboard Navigation

| Key | Action |
|-----|--------|
| `Tab` | Switch tabs |
| `q` | Quit |
| `?` | Help |
| `Up/Down` | Scroll lists |
| `PgUp/PgDn` | Page scroll |
| `Home/End` | Jump to start/end |
| `Ctrl+B` | File browser |

### 8.4 Performance

- **Frame Rate:** 60 FPS
- **Event Throughput:** 10,000+ events/second
- **Memory:** ~50-100 MB overhead

### 8.5 Requirements

TUI requires:
- Interactive terminal (TTY)
- Terminal size: 80x24 minimum (120x40 recommended)
- Not available over SSH without `-t` flag

---

## 9. Output Formats

### 9.1 Text Output (Default)

Human-readable format:
```bash
prtip -sS -p 1-1000 -oN results.txt target.com
```

### 9.2 JSON Output

Machine-parseable:
```bash
prtip -sS -p 1-1000 -o json target.com > results.json
```

Example:
```json
{
  "scan": {
    "start_time": "2025-01-25T10:00:00Z",
    "targets": ["192.168.1.1"],
    "ports": "1-1000",
    "scan_type": "syn"
  },
  "hosts": [
    {
      "ip": "192.168.1.1",
      "status": "up",
      "ports": [
        {
          "port": 22,
          "protocol": "tcp",
          "state": "open",
          "service": "ssh",
          "version": "OpenSSH 8.9p1"
        }
      ]
    }
  ]
}
```

### 9.3 XML Output (Nmap Compatible)

For tool integration:
```bash
prtip -sS -p 1-1000 -oX results.xml target.com
```

### 9.4 Greppable Output

One host per line:
```bash
prtip -sS -p 1-1000 -oG results.gnmap target.com
```

Format:
```
Host: 192.168.1.1 () Status: Up
Host: 192.168.1.1 () Ports: 22/open/tcp//ssh//, 80/open/tcp//http//
```

### 9.5 All Formats

Generate all output formats:
```bash
prtip -sS -p 1-1000 -oA scan-results target.com
# Creates: scan-results.txt, scan-results.xml, scan-results.gnmap
```

### 9.6 PCAPNG Capture

Capture packets for analysis:
```bash
prtip -sS -p 80,443 --packet-capture capture.pcapng target.com
# Analyze with: wireshark capture.pcapng
```

### 9.7 SQLite Database

Persistent storage:
```bash
prtip -sS -p 1-1000 --with-db --database scans.db target.com

# Query results
sqlite3 scans.db "SELECT * FROM scan_results WHERE port=22"
```

---

## 10. Evasion Techniques

### 10.1 Timing Control

Slow down scans to avoid detection:
```bash
# Paranoid mode (IDS evasion)
prtip -sS -T0 -p 80,443 target.com

# Polite mode (minimize load)
prtip -sS -T2 -p 1-1000 target.com
```

### 10.2 Packet Fragmentation

Split packets to bypass filters:
```bash
# 8-byte fragments (most aggressive)
prtip -sS -f -p 80,443 target.com

# Custom MTU (200-byte fragments)
prtip -sS --mtu 200 -p 80,443 target.com
```

### 10.3 TTL Manipulation

Test intermediate devices:
```bash
# Low TTL to discover firewalls
prtip -sS --ttl 32 -p 80,443 target.com
```

### 10.4 Decoy Scanning

Hide among fake sources:
```bash
# 10 random decoys
prtip -sS -D RND:10 -p 80,443 target.com

# Specific decoys (ME = your IP position)
prtip -sS -D 10.0.0.1,ME,10.0.0.2 -p 80,443 target.com
```

### 10.5 Source Port Spoofing

Use trusted source port:
```bash
# Use DNS port (often allowed through firewalls)
prtip -sS -g 53 -p 80,443 target.com
```

### 10.6 Idle Scan

Complete anonymity:
```bash
prtip -sI 192.168.1.100 -p 80,443 target.com
```

### 10.7 Bad Checksums

Test firewall checksum validation:
```bash
prtip -sS --badsum -p 80,443 target.com
```

---

## 11. Troubleshooting

### 11.1 Permission Denied

**Problem:** `Error: Permission denied (raw socket)`

**Solutions:**
```bash
# Option 1: Run as root
sudo prtip -sS -p 80 target.com

# Option 2: Grant capabilities (Linux)
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip

# Option 3: Use TCP Connect scan (no privileges)
prtip -sT -p 80 target.com
```

### 11.2 No Network Interface Found

**Problem:** `Error: No suitable network interface`

**Solutions:**
```bash
# List interfaces
prtip --iflist

# Specify interface
prtip -sS --interface eth0 -p 80 target.com
```

### 11.3 Scan Timeouts

**Problem:** All ports showing as filtered

**Solutions:**
```bash
# Increase timeout
prtip -sS --timeout 5000 -p 80 target.com

# Skip host discovery
prtip -sS -Pn -p 80 target.com

# Try different scan type
prtip -sT -p 80 target.com
```

### 11.4 Windows Npcap Issues

**Problem:** `Error: Cannot find Npcap`

**Solutions:**
1. Install Npcap from https://npcap.com/
2. Enable "WinPcap API-compatible mode"
3. Run as Administrator
4. Check PATH includes Npcap DLLs

### 11.5 macOS Raw Socket Issues

**Problem:** `Error: Cannot open BPF device`

**Solutions:**
```bash
# Option 1: Run as root
sudo prtip -sS -p 80 target.com

# Option 2: Install ChmodBPF (from Wireshark)
brew install wireshark
# Grant BPF access to your user
```

### 11.6 TUI Not Starting

**Problem:** `Error: Not a TTY`

**Solutions:**
```bash
# Over SSH, use -t flag
ssh -t user@host "prtip --tui -sS -p 80 target.com"

# Ensure terminal is interactive
tty  # Should show /dev/tty or /dev/pts/*
```

---

## 12. FAQ

### General

**Q: Is ProRT-IP legal to use?**
A: ProRT-IP is a legal security tool. However, scanning networks without authorization may be illegal. Always obtain proper permission before scanning.

**Q: How does ProRT-IP compare to Nmap?**
A: ProRT-IP is 3-48x faster for most operations while maintaining similar accuracy. It's written in Rust for memory safety and includes modern features like TUI dashboard.

**Q: What platforms are supported?**
A: Linux, Windows, macOS, and FreeBSD are fully supported with pre-built binaries.

### Scanning

**Q: Why are all ports showing as filtered?**
A: This usually means:
1. Firewall is blocking probes
2. Target is down
3. Network issue between you and target

Try: `prtip -sT -Pn -p 80 target.com`

**Q: How do I scan faster?**
A: Use aggressive timing and higher rate limits:
```bash
prtip -sS -T5 --max-rate 100000 -p- target.com
```

**Q: How do I scan slower (stealthier)?**
A: Use paranoid timing:
```bash
prtip -sS -T0 --max-rate 100 -p 80,443 target.com
```

### Detection

**Q: Why isn't OS detection working?**
A: OS detection requires at least one open and one closed port. Ensure you're scanning enough ports:
```bash
prtip -O -p 1-10000 target.com
```

**Q: Service detection is slow. How can I speed it up?**
A: Lower the intensity:
```bash
prtip -sV --version-intensity 3 -p 1-1000 target.com
```

### Output

**Q: How do I integrate with other tools?**
A: Use XML or JSON output:
```bash
prtip -sS -oX results.xml target.com
# Import into Metasploit, Nessus, etc.
```

**Q: How do I save a packet capture?**
A: Use --packet-capture:
```bash
prtip -sS --packet-capture scan.pcapng -p 80 target.com
```

---

## Appendix

### A. Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Permission denied |
| 4 | Network error |
| 5 | Target unreachable |

### B. Environment Variables

| Variable | Description |
|----------|-------------|
| `PRTIP_CONFIG` | Path to config file |
| `PRTIP_LOG_LEVEL` | Logging level (trace, debug, info, warn, error) |
| `PRTIP_NO_COLOR` | Disable colored output |

### C. Configuration File

Create `~/.prtip/config.toml`:

```toml
[defaults]
timing = 3
output_format = "text"
verbose = 1

[network]
interface = "eth0"
max_rate = 10000

[detection]
service_detection = true
version_intensity = 7

[evasion]
fragment = false
decoys = ""
```

### D. Nmap Flag Compatibility

| Nmap Flag | ProRT-IP Equivalent | Notes |
|-----------|---------------------|-------|
| `-sS` | `-sS` | Identical |
| `-sT` | `-sT` | Identical |
| `-sU` | `-sU` | Identical |
| `-sF/-sN/-sX` | `-sF/-sN/-sX` | Identical |
| `-sA` | `-sA` | Identical |
| `-sI` | `-sI` | Identical |
| `-O` | `-O` | Identical |
| `-sV` | `-sV` | Identical |
| `-A` | `-A` | Identical |
| `-T0-5` | `-T0-5` | Identical |
| `-p` | `-p` | Identical |
| `-F` | `-F` | Identical |
| `-oN/-oX/-oG/-oA` | Same flags | Identical |
| `-f` | `-f` | Identical |
| `--mtu` | `--mtu` | Identical |
| `-D` | `-D` | Identical |
| `-g` | `-g` | Identical |
| `-n` | `-n` | Identical |
| `-Pn` | `-Pn` (via `--skip-ping`) | Different flag |
| `-6` | `-6` | Identical |
| `-v` | `-v` | Identical |

### E. Resources

- **GitHub Repository:** https://github.com/doublegate/ProRT-IP
- **Documentation:** https://github.com/doublegate/ProRT-IP/tree/main/docs
- **Issues:** https://github.com/doublegate/ProRT-IP/issues
- **Discussions:** https://github.com/doublegate/ProRT-IP/discussions

---

**ProRT-IP WarScan v1.0.0** - Modern network scanning at Masscan speed with Nmap detection depth.

Copyright 2025 ProRT-IP Contributors. Licensed under GPL-3.0.
