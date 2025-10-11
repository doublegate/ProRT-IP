Finalize sprint with comprehensive summary and documentation: $*

---

## SPRINT COMPLETION WORKFLOW

**Purpose:** Generate comprehensive sprint completion summary, update documentation, and prepare for git commit

**Usage:** `/sprint-complete <sprint-id>`
- **sprint-id:** Sprint identifier (e.g., "4.15", "5.1", "phase5-idle-scanning")

**Example:** `/sprint-complete 4.15`

---

## Phase 1: VALIDATE SPRINT AND GATHER METRICS

**Objective:** Verify sprint directory exists and collect all completion metrics

### Step 1.1: Validate Sprint Directory

```bash
SPRINT_ID="$1"

if [ -z "$SPRINT_ID" ]; then
  echo "❌ ERROR: Sprint ID required"
  echo ""
  echo "Usage: /sprint-complete <sprint-id>"
  echo "Example: /sprint-complete 4.15"
  exit 1
fi

SPRINT_DIR="/tmp/ProRT-IP/sprint-${SPRINT_ID}"

if [ ! -d "$SPRINT_DIR" ]; then
  echo "❌ ERROR: Sprint directory not found: $SPRINT_DIR"
  echo ""
  echo "Did you run /sprint-start first?"
  echo ""
  echo "To create sprint: /sprint-start $SPRINT_ID \"<objective>\""
  exit 1
fi

echo "✅ Sprint directory found: $SPRINT_DIR"
echo ""
```

### Step 1.2: Validate Sprint Completion Readiness

```bash
# Check task-checklist.md for incomplete tasks
if [ -f "$SPRINT_DIR/task-checklist.md" ]; then
  INCOMPLETE=$(grep -c '^\- \[ \]' "$SPRINT_DIR/task-checklist.md" 2>/dev/null || echo "0")
  TOTAL=$(grep -c '^\- \[' "$SPRINT_DIR/task-checklist.md" 2>/dev/null || echo "0")

  if [ "$INCOMPLETE" -gt 0 ] && [ "$TOTAL" -gt 0 ]; then
    COMPLETE=$((TOTAL - INCOMPLETE))
    PERCENT=$((COMPLETE * 100 / TOTAL))

    echo "⚠️  WARNING: $INCOMPLETE/$TOTAL tasks still incomplete ($PERCENT% complete)"
    echo ""
    read -p "Continue with sprint completion anyway? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      echo "Aborted - complete remaining tasks first"
      echo ""
      echo "View tasks: cat $SPRINT_DIR/task-checklist.md"
      exit 1
    fi
  else
    echo "✅ All tasks marked complete ($TOTAL/$TOTAL)"
  fi
fi

# Verify all tests passing
echo "Verifying test status..."
cargo test --workspace --quiet > /tmp/test-final.txt 2>&1
TEST_EXIT_CODE=$?

if [ "$TEST_EXIT_CODE" -ne 0 ]; then
  echo "❌ ERROR: Tests are failing - cannot complete sprint with failing tests"
  echo ""
  echo "Run /rust-check to identify issues"
  echo "Or run /test-quick <pattern> for targeted testing"
  echo ""
  read -p "Override and complete sprint anyway? (NOT RECOMMENDED) (y/N): " -n 1 -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
  echo "⚠️  WARNING: Completing sprint with failing tests"
else
  echo "✅ All tests passing"
fi

echo ""
```

### Step 1.3: Gather Test Metrics

```bash
# Run full test suite and capture results
cargo test --workspace 2>&1 | tee /tmp/test-results.txt

# Parse test counts
TOTAL_TESTS=$(grep -oP '\d+(?= tests)' /tmp/test-results.txt | tail -1)
PASSED_TESTS=$(grep -oP '\d+(?= passed)' /tmp/test-results.txt | tail -1)
FAILED_TESTS=$(grep -oP '\d+(?= failed)' /tmp/test-results.txt | tail -1 || echo "0")
```

