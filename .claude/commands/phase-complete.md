# Phase Completion Automation

Comprehensive automation for Phase completion: verification, validation, testing, and release.

---

## USAGE

```
/phase-complete <phase_number> <target_version> <phase_name>
```

**Parameters:**
- `phase_number` - Phase number (e.g., 5, 6, 7)
- `target_version` - Target version (e.g., v0.5.0, v0.6.0, v1.0.0)
- `phase_name` - Phase name in quotes (e.g., "Advanced Features", "TUI Interface")

**Examples:**
```
/phase-complete 5 v0.5.0 "Advanced Features"
/phase-complete 6 v0.6.0 "Terminal User Interface"
/phase-complete 7 v1.0.0 "Production Release"
```

---

## OVERVIEW

**Purpose:** Automate comprehensive Phase completion workflow when all Phase sprints are finished. This command orchestrates verification, documentation updates, version management, testing, git workflow, and release creation with professional quality standards.

**Scope:** End-to-end Phase completion automation from planning through GitHub release

**Duration:** 14-19 hours (4 phases)

**Quality Standard:** Production-ready release meeting all ProRT-IP quality gates

---

## WORKFLOW PHASES

### Phase 1: Planning and Gap Analysis (3-4 hours)
1. Generate comprehensive TODO file
2. Analyze planned vs delivered features
3. Update version numbers across codebase
4. Verify build successful

### Phase 2: Documentation Comprehensive Update (5-7 hours)
1. Update README.md (version, status, metrics, features)
2. Update CHANGELOG.md (comprehensive Phase entry)
3. Update ROADMAP and STATUS
4. Review ALL docs/ files for accuracy
5. Generate and verify API documentation

### Phase 3: Git Workflow and Release (4-5 hours)
1. Quality validation (tests, clippy, rustdoc)
2. Create comprehensive commit
3. Create annotated git tag (150-200 lines)
4. Push to remote
5. Create GitHub release (200-250 lines)

### Phase 4: Final Verification (2-3 hours)
1. Release readiness checklist
2. Smoke testing
3. Update CLAUDE.local.md
4. Generate completion report

---

## SUB-AGENT INSTRUCTIONS

**You are a specialized Phase Completion Agent for ProRT-IP.** Your role is to execute the comprehensive Phase completion workflow autonomously with minimal user interaction. Follow these instructions systematically.

---

## INITIALIZATION

### Step 0.1: Validate Input Parameters

**Required Parameters:**
- `PHASE_NUMBER` - Extract from $1 (e.g., "5", "6", "7")
- `TARGET_VERSION` - Extract from $2 (e.g., "v0.5.0", "v0.6.0")
- `PHASE_NAME` - Extract from $3 (e.g., "Advanced Features")

**Validation:**
```bash
if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ]; then
    echo "❌ ERROR: Missing required parameters"
    echo ""
    echo "Usage: /phase-complete <phase_number> <target_version> <phase_name>"
    echo "Example: /phase-complete 5 v0.5.0 \"Advanced Features\""
    exit 1
fi

PHASE_NUMBER="$1"
TARGET_VERSION="$2"
PHASE_NAME="$3"

# Validate version format
if [[ ! "$TARGET_VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "❌ ERROR: Invalid version format: $TARGET_VERSION"
    echo "Expected format: vX.Y.Z (e.g., v0.5.0)"
    exit 1
fi
```

### Step 0.2: Pre-Flight Checks

**Check repository status:**
```bash
# Verify we're in ProRT-IP repository
if [ ! -f "Cargo.toml" ] || ! grep -q "prtip" Cargo.toml; then
    echo "❌ ERROR: Not in ProRT-IP repository"
    exit 1
fi

# Get current git status
UNCOMMITTED=$(git status --porcelain | wc -l)
if [ "$UNCOMMITTED" -gt 0 ]; then
    echo "⚠️  WARNING: $UNCOMMITTED uncommitted changes detected"
    echo ""
    git status --short
    echo ""
    echo "Phase completion will include these changes."
    echo ""
fi
```

**Check all Phase sprints complete:**
```bash
# This is informational - proceed regardless
echo "Checking Phase $PHASE_NUMBER sprint status..."

# Look for SPRINT-X.*-COMPLETE.md files
SPRINT_COMPLETE_COUNT=$(find . -name "SPRINT-${PHASE_NUMBER}.*-COMPLETE.md" 2>/dev/null | wc -l)

if [ "$SPRINT_COMPLETE_COUNT" -eq 0 ]; then
    echo "⚠️  WARNING: No sprint completion files found for Phase $PHASE_NUMBER"
    echo "Expected pattern: SPRINT-${PHASE_NUMBER}.*-COMPLETE.md"
    echo ""
else
    echo "✅ Found $SPRINT_COMPLETE_COUNT sprint completion reports"
fi

echo ""
```

**Check CI/CD status (optional):**
```bash
if command -v gh &> /dev/null; then
    echo "Checking latest CI status..."
    CI_STATUS=$(gh run list --limit 1 --json conclusion --jq '.[0].conclusion' 2>/dev/null || echo "unknown")

    if [ "$CI_STATUS" = "success" ]; then
        echo "✅ Latest CI run: PASSING"
    elif [ "$CI_STATUS" = "failure" ]; then
        echo "⚠️  WARNING: Latest CI run: FAILED"
        echo "Consider fixing CI before Phase completion"
    else
        echo "ℹ️  CI status: $CI_STATUS"
    fi
else
    echo "ℹ️  GitHub CLI not available - skipping CI check"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Starting Phase $PHASE_NUMBER Completion: $PHASE_NAME"
echo "Target Version: $TARGET_VERSION"
echo "════════════════════════════════════════════════════════════════"
echo ""
```

---

## PHASE 1: PLANNING AND GAP ANALYSIS

**Duration:** 3-4 hours
**Objective:** Generate comprehensive TODO, perform gap analysis, update version numbers

### Task 1.1: Generate Comprehensive TODO File

**File Location:** `to-dos/PHASE-${PHASE_NUMBER}-FINAL-VERIFICATION-${TARGET_VERSION}.md`

**Steps:**
1. Read existing TODO template if it exists (PHASE-5-FINAL-VERIFICATION-v0.5.0.md)
2. Create new TODO file for this Phase
3. Include all 4 phases with detailed tasks
4. Add comprehensive templates for commits, tags, releases

**TODO Structure (2,400+ lines minimum):**

