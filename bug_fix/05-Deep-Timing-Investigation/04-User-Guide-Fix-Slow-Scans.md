# User Guide: Fix Slow Network Scans

**Problem:** Your scan is showing ~536 pps with 20-30 second "hangs" between hosts.

**Root Cause:** You're scanning mostly dead/unresponsive hosts with default conservative timeout (1000ms). Dead hosts take 20 seconds each with current config.

**Solution:** Optimize configuration for your network! Choose one of the options below.

---

## Quick Diagnosis: Is This Your Problem?

Run this test scan on a small subnet:
```bash
prtip --scan-type connect -p 1-10000 --progress 192.168.4.0/28 2>&1 | tee test-scan.log
```

Look for this pattern in the log:
```
[TIMING] ═══ HOST X COMPLETE: 20.0Xs ═══    ← Dead host (20 seconds)
[TIMING] ═══ HOST Y COMPLETE: 3.Xs ═══      ← Partial response (3 seconds)
[TIMING] ═══ HOST Z COMPLETE: 0.03s ═══     ← Live host (30ms!)
```

**If you see 20+ second hosts:** This guide is for you! Continue below.

**If all hosts are < 1 second:** Different issue - report as bug on GitHub.

---

## Solution 1: Quick Fix (Single Flag) ⭐ RECOMMENDED

**Use the T4 timing preset** (aggressive, optimized for LANs):

```bash
prtip --scan-type connect -p 1-10000 --timing-template T4 --progress 192.168.4.0/24
```

**What it does:**
- Sets timeout to 200ms (5x faster)
- Increases parallelism to 1000 (2x faster)
- **Combined: 10x faster scan** (70 min → 7 min)

**Prerequisites:**
```bash
# Check file descriptor limit
ulimit -n
# Should show >= 2000. If less, increase:
ulimit -n 4096
```

**Expected results:**
- Dead hosts: 2s each (vs 20s current)
- Live hosts: <100ms each
- Total scan time: ~7 minutes (vs 70 minutes)
- Smooth progress bar, no long "hangs"

---

## Solution 2: Best Practice (Discovery First) ⭐⭐ OPTIMAL

**Step 1:** Discover live hosts (fast)
```bash
prtip --discovery -p 80,443,22 192.168.4.0/24 -o live-hosts.txt
# Takes ~2 minutes, finds live hosts
```

**Step 2:** Scan only live hosts
```bash
prtip --scan-type connect -p 1-10000 -T4 --progress -t live-hosts.txt
# Takes ~8 seconds for 10-20 live hosts!
```

**Total time:** 2 minutes (vs 70 minutes) - **98% faster!**

**Why this is better:**
- Skips dead hosts entirely (most efficient)
- Industry standard workflow (Nmap does this by default)
- Minimal network impact
- Focuses resources on live targets

---

## Solution 3: Manual Tuning (Advanced)

**Adjust timeout and parallelism individually:**

### For Fast Local Networks (< 5ms latency)
```bash
prtip --scan-type connect -p 1-10000 --timeout 100 --parallel 1000 --progress 192.168.4.0/24
```
- Timeout: 100ms (10x faster than default)
- Parallelism: 1000
- **Dead host time: 1 second** (vs 20 seconds)
- Risk: May miss slow-responding hosts (acceptable for LANs)

### For Regular Networks (5-20ms latency)
```bash
prtip --scan-type connect -p 1-10000 --timeout 200 --parallel 1000 --progress 192.168.4.0/24
```
- Timeout: 200ms (5x faster than default)
- Parallelism: 1000
- **Dead host time: 2 seconds** (vs 20 seconds)
- Balanced: Speed + reliability

### For Internet/High-Latency Networks (> 50ms)
```bash
prtip --scan-type connect -p 1-10000 --timeout 500 --parallel 500 --progress 192.168.4.0/24
```
- Timeout: 500ms (2x faster than default)
- Parallelism: 500 (default, safe)
- **Dead host time: 10 seconds** (vs 20 seconds)
- Conservative: Won't miss slow hosts

---

## Solution 4: Configuration File (Set-and-Forget)

Create a configuration file for permanent optimized defaults:

**File:** `~/.prtip/config.toml`
```toml
[scan]
timeout_ms = 200              # 200ms timeout (fast LANs)
timing_template = "T4"        # Aggressive preset

[performance]
parallelism = 1000            # High concurrency
max_rate = 5000              # 5K packets/second

[output]
progress = true              # Always show progress
verbose = 1                  # Show what's happening
```

**Then run simplified commands:**
```bash
prtip -p 1-10000 192.168.4.0/24
# Automatically uses optimized config!
```

---

## Choosing the Right Solution

| Solution | Time Saved | Difficulty | Best For |
|----------|------------|------------|----------|
| **T4 Preset** | 90% | Easy (1 flag) | Quick scans, LANs |
| **Discovery First** | 98% | Medium (2 commands) | Large subnets, security audits |
| **Manual Tuning** | 50-90% | Advanced | Custom networks, specific needs |
| **Config File** | 90% | Medium (one-time setup) | Regular scanning, automation |

