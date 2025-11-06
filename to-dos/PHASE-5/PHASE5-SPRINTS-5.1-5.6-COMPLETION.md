# Phase 5 Sprints 5.1-5.6 Completion To-Dos

**Generated:** 2025-11-04
**Based On:** Gap analysis from phase5-gap-analysis.md
**Scope:** Complete remaining work for Sprints 5.1-5.6
**Total Items:** 12 tasks across 4 priority levels

---

## Executive Summary

**Status:** 5 of 6 planned sprints COMPLETE. One critical sprint (Code Coverage 5.6) NOT STARTED.

**Critical Finding:** The planned "Sprint 5.6: Code Coverage Sprint" was replaced with an unplanned "Sprint 5.6: TLS Network Testing & SNI Support". Coverage remains at 62.5% instead of targeted 80%.

**Immediate Action Required:** Execute Code Coverage Sprint (15-18h effort) before continuing to Sprints 5.7-5.10.

---

## Priority Levels

- **CRITICAL:** Blocking issue, must complete for v1.0 readiness
- **HIGH:** Should complete for quality/consistency
- **MEDIUM:** Nice to have, improves project quality
- **LOW:** Optional improvements

---

## CRITICAL Priority (Execute Immediately)

### C-1: Execute Sprint 5.6 Code Coverage Sprint

**Priority:** CRITICAL
**Status:** NOT STARTED
**Effort:** 15-18 hours
**Blocking:** v1.0 release (80% coverage requirement)
**Due:** Before Sprint 5.7

**Description:**
Execute the PLANNED Sprint 5.6 (Code Coverage Sprint), NOT the actual TLS SNI sprint that was labeled as 5.6.

**Gap:**
- Current coverage: 62.5%
- Target coverage: 80%
- Gap: -17.5 percentage points
- No CI/CD coverage integration
- No coverage badge in README
- No coverage-driven bug fixes

**Required Deliverables:**

1. **Coverage Analysis (3h)**
   - [ ] Generate baseline coverage report using cargo-tarpaulin or llvm-cov
   - [ ] Identify modules <60% coverage
   - [ ] Identify critical untested paths (parsers, error handlers, edge cases)
   - [ ] Create coverage target document (by module)
   - [ ] Document intentional exclusions with justification

2. **Critical Path Testing (6h)**
   - [ ] Add 8+ IPv6 parser edge case tests
   - [ ] Add 8+ TCP/UDP parser edge case tests
   - [ ] Add 6+ idle scan edge case tests
   - [ ] Add 10+ error propagation tests
   - [ ] Add 8+ circuit breaker/retry edge case tests
   - [ ] Add 6+ service detection edge case tests
   - [ ] Add 6+ TLS analysis edge case tests
   - **Subtotal:** 52+ new tests

3. **Bug Fixes from Coverage (4h)**
   - [ ] Run all new tests from phase 2
   - [ ] Triage discovered bugs (critical vs minor)
   - [ ] Fix critical bugs (crashes, panics, wrong results)
   - [ ] Fix minor bugs (logging, formatting, edge cases)
   - [ ] Validate all bugs fixed

4. **CI/CD Integration (2h)**
   - [ ] Create `.github/workflows/coverage.yml`
   - [ ] Configure coverage upload (Codecov or Coveralls)
   - [ ] Set failure threshold (coverage <75% fails PR)
   - [ ] Add coverage badge to README.md
   - [ ] Document coverage in docs/06-TESTING.md

5. **Validation & Completion (1.5h)**
   - [ ] Run final coverage report
   - [ ] Verify ≥80% overall coverage
   - [ ] Verify ≥90% critical module coverage
   - [ ] Create Sprint 5.6 (Coverage) completion report
   - [ ] Update CHANGELOG.md with Sprint 5.6 (Coverage) entry

**Verification Commands:**
```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# Check coverage percentage
cargo tarpaulin --out Stdout | grep "Coverage:"

# Verify CI workflow exists
ls -la .github/workflows/coverage.yml

# Check README has coverage badge
grep -i "coverage" README.md | grep badge
```

**Success Criteria:**
- ✅ Overall coverage ≥80%
- ✅ Critical modules (parsers, detection, security) ≥90%
- ✅ 52+ new tests added and passing
- ✅ CI/CD coverage job runs on PRs
- ✅ Coverage badge visible in README
- ✅ 0 critical bugs remaining from coverage analysis

**Estimated Completion:** 15-18 hours

**Dependencies:** None (can start immediately)

**Blocks:** Sprint 5.7 (Fuzz Testing), Sprint 5.8 (Plugin System)

---

## HIGH Priority (Complete This Week)

### H-1: Rename Actual Sprint 5.6 to 5.5b

