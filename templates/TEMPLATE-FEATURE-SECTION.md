# ProRT-IP Feature Section Template

**Version:** 1.0.0
**Last Updated:** 2025-11-15
**Purpose:** Standard format for documenting new features in README.md

---

## Overview

This template provides a standardized structure for documenting new features in ProRT-IP's README.md. Use this template when adding feature sections to ensure consistency, completeness, and professional presentation.

### When to Use This Template

- Adding new major features to README.md (scan types, protocols, analysis capabilities)
- Documenting feature enhancements that warrant dedicated sections
- Creating feature subsections within phase documentation
- Updating existing feature documentation for consistency

### Template Structure

Each feature section should include:
1. **Feature Header** - Name, status badge, version
2. **Feature Overview** - Purpose, capabilities, use cases
3. **Technical Details** - Implementation, architecture, performance
4. **Usage Examples** - CLI commands, code snippets, workflows
5. **Performance Metrics** - Speed, accuracy, resource usage (if applicable)
6. **Configuration** - Available options, flags, parameters
7. **Limitations** - Known constraints, edge cases, platform differences
8. **Documentation References** - Links to detailed guides

---

## Feature Section Template

```markdown
### [FEATURE_NAME] [STATUS_BADGE]

**Version:** v[X.Y.Z]
**Phase:** [PHASE_NUMBER]
**Sprint:** [SPRINT_NUMBER]

#### Overview

[FEATURE_DESCRIPTION - 2-3 sentences explaining what the feature does and why it's valuable]

**Key Capabilities:**
- [CAPABILITY_1] - [Brief description]
- [CAPABILITY_2] - [Brief description]
- [CAPABILITY_3] - [Brief description]

**Use Cases:**
- [USE_CASE_1]
- [USE_CASE_2]
- [USE_CASE_3]

#### Technical Details

**Implementation:**
- [TECH_DETAIL_1]
- [TECH_DETAIL_2]
- [TECH_DETAIL_3]

**Architecture:**
```text
[ASCII diagram or flowchart if applicable]
```

**Dependencies:**
- [DEPENDENCY_1] v[X.Y] - [Purpose]
- [DEPENDENCY_2] v[X.Y] - [Purpose]

#### Usage Examples

**Basic Usage:**
```bash
prtip [BASIC_COMMAND]
```

**Advanced Usage:**
```bash
prtip [ADVANCED_COMMAND]
```

**With Other Features:**
```bash
prtip [COMBINED_COMMAND]
```

#### Performance Metrics

| Metric | Value | Comparison |
|--------|-------|------------|
| [METRIC_1] | [VALUE] | [BASELINE_COMPARISON] |
| [METRIC_2] | [VALUE] | [BASELINE_COMPARISON] |
| [METRIC_3] | [VALUE] | [BASELINE_COMPARISON] |

**Benchmarks:**
- [BENCHMARK_1]: [RESULT]
- [BENCHMARK_2]: [RESULT]

#### Configuration

**CLI Flags:**
- `--[FLAG_1]` - [Description]
- `--[FLAG_2]` - [Description]
- `--[FLAG_3]` - [Description]

**Environment Variables:**
- `PRTIP_[VAR_1]` - [Description] (default: `[DEFAULT]`)
- `PRTIP_[VAR_2]` - [Description] (default: `[DEFAULT]`)

**Config File:**
```toml
[section]
option = "value"
```

#### Limitations

- **[LIMITATION_1]:** [Description and workaround if available]
- **[LIMITATION_2]:** [Description and workaround if available]
- **Platform Differences:** [OS-specific behaviors if applicable]

#### Documentation

- 📖 [Comprehensive Guide](docs/[NUMBER]-[FEATURE]-GUIDE.md)
- 🔧 [API Reference](docs/[NUMBER]-[FEATURE]-REFERENCE.md)
- 📝 [Examples](docs/[NUMBER]-[FEATURE]-EXAMPLES.md)
- 🎯 [Use Cases](docs/[NUMBER]-[FEATURE]-USE-CASES.md)
```

---

## Placeholder Reference Guide

### Required Placeholders

