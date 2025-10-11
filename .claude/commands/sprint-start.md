Initialize new sprint with planning documents: $*

---

## SPRINT INITIALIZATION WORKFLOW

**Purpose:** Create comprehensive sprint directory structure with planning documents, task checklists, and implementation notes

**Usage:** `/sprint-start <sprint-id> <objective>`
- **sprint-id:** Sprint identifier (e.g., "4.15", "5.1", "phase5-advanced-features")
- **objective:** Brief sprint objective (e.g., "Implement idle scanning", "Performance optimization")

**Example:** `/sprint-start 4.15 "Implement idle/zombie scanning for ultimate anonymity"`

---

## Phase 1: VALIDATE SPRINT PARAMETERS

**Objective:** Ensure sprint ID and objective are provided and valid

### Step 1.1: Parse Arguments

```bash
SPRINT_ID="$1"
OBJECTIVE="${@:2}"

if [ -z "$SPRINT_ID" ]; then
  echo "ERROR: Sprint ID required"
  echo "Usage: /sprint-start <sprint-id> <objective>"
  exit 1
fi

if [ -z "$OBJECTIVE" ]; then
  echo "ERROR: Sprint objective required"
  echo "Usage: /sprint-start <sprint-id> <objective>"
  exit 1
fi
```

### Step 1.2: Validate Sprint ID Format

```bash
# Validate format: X.Y (phase.sprint) or descriptive
if ! [[ "$SPRINT_ID" =~ ^[0-9]+\.[0-9]+$ ]] && ! [[ "$SPRINT_ID" =~ ^phase[0-9]+-[a-z-]+$ ]] && ! [[ "$SPRINT_ID" =~ ^cycle[0-9]+-[a-z-]+$ ]]; then
  echo "❌ ERROR: Invalid sprint ID format"
  echo ""
  echo "Valid formats:"
  echo "  - Numeric: 4.15, 5.1, 5.2 (phase.sprint)"
  echo "  - Descriptive: phase5-idle-scanning, cycle9-optimization"
  echo ""
  echo "Current: '$SPRINT_ID'"
  exit 1
fi

# Extract phase number for reference
if [[ "$SPRINT_ID" =~ ^[0-9]+\.[0-9]+$ ]]; then
  PHASE_NUM=$(echo "$SPRINT_ID" | cut -d. -f1)
  SPRINT_NUM=$(echo "$SPRINT_ID" | cut -d. -f2)
  echo "Phase: $PHASE_NUM, Sprint: $SPRINT_NUM"
elif [[ "$SPRINT_ID" =~ ^phase([0-9]+)- ]]; then
  PHASE_NUM="${BASH_REMATCH[1]}"
  echo "Phase: $PHASE_NUM (descriptive format)"
elif [[ "$SPRINT_ID" =~ ^cycle([0-9]+)- ]]; then
  CYCLE_NUM="${BASH_REMATCH[1]}"
  echo "Cycle: $CYCLE_NUM (enhancement cycle format)"
fi

echo "✅ Sprint ID format valid: $SPRINT_ID"
echo ""
```

**Expected Formats:**
- Phase.Sprint: `4.15`, `5.1`, `5.2`
- Descriptive: `phase5-idle-scanning`, `cycle9-optimization`

### Step 1.3: Check for Existing Sprint Directory and Handle Conflicts

```bash
SPRINT_DIR="/tmp/ProRT-IP/sprint-${SPRINT_ID}"

if [ -d "$SPRINT_DIR" ]; then
  echo "⚠️  WARNING: Sprint directory already exists: $SPRINT_DIR"
  echo ""
  echo "Options:"
  echo "  1. Continue (overwrite existing files)"
  echo "  2. Archive existing sprint to: ${SPRINT_DIR}-archive-$(date +%s)"
  echo "  3. Abort"
  echo ""
  read -p "Choose option (1/2/3): " -n 1 -r
  echo ""

  case $REPLY in
    1)
      echo "Continuing with existing directory..."
      echo "⚠️  Existing files may be overwritten"
      ;;
    2)
      ARCHIVE_DIR="${SPRINT_DIR}-archive-$(date +%s)"
      mv "$SPRINT_DIR" "$ARCHIVE_DIR"
      echo "✅ Archived to: $ARCHIVE_DIR"
      ;;
    3)
      echo "Aborted by user"
      exit 0
      ;;
    *)
      echo "❌ Invalid option, aborting"
      exit 1
      ;;
  esac
fi

echo "✅ Sprint directory path validated"
echo ""
```

