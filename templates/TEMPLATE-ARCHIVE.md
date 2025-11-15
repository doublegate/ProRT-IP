# TEMPLATE: Phase Archive Document

**Purpose:** Standard format for phase archive files in `docs/archive/`  
**Location:** `docs/archive/PHASE-N-README-ARCHIVE.md`  
**When to Use:** When archiving completed phases using `scripts/archive-phase.sh`

---

## Template Structure

```markdown
# ProRT-IP Phase [N] README Archive

**Archive Date:** [YYYY-MM-DD]  
**Archived From:** README.md (root level) + CLAUDE.local.md sprint summaries  
**Phase [N] Status:** ✅ COMPLETE ([X]/[Y] sprints)  
**Final Phase Version:** v[X.Y.Z] (released [YYYY-MM-DD])  
**Phase [N+1] Transition:** v[X.Y.Z+1] (Sprint [N+1].1 [STATUS])  
**Final Tests:** [N],[ N] ([100]% passing)  
**Final Coverage:** [X.Y]% (+[X.Y]pp improvement from [Z.Z]%)  
**Fuzz Testing:** [N]M+ executions, 0 crashes  
**CI/CD Status:** [X]/[Y] workflows passing  
**Release Targets:** [X]/[Y] architectures  
**Total Phase Duration:** [START_DATE] - [END_DATE] ([N] days)  
**Total Development Effort:** ~[X] hours

---

## Purpose

This document archives comprehensive Phase [N] content that has now been superseded by Phase [N+1] development.

**Phase [N] is now complete:**
- **Phase [N]:** [X] sprints ([N.1]-[N.X]) - [PRIMARY_FOCUS]
- **Phase [N+1]:** Sprint [N+1].1 [SPRINT_NAME] marks transition to [NEW_FOCUS]

**For the current README, see:** [`/README.md`](../../README.md)

**For Phase [N+1] planning, see:** [`to-dos/PHASE-[N+1]/`](../../to-dos/PHASE-[N+1]/)

**For Phase [N+1] sprints, see:** `to-dos/PHASE-[N+1]/SPRINT-[N+1].*-TODO.md`

---

## Phase [N] Overview

**Phase [N]** focused on **[PRIMARY_GOAL]** ([START_MONTH]-[END_MONTH] [YEAR]).

**Key Objectives:**
- [OBJECTIVE_1]
- [OBJECTIVE_2]
- [OBJECTIVE_3]
- [OBJECTIVE_4]
- [OBJECTIVE_5]

**Final Status:**
- ✅ **[X] sprints complete** ([N.1]-[N.X], 100% completion)
- ✅ **v[X.Y.Z]-v[X.Y.Z+N] releases** ([N] production releases)
- ✅ **[N],[ N] tests** (100% passing, +[N] from Phase [N-1])
- ✅ **[X.Y]% coverage** (+[X.Y]pp improvement, +[N] coverage tests)
- ✅ **[N]M+ fuzz executions** (0 crashes, [N] fuzz targets)
- ✅ **Zero clippy warnings, zero panics** (production quality maintained)
- ✅ **[X] scan types [FEATURE] complete** ([LIST])
- ✅ **[N]+ code examples** ([DOC_NAME], all categories)
- ✅ **[N],[ N]+ documentation lines** (professional quality)

---

## [🚀 / ⚡ / 🎯] v[X.Y.Z] Release Highlights ([YYYY-MM-DD])

**Phase [N] COMPLETE - [RELEASE_TAGLINE]** ✅

### Sprint [N.1]: [SPRINT_NAME] (~[X]h)

- ✅ **[DELIVERABLE_1]:** [DESCRIPTION]
- ✅ **[DELIVERABLE_2]:** [DESCRIPTION]
- ✅ **[DELIVERABLE_3]:** [DESCRIPTION]
- ✅ **[DELIVERABLE_4]:** [DESCRIPTION]

**Grade:** [A+ / A / A-] ([RATING])

**Strategic Value:**
[1-2 paragraphs explaining sprint's contribution to project goals]

### Sprint [N.2]: [SPRINT_NAME] (~[X]h)

[... repeat structure for each sprint ...]

### Sprint [N.X]: [SPRINT_NAME] (~[X]h)

[... final sprint ...]

---

## Sprint Summary Table

| Sprint | Duration | Key Deliverables | Grade | Status |
|--------|----------|------------------|-------|--------|
| [N.1]: [NAME] | ~[X]h | [DELIVERABLE_LIST] | [A+ / A / A-] | ✅ COMPLETE |
| [N.2]: [NAME] | ~[X]h | [DELIVERABLE_LIST] | [A+ / A / A-] | ✅ COMPLETE |
| [N.3]: [NAME] | ~[X]h | [DELIVERABLE_LIST] | [A+ / A / A-] | ✅ COMPLETE |
| [N.4]: [NAME] | ~[X]h | [DELIVERABLE_LIST] | [A+ / A / A-] | ✅ COMPLETE |
| [N.5]: [NAME] | ~[X]h | [DELIVERABLE_LIST] | [A+ / A / A-] | ✅ COMPLETE |

**Total Effort:** ~[X] hours  
**Average Grade:** [A+ / A / A-]  
**Success Rate:** [X]/[Y] sprints ([Z]%)

---

## Technical Achievements

### Core Features

**[CATEGORY_1]:**
- [FEATURE_1]: [DESCRIPTION] ([SPRINT])
- [FEATURE_2]: [DESCRIPTION] ([SPRINT])
- [FEATURE_3]: [DESCRIPTION] ([SPRINT])

**[CATEGORY_2]:**
- [FEATURE_1]: [DESCRIPTION] ([SPRINT])
- [FEATURE_2]: [DESCRIPTION] ([SPRINT])

**[CATEGORY_3]:**
- [FEATURE_1]: [DESCRIPTION] ([SPRINT])
- [FEATURE_2]: [DESCRIPTION] ([SPRINT])

### Performance Metrics

**Phase [N] Performance:**
- [METRIC_1]: [VALUE] ([COMPARISON])
- [METRIC_2]: [VALUE] ([COMPARISON])
- [METRIC_3]: [VALUE] ([COMPARISON])
- [METRIC_4]: [VALUE] ([COMPARISON])

**Benchmarking:**
- Scan Speed: [VALUE] ([COMPARISON] to Phase [N-1])
- Memory Usage: [VALUE] ([COMPARISON])
- Throughput: [VALUE] ([COMPARISON])

### Quality Improvements

**Testing:**
- Unit Tests: [N] ([+/-]X from Phase [N-1])
- Integration Tests: [N] ([+/-]X from Phase [N-1])
- Coverage: [X.Y]% ([+/-]X.Ypp from [Z.Z]%)
- Fuzz Executions: [N]M+ (0 crashes)

**Code Quality:**
- Clippy Warnings: 0 (maintained)
- Unsafe Code: [N] blocks ([JUSTIFICATION])
- Documentation Coverage: [X]% (public APIs)
- Example Coverage: [N] examples ([N] categories)

### Infrastructure

**CI/CD:**
- Workflows: [X]/[Y] passing
- Platforms: [LIST]
- Release Targets: [X]/[Y] architectures

**Tooling:**
- [TOOL_1]: [DESCRIPTION]
- [TOOL_2]: [DESCRIPTION]
- [TOOL_3]: [DESCRIPTION]

---

## Documentation

### Created in Phase [N]

**Guides ([N] total):**
- [`docs/[NN]-[NAME]-GUIDE.md`](../[NN]-[NAME]-GUIDE.md) ([N] lines) - [DESCRIPTION]
- [`docs/[NN]-[NAME]-GUIDE.md`](../[NN]-[NAME]-GUIDE.md) ([N] lines) - [DESCRIPTION]
- [`docs/[NN]-[NAME]-GUIDE.md`](../[NN]-[NAME]-GUIDE.md) ([N] lines) - [DESCRIPTION]

**Architecture ([N] total):**
- [`docs/[NAME]-ARCHITECTURE.md`](../[NAME]-ARCHITECTURE.md) ([N] lines) - [DESCRIPTION]
- [`docs/[NAME]-ARCHITECTURE.md`](../[NAME]-ARCHITECTURE.md) ([N] lines) - [DESCRIPTION]

**Reference ([N] total):**
- [`docs/[NN]-[NAME]-REFERENCE.md`](../[NN]-[NAME]-REFERENCE.md) ([N] lines) - [DESCRIPTION]

**Total Documentation:** [N],[ N]+ lines

### Sprint Completion Reports

All sprint completion reports available in [`to-dos/PHASE-[N]/`](../../to-dos/PHASE-[N]/):
- [SPRINT-[N.1]-COMPLETE.md](../../to-dos/PHASE-[N]/SPRINT-[N.1]-COMPLETE.md) ✅
- [SPRINT-[N.2]-COMPLETE.md](../../to-dos/PHASE-[N]/SPRINT-[N.2]-COMPLETE.md) ✅
- [SPRINT-[N.3]-COMPLETE.md](../../to-dos/PHASE-[N]/SPRINT-[N.3]-COMPLETE.md) ✅
- [... etc ...]

---

## Lessons Learned

### What Worked Well

1. **[SUCCESS_1]:** [DESCRIPTION]
   - [DETAIL_1]
   - [DETAIL_2]

2. **[SUCCESS_2]:** [DESCRIPTION]
   - [DETAIL_1]
   - [DETAIL_2]

3. **[SUCCESS_3]:** [DESCRIPTION]
   - [DETAIL_1]
   - [DETAIL_2]

### Challenges Overcome

1. **[CHALLENGE_1]:** [DESCRIPTION]
   - **Problem:** [ISSUE]
   - **Solution:** [FIX]
   - **Outcome:** [RESULT]

2. **[CHALLENGE_2]:** [DESCRIPTION]
   - **Problem:** [ISSUE]
   - **Solution:** [FIX]
   - **Outcome:** [RESULT]

### Improvements for Future Phases

1. **[IMPROVEMENT_1]:** [RECOMMENDATION]
2. **[IMPROVEMENT_2]:** [RECOMMENDATION]
3. **[IMPROVEMENT_3]:** [RECOMMENDATION]

---

## Version History

| Version | Release Date | Highlights |
|---------|--------------|------------|
| v[X.Y.Z] | [YYYY-MM-DD] | [SPRINT] - [HIGHLIGHTS] |
| v[X.Y.Z+1] | [YYYY-MM-DD] | [SPRINT] - [HIGHLIGHTS] |
| v[X.Y.Z+2] | [YYYY-MM-DD] | [SPRINT] - [HIGHLIGHTS] |
| ... | ... | ... |

**Total Releases:** [N] production releases  
**Release Cadence:** ~[N] days per release

---

## Migration to Phase [N+1]

### Phase Transition

**Phase [N] → Phase [N+1] Changes:**
- **Focus Shift:** [OLD_FOCUS] → [NEW_FOCUS]
- **New Capabilities:** [CAPABILITY_LIST]
- **Architecture Evolution:** [DESCRIPTION]

**Breaking Changes:**
- [CHANGE_1] (if any)
- [CHANGE_2] (if any)
or **None** (fully backward compatible)

### Phase [N+1] Preview

**Phase [N+1] Goals:**
- [GOAL_1]
- [GOAL_2]
- [GOAL_3]

**Initial Sprint ([N+1].1):** [SPRINT_NAME]  
**Target Completion:** [YYYY-MM-DD] ([N] weeks)

---

## References

### Related Documentation

- **Current README:** [`README.md`](../../README.md)
- **Project Roadmap:** [`docs/01-ROADMAP.md`](../01-ROADMAP.md)
- **Project Status:** [`docs/10-PROJECT-STATUS.md`](../10-PROJECT-STATUS.md)
- **Previous Phase:** [`docs/archive/PHASE-[N-1]-README-ARCHIVE.md`](PHASE-[N-1]-README-ARCHIVE.md)

### Sprint Planning

- **Phase [N] Planning:** [`to-dos/PHASE-[N]/PHASE-[N]-PLANNING.md`](../../to-dos/PHASE-[N]/PHASE-[N]-PLANNING.md)
- **Sprint TODOs:** [`to-dos/PHASE-[N]/SPRINT-*.md`](../../to-dos/PHASE-[N]/)
- **Completion Reports:** [`to-dos/PHASE-[N]/SPRINT-*-COMPLETE.md`](../../to-dos/PHASE-[N]/)

### Guides Created

[Links to all guides created in this phase - see "Documentation" section above]

---

## Statistics

### Code Metrics

- **Total Lines Added:** [N],[ N]+ (production code)
- **Tests Added:** [N] ([N] unit + [N] integration + [N] doc)
- **Documentation Added:** [N],[ N]+ lines
- **Files Modified:** [N] total across [N] sprints
- **Commits:** [N] comprehensive commits

### Time Investment

- **Total Development:** ~[N] hours
- **Average Sprint:** ~[N] hours
- **Shortest Sprint:** [N.M] - [NAME] ([N]h)
- **Longest Sprint:** [N.M] - [NAME] ([N]h)
- **Efficiency:** [X]% average (estimated vs actual)

### Quality Gates

- ✅ All [N] sprints completed successfully
- ✅ [N],[ N]/[N],[ N] tests passing (100%)
- ✅ [X.Y]% code coverage (target: [Z]%)
- ✅ 0 clippy warnings
- ✅ 0 critical bugs
- ✅ [X]/[Y] CI/CD workflows passing
- ✅ [N]M+ fuzz executions, 0 crashes

---

## Summary

Phase [N] successfully delivered [PRIMARY_ACHIEVEMENT] through [X] comprehensive sprints over [N] days. The phase introduced [KEY_FEATURE_1], [KEY_FEATURE_2], and [KEY_FEATURE_3], while maintaining [X]% test coverage and zero quality regressions.

**Key Accomplishments:**
- ✅ [ACCOMPLISHMENT_1]
- ✅ [ACCOMPLISHMENT_2]
- ✅ [ACCOMPLISHMENT_3]
- ✅ [ACCOMPLISHMENT_4]

**Strategic Impact:**
[1-2 paragraphs describing how Phase [N] advanced the project toward its ultimate goals]

**Looking Forward:**
Phase [N+1] builds on this foundation to deliver [NEXT_PHASE_GOAL], beginning with Sprint [N+1].1 ([SPRINT_NAME]).

---

**Archive Maintained By:** ProRT-IP Documentation Team  
**Archive Format Version:** 1.0.0  
**Last Updated:** [YYYY-MM-DD] (archive creation)  
**Source:** README.md Phase [N] section (2025-11-XX version)

**For current project status, see:** [`README.md`](../../README.md)
```

