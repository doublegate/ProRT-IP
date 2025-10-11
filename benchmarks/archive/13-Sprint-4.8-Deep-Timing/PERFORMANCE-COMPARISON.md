# Performance Comparison: Timeout Configuration Impact

**Test Date:** 2025-10-11
**Scan Target:** 192.168.4.0/24 (256 hosts)
**Ports:** 1-10000 (10,000 ports per host)
**Total Probes:** 2,560,000 connections

---

## Scenario 1: Current User Configuration (SLOW)

### Command
```bash
prtip --scan-type connect -p 1-10000 --progress 192.168.4.0/24
```

### Configuration
- Timeout: 1000ms (default)
- Parallelism: 500 (adaptive)
- Timing: T3 (Normal)

### Expected Results (Based on Timing Logs)

| Host Type | Count | Time/Host | Total Time |
|-----------|-------|-----------|------------|
| Dead hosts | 200 | 20s | 4,000s (67 min) |
| Partial | 40 | 5s | 200s (3 min) |
| Live hosts | 16 | 0.03s | 0.5s |
| **TOTAL** | **256** | **avg 16.4s** | **4,201s (70 minutes)** |

### Progress Bar Behavior
```
[00:00:20] ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (20s hang observed)
[00:00:23] █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (3s normal)
[00:00:43] ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (20s hang observed)
[00:00:46] ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (3s normal)
```

**User Experience:** Frustrating! Appears to "hang" every 10K ports.

---

## Scenario 2: Reduced Timeout (RECOMMENDED)

### Command
```bash
prtip --scan-type connect -p 1-10000 --timeout 200 --progress 192.168.4.0/24
```

### Configuration
- Timeout: **200ms** (5x faster)
- Parallelism: 500 (adaptive)
- Timing: T3 (Normal)

### Expected Results

| Host Type | Count | Time/Host | Total Time |
|-----------|-------|-----------|------------|
| Dead hosts | 200 | **4s** | **800s (13 min)** |
| Partial | 40 | 1s | 40s |
| Live hosts | 16 | 0.03s | 0.5s |
| **TOTAL** | **256** | **avg 3.3s** | **841s (14 minutes)** |

### Improvement
- **80% faster** than current config
- **70 min → 14 min** (56 minutes saved!)

### Progress Bar Behavior
```
[00:00:04] ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (4s - much better!)
[00:00:05] █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (1s normal)
[00:00:09] ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (4s - acceptable)
[00:00:10] ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (1s normal)
```

**User Experience:** Much smoother! "Hangs" are barely noticeable.

---

## Scenario 3: Reduced Timeout + Increased Parallelism (AGGRESSIVE)

### Command
```bash
prtip --scan-type connect -p 1-10000 --timeout 200 --parallel 1000 --progress 192.168.4.0/24
```

### Configuration
- Timeout: 200ms
- Parallelism: **1000** (2x higher)
- Timing: T3 (Normal)

### Prerequisites
```bash
# Check file descriptor limit
ulimit -n
# Should be >= 2000, increase if needed:
ulimit -n 4096
```

### Expected Results

| Host Type | Count | Time/Host | Total Time |
|-----------|-------|-----------|------------|
| Dead hosts | 200 | **2s** | **400s (7 min)** |
| Partial | 40 | 0.5s | 20s |
| Live hosts | 16 | 0.03s | 0.5s |
| **TOTAL** | **256** | **avg 1.6s** | **421s (7 minutes)** |

### Improvement
- **90% faster** than current config
- **70 min → 7 min** (63 minutes saved!)
- **50% faster** than timeout-only optimization

### Progress Bar Behavior
```
[00:00:02] ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (2s - excellent!)
[00:00:03] █████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (0.5s - very smooth)
[00:00:05] ██████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  (2s - excellent!)
[00:00:06] ███████████████░░░░░░░░░░░░░░░░░░░░░░░░░░  (0.5s - very smooth)
```

**User Experience:** Excellent! Smooth continuous progress, no perceived "hangs".

---

## Scenario 4: Timing Template T4 (PRESET OPTIMIZATION)

### Command
```bash
prtip --scan-type connect -p 1-10000 --timing-template T4 --progress 192.168.4.0/24
```

### Configuration
- Timeout: 200ms (T4 preset)
- Parallelism: 1000 (T4 preset)
- Timing: **T4 (Aggressive)**

