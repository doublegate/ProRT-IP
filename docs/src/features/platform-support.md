# Platform Support

ProRT-IP provides production-ready binaries for 5 major platforms, covering 95% of the user base. Experimental support is available for 4 additional platforms.

## Overview

**Production Platforms** (fully supported, thoroughly tested):
- Linux x86_64 (glibc) - Debian, Ubuntu, Fedora, RHEL, Arch
- Windows x86_64 - Windows 10+, Windows Server 2016+
- macOS Intel (x86_64) - macOS 10.13+ (High Sierra)
- macOS Apple Silicon (ARM64) - macOS 11+ (Big Sur, M1/M2/M3/M4)
- FreeBSD x86_64 - FreeBSD 12.x, 13.x, 14.x

**Experimental Platforms** (known limitations):
- Linux x86_64 (musl) - Type mismatch issues
- Linux ARM64 (glibc) - OpenSSL cross-compilation issues
- Linux ARM64 (musl) - Multiple compilation issues
- Windows ARM64 - Removed from CI (toolchain unavailable)

**Platform Coverage**: 5/9 production-ready, 95% user base

## Linux Support

### Linux x86_64 (glibc)

**Target**: `x86_64-unknown-linux-gnu`

**Supported Distributions**:
- Debian 10+ (Buster and later)
- Ubuntu 18.04+ (Bionic and later)
- Fedora 30+
- CentOS 8+, RHEL 8+
- Arch Linux (current)

**Installation**:

```bash
# Download the binary
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar xzf prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz

# Install to system path
sudo mv prtip /usr/local/bin/

# Grant capabilities (no root required for scanning)
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip
```

**Quick Verification**:

```bash
# Check version
prtip --version

# Test basic scan (no sudo needed with capabilities)
prtip -sT -p 80 scanme.nmap.org

# Test SYN scan (requires capabilities or sudo)
prtip -sS -p 80,443 scanme.nmap.org
```

**Requirements**:
- glibc 2.27+ (check: `ldd --version`)
- libpcap 1.9+
- Kernel 4.15+ (for sendmmsg/recvmmsg support)

**Installing Dependencies**:

```bash
# Debian/Ubuntu
sudo apt install libpcap-dev

# Fedora/RHEL/CentOS
sudo dnf install libpcap-devel

# Arch Linux
sudo pacman -S libpcap
```

**Troubleshooting**:

| Problem | Solution |
|---------|----------|
| **Permission denied** | Run `sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip` |
| **libpcap missing** | Install with package manager (see above) |
| **Network unreachable** | Check firewall settings (`ufw status`, `iptables -L`) |
| **Capability lost** | Re-run setcap after binary updates |

**Known Issues**: None

---

## Windows Support

### Windows x86_64

**Target**: `x86_64-pc-windows-msvc`

**Supported Versions**:
- Windows 10 (1809+)
- Windows 11
- Windows Server 2016+

**Installation**:

```powershell
# Download the binary (PowerShell)
Invoke-WebRequest -Uri "https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-x86_64-pc-windows-msvc.zip" -OutFile "prtip.zip"

# Extract
Expand-Archive -Path prtip.zip -DestinationPath .

# Move to desired location (optional)
Move-Item prtip.exe C:\Tools\prtip.exe

# Add to PATH (optional)
$env:PATH += ";C:\Tools"
```

**Installing Npcap** (required for packet capture):

1. Download from: <https://npcap.com/#download>
2. Run installer with **Administrator privileges**
3. Enable "WinPcap API-compatible Mode" (recommended)
4. **Restart computer** after installation

**Quick Verification**:

```powershell
# Check version
prtip --version

# Test basic scan (requires Administrator)
prtip -sT -p 80 scanme.nmap.org

# Test SYN scan (requires Administrator + Npcap)
prtip -sS -p 80,443 scanme.nmap.org
```

**Requirements**:
- MSVC Runtime (usually pre-installed)
- **Npcap 1.79+** (for packet capture)
- Administrator privileges (for raw sockets)

**Running as Administrator**:

```powershell
# PowerShell: Right-click PowerShell icon → "Run as Administrator"
# Command Prompt: Right-click CMD icon → "Run as Administrator"

# Or use runas command
runas /user:Administrator "prtip -sS -p 80,443 target.com"
```

**Troubleshooting**:

| Problem | Solution |
|---------|----------|
| **DLL not found** | Install Npcap from <https://npcap.com/> |
| **Access denied** | Run PowerShell/CMD as Administrator |
| **Npcap not working** | Restart computer after Npcap installation |
| **Loopback not working** | Enable "Support loopback traffic" in Npcap installer |

**Known Issues**:
- SYN discovery tests fail on loopback (127.0.0.1) - this is expected Npcap behavior
- Administrator privileges required (cannot use capabilities like Linux)

**Package Managers**:

```powershell
# Chocolatey (future support planned)
choco install prtip

# Winget (future support planned)
winget install ProRT-IP
```

---

## macOS Support

