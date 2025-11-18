# Security Overview

**Last Updated:** 2025-11-15
**Version:** 2.0
**Security Contact:** [SECURITY.md](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md)

---

## Introduction

ProRT-IP WarScan is a network security scanner designed with security-first principles. As a tool that operates with elevated privileges and interacts with potentially hostile network environments, security is paramount to both the scanner's operation and the safety of systems running it.

This document provides a comprehensive overview of ProRT-IP's security architecture, implementation patterns, and best practices. It serves as the foundation for understanding how the scanner protects itself, users, and target networks from security vulnerabilities.

---

## Security Philosophy

ProRT-IP's security model is built on five core principles:

### 1. **Least Privilege**
Drop elevated privileges immediately after creating privileged resources. The scanner runs unprivileged for 99.9% of its execution time.

### 2. **Defense in Depth**
Multiple layers of validation and error handling ensure that a single failure doesn't compromise security.

### 3. **Fail Securely**
Errors and unexpected conditions never expose sensitive information or create security vulnerabilities. The scanner fails closed, not open.

### 4. **Input Validation**
All external input—network packets, user arguments, configuration files—is untrusted and rigorously validated.

### 5. **Memory Safety**
Leverage Rust's ownership system and type safety to prevent entire classes of vulnerabilities (buffer overflows, use-after-free, data races).

---

## Threat Model

### Assets to Protect

ProRT-IP protects four critical asset classes:

1. **Scanner Integrity**
   - Prevent exploitation of the scanner process itself
   - Protect against malicious network responses
   - Ensure accurate scan results

2. **Network Stability**
   - Avoid unintentional denial-of-service of target networks
   - Respect rate limits and resource constraints
   - Prevent network disruption

3. **Confidential Data**
   - Scan results may contain sensitive network topology
   - TLS certificates reveal organizational information
   - Service banners expose application versions

4. **Host System**
   - Prevent privilege escalation
   - Protect system resources (CPU, memory, disk)
   - Avoid system compromise through scanner vulnerabilities

### Threat Actors

ProRT-IP defends against four primary threat actors:

#### 1. Malicious Network Targets
**Threat:** Network hosts sending crafted responses to exploit scanner vulnerabilities.

**Examples:**
- Malformed TCP packets with invalid length fields
- Oversized service banners causing memory exhaustion
- Crafted TLS certificates triggering parser vulnerabilities

**Mitigations:**
- Robust packet parsing with bounds checking
- Memory limits on response data
- Fuzzing of all network protocol parsers

#### 2. Malicious Users
**Threat:** Scanner operators attempting to abuse the tool for attacks.

**Examples:**
- Internet-scale scans without authorization
- Denial-of-service attacks via high packet rates
- Command injection through configuration files

**Mitigations:**
- User confirmation for large-scale scans
- Rate limiting enforced by default
- Input validation on all user-controlled data

#### 3. Network Defenders
**Threat:** IDS/IPS systems attempting to detect and block scanner.

**Examples:**
- Signature-based detection of scan patterns
- Behavior-based anomaly detection
- IP-based blacklisting

**Mitigations:**
- Evasion techniques (timing randomization, fragmentation)
- Decoy scanning to obscure true source
- Idle scan for maximum anonymity

#### 4. Local Attackers
**Threat:** Unprivileged users attempting privilege escalation via scanner.

**Examples:**
- Exploiting setuid binaries
- Race conditions in privilege dropping
- Capability misuse

**Mitigations:**
- Linux capabilities instead of setuid root
- Immediate and irreversible privilege dropping
- Verification that privileges cannot be regained

---

## Core Security Components

### Privilege Management

ProRT-IP uses a **create-privileged, drop-immediately** pattern:

```rust
pub fn initialize_scanner() -> Result<Scanner> {
    // 1. Create privileged resources FIRST
    let raw_socket = create_raw_socket()?;  // Requires CAP_NET_RAW
    let pcap_handle = open_pcap_capture()?; // Requires CAP_NET_RAW

    // 2. Drop privileges IMMEDIATELY (irreversible)
    drop_privileges_safely("scanner", "scanner")?;

    // 3. Continue with unprivileged operations
    let scanner = Scanner::new(raw_socket, pcap_handle)?;

    Ok(scanner)
}
```