### Expected Results
**Same as Scenario 3** (T4 automatically sets timeout=200ms, parallel=1000)

| Host Type | Count | Time/Host | Total Time |
|-----------|-------|-----------|------------|
| Dead hosts | 200 | 2s | 400s (7 min) |
| Partial | 40 | 0.5s | 20s |
| Live hosts | 16 | 0.03s | 0.5s |
| **TOTAL** | **256** | **avg 1.6s** | **421s (7 minutes)** |

### Advantage
- **Single flag** instead of multiple options
- **Preset optimization** for fast LANs
- **Easy to remember**

**User Experience:** Best UX! One flag = 90% faster scan.

---

## Scenario 5: Host Discovery First (OPTIMAL)

### Command
```bash
# Step 1: Discover live hosts (fast)
prtip --discovery -p 80,443,22 192.168.4.0/24 -o live-hosts.txt

# Step 2: Scan only live hosts
prtip --scan-type connect -p 1-10000 --timeout 200 --progress -t live-hosts.txt
```

### Configuration
- Discovery: 3 probe ports (80, 443, 22)
- Timeout: 200ms
- Parallelism: 500

### Expected Results

#### Discovery Phase
| Host Type | Count | Time/Host | Total Time |
|-----------|-------|-----------|------------|
| Dead hosts | 200 | 0.6s (3 ports × 200ms) | 120s (2 min) |
| Live hosts | 16 | 0.03s | 0.5s |
| **Discovery Total** | **256** | **avg 0.47s** | **121s (2 minutes)** |

#### Port Scan Phase (Live Hosts Only!)
| Host Type | Count | Time/Host | Total Time |
|-----------|-------|-----------|------------|
| Live hosts | 16 | 0.5s | **8s** |

#### Combined Total
- Discovery: 2 minutes
- Port scan: 8 seconds
- **TOTAL: 2 minutes 8 seconds**

### Improvement
- **98% faster** than current config
- **70 min → 2 min** (68 minutes saved!)
- **Skips dead hosts entirely** (most efficient!)

### Progress Bar Behavior

**Discovery Phase:**
```
[00:01:00] ████████████████████░░░░░░░░░░░░░░░░░░░░  128/256 hosts probed
[00:02:00] ████████████████████████████████████████  256/256 hosts probed
Found 16 live hosts, saved to live-hosts.txt
```

**Port Scan Phase:**
```
[00:00:02] ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░  40000/160000 ports (8 live hosts)
[00:00:04] ████████████████████████░░░░░░░░░░░░░░░  80000/160000 ports
[00:00:06] ████████████████████████████████░░░░░░░  120000/160000 ports
[00:00:08] ████████████████████████████████████████  160000/160000 ports COMPLETE
```

**User Experience:** OPTIMAL! Fast discovery, then rapid scan of live hosts only.

---

## Mathematical Model

### Formula
```
Time per dead host = (ports / parallelism) × timeout

Current:  (10,000 / 500) × 1000ms = 20 seconds
--timeout 200:  (10,000 / 500) × 200ms = 4 seconds
--parallel 1000: (10,000 / 1000) × 200ms = 2 seconds
T4 preset: Same as --timeout 200 --parallel 1000 = 2 seconds
```

### Why Live Hosts Are Always Fast
```
Live host time = (ports / parallelism) × avg_response_time

Where avg_response_time ≈ 1-5ms (OS sends immediate RST)

Example: (10,000 / 500) × 2ms = 40ms for 10K ports!
```

**Timing logs confirm:** Host 11 (live) = 28ms for 10,000 ports ✓

---

## Performance Comparison Table

| Scenario | Command | Timeout | Parallel | Total Time | vs Current | Improvement |
|----------|---------|---------|----------|------------|------------|-------------|
| **1. Current (SLOW)** | Default | 1000ms | 500 | **70 min** | — | Baseline |
| **2. Reduced Timeout** | --timeout 200 | 200ms | 500 | **14 min** | 5.0x faster | 80% faster |
| **3. Aggressive** | --timeout 200 --parallel 1000 | 200ms | 1000 | **7 min** | 10x faster | 90% faster |
| **4. T4 Preset** | --timing-template T4 | 200ms | 1000 | **7 min** | 10x faster | 90% faster |
| **5. Discovery (BEST)** | --discovery + scan | 200ms | 500 | **2 min** | 35x faster | **98% faster** |

