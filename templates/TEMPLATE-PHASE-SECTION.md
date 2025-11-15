# TEMPLATE: Phase Section for README.md

**Purpose:** Standard format for active phase sections in root README.md  
**Location:** README.md → Development Progress → Active Phase  
**When to Use:** When starting a new phase or updating current phase status

---

## Template Structure

```markdown
## Phase [N]: [PHASE NAME] 🔄 IN PROGRESS

**Status:** Sprint [N.M] [STATUS] ([X]/[Y] sprints, [Z]%)  
**Duration:** [START_DATE] - [CURRENT_DATE] ([N] days)  
**Current Version:** v[X.Y.Z]

### Phase Overview

[1-2 paragraph description of phase goals and scope]

**Key Objectives:**
- [PRIMARY_OBJECTIVE_1]
- [PRIMARY_OBJECTIVE_2]
- [PRIMARY_OBJECTIVE_3]
- [PRIMARY_OBJECTIVE_4]

**Strategic Value:**
[1 paragraph explaining why this phase matters for the project's long-term goals]

### Sprint Progress

| Sprint | Status | Duration | Key Deliverables |
|--------|--------|----------|------------------|
| [N.1]: [NAME] | ✅ COMPLETE | ~[X]h | [DELIVERABLE_1], [DELIVERABLE_2] |
| [N.2]: [NAME] | ✅ COMPLETE | ~[X]h | [DELIVERABLE_1], [DELIVERABLE_2] |
| [N.3]: [NAME] | 🔄 IN PROGRESS | ~[X]h | [DELIVERABLE_1], [DELIVERABLE_2] |
| [N.4]: [NAME] | 📋 PLANNED | ~[X]h | [DELIVERABLE_1], [DELIVERABLE_2] |

**Completed:** [X]/[Y] sprints ([Z]%)  
**Estimated Completion:** [TARGET_DATE] ([WEEKS] weeks remaining)

### Current Sprint: [N.M] - [SPRINT_NAME]

**Status:** [IN PROGRESS / PARTIAL / BLOCKED]  
**Progress:** [X]/[Y] tasks complete ([Z]%)  
**Started:** [START_DATE]

**Tasks:**
- ✅ [COMPLETED_TASK_1]
- ✅ [COMPLETED_TASK_2]
- 🔄 [IN_PROGRESS_TASK] (current)
- ⏳ [PENDING_TASK_1]
- ⏳ [PENDING_TASK_2]

**Blockers:** [NONE / LIST_BLOCKERS]

**Recent Achievements:**
- [ACHIEVEMENT_1] - [BRIEF_DESCRIPTION]
- [ACHIEVEMENT_2] - [BRIEF_DESCRIPTION]
- [ACHIEVEMENT_3] - [BRIEF_DESCRIPTION]

### Quality Metrics

| Metric | Value | Change | Target |
|--------|-------|--------|--------|
| Tests Passing | [N]/[N] ([100]%) | +[X] | 100% |
| Code Coverage | [X.Y]% | +[X.Y]pp | ≥[TARGET]% |
| Clippy Warnings | [N] | -[X] | 0 |
| CI/CD Status | [X]/[Y] passing | - | [Y]/[Y] |
| Fuzz Executions | [N]M+ | +[X]M | 0 crashes |

**Performance:**
- [METRIC_1]: [VALUE] ([CHANGE] from Phase [N-1])
- [METRIC_2]: [VALUE] ([CHANGE] from Phase [N-1])
- [METRIC_3]: [VALUE] ([CHANGE] from Phase [N-1])

### Key Features Delivered

**New Capabilities:**
1. **[FEATURE_1]:** [DESCRIPTION] ([SPRINT])
2. **[FEATURE_2]:** [DESCRIPTION] ([SPRINT])
3. **[FEATURE_3]:** [DESCRIPTION] ([SPRINT])

**Improvements:**
- [IMPROVEMENT_1]
- [IMPROVEMENT_2]
- [IMPROVEMENT_3]

**Documentation:**
- [DOC_1] ([X] lines)
- [DOC_2] ([X] lines)
- [DOC_3] ([X] lines)

### Upcoming Work

**Next Sprint ([N.M+1]):** [SPRINT_NAME]  
**Estimated Duration:** ~[X]h ([Y] days)  
**Key Deliverables:**
- [DELIVERABLE_1]
- [DELIVERABLE_2]
- [DELIVERABLE_3]

**Phase [N] Remaining:**
- Sprint [N.M+2]: [NAME] (~[X]h)
- Sprint [N.M+3]: [NAME] (~[X]h)
- Sprint [N.M+4]: [NAME] (~[X]h)

**Estimated Phase Completion:** [DATE] ([WEEKS] weeks)

### Documentation

- **Planning:** [`to-dos/PHASE-[N]/PHASE-[N]-PLANNING.md`](to-dos/PHASE-[N]/PHASE-[N]-PLANNING.md)
- **Current Sprint:** [`to-dos/PHASE-[N]/SPRINT-[N.M]-TODO.md`](to-dos/PHASE-[N]/SPRINT-[N.M]-TODO.md)
- **Architecture:** [`docs/[NN]-[COMPONENT]-ARCHITECTURE.md`](docs/[NN]-[COMPONENT]-ARCHITECTURE.md)
- **User Guide:** [`docs/[NN]-[FEATURE]-GUIDE.md`](docs/[NN]-[FEATURE]-GUIDE.md)

**Completion Reports:**
- [Sprint [N.1]](to-dos/PHASE-[N]/SPRINT-[N.1]-COMPLETE.md) ✅
- [Sprint [N.2]](to-dos/PHASE-[N]/SPRINT-[N.2]-COMPLETE.md) ✅

---

**For detailed phase history, see:** [`docs/archive/PHASE-[N-1]-README-ARCHIVE.md`](docs/archive/PHASE-[N-1]-README-ARCHIVE.md)  
**For project roadmap, see:** [`docs/01-ROADMAP.md`](docs/01-ROADMAP.md)  
**For current status, see:** [`docs/10-PROJECT-STATUS.md`](docs/10-PROJECT-STATUS.md)
```

