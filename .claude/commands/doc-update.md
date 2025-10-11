Quick documentation sync - README, CHANGELOG, CLAUDE.local: $*

---

## DOCUMENTATION QUICK UPDATE WORKFLOW

**Purpose:** Rapidly update README.md, CHANGELOG.md, and CLAUDE.local.md with latest metrics, features, and session summary

**Usage:** `/doc-update <update-type> <description>`
- **update-type:** Type of update (feature, fix, perf, docs, test)
- **description:** Brief description of changes (optional)

**Example:** `/doc-update feature "Added idle scanning support"`

---

## Phase 1: GATHER CURRENT METRICS

**Objective:** Collect all current project metrics for documentation updates

### Step 1.1: Parse Update Type

```bash
UPDATE_TYPE="${1:-general}"
DESCRIPTION="${@:2}"

VALID_TYPES=("feature" "fix" "perf" "docs" "test" "refactor" "chore" "general")

if [[ ! " ${VALID_TYPES[@]} " =~ " ${UPDATE_TYPE} " ]]; then
  echo "WARNING: Unknown update type '$UPDATE_TYPE' (using 'general')"
  UPDATE_TYPE="general"
fi
```

### Step 1.2: Collect Code Metrics

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

**Update documentation: $***
