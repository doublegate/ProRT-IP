Check GitHub Actions CI/CD pipeline status: $*

---

## CI/CD PIPELINE STATUS WORKFLOW

**Purpose:** Check GitHub Actions workflow status, identify failures, and provide debugging guidance

**Usage:** `/ci-status [run-number|workflow-name|--failed]`

**Requirements:** GitHub CLI (`gh`) must be installed and authenticated

---

## PARAMETERS

**No arguments:** Show last 10 runs across all workflows
**run-number (digits):** Show specific run by ID number
**workflow-name (string):** Filter by workflow name (e.g., "CI", "Release")
**--failed flag:** Show only failed runs from last 50

---

## USAGE PATTERNS

**Basic:** `/ci-status` (show last 10 runs across all workflows)
**Specific Run:** `/ci-status 1234567890` (show details for run ID)
**Workflow Filter:** `/ci-status CI` (show only CI workflow runs)
**Failed Only:** `/ci-status --failed` (show only failed runs)

## USAGE EXAMPLES

```bash
/ci-status                    # Last 10 runs, all workflows
/ci-status 1234567890         # Specific run details
/ci-status CI                 # Filter to CI workflow
/ci-status --failed           # Only failed runs
```

---

## Phase 1: VERIFY GITHUB CLI AND PARSE PARAMETERS

**Objective:** Ensure gh CLI is installed and authenticated

### Step 1.1: Check gh Installation

```bash
if ! command -v gh &> /dev/null; then
  echo "ERROR: GitHub CLI (gh) not installed"
  echo ""
  echo "Install:"
  echo "  Arch: sudo pacman -S github-cli"
  echo "  Ubuntu: sudo apt install gh"
  echo "  macOS: brew install gh"
  echo ""
  echo "After install: gh auth login"
  exit 1
fi
```

### Step 1.2: Check Authentication

```bash
if ! gh auth status &> /dev/null; then
  echo "ERROR: GitHub CLI not authenticated"
  echo ""
  echo "Authenticate with: gh auth login"
  echo "Follow prompts to authenticate with GitHub"
  exit 1
fi

echo "✅ GitHub CLI authenticated"
```

### Step 1.3: Parse and Validate Optional Parameters

```bash
PARAM="$1"
FILTER_TYPE="all"
RUN_NUMBER=""
WORKFLOW_NAME=""

if [ -n "$PARAM" ]; then
  # Check if parameter is a run number (digits only)
  if [[ "$PARAM" =~ ^[0-9]+$ ]]; then
    FILTER_TYPE="run"
    RUN_NUMBER="$PARAM"
    echo "🎯 Filter: Specific run #$RUN_NUMBER"

  # Check if --failed flag
  elif [ "$PARAM" = "--failed" ]; then
    FILTER_TYPE="failed"
    echo "🎯 Filter: Failed runs only"

  # Check for invalid flags (starting with --)
  elif [[ "$PARAM" =~ ^-- ]]; then
    echo "❌ ERROR: Invalid flag '$PARAM'"
    echo ""
    echo "Valid flags:"
    echo "  --failed  Show only failed runs"
    echo ""
    echo "Usage: /ci-status [run-number|workflow-name|--failed]"
    echo ""
    echo "Examples:"
    echo "  /ci-status              # Last 10 runs"
    echo "  /ci-status 1234567890   # Specific run"
    echo "  /ci-status CI           # CI workflow only"
    echo "  /ci-status --failed     # Failed runs only"
    exit 1

  # Otherwise treat as workflow name
  else
    FILTER_TYPE="workflow"
    WORKFLOW_NAME="$PARAM"
    echo "🎯 Filter: Workflow '$WORKFLOW_NAME'"
  fi
else
  echo "🎯 Filter: All workflows (last 10 runs)"
fi

echo ""
```

---

## Phase 2: FETCH WORKFLOW RUNS

**Objective:** Get recent workflow runs for the repository

### Step 2.1: Fetch Workflow Runs Based on Filter