---

## Usage Instructions

### 1. Create New Phase Section

When starting Phase N:

1. **Copy template above** to README.md appropriate location
2. **Replace ALL placeholders:**
   - `[N]` - Phase number (e.g., 7, 8, 9)
   - `[PHASE NAME]` - Descriptive name (e.g., "Advanced Features", "Production Hardening")
   - `[N.M]` - Sprint number (e.g., 7.1, 7.2)
   - `[STATUS]` - Sprint status (PLANNED, IN PROGRESS, COMPLETE)
   - `[X]`, `[Y]`, `[Z]` - Numeric values
   - `[DATE]` - ISO format (YYYY-MM-DD) or relative (e.g., "Nov 15, 2025")
   - `[DESCRIPTION]` - 1-2 sentence descriptions

3. **Update sprint table:**
   - Add all planned sprints with estimates
   - Use status emojis: ✅ COMPLETE, 🔄 IN PROGRESS, 📋 PLANNED
   - Include realistic duration estimates

4. **Fill metrics table:**
   - Use actual values from CLAUDE.local.md
   - Include change from previous phase
   - Set appropriate targets

### 2. Update During Sprint Execution

**Daily/Weekly Updates:**
- Update "Current Sprint" task checklist (✅, 🔄, ⏳)
- Update progress percentage
- Add recent achievements as completed
- Update quality metrics (tests, coverage, warnings)

**Sprint Completion:**
- Change sprint status to ✅ COMPLETE
- Update duration with actual time
- Add completion report link
- Move to next sprint section

**Phase Completion:**
- Change status to ✅ COMPLETE
- Add final metrics summary
- Archive to `docs/archive/PHASE-N-README-ARCHIVE.md`
- Replace with single-line archive link in README

### 3. Section Placement