#### Linux Capabilities (Recommended)

Instead of setuid root (which grants all privileges), ProRT-IP uses Linux capabilities:

```bash
# Build the binary
cargo build --release

# Grant ONLY network packet capabilities
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Verify capabilities
getcap target/release/prtip
# Output: target/release/prtip = cap_net_admin,cap_net_raw+eip

# Now runs without root
./target/release/prtip -sS -p 80,443 192.168.1.1
```

**Security Properties:**
- ✅ No setuid root binary (massive attack surface reduction)
- ✅ Only CAP_NET_RAW and CAP_NET_ADMIN granted (minimal necessary)
- ✅ Capabilities dropped immediately after socket creation
- ✅ Cannot regain privileges after dropping (verified)

#### Privilege Dropping Implementation

```rust
use nix::unistd::{setuid, setgid, setgroups, Uid, Gid};
use caps::{Capability, CapSet};

pub fn drop_privileges_safely(username: &str, groupname: &str) -> Result<()> {
    // Step 1: Clear supplementary groups (requires root)
    setgroups(&[])?;

    // Step 2: Drop group privileges
    let group = Group::from_name(groupname)?
        .ok_or(Error::GroupNotFound)?;
    setgid(Gid::from_raw(group.gid))?;

    // Step 3: Drop user privileges (irreversible on Linux)
    let user = User::from_name(username)?
        .ok_or(Error::UserNotFound)?;
    setuid(Uid::from_raw(user.uid))?;

    // Step 4: VERIFY privileges cannot be regained
    assert!(setuid(Uid::from_raw(0)).is_err(), "Failed to drop privileges!");

    // Step 5: Drop remaining capabilities
    caps::clear(None, CapSet::Permitted)?;
    caps::clear(None, CapSet::Effective)?;

    tracing::info!("Privileges dropped to {}:{}", username, groupname);

    Ok(())
}
```

**Critical:** The assertion in Step 4 verifies that `setuid(0)` **fails**, confirming privileges were successfully and irreversibly dropped.

#### Windows Privilege Handling

Windows requires Administrator privileges for raw packet access via Npcap:

```rust
#[cfg(target_os = "windows")]
pub fn check_admin_privileges() -> Result<()> {
    use windows::Win32::Security::*;

    unsafe {
        let result = IsUserAnAdmin();

        if result == FALSE {
            return Err(Error::InsufficientPrivileges(
                "Administrator privileges required for raw packet access on Windows.\n\
                 Right-click the terminal and select 'Run as Administrator'."
            ));
        }
    }

    tracing::warn!("Running with Administrator privileges on Windows");
    Ok(())
}
```

**Security Note:** Windows privilege management is less granular than Linux. The scanner must run as Administrator, increasing attack surface. Users should:
- Use dedicated non-administrative user for daily operations
- Run scanner only when needed
- Consider virtualization for additional isolation

---

### Input Validation

All external input is untrusted and validated using allowlist-based approaches.

#### IP Address Validation

```rust
use std::net::IpAddr;

pub fn validate_ip_address(input: &str) -> Result<IpAddr> {
    // Use standard library parser (validates format)
    let ip = input.parse::<IpAddr>()
        .map_err(|_| Error::InvalidIpAddress(input.to_string()))?;

    // Reject reserved addresses
    match ip {
        IpAddr::V4(addr) => {
            if addr.is_unspecified() || addr.is_broadcast() {
                return Err(Error::InvalidIpAddress("reserved address"));
            }
            Ok(IpAddr::V4(addr))
        }
        IpAddr::V6(addr) => {
            if addr.is_unspecified() {
                return Err(Error::InvalidIpAddress("unspecified address"));
            }
            Ok(IpAddr::V6(addr))
        }
    }
}
```

**Validated Properties:**
- ✅ Valid IPv4/IPv6 format (via std::net parser)
- ✅ Not unspecified (0.0.0.0 or ::)
- ✅ Not broadcast (255.255.255.255)
- ✅ Returns structured IpAddr type (type safety)

#### CIDR Range Validation

