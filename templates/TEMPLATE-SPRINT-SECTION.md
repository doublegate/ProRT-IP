# TEMPLATE: Sprint Section for README.md

**Purpose:** Standard format for sprint details within phase sections  
**Location:** README.md → Phase Section → Current Sprint subsection  
**When to Use:** When updating active sprint progress in README

---

## Template Structure

```markdown
### Current Sprint: [N.M] - [SPRINT_NAME]

**Status:** [IN PROGRESS / PARTIAL / BLOCKED / COMPLETE]  
**Progress:** [X]/[Y] tasks complete ([Z]%)  
**Started:** [START_DATE]  
**Duration:** [ESTIMATED]h (target) | [ACTUAL]h (actual)  
**Grade:** [A+ / A / A- / B+] ([RATING])

#### Sprint Overview

[1-2 paragraph description of sprint goals, scope, and approach]

**Success Criteria:**
- [CRITERION_1]
- [CRITERION_2]
- [CRITERION_3]

**Dependencies:**
- [DEPENDENCY_1] ([STATUS])
- [DEPENDENCY_2] ([STATUS])
or **None**

#### Task Breakdown

**Implementation:** [X]/[Y] complete

| Task | Status | Priority | Estimate | Actual |
|------|--------|----------|----------|--------|
| [TASK_1] | ✅ COMPLETE | High | [X]h | [Y]h |
| [TASK_2] | ✅ COMPLETE | High | [X]h | [Y]h |
| [TASK_3] | 🔄 IN PROGRESS | Medium | [X]h | [Y]h |
| [TASK_4] | ⏳ PENDING | Medium | [X]h | - |
| [TASK_5] | ⏳ PENDING | Low | [X]h | - |

**Total Estimated:** [X]h | **Total Actual:** [Y]h | **Efficiency:** [Z]%

**Current Focus:** [ACTIVE_TASK_DESCRIPTION]

#### Deliverables

**Code:**
- [x] [DELIVERABLE_1] ([X] lines, [Y] tests)
- [x] [DELIVERABLE_2] ([X] lines, [Y] tests)
- [ ] [DELIVERABLE_3] ([X] lines, [Y] tests) - In progress
- [ ] [DELIVERABLE_4] ([X] lines, [Y] tests) - Pending

**Documentation:**
- [x] [DOC_1] ([X] lines)
- [ ] [DOC_2] ([X] lines) - Pending

**Tests:**
- Unit Tests: [X]/[Y] passing
- Integration Tests: [X]/[Y] passing
- Coverage: [X]% (target: [Y]%)

#### Quality Metrics

| Metric | Current | Change | Target | Status |
|--------|---------|--------|--------|--------|
| Tests Passing | [N]/[N] | +[X] | [N]/[N] | ✅ |
| Code Coverage | [X]% | +[Y]pp | ≥[Z]% | [✅ / ⚠️ / ❌] |
| Clippy Warnings | [N] | -[X] | 0 | [✅ / ⚠️ / ❌] |
| Build Time | [X]s | ±[Y]s | <[Z]s | [✅ / ⚠️ / ❌] |
| Binary Size | [X]MB | ±[Y]MB | <[Z]MB | [✅ / ⚠️ / ❌] |

**Performance:**
- [METRIC_1]: [VALUE] ([COMPARISON])
- [METRIC_2]: [VALUE] ([COMPARISON])
- [METRIC_3]: [VALUE] ([COMPARISON])

#### Recent Progress

**[DATE_1] ([DAY_OF_WEEK]):**
- ✅ [COMPLETED_WORK_1]
- ✅ [COMPLETED_WORK_2]
- 🔄 [IN_PROGRESS_WORK]

**[DATE_2] ([DAY_OF_WEEK]):**
- ✅ [COMPLETED_WORK_1]
- ✅ [COMPLETED_WORK_2]

**[DATE_3] ([DAY_OF_WEEK]):**
- ✅ [COMPLETED_WORK_1]
- ⏳ [PLANNED_WORK] (next session)

#### Blockers & Risks

**Current Blockers:**
- [BLOCKER_1] - [MITIGATION]
- [BLOCKER_2] - [MITIGATION]
or **None**

**Risks:**
- [RISK_1]: [PROBABILITY]/[IMPACT] - [MITIGATION]
- [RISK_2]: [PROBABILITY]/[IMPACT] - [MITIGATION]
or **None**

#### Next Steps

**Immediate (Next Session):**
1. [NEXT_TASK_1]
2. [NEXT_TASK_2]
3. [NEXT_TASK_3]

**This Week:**
- [GOAL_1]
- [GOAL_2]
- [GOAL_3]

**Sprint Completion:**
- Estimated: [DATE] ([N] days remaining)
- Confidence: [High / Medium / Low]
- At Risk?: [Yes / No] - [REASON]

#### Documentation

- **Sprint TODO:** [`to-dos/PHASE-[N]/SPRINT-[N.M]-TODO.md`](to-dos/PHASE-[N]/SPRINT-[N.M]-TODO.md)
- **Related Docs:** [`docs/[NN]-[TOPIC]-[TYPE].md`](docs/[NN]-[TOPIC]-[TYPE].md)
- **Architecture:** [`docs/[COMPONENT]-ARCHITECTURE.md`](docs/[COMPONENT]-ARCHITECTURE.md)

**Session Logs:**
- [2025-11-15: Session 1](daily_logs/2025-11-15/) - [SUMMARY]
- [2025-11-16: Session 2](daily_logs/2025-11-16/) - [SUMMARY]

---

**Previous Sprint:** [Sprint [N.M-1]](to-dos/PHASE-[N]/SPRINT-[N.M-1]-COMPLETE.md) ✅  
**Next Sprint:** Sprint [N.M+1] - [NEXT_SPRINT_NAME] (planned start: [DATE])
```

