# Testing Infrastructure

Comprehensive guide to ProRT-IP's test infrastructure including test organization, mocking frameworks, test utilities, error injection, and supporting tools.

## Overview

ProRT-IP's test infrastructure provides robust support for comprehensive testing across all components. The infrastructure includes:

- **Test Organization:** Common modules, fixtures, and platform-specific utilities
- **Error Injection Framework:** Deterministic error simulation for robustness testing
- **Mock Services:** Docker Compose environments and mock servers
- **Test Utilities:** Binary discovery, execution helpers, assertion utilities
- **Test Isolation:** Environment variables and concurrent test safety
- **Platform Support:** Cross-platform utilities with conditional compilation

**Key Metrics:**
- **Test Count:** 2,111 tests (100% passing)
- **Coverage:** 54.92% overall, 90%+ core modules
- **Test Infrastructure:** 500+ lines of utilities, 11 failure modes, 4 mock services
- **Platforms Tested:** Linux, macOS, Windows via GitHub Actions CI/CD

## Test Organization

### Directory Structure

```
ProRT-IP/
├── tests/                           # Integration tests
│   ├── common/                      # Top-level test utilities
│   │   └── mod.rs                   # Shared helpers
│   └── fixtures/                    # Test data
│       ├── sample_targets.json
│       ├── nmap_compatible_flags.json
│       └── expected_outputs.json
│
├── crates/
│   ├── prtip-cli/
│   │   └── tests/
│   │       ├── common/              # CLI test utilities
│   │       │   └── mod.rs           # Binary discovery, execution
│   │       └── fixtures/            # CLI-specific test data
│   │
│   ├── prtip-scanner/
│   │   └── tests/
│   │       ├── common/              # Scanner test utilities
│   │       │   ├── mod.rs           # Module declarations
│   │       │   └── error_injection.rs  # Error injection framework
│   │       └── integration/         # Integration tests
│   │
│   ├── prtip-network/
│   │   └── tests/                   # Network layer tests
│   │
│   ├── prtip-service-detection/
│   │   └── tests/                   # Service detection tests
│   │
│   └── prtip-tui/
│       └── tests/                   # TUI component tests
```

### Common Test Modules

**Purpose:** Shared test utilities across crates to reduce duplication and ensure consistency.

**Top-Level Common Module** (`tests/common/mod.rs`):
- Workspace-level shared utilities
- Cross-crate test helpers
- Minimal to avoid circular dependencies

**CLI Common Module** (`crates/prtip-cli/tests/common/mod.rs`):
- Binary discovery and execution
- CLI test isolation
- Assertion utilities
- Privilege detection
- Echo server for integration tests

**Scanner Common Module** (`crates/prtip-scanner/tests/common/`):
- Error injection framework
- Mock target servers
- Response validation

**Benefits:**
- **DRY Principle:** Reusable utilities across test suites
- **Consistency:** Standardized test patterns
- **Maintainability:** Centralized utility updates
- **Isolation:** Per-crate utilities prevent coupling

## Error Injection Framework

### Overview

The error injection framework provides deterministic simulation of network failures for robustness testing. Located in `crates/prtip-scanner/tests/common/error_injection.rs`.

**Purpose:**
- Test retry logic and error handling
- Validate timeout behavior
- Simulate transient vs permanent failures
- Verify graceful degradation

### FailureMode Enum

Defines 11 failure modes for comprehensive error simulation:

```rust
#[derive(Debug, Clone)]
pub enum FailureMode {
    /// Connection refused (ECONNREFUSED)
    ConnectionRefused,

    /// Operation timed out (ETIMEDOUT)
    Timeout(Duration),

    /// Network unreachable (ENETUNREACH)
    NetworkUnreachable,

    /// Host unreachable (EHOSTUNREACH)
    HostUnreachable,

    /// Connection reset by peer (ECONNRESET)
    ConnectionReset,

    /// Connection aborted (ECONNABORTED)
    ConnectionAborted,

    /// Would block / try again (EWOULDBLOCK)
    WouldBlock,

    /// Operation interrupted (EINTR)
    Interrupted,

    /// Too many open files (EMFILE)
    TooManyOpenFiles,

    /// Malformed response (truncated data)
    MalformedResponse { data: Vec<u8> },

    /// Invalid encoding (bad UTF-8)
    InvalidEncoding { data: Vec<u8> },

    /// Success after N attempts (retry testing)
    SuccessAfter { attempts: u32 },

    /// Probabilistic failure (0.0 = never, 1.0 = always)
    Probabilistic { rate: f64 },
}
```

