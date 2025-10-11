Generate comprehensive bug report with system info and logs: $*

---

## COMPREHENSIVE BUG REPORT WORKFLOW

**Purpose:** Generate detailed bug report with system information, reproduction steps, logs, and analysis

**Usage:** `/bug-report <issue-summary> <reproduction-command>`
- **issue-summary:** Brief description of the bug
- **reproduction-command:** Command that triggers the bug

**Example:** `/bug-report "Scanner hangs on filtered network" "./target/release/prtip -p 1-10000 192.168.4.1"`

---

## Phase 1: VALIDATE PARAMETERS AND SETUP

**Objective:** Parse arguments and create bug report directory

### Step 1.1: Parse Arguments

```bash
ISSUE_SUMMARY="$1"
REPRO_CMD="${@:2}"

if [ -z "$ISSUE_SUMMARY" ]; then
  echo "ERROR: Issue summary required"
  echo "Usage: /bug-report <issue-summary> <reproduction-command>"
  echo "Example: /bug-report \"Scanner hangs\" \"./target/release/prtip -p 1-1000 127.0.0.1\""
  exit 1
fi

if [ -z "$REPRO_CMD" ]; then
  echo "WARNING: No reproduction command provided"
  echo "Bug report will be created without reproduction test"
fi
```

### Step 1.2: Create Bug Report Directory

```bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BUG_REPORT_DIR="/tmp/ProRT-IP/bug-report-${TIMESTAMP}"

mkdir -p "$BUG_REPORT_DIR"

echo "Bug Report Directory: $BUG_REPORT_DIR"
```

---

## Phase 2: COLLECT SYSTEM INFORMATION

**Objective:** Gather comprehensive system and environment details

### Step 2.1: System Information

```bash
echo "Collecting system information..."

cat > "$BUG_REPORT_DIR/system-info.txt" <<EOF
# System Information

**Generated:** $(date)
**Issue:** $ISSUE_SUMMARY

## Operating System

$(uname -a)

**Distribution:**
$(cat /etc/os-release 2>/dev/null || echo "Unknown")

**Kernel Version:**
$(uname -r)

## Hardware

**CPU:**
$(lscpu | grep -E 'Model name|Architecture|CPU\(s\):' || echo "Unknown")

**Memory:**
$(free -h | grep -E 'Mem:' || echo "Unknown")

**Network Interfaces:**
$(ip link show 2>/dev/null || ifconfig 2>/dev/null || echo "Unknown")

## Rust Environment

**Rust Version:**
$(rustc --version)

**Cargo Version:**
$(cargo --version)

**Rustup Version:**
$(rustup --version)

**Active Toolchain:**
$(rustup show active-toolchain)

## ProRT-IP Version

**Version:**
$(grep -oP '^version = "\K[^"]+' Cargo.toml | head -1)

**Git Commit:**
$(git rev-parse --short HEAD 2>/dev/null || echo "Unknown")

**Git Branch:**
$(git branch --show-current 2>/dev/null || echo "Unknown")

**Git Status:**
$(git status --short 2>/dev/null || echo "Clean or not a git repo")

## Dependencies

**Key Dependencies:**
$(cargo tree --depth 1 2>/dev/null | head -20 || echo "Run cargo tree for full list")

## Build Configuration

**Profile:**
$(grep -A 10 '\[profile.release\]' Cargo.toml || echo "Default")

EOF

echo "✅ System information collected"
```

### Step 2.2: Network Configuration

```bash
echo "Collecting network configuration..."

cat > "$BUG_REPORT_DIR/network-config.txt" <<EOF
# Network Configuration

## Interfaces

$(ip addr show 2>/dev/null || ifconfig 2>/dev/null)

## Routing Table

$(ip route show 2>/dev/null || route -n 2>/dev/null)

## DNS Configuration

$(cat /etc/resolv.conf 2>/dev/null || echo "Unknown")

## Firewall Status

$(sudo iptables -L -n 2>/dev/null || echo "Unable to read iptables (requires sudo)")

## File Descriptor Limits

$(ulimit -a)

EOF

echo "✅ Network configuration collected"
```

---

## Phase 3: EXECUTE REPRODUCTION COMMAND

**Objective:** Run reproduction command and capture all output

### Step 3.1: Standard Execution