### Step 1.3: Gather Code Metrics

```bash
# Count lines of code added/modified
NEW_FILES=$(find crates/ -name "*.rs" -newer "$SPRINT_DIR" | wc -l)
TOTAL_LINES=$(find crates/ -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')

# Get git diff statistics
FILES_CHANGED=$(git diff --stat | tail -1 | grep -oP '\d+(?= files? changed)')
INSERTIONS=$(git diff --stat | tail -1 | grep -oP '\d+(?= insertions?)')
DELETIONS=$(git diff --stat | tail -1 | grep -oP '\d+(?= deletions?)')
```

### Step 1.4: Check Task Completion

```bash
# Count completed tasks from task-checklist.md
TOTAL_TASKS=$(grep -c '^\- \[' "$SPRINT_DIR/task-checklist.md")
COMPLETED_TASKS=$(grep -c '^\- \[x\]' "$SPRINT_DIR/task-checklist.md")
INCOMPLETE_TASKS=$((TOTAL_TASKS - COMPLETED_TASKS))
```

---

## Phase 2: GENERATE IMPLEMENTATION SUMMARY

**Objective:** Create comprehensive implementation-summary.md documenting all changes

### Step 2.1: Create Implementation Summary Document

**Template Location:** `$SPRINT_DIR/implementation-summary.md`

**Document Structure:**

```markdown
# Sprint $SPRINT_ID Implementation Summary

**Sprint:** $SPRINT_ID
**Objective:** [Extract from sprint-plan.md]
**Status:** ✅ COMPLETE
**Started:** [Extract from sprint-plan.md]
**Completed:** $(date +%Y-%m-%d)
**Duration:** [Calculate days between start and completion]

## Git Information

**Commit Hash:** $(git rev-parse --short HEAD 2>/dev/null || echo "Not committed")
**Branch:** $(git branch --show-current 2>/dev/null || echo "Unknown")
**Staged Files:** $(git diff --cached --name-only 2>/dev/null | wc -l)
**Unstaged Changes:** $(git diff --name-only 2>/dev/null | wc -l)

## Objective Achievement

✅ **Primary Objective:** [Restate sprint objective]

[Analysis: Was the objective fully achieved? Partially? Explain.]

## Implementation Details

### Features Implemented

1. **[Feature 1 Name]** ([file paths])
   - **Description:** [What was implemented]
   - **Lines of Code:** [LOC count]
   - **Tests Added:** [Test count]
   - **Key Functions:** [List main functions]

2. **[Feature 2 Name]** ([file paths])
   - [Same structure]

### Files Created

| File | Purpose | Lines | Tests |
|------|---------|-------|-------|
| [path] | [description] | [LOC] | [count] |

### Files Modified

| File | Changes | Reason |
|------|---------|--------|
| [path] | [what changed] | [why] |

## Technical Decisions

[Extract from implementation-notes.md - key decisions made during sprint]

### Decision 1: [Topic]
**Chosen:** [Option selected]
**Rationale:** [Why this was chosen]
**Impact:** [How this affects the project]

## Performance Impact

### Benchmark Results

[Include benchmark data from benchmarks/ directory]

| Scenario | Before | After | Improvement |
|----------|--------|-------|-------------|
| [test] | [baseline] | [result] | [delta] |

### Performance Analysis

[Interpretation of benchmark results - regressions? improvements?]

## Testing

**Total Tests:** $TOTAL_TESTS
**Passing:** $PASSED_TESTS
**Failing:** $FAILED_TESTS
**Success Rate:** [Calculate percentage]

**New Tests Added:** [Count]
**Test Coverage:** [Estimate percentage]

### Test Categories

- **Unit Tests:** [Count and description]
- **Integration Tests:** [Count and description]
- **Performance Tests:** [Count and description]

## Code Metrics

**Files Changed:** $FILES_CHANGED
**Insertions:** +$INSERTIONS lines
**Deletions:** -$DELETIONS lines
**Net Change:** [Calculate net]

**New Files:** $NEW_FILES
**Total Project Lines:** $TOTAL_LINES

## Issues Encountered

[Extract from implementation-notes.md - problems and solutions]

### Issue 1: [Problem]
**Symptoms:** [What went wrong]
**Root Cause:** [Why]
**Solution:** [How fixed]
**Time Lost:** [Estimate]

## Documentation Updates

- [ ] README.md updated with new features
- [ ] CHANGELOG.md updated with sprint entry
- [ ] CLAUDE.local.md updated with session summary
- [ ] API documentation updated
- [ ] User guide updated (if applicable)

## Deliverables

✅ [List all deliverables from sprint-plan.md]

## Sprint Retrospective

### What Went Well
- [Success 1]
- [Success 2]
- [Success 3]

### What Could Be Improved
- [Improvement 1]
- [Improvement 2]

### Lessons Learned
- [Lesson 1]
- [Lesson 2]

## Next Steps

**Immediate:**
- Update CHANGELOG.md with comprehensive sprint entry
- Update README.md with performance metrics
- Commit changes with detailed commit message

**Future Sprints:**
- [Follow-up task 1]
- [Follow-up task 2]

---

**Generated:** $(date)
**Sprint Directory:** $SPRINT_DIR
```

