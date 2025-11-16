# Timing Templates

Control scan speed and stealth through six predefined timing templates (T0-T5).

## What are Timing Templates?

**Timing templates** are predefined configurations that control how aggressively ProRT-IP scans targets. They provide a simple way to balance three competing priorities:

- **Speed**: How fast the scan completes
- **Stealth**: How likely the scan is to evade detection (IDS/IPS)
- **Accuracy**: How reliably the scan detects open ports

**Nmap Compatibility:** ProRT-IP's timing templates are compatible with Nmap's `-T0` through `-T5` flags, making migration straightforward for existing Nmap users.

---

## Template Overview

| Template | Flag | Name | Speed | Stealth | Use Case |
|----------|------|------|-------|---------|----------|
| **T0** | `-T0` | Paranoid | Extremely slow | Maximum | IDS/IPS evasion, stealth operations |
| **T1** | `-T1` | Sneaky | Very slow | High | Avoid detection, low-priority scans |
| **T2** | `-T2` | Polite | Slow | Medium | Production environments, courtesy |
| **T3** | `-T3` | Normal | Moderate | Low | Default, balanced performance |
| **T4** | `-T4` | Aggressive | Fast | Very low | Local networks, time-sensitive |
| **T5** | `-T5` | Insane | Extremely fast | None | Maximum speed, may miss results |

**Default:** T3 (Normal) if no `-T` flag is specified.

**Selection Guide:**
- **Unknown network?** Start with T2 (Polite), increase if safe
- **Local network?** Use T4 (Aggressive) for speed
- **Stealth required?** Use T0 (Paranoid) or T1 (Sneaky)
- **Production environment?** Use T2 (Polite) to avoid disruption
- **Need speed?** Use T4 (Aggressive) or T5 (Insane), but verify results

---

## Timing Parameters

Each template configures eight timing parameters:

### 1. Initial Timeout
**What it controls:** How long to wait for a response before declaring a port non-responsive.

**Impact:**
- **Too low:** Miss open ports on slow networks (false negatives)
- **Too high:** Waste time waiting for closed/filtered ports

**Range:** 250ms (T5) to 300s (T0)

### 2. Min Timeout
**What it controls:** Minimum timeout value that adaptive algorithms cannot go below.

**Impact:**
- **Safety net:** Prevents timeouts from becoming too aggressive
- **Ensures accuracy:** Guarantees minimum wait time even on fast networks

**Range:** 50ms (T5) to 100s (T0)

### 3. Max Timeout
**What it controls:** Maximum timeout value that adaptive algorithms cannot exceed.

**Impact:**
- **Performance cap:** Prevents excessive waiting
- **Bounds worst case:** Limits time spent on unresponsive targets

**Range:** 300ms (T5) to 300s (T0)

### 4. Max Retries
**What it controls:** Number of times to retry a probe before giving up.

**Impact:**
- **More retries:** Higher accuracy, slower scans
- **Fewer retries:** Faster scans, may miss intermittent responses

**Range:** 2 (T3, T5) to 6 (T4)

### 5. Scan Delay
**What it controls:** Delay between consecutive probes to the same target.

**Impact:**
- **Longer delays:** Lower network load, more stealthy
- **Zero delay:** Maximum speed, may trigger rate limiting

**Range:** 0ms (T3-T5) to 300s (T0)

### 6. Max Parallelism
**What it controls:** Maximum number of probes in flight simultaneously.

**Impact:**
- **Higher parallelism:** Faster scans, higher network load
- **Lower parallelism:** Slower scans, more stealthy, lower resource usage

**Range:** 1 (T0) to 10,000 (T5)

### 7. Enable Jitter
**What it controls:** Whether to randomize timing to evade pattern detection.

**Impact:**
- **Enabled:** Harder to detect by IDS/IPS, slightly slower
- **Disabled:** Predictable timing, easier to detect

**Values:** true (T0-T2), false (T3-T5)

### 8. Jitter Factor
**What it controls:** Amount of randomness applied to delays (percentage variance).

**Impact:**
- **Higher factor:** More randomness, better IDS evasion
- **Zero factor:** No randomness, predictable timing

**Range:** 0.0 (T3-T5) to 0.3 (T0)

---

## T0: Paranoid

**Goal:** Maximum stealth, evade even the most sensitive IDS/IPS systems.

**Use Cases:**
- Penetration testing against heavily monitored networks
- Red team operations requiring complete stealth
- Scanning highly sensitive targets
- Avoiding security alerts at all costs

**Configuration:**

```rust
initial_timeout: 300 seconds     // 5 minutes
min_timeout: 100 seconds         // 1 minute 40 seconds
max_timeout: 300 seconds         // 5 minutes
max_retries: 5
scan_delay: 300 seconds          // 5 minutes between probes
max_parallelism: 1               // One probe at a time
enable_jitter: true
jitter_factor: 0.3               // ±30% timing variance
```

**Performance Characteristics:**
- **Speed:** ~300 seconds per port (5 minutes/port)
- **Example:** Scanning 100 ports takes ~500 hours (20+ days)
- **Network load:** Negligible (1 probe every 5 minutes)
- **Detection risk:** Minimal (spacing defeats IDS correlation)

**Command Examples:**

```bash
# Basic T0 scan
sudo prtip -T0 -p 80,443 target.example.com

# T0 scan with extended port range (very slow)
sudo prtip -T0 -p 1-1000 192.168.1.1
# Expected duration: ~347 days for 1,000 ports

# T0 scan on subnet (not recommended - extremely slow)
sudo prtip -T0 -p 22,80,443 192.168.1.0/24
# Expected duration: Months to years
```

**Best Practices:**
- ✅ Use for small port lists (1-10 ports maximum)
- ✅ Run overnight or over weekends
- ✅ Monitor progress with verbose output (`-v`)
- ❌ **Never** use for large port ranges or subnets
- ❌ **Never** use for time-sensitive operations

**When to Use:**
- You have **unlimited time** and **zero tolerance** for detection
- Target has **known IDS/IPS** with aggressive correlation
- Legal/compliance requirements mandate **maximum stealth**
- Red team engagement with **strict stealth rules of engagement**

**Performance vs T3 (Normal):** ~6,000x slower

---

## T1: Sneaky

**Goal:** High stealth while maintaining reasonable scan times.

**Use Cases:**
- Evading basic IDS/IPS systems
- Scanning production environments cautiously
- Avoiding rate limiting on sensitive targets
- Stealth reconnaissance with time constraints

**Configuration:**

```rust
initial_timeout: 15 seconds
min_timeout: 5 seconds
max_timeout: 15 seconds
max_retries: 5
scan_delay: 15 seconds           // 15 seconds between probes
max_parallelism: 10              // 10 concurrent probes
enable_jitter: true
jitter_factor: 0.2               // ±20% timing variance
```