```rust
use ipnetwork::IpNetwork;

pub fn validate_cidr(input: &str) -> Result<IpNetwork> {
    let network = input.parse::<IpNetwork>()
        .map_err(|e| Error::InvalidCidr(input.to_string(), e))?;

    // Reject overly broad scans without explicit confirmation
    match network {
        IpNetwork::V4(net) if net.prefix() < 8 => {
            return Err(Error::CidrTooBoard(
                "IPv4 networks larger than /8 (16.7M hosts) require --confirm-large-scan flag.\n\
                 This prevents accidental internet-scale scans."
            ));
        }
        IpNetwork::V6(net) if net.prefix() < 48 => {
            return Err(Error::CidrTooBoard(
                "IPv6 networks larger than /48 require --confirm-large-scan flag.\n\
                 This prevents accidental massive scans."
            ));
        }
        _ => Ok(network)
    }
}
```

**Safety Properties:**
- ✅ Prevents accidental internet-scale scans
- ✅ Requires explicit confirmation for large ranges
- ✅ IPv4 /8 = 16.7M hosts, IPv6 /48 = 1.2 quadrillion hosts
- ✅ User intent verification before resource-intensive operations

#### Port Range Validation

```rust
pub fn validate_port_range(start: u16, end: u16) -> Result<(u16, u16)> {
    // Port 0 is reserved
    if start == 0 {
        return Err(Error::InvalidPortRange("start port cannot be 0"));
    }

    // Logical range check
    if end < start {
        return Err(Error::InvalidPortRange("end port must be >= start port"));
    }

    // Warn on full port scan (informational, not error)
    if start == 1 && end == 65535 {
        tracing::warn!(
            "Scanning all 65,535 ports. This will take significant time.\n\
             Consider using -F (fast, top 100 ports) or -p 1-1000 for faster scans."
        );
    }

    Ok((start, end))
}
```

#### Path Traversal Prevention

```rust
use std::path::{Path, PathBuf};

pub fn validate_output_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);

    // Resolve to canonical path (follows symlinks, resolves ..)
    let canonical = path.canonicalize()
        .or_else(|_| {
            // If file doesn't exist yet, canonicalize parent directory
            let parent = path.parent()
                .ok_or(Error::InvalidPath("no parent directory"))?;
            let filename = path.file_name()
                .ok_or(Error::InvalidPath("no filename"))?;
            parent.canonicalize()
                .map(|p| p.join(filename))
        })?;

    // Define allowed output directories
    let allowed_dirs = vec![
        PathBuf::from("/tmp/prtip"),
        PathBuf::from("/var/lib/prtip"),
        std::env::current_dir()?,
        PathBuf::from(std::env::var("HOME")?).join(".prtip"),
    ];

    // Verify path is within allowed directories
    let is_allowed = allowed_dirs.iter().any(|allowed| {
        canonical.starts_with(allowed)
    });

    if !is_allowed {
        return Err(Error::PathTraversalAttempt(canonical));
    }

    // Reject suspicious patterns (defense in depth)
    let path_str = canonical.to_string_lossy();
    if path_str.contains("..") || path_str.contains('\0') {
        return Err(Error::SuspiciousPath(path_str.to_string()));
    }

    Ok(canonical)
}
```

**Attack Prevention:**
- ✅ Path traversal (../../etc/passwd) blocked by canonicalization + allowlist
- ✅ Null byte injection (\0) rejected
- ✅ Symlink attacks prevented by canonical path checking
- ✅ Directory traversal outside allowed paths rejected

#### Command Injection Prevention

**Rule:** Never construct shell commands from user input!

```rust
use std::process::Command;

// ❌ WRONG: Vulnerable to command injection
fn resolve_hostname_unsafe(hostname: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("nslookup {}", hostname))  // DANGER!
        .output()?;
    // Attacker input: "example.com; rm -rf /"
    // Executes: nslookup example.com; rm -rf /
}

// ✅ CORRECT: Direct process spawn, no shell
fn resolve_hostname_safe(hostname: &str) -> Result<String> {
    let output = Command::new("nslookup")
        .arg(hostname)  // Passed as separate argument, not interpolated
        .output()?;

    String::from_utf8(output.stdout)
        .map_err(|e| Error::Utf8Error(e))
}

// ✅ BEST: Use Rust library instead of external command
fn resolve_hostname_best(hostname: &str) -> Result<IpAddr> {
    use trust_dns_resolver::Resolver;

    let resolver = Resolver::from_system_conf()?;
    let response = resolver.lookup_ip(hostname)?;
    let addr = response.iter().next()
        .ok_or(Error::NoAddressFound)?;

    Ok(addr)
}
```