---

## Phase 3: UPDATE CHANGELOG.md

**Objective:** Add comprehensive sprint entry to CHANGELOG.md

### Step 3.1: Determine Version Impact

```bash
# Analyze changes to determine version bump
if grep -q "BREAKING" "$SPRINT_DIR/implementation-summary.md"; then
  VERSION_TYPE="MAJOR"
elif grep -q "feat:" "$SPRINT_DIR/implementation-summary.md"; then
  VERSION_TYPE="MINOR"
else
  VERSION_TYPE="PATCH"
fi
```

### Step 3.2: Generate CHANGELOG Entry

**Location:** `CHANGELOG.md` (prepend to [Unreleased] section)

**Entry Format:**

```markdown
### Sprint $SPRINT_ID - [Sprint Objective] ($(date +%Y-%m-%d))

**Objective:** [Brief sprint objective]

#### Features Implemented
- **[Feature 1]** - [Description] ([files])
- **[Feature 2]** - [Description] ([files])

#### Performance Improvements
- [Improvement 1]: [Baseline] → [Result] ([X% faster/slower])
- [Improvement 2]: [Details]

#### Technical Changes
- [Change 1] ([rationale])
- [Change 2] ([rationale])

#### Testing
- Added [X] new tests ([categories])
- Total tests: [X] ([100%] passing)
- Coverage: [X%]

#### Files Modified
- **Created:** [X] files ([LOC] lines)
- **Modified:** [X] files ([+insertions/-deletions])
- **Deleted:** [X] files

#### Breaking Changes
[List any breaking changes, or "None"]

#### Documentation
- Updated README.md with [changes]
- Updated [other docs]

#### Known Issues
[List any known issues introduced or discovered]

#### Next Sprint
- [Preview of next sprint focus]
```

### Step 3.3: Update CHANGELOG.md

```bash
# Read current CHANGELOG
CURRENT_CHANGELOG=$(cat CHANGELOG.md)

# Generate new entry
NEW_ENTRY="[Generated entry from template above]"

# Prepend to [Unreleased] section
# [Implementation depends on parsing logic]
```

---

## Phase 4: UPDATE CLAUDE.local.md

**Objective:** Add sprint completion session to CLAUDE.local.md

### Step 4.1: Generate Session Entry

**Session Format:**

