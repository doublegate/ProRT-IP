/sub-agent THINK DEEPLY and execute comprehensive pre-commit workflow:

## Phase 1: ANALYZE CHANGES (Understand Context)
- Run `git status` to identify all modified/new/deleted files
- Run `git diff --stat` to see scope of changes
- Identify which areas changed (src, docs, tests, config)
- Determine commit scope and impact level
- Check for uncommitted work in progress (TODO, FIXME, WIP comments)

## Phase 2: CODE QUALITY (Validate & Format)
- Run `cargo fmt --all` (Rust formatting)
- Run `cargo clippy --all-targets -- -D warnings` (linting)
- Check for compilation warnings: `cargo build --release 2>&1 | grep -i warning`
- Verify all tests pass: `cargo test --workspace` (if changes affect code)
- Document any intentional skips (tests, clippy warnings)

## Phase 3: GITIGNORE MAINTENANCE
- Review .gitignore for missing patterns
- Add entries for: temporary files, build artifacts, IDE files, OS files
- Ensure /tmp/ files not committed
- Verify no sensitive files staged (*.env, credentials, keys)

## Phase 4: DOCUMENTATION UPDATES
- **docs/ Analysis:**
  - Read all modified .md files in docs/
  - Verify technical accuracy
  - Check for broken cross-references
  - Update version numbers if needed
  - Ensure consistency with code changes
- **README.md (Root):**
  - Update Project Status section (phase, version, test count)
  - Update statistics (files, lines, modules)
  - Add new features to usage examples
  - Update performance metrics if improved
  - Verify all badges/links current
- **CHANGELOG.md:**
  - Add comprehensive entry to [Unreleased] section
  - Format: feat/fix/docs/perf/refactor/test/chore
  - Include: what changed, why, impact, related issues
  - List affected files/modules
  - Document breaking changes prominently

## Phase 5: MEMORY BANK OPTIMIZATION
- **Read Current State:**
  - Read CLAUDE.md (project guidance)
  - Read CLAUDE.local.md (session history, current status)
  - Read ~/.claude/CLAUDE.md (user patterns) if relevant
  - Identify stale/duplicate/verbose content
- **Update with Session Info:**
  - Add current session summary to CLAUDE.local.md
  - Update metrics (test count, file count, version)
  - Record key decisions made
  - Update Known Issues section
  - Add Next Actions if applicable
- **Optimize (Inline /mem-reduce Logic):**
  - Remove completed tasks from CLAUDE.local.md
  - Compress verbose prose (20-30% reduction target)
  - Consolidate duplicate information
  - Use tables/lists for efficiency
  - Archive old session details if >10 sessions ago
  - Ensure critical info preserved
- **Verify:**
  - No information loss
  - Cross-references still valid
  - Quick reference sections intact

## Phase 6: CROSS-REFERENCE VALIDATION
- Check README links point to existing files
- Verify CHANGELOG references match actual changes
- Ensure docs/ cross-references valid
- Validate benchmarks/ and bug_fix/ references in README
- Check all relative paths correct

## Phase 7: FINAL VERIFICATION
- **File Count Check:**
  ```bash
  echo "Total files: $(git ls-files | wc -l)"
  echo "Staged changes: $(git diff --cached --name-only | wc -l)"
  ```
- **No Unintended Files:**
  - Verify no .swp, .tmp, .bak files staged
  - Check no large binary files added unintentionally
  - Confirm no debug/profiling artifacts staged
- **Build Verification:**
  - Final `cargo build --release` succeeds (if code changed)
  - All tests still passing (if code changed)

## Phase 8: STAGE ALL CHANGES
```bash
git add -A
```

## Phase 9: CREATE COMPREHENSIVE COMMIT
**Commit Message Format:**
```
<type>(<scope>): <summary>

<detailed description>

## Changes Made
- Category 1: changes
- Category 2: changes

## Impact
- Performance: metrics if applicable
- Features: new functionality
- Fixes: bugs resolved

## Files Modified
- path/to/file1 (description)
- path/to/file2 (description)

## Testing
- Tests passing: X/X
- New tests added: X
- Coverage: X%

## Documentation
- README updated: yes/no
- CHANGELOG updated: yes/no
- docs/ updated: yes/no
- Memory banks updated: yes/no

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Commit Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `perf`: Performance improvement
- `refactor`: Code restructuring
- `test`: Test additions/fixes
- `chore`: Maintenance (deps, build, ci)

**Requirements:**
- Summary: <72 chars, imperative mood
- Body: Detailed technical description
- Lists: Concrete changes, not vague
- Metrics: Include numbers where relevant
- Attribution: Always include Co-Authored-By

## Phase 10: FINAL REVIEW (Before Commit)
- Display commit message for review
- Show `git diff --cached --stat` (final summary)
- Confirm all phases completed successfully
- Ask: "Ready to commit? (y/n)" - wait for user confirmation
- If yes: `git commit -F message.txt`
- If no: Stage remains, user can modify

## Success Criteria
✅ All code quality checks passed (format, lint, build, test)
✅ All documentation updated and accurate
✅ Memory banks optimized and current
✅ No broken references or links
✅ Comprehensive commit message created
✅ All files properly staged
✅ No sensitive/temporary files included
✅ User approved final commit

## Error Handling
- If tests fail: Abort, report failures, let user fix
- If linting fails: Abort, report issues, let user fix
- If build fails: Abort, report errors, let user fix
- If broken links found: Report but continue (doc-only issue)
- If memory banks corrupted: Report, skip optimization, continue

IMPORTANT: Be thorough but efficient. Skip code quality checks (Phase 2, 7) if only documentation changed. Always create detailed commit messages. Always ask for user confirmation before final commit.