---

## Phase 2: CREATE SPRINT DIRECTORY STRUCTURE

**Objective:** Set up organized directory with all necessary planning documents

### Step 2.1: Create Sprint Directory

```bash
mkdir -p "$SPRINT_DIR"
echo "Created sprint directory: $SPRINT_DIR"
```

### Step 2.2: Create Subdirectories

```bash
mkdir -p "$SPRINT_DIR/benchmarks"    # Performance benchmarks
mkdir -p "$SPRINT_DIR/tests"          # Test outputs
mkdir -p "$SPRINT_DIR/analysis"       # Investigation results
mkdir -p "$SPRINT_DIR/docs"           # Sprint-specific documentation
```

**Directory Structure:**
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

---

## Phase 3: GENERATE SPRINT PLAN DOCUMENT

**Objective:** Create comprehensive sprint-plan.md with objectives, scope, timeline, success criteria

### Step 3.1: Create sprint-plan.md

**Template Location:** `$SPRINT_DIR/sprint-plan.md`

**Document Structure:**

```markdown
# Sprint $SPRINT_ID - $OBJECTIVE

**Status:** 🚧 In Progress
**Started:** $(date +%Y-%m-%d)
**Target Completion:** [TBD - estimate based on scope]

## Objective

$OBJECTIVE

## Background

[Why this sprint is needed - problems to solve, opportunities to capture]

## Scope

### In Scope
- [Feature/improvement 1]
- [Feature/improvement 2]
- [Feature/improvement 3]

### Out of Scope
- [Explicitly what will NOT be done this sprint]
- [Deferred to future sprints]

## Technical Approach

### Architecture Changes
[What architectural changes are needed?]

### Key Decisions
[Major technical decisions to be made]

### Dependencies
[What must be completed first? External dependencies?]

## Success Criteria

✅ [Measurable success criterion 1]
✅ [Measurable success criterion 2]
✅ [Measurable success criterion 3]

## Performance Targets

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| [e.g., Scan rate] | [baseline] | [goal] | [how to measure] |

## Testing Strategy

- **Unit Tests:** [What to test, coverage target]
- **Integration Tests:** [End-to-end scenarios]
- **Performance Tests:** [Benchmark scenarios]

## Timeline

- **Phase 1 (Days 1-2):** [Initial implementation]
- **Phase 2 (Days 3-4):** [Integration and testing]
- **Phase 3 (Days 5-6):** [Optimization and documentation]
- **Phase 4 (Day 7):** [Final validation and sprint completion]

## Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| [Risk 1] | [High/Med/Low] | [High/Med/Low] | [How to mitigate] |

## References

- [Relevant documentation]
- [Related sprints]
- [External resources]

---

**Created:** $(date)
**Last Updated:** $(date)
```

---

## Phase 4: GENERATE TASK CHECKLIST

**Objective:** Break down sprint into 3-8 actionable tasks with clear completion criteria

### Step 4.1: Analyze Sprint Objective

**Analysis Prompts:**
- What are the major components of this sprint?
- What are the dependencies between tasks?
- What is the critical path?
- What can be parallelized?

### Step 4.2: Create task-checklist.md

**Template Location:** `$SPRINT_DIR/task-checklist.md`

**Document Structure:**

