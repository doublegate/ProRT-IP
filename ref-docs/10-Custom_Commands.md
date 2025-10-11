Custom Claude Code Commands for ProRT-IP - Comprehensive Analysis & Generation

Analysis Summary

Existing Commands Analysis

After analyzing the three existing commands:

1. /sub-agent - Complex workflow orchestration pattern
  - Uses $* for flexible task passing
  - Comprehensive PROJECT CONTEXT embedding
  - SYSTEMATIC APPROACH with 8-12 phases
  - ERROR HANDLING and SUCCESS CRITERIA sections
  - Designed for autonomous sub-task execution
2. /mem-reduce - Documentation optimization
  - Memory bank consolidation and cleanup
  - Structured approach to reducing token usage
  - Project-specific context preservation
3. /stage-commit - Git workflow automation
  - Multi-phase commit preparation
  - Parallel git operations (status, diff, log)
  - Conventional commit format enforcement
  - Testing before commit

Project Pattern Insights

From CLAUDE.local.md and recent sessions:

- Sprint-based development (4.1-4.14 complete)
- Performance-obsessed (198x improvements, 643 tests, comprehensive benchmarking)
- Documentation-heavy (28KB+ reports, comprehensive summaries)
- Quality-focused (100% test pass rate, zero clippy warnings)
- Multi-platform (Linux/Windows/macOS/FreeBSD support)
- Systematic workflows (benchmark → analyze → fix → test → document)

Common Development Workflows Identified

1. Performance optimization cycle: benchmark → profile → optimize → validate
2. Bug fix workflow: reproduce → analyze → fix → test → document
3. Sprint completion: validate → benchmark → document → commit → release
4. Feature development: plan → implement → test → integrate → document
5. Quality assurance: fmt → clippy → test → audit → build
6. Documentation updates: README → CHANGELOG → memory banks → status files

---
Generated Custom Commands (14 Total)

1. /rust-check - Fast Rust Quality Pipeline

Purpose: Run complete Rust quality checks (fmt, clippy, test) in optimal order
File Path: .claude/commands/rust-check.md
Estimated Complexity: Simple (3 phases)

You are an expert Rust developer working on ProRT-IP, a high-performance network scanner.

Run a comprehensive Rust quality check pipeline optimized for speed and thoroughness.

## PROJECT CONTEXT

**ProRT-IP Status:**
- 643 tests across 4 crates (prtip-core, prtip-network, prtip-scanner, prtip-cli)
- Zero tolerance for clippy warnings or formatting issues
- Maintains >90% code coverage for core modules
- MSRV: Rust 1.70+
- Quality standard: All checks must pass before considering work complete

**Quality Requirements:**
- `cargo fmt --check` must pass (no formatting changes needed)
- `cargo clippy -- -D warnings` must pass (zero warnings)
- All tests must pass (643/643)
- Build must succeed in release mode

## SYSTEMATIC APPROACH

### Phase 1: Fast Fail Checks (Parallel Execution)

Execute these in parallel for fastest feedback:

```bash
cargo fmt --check && cargo clippy -- -D warnings

Expected: Zero output = success. Any output = needs fixing.

Phase 2: Full Test Suite

cargo test --all-features

Expected: "test result: ok. 643 passed"

Phase 3: Release Build Verification

cargo build --release

Expected: Build succeeds, binary at target/release/prtip

ERROR HANDLING

If cargo fmt fails:
- Run cargo fmt to auto-fix
- Report which files were modified
- Re-run phase 1

If clippy fails:
- Show all warnings with file:line locations
- Categorize: must-fix vs suggestions
- Provide fix recommendations

If tests fail:
- Show which tests failed with details
- Check for platform-specific failures (Windows timing tests)
- Suggest running specific test: cargo test test_name -- --nocapture

If build fails:
- Check for dependency issues (cargo update?)
- Verify MSRV compatibility
- Check for platform-specific build errors

SUCCESS CRITERIA

✅ cargo fmt --check passes (zero changes needed)
✅ cargo clippy passes with -D warnings (zero warnings)
✅ All 643 tests pass
✅ Release build succeeds
✅ No compilation warnings
✅ Total execution time reported (<5 minutes typical)

DELIVERABLES

Provide concise summary:

Rust Quality Check Results
==========================
✅ Formatting: PASS
✅ Clippy: PASS (0 warnings)
✅ Tests: PASS (643/643)
✅ Build: PASS
Duration: Xm Ys

If any checks fail, provide actionable next steps.

**Usage Example:**
```bash
/rust-check

---
2. /bench-compare - Performance Comparison Between Commits

Purpose: Compare benchmark results between two git refs (commits/branches/tags)
File Path: .claude/commands/bench-compare.md
Estimated Complexity: Medium (5 phases)

You are a performance optimization expert working on ProRT-IP, analyzing benchmark results.

Compare performance metrics between two git references (commits, branches, or tags) to detect regressions or
improvements.

**Usage:** `/bench-compare <baseline-ref> <comparison-ref> [test-args]`

**Examples:**
- `/bench-compare v0.2.0 HEAD` - Compare current code vs v0.2.0
- `/bench-compare origin/main HEAD -p 80,443 127.0.0.1` - Compare with custom test
- `/bench-compare ed1d105 HEAD` - Compare specific commits

## PROJECT CONTEXT

**ProRT-IP Performance History:**
- Sprint 4.13: Fixed 10x regression (polling overhead)
- Sprint 4.14: 3-17x faster network timeouts
- Phase 4 final: 66.3% improvement (10K ports: 117ms → 39.4ms)
- Typical benchmarks: 1K ports (~4.5ms), 10K ports (~39ms), 65K ports (~191ms)
- Target: 227K+ ports/second on localhost (i9-10850K)

**Benchmark Standards:**
- Use hyperfine for statistical analysis (--warmup 3 --runs 10)
- Measure: mean time, stddev, min/max, user/system CPU
- Test scenarios: 1K common ports, 10K ports, 65K full range
- Localhost (127.0.0.1) for consistent results
- Report regression threshold: >5% slower = investigate, >10% = blocker

## SYSTEMATIC APPROACH

### Phase 1: Validate Git References

```bash
git rev-parse --verify $1 && git rev-parse --verify $2

Store full commit hashes for documentation.

Phase 2: Build Baseline Version

git checkout $1
cargo build --release
cp target/release/prtip /tmp/prtip-baseline
git log -1 --oneline

Capture build time and commit info.

Phase 3: Build Comparison Version

git checkout $2
cargo build --release
cp target/release/prtip /tmp/prtip-comparison
git log -1 --oneline

Capture build time and commit info.

Phase 4: Execute Benchmark Suite

Default test (if no args provided): -p 1-1000 127.0.0.1

hyperfine --warmup 3 --runs 10 \
  --export-json /tmp/bench-comparison.json \
  --export-markdown /tmp/bench-comparison.md \
  "/tmp/prtip-baseline $TEST_ARGS" \
  "/tmp/prtip-comparison $TEST_ARGS"

Phase 5: Analysis & Reporting

Parse hyperfine JSON output:
- Extract mean, stddev, min, max for both versions
- Calculate percentage change: ((new - old) / old) * 100
- Determine significance: >5% = notable, >10% = critical
- Check stddev overlap (statistical significance)

ERROR HANDLING

If git checkout fails:
- Stash uncommitted changes first
- Provide clear error about which ref is invalid
- Suggest git branch -a or git tag to find valid refs

If build fails:
- Report which version failed to build
- Show compilation errors
- Suggest checking CHANGELOG for breaking changes

If hyperfine not installed:
- Provide installation command: cargo install hyperfine
- Offer fallback: manual time measurements with 10 iterations

If benchmark crashes:
- Check if test requires sudo/root
- Verify target is reachable
- Suggest simpler test: -p 80 127.0.0.1

SUCCESS CRITERIA

✅ Both versions build successfully
✅ Both versions execute without errors
✅ Statistical confidence (stddev < 10% of mean)
✅ Clear percentage change calculated
✅ Regression threshold evaluation
✅ Detailed report with commit info

DELIVERABLES

Benchmark Comparison Report

# Performance Comparison: <baseline> vs <comparison>

## Test Configuration
- Command: prtip $TEST_ARGS
- Runs: 10 (3 warmup)
- System: <hostname>, <CPU>, <kernel>

## Results

| Metric | Baseline (<short-hash>) | Comparison (<short-hash>) | Change |
|--------|------------------------|---------------------------|---------|
| Mean | X.XXs ± X.XXs | X.XXs ± X.XXs | ±XX.X% |
| Min | X.XXs | X.XXs | ±XX.X% |
| Max | X.XXs | X.XXs | ±XX.X% |
| User CPU | X.XXs | X.XXs | ±XX.X% |
| System CPU | X.XXs | X.XXs | ±XX.X% |

## Analysis

**Performance Verdict:** [IMPROVEMENT | REGRESSION | NEUTRAL]

- **Threshold Check:** [PASS | INVESTIGATE | BLOCKER]
- **Statistical Significance:** [HIGH | MEDIUM | LOW]
- **Recommendation:** [...]

## Commit Details

**Baseline:** <full-hash>
<commit-message>

**Comparison:** <full-hash>
<commit-message>

## Raw Data

- JSON: /tmp/bench-comparison.json
- Markdown: /tmp/bench-comparison.md

Save report to /tmp/ProRT-IP/bench-compare-<timestamp>.md

Restore original git state: git checkout -

Provide actionable next steps based on results.

**Usage Example:**
```bash
/bench-compare ed1d105 HEAD
/bench-compare v0.2.0 HEAD -p 1-10000 192.168.1.1

---
3. /sprint-start - Initialize New Sprint with Planning

Purpose: Set up a new sprint with task breakdown and documentation structure
File Path: .claude/commands/sprint-start.md
Estimated Complexity: Medium (6 phases)

You are a project manager and technical lead for ProRT-IP, starting a new development sprint.

Initialize a new sprint with proper planning, task breakdown, and documentation structure.

**Usage:** `/sprint-start <sprint-id> <sprint-objective>`

**Examples:**
- `/sprint-start 4.15 "Implement packet fragmentation for IDS evasion"`
- `/sprint-start 5.1 "Phase 5 kickoff - Plugin system foundation"`

## PROJECT CONTEXT

**ProRT-IP Sprint History:**
- Sprint 4.1-4.14: Phase 4 performance optimization (COMPLETE)
- Sprint pattern: 4.1-4.14 format (phase.sprint within phase)
- Phase 5 upcoming: Advanced features (plugins, idle scanning, fragmentation)
- Sprint deliverables: Code + tests + benchmarks + documentation + CHANGELOG entry

**Sprint Standards:**
- Duration: 1-3 days typical
- Task breakdown: 3-8 actionable tasks
- Success criteria: All tests pass, zero regressions, comprehensive docs
- Deliverables: Implementation summary, benchmark results, CHANGELOG update
- Memory bank updates: CLAUDE.local.md session entry

**Current Status:**
- Phase 4: COMPLETE (643 tests, 12,016+ lines, production-ready)
- Next: Phase 5 Advanced Features (docs/01-ROADMAP.md weeks 14-16)

## SYSTEMATIC APPROACH

### Phase 1: Sprint Context Analysis

Parse sprint ID and objective:
- Extract phase number: `4.15` → Phase 4
- Extract sprint number: `4.15` → Sprint 15
- Validate objective is specific and measurable
- Check if phase exists in docs/01-ROADMAP.md

### Phase 2: Task Breakdown

Decompose objective into 3-8 actionable tasks:

**Template Structure:**
1. **Research/Analysis:** Review existing code, reference implementations
2. **Core Implementation:** New modules/functions with tests
3. **Integration:** Wire into existing systems (scheduler, CLI, config)
4. **Testing:** Unit tests + integration tests + edge cases
5. **Performance Validation:** Benchmarks comparing before/after
6. **Documentation:** Code docs + user docs + CHANGELOG entry
7. **Memory Bank Update:** Session summary in CLAUDE.local.md

**Task Format:**
- [ ] Task description (estimated time, files affected)

### Phase 3: Create Sprint Directory Structure