| Placeholder | Type | Description | Example |
|------------|------|-------------|---------|
| `[FEATURE_NAME]` | String | Feature name in Title Case | "IPv6 Support" |
| `[STATUS_BADGE]` | Emoji | Feature status indicator | "✅ STABLE" or "🔄 BETA" |
| `[X.Y.Z]` | Version | Version when feature introduced | "0.5.0" |
| `[PHASE_NUMBER]` | Integer | Phase number | "5" or "6" |
| `[SPRINT_NUMBER]` | String | Sprint identifier | "5.1" or "6.3" |
| `[FEATURE_DESCRIPTION]` | Text | 2-3 sentence overview | See examples below |
| `[CAPABILITY_N]` | String | Key capability name | "Protocol Detection" |
| `[USE_CASE_N]` | String | Common use case | "Security auditing" |
| `[TECH_DETAIL_N]` | String | Technical implementation detail | "Async I/O with Tokio" |
| `[DEPENDENCY_N]` | String | External dependency | "pnet" |
| `[BASIC_COMMAND]` | Bash | Simple CLI example | "-sS -p 80 192.168.1.1" |
| `[ADVANCED_COMMAND]` | Bash | Complex CLI example | "-sS -6 -p 1-65535 --ipv6-discovery 2001:db8::1" |
| `[COMBINED_COMMAND]` | Bash | Feature combination example | "-sS -sV -O --ipv6 target" |
| `[METRIC_N]` | String | Performance metric name | "Scan Speed" |
| `[VALUE]` | Number/String | Metric value | "10M pps" or "85-90%" |
| `[BASELINE_COMPARISON]` | String | Comparison to baseline/competitors | "+200% vs Nmap" |
| `[BENCHMARK_N]` | String | Benchmark scenario | "65K ports IPv6 scan" |
| `[RESULT]` | String | Benchmark result | "287ms (103K pps)" |
| `[FLAG_N]` | String | CLI flag name | "ipv6-discovery" |
| `[VAR_N]` | String | Environment variable name | "IPV6_ENABLED" |
| `[DEFAULT]` | String | Default value | "true" or "1000" |
| `[LIMITATION_N]` | String | Known limitation | "IPv6 Windows requires Admin" |
| `[NUMBER]` | Integer | Documentation file number | "23" or "24" |
| `[FEATURE]` | String | Feature slug for filenames | "IPV6-SUPPORT" |

### Status Badges

Use these standardized status indicators:

- `✅ STABLE` - Production-ready, fully tested
- `🚀 PRODUCTION` - Actively used in production
- `🔄 BETA` - Feature complete, testing in progress
- `⚠️ EXPERIMENTAL` - Early development, API may change
- `🔧 IN PROGRESS` - Active development
- `📋 PLANNED` - Scheduled for future phase
- `🔒 DEPRECATED` - Marked for removal

---

## Usage Instructions

### Adding a New Feature Section

1. **Copy the template** from the "Feature Section Template" section above
2. **Replace all placeholders** with actual feature details
3. **Customize sections** - remove irrelevant sections, add feature-specific content
4. **Add to README.md** in the appropriate location:
   - Major features: Dedicated section under "Key Features"
   - Phase features: Within active phase section
   - Scan types: Under "Scanning Capabilities"
   - Performance: Under "Performance" section
5. **Create supporting docs** - comprehensive guide, examples, API reference
6. **Update cross-references** - link from other related sections
7. **Validate links** - ensure all documentation links resolve
8. **Quality check** - run markdown-link-check, verify formatting

### Section Placement Guidelines

**Major Features (Top-Level Sections):**
Place after "Project Overview" and before "Development Roadmap"

**Phase Features:**
Place within the active phase section (Phase N: [NAME])

**Capability Enhancements:**
Update existing capability sections (e.g., "Scanning Capabilities")

**Performance Features:**
Add to "Performance" section with benchmarks

### Maintenance Procedures

**When Feature Status Changes:**
1. Update `[STATUS_BADGE]` to reflect new status
2. Update version if API changed
3. Add migration notes if breaking changes
4. Update performance metrics if improved