---

## Usage Instructions

### 1. Automated Archive Creation

**Primary Method:** Use automation script

```bash
# Run archive script (dry-run first)
./scripts/archive-phase.sh [PHASE_NUMBER] --dry-run

# Review output, then execute
./scripts/archive-phase.sh [PHASE_NUMBER]

# Script will:
# - Extract phase content from README.md
# - Generate archive with header
# - Create docs/archive/PHASE-N-README-ARCHIVE.md
# - Prompt for manual README.md updates
```

### 2. Manual Archive Creation

**If script unavailable:**

1. **Copy template** to `docs/archive/PHASE-N-README-ARCHIVE.md`
2. **Replace ALL placeholders** (use editor find/replace)
3. **Extract content** from README.md Phase N section
4. **Paste after** "Phase N Overview" section
5. **Fill header values** from CLAUDE.local.md
6. **Add sprint summaries** from completion reports
7. **Verify formatting** (tables, links, emojis)
8. **Run link check** to validate cross-references

### 3. Post-Archive Steps

After creating archive:

1. **Update README.md:**
   - Replace detailed Phase N section with single line:
   ```markdown
   **Phase N:** ✅ COMPLETE - See [`docs/archive/PHASE-N-README-ARCHIVE.md`](docs/archive/PHASE-N-README-ARCHIVE.md) for detailed history
   ```

