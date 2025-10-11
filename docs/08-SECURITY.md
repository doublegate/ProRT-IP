# ProRT-IP WarScan: Security Implementation Guide

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [Security Overview](#security-overview)
2. [Privilege Management](#privilege-management)
3. [Input Validation](#input-validation)
4. [Packet Parsing Safety](#packet-parsing-safety)
5. [DoS Prevention](#dos-prevention)
6. [Secrets Management](#secrets-management)
7. [Secure Coding Practices](#secure-coding-practices)
8. [Security Audit Checklist](#security-audit-checklist)

---

## Security Overview

### Security Principles

1. **Least Privilege:** Drop privileges immediately after creating privileged resources
2. **Defense in Depth:** Multiple layers of validation and error handling
3. **Fail Securely:** Errors should not expose sensitive information or create vulnerabilities
4. **Input Validation:** All external input is untrusted and must be validated
5. **Memory Safety:** Leverage Rust's guarantees to prevent memory corruption

### Threat Model

#### Assets to Protect

- **Scanner integrity:** Prevent exploitation of the scanner process
- **Network stability:** Avoid unintentional DoS of target networks
- **Confidential data:** Scan results may contain sensitive information
- **Host system:** Prevent privilege escalation or system compromise

#### Threat Actors

1. **Malicious targets:** Network hosts sending crafted responses to exploit scanner
2. **Malicious users:** Operators attempting to abuse scanner for attacks
3. **Network defenders:** IDS/IPS systems attempting to detect scanner
4. **Local attackers:** Unprivileged users trying to escalate via scanner

---

## Privilege Management

### The Privilege Dropping Pattern

**Critical:** Raw packet capabilities are only needed during socket creation. Drop privileges **immediately** after.

#### Linux Capabilities (Recommended)

```rust
use nix::unistd::{setuid, setgid, setgroups, Uid, Gid};
use nix::sys::stat::Mode;
use caps::{Capability, CapSet};

pub fn drop_privileges_safely(username: &str, groupname: &str) -> Result<()> {
    // Step 1: Clear supplementary groups (requires root)
    setgroups(&[])?;

    // Step 2: Drop group privileges
    let group = Group::from_name(groupname)?
        .ok_or(Error::GroupNotFound)?;
    setgid(Gid::from_raw(group.gid))?;

    // Step 3: Drop user privileges (irreversible)
    let user = User::from_name(username)?
        .ok_or(Error::UserNotFound)?;
    setuid(Uid::from_raw(user.uid))?;

    // Step 4: Verify we cannot regain privileges
    assert!(setuid(Uid::from_raw(0)).is_err(), "Failed to drop privileges!");

    // Step 5: Drop remaining capabilities (keep only necessary ones)
    caps::clear(None, CapSet::Permitted)?;
    caps::clear(None, CapSet::Effective)?;

    tracing::info!("Privileges dropped to {}:{}", username, groupname);

    Ok(())
}
```

#### Usage Pattern

```rust
pub fn initialize_scanner() -> Result<Scanner> {
    // 1. Create privileged resources FIRST
    let raw_socket = create_raw_socket()?;  // Requires CAP_NET_RAW
    let pcap_handle = open_pcap_capture()?; // Requires CAP_NET_RAW

    // 2. Drop privileges IMMEDIATELY
    drop_privileges_safely("scanner", "scanner")?;

    // 3. Continue with unprivileged operations
    let scanner = Scanner::new(raw_socket, pcap_handle)?;

    Ok(scanner)
}
```

### Grant Capabilities Without setuid Root

```bash
# Build the binary
cargo build --release

# Grant specific capabilities (instead of setuid root)
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Verify
getcap target/release/prtip
# Output: target/release/prtip = cap_net_admin,cap_net_raw+eip

# Now runs without root
./target/release/prtip [args]
```

### Windows Privilege Management

```rust
#[cfg(target_os = "windows")]
pub fn check_admin_privileges() -> Result<()> {
    use windows::Win32::Security::*;
    use windows::Win32::Foundation::*;

    unsafe {
        let mut is_admin = FALSE;
        let result = IsUserAnAdmin();

        if result == FALSE {
            return Err(Error::InsufficientPrivileges(
                "Administrator privileges required for raw packet access on Windows"
            ));
        }
    }

    Ok(())
}
```

---

## Input Validation

### IP Address Validation

```rust
use std::net::IpAddr;

pub fn validate_ip_address(input: &str) -> Result<IpAddr> {
    // Use standard library parser (already validates format)
    let ip = input.parse::<IpAddr>()
        .map_err(|_| Error::InvalidIpAddress(input.to_string()))?;

    // Additional checks
    match ip {
        IpAddr::V4(addr) => {
            // Reject unspecified/broadcast
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

### CIDR Validation

```rust
use ipnetwork::IpNetwork;

pub fn validate_cidr(input: &str) -> Result<IpNetwork> {
    let network = input.parse::<IpNetwork>()
        .map_err(|e| Error::InvalidCidr(input.to_string(), e))?;

    // Reject overly broad scans without confirmation
    match network {
        IpNetwork::V4(net) if net.prefix() < 8 => {
            return Err(Error::CidrTooBoard(
                "IPv4 networks larger than /8 require --confirm-large-scan"
            ));
        }
        IpNetwork::V6(net) if net.prefix() < 48 => {
            return Err(Error::CidrTooBoard(
                "IPv6 networks larger than /48 require --confirm-large-scan"
            ));
        }
        _ => Ok(network)
    }
}
```

### Port Range Validation

```rust
pub fn validate_port_range(start: u16, end: u16) -> Result<(u16, u16)> {
    if start == 0 {
        return Err(Error::InvalidPortRange("start port cannot be 0"));
    }

    if end < start {
        return Err(Error::InvalidPortRange("end port < start port"));
    }

    // Warn on full port scan
    if start == 1 && end == 65535 {
        tracing::warn!("Scanning all 65535 ports - this will take significant time");
    }

    Ok((start, end))
}
```

### Filename Validation (Path Traversal Prevention)

```rust
use std::path::{Path, PathBuf};

pub fn validate_output_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);

    // Resolve to canonical path
    let canonical = path.canonicalize()
        .or_else(|_| {
            // If file doesn't exist yet, canonicalize parent
            let parent = path.parent()
                .ok_or(Error::InvalidPath("no parent directory"))?;
            let filename = path.file_name()
                .ok_or(Error::InvalidPath("no filename"))?;
            parent.canonicalize()
                .map(|p| p.join(filename))
        })?;

    // Ensure path doesn't escape allowed directories
    let allowed_dirs = vec![
        PathBuf::from("/tmp/prtip"),
        PathBuf::from("/var/lib/prtip"),
        std::env::current_dir()?,
    ];

    let is_allowed = allowed_dirs.iter().any(|allowed| {
        canonical.starts_with(allowed)
    });

    if !is_allowed {
        return Err(Error::PathTraversalAttempt(canonical));
    }

    // Reject suspicious patterns
    let path_str = canonical.to_string_lossy();
    if path_str.contains("..") || path_str.contains('\0') {
        return Err(Error::SuspiciousPath(path_str.to_string()));
    }

    Ok(canonical)
}
```

### Command Injection Prevention

**Never construct shell commands from user input!**

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

// ✅ CORRECT: Direct process spawn, no shell interpretation
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

---

## Packet Parsing Safety

### Safe Packet Parsing Pattern

```rust
pub fn parse_tcp_packet_safe(data: &[u8]) -> Option<TcpHeader> {
    // 1. Explicit length check BEFORE any access
    if data.len() < 20 {
        tracing::warn!("TCP packet too short: {} bytes", data.len());
        return None;
    }

    // 2. Use safe indexing or validated slices
    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let ack = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

    // 3. Validate data offset field before trusting it
    let data_offset_raw = data[12] >> 4;
    let data_offset = (data_offset_raw as usize) * 4;

    if data_offset < 20 {
        tracing::warn!("Invalid TCP data offset: {}", data_offset);
        return None;
    }

    if data_offset > data.len() {
        tracing::warn!(
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

### Error Handling for Malformed Packets

```rust
// ❌ WRONG: panic! in packet parsing
fn parse_packet_wrong(data: &[u8]) -> TcpPacket {
    assert!(data.len() >= 20, "Packet too short!");  // PANIC!
    // Attacker sends 10-byte packet -> process crashes
}

// ✅ CORRECT: Return Option/Result
fn parse_packet_correct(data: &[u8]) -> Option<TcpPacket> {
    if data.len() < 20 {
        return None;  // Graceful handling
    }
    // ... continue parsing
}

// ✅ BETTER: Log and continue
fn parse_packet_better(data: &[u8]) -> Option<TcpPacket> {
    if data.len() < 20 {
        tracing::debug!(
            "Ignoring short packet ({} bytes) from {:?}",
            data.len(),
            source_ip
        );
        return None;
    }
    // ... continue parsing
}
```

### Using `pnet` for Safe Parsing

```rust
use pnet::packet::tcp::{TcpPacket, TcpFlags};

pub fn parse_with_pnet(data: &[u8]) -> Option<TcpInfo> {
    // pnet performs bounds checking automatically
    let tcp = TcpPacket::new(data)?;  // Returns None if invalid

    Some(TcpInfo {
        src_port: tcp.get_source(),
        dst_port: tcp.get_destination(),
        flags: tcp.get_flags(),
        // ... other fields
    })
}
```

---

## DoS Prevention

### Rate Limiting

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

    pub fn try_acquire(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

// Usage in scanning loop
let rate_limiter = ScanRateLimiter::new(100_000);  // 100K pps max

for target in targets {
    rate_limiter.wait_for_permit().await;
    send_packet(target).await?;
}
```

### Connection Limits

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
let limiter = ConnectionLimiter::new(1000);  // Max 1000 concurrent connections

for target in targets {
    let _permit = limiter.acquire().await;  // Blocks if limit reached

    tokio::spawn(async move {
        scan_target(target).await;
        // _permit dropped here, slot freed
    });
}
```

### Memory Limits

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

### Scan Duration Limits

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
                Err(Error::ScanTimeout(timeout))
            }
        }
    }
}
```

---

## Secrets Management

### Configuration Files

```rust
use serde::{Deserialize, Serialize};
use std::fs::{Permissions, set_permissions};
use std::os::unix::fs::PermissionsExt;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub database_url: Option<String>,
    // ... other config
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        // Check file permissions
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

        // Load and parse config
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;

        // Set secure permissions
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o600);
            set_permissions(path, perms)?;
        }

        Ok(())
    }
}
```

### Environment Variables (Preferred)

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

// Usage
let creds = Credentials::from_env()?;
let db = connect_database(&creds.db_password)?;
```

### Never Log Secrets

```rust
use tracing::{info, warn};

// ❌ WRONG: Logs password
info!("Connecting to database with password: {}", password);

// ✅ CORRECT: Redact secrets
info!("Connecting to database at {}", db_url.host());

// ✅ BETTER: Use structured logging with filtering
info!(
    db_host = %db_url.host(),
    db_port = db_url.port(),
    "Connecting to database"
);
// Password field omitted entirely
```

---

## Secure Coding Practices

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

// ✅ BETTER: Use saturating arithmetic when wrapping is acceptable
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

### 3. Secure Random Number Generation

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

### 4. Constant-Time Comparisons (for secrets)

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

## Security Audit Checklist

### Pre-Release Security Audit

- [ ] **Privilege Management**
  - [ ] Privileges dropped immediately after socket creation
  - [ ] Cannot regain elevated privileges after dropping
  - [ ] Capabilities documented and minimal

- [ ] **Input Validation**
  - [ ] All user input validated with allowlists
  - [ ] Path traversal attempts rejected
  - [ ] No command injection vectors
  - [ ] CIDR ranges size-limited

- [ ] **Packet Parsing**
  - [ ] All packet parsers handle malformed input
  - [ ] No panics in packet parsing code
  - [ ] Length fields validated before use
  - [ ] No buffer overruns possible

- [ ] **Resource Limits**
  - [ ] Rate limiting enforced
  - [ ] Connection limits enforced
  - [ ] Memory usage bounded
  - [ ] Scan duration limits enforced

- [ ] **Secrets Management**
  - [ ] No hardcoded credentials
  - [ ] Config files have secure permissions
  - [ ] Secrets not logged
  - [ ] Environment variables used for sensitive data

- [ ] **Dependencies**
  - [ ] `cargo audit` passes with no criticals
  - [ ] All dependencies from crates.io (no git deps)
  - [ ] SBOM (Software Bill of Materials) generated

- [ ] **Fuzzing**
  - [ ] Packet parsers fuzzed for 24+ hours
  - [ ] CLI argument parsing fuzzed
  - [ ] Configuration file parsing fuzzed

- [ ] **Code Review**
  - [ ] No `unsafe` blocks without justification
  - [ ] All `unsafe` blocks audited
  - [ ] No TODO/FIXME in security-critical code

---

## Next Steps

- Review [Testing Strategy](06-TESTING.md) for security testing
- Consult [Architecture](00-ARCHITECTURE.md) for security boundaries
- See [Development Setup](03-DEV-SETUP.md) for secure build configuration