**When Feature is Complete:**
1. Change status to ✅ STABLE or 🚀 PRODUCTION
2. Add final performance benchmarks
3. Complete all documentation links
4. Add to feature comparison matrix (if applicable)

**When Feature is Deprecated:**
1. Change status to 🔒 DEPRECATED
2. Add deprecation notice with timeline
3. Provide migration path to replacement
4. Update all references to warn users

---

## Examples

### Example 1: Simple Feature (Protocol Support)

```markdown
### UDP Protocol Support ✅ STABLE

**Version:** v0.3.0
**Phase:** 3
**Sprint:** 3.2

#### Overview

Full UDP scanning support with protocol-specific payloads and intelligent ICMP interpretation. Enables detection of UDP services like DNS, SNMP, and NetBIOS that don't respond to empty probes.

**Key Capabilities:**
- Protocol-specific UDP payloads (DNS, SNMP, NetBIOS, NTP)
- ICMP port unreachable interpretation
- Open|filtered state detection
- Retry logic for UDP reliability

**Use Cases:**
- DNS server discovery
- SNMP enumeration
- NetBIOS name service scanning
- VoIP infrastructure mapping

#### Technical Details

**Implementation:**
- Custom payload generation per protocol
- ICMP unreachable packet capture
- Exponential backoff retry (max 3 attempts)
- State inference from ICMP responses

**Dependencies:**
- pnet v0.34 - Raw socket access
- pnet_packet - UDP/ICMP packet construction

#### Usage Examples

**Basic UDP Scan:**
```bash
prtip -sU -p 53,161,137 192.168.1.0/24
```

**Full UDP Port Range:**
```bash
prtip -sU -p- --max-retries 3 target.com
```

**Combined TCP + UDP:**
```bash
prtip -sS -sU -p 1-1000 target.com
```

#### Performance Metrics

| Metric | Value | Comparison |
|--------|-------|------------|
| Scan Speed | 1K-10K pps | 10-100x slower than TCP SYN |
| Accuracy | 95%+ | Matches Nmap UDP detection |
| Retry Overhead | 2-3x | Configurable (--max-retries) |

**Note:** UDP scanning is inherently slower due to lack of three-way handshake and reliance on ICMP responses.

#### Configuration

**CLI Flags:**
- `-sU` - Enable UDP scanning
- `--max-retries N` - Maximum retry attempts (default: 2)
- `--udp-timeout MS` - Per-probe timeout (default: 1000ms)

#### Limitations

- **Speed:** UDP is 10-100x slower than TCP due to stateless nature
- **ICMP Rate Limiting:** Many networks rate-limit ICMP unreachable responses
- **Firewall Detection:** Difficult to distinguish filtered vs open|filtered ports
- **Platform Differences:** Windows requires Npcap, macOS needs root privileges

#### Documentation

- 📖 [UDP Scanning Guide](docs/16-UDP-SCANNING-GUIDE.md)
- 🔧 [Protocol Reference](docs/17-PROTOCOL-REFERENCE.md)
```

### Example 2: Complex Feature (Service Detection)

```markdown
### Service Detection & Version Fingerprinting 🚀 PRODUCTION

**Version:** v0.4.0
**Phase:** 5
**Sprint:** 5.2

#### Overview

Deep service detection using nmap-service-probes database (187 probes, 2,600+ signatures). Achieves 85-90% accurate identification of services, versions, and additional metadata through intelligent protocol-based probing.

**Key Capabilities:**
- 187 protocol-specific probes (HTTP, SSH, FTP, SMTP, etc.)
- 2,600+ service/version signatures
- Banner grabbing and regex matching
- CPE (Common Platform Enumeration) extraction
- OS hints from service banners
- SSL/TLS certificate analysis integration

**Use Cases:**
- Vulnerability assessment (version-based CVE mapping)
- Service inventory and asset discovery
- Compliance auditing (unauthorized service detection)
- Network documentation and mapping

#### Technical Details

**Implementation:**
- Probe selection algorithm (TCP/UDP protocol-aware)
- Parallel probe execution (max 5 concurrent per target)
- Regex engine for signature matching (regex crate)
- Fallback to banner grabbing for unknown services
- Integration with TLS certificate parser (Phase 5.5)

**Architecture:**
```text
Target Port (Open)
      ↓
  Probe Selector ──→ Protocol Hints (SYN response flags)
      ↓
  Probe Executor ──→ Send up to 5 probes in parallel
      ↓
  Response Parser ──→ Regex match against 2,600+ signatures
      ↓
