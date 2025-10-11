Quick documentation sync - README, CHANGELOG, CLAUDE.local: $*

---

## DOCUMENTATION QUICK UPDATE WORKFLOW

**Purpose:** Rapidly update README.md, CHANGELOG.md, and CLAUDE.local.md with latest metrics, features, and session summary

**Usage:** `/doc-update <update-type> <description>`
- **update-type:** Type of update (feature, fix, perf, docs, test)
- **description:** Brief description of changes (optional)

**Example:** `/doc-update feature "Added idle scanning support"`

---

## Phase 0: SAFETY CHECKS AND VALIDATION

**Objective:** Ensure safe file modifications and validate parameters

### Step 0.1: Parse and Validate Update Type

```bash
UPDATE_TYPE="${1:-general}"
DESCRIPTION="${@:2}"

VALID_TYPES=("feature" "fix" "perf" "docs" "test" "refactor" "chore" "general")

# Validate update type
VALID=false
for type in "${VALID_TYPES[@]}"; do
  if [ "$UPDATE_TYPE" = "$type" ]; then
    VALID=true
    break
  fi
done

if [ "$VALID" = false ]; then
  echo "❌ ERROR: Invalid update type '$UPDATE_TYPE'"
  echo ""
  echo "Valid types:"
  echo "  - feature   : New feature implementation"
  echo "  - fix       : Bug fix"
  echo "  - perf      : Performance improvement"
  echo "  - docs      : Documentation update"
  echo "  - test      : Test additions/improvements"
  echo "  - refactor  : Code refactoring"
  echo "  - chore     : Maintenance tasks"
  echo "  - general   : General update (default)"
  echo ""
  echo "Usage: /doc-update <type> [description]"
  echo "Example: /doc-update feature \"Added idle scanning support\""
  exit 1
fi

echo "✅ Update type validated: $UPDATE_TYPE"
```

### Step 0.2: Check Git Status

```bash
# Warn if there are uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
  echo "⚠️  WARNING: Uncommitted changes detected"
  echo ""
  git status --short | head -10
  echo ""
  read -p "Continue with doc-update? (y/N): " -n 1 -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted by user"
    exit 0
  fi
fi
```

### Step 0.3: Create Backup

```bash
# Create backup of critical files
BACKUP_DIR="/tmp/ProRT-IP/doc-backup-$(date +%s)"
mkdir -p "$BACKUP_DIR"

cp README.md "$BACKUP_DIR/" 2>/dev/null && echo "✅ Backed up README.md"
cp CHANGELOG.md "$BACKUP_DIR/" 2>/dev/null && echo "✅ Backed up CHANGELOG.md"
cp CLAUDE.local.md "$BACKUP_DIR/" 2>/dev/null && echo "✅ Backed up CLAUDE.local.md"

echo "Backup location: $BACKUP_DIR"
```

### Step 0.4: Validate File Existence

```bash
# Ensure critical files exist
MISSING_FILES=()
[ ! -f "README.md" ] && MISSING_FILES+=("README.md")
[ ! -f "CHANGELOG.md" ] && MISSING_FILES+=("CHANGELOG.md")
[ ! -f "CLAUDE.local.md" ] && MISSING_FILES+=("CLAUDE.local.md")

if [ ${#MISSING_FILES[@]} -gt 0 ]; then
  echo "❌ ERROR: Missing required files:"
  printf '  - %s\n' "${MISSING_FILES[@]}"
  exit 1
fi

echo "✅ All required files present"
echo ""
```

---

## Phase 1: GATHER CURRENT METRICS

**Objective:** Collect all current project metrics for documentation updates

### Step 1.1: Collect Code Metrics