**Security Layers:**
1. **Best:** Pure Rust implementation (no external process)
2. **Good:** Direct process spawn with separate arguments
3. **Never:** Shell command interpolation

---

### Packet Parsing Safety

Network packets are untrusted input from potentially hostile sources. Packet parsers must handle malformed, truncated, and malicious packets gracefully.

#### Safe Parsing Pattern

```rust
pub fn parse_tcp_packet_safe(data: &[u8]) -> Option<TcpHeader> {
    // 1. Explicit length check BEFORE any access
    if data.len() < 20 {
        tracing::debug!("TCP packet too short: {} bytes (min 20)", data.len());
        return None;
    }

    // 2. Safe indexing with checked bounds
    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let ack = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

    // 3. Validate data offset field BEFORE using it
    let data_offset_raw = data[12] >> 4;
    let data_offset = (data_offset_raw as usize) * 4;

    if data_offset < 20 {
        tracing::debug!("Invalid TCP data offset: {} (min 20)", data_offset);
        return None;
    }

    if data_offset > data.len() {
        tracing::debug!(
            "TCP data offset {} exceeds packet length {}",
            data_offset,
            data.len()
        );
        return None;
    }

    // 4. Parse flags safely
    let flags = TcpFlags::from_bits_truncate(data[13]);

    // 5. Return structured data
    Some(TcpHeader {
        src_port,
        dst_port,
        seq,
        ack,
        flags,
        data_offset,
    })
}
```

**Safety Properties:**
- ✅ Length validated before any access
- ✅ No panic! on malformed packets (returns None)
- ✅ Length fields validated before use as indices
- ✅ Structured return type (not raw bytes)

#### Error Handling for Malformed Packets

```rust
// ❌ WRONG: panic! in packet parsing
fn parse_packet_wrong(data: &[u8]) -> TcpPacket {
    assert!(data.len() >= 20, "Packet too short!");  // PANIC!
    // Attacker sends 10-byte packet -> process crashes -> DoS
}

// ✅ CORRECT: Return Option/Result
fn parse_packet_correct(data: &[u8]) -> Option<TcpPacket> {
    if data.len() < 20 {
        return None;  // Graceful handling
    }
    // ... continue parsing
}

// ✅ BETTER: Log for debugging and monitoring
fn parse_packet_better(data: &[u8], source_ip: IpAddr) -> Option<TcpPacket> {
    if data.len() < 20 {
        tracing::debug!(
            "Ignoring short packet ({} bytes) from {}",
            data.len(),
            source_ip
        );
        return None;
    }
    // ... continue parsing
}
```

**Rule:** Packet parsing code must **never** panic. Malformed packets are expected in hostile network environments.

#### Using `pnet` for Safe Parsing

ProRT-IP uses the `pnet` crate for packet parsing, which provides automatic bounds checking:

```rust
use pnet::packet::tcp::{TcpPacket, TcpFlags};

pub fn parse_with_pnet(data: &[u8]) -> Option<TcpInfo> {
    // pnet::TcpPacket::new() performs bounds checking automatically
    let tcp = TcpPacket::new(data)?;  // Returns None if invalid

    Some(TcpInfo {
        src_port: tcp.get_source(),
        dst_port: tcp.get_destination(),
        flags: tcp.get_flags(),
        seq: tcp.get_sequence(),
        ack: tcp.get_acknowledgement(),
        window: tcp.get_window(),
    })
}
```

**Benefits:**
- ✅ Bounds checking built into pnet accessors
- ✅ Type-safe access to packet fields
- ✅ Well-tested library (used by production network tools)
- ✅ Returns None on invalid packets (no panic)

---

### DoS Prevention

ProRT-IP implements multiple layers of resource limiting to prevent denial-of-service, both accidental and intentional.