---

## Usage Instructions

### 1. Start New Sprint

When beginning Sprint N.M:

1. **Copy template** to README.md phase section
2. **Replace placeholders:**
   - `[N.M]` - Sprint identifier (e.g., 6.3, 7.1)
   - `[SPRINT_NAME]` - Descriptive name (e.g., "Live Dashboard", "Remote Monitoring")
   - `[START_DATE]` - ISO date (YYYY-MM-DD)
   - `[ESTIMATED]h` - Time estimate from TODO
   - `[X]/[Y]` - Progress fractions

3. **Populate task table:**
   - Copy from SPRINT-N.M-TODO.md
   - Set all to ⏳ PENDING
   - Include time estimates

4. **Set success criteria:**
   - 3-5 measurable criteria
   - Clear pass/fail conditions
   - From sprint TODO file

### 2. Daily Updates

**After Each Work Session:**

1. **Update task status:**
   - Change ⏳ → 🔄 for started tasks
   - Change 🔄 → ✅ for completed tasks
   - Add actual time spent

2. **Update deliverables:**
   - Check [x] for completed items
   - Update line/test counts
   - Add new deliverables if discovered

3. **Add progress entry:**
   - Date in ISO format
   - List completed work with ✅
   - List in-progress work with 🔄
   - Note blockers with ⚠️

4. **Update metrics:**
   - Run test suite, update counts
   - Check coverage, update percentage
   - Run clippy, update warnings
   - Measure performance if relevant

### 3. Sprint Completion

When sprint finishes:

1. **Final metrics:**
   - All tests ✅ passing
   - Final coverage percentage
   - Zero clippy warnings
   - Performance validated

2. **Completion data:**
   - Status → COMPLETE
   - Actual duration filled in
   - Grade assigned (A+, A, A-, B+)
   - Efficiency calculated

3. **Create completion report:**
   - Generate SPRINT-N.M-COMPLETE.md
   - Link from sprint section
   - Archive sprint TODO

4. **Move to next sprint:**
   - Clear "Current Sprint" section
   - Populate with Sprint N.M+1
   - Update phase progress table

---

## Field Definitions

### Status Values

| Status | Meaning | When to Use |
|--------|---------|-------------|
| `PLANNED` | Not yet started | Sprint in future |
| `IN PROGRESS` | Actively working | Most common active state |
| `PARTIAL` | Some tasks complete, significant work remains | Mid-sprint checkpoint |
| `BLOCKED` | Cannot proceed due to blocker | External dependency or issue |
| `COMPLETE` | All tasks finished, quality gates passed | Sprint done |