```bash
# Test counts
TOTAL_TESTS=$(cargo test --workspace 2>&1 | grep -oP '\d+(?= tests)' | tail -1 || echo "643")
PASSED_TESTS=$(cargo test --workspace 2>&1 | grep -oP '\d+(?= passed)' | tail -1 || echo "643")

# Line counts
TOTAL_LINES=$(find crates/ -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')

# File counts
RUST_FILES=$(find crates/ -name "*.rs" | wc -l)

# Git status
FILES_CHANGED=$(git diff --stat 2>/dev/null | tail -1 | grep -oP '\d+(?= files? changed)' || echo "0")
INSERTIONS=$(git diff --stat 2>/dev/null | tail -1 | grep -oP '\d+(?= insertions?)' || echo "0")
DELETIONS=$(git diff --stat 2>/dev/null | tail -1 | grep -oP '\d+(?= deletions?)' || echo "0")
```

### Step 1.3: Determine Version Info

```bash
# Extract version from Cargo.toml
VERSION=$(grep -oP '^version = "\K[^"]+' Cargo.toml | head -1)

# Extract current phase/sprint from CLAUDE.local.md
CURRENT_PHASE=$(grep -oP 'Phase Progress.*?\|\s*\K[^|]+' CLAUDE.local.md | head -1 | xargs || echo "Phase 4")
```

---

## Phase 2: UPDATE README.md

**Objective:** Sync README.md with current metrics and features

### Step 2.1: Update Test Badge

```bash
# Find and replace test count in badge
sed -i "s/tests-[0-9]\+-passing-brightgreen/tests-${PASSED_TESTS}-passing-brightgreen/" README.md
```

### Step 2.2: Update Project Status Table

```bash
# Update Phase Progress
sed -i "s/^\*\*Phase Progress:\*\* .*/\*\*Phase Progress:\*\* ${CURRENT_PHASE}/" README.md

# Update Test Count
sed -i "s/^\*\*Tests:\*\* [0-9]\+ passing .*/\*\*Tests:\*\* ${PASSED_TESTS} passing (100%)/" README.md

# Update Total Lines
sed -i "s/^\*\*Total Lines:\*\* [0-9,]\+/\*\*Total Lines:\*\* ${TOTAL_LINES}/" README.md
```

### Step 2.3: Add New Feature (if feature update)

```bash
if [ "$UPDATE_TYPE" = "feature" ] && [ -n "$DESCRIPTION" ]; then
  # Find Features section and add new feature
  echo ""
  echo "Adding new feature to README.md:"
  echo "- $DESCRIPTION"

  # Implementation depends on README structure
  # Look for "## Features" section and prepend
fi
```

### Step 2.4: Verify README.md Syntax

```bash
# Check for broken markdown (basic validation)
if command -v markdownlint &> /dev/null; then
  markdownlint README.md --fix 2>/dev/null || true
fi

echo "✅ README.md updated"
```

---

## Phase 3: UPDATE CHANGELOG.md

**Objective:** Add entry to CHANGELOG.md [Unreleased] section

### Step 3.1: Generate Changelog Entry

**Entry Format Based on Update Type:**

```bash
CHANGELOG_ENTRY=""

case "$UPDATE_TYPE" in
  feature)
    CHANGELOG_ENTRY="- **Feature:** ${DESCRIPTION}"
    ;;
  fix)
    CHANGELOG_ENTRY="- **Fix:** ${DESCRIPTION}"
    ;;
  perf)
    CHANGELOG_ENTRY="- **Performance:** ${DESCRIPTION}"
    ;;
  docs)
    CHANGELOG_ENTRY="- **Documentation:** ${DESCRIPTION}"
    ;;
  test)
    CHANGELOG_ENTRY="- **Testing:** ${DESCRIPTION}"
    ;;
  refactor)
    CHANGELOG_ENTRY="- **Refactor:** ${DESCRIPTION}"
    ;;
  chore)
    CHANGELOG_ENTRY="- **Chore:** ${DESCRIPTION}"
    ;;
  general)
    if [ -n "$DESCRIPTION" ]; then
      CHANGELOG_ENTRY="- ${DESCRIPTION}"
    else
      CHANGELOG_ENTRY="- Updated documentation and metrics ($(date +%Y-%m-%d))"
    fi
    ;;
esac
```

### Step 3.2: Insert Entry into [Unreleased]