**Performance Characteristics:**
- **Speed:** ~15 seconds per port (with parallelism)
- **Example:** Scanning 100 ports takes ~2.5 minutes (10 parallel streams)
- **Network load:** Very low (10 probes/15 seconds = 0.67 pps)
- **Detection risk:** Low (spacing + jitter defeats basic IDS)

**Command Examples:**

```bash
# Basic T1 scan
sudo prtip -T1 -p 1-1000 target.example.com
# Expected duration: ~25 minutes

# T1 scan on small subnet
sudo prtip -T1 -p 80,443,8080 192.168.1.0/24
# Expected duration: ~1 hour for 256 hosts × 3 ports

# T1 with service detection
sudo prtip -T1 -sV -p 22,80,443 target.example.com
# Expected duration: ~1-2 minutes
```

**Best Practices:**
- ✅ Use for moderate port ranges (1-5,000 ports)
- ✅ Suitable for small subnets (/24-/28)
- ✅ Good balance of stealth and practicality
- ✅ Monitor with verbose output for progress
- ⚠️ Still slow for large networks

**When to Use:**
- Target has **moderate IDS/IPS** monitoring
- You can afford **minutes to hours** for scan completion
- Stealth is **important** but not absolute priority
- Avoiding **rate limiting** on API endpoints or web servers

**Performance vs T3 (Normal):** ~100-200x slower

---

## T2: Polite

**Goal:** Courteous scanning that minimizes network impact.

**Use Cases:**
- Production environment scanning
- Scanning customer networks
- Compliance-driven security audits
- Avoiding rate limiting on web servers

**Configuration:**

```rust
initial_timeout: 10 seconds
min_timeout: 1 second
max_timeout: 10 seconds
max_retries: 5
scan_delay: 400 milliseconds     // 0.4 seconds between probes
max_parallelism: 100             // 100 concurrent probes
enable_jitter: true
jitter_factor: 0.1               // ±10% timing variance
```

**Performance Characteristics:**
- **Speed:** ~400ms per port (with parallelism)
- **Example:** Scanning 1,000 ports takes ~4 seconds (100 parallel streams)
- **Network load:** Low (~250 pps sustained)
- **Detection risk:** Medium (normal traffic pattern)

**Command Examples:**

```bash
# Basic T2 scan (production safe)
sudo prtip -T2 -p 1-10000 target.example.com
# Expected duration: ~40 seconds

# T2 subnet scan
sudo prtip -T2 -p 80,443 192.168.1.0/24
# Expected duration: ~2 minutes for 256 hosts × 2 ports

# T2 with comprehensive service detection
sudo prtip -T2 -sV -O -p 1-5000 target.example.com
# Expected duration: ~30-60 seconds
```

**Best Practices:**
- ✅ **Recommended default** for production environments
- ✅ Use for customer networks and audits
- ✅ Safe for large port ranges (1-65,535)
- ✅ Suitable for /16 to /24 subnets
- ✅ Balances speed and courtesy

**When to Use:**
- Scanning **production systems** during business hours
- **Compliance requirements** mandate low-impact scanning
- Avoiding **rate limiting** or **throttling**
- Customer-facing security audits
- Default choice when **stealth not required** but **courtesy important**

**Performance vs T3 (Normal):** ~2-3x slower

---

## T3: Normal (Default)

**Goal:** Balanced performance for general-purpose scanning.

**Use Cases:**
- Default scanning mode
- Internal network assessments
- Security research
- Most penetration testing scenarios

**Configuration:**

```rust
initial_timeout: 3 seconds
min_timeout: 500 milliseconds
max_timeout: 10 seconds
max_retries: 2
scan_delay: 0 milliseconds       // No artificial delay
max_parallelism: 1000            // 1,000 concurrent probes
enable_jitter: false
jitter_factor: 0.0               // No jitter
```

**Performance Characteristics:**
- **Speed:** ~3ms per port (with parallelism, local network)
- **Example:** Scanning 65,535 ports takes ~3-5 seconds (local network)
- **Network load:** Moderate (~10,000-50,000 pps burst)
- **Detection risk:** High (normal scan signature)

**Command Examples:**

```bash
# Basic T3 scan (default, -T3 can be omitted)
sudo prtip -p 1-65535 192.168.1.1
# Expected duration: ~5-10 seconds (local network)

# T3 subnet scan
sudo prtip -p 80,443,8080 192.168.0.0/16
# Expected duration: ~5-10 minutes for 65,536 hosts × 3 ports

# T3 with all detection features
sudo prtip -A -p 1-10000 target.example.com
# Expected duration: ~30-60 seconds
```

**Best Practices:**
- ✅ **Default choice** for most scenarios
- ✅ Excellent for internal network assessments
- ✅ Fast enough for large networks
- ✅ Accurate on stable networks
- ⚠️ May trigger IDS/IPS alerts
- ⚠️ Can overwhelm slow/congested networks

**When to Use:**
- **Internal network** scanning (trusted environment)
- **No stealth requirement** (authorized testing)
- **Balanced performance** needed (not maximum speed)
- **General-purpose** security assessments
- Default choice when **no specific timing requirements**

**Performance Baseline:** This is the reference template (1.0x speed)

---

## T4: Aggressive

**Goal:** Fast scanning for local networks and time-sensitive operations.

**Use Cases:**
- Local network scanning (LAN)
- Time-critical assessments
- High-bandwidth environments
- CTF competitions
- Internal penetration testing

**Configuration:**

```rust
initial_timeout: 1 second
min_timeout: 100 milliseconds
max_timeout: 1.25 seconds        // Lower max than default
max_retries: 6                   // More retries for reliability
scan_delay: 0 milliseconds       // No artificial delay
max_parallelism: 5000            // 5,000 concurrent probes
enable_jitter: false
jitter_factor: 0.0               // No jitter
```

**Performance Characteristics:**
- **Speed:** ~1ms per port (local network, high parallelism)
- **Example:** Scanning 65,535 ports takes ~1-2 seconds (local network)
- **Network load:** High (~50,000-100,000 pps burst)
- **Detection risk:** Very high (obvious scan signature)
- **Accuracy:** Good on local networks, may miss results on slow/internet targets

**Command Examples:**

```bash
# Basic T4 local network scan
sudo prtip -T4 -p- 192.168.1.1
# Expected duration: ~1-2 seconds for all 65,535 ports

# T4 subnet sweep
sudo prtip -T4 -p 22,80,443,3389 192.168.0.0/16
# Expected duration: ~2-5 minutes for 65,536 hosts × 4 ports

# T4 with service detection (local network)
sudo prtip -T4 -sV -p 1-10000 192.168.1.10
# Expected duration: ~10-20 seconds
```

**Best Practices:**
- ✅ **Excellent for local networks** (LAN/data center)
- ✅ Use when **speed is critical** and accuracy can be verified
- ✅ **High-bandwidth environments** (10+ Gbps)
- ⚠️ **Not recommended for internet targets** (packet loss likely)
- ⚠️ **May overwhelm** slow networks or endpoints
- ⚠️ **Will trigger IDS/IPS** alerts (obvious scan)
- ❌ **Never use** on production internet-facing systems without permission