```markdown
### $(date +%Y-%m-%d): Sprint $SPRINT_ID Complete - [Objective] (SUCCESS ✅)

**Objective:** [Sprint objective]
**Duration:** [X days/weeks]
**Result:** [SUCCESS ✅ / PARTIAL SUCCESS ⚠️]

**Activities:**

- **Implementation:**
  - [Feature 1]: [Brief description]
  - [Feature 2]: [Brief description]
  - [Feature 3]: [Brief description]

- **Testing:**
  - All [X] tests passing (100% success rate)
  - Added [X] new tests ([categories])
  - Zero regressions

- **Performance:**
  - [Metric 1]: [Baseline] → [Result] ([improvement])
  - [Metric 2]: [Details]

- **Documentation:**
  - Updated CHANGELOG.md with comprehensive sprint entry
  - Updated README.md with [changes]
  - Created implementation-summary.md ([X KB])

**Deliverables:**

- [X] files created ([LOC] lines)
- [X] files modified ([+insertions/-deletions])
- [X] benchmark files
- [X] documentation updates

**Files Modified:**
- [List key files]

**Issues Encountered:**
- [Issue 1]: [Resolution]
- [Issue 2]: [Resolution]

**Result:** Sprint $SPRINT_ID COMPLETE - [Summary of achievement]

**Next Sprint:** [Preview if known]
```

### Step 4.2: Update Current Status Table

```markdown
| Metric | Value | Details |
|--------|-------|---------|
| **Phase Progress** | Sprint $SPRINT_ID COMPLETE | [Achievement] |
| **Tests** | $TOTAL_TESTS passing (100%) | Zero regressions |
| **Total Lines** | $TOTAL_LINES | Sprint $SPRINT_ID: +$INSERTIONS/-$DELETIONS |
```

---

## Phase 5: GENERATE COMMIT MESSAGE

**Objective:** Create comprehensive commit message for git commit

### Step 5.1: Generate Commit Message

**Format:** Conventional Commits with comprehensive body

**Template Location:** `/tmp/ProRT-IP/sprint-${SPRINT_ID}-commit-message.txt`

**Commit Message Structure:**