#### 1. Rate Limiting

All scan types enforce packet rate limits:

```rust
use governor::{Quota, RateLimiter, clock::DefaultClock};
use std::num::NonZeroU32;

pub struct ScanRateLimiter {
    limiter: RateLimiter<DefaultClock>,
    max_rate: u32,
}

impl ScanRateLimiter {
    pub fn new(packets_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(packets_per_second).unwrap());
        let limiter = RateLimiter::direct(quota);

        Self {
            limiter,
            max_rate: packets_per_second,
        }
    }

    pub async fn wait_for_permit(&self) {
        self.limiter.until_ready().await;
    }
}

// Usage in scanning loop
let rate_limiter = ScanRateLimiter::new(100_000);  // 100K pps max

for target in targets {
    rate_limiter.wait_for_permit().await;  // Blocks until rate limit allows
    send_packet(target).await?;
}
```

**Default Limits:**
- **T0 (Paranoid):** 10 packets/second
- **T1 (Sneaky):** 100 packets/second
- **T2 (Polite):** 1,000 packets/second
- **T3 (Normal):** 10,000 packets/second (default)
- **T4 (Aggressive):** 100,000 packets/second
- **T5 (Insane):** 1,000,000 packets/second (localhost only)

**Performance Impact:** -1.8% overhead (industry-leading efficiency)

#### 2. Connection Limits

Maximum concurrent connections prevent resource exhaustion:

```rust
use tokio::sync::Semaphore;

pub struct ConnectionLimiter {
    semaphore: Arc<Semaphore>,
    max_connections: usize,
}

impl ConnectionLimiter {
    pub fn new(max_connections: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            max_connections,
        }
    }

    pub async fn acquire(&self) -> SemaphorePermit<'_> {
        self.semaphore.acquire().await.unwrap()
    }
}

// Usage
let limiter = ConnectionLimiter::new(1000);  // Max 1000 concurrent

for target in targets {
    let _permit = limiter.acquire().await;  // Blocks if limit reached

    tokio::spawn(async move {
        scan_target(target).await;
        // _permit dropped here, slot freed
    });
}
```

**Benefits:**
- ✅ Prevents file descriptor exhaustion
- ✅ Bounds memory usage (each connection = memory)
- ✅ Prevents network congestion
- ✅ Automatic backpressure

#### 3. Memory Limits

Result buffering with automatic flushing prevents unbounded memory growth:

```rust
pub struct ResultBuffer {
    buffer: Vec<ScanResult>,
    max_size: usize,
    flush_tx: mpsc::Sender<Vec<ScanResult>>,
}

impl ResultBuffer {
    pub fn push(&mut self, result: ScanResult) -> Result<()> {
        self.buffer.push(result);

        // Flush when buffer reaches limit
        if self.buffer.len() >= self.max_size {
            self.flush()?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let batch = std::mem::replace(&mut self.buffer, Vec::new());
        self.flush_tx.send(batch)
            .map_err(|_| Error::FlushFailed)?;

        Ok(())
    }
}
```

**Memory Characteristics:**
- **Stateless Scans:** <100 MB typical, linear scaling (2 MB + ports × 1.0 KB)
- **Service Detection:** 493 MB/port (recommend limiting to 10-20 ports)
- **Buffering:** 1,000-10,000 results per batch (configurable)

#### 4. Scan Duration Limits

Timeouts prevent runaway scans:

```rust
pub struct ScanExecutor {
    config: ScanConfig,
    start_time: Instant,
}

impl ScanExecutor {
    pub async fn execute(&self) -> Result<ScanReport> {
        let timeout = self.config.max_duration
            .unwrap_or(Duration::from_secs(3600)); // Default 1 hour

        tokio::select! {
            result = self.run_scan() => {
                result
            }
            _ = tokio::time::sleep(timeout) => {
                tracing::warn!("Scan exceeded maximum duration of {:?}", timeout);
                Err(Error::ScanTimeout(timeout))
            }
        }
    }
}
```

---

### Secrets Management

#### Environment Variables (Preferred)

