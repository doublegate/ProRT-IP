# Service Detection Fix Guide

## Problem

Service detection is completely broken (0% detection rate) because `ServiceProbeDb::default()` creates an empty database with zero probes.

**Affected Code:**
- File: `crates/prtip-scanner/src/scheduler.rs`
- Line: 393
- Code: `let probe_db = ServiceProbeDb::default();`

**Current Behavior:**
```rust
impl Default for ServiceProbeDb {
    fn default() -> Self {
        Self::new()  // Creates empty Vec<ServiceProbe>
    }
}
```

## Solution Options

### Option A: Embedded Probe Database (RECOMMENDED)

Embed nmap-service-probes directly in binary for zero-dependency operation.

**Advantages:**
- ✅ No external file dependencies
- ✅ Always available (can't fail to load)
- ✅ Portable across systems
- ✅ Faster startup (no file I/O)

**Disadvantages:**
- ❌ Larger binary size (~200KB)
- ❌ Requires rebuild to update probes

**Implementation:**

```rust
// In crates/prtip-core/src/service_db.rs

// Add embedded probes constant
const EMBEDDED_SERVICE_PROBES: &str = include_str!("../data/nmap-service-probes");

impl ServiceProbeDb {
    /// Create database with embedded probes
    pub fn with_embedded_probes() -> Result<Self, Error> {
        Self::parse(EMBEDDED_SERVICE_PROBES)
    }
}

impl Default for ServiceProbeDb {
    fn default() -> Self {
        // Use embedded probes, fallback to empty if parsing fails
        Self::with_embedded_probes()
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load embedded probes: {}", e);
                Self::new()
            })
    }
}
```

**File Setup:**
```bash
# Download nmap-service-probes
mkdir -p crates/prtip-core/data
curl -o crates/prtip-core/data/nmap-service-probes \
  https://raw.githubusercontent.com/nmap/nmap/master/nmap-service-probes

# Verify file
head -20 crates/prtip-core/data/nmap-service-probes
```

### Option B: Load from Filesystem

Load probes from standard nmap location with fallback.

**Advantages:**
- ✅ Smaller binary size
- ✅ Easy to update (just replace file)
- ✅ Uses system nmap installation

**Disadvantages:**
- ❌ Requires nmap to be installed
- ❌ May fail if file not found
- ❌ Path differences across OSes

**Implementation:**

```rust
impl ServiceProbeDb {
    /// Load from standard nmap locations
    pub fn load_from_system() -> Result<Self, Error> {
        let paths = [
            "/usr/share/nmap/nmap-service-probes",           // Linux
            "/usr/local/share/nmap/nmap-service-probes",     // BSD/macOS Homebrew
            "/opt/nmap/share/nmap-service-probes",           // Alternative
            "C:\\Program Files\\Nmap\\nmap-service-probes", // Windows
        ];

        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                return Self::parse(&content);
            }
        }

        Err(Error::InvalidFormat(
            "nmap-service-probes not found in standard locations".to_string()
        ))
    }
}

impl Default for ServiceProbeDb {
    fn default() -> Self {
        Self::load_from_system()
            .unwrap_or_else(|e| {
                eprintln!("Warning: {}", e);
                eprintln!("Service detection disabled. Install nmap or use --probe-db <file>");
                Self::new()
            })
    }
}
```

### Option C: Hybrid Approach (BEST)

Combine embedded fallback with optional external file.

**Advantages:**
- ✅ Always works (embedded fallback)
- ✅ Updatable (external file override)
- ✅ User control (--probe-db flag)
- ✅ Best of both worlds

**Disadvantages:**
- ❌ Slightly more complex
- ❌ Larger binary (embedded probes)

**Implementation:**

```rust
impl ServiceProbeDb {
    /// Create database with best available source
    pub fn load_default() -> Result<Self, Error> {
        // 1. Try embedded probes
        if let Ok(db) = Self::with_embedded_probes() {
            return Ok(db);
        }

        // 2. Try system installation
        if let Ok(db) = Self::load_from_system() {
            return Ok(db);
        }

        // 3. Return empty with warning
        eprintln!("Warning: No service probes available");
        eprintln!("Service detection disabled");
        Ok(Self::new())
    }

    /// Load from custom file path
    pub fn load_from_file(path: &str) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::InvalidFormat(format!("Failed to read {}: {}", path, e)))?;
        Self::parse(&content)
    }
}

impl Default for ServiceProbeDb {
    fn default() -> Self {
        Self::load_default().unwrap_or_else(|_| Self::new())
    }
}
```

**CLI Enhancement:**

```rust
// In crates/prtip-cli/src/args.rs

/// Custom service probe database file
#[arg(long, value_name = "FILE", help_heading = "DETECTION")]
pub probe_db: Option<String>,
```

```rust
// In scheduler.rs

let probe_db = if let Some(path) = config.probe_db_path {
    ServiceProbeDb::load_from_file(&path)?
} else {
    ServiceProbeDb::default()  // Now uses hybrid approach
};
```

## Recommended Solution: Option C (Hybrid)

**Implementation Steps:**

1. **Download nmap-service-probes** (5 minutes)
   ```bash
   mkdir -p crates/prtip-core/data
   curl -o crates/prtip-core/data/nmap-service-probes \
     https://raw.githubusercontent.com/nmap/nmap/master/nmap-service-probes
   ```

2. **Update service_db.rs** (30 minutes)
   - Add `EMBEDDED_SERVICE_PROBES` constant
   - Implement `with_embedded_probes()`
   - Implement `load_from_system()`
   - Implement `load_from_file()`
   - Implement `load_default()` with fallback chain
   - Update `Default` impl to use `load_default()`

3. **Add CLI flag** (15 minutes)
   - Add `--probe-db <FILE>` option in `args.rs`
   - Pass to config
   - Use in scheduler

4. **Test** (30 minutes)
   ```bash
   # Test embedded probes
   cargo build --release
   ./target/release/prtip -p 80 --sV example.com

   # Test external file
   ./target/release/prtip -p 80 --sV --probe-db /usr/share/nmap/nmap-service-probes example.com

   # Verify detection
   # Expected: Service detection should show "http" for port 80
   ```

5. **Add integration test** (15 minutes)
   ```rust
   #[tokio::test]
   async fn test_service_detection_http() {
       let db = ServiceProbeDb::default();
       assert!(!db.is_empty(), "Probe database should not be empty");
       assert!(db.len() > 100, "Should have >100 probes");
   }
   ```

**Total Effort:** ~1.5 hours

## Testing Checklist

After fix implementation:

- [ ] Embedded probes load successfully
- [ ] `ServiceProbeDb::default()` returns non-empty database
- [ ] HTTP service detected on port 80
- [ ] SSH service detected on port 22
- [ ] HTTPS service detected on port 443
- [ ] `--probe-db <file>` flag works
- [ ] Warning shown if probes fail to load
- [ ] Integration tests pass
- [ ] Unit tests pass (all 551 tests)
- [ ] Service detection shows in output

**Expected Output After Fix:**

```
=== ProRT-IP Scan Results ===
Scan Type: TCP Connect
Service Detection: Enabled (Intensity: 7)

Host: example.com (23.215.0.136)
Open Ports:
     80 open   http    Apache httpd 2.4.7 ((Ubuntu))
    443 open   https   Apache httpd 2.4.7 ((Ubuntu))

Service detection complete: 2/2 services identified
```

## License Considerations

**nmap-service-probes License:**
- GPL-compatible (nmap is GPLv2 with exceptions)
- ProRT-IP is GPL-3.0 (compatible)
- ✅ OK to include in repository
- ✅ OK to distribute in binary

**Attribution:**
Add to NOTICE or README:
```
This software includes nmap-service-probes from the Nmap Security Scanner:
https://github.com/nmap/nmap
Copyright (C) 1996-2024 Nmap Software LLC
Licensed under GPL-2.0
```

## References

- **nmap source:** https://github.com/nmap/nmap/blob/master/nmap-service-probes
- **Probe format:** https://nmap.org/book/vscan-fileformat.html
- **Service detection:** https://nmap.org/book/vscan.html
- **ProRT-IP parser:** `crates/prtip-core/src/service_db.rs` (already implemented)

## Quick Fix (Minimal)

If you just want to get it working quickly:

```rust
// In scheduler.rs, replace line 393:
let probe_db = ServiceProbeDb::default();

// With:
const MINIMAL_PROBES: &str = r#"
Probe TCP NULL q||
Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
ports 80,443,8080
match http m|^HTTP/1\.[01]| p/HTTP/
match http m|^HTTP/1\.[01] \d+ | p/HTTP/ v/$1/

Probe TCP SSHVersionRequest q|SSH-2.0-OpenSSH_7.4\r\n|
ports 22
match ssh m|^SSH-([\\d.]+)-OpenSSH[_-]([\\w.]+)| p/OpenSSH/ v/$2/ i/protocol $1/
"#;

let probe_db = ServiceProbeDb::parse(MINIMAL_PROBES)
    .unwrap_or_else(|_| ServiceProbeDb::new());
```

This provides basic HTTP and SSH detection without external files.

---

**Created:** 2025-10-11 07:16:00 UTC
**Priority:** 🔴 HIGH - Blocks service detection functionality
**Effort:** ~1.5 hours (full hybrid solution) or ~15 minutes (minimal fix)