**When to Use:**
- **Local network** scanning (192.168.x.x, 10.x.x.x)
- **Time-critical** assessments (incident response, CTF)
- **High-bandwidth** environments (data center, lab)
- **Internal penetration testing** with permission
- You can **verify results** afterward (accept some false negatives)

**Performance vs T3 (Normal):** ~5-10x faster

**Warning:** On internet targets, T4 often **performs worse** than T3 due to packet loss from aggressive timeouts. Use T3 for internet scans.

---

## T5: Insane

**Goal:** Maximum speed at the cost of accuracy and reliability.

**Use Cases:**
- Quick host discovery on local networks
- Initial reconnaissance (followed by slower verification)
- CTF competitions with strict time limits
- High-bandwidth lab environments
- Situations where false negatives are acceptable

**Configuration:**

```rust
initial_timeout: 250 milliseconds
min_timeout: 50 milliseconds
max_timeout: 300 milliseconds    // Very aggressive cap
max_retries: 2                   // Minimal retries
scan_delay: 0 milliseconds       // No artificial delay
max_parallelism: 10000           // 10,000 concurrent probes
enable_jitter: false
jitter_factor: 0.0               // No jitter
```

**Performance Characteristics:**
- **Speed:** ~0.5ms per port (local network, maximum parallelism)
- **Example:** Scanning 65,535 ports takes ~0.5-1 second (local network)
- **Network load:** Extreme (~100,000+ pps burst)
- **Detection risk:** Maximum (unmistakable scan signature)
- **Accuracy:** **Poor on anything but fast local networks** (high false negative rate)

**Command Examples:**

```bash
# Basic T5 local network scan (extremely fast)
sudo prtip -T5 -p- 192.168.1.1
# Expected duration: ~0.5-1 second for all 65,535 ports

# T5 subnet discovery (quick check for live hosts)
sudo prtip -T5 -sn 192.168.0.0/16
# Expected duration: ~30-60 seconds for 65,536 hosts

# T5 common ports (initial reconnaissance)
sudo prtip -T5 -F 192.168.1.0/24
# Expected duration: ~2-5 seconds for 256 hosts × 100 ports
```

**Best Practices:**
- ✅ **Use only on local networks** (same LAN segment)
- ✅ **Initial reconnaissance** followed by slower verification
- ✅ **Host discovery** when you need a quick list
- ✅ **CTF competitions** with strict time constraints
- ⚠️ **Always verify results** with slower scan (T3 or T4)
- ⚠️ **Expect false negatives** (missed open ports)
- ❌ **Never use** on internet targets (useless - too many false negatives)
- ❌ **Never use** on slow networks or wireless
- ❌ **Never rely on results** without verification

**When to Use:**
- **Gigabit LAN** scanning only (wired, same subnet)
- **Initial quick sweep** before comprehensive scan
- **Host discovery** to build target list
- **Time pressure** (CTF, incident response) and accuracy secondary
- You **will verify results** with slower scan

**Performance vs T3 (Normal):** ~10-20x faster (but **much less accurate**)

**Critical Warning:** T5 is **not recommended** for most use cases. The speed gain comes at significant cost to accuracy. On internet targets or slow networks, T5 will **miss most open ports** and produce unreliable results. Use T3 or T4 instead unless you have a **specific reason** to sacrifice accuracy for speed.

---

## Jitter: IDS/IPS Evasion

**What is Jitter?**

**Jitter** is random timing variance applied to probe delays to break predictable patterns that intrusion detection systems (IDS) and intrusion prevention systems (IPS) use for correlation.

**How IDS/IPS Detection Works:**

Modern IDS/IPS systems detect port scans by analyzing **timing patterns**:

1. **Probe Spacing:** Regular intervals between probes (e.g., exactly 100ms apart)
2. **Probe Count:** Rapid probes to many ports on same host
3. **Probe Signature:** TCP SYN packets with no follow-up ACK
4. **Temporal Correlation:** Multiple probes within short time window

**Example Detection Rule (Snort-style):**
```
alert tcp any any -> $HOME_NET any (
    flags: S;
    threshold: type both, track by_src, count 20, seconds 10;
    msg: "Possible port scan detected";
)
```

This rule triggers if **20 or more SYN packets** are sent to different ports on the same host **within 10 seconds**. Regular timing (e.g., 1 probe every 500ms exactly) makes correlation trivial.

**How Jitter Defeats Detection:**

Jitter randomizes probe timing to make correlation harder:

```
Without Jitter (T3):
Probe 1: 0.000s
Probe 2: 0.500s  (exactly 500ms later)
Probe 3: 1.000s  (exactly 500ms later)
Probe 4: 1.500s  (exactly 500ms later)
→ Pattern: 500ms intervals (trivial to detect)

With 30% Jitter (T0):
Probe 1: 0.000s
Probe 2: 0.621s  (621ms, +24% variance)
Probe 3: 1.347s  (726ms delay, +45% variance)
Probe 4: 1.942s  (595ms delay, +19% variance)
→ Pattern: Irregular intervals (harder to correlate)
```

**Jitter Implementation:**

```rust
pub fn apply_jitter(&self, duration: Duration) -> Duration {
    if !self.enable_jitter || self.jitter_factor == 0.0 {
        return duration;  // No jitter
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Jitter range: [duration * (1 - factor), duration * (1 + factor)]
    let millis = duration.as_millis() as f64;
    let min_millis = millis * (1.0 - self.jitter_factor);
    let max_millis = millis * (1.0 + self.jitter_factor);

    let jittered_millis = rng.gen_range(min_millis..max_millis);
    Duration::from_millis(jittered_millis as u64)
}
```

**Jitter by Template:**

| Template | Jitter Enabled | Jitter Factor | Variance | Example (100ms base) |
|----------|----------------|---------------|----------|----------------------|
| T0 Paranoid | ✅ Yes | 0.3 (30%) | ±30% | 70ms - 130ms |
| T1 Sneaky | ✅ Yes | 0.2 (20%) | ±20% | 80ms - 120ms |
| T2 Polite | ✅ Yes | 0.1 (10%) | ±10% | 90ms - 110ms |
| T3 Normal | ❌ No | 0.0 (0%) | None | 100ms (exact) |
| T4 Aggressive | ❌ No | 0.0 (0%) | None | 100ms (exact) |
| T5 Insane | ❌ No | 0.0 (0%) | None | 100ms (exact) |

**Trade-offs:**

**Benefits:**
- ✅ Evades timing-based IDS/IPS correlation
- ✅ Breaks predictable patterns
- ✅ Makes scan harder to fingerprint
- ✅ Reduces likelihood of triggering rate limiting

**Costs:**
- ⚠️ Slightly slower (average delay increases by factor/2)
- ⚠️ Less predictable scan duration
- ⚠️ Minimal CPU overhead (random number generation)

**When Jitter Matters:**

