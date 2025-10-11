Fast targeted test execution - avoid full 643-test suite: $*

---

## QUICK TEST EXECUTION WORKFLOW

**Purpose:** Run specific tests or test subsets without executing the full 643-test suite (~5-6 minutes)

**Usage:** `/test-quick <pattern>`
- **pattern:** Test pattern to filter (package name, test name, or module path)

**Examples:**
- `/test-quick tcp_connect` - Run all TCP connect tests
- `/test-quick scheduler` - Run scheduler tests only
- `/test-quick prtip-network` - Run network package tests only

---

## Phase 1: PARSE TEST PATTERN AND STRATEGY

**Objective:** Determine test filtering strategy based on user pattern

### Step 1.1: Parse and Validate Pattern Argument

```bash
PATTERN="$*"

if [ -z "$PATTERN" ]; then
  echo "❌ ERROR: Test pattern required"
  echo ""
  echo "USAGE: /test-quick <pattern>"
  echo ""
  echo "COMMON PATTERNS:"
  echo "  Packages:    prtip-core, prtip-network, prtip-scanner, prtip-cli"
  echo "  Categories:  integration, unit, doc"
  echo "  Modules:     tcp_connect, syn_scanner, scheduler, progress"
  echo "  Features:    timing, rate_limit, blackrock, decoy"
  echo ""
  echo "EXAMPLES:"
  echo "  /test-quick tcp_connect    - Run TCP connect tests"
  echo "  /test-quick prtip-network  - Run network package tests"
  echo "  /test-quick integration    - Run integration tests only"
  echo ""
  exit 1
fi

# Validate pattern doesn't contain dangerous characters
if [[ "$PATTERN" =~ [;\&\|\`\$\(\)] ]]; then
  echo "❌ ERROR: Pattern contains invalid/dangerous characters"
  echo "Invalid characters: ; & | \` $ ( )"
  echo "Pattern: $PATTERN"
  echo ""
  echo "Use alphanumeric characters, underscores, hyphens, and spaces only"
  exit 1
fi

echo "✅ Test pattern validated: $PATTERN"
echo ""
```

### Step 1.2: Detect Pattern Type

```bash
PATTERN_TYPE="unknown"

# Check if pattern matches a package name
if [[ "$PATTERN" =~ ^prtip-(core|network|scanner|cli)$ ]]; then
  PATTERN_TYPE="package"
  PACKAGE="${PATTERN}"

# Check if pattern matches common test categories
elif [[ "$PATTERN" =~ ^(integration|unit|doc)$ ]]; then
  PATTERN_TYPE="category"

# Otherwise treat as test name filter
else
  PATTERN_TYPE="filter"
fi

echo "Pattern Type: $PATTERN_TYPE"
```

---

## Phase 2: EXECUTE TARGETED TESTS

**Objective:** Run only the filtered test subset for fast feedback

### Step 2.1: Package-Specific Tests

```bash
if [ "$PATTERN_TYPE" = "package" ]; then
  echo "Running all tests in package: $PACKAGE"
  echo ""

  cargo test --package "$PACKAGE"

  TEST_RESULT=$?

# Category-Based Tests
elif [ "$PATTERN_TYPE" = "category" ]; then
  case "$PATTERN" in
    integration)
      echo "Running integration tests only"
      cargo test --workspace --test '*'
      ;;
    unit)
      echo "Running unit tests only"
      cargo test --workspace --lib
      ;;
    doc)
      echo "Running doc tests only"
      cargo test --workspace --doc
      ;;
  esac

  TEST_RESULT=$?

# Filter-Based Tests (by test name)
else
  echo "Running tests matching pattern: $PATTERN"
  echo ""

  cargo test --workspace "$PATTERN"

  TEST_RESULT=$?
fi
```

### Step 2.2: Capture Test Metrics

```bash
# Parse test output (assumes standard cargo test output)
TOTAL_TESTS=$(grep -oP '\d+(?= tests)' /tmp/test-output.txt | tail -1 || echo "unknown")
PASSED_TESTS=$(grep -oP '\d+(?= passed)' /tmp/test-output.txt | tail -1 || echo "unknown")
FAILED_TESTS=$(grep -oP '\d+(?= failed)' /tmp/test-output.txt | tail -1 || echo "0")

# Calculate duration
TEST_DURATION=$(grep -oP 'finished in \K[\d.]+s' /tmp/test-output.txt | tail -1 || echo "unknown")
```

### Step 2.3: Extract Failed Tests (if any)

```bash
if [ "$TEST_RESULT" -ne 0 ] && [ "$FAILED_TESTS" != "0" ]; then
  echo ""
  echo "Extracting failed test details..."

  # Extract failed test names
  FAILED_LIST=$(grep -A 3 "^test " /tmp/test-output.txt 2>/dev/null | grep "FAILED" | awk '{print $2}' || echo "")

  if [ -n "$FAILED_LIST" ]; then
    echo ""
    echo "Failed Tests:"
    echo "============="
    echo "$FAILED_LIST" | while read -r test_name; do
      echo "  ❌ $test_name"
    done

    # Save to file for easy re-running
    echo "$FAILED_LIST" > /tmp/failed-tests.txt
    echo ""
    echo "Failed test list saved to: /tmp/failed-tests.txt"
    echo ""
    echo "Re-run failed tests:"
    echo "  while read test; do cargo test \$test -- --exact; done < /tmp/failed-tests.txt"
  fi
fi
```