```bash
echo "Fetching CI/CD workflow runs..."
echo ""

# Build gh command based on filter type
GH_CMD="gh run list --limit 10 --json status,conclusion,name,createdAt,event,number,headBranch"

if [ "$FILTER_TYPE" = "run" ]; then
  # Fetch specific run details
  gh run view "$RUN_NUMBER" --json status,conclusion,name,createdAt,event,number,headBranch,jobs > /tmp/gh-run-specific.json
  if [ $? -ne 0 ]; then
    echo "❌ ERROR: Failed to fetch run #$RUN_NUMBER"
    echo "Check run number exists: gh run list"
    exit 1
  fi
  # Convert to array format for consistency
  jq -s '.' /tmp/gh-run-specific.json > /tmp/gh-runs.json

elif [ "$FILTER_TYPE" = "workflow" ]; then
  # Filter by workflow name
  gh run list --workflow "$WORKFLOW_NAME" --limit 10 --json status,conclusion,name,createdAt,event,number,headBranch > /tmp/gh-runs.json

elif [ "$FILTER_TYPE" = "failed" ]; then
  # Fetch all runs and filter failed
  gh run list --limit 50 --json status,conclusion,name,createdAt,event,number,headBranch > /tmp/gh-runs-all.json
  jq '[.[] | select(.conclusion == "failure")]' /tmp/gh-runs-all.json | head -10 > /tmp/gh-runs.json

else
  # Default: last 10 runs
  gh run list --limit 10 --json status,conclusion,name,createdAt,event,number,headBranch > /tmp/gh-runs.json
fi

if [ $? -ne 0 ]; then
  echo "❌ ERROR: Failed to fetch workflow runs"
  echo "Check repository access and authentication"
  exit 1
fi

RUNS_COUNT=$(jq '. | length' /tmp/gh-runs.json)
echo "✅ Fetched $RUNS_COUNT workflow run(s)"
echo ""
```

### Step 2.2: Parse Workflow Data

```bash
# Extract latest run details
LATEST_RUN=$(jq -r '.[0]' /tmp/gh-runs.json)

RUN_NUMBER=$(echo "$LATEST_RUN" | jq -r '.number')
RUN_STATUS=$(echo "$LATEST_RUN" | jq -r '.status')
RUN_CONCLUSION=$(echo "$LATEST_RUN" | jq -r '.conclusion')
RUN_NAME=$(echo "$LATEST_RUN" | jq -r '.name')
RUN_BRANCH=$(echo "$LATEST_RUN" | jq -r '.headBranch')
RUN_EVENT=$(echo "$LATEST_RUN" | jq -r '.event')
RUN_DATE=$(echo "$LATEST_RUN" | jq -r '.createdAt')
```

---

## Phase 3: DISPLAY OVERALL STATUS

**Objective:** Show high-level CI/CD pipeline status

### Step 3.1: Display Latest Run Summary

```bash
echo "=========================================="
echo "Latest CI/CD Workflow Run"
echo "=========================================="
echo ""
echo "📋 RUN DETAILS"
echo "  Number: #$RUN_NUMBER"
echo "  Workflow: $RUN_NAME"
echo "  Branch: $RUN_BRANCH"
echo "  Event: $RUN_EVENT"
echo "  Created: $RUN_DATE"
echo ""

# Status with color coding
if [ "$RUN_STATUS" = "completed" ]; then
  if [ "$RUN_CONCLUSION" = "success" ]; then
    echo "✅ STATUS: SUCCESS"
  elif [ "$RUN_CONCLUSION" = "failure" ]; then
    echo "❌ STATUS: FAILURE"
  else
    echo "⚠️ STATUS: $RUN_CONCLUSION"
  fi
else
  echo "🔄 STATUS: IN PROGRESS"
fi

echo ""
```

### Step 3.2: Display Recent Runs Summary

```bash
echo "=========================================="
echo "Recent Workflow Runs (Last 10)"
echo "=========================================="
echo ""

# Format as table
jq -r '.[] | "\(.number)\t\(.status)\t\(.conclusion // "N/A")\t\(.name)\t\(.headBranch)"' /tmp/gh-runs.json | \
  awk 'BEGIN {printf "%-8s %-12s %-12s %-20s %s\n", "Run #", "Status", "Conclusion", "Workflow", "Branch"; print "------------------------------------------------------------------------"} {printf "%-8s %-12s %-12s %-20s %s\n", $1, $2, $3, $4, $5}'

echo ""
```

---

## Phase 4: ANALYZE FAILURES (IF ANY)

**Objective:** Identify failed jobs and provide debugging information

### Step 4.1: Check for Failures

