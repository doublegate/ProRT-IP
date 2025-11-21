# TODO/FIXME Cleanup Analysis

**Analysis Date:** 2025-11-21
**ProRT-IP Version:** v0.5.4 (Phase 6 Sprint 6.4)
**Total Items Found:** 51

## Executive Summary

Comprehensive analysis of all 51 TODO/FIXME comments in the ProRT-IP codebase reveals that **26 are intentional template stubs**, **16 require significant architectural design** (40+ hours), and only **9 could potentially be implemented** as straightforward code improvements.

**Key Finding:** Many TODOs marked in code are actually placeholders for features requiring comprehensive design, implementation, and testing phases - not simple "forgotten" items.

### Breakdown by Category

| Category | Count | Status | Estimated Effort |
|----------|-------|--------|------------------|
| **Example Templates** | 26 | INTENTIONAL - Do not implement | N/A |
| **Complex Features** | 16 | Requires architectural design | 40-80 hours |
| **Medium Complexity** | 6 | Requires design + testing | 12-20 hours |
| **Simple Fixes** | 3 | Could implement now | 2-4 hours |

---

## Category 1: Example Templates (26 items) - INTENTIONAL

These are deliberately incomplete examples in the `examples/` directory meant for users to extend.

**Status:** ✅ CORRECT AS-IS - These are user extension points, not incomplete features.

### Bonus Examples (9 items)
1. **Line 24-25**: `bonus_blockchain_audit.rs` - Ethereum/Solana node scanning template
2. **Line 26-27**: `bonus_machine_learning.rs` - ML-based service detection template
3. **Line 28-29**: `bonus_realtime_dashboard.rs` - Live metrics display template

### Template Examples (17 items)
4-5. **Line 30-31**: `template_cloud_scanning.rs` - Cloud-specific scanning template
6-7. **Line 32-33**: `template_compliance_reporting.rs` - PCI-DSS/HIPAA reports template
8-9. **Line 34-35**: `template_custom_scanner.rs` - Custom scanner type template
10-11. **Line 36-37**: `template_distributed_scanning.rs` - Multi-node coordination template
12-13. **Line 38-39**: `template_output_formatter.rs` - Custom output format template
14-15. **Line 40-41**: `template_plugin_development.rs` - Lua plugin creation template
16-17. **Line 42-43**: `template_protocol_handler.rs` - Custom protocol detection template
18-19. **Line 44-45**: `template_resume_capability.rs` - Scan state persistence template
20-21. **Line 46-47**: `template_threat_intelligence.rs` - IOC/threat feed integration template
22-23. **Line 48-49**: `template_web_ui_integration.rs` - REST API for web frontend template

---

## Category 2: Complex Features Requiring Architectural Design (16 items)

These require comprehensive design, implementation, testing phases (not simple code fixes).

### 2A. Idle Scan IPID Tracking (2 items) - CRITICAL BUG

**File:** `crates/prtip-scanner/src/idle/ipid_tracker.rs`

24. **Line 245** - `send_syn_ack_probe()`: "TODO: Implement SYN/ACK packet building and sending"
25. **Line 255** - `receive_rst_response()`: "TODO: Implement RST packet reception and IPID extraction"

**Status:** 🔴 CRITICAL - Feature advertised as complete (Sprint 5.3) but core functionality is stubbed

**Impact:** Idle scanning returns stub IPID values (always 0), making scan results incorrect

**Complexity:** HIGH (8-12 hours)
- Raw TCP packet crafting (SYN/ACK with specific flags)
- Raw socket reception and parsing
- IPID field extraction from IP header
- Error handling for network failures
- Comprehensive testing (requires root privileges)
- IPv4 and IPv6 support

**Dependencies:**
- pnet packet construction
- Raw socket privileges
- Platform-specific handling (Windows/Linux/macOS)

---

### 2B. Decoy Scanner Integration (3 items) - CRITICAL BUG

**File:** `crates/prtip-scanner/src/decoy_scanner.rs`

26. **Line 578-579** - `build_syn_probe()`: "TODO: handle multiple fragments properly"
27. **Line 584** - `send_raw_packet()`: "TODO: Integrate with actual raw socket sender"
28. **Line 597** - `wait_for_response()`: "TODO: Integrate with actual response receiver"

**Status:** 🔴 CRITICAL - Feature exported in public API but non-functional