```rust
use std::env;

pub struct Credentials {
    pub db_password: String,
    pub api_key: Option<String>,
}

impl Credentials {
    pub fn from_env() -> Result<Self> {
        let db_password = env::var("PRTIP_DB_PASSWORD")
            .map_err(|_| Error::MissingCredential("PRTIP_DB_PASSWORD"))?;

        let api_key = env::var("PRTIP_API_KEY").ok();

        Ok(Self {
            db_password,
            api_key,
        })
    }
}
```

#### Configuration File Security

```rust
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let permissions = metadata.permissions();

        #[cfg(unix)]
        {
            let mode = permissions.mode();
            // Must be 0600 or 0400 (owner read/write or owner read-only)
            if mode & 0o077 != 0 {
                return Err(Error::InsecureConfigPermissions(
                    format!("Config file {:?} has insecure permissions: {:o}", path, mode)
                ));
            }
        }

        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }
}
```

**Best Practices:**
- ✅ Use environment variables for secrets (12-factor app)
- ✅ Config files must be 0600 or 0400 permissions
- ✅ Never log secrets (even in debug mode)
- ✅ Redact secrets in error messages

---

## Secure Development Practices

### 1. Avoid Integer Overflows

```rust
// ❌ WRONG: Can overflow
fn calculate_buffer_size(count: u32, size_per_item: u32) -> usize {
    (count * size_per_item) as usize  // May wrap around!
}

// ✅ CORRECT: Check for overflow
fn calculate_buffer_size_safe(count: u32, size_per_item: u32) -> Result<usize> {
    count.checked_mul(size_per_item)
        .ok_or(Error::IntegerOverflow)?
        .try_into()
        .map_err(|_| Error::IntegerOverflow)
}

// ✅ BETTER: Use saturating arithmetic when appropriate
fn calculate_buffer_size_saturating(count: u32, size_per_item: u32) -> usize {
    count.saturating_mul(size_per_item) as usize
}
```

### 2. Prevent Time-of-Check to Time-of-Use (TOCTOU)

```rust
// ❌ WRONG: File could change between check and open
if Path::new(&filename).exists() {
    let file = File::open(&filename)?;  // TOCTOU race!
}

// ✅ CORRECT: Open directly and handle error
let file = match File::open(&filename) {
    Ok(f) => f,
    Err(e) if e.kind() == io::ErrorKind::NotFound => {
        return Err(Error::FileNotFound(filename));
    }
    Err(e) => return Err(Error::IoError(e)),
};
```

### 3. Cryptographically Secure RNG

```rust
use rand::rngs::OsRng;
use rand::RngCore;

// ✅ CORRECT: Use cryptographically secure RNG for security-sensitive values
fn generate_sequence_number() -> u32 {
    let mut rng = OsRng;
    rng.next_u32()
}

// ❌ WRONG: Thread RNG is fast but not cryptographically secure
fn generate_sequence_number_weak() -> u32 {
    use rand::thread_rng;
    let mut rng = thread_rng();
    rng.next_u32()  // Predictable for security purposes!
}
```

**Use Cases for Cryptographic RNG:**
- TCP sequence numbers (idle scan requires unpredictability)
- IP ID values (fingerprinting requires randomness)
- Source port selection (evasion benefits from randomness)

### 4. Constant-Time Comparisons

```rust
use subtle::ConstantTimeEq;

// ✅ CORRECT: Constant-time comparison prevents timing attacks
fn verify_api_key(provided: &str, expected: &str) -> bool {
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}

// ❌ WRONG: Early exit on mismatch leaks information via timing
fn verify_api_key_weak(provided: &str, expected: &str) -> bool {
    provided == expected  // Timing attack vulnerable!
}
```

---

## Security Audit Process

### Pre-Release Security Checklist

ProRT-IP follows a comprehensive security audit process before each release:

#### 1. Privilege Management
- ✅ Privileges dropped immediately after socket creation
- ✅ Cannot regain elevated privileges after dropping
- ✅ Capabilities documented and minimal
- ✅ Windows admin requirement documented

#### 2. Input Validation
- ✅ All user input validated with allowlists
- ✅ Path traversal attempts rejected
- ✅ No command injection vectors
- ✅ CIDR ranges size-limited
- ✅ Port ranges validated