**Error Classification:**

```rust
impl FailureMode {
    /// Convert to io::Error
    pub fn to_io_error(&self) -> io::Result<()> {
        match self {
            Self::ConnectionRefused => {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "connection refused"
                ))
            }
            Self::Timeout(_) => {
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "operation timed out"
                ))
            }
            Self::NetworkUnreachable => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "network unreachable"
                ))
            }
            // ... other error types
            _ => Ok(()),
        }
    }

    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_)
                | Self::WouldBlock
                | Self::Interrupted
                | Self::ConnectionReset
                | Self::ConnectionAborted
                | Self::TooManyOpenFiles
        )
    }
}
```

**Retriable Errors:**
- `Timeout`: Network congestion, slow response
- `WouldBlock`: Non-blocking socket not ready
- `Interrupted`: Signal interruption (EINTR)
- `ConnectionReset`: Peer closed connection abruptly
- `ConnectionAborted`: Local connection abort
- `TooManyOpenFiles`: Resource exhaustion (may recover)

**Non-Retriable Errors:**
- `ConnectionRefused`: Port closed, service down
- `NetworkUnreachable`: Routing failure
- `HostUnreachable`: Target not reachable

### ErrorInjector Usage

```rust
pub struct ErrorInjector {
    target: SocketAddr,
    failure_mode: FailureMode,
    attempt_count: std::cell::RefCell<u32>,
}

impl ErrorInjector {
    pub fn new(target: SocketAddr, failure_mode: FailureMode) -> Self {
        Self {
            target,
            failure_mode,
            attempt_count: std::cell::RefCell::new(0),
        }
    }

    pub fn inject_connection_error(&self) -> io::Result<()> {
        let mut count = self.attempt_count.borrow_mut();
        *count += 1;

        match &self.failure_mode {
            FailureMode::SuccessAfter { attempts } => {
                if *count >= *attempts {
                    Ok(())
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::ConnectionRefused,
                        "not yet"
                    ))
                }
            }
            FailureMode::Probabilistic { rate } => {
                use rand::Rng;
                if rand::thread_rng().gen::<f64>() < *rate {
                    Err(io::Error::new(
                        io::ErrorKind::ConnectionRefused,
                        "probabilistic failure"
                    ))
                } else {
                    Ok(())
                }
            }
            _ => self.failure_mode.to_io_error(),
        }
    }

    pub fn attempt_count(&self) -> u32 {
        *self.attempt_count.borrow()
    }

    pub fn reset(&self) {
        *self.attempt_count.borrow_mut() = 0;
    }
}
```

**Example: Retry Testing**

```rust
#[test]
fn test_retry_logic() {
    let target = "127.0.0.1:8080".parse().unwrap();

    // Succeed after 3 attempts
    let injector = ErrorInjector::new(
        target,
        FailureMode::SuccessAfter { attempts: 3 }
    );

    // First two attempts fail
    assert!(injector.inject_connection_error().is_err());
    assert!(injector.inject_connection_error().is_err());

    // Third attempt succeeds
    assert!(injector.inject_connection_error().is_ok());
    assert_eq!(injector.attempt_count(), 3);
}
```

**Example: Probabilistic Failures**

```rust
#[test]
fn test_probabilistic_failure() {
    let target = "127.0.0.1:8080".parse().unwrap();

    // 50% failure rate
    let injector = ErrorInjector::new(
        target,
        FailureMode::Probabilistic { rate: 0.5 }
    );

    let mut success_count = 0;
    let mut failure_count = 0;

    for _ in 0..1000 {
        match injector.inject_connection_error() {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    // Expect ~500 successes, ~500 failures (with variance)
    assert!(success_count > 400 && success_count < 600);
    assert!(failure_count > 400 && failure_count < 600);
}
```

