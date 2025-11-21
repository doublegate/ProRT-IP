# Improvement Roadmap

This document outlines planned improvements and optimization opportunities for ProRT-IP.

## Current Status (v0.6.0)

- **Phase:** 6 (TUI + Network Optimizations)
- **Sprint:** 6.3 Complete
- **Progress:** ~73% overall (5.5/8 phases)

## Optimization Tiers

### Tier 1: Quick Wins (High ROI)

| Optimization | Impact | Effort | Status |
|--------------|--------|--------|--------|
| O(N) Connection State | 50-1000x | 8h | Complete |
| Batch I/O Defaults | 8-12% | 4h | Complete |
| CDN Deduplication | 83.3% reduction | 6h | Complete |
| Adaptive Batching | Configurable | 4h | Complete |

### Tier 2: Medium Term

| Optimization | Expected Impact | Effort |
|--------------|-----------------|--------|
| Zero-Copy TUI Integration | 15-25% memory | 8h |
| DashMap Replacement (papaya/scc) | 2-5x gains | 12h |
| Result Vector Preallocation | 10-15% memory | 4h |
| SIMD Packet Processing | 20-30% CPU | 16h |

### Tier 3: Long Term

| Optimization | Expected Impact | Effort |
|--------------|-----------------|--------|
| io_uring Integration | 30-50% I/O | 40h |
| AF_XDP Support | 2x throughput | 60h |
| GPU Acceleration | 10x crypto | 80h |

## Feature Roadmap

### Phase 6 Remaining (Sprints 6.4-6.8)

| Sprint | Focus | Duration |
|--------|-------|----------|
| 6.4 | Zero-Copy TUI Integration | 1 week |
| 6.5 | Interactive Selection | 1 week |
| 6.6 | Configuration Profiles | 1 week |
| 6.7 | Help System | 1 week |
| 6.8 | Polish & Documentation | 1 week |

### Phase 7: Advanced Detection

| Feature | Description |
|---------|-------------|
| Script Engine | NSE-compatible scripting |
| Vulnerability Detection | CVE correlation |
| Asset Discovery | Network topology mapping |
| Protocol Dissection | Deep packet inspection |

### Phase 8: Enterprise Features

| Feature | Description |
|---------|-------------|
| Distributed Scanning | Multi-node coordination |
| REST API | Remote control interface |
| Web Dashboard | Browser-based management |
| Report Generation | PDF/HTML reports |

## Performance Targets

### Current Achievements

| Metric | Target | Achieved |
|--------|--------|----------|
| TUI FPS | 60 | 60 |
| Event Throughput | 5K/sec | 10K+/sec |
| Syscall Reduction | 90% | 96.87-99.90% |
| CDN Filtering | 80% | 83.3% |
| Rate Limit Overhead | <5% | -1.8% |

### Future Targets

| Metric | Phase 7 | Phase 8 |
|--------|---------|---------|
| Throughput | 15M pps | 20M pps |
| Memory (65K scan) | 75MB | 50MB |
| Service Detection | 92% | 95% |
| IPv6 Coverage | 100% | 100% |

## Architecture Improvements

### Planned Refactoring

1. **Connection State Manager**
   - Abstract scanner-specific implementations
   - Enable pluggable backends

2. **Plugin API v2**
   - Async plugin support
   - Capability-based sandboxing
   - Hot reload improvements

3. **Output Pipeline**
   - Streaming JSON support
   - Custom formatters
   - Compression options

### Code Quality Goals

| Metric | Current | Target |
|--------|---------|--------|
| Test Coverage | 54.92% | 70% |
| Clippy Warnings | 0 | 0 |
| Documentation | Good | Excellent |
| Fuzzing Executions | 230M+ | 500M+ |

## Community Contributions

### Contribution Opportunities

| Area | Difficulty | Impact |
|------|------------|--------|
| Service probes | Easy | High |
| OS fingerprints | Medium | High |
| Lua plugins | Easy | Medium |
| Documentation | Easy | Medium |
| Performance testing | Medium | High |

## See Also

- [Competitive Analysis](./competitive-analysis.md)
- [Project Roadmap](../../project/roadmap.md)
- [Contributing Guide](../../development/contributing.md)