**Recommendation:**
1. **First-time users:** Try T4 preset (`--timing-template T4`)
2. **Regular users:** Set up config file for permanent optimization
3. **Security pros:** Use discovery workflow (`--discovery` → scan)

---

## Before and After Comparison

### Your Current Scan (SLOW)

**Command:**
```bash
prtip --scan-type connect -p 1-10000 --progress 192.168.4.0/24
```

**Results:**
```
[00:01:41] █▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 110000/2560000 ports
   Scan rate: 536 pps (packets per second)
   Estimated time: 76 minutes
   Observation: "Hangs for 20-30 seconds between hosts"
```

**Why it's slow:**
- Timeout: 1000ms (conservative default)
- Dead hosts: 20 seconds each
- 80% of 256 hosts are dead = 200 × 20s = 4,000s (67 minutes)

### After T4 Optimization (FAST)

**Command:**
```bash
prtip --scan-type connect -p 1-10000 --timing-template T4 --progress 192.168.4.0/24
```

**Expected Results:**
```
[00:01:41] ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░ 550000/2560000 ports
   Scan rate: 5,500 pps (10x faster!)
   Estimated time: 8 minutes (90% improvement!)
   Observation: "Smooth continuous progress, no noticeable hangs"
```

**Why it's fast:**
- Timeout: 200ms (5x faster)
- Parallelism: 1000 (2x faster)
- Dead hosts: 2 seconds each
- 200 × 2s = 400s (7 minutes)

### After Discovery Optimization (FASTEST)

**Commands:**
```bash
# Step 1: Discovery
prtip --discovery -p 80,443,22 192.168.4.0/24 -o live-hosts.txt

# Step 2: Scan live hosts only
prtip --scan-type connect -p 1-10000 -T4 --progress -t live-hosts.txt
```

**Expected Results:**
```
Discovery Phase:
[00:02:00] ████████████████████████████████████████ 256/256 hosts
   Found: 16 live hosts (6.25%)
   Time: 2 minutes

Port Scan Phase:
[00:00:08] ████████████████████████████████████████ 160000/160000 ports
   Scan rate: 20,000 pps (focused on live hosts!)
   Time: 8 seconds

Total time: 2 minutes 8 seconds (98% improvement!)
```

**Why it's fastest:**
- Only scans live hosts (16 instead of 256)
- Skips dead hosts entirely
- 94% fewer packets (160K vs 2.56M)

---

## Troubleshooting

### "ulimit: command not found" (Windows)

Windows doesn't have ulimit. Use PowerShell:
```powershell
# Check process limit (Windows equivalent)
Get-Process | Measure-Object -Property Handles -Sum
```

Npcap on Windows is limited to ~1000 concurrent connections. Use lower parallelism:
```bash
prtip --scan-type connect -p 1-10000 --timeout 200 --parallel 500 --progress 192.168.4.0/24
```

### "Too many open files" Error

Your system's file descriptor limit is too low:

**Check current limit:**
```bash
ulimit -n
```

**Increase temporarily (current session):**
```bash
ulimit -n 4096
```

**Increase permanently (Linux):**
```bash
# Edit /etc/security/limits.conf
sudo nano /etc/security/limits.conf

# Add these lines:
*               soft    nofile          4096
*               hard    nofile          65536

# Logout and login again
```

### Still Seeing 20-Second Hangs After T4

Check if T4 is actually being applied:
```bash
prtip --scan-type connect -p 1-10000 -T4 --progress 192.168.4.0/28 2>&1 | grep "Timeout:"
```

Should show:
```
Timeout:  200ms
```

If it shows `1000ms`, T4 is not being applied. Try explicit flags:
```bash
prtip --scan-type connect -p 1-10000 --timeout 200 --parallel 1000 --progress 192.168.4.0/28
```

### Network Administrator Complaints

If your network team complains about scan traffic:

**Reduce rate:**
```bash
prtip --scan-type connect -p 1-10000 -T4 --max-rate 1000 --progress 192.168.4.0/24
```
- Limits to 1000 packets/second (gentler on network)
- Still faster than current 536 pps
- Takes longer but less disruptive

**Use polite timing:**
```bash
prtip --scan-type connect -p 1-10000 --timing-template T2 --progress 192.168.4.0/24
```
- T2 = "Polite" mode (conservative)
- Slower than T4 but less network impact
- Good for production networks during business hours

---

## Performance Expectations

### Network Latency Impact

| Network Type | Latency | Recommended Timeout | Expected Rate |
|--------------|---------|---------------------|---------------|
| Localhost | < 1ms | 50ms | 10,000+ pps |
| Fast LAN | 1-5ms | 100ms | 5,000-10,000 pps |
| Regular LAN | 5-20ms | 200ms | 2,000-5,000 pps |
| Slow LAN/WAN | 20-50ms | 500ms | 1,000-2,000 pps |
| Internet | > 50ms | 1000ms | 500-1,000 pps |

**Your case:** Regular LAN (5-20ms latency)
**Recommended:** T4 preset (200ms timeout, 1000 parallel)
**Expected:** 5,000 pps, 7-minute total scan