2. **Verify Links:**
   ```bash
   # Test all links in archive
   markdown-link-check docs/archive/PHASE-N-README-ARCHIVE.md
   
   # Test updated README
   markdown-link-check README.md
   ```

3. **Commit Changes:**
   ```bash
   git add docs/archive/PHASE-N-README-ARCHIVE.md README.md
   git commit -m "docs: Archive Phase N to historical reference

   - Move Phase N details to docs/archive/PHASE-N-README-ARCHIVE.md
   - Update README.md with archive link
   - Preserve all sprint completion reports
   - Verified all cross-references resolve

   Phase N completed N sprints over N days (~N hours total).
   All quality gates passed (N tests, X% coverage, 0 warnings).
   "
   ```

---

## Placeholder Reference

### Header Placeholders

| Placeholder | Type | Example | Source |
|-------------|------|---------|--------|
| `[N]` | Integer | `7` | Phase number |
| `[YYYY-MM-DD]` | Date | `2025-11-15` | Current date |
| `[X]/[Y]` | Ratio | `8/8` | Sprint count |
| `v[X.Y.Z]` | Version | `v0.7.5` | Git tag |
| `[N],[ N]` | Integer | `2,361` | Large numbers (with commas) |
| `[X.Y]%` | Percentage | `67.3%` | Decimal percentage |
| `[N]M+` | Number | `450M+` | Million-scale metrics |

