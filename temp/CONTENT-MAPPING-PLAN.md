# ProRT-IP mdBook Content Mapping Plan

**Date**: 2025-11-15
**Purpose**: Comprehensive strategy for integrating all ProRT-IP documentation into mdBook

---

## Current State Analysis

### File Inventory

**Total Files**: 106 markdown files (61,435+ lines)

**Distribution**:
- `docs/`: 63 files (core documentation, guides, archive)
- `ref-docs/`: 11 files (technical references, comparisons)
- `to-dos/`: 32 files (planning documents, sprint TODO files)

### Current book.toml Configuration

**Location**: `/home/parobek/Code/ProRT-IP/book.toml` (root directory)
**Configuration**:
```toml
src = "docs"           # Points to docs/ directory
build-dir = "book"     # Builds to /book/ directory
```

**Issue**: OctoLLM pattern has book.toml in docs/ directory, not root
- Source should be `docs/src/` not `docs/`
- Build should be `docs/book/` not `/book/`

---

## Required Transformations

### 1. Directory Restructuring

**Current Structure**:
```
ProRT-IP/
├── book.toml          # In root (needs to move)
├── docs/              # Contains markdown files directly
│   ├── 00-ARCHITECTURE.md
│   ├── 01-ROADMAP.md
│   ├── archive/
│   └── ...
├── ref-docs/
└── to-dos/
```

**Target Structure** (OctoLLM pattern):
```
ProRT-IP/
├── docs/
│   ├── book.toml      # Moved here
│   ├── src/           # New: mdBook source directory
│   │   ├── SUMMARY.md
│   │   ├── introduction.md
│   │   ├── getting-started/
│   │   ├── user-guide/
│   │   ├── features/
│   │   ├── advanced/
│   │   ├── reference/
│   │   ├── development/
│   │   └── appendices/
│   └── book/          # New: mdBook build output (.gitignore)
├── ref-docs/          # Keep for now (may symlink or copy)
└── to-dos/            # Keep for now (may symlink or copy)
```

### 2. Configuration Updates

**book.toml changes** (after moving to docs/):
```toml
[book]
title = "ProRT-IP WarScan Documentation"
# ... existing config ...
src = "src"           # Changed from "docs" (now relative to docs/)

[build]
build-dir = "book"    # Changed path (now relative to docs/)
# ... rest unchanged
```

---

## Content Categorization & Mapping

### Category 1: Core Documentation (docs/)

**Files**: 63 total (36 primary + 27 archive)

#### 1.1 Foundational Documents (7 files)
→ **Introduction & Overview Section**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| README.md | src/introduction.md | Landing page, project overview |
| 00-ARCHITECTURE.md | src/architecture/overview.md | System architecture |
| 00-DOCUMENTATION-INDEX.md | src/reference/index.md | Documentation navigation |
| 01-ROADMAP.md | src/project/roadmap.md | Development roadmap |
| 10-PROJECT-STATUS.md | src/project/status.md | Current status |
| 09-FAQ.md | src/reference/faq.md | Frequently asked questions |
| TROUBLESHOOTING.md | src/reference/troubleshooting.md | Common issues |

#### 1.2 Getting Started Guides (4 files)
→ **Getting Started Section**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| 03-DEV-SETUP.md | src/getting-started/installation.md | Installation guide |
| 32-USER-GUIDE.md | src/getting-started/user-guide.md | Basic usage |
| 33-TUTORIALS.md | src/getting-started/tutorials.md | Step-by-step tutorials |
| 34-EXAMPLES-GALLERY.md | src/getting-started/examples.md | Example scans |

#### 1.3 Feature Guides (11 files)
→ **Feature Guides Section**