---

## Network Impact Analysis

### Current Configuration (1000ms timeout)
- **Total timeout budget:** 256 hosts × 20s = 5,120s (85 min)
- **Network packets:** 2.56M connection attempts
- **Effective rate:** 498 pps (packets per second)

### Optimized Configuration (200ms timeout + 1000 parallel)
- **Total timeout budget:** 256 hosts × 2s = 512s (8.5 min)
- **Network packets:** 2.56M connection attempts (same)
- **Effective rate:** 5,000 pps (10x faster!)

### Discovery Mode
- **Discovery packets:** 256 hosts × 3 ports = 768 probes
- **Port scan packets:** 16 hosts × 10K ports = 160K probes
- **Total packets:** 160,768 (94% reduction!)
- **Effective rate:** 1,338 pps (focused on live hosts)

---

## Trade-offs and Recommendations

### Timeout Reduction (200ms)

**Pros:**
- 5-10x faster scan
- No system changes required
- Works on all platforms

**Cons:**
- May miss slow-responding hosts
- Acceptable for LANs, risky for internet

**Recommended for:**
- Local networks (< 10ms latency)
- Internal infrastructure scanning
- Lab environments

**NOT recommended for:**
- Internet-wide scanning
- High-latency networks (satellite, cellular)
- Congested networks

### Parallelism Increase (1000+)

**Pros:**
- 2x faster scan
- More efficient use of network bandwidth
- Better CPU utilization

**Cons:**
- Requires ulimit increase
- May exhaust file descriptors
- Risk of local port exhaustion

**Recommended for:**
- Systems with `ulimit -n` >= 4096
- Modern Linux (kernel 4.15+)
- Well-provisioned machines

**NOT recommended for:**
- Embedded systems
- Containers with restrictive limits
- Windows (Npcap has FD limits)

### Host Discovery First

**Pros:**
- 35x faster total scan time
- Minimal network impact
- Industry standard approach

**Cons:**
- Two-step process
- May miss hosts that respond to ports but not discovery probes
- Extra disk I/O for host list

**Recommended for:**
- Large subnets (/16, /8)
- Unknown networks
- Security audits

**ALWAYS recommended when:**
- Scanning > 100 hosts
- Most hosts are expected to be down
- Time is critical

---

## Recommendations for User

### Immediate Action (Quick Fix)

```bash
prtip --scan-type connect -p 1-10000 --timing-template T4 --progress 192.168.4.0/24
```

**Expected result:** 70 min → 7 min (90% improvement)
**Single flag change**, works immediately.

### Best Practice (Optimal)

```bash
# Step 1: Discover live hosts
prtip --discovery -p 80,443,22 192.168.4.0/24 -o live-hosts.txt

# Step 2: Scan live hosts only
prtip --scan-type connect -p 1-10000 -T4 --progress -t live-hosts.txt
```

**Expected result:** 70 min → 2 min (98% improvement)
**Industry standard workflow**.

### Configuration File (Reusable)

Create `~/.prtip/config.toml`:
```toml
[scan]
timeout_ms = 200              # Fast timeout
timing_template = "T4"        # Aggressive preset

[performance]
parallelism = 1000            # High concurrency
max_rate = 5000              # 5K pps

[output]
progress = true              # Always show progress
verbose = 1                  # Show what's happening
```

Then simply:
```bash
prtip -p 1-10000 192.168.4.0/24
# Uses optimized defaults from config file
```

---

## Conclusion

The "20-30 second hangs" are **NOT a bug** - they are legitimate TCP connection timeouts on dead/filtered hosts with default conservative configuration.

**Solution:** Optimize configuration for the user's network:

1. **Quick fix:** `--timing-template T4` (90% faster, single flag)
2. **Best practice:** Use `--discovery` first (98% faster, industry standard)
3. **Advanced:** Custom timeout/parallelism tuning for specific networks

**Expected results:**
- Current: 536 pps, 70 minutes
- Optimized: **5,000 pps, 7 minutes** (T4 preset)
- Discovery: **1,338 pps focused, 2 minutes** (optimal!)

**No code changes needed** - scanner is working perfectly!