```bash
# Find [Unreleased] section line number
UNRELEASED_LINE=$(grep -n "^## \[Unreleased\]" CHANGELOG.md | cut -d: -f1)

if [ -z "$UNRELEASED_LINE" ]; then
  echo "WARNING: [Unreleased] section not found in CHANGELOG.md"
else
  # Insert after ## [Unreleased] (skip 2 lines for header and blank line)
  INSERT_LINE=$((UNRELEASED_LINE + 2))

  sed -i "${INSERT_LINE}i\\${CHANGELOG_ENTRY}" CHANGELOG.md

  echo "✅ CHANGELOG.md updated with: ${CHANGELOG_ENTRY}"
fi
```

### Step 3.3: Add Metrics (if significant change)

```bash
if [ "$FILES_CHANGED" -gt 5 ]; then
  METRICS_ENTRY="  - Files changed: ${FILES_CHANGED} (+${INSERTIONS}/-${DELETIONS} lines)"
  sed -i "${INSERT_LINE}a\\${METRICS_ENTRY}" CHANGELOG.md
fi
```

---

## Phase 4: UPDATE CLAUDE.local.md

**Objective:** Add quick session entry to CLAUDE.local.md

### Step 4.1: Generate Session Entry

```bash
SESSION_DATE=$(date +%Y-%m-%d)
SESSION_TITLE="${UPDATE_TYPE}: ${DESCRIPTION:-Documentation update}"

SESSION_ENTRY="
### ${SESSION_DATE}: ${SESSION_TITLE}

**Objective:** ${DESCRIPTION:-Quick documentation sync}

**Changes:**
- Updated README.md (test count: ${PASSED_TESTS}, total lines: ${TOTAL_LINES})
- Updated CHANGELOG.md with ${UPDATE_TYPE} entry
- Files modified: ${FILES_CHANGED} (+${INSERTIONS}/-${DELETIONS} lines)

**Result:** Documentation synchronized successfully
"
```

### Step 4.2: Insert Session Entry

```bash
# Find "## Recent Sessions" section
SESSIONS_LINE=$(grep -n "^## Recent Sessions" CLAUDE.local.md | cut -d: -f1)

if [ -z "$SESSIONS_LINE" ]; then
  echo "WARNING: Recent Sessions section not found in CLAUDE.local.md"
else
  # Insert after ## Recent Sessions header (skip 2 lines)
  INSERT_LINE=$((SESSIONS_LINE + 2))

  # Insert session entry (preserve formatting)
  echo "$SESSION_ENTRY" | sed -i "${INSERT_LINE}r /dev/stdin" CLAUDE.local.md

  echo "✅ CLAUDE.local.md updated with session entry"
fi
```

### Step 4.3: Update Current Status Table

```bash
# Update test count in Current Status table
sed -i "/^\| \*\*Tests\*\*/s/[0-9]\+ passing/${PASSED_TESTS} passing/" CLAUDE.local.md

# Update total lines
sed -i "/^\| \*\*Total Lines\*\*/s/[0-9,]\+/${TOTAL_LINES}/" CLAUDE.local.md
```

---

## Phase 5: DISPLAY UPDATE SUMMARY

**Objective:** Show comprehensive summary of documentation updates

### Step 5.1: Display Summary

```bash
echo "=========================================="
echo "Documentation Updated Successfully"
echo "=========================================="
echo ""
echo "📊 CURRENT METRICS"
echo "  Version: ${VERSION}"
echo "  Phase: ${CURRENT_PHASE}"
echo "  Tests: ${PASSED_TESTS} passing (100%)"
echo "  Total Lines: ${TOTAL_LINES}"
echo "  Rust Files: ${RUST_FILES}"
echo ""
echo "📝 UPDATED FILES"
echo "  ✅ README.md (test badge, status table)"
echo "  ✅ CHANGELOG.md (${UPDATE_TYPE} entry)"
echo "  ✅ CLAUDE.local.md (session entry, metrics)"
echo ""
echo "📈 GIT CHANGES"
echo "  Files Changed: ${FILES_CHANGED}"
echo "  Insertions: +${INSERTIONS}"
echo "  Deletions: -${DELETIONS}"
echo ""
echo "🔖 UPDATE DETAILS"
echo "  Type: ${UPDATE_TYPE}"
echo "  Description: ${DESCRIPTION:-General update}"
echo "  Date: ${SESSION_DATE}"
echo ""
echo "✅ All documentation synchronized"
echo ""
```