**Use jitter (T0, T1, T2) when:**
- Target has known IDS/IPS (e.g., Snort, Suricata, Zeek)
- Stealth is required (red team, penetration testing)
- Avoiding detection is more important than speed
- Target has rate limiting based on probe frequency

**Skip jitter (T3, T4, T5) when:**
- Internal network with no IDS/IPS
- Speed is critical and detection acceptable
- Scanning your own systems
- Lab/testing environment

**Combining Jitter with Other Techniques:**

For maximum stealth, combine jitter with:
- **Slow timing templates** (T0, T1)
- **Decoy scanning** (`-D` flag): Spoof source IPs
- **Packet fragmentation** (`-f` flag): Split packets
- **Randomized port order** (default): Avoid sequential patterns
- **Source port manipulation** (`-g` flag): Spoof source port

**Example Maximum Stealth:**
```bash
sudo prtip -T0 -D RND:10 -f -g 53 -p 1-1000 target.example.com
# T0: Paranoid timing with 30% jitter
# -D RND:10: 10 random decoy IPs
# -f: Fragment packets
# -g 53: Source port 53 (DNS)
# Expected: Extremely hard to detect, extremely slow
```

---

## RTT Estimation: Adaptive Timeouts

**What is RTT?**

**RTT (Round Trip Time)** is the time elapsed between sending a probe and receiving a response. Accurate RTT estimation allows ProRT-IP to dynamically adjust timeouts based on actual network performance.

**Why RTT Matters:**

**Problem:** Static timeouts are inefficient:
- **Too short:** Miss responses on slow networks (false negatives)
- **Too long:** Waste time on fast networks (slow scans)

**Solution:** Adaptive timeouts based on measured RTT:
- **Fast networks:** Use shorter timeouts (e.g., 50ms for LAN)
- **Slow networks:** Use longer timeouts (e.g., 5s for satellite)
- **Varying networks:** Adjust dynamically as conditions change

**RFC 6298 Algorithm:**

ProRT-IP uses the **RFC 6298** algorithm for calculating timeouts, the same algorithm used by TCP congestion control:

### SRTT (Smoothed Round Trip Time)

**Definition:** Exponentially weighted moving average of RTT measurements.

**Purpose:** Smooth out RTT variations to avoid overreacting to single outliers.

**Formula (initial measurement):**
```
SRTT = RTT_measured
RTTVAR = RTT_measured / 2
```

**Formula (subsequent measurements):**
```
ALPHA = 0.125 (1/8)
SRTT_new = (1 - ALPHA) × SRTT_old + ALPHA × RTT_measured
SRTT_new = 0.875 × SRTT_old + 0.125 × RTT_measured
```

**Example:**
```
Initial RTT: 100ms
SRTT = 100ms

Second RTT: 120ms
SRTT = 0.875 × 100ms + 0.125 × 120ms = 87.5ms + 15ms = 102.5ms

Third RTT: 80ms
SRTT = 0.875 × 102.5ms + 0.125 × 80ms = 89.7ms + 10ms = 99.7ms
```

**Interpretation:** SRTT slowly converges toward average RTT, smoothing out spikes.

### RTTVAR (RTT Variance)

**Definition:** Measure of RTT variation (jitter/instability).

**Purpose:** Account for network instability when calculating timeouts.

**Formula (subsequent measurements):**
```
BETA = 0.25 (1/4)
diff = |RTT_measured - SRTT|
RTTVAR_new = (1 - BETA) × RTTVAR_old + BETA × diff
RTTVAR_new = 0.75 × RTTVAR_old + 0.25 × diff
```

**Example:**
```
SRTT = 100ms, RTTVAR = 50ms

New RTT: 150ms
diff = |150ms - 100ms| = 50ms
RTTVAR = 0.75 × 50ms + 0.25 × 50ms = 37.5ms + 12.5ms = 50ms

New RTT: 80ms
diff = |80ms - 100ms| = 20ms
RTTVAR = 0.75 × 50ms + 0.25 × 20ms = 37.5ms + 5ms = 42.5ms
```

**Interpretation:** RTTVAR increases with RTT instability, decreases with stability.

### RTO (Retransmission Timeout)

**Definition:** Timeout value used for probes.

**Purpose:** Balance between waiting long enough for slow responses and not wasting time on non-responses.

**Formula:**
```
K = 4 (variance multiplier)
G = 10ms (clock granularity)
RTO = SRTT + max(G, K × RTTVAR)
```

**Example:**
```
SRTT = 100ms
RTTVAR = 20ms
K = 4
G = 10ms

RTO = 100ms + max(10ms, 4 × 20ms)
RTO = 100ms + max(10ms, 80ms)
RTO = 100ms + 80ms = 180ms
```

**Interpretation:**
- **Stable network** (low RTTVAR): RTO ≈ SRTT + small buffer
- **Unstable network** (high RTTVAR): RTO = SRTT + large buffer
- **Minimum buffer:** Always at least G (10ms) to account for timer granularity

### Bounded by Template Limits

**Final timeout** is bounded by template's min/max:

```rust
timeout = min(max(RTO, min_timeout), max_timeout)
```

**Example (T3 Normal):**
```
Calculated RTO: 180ms
min_timeout: 500ms
max_timeout: 10s

Final timeout = min(max(180ms, 500ms), 10s)
              = min(500ms, 10s)
              = 500ms
```

**Why bounds matter:**
- **min_timeout:** Prevents too-aggressive timeouts (avoid false negatives)
- **max_timeout:** Prevents excessive waiting (maintain reasonable scan speed)

### RTT-Based Adaptation Example

**Scenario:** Scanning target over VPN with variable latency

```
Probe 1: Response in 150ms
  → SRTT = 150ms, RTTVAR = 75ms
  → RTO = 150ms + max(10ms, 300ms) = 450ms
  → Use max(450ms, 500ms) = 500ms (T3 min_timeout)

Probe 2: Response in 180ms
  → SRTT = 0.875 × 150ms + 0.125 × 180ms = 153.75ms
  → diff = |180ms - 150ms| = 30ms
  → RTTVAR = 0.75 × 75ms + 0.25 × 30ms = 63.75ms
  → RTO = 153.75ms + 255ms = 408.75ms
  → Use 500ms (T3 min_timeout)

Probe 3: Response in 800ms (VPN congestion)
  → SRTT = 0.875 × 153.75ms + 0.125 × 800ms = 234.53ms
  → diff = |800ms - 153.75ms| = 646.25ms
  → RTTVAR = 0.75 × 63.75ms + 0.25 × 646.25ms = 209.38ms
  → RTO = 234.53ms + 837.52ms = 1072ms
  → Use 1072ms (between min and max)

Probe 4: Response in 200ms (VPN recovers)
  → SRTT = 0.875 × 234.53ms + 0.125 × 200ms = 230.21ms
  → diff = |200ms - 234.53ms| = 34.53ms
  → RTTVAR = 0.75 × 209.38ms + 0.25 × 34.53ms = 165.67ms
  → RTO = 230.21ms + 662.68ms = 892.89ms
  → Use 892.89ms
```