---

## Phase 3: DISPLAY RESULTS AND RECOMMENDATIONS

**Objective:** Provide clear test results and next steps

### Step 3.1: Display Test Summary

```bash
echo ""
echo "=========================================="
echo "Quick Test Results"
echo "=========================================="
echo ""
echo "📊 TEST METRICS"
echo "  Pattern: $PATTERN"
echo "  Type: $PATTERN_TYPE"
echo "  Total Tests: $TOTAL_TESTS"
echo "  Passed: $PASSED_TESTS"
echo "  Failed: $FAILED_TESTS"
echo "  Duration: $TEST_DURATION"
echo ""

if [ "$TEST_RESULT" -eq 0 ]; then
  echo "✅ ALL TESTS PASSED"
else
  echo "❌ TESTS FAILED"
fi

echo ""
```

### Step 3.2: Provide Next Steps

```bash
if [ "$TEST_RESULT" -eq 0 ]; then
  echo "🚀 NEXT STEPS"
  echo "  1. Tests passing for pattern '$PATTERN'"
  echo "  2. Consider running full suite before commit: cargo test --workspace"
  echo "  3. Or continue development and test incrementally"

else
  echo "🔍 DEBUG FAILED TESTS"
  echo "  1. Run specific failing test with output:"
  echo "     cargo test <test_name> -- --nocapture"
  echo ""
  echo "  2. Run with logging enabled:"
  echo "     RUST_LOG=debug cargo test <test_name>"
  echo ""
  echo "  3. Isolate exact test:"
  echo "     cargo test <test_name> -- --exact"
  echo ""
  echo "  4. Show full backtrace:"
  echo "     RUST_BACKTRACE=1 cargo test <test_name>"
fi

echo ""
```

### Step 3.3: Optional Full Quality Check

```bash
if [ "$TEST_RESULT" -eq 0 ]; then
  echo "🔧 POST-TEST VALIDATION"
  echo ""
  read -p "Run full rust-check after tests? (y/N): " -n 1 -r
  echo ""
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Running comprehensive quality checks..."

    if [ -f ".claude/commands/rust-check.md" ]; then
      echo "💡 Execute: /rust-check for comprehensive validation"
      echo "   This will run format, clippy, all tests, and build checks"
    else
      echo "Running cargo clippy..."
      cargo clippy --all-targets -- -D warnings
    fi
  fi
fi
```

---

## COMMON TEST PATTERNS

### By Package

```bash
/test-quick prtip-core       # Core types and config (fastest)
/test-quick prtip-network    # Network packet handling
/test-quick prtip-scanner    # Scanner implementations
/test-quick prtip-cli        # CLI and output formatting
```

**Typical Duration:**
- prtip-core: ~10-15 seconds (fastest)
- prtip-network: ~20-30 seconds
- prtip-scanner: ~60-90 seconds (most tests)
- prtip-cli: ~15-20 seconds

### By Module

```bash
/test-quick tcp_connect      # TCP connect scanner tests
/test-quick syn_scanner      # SYN scanner tests
/test-quick scheduler        # Scheduler orchestration tests
/test-quick service_detector # Service detection tests
/test-quick progress         # Progress bar tests
```

### By Category

```bash
/test-quick integration      # Integration tests only (~2-3 min)
/test-quick unit             # Unit tests only (~3-4 min)
/test-quick doc              # Doc tests only (~30 sec)
```

### By Feature

```bash
/test-quick timing           # Timing template tests
/test-quick rate_limit       # Rate limiting tests
/test-quick blackrock        # Port randomization tests
/test-quick decoy            # Decoy scanning tests
```

---

## PERFORMANCE COMPARISON

| Test Type | Tests | Duration | Use Case |
|-----------|-------|----------|----------|
| **Full Suite** | 643 | ~5-6 min | Pre-commit, CI/CD |
| **prtip-core** | ~60 | ~10-15s | Core type changes |
| **prtip-scanner** | ~400 | ~60-90s | Scanner logic changes |
| **Integration** | ~50 | ~2-3 min | End-to-end validation |
| **Specific module** | ~10-30 | ~5-20s | Rapid iteration |

**Recommendation:** Use `/test-quick` for development iteration, full suite before commit

---

## ADVANCED USAGE

### Test Single Function

```bash
cargo test test_tcp_connect_basic -- --exact
```

**Shows only that specific test (no substring matches)**

### Test with Output

```bash
cargo test tcp_connect -- --nocapture
```

**Shows println! and debug output from tests**

### Test with Logging

```bash
RUST_LOG=debug cargo test scheduler
```

**Shows tracing logs during test execution**

### Test Multiple Patterns

```bash
cargo test tcp_connect syn_scanner
```

**Runs tests matching either pattern (OR logic)**

### List Tests Without Running

```bash
cargo test --workspace -- --list
```

**Shows all test names without execution**

### Show Test Binary

```bash
cargo test --no-run
```

**Compiles tests but doesn't run (check for compilation errors)**

---

## SUCCESS CRITERIA

✅ Pattern parsed and validated
✅ Tests executed (subset only, not full suite)
✅ Results displayed with metrics
✅ Duration <90 seconds for typical patterns
✅ Next steps provided based on results

---

## DELIVERABLES

1. **Test Results:** Pass/fail status for filtered tests
2. **Metrics:** Test count, duration, pass rate
3. **Recommendations:** Next steps based on results

---

**Run quick tests: $***
