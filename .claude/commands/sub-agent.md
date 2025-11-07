---
🤖 **SUB-AGENT COMMAND** - ALWAYS generates a Task tool call for autonomous execution
---

**CRITICAL:** This command MUST use the Task tool to spawn a separate sub-agent that runs independently. NEVER execute the user's request in the main conversation - ALWAYS delegate to a sub-agent.

## Your Task (Delegate to Sub-Agent)

**User Request:** $*

**Required Action:** Use the Task tool with appropriate subagent_type to execute this request autonomously in a separate sub-agent context.

---

## SUB-AGENT SELECTION GUIDE

Choose the appropriate subagent_type based on the task:

### 1. **general-purpose** (Most Common)
**Use for:** Complex multi-step tasks, code implementation, documentation generation, testing, releases
**Example tasks:**
- Sprint execution and completion
- Feature implementation with testing
- Comprehensive documentation updates
- Release preparation and execution
- Performance optimization tasks
- Bug fixing across multiple files

### 2. **Explore** (Codebase Discovery)
**Use for:** Understanding codebase structure, finding implementations, analyzing patterns
**Example tasks:**
- "Where are errors handled?"
- "How does the rate limiting work?"
- "What is the project structure?"
- "Find all tests related to X"

### 3. **Plan** (Strategy Development)
**Use for:** Planning complex implementations, designing architecture, creating roadmaps
**Example tasks:**
- Sprint planning and task breakdown
- Architecture design decisions
- Multi-phase project planning
- Risk assessment and mitigation strategies

## TASK TOOL USAGE PATTERN

```
Use the Task tool with this structure:

Task(
  subagent_type="general-purpose",  // or "Explore" or "Plan"
  description="Brief 3-5 word description",
  prompt="COMPREHENSIVE DETAILED PROMPT WITH:
    1. Clear objective and success criteria
    2. All context needed (read CLAUDE.md, CLAUDE.local.md, relevant files)
    3. Specific files/directories to examine
    4. Expected deliverables with detail level
    5. Quality standards to meet
    6. Verification requirements
    7. Documentation update requirements
    8. MCP server/command usage if applicable
    9. All guidance from PROJECT CONTEXT section below
  "
)
```

## ENHANCED PROMPT CONSTRUCTION

Your sub-agent prompt should include:

### 1. **UltraThink Directive** (for complex tasks)
```
ultrathink: [Task description requiring deep analysis and planning]
```
**Use for:** Complex multi-step tasks requiring strategic thinking, architecture decisions, comprehensive analysis

### 2. **MCP Server/Command Integration**
```
utilize any MCP servers/commands available to best complete this tasking
(these MCP servers/commands should be run in this new separate sub-agent
tool task, as well)
```
**Available MCP capabilities:**
- GitHub operations (code search, file operations, issues, PRs)
- Knowledge graph (entities, relations, observations)
- Web search and documentation lookup
- File system operations
- Sequential thinking for complex problems

### 3. **Reference Documentation**
```
as guidance, analyze/use the Markdown documents in:
- to-dos/ (sprint plans, phase development plans)
- docs/ (architecture, guides, project status)
- CLAUDE.md (project guidelines)
- CLAUDE.local.md (current state and decisions)
```

### 4. **Quality Standards**
Include all standards from PROJECT CONTEXT section below

---

## PROJECT CONTEXT (ProRT-IP WarScan)
**Type:** Rust network scanner combining Masscan speed with Nmap detection depth
**Version:** v0.4.7 (Sprint 5.7 COMPLETE - Fuzz Testing Infrastructure)
**Status:** Phase 5 IN PROGRESS (70% complete, Sprints 5.1-5.7 COMPLETE)
**Tests:** 1,754 passing (100% success rate)
**Coverage:** 54.92% (automated CI/CD tracking)
**Fuzz Testing:** 230M+ executions, zero crashes, 5 targets
**Architecture:** 4 crates (core, network, scanner, cli) - production-ready

## QUALITY STANDARDS (Apply to ALL work)
✅ **Code:** cargo fmt + clippy passing, zero warnings, tests passing
✅ **Documentation:** Technical accuracy, consistent formatting, no broken links
✅ **Commits:** Comprehensive messages with context, impact, and file lists
✅ **Memory Banks:** Keep updated with decisions, progress, and metrics
✅ **File Organization:** Follow project conventions (mixed-case, numerical prefixes)