### Progress Bar Interpretation

**Current (slow):**
```
[00:00:20] █░░░░░░░  10000/2560000  (536 pps)  ETA 76m
[00:00:40] █░░░░░░░  10000/2560000  (0 pps!)   ← 20s "hang"
[00:00:43] ██░░░░░░  20000/2560000  (536 pps)  ← resumed
```

**After T4 optimization:**
```
[00:00:02] ██░░░░░░  50000/2560000  (5500 pps)  ETA 8m
[00:00:04] ████░░░░  100000/2560000 (5500 pps)  ETA 7m
[00:00:06] ██████░░  150000/2560000 (5500 pps)  ETA 7m
```

**After discovery optimization:**
```
Discovery: 2 min → Found 16 hosts

[00:00:02] ████████  40000/160000  (20000 pps)  ETA 6s
[00:00:04] ████████████████  80000/160000  (20000 pps)  ETA 4s
[00:00:06] ████████████████████████  120000/160000  (20000 pps)  ETA 2s
[00:00:08] ████████████████████████████████████  160000/160000  COMPLETE
```

---

## Additional Tips

### 1. Test Configuration on Small Subnet First

Before scanning /24 (256 hosts), test on /28 (16 hosts):
```bash
prtip --scan-type connect -p 1-10000 -T4 --progress 192.168.4.0/28
# Should complete in < 1 minute
```

If that works well, scale up:
```bash
prtip --scan-type connect -p 1-10000 -T4 --progress 192.168.4.0/24
```

### 2. Save Results for Later Analysis

```bash
prtip --scan-type connect -p 1-10000 -T4 --progress 192.168.4.0/24 -o scan-results.json
```

Results saved to JSON for later review. Progress shown in real-time.

### 3. Scan Common Ports First

Instead of 1-10000, scan most likely ports:
```bash
prtip --scan-type connect -p 21,22,23,25,53,80,110,135,139,143,443,445,3306,3389,5432,8080 -T4 --progress 192.168.4.0/24
```
- Much faster (16 ports vs 10,000)
- Still finds most services
- Good for quick reconnaissance

### 4. Use Timing Template Reference

| Template | Timeout | Parallel | Use Case |
|----------|---------|----------|----------|
| T0 (Paranoid) | 5000ms | 10 | IDS evasion, stealth |
| T1 (Sneaky) | 3000ms | 50 | Slow, low-profile |
| T2 (Polite) | 2000ms | 100 | Production networks |
| **T3 (Normal)** | **1000ms** | **500** | **Default (your current)** |
| **T4 (Aggressive)** | **200ms** | **1000** | **Recommended for LANs** |
| T5 (Insane) | 50ms | 2000 | Max speed, may miss hosts |

---

## Quick Reference Command Cheat Sheet

```bash
# CURRENT (SLOW) - Don't use this!
prtip --scan-type connect -p 1-10000 --progress 192.168.4.0/24

# QUICK FIX - Add T4 flag
prtip --scan-type connect -p 1-10000 --timing-template T4 --progress 192.168.4.0/24

# BEST PRACTICE - Discovery first
prtip --discovery -p 80,443,22 192.168.4.0/24 -o live.txt
prtip --scan-type connect -p 1-10000 -T4 --progress -t live.txt

# MANUAL TUNING - Custom timeout
prtip --scan-type connect -p 1-10000 --timeout 200 --parallel 1000 --progress 192.168.4.0/24

# CHECK ULIMIT - Before high parallelism
ulimit -n  # Should be >= 2000

# INCREASE ULIMIT - If needed
ulimit -n 4096

# POLITE MODE - Less network impact
prtip --scan-type connect -p 1-10000 --timing-template T2 --progress 192.168.4.0/24

# SAVE RESULTS - For later analysis
prtip --scan-type connect -p 1-10000 -T4 --progress 192.168.4.0/24 -o results.json
```

---

## Need More Help?

1. **Check timing logs:**
   ```bash
   prtip --scan-type connect -p 1-10000 -T4 --progress 192.168.4.0/28 2>&1 | tee debug.log
   # Send debug.log if still having issues
   ```

2. **Report issues:**
   - GitHub: https://github.com/doublegate/ProRT-IP/issues
   - Include: Command used, network type, ulimit, OS version

3. **Community:**
   - Discussions: https://github.com/doublegate/ProRT-IP/discussions
   - Docs: https://github.com/doublegate/ProRT-IP/tree/main/docs

---

## Summary

**Problem:** 20-30 second "hangs", 536 pps, 76-minute ETA

**Root Cause:** Default 1000ms timeout + dead hosts = 20s per host

**Quick Fix:** Add `--timing-template T4` flag
- 70 min → 7 min (90% faster)
- 536 pps → 5,500 pps (10x faster)
- One flag change!

**Best Practice:** Use discovery first
- 70 min → 2 min (98% faster)
- Skip dead hosts entirely
- Industry standard workflow

**Choose:** T4 for quick wins, discovery for best results!

**No code changes needed** - scanner is working perfectly, just needs configuration tuning for your network! 🚀