| Current File | Target Location | Feature |
|--------------|-----------------|---------|
| 23-IPv6-GUIDE.md | src/features/ipv6-scanning.md | IPv6 support |
| 24-SERVICE-DETECTION-GUIDE.md | src/features/service-detection.md | Service detection |
| 25-IDLE-SCAN-GUIDE.md | src/features/idle-scan.md | Idle scan technique |
| 26-RATE-LIMITING-GUIDE.md | src/features/rate-limiting.md | Rate limiting |
| 27-TLS-CERTIFICATE-GUIDE.md | src/features/tls-certificates.md | TLS analysis |
| 30-PLUGIN-SYSTEM-GUIDE.md | src/features/plugin-system.md | Plugin architecture |
| 35-EVENT-SYSTEM-GUIDE.md | src/features/event-system.md | Event-driven architecture |
| 19-EVASION-GUIDE.md | src/features/evasion-techniques.md | Firewall evasion |
| 14-NMAP-COMPATIBILITY.md | src/features/nmap-compatibility.md | Nmap compatibility |
| 13-PLATFORM-SUPPORT.md | src/features/platform-support.md | Cross-platform support |
| DATABASE.md | src/features/database-storage.md | Database integration |

#### 1.4 Advanced Topics (8 files)
→ **Advanced Topics Section**

| Current File | Target Location | Topic |
|--------------|-----------------|-------|
| 07-PERFORMANCE.md | src/advanced/performance-tuning.md | Performance optimization |
| 21-PERFORMANCE-GUIDE.md | src/advanced/performance-analysis.md | Performance analysis |
| 34-PERFORMANCE-CHARACTERISTICS.md | src/advanced/performance-characteristics.md | Benchmarking results |
| 31-BENCHMARKING-GUIDE.md | src/advanced/benchmarking.md | Benchmarking framework |
| 12-BENCHMARKING-GUIDE.md | src/advanced/benchmarking-legacy.md | Legacy benchmarking |
| 08-SECURITY.md | src/advanced/security-best-practices.md | Security considerations |
| 18-EFFICIENCY-REPORT.md | src/advanced/efficiency-analysis.md | Efficiency metrics |
| TUI-ARCHITECTURE.md | src/advanced/tui-architecture.md | TUI framework design |

#### 1.5 Development Documentation (9 files)
→ **Development Section**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| 04-IMPLEMENTATION-GUIDE.md | src/development/implementation.md | Implementation guide |
| 05-API-REFERENCE.md | src/development/api-reference.md | API documentation |
| 06-TESTING.md | src/development/testing.md | Testing strategy |
| 17-TESTING-INFRASTRUCTURE.md | src/development/testing-infrastructure.md | Testing framework |
| 28-CI-CD-COVERAGE.md | src/development/ci-cd.md | CI/CD pipeline |
| 29-FUZZING-GUIDE.md | src/development/fuzzing.md | Fuzz testing |
| 11-RELEASE-PROCESS.md | src/development/release-process.md | Release workflow |
| 02-TECHNICAL-SPECS.md | src/development/technical-specs.md | Technical specifications |
| DOCUMENTATION-NAMING-STANDARDS.md | src/development/doc-standards.md | Documentation standards |

#### 1.6 Legacy/Compliance Documents (6 files)
→ **Reference or Appendix**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| 15-PHASE4-COMPLIANCE.md | src/appendices/phase4-compliance.md | Phase 4 compliance |
| 16-REGRESSION-FIX-STRATEGY.md | src/appendices/regression-strategy.md | Regression handling |
| 20-PHASE4-ENHANCEMENTS.md | src/appendices/phase4-enhancements.md | Phase 4 enhancements |
| PHASE-5-BACKLOG.md | src/appendices/phase5-backlog.md | Phase 5 backlog |
| 34-EXAMPLES.md | src/appendices/examples-legacy.md | Legacy examples |
| DOCUMENTATION-NUMBERING-SYSTEM.md | src/appendices/numbering-system.md | Numbering system |

#### 1.7 Archive Documents (27 files)
→ **Appendices: Historical Archive**

