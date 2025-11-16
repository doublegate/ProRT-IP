# Reference Documentation

Comprehensive technical reference, API documentation, troubleshooting guides, and frequently asked questions for ProRT-IP.

## Quick Navigation

### [Technical Specification v2.0](tech-spec-v2.md)
Complete technical specification covering architecture, implementation details, and design decisions.

**Topics:**
- System architecture and component design
- Network protocols and packet structures
- Performance characteristics and benchmarks
- Security model and privilege handling
- Database schema and storage optimization
- Platform support and compatibility matrix

**When to use:** Detailed understanding of ProRT-IP internals, architecture decisions, or implementation details.

---

### [API Reference](api-reference.md)
Complete API documentation for ProRT-IP's public interfaces, configuration options, and plugin system.

**Topics:**
- Core scanning APIs (SYN, Connect, UDP, Stealth)
- Service detection and OS fingerprinting APIs
- Configuration and timing options
- Output and export interfaces
- Plugin system and Lua integration
- Rate limiting and performance tuning

**When to use:** Integrating ProRT-IP into applications, developing plugins, or programmatic usage.

---

### [FAQ](faq.md)
Frequently asked questions covering installation, usage, performance, and best practices.

**Topics:**
- General questions (comparison to Nmap, platform support, legality)
- Installation and setup (dependencies, privilege configuration)
- Usage questions (scanning networks, service detection, OS fingerprinting)
- Performance questions (packet rates, optimization, distributed scanning)
- Common errors (permissions, file limits, networking issues)
- Best practices (timing templates, incremental saves, progress monitoring)

**When to use:** Quick answers to common questions, installation guidance, or usage examples.

---

### [Troubleshooting Guide](troubleshooting.md)
Comprehensive troubleshooting procedures for all ProRT-IP issues across platforms.

**Topics:**
- Common issues (permission denied, packet capture failures, timeouts)
- Platform-specific issues (Linux, Windows, macOS)
- Performance issues (slow scanning, high memory, CPU bottlenecks)
- Output and export problems
- Database issues
- IPv6 support status
- Advanced troubleshooting tools
- Getting help and reporting bugs

**When to use:** Diagnosing errors, fixing platform-specific problems, or performance optimization.

---

## Common Reference Paths

