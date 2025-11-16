# Secure Configuration

Production deployment best practices, platform-specific hardening guides, and operational security procedures for ProRT-IP.

## Quick Reference

### Security Checklist

| Category | Items | Status |
|----------|-------|--------|
| **Pre-Deployment** | 7 configuration items | See [Pre-Deployment](#pre-deployment-checklist) |
| **Platform Hardening** | 4 platforms (Linux/macOS/Windows/Docker) | See [Platform Hardening](#platform-hardening) |
| **Runtime Security** | 5 monitoring items | See [Monitoring](#security-monitoring) |
| **Incident Response** | 6-phase process | See [Incident Response](#incident-response) |

### Default Security Posture

```bash
# Secure defaults (no configuration required)
✅ Privilege drop enabled (Linux/macOS)
✅ Resource limits enforced (10 categories)
✅ Input validation enabled (all inputs)
✅ TLS certificate verification (HTTPS)
✅ Plugin sandboxing enabled (Lua)
✅ Audit logging disabled by default (enable with --audit-log)

# Manual configuration required
⚠️ Dedicated user/group creation (recommended)
⚠️ Capability grants (Linux) or BPF access (macOS)
⚠️ Output directory permissions (0700 recommended)
⚠️ Firewall rules (block outbound to sensitive networks)
```

---

## Pre-Deployment Checklist

Complete this checklist before deploying ProRT-IP to production.

### 1. User & Group Configuration

**Linux:**
```bash
# Create dedicated user/group
sudo useradd -r -s /bin/false -c "ProRT-IP Scanner" scanner

# Verify user creation
id scanner
# Expected: uid=999(scanner) gid=999(scanner) groups=999(scanner)

# Lock password (prevent login)
sudo passwd -l scanner
```

**macOS:**
```bash
# Add user to BPF access group
sudo dseditgroup -o edit -a scanner -t user access_bpf

# Verify group membership
dseditgroup -o checkmember -m scanner access_bpf
# Expected: yes scanner is a member of access_bpf

# Logout and login for group membership to take effect
```

**Windows:**
```powershell
# Create dedicated user (PowerShell as Administrator)
New-LocalUser -Name "scanner" -Description "ProRT-IP Scanner" `
    -NoPassword -UserMayNotChangePassword

# Add to Users group (not Administrators)
Add-LocalGroupMember -Group "Users" -Member "scanner"

# Verify user creation
Get-LocalUser scanner
```

### 2. Binary Installation

**Linux:**
```bash
# Install binary with restrictive permissions
sudo install -o root -g root -m 0755 prtip /usr/local/bin/prtip

# Grant capabilities (no root required for scanning)
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip

# Verify capabilities
getcap /usr/local/bin/prtip
# Expected: /usr/local/bin/prtip = cap_net_admin,cap_net_raw+eip

# Verify no setuid bit (security risk)
ls -l /usr/local/bin/prtip
# Expected: -rwxr-xr-x (no 's' bit)
```

**macOS:**
```bash
# Install binary
sudo install -o root -g wheel -m 0755 prtip /usr/local/bin/prtip

# Remove quarantine attribute (macOS Gatekeeper)
sudo xattr -d com.apple.quarantine /usr/local/bin/prtip

# Verify no setuid bit
ls -l /usr/local/bin/prtip
# Expected: -rwxr-xr-x
```

**Windows:**
```powershell
# Copy binary to program files (as Administrator)
Copy-Item prtip.exe "C:\Program Files\ProRT-IP\prtip.exe"

# Set restrictive permissions (Users: Read & Execute only)
icacls "C:\Program Files\ProRT-IP\prtip.exe" /inheritance:r /grant:r "Administrators:(F)" "Users:(RX)"

# Verify permissions
icacls "C:\Program Files\ProRT-IP\prtip.exe"
# Expected: Administrators:(F), Users:(RX)
```

### 3. Output Directory Configuration

**Linux:**
```bash
# Create output directory
sudo mkdir -p /var/lib/prtip/scans

# Set ownership to scanner user
sudo chown scanner:scanner /var/lib/prtip/scans

# Set restrictive permissions (owner only)
sudo chmod 0700 /var/lib/prtip/scans

# Verify permissions
ls -ld /var/lib/prtip/scans
# Expected: drwx------ scanner scanner
```

**macOS:**
```bash
# Create output directory
sudo mkdir -p /var/lib/prtip/scans

# Set ownership
sudo chown scanner:staff /var/lib/prtip/scans

# Set permissions
sudo chmod 0700 /var/lib/prtip/scans
```

**Windows:**
```powershell
# Create output directory
New-Item -Path "C:\ProgramData\ProRT-IP\scans" -ItemType Directory -Force

# Set permissions (scanner: Full Control, Administrators: Full Control)
icacls "C:\ProgramData\ProRT-IP\scans" /inheritance:r /grant:r "scanner:(OI)(CI)(F)" "Administrators:(OI)(CI)(F)"
```

### 4. Configuration File

**Location:**
- Linux/macOS: `/etc/prtip/prtip.toml` or `~/.config/prtip/prtip.toml`
- Windows: `C:\ProgramData\ProRT-IP\prtip.toml` or `%APPDATA%\ProRT-IP\prtip.toml`

**Secure Template:**
```toml
# /etc/prtip/prtip.toml - Secure production configuration

[scanner]
# Scan parameters
default_scan_type = "syn"          # SYN scan (requires privileges)
default_timing = "normal"          # T3 timing (balanced)
max_rate = 100000                  # 100K pps (safe for most networks)
max_retries = 3                    # Retry failed ports 3 times

[privileges]
# Privilege drop (Linux/macOS only)
drop_privileges = true             # Drop after socket creation
unprivileged_user = "scanner"      # User to drop to
unprivileged_group = "scanner"     # Group to drop to

[output]
# Output configuration
output_dir = "/var/lib/prtip/scans"  # Default output directory
default_format = "json"              # JSON output
create_directories = true            # Create output dirs if missing

[security]
# Security settings
enable_audit_log = true            # Enable audit logging
audit_log_path = "/var/log/prtip/audit.log"
confirmation_required = true       # Prompt before internet scans
max_targets = 256                  # Limit targets per scan

[resource_limits]
# Resource limits (DoS prevention)
max_concurrent = 10000             # Max concurrent connections
max_memory_mb = 1024               # Max memory (1GB)
max_duration_sec = 86400           # Max scan duration (24h)
network_timeout_ms = 1000          # Network I/O timeout (1s)

[plugins]
# Plugin system security
enable_plugins = false             # Disable plugins by default
plugin_dir = "/var/lib/prtip/plugins"
sandbox_enabled = true             # Lua sandboxing
max_instructions = 1000000         # Max Lua instructions
max_memory_kb = 10240              # Max plugin memory (10MB)

[tui]
# TUI configuration
enable_tui = false                 # Disable TUI by default (CLI mode)
event_log_enabled = true           # Enable event logging
max_events = 10000                 # Max events in memory
```

**Permissions:**
```bash
# Linux/macOS
sudo chown root:root /etc/prtip/prtip.toml
sudo chmod 0644 /etc/prtip/prtip.toml  # World-readable (no secrets)

# Windows
icacls "C:\ProgramData\ProRT-IP\prtip.toml" /inheritance:r /grant:r "Administrators:(F)" "Users:(R)"
```

### 5. Logging Configuration

**Audit Log Setup (Linux/macOS):**
```bash
# Create log directory
sudo mkdir -p /var/log/prtip

# Set ownership and permissions
sudo chown scanner:scanner /var/log/prtip
sudo chmod 0700 /var/log/prtip

# Create audit log file
sudo touch /var/log/prtip/audit.log
sudo chown scanner:scanner /var/log/prtip/audit.log
sudo chmod 0600 /var/log/prtip/audit.log

# Configure log rotation
sudo tee /etc/logrotate.d/prtip <<EOF
/var/log/prtip/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0600 scanner scanner
    sharedscripts
}
EOF
```

**Windows Event Log:**
```powershell
# Create event log source (as Administrator)
New-EventLog -Source "ProRT-IP" -LogName "Application"

# Verify creation
Get-EventLog -LogName "Application" -Source "ProRT-IP" -Newest 1
```

### 6. Firewall Configuration

**Linux (iptables):**
```bash
# Block outbound to private networks (prevent accidental scans)
sudo iptables -A OUTPUT -m owner --uid-owner scanner \
    -d 10.0.0.0/8 -j DROP
sudo iptables -A OUTPUT -m owner --uid-owner scanner \
    -d 172.16.0.0/12 -j DROP
sudo iptables -A OUTPUT -m owner --uid-owner scanner \
    -d 192.168.0.0/16 -j DROP

# Block outbound to localhost (prevent self-scan)
sudo iptables -A OUTPUT -m owner --uid-owner scanner \
    -d 127.0.0.0/8 -j DROP

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

**Linux (ufw):**
```bash
# Enable firewall
sudo ufw enable

# Deny outbound from scanner user (whitelist approach)
sudo ufw deny out on eth0 from any to any uid scanner

# Allow specific targets (example)
sudo ufw allow out on eth0 from any to 203.0.113.0/24 uid scanner
```

**macOS (pf):**
```bash
# Create pf rule file
sudo tee /etc/pf.anchors/prtip <<EOF
# Block scanner user from private networks
block out proto tcp from any to 10.0.0.0/8 user scanner
block out proto tcp from any to 172.16.0.0/12 user scanner
block out proto tcp from any to 192.168.0.0/16 user scanner
block out proto tcp from any to 127.0.0.0/8 user scanner
EOF

# Load pf anchor
sudo pfctl -a prtip -f /etc/pf.anchors/prtip

# Enable pf
sudo pfctl -e
```

**Windows Firewall:**
```powershell
# Block outbound to private networks (PowerShell as Administrator)
New-NetFirewallRule -DisplayName "Block ProRT-IP to Private 10/8" `
    -Direction Outbound -Action Block `
    -RemoteAddress 10.0.0.0/8 `
    -Program "C:\Program Files\ProRT-IP\prtip.exe"

New-NetFirewallRule -DisplayName "Block ProRT-IP to Private 172.16/12" `
    -Direction Outbound -Action Block `
    -RemoteAddress 172.16.0.0/12 `
    -Program "C:\Program Files\ProRT-IP\prtip.exe"

New-NetFirewallRule -DisplayName "Block ProRT-IP to Private 192.168/16" `
    -Direction Outbound -Action Block `
    -RemoteAddress 192.168.0.0/16 `
    -Program "C:\Program Files\ProRT-IP\prtip.exe"

# Verify rules
Get-NetFirewallRule -DisplayName "Block ProRT-IP*"
```

### 7. Verification

**Test Privilege Drop:**
```bash
# Run ProRT-IP and verify it drops privileges
sudo -u scanner prtip -sS -p 80 scanme.nmap.org

# Check running process UID (in another terminal)
ps aux | grep prtip
# Expected: UID should be 'scanner', not 'root'
```

**Test Configuration:**
```bash
# Verify configuration loaded
prtip --show-config
# Should display /etc/prtip/prtip.toml settings

# Test scan with audit logging
sudo -u scanner prtip -sT -p 80 scanme.nmap.org

# Verify audit log created
sudo cat /var/log/prtip/audit.log
# Should contain scan entry with timestamp, user, target
```

**Test Resource Limits:**
```bash
# Attempt scan exceeding max_targets (should fail)
sudo -u scanner prtip -sS -p 80 0.0.0.0/0
# Expected: Error: "Exceeds max_targets limit (256)"

# Attempt scan exceeding max_duration_sec (should timeout)
sudo -u scanner prtip -sS -p- --max-duration 10 192.168.1.0/24
# Expected: Scan times out after 10 seconds
```

---

## Platform Hardening

### Linux Hardening

**1. AppArmor Profile:**

Create `/etc/apparmor.d/usr.local.bin.prtip`:

```apparmor
#include <tunables/global>

/usr/local/bin/prtip {
  #include <abstractions/base>
  #include <abstractions/nameservice>

  # Capabilities (raw sockets)
  capability net_raw,
  capability net_admin,

  # Binary execution
  /usr/local/bin/prtip mr,

  # Configuration files
  /etc/prtip/** r,
  owner @{HOME}/.config/prtip/** r,

  # Output directory (write only)
  /var/lib/prtip/scans/** rw,

  # Temporary files
  owner /tmp/prtip-** rw,

  # Logging
  /var/log/prtip/*.log w,

  # Network access
  network inet raw,
  network inet6 raw,
  network inet stream,
  network inet6 stream,
  network inet dgram,
  network inet6 dgram,

  # Deny sensitive files
  deny /etc/shadow r,
  deny /etc/gshadow r,
  deny /root/** rw,
  deny /home/*/.ssh/** rw,

  # Deny dangerous capabilities
  deny capability setuid,
  deny capability setgid,
  deny capability sys_admin,
  deny capability sys_module,

  # Deny process tracing
  deny ptrace,

  # Deny mounting
  deny mount,
  deny umount,
}
```

**Load and enforce:**
```bash
# Parse profile
sudo apparmor_parser -r /etc/apparmor.d/usr.local.bin.prtip

# Set to enforce mode
sudo aa-enforce /usr/local/bin/prtip

# Verify status
sudo aa-status | grep prtip
# Expected: /usr/local/bin/prtip (enforce)
```

**2. SELinux Policy:**

Create `prtip.te`:

```selinux
policy_module(prtip, 1.0.0)

# Type declarations
type prtip_t;
type prtip_exec_t;

# Domain transition
init_daemon_domain(prtip_t, prtip_exec_t)

# Capabilities
allow prtip_t self:capability { net_raw net_admin };
allow prtip_t self:packet_socket { create bind write read };

# Network access
allow prtip_t self:tcp_socket { create connect };
allow prtip_t self:udp_socket { create bind };
corenet_tcp_connect_all_ports(prtip_t)
corenet_udp_bind_all_ports(prtip_t)

# File access
files_read_etc_files(prtip_t)
allow prtip_t prtip_var_lib_t:dir { create read write add_name remove_name };
allow prtip_t prtip_var_lib_t:file { create read write append };

# Logging
logging_send_syslog_msg(prtip_t)

# Deny dangerous operations
dontaudit prtip_t self:capability { setuid setgid sys_admin };
```

**Compile and install:**
```bash
# Compile policy
checkmodule -M -m -o prtip.mod prtip.te
semodule_package -o prtip.pp -m prtip.mod

# Install policy
sudo semodule -i prtip.pp

# Verify installation
sudo semodule -l | grep prtip
# Expected: prtip    1.0.0
```

**3. Seccomp Filter:**

ProRT-IP includes built-in seccomp filtering (future):

```rust
// Future enhancement: Seccomp filter
use seccomp::{Context, Action, Cmp};

pub fn apply_seccomp_filter() -> Result<()> {
    let mut ctx = Context::new(Action::Allow)?;

    // Deny dangerous syscalls
    ctx.add_rule(Action::Errno(libc::EPERM), libc::SYS_ptrace)?;
    ctx.add_rule(Action::Errno(libc::EPERM), libc::SYS_mount)?;
    ctx.add_rule(Action::Errno(libc::EPERM), libc::SYS_umount2)?;
    ctx.add_rule(Action::Errno(libc::EPERM), libc::SYS_kexec_load)?;
    ctx.add_rule(Action::Errno(libc::EPERM), libc::SYS_init_module)?;
    ctx.add_rule(Action::Errno(libc::EPERM), libc::SYS_delete_module)?;

    // Allow only after privilege drop
    ctx.add_rule_conditional(
        Action::Allow,
        libc::SYS_setuid,
        &[Cmp::eq(0, getuid())],  // Only if not root
    )?;

    ctx.load()?;
    Ok(())
}
```

**4. Systemd Service Hardening:**

Create `/etc/systemd/system/prtip@.service`:

```ini
[Unit]
Description=ProRT-IP Network Scanner
After=network.target

[Service]
Type=simple
User=scanner
Group=scanner

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/prtip/scans /var/log/prtip
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true
LockPersonality=true
MemoryDenyWriteExecute=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_PACKET
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources
SystemCallErrorNumber=EPERM

# Resource limits
LimitNOFILE=65535
LimitNPROC=512
CPUQuota=200%
MemoryLimit=1G

# Working directory
WorkingDirectory=/var/lib/prtip

# Command
ExecStart=/usr/local/bin/prtip --config /etc/prtip/prtip.toml %i

[Install]
WantedBy=multi-user.target
```

**Enable and start:**
```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service
sudo systemctl enable prtip@scanme.nmap.org.service

# Start service
sudo systemctl start prtip@scanme.nmap.org.service

# Verify security settings
sudo systemd-analyze security prtip@scanme.nmap.org.service
# Expected: Overall exposure level: 9.6 (SAFE)
```

---

### macOS Hardening

**1. Sandboxing (sandbox-exec):**

Create `prtip.sb`:

```scheme
(version 1)

; Default deny
(deny default)

; Allow basic system access
(allow file-read* (literal "/"))
(allow file-read-metadata)
(allow process-exec (literal "/usr/local/bin/prtip"))

; Allow configuration files
(allow file-read* (subpath "/etc/prtip"))
(allow file-read* (subpath (string-append (param "HOME") "/.config/prtip")))

; Allow output directory (read/write)
(allow file-read* file-write* (subpath "/var/lib/prtip/scans"))

; Allow networking
(allow network-outbound)
(allow network-bind)
(allow system-socket)

; Deny sensitive files
(deny file-read* file-write* (subpath "/private/etc/shadow"))
(deny file-read* file-write* (subpath "/var/root"))
(deny file-read* file-write* (regex #"^/Users/.*/\.ssh"))

; Deny process injection
(deny process-fork)
(deny signal)
(deny mach-lookup)
```

**Run with sandbox:**
```bash
# Execute with sandbox
sandbox-exec -f prtip.sb /usr/local/bin/prtip -sS -p 80 scanme.nmap.org

# Verify sandboxing active
ps aux | grep prtip
# Should show 'sandbox-exec' as parent process
```

**2. Code Signing:**

```bash
# Create self-signed certificate (development)
security create-keychain -p "password" prtip.keychain
security set-keychain-settings prtip.keychain
security unlock-keychain -p "password" prtip.keychain

# Generate certificate
certtool r genkey k=prtip.key
certtool c req="C=US,O=ProRT-IP,CN=ProRT-IP Developer" k=prtip.key o=prtip.csr
certtool c cert="C=US,O=ProRT-IP,CN=ProRT-IP Developer" k=prtip.key i=prtip.csr o=prtip.crt

# Sign binary
codesign --force --sign "ProRT-IP Developer" \
    --entitlements prtip.entitlements \
    /usr/local/bin/prtip

# Verify signature
codesign --verify --deep --strict --verbose=2 /usr/local/bin/prtip
spctl --assess --type execute --verbose=4 /usr/local/bin/prtip
```

**3. Entitlements (prtip.entitlements):**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Network access -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>

    <!-- File system access -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>

    <!-- Deny dangerous entitlements -->
    <key>com.apple.security.cs.allow-jit</key>
    <false/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <false/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <false/>
    <key>com.apple.security.cs.allow-dyld-environment-variables</key>
    <false/>
</dict>
</plist>
```

---

### Windows Hardening

**1. AppContainer Isolation (future):**

```powershell
# Create AppContainer (future ProRT-IP enhancement)
# This will isolate ProRT-IP from the rest of the system

# Get SID for scanner user
$user = New-Object System.Security.Principal.NTAccount("scanner")
$sid = $user.Translate([System.Security.Principal.SecurityIdentifier])

# Create AppContainer profile
Add-AppxPackage -Register "C:\Program Files\ProRT-IP\AppxManifest.xml"

# Grant network capability
netsh advfirewall firewall add rule name="ProRT-IP" `
    dir=out action=allow program="C:\Program Files\ProRT-IP\prtip.exe" `
    enable=yes profile=any
```

**2. Software Restriction Policy:**

```powershell
# Create software restriction policy (as Administrator)
$rule = New-Object -TypeName System.Security.Cryptography.X509Certificates.X509Certificate2
$rule.Import("C:\Program Files\ProRT-IP\prtip.exe")

# Add to trusted publishers
Import-Certificate -FilePath "C:\Program Files\ProRT-IP\prtip.cer" `
    -CertStoreLocation Cert:\LocalMachine\TrustedPublisher

# Restrict execution to signed binaries only
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Safer\CodeIdentifiers" `
    -Name "DefaultLevel" -Value 0  # Disallowed by default

# Allow ProRT-IP (by hash or certificate)
New-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Safer\CodeIdentifiers\0\Hashes\{GUID}" `
    -Name "ItemData" -Value $rule.GetCertHashString()
```

**3. Windows Defender Application Control (WDAC):**

```powershell
# Create WDAC policy (as Administrator)
New-CIPolicy -Level Publisher -FilePath "C:\WDAC\ProRT-IP.xml" `
    -Fallback Hash -UserPEs `
    -ScanPath "C:\Program Files\ProRT-IP"

# Convert to binary policy
ConvertFrom-CIPolicy -XmlFilePath "C:\WDAC\ProRT-IP.xml" `
    -BinaryFilePath "C:\Windows\System32\CodeIntegrity\SIPolicy.p7b"

# Refresh policy
Invoke-CimMethod -Namespace root\Microsoft\Windows\CI `
    -ClassName PS_UpdateAndCompareCIPolicy `
    -MethodName Update -Arguments @{FilePath = "C:\Windows\System32\CodeIntegrity\SIPolicy.p7b"}
```

---

### Docker Hardening

**1. Secure Dockerfile:**

```dockerfile
# Multi-stage build
FROM rust:1.70 AS builder

# Create app directory
WORKDIR /app

# Copy source
COPY . .

# Build release binary
RUN cargo build --release --locked

# Runtime image (Alpine for minimal attack surface)
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache libgcc libcap

# Create dedicated user/group
RUN addgroup -g 1000 scanner && \
    adduser -D -u 1000 -G scanner scanner

# Copy binary from builder
COPY --from=builder /app/target/release/prtip /usr/local/bin/prtip

# Grant capabilities
RUN setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip

# Set ownership
RUN chown root:root /usr/local/bin/prtip && \
    chmod 0755 /usr/local/bin/prtip

# Create output directory
RUN mkdir -p /var/lib/prtip/scans && \
    chown scanner:scanner /var/lib/prtip/scans && \
    chmod 0700 /var/lib/prtip/scans

# Switch to unprivileged user
USER scanner

# Set working directory
WORKDIR /var/lib/prtip

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD prtip --version || exit 1

# Entrypoint
ENTRYPOINT ["/usr/local/bin/prtip"]
CMD ["--help"]
```

**2. Docker Compose (production):**

```yaml
version: '3.8'

services:
  prtip:
    image: prtip:latest
    container_name: prtip-scanner

    # Security options
    security_opt:
      - no-new-privileges:true
      - apparmor:docker-default
      - seccomp:default

    # Capabilities (drop all, add only required)
    cap_drop:
      - ALL
    cap_add:
      - NET_RAW
      - NET_ADMIN

    # Read-only root filesystem
    read_only: true

    # Temporary filesystem for /tmp
    tmpfs:
      - /tmp:rw,noexec,nosuid,size=100m

    # Resource limits
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M

    # Volumes (output directory only)
    volumes:
      - ./scans:/var/lib/prtip/scans:rw
      - ./config:/etc/prtip:ro

    # Network (custom network for isolation)
    networks:
      - scan-network

    # User (run as unprivileged user)
    user: "1000:1000"

    # Environment variables
    environment:
      - RUST_LOG=info
      - PRTIP_CONFIG=/etc/prtip/prtip.toml

    # Command
    command: ["-sS", "-p", "80,443", "scanme.nmap.org"]

    # Restart policy
    restart: unless-stopped

networks:
  scan-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
```

**3. Docker Run (production):**

```bash
# Run with security hardening
docker run -d \
    --name prtip-scanner \
    --security-opt no-new-privileges:true \
    --security-opt apparmor=docker-default \
    --security-opt seccomp=default \
    --cap-drop ALL \
    --cap-add NET_RAW \
    --cap-add NET_ADMIN \
    --read-only \
    --tmpfs /tmp:rw,noexec,nosuid,size=100m \
    --memory 1g \
    --cpus 2.0 \
    --user 1000:1000 \
    --volume $(pwd)/scans:/var/lib/prtip/scans:rw \
    --volume $(pwd)/config:/etc/prtip:ro \
    --network scan-network \
    --restart unless-stopped \
    prtip:latest \
    -sS -p 80,443 scanme.nmap.org

# Verify security settings
docker inspect prtip-scanner | jq '.[0].HostConfig.SecurityOpt'
# Expected: ["no-new-privileges:true", "apparmor=docker-default", ...]
```

---

## Configuration Validation

### Automated Validation

**Linux/macOS:**
```bash
#!/bin/bash
# validate-config.sh - Validate ProRT-IP security configuration

set -e

echo "ProRT-IP Security Configuration Validation"
echo "==========================================="
echo

# 1. User/Group existence
echo "[1/10] Checking user/group..."
if id scanner &>/dev/null; then
    echo "✅ User 'scanner' exists"
else
    echo "❌ User 'scanner' does not exist"
    exit 1
fi

# 2. Binary permissions
echo "[2/10] Checking binary permissions..."
BINARY="/usr/local/bin/prtip"
if [[ -f "$BINARY" ]]; then
    OWNER=$(stat -c '%U:%G' "$BINARY" 2>/dev/null || stat -f '%Su:%Sg' "$BINARY")
    PERMS=$(stat -c '%a' "$BINARY" 2>/dev/null || stat -f '%A' "$BINARY" | tail -c 4)

    if [[ "$OWNER" == "root:root" ]] && [[ "$PERMS" == "755" ]]; then
        echo "✅ Binary ownership and permissions correct"
    else
        echo "❌ Binary ownership ($OWNER) or permissions ($PERMS) incorrect"
        exit 1
    fi
else
    echo "❌ Binary not found at $BINARY"
    exit 1
fi

# 3. Capabilities (Linux only)
if [[ "$(uname)" == "Linux" ]]; then
    echo "[3/10] Checking capabilities..."
    CAPS=$(getcap "$BINARY" | grep -o 'cap_net_raw,cap_net_admin+eip' || true)
    if [[ -n "$CAPS" ]]; then
        echo "✅ Capabilities granted"
    else
        echo "❌ Capabilities not granted"
        exit 1
    fi
else
    echo "[3/10] Skipping capabilities check (macOS)"
fi

# 4. Output directory
echo "[4/10] Checking output directory..."
OUTPUT_DIR="/var/lib/prtip/scans"
if [[ -d "$OUTPUT_DIR" ]]; then
    OWNER=$(stat -c '%U:%G' "$OUTPUT_DIR" 2>/dev/null || stat -f '%Su:%Sg' "$OUTPUT_DIR")
    PERMS=$(stat -c '%a' "$OUTPUT_DIR" 2>/dev/null || stat -f '%A' "$OUTPUT_DIR" | tail -c 4)

    if [[ "$OWNER" == "scanner:scanner" ]] && [[ "$PERMS" == "700" ]]; then
        echo "✅ Output directory ownership and permissions correct"
    else
        echo "❌ Output directory ownership ($OWNER) or permissions ($PERMS) incorrect"
        exit 1
    fi
else
    echo "❌ Output directory does not exist"
    exit 1
fi

# 5. Configuration file
echo "[5/10] Checking configuration file..."
CONFIG_FILE="/etc/prtip/prtip.toml"
if [[ -f "$CONFIG_FILE" ]]; then
    PERMS=$(stat -c '%a' "$CONFIG_FILE" 2>/dev/null || stat -f '%A' "$CONFIG_FILE" | tail -c 4)
    if [[ "$PERMS" == "644" ]]; then
        echo "✅ Configuration file permissions correct"
    else
        echo "⚠️  Configuration file permissions ($PERMS) should be 644"
    fi
else
    echo "⚠️  Configuration file not found (optional)"
fi

# 6. Audit log directory
echo "[6/10] Checking audit log directory..."
LOG_DIR="/var/log/prtip"
if [[ -d "$LOG_DIR" ]]; then
    OWNER=$(stat -c '%U:%G' "$LOG_DIR" 2>/dev/null || stat -f '%Su:%Sg' "$LOG_DIR")
    PERMS=$(stat -c '%a' "$LOG_DIR" 2>/dev/null || stat -f '%A' "$LOG_DIR" | tail -c 4)

    if [[ "$OWNER" == "scanner:scanner" ]] && [[ "$PERMS" == "700" ]]; then
        echo "✅ Audit log directory ownership and permissions correct"
    else
        echo "⚠️  Audit log directory ownership ($OWNER) or permissions ($PERMS) should be scanner:scanner 700"
    fi
else
    echo "⚠️  Audit log directory not found (optional)"
fi

# 7. No setuid bit
echo "[7/10] Checking for setuid bit..."
if [[ -u "$BINARY" ]]; then
    echo "❌ setuid bit is set (security risk)"
    exit 1
else
    echo "✅ No setuid bit (secure)"
fi

# 8. AppArmor/SELinux (Linux only)
if [[ "$(uname)" == "Linux" ]]; then
    echo "[8/10] Checking mandatory access control..."
    if command -v aa-status &>/dev/null; then
        if aa-status | grep -q "$BINARY"; then
            echo "✅ AppArmor profile loaded"
        else
            echo "⚠️  AppArmor profile not loaded (optional)"
        fi
    elif command -v sestatus &>/dev/null; then
        if sestatus | grep -q "enabled"; then
            echo "✅ SELinux enabled"
        else
            echo "⚠️  SELinux not enabled (optional)"
        fi
    else
        echo "⚠️  No MAC system detected (optional)"
    fi
else
    echo "[8/10] Skipping MAC check (macOS)"
fi

# 9. Privilege drop test
echo "[9/10] Testing privilege drop..."
TEST_OUTPUT=$(sudo -u scanner "$BINARY" --version 2>&1 || true)
if echo "$TEST_OUTPUT" | grep -q "ProRT-IP"; then
    echo "✅ Binary executes as scanner user"
else
    echo "❌ Binary execution test failed"
    exit 1
fi

# 10. Configuration syntax
echo "[10/10] Validating configuration syntax..."
if [[ -f "$CONFIG_FILE" ]]; then
    if "$BINARY" --config "$CONFIG_FILE" --validate-config 2>/dev/null; then
        echo "✅ Configuration syntax valid"
    else
        echo "❌ Configuration syntax invalid"
        exit 1
    fi
else
    echo "⚠️  Configuration file not found, skipping syntax check"
fi

echo
echo "✅ Security configuration validation complete!"
echo "All critical checks passed."
```

**Windows (PowerShell):**
```powershell
# validate-config.ps1 - Validate ProRT-IP security configuration

Write-Host "ProRT-IP Security Configuration Validation" -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host

# 1. User existence
Write-Host "[1/8] Checking user..." -ForegroundColor Yellow
try {
    $user = Get-LocalUser -Name "scanner" -ErrorAction Stop
    Write-Host "✅ User 'scanner' exists" -ForegroundColor Green
} catch {
    Write-Host "❌ User 'scanner' does not exist" -ForegroundColor Red
    exit 1
}

# 2. Binary existence
Write-Host "[2/8] Checking binary..." -ForegroundColor Yellow
$binary = "C:\Program Files\ProRT-IP\prtip.exe"
if (Test-Path $binary) {
    Write-Host "✅ Binary found" -ForegroundColor Green
} else {
    Write-Host "❌ Binary not found" -ForegroundColor Red
    exit 1
}

# 3. Binary permissions
Write-Host "[3/8] Checking binary permissions..." -ForegroundColor Yellow
$acl = Get-Acl $binary
$hasAdmin = $acl.Access | Where-Object { $_.IdentityReference -like "*Administrators*" -and $_.FileSystemRights -like "*FullControl*" }
$hasUsers = $acl.Access | Where-Object { $_.IdentityReference -like "*Users*" -and $_.FileSystemRights -like "*ReadAndExecute*" }

if ($hasAdmin -and $hasUsers) {
    Write-Host "✅ Binary permissions correct" -ForegroundColor Green
} else {
    Write-Host "❌ Binary permissions incorrect" -ForegroundColor Red
    exit 1
}

# 4. Output directory
Write-Host "[4/8] Checking output directory..." -ForegroundColor Yellow
$outputDir = "C:\ProgramData\ProRT-IP\scans"
if (Test-Path $outputDir) {
    $acl = Get-Acl $outputDir
    $hasScanner = $acl.Access | Where-Object { $_.IdentityReference -like "*scanner*" }
    if ($hasScanner) {
        Write-Host "✅ Output directory permissions correct" -ForegroundColor Green
    } else {
        Write-Host "❌ Output directory permissions incorrect" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "❌ Output directory does not exist" -ForegroundColor Red
    exit 1
}

# 5. Configuration file
Write-Host "[5/8] Checking configuration file..." -ForegroundColor Yellow
$configFile = "C:\ProgramData\ProRT-IP\prtip.toml"
if (Test-Path $configFile) {
    Write-Host "✅ Configuration file found" -ForegroundColor Green
} else {
    Write-Host "⚠️  Configuration file not found (optional)" -ForegroundColor Yellow
}

# 6. Firewall rules
Write-Host "[6/8] Checking firewall rules..." -ForegroundColor Yellow
$rules = Get-NetFirewallRule -DisplayName "Block ProRT-IP*" -ErrorAction SilentlyContinue
if ($rules) {
    Write-Host "✅ Firewall rules configured ($($rules.Count) rules)" -ForegroundColor Green
} else {
    Write-Host "⚠️  No firewall rules found (optional)" -ForegroundColor Yellow
}

# 7. Npcap installation
Write-Host "[7/8] Checking Npcap installation..." -ForegroundColor Yellow
$npcap = Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Npcap" -ErrorAction SilentlyContinue
if ($npcap) {
    Write-Host "✅ Npcap installed (version $($npcap.Version))" -ForegroundColor Green
} else {
    Write-Host "❌ Npcap not installed" -ForegroundColor Red
    exit 1
}

# 8. Binary signature
Write-Host "[8/8] Checking binary signature..." -ForegroundColor Yellow
$signature = Get-AuthenticodeSignature $binary
if ($signature.Status -eq "Valid") {
    Write-Host "✅ Binary signature valid" -ForegroundColor Green
} else {
    Write-Host "⚠️  Binary not signed (optional)" -ForegroundColor Yellow
}

Write-Host
Write-Host "✅ Security configuration validation complete!" -ForegroundColor Green
Write-Host "All critical checks passed." -ForegroundColor Green
```

---

## Security Monitoring

### Audit Logging

**Enable Audit Logging:**
```bash
# Linux/macOS
sudo -u scanner prtip -sS -p 80,443 scanme.nmap.org \
    --audit-log /var/log/prtip/audit.log

# Windows
prtip -sS -p 80,443 scanme.nmap.org `
    --audit-log "C:\ProgramData\ProRT-IP\audit.log"
```

**Audit Log Format (JSON):**
```json
{
  "timestamp": "2025-11-15T10:30:45.123Z",
  "event_type": "scan_started",
  "user": "scanner",
  "uid": 1000,
  "gid": 1000,
  "pid": 12345,
  "binary_path": "/usr/local/bin/prtip",
  "binary_hash": "sha256:abc123...",
  "command_line": "prtip -sS -p 80,443 scanme.nmap.org",
  "scan_parameters": {
    "scan_type": "syn",
    "targets": ["scanme.nmap.org"],
    "ports": "80,443",
    "timing": "normal"
  },
  "network": {
    "source_ip": "192.168.1.100",
    "interface": "eth0"
  },
  "privileges": {
    "effective_uid": 1000,
    "effective_gid": 1000,
    "capabilities": []
  }
}
```

**Monitor Audit Logs:**
```bash
# Real-time monitoring (Linux)
tail -f /var/log/prtip/audit.log | jq .

# Search for failed privilege drops
jq 'select(.event_type == "privilege_drop_failed")' /var/log/prtip/audit.log

# Search for unauthorized targets
jq 'select(.scan_parameters.targets[] | test("10\\.|192\\.168\\."))' /var/log/prtip/audit.log
```

### System Monitoring

**Process Monitoring:**
```bash
# Monitor ProRT-IP processes
watch -n 1 'ps aux | grep prtip'

# Monitor privilege elevation attempts
auditctl -w /usr/local/bin/prtip -p x -k prtip_execution
ausearch -k prtip_execution

# Monitor capability usage (Linux)
filecap /usr/local/bin/prtip | grep -v "^/"
```

**Network Monitoring:**
```bash
# Monitor outbound connections
sudo tcpdump -i eth0 'src host 192.168.1.100 and dst port 80' -n

# Monitor packet rates (ensure within limits)
watch -n 1 'ifconfig eth0 | grep "TX packets"'

# Monitor for unexpected destinations
sudo tcpdump -i eth0 'dst net 10.0.0.0/8 or dst net 172.16.0.0/12 or dst net 192.168.0.0/16' -n
# Expected: No packets (should be blocked by firewall)
```

**Resource Monitoring:**
```bash
# Monitor memory usage
watch -n 1 'pmap $(pgrep prtip) | tail -n 1'

# Monitor CPU usage
top -p $(pgrep prtip)

# Monitor file descriptor usage
lsof -p $(pgrep prtip) | wc -l
```

### Alerting

**Linux (syslog-ng):**
```bash
# /etc/syslog-ng/conf.d/prtip.conf
source s_prtip {
    file("/var/log/prtip/audit.log"
         flags(no-parse)
         follow-freq(1));
};

filter f_privilege_drop_failed {
    match("privilege_drop_failed" value("MESSAGE"));
};

filter f_unauthorized_target {
    match("10\\.|192\\.168\\." value("MESSAGE"));
};

destination d_alert {
    program("/usr/local/bin/send-alert.sh");
};

log {
    source(s_prtip);
    filter(f_privilege_drop_failed);
    destination(d_alert);
};

log {
    source(s_prtip);
    filter(f_unauthorized_target);
    destination(d_alert);
};
```

**Alerting Script:**
```bash
#!/bin/bash
# /usr/local/bin/send-alert.sh

MESSAGE="$1"

# Send email alert
echo "$MESSAGE" | mail -s "ProRT-IP Security Alert" security@example.com

# Send Slack notification (optional)
curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
    -H 'Content-Type: application/json' \
    -d "{\"text\": \"ProRT-IP Security Alert: $MESSAGE\"}"

# Log to syslog
logger -t prtip-alert "Security alert: $MESSAGE"
```

---

## Incident Response

### Detection

**Indicators of Compromise:**
```bash
# 1. Privilege drop failure
grep "privilege_drop_failed" /var/log/prtip/audit.log

# 2. Unauthorized targets
grep -E "10\.|192\.168\.|172\.16\." /var/log/prtip/audit.log

# 3. Excessive resource usage
ps aux | grep prtip | awk '{if ($4 > 50) print $0}'  # >50% memory

# 4. Suspicious binary modifications
stat /usr/local/bin/prtip | grep Modify
sha256sum /usr/local/bin/prtip  # Compare with known-good hash

# 5. Unexpected network connections
netstat -tunapl | grep prtip | grep ESTABLISHED
```

### Containment

**Immediate Actions:**
```bash
# 1. Stop all ProRT-IP processes
sudo killall -9 prtip

# 2. Disable systemd service (if configured)
sudo systemctl stop prtip@*.service
sudo systemctl disable prtip@*.service

# 3. Revoke capabilities (Linux)
sudo setcap -r /usr/local/bin/prtip

# 4. Remove from BPF group (macOS)
sudo dseditgroup -o edit -d scanner -t user access_bpf

# 5. Block network access
sudo iptables -A OUTPUT -m owner --uid-owner scanner -j DROP  # Linux
sudo pfctl -a prtip -F all  # macOS
```

### Eradication

**Remove Compromise:**
```bash
# 1. Quarantine binary
sudo mv /usr/local/bin/prtip /var/quarantine/prtip.$(date +%s)

# 2. Remove configuration
sudo rm -rf /etc/prtip /var/lib/prtip /var/log/prtip

# 3. Remove user/group
sudo userdel scanner
sudo groupdel scanner

# 4. Clear audit logs (preserve for forensics)
sudo cp -r /var/log/prtip /var/forensics/prtip-$(date +%s)
sudo rm -rf /var/log/prtip
```

### Recovery

**Restore from Known-Good State:**
```bash
# 1. Download verified binary
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.5.0/prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz

# 2. Verify checksum
sha256sum prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz
# Compare with official checksum

# 3. Verify signature
gpg --verify prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz.sig

# 4. Extract and install
tar xzf prtip-0.5.0-x86_64-unknown-linux-gnu.tar.gz
sudo install -o root -g root -m 0755 prtip /usr/local/bin/prtip

# 5. Re-apply security configuration
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip
sudo chown root:root /usr/local/bin/prtip

# 6. Recreate user/group
sudo useradd -r -s /bin/false -c "ProRT-IP Scanner" scanner

# 7. Restore configuration from backup
sudo cp /var/backups/prtip/prtip.toml /etc/prtip/prtip.toml

# 8. Test security configuration
./validate-config.sh
```

### Post-Incident

**Lessons Learned:**
```bash
# 1. Document incident
cat > /var/forensics/incident-$(date +%s).md <<EOF
# Incident Report: ProRT-IP Security Incident

**Date**: $(date)
**Severity**: [Critical/High/Medium/Low]
**Reporter**: [Name/Email]

## Summary
[Brief description of incident]

## Timeline
- [Time]: Initial detection
- [Time]: Containment actions
- [Time]: Eradication complete
- [Time]: Recovery complete

## Root Cause
[Analysis of how incident occurred]

## Impact
- Systems affected: [List]
- Data compromised: [Yes/No/Unknown]
- Duration: [X hours/days]

## Remediation
- Immediate actions: [List]
- Long-term fixes: [List]

## Lessons Learned
- What went well: [List]
- What to improve: [List]
- Action items: [List with owners and deadlines]
EOF

# 2. Update security controls
# - Tighten firewall rules
# - Add monitoring alerts
# - Update AppArmor/SELinux policies
# - Implement additional input validation

# 3. Security audit
cargo audit
cargo clippy -- -D warnings
cargo test

# 4. Notify stakeholders
# - Security team
# - Management
# - Users (if applicable)
```

---

## Compliance

### GDPR Compliance

**Data Minimization:**
```toml
# prtip.toml - GDPR-compliant configuration

[output]
# Minimize data collection
include_timestamps = false     # Don't collect timestamps (unless required)
include_hostnames = false      # Don't resolve hostnames (IP only)
include_banners = false        # Don't collect service banners (unless required)

[database]
# Retention policy
enable_auto_delete = true      # Auto-delete old scans
retention_days = 30            # Keep scans for 30 days only

[audit]
# Audit log retention
audit_retention_days = 90      # Keep audit logs for 90 days
```

**Data Subject Rights:**
```bash
# Right to Access (Article 15)
prtip --export-scan-data --user scanner --output gdpr-export.json

# Right to Erasure (Article 17)
prtip --delete-scan-data --scan-id <scan-id>

# Right to Rectification (Article 16)
prtip --update-scan-data --scan-id <scan-id> --field target --value scanme.nmap.org
```

### PCI DSS Compliance

**Requirement 11.2 (Vulnerability Scanning):**
```bash
# Quarterly external scans
prtip -sS -sV -p 1-65535 --output-format pci-dss \
    --scan-type quarterly-external \
    --target public-facing-systems.txt \
    --output Q4-2025-external-scan.xml

# Requirement: After significant changes
prtip -sS -sV -p 1-65535 --output-format pci-dss \
    --scan-type post-change \
    --target cardholder-data-environment.txt \
    --output post-migration-scan.xml
```

**Requirement 11.3 (Penetration Testing):**
```bash
# Annual penetration testing (use with manual testing)
prtip -sS -sV -O -A -p- --output-format pci-dss \
    --scan-type annual-pentest \
    --target cde-systems.txt \
    --output 2025-annual-pentest.xml
```

### NIST CSF Compliance

**Identify Function:**
```bash
# Asset Discovery (ID.AM-1)
prtip -sS -p 80,443,22,3389 192.168.0.0/16 \
    --output asset-inventory.json

# Vulnerability Identification (ID.RA-1)
prtip -sS -sV -p- --version-intensity 9 \
    --target known-assets.txt \
    --output vulnerability-assessment.json
```

**Protect Function:**
```bash
# Access Control Verification (PR.AC-1)
prtip -sS -p 22,3389,5900 --target servers.txt \
    --output access-control-audit.json

# Network Segmentation (PR.AC-5)
prtip -sS -p 1-65535 --target dmz-hosts.txt \
    --output segmentation-validation.json
```

---

## Troubleshooting

### Common Issues

**1. Privilege Drop Fails:**
```bash
# Symptom
Error: Privilege drop failed: User 'scanner' not found

# Solution
sudo useradd -r -s /bin/false scanner
```

**2. Permission Denied (Capabilities):**
```bash
# Symptom
Error: Permission denied: Cannot create raw socket

# Solution (Linux)
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip

# Verify
getcap /usr/local/bin/prtip
```

**3. Firewall Blocks Legitimate Scans:**
```bash
# Symptom
Error: Network unreachable

# Solution (Linux - temporarily disable for testing)
sudo iptables -D OUTPUT -m owner --uid-owner scanner -d 203.0.113.0/24 -j DROP

# Or whitelist target
sudo iptables -I OUTPUT 1 -m owner --uid-owner scanner -d 203.0.113.0/24 -j ACCEPT
```

**4. AppArmor Denials:**
```bash
# Symptom
Error: Permission denied: /var/lib/prtip/scans/output.json

# Check AppArmor logs
sudo dmesg | grep apparmor | grep prtip

# Solution (add to profile)
/var/lib/prtip/scans/** rw,

# Reload profile
sudo apparmor_parser -r /etc/apparmor.d/usr.local.bin.prtip
```

**5. Resource Limit Exceeded:**
```bash
# Symptom
Error: Too many open files

# Solution (temporary)
ulimit -n 65535

# Solution (permanent)
echo "scanner soft nofile 65535" | sudo tee -a /etc/security/limits.conf
echo "scanner hard nofile 65535" | sudo tee -a /etc/security/limits.conf
```

---

## See Also

- [Security Model](security-model.md) - Comprehensive security architecture
- [Vulnerability Disclosure](vulnerability-disclosure.md) - Reporting security issues
- [Audit Log](audit-log.md) - Security audit history
- [Platform Support](../features/platform-support.md) - Platform-specific installation
- [Troubleshooting](../reference/troubleshooting.md) - General troubleshooting guide
