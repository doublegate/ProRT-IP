# Sprint 6.3: Network Optimization Benchmarks

**Version:** 1.0.0
**Created:** 2025-11-15
**Sprint:** 6.3 (Network Optimizations)

## Overview

This directory contains benchmark scenarios for Sprint 6.3 Network Optimization features:

1. **CDN IP Deduplication** - Validates CDN filtering effectiveness and performance
2. **Batch I/O Performance** - Measures sendmmsg/recvmmsg throughput improvements (planned)
3. **Adaptive Batch Sizing** - Tests dynamic batch size adjustments (planned)

## Benchmark Files

### 01-CDN-Deduplication-Bench.json

Comprehensive CDN filtering validation with 6 scenarios:

| Scenario | Description | Expected Reduction | Overhead Limit |
|----------|-------------|-------------------|----------------|
| **Baseline** | No CDN filtering (reference) | 0% | 0% |
| **Default Mode** | Skip all detected CDNs | ≥45% | <10% |
| **Whitelist Mode** | Skip CF+AWS only | ≥18% | <10% |
| **Blacklist Mode** | Skip all except CF | ≥35% | <10% |
| **IPv6 Filtering** | IPv6 CDN detection | ≥45% | <10% |
| **Mixed IPv4/IPv6** | Dual-stack filtering | ≥45% | <10% |

## Execution Instructions

### Prerequisites

```bash
# Build release binary
cargo build --release

# Requires root for raw sockets
sudo -v
```

### Running CDN Benchmarks

**Scenario 1: Baseline (No Filtering)**

```bash
# 1000 IPs, no CDN filtering
sudo ./target/release/prtip -sS -p 80,443 \
  --target-file benchmarks/04-Sprint6.3-Network-Optimization/targets/baseline-1000.txt \
  --output-json baseline-results.json
```

**Scenario 2: Default Mode (Skip All CDNs)**

```bash
# 1000 IPs, skip all CDN providers
sudo ./target/release/prtip -sS -p 80,443 \
  --cdn-filter \
  --target-file benchmarks/04-Sprint6.3-Network-Optimization/targets/baseline-1000.txt \
  --output-json default-mode-results.json
```

**Scenario 3: Whitelist Mode (Cloudflare + AWS Only)**

```bash
# 1000 IPs, skip only Cloudflare and AWS
sudo ./target/release/prtip -sS -p 80,443 \
  --cdn-filter \
  --cdn-whitelist cloudflare,aws \
  --target-file benchmarks/04-Sprint6.3-Network-Optimization/targets/baseline-1000.txt \
  --output-json whitelist-mode-results.json
```

**Scenario 4: Blacklist Mode (Exclude Cloudflare)**

```bash
# 1000 IPs, skip all CDNs except Cloudflare
sudo ./target/release/prtip -sS -p 80,443 \
  --cdn-filter \
  --cdn-blacklist cloudflare \
  --target-file benchmarks/04-Sprint6.3-Network-Optimization/targets/baseline-1000.txt \
  --output-json blacklist-mode-results.json
```

**Scenario 5: IPv6 Filtering**

```bash
# 500 IPv6 IPs, skip detected CDNs
sudo ./target/release/prtip -sS -p 80,443 \
  --cdn-filter \
  --target-file benchmarks/04-Sprint6.3-Network-Optimization/targets/ipv6-500.txt \
  --output-json ipv6-results.json
```

**Scenario 6: Mixed IPv4/IPv6**

```bash
# 1000 mixed IPs (500 IPv4 + 500 IPv6), skip CDNs
sudo ./target/release/prtip -sS -p 80,443 \
  --cdn-filter \
  --target-file benchmarks/04-Sprint6.3-Network-Optimization/targets/mixed-1000.txt \
  --output-json mixed-results.json
```

## Target IP Lists

### Required Files

Create target IP files in `benchmarks/04-Sprint6.3-Network-Optimization/targets/`:

1. **baseline-1000.txt** - 1000 IPs (500 CDN + 500 non-CDN)
   - 100 Cloudflare IPs (104.16.x.x, 172.64.x.x)
   - 100 AWS CloudFront IPs (13.32.x.x, 13.224.x.x)
   - 100 Azure CDN IPs (20.21.x.x, 147.243.x.x)
   - 100 Akamai IPs (23.x.x.x, 104.64.x.x)
   - 50 Fastly IPs (151.101.x.x)
   - 50 Google Cloud IPs (34.64.x.x, 35.192.x.x)
   - 500 non-CDN IPs (192.168.x.x, 10.x.x.x)

2. **ipv6-500.txt** - 500 IPv6 IPs (250 CDN + 250 non-CDN)
   - CDN IPv6 ranges: 2606:4700::/32, 2600:9000::/28, etc.
   - Non-CDN IPv6: 2001:db8::/32, fc00::/7

3. **mixed-1000.txt** - 1000 mixed IPs (500 IPv4 + 500 IPv6)
   - 250 IPv4 CDN + 250 IPv4 non-CDN
   - 250 IPv6 CDN + 250 IPv6 non-CDN

## Validation Metrics

### Performance Overhead

Measure CDN filtering overhead:

```bash
# Time baseline (no filtering)
time sudo ./target/release/prtip -sS -p 80,443 --target-file baseline-1000.txt

# Time with CDN filtering
time sudo ./target/release/prtip -sS -p 80,443 --cdn-filter --target-file baseline-1000.txt

# Calculate overhead percentage
# Overhead % = ((filtered_time - baseline_time) / baseline_time) × 100
# Target: < 5% | Acceptable: < 10%
```

### Scan Reduction

Measure IPs skipped vs scanned:

```bash
# Parse output JSON for metrics
jq '.summary.total_ips, .summary.ips_scanned, .summary.ips_skipped' results.json

# Calculate reduction percentage
# Reduction % = (ips_skipped / total_ips) × 100
# Default mode target: ≥ 45% reduction
```

### Detection Accuracy

Validate CDN IPs correctly detected:

```bash
# Count CDN IPs that should have been skipped but weren't
# Accuracy % = (correctly_filtered / total_cdn_ips) × 100
# Target: ≥ 99% accuracy
```

## Expected Results

### Scenario 1: Baseline
- **Total IPs**: 1000
- **Scanned**: 1000
- **Skipped**: 0
- **Reduction**: 0%
- **Overhead**: 0% (reference)

### Scenario 2: Default Mode
- **Total IPs**: 1000
- **Scanned**: 500 (non-CDN only)
- **Skipped**: 500 (all CDN providers)
- **Reduction**: 50%
- **Overhead**: < 5% (target), < 10% (acceptable)

### Scenario 3: Whitelist Mode
- **Total IPs**: 1000
- **Scanned**: 800 (non-CDN + Azure/Akamai/Fastly/Google)
- **Skipped**: 200 (Cloudflare + AWS only)
- **Reduction**: 20%
- **Overhead**: < 5% (target), < 10% (acceptable)

### Scenario 4: Blacklist Mode
- **Total IPs**: 1000
- **Scanned**: 600 (non-CDN + Cloudflare)
- **Skipped**: 400 (AWS/Azure/Akamai/Fastly/Google)
- **Reduction**: 40%
- **Overhead**: < 5% (target), < 10% (acceptable)

### Scenario 5: IPv6 Filtering
- **Total IPs**: 500
- **Scanned**: 250 (non-CDN IPv6)
- **Skipped**: 250 (CDN IPv6)
- **Reduction**: 50%
- **Overhead**: < 5% (target), < 10% (acceptable)

### Scenario 6: Mixed IPv4/IPv6
- **Total IPs**: 1000 (500 IPv4 + 500 IPv6)
- **Scanned**: 500 (250 non-CDN IPv4 + 250 non-CDN IPv6)
- **Skipped**: 500 (250 CDN IPv4 + 250 CDN IPv6)
- **Reduction**: 50%
- **Overhead**: < 5% (target), < 10% (acceptable)

## Success Criteria

### Performance ✅
- [ ] CDN filtering overhead < 5% (target) or < 10% (acceptable)
- [ ] No regression in baseline scan performance
- [ ] Memory usage increase < 5 MB

### Functionality ✅
- [ ] Default mode: ≥45% reduction (500/1000 IPs skipped)
- [ ] Whitelist mode: ≥18% reduction (200/1000 IPs skipped)
- [ ] Blacklist mode: ≥35% reduction (400/1000 IPs skipped)
- [ ] IPv6 detection: ≥45% reduction (250/500 IPs skipped)
- [ ] Mixed IPv4/IPv6: ≥45% reduction (500/1000 IPs skipped)

### Accuracy ✅
- [ ] CDN detection accuracy ≥99%
- [ ] Zero false positives (non-CDN IPs incorrectly filtered)
- [ ] Zero false negatives (CDN IPs not detected)

## Analysis Scripts

### Generate Summary Report

```bash
#!/bin/bash
# benchmarks/04-Sprint6.3-Network-Optimization/analyze-cdn-results.sh

echo "CDN Deduplication Benchmark Results"
echo "===================================="
echo ""

for scenario in baseline default whitelist blacklist ipv6 mixed; do
  echo "Scenario: ${scenario}"

  # Extract metrics from JSON
  total=$(jq '.summary.total_ips' ${scenario}-results.json)
  scanned=$(jq '.summary.ips_scanned' ${scenario}-results.json)
  skipped=$(jq '.summary.ips_skipped' ${scenario}-results.json)
  time=$(jq '.summary.scan_duration_ms' ${scenario}-results.json)

  # Calculate reduction
  reduction=$(echo "scale=2; ($skipped / $total) * 100" | bc)

  echo "  Total IPs:    $total"
  echo "  Scanned:      $scanned"
  echo "  Skipped:      $skipped"
  echo "  Reduction:    ${reduction}%"
  echo "  Time:         ${time}ms"
  echo ""
done

# Calculate overhead
baseline_time=$(jq '.summary.scan_duration_ms' baseline-results.json)
filtered_time=$(jq '.summary.scan_duration_ms' default-results.json)
overhead=$(echo "scale=2; (($filtered_time - $baseline_time) / $baseline_time) * 100" | bc)

echo "Performance Overhead: ${overhead}%"
echo ""

# Validate targets
if (( $(echo "$overhead < 10.0" | bc -l) )); then
  echo "✅ Overhead < 10% (acceptable)"
else
  echo "❌ Overhead ≥ 10% (FAIL)"
fi
```

## Notes

- **Root Required**: Raw socket creation requires elevated privileges
- **Target Files**: Generate IP lists from CDN CIDR ranges (see sample_ip_ranges in JSON)
- **Result Format**: JSON output includes summary metrics for automated validation
- **CI Integration**: Can be integrated into GitHub Actions for regression detection

## Related Documentation

- [31-BENCHMARKING-GUIDE.md](/docs/31-BENCHMARKING-GUIDE.md) - General benchmarking guide
- [34-PERFORMANCE-CHARACTERISTICS.md](/docs/34-PERFORMANCE-CHARACTERISTICS.md) - Performance baselines
- Sprint 6.3 Task Area 2 Completion Report (pending)

---

**Last Updated:** 2025-11-15
**Status:** Benchmark scenarios defined, ready for execution