### Installation Issues
1. **libpcap not found** → [FAQ: Installation and Setup](faq.md#libpcap-not-found-during-build)
2. **Permission denied** → [Troubleshooting: Permission Denied Errors](troubleshooting.md#permission-denied-errors)
3. **Running without root** → [FAQ: How do I run without root/sudo?](faq.md#how-do-i-run-without-rootsudo)

### Usage Guidance
1. **Fast network scanning** → [FAQ: What's the fastest way to scan a /24 network?](faq.md#whats-the-fastest-way-to-scan-a-24-network)
2. **Service detection** → [FAQ: How do I detect service versions?](faq.md#how-do-i-detect-service-versions)
3. **OS fingerprinting** → [FAQ: How do I perform OS fingerprinting?](faq.md#how-do-i-perform-os-fingerprinting)

### Performance Optimization
1. **Slow scans** → [FAQ: Why is my scan slow?](faq.md#why-is-my-scan-slow)
2. **Packet rates** → [FAQ: How many packets per second?](faq.md#how-many-packets-per-second-can-prt-ip-achieve)
3. **Memory usage** → [Troubleshooting: High Memory Usage](troubleshooting.md#high-memory-usage)
4. **Performance profiling** → [Troubleshooting: Performance Profiling](troubleshooting.md#performance-profiling)

### Platform-Specific Help
1. **Linux** → [Troubleshooting: Linux](troubleshooting.md#linux)
2. **Windows** → [Troubleshooting: Windows](troubleshooting.md#windows)
3. **macOS** → [Troubleshooting: macOS](troubleshooting.md#macos)

### Technical Details
1. **Architecture** → [Technical Specification: Architecture](tech-spec-v2.md#architecture)
2. **Scan types** → [Technical Specification: Scan Types](tech-spec-v2.md#scan-types)
3. **Performance** → [Technical Specification: Performance Characteristics](tech-spec-v2.md#performance-characteristics)
4. **Security** → [Technical Specification: Security Model](tech-spec-v2.md#security-model)

### API Integration
1. **Core APIs** → [API Reference: Core APIs](api-reference.md#core-apis)
2. **Configuration** → [API Reference: Configuration](api-reference.md#configuration)
3. **Plugin system** → [API Reference: Plugin System](api-reference.md#plugin-system)
4. **Output formats** → [API Reference: Output Formats](api-reference.md#output-formats)

---

## Reference Document Comparison

| Document | Purpose | Audience | Depth |
|----------|---------|----------|-------|
| **Technical Spec** | Architecture and design | Developers, contributors | Deep technical |
| **API Reference** | Public interfaces and APIs | Integrators, plugin developers | Complete API coverage |
| **FAQ** | Common questions and answers | All users | Quick answers |
| **Troubleshooting** | Problem diagnosis and fixes | All users | Step-by-step procedures |

---

## Quick Reference Cards

### Essential Commands
```bash
# Fast network scan
prtip -sS -p 80,443,22,21,25 --max-rate 100000 192.168.1.0/24

# Service detection
prtip -sS -sV -p 1-1000 target.com

# OS fingerprinting
prtip -sS -O target.com

# Full scan with service detection
prtip -sS -sV -p- -T4 target.com

# Stealth scan with evasion
prtip -sS -f -D RND:10 --ttl 64 target.com
```

### Timing Templates
```bash
-T0  # Paranoid (5 min timeout, 1-10 pps, IDS evasion)
-T1  # Sneaky (15 sec timeout, 10-100 pps, unreliable networks)
-T2  # Polite (1 sec timeout, 100-1K pps, Internet scanning)
-T3  # Normal (1 sec timeout, 1K-10K pps, default)
-T4  # Aggressive (500ms timeout, 10K-100K pps, LANs)
-T5  # Insane (100ms timeout, 10K-100K pps, fast LANs)
```

### Output Formats
```bash
-oN results.txt      # Normal output
-oX results.xml      # XML output (Nmap-compatible)
-oG results.gnmap    # Greppable output
-oJ results.json     # JSON output
-oA results          # All formats (txt, xml, json)
--with-db            # SQLite database storage
```

### Debug Logging
```bash
RUST_LOG=info prtip -sS target.com     # Basic logging
RUST_LOG=debug prtip -sS target.com    # Detailed logging
RUST_LOG=trace prtip -sS target.com    # Maximum verbosity
```

---

## Version-Specific Notes

### v0.4.0 (Phase 4 Complete)
- Partial IPv6 support (TCP Connect only)
- 8 scan types available
- Service detection: 500+ services
- OS fingerprinting with Nmap database
- PCAPNG packet capture
- Rate limiting with -1.8% overhead

### v0.5.0 (Phase 5 Complete)
- **Full IPv6 support** (all scan types)
- TLS certificate analysis
- Enhanced service detection (85-90% accuracy)
- Idle scan implementation
- Lua plugin system
- Comprehensive fuzz testing (230M+ executions)
- 54.92% code coverage
- 2,102 tests passing

### v0.6.0+ (Planned)
- Terminal UI (TUI) interface
- Network optimizations
- Interactive target selection
- Configuration profiles
- Enhanced help system

---

## External Resources

### Official Documentation
- [GitHub Repository](https://github.com/doublegate/ProRT-IP)
- [Release Notes](https://github.com/doublegate/ProRT-IP/releases)
- [Issue Tracker](https://github.com/doublegate/ProRT-IP/issues)

### Community
- [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)
- [Security Policy](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md)
- [Contributing Guide](https://github.com/doublegate/ProRT-IP/blob/main/CONTRIBUTING.md)

---

## See Also

- [User Guide](../user-guide/index.md) - Step-by-step usage instructions
- [Getting Started](../getting-started/index.md) - Installation and first scan
- [Features](../features/index.md) - Detailed feature documentation
- [Advanced Topics](../advanced/index.md) - Performance tuning and optimization
- [Development](../development/index.md) - Contributing and architecture
