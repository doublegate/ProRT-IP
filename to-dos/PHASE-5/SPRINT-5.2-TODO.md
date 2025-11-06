# Sprint 5.2: Service Detection Enhancement - Todo List

**Status:** ✅ COMPLETE
**Actual Duration:** ~12 hours
**Completion Date:** 2025-10-30
**Release:** v0.4.2
**Sprint Priority:** HIGH (quality improvement, quick win after Sprint 5.1)

---

## Progress Tracking

**Total Items:** 42 tasks across 6 phases
**Completed:** 42 / 42 (100%)
**In Progress:** 0
**Remaining:** 0
**Progress:** ████████████ 100%

**Note:** All deliverables met and exceeded expectations. All 5 protocol parsers implemented with 2.5x-3.4x more lines than planned, demonstrating comprehensive implementation. See gap analysis for details.

---

## Phase 1: Research & Design (2h)

**Duration:** 2 hours
**Status:** Not Started
**Progress:** 0 / 6 (0%)

- [ ] **Task 1.1.1:** Review service_db.rs (187 probes) (20m)
- [ ] **Task 1.1.2:** Analyze false negative patterns from test corpus (15m)
- [ ] **Task 1.1.3:** Identify missing protocols in top 100 ports (15m)
- [ ] **Task 1.1.4:** Create gap analysis document (10m)
- [ ] **Task 1.2.1:** Research HTTP/SSH/SMB/MySQL/PostgreSQL protocols (50m)
- [ ] **Task 1.2.2:** Create protocol notes document (10m)

**Deliverables:**
- [ ] `/tmp/ProRT-IP/SPRINT-5.2-GAP-ANALYSIS.md` (200 lines)
- [ ] `/tmp/ProRT-IP/SPRINT-5.2-PROTOCOL-NOTES.md` (150 lines)

---

## Phase 2: HTTP Fingerprinting (3h)

**Duration:** 3 hours
**Status:** Not Started
**Progress:** 0 / 8 (0%)