### macOS Intel (x86_64)

**Target**: `x86_64-apple-darwin`

**Supported Versions**:
- macOS 10.13 (High Sierra) and later
- macOS 11+ (Big Sur) recommended

**Installation**:

```bash
# Download the binary
curl -L -o prtip.tar.gz https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-x86_64-apple-darwin.tar.gz

# Extract
tar xzf prtip-0.5.0-x86_64-apple-darwin.tar.gz

# Install to system path
sudo mv prtip /usr/local/bin/

# Remove quarantine attribute (macOS Gatekeeper)
sudo xattr -d com.apple.quarantine /usr/local/bin/prtip
```

**Setup BPF Access** (recommended):

```bash
# Grant your user BPF access (one-time setup)
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf

# Verify group membership
dseditgroup -o checkmember -m $(whoami) access_bpf

# Logout and login for changes to take effect
```

**Quick Verification**:

```bash
# Check version
prtip --version

# Test basic scan (with BPF access)
prtip -sT -p 80 scanme.nmap.org

# Test SYN scan (requires BPF or sudo)
prtip -sS -p 80,443 scanme.nmap.org
```

**Requirements**:
- libpcap (pre-installed on macOS)
- BPF device access (setup above or use sudo)

**Troubleshooting**:

| Problem | Solution |
|---------|----------|
| **Permission denied** | Setup BPF access (see above) or use `sudo` |
| **Binary quarantined** | Run `xattr -d com.apple.quarantine /usr/local/bin/prtip` |
| **BPF not working** | Logout and login after adding user to access_bpf group |
| **"prtip" is damaged** | Remove quarantine attribute (see above) |

**Known Issues**: None

**Homebrew** (future support planned):

```bash
brew install prtip
```

---

### macOS Apple Silicon (ARM64)

**Target**: `aarch64-apple-darwin`

**Supported Versions**:
- macOS 11+ (Big Sur) with M1 chip
- macOS 12+ (Monterey) with M1/M2 chips
- macOS 13+ (Ventura) with M1/M2/M3 chips
- macOS 14+ (Sonoma) with M1/M2/M3/M4 chips

**Installation**:

```bash
# Download the native ARM64 binary
curl -L -o prtip.tar.gz https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-aarch64-apple-darwin.tar.gz

# Extract
tar xzf prtip-0.5.0-aarch64-apple-darwin.tar.gz

# Install to system path
sudo mv prtip /usr/local/bin/

# Remove quarantine attribute
sudo xattr -d com.apple.quarantine /usr/local/bin/prtip
```

**Setup BPF Access** (same as Intel):

```bash
# Grant your user BPF access
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf

# Logout and login for changes to take effect
```

**Verify Architecture**:

```bash
# Check version and architecture
prtip --version
file /usr/local/bin/prtip  # Should show "arm64"

# Test basic scan
prtip -sT -p 80 scanme.nmap.org
```

**Performance**:
- **20-30% faster** than Rosetta-translated x86_64 binaries
- Native Apple Silicon optimization
- Lower power consumption

**Requirements**:
- Native ARM64 binary (no Rosetta required)
- libpcap (pre-installed)
- BPF device access (same as Intel)

**Troubleshooting**:

| Problem | Solution |
|---------|----------|
| **Permission denied** | Setup BPF access or use `sudo` |
| **Binary quarantined** | Run `xattr -d com.apple.quarantine /usr/local/bin/prtip` |
| **Wrong architecture** | Ensure you downloaded `aarch64` version, not `x86_64` |
| **Rosetta warning** | You're using x86_64 version - download `aarch64` for better performance |

**Known Issues**: None

---

## FreeBSD Support

### FreeBSD x86_64

**Target**: `x86_64-unknown-freebsd`

**Supported Versions**:
- FreeBSD 12.x
- FreeBSD 13.x (recommended)
- FreeBSD 14.x

**Installation**:

```bash
# Download the binary
fetch https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-x86_64-unknown-freebsd.tar.gz

# Extract
tar xzf prtip-0.5.0-x86_64-unknown-freebsd.tar.gz

# Install to system path
sudo mv prtip /usr/local/bin/

# Install libpcap if not present
sudo pkg install libpcap
```

**Quick Verification**:

```bash
# Check version
prtip --version

# Test basic scan
prtip -sT -p 80 scanme.nmap.org

# Test SYN scan
sudo prtip -sS -p 80,443 scanme.nmap.org
```

**Requirements**:
- libpcap (install: `pkg install libpcap`)
- BPF device access

**Troubleshooting**:

| Problem | Solution |
|---------|----------|
| **libpcap missing** | Run `sudo pkg install libpcap` |
| **Permission denied** | Check BPF device permissions: `ls -l /dev/bpf*` |
| **No BPF devices** | Load module: `kldload if_tap` |

**Known Issues**: None

---

## Experimental Platforms

These platforms have builds available but may have known limitations. Use with caution.

### Linux x86_64 (musl)

**Target**: `x86_64-unknown-linux-musl`