#### 3. Packet Parsing
- ✅ All packet parsers handle malformed input
- ✅ No panics in packet parsing code
- ✅ Length fields validated before use
- ✅ No buffer overruns possible
- ✅ Using `pnet` for bounds-checked parsing

#### 4. Resource Limits
- ✅ Rate limiting enforced by default
- ✅ Connection limits enforced
- ✅ Memory usage bounded
- ✅ Scan duration limits enforced

#### 5. Secrets Management
- ✅ No hardcoded credentials
- ✅ Config files have secure permissions (0600)
- ✅ Secrets not logged
- ✅ Environment variables used for sensitive data

#### 6. Dependencies
- ✅ `cargo audit` passes with no critical vulnerabilities
- ✅ All dependencies from crates.io (no git dependencies)
- ✅ SBOM (Software Bill of Materials) generated
- ✅ Dependency versions pinned

#### 7. Fuzzing
- ✅ Packet parsers fuzzed for 24+ hours (230M+ executions)
- ✅ CLI argument parsing fuzzed
- ✅ Configuration file parsing fuzzed
- ✅ Zero crashes detected in fuzzing

#### 8. Code Review
- ✅ No `unsafe` blocks without justification
- ✅ All `unsafe` blocks audited and documented
- ✅ No TODO/FIXME in security-critical code
- ✅ Clippy lints enforced (`-D warnings`)

### Continuous Security Monitoring

- **GitHub Security Advisories:** Automated dependency scanning
- **CodeQL Analysis:** Static analysis on every commit
- **Cargo Audit:** Weekly security audit in CI/CD
- **Fuzzing:** Continuous fuzzing in development

---

## Vulnerability Reporting

ProRT-IP takes security vulnerabilities seriously.

### Reporting Process

**DO NOT** open public GitHub issues for security vulnerabilities.

**Instead:**

1. **Email:** security[at]prtip.example.com (replace with actual contact)
2. **PGP Key:** Available at https://github.com/doublegate/ProRT-IP/security/policy
3. **Expected Response:** Within 48 hours

### Report Should Include

- **Description:** What is the vulnerability?
- **Impact:** What can an attacker do?
- **Affected Versions:** Which versions are vulnerable?
- **Reproduction:** Steps to reproduce the issue
- **Proof of Concept:** Code or commands demonstrating the vulnerability

### What to Expect

1. **Acknowledgment:** Within 48 hours
2. **Assessment:** Within 1 week (severity, scope, impact)
3. **Fix Development:** Timeline based on severity
4. **Security Advisory:** Published after fix is available
5. **Credit:** Reporter credited in advisory (if desired)

### Severity Levels

- **Critical:** Immediate release (e.g., remote code execution)
- **High:** Release within 7 days (e.g., privilege escalation)
- **Medium:** Release within 30 days (e.g., information disclosure)
- **Low:** Next regular release (e.g., minor info leak)

---

## Compliance and Standards

ProRT-IP aligns with industry security standards and best practices:

### Security Standards

- **OWASP Top 10:** Protection against common web application vulnerabilities
- **CWE Top 25:** Mitigation of most dangerous software weaknesses
- **NIST Guidelines:** Following NIST SP 800-115 (Technical Security Testing)

### Responsible Use

ProRT-IP is a **penetration testing and security auditing tool**. Users must:

1. **Authorization:** Obtain written authorization before scanning
2. **Scope:** Only scan authorized systems and networks
3. **Legal Compliance:** Follow applicable laws and regulations
4. **Network Safety:** Use rate limiting to avoid network disruption
5. **Data Protection:** Protect scan results (may contain sensitive data)

See [Responsible Use Guidelines](./responsible-use.md) for detailed guidance.

### Audit Checklist

For organizations deploying ProRT-IP, see [Security Audit Checklist](./audit-checklist.md) for:

- Pre-deployment security review
- Operational security procedures
- Post-scan security verification
- Compliance documentation

---

## Related Documentation

### Security Documentation

- **[Responsible Use Guidelines](./responsible-use.md)** - Legal and ethical considerations
- **[Security Audit Checklist](./audit-checklist.md)** - Pre-deployment, operational, post-scan reviews
- **[Compliance](./compliance.md)** - Industry standards, regulatory requirements, best practices