```
feat(sprint$SPRINT_ID): [Sprint objective - concise summary <72 chars]

[Detailed description of sprint achievements - 2-3 paragraphs]

## Changes Made

**Features Implemented:**
- [Feature 1]: [Description]
- [Feature 2]: [Description]

**Performance Improvements:**
- [Metric]: [Baseline] → [Result] ([X% improvement])

**Technical Changes:**
- [Change 1]
- [Change 2]

## Impact

**Performance:**
- [Key performance metric changes]

**Features:**
- [New functionality]

**Testing:**
- [X] new tests added ([categories])
- Total tests: $TOTAL_TESTS (100% passing)

## Files Modified

**Created ([X] files, [LOC] lines):**
- [file path] - [purpose]

**Modified ([X] files):**
- [file path] - [changes]

## Testing

- Tests passing: $TOTAL_TESTS/$TOTAL_TESTS (100%)
- New tests added: [X]
- Coverage: [X%]
- Zero regressions

## Documentation

- README.md updated: yes
- CHANGELOG.md updated: yes
- CLAUDE.local.md updated: yes
- Sprint summary: $SPRINT_DIR/implementation-summary.md

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## Phase 6: DISPLAY SPRINT COMPLETION SUMMARY

**Objective:** Provide comprehensive completion summary and next steps

### Step 6.1: Display Summary

```bash
echo "=========================================="
echo "Sprint $SPRINT_ID Completed Successfully"
echo "=========================================="
echo ""
echo "📊 METRICS"
echo "  Duration: [X days/weeks]"
echo "  Tasks Completed: $COMPLETED_TASKS/$TOTAL_TASKS"
echo "  Tests Passing: $TOTAL_TESTS (100%)"
echo "  Files Changed: $FILES_CHANGED (+$INSERTIONS/-$DELETIONS)"
echo ""
echo "📝 DOCUMENTATION UPDATES"
echo "  ✅ implementation-summary.md created"
echo "  ✅ CHANGELOG.md updated"
echo "  ✅ CLAUDE.local.md updated"
echo "  ✅ Commit message generated"
echo ""
echo "📁 SPRINT ARTIFACTS"
echo "  Sprint Directory: $SPRINT_DIR"
echo "  Implementation Summary: $SPRINT_DIR/implementation-summary.md"
echo "  Commit Message: /tmp/ProRT-IP/sprint-${SPRINT_ID}-commit-message.txt"
echo ""
echo "🚀 NEXT STEPS"
echo "  1. Review implementation-summary.md"
echo "  2. Verify CHANGELOG.md entry"
echo "  3. Run /rust-check for final validation"
echo "  4. Commit changes: git commit -F /tmp/ProRT-IP/sprint-${SPRINT_ID}-commit-message.txt"
echo "  5. Plan next sprint with /sprint-start"
echo ""
```

---

## SUCCESS CRITERIA

✅ Sprint directory validated and exists
✅ All metrics collected (tests, code, tasks)
✅ implementation-summary.md generated (comprehensive)
✅ CHANGELOG.md updated with sprint entry
✅ CLAUDE.local.md updated with session
✅ Commit message generated (ready for git commit)
✅ Summary displayed with next steps

---

## DELIVERABLES

1. **Implementation Summary:** `$SPRINT_DIR/implementation-summary.md` (comprehensive report)
2. **CHANGELOG Entry:** Updated with sprint achievements
3. **Memory Bank Update:** CLAUDE.local.md session entry
4. **Commit Message:** Ready-to-use git commit message
5. **Completion Summary:** Console output with metrics

---

## RELATED COMMANDS

**Sprint Workflow:**
- `/sprint-start <sprint-id> <objective>` - Initialize sprint (required before completion)
- Complete workflow pair for sprint-based development

**Quality Validation:**
- `/rust-check` - Final quality check before sprint completion
- `/test-quick <pattern>` - Debug failing tests identified during completion
- Sprint completion validates all tests passing automatically

**Performance Tracking:**
- `/bench-compare <baseline> <comparison>` - Measure sprint performance impact
- `/perf-profile <command>` - Profile optimizations implemented during sprint
- Results integrated into implementation-summary.md

**Documentation:**
- `/doc-update <type> <desc>` - Sprint completion internally updates documentation
- README.md, CHANGELOG.md, and CLAUDE.local.md all synchronized

## WORKFLOW INTEGRATION

**Sprint Completion Workflow:**

```
1. Pre-Completion Validation:
   - Review task-checklist.md (ensure all tasks marked complete)
   - /rust-check  # Verify quality standards met
   - /bench-compare <last-sprint> HEAD  # Measure impact

2. Sprint Finalization:
   /sprint-complete X.Y
   # Validates tests, gathers metrics, generates summary

3. Review Generated Artifacts:
   - implementation-summary.md  # Comprehensive sprint report
   - CHANGELOG.md updated entry
   - CLAUDE.local.md session entry
   - Commit message template ready

4. Git Commit:
   git add .
   git commit -F /tmp/ProRT-IP/sprint-X.Y-commit-message.txt
   git push

5. Next Sprint Planning:
   /sprint-start X.Z "Next objective"
```

**Sprint Success Criteria:**

- ✅ All tasks in task-checklist.md marked complete
- ✅ All 643 tests passing (100% success rate)
- ✅ Zero clippy warnings
- ✅ Documentation updated (README, CHANGELOG, CLAUDE.local)
- ✅ Performance metrics documented (if applicable)
- ✅ Git commit message generated and reviewed

## SEE ALSO

- `docs/01-ROADMAP.md` - Project roadmap and sprint planning
- `docs/10-PROJECT-STATUS.md` - Overall project task tracking
- `CLAUDE.local.md` - Sprint history and sessions
- `CONTRIBUTING.md` - Sprint workflow guidelines
- `ref-docs/10-Custom-Commands_Analysis.md` - Command patterns

---

**Finalize sprint: $***