### Content Placeholders

| Placeholder | Type | Example |
|-------------|------|---------|
| `[SPRINT_NAME]` | String | `Live Dashboard` |
| `[DESCRIPTION]` | Paragraph | `Implements real-time TUI...` |
| `[OBJECTIVE_N]` | Bullet | `100% IPv6 coverage` |
| `[DELIVERABLE_N]` | Bullet | `EventBus implementation` |
| `[A+ / A / A-]` | Grade | `A` |
| `[METRIC_NAME]` | String | `Code Coverage` |
| `[VALUE]` | Mixed | `91%`, `2.8ms`, `10M pps` |

### Status/Emoji Placeholders

| Placeholder | Values | Example |
|-------------|--------|---------|
| `[STATUS]` | PLANNED, IN PROGRESS, COMPLETE | `COMPLETE` |
| Status Emoji | ✅, 🔄, 📋, ⏳, ⚠️, ❌ | `✅ COMPLETE` |
| Category Emoji | 🚀, ⚡, 🎯, 🔒, 📊, 🧪 | `🚀 Launch` |

---

## Quality Checklist

Before finalizing archive:

**Content Completeness:**
- [ ] All [PLACEHOLDER] values replaced
- [ ] Header metrics accurate (from CLAUDE.local.md)
- [ ] All sprint summaries included
- [ ] Sprint table complete and accurate
- [ ] Documentation links point to correct files
- [ ] Version history table populated
- [ ] Lessons learned section completed