### Technical Documentation

- **[Architecture Overview](../development/architecture.md)** - Security boundaries and trust zones
- **[Testing Strategy](../development/testing.md)** - Security testing approaches
- **[Development Guide](../development/implementation.md)** - Secure coding practices

### Feature Guides

- **[Rate Limiting](../features/rate-limiting.md)** - DoS prevention and network courtesy
- **[Firewall Evasion](../features/evasion-techniques.md)** - Stealth scanning techniques
- **[Plugin System](../features/plugin-system.md)** - Sandboxed plugin security

---

## Security by Design

ProRT-IP's architecture embeds security at every layer:

### 1. Type Safety
Rust's ownership system prevents:
- Buffer overflows
- Use-after-free
- Data races
- Null pointer dereferences

### 2. Memory Safety
Zero `unsafe` blocks in critical security code:
- Packet parsing (uses `pnet` with bounds checking)
- Input validation (pure safe Rust)
- Privilege management (uses `nix` and `caps` crates)

### 3. Fail-Safe Defaults
- Rate limiting enabled by default (T3 = 10K pps)
- Large scan confirmation required (prevents accidents)
- Secure config permissions enforced (0600)
- Privileges dropped immediately after initialization

### 4. Defense in Depth
Multiple validation layers:
- Input validation (allowlist-based)
- Packet parsing (bounds checking)
- Resource limits (rate, memory, connections, duration)
- Error handling (no information leaks)

### 5. Least Privilege
Minimal permissions required:
- Linux: Only CAP_NET_RAW and CAP_NET_ADMIN
- Privileges dropped after socket creation
- Runs unprivileged for 99.9% of execution
- Cannot regain privileges (verified)

---

## Security Testing

ProRT-IP undergoes rigorous security testing:

### Fuzzing Results

**Current Status (v0.5.2):**
- **Total Executions:** 230,045,372 (230M+)
- **Crashes:** 0
- **Hangs:** 0
- **Targets:** 5 fuzz targets
- **Seeds:** 807 generated

**Fuzz Targets:**
1. **IPv4 Packet Parser** - 52.3M executions
2. **IPv6 Packet Parser** - 48.1M executions
3. **Service Detection** - 45.7M executions
4. **TLS Certificate Parser** - 42.9M executions
5. **CLI Argument Parser** - 41.0M executions

**Methodology:**
- Structure-aware fuzzing using `arbitrary` crate
- 24+ hours continuous fuzzing per target
- Coverage-guided fuzzing (libFuzzer)
- Regression testing with seed corpus

### Test Coverage

- **Total Tests:** 2,111 (100% passing)
- **Code Coverage:** 54.92%
- **Security-Critical Code Coverage:** >90%

**Security Test Categories:**
1. Input validation tests (247 tests)
2. Packet parsing malformed input tests (156 tests)
3. Privilege dropping verification tests (23 tests)
4. Resource limit enforcement tests (89 tests)
5. Secrets management tests (34 tests)

### Static Analysis

- **Clippy:** Zero warnings (`-D warnings` enforced)
- **CodeQL:** Continuous scanning on all commits
- **Cargo Audit:** Weekly dependency vulnerability scanning
- **RUSTSEC:** Monitored for security advisories

---

## Conclusion

ProRT-IP's security model is built on:

1. **Least Privilege** - Minimal permissions, immediately dropped
2. **Input Validation** - All external input rigorously validated
3. **Memory Safety** - Rust's guarantees prevent entire vulnerability classes
4. **Resource Limits** - DoS prevention at multiple layers
5. **Defense in Depth** - Multiple validation and error handling layers
6. **Secure by Default** - Safe defaults, explicit confirmation for risky operations
7. **Continuous Testing** - Fuzzing, static analysis, security audits

For security questions, vulnerability reports, or general inquiries:
- **Email:** security[at]prtip.example.com
- **Repository:** https://github.com/doublegate/ProRT-IP
- **Security Policy:** [SECURITY.md](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md)

**Remember:** ProRT-IP is a powerful security tool. With great power comes great responsibility. Always obtain authorization before scanning, use rate limiting to avoid network disruption, and protect scan results containing sensitive information.