## Mock Services

### Mock TCP Server

Async TCP server for integration testing with custom response handlers:

```rust
pub async fn spawn_mock_tcp_server(
    port: u16,
    response_handler: impl Fn(&[u8]) -> Vec<u8> + Send + 'static,
) -> MockServer {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    let handle = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            if let Ok(n) = socket.read(&mut buf).await {
                let response = response_handler(&buf[..n]);
                socket.write_all(&response).await.ok();
            }
        }
    });

    MockServer { handle, port }
}
```

**Example: HTTP Mock**

```rust
#[tokio::test]
async fn test_http_detection() {
    let mock = spawn_mock_tcp_server(8080, |request| {
        if request.starts_with(b"GET") {
            b"HTTP/1.1 200 OK\r\n\
              Server: nginx/1.20.0\r\n\
              Content-Length: 5\r\n\
              \r\n\
              hello".to_vec()
        } else {
            b"HTTP/1.1 400 Bad Request\r\n\r\n".to_vec()
        }
    }).await;

    // Test service detection
    let result = detect_service("127.0.0.1", mock.port).await.unwrap();
    assert_eq!(result.name, "http");
    assert_eq!(result.version, Some("1.20.0".to_string()));
}
```

### Docker Compose Test Environment

Multi-service environment for comprehensive integration testing:

```yaml
version: '3.8'

services:
  web-server:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    networks:
      testnet:
        ipv4_address: 172.20.0.10

  ssh-server:
    image: linuxserver/openssh-server
    environment:
      - PASSWORD_ACCESS=true
      - USER_PASSWORD=testpass
    ports:
      - "2222:2222"
    networks:
      testnet:
        ipv4_address: 172.20.0.11

  ftp-server:
    image: delfer/alpine-ftp-server
    environment:
      - USERS=testuser|testpass
    ports:
      - "21:21"
    networks:
      testnet:
        ipv4_address: 172.20.0.12

  database:
    image: postgres:15-alpine
    environment:
      - POSTGRES_PASSWORD=testpass
    ports:
      - "5432:5432"
    networks:
      testnet:
        ipv4_address: 172.20.0.13

networks:
  testnet:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/24
```

**Usage:**

```bash
# Start test environment
docker-compose -f tests/docker-compose.yml up -d

# Run integration tests
cargo test --test integration -- --test-threads=1

# Cleanup
docker-compose -f tests/docker-compose.yml down
```

**Benefits:**
- **Isolation:** Dedicated test network (172.20.0.0/24)
- **Determinism:** Fixed IP addresses, predictable responses
- **Realism:** Real services (nginx, OpenSSH, PostgreSQL)
- **Reproducibility:** Version-pinned Docker images

## Test Utilities

### Binary Discovery

Platform-aware binary path detection with debug/release preference:

```rust
pub fn get_binary_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Navigate to workspace root (from crates/prtip-cli to project root)
    let mut workspace_root = PathBuf::from(manifest_dir);
    workspace_root.pop(); // Remove prtip-cli
    workspace_root.pop(); // Remove crates

    // Windows requires .exe extension
    let binary_name = if cfg!(target_os = "windows") {
        "prtip.exe"
    } else {
        "prtip"
    };

    let mut debug_path = workspace_root.clone();
    debug_path.push("target");
    debug_path.push("debug");
    debug_path.push(binary_name);

    let mut release_path = workspace_root.clone();
    release_path.push("target");
    release_path.push("release");
    release_path.push(binary_name);

    // Prefer release (faster), fallback to debug
    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        panic!(
            "prtip binary not found. Run `cargo build` first.\n\
             Tried:\n  - {:?}\n  - {:?}",
            release_path, debug_path
        );
    }
}
```

**Key Features:**
- **Workspace Navigation:** Handles nested crate structure
- **Platform Detection:** Windows .exe extension via `cfg!(target_os)`
- **Performance:** Prefers release builds (10-100x faster)
- **Clear Errors:** Helpful panic message with attempted paths