**Outcome:** Timeout automatically adjusts to network conditions without manual intervention.

---

## AIMD Congestion Control

**What is AIMD?**

**AIMD (Additive Increase, Multiplicative Decrease)** is a congestion control algorithm that dynamically adjusts scan rate based on network feedback. It's the same algorithm used by TCP for congestion control.

**Purpose:** Prevent network congestion and packet loss by adapting scan rate to network capacity.

**How It Works:**

### Additive Increase (Success)

**Rule:** When probes succeed, **gradually increase** scan rate.

**Implementation:**
```rust
// Increase rate by 1% every 100ms when successful
let increase = current_rate × 0.01;
new_rate = min(current_rate + increase, max_rate);
```

**Example:**
```
Initial rate: 1,000 pps
After 100ms success: 1,000 × 1.01 = 1,010 pps
After 200ms success: 1,010 × 1.01 = 1,020 pps
After 300ms success: 1,020 × 1.01 = 1,030 pps
...
After 10 seconds: ~1,105 pps (10.5% increase)
```

**Why additive?**
- **Conservative growth:** Prevents sudden rate spikes
- **Stable convergence:** Approaches network capacity gradually
- **Predictable behavior:** Linear increase over time

### Multiplicative Decrease (Failure)

**Rule:** When timeouts occur, **aggressively decrease** scan rate.

**Implementation:**
```rust
// After 3 consecutive timeouts, cut rate in half
if consecutive_timeouts >= 3 {
    new_rate = max(current_rate × 0.5, min_rate);
    consecutive_timeouts = 0;
}
```

**Example:**
```
Current rate: 2,000 pps
Timeout 1: (continue)
Timeout 2: (continue)
Timeout 3: 2,000 × 0.5 = 1,000 pps (cut in half)

If still timing out:
Timeout 4: (continue)
Timeout 5: (continue)
Timeout 6: 1,000 × 0.5 = 500 pps (cut in half again)
```

**Why multiplicative?**
- **Fast response:** Quickly backs off when congestion detected
- **Prevent collapse:** Avoids overwhelming network further
- **Safety:** Ensures scan doesn't cause network issues

### AIMD in Action

**Scenario:** Scanning network with variable load

```
Time    Rate (pps)  Event                        Action
------  ----------  ---------------------------  -------------------
0.0s    1,000       Start scanning               (initial rate)
0.1s    1,010       Responses received           +1% (additive)
0.2s    1,020       Responses received           +1% (additive)
0.3s    1,030       Responses received           +1% (additive)
...
5.0s    1,500       Responses received           +1% (additive)
5.1s    1,515       Timeout (network congestion) (count: 1)
5.2s    1,530       Timeout                      (count: 2)
5.3s    1,545       Timeout (3rd consecutive)    ×0.5 (multiplicative)
5.3s    772         Backed off to half rate      (reset count)
5.4s    780         Responses resume             +1% (additive)
5.5s    788         Responses received           +1% (additive)
...
10.0s   950         Stable rate                  (settled)
```

**Interpretation:**
- **0-5s:** Rate increases gradually (1,000 → 1,545 pps)
- **5.3s:** Congestion detected (3 timeouts) → cut rate in half
- **5.4s+:** Recovery begins, rate increases again
- **10s:** Stable rate found (~950 pps, network capacity)

### Rate Limiting by Template

**Templates with AIMD enabled:**

| Template | AIMD | Initial Rate | Min Rate | Max Rate | Behavior |
|----------|------|--------------|----------|----------|----------|
| T0 Paranoid | No | 0.003 pps | N/A | N/A | Fixed (too slow) |
| T1 Sneaky | No | 0.67 pps | N/A | N/A | Fixed (too slow) |
| T2 Polite | Yes | 250 pps | 10 pps | 500 pps | Adaptive |
| T3 Normal | Yes | 1,000 pps | 100 pps | 10,000 pps | Adaptive |
| T4 Aggressive | Yes | 5,000 pps | 500 pps | 50,000 pps | Adaptive |
| T5 Insane | Yes | 10,000 pps | 1,000 pps | 100,000 pps | Adaptive |

**Why no AIMD for T0/T1?**
- Scan rate too low for meaningful adaptation (< 1 pps)
- Fixed delays provide predictable stealth behavior
- Network congestion unlikely at these rates

### Thread-Safe Implementation

**Challenge:** AIMD must work correctly with parallel scanners.

**Solution:** Atomic operations for lock-free updates.

```rust
pub struct AdaptiveRateLimiter {
    /// Current rate in millihertz (mHz = packets/sec × 1000)
    /// Stored as mHz to allow atomic u64 storage
    current_rate_mhz: AtomicU64,

    /// Number of consecutive timeouts
    consecutive_timeouts: AtomicUsize,

    /// Number of successful responses
    successful_responses: AtomicUsize,
}

impl AdaptiveRateLimiter {
    pub fn report_response(&self, success: bool, rtt: Duration) {
        if success {
            // Additive increase (atomic compare-exchange loop)
            loop {
                let current_mhz = self.current_rate_mhz.load(Ordering::Relaxed);
                let increase_mhz = (current_mhz as f64 * 0.01) as u64;
                let new_mhz = (current_mhz + increase_mhz).min(self.max_rate_mhz);

                if self.current_rate_mhz
                    .compare_exchange_weak(
                        current_mhz,
                        new_mhz,
                        Ordering::Release,
                        Ordering::Relaxed
                    )
                    .is_ok()
                {
                    break;  // Successfully updated
                }
                // Retry if another thread modified rate concurrently
            }

            // Reset timeout counter
            self.consecutive_timeouts.store(0, Ordering::Release);
        } else {
            // Multiplicative decrease (after 3 timeouts)
            let timeouts = self.consecutive_timeouts.fetch_add(1, Ordering::AcqRel) + 1;

            if timeouts >= 3 {
                loop {
                    let current_mhz = self.current_rate_mhz.load(Ordering::Relaxed);
                    let new_mhz = ((current_mhz as f64 * 0.5) as u64)
                        .max(self.min_rate_mhz);

                    if self.current_rate_mhz
                        .compare_exchange_weak(
                            current_mhz,
                            new_mhz,
                            Ordering::Release,
                            Ordering::Relaxed
                        )
                        .is_ok()
                    {
                        break;  // Successfully updated
                    }
                }

                // Reset timeout counter
                self.consecutive_timeouts.store(0, Ordering::Release);
            }
        }
    }
}
```

**Key Points:**
- **Atomic operations:** Lock-free updates (no mutexes)
- **Compare-exchange loop:** Handle concurrent updates safely
- **Millihertz storage:** Allow fractional rates in u64 (1.5 pps = 1,500 mHz)
- **Ordering semantics:** Release/Acquire ensures memory consistency

### Benefits of AIMD

**Network Protection:**
- ✅ Prevents overwhelming target network
- ✅ Avoids triggering rate limiting
- ✅ Reduces packet loss from congestion
- ✅ Maintains scan reliability

