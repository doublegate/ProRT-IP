# Service Detection Empty Probe Database Issue

**Status:** ❌ OPEN - Needs Implementation
**Priority:** CRITICAL
**Impact:** 0% service detection rate
**Estimated Fix:** 1-2 hours

---

## Issue Summary

### Problem
The `--sV` (service detection) flag is non-functional due to an empty probe database. All service detection attempts return no results.

### Root Cause
`ServiceProbeDb::default()` at line 393 in `scheduler.rs` creates an empty `Vec::new()` instead of loading probe definitions.

### Impact
- Service detection feature completely broken
- `--sV`, `--version-intensity`, and related flags have no effect
- Users cannot identify service versions
- Port scanning works perfectly (100% accuracy)

---

## Files in This Directory

### Investigation & Analysis
- **02-Debug-Report.md** - Detailed debugging investigation with test outputs
- **04-Test-Report.md** - Initial service detection testing results

### Fix Documentation
- **03-Fix-Guide.md** - Comprehensive fix implementation guide with 3 solution options:
  - **Option A:** Filesystem loading (nmap-service-probes file)
  - **Option B:** Embedded resource (compile-time inclusion)
  - **Option C:** Hybrid approach (embedded + optional filesystem override) **[RECOMMENDED]**
- **05-Implementation-Summary.md** - Tasks 1 & 2 implementation summary (service detection + progress bar)
- **06-Task-Summary.md** - Task 1 service detection fix detailed summary

### Test Outputs
- **05-Test-Output.txt** - Service detection test run output (renamed)
- **06-Test-Fallback.txt** - Fallback detection test output (renamed)
- **debug-detection-fallback.txt** - Fallback mechanism debug log
- **debug-detection-initial.txt** - Initial detection debug log
- **debug-detection-verbose.txt** - Verbose detection debug log
- **debug-fallback-retry.txt** - Fallback retry attempt #1
- **debug-fallback-retry2.txt** - Fallback retry attempt #2
- **debug-scanme-http.txt** - HTTP service detection test (scanme.nmap.org)

---

## Technical Details

### Current Architecture (Broken)
```rust
// scheduler.rs:393
let service_db = ServiceProbeDb::default();  // Returns Vec::new()
```

### Recommended Fix (Option C: Hybrid)
```rust
const EMBEDDED_PROBES: &str = include_str!("nmap-service-probes");

impl ServiceProbeDb {
    pub fn default() -> Self {
        Self::from_embedded()
    }

    pub fn from_embedded() -> Self {
        Self::parse(EMBEDDED_PROBES).unwrap_or_else(|_| Self::empty())
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        Self::parse(&data)
    }
}
```

### Required Code Changes
1. **Add nmap-service-probes file** to `crates/prtip-scanner/src/` (or resources/)
2. **Modify ServiceProbeDb::default()** to load embedded probes
3. **Add CLI flag** `--service-db <path>` for optional filesystem override
4. **Update tests** to verify probe loading

---

## Validation Plan

### Step 1: Pre-Fix Verification
```bash
# Current behavior (0% detection)
prtip -p 80,443 -sV scanme.nmap.org

# Expected output: No service version info
```

### Step 2: Post-Fix Verification
```bash
# Fixed behavior (service detection working)
prtip -p 80,443 -sV scanme.nmap.org

# Expected output:
# 80/tcp   open  http     Apache httpd 2.4.7
# 443/tcp  open  ssl/http Apache httpd 2.4.7
```

### Step 3: Cross-Reference with nmap
```bash
# nmap comparison
nmap -p 80,443 -sV scanme.nmap.org

# ProRT-IP should match nmap results
```

---

## References

- **nmap-service-probes format:** https://nmap.org/book/vscan-fileformat.html
- **nmap probe database:** `/usr/share/nmap/nmap-service-probes` (16,439 lines)
- **Implementation guide:** 03-Fix-Guide.md (comprehensive with code examples)

---

## Next Steps

1. Read **03-Fix-Guide.md** for detailed implementation options
2. Choose implementation approach (Hybrid Option C recommended)
3. Implement probe loading in ServiceProbeDb
4. Test against scanme.nmap.org and example.com
5. Validate against nmap results
6. Update version to v0.3.1 after fix

---

**Last Updated:** 2025-10-11
**Related Issues:** None (standalone critical bug)
**Fix Branch:** `feature/service-detection-fix` (recommended)