**Priority:** HIGH
**Status:** NOT STARTED
**Effort:** 30 minutes
**Due:** This week

**Description:**
Resolve sprint numbering confusion by renaming the actual "Sprint 5.6: TLS Network Testing & SNI Support" to "Sprint 5.5b: TLS Network Testing & SNI Support".

**Gap:**
- CHANGELOG has "Sprint 5.6" for TLS SNI work
- CLAUDE.local.md has "Sprint 5.6" for TLS SNI work
- Phase 5 plan has "Sprint 5.6" for Code Coverage work
- Causes confusion about what Sprint 5.6 actually is

**Required Actions:**
- [ ] Update CHANGELOG.md section header: "Sprint 5.6" → "Sprint 5.5b"
- [ ] Update CLAUDE.local.md: "Sprint 5.6" → "Sprint 5.5b"
- [ ] Add note in both files explaining numbering change
- [ ] Preserve "Sprint 5.6" designation for Code Coverage sprint
- [ ] Update README.md if it references Sprint 5.6 TLS work

**Example Renaming:**
```markdown
# OLD
## [Unreleased]
### Added - Sprint 5.6: TLS Network Testing & SNI Support

# NEW
## [Unreleased]
### Added - Sprint 5.5b: TLS Network Testing & SNI Support
**Note:** Originally labeled Sprint 5.6, renamed to 5.5b to preserve Sprint 5.6 designation for planned Code Coverage Sprint per Phase 5 development plan.
```

**Success Criteria:**
- ✅ CHANGELOG Sprint 5.6 → 5.5b
- ✅ CLAUDE.local.md Sprint 5.6 → 5.5b
- ✅ Note added explaining rename
- ✅ "Sprint 5.6" available for Code Coverage Sprint

**Estimated Completion:** 30 minutes

---

### H-2: Update SPRINT-5.2-TODO.md Status

**Priority:** HIGH
**Status:** NOT STARTED
**Effort:** 5 minutes
**Due:** This week

**Description:**
Update the Sprint 5.2 TODO file to reflect actual completion status.

**Gap:**
- File header shows "Status: Not Started"
- File shows 0% progress (0 / 42 tasks complete)
- But Sprint 5.2 was completed and released as v0.4.2 (2025-10-30)
- All 5 protocol parsers exist and are production-ready

**Required Actions:**
- [ ] Update `to-dos/SPRINT-5.2-TODO.md` header: "Not Started" → "COMPLETE"
- [ ] Update progress: "0 / 42 (0%)" → "42 / 42 (100%)"
- [ ] Add completion date: 2025-10-30
- [ ] Add note referencing v0.4.2 release
- [ ] Optionally: Archive file to `to-dos/archive/` or `daily_logs/`

**Success Criteria:**
- ✅ File status updated to COMPLETE
- ✅ Progress shows 100%
- ✅ No confusion about Sprint 5.2 status

**Estimated Completion:** 5 minutes

---

### H-3: Clarify Test Count in README

**Priority:** HIGH
**Status:** NOT STARTED
**Effort:** 30 minutes
**Due:** This week

**Description:**
Add clear explanation of test count categories to resolve discrepancy between different counts.

**Gap:**
- README badge: 839 tests passing
- README Sprint 5.5: 868/868 tests passing
- CLAUDE.local.md: 1,618 tests total
- cargo test output: 1,618 tests total

**Required Actions:**
- [ ] Add test count breakdown section to README
- [ ] Explain categories:
  - Total workspace tests: 1,618
  - Active production tests: 839
  - Platform-specific ignored: ~X
  - Benchmark tests: ~X
  - Doc tests: ~X
- [ ] Update badge to show both: "839/1618 tests passing"
- [ ] Document in docs/06-TESTING.md
- [ ] Explain why some tests are ignored (platform-specific)

**Example Section:**
```markdown
### Test Suite Composition

**Total Tests:** 1,618 (100% passing)
- **Active Production Tests:** 839 tests
  - Unit tests: ~400
  - Integration tests: ~400
  - Module-specific tests: ~39
- **Platform-Specific (Ignored on some platforms):** ~X tests
- **Benchmark Tests:** ~X tests
- **Documentation Tests:** ~X tests

All 1,618 tests pass successfully on their respective platforms.
```

**Success Criteria:**
- ✅ Test count categories documented
- ✅ No confusion about 839 vs 1,618
- ✅ README and TESTING.md aligned

**Estimated Completion:** 30 minutes

---

## MEDIUM Priority (Nice to Have)

### M-1: Verify ICMP Monitor Implementation

**Priority:** MEDIUM
**Status:** NOT STARTED
**Effort:** 30 minutes
**Due:** Next 2 weeks