**Status**: ⚠️ Known type mismatch issues

**Distributions**: Alpine Linux 3.14+

**Known Issues**:
- Type mismatches in prtip-network crate
- Requires conditional compilation fixes

**Benefits**:
- Static binary (no glibc dependency)
- Smaller binary size (~6MB vs ~8MB)
- Faster startup (<30ms vs <50ms)

**Workaround**: Use glibc build (`x86_64-unknown-linux-gnu`) or build from source with musl-specific patches

---

### Linux ARM64 (glibc/musl)

**Target**: `aarch64-unknown-linux-gnu` / `aarch64-unknown-linux-musl`

**Status**: ⚠️ OpenSSL cross-compilation issues

**Devices**:
- Raspberry Pi 4/5 (64-bit OS)
- Ubuntu Server ARM64
- Debian ARM64

**Known Issues**:
- Cross-compilation of OpenSSL fails in CI
- Requires native ARM64 builder or rustls alternative

**Workaround**: Build from source on native ARM64 hardware

```bash
# On Raspberry Pi or ARM64 server
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release
```

---

### Windows ARM64

**Target**: `aarch64-pc-windows-msvc`

**Status**: ⚠️ Removed from CI/CD (toolchain unavailable)

**Devices**:
- Surface Pro X
- Windows ARM64 laptops

**Known Issues**:
- GitHub Actions lacks ARM64 Windows cross-compilation support
- MSVC ARM64 toolchain not available in CI environment

**Workaround**: Build from source on native Windows ARM64 device with MSVC ARM64 toolchain

---

## Building from Source

For unsupported or experimental platforms, build from source:

### Basic Build

```bash
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release
```

### Platform-Specific Builds

**musl static builds** (no glibc dependency):

```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build with vendored OpenSSL
cargo build --release \
  --target x86_64-unknown-linux-musl \
  --features prtip-scanner/vendored-openssl

# Binary location
ls target/x86_64-unknown-linux-musl/release/prtip
```

**Cross-compilation** (Linux ARM64):

```bash
# Install cross-compilation tool
cargo install cross

# Cross-compile to ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Binary location
ls target/aarch64-unknown-linux-gnu/release/prtip
```

**Windows with Npcap SDK**:

```powershell
# Set environment variables
$env:LIB = "C:\Program Files\Npcap\SDK\Lib\x64"
$env:PATH += ";C:\Program Files\Npcap"

# Build
cargo build --release

# Binary location
ls target\release\prtip.exe
```

---

## Platform Comparison

Performance and characteristics relative to Linux x86_64 baseline:

| Platform | Binary Size | Startup Time | Performance | Package Manager |
|----------|-------------|--------------|-------------|-----------------|
| Linux x86_64 (glibc) | ~8MB | <50ms | 100% (baseline) | apt, dnf, pacman |
| Linux x86_64 (musl) | ~6MB | <30ms | 95% | apk |
| Linux ARM64 | ~8MB | <60ms | 85% | apt, dnf |
| Windows x86_64 | ~9MB | <100ms | 90% | chocolatey, winget |
| macOS Intel | ~8MB | <70ms | 95% | brew |
| **macOS ARM64** | **~7MB** | **<40ms** | **110%** | **brew** |
| FreeBSD x86_64 | ~8MB | <60ms | 90% | pkg |

**Notes**:
- macOS ARM64 is fastest platform (110% baseline, native optimization)
- musl builds are smallest and fastest startup
- Performance measured with 65,535-port SYN scan baseline

---

## Future Platform Support

Planned for future releases:

| Platform | Status | ETA |
|----------|--------|-----|
| Linux ARM64 (native builds) | ⏳ Planned | Q1 2026 |
| Windows ARM64 (native toolchain) | ⏳ Planned | Q2 2026 |
| NetBSD x86_64 | ⏳ Planned | Q2 2026 |
| OpenBSD x86_64 | ⏳ Planned | Q3 2026 |
| Linux RISC-V | ⏳ Experimental | Q4 2026 |

---

## Reporting Platform Issues

If you encounter platform-specific issues:

1. **Check Known Issues**: Review this guide's platform-specific sections
2. **Verify Requirements**: Ensure system meets minimum requirements
3. **Try Building from Source**: May resolve toolchain-specific issues
4. **Report Issue**: Include platform details:
   - OS version (`uname -a` or `systeminfo`)
   - Architecture (`uname -m` or `echo %PROCESSOR_ARCHITECTURE%`)
   - Error messages (full output)
   - ProRT-IP version (`prtip --version`)

**GitHub Issues**: <https://github.com/doublegate/ProRT-IP/issues>

---

## See Also

- [Installation & Setup](../getting-started/installation.md) - Complete installation guide
- [Quick Start Guide](../getting-started/quick-start.md) - First scan tutorial
- [Troubleshooting](../reference/troubleshooting.md) - Platform-specific troubleshooting
- [Building from Source](../development/implementation.md) - Advanced build instructions
