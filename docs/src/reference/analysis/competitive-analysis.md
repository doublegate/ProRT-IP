# Competitive Analysis

This document provides a comprehensive comparison of ProRT-IP against other network scanning tools.

## Scanner Comparison Matrix

| Feature | ProRT-IP | Nmap | Masscan | RustScan | ZMap |
|---------|----------|------|---------|----------|------|
| **Speed (pps)** | 10M+ | 1K-10K | 10M+ | 10M+ | 10M+ |
| **Service Detection** | 85-90% | 95%+ | None | Nmap wrapper | None |
| **OS Fingerprinting** | Yes | Yes | No | Nmap wrapper | No |
| **IPv6 Support** | 100% | Yes | Limited | Partial | Yes |
| **Stealth Scans** | 8 types | 6 types | SYN only | SYN only | SYN only |
| **TUI Dashboard** | Yes | No | No | Yes | No |
| **Plugin System** | Lua 5.4 | NSE | No | No | No |

## Speed Comparison

### Throughput Benchmarks

| Scanner | 1K Ports | 10K Ports | 65K Ports |
|---------|----------|-----------|-----------|
| ProRT-IP | 250ms | 1.8s | 8.2s |
| Nmap | 3.2s | 28s | 180s+ |
| Masscan | 200ms | 1.5s | 7s |
| RustScan | 220ms | 1.6s | 7.5s |

**Key Insight:** ProRT-IP achieves 12-15x speedup over Nmap while maintaining service detection capabilities.

## Feature Deep Dive

### Service Detection

| Scanner | Accuracy | Probe Count | Version Detection |
|---------|----------|-------------|-------------------|
| ProRT-IP | 85-90% | 187 | Yes |
| Nmap | 95%+ | 11,000+ | Yes |
| Masscan | N/A | N/A | No |
| RustScan | Via Nmap | Via Nmap | Via Nmap |

ProRT-IP optimizes for the most common services, covering 85-90% of real-world scenarios with significantly fewer probes.

### Stealth Capabilities

| Technique | ProRT-IP | Nmap | Others |
|-----------|----------|------|--------|
| SYN Scan | Yes | Yes | Yes |
| FIN Scan | Yes | Yes | No |
| NULL Scan | Yes | Yes | No |
| Xmas Scan | Yes | Yes | No |
| ACK Scan | Yes | Yes | No |
| Idle Scan | Yes | Yes | No |
| Fragmentation | Yes | Yes | Limited |
| Decoy Scanning | Yes | Yes | No |

### Memory Efficiency

| Scanner | Idle | 1K Ports | 65K Ports |
|---------|------|----------|-----------|
| ProRT-IP | 12MB | 45MB | 95MB |
| Nmap | 50MB | 150MB | 400MB+ |
| Masscan | 20MB | 60MB | 150MB |

ProRT-IP's zero-copy architecture minimizes memory overhead.

## Unique ProRT-IP Features

### 1. Hybrid Architecture
Combines Masscan-level speed with Nmap-level detection in a single tool.

### 2. Real-Time TUI
60 FPS dashboard with live port discovery, service detection, and metrics.

### 3. Adaptive Rate Limiting
-1.8% overhead with automatic network condition adaptation.

### 4. CDN Deduplication
83.3% reduction in redundant scans through intelligent IP filtering.

### 5. Batch I/O
96.87-99.90% syscall reduction through sendmmsg/recvmmsg.

## When to Use Each Scanner

| Use Case | Recommended | Reason |
|----------|-------------|--------|
| Internet-scale surveys | ProRT-IP, Masscan | Speed |
| Detailed host analysis | Nmap | Comprehensive scripts |
| Quick network inventory | ProRT-IP, RustScan | Speed + detection |
| Stealth penetration testing | ProRT-IP, Nmap | Evasion techniques |
| Research projects | ZMap | Academic tooling |

## Conclusion

ProRT-IP occupies a unique position combining:
- **Masscan speed** (10M+ pps)
- **Nmap features** (service detection, OS fingerprinting, stealth)
- **Modern UX** (TUI dashboard, progress indicators)
- **Rust safety** (memory safety, thread safety)

## See Also

- [Performance Characteristics](../../advanced/performance-characteristics.md)
- [Benchmarking Guide](../../advanced/benchmarking.md)
- [Improvement Roadmap](./improvements.md)