### Test Execution

Execute binary with test isolation:

```rust
pub fn run_prtip(args: &[&str]) -> Output {
    let binary = get_binary_path();
    Command::new(binary)
        .env("PRTIP_DISABLE_HISTORY", "1") // Prevent concurrent corruption
        .args(args)
        .output()
        .expect("Failed to execute prtip")
}

pub fn run_prtip_success(args: &[&str]) -> Output {
    let output = run_prtip(args);
    assert_scan_success(&output);
    output
}
```

**Test Isolation:**
- `PRTIP_DISABLE_HISTORY=1`: Prevents concurrent test corruption of shared `~/.prtip/history.json`
- Each test gets independent execution context
- No shared state between parallel tests

### Assertion Utilities

Validate test results with clear error output:

```rust
pub fn assert_scan_success(output: &Output) {
    if !output.status.success() {
        eprintln!(
            "=== STDOUT ===\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "=== STDERR ===\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!("Scan failed with exit code: {:?}", output.status.code());
    }
}

pub fn parse_json_output(output: &[u8]) -> serde_json::Value {
    serde_json::from_slice(output)
        .expect("Failed to parse JSON output")
}

pub fn parse_xml_output(output: &[u8]) -> String {
    String::from_utf8_lossy(output).to_string()
}
```

**Benefits:**
- **Clear Failures:** Full stdout/stderr on assertion failure
- **Structured Output:** JSON parsing helpers
- **Format Support:** JSON, XML, text parsing

### Privilege Detection

Platform-specific privilege checking:

```rust
pub fn has_elevated_privileges() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        // Windows privilege check is complex, assume false for safety
        false
    }
}
```

**Skip Macro:**

```rust
#[macro_export]
macro_rules! skip_without_privileges {
    () => {
        if !$crate::common::has_elevated_privileges() {
            eprintln!("Skipping test (requires elevated privileges)");
            return;
        }
    };
}
```

**Usage:**

```rust
#[test]
fn test_syn_scan() {
    skip_without_privileges!();

    let output = run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);
    assert_scan_success(&output);
}
```

### Echo Server

Simple TCP echo server for integration tests:

```rust
pub fn start_echo_server() -> (SocketAddr, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind echo server");
    let addr = listener.local_addr()
        .expect("Failed to get address");

    let handle = std::thread::spawn(move || {
        // Accept one connection and echo data
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            if let Ok(n) = stream.read(&mut buf) {
                let _ = stream.write_all(&buf[..n]);
            }
        }
    });

    (addr, handle)
}
```

**Example:**

```rust
#[test]
fn test_tcp_connect_scan() {
    let (addr, handle) = start_echo_server();

    let output = run_prtip(&[
        "-sT",
        "-p", &addr.port().to_string(),
        "127.0.0.1"
    ]);

    assert_scan_success(&output);
    let _ = handle.join();
}
```

### Port Discovery

Find available ports for test servers:

```rust
pub fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to any port");
    listener
        .local_addr()
        .expect("Failed to get local address")
        .port()
}
```

**Usage:**

```rust
#[tokio::test]
async fn test_service_detection() {
    let port = find_available_port();

    let mock = spawn_mock_tcp_server(port, |_| {
        b"SSH-2.0-OpenSSH_8.2p1\r\n".to_vec()
    }).await;

    // Test SSH detection
    let result = detect_service("127.0.0.1", port).await.unwrap();
    assert_eq!(result.name, "ssh");
}
```

## Test Fixtures

### PCAP Samples

Pre-captured packet traces for packet parsing tests:

```rust
pub mod pcap_samples {
    pub fn load_syn_scan_capture() -> Vec<u8> {
        include_bytes!("pcaps/syn_scan.pcap").to_vec()
    }

    pub fn load_os_fingerprint_capture() -> Vec<u8> {
        include_bytes!("pcaps/os_fingerprint.pcap").to_vec()
    }

    pub fn load_service_detection_capture() -> Vec<u8> {
        include_bytes!("pcaps/service_detection.pcap").to_vec()
    }
}
```

