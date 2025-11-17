# Rate Limiting

ProRT-IP implements industry-leading rate limiting with **breakthrough -1.8% average overhead** - faster than no rate limiting through system-wide optimization.

## Overview

Rate limiting controls scan speed to:
- Prevent network saturation
- Avoid IDS/IPS detection
- Respect target rate limits
- Optimize CPU utilization

**Key Achievement:** AdaptiveRateLimiterV3 achieves negative overhead (faster than uncontrolled scanning) through CPU optimization and predictable memory access patterns.

## Adaptive Rate Control

### Two-Tier Architecture

1. **Hostgroup-Level**: Aggregate rate across all targets
2. **Per-Target**: Individual rate control with batch scheduling
3. **Convergence**: Automatic adjustment using `batch *= (target/observed)^0.5`
4. **Range**: 1.0 → 10,000.0 packets per batch

### Performance Characteristics

| Rate (pps) | Overhead | Use Case |
|------------|----------|----------|
| 10K        | -8.2%    | Best case (low rate) |
| 50K        | -1.8%    | Typical scanning |
| 75K-200K   | -3% to -4% | Sweet spot (optimal) |
| 500K-1M    | +0% to +3% | Near-zero (extreme rates) |

**Average:** -1.8% overhead across typical usage patterns

### Why Negative Overhead?

Controlled rate limiting enables:
- Better CPU speculation and pipelining
- Reduced memory contention from batch scheduling
- Improved L1/L2 cache efficiency
- More consistent timing for hardware optimization

## Configuration

### Basic Usage

```bash
# Automatic rate limiting (recommended)
prtip -sS -p 80,443 --max-rate 100000 192.168.1.0/24

# Sweet spot for optimal performance (75K-200K pps)
prtip -sS -p 1-10000 --max-rate 150000 10.0.0.0/16

# Extreme high-speed scanning
prtip -sS -p- --max-rate 500000 10.0.0.0/8
```

### Timing Templates

Rate limits are included in timing templates:

```bash
-T0  # Paranoid:   100 pps
-T1  # Sneaky:     500 pps
-T2  # Polite:    2000 pps
-T3  # Normal:   10000 pps (default)
-T4  # Aggressive: 50000 pps
-T5  # Insane:   100000 pps
```

**Example:**

```bash
prtip -T4 -p- 192.168.1.0/24
# Equivalent to: --max-rate 50000
```

### Hostgroup Limiting

Control concurrent targets for network-friendly scanning:

```bash
# Limit to 16 concurrent hosts
prtip -sS -p- --max-hostgroup 16 10.0.0.0/24

# Aggressive scanning (128 hosts)
prtip -sS -p 80,443 --max-hostgroup 128 targets.txt

# With minimum parallelism
prtip -sS -p 1-1000 --min-hostgroup 8 --max-hostgroup 64 10.0.0.0/16
```

**Hostgroup Guidelines:**

| Value | Impact | Use Case |
|-------|--------|----------|
| 1-16 | Minimal network load | Sensitive environments |
| 32-128 | Balanced performance | General-purpose scanning |
| 256-1024 | Maximum speed | Internal networks, authorized tests |

**Performance:** 1-9% overhead (excellent concurrency control)

### Combined Rate Limiting

Stack both layers for maximum control:

```bash
# Full rate limiting: V3 (50K pps) + Hostgroup (32 hosts)
prtip -sS -p- \
  --max-rate 50000 \
  --max-hostgroup 32 \
  --min-hostgroup 8 \
  10.0.0.0/16
```

## ICMP Monitoring (Optional)

Automatically detects and responds to ICMP rate limiting errors.

### Activation

```bash
# Enable ICMP monitoring for adaptive backoff
prtip -sS -p 1-1000 --adaptive-rate 192.168.1.0/24
```

### How It Works

1. Background task listens for ICMP Type 3 Code 13 (Communication Administratively Prohibited)
2. Applies per-target exponential backoff
3. Scanner waits for backoff expiration before resuming

**Backoff Levels:**
- Level 0: No backoff (initial state)
- Level 1: 2 seconds
- Level 2: 4 seconds
- Level 3: 8 seconds
- Level 4: 16 seconds (maximum)

**Platform Support:**
- **Linux/macOS**: Full support
- **Windows**: Graceful degradation (monitoring inactive)

**Performance:** 4-6% overhead (acceptable for adaptive backoff)

## Configuration File

Set default rate limits in configuration:

```toml
[timing]
template = "normal"      # T3
min_rate = 10           # Minimum packets/sec
max_rate = 1000         # Maximum packets/sec

[performance]
batch_size = 1000       # Batch size for parallelism
```

**Location:** `~/.config/prtip/config.toml`

## Performance Impact

### Benchmark Results

Based on comprehensive testing with hyperfine 1.19.0:

**AdaptiveRateLimiterV3 (Default):**
- Best case: -8.2% overhead at 10K pps
- Typical: -1.8% overhead at 50K pps
- Sweet spot: -3% to -4% overhead at 75K-200K pps
- Extreme rates: +0% to +3% overhead at 500K-1M pps

**Hostgroup Limiter:**
- Small scans: +1% overhead (18 ports)
- Large scans: 1-9% overhead
- Sometimes faster than baseline

**ICMP Monitor:**
- +4-6% overhead with adaptive backoff
- Combined (V3 + ICMP + Hostgroup): ~6% total overhead

### Variance Reduction

V3 achieves 34% reduction in timing variance compared to previous implementations, providing more consistent and predictable scan performance.

## Best Practices

### Always Use Rate Limiting

With V3's negative overhead, **always enable rate limiting** for optimal performance:

```bash
# Recommended: explicit rate limit
prtip -sS -p- --max-rate 100000 target.com

# Also good: timing template
prtip -T4 -p 1-10000 192.168.1.0/24
```

The old tradeoff ("fast but uncontrolled" vs "slow but controlled") no longer applies - rate-limited scans are now faster.

### Network-Friendly Scanning

For sensitive environments:

```bash
# Polite scanning with hostgroup limits
prtip -sS -p- --max-rate 50000 --max-hostgroup 32 10.0.0.0/24

# With ICMP monitoring for automatic backoff
prtip -sS -p 1-1000 --adaptive-rate 192.168.1.0/24
```

### High-Performance Scanning

For maximum speed on capable networks:

```bash
# Sweet spot (75K-200K pps, -3% to -4% overhead)
prtip -sS -p 1-10000 --max-rate 150000 10.0.0.0/16

# Extreme high-speed (near-zero overhead)
prtip -sS -p- --max-rate 500000 10.0.0.0/8
```

## Nmap Compatibility

ProRT-IP supports standard Nmap rate limiting flags:

| Flag | Description | Compatibility |
|------|-------------|---------------|
| `--max-rate <N>` | Maximum packets per second | ✅ Enhanced (V3 algorithm) |
| `--min-rate <N>` | Minimum packets per second | ✅ 100% compatible |
| `--max-hostgroup <N>` | Maximum concurrent targets | ✅ 100% compatible |
| `--min-hostgroup <N>` | Minimum concurrent targets | ✅ 100% compatible |
| `--max-parallelism <N>` | Alias for max-hostgroup | ✅ 100% compatible |

**ProRT-IP Exclusive:**
- **ICMP backoff** (`--adaptive-rate`) - Automatic IDS/IPS avoidance
- **Negative overhead** - Faster than Nmap's rate limiting

## Troubleshooting

### Slow Convergence

**Problem:** Rate limiter not reaching target rate quickly

**Solutions:**
```bash
# Increase max rate
prtip -sS -p- --max-rate 200000 target.com

# Check network bandwidth
iftop -i eth0

# Verify no external rate limiting
tcpdump -i eth0 icmp
```

### ICMP Monitor Issues

**Error:** "ICMP monitor already running"

**Fix:** Restart application (only one monitor per process allowed)

---

**Error:** "No targets scanned (all backed off)"

**Fix:** Targets have strict rate limiting. Disable adaptive monitoring or reduce rate:
```bash
# Option 1: Disable ICMP monitoring
prtip -sS -p- --max-rate 10000 target.com

# Option 2: Reduce rate
prtip -sS -p- --max-rate 5000 --adaptive-rate target.com
```

### Hostgroup Warnings

**Warning:** "Active targets below min_hostgroup"

**Cause:** Not enough targets or slow scan progress

**Fix:** Increase target count or reduce minimum:
```bash
# Reduce minimum hostgroup
prtip -sS -p- --min-hostgroup 4 --max-hostgroup 32 targets.txt
```

## See Also

- [Timing & Performance](../advanced/performance-tuning.md) - Performance optimization guide
- [Nmap Compatibility](./nmap-compatibility.md) - Flag compatibility reference
- [CLI Reference](../user-guide/cli-reference.md#rate-limiting) - Complete flag documentation
- [Benchmarking](../31-BENCHMARKING-GUIDE.md) - Performance validation