**Performance:**
- ✅ Automatically finds optimal rate
- ✅ Adapts to changing network conditions
- ✅ Maximizes throughput without manual tuning
- ✅ Recovers from temporary congestion

**Monitoring AIMD:**

```bash
# Verbose output shows rate adjustments
sudo prtip -T3 -v -p 1-10000 target.example.com

# Example verbose output:
# [2025-01-15 10:30:00] Starting scan at 1,000 pps
# [2025-01-15 10:30:01] Rate increased to 1,105 pps (10.5% growth)
# [2025-01-15 10:30:02] Rate increased to 1,220 pps (20.5% growth)
# [2025-01-15 10:30:03] Timeout detected (1/3)
# [2025-01-15 10:30:03] Timeout detected (2/3)
# [2025-01-15 10:30:03] Timeout detected (3/3), reducing to 610 pps
# [2025-01-15 10:30:04] Rate increased to 616 pps
# ...
```

---

## Performance Comparison

**Benchmark Setup:**
- **Target:** localhost (127.0.0.1)
- **Ports:** 22, 80, 443
- **System:** Linux x86_64, 16 GB RAM

**Results:**

| Template | Duration | Relative Speed | Open Ports Found | Accuracy |
|----------|----------|----------------|------------------|----------|
| T0 Paranoid | 15m 30s | 1.0x (baseline) | 3/3 | 100% |
| T1 Sneaky | 48.2s | 19.3x faster | 3/3 | 100% |
| T2 Polite | 3.8s | 244.7x faster | 3/3 | 100% |
| **T3 Normal** | **1.2s** | **775.0x faster** | **3/3** | **100%** |
| T4 Aggressive | 0.3s | 3,100.0x faster | 3/3 | 100% |
| T5 Insane | 0.1s | 9,300.0x faster | 2/3 | 67% ⚠️ |

**Key Findings:**

1. **T5 missed 1 port** (false negative) due to aggressive timeout
2. **T0-T4 achieved 100% accuracy** on all ports
3. **T3 provides excellent balance** (1.2s, 100% accuracy)
4. **T4 is 2.6x faster than T3** with same accuracy (local network)
5. **T0 is impractically slow** for even 3 ports (15+ minutes)

**Performance vs Accuracy Trade-off:**

```
Stealth/Accuracy                                  Speed
       ▲                                            ▲
  100% │ T0 ────── T1 ─── T2 ── T3 ─ T4           │
       │                              \            │
   80% │                               \           │
       │                                T5         │
   60% │                                           │
       └───────────────────────────────────────────┘
          Slowest                         Fastest
```

**Recommendations by Scenario:**

| Scenario | Template | Rationale |
|----------|----------|-----------|
| Internet target, unknown network | T3 Normal | Balanced, reliable |
| Local network (LAN) | T4 Aggressive | Fast, minimal loss |
| Production environment | T2 Polite | Courteous, safe |
| IDS/IPS present | T1 Sneaky | Stealth, acceptable speed |
| Maximum stealth required | T0 Paranoid | Maximum evasion |
| Quick host discovery | T5 Insane → T3 | Fast initial + verify |
| Large subnet (>/24) | T3 Normal | Balanced for scale |
| Satellite/high-latency link | T2 Polite | Tolerates delay |
| CTF competition | T4 or T5 | Speed critical |
| Security audit (customer) | T2 Polite | Professional courtesy |

---

## Use Case Guide

### Scenario 1: Internal Network Assessment

**Context:** Assessing internal corporate network (trusted environment, no stealth requirement).

**Recommended Template:** T3 Normal or T4 Aggressive

**Rationale:**
- No IDS/IPS to evade
- Speed matters for large IP ranges
- Accuracy important for complete inventory
- Network bandwidth available

**Command:**
```bash
sudo prtip -T3 -p 1-10000 10.0.0.0/8 -oJ internal-scan.json
# Or for faster scanning:
sudo prtip -T4 -p 1-10000 10.0.0.0/8 -oJ internal-scan.json
```

---

### Scenario 2: External Penetration Test

**Context:** Authorized penetration test against client's internet-facing infrastructure.

**Recommended Template:** T2 Polite

**Rationale:**
- Client relationship requires courtesy
- May have IDS/IPS monitoring
- Production systems must not be disrupted
- Compliance requirements

**Command:**
```bash
sudo prtip -T2 -sV -O -p 1-65535 client-target.com -oA pentest-results
```

---

### Scenario 3: Red Team Engagement

**Context:** Adversary simulation with strict stealth requirements (must avoid detection).

**Recommended Template:** T0 Paranoid or T1 Sneaky

**Rationale:**
- Detection = mission failure
- Time is secondary to stealth
- Advanced IDS/IPS likely present
- Rules of engagement require stealth

**Command:**
```bash
# Maximum stealth (very slow)
sudo prtip -T0 -D RND:10 -f -g 53 -p 80,443,8080 target.example.com

# Stealth with reasonable speed
sudo prtip -T1 -D RND:5 -f -p 1-1000 target.example.com
```

---

### Scenario 4: Quick Host Discovery

**Context:** Building target list for subsequent detailed scanning.

**Recommended Template:** T5 Insane (initial) → T3 Normal (verification)

**Rationale:**
- Speed critical for initial survey
- False negatives acceptable (will verify)
- Two-phase approach: fast discovery + accurate confirmation

**Command:**
```bash
# Phase 1: Quick discovery
sudo prtip -T5 -sn 192.168.0.0/16 -oN live-hosts-quick.txt

# Phase 2: Verify discovered hosts
sudo prtip -T3 -p 1-1000 -iL live-hosts-quick.txt -oA verified-scan
```

---

### Scenario 5: Production Environment Audit

**Context:** Security audit during business hours on live production systems.

**Recommended Template:** T2 Polite

**Rationale:**
- Cannot disrupt services
- Must respect rate limits
- Professional courtesy required
- Compliance documentation

**Command:**
```bash
sudo prtip -T2 -sV -p 80,443,22,3389 prod-servers.txt -oX compliance-report.xml
```

---

### Scenario 6: High-Latency Network

**Context:** Scanning over satellite link, VPN, or high-latency internet connection (300+ ms RTT).

**Recommended Template:** T2 Polite or T3 Normal (with custom max timeout)

**Rationale:**
- High RTT requires longer timeouts
- T4/T5 will produce false negatives
- Jitter not needed (latency provides natural variance)

**Command:**
```bash
# T2 with extended max timeout
sudo prtip -T2 --max-rtt 5000 -p 1-1000 satellite-target.example.com

# Or T3 with custom settings
sudo prtip -T3 --max-rtt 5000 --max-retries 5 -p 1-1000 vpn-target.internal
```

---

### Scenario 7: CTF Competition

**Context:** Capture-the-flag competition with strict time limit (e.g., 30 minutes).

**Recommended Template:** T4 Aggressive or T5 Insane