**In README.md:**
```
# ProRT-IP

[... project header ...]

## Development Progress

### Current Status
[... high-level status ...]

### Active Phase

[👉 INSERT PHASE SECTION HERE 👈]

### Completed Phases
- **Phase 1-5:** ✅ COMPLETE - See archives
- **Phase 6:** ✅ COMPLETE - See [docs/archive/PHASE-6-README-ARCHIVE.md](...)

[... rest of README ...]
```

---

## Field Definitions

### Required Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `[N]` | Integer | Phase number | `7` |
| `[PHASE NAME]` | String | Descriptive phase name | `Advanced Features` |
| `[N.M]` | Float | Sprint identifier | `7.2` |
| `[STATUS]` | Enum | Sprint status | `IN PROGRESS` |
| `[X]/[Y]` | Ratio | Progress fraction | `3/8` (3 of 8 sprints) |
| `[Z]%` | Percentage | Completion percentage | `38%` |
| `[START_DATE]` | Date | Phase/sprint start | `2025-11-15` |
| `[CURRENT_DATE]` | Date | Current date | `2025-11-20` |
| `v[X.Y.Z]` | Version | Semantic version | `v0.6.0` |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `[BLOCKERS]` | List | Current blockers or "NONE" |
| `[IMPROVEMENT_N]` | String | Infrastructure improvements |
| `[METRIC_N]` | String | Performance/quality metrics |
| `[TARGET]` | Number | Quality target value |

---

## Examples

### Example 1: Early Phase (First Sprint)

```markdown
## Phase 7: Real-Time Monitoring 🔄 IN PROGRESS

**Status:** Sprint 7.1 IN PROGRESS (1/6 sprints, 17%)  
**Duration:** 2025-11-15 - 2025-11-20 (5 days)  
**Current Version:** v0.6.0

### Phase Overview

Phase 7 focuses on adding real-time monitoring and observability to ProRT-IP, enabling users to track scan progress, view live metrics, and analyze performance during execution.

**Key Objectives:**
- Real-time metrics dashboard with 60 FPS rendering
- Live scan progress tracking with ETAs
- Performance profiling and bottleneck identification
- WebSocket API for remote monitoring
- Prometheus metrics export

**Strategic Value:**
Phase 7 transforms ProRT-IP from a batch scanner to an observable system, enabling operators to make informed decisions during long-running scans and identify performance issues in real-time.

### Sprint Progress

| Sprint | Status | Duration | Key Deliverables |
|--------|--------|----------|------------------|
| 7.1: Metrics Infrastructure | 🔄 IN PROGRESS | ~20h | EventBus, metric collectors, aggregation |
| 7.2: Live Dashboard | 📋 PLANNED | ~25h | Terminal UI, widget system, rendering |
| 7.3: Remote Monitoring | 📋 PLANNED | ~18h | WebSocket API, client library |
| 7.4: Performance Profiling | 📋 PLANNED | ~22h | CPU/memory profiling, bottleneck detection |
| 7.5: Metrics Export | 📋 PLANNED | ~15h | Prometheus integration, Grafana dashboards |
| 7.6: Documentation | 📋 PLANNED | ~12h | User guides, API docs, examples |

**Completed:** 0/6 sprints (0%)  
**Estimated Completion:** 2025-12-20 (5 weeks remaining)

### Current Sprint: 7.1 - Metrics Infrastructure

**Status:** IN PROGRESS  
**Progress:** 3/7 tasks complete (43%)  
**Started:** 2025-11-15

**Tasks:**
- ✅ Design metrics data model (counters, gauges, histograms)
- ✅ Implement EventBus with pub-sub pattern
- ✅ Create metric collectors for scanners
- 🔄 Add aggregation layer (rolling windows, percentiles) (current)
- ⏳ Integrate with existing scanners
- ⏳ Add metric persistence (SQLite)
- ⏳ Write comprehensive tests (target: 90% coverage)

**Blockers:** None

**Recent Achievements:**
- EventBus implementation with <5ms publish latency
- Zero-copy metric collection (no allocations in hot path)
- Thread-safe aggregation with lock-free queues

[... rest of section ...]
```

