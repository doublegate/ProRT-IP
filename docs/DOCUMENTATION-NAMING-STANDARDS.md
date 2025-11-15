# ProRT-IP Documentation Naming Standards

**Version:** 1.0.0  
**Date:** 2025-11-15  
**Status:** APPROVED (Proposed Implementation)  
**Purpose:** Establish consistent, discoverable naming conventions for all ProRT-IP documentation

---

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [Naming Conventions](#naming-conventions)
4. [File Type Standards](#file-type-standards)
5. [Current State Analysis](#current-state-analysis)
6. [Proposed Renames](#proposed-renames)
7. [Migration Strategy](#migration-strategy)
8. [Future Guidelines](#future-guidelines)
9. [References](#references)

---

## Overview

### Purpose

This document establishes **mandatory** naming standards for all documentation files in the ProRT-IP project. Consistent naming improves:

- **Discoverability**: Users can find documents quickly
- **Maintainability**: Clear purpose from filename alone
- **Automation**: Scripts can rely on predictable patterns
- **Professionalism**: Consistent conventions signal quality

### Scope

These standards apply to ALL markdown files in:
- `/docs/` - Primary documentation
- `/docs/archive/` - Historical/archived content
- `/to-dos/` - Planning and task tracking
- `/templates/` - Documentation templates
- Root level (`README.md`, `CHANGELOG.md`, etc.)

### Authority

This document supersedes all previous informal naming conventions. All new documentation MUST follow these standards. Existing documentation SHOULD be migrated during quarterly reviews (see `DOCUMENTATION-REVIEW-SCHEDULE.md`).

---

## Core Principles

### 1. Descriptive Over Cryptic

**Good:** `IPv6-GUIDE.md`, `TUI-ARCHITECTURE.md`, `PHASE-5-README-ARCHIVE.md`  
**Bad:** `ipv6.md`, `tui.md`, `p5.md`

### 2. Consistent Separators

Use **hyphens** (`-`) for multi-word names, never underscores or spaces:

**Good:** `SERVICE-DETECTION-GUIDE.md`, `RATE-LIMITING-GUIDE.md`  
**Bad:** `service_detection_guide.md`, `Service Detection Guide.md`

**Rationale:** Hyphens are URL-safe, improve readability, and work across all filesystems.

### 3. Uppercase for Permanence

Use **UPPERCASE** for files that are:
- Reference documentation (guides, specifications)
- Infrastructure files (README, CHANGELOG, CONTRIBUTING)
- Templates and standards

Use **lowercase** for:
- Temporary/working files
- Scripts (e.g., `archive-phase.sh`)
- Configuration files

### 4. Suffix Consistency

Use standard suffixes to indicate document type:
- `-GUIDE.md` - User-facing tutorials and how-tos
- `-ARCHITECTURE.md` - System design and structure
- `-REFERENCE.md` - API and technical specifications
- `-TODO.md` - Planning and task tracking
- `-COMPLETE.md` - Completion reports
- `-ARCHIVE.md` - Historical snapshots

### 5. Numbering for Ordering

Use **two-digit** zero-padded numbers (`00-`, `01-`, `23-`) for:
- Core documentation that should be read in sequence
- Version-controlled guides (prevents reordering on major additions)

**Format:** `NN-DOCUMENT-NAME.md` where `NN` is `00-99`

---

## Naming Conventions

### Standard Format

```
[NUMBER-]TOPIC[-TYPE][-QUALIFIER].md

Examples:
00-ARCHITECTURE.md          # Numbered core doc
23-IPv6-GUIDE.md           # Numbered guide
TUI-ARCHITECTURE.md        # Unnumbered architecture
PHASE-5-README-ARCHIVE.md  # Archive with qualifier
SPRINT-6.1-TODO.md         # Task tracking with version
```

### Component Breakdown

1. **Number** (Optional): `00-99` for ordered documents
2. **Topic**: Primary subject (e.g., `IPv6`, `SERVICE-DETECTION`, `TUI`)
3. **Type**: Document category (`GUIDE`, `ARCHITECTURE`, `REFERENCE`, `TODO`)
4. **Qualifier** (Optional): Version or scope (e.g., `PHASE-5`, `v1.0.0`)

### Topic Naming Rules

- Use **UPPERCASE** for acronyms: `IPv6`, `TLS`, `API`, `CLI`, `TUI`
- Use **Title-Case-With-Hyphens** for multi-word topics: `Service-Detection`, `Rate-Limiting`
- Use **semantic names** over generic: `IPv6-GUIDE` not `NETWORK-GUIDE`
- Keep **concise**: Prefer `TLS-CERTIFICATE` over `TLS-X509-CERTIFICATE-PARSING`

### Type Suffix Standards

| Suffix | Purpose | Example | Target Audience |
|--------|---------|---------|-----------------|
| `-GUIDE` | User-facing tutorials, how-tos | `23-IPv6-GUIDE.md` | End users, operators |
| `-ARCHITECTURE` | System design, structure | `TUI-ARCHITECTURE.md` | Developers, architects |
| `-REFERENCE` | API docs, specifications | `05-API-REFERENCE.md` | Developers |
| `-TODO` | Planning, tasks | `SPRINT-6.1-TODO.md` | Project team |
| `-COMPLETE` | Sprint/task reports | `SPRINT-5.1-COMPLETE.md` | Project team, archives |
| `-ARCHIVE` | Historical snapshots | `PHASE-4-README-ARCHIVE.md` | Reference, history |
| `-INDEX` | Directory/navigation | `00-DOCUMENTATION-INDEX.md` | All users |
| `-STANDARDS` | Process documentation | `DOCUMENTATION-NAMING-STANDARDS.md` | Maintainers |

### Qualifier Standards

**Version Qualifiers:**
- Use semantic versioning: `v1.0.0`, `v2.1.0`
- Place at end: `USER-GUIDE-v2.0.0.md`

**Phase/Sprint Qualifiers:**
- Format: `PHASE-N`, `SPRINT-N.M`
- Examples: `PHASE-5-README-ARCHIVE.md`, `SPRINT-6.1-TODO.md`

**Scope Qualifiers:**
- Use when multiple docs cover same topic: `TESTING-UNIT.md`, `TESTING-INTEGRATION.md`
- Or: `API-REFERENCE-CORE.md`, `API-REFERENCE-PLUGINS.md`

---

## File Type Standards

### 1. Core Documentation (Numbered 00-99)

**Purpose:** Primary reference material, read in logical order  
**Location:** `/docs/`  
**Format:** `NN-TOPIC[-TYPE].md`

**Reserved Ranges:**
- `00-09`: Meta/navigation (INDEX, ARCHITECTURE, ROADMAP)
- `10-19`: Development (DEV-SETUP, IMPLEMENTATION-GUIDE, TESTING)
- `20-29`: Feature guides (IPv6, Service Detection, TLS, etc.)
- `30-39`: User documentation (USER-GUIDE, TUTORIALS, EXAMPLES)
- `40-49`: Operations (Performance, Security, Troubleshooting)
- `50-59`: Reference (API, CLI, Configuration)
- `60-99`: Reserved for future expansion

**Examples:**
```
00-DOCUMENTATION-INDEX.md    # Navigation hub
00-ARCHITECTURE.md           # System overview
01-ROADMAP.md                # Project timeline
03-DEV-SETUP.md              # Developer onboarding
23-IPv6-GUIDE.md             # Feature guide
32-USER-GUIDE.md             # User manual
```

### 2. Feature Guides (User-Facing)

**Purpose:** How to use specific features  
**Location:** `/docs/` (numbered 20-39)  
**Format:** `NN-FEATURE-GUIDE.md`  
**Suffix:** MUST use `-GUIDE`

**Examples:**
```
23-IPv6-GUIDE.md
24-SERVICE-DETECTION-GUIDE.md
25-IDLE-SCAN-GUIDE.md
26-RATE-LIMITING-GUIDE.md
27-TLS-CERTIFICATE-GUIDE.md
```

**Standard Sections:**
1. Overview
2. Quick Start
3. Configuration
4. Examples
5. Troubleshooting
6. See Also

### 3. Architecture Documentation

**Purpose:** System design, technical structure  
**Location:** `/docs/`  
**Format:** `COMPONENT-ARCHITECTURE.md` (unnumbered unless core)  
**Suffix:** MUST use `-ARCHITECTURE`

**Examples:**
```
00-ARCHITECTURE.md           # Overall system (numbered, core)
TUI-ARCHITECTURE.md          # Component-specific (unnumbered)
PLUGIN-ARCHITECTURE.md       # Subsystem design (unnumbered)
```

**Standard Sections:**
1. Overview
2. Components
3. Data Flow
4. Design Decisions
5. Alternatives Considered
6. Future Work

### 4. Task Tracking (TODO/COMPLETE)

**Purpose:** Sprint planning and completion reports  
**Location:** `/to-dos/PHASE-N/`  
**Format:** `SPRINT-N.M-DESCRIPTOR-TODO.md` or `SPRINT-N.M-COMPLETE.md`

**Examples:**
```
SPRINT-6.1-TUI-FRAMEWORK-TODO.md
SPRINT-6.1-COMPLETE.md
SPRINT-6.3-NETWORK-OPTIMIZATION-TODO.md
PHASE-6-PLANNING.md
```

**Standard Sections (TODO):**
1. Overview
2. Success Criteria
3. Tasks (checkbox list)
4. Dependencies
5. Timeline
6. Risks

**Standard Sections (COMPLETE):**
1. Executive Summary
2. Deliverables
3. Metrics
4. Challenges & Solutions
5. Lessons Learned
6. Next Steps

### 5. Archives

**Purpose:** Historical snapshots of completed phases  
**Location:** `/docs/archive/`  
**Format:** `PHASE-N-README-ARCHIVE.md`

**Examples:**
```
PHASE-4-README-ARCHIVE.md
PHASE-5-README-ARCHIVE.md
PHASE-6-README-ARCHIVE.md
```

**Naming Rules:**
- MUST include phase number
- MUST use `-ARCHIVE` suffix
- MUST indicate source (e.g., `README-ARCHIVE` vs `STATUS-ARCHIVE`)
- Use date in content header, not filename

### 6. Templates

**Purpose:** Reusable document structures  
**Location:** `/templates/`  
**Format:** `TEMPLATE-PURPOSE.md`  
**Prefix:** MUST use `TEMPLATE-`

**Examples:**
```
TEMPLATE-PHASE-SECTION.md
TEMPLATE-SPRINT-SECTION.md
TEMPLATE-FEATURE-SECTION.md
TEMPLATE-ARCHIVE.md
```

### 7. Standards & Process Documents

**Purpose:** Define conventions and workflows  
**Location:** `/docs/`  
**Format:** `TOPIC-STANDARDS.md` or `TOPIC-PROCESS.md`  
**Suffix:** `-STANDARDS`, `-PROCESS`, `-SCHEDULE`

**Examples:**
```
DOCUMENTATION-NAMING-STANDARDS.md
DOCUMENTATION-NUMBERING-SYSTEM.md
DOCUMENTATION-REVIEW-SCHEDULE.md
RELEASE-PROCESS.md
```

### 8. Root-Level Files

**Purpose:** Critical project files  
**Location:** `/` (repository root)  
**Format:** `UPPERCASE.md`  
**Names:** Standardized by convention

**Required Files:**
```
README.md              # Project overview
CHANGELOG.md           # Version history
CONTRIBUTING.md        # Contribution guide
SECURITY.md            # Security policy
LICENSE                # Legal terms
AUTHORS.md             # Contributors
SUPPORT.md             # Help resources
```

**Style:**
- Always UPPERCASE
- No prefixes or suffixes
- Industry-standard names (don't invent new names)

---

## Current State Analysis

### Compliant Files (No Changes Needed)

**Correctly Formatted:**
```
00-ARCHITECTURE.md                    ✅ Numbered core doc
01-ROADMAP.md                         ✅ Numbered core doc
23-IPv6-GUIDE.md                      ✅ Numbered guide with suffix
25-IDLE-SCAN-GUIDE.md                 ✅ Numbered guide with suffix
26-RATE-LIMITING-GUIDE.md             ✅ Numbered guide with suffix
27-TLS-CERTIFICATE-GUIDE.md           ✅ Numbered guide with suffix
TUI-ARCHITECTURE.md                   ✅ Unnumbered architecture
PHASE-4-README-ARCHIVE.md             ✅ Archive with qualifier
PHASE-5-README-ARCHIVE.md             ✅ Archive with qualifier
```

### Non-Compliant Files (Require Attention)

**Missing -GUIDE Suffix:**
```
24-SERVICE-DETECTION.md               ❌ Should be: 24-SERVICE-DETECTION-GUIDE.md
```

**Inconsistent Naming:**
```
34-EXAMPLES.md                        ❌ Should be: 34-EXAMPLES-REFERENCE.md
34-EXAMPLES-GALLERY.md                ⚠️  Duplicate purpose with 34-EXAMPLES.md
34-PERFORMANCE-CHARACTERISTICS.md     ⚠️  Number 34 conflicts with EXAMPLES
```

**Underscore Instead of Hyphen:**
```
18-EFFICIENCY_REPORT.md               ❌ Should be: 18-EFFICIENCY-REPORT.md
14-NMAP_COMPATIBILITY.md              ❌ Should be: 14-NMAP-COMPATIBILITY.md (or remove number)
```

**Missing Type Suffix:**
```
02-TECHNICAL-SPECS.md                 ⚠️  Acceptable (SPECS is implicit type)
09-FAQ.md                             ✅ FAQ is self-descriptive
13-PLATFORM-SUPPORT.md                ⚠️  Could add -REFERENCE
```

**Unclear Purpose:**
```
22.1-CLAUDE-POST-PHASE4_1of2.md       ❌ Non-standard numbering, underscore
22.2-CLAUDE-POST-PHASE4_2of2.md       ❌ Should be archived or renamed
```

**Archive Candidates:**
```
15-PHASE4-COMPLIANCE.md               ⚠️  Should move to archive/
16-REGRESSION-FIX-STRATEGY.md         ⚠️  Should move to archive/ (if historical)
20-PHASE4-ENHANCEMENTS.md             ⚠️  Already in archive/, remove from docs/
```

---

## Proposed Renames

### High Priority (Consistency Issues)

| Current Name | Proposed Name | Reason |
|--------------|---------------|--------|
| `24-SERVICE-DETECTION.md` | `24-SERVICE-DETECTION-GUIDE.md` | Missing -GUIDE suffix (feature guide) |
| `18-EFFICIENCY_REPORT.md` | `18-EFFICIENCY-REPORT.md` | Underscore → hyphen |
| `14-NMAP_COMPATIBILITY.md` | `NMAP-COMPATIBILITY-REFERENCE.md` | Underscore → hyphen, consider unnumbered |
| `22.1-CLAUDE-POST-PHASE4_1of2.md` | `archive/PHASE-4-CLAUDE-NOTES-PART-1.md` | Non-standard format, archive |
| `22.2-CLAUDE-POST-PHASE4_2of2.md` | `archive/PHASE-4-CLAUDE-NOTES-PART-2.md` | Non-standard format, archive |

### Medium Priority (Clarity Improvements)

| Current Name | Proposed Name | Reason |
|--------------|---------------|--------|
| `34-EXAMPLES-GALLERY.md` | `34-EXAMPLES-GALLERY.md` (keep) | Consolidate with 34-EXAMPLES.md |
| `34-EXAMPLES.md` | *(merge into GALLERY or rename to INDEX)* | Duplicate purpose |
| `34-PERFORMANCE-CHARACTERISTICS.md` | `07-PERFORMANCE-CHARACTERISTICS.md` | Move to performance range (40-49) or renumber |

### Low Priority (Optional Improvements)

| Current Name | Proposed Name | Reason |
|--------------|---------------|--------|
| `13-PLATFORM-SUPPORT.md` | `13-PLATFORM-SUPPORT-REFERENCE.md` | Add type suffix for clarity |
| `02-TECHNICAL-SPECS.md` | `02-TECHNICAL-SPECIFICATIONS.md` | Expand abbreviation |

### Archive Migrations

**Move to `/docs/archive/`:**
```
docs/15-PHASE4-COMPLIANCE.md          → archive/PHASE-4-COMPLIANCE.md
docs/16-REGRESSION-FIX-STRATEGY.md    → archive/PHASE-4-REGRESSION-FIX.md
docs/20-PHASE4-ENHANCEMENTS.md        → (already exists in archive, delete duplicate)
```

---

## Migration Strategy

### Phase 1: Immediate (High Priority)

**Timeframe:** Next quarterly review (within 3 months)  
**Scope:** Fix critical inconsistencies (underscores, missing suffixes)

**Actions:**
1. Rename `24-SERVICE-DETECTION.md` → `24-SERVICE-DETECTION-GUIDE.md`
2. Rename `18-EFFICIENCY_REPORT.md` → `18-EFFICIENCY-REPORT.md`
3. Rename `14-NMAP_COMPATIBILITY.md` → `14-NMAP-COMPATIBILITY.md`
4. Archive `22.1` and `22.2` CLAUDE files
5. Update all cross-references in other docs
6. Run automated link check (new CI workflow)

**Validation:**
- ✅ All links still resolve
- ✅ No 404s in documentation
- ✅ CI passes

### Phase 2: Medium Priority (Clarity)

**Timeframe:** Within 6 months  
**Scope:** Resolve numbering conflicts, consolidate duplicates

**Actions:**
1. Consolidate `34-EXAMPLES.md` and `34-EXAMPLES-GALLERY.md`
2. Renumber `34-PERFORMANCE-CHARACTERISTICS.md` to performance range
3. Archive Phase 4 compliance docs
4. Review all files for missing type suffixes

### Phase 3: Long-Term (Optional)

**Timeframe:** As needed during major versions  
**Scope:** Comprehensive renumbering, perfect consistency

**Actions:**
1. Review entire numbering system (gaps, logical grouping)
2. Consider comprehensive renumbering per `DOCUMENTATION-NUMBERING-SYSTEM.md`
3. Add type suffixes to all ambiguous files
4. Standardize all version qualifiers

### Automation

**Link Updates:**
```bash
# Use scripts/update-doc-links.sh (to be created)
./scripts/update-doc-links.sh OLD_NAME.md NEW_NAME.md
```

**Validation:**
```bash
# Run after any rename
.github/workflows/markdown-links.yml  # Automated in CI
```

---

## Future Guidelines

### Creating New Documentation

**Checklist:**
1. ✅ Choose appropriate number range (00-09, 10-19, 20-29, etc.)
2. ✅ Use hyphens for multi-word names (never underscores)
3. ✅ Add type suffix (`-GUIDE`, `-ARCHITECTURE`, `-REFERENCE`)
4. ✅ Use UPPERCASE for permanent docs, lowercase for scripts
5. ✅ Follow section structure from templates
6. ✅ Add entry to `00-DOCUMENTATION-INDEX.md`
7. ✅ Verify links with CI workflow

**Before Committing:**
```bash
# Verify naming compliance
./scripts/validate-doc-names.sh NEW_FILE.md  # To be created

# Check links
npm install -g markdown-link-check
markdown-link-check NEW_FILE.md
```

### Renaming Existing Documentation

**Process:**
1. Create GitHub issue: "Rename [OLD] to [NEW]"
2. Update all cross-references (use grep/sed)
3. Update `00-DOCUMENTATION-INDEX.md`
4. Update `CHANGELOG.md` under `## [Unreleased] → Changed`
5. Create PR with descriptive commit message
6. Verify CI passes (link check)

**Commit Message Format:**
```
docs: Rename 24-SERVICE-DETECTION to 24-SERVICE-DETECTION-GUIDE

- Add missing -GUIDE suffix for consistency
- Update 15 cross-references in other docs
- Verified all links resolve correctly

Follows DOCUMENTATION-NAMING-STANDARDS.md guidelines.
```

### Quarterly Review

During quarterly documentation reviews (see `DOCUMENTATION-REVIEW-SCHEDULE.md`):

1. **Audit Compliance:**
   - List all files not following standards
   - Prioritize by impact (broken links > missing suffixes > style)

2. **Plan Renames:**
   - Group related renames (batch updates more efficient)
   - Create tracking issues
   - Assign to maintainers

3. **Execute:**
   - Rename files (preserve git history with `git mv`)
   - Update cross-references
   - Verify links
   - Merge with review

4. **Document:**
   - Update this standards doc if patterns emerge
   - Add lessons learned to review notes

---

## References

### Related Documents

- `docs/00-DOCUMENTATION-INDEX.md` - Complete file listing
- `docs/DOCUMENTATION-NUMBERING-SYSTEM.md` - Number allocation strategy
- `docs/DOCUMENTATION-REVIEW-SCHEDULE.md` - Quarterly review process
- `templates/TEMPLATE-*.md` - Document structure templates

### External Standards

- [Google Developer Documentation Style Guide](https://developers.google.com/style)
- [Microsoft Writing Style Guide](https://learn.microsoft.com/en-us/style-guide/)
- [Semantic Versioning](https://semver.org/) - Version qualifier format
- [Conventional Commits](https://www.conventionalcommits.org/) - Commit message format

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-15 | Initial standards document created |

---

## Summary

### Key Takeaways

1. **Use hyphens**, never underscores or spaces
2. **Add type suffixes** (`-GUIDE`, `-ARCHITECTURE`, `-REFERENCE`)
3. **Number core docs** with `NN-` prefix (00-99)
4. **UPPERCASE permanent** docs, lowercase scripts
5. **Follow templates** for section structure
6. **Update cross-references** when renaming
7. **Verify links** with CI automation

### Quick Reference

**Format:** `[NN-]TOPIC[-TYPE][-QUALIFIER].md`

**Examples:**
- `23-IPv6-GUIDE.md` - Numbered feature guide
- `TUI-ARCHITECTURE.md` - Unnumbered architecture doc
- `SPRINT-6.1-TODO.md` - Task tracking with version
- `PHASE-5-README-ARCHIVE.md` - Historical archive

### Compliance Checklist

Before merging new documentation:
- [ ] Filename uses hyphens (not underscores)
- [ ] Type suffix added (`-GUIDE`, `-ARCHITECTURE`, etc.)
- [ ] Number assigned if core/ordered doc
- [ ] Added to `00-DOCUMENTATION-INDEX.md`
- [ ] All links verified (CI passes)
- [ ] Follows template structure
- [ ] CHANGELOG.md updated

---

**Maintained by:** ProRT-IP Documentation Team  
**Questions?** Open an issue with `documentation` label  
**Updates:** Reviewed quarterly (see `DOCUMENTATION-REVIEW-SCHEDULE.md`)