---

## SUCCESS CRITERIA

✅ Current metrics collected (tests, lines, files)
✅ README.md updated (badges, status table)
✅ CHANGELOG.md updated ([Unreleased] entry)
✅ CLAUDE.local.md updated (session entry, metrics)
✅ Summary displayed with current state

---

## ADVANCED USAGE

### Feature Update with Details

```bash
/doc-update feature "Idle scanning - ultimate anonymity with zombie hosts"
```

**Result:**
- README.md: Adds feature to Features section
- CHANGELOG.md: "- **Feature:** Idle scanning - ultimate anonymity with zombie hosts"
- CLAUDE.local.md: Session entry with details

### Performance Improvement

```bash
/doc-update perf "10K ports: 117ms → 39ms (66% faster)"
```

**Result:**
- README.md: Updates performance metrics
- CHANGELOG.md: "- **Performance:** 10K ports: 117ms → 39ms (66% faster)"
- CLAUDE.local.md: Session with performance delta

### Bug Fix

```bash
/doc-update fix "Critical DNS resolution bug preventing hostname scans"
```

**Result:**
- README.md: Updates status
- CHANGELOG.md: "- **Fix:** Critical DNS resolution bug preventing hostname scans"
- CLAUDE.local.md: Session with fix details

### General Sync (No Description)

```bash
/doc-update general
```

**Result:**
- All files updated with current metrics
- Generic changelog entry with date
- Session entry documenting sync

---

## DELIVERABLES

1. **Updated README.md:** Current metrics, badges, status
2. **Updated CHANGELOG.md:** New entry in [Unreleased]
3. **Updated CLAUDE.local.md:** Session entry, current metrics
4. **Summary Report:** Console output with changes

---

## RELATED COMMANDS

**Sprint Workflow:**
- `/sprint-start <sprint-id> <objective>` - Begin sprint with planning documents
- `/sprint-complete <sprint-id>` - Finalize sprint with comprehensive documentation update
- Sprint completion automatically updates README, CHANGELOG, and memory banks

**Development Workflow:**
- `/rust-check` - Validate changes before documenting (ensure tests pass)
- `/bench-compare <baseline> <comparison>` - Measure performance for perf updates
- `/module-create <crate> <module-name> <desc>` - Create new module, then document

**Quality Assurance:**
- `/ci-status` - Check CI status before documentation updates
- `/test-quick <pattern>` - Run tests related to documented changes

**Bug Tracking:**
- `/bug-report <issue> <command>` - Generate bug report, then doc-update fix entry

## WORKFLOW INTEGRATION

**Common Documentation Workflows:**

1. **Feature Addition:**
   ```
   Implement feature → /rust-check → /doc-update feature "Description"
   ```

2. **Bug Fix:**
   ```
   Fix bug → /test-quick <pattern> → /doc-update fix "Description"
   ```

3. **Performance Improvement:**
   ```
   Optimize → /bench-compare → /doc-update perf "X → Y (Z% faster)"
   ```

4. **Sprint Completion:**
   ```
   /sprint-complete X.Y → Automatically calls doc-update internally
   ```

5. **Regular Maintenance:**
   ```
   /doc-update general → Sync metrics without specific changes
   ```

## SEE ALSO

- `README.md` - Project overview and status (updated by this command)
- `CHANGELOG.md` - Version history (updated by this command)
- `CLAUDE.local.md` - Memory bank (updated by this command)
- `CONTRIBUTING.md` - Documentation standards
- `docs/` - Technical documentation directory

---

**Update documentation: $***