Service/Version ──→ Extract CPE, OS hints, metadata
```

**Dependencies:**
- regex v1.10 - Pattern matching engine
- tokio v1.35 - Async probe execution
- prtip-network - Raw socket communication

#### Usage Examples

**Basic Service Detection:**
```bash
prtip -sS -sV -p 1-1000 192.168.1.1
```

**Aggressive Service Detection:**
```bash
prtip -sS -sV --version-intensity 9 -p- target.com
```

**Combined with OS Detection:**
```bash
prtip -A -p 1-1000 target.com  # -A includes -sV -O
```

**Service Detection on Specific Ports:**
```bash
prtip -sS -sV -p 22,80,443,3306 webserver.com
```

#### Performance Metrics

| Metric | Value | Comparison |
|--------|-------|------------|
| Accuracy | 85-90% | Matches Nmap service detection |
| Probe Count | 187 probes | Nmap-service-probes v7.94 |
| Signature Count | 2,600+ | Full nmap database coverage |
| Avg. Detection Time | 2-5s per port | Depends on probe count |
| Memory Usage | ~500 MB | Probe/signature database |

**Benchmarks:**
- 10 ports HTTP/SSH/FTP: ~3.2s (parallel probing)
- 100 ports mixed services: ~28s (avg 0.28s per port)
- Full 65K port service scan: ~4.5 hours (sequential probing)

#### Configuration

**CLI Flags:**
- `-sV` - Enable service/version detection
- `--version-intensity N` - Probe aggressiveness (0-9, default: 7)
- `--version-light` - Limit to high-probability probes (faster, less accurate)
- `--version-all` - Try all probes (slower, most accurate)

**Environment Variables:**
- `PRTIP_SERVICE_PROBES` - Custom probe database path (default: embedded)
- `PRTIP_VERSION_TIMEOUT` - Per-probe timeout in ms (default: 5000)

**Config File:**
```toml
[service_detection]
enabled = true
intensity = 7
timeout_ms = 5000
max_parallel_probes = 5
```

#### Limitations

- **Speed:** Service detection adds 2-5s overhead per open port
- **Memory:** Requires ~500 MB for probe/signature database
- **Network Load:** Can trigger IDS/IPS due to probe patterns
- **Accuracy:** 10-15% of services may not match known signatures
- **Platform Differences:**
  - Windows: Some probes may timeout due to Npcap overhead
  - macOS: SSL/TLS probes require LibreSSL compatibility
  - Linux: Best performance, all probes supported

#### Documentation

- 📖 [Service Detection Guide](docs/24-SERVICE-DETECTION-GUIDE.md)
- 🔧 [Probe Customization](docs/24-SERVICE-DETECTION-GUIDE.md#custom-probes)
- 📝 [Service Examples](docs/34-EXAMPLES-GALLERY.md#service-detection)
- 🎯 [Accuracy Benchmarks](benchmarks/service-detection/)
```

### Example 3: Performance-Critical Feature (Rate Limiting)

```markdown
### Adaptive Rate Limiting V3 🚀 PRODUCTION

**Version:** v0.4.4
**Phase:** 5
**Sprint:** 5.X

#### Overview

Industry-leading adaptive rate limiting with -1.8% overhead (vs Nmap 10-20%, Masscan 5-10%). Three-layer architecture balances network courtesy, target protection, and maximum throughput through intelligent backpressure and burst management.

**Key Capabilities:**
- Three-layer rate control (global/per-target/burst)
- Adaptive backpressure (target response-based)
- Token bucket with configurable burst (100 packets)
- Sub-microsecond overhead (180ns per packet)
- Integration with all scan types

**Use Cases:**
- Respectful scanning (avoid overwhelming targets)
- IDS/IPS evasion (rate-based detection bypass)
- Network capacity management
- Compliance with scanning policies

#### Technical Details

**Implementation:**
- Lock-free token bucket (atomic operations)
- Per-target tracking with LRU eviction (10,000 target cache)
- Exponential backoff on ICMP admin prohibited
- Zero-copy integration (no packet buffering)

**Architecture:**
```text
Packet → Global Rate Limiter (max_pps)
            ↓
         Per-Target Limiter (max_pps_per_target)
            ↓
         Burst Manager (burst_size=100)
            ↓
         Network Send (zero overhead)