**Impact:** Decoy scanning does nothing - packets aren't sent, responses aren't received

**Complexity:** HIGH (12-16 hours)
- Signature change for multi-fragment support (ripples through codebase)
- Integration with existing BatchSender infrastructure
- Response correlation with decoy sources
- Comprehensive testing
- Performance validation

**Dependencies:**
- BatchSender/BatchReceiver integration
- Connection state tracking modification
- Raw socket infrastructure

---

### 2C. Plugin System Lua Callbacks (6 items) - FEATURE INCOMPLETE

**Files:**
- `crates/prtip-scanner/src/plugin/plugin_api.rs`
- `crates/prtip-scanner/src/plugin/lua_api.rs`
- `crates/prtip-scanner/src/plugin/plugin_manager.rs`

29. **Line 245** - `LuaScanPlugin::pre_scan()`: "TODO: Call Lua on_pre_scan function"
30. **Line 250** - `LuaScanPlugin::on_target()`: "TODO: Call Lua on_target function"
31. **Line 255** - `LuaScanPlugin::post_scan()`: "TODO: Call Lua on_post_scan function"
32. **Line 303** - `LuaOutputPlugin::format_result()`: "TODO: Call Lua format_result function"
33. **Line 308** - `LuaOutputPlugin::export()`: "TODO: Call Lua export function"
34. **Line 249** - `plugin_manager.rs`: "TODO: Pass actual configuration"

**Status:** 🟡 MEDIUM PRIORITY - Plugin system loads plugins but doesn't execute callbacks

**Impact:** Plugins can be loaded but don't actually run - all callbacks are no-ops

**Complexity:** MEDIUM (6-10 hours total)
- Lua FFI bindings for each callback (mlua crate)
- Error handling and type conversion (Rust ↔ Lua)
- Configuration file format design (TOML/JSON)
- Security: ensure sandboxing works with callbacks
- Comprehensive testing with example plugins
- Documentation updates

**Dependencies:**
- mlua crate capabilities
- Plugin configuration schema design
- Sandboxing verification

---

### 2D. Plugin Lua Network API (5 items) - BLOCKED BY SANDBOXING

**File:** `crates/prtip-scanner/src/plugin/lua_api.rs`

35. **Line 171** - `prtip_network_connect()`: "TODO: Implement actual network connection"
36. **Line 192** - `prtip_network_send()`: "TODO: Implement actual network send"
37. **Line 218** - `prtip_network_receive()`: "TODO: Implement actual network receive"
38. **Line 231** - `prtip_network_close()`: "TODO: Implement actual socket close"
39. **Line 245** - `prtip_add_result()`: "TODO: Store result in scan context"

**Status:** ⏸️ BLOCKED - Requires comprehensive sandboxing design

**Impact:** Lua plugins cannot perform network operations (security feature, not bug)

**Complexity:** VERY HIGH (16-24 hours)
- Security model design: what network access should plugins have?
- Capability system integration
- Rate limiting for plugin network calls
- Connection pooling and resource limits
- Async Rust ↔ Lua bridge (complex with mlua)
- Security audit required
- Comprehensive testing and fuzzing

**Dependencies:**
- Security policy design
- Capability system enhancement
- Async/await support in Lua bindings

**Recommendation:** Consider whether plugins *should* have direct network access, or if they should operate on scan results only.

---

## Category 3: Medium Complexity Enhancements (6 items)

### 3A. IPv6 CDN Detection (1 item)

**File:** `crates/prtip-core/src/cdn_detector.rs`

40. **Line 319** - `check_ipv6()`: "TODO: Implement IPv6 CDN range detection"

**Status:** 🟡 ENHANCEMENT - Currently returns None for all IPv6 addresses

**Impact:** CDN deduplication doesn't work for IPv6 targets (minor, as most CDN detection is IPv4)

**Complexity:** MEDIUM (4-6 hours)
- Create `Ipv6Cidr` struct (similar to Ipv4Cidr)
- Refactor `CdnRange` to support both IPv4/IPv6 (enum or generic)
- Research and add 50-100 IPv6 CIDR ranges for 6 CDN providers:
  - Cloudflare IPv6 ranges
  - Akamai IPv6 ranges
  - AWS CloudFront IPv6 ranges
  - Fastly IPv6 ranges
  - Google Cloud CDN IPv6 ranges
  - Azure CDN IPv6 ranges