```bash
if [ -n "$REPRO_CMD" ]; then
  echo "Executing reproduction command (standard)..."

  timeout 60s $REPRO_CMD > "$BUG_REPORT_DIR/reproduction-stdout.txt" 2> "$BUG_REPORT_DIR/reproduction-stderr.txt"

  REPRO_EXIT_CODE=$?

  echo "Exit Code: $REPRO_EXIT_CODE" > "$BUG_REPORT_DIR/reproduction-exit-code.txt"

  if [ "$REPRO_EXIT_CODE" -eq 124 ]; then
    echo "⚠️ Reproduction command timed out after 60 seconds"
  elif [ "$REPRO_EXIT_CODE" -ne 0 ]; then
    echo "❌ Reproduction command failed with exit code: $REPRO_EXIT_CODE"
  else
    echo "✅ Reproduction command completed successfully"
  fi
fi
```

### Step 3.2: Verbose Execution

```bash
if [ -n "$REPRO_CMD" ]; then
  echo "Executing reproduction command (verbose logging)..."

  RUST_LOG=debug timeout 60s $REPRO_CMD > "$BUG_REPORT_DIR/reproduction-verbose-stdout.txt" 2> "$BUG_REPORT_DIR/reproduction-verbose-stderr.txt"

  echo "✅ Verbose logs captured"
fi
```

### Step 3.3: Trace Execution (if debug symbols available)

```bash
if [ -n "$REPRO_CMD" ] && command -v strace &> /dev/null; then
  echo "Executing reproduction command (strace)..."

  strace -c -o "$BUG_REPORT_DIR/strace-summary.txt" timeout 60s $REPRO_CMD > /dev/null 2>&1

  echo "✅ strace summary captured"
fi
```

---

## Phase 4: GENERATE BUG REPORT MARKDOWN

**Objective:** Create comprehensive markdown bug report

### Step 4.1: Generate Bug Report

```bash
cat > "$BUG_REPORT_DIR/BUG-REPORT.md" <<EOF
# Bug Report: $ISSUE_SUMMARY

**Generated:** $(date)
**Reporter:** $(whoami)@$(hostname)

## Issue Summary

$ISSUE_SUMMARY

## Expected Behavior

[User should describe expected behavior]

## Actual Behavior

[User should describe actual behavior]

## Reproduction Steps

1. Build: \`cargo build --release\`
2. Execute: \`$REPRO_CMD\`
3. Observe: [Describe what happens]

## Reproduction Command

\`\`\`bash
$REPRO_CMD
\`\`\`

**Exit Code:** $REPRO_EXIT_CODE

## Environment

**OS:** $(uname -s) $(uname -r)
**Rust:** $(rustc --version)
**ProRT-IP Version:** $(grep -oP '^version = "\K[^"]+' Cargo.toml | head -1)
**Git Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "Unknown")

## System Information

See: \`system-info.txt\` for comprehensive details

**Key Details:**
- **CPU:** $(lscpu | grep 'Model name' | cut -d: -f2 | xargs || echo "Unknown")
- **Memory:** $(free -h | grep Mem | awk '{print $2}' || echo "Unknown")
- **Network Interfaces:** $(ip link show | grep -E '^[0-9]+:' | awk '{print $2}' | tr '\n' ' ' || echo "Unknown")

## Logs

### Standard Output

\`\`\`
$(cat "$BUG_REPORT_DIR/reproduction-stdout.txt" 2>/dev/null || echo "No stdout captured")
\`\`\`

### Standard Error

\`\`\`
$(cat "$BUG_REPORT_DIR/reproduction-stderr.txt" 2>/dev/null || echo "No stderr captured")
\`\`\`

### Verbose Logs (RUST_LOG=debug)

See: \`reproduction-verbose-stderr.txt\` for full verbose logs

**Key Errors:**
\`\`\`
$(grep -i 'error\|panic\|fatal' "$BUG_REPORT_DIR/reproduction-verbose-stderr.txt" 2>/dev/null | head -20 || echo "No errors found in verbose logs")
\`\`\`

## Investigation Notes

### Potential Root Causes

[To be filled by investigator]

1. **Hypothesis 1:** [Description]
   - **Evidence:** [What supports this]
   - **Test:** [How to verify]

2. **Hypothesis 2:** [Description]
   - **Evidence:** [What supports this]
   - **Test:** [How to verify]

### Related Issues

- [Link to similar issues]
- [Link to related PRs]

### Attempted Fixes

[Document any attempted fixes and their results]

## Additional Context

[Any additional context, screenshots, or information]

## Suggested Fix

[If root cause is known, describe suggested fix]

## Files Included in Bug Report

- \`BUG-REPORT.md\` - This file (comprehensive summary)
- \`system-info.txt\` - Complete system information
- \`network-config.txt\` - Network configuration details
- \`reproduction-stdout.txt\` - Standard output from reproduction
- \`reproduction-stderr.txt\` - Standard error from reproduction
- \`reproduction-verbose-stdout.txt\` - Verbose output (RUST_LOG=debug)
- \`reproduction-verbose-stderr.txt\` - Verbose errors (RUST_LOG=debug)
- \`reproduction-exit-code.txt\` - Exit code from reproduction
- \`strace-summary.txt\` - System call trace summary (if available)

---

**Next Steps:**

1. Review logs for error messages
2. Test hypotheses in Investigation Notes
3. Create GitHub issue with this report
4. Implement fix and verify with reproduction command

EOF

echo "✅ Bug report generated"
```