## SYSTEMATIC APPROACH (Work Through Phases)
1. **ANALYZE** - Read relevant files, understand current state, identify scope
2. **PLAN** - Break down into subtasks, identify dependencies, estimate effort
3. **EXECUTE** - Implement systematically, test as you go, document decisions
4. **VERIFY** - Test thoroughly, check cross-references, validate no regressions
5. **DOCUMENT** - Update README/CHANGELOG/memory banks, create deliverables report
6. **REPORT** - Provide comprehensive summary with metrics, changes, and next steps

## COMMUNICATION GUIDELINES
- **Progress Updates:** Report after each major phase (not just at end)
- **Decisions:** Explain rationale for significant choices
- **Issues:** Report blockers immediately, suggest solutions
- **Questions:** Ask for clarification if requirements unclear
- **Metrics:** Include concrete numbers (files changed, tests added, performance gains)

## DOCUMENTATION REQUIREMENTS
- **Update CHANGELOG.md** if changes affect user/behavior/features
- **Update README.md** if changes affect statistics/features/usage
- **Update CLAUDE.local.md** with session summary, metrics, decisions
- **Create README files** for new directories (benchmarks/, bug_fix/ subdirs)
- **Log major decisions** in appropriate memory bank

## VERIFICATION CHECKLIST (Before Completion)
✅ All subtasks completed successfully
✅ Tests passing (if code changed)
✅ Documentation updated (if user-facing changes)
✅ Memory banks updated with session info
✅ No TODO/FIXME/WIP markers left uncommitted
✅ Cross-references validated (links, paths, file references)
✅ Git history clean (proper git mv for renames)
✅ No temporary files left in /tmp/

## DELIVERABLES (Always Provide)
1. **Summary Report:** What was accomplished, metrics, key decisions
2. **File Changes Log:** Files created/modified/deleted with line counts
3. **Issues Encountered:** Problems found and how resolved
4. **Verification Results:** Test results, validation checks performed
5. **Next Steps:** Recommendations for follow-up work (if applicable)
6. **Git Status:** Ready to commit? Any conflicts? Staging status

## ERROR HANDLING
- **Compilation Errors:** Report exact error, affected file, suggested fix
- **Test Failures:** Report which tests, failure details, root cause analysis
- **Linting Issues:** Fix automatically if possible, else report with solutions
- **Broken References:** Fix or report with list of broken links
- **Unclear Requirements:** Ask for clarification before proceeding
- **Blockers:** Report immediately with impact assessment and suggested workarounds

## EFFICIENCY GUIDELINES
- **Read Before Write:** Always read files before editing (understand context)
- **Batch Operations:** Use git mv for multiple renames in single session
- **Parallel Work:** When independent tasks exist, document all, then execute
- **Avoid Redundancy:** Check if work already done (grep, read existing docs)
- **Preserve History:** Always use git mv (never delete + create for renames)
- **Smart Skipping:** If only docs changed, skip code quality checks

## MEMORY BANK UPDATES (Critical)
**Add to CLAUDE.local.md after completion:**
- **Date:** Current date (YYYY-MM-DD)
- **Task:** Brief description of work completed
- **Duration:** Approximate time spent
- **Key Decisions:** Important choices made and rationale
- **Metrics:** Files changed, tests added, performance improvements
- **Issues:** Problems encountered and resolutions
- **Status:** Current project state after changes
- **Next Actions:** Recommended follow-up tasks

## PROJECT-SPECIFIC NOTES
- **Benchmarks:** Organize by Phase/Sprint (01-Phase4_PreFinal-Bench/, etc.)
- **Bug Fixes:** Categorize by issue (01-Service-Detection/, 02-Progress-Bar/, etc.)
- **Documentation:** MAJOR docs in docs/ root (ALL CAPS), historical in docs/archive/
- **Naming:** Mixed-case with numerical prefixes (01-Hyperfine-1K-Ports.json)
- **Testing:** Run full suite only if code changed, else smoke test
- **Performance:** Include metrics in reports (X% faster, Y seconds → Z seconds)

## SUCCESS CRITERIA (Must Meet ALL)
✅ Task completed as specified (100% of requirements)
✅ No regressions introduced (tests still passing)
✅ Documentation comprehensive and accurate
✅ Memory banks updated with session
✅ All deliverables provided
✅ Clean git state (no conflicts, proper staging)
✅ Ready for user review/commit

## EXAMPLE SUB-AGENT INVOCATIONS