```bash
if [ "$RUN_CONCLUSION" = "failure" ]; then
  echo "=========================================="
  echo "Failure Analysis"
  echo "=========================================="
  echo ""

  # Get job details for failed run
  gh run view "$RUN_NUMBER" --json jobs > /tmp/gh-jobs.json

  # Extract failed jobs
  FAILED_JOBS=$(jq -r '.jobs[] | select(.conclusion == "failure") | .name' /tmp/gh-jobs.json)

  if [ -z "$FAILED_JOBS" ]; then
    echo "No failed jobs found (workflow may have failed for other reasons)"
  else
    echo "❌ FAILED JOBS:"
    echo "$FAILED_JOBS" | while read -r job; do
      echo "  - $job"
    done
    echo ""
  fi
fi
```

### Step 4.2: Display Common Failure Patterns

```bash
if [ "$RUN_CONCLUSION" = "failure" ]; then
  echo "🔍 COMMON FAILURE CAUSES:"
  echo ""
  echo "1. **Format Job Failure:**"
  echo "   - Fix: cargo fmt --all"
  echo "   - Verify: cargo fmt --check"
  echo ""
  echo "2. **Clippy Job Failure:**"
  echo "   - Fix: cargo clippy --all-targets --fix --allow-dirty"
  echo "   - Verify: cargo clippy --all-targets -- -D warnings"
  echo ""
  echo "3. **Test Job Failure (Linux/Windows/macOS):**"
  echo "   - Debug locally: cargo test --workspace"
  echo "   - Check platform-specific tests (Windows timing tolerances)"
  echo ""
  echo "4. **MSRV Job Failure:**"
  echo "   - Check Rust version: Cargo.toml rust-version = \"1.70\""
  echo "   - Verify locally: rustup install 1.70 && cargo +1.70 test"
  echo ""
  echo "5. **Security Audit Failure:**"
  echo "   - Run: cargo audit"
  echo "   - Fix vulnerabilities or update dependencies"
  echo ""
fi
```

---

## Phase 5: PROVIDE DEBUGGING COMMANDS

**Objective:** Give user actionable commands to investigate failures

### Step 5.1: Display Debugging Commands

```bash
echo "=========================================="
echo "Debugging Commands"
echo "=========================================="
echo ""
echo "📊 View Full Run Details:"
echo "  gh run view $RUN_NUMBER"
echo ""
echo "📝 View Run Logs:"
echo "  gh run view $RUN_NUMBER --log"
echo ""
echo "🔍 View Specific Job Logs:"
echo "  gh run view $RUN_NUMBER --job <job-id> --log"
echo ""
echo "🔄 Re-run Failed Jobs:"
echo "  gh run rerun $RUN_NUMBER --failed"
echo ""
echo "🔄 Re-run Entire Workflow:"
echo "  gh run rerun $RUN_NUMBER"
echo ""
echo "📂 Download Artifacts (if any):"
echo "  gh run download $RUN_NUMBER"
echo ""
```

### Step 5.2: Local Reproduction Guide

```bash
echo "=========================================="
echo "Local Reproduction"
echo "=========================================="
echo ""
echo "To reproduce CI failures locally:"
echo ""
echo "1. **Format Check:**"
echo "   cargo fmt --all -- --check"
echo ""
echo "2. **Lint Check:**"
echo "   cargo clippy --all-targets --all-features -- -D warnings"
echo ""
echo "3. **Full Test Suite:**"
echo "   cargo test --workspace"
echo ""
echo "4. **MSRV Verification:**"
echo "   rustup install 1.70"
echo "   cargo +1.70 test --workspace"
echo ""
echo "5. **Security Audit:**"
echo "   cargo install cargo-audit"
echo "   cargo audit"
echo ""
echo "6. **Complete CI Pipeline (locally):**"
echo "   /rust-check"
echo ""
```

### Step 5.3: Run Local Validation (if failures detected)