**Usage:**

```rust
#[test]
fn test_syn_scan_parsing() {
    let pcap_data = pcap_samples::load_syn_scan_capture();
    let packets = parse_pcap(&pcap_data).unwrap();

    assert_eq!(packets.len(), 100); // Expected packet count
    assert!(packets[0].is_syn());
    assert!(packets[50].is_syn_ack());
}
```

### OS Fingerprints

Test fingerprint database for OS detection:

```rust
pub mod fingerprints {
    pub fn test_fingerprints() -> Vec<OsFingerprint> {
        vec![
            OsFingerprint {
                name: "Linux 5.x",
                signature: "T1(R=Y%DF=Y%T=40%TG=40%W=7210%S=O%A=S+%F=AS%O=%RD=0%Q=)",
            },
            OsFingerprint {
                name: "Windows 10",
                signature: "T1(R=Y%DF=Y%T=80%TG=80%W=8000%S=O%A=S+%F=AS%O=%RD=0%Q=)",
            },
            OsFingerprint {
                name: "macOS 12.x",
                signature: "T1(R=Y%DF=Y%T=40%TG=40%W=FFFF%S=O%A=S+%F=AS%O=%RD=0%Q=)",
            },
        ]
    }
}
```

### JSON Test Data

Structured test data for CLI and scanner tests:

**sample_targets.json:**
```json
{
  "single_ip": "192.168.1.1",
  "cidr_range": "10.0.0.0/24",
  "hostname": "example.com",
  "ipv6": "2001:db8::1",
  "invalid_ip": "999.999.999.999",
  "port_list": [80, 443, 22, 21],
  "port_range": "1-1024"
}
```

**nmap_compatible_flags.json:**
```json
{
  "syn_scan": ["-sS", "-p", "80,443"],
  "connect_scan": ["-sT", "-p", "1-1000"],
  "udp_scan": ["-sU", "-p", "53,123"],
  "fast_scan": ["-F"],
  "aggressive": ["-A"],
  "timing_template": ["-T4"],
  "output_formats": ["-oN", "results.txt", "-oX", "results.xml"]
}
```

**expected_outputs.json:**
```json
{
  "successful_scan": {
    "exit_code": 0,
    "stdout_contains": ["Scan complete", "ports scanned"],
    "open_ports": [80, 443]
  },
  "permission_denied": {
    "exit_code": 1,
    "stderr_contains": ["Permission denied", "requires elevated privileges"]
  }
}
```

### Fixture Loading

```rust
pub fn load_fixture(filename: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Fixture path: crates/prtip-cli/tests/fixtures/
    let fixture_path = PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(filename);

    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {:?}", fixture_path))
}

pub fn load_json_fixture(filename: &str) -> serde_json::Value {
    let content = load_fixture(filename);
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON fixture {}: {}", filename, e))
}
```

**Usage:**

```rust
#[test]
fn test_nmap_compatibility() {
    let flags = load_json_fixture("nmap_compatible_flags.json");

    let syn_scan = flags["syn_scan"].as_array().unwrap();
    let args: Vec<&str> = syn_scan.iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let output = run_prtip(&args);
    assert_scan_success(&output);
}
```

## Test Isolation

### Environment Variables

**PRTIP_DISABLE_HISTORY:**
- Prevents concurrent test corruption of shared history file
- Set to "1" in `run_prtip()` helper
- Causes history to use in-memory dummy path (`/dev/null`)

```rust
pub fn run_prtip(args: &[&str]) -> Output {
    let binary = get_binary_path();
    Command::new(binary)
        .env("PRTIP_DISABLE_HISTORY", "1") // Test isolation
        .args(args)
        .output()
        .expect("Failed to execute prtip")
}
```

**Other Isolation Variables:**
- `PRTIP_CONFIG_PATH`: Override config file location
- `PRTIP_CACHE_DIR`: Override cache directory
- `RUST_BACKTRACE`: Enable backtraces for debugging

### Temporary Directories

