# ProRT-IP Custom Commands

**Location:** `.claude/commands/`
**Count:** 25 commands (15 existing + 10 new)
**Purpose:** Development workflow automation for Claude Code
**Updated:** 2025-11-06

## Overview

This directory contains 14 custom slash commands designed to streamline the ProRT-IP development workflow when using [Claude Code](https://claude.com/claude-code). These commands automate repetitive tasks, enforce quality standards, and provide comprehensive development utilities.

### Command Categories

1. **Quality Assurance** (3 commands)
   - `/rust-check` - Comprehensive Rust quality pipeline
   - `/test-quick` - Fast targeted test execution
   - `/ci-status` - GitHub Actions CI/CD status monitoring

2. **Sprint Management** (2 commands)
   - `/sprint-start` - Initialize sprint with planning documents
   - `/sprint-complete` - Finalize sprint with comprehensive summary

3. **Performance Analysis** (2 commands)
   - `/bench-compare` - Compare performance between git refs
   - `/perf-profile` - Generate CPU profiles with flamegraphs

4. **Development Utilities** (3 commands)
   - `/module-create` - Generate new Rust modules with boilerplate
   - `/doc-update` - Quick documentation sync
   - `/bug-report` - Comprehensive bug report generation

5. **Workflow Automation** (4 commands)
   - `/sub-agent` - Launch specialized sub-agent tasks
   - `/stage-commit` - Comprehensive pre-commit workflow
   - `/mem-reduce` - Memory bank optimization
   - `/daily-log` - End-of-day comprehensive consolidation

6. **Strategic Planning** (1 command)
   - `/inspire-me` - Competitive analysis and enhancement planning

---

## Command Reference

### 1. `/rust-check` - Comprehensive Rust Quality Pipeline

**Purpose:** Execute complete Rust quality checks in optimized phases (fast-fail → comprehensive → build)

**Background:** Runs the same checks as CI/CD locally to catch issues before pushing. Uses early termination to fail fast on format/lint issues before running expensive tests.

**Usage:**
```bash
/rust-check                      # Full pipeline, all packages
/rust-check --package prtip-core # Single package only
/rust-check quick                # Fast checks only (format + clippy)
/rust-check "tcp_connect"        # Test pattern match
```

**Phases:**
1. **Phase 0:** Validate prerequisites (cargo, rustfmt, clippy)
2. **Phase 1:** Fast-fail quality checks (format + clippy in parallel)
3. **Phase 2:** Comprehensive test suite (643 tests)
4. **Phase 3:** Release build verification

**When to Use:**
- Before every git commit
- After implementing new features
- When debugging CI failures locally
- Pre-push validation

**Example Workflow:**
```bash
# Development cycle
/test-quick tcp_connect  # Quick iteration
/rust-check             # Before commit
git commit
```

---

### 2. `/test-quick` - Fast Targeted Test Execution

**Purpose:** Run specific tests or test subsets without executing the full 643-test suite (~5-6 minutes)

**Background:** ProRT-IP has 643 tests that take 5-6 minutes to run. This command allows rapid iteration by testing only what's needed.

**Usage:**
```bash
/test-quick tcp_connect      # Run TCP connect tests
/test-quick prtip-network    # Run network package tests
/test-quick integration      # Integration tests only
/test-quick scheduler        # Scheduler tests
```

**Pattern Types:**
- **Package:** `prtip-core`, `prtip-network`, `prtip-scanner`, `prtip-cli`
- **Category:** `integration`, `unit`, `doc`
- **Module:** `tcp_connect`, `syn_scanner`, `scheduler`, etc.
- **Feature:** `timing`, `rate_limit`, `blackrock`, `decoy`

**Performance Comparison:**

| Test Type | Tests | Duration | Use Case |
|-----------|-------|----------|----------|
| Full Suite | 643 | ~5-6 min | Pre-commit, CI/CD |
| prtip-core | ~60 | ~10-15s | Core type changes |
| prtip-scanner | ~400 | ~60-90s | Scanner logic changes |
| Specific module | ~10-30 | ~5-20s | Rapid iteration |

**When to Use:**
- During active development (rapid feedback)
- Debugging specific test failures
- Testing module-specific changes
- Validating bug fixes

**Example Workflow:**
```bash
# Rapid iteration cycle
/test-quick syn_scanner  # Test your changes
# ... make fixes ...
/test-quick syn_scanner  # Retest quickly
/rust-check             # Full validation before commit
```

---

### 3. `/ci-status` - GitHub Actions CI/CD Status

**Purpose:** Check GitHub Actions workflow status, identify failures, and provide debugging guidance

**Background:** Monitors CI/CD pipeline health and helps debug failures by suggesting local reproduction steps.

**Usage:**
```bash
/ci-status                    # Last 10 runs, all workflows
/ci-status 1234567890         # Specific run details
/ci-status CI                 # Filter to CI workflow
/ci-status --failed           # Only failed runs
```

**Features:**
- Latest run summary with status
- Recent runs table (last 10)
- Failure analysis with common causes
- Local reproduction guide
- Pull request status checks

**When to Use:**
- After pushing to check if CI passed
- Investigating CI failures
- Before merging pull requests
- Validating fixes pushed to remote

**Example Workflow:**
```bash
# CI failure investigation
/ci-status --failed          # Identify what failed
# Review failure details
/rust-check                  # Reproduce locally
# Fix issues
git push
/ci-status                   # Verify fix
```

---

### 4. `/sprint-start` - Initialize Sprint Planning

**Purpose:** Create comprehensive sprint directory structure with planning documents, task checklists, and implementation notes

**Background:** Implements structured sprint-based development with clear objectives, tasks, and deliverables tracking.

**Usage:**
```bash
/sprint-start 4.15 "Implement idle scanning"
/sprint-start 5.1 "Performance optimization sprint"
/sprint-start phase5-idle-scanning "Zombie scanning for anonymity"
```

**Creates:**
```
/tmp/ProRT-IP/sprint-X.Y/
├── sprint-plan.md           # Master planning document
├── task-checklist.md        # Actionable task list
├── implementation-notes.md  # Technical decisions
├── benchmarks/              # Performance data
├── tests/                   # Test outputs
├── analysis/                # Investigation results
└── docs/                    # Sprint documentation
```

**Generated Documents:**
1. **sprint-plan.md** - Objectives, scope, timeline, success criteria
2. **task-checklist.md** - 3-8 actionable tasks with checkboxes
3. **implementation-notes.md** - Technical decisions log

**When to Use:**
- Beginning a new feature or enhancement
- Starting performance optimization work
- Planning bug fix iterations
- Multi-day development efforts

**Example Workflow:**
```bash
# Sprint lifecycle
/sprint-start 5.1 "Add idle scanning support"
# ... development work, track in task-checklist.md ...
/sprint-complete 5.1
git commit -F /tmp/ProRT-IP/sprint-5.1-commit-message.txt
```

---

### 5. `/sprint-complete` - Finalize Sprint

**Purpose:** Generate comprehensive sprint completion summary, update documentation, and prepare for git commit

**Background:** Captures all sprint achievements, metrics, and changes in a structured format for documentation and git history.

**Usage:**
```bash
/sprint-complete 4.15
/sprint-complete 5.1
```

**Generates:**
1. **implementation-summary.md** - Comprehensive sprint report
2. **CHANGELOG.md entry** - Sprint achievements
3. **CLAUDE.local.md session** - Memory bank update
4. **Git commit message** - Ready-to-use template

**Validation:**
- Checks task-checklist.md for incomplete tasks
- Verifies all 643 tests passing
- Warns if tests failing
- Gathers git metrics (files changed, insertions, deletions)

**When to Use:**
- After completing all sprint tasks
- Before committing sprint changes
- To document sprint achievements
- When transitioning to next sprint

**Example Workflow:**
```bash
# Sprint completion
/sprint-complete 5.1
# Review: /tmp/ProRT-IP/sprint-5.1/implementation-summary.md
git add .
git commit -F /tmp/ProRT-IP/sprint-5.1-commit-message.txt
git push
/sprint-start 5.2 "Next sprint objective"
```

---

### 6. `/bench-compare` - Performance Comparison

**Purpose:** Compare benchmark performance between two git references to detect regressions or validate optimizations

**Background:** Uses hyperfine for statistical benchmarking (20 runs) to measure performance differences with confidence intervals.

**Usage:**
```bash
/bench-compare v0.3.0 main              # Compare release vs current
/bench-compare baseline-tag HEAD        # Before/after optimization
/bench-compare sprint-4.14 sprint-4.15  # Sprint comparison
```

**Process:**
1. Validates git refs and checks working tree
2. Builds baseline binary (`/tmp/prtip-baseline`)
3. Builds comparison binary (`/tmp/prtip-comparison`)
4. Runs hyperfine benchmarks (20 runs, 3 warmup)
5. Generates comprehensive report with delta analysis

**Performance Interpretation:**
- **DELTA < -5%:** 🚀 Performance improvement (faster)
- **-5% ≤ DELTA ≤ +5%:** ✅ No significant change (within noise)
- **DELTA > +5%:** ⚠️ Performance regression (slower)

**When to Use:**
- After performance optimizations
- Before merging optimization PRs
- Validating sprint performance impact
- Detecting unexpected regressions

**Example Workflow:**
```bash
# Optimization validation
git tag baseline-before-opt
# ... implement optimizations ...
/bench-compare baseline-before-opt HEAD
# Review: /tmp/bench-compare-report.md
/doc-update perf "10K ports: 117ms → 39ms (66% faster)"
```

---

### 7. `/perf-profile` - CPU Performance Profiling

**Purpose:** Execute comprehensive CPU profiling using Linux perf and generate interactive flamegraph visualization

**Background:** Profiles CPU usage to identify performance bottlenecks using DWARF call graphs and generates interactive SVG flamegraphs.

**Usage:**
```bash
/perf-profile ./target/release/prtip -p 1-10000 127.0.0.1
/perf-profile ./target/release/prtip -p 80,443 192.168.4.0/24
```

**Process:**
1. Validates perf and flamegraph tools
2. Builds release binary with debug symbols
3. Runs `perf record` with call-graph capture
4. Generates text reports (top functions)
5. Creates interactive flamegraph SVG

**Generates:**
- `/tmp/perf.data` - Raw perf data
- `/tmp/perf-report.txt` - Text report with top functions
- `/tmp/perf-script.txt` - Detailed sample data
- `/tmp/flamegraph.svg` - Interactive visualization

**Flamegraph Navigation:**
- **Width:** Proportion of CPU time (wider = more time)
- **Height:** Call stack depth (bottom = caller, top = callee)
- **Click:** Zoom into function
- **Hover:** Show function name and percentage

**When to Use:**
- After bench-compare detects regression
- Identifying optimization opportunities
- Understanding CPU hotspots
- Before major performance sprints

**Example Workflow:**
```bash
# Performance optimization
/bench-compare baseline HEAD  # Detected regression
/perf-profile ./target/release/prtip -p 1-10000 127.0.0.1
# Open /tmp/flamegraph.svg in browser
# Identify hot function
# Optimize code
/bench-compare baseline HEAD  # Validate improvement
```

---

### 8. `/module-create` - Generate Rust Modules

**Purpose:** Generate new Rust module with comprehensive boilerplate, tests, documentation, and integration into lib.rs

**Background:** Automates module creation with ~200 lines of boilerplate including docs, tests, and proper integration.

**Usage:**
```bash
/module-create scanner idle_scanner "Idle/zombie scanning for ultimate anonymity"
/module-create network packet_fragment "Packet fragmentation for evasion"
/module-create core config_parser "Configuration file parsing"
```

**Crates Available:**
- `core` - Data types, configuration, utilities
- `network` - Packet handling, socket operations
- `scanner` - Scanning algorithms, detection engines
- `cli` - User interface, output formatting

**Generates:**
1. **Module file** (~200 lines with boilerplate)
   - Documentation comments with examples
   - Struct/enum definitions
   - Implementation with methods
   - Unit tests (4 tests included)
   - Async test template

2. **lib.rs integration** - Module declaration added
3. **Integration guide** - Usage documentation

**When to Use:**
- Starting new feature implementation
- Adding new scanner types
- Creating utility modules
- Extending functionality

**Example Workflow:**
```bash
# New feature development
/sprint-start 5.1 "Implement idle scanning"
/module-create scanner idle_scanner "Idle scanning implementation"
# Edit generated module: crates/prtip-scanner/src/idle_scanner.rs
/test-quick idle_scanner
/rust-check
/sprint-complete 5.1
```

---

### 9. `/doc-update` - Quick Documentation Sync

**Purpose:** Rapidly update README.md, CHANGELOG.md, and CLAUDE.local.md with latest metrics, features, and session summary

**Background:** Keeps documentation synchronized with codebase without manual metric gathering.

**Usage:**
```bash
/doc-update feature "Added idle scanning support"
/doc-update fix "Critical DNS resolution bug"
/doc-update perf "10K ports: 117ms → 39ms (66% faster)"
/doc-update docs "Updated API documentation"
/doc-update general  # Sync metrics only
```

**Update Types:**
- `feature` - New feature implementation
- `fix` - Bug fix
- `perf` - Performance improvement
- `docs` - Documentation update
- `test` - Test additions/improvements
- `refactor` - Code refactoring
- `chore` - Maintenance tasks
- `general` - General update (default)

**Updates:**
1. **README.md** - Test badge, status table, metrics
2. **CHANGELOG.md** - Entry in [Unreleased] section
3. **CLAUDE.local.md** - Session entry, current metrics

**When to Use:**
- After implementing features
- After fixing bugs
- After performance improvements
- Regular metric synchronization

**Example Workflow:**
```bash
# Feature development
# ... implement feature ...
/rust-check
/doc-update feature "Idle scanning - ultimate anonymity with zombie hosts"
git add .
git commit -m "feat: Add idle scanning support"
```

---

### 10. `/bug-report` - Comprehensive Bug Reports

**Purpose:** Generate detailed bug report with system information, reproduction steps, logs, and analysis

**Background:** Captures comprehensive debugging information including system state, verbose logs, and strace output.

**Usage:**
```bash
/bug-report "Scanner hangs on filtered network" "./target/release/prtip -p 1-10000 192.168.4.1"
/bug-report "DNS resolution fails" "./target/release/prtip scanme.nmap.org"
/bug-report "Panic on port 65535" "./target/release/prtip -p 65535 127.0.0.1"
```

**Generates:**
```
/tmp/ProRT-IP/bug-report-TIMESTAMP/
├── BUG-REPORT.md              # Comprehensive markdown report
├── system-info.txt            # OS, hardware, Rust, dependencies
├── network-config.txt         # Network configuration
├── reproduction-stdout.txt    # Standard output
├── reproduction-stderr.txt    # Standard error
├── reproduction-verbose-*.txt # RUST_LOG=debug output
├── reproduction-exit-code.txt # Exit code
└── strace-summary.txt         # System call trace (if available)
```

**Execution Modes:**
1. **Standard** - Normal execution with output capture
2. **Verbose** - With `RUST_LOG=debug` for detailed logs
3. **Strace** - System call tracing (if strace available)

**When to Use:**
- Investigating crashes or panics
- Debugging unexpected behavior
- Creating GitHub issues
- Performance investigations
- CI/CD failure reproduction

**Example Workflow:**
```bash
# Bug investigation
/bug-report "Scanner timeout on filtered ports" "./target/release/prtip -p 1-10000 192.168.4.1"
# Review: /tmp/ProRT-IP/bug-report-*/BUG-REPORT.md
# Analyze verbose logs
# Fix bug
/test-quick <pattern>
/rust-check
/doc-update fix "Fixed scanner timeout on filtered networks"
```

---

### 11. `/sub-agent` - Launch Specialized Sub-Agents

**Purpose:** Generate new sub-agent tool task to delegate complex, multi-step work to specialized agents

**Background:** Embeds comprehensive project context and systematic approach instructions for autonomous task completion.

**Usage:**
```bash
/sub-agent Implement 23 enhancements across all custom commands
/sub-agent Refactor scanner module for better performance
/sub-agent Analyze and fix CI pipeline failures
```

**Features:**
- **Project Context Embedding** - ProRT-IP details, status, architecture
- **Quality Standards** - Code, docs, commits, memory banks
- **Systematic Approach** - Analyze → Plan → Execute → Verify → Document → Report
- **Comprehensive Instructions** - Communication, documentation, verification, deliverables

**When to Use:**
- Complex multi-step tasks
- Large-scale refactoring
- Comprehensive analysis tasks
- Systematic code reviews
- Automated documentation generation

**Example Workflow:**
```bash
# Complex task delegation
/sub-agent Implement idle scanning module with:
- Zombie host discovery
- IP ID sequence detection
- Stealth port scanning
- Comprehensive tests (>90% coverage)
- Documentation and benchmarks
```

---

### 12. `/stage-commit` - Pre-Commit Workflow

**Purpose:** Execute comprehensive pre-commit workflow with code quality, documentation updates, and commit preparation

**Background:** Automates the complete pre-commit checklist to ensure quality and consistency before every commit.

**Phases:**
1. **Analyze Changes** - git status, diff, scope determination
2. **Code Quality** - cargo fmt, clippy, tests, build (if code changed)
3. **Gitignore Maintenance** - Verify no sensitive files
4. **Documentation Updates** - README, CHANGELOG, memory banks
5. **Memory Bank Optimization** - Compress and update CLAUDE.local.md
6. **Cross-Reference Validation** - Check all links
7. **Final Verification** - File counts, build, tests
8. **Stage Changes** - git add -A
9. **Create Commit Message** - Comprehensive conventional commit
10. **Final Review** - User approval before commit

**Usage:**
```bash
/stage-commit
```

**When to Use:**
- Before every significant commit
- After completing features or fixes
- Sprint completion preparation
- Release preparation

**Example Workflow:**
```bash
# Feature completion
# ... implement feature, tests pass ...
/stage-commit
# Review proposed commit message
# Type: yes
# Commit is created automatically
git push
```

---

### 13. `/mem-reduce` - Memory Bank Optimization

**Purpose:** Comprehensively optimize all memory banks (CLAUDE.md files) through systematic compression

**Background:** Memory banks grow over time with sessions and details. This optimizes them for faster Claude Code access.

**Phases:**
1. **Analysis** - Read all memory banks, identify duplication
2. **Organize** - Correct information placement (user/workspace/project/local)
3. **Prioritize** - Most critical info at top
4. **Optimize** - Restructure for fast scanning (tables, lists)
5. **Eliminate** - Remove redundancy and obsolete content
6. **Compress** - Condense prose to bullets, use compact formatting
7. **Verification** - Ensure no information loss

**Usage:**
```bash
/mem-reduce
```

**Target:** 20-40% character count reduction across all memory banks

**When to Use:**
- After 10+ development sessions
- When memory banks feel cluttered
- Performance optimization for Claude Code
- Periodic maintenance (monthly)

**Example Results:**
- Workspace CLAUDE.md: 15,496→6,412 chars (58.6% reduction)
- Project CLAUDE.md: 16,213→9,086 chars (44.0% reduction)
- Local CLAUDE.local.md: 56,962→16,528 chars (71.0% reduction)

---

### 14. `/daily-log` - End-of-Day Consolidation

**Purpose:** Create comprehensive daily log that consolidates all development activity, preserves temporary files, and generates historical record

**Background:** Automates the 2.5-hour manual daily log creation process into a systematic 80-minute workflow. Ensures zero information loss on system reboots by preserving all /tmp/ files and documenting complete development history.

**Usage:**
```bash
/daily-log              # Create daily log for today
```

**Process (6 Phases):**
1. **Initialize** (5 min) - Create directory structure for today
2. **Scan Files** (10 min) - Discover temporary files from /tmp/, docs/, root
3. **Extract Data** (10 min) - Git history, metrics, CI status
4. **Organize Files** (15 min) - Categorize and move/copy files
5. **Generate README** (30 min) - Comprehensive 10-20 page master summary
6. **Verify & Report** (10 min) - Validation and completion metrics

**Creates:**
```
daily_logs/YYYY-MM-DD/
├── README.md (10-20 pages) - Comprehensive daily summary
├── 01-commits/          - Git history, diffs, timeline
├── 02-releases/         - Release notes and artifacts
├── 03-ci-fixes/         - CI troubleshooting docs
├── 04-documentation/    - Documentation updates
├── 05-optimization/     - Performance work
├── 06-sessions/         - Session summaries
├── 07-metrics/          - Tests, coverage, CI status
└── 08-artifacts/        - Temporary files preserved
```

**File Scanning (4 Priority Levels):**
1. **Priority 1:** /tmp/ProRT-IP/ - MOVE all files (explicit temp)
2. **Priority 2:** /tmp/ - MOVE project files (ProRT, prtip, scan)
3. **Priority 3:** docs/ - COPY temporary files (draft, tmp, WIP)
4. **Priority 4:** Root - COPY temporary .md files (RELEASE, DRAFT, etc.)

**Smart Categorization:**
- Release files → 02-releases/
- CI/workflow files → 03-ci-fixes/
- Optimization/performance → 05-optimization/
- Conversation/session → 06-sessions/
- Metrics/coverage → 07-metrics/
- Analysis/reports → 08-artifacts/

**Master README Contents:**
- Executive summary (2-3 paragraphs)
- Detailed timeline (hour-by-hour sessions)
- Major accomplishments (3-7 with details)
- Complete commit summary (table)
- Files modified (categorized)
- Temporary files preserved (inventory)
- Metrics & statistics (current state)
- Decisions made (with rationale)
- Issues encountered & resolved
- Next steps (actionable recommendations)

**Quality Standards:**
- **Length:** 10-20 pages minimum (5,000-10,000 words)
- **Detail:** Every commit documented (2-4 paragraphs each)
- **Completeness:** 100% (all files, commits, metrics)
- **Grade:** A+ target (99/100)

**When to Use:**
- End of development day (before shutdown)
- After major milestones (releases, phase completions)
- Before system reboots (preserve /tmp/ files)
- Weekly consolidation (Friday end of day)

**Benefits:**
- **Zero Information Loss** - All temp files preserved before reboot
- **Historical Record** - Complete development history
- **Time Savings** - 80 min vs 2.5 hours manual
- **Reference Material** - CI fixes, optimizations, decisions
- **Documentation** - Comprehensive daily summaries

**Example Workflow:**
```bash
# End of day routine
/daily-log                                # Create comprehensive log
# Review: daily_logs/YYYY-MM-DD/README.md
# All /tmp/ProRT-IP/ files now preserved
# Safe to reboot - no information loss
```

**Reference Example:** See `daily_logs/2025-10-13/` (1.5MB, 32 files, 57KB master README)

---

### 15. `/inspire-me` - Competitive Analysis & Enhancement Planning

**Purpose:** Execute comprehensive competitive analysis against industry leaders (Nmap, Masscan, RustScan) and generate 8-sprint enhancement roadmap

**Background:** Run at end of each development phase to discover competitive improvements and identify gaps before proceeding to next phase. Automates ~3-4 hours of manual research and planning into a systematic 6-phase workflow.

**Usage:**
```bash
/inspire-me              # Execute full competitive analysis
```

**Process (6 Phases):**
1. **Context Gathering** (15 min) - Understand ProRT-IP current state from docs
2. **Reference Analysis** (30 min) - Analyze code_ref/, GitHub repos, online sources
3. **Gap Analysis** (45 min) - Create feature matrix, identify gaps and opportunities
4. **Sprint Planning** (60 min) - Design 8 enhancement sprints with ROI scores
5. **Document Generation** (45 min) - Create comprehensive enhancement document
6. **Verification & Report** (30 min) - Validate completeness and generate summary

**Generates:**
```
docs/XX-PHASEXX-ENHANCEMENTS.md  - Comprehensive roadmap (18,000+ words)
/tmp/ProRT-IP/inspire-me-report.md - Execution summary
```

**Enhancement Document Contents:**
- **Executive Summary** (2-3 paragraphs with key findings)
- **Competitive Analysis** (Strengths, Gaps, Opportunities)
- **Feature Matrix** (10+ categories, 4+ competitors)
- **8 Detailed Sprints** (Tasks, estimates, success criteria, ROI scores)
- **Performance Benchmarks** (Current + projected)
- **Research Sources** (15+ sources cited)
- **Decision Log** (Sprint prioritization rationale)

**Sprint Structure (Per Sprint):**
- Priority (HIGH/MEDIUM/LOW)
- Duration (3-5 days)
- ROI Score (X.X/10)
- Objective (clear, measurable)
- Rationale (competitive findings)
- 5-10 Tasks (with time estimates)
- Deliverables (code, tests, docs, benchmarks)
- Success Criteria (quantitative + qualitative)
- References (code, research, documentation)

**Research Scope:**
- **Code Analysis:** code_ref/ directory (RustScan, Nmap, Masscan implementations)
- **GitHub Repos:** 4+ repositories (stars, features, activity)
- **Online Research:** Performance comparisons, blog posts, Reddit discussions
- **Documentation:** Official docs, RFCs, technical specs

**Quality Standards:**
- **Document Length:** >10,000 words (comprehensive coverage)
- **Sprint Detail:** 8 fully-detailed sprints with tasks and metrics
- **Research Depth:** 4+ competitors, 15+ sources cited
- **Feature Matrix:** 10+ categories compared
- **Grade Target:** A+ (professional quality)

**When to Use:**
- End of each development phase (Phase 1→2, Phase 4→5, etc.)
- Before major feature additions (understand competitive landscape)
- Quarterly competitive reviews (stay ahead of trends)
- After competitor releases (analyze new features)

**Benefits:**
- **Strategic Planning:** Data-driven 8-sprint roadmap
- **Competitive Intelligence:** Know strengths and gaps vs industry leaders
- **Innovation Opportunities:** Identify unique advantages
- **Time Savings:** 3-4 hours automated research + analysis
- **Comprehensive Documentation:** Full research sources cited

**Example Output (Phase 4):**
- **Document:** `docs/19-PHASE4-ENHANCEMENTS.md` (18,500 words, 80 pages)
- **Competitors Analyzed:** Nmap, Masscan, RustScan, Naabu
- **Sprints Planned:** 4.15-4.22 (Service Detection, CLI, Performance, etc.)
- **Research Sources:** 50+ code files, 4 GitHub repos, 15+ articles
- **Duration:** 4-6 weeks (8 sprints × 3-5 days)

**Reference Example:** See `docs/19-PHASE4-ENHANCEMENTS.md` (v0.3.7→v0.4.0 roadmap)

---

## Common Workflows

### Daily Development

```bash
# Morning routine
/ci-status                           # Check if CI green

# Feature development
/test-quick <module>                 # Rapid iteration
/rust-check                          # Before commit
/doc-update feature "Description"    # Update docs
git commit

# Bug fixing
/bug-report "Issue" "command"        # Capture details
# ... fix bug ...
/test-quick <pattern>                # Verify fix
/doc-update fix "Description"        # Document fix

# End of day
/daily-log                           # Consolidate activity, preserve files
```

### Sprint-Based Development

```bash
# Sprint start
/sprint-start 5.1 "Feature X implementation"

# Development cycle
/module-create scanner feature_x "Description"
/test-quick feature_x
/rust-check

# Sprint end
/sprint-complete 5.1
git commit -F /tmp/ProRT-IP/sprint-5.1-commit-message.txt
```

### Performance Optimization

```bash
# Baseline
git tag baseline-perf
/bench-compare v0.3.0 baseline-perf

# Profile
/perf-profile ./target/release/prtip <args>
# Analyze flamegraph

# Optimize
# ... make changes ...
/test-quick <module>
/bench-compare baseline-perf HEAD

# Document
/doc-update perf "X → Y (Z% faster)"
```

### CI/CD Monitoring

```bash
# Check status
/ci-status

# Debug failures
/ci-status --failed
/rust-check                # Reproduce locally

# After fix
git push
/ci-status                 # Verify
```

---

## Installation

These commands are automatically available in Claude Code when working in the ProRT-IP directory. No installation required.

### Requirements

**Core Tools:**
- `cargo` - Rust build tool (required for rust-check, test-quick)
- `git` - Version control (required for bench-compare, stage-commit)
- `gh` - GitHub CLI (required for ci-status)

**Optional Tools:**
- `hyperfine` - Statistical benchmarking (required for bench-compare)
- `perf` - CPU profiling (required for perf-profile)
- `flamegraph.pl` - Flamegraph generation (required for perf-profile)
- `strace` - System call tracing (optional for bug-report)

### Installing Optional Tools

```bash
# hyperfine
cargo install hyperfine

# perf (Linux)
sudo pacman -S perf          # Arch
sudo apt install linux-tools # Ubuntu

# FlameGraph
git clone https://github.com/brendangregg/FlameGraph
export PATH=$PATH:/path/to/FlameGraph

# strace (usually pre-installed)
sudo pacman -S strace        # Arch
sudo apt install strace      # Ubuntu
```

---

## Best Practices

### Command Chaining

Commands are designed to work together:

```bash
/test-quick → /rust-check → /stage-commit
/bench-compare → /perf-profile → /doc-update
/sprint-start → /module-create → /sprint-complete
/ci-status → /rust-check → /bug-report
```

### Error Handling

All commands include comprehensive error handling:
- Clear error messages with context
- Troubleshooting guidance
- Actionable next steps
- Safe failures (no data loss)

### Documentation Discipline

Update docs with every change:
```bash
/doc-update <type> "Description"  # After every significant change
/sprint-complete <id>             # Comprehensive sprint docs
/stage-commit                     # Automates doc updates
```

### Testing Strategy

```bash
# Development: Fast feedback
/test-quick <pattern>

# Pre-commit: Comprehensive validation
/rust-check

# Performance: Validate no regressions
/bench-compare baseline HEAD
```

---

## Contributing

When adding new custom commands:

1. **Follow Patterns** - Use existing commands as templates
2. **Comprehensive Docs** - Include purpose, usage, examples
3. **Error Handling** - Use `set -e` and `trap ERR`
4. **Cross-References** - Link to related commands
5. **Update README** - Document in this file

---

## Support

**Documentation:**
- This README
- `ref-docs/10-Custom-Commands_Analysis.md` - Detailed analysis
- Individual command files - Inline documentation

**Issues:**
- GitHub Issues: https://github.com/doublegate/ProRT-IP/issues
- Tag with `custom-commands` label

---

**Last Updated:** 2025-10-13
**Command Count:** 14
**Total Lines:** ~5,400 lines across all commands
