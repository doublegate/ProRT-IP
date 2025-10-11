Check GitHub Actions CI/CD pipeline status

---

## CI/CD PIPELINE STATUS WORKFLOW

**Purpose:** Check GitHub Actions workflow status, identify failures, and provide debugging guidance

**Usage:** `/ci-status`

**Requirements:** GitHub CLI (`gh`) must be installed and authenticated

---

## Phase 1: VERIFY GITHUB CLI AVAILABILITY

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

---

## Phase 2: FETCH WORKFLOW RUNS

**Objective:** Get recent workflow runs for the repository

### Step 2.1: Get Latest Workflow Runs

```bash
echo "Fetching latest CI/CD workflow runs..."
echo ""

# Get last 10 workflow runs
gh run list --limit 10 --json status,conclusion,name,createdAt,event,number,headBranch > /tmp/gh-runs.json

if [ $? -ne 0 ]; then
  echo "ERROR: Failed to fetch workflow runs"
  echo "Check repository access and authentication"
  exit 1
fi
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

## DELIVERABLES

1. **Latest Run Status:** Success/failure with details
2. **Recent Runs Table:** Last 10 workflow runs
3. **Failure Analysis:** Failed jobs identified (if applicable)
4. **Debugging Guide:** Commands to investigate and fix issues
5. **Local Reproduction:** How to reproduce CI failures locally

---

**Check CI/CD status now**