```rust
pub fn create_temp_dir(prefix: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join(format!(
        "prtip-test-{}-{}",
        prefix,
        std::process::id()
    ));
    fs::create_dir_all(&test_dir)
        .expect("Failed to create temp dir");
    test_dir
}

pub fn cleanup_temp_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}
```

**Usage:**

```rust
#[test]
fn test_output_to_file() {
    let temp = create_temp_dir("output");
    let output_file = temp.join("results.json");

    let output = run_prtip(&[
        "-sT",
        "-p", "80",
        "127.0.0.1",
        "-oJ", output_file.to_str().unwrap()
    ]);

    assert_scan_success(&output);
    assert!(output_file.exists());

    cleanup_temp_dir(&temp);
}
```

### Concurrent Test Safety

**Test Initialization:**

```rust
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        // Set up logging for tests (once per test binary)
        let _ = tracing_subscriber::fmt()
            .with_env_filter("warn")
            .try_init();
    });
}
```

**Thread Safety:**
- Use `std::sync::Once` for one-time initialization
- Avoid shared mutable state
- Use `PRTIP_DISABLE_HISTORY` for file isolation
- Use unique temp directories per test

## Platform-Specific Testing

### Conditional Compilation

```rust
#[cfg(unix)]
pub fn has_elevated_privileges() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(windows)]
pub fn has_elevated_privileges() -> bool {
    // Windows privilege check is complex, conservative false
    false
}

#[cfg(target_os = "macos")]
pub fn setup_bpf_access() {
    // macOS-specific BPF device setup
}

#[cfg(target_os = "linux")]
pub fn setup_capabilities() {
    // Linux-specific capability setup
}
```

### Platform-Specific Tests

```rust
#[test]
#[cfg(unix)]
fn test_raw_socket_creation() {
    skip_without_privileges!();

    let socket = create_raw_socket().unwrap();
    assert!(socket.as_raw_fd() > 0);
}

#[test]
#[cfg(windows)]
fn test_npcap_initialization() {
    // Windows-specific Npcap test
    let result = initialize_npcap();
    assert!(result.is_ok());
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_sendmmsg() {
    // Linux-specific sendmmsg/recvmmsg test
    let count = batch_send_packets(&packets);
    assert!(count > 0);
}
```

### CI/CD Platform Matrix

GitHub Actions tests on multiple platforms:

```yaml
strategy:
  matrix:
    os:
      - ubuntu-latest
      - macos-latest
      - windows-latest
    rust:
      - stable
      - 1.75.0  # MSRV
```

**Platform-Specific Behavior:**
- **Linux:** Full raw socket support, sendmmsg/recvmmsg
- **macOS:** BPF device access, group membership required
- **Windows:** Npcap dependency, administrator privileges required

## Best Practices

### Test Organization

1. **Common Modules:** Use per-crate `tests/common/` for shared utilities
2. **Fixtures:** Store test data in `tests/fixtures/` with descriptive names
3. **Integration Tests:** Use `tests/*.rs` for cross-component tests
4. **Unit Tests:** Use `#[cfg(test)]` modules in source files

### Error Injection

1. **Deterministic:** Use `SuccessAfter` for retry testing
2. **Realistic:** Use `Probabilistic` for real-world simulation
3. **Comprehensive:** Test all failure modes (11 total)
4. **Retriability:** Verify retry logic with `is_retriable()`

### Mock Services

1. **Isolation:** Use Docker Compose for integration tests
2. **Cleanup:** Always tear down mock servers after tests
3. **Determinism:** Use fixed IP addresses and ports when possible
4. **Realism:** Use real service Docker images (nginx, OpenSSH)

### Test Utilities

1. **Reusability:** Centralize common utilities in `tests/common/`
2. **Clear Errors:** Provide helpful panic messages with attempted paths
3. **Platform Support:** Use conditional compilation for platform-specific code
4. **Isolation:** Use environment variables for test independence

## See Also

- [Testing Guide](testing.md) - Testing philosophy and levels
- [Fuzzing Guide](fuzzing.md) - Fuzz testing infrastructure
- [CI/CD Guide](ci-cd.md) - Continuous integration workflows
- [Contributing Guide](contributing.md) - Development workflow