- Update storage (sorted Vec needs separate IPv4/IPv6 lists or unified sorting)
- Binary search logic adaptation
- Comprehensive tests (unit + integration)

**Dependencies:**
- CDN provider IPv6 range documentation
- None (self-contained feature)

---

### 3B. IPv6 Extension Header Handling (2 items)

**Files:**
- `crates/prtip-scanner/src/stealth_scanner.rs`
- `crates/prtip-scanner/src/syn_scanner.rs`

41. **Line 921** (stealth_scanner.rs): "TODO Sprint 5.1 Phase 2.3: Handle extension headers"
42. **Line 872** (syn_scanner.rs): "TODO Sprint 5.1 Phase 1.5: Handle extension headers"

**Status:** 🟡 ENHANCEMENT - IPv6 works but doesn't properly parse extension headers

**Impact:** IPv6 scans may fail to parse responses with extension headers (Fragment, Hop-by-Hop, Routing, Destination Options)

**Complexity:** MEDIUM (6-8 hours for both scanners)
- Implement RFC 8200 extension header parsing chain
- Handle 4 extension header types:
  - Fragment Header (Type 44)
  - Hop-by-Hop Options (Type 0)
  - Routing Header (Type 43)
  - Destination Options (Type 60)
- Chain parsing (extension headers form a linked list)
- Comprehensive testing with captured IPv6 packets
- Documentation updates

**Dependencies:**
- pnet crate IPv6 extension header support (may need enhancement)
- Test packet captures with extension headers

---

### 3C. Database Schema Enhancement (2 items)

**File:** `crates/prtip-scanner/src/db_reader.rs`

43. **Line 262** - Query result: "TODO: Add version column to schema"
44. **Line 323** - Result struct: "TODO: Add version column"

**Status:** 🟡 ENHANCEMENT - Missing version field in database queries

**Impact:** Service version information not retrieved from database (minor, as version detection works during scanning)

**Complexity:** MEDIUM (3-4 hours)
- Add `version` column to database schema (requires migration)
- Update INSERT/UPDATE queries to include version
- Update all SELECT queries to retrieve version
- Data migration for existing databases
- Update ResultRow struct
- Update all consumers of ResultRow
- Testing (unit + integration)

**Dependencies:**
- sqlx migrations
- Backward compatibility testing

---

### 3D. TLS Certificate Validation (1 item)

**File:** `crates/prtip-scanner/src/tls_certificate.rs`

45. **Line 2559** - `is_certificate_expired()`: "TODO: Parse RFC 2822 dates and compare against current time"

**Status:** 🟡 ENHANCEMENT - Currently always returns false (relies on TLS handshake validation)

**Impact:** Certificate expiry not checked programmatically (minor, as TLS library handles validation during handshake)

**Complexity:** LOW-MEDIUM (2-3 hours)
- Parse validity dates (chrono crate)
- Format investigation (x509-parser produces ISO 8601-like format, not RFC 2822)
- Current time comparison
- Timezone handling
- Edge cases (not-yet-valid certificates)
- Comprehensive tests
- Documentation

**Note:** TODO comment mentions "RFC 2822" but actual format is ISO 8601-like from x509-parser crate

---

## Category 4: Simple Fixes (3 items)

### 4A. Batch Sender Address Parsing (1 item)

**File:** `crates/prtip-network/src/batch_sender.rs`

46. **Line 861** - `src_addr: None`: "TODO: Parse sockaddr_storage to SocketAddr"

**Status:** 🟢 LOW PRIORITY - Debug/logging information only

**Impact:** Source address not logged in debug messages (no functional impact)

**Complexity:** LOW (30-60 minutes)
- Convert `sockaddr_storage` C struct to Rust `SocketAddr`
- Handle IPv4 (sockaddr_in) and IPv6 (sockaddr_in6) variants
- Use `libc` crate for safe conversion
- Testing

**Implementation:**
```rust
// Pseudocode
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::mem;

unsafe fn parse_sockaddr(addr: *const libc::sockaddr_storage) -> Option<SocketAddr> {
    match (*addr).ss_family as i32 {
        libc::AF_INET => {
            let addr_in = addr as *const libc::sockaddr_in;
            Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from(u32::from_be((*addr_in).sin_addr.s_addr))),
                u16::from_be((*addr_in).sin_port)
            ))
        },
        libc::AF_INET6 => {
            let addr_in6 = addr as *const libc::sockaddr_in6;
            Some(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from((*addr_in6).sin6_addr.s6_addr)),
                u16::from_be((*addr_in6).sin6_port)
            ))
        },
        _ => None
    }
}
```

