# Phase 5 Backlog - Advanced Features

**Last Updated:** 2025-10-26
**Status:** Planned for v0.5.0 (Q1 2026)

## Overview

This document tracks advanced features deferred to Phase 5 (v0.5.0) based on strategic prioritization for v0.4.0 release.

---

## Sprint 5.X: IPv6 Scanner Integration (DEFERRED from Sprint 4.21)

**Priority:** MEDIUM
**Estimated Duration:** 25-30 hours
**ROI Score:** 6.8/10
**Dependencies:** Sprint 4.21a + 4.21b (TCP Connect) COMPLETE

### Objective

Complete IPv6 integration for remaining 5 scanner types: SYN, UDP, Stealth (FIN/NULL/Xmas), Discovery, and Decoy.

### Current Status (v0.4.0)

**✅ Completed in Sprint 4.21:**
- IPv6 packet building infrastructure (ipv6_packet.rs, 671 lines, RFC 8200)
- ICMPv6 protocol implementation (icmpv6.rs, 556 lines, RFC 4443)
- packet_builder.rs IPv6 integration (+326 lines)
- TCP Connect scanner IPv6 support (+95 lines, 6 tests)
- **Coverage:** 80% of IPv6 use cases (SSH, HTTP, HTTPS work)

**⏸️ Remaining Work:**
- SYN Scanner IPv6
- UDP Scanner IPv6
- Stealth Scanner IPv6 (FIN/NULL/Xmas/ACK)
- Discovery Engine IPv6
- Decoy Scanner IPv6

### Why Deferred

1. **Complexity Underestimated:** Original 8-10h estimate was 3x too low (actual: 25-30h)
2. **Architecture Challenges:** All scanners use `Ipv4Addr` throughout, require significant refactoring to `IpAddr`
3. **ROI Analysis:** TCP Connect covers 80% of IPv6 use cases (web services, SSH, databases)
4. **Timeline Impact:** Full implementation would delay v0.4.0 by 1+ month
5. **Strategic Priority:** Error handling, service detection more critical for v0.4.0

### Remaining Work

#### Phase 1: SYN Scanner IPv6 (5 hours)

**Objective:** Add IPv6 support to SYN scanner with dual-stack management.

**Tasks:**
- Refactor to use `IpAddr` instead of `Ipv4Addr` (2 hours)
  - Update scanner struct fields
  - Update method signatures
  - Update connection tracking
- Add IPv6 response parsing (1.5 hours)
  - Handle 40-byte IPv6 header (vs 20-byte IPv4)
  - Skip extension headers to find TCP header
  - Parse TCP flags from IPv6 packets
- Implement dual-stack local IP management (1 hour)
  - Detect local IPv6 addresses
  - Select appropriate source IP per target
  - Handle ICMPv6 errors
- Add 8+ integration tests (0.5 hours)
  - IPv6 SYN scan tests
  - Dual-stack tests
  - Response parsing tests

**Files to Modify:**
- `crates/prtip-scanner/src/syn_scanner.rs` (~100 lines)
- `crates/prtip-scanner/tests/test_syn_scanner_ipv6.rs` (NEW, ~150 lines)

**Technical Challenges:**
- Extension header traversal (Hop-by-Hop, Routing, Fragment, Destination Options)
- Fragment reassembly (IPv6 uses extension headers, not IP header flags)
- ICMPv6 Type 1 Code 1 (communication administratively prohibited)

#### Phase 2: UDP + Stealth Scanners IPv6 (8 hours)

**Objective:** Add IPv6 support to UDP and Stealth scanners.

**UDP Scanner IPv6 (4 hours):**
- Add ICMPv6 Type 1 Code 4 (port unreachable) handling (2 hours)
  - Parse ICMPv6 headers
  - Extract original packet from ICMPv6 payload
  - Map to scan results
- Update connection tracking for dual-stack (1 hour)
- Add 8+ integration tests (1 hour)
  - IPv6 UDP scan tests
  - ICMPv6 parsing tests
  - Dual-stack tests