| Category | Files | Target Location |
|----------|-------|-----------------|
| Phase Archives | PHASE-4/5/6-README-ARCHIVE.md | src/appendices/archives/phase-{4,5,6}.md |
| Sprint Reports | SPRINT-4.22-*.md | src/appendices/archives/sprint-reports/ |
| Historical Docs | PHASE-4-CLAUDE-NOTES-*.md | src/appendices/archives/historical/ |
| Organization | File-Reorganization-*.md | src/appendices/archives/organization/ |
| Index | 00-INDEX.md, README.md | src/appendices/archives/index.md |

### Category 2: Reference Documentation (ref-docs/)

**Files**: 11 files

#### 2.1 Technical Specifications (2 files)
→ **Reference Section**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| ProRT-IP_WarScan_Technical_Specification.md | src/reference/tech-spec-v1.md | Technical spec v1 |
| ProRT-IP_WarScan_Technical_Specification-v2.md | src/reference/tech-spec-v2.md | Technical spec v2 |
| ProRT-IP_Overview.md | src/reference/overview.md | Project overview |

#### 2.2 Comparisons (5 files)
→ **Reference: Comparisons**

| Current File | Target Location | Tool |
|--------------|-----------------|------|
| Nmap-Ref_Info.md | src/reference/comparisons/nmap.md | Nmap comparison |
| Masscan-Ref_Info.md | src/reference/comparisons/masscan.md | Masscan comparison |
| ZMap-Ref_Info.md | src/reference/comparisons/zmap.md | ZMap comparison |
| RustScan-Ref_Info.md | src/reference/comparisons/rustscan.md | RustScan comparison |
| Naabu-Ref_Info.md | src/reference/comparisons/naabu.md | Naabu comparison |
| Net_Tools-Comparison.md | src/reference/comparisons/overview.md | Comparison matrix |

#### 2.3 Custom Commands (2 files)
→ **Reference: Commands**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| 10-Custom_Commands.md | src/reference/commands/overview.md | Commands overview |
| 10-Custom-Commands_Analysis.md | src/reference/commands/analysis.md | Commands analysis |

### Category 3: Planning Documents (to-dos/)

**Files**: 32 files

#### 3.1 Phase Planning (20 files)
→ **Appendices: Planning** (optional, may exclude from public docs)

| Subcategory | Files | Include? |
|-------------|-------|----------|
| Phase 4 | v0.4.0-*.md (4 files) | Maybe (historical context) |
| Phase 5 | SPRINT-5.*.md (15 files) | Maybe (development process) |
| Phase 6 | SPRINT-6.*.md (8 files) | Maybe (active planning) |

**Decision**: Include summaries in Appendix D: Development Planning
- Phase overviews (not individual TODO files)
- Sprint completion reports (not TODO files)
- Active roadmap items (link to to-dos/ directory)

#### 3.2 Analysis Documents (3 files)
→ **Reference: Analysis**

| Current File | Target Location | Purpose |
|--------------|-----------------|---------|
| REFERENCE-ANALYSIS-IMPROVEMENTS.md | src/reference/analysis/improvements.md | Improvement roadmap |
| REFERENCE-ANALYSIS-REPORT.md | src/reference/analysis/competitive-analysis.md | Competitive analysis |
| PHASE-6-PLANNING-REPORT.md | src/appendices/planning/phase6-plan.md | Phase 6 planning |

---

## Proposed mdBook Structure

### 11 Major Sections, ~180-200 Chapters