```bash
mkdir -p /tmp/ProRT-IP/sprint-$SPRINT_ID/
cd /tmp/ProRT-IP/sprint-$SPRINT_ID/

# Create planning documents
touch sprint-plan.md
touch task-checklist.md
touch implementation-notes.md

Phase 4: Generate Sprint Plan Document

Create comprehensive sprint plan at /tmp/ProRT-IP/sprint-$SPRINT_ID/sprint-plan.md:

# Sprint $SPRINT_ID: $OBJECTIVE

**Status:** Planning
**Start Date:** $(date +%Y-%m-%d)
**Estimated Duration:** X days
**Phase:** Phase X

## Objective

$OBJECTIVE

## Background

[Context from ROADMAP.md, related sprints, technical requirements]

## Task Breakdown

### 1. Research/Analysis (Xh)
- [ ] Review existing codebase modules: [list]
- [ ] Study reference implementations: [list]
- [ ] Identify integration points: [list]

### 2. Core Implementation (Xh)
- [ ] Create module: crates/prtip-*/src/*.rs
- [ ] Implement core functionality: [list]
- [ ] Add unit tests (target: 10+ tests)

### 3. Integration (Xh)
- [ ] Wire into scheduler.rs
- [ ] Add CLI flags to args.rs
- [ ] Update config.rs with new options

### 4. Testing (Xh)
- [ ] Unit tests for new modules
- [ ] Integration tests in integration_*.rs
- [ ] Edge case coverage
- [ ] Cross-platform validation

### 5. Performance Validation (Xh)
- [ ] Benchmark baseline (before changes)
- [ ] Benchmark with feature (after changes)
- [ ] Verify zero regression on existing tests
- [ ] Document performance impact

### 6. Documentation (Xh)
- [ ] Inline code documentation (/// comments)
- [ ] Update README.md if user-facing
- [ ] Add CHANGELOG.md entry
- [ ] Create implementation summary

### 7. Memory Bank Update (30m)
- [ ] Update CLAUDE.local.md with session summary
- [ ] Update Current Status metrics
- [ ] Add sprint entry to Recent Sessions

## Success Criteria

✅ All tasks completed
✅ All tests passing (643+ expected)
✅ Zero clippy warnings
✅ Performance validated (zero regression)
✅ Documentation complete
✅ CHANGELOG.md updated
✅ Memory bank updated

## Deliverables

1. Implementation code (*.rs files)
2. Test suite (XX+ new tests)
3. Benchmark results (JSON + Markdown)
4. Implementation summary (/tmp/ProRT-IP/sprint-$SPRINT_ID/implementation-summary.md)
5. CHANGELOG.md entry
6. CLAUDE.local.md session entry

## Technical Approach

[Detailed technical approach based on objective]

## Reference Materials

- docs/01-ROADMAP.md: [relevant section]
- docs/00-ARCHITECTURE.md: [relevant components]
- docs/04-IMPLEMENTATION-GUIDE.md: [relevant patterns]
- Reference code: [external references if applicable]

## Risk Assessment

**Potential Issues:**
- [Risk 1]: [Mitigation strategy]
- [Risk 2]: [Mitigation strategy]

## Timeline

- Day 1: Research + Core Implementation
- Day 2: Integration + Testing
- Day 3: Performance + Documentation

## Notes

[Any additional context, decisions, or considerations]

Phase 5: Create Task Checklist

Create simple checklist at /tmp/ProRT-IP/sprint-$SPRINT_ID/task-checklist.md:

# Sprint $SPRINT_ID Task Checklist

## Phase 1: Research ⏳
- [ ] Review existing code
- [ ] Study references
- [ ] Identify integration points

## Phase 2: Implementation ⏳
- [ ] Create new module(s)
- [ ] Core functionality
- [ ] Unit tests

## Phase 3: Integration ⏳
- [ ] Scheduler integration
- [ ] CLI flags
- [ ] Config updates

## Phase 4: Testing ⏳
- [ ] Unit tests
- [ ] Integration tests
- [ ] Edge cases
- [ ] Cross-platform

## Phase 5: Performance ⏳
- [ ] Baseline benchmark
- [ ] Feature benchmark
- [ ] Regression check

## Phase 6: Documentation ⏳
- [ ] Code docs
- [ ] User docs
- [ ] CHANGELOG
- [ ] Implementation summary

## Phase 7: Finalization ⏳
- [ ] Memory bank update
- [ ] Final testing
- [ ] Ready for commit

Phase 6: Generate Implementation Notes Template

Create notes file at /tmp/ProRT-IP/sprint-$SPRINT_ID/implementation-notes.md:

# Sprint $SPRINT_ID Implementation Notes

## Session Log

### $(date +%Y-%m-%d) - Sprint Start
- Sprint initialized
- Planning complete
- Ready to begin implementation

---

[Add notes as sprint progresses]

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| | | |

## Code Locations

| Component | File Path | Lines | Tests |
|-----------|-----------|-------|-------|
| | | | |

## Performance Metrics

| Scenario | Before | After | Change |
|----------|--------|-------|--------|
| | | | |

## Issues Encountered

| Issue | Solution | Date |
|-------|----------|------|
| | | |

## TODO / Blockers

- [ ]

## References Used

-

ERROR HANDLING

If sprint ID invalid:
- Validate format: X.Y (phase.sprint)
- Suggest checking docs/01-ROADMAP.md for phase numbers
- Recommend sequential sprint numbers

If objective unclear:
- Request specific, measurable objective
- Provide examples from past sprints
- Suggest consulting ROADMAP.md

If directory creation fails:
- Check /tmp/ProRT-IP/ exists
- Verify write permissions
- Suggest alternative: ~/ProRT-IP-sprints/

SUCCESS CRITERIA

✅ Sprint ID validated and parsed
✅ Objective is specific and actionable
✅ Task breakdown is comprehensive (3-8 tasks)
✅ Sprint directory structure created
✅ Planning documents generated (sprint-plan.md, task-checklist.md, implementation-notes.md)
✅ Timeline estimated
✅ Success criteria defined
✅ Ready to begin implementation

DELIVERABLES

Sprint Initialization Summary

Sprint $SPRINT_ID Initialized
=============================
Objective: $OBJECTIVE
Phase: Phase X
Duration: X days estimated
Tasks: X identified

Directory: /tmp/ProRT-IP/sprint-$SPRINT_ID/

Documents Created:
- sprint-plan.md (comprehensive planning)
- task-checklist.md (quick reference)
- implementation-notes.md (session log)

Next Steps:
1. Review sprint-plan.md
2. Begin Phase 1: Research/Analysis
3. Update task-checklist.md as you progress
4. Log decisions in implementation-notes.md

Ready to start Sprint $SPRINT_ID!

Provide clear next actions to begin sprint work.

**Usage Example:**
```bash
/sprint-start 5.1 "Implement Lua plugin system foundation"

---
4. /sprint-complete - Finalize Sprint with Documentation

Purpose: Complete sprint with comprehensive documentation and memory bank updates
File Path: .claude/commands/sprint-complete.md
Estimated Complexity: Medium (6 phases)

You are a project manager and technical lead for ProRT-IP, finalizing a completed sprint.

Generate comprehensive sprint completion documentation including implementation summary, benchmark results,
and memory bank updates.

**Usage:** `/sprint-complete <sprint-id> <brief-summary>`

**Examples:**
- `/sprint-complete 4.14 "Network timeout optimization (3-17x faster)"`
- `/sprint-complete 4.13 "Fixed polling overhead regression (10x speedup)"`

## PROJECT CONTEXT

**ProRT-IP Sprint Completion Standards:**
- Implementation summary (10-30KB comprehensive report)
- Benchmark results (JSON + Markdown from hyperfine)
- CHANGELOG.md entry with technical details
- CLAUDE.local.md session entry (Recent Sessions section)
- Current Status metrics update
- Zero loose ends or incomplete tasks

**Required Sections in Implementation Summary:**
1. Objective & Background
2. Root Cause Analysis (if bug fix)
3. Implementation Details (files changed, lines added/removed)
4. Testing Results (test count, pass rate)
5. Performance Results (before/after with percentage change)
6. Deliverables List
7. Result Assessment (SUCCESS ✅ / PARTIAL ⚠️ / BLOCKED ❌)

**Sprint History Reference:**
- Sprint 4.14: Network timeout optimization (comprehensive 28KB report)
- Sprint 4.13: Polling regression fix (10x improvement)
- Sprint 4.12: Progress bar sub-millisecond polling fix
- All sprints follow consistent documentation format

## SYSTEMATIC APPROACH

### Phase 1: Gather Sprint Artifacts

Collect all sprint-related files:

```bash
SPRINT_DIR="/tmp/ProRT-IP/sprint-$SPRINT_ID"

# List all artifacts
ls -lh $SPRINT_DIR/

Expected files:
- sprint-plan.md (if exists)
- task-checklist.md (if exists)
- implementation-notes.md (if exists)
- Benchmark files (*.json, *.md)
- Test outputs (*.txt)
- Any analysis documents

Phase 2: Generate Implementation Summary

Create comprehensive report at $SPRINT_DIR/implementation-summary.md:

# Sprint $SPRINT_ID Implementation Summary

**Date:** $(date +%Y-%m-%d)
**Status:** COMPLETE ✅
**Duration:** X hours/days
**Sprint Objective:** $BRIEF_SUMMARY

## Objective

[Detailed objective from sprint-plan.md or user input]

## Background

[Context: Why was this sprint needed? What problem did it solve?]

## Root Cause Analysis

[If bug fix: Detailed analysis of root cause]
[If feature: Technical requirements and design decisions]

### Problem Statement

[Specific issue or requirement]

### Investigation Findings

[What was discovered during analysis]

### Technical Root Cause

[Exact cause: file, line, logic issue]

## Implementation Details

### Files Modified

| File | Changes | Description |
|------|---------|-------------|
| path/to/file.rs | +XX -YY lines | [Brief description] |
| ... | ... | ... |

**Total:** XX files modified, YYY lines added, ZZZ lines removed (net: ±AAA lines)

### Key Changes

1. **[Component/Module]:** [Description of changes]
2. **[Component/Module]:** [Description of changes]
3. ...

### Code Highlights

[Show critical code snippets with explanations - 2-3 examples]

```rust
// Example code snippet

Testing Results

Test Suite Status

- Total Tests: XXX passing (100% success rate)
- New Tests: YY added this sprint
- Test Coverage: >90% for modified modules
- Regression Check: Zero regressions detected

Test Breakdown

| Package| Tests | Status |
|---------------|-------|--------|
| prtip-core    | XXX   | ✅ PASS |
| prtip-network | XXX   | ✅ PASS |
| prtip-scanner | XXX   | ✅ PASS |
| prtip-cli     | XXX   | ✅ PASS |

Cross-Platform Validation

- ✅ Linux: All tests pass
- ✅ Windows: All tests pass (timing tolerances adjusted if needed)
- ✅ macOS: All tests pass

Performance Results

Benchmark Configuration

- Tool: hyperfine --warmup 3 --runs 10
- System: , ,
- Test Scenario: [Description]

Before vs After

| Metric     | Before (Baseline) | After (Sprint $SPRINT_ID) | Change |
|------------|-------------------|---------------------------|--------|
| Mean Time  | X.XXs ± X.XXs     | X.XXs ± X.XXs      | ±XX.X% |
| Min Time   | X.XXs      | X.XXs| ±XX.X% |
| Max Time   | X.XXs      | X.XXs| ±XX.X% |
| Throughput | XX,XXX pps | XX,XXX pps  | ±XX.X% |

Performance Verdict

Result: [MAJOR IMPROVEMENT | IMPROVEMENT | NEUTRAL | REGRESSION]

[Detailed analysis of performance impact]

Benchmark Files

- JSON: $SPRINT_DIR/benchmark-*.json
- Markdown: $SPRINT_DIR/benchmark-*.md
- Test outputs: $SPRINT_DIR/test-*.txt

Quality Assurance

- ✅ cargo fmt --check passes
- ✅ cargo clippy -- -D warnings passes (zero warnings)
- ✅ cargo build --release succeeds
- ✅ cargo test passes (XXX/XXX)
- ✅ cargo audit passes (zero vulnerabilities)

Documentation Updates

Files Updated