---

### 4B. TUI Horizontal Scroll (2 items)

**File:** `crates/prtip-tui/src/events/loop.rs`

47. **Line 81** - Left arrow key: "TODO: Sprint 6.2 - horizontal scroll"
48. **Line 90** - Right arrow key: "TODO: Sprint 6.2 - horizontal scroll"

**Status:** ❓ UNCLEAR - Sprint 6.2 marked COMPLETE, but TODOs remain

**Impact:** Left/Right arrow keys do nothing in TUI (Vim h/l keys also affected)

**Complexity:** LOW (1-2 hours) IF horizontal scroll is intended feature
- Add `horizontal_scroll_offset` to UIState
- Implement scroll logic (similar to vertical cursor)
- Update rendering to respect horizontal offset
- Add bounds checking
- Testing

**Questions:**
1. Was horizontal scroll intentionally deferred beyond Sprint 6.2?
2. Is it needed for current dashboard layout?
3. Should it scroll content or switch tabs?

**Recommendation:** Clarify requirement before implementing. May be deferred feature or obsolete TODO.

---

### 4C. TLS X509Certificate Storage (1 item)

**File:** `crates/prtip-scanner/src/tls_certificate.rs`

49. **Line 2519** - Certificate chain parsing: "TODO: Store X509Certificate alongside CertificateInfo in TASK-3"

**Status:** ❓ NEEDS INVESTIGATION - References unknown "TASK-3"

**Impact:** Unknown - depends on TASK-3 requirements

**Complexity:** UNKNOWN - Cannot assess without understanding TASK-3 context

**Recommendation:** Investigate TASK-3 requirements before implementation. May be obsolete comment from earlier development phase.

---

## Implementation Recommendations

### Immediate Actions (High Priority Bugs)

1. **Idle Scan IPID Tracking (Items 24-25):**
   - **Priority:** CRITICAL
   - **Rationale:** Feature advertised as complete but non-functional
   - **Effort:** 8-12 hours
   - **Sprint:** 6.5 or 7.1

2. **Decoy Scanner Integration (Items 26-28):**
   - **Priority:** CRITICAL
   - **Rationale:** Public API feature that doesn't work
   - **Effort:** 12-16 hours
   - **Sprint:** 6.5 or 7.1

### Medium Priority Enhancements

3. **Plugin Lua Callbacks (Items 29-34):**
   - **Priority:** MEDIUM
   - **Rationale:** Complete plugin system functionality
   - **Effort:** 6-10 hours
   - **Sprint:** 6.5 or 7.2

4. **IPv6 Extension Headers (Items 41-42):**
   - **Priority:** MEDIUM
   - **Rationale:** Improve IPv6 scan reliability
   - **Effort:** 6-8 hours
   - **Sprint:** 7.1 or 7.2

### Low Priority / Future

5. **IPv6 CDN Detection (Item 40):**
   - **Priority:** LOW
   - **Rationale:** Most CDN traffic is IPv4
   - **Effort:** 4-6 hours
   - **Sprint:** 7.x or Phase 7

6. **Database Version Column (Items 43-44):**
   - **Priority:** LOW
   - **Rationale:** Version already available during scanning
   - **Effort:** 3-4 hours
   - **Sprint:** 7.x or Phase 7

7. **TLS Certificate Expiry (Item 45):**
   - **Priority:** LOW
   - **Rationale:** TLS handshake already validates
   - **Effort:** 2-3 hours
   - **Sprint:** 7.x or Phase 7

8. **sockaddr Parsing (Item 46):**
   - **Priority:** LOW
   - **Rationale:** Debug logging only
   - **Effort:** 30-60 minutes
   - **Sprint:** Any (filler task)

### Needs Investigation

9. **TUI Horizontal Scroll (Items 47-48):**
   - Clarify if feature is needed or obsolete TODO
   - May be 1-2 hours if needed

10. **X509Certificate Storage (Item 49):**
    - Investigate TASK-3 requirements
    - May be obsolete comment

### Do Not Implement

11. **Example Templates (Items 1-23):**
    - These are intentional user extension points
    - Implementing them would remove their value as templates