**Description:**
Verify that Sprint 5.4 ICMP monitor functionality exists, even if not as separate module.

**Gap:**
- Sprint 5.4 plan specified `icmp_monitor.rs` module (~180 lines)
- No standalone `icmp_monitor.rs` file found
- May be integrated into `adaptive_rate_limiter_v3.rs`

**Required Actions:**
- [ ] Search for ICMP Type 3 Code 13 handling in codebase
- [ ] Check if ICMP monitor is integrated into V3 rate limiter
- [ ] Verify rate limit detection functionality works
- [ ] Document findings
- [ ] If missing: Create issue for future work
- [ ] If present: Update documentation to clarify integration

**Verification Commands:**
```bash
# Search for ICMP monitoring code
grep -r "ICMP.*Type 3.*Code 13" crates/
grep -r "icmp.*monitor" crates/
grep -r "Communication Administratively Prohibited" crates/

# Check V3 rate limiter for ICMP handling
grep -A10 -B10 "icmp\|ICMP" crates/prtip-scanner/src/adaptive_rate_limiter_v3.rs
```

**Success Criteria:**
- ✅ ICMP monitor functionality verified (exists or documented as missing)
- ✅ If present: Integration documented
- ✅ If missing: Issue created for tracking

**Estimated Completion:** 30 minutes

---

### M-2: Add Coverage Badge to README (After C-1)

**Priority:** MEDIUM (depends on C-1)
**Status:** BLOCKED (waiting for C-1)
**Effort:** 15 minutes
**Due:** After Sprint 5.6 coverage complete

**Description:**
Add coverage badge to README once CI/CD coverage integration is complete.

**Gap:**
- README has test passing badge
- No coverage badge present
- Industry standard to show coverage %

**Required Actions:**
- [ ] Configure Codecov or Coveralls integration
- [ ] Generate coverage badge URL
- [ ] Add badge to README.md (top section with other badges)
- [ ] Verify badge updates automatically on commits
- [ ] Update badge link in CLAUDE.md if referenced

**Example Badge:**
```markdown
[![Coverage](https://img.shields.io/codecov/c/github/doublegate/ProRT-IP?label=coverage)]
```

**Success Criteria:**
- ✅ Coverage badge visible in README
- ✅ Badge shows accurate coverage percentage
- ✅ Badge updates automatically

**Estimated Completion:** 15 minutes

**Blocks:** None

**Blocked By:** C-1 (CI/CD coverage integration must be done first)

---

### M-3: Verify All CLI Flags Documented in --help

**Priority:** MEDIUM
**Status:** NOT STARTED
**Effort:** 1 hour
**Due:** Next 2 weeks

**Description:**
Verify all CLI flags from Sprints 5.1-5.6 are documented in --help output.

**Gap:**
- Multiple sprints added CLI flags
- Need to verify all are in --help
- Need to verify examples are accurate

**Flags to Verify:**

**Sprint 5.1 (IPv6):**
- `-6` / `--ipv6-only`
- `-4` / `--ipv4-only`
- `--dual-stack`
- `--prefer-ipv6`
- `--prefer-ipv4`

**Sprint 5.3 (Idle):**
- `-sI <ZOMBIE_IP>`
- `-I`
- `--idle-scan <ZOMBIE_IP>`

**Sprint 5.4 (Rate Limiting):**
- `--adaptive-rate` (if still present)
- `--rate-limit-threshold N` (if present)
- `--backoff-max N` (if present)
- Note: May have changed with V3 promotion

**Sprint 5.5 (TLS):**
- `--tls-analysis` (if added)
- `--no-tls` (if still present)

**Required Actions:**
- [ ] Run `prtip --help` and capture output
- [ ] Verify each flag listed above is present
- [ ] Check flag descriptions are accurate
- [ ] Verify examples use correct syntax
- [ ] Update help text if any flags missing
- [ ] Test each flag works as documented

**Verification Command:**
```bash
# Check help output
cargo build --release
./target/release/prtip --help | grep -E "\-6|\-4|dual-stack|ipv6|ipv4|sI|idle|adaptive|tls"
```

**Success Criteria:**
- ✅ All Sprint 5.1-5.6 flags documented in --help
- ✅ Examples accurate
- ✅ No obsolete flags present

**Estimated Completion:** 1 hour

---

## LOW Priority (Optional Improvements)

### L-1: Add Links to Guides from README

**Priority:** LOW
**Status:** NOT STARTED
**Effort:** 15 minutes
**Due:** Anytime

**Description:**
Ensure all Phase 5 guides are linked from README documentation section.

**Gap:**
- 5 guides exist: IPv6, Service Detection, Idle Scan, Rate Limiting, TLS
- Need to verify all are linked from README