### Task Status Emojis

| Emoji | Status | Description |
|-------|--------|-------------|
| ✅ | COMPLETE | Task finished and validated |
| 🔄 | IN PROGRESS | Currently working on |
| ⏳ | PENDING | Not yet started |
| ⚠️ | BLOCKED | Cannot proceed |
| ❌ | CANCELLED | Removed from scope |
| 🔄➜✅ | JUST COMPLETED | Completed in current session (use ✅ in next update) |

### Grade Scale

| Grade | Criteria |
|-------|----------|
| **A+** | 100% complete, exceeded targets, zero issues, high efficiency (>90%) |
| **A** | 100% complete, met all targets, minor issues, good efficiency (80-90%) |
| **A-** | 100% complete, met most targets, some issues, acceptable efficiency (70-79%) |
| **B+** | 90-99% complete, met key targets, notable issues, lower efficiency (60-69%) |
| **B** | 80-89% complete, partial targets, significant issues, efficiency <60% |

### Efficiency Calculation

```
Efficiency = (Estimated Hours / Actual Hours) × 100%

Examples:
- Estimated 20h, actual 18h = 111% (under budget, excellent)
- Estimated 20h, actual 20h = 100% (on budget, good)
- Estimated 20h, actual 25h = 80% (over budget, acceptable)
- Estimated 20h, actual 30h = 67% (significantly over, investigate)
```

---

## Examples

### Example 1: Early Sprint (First Day)

```markdown
### Current Sprint: 7.1 - Metrics Infrastructure

**Status:** IN PROGRESS  
**Progress:** 1/7 tasks complete (14%)  
**Started:** 2025-11-15  
**Duration:** 20h (target) | 3h (actual)  
**Grade:** TBD

#### Sprint Overview

Establish foundational metrics infrastructure for real-time monitoring. Implement EventBus for metric distribution, create collectors for all scanner types, and design aggregation layer for statistical analysis.

**Success Criteria:**
- EventBus publishes metrics with <5ms latency
- All 8 scanner types integrated with collectors
- Aggregation supports rolling windows (1s, 5s, 60s)
- 90%+ test coverage for all metric components
- Zero performance regression (<1% overhead)

**Dependencies:**
- None (foundation sprint, no external dependencies)

#### Task Breakdown

**Implementation:** 1/7 complete

| Task | Status | Priority | Estimate | Actual |
|------|--------|----------|----------|--------|
| Design metrics data model | ✅ COMPLETE | High | 3h | 3h |
| Implement EventBus | 🔄 IN PROGRESS | High | 4h | 2h |
| Create metric collectors | ⏳ PENDING | High | 5h | - |
| Add aggregation layer | ⏳ PENDING | Medium | 4h | - |
| Scanner integration | ⏳ PENDING | Medium | 3h | - |
| Metric persistence | ⏳ PENDING | Low | 2h | - |
| Write tests | ⏳ PENDING | High | 4h | - |

**Total Estimated:** 25h | **Total Actual:** 5h | **Efficiency:** TBD

**Current Focus:** Implementing EventBus pub-sub pattern with lock-free queues

[...]
```

### Example 2: Mid-Sprint (Active Development)