12. **Plugin Network API (Items 35-39):**
    - Blocked pending security design
    - Consider whether plugins should have direct network access
    - May be Phase 7+ or never (by design)

---

## Statistics

### By Status

- ✅ **INTENTIONAL (26):** Example templates - correct as-is
- 🔴 **CRITICAL BUGS (5):** Idle scan, decoy scan - broken features
- 🟡 **ENHANCEMENTS (9):** IPv6 CDN, extension headers, DB schema, plugins
- 🟢 **SIMPLE FIXES (3):** sockaddr parsing, TUI scroll (maybe)
- ⏸️ **BLOCKED (5):** Plugin network API - requires security design
- ❓ **NEEDS INVESTIGATION (2):** TUI scroll status, TASK-3 context

### By Effort

- **<1 hour:** 1 item (sockaddr parsing)
- **1-3 hours:** 3 items (TLS expiry, TUI scroll if needed, TASK-3 if simple)
- **3-6 hours:** 2 items (DB schema, IPv6 CDN)
- **6-10 hours:** 2 items (IPv6 extension headers, plugin callbacks)
- **10-16 hours:** 2 items (idle scan, decoy scan)
- **16+ hours:** 5 items (plugin network API - if ever)
- **N/A:** 26 items (example templates)

### Total Implementable Effort

- **Critical bugs:** 20-28 hours
- **Medium enhancements:** 15-24 hours
- **Low priority:** 5-7 hours
- **Simple fixes:** 1-3 hours
- **Total:** 41-62 hours

---

## Lessons Learned

### About TODO Comments

1. **Many TODOs are feature placeholders, not forgotten code**
   - Idle scan, decoy scan, and plugin callbacks have complete infrastructure but intentionally stub core functionality
   - These require design phases, not just "filling in" code

2. **"Simple" TODOs often have hidden complexity**
   - Example: "handle multiple fragments" requires signature changes rippling through codebase
   - Example: "pass actual configuration" requires defining config file format and schema

3. **Example code should not have implementation TODOs**
   - The 26 example template files have TODOs that confuse analysis
   - Recommendation: Use different markers for "user should implement here" (e.g., `// EXAMPLE: implement ...`)

### About Code Quality

4. **Several "complete" features are actually non-functional**
   - Idle scan (Sprint 5.3 "complete") returns stub IPID values
   - Decoy scan (exported in public API) doesn't send packets
   - Plugin system (Sprint 5.8 "complete") doesn't execute callbacks

5. **Missing test coverage for stub implementations**
   - Tests likely mock or skip the stub code paths
   - Integration tests would catch these issues

### Recommendations

1. **Update Sprint Status:**
   - Mark Idle Scan (Sprint 5.3) as "Infrastructure Complete, Core Functionality TODO"
   - Mark Decoy Scan (Phase 4) as "API Complete, Integration TODO"
   - Mark Plugin System (Sprint 5.8) as "Loading Complete, Callbacks TODO"

2. **Create Feature Completion Sprints:**
   - Sprint 6.5 or 7.1: Complete Idle Scan + Decoy Scan
   - Sprint 7.2: Complete Plugin Callbacks

3. **Improve TODO Markers:**
   - Use `// TEMPLATE:` for example extension points (not `// TODO:`)
   - Use `// BLOCKED:` for items requiring design (not `// TODO:`)
   - Reserve `// TODO:` for simple forgotten items

4. **Add Integration Tests:**
   - End-to-end idle scan test (requires root, can be in separate test suite)
   - End-to-end decoy scan test
   - Plugin callback execution test

---

## Conclusion

Of 51 TODO/FIXME comments found:
- **26 (51%)** are intentional example templates
- **16 (31%)** require significant design and implementation work
- **6 (12%)** are medium-complexity enhancements
- **3 (6%)** are simple fixes

**Key Finding:** Most TODOs are not "forgotten" items but rather placeholders for future phases or architectural design decisions. Several features marked "complete" in sprint documentation have core functionality stubbed with TODO comments.

**Recommendation:** Prioritize completing the advertised features (idle scan, decoy scan, plugin callbacks) in Sprint 6.5 or Phase 7, rather than attempting to implement all 51 TODOs.

---

**Document Version:** 1.0
**Author:** Claude Code
**Review Status:** Initial Analysis
**Next Review:** After Sprint 6.5 planning