---

## Phase 5: DISPLAY SUMMARY AND NEXT STEPS

**Objective:** Provide user with actionable summary

### Step 5.1: Display Summary

```bash
echo ""
echo "=========================================="
echo "Bug Report Generated Successfully"
echo "=========================================="
echo ""
echo "📋 ISSUE SUMMARY"
echo "  Issue: $ISSUE_SUMMARY"
echo "  Reproduction: $REPRO_CMD"
echo "  Exit Code: $REPRO_EXIT_CODE"
echo ""
echo "📁 BUG REPORT DIRECTORY"
echo "  Location: $BUG_REPORT_DIR"
echo ""
echo "📝 GENERATED FILES"
echo "  ✅ BUG-REPORT.md (comprehensive summary)"
echo "  ✅ system-info.txt (system details)"
echo "  ✅ network-config.txt (network configuration)"
echo "  ✅ reproduction-stdout.txt (standard output)"
echo "  ✅ reproduction-stderr.txt (standard error)"
echo "  ✅ reproduction-verbose-stderr.txt (verbose logs)"
echo "  ✅ reproduction-exit-code.txt (exit code)"
if [ -f "$BUG_REPORT_DIR/strace-summary.txt" ]; then
  echo "  ✅ strace-summary.txt (system call trace)"
fi
echo ""
echo "🔍 QUICK ANALYSIS"

# Check for common error patterns
if grep -qi 'panic' "$BUG_REPORT_DIR/reproduction-stderr.txt" 2>/dev/null; then
  echo "  ⚠️ PANIC detected in stderr"
fi

if grep -qi 'timeout' "$BUG_REPORT_DIR/reproduction-stderr.txt" 2>/dev/null; then
  echo "  ⚠️ TIMEOUT detected in stderr"
fi

if grep -qi 'permission denied' "$BUG_REPORT_DIR/reproduction-stderr.txt" 2>/dev/null; then
  echo "  ⚠️ PERMISSION DENIED detected in stderr"
fi

if [ "$REPRO_EXIT_CODE" -eq 124 ]; then
  echo "  ⚠️ Command TIMED OUT after 60 seconds"
fi

echo ""
echo "🚀 NEXT STEPS"
echo "  1. Review BUG-REPORT.md: $BUG_REPORT_DIR/BUG-REPORT.md"
echo "  2. Analyze verbose logs for root cause"
echo "  3. Test hypotheses and document in Investigation Notes"
echo "  4. Create GitHub issue with bug report"
echo "  5. Implement fix and verify with reproduction command"
echo ""
```

---

## SUCCESS CRITERIA

✅ Bug report directory created
✅ System information collected (OS, hardware, Rust, dependencies)
✅ Network configuration captured
✅ Reproduction command executed (standard, verbose, strace)
✅ Comprehensive BUG-REPORT.md generated
✅ Summary displayed with quick analysis

---

## DELIVERABLES

1. **BUG-REPORT.md:** Comprehensive markdown bug report
2. **system-info.txt:** Complete system details
3. **network-config.txt:** Network configuration
4. **reproduction logs:** stdout, stderr, verbose, strace
5. **Quick analysis:** Console summary with error detection

---

**Generate bug report: $***