```markdown
# Sprint $SPRINT_ID Task Checklist

**Objective:** $OBJECTIVE

## Critical Path Tasks (Must Complete in Order)

- [ ] **TASK-1:** [First critical task]
  - **Estimated:** [hours/days]
  - **Dependencies:** None
  - **Completion Criteria:** [How to verify done]

- [ ] **TASK-2:** [Second critical task]
  - **Estimated:** [hours/days]
  - **Dependencies:** TASK-1
  - **Completion Criteria:** [How to verify done]

- [ ] **TASK-3:** [Third critical task]
  - **Estimated:** [hours/days]
  - **Dependencies:** TASK-2
  - **Completion Criteria:** [How to verify done]

## Parallel Tasks (Can Work Simultaneously)

- [ ] **TASK-4:** [Parallel task 1]
  - **Estimated:** [hours/days]
  - **Dependencies:** None
  - **Completion Criteria:** [How to verify done]

- [ ] **TASK-5:** [Parallel task 2]
  - **Estimated:** [hours/days]
  - **Dependencies:** None
  - **Completion Criteria:** [How to verify done]

## Finalization Tasks (Sprint Completion)

- [ ] **TASK-6:** Testing and validation
  - **Estimated:** [hours/days]
  - **Dependencies:** All implementation tasks
  - **Completion Criteria:** All tests passing, zero regressions

- [ ] **TASK-7:** Documentation and benchmarking
  - **Estimated:** [hours/days]
  - **Dependencies:** TASK-6
  - **Completion Criteria:** CHANGELOG updated, benchmarks executed

- [ ] **TASK-8:** Sprint completion and commit
  - **Estimated:** [hours/days]
  - **Dependencies:** TASK-7
  - **Completion Criteria:** Clean git state, ready for merge

## Progress Tracking

**Total Tasks:** 8
**Completed:** 0
**In Progress:** 0
**Blocked:** 0

**Estimated Duration:** [X days/weeks]
**Actual Duration:** [TBD]

---

**Created:** $(date)
**Last Updated:** $(date)
```

---

## Phase 5: GENERATE IMPLEMENTATION NOTES TEMPLATE

**Objective:** Create implementation-notes.md for capturing technical decisions and discoveries

### Step 5.1: Create implementation-notes.md

**Template Location:** `$SPRINT_DIR/implementation-notes.md`

**Document Structure:**

```markdown
# Sprint $SPRINT_ID Implementation Notes

**Sprint:** $SPRINT_ID - $OBJECTIVE

## Technical Decisions Log

### Decision 1: [Topic]
**Date:** [YYYY-MM-DD]
**Context:** [Why this decision was needed]
**Options Considered:**
1. [Option A] - [pros/cons]
2. [Option B] - [pros/cons]
3. [Option C] - [pros/cons]

**Decision:** [Chosen option]
**Rationale:** [Why this option was selected]

---

### Decision 2: [Topic]
[Repeat structure]

---

## Discoveries and Insights

### Discovery 1: [Topic]
**Date:** [YYYY-MM-DD]
**Context:** [What was being investigated]
**Finding:** [What was discovered]
**Impact:** [How this affects the sprint]
**Action:** [What changes resulted from this]

---

## Code Patterns and Best Practices

### Pattern 1: [Pattern Name]
**Use Case:** [When to use this pattern]
**Implementation:** [Code example or reference]
**Benefits:** [Why this pattern is valuable]

---

## Performance Observations

### Observation 1: [Topic]
**Benchmark:** [What was measured]
**Results:** [Actual numbers]
**Analysis:** [Interpretation of results]
**Optimization:** [What was done to improve]

---

## Issues Encountered and Resolutions

### Issue 1: [Problem Description]
**Date:** [YYYY-MM-DD]
**Symptoms:** [What went wrong]
**Root Cause:** [Why it happened]
**Solution:** [How it was fixed]
**Prevention:** [How to avoid in future]

---

## References and Resources

- [Link to relevant documentation]
- [External references used]
- [Related code examples]

---

**Created:** $(date)
**Last Updated:** $(date)
```

---

## Phase 6: DISPLAY SPRINT INITIALIZATION SUMMARY

**Objective:** Provide comprehensive summary and next steps

### Step 6.1: Display Sprint Information

```bash
echo "=========================================="
echo "Sprint $SPRINT_ID Initialized Successfully"
echo "=========================================="
echo ""
echo "Objective: $OBJECTIVE"
echo "Sprint Directory: $SPRINT_DIR"
echo ""
echo "Created Files:"
echo "  ✅ sprint-plan.md           - Master planning document"
echo "  ✅ task-checklist.md        - Actionable task list (8 tasks)"
echo "  ✅ implementation-notes.md  - Technical decisions log"
echo ""
echo "Created Directories:"
echo "  📁 benchmarks/    - Performance benchmarks"
echo "  📁 tests/         - Test outputs"
echo "  📁 analysis/      - Investigation results"
echo "  📁 docs/          - Sprint documentation"
echo ""
echo "Next Steps:"
echo "  1. Review sprint-plan.md and refine scope"
echo "  2. Estimate tasks in task-checklist.md"
echo "  3. Begin implementation with TASK-1"
echo "  4. Document decisions in implementation-notes.md"
echo "  5. Run /sprint-complete when finished"
echo ""
```