```

**Dependencies:**
- crossbeam v0.8 - Lock-free data structures
- tokio v1.35 - Async timing primitives

#### Usage Examples

**Conservative Scanning:**
```bash
prtip -sS --max-rate 1000 -p 1-65535 target.com
```

**Aggressive Scanning:**
```bash
prtip -sS --max-rate 100000 -p- target.com
```

**Per-Target Limiting:**
```bash
prtip -sS --max-rate 50000 --max-pps-per-target 1000 192.168.1.0/24
```

**Burst Control:**
```bash
prtip -sS --max-rate 10000 --burst-size 500 target.com
```

#### Performance Metrics

| Metric | Value | Comparison |
|--------|-------|------------|
| Overhead | -1.8% | Industry-leading (Nmap: 10-20%, Masscan: 5-10%) |
| Latency | 180ns per packet | Sub-microsecond decision time |
| Throughput | 10M+ pps | Validated at 100K pps (localhost) |
| Memory | 2.4 MB | 10,000 target tracking cache |
| Accuracy | ±0.5% | Actual rate within 0.5% of target |

**Benchmarks:**
- 65K ports @10K pps: 287ms (-1.6% overhead vs unlimited)
- 65K ports @50K pps: 259ms (-1.9% overhead)
- 65K ports @100K pps: 241ms (-2.1% overhead)
- **Negative overhead**: Rate limiter optimizes packet batching

#### Configuration

**CLI Flags:**
- `--max-rate N` - Global max packets per second (default: unlimited)
- `--max-pps-per-target N` - Per-target rate limit (default: unlimited)
- `--burst-size N` - Token bucket burst size (default: 100)
- `--rate-backoff FACTOR` - Backoff multiplier on ICMP admin prohibited (default: 2.0)

**Environment Variables:**
- `PRTIP_MAX_RATE` - Default global rate limit (default: unlimited)
- `PRTIP_BURST_SIZE` - Default burst size (default: 100)

**Config File:**
```toml
[rate_limiting]
max_rate = 50000              # Global limit (pps)
max_pps_per_target = 1000     # Per-target limit (pps)
burst_size = 100              # Burst buffer
backoff_factor = 2.0          # ICMP backoff multiplier
target_cache_size = 10000     # LRU cache size
```

#### Limitations

- **Target Cache:** LRU eviction at 10,000 targets (large scans may re-evaluate targets)
- **Burst Timing:** Initial burst may trigger rate-based IDS on sensitive networks
- **Localhost Testing:** Unrealistic speeds (10M+ pps) not achievable on internet targets
- **Platform Differences:**
  - Linux: sendmmsg batching enhances rate limiter efficiency
  - Windows: Npcap overhead may reduce effective max rate
  - macOS: BPF buffer limits may cause packet drops above 1M pps

#### Documentation

- 📖 [Rate Limiting Guide v1.1](docs/26-RATE-LIMITING-GUIDE.md)
- 🔧 [Performance Tuning](docs/34-PERFORMANCE-CHARACTERISTICS.md#rate-limiting)
- 📝 [Benchmarks](benchmarks/rate-limiting/)
- 🎯 [IDS Evasion Techniques](docs/07-SCANNING-TECHNIQUES.md#rate-based-evasion)
```

---

## Best Practices

### Content Quality

1. **Be Specific:** Use concrete numbers, benchmarks, and comparisons
2. **Show Examples:** Provide multiple usage examples from simple to complex
3. **Document Limitations:** Be transparent about constraints and edge cases
4. **Link to Guides:** Always reference comprehensive documentation
5. **Maintain Accuracy:** Update metrics when performance improves

### Writing Style