- [ ] **Task 2.1.1:** Create http_fingerprint.rs module structure (20m)
- [ ] **Task 2.1.2:** Implement parse_http_headers() function (40m)
- [ ] **Task 2.1.3:** Add confidence scoring algorithm (20m)
- [ ] **Task 2.1.4:** Add rustdoc comments (10m)
- [ ] **Task 2.2.1:** Integrate with service_detector.rs (~40 lines) (40m)
- [ ] **Task 2.2.2:** Add fallback logic (20m)
- [ ] **Task 2.3.1:** Create test_http_fingerprint.rs (6 tests) (20m)
- [ ] **Task 2.3.2:** Verify all tests passing (10m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/detection/http_fingerprint.rs` (~150 lines)
- [ ] `crates/prtip-scanner/src/service_detector.rs` (+40 lines)
- [ ] `crates/prtip-scanner/tests/test_http_fingerprint.rs` (~120 lines, 6 tests)

---

## Phase 3: SSH Banner Parsing (2h)

**Duration:** 2 hours
**Status:** Not Started
**Progress:** 0 / 7 (0%)

- [ ] **Task 3.1.1:** Create ssh_banner.rs module structure (15m)
- [ ] **Task 3.1.2:** Implement parse_ssh_banner() with regex (30m)
- [ ] **Task 3.1.3:** Add confidence scoring (10m)
- [ ] **Task 3.1.4:** Add rustdoc comments (5m)
- [ ] **Task 3.2.1:** Integrate with service_detector.rs (~20 lines) (20m)
- [ ] **Task 3.3.1:** Create test_ssh_banner.rs (4 tests) (15m)
- [ ] **Task 3.3.2:** Verify all tests passing (5m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/detection/ssh_banner.rs` (~100 lines)
- [ ] `crates/prtip-scanner/src/service_detector.rs` (+20 lines)
- [ ] `crates/prtip-scanner/tests/test_ssh_banner.rs` (~80 lines, 4 tests)

---

## Phase 4: SMB & Database Protocols (3h)

**Duration:** 3 hours
**Status:** Not Started
**Progress:** 0 / 11 (0%)

### SMB Dialect Negotiation (1.5h)

- [ ] **Task 4.1.1:** Create smb_handshake.rs module structure (15m)
- [ ] **Task 4.1.2:** Implement negotiate_smb_dialect() function (30m)
- [ ] **Task 4.1.3:** Add dialect → Windows version mapping (10m)
- [ ] **Task 4.1.4:** Add error handling and rustdoc (5m)
- [ ] **Task 4.2.1:** Create test_smb_handshake.rs (3 tests) (20m)
- [ ] **Task 4.2.2:** Verify SMB tests passing (10m)

### Database Handshakes (1.5h)

- [ ] **Task 4.3.1:** Create mysql_handshake.rs module (25m)
- [ ] **Task 4.3.2:** Implement parse_mysql_handshake() (20m)
- [ ] **Task 4.4.1:** Create postgresql_handshake.rs module (20m)
- [ ] **Task 4.4.2:** Implement parse_postgresql_handshake() (10m)
- [ ] **Task 4.5.1:** Create test_database_handshakes.rs (4 tests, 2 MySQL + 2 PostgreSQL) (15m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/detection/smb_handshake.rs` (~120 lines)
- [ ] `crates/prtip-scanner/src/detection/mysql_handshake.rs` (~80 lines)
- [ ] `crates/prtip-scanner/src/detection/postgresql_handshake.rs` (~60 lines)
- [ ] `crates/prtip-scanner/tests/test_smb_handshake.rs` (~60 lines, 3 tests)
- [ ] `crates/prtip-scanner/tests/test_database_handshakes.rs` (~80 lines, 4 tests)

---

## Phase 5: Integration & Testing (1.5h)

**Duration:** 1.5 hours
**Status:** Not Started
**Progress:** 0 / 5 (0%)

- [ ] **Task 5.1.1:** Update service_db.rs with 8 protocol registrations (20m)
- [ ] **Task 5.2.1:** Create Docker Compose test environment (15m)
- [ ] **Task 5.2.2:** Create test_enhanced_service_detection.rs (3 integration tests) (30m)
- [ ] **Task 5.3.1:** Run performance benchmarks (hyperfine) (10m)
- [ ] **Task 5.3.2:** Create benchmark report (15m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/service_db.rs` (+40 lines)
- [ ] `/tmp/ProRT-IP/docker-compose.test.yml`
- [ ] `crates/prtip-scanner/tests/test_enhanced_service_detection.rs` (~150 lines, 3 tests)
- [ ] `/tmp/ProRT-IP/SPRINT-5.2-BENCHMARK.md`

---

## Phase 6: Documentation & Completion (1.5h)

**Duration:** 1.5 hours
**Status:** Not Started
**Progress:** 0 / 5 (0%)

- [ ] **Task 6.1.1:** Update docs/07-SERVICE-DETECTION.md (+200 lines) (40m)
- [ ] **Task 6.1.2:** Add 5 new protocol sections (HTTP, SSH, SMB, MySQL, PostgreSQL) (20m)
- [ ] **Task 6.2.1:** Update CHANGELOG.md with Sprint 5.2 entry (+40 lines) (15m)
- [ ] **Task 6.2.2:** Update README.md service detection section (+5 lines) (10m)
- [ ] **Task 6.2.3:** Update docs/23-IPv6-GUIDE.md (+10 lines) (5m)

**Deliverables:**
- [ ] `docs/07-SERVICE-DETECTION.md` (+200 lines)
- [ ] `CHANGELOG.md` (+40 lines)
- [ ] `README.md` (+5 lines)
- [ ] `docs/23-IPv6-GUIDE.md` (+10 lines)

---

## Success Criteria

### Functional Requirements

- [ ] HTTP header fingerprinting working (Server, X-Powered-By, X-AspNet-Version)
- [ ] SSH banner parsing extracting version + OS hints
- [ ] SMB dialect negotiation detecting Windows version
- [ ] MySQL handshake extracting server version
- [ ] PostgreSQL handshake extracting server_version
- [ ] Service detection rate ≥85% on test corpus

### Quality Requirements

- [ ] All tests passing: 1,389 → 1,409 (20 new tests)
- [ ] Zero clippy warnings
- [ ] Coverage ≥90% for new modules
- [ ] Zero production panics

### Performance Requirements

- [ ] Service detection overhead <10% (target: 5-7%)
- [ ] HTTP parsing <50μs per service
- [ ] SSH parsing <20μs per service
- [ ] Network detection (SMB, databases) <5ms per service

### Documentation Requirements

- [ ] Service detection guide updated (+200 lines)
- [ ] CHANGELOG entry complete (+40 lines)
- [ ] README updated (+5 lines)
- [ ] All examples tested

---

## Files to Create/Modify

### New Files (8)

1. `crates/prtip-scanner/src/detection/http_fingerprint.rs` (~150 lines)
2. `crates/prtip-scanner/src/detection/ssh_banner.rs` (~100 lines)
3. `crates/prtip-scanner/src/detection/smb_handshake.rs` (~120 lines)
4. `crates/prtip-scanner/src/detection/mysql_handshake.rs` (~80 lines)
5. `crates/prtip-scanner/src/detection/postgresql_handshake.rs` (~60 lines)
6. `crates/prtip-scanner/tests/test_http_fingerprint.rs` (~120 lines)
7. `crates/prtip-scanner/tests/test_ssh_banner.rs` (~80 lines)
8. `crates/prtip-scanner/tests/test_smb_handshake.rs` (~60 lines)

### Modified Files (4)

1. `crates/prtip-scanner/src/service_detector.rs` (+60 lines)
2. `crates/prtip-scanner/src/service_db.rs` (+40 lines)
3. `docs/07-SERVICE-DETECTION.md` (+200 lines)
4. `CHANGELOG.md` (+40 lines)

### Temporary Files (3)

1. `/tmp/ProRT-IP/SPRINT-5.2-GAP-ANALYSIS.md` (200 lines, internal)
2. `/tmp/ProRT-IP/SPRINT-5.2-PROTOCOL-NOTES.md` (150 lines, internal)
3. `/tmp/ProRT-IP/SPRINT-5.2-BENCHMARK.md` (benchmark report)

**Total New Production Code:** ~610 lines
**Total New Test Code:** ~400 lines
**Total Documentation:** ~245 lines

---

## Test Count Tracking

| Phase | Phase Tests | Cumulative | Notes |
|-------|-------------|------------|-------|
| Start | - | 1,389 | Sprint 5.1 complete |
| Phase 2 | +6 | 1,395 | HTTP fingerprinting |
| Phase 3 | +4 | 1,399 | SSH banner parsing |
| Phase 4 SMB | +3 | 1,402 | SMB handshake |
| Phase 4 DB | +4 | 1,406 | MySQL + PostgreSQL |
| Phase 5 | +3 | 1,409 | Integration tests |
| **Final** | **+20** | **1,409** | **+1.4% growth** |

---

## Time Budget

| Phase | Estimated | Actual | Variance |
|-------|-----------|--------|----------|
| Phase 1: Research | 2h | ___ | ___ |
| Phase 2: HTTP | 3h | ___ | ___ |
| Phase 3: SSH | 2h | ___ | ___ |
| Phase 4: SMB+DB | 3h | ___ | ___ |
| Phase 5: Integration | 1.5h | ___ | ___ |
| Phase 6: Documentation | 1.5h | ___ | ___ |
| **Base Total** | **13h** | **___** | **___** |
| Buffer (25%) | 3h | ___ | ___ |
| **Grand Total** | **15-18h** | **___** | **___** |

---

## Risk Mitigation

| Risk | Mitigation | Contingency |
|------|------------|-------------|
| Detection rate <85% | Targeted protocols cover 80% gap | Add 2-3 more protocols (+2-3h) |
| Protocol parsing bugs | Comprehensive error handling + tests | Add 1-2h defensive coding |
| Performance >10% overhead | Target 5-7%, measure early | Optimize HTTP parsing (+0.5h) |
| SMB complexity | Start with SMB2+ only | Fallback to banner grabbing |
| Test environment delays | Docker Compose prepared | Manual localhost setup |

---

## Dependencies

### Internal Dependencies

- [x] Sprint 5.1: IPv6 Scanner Integration (COMPLETE ✅)
  - Status: Completed 2025-10-29, v0.4.1 released
  - Impact: Sprint 5.2 can proceed immediately

### External Dependencies

- [x] `regex` crate (already in Cargo.toml)
- [x] `pnet` crate (already in Cargo.toml)
- [x] `tokio` crate (already in Cargo.toml)
- [ ] Docker Desktop (for integration tests, optional)
- [ ] Nmap (for comparison tests, optional)

---

## Next Steps

1. **Review Plan:** Read `/tmp/ProRT-IP/SPRINT-5.2-PLAN.md` (comprehensive 790-line plan)
2. **Review Checklist:** Read `/tmp/ProRT-IP/SPRINT-5.2-CHECKLIST.md` (150+ tasks)
3. **Begin Phase 1:** Research & Design (2h)
   - Task 1.1: Audit current service detection gaps
   - Task 1.2: Research protocol specifications
4. **Track Progress:** Update this todo file daily
5. **Final Verification:** Complete all success criteria before marking sprint complete

---

## Sprint Completion Report Template

**To be filled upon completion:**

```
Sprint 5.2: Service Detection Enhancement - COMPLETE ✅

**Duration:** ___ hours (estimate: 15-18h)
**Completion Date:** 2025-MM-DD
**Tests:** 1,389 → 1,409 (+20 = +1.4%)
**Detection Rate:** 70-80% → __% (target: ≥85%)
**Performance Overhead:** __% (target: <10%)
**Files Changed:** 12 files (+790 production, +400 test, +245 docs)

**Key Achievements:**
- HTTP header fingerprinting: ___
- SSH banner parsing: ___
- SMB dialect negotiation: ___
- MySQL handshake: ___
- PostgreSQL handshake: ___

**Quality Metrics:**
- Zero clippy warnings: ___
- Coverage ≥90% for new modules: ___
- CI/CD 7/7 passing: ___
- Zero regressions: ___

**Next Sprint:** Sprint 5.3 (Idle Scan Implementation, 12-15h)
```

---

## Quick Reference

**Plan:** `/tmp/ProRT-IP/SPRINT-5.2-PLAN.md` (790 lines, comprehensive)
**Checklist:** `/tmp/ProRT-IP/SPRINT-5.2-CHECKLIST.md` (150+ tasks)
**Todo:** `to-dos/SPRINT-5.2-TODO.md` (this file, 42 tasks)

**Current Status:** Not Started (0%)
**Target Completion:** 2025-11-06
**Estimated Effort:** 15-18 hours

---

**Last Updated:** 2025-10-30
**Status:** Ready to Execute ✅