```markdown
# PHASE ${PHASE_NUMBER} FINAL VERIFICATION TODO: ${TARGET_VERSION} Release Preparation

**Phase:** Phase ${PHASE_NUMBER} Final Verification and Release
**Status:** 🔄 IN PROGRESS (automated via /phase-complete)
**Started:** $(date +%Y-%m-%d)
**Target Completion:** $(date -d "+2 days" +%Y-%m-%d)
**Estimated Duration:** 14-19 hours
**Priority:** CRITICAL (Phase ${PHASE_NUMBER} completion milestone)
**Version Target:** ${TARGET_VERSION} (Phase ${PHASE_NUMBER} COMPLETE)

---

## Table of Contents

1. [Overview](#overview)
2. [Success Criteria](#success-criteria)
3. [Phase 1: Gap Analysis and Version Bump](#phase-1-gap-analysis-and-version-bump)
4. [Phase 2: Documentation Comprehensive Update](#phase-2-documentation-comprehensive-update)
5. [Phase 3: Git Workflow and Release](#phase-3-git-workflow-and-release)
6. [Phase 4: Additional Enhancements and Final Verification](#phase-4-additional-enhancements-and-final-verification)

---

## Overview

### Objective

Complete Phase ${PHASE_NUMBER} (${PHASE_NAME}) with comprehensive verification, validation, testing, and release of ProRT-IP ${TARGET_VERSION}.

### Context

**Current State:**
- **Phase ${PHASE_NUMBER}:** All sprints complete
- **Tests:** [To be determined]
- **Coverage:** [To be determined]
- **CI/CD:** All workflows passing
- **Quality:** Zero blocking issues

**Target State (${TARGET_VERSION}):**
- **Phase ${PHASE_NUMBER}:** 100% COMPLETE (official milestone)
- **Version:** ${TARGET_VERSION}
- **Quality:** All verification gates passed
- **Release:** GitHub release with comprehensive notes
- **Documentation:** All files current and accurate

### Deliverables

1. **TODO File:** This comprehensive planning document
2. **Gap Analysis Report:** Planned vs delivered features
3. **Version-Bumped Codebase:** All Cargo.toml + source references
4. **Updated Documentation:** README, CHANGELOG, ROADMAP, STATUS, guides
5. **Git Commit:** Comprehensive Phase ${PHASE_NUMBER} completion commit
6. **Git Tag ${TARGET_VERSION}:** 150-200 line annotated tag message
7. **GitHub Release:** 200-250 line release notes with assets
8. **Session Summary:** CLAUDE.local.md updated

---

## Success Criteria

### Must-Have (All Required)

- ✅ **Gap analysis complete** - All sprints verified against plans
- ✅ **Version consistency** - All version numbers updated to ${TARGET_VERSION}
- ✅ **Documentation current** - README, CHANGELOG, ROADMAP, STATUS, guides
- ✅ **Code quality gates** - cargo fmt, clippy (zero warnings), all tests passing
- ✅ **API documentation** - cargo doc successful, zero warnings
- ✅ **Git workflow complete** - Commit, tag, push, GitHub release
- ✅ **Professional quality** - Release notes 200-250 lines, technically detailed
- ✅ **Memory bank updated** - CLAUDE.local.md session summary
- ✅ **Production ready** - Ready for user deployment

---

## Phase 1: Gap Analysis and Version Bump

**Duration:** 3-4 hours

### Task 1.1: Read Phase Planning Documents

**Steps:**
- [ ] Find and read Phase ${PHASE_NUMBER} planning documents in to-dos/
- [ ] Identify all planned sprints and deliverables
- [ ] Review all SPRINT-${PHASE_NUMBER}.*-COMPLETE.md files
- [ ] Extract actual achievements from each sprint

### Task 1.2: Gap Analysis

**Steps:**
- [ ] Create comparison matrix: Planned vs Delivered
- [ ] Identify gaps: Features planned but not delivered
- [ ] Document descoped features with rationale
- [ ] Document deferred features with timeline
- [ ] Create Phase ${PHASE_NUMBER} achievements list

**Deliverable:** /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-GAP-ANALYSIS.md

### Task 1.3: Version Number Update

**Files to Update:**
- [ ] Cargo.toml (root)
- [ ] crates/*/Cargo.toml (all crates)
- [ ] src/cli.rs (version display)
- [ ] Search codebase for old version references

**Verification:**
- [ ] cargo build --release successful
- [ ] All version numbers consistent
- [ ] No old version references remain

---

## Phase 2: Documentation Comprehensive Update

**Duration:** 5-7 hours

### Task 2.1: Update README.md

**Sections to Update:**
- [ ] Version badges and current version
- [ ] Status and Phase ${PHASE_NUMBER} completion
- [ ] Features list (add Phase ${PHASE_NUMBER} features)
- [ ] Installation instructions
- [ ] Performance metrics
- [ ] Test count and coverage

### Task 2.2: Update CHANGELOG.md

**Add Phase ${PHASE_NUMBER} Entry:**

\`\`\`markdown
## [${TARGET_VERSION}] - $(date +%Y-%m-%d) - Phase ${PHASE_NUMBER}: ${PHASE_NAME} COMPLETE

**MAJOR MILESTONE:** Phase ${PHASE_NUMBER} completion with [X] sprints successfully delivered.

### Executive Summary

[2-3 paragraph overview of Phase ${PHASE_NUMBER} achievements]

### Sprint Achievements

**Sprint ${PHASE_NUMBER}.1: [Name]**
- [Key deliverables]
- [Metrics]

[Continue for all sprints...]

### Phase ${PHASE_NUMBER} Metrics

**Testing:**
- Total tests: [count] ([X]% passing)
- New tests: +[count] tests ([X]% growth)
- Coverage: [X]% ([+/-X]pp from Phase ${PHASE_NUMBER-1})

**Documentation:**
- Total documentation: [X] lines across [X] files
- New guides: [count]

**Performance:**
- [Key performance achievements]

### Technical Achievements

- [Technical highlight 1]
- [Technical highlight 2]
- [Technical highlight 3]

### Breaking Changes

[List any breaking changes, or state "None"]

### Files Modified

**Created:** [X] files ([X] lines)
**Modified:** [X] files ([+insertions/-deletions])
**Total commits:** [X] across [X] sprints

### Strategic Value

[How Phase ${PHASE_NUMBER} advances ProRT-IP's strategic goals]

### Next Phase

**Phase ${PHASE_NUMBER+1}:** [Brief preview if planned]

---
\`\`\`

### Task 2.3: Update docs/01-ROADMAP.md

**Updates:**
- [ ] Mark Phase ${PHASE_NUMBER} as COMPLETE ✅
- [ ] Update Phase ${PHASE_NUMBER} completion date
- [ ] Update overall project timeline
- [ ] Add Phase ${PHASE_NUMBER} achievements summary

### Task 2.4: Update docs/10-PROJECT-STATUS.md

**Updates:**
- [ ] Update Phase ${PHASE_NUMBER} status to COMPLETE
- [ ] Update overall completion percentage
- [ ] Update test metrics
- [ ] Update coverage metrics
- [ ] Update "Recent Achievements" section

### Task 2.5: Review ALL Documentation Files

**Review for Accuracy:**
- [ ] docs/00-ARCHITECTURE.md - Architecture current
- [ ] docs/03-DEV-SETUP.md - Setup instructions work
- [ ] docs/04-IMPLEMENTATION-GUIDE.md - Code structure current
- [ ] docs/06-TESTING.md - Test strategy accurate
- [ ] docs/08-SECURITY.md - Security practices current
- [ ] All Phase ${PHASE_NUMBER} guide files (if any)

### Task 2.6: Generate and Verify API Documentation

**Steps:**
- [ ] Run: cargo doc --no-deps --workspace
- [ ] Verify zero rustdoc warnings
- [ ] Check public API documentation complete
- [ ] Test doc examples compile (cargo test --doc)

---

## Phase 3: Git Workflow and Release

**Duration:** 4-5 hours

### Task 3.1: Quality Validation

**Run comprehensive quality checks:**

\`\`\`bash
# Format check
cargo fmt --all --check

# Clippy (zero warnings required)
cargo clippy --all-features -- -D warnings

# All tests passing
cargo test --workspace

# Documentation builds
cargo doc --no-deps --workspace

# Optional: Fuzz check
if [ -d fuzz ]; then
    echo "Note: Fuzz testing should have been run during Phase development"
fi
\`\`\`

**Success Criteria:**
- [ ] cargo fmt: No formatting needed
- [ ] cargo clippy: Zero warnings
- [ ] cargo test: 100% passing
- [ ] cargo doc: Zero warnings

### Task 3.2: Create Comprehensive Commit

**Commit Message Template (100-150 lines):**

\`\`\`
feat(phase${PHASE_NUMBER}): ${PHASE_NAME} - Phase ${PHASE_NUMBER} Complete

Complete Phase ${PHASE_NUMBER} (${PHASE_NAME}) marking a major milestone in ProRT-IP
development with [X] sprints successfully delivered, [Y] new tests, and [Z] key features
implemented over [N] weeks of focused development.

## Executive Summary

Phase ${PHASE_NUMBER} delivers comprehensive ${PHASE_NAME} capabilities including:
- [Key feature 1]
- [Key feature 2]
- [Key feature 3]

This phase represents [X] weeks of systematic development across [Y] sprints,
achieving [Z]% of planned objectives with zero blocking issues and production-ready quality.

## Sprint Summary

### Sprint ${PHASE_NUMBER}.1: [Sprint Name]
**Deliverables:**
- [Feature 1]: [Brief description]
- [Feature 2]: [Brief description]
**Impact:** [Performance/capability improvement]

[Continue for all sprints in Phase ${PHASE_NUMBER}...]

## Phase ${PHASE_NUMBER} Achievements

**Testing Excellence:**
- Total tests: [count] (100% passing)
- New tests added: +[count] ([X]% growth)
- Coverage: [X]% ([+/-X]pp change)
- Zero test regressions across all sprints

**Documentation Quality:**
- Total documentation: [X] lines across [Y] files
- New comprehensive guides: [count]
- Page equivalent: [X]+ pages
- 100% API documentation coverage

**Performance:**
- [Key performance metric 1]
- [Key performance metric 2]
- [Key performance metric 3]

**Code Quality:**
- Zero clippy warnings across entire codebase
- Zero rustdoc warnings
- Clean build on all platforms (7/7 CI jobs)
- 8/8 release targets successful

## Technical Highlights

**Architecture:**
- [Architectural improvement 1]
- [Architectural improvement 2]

**Implementation:**
- [Implementation highlight 1]
- [Implementation highlight 2]

**Integration:**
- [Integration achievement 1]
- [Integration achievement 2]

## Strategic Value

**Capability Enhancement:**
- [How Phase ${PHASE_NUMBER} enhances ProRT-IP capabilities]

**Market Position:**
- [How this positions ProRT-IP vs competitors]

**User Value:**
- [Direct user benefits from Phase ${PHASE_NUMBER}]

## Files Modified (Summary)

**Created:** [X] files ([Y] lines)
- [Major file 1] - [purpose]
- [Major file 2] - [purpose]

**Modified:** [X] files ([+insertions/-deletions])
- README.md - Version ${TARGET_VERSION}, Phase ${PHASE_NUMBER} status
- CHANGELOG.md - Comprehensive Phase ${PHASE_NUMBER} entry
- docs/01-ROADMAP.md - Phase ${PHASE_NUMBER} completion
- docs/10-PROJECT-STATUS.md - Updated metrics
- [Other major file updates]

**Documentation:**
- Updated [X] existing documentation files
- Created [Y] new guide files
- Total documentation impact: +[Z] lines

## Breaking Changes

[List breaking changes or state "None"]

## Testing Summary

**Test Execution:**
- Unit tests: [count] (100% passing)
- Integration tests: [count] (100% passing)
- Doc tests: [count] (if applicable)
- Fuzz tests: [executions] (0 crashes)

**Coverage Analysis:**
- Overall coverage: [X]%
- Core modules: [Y]%
- Critical paths: [Z]%

## Quality Assurance

**Code Quality:**
- ✅ cargo fmt (all code formatted)
- ✅ cargo clippy (zero warnings)
- ✅ cargo doc (zero warnings)
- ✅ All tests passing (100%)

**CI/CD:**
- ✅ 7/7 workflow jobs passing
- ✅ 8/8 release targets building
- ✅ Coverage reporting active
- ✅ Fuzz testing infrastructure (if applicable)

## Documentation Updates

**Updated Files:**
- README.md: Version, status, features, metrics
- CHANGELOG.md: Comprehensive Phase ${PHASE_NUMBER} entry
- ROADMAP.md: Phase ${PHASE_NUMBER} marked complete
- PROJECT-STATUS.md: Updated metrics and status
- CLAUDE.local.md: Session summary

**New Documentation:**
- [List any new guide files created]

## Deployment Readiness

✅ **Production Ready** - Phase ${PHASE_NUMBER} deliverables ready for user deployment
- All quality gates passed
- Comprehensive testing completed
- Documentation complete and current
- Zero known blocking issues

## Future Work

**Phase ${PHASE_NUMBER+1} Preview:**
- [Brief preview of next phase if planned]

**Follow-up Items:**
- [Any deferred features with target versions]

## Acknowledgments

Phase ${PHASE_NUMBER} represents [X] hours of focused development, comprehensive testing,
and professional documentation, maintaining ProRT-IP's commitment to quality and
reliability.

---

**Version:** ${TARGET_VERSION}
**Phase:** ${PHASE_NUMBER} COMPLETE
**Date:** $(date +%Y-%m-%d)
**Sprints:** [X] ([List sprint IDs])
**Quality:** ✅ Production Ready

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
\`\`\`

**Save commit message to:** /tmp/ProRT-IP/phase-${PHASE_NUMBER}-commit-${TARGET_VERSION}.txt

### Task 3.3: Stage and Commit Changes

\`\`\`bash
# Stage all changes
git add .

# Commit with comprehensive message
git commit -F /tmp/ProRT-IP/phase-${PHASE_NUMBER}-commit-${TARGET_VERSION}.txt

# Verify commit successful
if [ $? -eq 0 ]; then
    echo "✅ Commit successful"
    git log -1 --stat
else
    echo "❌ Commit failed"
    exit 1
fi
\`\`\`

### Task 3.4: Create Annotated Git Tag

**Git Tag Message Template (150-200 lines):**

\`\`\`
ProRT-IP ${TARGET_VERSION} - Phase ${PHASE_NUMBER}: ${PHASE_NAME} COMPLETE

═══════════════════════════════════════════════════════════════════════════
               PHASE ${PHASE_NUMBER} COMPLETION - MAJOR MILESTONE
═══════════════════════════════════════════════════════════════════════════

Release Date: $(date +%Y-%m-%d)
Phase: ${PHASE_NUMBER} (${PHASE_NAME})
Status: ✅ COMPLETE (100% of planned sprints)
Quality: Production Ready

---

## EXECUTIVE SUMMARY

ProRT-IP ${TARGET_VERSION} marks the successful completion of Phase ${PHASE_NUMBER}
(${PHASE_NAME}), representing [X] weeks of systematic development across [Y] sprints.
This release delivers [Z] major features, [A] new tests, and [B] comprehensive guides,
advancing ProRT-IP's position as a modern, production-ready network scanner.

**Key Achievements:**
- [Achievement 1]
- [Achievement 2]
- [Achievement 3]

**Phase ${PHASE_NUMBER} Impact:**
- Tests: [start] → [end] (+[X]%)
- Coverage: [start]% → [end]% (+[X]pp)
- Documentation: +[X] lines across [Y] files

---

## PHASE ${PHASE_NUMBER} SPRINT SUMMARY

### Sprint ${PHASE_NUMBER}.1: [Sprint Name]
**Duration:** [X] hours | **Status:** ✅ COMPLETE

**Deliverables:**
- [Feature 1]: [Description and impact]
- [Feature 2]: [Description and impact]

**Metrics:**
- Tests: [count] (100% passing)
- Coverage: [X]%
- Performance: [metric]

**Strategic Value:** [How this sprint advances overall goals]

[Continue for ALL sprints in Phase ${PHASE_NUMBER}...]

---

## COMPREHENSIVE FEATURES DELIVERED

### Category 1: [Feature Category]

**[Feature 1 Name]:**
- **What:** [Description]
- **Why:** [Value proposition]
- **How:** [Technical approach]
- **Impact:** [Performance/capability metrics]

**[Feature 2 Name]:**
[Same structure...]

[Continue for all major features...]

---

## TECHNICAL SPECIFICATIONS

### Architecture

**Enhancements:**
- [Architectural improvement 1]
- [Architectural improvement 2]

**Design Patterns:**
- [Pattern 1 and application]
- [Pattern 2 and application]

### Implementation Details

**Core Components:**
- [Component 1]: [Lines of code, tests, purpose]
- [Component 2]: [Lines of code, tests, purpose]

**Integration:**
- [Integration point 1]
- [Integration point 2]

### Performance Characteristics

**Benchmarks:**
- [Metric 1]: [Baseline] → [Result] ([X]% improvement)
- [Metric 2]: [Baseline] → [Result] ([X]% improvement)

**Resource Usage:**
- Memory: [Baseline] → [Result]
- CPU: [Baseline] → [Result]

---

## TESTING AND QUALITY ASSURANCE

### Test Coverage

**Test Statistics:**
- **Total Tests:** [count] (100% passing)
- **New Tests:** +[count] ([X]% growth from Phase ${PHASE_NUMBER-1})
- **Unit Tests:** [count]
- **Integration Tests:** [count]
- **Coverage:** [X]% overall, [Y]% core modules

**Fuzz Testing:** (if applicable)
- Total executions: [count]
- Crashes found: 0
- Coverage: [X] code paths

### Quality Metrics

**Code Quality:**
- ✅ Zero clippy warnings across entire codebase
- ✅ Zero rustdoc warnings
- ✅ 100% API documentation coverage
- ✅ All tests passing (100% success rate)

**CI/CD Status:**
- ✅ 7/7 workflow jobs passing
- ✅ 8/8 release targets building
- ✅ Automated coverage reporting
- ✅ Security audit passing (0 vulnerabilities)

---

## DOCUMENTATION

### Documentation Metrics

**Total Documentation:** [X] lines across [Y] files
- README.md: [lines]
- CHANGELOG.md: [lines]
- Technical docs: [lines] across [count] files
- Guides: [lines] across [count] files

**New Documentation (Phase ${PHASE_NUMBER}):**
- [Guide 1 name]: [lines]
- [Guide 2 name]: [lines]

**Page Equivalent:** [X]+ pages of comprehensive technical documentation

### Documentation Updates

**Updated Files:**
- README.md - Version ${TARGET_VERSION}, Phase ${PHASE_NUMBER} status, features
- CHANGELOG.md - Comprehensive Phase ${PHASE_NUMBER} entry (150+ lines)
- docs/01-ROADMAP.md - Phase ${PHASE_NUMBER} marked complete
- docs/10-PROJECT-STATUS.md - Updated metrics and completion status
- [Other updated docs...]

---

## FILES MODIFIED

**Created ([X] files, [Y] lines):**
- [File 1] - [Purpose and size]
- [File 2] - [Purpose and size]
- [Continue for major new files...]

**Modified ([X] files, [+insertions/-deletions]):**
- [File 1] - [Changes and rationale]
- [File 2] - [Changes and rationale]
- [Continue for major modified files...]

**Commits:** [X] across Phase ${PHASE_NUMBER} ([Y] sprints)
- [Sprint 1]: [commits] commits
- [Sprint 2]: [commits] commits
- [Continue...]

---

## BREAKING CHANGES

[List breaking changes with migration notes, or state:]
**None** - This release maintains full backward compatibility.

---

## KNOWN ISSUES

[List any known issues, or state:]
**None** - Zero known blocking issues. All quality gates passed.

---

## STRATEGIC VALUE

### Capability Enhancement

[How Phase ${PHASE_NUMBER} enhances ProRT-IP's core capabilities]

### Competitive Position

[How Phase ${PHASE_NUMBER} positions ProRT-IP relative to alternatives like Nmap, Masscan, ZMap]

### User Value

[Direct benefits to end users from Phase ${PHASE_NUMBER} features]

### Foundation for Future Work

[How Phase ${PHASE_NUMBER} sets up Phase ${PHASE_NUMBER+1} and beyond]

---

## DEPLOYMENT

**Status:** ✅ Production Ready

**Platforms Supported:** (from CI/CD release matrix)
- Linux (x86_64, ARM64, musl)
- macOS (x86_64, ARM64)
- Windows (x86_64, ARM64)
- FreeBSD (x86_64)

**Installation:**
See README.md for installation instructions for ${TARGET_VERSION}.

**Requirements:**
- [System requirement 1]
- [System requirement 2]

---

## NEXT PHASE

**Phase ${PHASE_NUMBER+1}:** [Name if planned]
- [Preview of upcoming work]

**Planned Features:**
- [Feature 1]
- [Feature 2]

**Timeline:** [If available]

---

## ACKNOWLEDGMENTS

Phase ${PHASE_NUMBER} represents [X] hours of development, [Y] hours of testing,
and [Z] hours of documentation across [A] sprints. This phase maintains ProRT-IP's
commitment to production-quality software with comprehensive testing, professional
documentation, and zero-compromise on quality.

---

**Phase:** ${PHASE_NUMBER} (${PHASE_NAME})
**Version:** ${TARGET_VERSION}
**Status:** ✅ COMPLETE
**Quality:** Production Ready
**Date:** $(date +%Y-%m-%d)
**Sprints:** [X] ([List all sprint IDs])

═══════════════════════════════════════════════════════════════════════════

Generated with ProRT-IP Phase Completion Automation (/phase-complete)
\`\`\`

**Create tag:**

\`\`\`bash
# Save tag message
cat > /tmp/ProRT-IP/tag-message-${TARGET_VERSION}.txt << 'TAG_EOF'
[Paste the filled-in template above]
TAG_EOF

# Create annotated tag
git tag -a ${TARGET_VERSION} -F /tmp/ProRT-IP/tag-message-${TARGET_VERSION}.txt

# Verify tag created
if git tag | grep -q "^${TARGET_VERSION}$"; then
    echo "✅ Git tag ${TARGET_VERSION} created successfully"
    git show ${TARGET_VERSION} --no-patch
else
    echo "❌ Failed to create git tag"
    exit 1
fi
\`\`\`

### Task 3.5: Push to Remote

\`\`\`bash
# Push commit
echo "Pushing commit to origin..."
git push origin main

# Push tag
echo "Pushing tag ${TARGET_VERSION} to origin..."
git push origin ${TARGET_VERSION}

# Verify pushed
if git ls-remote --tags origin | grep -q "${TARGET_VERSION}"; then
    echo "✅ Tag ${TARGET_VERSION} pushed to remote"
else
    echo "⚠️  WARNING: Tag may not have pushed successfully"
fi
\`\`\`

### Task 3.6: Create GitHub Release

**GitHub Release Notes Template (200-250 lines):**

[INCLUDE ALL TAG CONTENT ABOVE, PLUS:]

\`\`\`markdown
## Installation

### Pre-built Binaries

Download pre-built binaries for your platform from the Assets section below.

**Linux (x86_64):**
\`\`\`bash
wget https://github.com/doublegate/ProRT-IP/releases/download/${TARGET_VERSION}/prtip-x86_64-unknown-linux-gnu
chmod +x prtip-x86_64-unknown-linux-gnu
sudo mv prtip-x86_64-unknown-linux-gnu /usr/local/bin/prtip
\`\`\`

**Linux (x86_64, static musl):**
\`\`\`bash
wget https://github.com/doublegate/ProRT-IP/releases/download/${TARGET_VERSION}/prtip-x86_64-unknown-linux-musl
chmod +x prtip-x86_64-unknown-linux-musl
sudo mv prtip-x86_64-unknown-linux-musl /usr/local/bin/prtip
\`\`\`

**macOS (ARM64 / Apple Silicon):**
\`\`\`bash
wget https://github.com/doublegate/ProRT-IP/releases/download/${TARGET_VERSION}/prtip-aarch64-apple-darwin
chmod +x prtip-aarch64-apple-darwin
sudo mv prtip-aarch64-apple-darwin /usr/local/bin/prtip
\`\`\`

**macOS (x86_64 / Intel):**
\`\`\`bash
wget https://github.com/doublegate/ProRT-IP/releases/download/${TARGET_VERSION}/prtip-x86_64-apple-darwin
chmod +x prtip-x86_64-apple-darwin
sudo mv prtip-x86_64-apple-darwin /usr/local/bin/prtip
\`\`\`

**Windows (x86_64):**
Download `prtip-x86_64-pc-windows-msvc.exe` from Assets below and place in your PATH.

### Build from Source

\`\`\`bash
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
git checkout ${TARGET_VERSION}
cargo build --release
sudo cp target/release/prtip /usr/local/bin/
\`\`\`

**Requirements:**
- Rust 1.70+
- libpcap (Linux/macOS) or Npcap (Windows)
- See docs/03-DEV-SETUP.md for details

---

## Platform Compatibility Matrix

| Platform | Architecture | Status | Binary Name |
|----------|--------------|--------|-------------|
| Linux | x86_64 (GNU) | ✅ Tested | prtip-x86_64-unknown-linux-gnu |
| Linux | x86_64 (musl) | ✅ Tested | prtip-x86_64-unknown-linux-musl |
| Linux | ARM64 | ✅ Tested | prtip-aarch64-unknown-linux-gnu |
| macOS | ARM64 (M1/M2) | ✅ Tested | prtip-aarch64-apple-darwin |
| macOS | x86_64 (Intel) | ✅ Tested | prtip-x86_64-apple-darwin |
| Windows | x86_64 | ✅ Tested | prtip-x86_64-pc-windows-msvc.exe |
| Windows | ARM64 | ⚠️  Experimental | prtip-aarch64-pc-windows-msvc.exe |
| FreeBSD | x86_64 | ⚠️  Community | prtip-x86_64-unknown-freebsd |

---

## Upgrade Notes

**From v0.4.x to ${TARGET_VERSION}:**

[Include any migration notes, or state:]
This release maintains backward compatibility. Simply replace your binary.

**Configuration Changes:**
[List any config changes, or state:] None

**Breaking Changes:**
[List any breaking changes with migration guide, or state:] None

---

## Known Issues

[List known issues or state:]
**None** - Zero known blocking issues in ${TARGET_VERSION}.

**Windows-Specific:**
- Npcap 1.70+ required (install from https://npcap.com/)
- Administrator privileges required for raw sockets

**macOS-Specific:**
- ChmodBPF or root privileges required
- See docs/03-DEV-SETUP.md for setup

---

## Documentation

**Comprehensive Documentation:** 50,000+ lines across 55+ files

**Quick Start:**
- README.md - Overview and quick start
- docs/00-ARCHITECTURE.md - System architecture
- docs/03-DEV-SETUP.md - Development setup
- docs/04-IMPLEMENTATION-GUIDE.md - Code structure

**Phase ${PHASE_NUMBER} Guides:**
[List Phase ${PHASE_NUMBER}-specific guide files if any]

**Online Documentation:**
- GitHub: https://github.com/doublegate/ProRT-IP
- API Docs: https://docs.rs/prtip

---

## Support

**Issues:** https://github.com/doublegate/ProRT-IP/issues
**Discussions:** https://github.com/doublegate/ProRT-IP/discussions
**Security:** See SECURITY.md for responsible disclosure

---

## Checksums

\`\`\`
SHA256 checksums for release binaries:
[To be filled in by release automation or manually:]

[checksum]  prtip-x86_64-unknown-linux-gnu
[checksum]  prtip-x86_64-unknown-linux-musl
[checksum]  prtip-aarch64-unknown-linux-gnu
[checksum]  prtip-x86_64-apple-darwin
[checksum]  prtip-aarch64-apple-darwin
[checksum]  prtip-x86_64-pc-windows-msvc.exe
[checksum]  prtip-aarch64-pc-windows-msvc.exe
[checksum]  prtip-x86_64-unknown-freebsd
\`\`\`

**Verification:**
\`\`\`bash
sha256sum -c checksums.txt
\`\`\`

---

## Release Assets

The following pre-built binaries are attached to this release:

- `prtip-x86_64-unknown-linux-gnu` - Linux x86_64 (GNU libc)
- `prtip-x86_64-unknown-linux-musl` - Linux x86_64 (static, musl)
- `prtip-aarch64-unknown-linux-gnu` - Linux ARM64
- `prtip-x86_64-apple-darwin` - macOS Intel
- `prtip-aarch64-apple-darwin` - macOS Apple Silicon
- `prtip-x86_64-pc-windows-msvc.exe` - Windows x86_64
- `prtip-aarch64-pc-windows-msvc.exe` - Windows ARM64 (experimental)
- `prtip-x86_64-unknown-freebsd` - FreeBSD x86_64

**Note:** Binaries are automatically built by GitHub Actions CI/CD.
Verify checksums above before use.

---

**Version:** ${TARGET_VERSION}
**Phase:** ${PHASE_NUMBER} COMPLETE (${PHASE_NAME})
**Released:** $(date +%Y-%m-%d)
**License:** GPL-3.0
**Repository:** https://github.com/doublegate/ProRT-IP

🤖 Generated with [Claude Code](https://claude.com/claude-code) via /phase-complete
\`\`\`

**Create GitHub Release:**

\`\`\`bash
# Save release notes
cat > /tmp/ProRT-IP/release-notes-${TARGET_VERSION}.md << 'RELEASE_EOF'
[Paste the filled-in template above]
RELEASE_EOF

# Create release via GitHub CLI
if command -v gh &> /dev/null; then
    echo "Creating GitHub release ${TARGET_VERSION}..."

    gh release create ${TARGET_VERSION} \
        --title "ProRT-IP ${TARGET_VERSION} - Phase ${PHASE_NUMBER}: ${PHASE_NAME} COMPLETE" \
        --notes-file /tmp/ProRT-IP/release-notes-${TARGET_VERSION}.md \
        --latest

    if [ $? -eq 0 ]; then
        echo "✅ GitHub release ${TARGET_VERSION} created successfully"
        echo ""
        echo "View release: https://github.com/doublegate/ProRT-IP/releases/tag/${TARGET_VERSION}"
    else
        echo "❌ Failed to create GitHub release"
        echo ""
        echo "Fallback: Create release manually at:"
        echo "https://github.com/doublegate/ProRT-IP/releases/new?tag=${TARGET_VERSION}"
        echo ""
        echo "Release notes saved to: /tmp/ProRT-IP/release-notes-${TARGET_VERSION}.md"
    fi
else
    echo "⚠️  GitHub CLI not available"
    echo ""
    echo "Create release manually at:"
    echo "https://github.com/doublegate/ProRT-IP/releases/new?tag=${TARGET_VERSION}"
    echo ""
    echo "Release notes saved to: /tmp/ProRT-IP/release-notes-${TARGET_VERSION}.md"
fi
\`\`\`

---

## Phase 4: Final Verification and Completion

**Duration:** 2-3 hours

### Task 4.1: Release Readiness Checklist

**Run comprehensive validation:**

\`\`\`bash
echo "Running release readiness checklist..."
echo ""

PASS=0
FAIL=0

# 1. All tests passing
if cargo test --workspace --quiet 2>&1 | grep -q "test result: ok"; then
    echo "✅ All tests passing"
    ((PASS++))
else
    echo "❌ Tests failing"
    ((FAIL++))
fi

# 2. Zero clippy warnings
if cargo clippy --all-features --quiet 2>&1 | grep -q "error:"; then
    echo "❌ Clippy warnings present"
    ((FAIL++))
else
    echo "✅ Zero clippy warnings"
    ((PASS++))
fi

# 3. Documentation builds
if cargo doc --no-deps --workspace --quiet 2>&1 | grep -q "error:"; then
    echo "❌ Documentation errors"
    ((FAIL++))
else
    echo "✅ Documentation builds successfully"
    ((PASS++))
fi

# 4. Version consistency
VERSIONS=$(find . -name Cargo.toml -exec grep "^version" {} \; | sort -u | wc -l)
if [ "$VERSIONS" -eq 1 ]; then
    echo "✅ Version consistent across all crates"
    ((PASS++))
else
    echo "❌ Version inconsistent"
    ((FAIL++))
fi

# 5. Git tag exists
if git tag | grep -q "^${TARGET_VERSION}$"; then
    echo "✅ Git tag ${TARGET_VERSION} exists"
    ((PASS++))
else
    echo "❌ Git tag ${TARGET_VERSION} missing"
    ((FAIL++))
fi

# 6. Tag pushed to remote
if git ls-remote --tags origin | grep -q "${TARGET_VERSION}"; then
    echo "✅ Tag pushed to remote"
    ((PASS++))
else
    echo "❌ Tag not on remote"
    ((FAIL++))
fi

# 7. GitHub release created
if command -v gh &> /dev/null; then
    if gh release view ${TARGET_VERSION} &>/dev/null; then
        echo "✅ GitHub release created"
        ((PASS++))
    else
        echo "❌ GitHub release missing"
        ((FAIL++))
    fi
else
    echo "⚠️  Cannot verify GitHub release (gh CLI not available)"
fi

# 8. CHANGELOG updated
if grep -q "${TARGET_VERSION}" CHANGELOG.md; then
    echo "✅ CHANGELOG updated"
    ((PASS++))
else
    echo "❌ CHANGELOG not updated"
    ((FAIL++))
fi

# 9. README updated
if grep -q "${TARGET_VERSION}" README.md; then
    echo "✅ README updated"
    ((PASS++))
else
    echo "❌ README not updated"
    ((FAIL++))
fi

# 10. ROADMAP updated
if grep -q "Phase ${PHASE_NUMBER}.*COMPLETE" docs/01-ROADMAP.md; then
    echo "✅ ROADMAP updated"
    ((PASS++))
else
    echo "❌ ROADMAP not updated"
    ((FAIL++))
fi

# 11. CI/CD passing
if command -v gh &> /dev/null; then
    CI_STATUS=$(gh run list --limit 1 --json conclusion --jq '.[0].conclusion' 2>/dev/null)
    if [ "$CI_STATUS" = "success" ]; then
        echo "✅ CI/CD passing"
        ((PASS++))
    else
        echo "❌ CI/CD not passing: $CI_STATUS"
        ((FAIL++))
    fi
else
    echo "⚠️  Cannot verify CI/CD (gh CLI not available)"
fi

# 12. No security vulnerabilities
if command -v cargo-audit &> /dev/null; then
    if cargo audit 2>&1 | grep -q "error:"; then
        echo "❌ Security vulnerabilities present"
        ((FAIL++))
    else
        echo "✅ No security vulnerabilities"
        ((PASS++))
    fi
else
    echo "⚠️  cargo-audit not installed (skipping)"
fi

# 13. Fuzz testing (if applicable)
if [ -d fuzz ]; then
    CRASHES=$(find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | wc -l)
    if [ "$CRASHES" -eq 0 ]; then
        echo "✅ No fuzz crashes"
        ((PASS++))
    else
        echo "❌ Fuzz crashes present: $CRASHES"
        ((FAIL++))
    fi
fi

echo ""
echo "════════════════════════════════════════"
echo "Release Readiness: $PASS passed, $FAIL failed"
echo "════════════════════════════════════════"
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo "⚠️  WARNING: $FAIL failed checks"
    echo "Review and fix before considering release complete"
fi
\`\`\`

### Task 4.2: Optional Smoke Testing

**Basic functionality verification:**

\`\`\`bash
# Build release binary
cargo build --release

# Verify binary exists and is executable
if [ -f target/release/prtip ]; then
    echo "✅ Release binary built"

    # Test version display
    if target/release/prtip --version | grep -q "${TARGET_VERSION}"; then
        echo "✅ Version display correct: ${TARGET_VERSION}"
    else
        echo "⚠️  WARNING: Version mismatch in binary"
    fi

    # Test help display
    if target/release/prtip --help &>/dev/null; then
        echo "✅ Help display working"
    else
        echo "⚠️  WARNING: Help display failed"
    fi

    # Optional: Basic scan test (requires root/sudo)
    # echo "Note: Full scan testing requires elevated privileges"
else
    echo "❌ Release binary not found"
fi
\`\`\`

### Task 4.3: Update CLAUDE.local.md

**Session Summary Format:**

\`\`\`markdown
## $(date +%Y-%m-%d): Phase ${PHASE_NUMBER} Completion - ${TARGET_VERSION} Released ✅

**Objective:** Complete Phase ${PHASE_NUMBER} (${PHASE_NAME}) and release ${TARGET_VERSION}
**Duration:** [X] hours (automated via /phase-complete)
**Result:** SUCCESS ✅

**Activities:**

- **Planning:**
  - Generated comprehensive TODO (2,400+ lines)
  - Gap analysis: Planned vs delivered features
  - Identified [X] achievements, [Y] descoped features

- **Version Management:**
  - Updated all Cargo.toml files to ${TARGET_VERSION}
  - Updated version references in source code
  - Verified build successful

- **Documentation:**
  - README.md: Version, status, Phase ${PHASE_NUMBER} features
  - CHANGELOG.md: Comprehensive Phase ${PHASE_NUMBER} entry (150+ lines)
  - ROADMAP.md: Phase ${PHASE_NUMBER} marked COMPLETE
  - PROJECT-STATUS.md: Updated metrics
  - Reviewed all [X] documentation files

- **Quality Assurance:**
  - cargo fmt: ✅ All code formatted
  - cargo clippy: ✅ Zero warnings
  - cargo test: ✅ [X] tests passing (100%)
  - cargo doc: ✅ Zero warnings

- **Git Workflow:**
  - Created comprehensive commit (100-150 lines)
  - Created annotated tag ${TARGET_VERSION} (150-200 lines)
  - Pushed to remote: commit + tag
  - Created GitHub release (200-250 lines)

- **Release Assets:**
  - 8/8 platform binaries building via CI/CD
  - Comprehensive release notes published
  - Installation instructions included

**Deliverables:**

- ✅ TODO file (2,400+ lines): to-dos/PHASE-${PHASE_NUMBER}-FINAL-VERIFICATION-${TARGET_VERSION}.md
- ✅ Gap analysis report: /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-GAP-ANALYSIS.md
- ✅ Version-bumped codebase (all references to ${TARGET_VERSION})
- ✅ Documentation updates ([X] files)
- ✅ Git commit (comprehensive, 100-150 lines)
- ✅ Git tag ${TARGET_VERSION} (150-200 lines)
- ✅ GitHub release (200-250 lines)

**Files Modified:**

- **Documentation ([X] files):**
  - README.md - Version, status, features
  - CHANGELOG.md - Phase ${PHASE_NUMBER} comprehensive entry
  - docs/01-ROADMAP.md - Phase ${PHASE_NUMBER} complete
  - docs/10-PROJECT-STATUS.md - Updated metrics
  - [Other updated files...]

- **Source Code ([X] files):**
  - Cargo.toml (root + [X] crates) - Version ${TARGET_VERSION}
  - src/cli.rs - Version display
  - [Other version references...]

**Phase ${PHASE_NUMBER} Metrics:**

- **Testing:**
  - Total tests: [X] (100% passing)
  - New tests (Phase ${PHASE_NUMBER}): +[Y]
  - Coverage: [Z]%

- **Documentation:**
  - Total lines: [X]+ across [Y] files
  - New guides: [Z]

- **Sprints:**
  - Completed: [X]/[X] (100%)
  - Duration: [Y] weeks

**Quality Verification:**

- ✅ All [X] tests passing
- ✅ Zero clippy warnings
- ✅ Zero rustdoc warnings
- ✅ CI/CD: 7/7 jobs passing
- ✅ Release assets: 8/8 platforms
- ✅ Documentation complete and current

**Issues Encountered:**

[List any issues, or state:] None - smooth execution

**Result:** Phase ${PHASE_NUMBER} COMPLETE - ${TARGET_VERSION} released successfully

**Next Phase:** [Brief preview of Phase ${PHASE_NUMBER+1} if planned]

---
\`\`\`

**Prepend to CLAUDE.local.md Recent Sessions section**

### Task 4.4: Generate Completion Report

**Final Report Format:**

\`\`\`markdown
# PHASE ${PHASE_NUMBER} COMPLETION REPORT

═══════════════════════════════════════════════════════════════════════════
               PHASE ${PHASE_NUMBER}: ${PHASE_NAME} - COMPLETE ✅
═══════════════════════════════════════════════════════════════════════════

**Phase:** ${PHASE_NUMBER} (${PHASE_NAME})
**Version:** ${TARGET_VERSION}
**Completed:** $(date +%Y-%m-%d)
**Duration:** [Total phase duration from first sprint to release]
**Status:** ✅ SUCCESS - Production Ready

---

## EXECUTIVE SUMMARY

Phase ${PHASE_NUMBER} successfully completed with [X] sprints delivered, [Y] tests added,
and [Z] comprehensive features implemented. This phase advances ProRT-IP's capabilities
in [domain] and positions the project for [future work].

**Key Achievements:**
- [Achievement 1]
- [Achievement 2]
- [Achievement 3]

---

## SPRINT SUMMARY

| Sprint | Name | Status | Tests | Duration |
|--------|------|--------|-------|----------|
| ${PHASE_NUMBER}.1 | [Name] | ✅ | [count] | [hours] |
| ${PHASE_NUMBER}.2 | [Name] | ✅ | [count] | [hours] |
[Continue for all sprints...]

**Total:** [X] sprints, [Y] hours, [Z] tests

---

## METRICS

### Testing

- **Start (Phase ${PHASE_NUMBER-1}):** [X] tests
- **End (Phase ${PHASE_NUMBER}):** [Y] tests
- **Growth:** +[Z] tests ([A]%)
- **Success Rate:** 100% (all tests passing)

### Coverage

- **Start:** [X]%
- **End:** [Y]%
- **Improvement:** +[Z] percentage points

### Documentation

- **Total:** [X] lines across [Y] files
- **New Guides:** [Z] files
- **Page Equivalent:** [A]+ pages

### Code Quality

- ✅ Zero clippy warnings
- ✅ Zero rustdoc warnings
- ✅ 100% API documentation
- ✅ CI/CD: 7/7 jobs passing

---

## DELIVERABLES

### Features

[List all major features delivered in Phase ${PHASE_NUMBER}]

### Documentation

[List all documentation created/updated]

### Infrastructure

[List any infrastructure improvements]

---

## GAP ANALYSIS

### Planned vs Delivered

- **Planned Features:** [X]
- **Delivered:** [Y]
- **Completion Rate:** [Z]%

### Descoped Features

[List features explicitly descoped with rationale and target version]

### Deferred Features

[List features deferred with rationale and target version]

---

## QUALITY ASSURANCE

### Code Quality Gates

- ✅ cargo fmt (all code formatted)
- ✅ cargo clippy (zero warnings)
- ✅ cargo test (100% passing)
- ✅ cargo doc (zero warnings)

### CI/CD

- ✅ 7/7 workflow jobs passing
- ✅ 8/8 release targets building
- ✅ Coverage reporting active
- ✅ Security audit passing

### Release Quality

- ✅ Commit message: 100-150 lines (comprehensive)
- ✅ Git tag: 150-200 lines (detailed)
- ✅ GitHub release: 200-250 lines (professional)
- ✅ Documentation: Complete and current

---

## STRATEGIC IMPACT

### Capability Enhancement

[How Phase ${PHASE_NUMBER} enhances ProRT-IP capabilities]

### Competitive Position

[How Phase ${PHASE_NUMBER} positions ProRT-IP vs alternatives]

### User Value

[Direct user benefits from Phase ${PHASE_NUMBER}]

### Technical Foundation

[How Phase ${PHASE_NUMBER} enables future work]

---

## LESSONS LEARNED

### What Went Well

- [Success 1]
- [Success 2]
- [Success 3]

### What Could Improve

- [Improvement opportunity 1]
- [Improvement opportunity 2]

### Process Refinements

- [Process improvement 1]
- [Process improvement 2]

---

## NEXT PHASE

**Phase ${PHASE_NUMBER+1}:** [Name if planned]

**Planned Focus:**
- [Focus area 1]
- [Focus area 2]

**Timeline:** [If available]

---

## FILES CREATED/MODIFIED

### Created ([X] files)

- [File 1] - [Purpose and size]
- [File 2] - [Purpose and size]
[Continue...]

### Modified ([X] files)

- [File 1] - [Changes]
- [File 2] - [Changes]
[Continue...]

### Commits

- Total commits: [X]
- Sprint commits: [breakdown]
- Final release commit: [SHA]

---

## RELEASE INFORMATION

**Version:** ${TARGET_VERSION}
**Tag:** ${TARGET_VERSION}
**Release URL:** https://github.com/doublegate/ProRT-IP/releases/tag/${TARGET_VERSION}

**Platforms:** 8 (Linux x86_64/ARM64/musl, macOS Intel/ARM64, Windows x86_64/ARM64, FreeBSD)

**Installation:** See README.md or GitHub release notes

---

## VERIFICATION

**Release Readiness Checklist:** [X]/[Y] passed

- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Documentation builds
- ✅ Version consistency
- ✅ Git tag exists and pushed
- ✅ GitHub release created
- ✅ CHANGELOG updated
- ✅ README updated
- ✅ ROADMAP updated
- ✅ CI/CD passing
- ✅ No security vulnerabilities

---

## ACKNOWLEDGMENTS

Phase ${PHASE_NUMBER} represents [X] hours of focused development across [Y] sprints,
maintaining ProRT-IP's commitment to production-quality software with comprehensive
testing, professional documentation, and zero-compromise on quality.

**Automated via:** /phase-complete command
**Execution:** [X] hours (estimated 14-19 hours)
**Grade:** [A+/A/B/C based on quality and completeness]

---

**Phase ${PHASE_NUMBER} Status:** ✅ COMPLETE
**Version Released:** ${TARGET_VERSION}
**Date:** $(date +%Y-%m-%d)
**Production Ready:** ✅ YES

═══════════════════════════════════════════════════════════════════════════

Generated: $(date +%Y-%m-%d %H:%M:%S)
Tool: ProRT-IP Phase Completion Automation (/phase-complete)
\`\`\`

**Save report to:** /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-COMPLETION-REPORT.md

---

## FINAL OUTPUT

Display comprehensive completion summary:

\`\`\`bash
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "       PHASE ${PHASE_NUMBER} COMPLETION - SUCCESS ✅"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Phase: ${PHASE_NUMBER} (${PHASE_NAME})"
echo "Version: ${TARGET_VERSION}"
echo "Status: ✅ COMPLETE - Production Ready"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "DELIVERABLES"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "✅ TODO File:"
echo "   to-dos/PHASE-${PHASE_NUMBER}-FINAL-VERIFICATION-${TARGET_VERSION}.md"
echo ""
echo "✅ Gap Analysis:"
echo "   /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-GAP-ANALYSIS.md"
echo ""
echo "✅ Version Updates:"
echo "   All Cargo.toml files → ${TARGET_VERSION}"
echo "   Source code version references updated"
echo ""
echo "✅ Documentation Updates:"
echo "   - README.md (version, status, features)"
echo "   - CHANGELOG.md (comprehensive Phase ${PHASE_NUMBER} entry)"
echo "   - docs/01-ROADMAP.md (Phase ${PHASE_NUMBER} complete)"
echo "   - docs/10-PROJECT-STATUS.md (updated metrics)"
echo "   - [Additional files updated]"
echo ""
echo "✅ Git Workflow:"
echo "   - Commit: Comprehensive (100-150 lines)"
echo "   - Tag: ${TARGET_VERSION} (150-200 lines)"
echo "   - Pushed: origin/main + ${TARGET_VERSION}"
echo ""
echo "✅ GitHub Release:"
echo "   https://github.com/doublegate/ProRT-IP/releases/tag/${TARGET_VERSION}"
echo "   Release notes: 200-250 lines (professional quality)"
echo ""
echo "✅ Completion Report:"
echo "   /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-COMPLETION-REPORT.md"
echo ""
echo "✅ Memory Bank Updated:"
echo "   CLAUDE.local.md (session summary added)"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "METRICS"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "Tests: [X] (100% passing)"
echo "Coverage: [Y]%"
echo "Documentation: [Z] lines across [A] files"
echo "Sprints: [B] completed"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "QUALITY ASSURANCE"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "✅ cargo fmt - All code formatted"
echo "✅ cargo clippy - Zero warnings"
echo "✅ cargo test - 100% passing"
echo "✅ cargo doc - Zero warnings"
echo "✅ CI/CD - All jobs passing"
echo "✅ Security - No vulnerabilities"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "RELEASE INFORMATION"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "Version: ${TARGET_VERSION}"
echo "Tag: ${TARGET_VERSION} (annotated)"
echo "Release: https://github.com/doublegate/ProRT-IP/releases/tag/${TARGET_VERSION}"
echo "Platforms: 8 (Linux, macOS, Windows, FreeBSD)"
echo "Status: ✅ Production Ready"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "ARTIFACTS"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "Planning:"
echo "  • to-dos/PHASE-${PHASE_NUMBER}-FINAL-VERIFICATION-${TARGET_VERSION}.md"
echo "  • /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-GAP-ANALYSIS.md"
echo ""
echo "Git:"
echo "  • /tmp/ProRT-IP/phase-${PHASE_NUMBER}-commit-${TARGET_VERSION}.txt"
echo "  • /tmp/ProRT-IP/tag-message-${TARGET_VERSION}.txt"
echo "  • /tmp/ProRT-IP/release-notes-${TARGET_VERSION}.md"
echo ""
echo "Reports:"
echo "  • /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-COMPLETION-REPORT.md"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "NEXT STEPS"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "1. ✅ Review GitHub release (link above)"
echo "2. ✅ Verify CI/CD builds all 8 platforms"
echo "3. ✅ Announce release (optional)"
echo "4. 📋 Plan Phase ${PHASE_NUMBER+1} (if applicable)"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo "REFERENCES"
echo "────────────────────────────────────────────────────────────────"
echo ""
echo "• Completion Report: /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-COMPLETION-REPORT.md"
echo "• Gap Analysis: /tmp/ProRT-IP/PHASE-${PHASE_NUMBER}-GAP-ANALYSIS.md"
echo "• GitHub Release: https://github.com/doublegate/ProRT-IP/releases/tag/${TARGET_VERSION}"
echo "• Documentation: docs/ directory"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "    Phase ${PHASE_NUMBER} Complete - Thank you! 🎉"
echo "═══════════════════════════════════════════════════════════════"
echo ""
\`\`\`

---

## INTEGRATION WITH OTHER COMMANDS

**Pre-Execution:**
- `/ci-status` - Check CI/CD before starting (optional, informational)

**During Execution:**
- `/doc-update` - Can be used for individual doc updates
- `/rust-check` - Quality validation (integrated into workflow)
- `/pre-release` - Pre-flight checks (partially integrated)

**Post-Execution:**
- `/daily-log` - Can generate session log (optional)
- Plan next phase manually or with custom tooling

---

## ERROR HANDLING

### Common Issues and Solutions

**Issue: Build fails after version bump**
```bash
# Solution: Check Cargo.toml syntax
cargo check
# Fix any dependency version conflicts
```

**Issue: Tests fail during quality checks**
```bash
# Solution: Run targeted test debugging
/test-quick <pattern>
# Fix failing tests before proceeding
```

**Issue: Clippy warnings present**
```bash
# Solution: Fix warnings
cargo clippy --all-features --fix --allow-dirty
# Review and commit fixes
```

**Issue: Git push fails**
```bash
# Solution: Check remote connection
git remote -v
# Verify authentication
gh auth status
```

**Issue: GitHub release creation fails**
```bash
# Solution: Create release manually
# Use saved release notes at:
# /tmp/ProRT-IP/release-notes-${TARGET_VERSION}.md
# URL: https://github.com/doublegate/ProRT-IP/releases/new?tag=${TARGET_VERSION}
```

### Error Handling Strategy

1. **Validation Errors:** Stop execution, display error, provide fix guidance
2. **Build Errors:** Stop execution, run `/rust-check`, provide diagnostics
3. **Test Failures:** Stop execution, identify failing tests, provide `/test-quick` command
4. **Git Errors:** Attempt to continue, log warning, provide manual instructions
5. **GitHub API Errors:** Attempt to continue, provide manual fallback instructions

---

## SUCCESS CRITERIA

**Phase 1 Success:**
- ✅ TODO file generated (2,400+ lines)
- ✅ Gap analysis complete
- ✅ Version numbers updated and consistent
- ✅ Build successful

**Phase 2 Success:**
- ✅ README.md updated (version, status, features)
- ✅ CHANGELOG.md updated (comprehensive entry)
- ✅ ROADMAP and STATUS updated
- ✅ All documentation reviewed
- ✅ API docs generated (zero warnings)

**Phase 3 Success:**
- ✅ All quality checks passed (fmt, clippy, test, doc)
- ✅ Comprehensive commit created
- ✅ Git tag created (150-200 lines)
- ✅ Changes pushed to remote
- ✅ GitHub release created (200-250 lines)

**Phase 4 Success:**
- ✅ Release readiness checklist passed
- ✅ CLAUDE.local.md updated
- ✅ Completion report generated
- ✅ Final summary displayed

**Overall Success:**
- ✅ All 4 phases completed
- ✅ Zero blocking issues
- ✅ Production-ready release
- ✅ Professional quality documentation
- ✅ Comprehensive audit trail

---

## ESTIMATED DURATION

**Phase 1:** 3-4 hours
- TODO generation: 1 hour
- Gap analysis: 1.5 hours
- Version updates: 0.5-1 hour

**Phase 2:** 5-7 hours
- README/CHANGELOG: 2 hours
- ROADMAP/STATUS: 1 hour
- Documentation review: 2-3 hours
- API docs: 0.5-1 hour

**Phase 3:** 4-5 hours
- Quality checks: 0.5 hour
- Commit creation: 1.5 hours
- Tag creation: 1.5 hours
- GitHub release: 1.5-2 hours

**Phase 4:** 2-3 hours
- Readiness checklist: 0.5 hour
- Smoke testing: 0.5-1 hour
- Documentation: 0.5-1 hour
- Reporting: 0.5 hour

**Total:** 14-19 hours

---

## RELATED COMMANDS

- `/sprint-complete <sprint-id>` - Complete individual sprint (used during Phase development)
- `/pre-release` - Pre-release validation (partially integrated)
- `/rust-check` - Quality validation (integrated)
- `/ci-status` - CI/CD status check (optional pre-check)
- `/doc-update <type>` - Individual doc updates (available during execution)
- `/daily-log` - Session logging (optional post-execution)

---

## SEE ALSO

- `docs/01-ROADMAP.md` - Project roadmap and phase planning
- `docs/10-PROJECT-STATUS.md` - Overall project status
- `CLAUDE.md` - Project development guidance
- `CLAUDE.local.md` - Session history and current status
- Previous releases (v0.4.0-v0.4.9) for quality examples

---

**Execute comprehensive Phase ${PHASE_NUMBER} completion workflow now.**