**Rationale:**
- Speed is paramount
- Detection doesn't matter (controlled environment)
- Can verify results manually if needed
- Time pressure

**Command:**
```bash
# Fastest possible scan
sudo prtip -T5 -p- ctf-target.local -oN quick-scan.txt

# If T5 produces false negatives, verify with T4
sudo prtip -T4 -p 1-10000 ctf-target.local -oN detailed-scan.txt
```

---

### Scenario 8: Wireless Network

**Context:** Scanning over WiFi or other wireless medium (unstable, variable latency).

**Recommended Template:** T2 Polite

**Rationale:**
- High packet loss on wireless
- Variable latency requires adaptive timeouts
- Aggressive scanning makes loss worse
- Jitter helps with interference

**Command:**
```bash
sudo prtip -T2 --max-retries 5 -p 1-5000 wireless-target.local
```

---

### Scenario 9: Large-Scale Internet Scan

**Context:** Scanning large IP ranges on the internet (e.g., /8 network, millions of IPs).

**Recommended Template:** T3 Normal

**Rationale:**
- T4/T5 produce too many false negatives on internet
- T2 too slow for massive scale
- T3 provides best balance
- Internet targets highly variable

**Command:**
```bash
# Scan common ports across large range
sudo prtip -T3 -p 80,443,22,21,25 8.0.0.0/8 --stream-to-disk results.db

# With adaptive rate limiting to avoid overwhelming network
sudo prtip -T3 --max-rate 10000 -p 1-1000 large-subnet.txt
```

---

### Scenario 10: Database Server Audit

**Context:** Auditing database servers for open ports (security assessment).

**Recommended Template:** T2 Polite

**Rationale:**
- Database servers sensitive to load
- Cannot risk disrupting queries
- Courtesy required
- Typically behind rate limiting

**Command:**
```bash
sudo prtip -T2 -p 3306,5432,1433,27017,6379 -sV db-servers.txt -oJ db-audit.json
```

---

## Custom Timing Parameters

**Beyond Templates:** For advanced users, ProRT-IP allows **manual override** of individual timing parameters.

**Use Cases:**
- Fine-tuning for specific network characteristics
- Balancing between two template levels
- Debugging timing issues
- Specialized scanning scenarios

**Available Flags:**

### `--min-rtt <MS>`
**Override minimum timeout.**

**Default:** Template-specific (50ms to 100s)

**Example:**
```bash
# Never timeout faster than 1 second (avoid false negatives on slow network)
sudo prtip -T3 --min-rtt 1000 -p 1-5000 slow-target.example.com
```

### `--max-rtt <MS>`
**Override maximum timeout.**

**Default:** Template-specific (300ms to 300s)

**Example:**
```bash
# Cap timeout at 2 seconds (avoid wasting time on unresponsive ports)
sudo prtip -T2 --max-rtt 2000 -p 1-65535 target.example.com
```

### `--initial-rtt <MS>`
**Override initial timeout (before RTT estimation).**

**Default:** Template-specific (250ms to 300s)

**Example:**
```bash
# Start with 500ms timeout, then adapt based on RTT
sudo prtip -T3 --initial-rtt 500 -p 1-10000 target.example.com
```

### `--max-retries <N>`
**Override maximum number of retries.**

**Default:** Template-specific (2 to 6)

**Example:**
```bash
# More retries for unreliable network (satellite, packet loss)
sudo prtip -T3 --max-retries 10 -p 1-5000 unreliable-target.com
```

### `--scan-delay <MS>`
**Override delay between probes to same target.**

**Default:** Template-specific (0ms to 300s)

**Example:**
```bash
# Add 100ms delay to avoid triggering rate limiting
sudo prtip -T3 --scan-delay 100 -p 1-10000 rate-limited.example.com
```

### `--max-rate <PPS>`
**Override maximum scan rate (packets per second).**

**Default:** Template-specific (derived from parallelism)

**Example:**
```bash
# Limit to 1,000 pps to avoid overwhelming network
sudo prtip -T4 --max-rate 1000 -p 1-65535 target.example.com
```

### `--min-rate <PPS>`
**Override minimum scan rate (packets per second).**

**Default:** Template-specific (derived from parallelism)

**Example:**
```bash
# Ensure at least 100 pps (avoid scan stalling)
sudo prtip -T3 --min-rate 100 -p 1-10000 target.example.com
```

### `--min-parallelism <N>`
**Override minimum parallel probes.**

**Default:** 1

**Example:**
```bash
# Force at least 10 parallel probes (even if AIMD backs off)
sudo prtip -T3 --min-parallelism 10 -p 1-5000 target.example.com
```

### `--max-parallelism <N>`
**Override maximum parallel probes.**

**Default:** Template-specific (1 to 10,000)

**Example:**
```bash
# Limit to 100 parallel probes (avoid overwhelming system)
sudo prtip -T4 --max-parallelism 100 -p 1-65535 target.example.com
```

**Combining Custom Parameters:**

```bash
# Custom timing profile: Fast but reliable
sudo prtip \
  --initial-rtt 500 \
  --min-rtt 200 \
  --max-rtt 3000 \
  --max-retries 4 \
  --scan-delay 50 \
  --max-parallelism 2000 \
  -p 1-10000 target.example.com

# Equivalent to: Between T3 and T4, with extra retries
```

**When to Use Custom Parameters:**

**✅ Use custom parameters when:**
- Network characteristics don't match any template
- Fine-tuning for specific target behavior
- Debugging timing-related issues
- Specialized scanning requirements

**❌ Avoid custom parameters when:**
- Standard template works well
- Unsure of impact (can make scan worse)
- No specific requirement (templates are well-tuned)

**Example: High-Latency VPN**

```bash
# Problem: T3 too aggressive (packet loss), T2 too slow
# Solution: Custom timing between T2 and T3

sudo prtip \
  -T3 \                      # Start with T3 base
  --min-rtt 1000 \           # 1s minimum (VPN latency)
  --max-rtt 5000 \           # 5s maximum (allow retries)
  --max-retries 5 \          # More retries (packet loss)
  --max-parallelism 500 \    # Reduce parallelism (avoid congestion)
  -p 1-5000 vpn-target.internal
```

---

## Best Practices

### 1. Start Conservative, Speed Up

**Guideline:** Always start with a slower template and increase speed if safe.

**Rationale:**
- Slower templates more reliable (fewer false negatives)
- Faster templates may miss results or trigger defenses
- Can always re-scan faster if initial scan successful

**Workflow:**
```bash
# Step 1: Try T2 (safe default)
sudo prtip -T2 -p 1-1000 unknown-target.com -oN scan-t2.txt

# Step 2: If successful and no issues, try T3
sudo prtip -T3 -p 1-1000 unknown-target.com -oN scan-t3.txt

# Step 3: If still good, try T4 (local network only)
sudo prtip -T4 -p 1-1000 192.168.1.1 -oN scan-t4.txt
```

---

### 2. Match Template to Network

**Guideline:** Choose template based on network type and characteristics.