### Example 1: Sprint Execution (Complex Multi-Step)
```
Task(
  subagent_type="general-purpose",
  description="Complete Sprint 5.8 work",
  prompt="ultrathink: execute and complete Sprint 5.8 Plugin System -->

  as guidance, analyze/use the Markdown Todo documents in 'to-dos/' as a
  reference (especially SPRINT-5.8-TODO.md) --> utilize any MCP
  servers/commands available to best complete this tasking

  Apply all PROJECT CONTEXT, QUALITY STANDARDS, and SYSTEMATIC APPROACH
  guidelines from the sub-agent command template.

  Expected deliverables:
  - Plugin architecture implementation
  - 100+ new tests
  - Comprehensive documentation
  - Updated CHANGELOG.md and README.md
  - Memory bank updates with session summary

  Use the 6-phase systematic approach and provide comprehensive final report."
)
```

### Example 2: Documentation Updates
```
Task(
  subagent_type="general-purpose",
  description="Update project documentation",
  prompt="fully add/update/modify/enhance the README.md (at the root-level)
  and CHANGELOG.md with current, up-to-date information --> along with any
  additional Markdown documents in 'docs/' and 'to-dos/' that need
  additions/modifications/enhancements (check them ALL)

  Apply all PROJECT CONTEXT, QUALITY STANDARDS, and VERIFICATION CHECKLIST
  guidelines.

  Expected deliverables:
  - Updated README.md with current metrics
  - CHANGELOG.md with recent changes
  - All docs/ files reviewed and updated
  - Broken links fixed
  - Comprehensive change summary"
)
```

### Example 3: Release Preparation
```
Task(
  subagent_type="general-purpose",
  description="Prepare v0.4.8 release",
  prompt="check the project's formatting/linting and fix anything needed -->
  update all of the project's crates and source code files with a version
  change to v0.4.8 --> stage and commit all local project files --> finally,
  push the local project to the remote GH repo w/ v0.4.8 release tag/notes
  (with robust - comprehensive and technically detailed release notes)

  utilize any MCP servers/commands needed to best complete all of this tasking

  Apply all PROJECT CONTEXT, QUALITY STANDARDS, and DOCUMENTATION REQUIREMENTS.

  Follow the 6-phase SYSTEMATIC APPROACH and provide full DELIVERABLES report."
)
```

### Example 4: CI/CD Optimization
```
Task(
  subagent_type="general-purpose",
  description="Optimize CI/CD pipeline",
  prompt="ultrathink: analyze and optimize the GH Workflow Actions to create
  a better CI/CD pipeline by chaining together separate Actions (with
  conditional logic) to determine when to run, etc. and optimize the use of
  caches across the entire Workflow

  Overall Goal: reduce the amount of time it takes upon each push and/or release

  utilize any MCP servers/commands needed

  Apply all PROJECT CONTEXT and QUALITY STANDARDS guidelines.

  Provide comprehensive report with timing improvements and metrics."
)
```

### Example 5: Code Exploration (Use Explore subagent)
```
Task(
  subagent_type="Explore",
  description="Find rate limiting implementation",
  prompt="Where and how is rate limiting implemented in the codebase?

  Analyze the architecture, key files, and algorithms used. Provide detailed
  explanation with file references and line numbers."
)
```

## CRITICAL ENFORCEMENT

🚨 **MANDATORY:** This command MUST invoke the Task tool. Do NOT:
- Execute the user's request directly in the main conversation
- Provide analysis without launching a sub-agent
- Skip the Task tool invocation
- Attempt to handle complex tasks inline

✅ **CORRECT BEHAVIOR:**
1. Read and understand user request
2. Select appropriate subagent_type
3. Construct comprehensive prompt with all context
4. Invoke Task tool with proper parameters
5. Let the sub-agent execute autonomously
6. Report sub-agent results to user when complete

## FINAL REMINDERS
⚠️ **ALWAYS use Task tool** - this command NEVER executes inline
⚠️ **ALWAYS provide comprehensive prompts** - include all context and guidance
⚠️ **ALWAYS select correct subagent_type** - general-purpose, Explore, or Plan
⚠️ **ALWAYS include quality standards** - from PROJECT CONTEXT section
⚠️ **ALWAYS specify deliverables** - be explicit about expected outputs
⚠️ **ALWAYS read files before editing** - sub-agents should understand context
⚠️ **NEVER commit without user approval** - sub-agents stage only
⚠️ **ALWAYS update memory banks** - critical for session continuity
⚠️ **ALWAYS provide comprehensive reports** - metrics, decisions, next steps
⚠️ **ALWAYS ask if unclear** - better to clarify than assume

---

**Now invoke the Task tool with the user's request, following all guidelines above.**