```markdown
# Summary

[Introduction](./introduction.md)

---

# Getting Started

- [Installation & Setup](./getting-started/installation.md)
  - [Prerequisites](./getting-started/installation.md#prerequisites)
  - [Building from Source](./getting-started/installation.md#building-from-source)
  - [Binary Installation](./getting-started/installation.md#binary-installation)
- [Quick Start Guide](./getting-started/quick-start.md)
- [First Scan Tutorial](./getting-started/tutorials.md)
- [Example Scans](./getting-started/examples.md)

# User Guide

- [Basic Usage](./user-guide/basic-usage.md)
- [Scan Types](./user-guide/scan-types.md)
  - [TCP SYN Scan](./user-guide/scan-types.md#tcp-syn-scan)
  - [TCP Connect Scan](./user-guide/scan-types.md#tcp-connect-scan)
  - [UDP Scan](./user-guide/scan-types.md#udp-scan)
  - [Stealth Scans](./user-guide/scan-types.md#stealth-scans)
- [CLI Reference](./user-guide/cli-reference.md)
- [Output Formats](./user-guide/output-formats.md)
- [Configuration](./user-guide/configuration.md)

# Feature Guides

- [IPv6 Scanning](./features/ipv6-scanning.md)
- [Service Detection](./features/service-detection.md)
- [Idle Scan Technique](./features/idle-scan.md)
- [TLS Certificate Analysis](./features/tls-certificates.md)
- [Rate Limiting](./features/rate-limiting.md)
- [Firewall Evasion](./features/evasion-techniques.md)
- [Plugin System](./features/plugin-system.md)
- [Event System](./features/event-system.md)
- [Database Storage](./features/database-storage.md)
- [Nmap Compatibility](./features/nmap-compatibility.md)
- [Platform Support](./features/platform-support.md)

# Advanced Topics

- [Performance Tuning](./advanced/performance-tuning.md)
- [Performance Analysis](./advanced/performance-analysis.md)
- [Performance Characteristics](./advanced/performance-characteristics.md)
- [Benchmarking](./advanced/benchmarking.md)
- [Security Best Practices](./advanced/security-best-practices.md)
- [Efficiency Analysis](./advanced/efficiency-analysis.md)
- [TUI Architecture](./advanced/tui-architecture.md)

# Reference

- [Technical Specifications v2](./reference/tech-spec-v2.md)
- [API Reference](./reference/api-reference.md)
- [FAQ](./reference/faq.md)
- [Troubleshooting](./reference/troubleshooting.md)
- [Documentation Index](./reference/index.md)
- [Scanner Comparisons](./reference/comparisons/overview.md)
  - [Nmap Comparison](./reference/comparisons/nmap.md)
  - [Masscan Comparison](./reference/comparisons/masscan.md)
  - [ZMap Comparison](./reference/comparisons/zmap.md)
  - [RustScan Comparison](./reference/comparisons/rustscan.md)
  - [Naabu Comparison](./reference/comparisons/naabu.md)
- [Custom Commands](./reference/commands/overview.md)
- [Competitive Analysis](./reference/analysis/competitive-analysis.md)
- [Improvement Roadmap](./reference/analysis/improvements.md)

# Development

- [Architecture Overview](./architecture/overview.md)
- [Implementation Guide](./development/implementation.md)
- [Technical Specifications](./development/technical-specs.md)
- [Testing Strategy](./development/testing.md)
- [Testing Infrastructure](./development/testing-infrastructure.md)
- [Fuzz Testing](./development/fuzzing.md)
- [CI/CD Pipeline](./development/ci-cd.md)
- [Release Process](./development/release-process.md)
- [Documentation Standards](./development/doc-standards.md)
- [Contributing](./development/contributing.md)

# Project Management

- [Project Roadmap](./project/roadmap.md)
- [Current Status](./project/status.md)
- [Phase 6 Planning](./project/phase6-planning.md)

# Security & Compliance

- [Security Overview](./security/overview.md)
- [Responsible Use Guidelines](./security/responsible-use.md)
- [Legal Considerations](./security/legal.md)
- [Compliance](./security/compliance.md)

---

# Appendices

- [Appendix A: Phase Archives](./appendices/archives.md)
  - [Phase 4 Archive](./appendices/archives/phase4.md)
  - [Phase 5 Archive](./appendices/archives/phase5.md)
  - [Phase 6 Archive](./appendices/archives/phase6.md)
- [Appendix B: Sprint Reports](./appendices/sprint-reports.md)
- [Appendix C: Legacy Documentation](./appendices/legacy.md)
  - [Phase 4 Enhancements](./appendices/phase4-enhancements.md)
  - [Regression Strategy](./appendices/regression-strategy.md)
  - [Numbering System](./appendices/numbering-system.md)
- [Appendix D: Development Planning](./appendices/planning.md)
  - [Phase 5 Backlog](./appendices/phase5-backlog.md)
  - [Phase 6 Planning Report](./appendices/planning/phase6-plan.md)
```