```markdown
### Current Sprint: 7.1 - Metrics Infrastructure

**Status:** IN PROGRESS  
**Progress:** 4/7 tasks complete (57%)  
**Started:** 2025-11-15  
**Duration:** 20h (target) | 14h (actual)  
**Grade:** On track for A

#### Sprint Overview

[... same overview ...]

#### Task Breakdown

**Implementation:** 4/7 complete

| Task | Status | Priority | Estimate | Actual |
|------|--------|----------|----------|--------|
| Design metrics data model | ✅ COMPLETE | High | 3h | 3h |
| Implement EventBus | ✅ COMPLETE | High | 4h | 5h |
| Create metric collectors | ✅ COMPLETE | High | 5h | 4h |
| Add aggregation layer | 🔄 IN PROGRESS | Medium | 4h | 2h |
| Scanner integration | ⏳ PENDING | Medium | 3h | - |
| Metric persistence | ⏳ PENDING | Low | 2h | - |
| Write tests | ⏳ PENDING | High | 4h | - |

**Total Estimated:** 25h | **Total Actual:** 14h | **Efficiency:** 54% (14/25, on pace)

**Current Focus:** Implementing aggregation layer with sliding windows and percentile calculations

#### Deliverables

**Code:**
- [x] EventBus implementation (487 lines, 12 tests)
- [x] Metric collector traits (234 lines, 8 tests)
- [x] Scanner metric collectors (612 lines, 15 tests)
- [ ] Aggregation layer (est. 400 lines, 10 tests) - 50% complete
- [ ] Persistence module (est. 200 lines, 5 tests) - Pending

**Documentation:**
- [x] Metrics architecture design (docs/METRICS-ARCHITECTURE.md, 450 lines)
- [ ] User guide (docs/40-METRICS-GUIDE.md, ~600 lines) - Pending

**Tests:**
- Unit Tests: 35/50 passing (70%, target: 90%)
- Integration Tests: 0/5 passing (pending integration)
- Coverage: 78% (target: 90%)

#### Quality Metrics

| Metric | Current | Change | Target | Status |
|--------|---------|--------|--------|--------|
| Tests Passing | 35/35 | +35 | 50/50 | 🔄 |
| Code Coverage | 78% | +78pp | ≥90% | ⚠️ |
| Clippy Warnings | 0 | -3 | 0 | ✅ |
| Build Time | 127s | +12s | <150s | ✅ |
| Binary Size | 8.2MB | +0.3MB | <10MB | ✅ |

**Performance:**
- EventBus latency: 3.2ms (target: <5ms) ✅
- Collector overhead: 0.8% (target: <1%) ✅
- Memory per metric: 48 bytes (acceptable)

#### Recent Progress

**2025-11-17 (Sunday):**
- ✅ Completed EventBus implementation with lock-free queues
- ✅ Fixed 3 clippy warnings in metric collection
- ✅ Added 12 unit tests for EventBus (100% coverage)
- 🔄 Started aggregation layer (sliding window logic)

**2025-11-16 (Saturday):**
- ✅ Implemented all scanner metric collectors
- ✅ Created collector trait abstraction
- ✅ Added 15 integration tests

**2025-11-15 (Friday):**
- ✅ Designed metrics data model (counters, gauges, histograms)
- ⏳ Next: Complete aggregation layer percentile calculations

#### Blockers & Risks

**Current Blockers:** None

**Risks:**
- **Test coverage behind pace**: Medium/Medium - Plan dedicated testing session
- **Aggregation complexity higher than estimated**: Low/Low - May need +2h

#### Next Steps

**Immediate (Next Session):**
1. Complete aggregation layer percentile calculations
2. Add unit tests for aggregation (target: 10 tests)
3. Begin scanner integration

**This Week:**
- Complete all 7 tasks
- Achieve 90% test coverage
- Write user guide draft

**Sprint Completion:**
- Estimated: 2025-11-20 (3 days remaining)
- Confidence: High
- At Risk?: No - On pace, no blockers

[...]
```

### Example 3: Sprint Completion