```bash
if [ "$RUN_CONCLUSION" = "failure" ]; then
  echo "=========================================="
  echo "Automated Local Validation"
  echo "=========================================="
  echo ""
  echo "Running /rust-check to identify local issues..."
  echo ""

  # Check if /rust-check command exists
  if [ -f ".claude/commands/rust-check.md" ]; then
    echo "💡 Recommended: Execute /rust-check for comprehensive local validation"
  else
    echo "Run manual validation:"
    echo "  cargo fmt --check"
    echo "  cargo clippy --all-targets -- -D warnings"
    echo "  cargo test --workspace"
  fi

  echo ""
  echo "If local checks pass but CI fails:"
  echo "  - Check platform-specific issues (Windows, macOS)"
  echo "  - Review CI logs: gh run view $RUN_NUMBER --log"
  echo "  - Compare environment: Rust version, dependencies"
  echo ""
fi
```

---

## Phase 6: DISPLAY WORKFLOW-SPECIFIC STATUS

**Objective:** Show status for individual workflow files

### Step 6.1: List All Workflows

```bash
echo "=========================================="
echo "Workflow Files Status"
echo "=========================================="
echo ""

gh workflow list > /tmp/gh-workflows.txt

cat /tmp/gh-workflows.txt

echo ""
```

### Step 6.2: CI Workflow Breakdown

```bash
if grep -q "CI" /tmp/gh-workflows.txt; then
  echo "📋 CI WORKFLOW (ci.yml):"
  echo "  Jobs:"
  echo "    1. Format       - cargo fmt --check"
  echo "    2. Clippy       - cargo clippy -- -D warnings"
  echo "    3. Test (Linux) - cargo test --workspace"
  echo "    4. Test (Win)   - cargo test --workspace (Windows)"
  echo "    5. Test (macOS) - cargo test --workspace (macOS)"
  echo "    6. MSRV         - Rust 1.70+ verification"
  echo "    7. Security     - cargo audit"
  echo ""
fi
```

### Step 6.3: Release Workflow Status

```bash
if grep -q "Release" /tmp/gh-workflows.txt; then
  echo "📦 RELEASE WORKFLOW (release.yml):"
  echo "  Trigger: Git tag push (v*.*.*)"
  echo "  Builds: 9 targets (5 working, 4 experimental)"
  echo "  Artifacts: Binary archives for each platform"
  echo ""
fi
```

---

## SUCCESS CRITERIA

✅ GitHub CLI available and authenticated
✅ Latest workflow run fetched and displayed
✅ Recent runs summary shown (last 10)
✅ Failures analyzed (if any)
✅ Debugging commands provided
✅ Local reproduction guide displayed

---

## Phase 7: PR-SPECIFIC STATUS (if applicable)

**Objective:** Show CI status for current PR branch

### Step 7.1: Check for Pull Request

```bash
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null)
PR_NUMBER=$(gh pr list --head "$CURRENT_BRANCH" --json number --jq '.[0].number' 2>/dev/null)

if [ -n "$PR_NUMBER" ]; then
  echo "=========================================="
  echo "Pull Request CI Status"
  echo "=========================================="
  echo ""
  echo "PR #$PR_NUMBER Status:"
  echo ""

  gh pr checks "$PR_NUMBER" --watch=false || true

  echo ""
  echo "PR Details:"
  gh pr view "$PR_NUMBER" --json title,state,mergeable,statusCheckRollup \
    --jq '"Title: " + .title, "State: " + .state, "Mergeable: " + (.mergeable // "UNKNOWN")'

  echo ""
fi
```

---

## DELIVERABLES

1. **Latest Run Status:** Success/failure with details
2. **Recent Runs Table:** Last 10 workflow runs (or filtered)
3. **Failure Analysis:** Failed jobs identified (if applicable)
4. **Debugging Guide:** Commands to investigate and fix issues
5. **Local Reproduction:** How to reproduce CI failures locally
6. **PR Status:** Pull request checks (if applicable)

---

## RELATED COMMANDS

- `/rust-check` - Run local quality checks (format, lint, test, build)
- `/test-quick <pattern>` - Run specific test subsets for debugging
- `/bug-report <issue> <command>` - Generate comprehensive bug report
- `/sprint-complete <sprint-id>` - Validates all CI checks pass before completion

**Workflow Integration:**
- **CI Failure:** `/ci-status` → Identify failures → `/rust-check` locally → Fix → Push
- **PR Review:** `/ci-status` → Check PR status → `/rust-check` → Merge when green
- **Pre-Release:** `/ci-status` → Ensure all passing → Tag release
- **Debugging:** `/ci-status` → View logs → `/bug-report` if needed

---

**Check CI/CD status now**