### Example 2: Mid-Phase (Multiple Sprints Complete)

```markdown
## Phase 7: Real-Time Monitoring 🔄 IN PROGRESS

**Status:** Sprint 7.3 IN PROGRESS (2/6 sprints, 33%)  
**Duration:** 2025-11-15 - 2025-12-05 (20 days)  
**Current Version:** v0.6.2

[... similar structure with updated values ...]

### Sprint Progress

| Sprint | Status | Duration | Key Deliverables |
|--------|--------|----------|------------------|
| 7.1: Metrics Infrastructure | ✅ COMPLETE | 22h | EventBus, collectors, aggregation (18 tests) |
| 7.2: Live Dashboard | ✅ COMPLETE | 28h | TUI framework, 8 widgets, 60 FPS (35 tests) |
| 7.3: Remote Monitoring | 🔄 IN PROGRESS | ~18h | WebSocket API, client library |
| 7.4: Performance Profiling | 📋 PLANNED | ~22h | CPU/memory profiling, detection |
| 7.5: Metrics Export | 📋 PLANNED | ~15h | Prometheus, Grafana dashboards |
| 7.6: Documentation | 📋 PLANNED | ~12h | Guides, API docs, examples |

**Completed:** 2/6 sprints (33%)  
**Estimated Completion:** 2025-12-20 (2 weeks remaining)

[...]
```

---

## Best Practices

### Content Guidelines

1. **Be Specific:** Use exact numbers, dates, and deliverables
2. **Stay Current:** Update at least weekly during active development
3. **Show Progress:** Use visual indicators (✅, 🔄, ⏳, 📋)
4. **Link Liberally:** Connect to detailed docs and completion reports
5. **Quantify Impact:** Include metrics, test counts, coverage changes

### Formatting Standards

- **Emojis:** Use sparingly for status only (not in descriptions)
- **Tables:** Align columns, use consistent formatting
- **Lists:** Bullet points for parallel items, numbered for sequential
- **Links:** Always use relative paths from repository root
- **Dates:** ISO format (YYYY-MM-DD) or clear relative ("5 days ago")

### Common Mistakes to Avoid

❌ Vague descriptions: "Working on features"  
✅ Specific: "Implementing WebSocket authentication with JWT tokens"

❌ Stale data: "Updated 3 weeks ago"  
✅ Current: Update with each significant change

❌ Broken links: Linking to renamed/moved files  
✅ Verified: Run markdown link check before committing

❌ Missing context: Just listing tasks  
✅ Contextual: Explain why each task matters

---

## Maintenance

### Weekly Review

During active phase development:
- [ ] Update task checkboxes (✅, 🔄, ⏳)
- [ ] Refresh quality metrics
- [ ] Add recent achievements
- [ ] Update progress percentages
- [ ] Verify all links resolve
- [ ] Commit with: `docs(readme): Update Phase N sprint progress`

### Sprint Transition

When completing a sprint:
- [ ] Mark sprint ✅ COMPLETE in table
- [ ] Add actual duration (vs estimate)
- [ ] Link to completion report
- [ ] Clear "Current Sprint" section
- [ ] Populate new "Current Sprint" from next row
- [ ] Update phase completion percentage
- [ ] Commit with: `docs(readme): Complete Sprint N.M - [SPRINT_NAME]`

### Phase Completion

When phase finishes:
- [ ] Mark all sprints ✅ COMPLETE
- [ ] Update final metrics
- [ ] Archive using `scripts/archive-phase.sh`
- [ ] Replace section with single archive link
- [ ] Update "Completed Phases" list
- [ ] Commit with: `docs(readme): Archive Phase N - [PHASE_NAME] COMPLETE`

---

**Version:** 1.0.0  
**Last Updated:** 2025-11-15  
**Maintained By:** ProRT-IP Documentation Team
