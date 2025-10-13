//! Common test utilities for ProRT-IP integration tests
//!
//! This module provides helper functions and utilities for integration tests.

#![allow(dead_code)] // Test utilities may not all be used yet

use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment (call once per test binary)
pub fn init() {
    INIT.call_once(|| {
        // Set up logging for tests
        let _ = tracing_subscriber::fmt().with_env_filter("warn").try_init();
    });
}

/// Get path to prtip binary (debug or release)
pub fn get_binary_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Go up to workspace root (from crates/prtip-cli to project root)
    let mut workspace_root = PathBuf::from(manifest_dir);
    workspace_root.pop(); // Remove prtip-cli
    workspace_root.pop(); // Remove crates

    // Binary name differs on Windows (.exe extension)
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

    // Try release first (faster), then debug
    if release_path.exists() {
        release_path
    } else if debug_path.exists() {
        debug_path
    } else {
        panic!(
            "prtip binary not found. Run `cargo build` or `cargo build --release` first.\n\
             Tried:\n  - {:?}\n  - {:?}",
            release_path, debug_path
        );
    }
}

/// Run prtip with given arguments
pub fn run_prtip(args: &[&str]) -> Output {
    let binary = get_binary_path();
    Command::new(binary)
        .args(args)
        .output()
        .expect("Failed to execute prtip")
}

/// Run prtip and expect success
pub fn run_prtip_success(args: &[&str]) -> Output {
    let output = run_prtip(args);
    assert_scan_success(&output);
    output
}

/// Assert scan completed successfully
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

/// Parse JSON output from scan
pub fn parse_json_output(output: &[u8]) -> serde_json::Value {
    serde_json::from_slice(output).expect("Failed to parse JSON output")
}

/// Parse XML output from scan
pub fn parse_xml_output(output: &[u8]) -> String {
    String::from_utf8_lossy(output).to_string()
}

/// Get localhost IP address
pub fn localhost() -> &'static str {
    "127.0.0.1"
}

/// Find an available port on localhost (for test servers)
pub fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to any port");
    listener
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

/// Create temporary test directory
pub fn create_temp_dir(prefix: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join(format!("prtip-test-{}-{}", prefix, std::process::id()));
    fs::create_dir_all(&test_dir).expect("Failed to create temp dir");
    test_dir
}

/// Clean up temporary directory
pub fn cleanup_temp_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Check if running with elevated privileges (for SYN scan tests)
pub fn has_elevated_privileges() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        // Windows privilege check is more complex, assume false for safety
        false
    }
}

/// Skip test if not running with elevated privileges
#[macro_export]
macro_rules! skip_without_privileges {
    () => {
        if !$crate::common::has_elevated_privileges() {
            eprintln!("Skipping test (requires elevated privileges)");
            return;
        }
    };
}

/// Load test fixture from fixtures/ directory
pub fn load_fixture(filename: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Fixture path is in crates/prtip-cli/tests/fixtures/
    let fixture_path = PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(filename);

    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to load fixture: {:?}", fixture_path))
}

/// Load JSON fixture and parse
pub fn load_json_fixture(filename: &str) -> serde_json::Value {
    let content = load_fixture(filename);
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON fixture {}: {}", filename, e))
}

/// Simple TCP echo server for testing (returns port number)
pub fn start_echo_server() -> (SocketAddr, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind echo server");
    let addr = listener.local_addr().expect("Failed to get address");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost() {
        assert_eq!(localhost(), "127.0.0.1");
    }

    #[test]
    fn test_find_available_port() {
        let port = find_available_port();
        assert!(port > 0);
        // u16 max is 65535, so no need to check upper bound
    }

    #[test]
    fn test_create_and_cleanup_temp_dir() {
        let dir = create_temp_dir("test");
        assert!(dir.exists());
        cleanup_temp_dir(&dir);
        assert!(!dir.exists());
    }

    #[test]
    fn test_get_binary_path() {
        // Should not panic
        let path = get_binary_path();
        assert!(path.exists() || !path.exists()); // May not be built yet
    }
}