**Required Actions:**
- [ ] Check README "Documentation" section
- [ ] Verify links to all 5 guides:
  - `docs/23-IPv6-GUIDE.md`
  - `docs/24-SERVICE-DETECTION-GUIDE.md` (if exists)
  - `docs/25-IDLE-SCAN-GUIDE.md`
  - `docs/26-RATE-LIMITING-GUIDE.md`
  - `docs/27-TLS-CERTIFICATE-GUIDE.md`
- [ ] Add brief description for each guide
- [ ] Ensure links work (files exist at paths)

**Success Criteria:**
- ✅ All guides linked from README
- ✅ Links work
- ✅ Descriptions accurate

**Estimated Completion:** 15 minutes

---

### L-2: Create Service Detection Guide (If Missing)

**Priority:** LOW
**Status:** UNKNOWN
**Effort:** 2-3 hours
**Due:** Before v0.5.0

**Description:**
Create comprehensive service detection guide if it doesn't exist.

**Gap:**
- Sprint 5.2 completed service detection enhancements
- Unclear if dedicated guide was created
- May be documented in other guides

**Required Actions:**
- [ ] Check if `docs/24-SERVICE-DETECTION-GUIDE.md` exists
- [ ] If missing: Create comprehensive guide covering:
  - HTTP fingerprinting
  - SSH banner parsing
  - SMB detection
  - MySQL detection
  - PostgreSQL detection
  - 187 embedded probes
  - Detection rate methodology
  - Usage examples
- [ ] If exists: Verify it's complete and up-to-date

**Success Criteria:**
- ✅ Service detection guide exists
- ✅ Covers all 5 protocol parsers
- ✅ Linked from README

**Estimated Completion:** 2-3 hours (if creating new)

---

## Summary Statistics

**Total Tasks:** 12
- CRITICAL: 1 (Sprint 5.6 Coverage)
- HIGH: 3 (Rename, TODO update, Test count)
- MEDIUM: 3 (ICMP verify, Coverage badge, CLI flags)
- LOW: 2 (Guide links, Service Detection guide)

**Total Estimated Effort:** 21-24 hours
- CRITICAL: 15-18 hours
- HIGH: 1.1 hours
- MEDIUM: 1.75 hours
- LOW: 2.25-3.25 hours

**Blocking Priority Order:**
1. C-1: Execute Sprint 5.6 Coverage (15-18h) - CRITICAL, blocks v1.0
2. H-1: Rename Sprint 5.6 → 5.5b (30m) - Clarity
3. H-2: Update TODO status (5m) - Cleanup
4. H-3: Clarify test counts (30m) - Documentation
5. M-1: Verify ICMP monitor (30m) - Verification
6. M-2: Add coverage badge (15m) - After C-1
7. M-3: Verify CLI flags (1h) - Quality
8. L-1: Link guides (15m) - Polish
9. L-2: Service Detection guide (2-3h) - Optional

---

## Next Steps

### Immediate (This Week)

1. **Execute C-1 (Sprint 5.6 Coverage)** - CRITICAL
   - This is the #1 priority
   - Blocks v1.0 release
   - Establishes quality foundation for remaining Phase 5 sprints

2. **Complete H-1, H-2, H-3** - HIGH
   - Quick wins (1.1 hours total)
   - Resolves documentation confusion
   - Improves project clarity

### Short-Term (Next 2 Weeks)

3. **Complete M-1, M-2, M-3** - MEDIUM
   - Verification and quality improvements
   - M-2 depends on C-1 completion

### Long-Term (Before v0.5.0)

4. **Complete L-1, L-2** - LOW
   - Polish and optional improvements
   - Not blocking but nice to have

---

## Verification Checklist

After completing all tasks, verify:

- [ ] Coverage ≥80% overall
- [ ] Critical modules ≥90% coverage
- [ ] CI/CD coverage job runs successfully
- [ ] Coverage badge shows in README
- [ ] All sprint completion docs updated
- [ ] No sprint numbering confusion
- [ ] Test counts clearly explained
- [ ] All guides linked from README
- [ ] All CLI flags documented
- [ ] Zero blocking issues for Phase 5 continuation

---

## References

- **Gap Analysis:** `/tmp/ProRT-IP/phase5-gap-analysis.md`
- **Phase 5 Plan:** `to-dos/v0.5.0-PHASE5-DEVELOPMENT-PLAN.md`
- **Phase 5 Part 2 Plan:** `to-dos/v0.5.0-PHASE5-PART2-SPRINTS-5.6-5.10.md`
- **CHANGELOG:** `CHANGELOG.md`
- **README:** `README.md`
- **Local Memory:** `CLAUDE.local.md`

---

**END OF TO-DOS DOCUMENT**