- README.md (if user-facing changes)
- CHANGELOG.md (comprehensive entry)
- CLAUDE.local.md (session summary + metrics)
- docs/* (if architectural changes)

CHANGELOG Entry

[Show generated CHANGELOG.md entry]

Deliverables

1. Implementation Code:
  - XX files modified (YYY lines)
  - ZZ new tests
  - Zero clippy warnings
2. Testing:
  - All XXX tests passing (100%)
  - Zero regressions
  - Cross-platform validated
3. Performance:
  - Benchmark results (JSON + Markdown)
  - XX% improvement vs baseline
4. Documentation:
  - Implementation summary (this document)
  - CHANGELOG.md entry
  - Memory bank updates

Lessons Learned

What Went Well

- [Positive aspect 1]
- [Positive aspect 2]

Challenges Encountered

- [Challenge 1]: [How it was resolved]
- [Challenge 2]: [How it was resolved]

Future Improvements

- [Recommendation 1]
- [Recommendation 2]

Next Steps

Immediate (Sprint $NEXT_SPRINT)

- [Next priority task]
- [Next priority task]

Future Sprints

- [Long-term improvement]
- [Long-term improvement]

Result Assessment

Status: SUCCESS ✅

[Overall assessment of sprint success, achievement of objectives, and readiness for next phase]

---
Sprint $SPRINT_ID COMPLETE

### Phase 3: Update CHANGELOG.md

Add comprehensive entry to CHANGELOG.md:

```markdown
### Sprint $SPRINT_ID - $BRIEF_SUMMARY ($(date +%Y-%m-%d))

**Objective:** [Full objective]

**Changes:**
- [Change 1]
- [Change 2]
- ...

**Performance:**
- [Metric]: Before XXms → After YYms (±ZZ% change)
- [Metric]: [Impact description]

**Files Modified:** (XX files, YYY lines)
- `path/to/file.rs`: [Description]
- ...

**Testing:**
- Total tests: XXX passing (100%)
- New tests: YY added
- Zero regressions

**Documentation:**
- Implementation summary: `/tmp/ProRT-IP/sprint-$SPRINT_ID/implementation-summary.md`
- Benchmark results: `sprint-$SPRINT_ID/benchmark-*.json`

**Result:** SUCCESS ✅ - [Brief assessment]

Phase 4: Update CLAUDE.local.md

Add session entry to "Recent Sessions" section:

### $(date +%Y-%m-%d): Sprint $SPRINT_ID - $BRIEF_SUMMARY (SUCCESS ✅)
**Objective:** [Full objective]
**Duration:** ~X hours
**Activities:**

- **[Phase Name]:** [Description]
  - [Key activity 1]
  - [Key activity 2]
  - Result: [Outcome]

- **[Phase Name]:** [Description]
  - [Key activity 1]
  - [Key activity 2]
  - Result: [Outcome]

**Deliverables:**
- XX files modified (YYY lines added, ZZZ removed = net ±AAA)
- ZZ new tests (total: XXX passing, 100%)
- Benchmark results (JSON + Markdown)
- Implementation summary (/tmp/ProRT-IP/sprint-$SPRINT_ID/)
- Documentation updates (CHANGELOG, CLAUDE.local)

**Results:**
- Performance: [Key improvement metric]
- Testing: All XXX tests passing (100% success)
- Quality: Zero clippy warnings, zero regressions
- Documentation: Comprehensive report (XKB)

**Result:** **SUCCESS ✅** - [Brief assessment of sprint success]

Update "Current Status" table metrics:
- Test count
- Total lines
- Sprint progress
- Known issues (add/remove as applicable)

Phase 5: Create Sprint Archive

Organize sprint artifacts:

# Ensure all artifacts are in sprint directory
cd /tmp/ProRT-IP/sprint-$SPRINT_ID/

# Create comprehensive README
cat > README.md << 'EOF'
# Sprint $SPRINT_ID: $BRIEF_SUMMARY

**Status:** COMPLETE ✅
**Date:** $(date +%Y-%m-%d)

## Contents

- `implementation-summary.md` - Comprehensive sprint report (XKB)
- `benchmark-*.json` - hyperfine JSON results
- `benchmark-*.md` - hyperfine markdown tables
- `test-*.txt` - Test execution outputs
- `sprint-plan.md` - Original planning document
- `task-checklist.md` - Task completion tracking
- `implementation-notes.md` - Session notes and decisions

## Quick Stats

- **Files Modified:** XX
- **Lines Changed:** +YYY -ZZZ (net: ±AAA)
- **Tests:** XXX passing (+ZZ new)
- **Performance:** ±XX% vs baseline
- **Duration:** X hours/days

## Key Achievements

- [Achievement 1]
- [Achievement 2]
- [Achievement 3]

## References

- CHANGELOG.md: Entry added
- CLAUDE.local.md: Session documented
- Git commits: [list if committed]
EOF

Phase 6: Final Sprint Report

Generate executive summary for console output:

═══════════════════════════════════════════════════════════
Sprint $SPRINT_ID Complete
═══════════════════════════════════════════════════════════

Objective: $BRIEF_SUMMARY

Status: SUCCESS ✅

Summary:
───────
Files Modified:     XX files (+YYY -ZZZ lines)
Tests:XXX passing (+ZZ new)
Performance: ±XX% vs baseline
Duration:    X hours/days
Quality:     Zero warnings, zero regressions

Key Achievements:
─────────────────
✅ [Achievement 1]
✅ [Achievement 2]
✅ [Achievement 3]

Deliverables:
─────────────
📄 Implementation summary (XKB):
   /tmp/ProRT-IP/sprint-$SPRINT_ID/implementation-summary.md

📊 Benchmark results:
   /tmp/ProRT-IP/sprint-$SPRINT_ID/benchmark-*.json
   /tmp/ProRT-IP/sprint-$SPRINT_ID/benchmark-*.md

🧪 Test outputs:
   /tmp/ProRT-IP/sprint-$SPRINT_ID/test-*.txt

📝 Documentation updates:
   - CHANGELOG.md (entry added)
   - CLAUDE.local.md (session documented)
   - README.md (if applicable)

Next Steps:
───────────
1. Review implementation-summary.md
2. Commit changes (use /stage-commit)
3. Plan next sprint (use /sprint-start)
4. Archive sprint directory

Sprint $SPRINT_ID COMPLETE - Ready for next phase!
═══════════════════════════════════════════════════════════

ERROR HANDLING

If sprint directory not found:
- Search /tmp/ProRT-IP/ for sprint-related files
- Ask user for sprint directory location
- Offer to create directory and generate docs retroactively

If benchmark files missing:
- Ask if benchmarks were run
- Offer to run benchmarks now: /bench-compare
- Document "no performance testing" in summary

If CHANGELOG.md not found:
- Search for CHANGELOG at project root
- Create if missing (with warning)
- Add entry at top under [Unreleased]

If CLAUDE.local.md not found:
- Warn about missing memory bank
- Create basic session entry in /tmp/
- Recommend proper memory bank setup

SUCCESS CRITERIA

✅ Implementation summary generated (comprehensive report)
✅ CHANGELOG.md entry added
✅ CLAUDE.local.md updated (Current Status + Recent Sessions)
✅ Sprint directory organized with README
✅ All artifacts preserved and documented
✅ Executive summary generated
✅ Clear next steps provided
✅ Zero loose ends or incomplete documentation

DELIVERABLES

1. Implementation Summary: /tmp/ProRT-IP/sprint-$SPRINT_ID/implementation-summary.md
2. CHANGELOG Entry: Added to CHANGELOG.md
3. Memory Bank Update: CLAUDE.local.md updated
4. Sprint Archive: Complete directory with README
5. Executive Summary: Console output with quick stats
6. Next Steps: Clear actionable items

Provide comprehensive documentation that allows future sessions to understand exactly what was accomplished,
how, and why.

**Usage Example:**
```bash
/sprint-complete 4.14 "Network timeout optimization (3-17x faster)"

---
5. /perf-profile - Quick Performance Profiling Setup

Purpose: Set up performance profiling with perf, flamegraph, and comprehensive analysis
File Path: .claude/commands/perf-profile.md
Estimated Complexity: Medium (5 phases)

You are a performance optimization expert working on ProRT-IP, setting up comprehensive profiling.

Execute a complete performance profiling workflow: build with debug symbols, capture CPU profile, generate
flamegraph, analyze hotspots.

**Usage:** `/perf-profile [test-command]`

**Examples:**
- `/perf-profile` - Profile default test (10K ports localhost)
- `/perf-profile -p 1-65535 127.0.0.1` - Profile full port range scan

## PROJECT CONTEXT

**ProRT-IP Performance Profiling History:**
- Sprint 4.5: perf analysis revealed SQLite futex contention (20,373 calls)
- Sprint 4.13: perf showed polling overhead (30% CPU time wasted)
- Phase 4 final: Comprehensive profiling suite (perf, flamegraph, strace, massif)
- Target hotspots: Tokio TCP ops (12.6%), packet parsing, result aggregation

**Profiling Standards:**
- perf record with dwarf call graphs (-g dwarf)
- 997 Hz sampling frequency (prime number avoids aliasing)
- Generate both text report and interactive flamegraph
- Identify functions consuming >5% of CPU time
- Check for unexpected bottlenecks (lock contention, syscalls)

**System Requirements:**
- Linux kernel with perf_events support
- Debug symbols in binary (RUSTFLAGS="-C debuginfo=2")
- flamegraph tool installed (cargo install flamegraph)
- Sufficient disk space for perf.data (~50-200MB typical)

## SYSTEMATIC APPROACH

### Phase 1: Verify Prerequisites

Check for required tools:

```bash
# Check perf availability
command -v perf >/dev/null 2>&1 || echo "ERROR: perf not installed (install: linux-tools-$(uname -r))"

# Check flamegraph availability
command -v flamegraph >/dev/null 2>&1 || echo "WARNING: flamegraph not installed (install: cargo install
flamegraph)"

# Check perf_event_paranoid setting
PARANOID=$(cat /proc/sys/kernel/perf_event_paranoid 2>/dev/null)
if [ "$PARANOID" -gt 1 ]; then
    echo "WARNING: perf_event_paranoid=$PARANOID (recommended: 1 or lower)"
    echo "Fix: sudo sysctl -w kernel.perf_event_paranoid=1"
fi

Phase 2: Build with Profiling Symbols

Create temporary Cargo config for profiling:

mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[profile.release]
debug = 2      # Full debug symbols
strip = false  # Don't strip symbols
lto = "thin"   # Thin LTO (faster build than "fat")
codegen-units = 1     # Single codegen unit for better optimization
force-frame-pointers = true  # Needed for good call graphs
EOF

# Build release binary with debug symbols
cargo build --release

# Verify binary has symbols
nm target/release/prtip | grep -q "prtip" && echo "✅ Symbols present"

Build time: ~30-60 seconds typical

Phase 3: Execute Profiling Run

Default test command if none provided: -p 1-10000 127.0.0.1

PROFILE_DIR="/tmp/ProRT-IP/profile-$(date +%s)"
mkdir -p "$PROFILE_DIR"
cd "$PROFILE_DIR"

# Record CPU profile with call graphs
sudo perf record \
    --call-graph dwarf \
    --freq 997 \
    --output perf.data \
    ../../target/release/prtip $TEST_COMMAND

# Check if recording succeeded
if [ -f perf.data ]; then
    SIZE=$(du -h perf.data | cut -f1)
    echo "✅ Profile captured: $SIZE"
else
    echo "❌ ERROR: perf record failed"
    exit 1
fi

Note: May require sudo for perf record depending on paranoid setting.

Phase 4: Generate Analysis Reports

Create multiple analysis formats:

# 1. Text report with call graph percentages
sudo perf report --stdio \
    --sort cpu,dso,symbol \
    --call-graph graph \
    > perf-report.txt

echo "✅ Text report: perf-report.txt"

# 2. Collapsed stacks for flamegraph
sudo perf script | \
    stackcollapse-perf.pl | \
    flamegraph.pl > flamegraph.svg

echo "✅ Flamegraph: flamegraph.svg"

# 3. Hardware counters summary
sudo perf stat \
    ../../target/release/prtip $TEST_COMMAND \
    2> perf-stat.txt

echo "✅ Hardware stats: perf-stat.txt"

# 4. System info for context
cat > system-info.txt << EOF
Hostname: $(hostname)
Kernel: $(uname -r)
CPU: $(lscpu | grep "Model name" | cut -d: -f2 | xargs)
Cores: $(nproc)
Memory: $(free -h | awk '/^Mem:/ {print $2}')
Date: $(date)
Command: prtip $TEST_COMMAND
EOF

echo "✅ System info: system-info.txt"

Phase 5: Analyze and Report

Parse perf report for top hotspots:

# Extract top 20 functions by CPU time
echo "Top 20 CPU Hotspots:"
echo "===================="
head -100 perf-report.txt | \
    grep -E "^\s+[0-9]+\.[0-9]+%" | \
    head -20

# Check for specific performance concerns
echo ""
echo "Performance Analysis:"
echo "===================="

# Futex contention check
FUTEX_COUNT=$(sudo perf script | grep -c "futex" || echo "0")
echo "- Futex calls: $FUTEX_COUNT (high = lock contention)"

# Tokio scheduler overhead
TOKIO_PERCENT=$(grep -o "[0-9.]*% .*tokio" perf-report.txt | head -1 | cut -d% -f1 || echo "0")
echo "- Tokio overhead: ${TOKIO_PERCENT}% (expected: 10-15%)"

# Memory allocation
ALLOC_PERCENT=$(grep -o "[0-9.]*% .*alloc" perf-report.txt | head -1 | cut -d% -f1 || echo "0")
echo "- Allocation overhead: ${ALLOC_PERCENT}% (expected: <5%)"

# System calls
SYSCALL_PERCENT=$(grep -o "[0-9.]*% .*system_call" perf-report.txt | head -1 | cut -d% -f1 || echo "0")
echo "- Syscall overhead: ${SYSCALL_PERCENT}% (expected: 10-20%)"

Create comprehensive summary:

cat > PROFILING-SUMMARY.md << 'EOF'
# Performance Profiling Summary

**Date:** $(date)
**Command:** prtip $TEST_COMMAND
**Profile Size:** $(du -h perf.data | cut -f1)

## System Configuration

- **Hostname:** $(hostname)
- **CPU:** $(lscpu | grep "Model name" | cut -d: -f2 | xargs)
- **Cores:** $(nproc)
- **Kernel:** $(uname -r)
- **Memory:** $(free -h | awk '/^Mem:/ {print $2}')

## Top CPU Hotspots

[Top 20 functions from perf-report.txt]

## Analysis

### Performance Characteristics

- **Futex Calls:** $FUTEX_COUNT (lock contention indicator)
- **Tokio Overhead:** ${TOKIO_PERCENT}% (async runtime)
- **Allocation Overhead:** ${ALLOC_PERCENT}% (memory management)
- **Syscall Overhead:** ${SYSCALL_PERCENT}% (kernel interactions)

### Hotspot Categories

1. **Async Runtime (Tokio):** XX% - [Assessment]
2. **Packet Processing:** XX% - [Assessment]
3. **Result Aggregation:** XX% - [Assessment]
4. **Network I/O:** XX% - [Assessment]
5. **Other:** XX% - [Assessment]

### Optimization Opportunities

[Based on profiling data, identify specific optimization targets]

1. **[Function/Module]:** Currently XX%, could be reduced by [technique]
2. **[Function/Module]:** Currently XX%, could be reduced by [technique]

### Comparison to Baseline

[If baseline profile exists, compare key metrics]

## Files Generated

- `perf.data` - Raw profile data (XXX MB)
- `perf-report.txt` - Text analysis report
- `flamegraph.svg` - Interactive call graph (open in browser)
- `perf-stat.txt` - Hardware counter statistics
- `system-info.txt` - System configuration
- `PROFILING-SUMMARY.md` - This summary

## Next Steps

1. Open `flamegraph.svg` in browser for interactive exploration
2. Identify functions consuming >5% CPU time
3. Review `perf-report.txt` for detailed call graphs
4. Compare to previous profiles (if available)
5. Implement optimizations for identified hotspots

## Commands Reference

```bash
# View interactive flamegraph
firefox flamegraph.svg  # or chrome, etc.

# Explore perf data interactively
sudo perf report --tui

# Re-generate specific reports
sudo perf report --stdio --no-children > flat-profile.txt

---
Profile Location: $PROFILE_DIR
EOF

echo "✅ Profiling summary: PROFILING-SUMMARY.md"

Remove temporary Cargo config:

```bash
cd ../../
rm .cargo/config.toml
rmdir .cargo 2>/dev/null || true

ERROR HANDLING

If perf not installed:
# Debian/Ubuntu
sudo apt-get install linux-tools-$(uname -r)

# Arch/CachyOS
sudo pacman -S perf

# Fedora
sudo dnf install perf

If perf_event_paranoid too restrictive:
# Temporary fix (until reboot)
sudo sysctl -w kernel.perf_event_paranoid=1

# Permanent fix
echo "kernel.perf_event_paranoid=1" | sudo tee -a /etc/sysctl.conf

If flamegraph tools missing:
cargo install flamegraph
# This installs: flamegraph, stackcollapse-perf.pl, etc.

If perf.data too large (>500MB):
- Reduce sampling frequency: --freq 99 instead of 997
- Shorten test duration
- Profile smaller test case

If no symbols in binary:
- Verify .cargo/config.toml was created
- Check: nm target/release/prtip | grep -q "prtip"
- Rebuild with cargo clean && cargo build --release

SUCCESS CRITERIA

✅ perf.data captured successfully
✅ perf-report.txt generated with call graphs
✅ flamegraph.svg created (interactive visualization)
✅ perf-stat.txt shows hardware counters
✅ Top hotspots identified (>5% CPU time)
✅ Optimization opportunities documented
✅ Comprehensive summary report generated
✅ Temporary build config cleaned up

DELIVERABLES

Profiling Artifacts

Directory: /tmp/ProRT-IP/profile-<timestamp>/

Files:
1. perf.data - Raw profiling data (50-200MB typical)
2. perf-report.txt - Text analysis with call graphs
3. flamegraph.svg - Interactive call graph visualization
4. perf-stat.txt - Hardware counter statistics
5. system-info.txt - System configuration context
6. PROFILING-SUMMARY.md - Comprehensive analysis report

Console Summary

═══════════════════════════════════════════════════════════
Performance Profiling Complete
═══════════════════════════════════════════════════════════

Profile Location: /tmp/ProRT-IP/profile-<timestamp>/

Top 5 Hotspots:
───────────────
1. XX.XX% - [function_name]
2. XX.XX% - [function_name]
3. XX.XX% - [function_name]
4. XX.XX% - [function_name]
5. XX.XX% - [function_name]

Quick Analysis:
───────────────
Futex Calls:XXX (lock contention: [HIGH/MEDIUM/LOW])
Tokio Overhead:    XX.X% (expected: 10-15%)
Allocation: XX.X% (expected: <5%)
Syscalls:   XX.X% (expected: 10-20%)

Next Steps:
───────────
1. Open flamegraph:  firefox profile-<timestamp>/flamegraph.svg
2. Review report:    less profile-<timestamp>/perf-report.txt
3. Read summary:     cat profile-<timestamp>/PROFILING-SUMMARY.md
4. Compare baseline: /bench-compare [if available]

═══════════════════════════════════════════════════════════

Provide actionable insights and specific optimization recommendations based on profiling data.

**Usage Example:**
```bash
/perf-profile
/perf-profile -p 1-10000 192.168.1.1

---
6. /module-create - Generate New Module with Tests

Purpose: Create a new Rust module with boilerplate, tests, and documentation
File Path: .claude/commands/module-create.md
Estimated Complexity: Simple (3 phases)

You are a Rust developer creating a new module for ProRT-IP with proper structure and testing.

Generate a new Rust module following ProRT-IP conventions: implementation, comprehensive tests, documentation,
 and crate integration.

**Usage:** `/module-create <crate-name> <module-name> <brief-description>`

**Examples:**
- `/module-create prtip-scanner packet_fragmentation "IDS evasion via packet fragmentation"`
- `/module-create prtip-core idle_scan "Zombie/idle scanning implementation"`
- `/module-create prtip-network ip_spoofing "IP address spoofing for decoy scanning"`

## PROJECT CONTEXT

**ProRT-IP Module Standards:**
- Comprehensive inline documentation (/// comments)
- At least 10 unit tests per module (>90% coverage target)
- Examples in doc comments (doc-tests)
- Public API with clear error types
- Integration with existing systems (scheduler, config, CLI)

**Crate Structure:**
- `prtip-core`: Core types, traits, algorithms (no I/O)
- `prtip-network`: Low-level networking (raw sockets, packet building)
- `prtip-scanner`: Scanning logic and orchestration
- `prtip-cli`: CLI interface and output formatting

**Module Examples:**
- `adaptive_parallelism.rs` - 342 lines, 17 tests (Sprint 4.4)
- `batch_sender.rs` - 656 lines, 9 tests (Enhancement Cycle 8)
- `cdn_detector.rs` - 455 lines, 12 tests (Enhancement Cycle 8)
- `decoy_scanner.rs` - 505 lines, 11 tests (Enhancement Cycle 8)

**Testing Pattern:**
- Unit tests at bottom of module (`#[cfg(test)]`)
- Test functions named `test_<functionality>`
- Use `assert_eq!`, `assert!`, `assert_matches!`
- Test happy path + edge cases + error cases

## SYSTEMATIC APPROACH

### Phase 1: Generate Module File

Create module file at `crates/$CRATE_NAME/src/$MODULE_NAME.rs`:

```rust
//! $BRIEF_DESCRIPTION
//!
//! This module provides [detailed description of functionality].
//!
//! # Examples
//!
//! ```
//! use $CRATE_NAME::$MODULE_NAME::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Example usage
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! [Describe module architecture, key components, data flow]
//!
//! # Performance Characteristics
//!
//! - Time complexity: O(?)
//! - Memory usage: [description]
//! - Concurrency: [thread-safe / not thread-safe]
//!
//! # References
//!
//! - [Reference 1]
//! - [Reference 2]

use std::fmt;
use std::error::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur in $MODULE_NAME operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ${MODULE_NAME_PASCAL}Error {
    /// [Error variant 1 description]
    ErrorVariant1(String),
    /// [Error variant 2 description]
    ErrorVariant2,
}

impl fmt::Display for ${MODULE_NAME_PASCAL}Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 match self {
     Self::ErrorVariant1(msg) => write!(f, "Error variant 1: {}", msg),
     Self::ErrorVariant2 => write!(f, "Error variant 2"),
 }
    }
}

impl Error for ${MODULE_NAME_PASCAL}Error {}

/// Convenience type alias for Results in this module
pub type Result<T> = std::result::Result<T, ${MODULE_NAME_PASCAL}Error>;

// ============================================================================
// Public Types
// ============================================================================

/// [Main struct description]
///
/// # Examples
///
/// ```
/// use $CRATE_NAME::$MODULE_NAME::*;
///
/// let instance = ${MODULE_NAME_PASCAL}::new();
/// ```
#[derive(Debug, Clone)]
pub struct ${MODULE_NAME_PASCAL} {
    // Fields
}

// ============================================================================
// Implementation
// ============================================================================

impl ${MODULE_NAME_PASCAL} {
    /// Creates a new instance
    ///
    /// # Examples
    ///
    /// ```
    /// use $CRATE_NAME::$MODULE_NAME::*;
    ///
    /// let instance = ${MODULE_NAME_PASCAL}::new();
    /// ```
    pub fn new() -> Self {
 Self {
     // Initialize fields
 }
    }

    /// [Primary functionality method]
    ///
    /// # Arguments
    ///
    /// * `param` - [Parameter description]
    ///
    /// # Returns
    ///
    /// [Return value description]
    ///
    /// # Errors
    ///
    /// Returns error if [condition].
    ///
    /// # Examples
    ///
    /// ```
    /// use $CRATE_NAME::$MODULE_NAME::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let instance = ${MODULE_NAME_PASCAL}::new();
    /// let result = instance.primary_method()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn primary_method(&self) -> Result<()> {
 // Implementation
 Ok(())
    }
}

impl Default for ${MODULE_NAME_PASCAL} {
    fn default() -> Self {
 Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
 let instance = ${MODULE_NAME_PASCAL}::new();
 // Assertions
    }

    #[test]
    fn test_primary_method_success() {
 let instance = ${MODULE_NAME_PASCAL}::new();
 let result = instance.primary_method();
 assert!(result.is_ok());
    }

    #[test]
    fn test_primary_method_error_case() {
 // Test error conditions
    }

    #[test]
    fn test_default_trait() {
 let instance = ${MODULE_NAME_PASCAL}::default();
 // Assertions
    }

    #[test]
    fn test_clone_trait() {
 let instance = ${MODULE_NAME_PASCAL}::new();
 let cloned = instance.clone();
 // Assertions
    }

    #[test]
    fn test_debug_trait() {
 let instance = ${MODULE_NAME_PASCAL}::new();
 let debug_str = format!("{:?}", instance);
 assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_error_display() {
 let error = ${MODULE_NAME_PASCAL}Error::ErrorVariant2;
 let display_str = format!("{}", error);
 assert!(!display_str.is_empty());
    }

    // Add 3-7 more domain-specific tests
    // [TODO: Implement comprehensive test cases]
}

Lines generated: ~150-200 (boilerplate + 7 initial tests)

Phase 2: Integrate into Crate

Update crates/$CRATE_NAME/src/lib.rs:

// Add module declaration (alphabetically ordered)
pub mod $MODULE_NAME;

// Add re-exports if needed
pub use $MODULE_NAME::{${MODULE_NAME_PASCAL}, ${MODULE_NAME_PASCAL}Error};

Verify crate compiles:

cargo build -p $CRATE_NAME
cargo test -p $CRATE_NAME

Phase 3: Generate Integration Guide

Create integration guide at /tmp/ProRT-IP/module-$MODULE_NAME-integration.md:

# $MODULE_NAME Integration Guide

**Created:** $(date +%Y-%m-%d)
**Crate:** $CRATE_NAME
**Module:** $MODULE_NAME
**Description:** $BRIEF_DESCRIPTION

## Module Status

- [x] Module file created: `crates/$CRATE_NAME/src/$MODULE_NAME.rs`
- [x] Exported in lib.rs
- [x] Compiles successfully
- [x] Basic tests passing (7 initial)
- [ ] Comprehensive tests (target: 10+)
- [ ] Integration with scheduler
- [ ] CLI flags added
- [ ] Config options added
- [ ] Documentation examples
- [ ] Performance benchmarks

## Next Steps

### 1. Complete Implementation (2-4 hours)

**Core Functionality:**
- [ ] Implement primary logic in `primary_method()`
- [ ] Add helper methods as needed
- [ ] Handle all error cases
- [ ] Add validation logic

**Example Pattern:**

```rust
impl ${MODULE_NAME_PASCAL} {
    pub fn core_functionality(&self, input: Input) -> Result<Output> {
 // 1. Validate input
 self.validate_input(&input)?;

 // 2. Process
 let result = self.process(input)?;

 // 3. Return
 Ok(result)
    }
}

2. Comprehensive Testing (1-2 hours)

Add Tests For:
- Happy path with typical inputs
- Edge cases (empty, max values, boundary conditions)
- Error cases (invalid input, resource exhaustion)
- Concurrency (if applicable)
- Performance characteristics
- Integration with other modules

Target: 10-15 total tests, >90% code coverage

3. Scheduler Integration (1-2 hours)

File: crates/prtip-scanner/src/scheduler.rs

Integration Points:
- Import module: use prtip_$CRATE_NAME::$MODULE_NAME::*;
- Add to ScanScheduler struct (if state needed)
- Call from appropriate scan phase
- Handle results and errors

Example:

impl ScanScheduler {
    pub async fn execute_with_$MODULE_NAME(&self) -> Result<()> {
 let module = ${MODULE_NAME_PASCAL}::new();
 let result = module.primary_method()?;
 // Handle result
 Ok(())
    }
}

4. CLI Integration (30 min - 1 hour)

File: crates/prtip-cli/src/args.rs

Add Flags:

#[derive(Parser, Debug, Clone)]
pub struct Args {
    // Existing flags...

    /// $BRIEF_DESCRIPTION
    #[arg(long = "$FLAG_NAME")]
    pub enable_$MODULE_NAME: bool,

    /// $MODULE_NAME configuration parameter
    #[arg(long = "$PARAM_NAME", default_value = "default")]
    pub ${MODULE_NAME}_param: String,
}

5. Config Integration (30 min)

File: crates/prtip-core/src/config.rs

Add Config Struct:

/// Configuration for $MODULE_NAME
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ${MODULE_NAME_PASCAL}Config {
    pub enabled: bool,
    pub param: String,
    // Additional config fields
}

impl Default for ${MODULE_NAME_PASCAL}Config {
    fn default() -> Self {
 Self {
     enabled: false,
     param: "default".to_string(),
 }
    }
}

Add to ScanConfig:

pub struct ScanConfig {
    // Existing fields...

    pub ${MODULE_NAME}_config: ${MODULE_NAME_PASCAL}Config,
}

6. Documentation (1 hour)

Update Files:
- README.md - Add to feature list with example
- docs/05-API-REFERENCE.md - Document public API
- Module-level docs - Add comprehensive examples
- CHANGELOG.md - Add unreleased entry

Example for README.md:

### $MODULE_NAME

$BRIEF_DESCRIPTION

\`\`\`bash
# Example usage
prtip --$FLAG_NAME -p 80,443 target.com
\`\`\`

7. Performance Benchmarks (optional, 1 hour)

Create Benchmark:

// benches/$MODULE_NAME_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use $CRATE_NAME::$MODULE_NAME::*;

fn benchmark_$MODULE_NAME(c: &mut Criterion) {
    c.bench_function("$MODULE_NAME", |b| {
 let instance = ${MODULE_NAME_PASCAL}::new();
 b.iter(|| instance.primary_method());
    });
}

criterion_group!(benches, benchmark_$MODULE_NAME);
criterion_main!(benches);

Testing Checklist

- cargo test -p $CRATE_NAME passes (10+ tests)
- cargo clippy -p $CRATE_NAME -- -D warnings passes
- cargo fmt --check passes
- cargo doc -p $CRATE_NAME --open shows complete API docs
- Integration tests pass if applicable
- Benchmarks run successfully (if created)

Files to Modify

1. Implementation:
  - crates/$CRATE_NAME/src/$MODULE_NAME.rs (150-500 lines expected)
  - crates/$CRATE_NAME/src/lib.rs (+2 lines)
2. Integration:
  - crates/prtip-scanner/src/scheduler.rs (+20-50 lines)
  - crates/prtip-cli/src/args.rs (+5-10 lines)
  - crates/prtip-core/src/config.rs (+15-30 lines)
3. Documentation:
  - README.md (+10-30 lines)
  - CHANGELOG.md (+5-10 lines)
  - docs/05-API-REFERENCE.md (+30-100 lines)

Estimated Effort

- Core Implementation: 2-4 hours
- Testing: 1-2 hours
- Integration: 2-3 hours
- Documentation: 1 hour
- Total: 6-10 hours

Success Criteria

✅ Module compiles without warnings
✅ 10+ comprehensive tests passing
✅ Integrated into scheduler workflow
✅ CLI flags functional
✅ Config options working
✅ Documentation complete
✅ Zero regressions in existing tests

---
Module created successfully!
Location: crates/$CRATE_NAME/src/$MODULE_NAME.rs
Next: Complete implementation following this guide

## ERROR HANDLING

**If crate name invalid:**
- Check crate exists in `crates/` directory
- Valid crates: prtip-core, prtip-network, prtip-scanner, prtip-cli
- Suggest correct crate name

**If module name conflicts:**
- Check if module already exists in lib.rs
- Suggest alternative name with suffix: `_v2`, `_new`, `_alt`

**If module file creation fails:**
- Check write permissions on crates/$CRATE_NAME/src/
- Verify directory exists
- Suggest manual creation with provided template

**If compilation fails:**
- Show cargo errors with file:line locations
- Check for missing dependencies in Cargo.toml
- Verify module syntax is correct

## SUCCESS CRITERIA

✅ Module file created with 150-200 lines boilerplate
✅ lib.rs updated with module declaration
✅ Crate compiles successfully
✅ 7 initial tests passing
✅ Integration guide generated
✅ Clear next steps provided
✅ Estimated 6-10 hour integration plan

## DELIVERABLES

1. **Module File:** `crates/$CRATE_NAME/src/$MODULE_NAME.rs`
   - Complete boilerplate (error types, main struct, implementation)
   - 7 initial tests
   - Comprehensive inline documentation

2. **Crate Integration:** `crates/$CRATE_NAME/src/lib.rs`
   - Module declaration added
   - Public exports added

3. **Integration Guide:** `/tmp/ProRT-IP/module-$MODULE_NAME-integration.md`
   - 7-step integration checklist
   - Code examples for each step
   - Testing checklist
   - Estimated effort breakdown

4. **Console Summary:**

═══════════════════════════════════════════════════════════
Module Created: $MODULE_NAME
═══════════════════════════════════════════════════════════

Crate: $CRATE_NAME
Description:  $BRIEF_DESCRIPTION

Files Created:
──────────────
✅ crates/$CRATE_NAME/src/$MODULE_NAME.rs (200 lines)
✅ Integration guide: /tmp/ProRT-IP/module-$MODULE_NAME-integration.md

Status:
───────
✅ Compiles successfully
✅ 7 initial tests passing
✅ Exported in lib.rs

Next Steps:
───────────
1. Review integration guide:
cat /tmp/ProRT-IP/module-$MODULE_NAME-integration.md
2. Complete core implementation:
  - Add logic to primary_method()
  - Implement helper methods
  - Add 3-8 more tests (target: 10-15 total)
3. Integration (6-10 hours estimated):
  - Scheduler integration (scheduler.rs)
  - CLI flags (args.rs)
  - Config options (config.rs)
  - Documentation (README.md, CHANGELOG.md)

═══════════════════════════════════════════════════════════

Provide comprehensive boilerplate following ProRT-IP conventions, ready for immediate implementation.

Usage Example:
/module-create prtip-scanner idle_scan "Zombie/idle scanning for anonymity"

---
7. /doc-update - Quick Documentation Sync

Purpose: Update README, CHANGELOG, and memory banks after changes
File Path: .claude/commands/doc-update.md
Estimated Complexity: Simple (3 phases)

You are a technical writer synchronizing ProRT-IP documentation after code changes.

Update README.md, CHANGELOG.md, and CLAUDE.local.md to reflect recent changes with proper formatting and
consistency.

**Usage:** `/doc-update <change-type> <brief-summary>`

**Change Types:** feature, fix, performance, docs, refactor, test, ci

**Examples:**
- `/doc-update feature "Added packet fragmentation support"`
- `/doc-update fix "Fixed timeout regression in large scans"`
- `/doc-update performance "Improved parallelism scaling (20-1000 concurrent)"`

## PROJECT CONTEXT

**ProRT-IP Documentation Standards:**
- **README.md:** User-facing documentation, feature examples, quick start
- **CHANGELOG.md:** Keep a Changelog format, [Unreleased] section, semantic versioning
- **CLAUDE.local.md:** Development memory bank, session summaries, current status

**CHANGELOG Format:**
```markdown
### [Type] - Description (YYYY-MM-DD)

**Changes:**
- Change 1
- Change 2

**Performance:** (if applicable)
- Metric: Before → After (±X% change)

**Files Modified:** (X files, Y lines)
- path/to/file.rs: Description

**Testing:**
- Total tests: XXX passing
- New tests: YY added

**Result:** SUCCESS ✅ / PARTIAL ⚠️ / BLOCKED ❌

Current Metrics (from CLAUDE.local.md):
- Tests: 643 passing
- Lines: 12,016+
- Crates: 4 (prtip-core, prtip-network, prtip-scanner, prtip-cli)
- Version: v0.3.0
- Phase: Phase 4 COMPLETE

SYSTEMATIC APPROACH

Phase 1: Analyze Changes

Gather information about what changed:

# Get modified files
git status --short

# Get line changes
git diff --stat

# Count current tests
cargo test --quiet 2>&1 | grep -o "[0-9]* passed" | grep -o "[0-9]*"

# Get current line count
find crates -name "*.rs" -type f -exec wc -l {} + | tail -1 | awk '{print $1}'

Parse change type and summary to determine documentation impact.

Phase 2: Update CHANGELOG.md

Add entry to [Unreleased] section (or create section if missing):

## [Unreleased]

### [$CHANGE_TYPE_CAPITALIZED] - $BRIEF_SUMMARY ($(date +%Y-%m-%d))

**Changes:**
- [Automatically detected or user-provided change list]

[If performance change:]
**Performance:**
- [Metric]: [Before] → [After] (±X% change)

**Files Modified:** (X files, Y lines)
[List of modified files with brief descriptions]

**Testing:**
- Total tests: XXX passing (100%)
- New tests: YY added (if applicable)
- Zero regressions

[If feature or fix:]
**Impact:** [User-facing impact description]

**Result:** SUCCESS ✅

Phase 3: Update README.md

Based on change type, update relevant sections:

If feature:
- Add to feature list in appropriate section
- Add usage example to relevant section (Basic/Advanced/Performance)
- Update test count badge if tests added

If performance:
- Update performance metrics in Project Status section
- Update benchmarks section if significant change

If fix:
- Remove from Known Issues section if bug fix
- Update feature description if behavior changed

Update sections:
1. Badges: Test count, version (if release)
2. Project Status: Key metrics if changed
3. Features: New capabilities
4. Usage Examples: New examples if feature added
5. Performance: Updated benchmarks if significant

Example Badge Update:
![Tests](https://img.shields.io/badge/tests-XXX_passing-success)

Phase 4: Update CLAUDE.local.md

Update "Current Status" table:
- Test count (if changed)
- Total lines (if changed)
- Sprint Progress (if sprint-related)
- Known Issues (add/remove as applicable)
- Performance metrics (if changed)

Add to "Recent Sessions" section:
### $(date +%Y-%m-%d): $CHANGE_TYPE_CAPITALIZED - $BRIEF_SUMMARY (SUCCESS ✅)
**Objective:** [Expanded objective]
**Activities:**

- **[Phase Name]:** [What was done]
  - Key change 1
  - Key change 2
  - Result: [Outcome]

**Deliverables:**
- X files modified (Y lines)
- Z new tests (if applicable)
- [Other deliverables]

**Result:** **SUCCESS ✅** - [Brief assessment]

Update metrics:
| Metric | Value | Details |
|--------|-------|---------|
| **Tests** | XXX passing (100%) | +YY from previous |
| **Total Lines** | XX,XXX+ | +YYY this session |
| ... | ... | ... |

ERROR HANDLING

If CHANGELOG.md missing [Unreleased] section:
- Create section at top of file after main header
- Add entry under new section

If README.md badges out of sync:
- Update all badges (tests, version, CI status)
- Verify badge URLs are correct

If git status shows no changes:
- Ask user if changes were committed
- Offer to update docs based on recent commits: git log -1
- Suggest using /stage-commit for comprehensive updates

If test count unavailable:
- Use previous known count from CLAUDE.local.md
- Mark as "[verify]" and run cargo test to confirm

SUCCESS CRITERIA

✅ CHANGELOG.md entry added with all required sections
✅ README.md updated in relevant sections
✅ CLAUDE.local.md Current Status table updated
✅ CLAUDE.local.md Recent Sessions entry added
✅ All metrics accurate and consistent across files
✅ Formatting follows project conventions
✅ No broken links or references

DELIVERABLES

Documentation Update Summary

═══════════════════════════════════════════════════════════
Documentation Updated
═══════════════════════════════════════════════════════════

Change Type: $CHANGE_TYPE
Summary:     $BRIEF_SUMMARY

Files Updated:
──────────────
✅ CHANGELOG.md
   - Entry added to [Unreleased]
   - X files modified, Y lines changed documented

✅ README.md
   - [Sections updated list]
   - Test count badge: XXX passing
   - [Other updates]

✅ CLAUDE.local.md
   - Current Status table updated
   - Session entry added to Recent Sessions
   - Metrics synchronized

Current Metrics:
────────────────
Tests: XXX passing (+YY)
Lines: XX,XXX+ (+YYY)
Files: X modified
Version:      v0.3.0 (or next)

Next Steps:
───────────
1. Review updates: git diff README.md CHANGELOG.md CLAUDE.local.md
2. Verify accuracy of metrics
3. Commit changes: /stage-commit "docs: $BRIEF_SUMMARY"

═══════════════════════════════════════════════════════════

Provide clear summary of all documentation updates with verification steps.

**Usage Example:**
```bash
/doc-update feature "Added idle/zombie scanning support"
/doc-update fix "Fixed race condition in result aggregator"
/doc-update performance "Optimized packet batching (30% faster)"

---
8. /test-quick - Fast Targeted Test Execution

Purpose: Run specific tests or test patterns quickly without full suite
File Path: .claude/commands/test-quick.md
Estimated Complexity: Simple (2 phases)

You are a Rust developer executing targeted tests for rapid feedback during ProRT-IP development.

Run specific tests, test modules, or test patterns without executing the full 643-test suite for faster
iteration.

**Usage:** `/test-quick [pattern] [options]`

**Examples:**
- `/test-quick` - Run tests in current package only
- `/test-quick tcp_connect` - Run all tests matching "tcp_connect"
- `/test-quick test_syn_scanner --nocapture` - Run specific test with output
- `/test-quick scheduler --package prtip-scanner` - Run module tests in specific package

## PROJECT CONTEXT

**ProRT-IP Test Suite:**
- **Total Tests:** 643 across 4 crates
- **Test Distribution:**
  - prtip-core: ~180 tests (types, config, algorithms)
  - prtip-network: ~120 tests (packet building, raw sockets)
  - prtip-scanner: ~280 tests (scanning logic, scheduler)
  - prtip-cli: ~63 tests (CLI parsing, output)
- **Full Suite Duration:** 5-6 minutes (322s typical)
- **Package-Specific Duration:** 30-90 seconds typical

**Test Categories:**
- Unit tests: Individual function/method tests
- Integration tests: Multi-component workflows
- Doc tests: Examples in documentation
- Platform-specific: Conditional compilation (#[cfg(target_os)])

**Common Test Patterns:**
- `test_*` - Standard test naming
- `*_scanner` - Scanner implementation tests
- `*_config` - Configuration parsing tests
- `*_integration` - Integration workflow tests
- `*_error` - Error handling tests

## SYSTEMATIC APPROACH

### Phase 1: Determine Test Scope

Parse arguments and build cargo test command:

```bash
# Default: Test current package only
if [ -z "$PATTERN" ]; then
    # Detect current package from pwd
    if [[ "$PWD" == *"prtip-core"* ]]; then
 PACKAGE="prtip-core"
    elif [[ "$PWD" == *"prtip-network"* ]]; then
 PACKAGE="prtip-network"
    elif [[ "$PWD" == *"prtip-scanner"* ]]; then
 PACKAGE="prtip-scanner"
    elif [[ "$PWD" == *"prtip-cli"* ]]; then
 PACKAGE="prtip-cli"
    else
 # Default: Run all tests if at workspace root
 PACKAGE=""
    fi
    PATTERN=""
else
    # User provided pattern
    PACKAGE="${PACKAGE:-}"  # Use --package flag if provided
fi

# Build command
CMD="cargo test"
[ -n "$PACKAGE" ] && CMD="$CMD --package $PACKAGE"
[ -n "$PATTERN" ] && CMD="$CMD $PATTERN"
[ -n "$OPTIONS" ] && CMD="$CMD -- $OPTIONS"

Phase 2: Execute Tests and Report

Run tests with timing:

echo "Running: $CMD"
echo "─────────────────────────────────────────────────"

START_TIME=$(date +%s)

# Run tests
$CMD 2>&1 | tee /tmp/test-quick-output.txt

EXIT_CODE=${PIPESTATUS[0]}
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Parse results
PASSED=$(grep -o "[0-9]* passed" /tmp/test-quick-output.txt | grep -o "[0-9]*" | head -1 || echo "0")
FAILED=$(grep -o "[0-9]* failed" /tmp/test-quick-output.txt | grep -o "[0-9]*" | head -1 || echo "0")
IGNORED=$(grep -o "[0-9]* ignored" /tmp/test-quick-output.txt | grep -o "[0-9]*" | head -1 || echo "0")

# Generate summary
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "Test Results Summary"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Command:  $CMD"
echo "Duration: ${DURATION}s"
echo ""
echo "Results:"
echo "  ✅ Passed:  $PASSED"
[ "$FAILED" -gt 0 ] && echo "  ❌ Failed:  $FAILED"
[ "$IGNORED" -gt 0 ] && echo "  ⏭️  Ignored: $IGNORED"
echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo "Status: ✅ ALL TESTS PASSED"

    # Performance hint
    if [ $DURATION -gt 60 ]; then
 FULL_SUITE_ESTIMATE=$((DURATION * 643 / PASSED))
 echo ""
 echo "💡 Hint: Full suite would take ~${FULL_SUITE_ESTIMATE}s"
 echo "   Consider using more specific patterns for faster iteration"
    fi
else
    echo "Status: ❌ SOME TESTS FAILED"

    # Show failed test names
    echo ""
    echo "Failed Tests:"
    grep "^test.*FAILED$" /tmp/test-quick-output.txt | sed 's/^/  - /'

    # Suggest next steps
    echo ""
    echo "Next Steps:"
    echo "  1. Review failure details above"
    echo "  2. Run specific test: /test-quick <test_name> --nocapture"
    echo "  3. Fix issues and re-run"
fi

echo "═══════════════════════════════════════════════════════════"

COMMON TEST PATTERNS

By Component:
/test-quick tcp_connect  # All TCP connect scanner tests
/test-quick syn_scanner  # All SYN scanner tests
/test-quick scheduler    # All scheduler tests
/test-quick config# All config tests
/test-quick progress     # All progress bar tests

By Test Type:
/test-quick integration  # All integration tests
/test-quick error # All error handling tests
/test-quick validation   # All input validation tests

By Package:
/test-quick --package prtip-core# All core tests (~180)
/test-quick --package prtip-network    # All network tests (~120)
/test-quick --package prtip-scanner    # All scanner tests (~280)
/test-quick --package prtip-cli # All CLI tests (~63)

Specific Tests:
/test-quick test_scan_common_ports --nocapture     # Single test with output
/test-quick test_adaptive_parallelism::test_# All tests in module

With Options:
/test-quick tcp --nocapture     # Show println! output
/test-quick scheduler --show-output    # Show all output
/test-quick config --ignored    # Run ignored tests
/test-quick -- --test-threads=1 # Single-threaded execution

ERROR HANDLING

If no tests match pattern:
No tests matched pattern: "$PATTERN"

Suggestions:
- Check spelling: cargo test --list | grep -i "$PATTERN"
- Try broader pattern: /test-quick ${PATTERN%_*}
- List all tests: cargo test --list

If compilation fails:
Compilation failed. Common fixes:

1. Check for syntax errors:
   cargo check

2. Run clippy for detailed errors:
   cargo clippy

3. Fix formatting first:
   cargo fmt

If tests timeout:
Tests timed out. Options:

1. Run single-threaded:
   /test-quick $PATTERN -- --test-threads=1

2. Increase timeout (Windows):
   /test-quick $PATTERN -- --test-timeout=10

3. Run specific test only:
   /test-quick test_specific_name

SUCCESS CRITERIA

✅ Tests execute faster than full suite
✅ Clear pass/fail results displayed
✅ Duration reported for performance tracking
✅ Failed test names shown (if any)
✅ Actionable next steps provided

DELIVERABLES

Quick Test Summary

Provide console output matching this format:

Running: cargo test --package prtip-scanner scheduler
─────────────────────────────────────────────────────

[Test execution output]

═══════════════════════════════════════════════════════════
Test Results Summary
═══════════════════════════════════════════════════════════

Command:  cargo test --package prtip-scanner scheduler
Duration: 12s

Results:
  ✅ Passed:  23

Status: ✅ ALL TESTS PASSED

💡 Hint: Full suite would take ~332s
   Consider using more specific patterns for faster iteration

═══════════════════════════════════════════════════════════

Tips for Efficient Testing:

1. Develop in package directory:
  - cd crates/prtip-scanner
  - /test-quick runs only that package (~280 tests, ~60s)
2. Test specific module during development:
  - Working on scheduler? /test-quick scheduler
  - Only ~15-20 tests, ~5-10s
3. Use --nocapture for debugging:
  - /test-quick test_failing --nocapture
  - See all println! output
4. Run full suite before commit:
  - Final validation: cargo test (full 643 tests)
  - Or use /rust-check for complete quality pipeline
5. Pattern matching tips:
  - Partial match: /test-quick tcp matches all TCP tests
  - Exact match: /test-quick test_tcp_connect_localhost
  - Module match: /test-quick scheduler::test_

Optimize for <30 second test cycles during active development, reserve full suite for final validation.

**Usage Example:**
```bash
/test-quick
/test-quick tcp_connect
/test-quick test_scheduler_basic --nocapture
/test-quick --package prtip-scanner

---
9. /ci-status - Check CI/CD Pipeline Status

Purpose: Check GitHub Actions CI/CD status and recent workflow runs
File Path: .claude/commands/ci-status.md
Estimated Complexity: Simple (2 phases)

You are a DevOps engineer monitoring ProRT-IP's CI/CD infrastructure on GitHub Actions.

Check the status of CI/CD workflows, recent runs, and provide actionable insights about pipeline health.

**Usage:** `/ci-status [workflow-name]`

**Examples:**
- `/ci-status` - Check all workflows
- `/ci-status ci` - Check CI workflow specifically
- `/ci-status release` - Check release workflow

## PROJECT CONTEXT

**ProRT-IP CI/CD Workflows:**
1. **ci.yml** - Main CI pipeline (7 jobs)
   - Format check (cargo fmt)
   - Clippy lint (cargo clippy)
   - Tests (Linux/Windows/macOS)
   - MSRV check (Rust 1.70+)
   - Security audit (cargo audit)

2. **release.yml** - Release builds (9 targets)
   - Multi-platform binary builds
   - Automated GitHub releases
   - Asset uploads (tar.gz, zip)

3. **dependency-review.yml** - PR dependency scanning
4. **codeql.yml** - Weekly security analysis

**Repository:**
- Owner: doublegate
- Repo: ProRT-IP
- Branch: main
- GitHub: https://github.com/doublegate/ProRT-IP

**CI Standards:**
- All 7 CI jobs must pass before merge
- Zero tolerance for formatting/clippy violations
- Multi-platform test validation required
- Security audit must pass (no high/critical vulnerabilities)

## SYSTEMATIC APPROACH

### Phase 1: Check Workflow Status

Use GitHub CLI to query workflow runs:

```bash
# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) not installed"
    echo ""
    echo "Install:"
    echo "  - Debian/Ubuntu: sudo apt install gh"
    echo "  - Arch/CachyOS: sudo pacman -S github-cli"
    echo "  - macOS: brew install gh"
    echo ""
    echo "Fallback: Check manually at:"
    echo "  https://github.com/doublegate/ProRT-IP/actions"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "⚠️  Not authenticated with GitHub"
    echo ""
    echo "Authenticate:"
    echo "  gh auth login"
    echo ""
    echo "Fallback: Check manually at:"
    echo "  https://github.com/doublegate/ProRT-IP/actions"
    exit 1
fi

# Query workflow runs
if [ -n "$WORKFLOW_NAME" ]; then
    # Specific workflow
    RUNS=$(gh run list --workflow "$WORKFLOW_NAME.yml" --limit 5 --json
status,conclusion,name,headBranch,createdAt,url)
else
    # All workflows
    RUNS=$(gh run list --limit 10 --json status,conclusion,name,headBranch,createdAt,url)
fi

Phase 2: Parse and Display Results

Parse JSON output and generate formatted report:

echo "═══════════════════════════════════════════════════════════"
echo "ProRT-IP CI/CD Status"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Repository: doublegate/ProRT-IP"
echo "Branch:     main"
echo "Checked:    $(date)"
echo ""

# Parse runs and display
echo "$RUNS" | jq -r '.[] |
    if .status == "completed" then
 if .conclusion == "success" then "✅"
 elif .conclusion == "failure" then "❌"
 elif .conclusion == "cancelled" then "⏹️ "
 else "⚠️ " end
    else "🔄" end +
    " " + .name +
    " (" + .headBranch + ")" +
    " - " + (.createdAt | fromdate | strftime("%Y-%m-%d %H:%M")) +
    "\n   " + .url'

echo ""

# Summary statistics
TOTAL=$(echo "$RUNS" | jq 'length')
SUCCESS=$(echo "$RUNS" | jq '[.[] | select(.conclusion == "success")] | length')
FAILURE=$(echo "$RUNS" | jq '[.[] | select(.conclusion == "failure")] | length')
IN_PROGRESS=$(echo "$RUNS" | jq '[.[] | select(.status != "completed")] | length')

echo "─────────────────────────────────────────────────────"
echo "Summary (last $TOTAL runs):"
echo "  ✅ Success:     $SUCCESS"
[ "$FAILURE" -gt 0 ] && echo "  ❌ Failed:      $FAILURE"
[ "$IN_PROGRESS" -gt 0 ] && echo "  🔄 In Progress: $IN_PROGRESS"
echo ""

# Overall health assessment
if [ "$FAILURE" -eq 0 ]; then
    echo "Overall Health: ✅ HEALTHY"
    echo ""
    echo "All recent workflows passing. CI/CD pipeline operational."
else
    echo "Overall Health: ⚠️  ISSUES DETECTED"
    echo ""
    echo "Recent workflow failures detected. Investigation recommended."
    echo ""
    echo "View failures:"
    echo "  gh run list --status failure --limit 5"
    echo ""
    echo "View specific run:"
    echo "  gh run view <run-id>"
fi

echo "═══════════════════════════════════════════════════════════"

Advanced: Workflow-Specific Details

If specific workflow requested, show job details:

if [ -n "$WORKFLOW_NAME" ]; then
    echo ""
    echo "Detailed Job Status for: $WORKFLOW_NAME"
    echo "─────────────────────────────────────────────────────"

    # Get latest run
    LATEST_RUN=$(gh run list --workflow "$WORKFLOW_NAME.yml" --limit 1 --json databaseId --jq
'.[0].databaseId')

    if [ -n "$LATEST_RUN" ]; then
 # Show job details
 gh run view "$LATEST_RUN" --log-failed

 echo ""
 echo "Full logs:"
 echo "  gh run view $LATEST_RUN --log"
    fi
fi

WORKFLOW-SPECIFIC CHECKS

CI Workflow (ci.yml):
# Check individual job status
gh run view <run-id> --json jobs --jq '.jobs[] | .name + ": " + .conclusion'

Expected jobs:
- format: success
- clippy: success
- test-linux: success
- test-windows: success
- test-macos: success
- msrv: success
- security-audit: success

Release Workflow (release.yml):
# Check build artifacts
gh run view <run-id> --json jobs --jq '.jobs[] | select(.name | contains("build")) | .name + ": " +
.conclusion'

Expected builds (5 working):
- Linux x86_64 (glibc): success
- Windows x86_64: success
- macOS x86_64 (Intel): success
- macOS aarch64 (Apple Silicon): success
- FreeBSD x86_64: success

ERROR HANDLING

If gh CLI not installed:
GitHub CLI (gh) not found. Install it:

**Debian/Ubuntu:**
sudo apt install gh

**Arch/CachyOS:**
sudo pacman -S github-cli

**macOS:**
brew install gh

**Fallback:**
Check CI status manually at:
https://github.com/doublegate/ProRT-IP/actions

If not authenticated:
GitHub authentication required.

Authenticate with:
gh auth login

Follow prompts to authenticate with GitHub.

**Fallback:**
Check CI status manually at:
https://github.com/doublegate/ProRT-IP/actions

If workflow not found:
Workflow "$WORKFLOW_NAME" not found.

Available workflows:
- ci
- release
- dependency-review
- codeql

List all workflows:
gh workflow list

SUCCESS CRITERIA

✅ Workflow status retrieved successfully
✅ Recent runs displayed with status icons
✅ Summary statistics calculated
✅ Overall health assessment provided
✅ Actionable next steps for failures
✅ Direct links to GitHub Actions

DELIVERABLES

CI Status Report

═══════════════════════════════════════════════════════════
ProRT-IP CI/CD Status
═══════════════════════════════════════════════════════════

Repository: doublegate/ProRT-IP
Branch:     main
Checked:    2025-10-11 14:30:00

✅ CI (main) - 2025-10-11 14:25
   https://github.com/doublegate/ProRT-IP/actions/runs/123456

✅ CI (main) - 2025-10-11 12:10
   https://github.com/doublegate/ProRT-IP/actions/runs/123455

🔄 Release (v0.3.1) - 2025-10-11 14:20 (in progress)
   https://github.com/doublegate/ProRT-IP/actions/runs/123457

✅ Security Analysis (main) - 2025-10-10 00:00
   https://github.com/doublegate/ProRT-IP/actions/runs/123450

─────────────────────────────────────────────────────
Summary (last 10 runs):
  ✅ Success:     9
  🔄 In Progress: 1

Overall Health: ✅ HEALTHY

All recent workflows passing. CI/CD pipeline operational.

═══════════════════════════════════════════════════════════

Quick Actions:

# View latest CI run
gh run view

# View failed runs
gh run list --status failure

# Watch current run
gh run watch

# Re-run failed workflows
gh run rerun <run-id>

# View workflow logs
gh run view <run-id> --log

# Check specific workflow
/ci-status ci
/ci-status release

Common Failure Scenarios:

1. Format Check Failed:
  - Fix: cargo fmt
  - Verify: cargo fmt --check
2. Clippy Warnings:
  - Fix: Address warnings shown in logs
  - Verify: cargo clippy -- -D warnings
3. Test Failures:
  - Platform-specific: Check Windows timing tests
  - Fix: Update test tolerances
  - Verify: /test-quick <test-name>
4. Security Audit Failed:
  - Check: cargo audit
  - Fix: Update vulnerable dependencies
  - Verify: cargo update && cargo audit
5. MSRV Check Failed:
  - Check: Rust version requirement
  - Fix: Use MSRV-compatible syntax
  - Verify: cargo +1.70 check

Provide comprehensive CI/CD health monitoring with actionable remediation steps.

**Usage Example:**
```bash
/ci-status
/ci-status ci
/ci-status release

---
10. /bug-report - Generate Comprehensive Bug Report

Purpose: Create detailed bug report with reproduction steps and system context
File Path: .claude/commands/bug-report.md
Estimated Complexity: Medium (4 phases)

You are a QA engineer creating a comprehensive bug report for ProRT-IP with all necessary context for
reproduction and fixing.

Generate a detailed bug report including system information, reproduction steps, expected vs actual behavior,
and relevant logs.

**Usage:** `/bug-report <brief-description>`

**Examples:**
- `/bug-report "Progress bar starts at 100% on fast scans"`
- `/bug-report "Timeout hangs on filtered ports"`
- `/bug-report "Service detection returns empty results"`

## PROJECT CONTEXT

**ProRT-IP Bug Report Standards:**
- Complete system information (OS, kernel, CPU, Rust version)
- Exact command that reproduces the issue
- Expected behavior vs actual behavior
- Relevant logs (with --verbose if applicable)
- Impact assessment (critical/high/medium/low)
- Suggested fix or investigation direction (if known)

**Bug Report Categories:**
1. **Functionality:** Feature not working as designed
2. **Performance:** Slower than expected or regression
3. **Correctness:** Incorrect results or false positives/negatives
4. **Usability:** Confusing behavior or poor UX
5. **Compatibility:** Platform-specific issues

**Recent Bug Patterns:**
- Progress bar display issues (Sprint 4.12)
- Performance regressions (Sprint 4.13: polling overhead)
- Network timeout behavior (Sprint 4.14)
- Service detection integration (Sprint 4.11)

## SYSTEMATIC APPROACH

### Phase 1: Gather System Information

Collect comprehensive system context:

```bash
BUG_DIR="/tmp/ProRT-IP/bug-report-$(date +%s)"
mkdir -p "$BUG_DIR"

# System information
cat > "$BUG_DIR/system-info.txt" << EOF
System Information
==================

Hostname:     $(hostname)
OS:    $(uname -s)
Kernel:$(uname -r)
Architecture: $(uname -m)
CPU:   $(lscpu 2>/dev/null | grep "Model name" | cut -d: -f2 | xargs || echo "N/A")
Cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "N/A")
Memory:$(free -h 2>/dev/null | awk '/^Mem:/ {print $2}' || sysctl -n hw.memsize 2>/dev/null | awk
'{print $1/1024/1024/1024 "GB"}' || echo "N/A")

Software Versions
=================

Rust:  $(rustc --version)
Cargo: $(cargo --version)
ProRT-IP:     $(cargo metadata --no-deps 2>/dev/null | jq -r '.packages[] | select(.name == "prtip-cli") |
.version' || echo "Unknown")

Git Information
===============

Branch:$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")
Commit:$(git rev-parse --short HEAD 2>/dev/null || echo "N/A")
Status:$(git status --short 2>/dev/null | wc -l | xargs echo "modified files:" || echo "N/A")

Build Information
=================

Profile:      $(cargo metadata --format-version 1 2>/dev/null | jq -r '.target_directory' | xargs -I{} sh -c
'if [ -f {}/release/prtip ]; then echo "release"; else echo "debug"; fi' || echo "Unknown")
Binary:$(which prtip 2>/dev/null || echo "./target/release/prtip")
Binary Size:  $(ls -lh $(which prtip 2>/dev/null || echo "./target/release/prtip") 2>/dev/null | awk '{print
$5}' || echo "N/A")

Date:  $(date)
EOF

echo "✅ System info collected: $BUG_DIR/system-info.txt"

Phase 2: Reproduce Bug and Capture Output

Execute reproduction command and capture all output:

# Prompt user for reproduction command
echo "Enter exact command to reproduce bug (or press Enter to skip):"
read -r REPRO_COMMAND

if [ -n "$REPRO_COMMAND" ]; then
    echo "Executing: $REPRO_COMMAND"
    echo ""

    # Run with verbose output
    echo "Running with standard output..."
    $REPRO_COMMAND > "$BUG_DIR/bug-output.txt" 2>&1
    EXIT_CODE=$?

    echo "✅ Output captured: $BUG_DIR/bug-output.txt"
    echo "   Exit code: $EXIT_CODE"

    # Run with verbose flag if available
    if [[ "$REPRO_COMMAND" != *"--verbose"* ]]; then
 echo ""
 echo "Running with --verbose flag..."
 $REPRO_COMMAND --verbose > "$BUG_DIR/bug-output-verbose.txt" 2>&1
 echo "✅ Verbose output captured: $BUG_DIR/bug-output-verbose.txt"
    fi

    # Run with debug logging
    echo ""
    echo "Running with RUST_LOG=debug..."
    RUST_LOG=debug $REPRO_COMMAND > "$BUG_DIR/bug-output-debug.txt" 2>&1
    echo "✅ Debug output captured: $BUG_DIR/bug-output-debug.txt"
else
    echo "⚠️  No reproduction command provided"
    echo "   Manual reproduction steps should be detailed in bug report"
fi

Phase 3: Analyze and Categorize

Analyze output to determine bug characteristics:

# Determine bug category based on description and output
CATEGORY="Unknown"
SEVERITY="Unknown"

# Simple heuristic categorization
if [[ "$BRIEF_DESCRIPTION" =~ "crash"||"panic"||"segfault" ]]; then
    CATEGORY="Stability"
    SEVERITY="Critical"
elif [[ "$BRIEF_DESCRIPTION" =~ "slow"||"hang"||"timeout"||"performance" ]]; then
    CATEGORY="Performance"
    SEVERITY="High"
elif [[ "$BRIEF_DESCRIPTION" =~ "wrong"||"incorrect"||"false"||"missing" ]]; then
    CATEGORY="Correctness"
    SEVERITY="High"
elif [[ "$BRIEF_DESCRIPTION" =~ "display"||"output"||"format"||"progress" ]]; then
    CATEGORY="Usability"
    SEVERITY="Medium"
fi

# Check for stack traces or panics
if grep -q "thread.*panicked" "$BUG_DIR/bug-output.txt" 2>/dev/null; then
    SEVERITY="Critical"
    PANIC_INFO=$(grep -A 5 "thread.*panicked" "$BUG_DIR/bug-output.txt")
fi

# Check for error messages
if grep -q "ERROR" "$BUG_DIR/bug-output.txt" 2>/dev/null; then
    ERROR_INFO=$(grep "ERROR" "$BUG_DIR/bug-output.txt")
fi

Phase 4: Generate Comprehensive Bug Report

Create detailed markdown report:

cat > "$BUG_DIR/BUG-REPORT.md" << EOF
# Bug Report: $BRIEF_DESCRIPTION

**Date:** $(date +%Y-%m-%d)
**Reporter:** [Auto-generated]
**Category:** $CATEGORY
**Severity:** $SEVERITY

## Summary

$BRIEF_DESCRIPTION

## System Information

\`\`\`
$(cat "$BUG_DIR/system-info.txt")
\`\`\`

## Reproduction Steps

### Command

\`\`\`bash
$REPRO_COMMAND
\`\`\`

### Steps to Reproduce

1. [Step 1]
2. [Step 2]
3. [Step 3]

### Frequency

- [ ] Always (100%)
- [ ] Often (>50%)
- [ ] Sometimes (<50%)
- [ ] Rarely (<10%)

### Platform Specificity

- [ ] Linux only
- [ ] Windows only
- [ ] macOS only
- [ ] Cross-platform
- [ ] Unknown

## Expected Behavior

[Describe what should happen]

## Actual Behavior

[Describe what actually happens]

## Logs and Output

### Standard Output

\`\`\`
$(head -100 "$BUG_DIR/bug-output.txt" 2>/dev/null || echo "No output captured")
\`\`\`

$(if [ -f "$BUG_DIR/bug-output-verbose.txt" ]; then
    echo "### Verbose Output"
    echo ""
    echo "\`\`\`"
    head -100 "$BUG_DIR/bug-output-verbose.txt"
    echo "\`\`\`"
fi)

$(if [ -f "$BUG_DIR/bug-output-debug.txt" ]; then
    echo "### Debug Output"
    echo ""
    echo "\`\`\`"
    head -100 "$BUG_DIR/bug-output-debug.txt"
    echo "\`\`\`"
fi)

$(if [ -n "$PANIC_INFO" ]; then
    echo "### Panic Information"
    echo ""
    echo "\`\`\`"
    echo "$PANIC_INFO"
    echo "\`\`\`"
fi)

$(if [ -n "$ERROR_INFO" ]; then
    echo "### Error Messages"
    echo ""
    echo "\`\`\`"
    echo "$ERROR_INFO"
    echo "\`\`\`"
fi)

## Impact Assessment

**User Impact:**
- [ ] Blocker: Cannot use application
- [ ] Major: Core functionality broken
- [ ] Minor: Workaround available
- [ ] Trivial: Cosmetic issue

**Scope:**
- [ ] All users affected
- [ ] Specific use case only
- [ ] Edge case only

**Urgency:**
- [ ] Critical: Fix immediately
- [ ] High: Fix in current sprint
- [ ] Medium: Fix in next sprint
- [ ] Low: Fix when convenient

## Investigation Notes

### Relevant Code Areas

[Based on symptoms, suggest which modules might be involved]

- \`crates/prtip-scanner/src/scheduler.rs\` - Main scanning orchestration
- \`crates/prtip-scanner/src/progress_bar.rs\` - Progress display
- \`crates/prtip-network/src/tcp_connect.rs\` - TCP connection logic
- [Add more based on bug description]

### Potential Root Causes

1. **[Hypothesis 1]:** [Explanation]
2. **[Hypothesis 2]:** [Explanation]
3. **[Hypothesis 3]:** [Explanation]

### Suggested Investigation Steps

1. [ ] Add debug logging to [module]
2. [ ] Profile with perf to identify bottleneck
3. [ ] Review recent changes to [file]
4. [ ] Test with different configurations
5. [ ] Compare with previous version (regression test)

### Similar Issues

[Reference any similar bugs from bug_fix/ directory or CLAUDE.local.md]

## Workarounds

**Available Workarounds:**
- [Workaround 1, if any]
- [Workaround 2, if any]

**None available** (check this if no workarounds)

## Proposed Fix

[If root cause suspected, describe potential fix approach]

### Files to Modify

- \`path/to/file.rs\`: [Description of changes]

### Testing Plan

1. [ ] Add regression test for this specific bug
2. [ ] Verify fix with reproduction steps
3. [ ] Run full test suite (643 tests)
4. [ ] Test on all platforms (Linux/Windows/macOS)
5. [ ] Performance benchmark (if performance-related)

## Additional Context

### Related Documentation

- CLAUDE.local.md: [Reference relevant session if applicable]
- bug_fix/: [Reference similar fixes if applicable]
- docs/: [Reference architectural docs if relevant]

### Environment Variables

\`\`\`bash
# Relevant environment variables (if any)
RUST_LOG=debug
RUST_BACKTRACE=1
# [Add others]
\`\`\`

### Test Case for Regression

\`\`\`rust
#[test]
fn test_bug_${BRIEF_DESCRIPTION//[ -]/_}() {
    // Regression test for: $BRIEF_DESCRIPTION
    // Reproduction steps:
    // 1. [Step]
    // 2. [Step]
    // Expected: [Expected behavior]

    // [Test implementation]
}
\`\`\`

## Attachments

- System info: \`$BUG_DIR/system-info.txt\`
- Standard output: \`$BUG_DIR/bug-output.txt\`
$([ -f "$BUG_DIR/bug-output-verbose.txt" ] && echo "- Verbose output: \`$BUG_DIR/bug-output-verbose.txt\`")
$([ -f "$BUG_DIR/bug-output-debug.txt" ] && echo "- Debug output: \`$BUG_DIR/bug-output-debug.txt\`")

## Status

- [ ] Reported
- [ ] Triaged
- [ ] In Progress
- [ ] Fixed
- [ ] Verified
- [ ] Closed

---

**Bug Report Location:** \`$BUG_DIR/BUG-REPORT.md\`
**Generated:** $(date)
EOF

echo "✅ Bug report generated: $BUG_DIR/BUG-REPORT.md"

Create summary for console:

cat << EOF

═══════════════════════════════════════════════════════════
Bug Report Created
═══════════════════════════════════════════════════════════

Bug:      $BRIEF_DESCRIPTION
Category: $CATEGORY
Severity: $SEVERITY

Location: $BUG_DIR/

Files Generated:
────────────────
📄 BUG-REPORT.md   - Comprehensive bug report
📋 system-info.txt - System configuration
📝 bug-output.txt  - Standard output
$([ -f "$BUG_DIR/bug-output-verbose.txt" ] && echo "📝 bug-output-verbose.txt - Verbose output")
$([ -f "$BUG_DIR/bug-output-debug.txt" ] && echo "📝 bug-output-debug.txt   - Debug output")

Next Steps:
───────────
1. Review bug report:
   cat $BUG_DIR/BUG-REPORT.md

2. Complete manual sections:
   - Expected/Actual Behavior
   - Frequency checkboxes
   - Impact assessment checkboxes

3. Add to bug tracking:
   - Copy to bug_fix/ directory if starting fix
   - Create GitHub issue if public repo
   - Reference in CLAUDE.local.md if internal tracking

4. Begin investigation:
   - Review suggested code areas
   - Test hypotheses
   - Add debug logging

═══════════════════════════════════════════════════════════
EOF

ERROR HANDLING

If reproduction command fails:
- Capture exit code and stderr
- Note in bug report as "command execution failed"
- Suggest trying with different parameters

If system info unavailable:
- Mark as "N/A" in report
- Include manual collection instructions
- Proceed with partial information

If logs too large (>1MB):
- Truncate to first 1000 lines
- Note truncation in report
- Suggest using --output flag for full logs

SUCCESS CRITERIA

✅ System information collected comprehensively
✅ Reproduction command executed and logged
✅ Bug categorized (category + severity)
✅ Comprehensive bug report generated
✅ All relevant logs captured
✅ Investigation suggestions provided
✅ Clear next steps documented

DELIVERABLES

Bug Report Package

Directory: /tmp/ProRT-IP/bug-report-<timestamp>/

Files:
1. BUG-REPORT.md - Comprehensive markdown report (primary deliverable)
2. system-info.txt - System configuration details
3. bug-output.txt - Standard execution output
4. bug-output-verbose.txt - Verbose flag output (if captured)
5. bug-output-debug.txt - Debug logging output (RUST_LOG=debug)

Report Sections:
- Summary
- System Information
- Reproduction Steps
- Expected vs Actual Behavior
- Logs and Output (multiple sections)
- Impact Assessment (checkboxes)
- Investigation Notes (code areas, hypotheses, steps)
- Workarounds
- Proposed Fix
- Additional Context
- Regression Test Template
- Attachments List
- Status Tracking

Provide actionable bug report ready for immediate investigation or GitHub issue creation.

**Usage Example:**
```bash
/bug-report "Progress bar shows 100% immediately on fast localhost scans"
/bug-report "Scanner hangs after 10K ports on network targets"
/bug-report "Service detection returns empty results for all services"

---
Summary of Generated Commands (10 Total)

| #   | Command   | Purpose     | Complexity | Priority |
|-----|------------------|-------------------------------------------------------|------------|----------|
| 1   | /rust-check      | Fast Rust quality pipeline (fmt, clippy, test, build) | Simple (3) | HIGH     |
| 2   | /bench-compare   | Performance comparison between git refs | Medium (5) | HIGH     |
| 3   | /sprint-start    | Initialize new sprint with planning     | Medium (6) | HIGH     |
| 4   | /sprint-complete | Finalize sprint with documentation      | Medium (6) | HIGH     |
| 5   | /perf-profile    | Quick performance profiling setup| Medium (5) | MEDIUM   |
| 6   | /module-create   | Generate new module with tests   | Simple (3) | MEDIUM   |
| 7   | /doc-update      | Quick documentation sync  | Simple (3) | MEDIUM   |
| 8   | /test-quick      | Fast targeted test execution     | Simple (2) | HIGH     |
| 9   | /ci-status| Check CI/CD pipeline status      | Simple (2) | MEDIUM   |
| 10  | /bug-report      | Generate comprehensive bug report| Medium (4) | HIGH     |

---
Implementation Priority

Tier 1 - Immediate Value (Create First)

1. /rust-check - Most frequently needed, replaces manual quality checks
2. /test-quick - Essential for rapid iteration during development
3. /doc-update - Reduces documentation sync friction

Tier 2 - High Value (Create Next)

4. /sprint-start - Structures sprint planning workflow
5. /sprint-complete - Ensures comprehensive sprint documentation
6. /bench-compare - Critical for performance validation
7. /bug-report - Standardizes bug tracking and investigation

Tier 3 - Supporting Tools (Create After Core)

8. /perf-profile - Specialized performance analysis
9. /module-create - Accelerates new feature development
10. /ci-status - Monitoring and DevOps support

---
File Creation Script

#!/bin/bash

# ProRT-IP Custom Claude Code Commands - Installation Script

COMMANDS_DIR=".claude/commands"

# Ensure commands directory exists
mkdir -p "$COMMANDS_DIR"

echo "Creating ProRT-IP custom commands in $COMMANDS_DIR/"
echo ""

# Command 1: /rust-check
cat > "$COMMANDS_DIR/rust-check.md" << 'RUSTCHECK_EOF'
[... full content from command 1 above ...]
RUSTCHECK_EOF
echo "✅ Created: /rust-check"

# Command 2: /bench-compare
cat > "$COMMANDS_DIR/bench-compare.md" << 'BENCHCOMPARE_EOF'
[... full content from command 2 above ...]
BENCHCOMPARE_EOF
echo "✅ Created: /bench-compare"

# Command 3: /sprint-start
cat > "$COMMANDS_DIR/sprint-start.md" << 'SPRINTSTART_EOF'
[... full content from command 3 above ...]
SPRINTSTART_EOF
echo "✅ Created: /sprint-start"

# Command 4: /sprint-complete
cat > "$COMMANDS_DIR/sprint-complete.md" << 'SPRINTCOMPLETE_EOF'
[... full content from command 4 above ...]
SPRINTCOMPLETE_EOF
echo "✅ Created: /sprint-complete"

# Command 5: /perf-profile
cat > "$COMMANDS_DIR/perf-profile.md" << 'PERFPROFILE_EOF'
[... full content from command 5 above ...]
PERFPROFILE_EOF
echo "✅ Created: /perf-profile"

# Command 6: /module-create
cat > "$COMMANDS_DIR/module-create.md" << 'MODULECREATE_EOF'
[... full content from command 6 above ...]
MODULECREATE_EOF
echo "✅ Created: /module-create"

# Command 7: /doc-update
cat > "$COMMANDS_DIR/doc-update.md" << 'DOCUPDATE_EOF'
[... full content from command 7 above ...]
DOCUPDATE_EOF
echo "✅ Created: /doc-update"

# Command 8: /test-quick
cat > "$COMMANDS_DIR/test-quick.md" << 'TESTQUICK_EOF'
[... full content from command 8 above ...]
TESTQUICK_EOF
echo "✅ Created: /test-quick"

# Command 9: /ci-status
cat > "$COMMANDS_DIR/ci-status.md" << 'CISTATUS_EOF'
[... full content from command 9 above ...]
CISTATUS_EOF
echo "✅ Created: /ci-status"

# Command 10: /bug-report
cat > "$COMMANDS_DIR/bug-report.md" << 'BUGREPORT_EOF'
[... full content from command 10 above ...]
BUGREPORT_EOF
echo "✅ Created: /bug-report"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "ProRT-IP Custom Commands Installed"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "10 commands created in $COMMANDS_DIR/"
echo ""
echo "Tier 1 (Daily Use):"
echo "  /rust-check      - Fast quality pipeline"
echo "  /test-quick      - Rapid test iteration"
echo "  /doc-update      - Documentation sync"
echo ""
echo "Tier 2 (Sprint Workflow):"
echo "  /sprint-start    - Initialize sprint"
echo "  /sprint-complete - Finalize sprint"
echo "  /bench-compare   - Performance comparison"
echo "  /bug-report      - Bug tracking"
echo ""
echo "Tier 3 (Specialized):"
echo "  /perf-profile    - Performance profiling"
echo "  /module-create   - New module generation"
echo "  /ci-status- CI/CD monitoring"
echo ""
echo "Ready to use! Example: /rust-check"
echo "═══════════════════════════════════════════════════════════"

Save this script as .claude/install-commands.sh and run:

cd /home/parobek/Code/ProRT-IP
chmod +x .claude/install-commands.sh
./.claude/install-commands.sh

---
Usage Scenarios

Daily Development

# Start work session
/test-quick scheduler# Quick iteration on scheduler.rs
/rust-check   # Full quality check before commit

# After changes
/doc-update feature "Added X"      # Sync documentation
/stage-commit "feat: Added X"      # Commit (existing command)

Sprint Workflow

# Sprint start
/sprint-start 5.1 "Plugin system foundation"

# Development
/module-create prtip-scanner plugin_loader "Lua plugin loading system"
/test-quick plugin_loader
/rust-check

# Sprint end
/bench-compare ed1d105 HEAD
/sprint-complete 5.1 "Plugin system foundation complete"

Performance Investigation

# Detect regression
/bench-compare v0.3.0 HEAD -p 1-10000 127.0.0.1

# Profile if regression found
/perf-profile -p 1-10000 127.0.0.1

# Fix and validate
/bench-compare HEAD~1 HEAD  # Compare before/after fix

Bug Fixing

# Report bug
/bug-report "Progress bar stuck at 0%"

# Investigation
/test-quick progress --nocapture
/rust-check

# Fix validation
/test-quick progress
/doc-update fix "Fixed progress bar display bug"

Release Preparation

# Final validation
/rust-check
/ci-status
/bench-compare v0.2.0 HEAD  # Compare to last release

# Documentation
/doc-update release "Version 0.3.0 preparation"

---
Conclusion

This comprehensive suite of 10 custom commands provides:

✅ Daily workflow acceleration (rust-check, test-quick, doc-update)
✅ Sprint management automation (sprint-start, sprint-complete)
✅ Performance optimization tools (bench-compare, perf-profile)
✅ Development acceleration (module-create, bug-report)
✅ DevOps monitoring (ci-status)

Each command follows ProRT-IP conventions, embeds project-specific context, and provides actionable outputs.
The commands are designed to eliminate repetitive manual workflows while maintaining ProRT-IP's high quality
standards.