**Total Estimated**: ~180-200 navigable entries

---

## Implementation Strategy

### Phase 3: Relocate & Configure

1. **Move book.toml**:
   ```bash
   git mv book.toml docs/book.toml
   ```

2. **Update book.toml paths**:
   ```toml
   src = "src"           # Changed from "docs"
   build-dir = "book"    # Remains "book" (now docs/book/)
   ```

3. **Create directory structure**:
   ```bash
   mkdir -p docs/src/{getting-started,user-guide,features,advanced,reference,development,project,security,appendices}
   mkdir -p docs/src/reference/{comparisons,commands,analysis}
   mkdir -p docs/src/appendices/{archives,planning}
   mkdir -p docs/book
   ```

4. **Update .gitignore**:
   ```
   # mdBook build output
   docs/book/
   ```

### Phase 4: Design SUMMARY.md

Create comprehensive `docs/src/SUMMARY.md` with:
- 11 major sections
- ~180-200 total entries
- Progressive complexity flow
- Clear navigation hierarchy

### Phase 5: Integrate Content

**Strategy**: Symlink or copy files from current locations to docs/src/
- Preserve original locations (don't delete)
- Update relative paths in markdown files
- Fix cross-references

**Example**:
```bash
# Copy with new names
cp docs/23-IPv6-GUIDE.md docs/src/features/ipv6-scanning.md
cp docs/24-SERVICE-DETECTION-GUIDE.md docs/src/features/service-detection.md
# ... etc
```

**Path Updates**:
- Change `../docs/` references to `./` or `../`
- Update image paths if needed
- Fix anchor links

### Phase 6: Polish & Verify

1. Build mdBook: `cd docs && mdbook build`
2. Check for broken links (link preprocessor)
3. Verify navigation (test prev/next)
4. Test search functionality
5. Review professional presentation

---

## File Exclusions

**Do NOT include in mdBook** (keep in repository only):

1. **Memory Bank Files**:
   - MEMORY-BANK-OPTIMIZATION-*.md
   - DOCUMENTATION-REVIEW-SCHEDULE.md

2. **Meta Documentation**:
   - SUMMARY.md (unless it's the mdBook SUMMARY.md)
   - README.md in archive/ (already has index)

3. **Sprint TODO Files** (active work):
   - SPRINT-*.TODO.md (keep in to-dos/ for development use)
   - Include only completion reports in appendices

4. **Duplicate Files**:
   - 34-EXAMPLES.md (prefer 34-EXAMPLES-GALLERY.md)
   - 12-BENCHMARKING-GUIDE.md (prefer 31-BENCHMARKING-GUIDE.md)

**Total Included**: ~85-90 files in mdBook (from 106 total)

---

## Success Metrics

✅ **Structure**: 11 sections, 180-200 chapters
✅ **Coverage**: 85-90 files integrated
✅ **Navigation**: 3 levels maximum depth
✅ **Build**: Zero errors, zero warnings
✅ **Links**: 100% internal links functional
✅ **Search**: Full-text search enabled
✅ **Quality**: Professional presentation matching OctoLLM

---

## Next Steps

**Phase 3**: Execute relocation (book.toml → docs/, create docs/src/, update config)
**Phase 4**: Create comprehensive SUMMARY.md
**Phase 5**: Integrate content (copy files, fix paths)
**Phase 6**: Build, test, polish

**Estimated Time**: 3-4 hours total

---

**End of Content Mapping Plan**