**Stealth Scanner IPv6 (4 hours):**
- Add FIN/NULL/Xmas/ACK IPv6 variants (2 hours)
  - Update packet building for IPv6
  - Handle IPv6 responses
- Update connection tracking (1 hour)
- Add 8+ integration tests (1 hour)
  - FIN scan IPv6 tests
  - NULL scan IPv6 tests
  - Xmas scan IPv6 tests
  - ACK scan IPv6 tests

**Files to Modify:**
- `crates/prtip-scanner/src/udp_scanner.rs` (~80 lines)
- `crates/prtip-scanner/src/stealth_scanner.rs` (~80 lines)
- `crates/prtip-scanner/tests/test_udp_scanner_ipv6.rs` (NEW, ~120 lines)
- `crates/prtip-scanner/tests/test_stealth_scanner_ipv6.rs` (NEW, ~120 lines)

**Technical Challenges:**
- ICMPv6 Type 1 has 6 codes (vs ICMP Type 3's 16 codes)
- IPv6 has no fragmentation in main header (uses extension headers)
- TCP options parsing same for IPv4/IPv6 (but offset differs due to IPv6 header size)

#### Phase 3: Discovery + Decoy Scanners IPv6 (7 hours)

**Objective:** Add IPv6 support to Discovery and Decoy scanners.

**Discovery Engine IPv6 (4 hours):**
- ICMPv6 Echo Request (Type 128) + Echo Reply (Type 129) (2 hours)
  - Build ICMPv6 packets
  - Parse responses
  - Handle timeouts
- Neighbor Discovery Protocol (NDP) support (1 hour)
  - Neighbor Solicitation (Type 135)
  - Neighbor Advertisement (Type 136)
- Add 7+ integration tests (1 hour)
  - ICMPv6 Echo tests
  - NDP tests
  - Dual-stack discovery tests

**Decoy Scanner IPv6 (3 hours):**
- Generate random IPv6 addresses in same /64 subnet (1.5 hours)
  - Use target's /64 prefix
  - Randomize lower 64 bits
  - Validate uniqueness
- Update packet building for IPv6 decoys (1 hour)
- Add 7+ integration tests (0.5 hours)
  - IPv6 decoy generation tests
  - IPv6 decoy scan tests

**Files to Modify:**
- `crates/prtip-scanner/src/discovery.rs` (~100 lines)
- `crates/prtip-scanner/src/decoy_scanner.rs` (~60 lines)
- `crates/prtip-scanner/tests/test_discovery_ipv6.rs` (NEW, ~140 lines)
- `crates/prtip-scanner/tests/test_decoy_scanner_ipv6.rs` (NEW, ~100 lines)

**Technical Challenges:**
- NDP uses link-local addresses (fe80::/10)
- IPv6 /64 subnets are standard (2^64 addresses = massive address space)
- Decoy addresses must be in same /64 to avoid routing issues

#### Phase 4: Integration + Documentation (5 hours)

**Objective:** CLI integration, comprehensive testing, and user documentation.

**CLI Integration (2 hours):**
- Add `-6` flag: Force IPv6 only
- Add `-4` flag: Force IPv4 only
- Add `--dual-stack` flag: Scan both IPv4 and IPv6 simultaneously
- Update help text and examples

**Cross-Scanner IPv6 Testing (2 hours):**
- Combined IPv4/IPv6 tests (5 tests)
- Dual-stack stress tests (3 tests)
- Edge case tests (invalid addresses, unreachable hosts) (4 tests)

**Documentation (1 hour):**
- Create `docs/21-IPv6-GUIDE.md` (~800 lines)
  - IPv6 addressing fundamentals
  - ProRT-IP IPv6 usage examples
  - Performance considerations
  - Troubleshooting common issues
- Update README.md (full IPv6 support note)
- Update CHANGELOG.md (Sprint 5.X section)

**Files to Modify:**
- `crates/prtip-cli/src/args.rs` (+30 lines)
- `crates/prtip-cli/tests/test_cli_args_ipv6.rs` (NEW, ~180 lines)
- `docs/21-IPv6-GUIDE.md` (NEW, ~800 lines)
- README.md (+10 lines)
- CHANGELOG.md (+60 lines)

### Success Metrics

**✅ Sprint 5.X Complete When:**
- All 6 scanners support IPv6: ✅ TCP Connect, ⏸️ SYN, ⏸️ UDP, ⏸️ Stealth, ⏸️ Discovery, ⏸️ Decoy
- CLI flags working: `-6`, `-4`, `--dual-stack`
- 50+ new tests passing (total: 1,175+)
- Coverage: 64%+ (up from 62.5%)
- Zero regressions
- Comprehensive IPv6 documentation

**Performance Targets:**
- IPv6 scan speed: ≥90% of IPv4 speed (minor header parsing overhead acceptable)
- Dual-stack scans: <2x IPv4-only time (parallel scanning)
- Memory usage: <10% increase (IPv6 addresses are 128-bit vs 32-bit)

**Quality Standards:**
- Zero clippy warnings
- All tests passing
- Comprehensive error handling for IPv6-specific cases
- User-facing documentation complete

### Target Version

**Version:** v0.5.0
**Timeline:** Q1 2026 (January-March 2026)
**Dependencies:** v0.4.0 released (error handling + service detection complete)

### References

- **RFC 8200:** IPv6 specification
- **RFC 4443:** ICMPv6 for IPv6
- **RFC 4861:** Neighbor Discovery for IPv6
- **Nmap IPv6 Support:** https://nmap.org/book/port-scanning-ipv6.html

---

## Other Phase 5 Features (TBD)

### Sprint 5.Y: Idle Scanning (Zombie Host Technique)

**Priority:** HIGH
**Estimated Duration:** 8-10 hours
**ROI Score:** 8.5/10

**Objective:** Implement Nmap-style idle scanning using zombie hosts for anonymity.

**Key Features:**
- Zombie host discovery
- IPID increment detection
- Binary search for multiple open ports
- Stealth scanning via third-party hosts

**Status:** Planned for v0.5.0

---

### Sprint 5.Z: Lua Plugin System

**Priority:** HIGH
**Estimated Duration:** 15-20 hours
**ROI Score:** 8.0/10

**Objective:** Implement mlua-based plugin system for extensibility.

**Key Features:**
- Plugin lifecycle (init, scan, report)
- Sandboxing for untrusted scripts
- 5+ example plugins (HTTP enum, SSL checker, etc.)
- Plugin developer guide

**Status:** Planned for v0.5.0

---

### Sprint 5.A: TUI Interface

**Priority:** MEDIUM
**Estimated Duration:** 10-12 hours
**ROI Score:** 7.5/10

**Objective:** Create interactive terminal UI using ratatui.

**Key Features:**
- Real-time progress display
- Interactive result browsing
- Keyboard navigation
- Color themes

**Status:** Planned for v0.5.0 or v0.6.0

---

## Roadmap Summary

| Sprint | Feature | Priority | Duration | Target Version |
|--------|---------|----------|----------|----------------|
| 5.X | IPv6 Scanner Integration | MEDIUM | 25-30 hours | v0.5.0 (Q1 2026) |
| 5.Y | Idle Scanning | HIGH | 8-10 hours | v0.5.0 (Q1 2026) |
| 5.Z | Lua Plugin System | HIGH | 15-20 hours | v0.5.0 (Q1 2026) |
| 5.A | TUI Interface | MEDIUM | 10-12 hours | v0.5.0 or v0.6.0 |

**Phase 5 Total:** ~60-75 hours estimated

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-10-26 | 1.0 | Initial creation - Sprint 4.21 deferral documentation |

---

**For questions or updates, see:**
- [ROADMAP.md](01-ROADMAP.md) - Overall project roadmap
- [CHANGELOG.md](../CHANGELOG.md) - Version history
- [CLAUDE.local.md](../CLAUDE.local.md) - Current sprint status