1. **Active Voice:** "Achieves 85% accuracy" not "Accuracy of 85% is achieved"
2. **Concise:** 2-3 sentences for overview, expand in guides
3. **Technical Precision:** Use exact version numbers, benchmark data
4. **User-Focused:** Explain "why" and "when to use", not just "what"

### Formatting Standards

1. **Consistent Headers:** Use H4 (####) for subsections
2. **Code Blocks:** Always specify language (```bash, ```toml, ```rust)
3. **Tables:** Use for structured data (metrics, flags, comparisons)
4. **Status Badges:** Standardized emojis for visual scanning

### Cross-Referencing

1. **Link to Guides:** Every feature section must link to comprehensive guide
2. **Update Related Sections:** Cross-link from phase sections, scan types, performance
3. **Maintain Bidirectional Links:** Guide should link back to README feature section
4. **Validate Links:** Run markdown-link-check before committing

---

## Quality Checklist

Before considering a feature section complete, verify:

- [ ] All placeholders replaced with actual values
- [ ] Status badge accurately reflects feature state
- [ ] Version number matches when feature was introduced
- [ ] Overview is 2-3 sentences, clear and concise
- [ ] At least 3 key capabilities listed
- [ ] At least 3 use cases provided
- [ ] Technical details explain implementation approach
- [ ] Minimum 3 usage examples (basic, advanced, combined)
- [ ] Performance metrics included with comparisons (if applicable)
- [ ] All CLI flags documented with descriptions
- [ ] Known limitations documented with workarounds
- [ ] Links to comprehensive guides included
- [ ] All documentation links resolve (run markdown-link-check)
- [ ] Code blocks specify language for syntax highlighting
- [ ] Tables properly formatted with alignment
- [ ] Cross-references to related sections added
- [ ] Feature added to appropriate location in README.md
- [ ] Supporting documentation created (guides, examples)
- [ ] Benchmarks executed and results documented (if performance-critical)

---

## Integration with Other Templates

### With Phase Section Template

Feature sections are typically embedded within phase sections:

```markdown
## Phase 5: Advanced Features 🔄 IN PROGRESS

### Sprint Progress
[Sprint table here]

### Current Sprint: 5.2 - Service Detection

### Key Features Delivered

#### Service Detection & Version Fingerprinting ✅ STABLE
[Use feature section template here]

#### TLS Certificate Analysis ✅ STABLE
[Use feature section template here]
```

### With Sprint Section Template

Features in active development appear in sprint sections:

```markdown
### Current Sprint: 6.1 - TUI Framework

**Deliverables:**
- [x] TUI Framework ✅
- [ ] Real-time Metrics Dashboard 🔄

#### TUI Framework ✅ STABLE
[Use feature section template here - marks deliverable as complete]
```

### With Archive Template

Completed features are preserved in phase archives:

```markdown
## Phase 5 Overview

### Major Features

#### IPv6 Complete Support ✅ STABLE
[Feature section from README, archived for historical reference]
```

---

## Maintenance Schedule

### Daily (During Active Development)
- Update status badge if feature state changes
- Add new usage examples as discovered
- Fix broken documentation links

### Weekly
- Update performance metrics if improved
- Add new limitations or workarounds discovered
- Review and improve usage examples

### Sprint Completion
- Finalize status badge (move from 🔄 BETA to ✅ STABLE)
- Complete all documentation links
- Add final benchmark results
- Cross-reference from phase section

### Phase Completion
- Archive feature section in PHASE-N-README-ARCHIVE.md
- Ensure all supporting docs complete
- Validate all cross-references
- Update feature comparison matrices

---

## Summary

This template ensures consistent, comprehensive, and professional documentation of ProRT-IP features in README.md. By following this structure, every feature section will provide users with clear understanding of capabilities, usage, performance, and limitations while maintaining cross-references to detailed documentation.

**Key Benefits:**
- Consistency across all feature documentation
- Complete information at-a-glance
- Clear usage examples for all skill levels
- Performance transparency with benchmarks
- Honest limitation disclosure
- Easy maintenance and updates

Use this template for all new features and to standardize existing feature sections during documentation cleanup phases.