### Step 6.2: Update CLAUDE.local.md

**Add Sprint Entry:**

```markdown
## Current Sprint: $SPRINT_ID

**Objective:** $OBJECTIVE
**Status:** 🚧 In Progress
**Started:** $(date +%Y-%m-%d)
**Sprint Directory:** $SPRINT_DIR

**Next Actions:**
- Review sprint-plan.md
- Estimate tasks in task-checklist.md
- Begin TASK-1 implementation
```

---

## SUCCESS CRITERIA

✅ Sprint directory created with proper structure
✅ sprint-plan.md generated with comprehensive planning sections
✅ task-checklist.md created with 3-8 actionable tasks
✅ implementation-notes.md template ready for use
✅ Subdirectories created (benchmarks/, tests/, analysis/, docs/)
✅ CLAUDE.local.md updated with current sprint
✅ Summary displayed with next steps

---

## DELIVERABLES

1. **Sprint Directory:** `/tmp/ProRT-IP/sprint-$SPRINT_ID/`
2. **Planning Documents:** 3 markdown files (plan, checklist, notes)
3. **Directory Structure:** 4 subdirectories for organization
4. **Summary Report:** Console output with next steps
5. **Memory Bank Update:** CLAUDE.local.md sprint entry

---

## RELATED COMMANDS

**Sprint Workflow:**
- `/sprint-complete <sprint-id>` - Finalize sprint with comprehensive summary
- Required after sprint work is complete to generate implementation summary

**Development Workflow:**
- `/module-create <crate> <module-name> <desc>` - Create new modules during sprint
- `/rust-check` - Validate code quality throughout sprint development
- `/test-quick <pattern>` - Run targeted tests during sprint iterations

**Performance Tracking:**
- `/bench-compare <baseline> <comparison>` - Measure sprint performance impact
- `/perf-profile <command>` - Profile performance-critical sprint changes

**Documentation:**
- `/doc-update <type> <desc>` - Document changes throughout sprint
- Sprint completion automatically updates all documentation

## WORKFLOW INTEGRATION

**Complete Sprint Workflow:**

```
1. Sprint Planning:
   /sprint-start 5.X "Implement idle/zombie scanning"
   # Creates: sprint-plan.md, task-checklist.md, implementation-notes.md

2. Development Iteration:
   - Work on tasks from task-checklist.md
   - /test-quick <pattern>  # Quick feedback
   - /rust-check  # Quality validation
   - Document decisions in implementation-notes.md

3. Continuous Documentation:
   - /doc-update feature "Progress update"
   - Update task-checklist.md (mark tasks complete)

4. Performance Validation:
   - /bench-compare v0.3.0 HEAD
   - /perf-profile ./target/release/prtip <args>
   - Document results in implementation-notes.md

5. Sprint Finalization:
   - Ensure all tasks complete in task-checklist.md
   - /rust-check  # Final validation
   - /sprint-complete 5.X  # Generate comprehensive summary

6. Git Workflow:
   - Review sprint-5.X/implementation-summary.md
   - git commit -F /tmp/ProRT-IP/sprint-5.X-commit-message.txt
   - git push
```

**Task Tracking Pattern:**

```markdown
# task-checklist.md format
- [ ] TASK-1: Implement core functionality
- [ ] TASK-2: Add comprehensive tests
- [x] TASK-3: Document API (completed)
- [ ] TASK-4: Performance optimization
```

## SEE ALSO

- `docs/01-ROADMAP.md` - Project phases and sprint planning
- `docs/10-PROJECT-STATUS.md` - Overall project status and task tracking
- `CLAUDE.local.md` - Recent sessions and current sprint
- `CONTRIBUTING.md` - Development workflow guidelines
- `ref-docs/10-Custom-Commands_Analysis.md` - Command usage patterns

---

**Initialize sprint: $***
