# Performance Fix Test Plan

**Date:** 2025-10-11
**Fix:** Total-scan-aware adaptive polling intervals

---

## Test Scenarios

### 1. Localhost 10K Ports (Regression Test)
**Purpose:** Ensure fix doesn't break fast localhost scans

```bash
time ./target/release/prtip --scan-type connect -p 1-10000 --progress 127.0.0.1
```

**Expected:**
- Duration: 40-100ms
- Rate: 100K-250K pps
- Progress: Smooth incremental updates (not jumping to 100%)
- Poll interval: 500µs (10,000 < 10,000 threshold)

---

### 2. Localhost 1K Ports (Tiny Scan)
**Purpose:** Verify 0.2ms polling still works for ultra-fast scans

```bash
time ./target/release/prtip --scan-type connect -p 1-1000 --progress 127.0.0.1
```

**Expected:**
- Duration: 4-10ms
- Rate: 100K-250K pps
- Progress: Incremental updates
- Poll interval: 200µs (1,000 < 1,000 threshold)

---

### 3. Small Network (16 hosts × 1K ports = 16K)
**Purpose:** Test medium scan threshold

```bash
time ./target/release/prtip --scan-type connect -p 1-1000 --progress 192.168.1.0/28
```

**Expected:**
- Duration: 5-20 seconds (network latency)
- Rate: 800-3,200 pps
- Progress: Smooth updates
- Poll interval: 1ms (16,000 < 100,000 threshold)

---

### 4. User's Exact Scenario (256 hosts × 10K ports = 2.56M)
**Purpose:** Validate fix for reported issue

```bash
time ./target/release/prtip --scan-type connect -p 1-10000 --progress 192.168.4.0/24
```

**Expected (BEFORE FIX):**
- Duration: 2 hours
- Rate: 289 pps
- Poll interval: 1ms (WRONG - based on 10K ports per host)

**Expected (AFTER FIX):**
- Duration: 10-30 minutes
- Rate: 1,500-4,500 pps
- Poll interval: 10ms (2.56M ≥ 1M threshold)
- Improvement: **4-12x faster**

---

### 5. Huge Scan (1M+ ports)
**Purpose:** Verify 10ms polling for massive scans

```bash
time ./target/release/prtip --scan-type connect -p 1-1000 --progress 10.0.0.0/22
```

**Note:** 1024 hosts × 1000 ports = 1,024,000 total ports
**Expected:**
- Poll interval: 10ms (1.024M ≥ 1M threshold)
- Progress: Updates every 1-2 seconds
- Overhead: < 5% of scan time

---

## Performance Metrics to Capture

For each test, record:

1. **Total duration** (wall clock time)
2. **Average rate** (ports per second)
3. **Progress updates** (should be smooth, not frozen)
4. **Final statistics** (open/closed/filtered counts)
5. **Poll interval** (verify correct threshold selected)

---

## Success Criteria

### Must Pass

✅ **Test 1 (Localhost 10K):** Duration < 100ms, rate > 100K pps
✅ **Test 2 (Localhost 1K):** Duration < 10ms, rate > 100K pps
✅ **Test 4 (User's scan):** Rate > 1,000 pps (3-10x faster than 289 pps)

### Nice to Have

✅ **Test 3 (Small net):** Rate > 500 pps
✅ **Test 4 (User's scan):** Duration < 30 minutes (8x faster than 2 hours)
✅ **Test 5 (Huge scan):** Progress updates every 1-2 seconds

---

## Verification Commands

```bash
# Build release binary
cargo build --release

# Run all tests
./scripts/performance-regression-tests.sh

# Individual tests
time ./target/release/prtip --scan-type connect -p 1-1000 --progress 127.0.0.1
time ./target/release/prtip --scan-type connect -p 1-10000 --progress 127.0.0.1
```

---

## Expected Polling Intervals

| Total Ports | Threshold | Interval | Example Scenario |
|-------------|-----------|----------|------------------|
| 100 | < 1K | 200µs | 1 host × 100 ports |
| 1,000 | < 1K | 200µs | 1 host × 1K ports |
| 10,000 | < 10K | 500µs | 1 host × 10K ports |
| 16,000 | < 100K | 1ms | 16 hosts × 1K ports |
| 100,000 | < 100K | 1ms | 10 hosts × 10K ports |
| 256,000 | < 1M | 5ms | 256 hosts × 1K ports |
| 2,560,000 | ≥ 1M | 10ms | 256 hosts × 10K ports |
| 65,536,000 | ≥ 1M | 10ms | 65,536 hosts × 1K ports |

---

## Notes

- **Localhost tests** run on 127.0.0.1 (loopback, no network latency)
- **Network tests** depend on actual network topology (results may vary)
- **User's scenario** (192.168.4.0/24) requires local network setup
- **Progress bar smoothness** is subjective but should not appear frozen
