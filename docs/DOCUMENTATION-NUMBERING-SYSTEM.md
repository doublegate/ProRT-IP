# ProRT-IP Documentation Numbering System

**Version:** 1.0.0  
**Date:** 2025-11-15  
**Status:** APPROVED (Proposed Implementation)  
**Purpose:** Define logical, gap-free numbering system for ordered documentation

---

## Table of Contents

1. [Overview](#overview)
2. [Current State Analysis](#current-state-analysis)
3. [Numbering Principles](#numbering-principles)
4. [Number Allocation Strategy](#number-allocation-strategy)
5. [Proposed Renumbering Plan](#proposed-renumbering-plan)
6. [Migration Process](#migration-process)
7. [Future Number Assignments](#future-number-assignments)
8. [References](#references)

---

## Overview

### Purpose

This document establishes a **systematic numbering scheme** (00-99) for core ProRT-IP documentation that requires ordered reading or logical grouping.

### Goals

1. **Eliminate Gaps**: Continuous numbering within ranges (no skipped numbers)
2. **Logical Grouping**: Related documents in adjacent numbers
3. **Future-Proof**: Reserved ranges for expansion
4. **Discoverability**: Number indicates document category
5. **Simplicity**: Easy to remember, easy to maintain

### Scope

**Applies To:**
- Core reference documentation in `/docs/`
- Documents intended for sequential reading
- Guides that build on each other

**Does NOT Apply To:**
- Architecture docs (`TUI-ARCHITECTURE.md` - unnumbered OK)
- Task tracking (`SPRINT-6.1-TODO.md` - uses sprint numbers)
- Archives (`PHASE-5-README-ARCHIVE.md` - uses phase numbers)
- Root files (`README.md`, `CHANGELOG.md` - no numbers)
- Templates (`TEMPLATE-*.md` - no numbers)

---

## Current State Analysis

### Existing Numbered Files (As Of 2025-11-15)

```
Range 00-09: Meta & Navigation (10 files)
├── 00-ARCHITECTURE.md             ✅ Core system overview
├── 00-DOCUMENTATION-INDEX.md      ✅ Navigation hub
├── 01-ROADMAP.md                  ✅ Project timeline
├── 02-TECHNICAL-SPECS.md          ✅ Technical specifications
├── 03-DEV-SETUP.md                ✅ Developer onboarding
├── 04-IMPLEMENTATION-GUIDE.md     ✅ Code structure guide
├── 05-API-REFERENCE.md            ✅ API documentation
├── 06-TESTING.md                  ✅ Testing strategy
├── 07-PERFORMANCE.md              ✅ Performance guide
├── 08-SECURITY.md                 ✅ Security practices
└── 09-FAQ.md                      ✅ Frequently asked questions

Range 10-19: Development & Process (10 files)
├── 10-PROJECT-STATUS.md           ✅ Current status tracking
├── 11-RELEASE-PROCESS.md          ✅ Release procedures
├── 12-BENCHMARKING-GUIDE.md       ✅ Benchmarking howto
├── 13-PLATFORM-SUPPORT.md         ✅ Platform compatibility
├── 14-NMAP_COMPATIBILITY.md       ⚠️  Has underscore (rename needed)
├── 15-PHASE4-COMPLIANCE.md        ⚠️  Should archive (historical)
├── 16-REGRESSION-FIX-STRATEGY.md  ⚠️  Should archive (historical)
├── 17-TESTING-INFRASTRUCTURE.md   ✅ Testing framework
├── 18-EFFICIENCY_REPORT.md        ⚠️  Has underscore (rename needed)
└── 19-EVASION-GUIDE.md            ✅ Evasion techniques

Range 20-29: Feature Guides (10 files used, conflicts exist)
├── 20-PHASE4-ENHANCEMENTS.md      ⚠️  Duplicate in archive (delete)
├── 21-PERFORMANCE-GUIDE.md        ⚠️  Duplicate of 07-PERFORMANCE? (verify)
├── 22.1-CLAUDE-POST-PHASE4_1of2.md ❌ Non-standard numbering
├── 22.2-CLAUDE-POST-PHASE4_2of2.md ❌ Non-standard numbering
├── 23-IPv6-GUIDE.md               ✅ IPv6 feature guide
├── 24-SERVICE-DETECTION.md        ⚠️  Missing -GUIDE suffix
├── 25-IDLE-SCAN-GUIDE.md          ✅ Idle scan guide
├── 26-RATE-LIMITING-GUIDE.md      ✅ Rate limiting guide
├── 27-TLS-CERTIFICATE-GUIDE.md    ✅ TLS certificate guide
├── 28-CI-CD-COVERAGE.md           ✅ CI/CD coverage guide
└── 29-FUZZING-GUIDE.md            ✅ Fuzzing guide

Range 30-39: User Documentation (6 files used, CONFLICT on 34)
├── 30-PLUGIN-SYSTEM-GUIDE.md      ✅ Plugin guide
├── 31-BENCHMARKING-GUIDE.md       ⚠️  Duplicate of 12? (verify)
├── 32-USER-GUIDE.md               ✅ User manual
├── 33-TUTORIALS.md                ✅ Tutorial collection
├── 34-EXAMPLES-GALLERY.md         ⚠️  CONFLICT: 34 used 3x
├── 34-EXAMPLES.md                 ⚠️  CONFLICT: 34 used 3x
└── 34-PERFORMANCE-CHARACTERISTICS.md ⚠️ CONFLICT: 34 used 3x (should be in 40s)

Range 40-99: UNUSED (reserved for future)
```

### Identified Issues

#### Critical Issues (Block Discoverability)

1. **Number Conflicts:**
   - `34` used THREE times (EXAMPLES-GALLERY, EXAMPLES, PERFORMANCE-CHARACTERISTICS)
   - Cannot have multiple files with same number

2. **Non-Standard Numbering:**
   - `22.1` and `22.2` use decimals (not allowed in numbering system)
   - Should be `22` and `23` or archived

3. **Duplicate Content:**
   - `07-PERFORMANCE.md` vs `21-PERFORMANCE-GUIDE.md` (same topic, different ranges)
   - `12-BENCHMARKING-GUIDE.md` vs `31-BENCHMARKING-GUIDE.md` (exact duplicate?)
   - `20-PHASE4-ENHANCEMENTS.md` exists in both `docs/` and `docs/archive/`

#### Medium Issues (Logical Grouping)

4. **Misplaced Documents:**
   - `34-PERFORMANCE-CHARACTERISTICS.md` in user docs (30s) should be in operations (40s)
   - `30-PLUGIN-SYSTEM-GUIDE.md` in user docs should be in feature guides (20s)
   - `28-CI-CD-COVERAGE.md` might fit better in development (10s)

5. **Archive Candidates:**
   - `15-PHASE4-COMPLIANCE.md` (historical, completed phase)
   - `16-REGRESSION-FIX-STRATEGY.md` (historical)
   - `20-PHASE4-ENHANCEMENTS.md` (historical)
   - `22.1/22.2-CLAUDE-POST-PHASE4` (notes, should archive)

6. **Missing Ranges:**
   - Range 40-49: Operations guides (Performance, Monitoring, Troubleshooting)
   - Range 50-59: Reference material (CLI, API, Configuration)
   - Range 60-99: Completely unused (future expansion)

---

## Numbering Principles

### Core Rules

1. **Unique Numbers**: Each number (00-99) used ONCE maximum
2. **No Decimals**: Use integers only (`22`, `23` not `22.1`, `22.2`)
3. **Zero-Padded**: Always two digits (`00-09`, not `0-9`)
4. **Sequential**: No intentional gaps within active ranges
5. **Reserved Ranges**: Leave expansion room (e.g., 40-49 empty for future operations docs)

### Ordering Strategy

**Documents within a range should be ordered by:**
1. **Dependency**: Foundation concepts before advanced topics
2. **Frequency**: More commonly needed documents get lower numbers
3. **Logical Flow**: Natural reading progression

**Example: Feature Guides (20-29)**
```
20 - SYN Scan (most common, foundation)
21 - Service Detection (builds on SYN)
22 - IPv6 (extension of core scanning)
23 - Stealth Techniques (advanced)
24 - TLS Certificate Analysis (specialized)
```

### Range Allocation Philosophy

**Small Ranges (10 numbers):**
- Forces prioritization
- Prevents clutter
- Clear scope boundaries

**Reserved Buffers:**
- Always leave 10-20% empty for future additions
- Example: Use 20-27 now, reserve 28-29 for future feature guides

---

## Number Allocation Strategy

### Definitive Range Assignments

```
00-09: Meta & Navigation (FULL - 10/10 used)
       Purpose: Project overview, structure, navigation
       Status: ✅ Complete, no gaps acceptable
       Expansion: None needed (complete set)

10-19: Development & Infrastructure (ACTIVE - 10/10 used, 3 need archiving)
       Purpose: Development workflows, testing, infrastructure
       Status: ⚠️ Needs cleanup (archive 15, 16, 20)
       Expansion: After cleanup, 3 slots available (15, 16, 18)
       
20-29: Feature Guides (ACTIVE - 7/10 used correctly)
       Purpose: How to use specific ProRT-IP features
       Status: ⚠️ Needs renumbering (resolve 22.1/22.2, move 30)
       Reserved: 28-29 for future features
       
30-39: User Documentation (ACTIVE - 4/10 used correctly)
       Purpose: End-user guides, tutorials, examples
       Status: ⚠️ Needs renumbering (resolve 34 conflict)
       Reserved: 36-39 for future user content
       
40-49: Operations & Performance (PROPOSED - 0/10 used)
       Purpose: Running ProRT-IP in production
       Candidates: Performance tuning, monitoring, troubleshooting
       Reserved: Entire range for Phase 7+ operations docs
       
50-59: Reference Material (PROPOSED - 0/10 used)
       Purpose: Quick reference, cheat sheets, API specs
       Candidates: CLI reference, config reference, protocol specs
       Reserved: Entire range for comprehensive reference docs
       
60-69: Advanced Topics (RESERVED)
       Purpose: Kernel internals, packet parsing, optimization
       Reserved: Phase 8+ advanced developer content
       
70-99: RESERVED FOR FUTURE EXPANSION
       Purpose: Unknown future needs
       Reserved: Do not allocate until 00-69 exceed 80% utilization
```

### Allocation Guidelines

**When to Number a Document:**
- ✅ Core reference material (read sequentially)
- ✅ Build on prerequisite knowledge
- ✅ Part of learning path
- ✅ Permanent reference (not phase/sprint-specific)

**When NOT to Number:**
- ❌ Temporary content (sprint TODOs, completion reports)
- ❌ Phase-specific archives
- ❌ Standalone architecture docs (TUI-ARCHITECTURE.md)
- ❌ Process documents (standards, schedules)
- ❌ Templates

---

## Proposed Renumbering Plan

### Phase 1: Critical Fixes (Immediate)

**Resolve Number Conflicts:**

| Current | Action | New Name | Rationale |
|---------|--------|----------|-----------|
| `22.1-CLAUDE-POST-PHASE4_1of2.md` | Archive | `archive/PHASE-4-CLAUDE-NOTES-PART-1.md` | Historical, non-standard numbering |
| `22.2-CLAUDE-POST-PHASE4_2of2.md` | Archive | `archive/PHASE-4-CLAUDE-NOTES-PART-2.md` | Historical, non-standard numbering |
| `34-EXAMPLES.md` | Merge/Delete | Merge into `34-EXAMPLES-GALLERY.md` | Duplicate content |
| `34-PERFORMANCE-CHARACTERISTICS.md` | Renumber | `40-PERFORMANCE-CHARACTERISTICS.md` | Move to operations range |
| `20-PHASE4-ENHANCEMENTS.md` | Delete | (already in archive/) | Duplicate |
| `15-PHASE4-COMPLIANCE.md` | Archive | `archive/PHASE-4-COMPLIANCE.md` | Historical |
| `16-REGRESSION-FIX-STRATEGY.md` | Archive | `archive/PHASE-4-REGRESSION-FIX.md` | Historical |

**Fix Naming Inconsistencies:**

| Current | New Name | Rationale |
|---------|----------|-----------|
| `24-SERVICE-DETECTION.md` | `24-SERVICE-DETECTION-GUIDE.md` | Add -GUIDE suffix |
| `14-NMAP_COMPATIBILITY.md` | `14-NMAP-COMPATIBILITY.md` | Fix underscore |
| `18-EFFICIENCY_REPORT.md` | `18-EFFICIENCY-REPORT.md` | Fix underscore |

**After Phase 1: Clean Numbering**
```
00-09: Meta (10/10) ✅ No conflicts
10-19: Development (7/10 after archiving 15,16,20) ✅ 3 slots free
20-29: Features (8/10 after fixing 22.1/22.2) ✅ 2 slots free (28-29)
30-39: User Docs (3/10 after resolving 34) ✅ 7 slots free (35-39)
40-49: Operations (1/10 after moving 34) ✅ 9 slots free
```

### Phase 2: Logical Reorganization (Quarterly Review)

**Verify Duplicates:**

| File 1 | File 2 | Action |
|--------|--------|--------|
| `07-PERFORMANCE.md` | `21-PERFORMANCE-GUIDE.md` | Investigate: Different content? If same, delete 21 |
| `12-BENCHMARKING-GUIDE.md` | `31-BENCHMARKING-GUIDE.md` | Investigate: Same content? If yes, delete 31 |

**Optimal Grouping (Optional):**

Move feature guides together (if duplicates confirmed):
```
Current:                        Proposed:
07-PERFORMANCE.md               07-PERFORMANCE.md (keep)
21-PERFORMANCE-GUIDE.md         (delete if duplicate)

20-??? (available)              20-SYN-SCAN-GUIDE.md (new, foundation)
21-??? (available)              21-SERVICE-DETECTION-GUIDE.md (was 24)
22-??? (archived)               22-STEALTH-TECHNIQUES-GUIDE.md (new, consolidate 19?)
23-IPv6-GUIDE.md                23-IPv6-GUIDE.md (keep)
24-SERVICE-DETECTION.md         24-TLS-CERTIFICATE-GUIDE.md (was 27)
25-IDLE-SCAN-GUIDE.md           25-IDLE-SCAN-GUIDE.md (keep)
26-RATE-LIMITING-GUIDE.md       26-RATE-LIMITING-GUIDE.md (keep)
27-TLS-CERTIFICATE-GUIDE.md     27-PLUGIN-SYSTEM-GUIDE.md (was 30)
28-CI-CD-COVERAGE.md            28-CI-CD-COVERAGE.md (or move to 15)
29-FUZZING-GUIDE.md             29-FUZZING-GUIDE.md (keep)
30-PLUGIN-SYSTEM-GUIDE.md       (moved to 27)
```

**Note:** Phase 2 requires content analysis to confirm duplicates. Do NOT execute without verification.

### Phase 3: Long-Term (Future Phases)

**Expansion Strategy:**

When adding new documentation:

1. **Identify Category:**
   - Meta/Navigation? → 00-09 (likely full, consider unnumbered)
   - Development? → 10-19 (3 slots after cleanup: 15, 16, 18)
   - Feature Guide? → 20-29 (2 slots: 28-29 reserved)
   - User Doc? → 30-39 (7 slots: 35-39)
   - Operations? → 40-49 (9 slots available)
   - Reference? → 50-59 (10 slots available)

2. **Assign Lowest Available Number:**
   - Within category range
   - Maintains sequential order
   - Preserves logical dependencies

3. **Update Index:**
   - Add to `00-DOCUMENTATION-INDEX.md`
   - Note number assignment in `CHANGELOG.md`
   - Run link validation

---

## Migration Process

### Pre-Migration Checklist

- [ ] Backup current documentation state (git commit)
- [ ] Identify all cross-references (grep for filename patterns)
- [ ] Create migration tracking issue
- [ ] Schedule maintenance window (freeze doc changes)

### Migration Steps

**For Each Rename:**

1. **Identify Cross-References:**
   ```bash
   # Find all files linking to old name
   grep -r "OLD-NAME.md" docs/ to-dos/ README.md CHANGELOG.md
   ```

2. **Update Cross-References:**
   ```bash
   # Update all references (careful with sed)
   find . -name "*.md" -exec sed -i 's/OLD-NAME\.md/NEW-NAME.md/g' {} +
   ```

3. **Rename File (Preserve Git History):**
   ```bash
   git mv docs/OLD-NAME.md docs/NEW-NAME.md
   ```

4. **Update Documentation Index:**
   ```bash
   # Edit 00-DOCUMENTATION-INDEX.md
   # Update entry with new name and number
   ```

5. **Verify Links:**
   ```bash
   # Run CI link check locally
   markdown-link-check docs/NEW-NAME.md
   ```

6. **Test Cross-References:**
   ```bash
   # Verify all updated links resolve
   find . -name "*.md" -exec markdown-link-check {} \;
   ```

### Post-Migration Validation

- [ ] All files renamed successfully (`git status` clean)
- [ ] No broken links (CI passes)
- [ ] Index updated (`00-DOCUMENTATION-INDEX.md` accurate)
- [ ] CHANGELOG updated (document renames)
- [ ] Cross-references verified (manual spot checks)

### Rollback Plan

If migration fails:
```bash
# Revert to pre-migration state
git reset --hard HEAD~1

# Or revert specific commit
git revert <commit-hash>
```

---

## Future Number Assignments

### Decision Tree

When creating new numbered documentation:

```
START: New documentation needed
  │
  ├─ Is it temporary/phase-specific?
  │  └─ YES → Do NOT number (use PHASE-N or SPRINT-N.M prefix)
  │
  ├─ Is it standalone architecture?
  │  └─ YES → Do NOT number (use COMPONENT-ARCHITECTURE.md)
  │
  ├─ Is it process/standards?
  │  └─ YES → Do NOT number (use TOPIC-STANDARDS.md)
  │
  └─ Is it core reference/guide?
     └─ YES → Assign number from appropriate range:
        │
        ├─ Meta/Navigation? → 00-09 (check if full)
        ├─ Development? → 10-19 (slots: 15, 16, 18)
        ├─ Feature Guide? → 20-29 (slots: 28-29 reserved)
        ├─ User Doc? → 30-39 (slots: 35-39)
        ├─ Operations? → 40-49 (slots: 40-49 all available)
        └─ Reference? → 50-59 (slots: 50-59 all available)
```

### Assignment Process

1. **Determine Category** (see decision tree)
2. **Find Lowest Available Number** in range
3. **Check Dependencies:**
   - Does this doc require reading others first?
   - Should it come before/after specific docs?
4. **Assign Number:**
   - Format: `NN-TOPIC-TYPE.md`
   - Example: `40-PRODUCTION-DEPLOYMENT-GUIDE.md`
5. **Update Index:**
   - Add entry to `00-DOCUMENTATION-INDEX.md`
   - Note category and number
6. **Document in CHANGELOG:**
   ```markdown
   ### Added
   - `docs/40-PRODUCTION-DEPLOYMENT-GUIDE.md`: Operations guide for production deployment (range 40-49)
   ```

### Example: Adding New Feature Guide

**Scenario:** New feature "Network Topology Mapping" needs guide

**Process:**
1. Category: Feature Guide → Range 20-29
2. Available slots: 28-29 (reserved for features)
3. Dependencies: Builds on service detection (24)
4. Assign: `28-NETWORK-TOPOLOGY-GUIDE.md`
5. Update: Add to index under "Feature Guides (20-29)"
6. Verify: Run link check, update CHANGELOG

---

## References

### Related Documents

- `docs/DOCUMENTATION-NAMING-STANDARDS.md` - Filename conventions
- `docs/00-DOCUMENTATION-INDEX.md` - Complete file listing
- `docs/DOCUMENTATION-REVIEW-SCHEDULE.md` - Quarterly review process
- `templates/TEMPLATE-*.md` - Document templates

### Tools & Scripts

- `.github/workflows/markdown-links.yml` - Automated link validation
- `scripts/archive-phase.sh` - Phase archival automation
- `scripts/update-doc-links.sh` - (To be created) Cross-reference updater
- `scripts/validate-doc-names.sh` - (To be created) Naming compliance checker

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-15 | Initial numbering system specification |

---

## Summary

### Current State (Before Renumbering)

- **00-09:** ✅ Complete (10/10), no conflicts
- **10-19:** ⚠️ Needs cleanup (3 archives, 2 underscores)
- **20-29:** ⚠️ Needs fixes (2 decimals, 1 duplicate)
- **30-39:** ❌ Critical conflict (34 used 3x)
- **40-99:** 🚧 Empty (reserved for future)

### Proposed State (After Phase 1 Migration)

- **00-09:** ✅ Complete (10/10)
- **10-19:** ✅ Clean (7/10), 3 slots free (15, 16, 18)
- **20-29:** ✅ Clean (8/10), 2 slots free (28-29)
- **30-39:** ✅ Clean (3/10), 7 slots free (35-39)
- **40-49:** 🆕 Operations (1/10), 9 slots free
- **50-99:** 🚧 Reserved (50/50)

### Quick Reference Card

```
┌─────────────────────────────────────────────────┐
│  ProRT-IP Documentation Numbering Guide        │
├─────────────────────────────────────────────────┤
│  00-09: Meta & Navigation     [FULL]           │
│  10-19: Development           [3 slots free]   │
│  20-29: Feature Guides        [2 slots free]   │
│  30-39: User Documentation    [7 slots free]   │
│  40-49: Operations            [9 slots free]   │
│  50-59: Reference             [10 slots free]  │
│  60-69: Advanced Topics       [RESERVED]       │
│  70-99: Future Expansion      [RESERVED]       │
└─────────────────────────────────────────────────┘

Rules:
  ✓ Unique numbers (no duplicates)
  ✓ Zero-padded (00-99, not 0-9)
  ✓ No decimals (22, 23 not 22.1)
  ✓ Sequential within range
  ✓ Leave 10-20% buffer for expansion
```

---

**Maintained by:** ProRT-IP Documentation Team  
**Questions?** Open an issue with `documentation` label  
**Next Review:** Quarterly (see `DOCUMENTATION-REVIEW-SCHEDULE.md`)