**Formatting:**
- [ ] Tables properly aligned
- [ ] Emoji usage consistent
- [ ] Links use relative paths
- [ ] Code blocks properly fenced
- [ ] Lists use consistent bullets
- [ ] Headers use proper hierarchy (##, ###, ####)

**Cross-References:**
- [ ] All internal links resolve
- [ ] Links to completion reports work
- [ ] Links to guides/docs valid
- [ ] README.md updated with archive link
- [ ] No broken links (run markdown-link-check)

**Quality:**
- [ ] Archive >1,000 lines (comprehensive)
- [ ] No spelling/grammar errors
- [ ] Technical accuracy verified
- [ ] Metrics match actual values
- [ ] Ready for long-term reference

---

## Examples

See actual phase archives for reference:

- [`docs/archive/PHASE-4-README-ARCHIVE.md`](../archive/PHASE-4-README-ARCHIVE.md)
- [`docs/archive/PHASE-5-README-ARCHIVE.md`](../archive/PHASE-5-README-ARCHIVE.md)
- [`docs/archive/PHASE-6-README-ARCHIVE.md`](../archive/PHASE-6-README-ARCHIVE.md)

---

**Version:** 1.0.0  
**Last Updated:** 2025-11-15  
**Maintained By:** ProRT-IP Documentation Team  
**Related:** `scripts/archive-phase.sh`, `TEMPLATE-PHASE-SECTION.md`