**Network Type Guide:**

| Network Type | RTT | Packet Loss | Recommended Template |
|--------------|-----|-------------|---------------------|
| Same LAN | <1ms | <0.1% | T4 Aggressive |
| Local campus | 1-10ms | <0.5% | T3 Normal |
| Regional internet | 10-50ms | <1% | T3 Normal |
| National internet | 50-100ms | 1-3% | T2 Polite |
| International internet | 100-300ms | 3-5% | T2 Polite |
| Satellite/VPN | 300-1000ms | 5-10% | T2 Polite + custom |
| Wireless (WiFi) | 5-50ms | 1-10% | T2 Polite |

---

### 3. Consider Stealth Requirements

**Guideline:** Use slower templates with jitter when stealth matters.

**Stealth Level Guide:**

| Stealth Requirement | Template | Additional Measures |
|---------------------|----------|---------------------|
| None (internal scan) | T3 Normal | - |
| Low (authorized test) | T2 Polite | - |
| Medium (avoid alerts) | T1 Sneaky | + Decoy scanning (-D) |
| High (red team) | T0 Paranoid | + Decoys + Fragmentation (-f) |
| Maximum (advanced adversary) | T0 Paranoid | + All evasion techniques |

---

### 4. Verify Fast Scans

**Guideline:** Always verify results from T5 (Insane) with slower scan.

**Two-Phase Approach:**
```bash
# Phase 1: Fast discovery (T5)
sudo prtip -T5 -p- 192.168.1.1 -oN quick-scan.txt
# Found: 5 open ports (may have false negatives)

# Phase 2: Verify with T3
sudo prtip -T3 -p- 192.168.1.1 -oN verify-scan.txt
# Found: 7 open ports (2 were missed by T5)
```

---

### 5. Monitor Scan Progress

**Guideline:** Use verbose output to monitor timing behavior and rate adjustments.

**Command:**
```bash
sudo prtip -T3 -v -p 1-10000 target.example.com
```

**Example Verbose Output:**
```
[2025-01-15 10:30:00] Starting T3 (Normal) scan
[2025-01-15 10:30:00] Initial rate: 1,000 pps, parallelism: 1,000
[2025-01-15 10:30:01] Scanned 1,000 ports (10.0%), 15 open
[2025-01-15 10:30:01] AIMD: Rate increased to 1,105 pps (+10.5%)
[2025-01-15 10:30:02] Scanned 2,200 ports (22.0%), 32 open
[2025-01-15 10:30:02] AIMD: Rate increased to 1,220 pps (+22.0%)
[2025-01-15 10:30:03] Timeout detected (1/3)
[2025-01-15 10:30:03] Timeout detected (2/3)
[2025-01-15 10:30:03] Timeout detected (3/3)
[2025-01-15 10:30:03] AIMD: Rate decreased to 610 pps (-50.0%)
[2025-01-15 10:30:05] Scanned 5,000 ports (50.0%), 78 open
...
```

**What to Watch:**
- **Rate adjustments:** Frequent decreases indicate network congestion
- **Timeout patterns:** Spikes suggest target rate limiting
- **RTT increases:** Growing RTT indicates network saturation
- **Completion rate:** Slower than expected suggests template too aggressive

---

### 6. Adjust Based on Feedback

**Guideline:** If scan produces unexpected results, adjust template.

**Common Issues and Solutions:**

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Many timeouts | Template too aggressive | Use slower template (T4→T3, T3→T2) |
| Very slow progress | Template too conservative | Use faster template (T2→T3, T3→T4) |
| High packet loss | Network congestion | Reduce parallelism or use T2 |
| Inconsistent results | Timeouts too short | Increase `--min-rtt` or `--max-retries` |
| Rate limiting errors | Scan too fast | Add `--scan-delay` or use T2 |
| IDS alerts triggered | Scan too obvious | Use T1 or T0 with evasion |

**Example Adjustment:**
```bash
# Initial scan: T3 produces many timeouts
sudo prtip -T3 -p 1-10000 target.com
# Result: 15% timeout rate (too high)

# Adjusted scan: Switch to T2
sudo prtip -T2 -p 1-10000 target.com
# Result: 2% timeout rate (acceptable)
```

---

### 7. Document Timing Choices

**Guideline:** Record template choice and rationale in scan logs.

**Example Documentation:**
```bash
# Scan log header
echo "Scan Date: $(date)" >> scan-log.txt
echo "Template: T2 (Polite)" >> scan-log.txt
echo "Rationale: Production environment, customer network, business hours" >> scan-log.txt
echo "Target: customer-production.example.com" >> scan-log.txt
echo "" >> scan-log.txt

# Run scan
sudo prtip -T2 -sV -p 1-10000 customer-production.example.com -oA customer-scan
```

**Why Documentation Matters:**
- Reproducibility (re-run scan with same settings)
- Audit trail (compliance requirements)
- Knowledge sharing (team members understand choices)
- Troubleshooting (understand what was tried)

---

### 8. Test Before Production

**Guideline:** Test timing template on non-production systems first.

**Safe Testing Workflow:**
```bash
# Test 1: Local loopback (baseline)
sudo prtip -T4 -p 1-10000 127.0.0.1
# Verify: Fast, 100% accuracy

# Test 2: Internal test system (same network)
sudo prtip -T4 -p 1-10000 test-server.internal
# Verify: Performance acceptable, no issues

# Test 3: Production system (if tests pass)
sudo prtip -T2 -p 1-10000 production-server.internal
# Note: Use T2 for production (courtesy)
```

---

## See Also

**Related Documentation:**

- **[Command Reference](./command-reference.md)** - Complete CLI flag reference
  - Section: Timing and Performance Flags (`-T`, `--min-rtt`, `--max-rtt`, etc.)

- **[Performance Guide](../advanced/performance.md)** - Performance tuning and optimization
  - Section: Scan Rate Optimization
  - Section: Network Bottleneck Analysis
  - Section: Benchmarking Methodology

- **[Stealth Scanning](../features/stealth-scanning.md)** - IDS/IPS evasion techniques
  - Section: Timing-Based Evasion (jitter, delays)
  - Section: Combining Evasion Techniques
  - Section: Advanced IDS Detection Avoidance

- **[Network Protocols](./network-protocols.md)** - TCP/IP protocol details
  - Section: TCP Congestion Control (AIMD algorithm)
  - Section: RTT Estimation (RFC 6298)

- **[Basic Usage](../user-guide/basic-usage.md)** - Getting started with scanning
  - Section: Timing Template Selection
  - Section: Scan Speed Optimization

**External Resources:**

- **RFC 6298:** Computing TCP's Retransmission Timer (RTT estimation)
- **RFC 5681:** TCP Congestion Control (AIMD algorithm)
- **Nmap Timing Documentation:** Original timing template reference
- **TCP/IP Illustrated Vol. 1:** Detailed TCP congestion control explanation

---

**Last Updated:** 2025-01-15
**ProRT-IP Version:** v0.5.2