```markdown
### Sprint 7.1 - Metrics Infrastructure ✅ COMPLETE

**Status:** COMPLETE  
**Progress:** 7/7 tasks complete (100%)  
**Started:** 2025-11-15  
**Duration:** 20h (target) | 22h (actual)  
**Grade:** A (Professional execution, minor efficiency variance)

#### Sprint Overview

[... same overview ...]

#### Final Deliverables

**Code:**
- [x] EventBus implementation (487 lines, 12 tests) ✅
- [x] Metric collector traits (234 lines, 8 tests) ✅
- [x] Scanner metric collectors (612 lines, 15 tests) ✅
- [x] Aggregation layer (445 lines, 11 tests) ✅
- [x] Persistence module (218 lines, 6 tests) ✅

**Documentation:**
- [x] Metrics architecture (docs/METRICS-ARCHITECTURE.md, 450 lines) ✅
- [x] User guide (docs/40-METRICS-GUIDE.md, 624 lines) ✅

**Tests:**
- Unit Tests: 52/52 passing (100%) ✅
- Integration Tests: 5/5 passing (100%) ✅
- Coverage: 91% (exceeded 90% target) ✅

#### Final Quality Metrics

| Metric | Final | Change | Target | Status |
|--------|-------|--------|--------|--------|
| Tests Passing | 57/57 | +57 | 50/50 | ✅ |
| Code Coverage | 91% | +91pp | ≥90% | ✅ |
| Clippy Warnings | 0 | -0 | 0 | ✅ |
| Build Time | 135s | +20s | <150s | ✅ |
| Binary Size | 8.5MB | +0.6MB | <10MB | ✅ |

**Performance (Validated):**
- EventBus latency: 2.8ms (44% under target) ✅
- Collector overhead: 0.6% (40% under target) ✅
- Memory per metric: 48 bytes ✅

#### Completion Summary

**Achievements:**
- 100% task completion (7/7)
- Exceeded test coverage target (91% vs 90%)
- Superior performance (EventBus 2.8ms vs <5ms target)
- Zero clippy warnings maintained
- Comprehensive documentation (1,074 lines)

**Challenges & Solutions:**
- Aggregation complexity underestimated (+2h)
  - Solution: Simplified percentile algorithm, deferred advanced features
- Test coverage initially behind pace (78% mid-sprint)
  - Solution: Dedicated 4h testing session, achieved 91%

**Lessons Learned:**
- Lock-free queues significantly improved EventBus performance
- Trait-based collector design enables easy scanner extension
- Early testing investment prevents late-sprint scrambles

**Efficiency:** 91% (20h estimated / 22h actual)  
**Quality:** A (100% complete, exceeded targets, minor efficiency variance)

---

**Completion Report:** [SPRINT-7.1-COMPLETE.md](to-dos/PHASE-7/SPRINT-7.1-COMPLETE.md) ✅  
**Next Sprint:** 7.2 - Live Dashboard (starts 2025-11-21)
```

---

## Best Practices

### Update Frequency

- **Daily:** Task status, progress entries, blockers
- **After Sessions:** Deliverables, metrics, recent progress
- **Weekly:** Full review, next steps, risk assessment
- **Sprint End:** Completion data, grade, lessons learned

### Quality Standards

1. **Accuracy:** Metrics must match actual state (run tests, don't guess)
2. **Honesty:** Report actual time, don't hide overruns
3. **Completeness:** Fill ALL sections, no TBD placeholders long-term
4. **Links:** Verify all cross-references resolve
5. **Formatting:** Consistent table alignment, emoji usage

### Common Mistakes

❌ Forgetting to update after sessions  
✅ Update immediately while work is fresh

❌ Optimistic task status (marking 🔄 when really ⏳)  
✅ Honest status reflects actual state

❌ Vague "working on X"  
✅ Specific "Implemented JWT authentication middleware (2h)"

❌ Ignoring blockers until critical  
✅ Document early, track mitigation

---

## Maintenance

### Session Checklist

After each work session:
- [ ] Update task status (✅, 🔄, ⏳)
- [ ] Add progress entry with date
- [ ] Update deliverable checkboxes
- [ ] Refresh quality metrics (test count, coverage)
- [ ] Note any new blockers
- [ ] Update "Current Focus"
- [ ] Commit: `docs(readme): Update Sprint N.M progress`

### Weekly Review

Once per week during sprint:
- [ ] Verify all metrics accurate
- [ ] Update efficiency calculation
- [ ] Review risks, update probabilities
- [ ] Refresh "Next Steps"
- [ ] Check links still valid
- [ ] Assess completion confidence
- [ ] Commit: `docs(readme): Sprint N.M weekly review`

### Sprint Completion

When finishing sprint:
- [ ] Mark status COMPLETE
- [ ] Fill final metrics
- [ ] Assign grade
- [ ] Calculate efficiency
- [ ] Write completion summary
- [ ] Create SPRINT-N.M-COMPLETE.md
- [ ] Archive sprint TODO
- [ ] Commit: `docs(readme): Complete Sprint N.M - [NAME]`

---

**Version:** 1.0.0  
**Last Updated:** 2025-11-15  
**Maintained By:** ProRT-IP Documentation Team
