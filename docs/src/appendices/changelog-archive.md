# Changelog Archive

Historical changelog entries for ProRT-IP versions prior to v0.4.0. For current changes, see the main [CHANGELOG.md](https://github.com/doublegate/ProRT-IP/blob/main/CHANGELOG.md) in the repository root.

All notable changes to this project are documented here following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## Archive Overview

This archive contains changelog entries for:
- **Phase 1** (v0.1.0 - v0.1.5): Foundation - Core scanning infrastructure
- **Phase 2** (v0.2.0 - v0.2.8): Protocols - Service detection and OS fingerprinting
- **Phase 3** (v0.3.0 - v0.3.9): Quality - Testing, documentation, refinement

**Current Version:** See [CHANGELOG.md](https://github.com/doublegate/ProRT-IP/blob/main/CHANGELOG.md)
**Phase 4+:** v0.4.0 and later (Performance, Advanced Features, TUI)

---

## Quick Navigation

- [Phase 3 (v0.3.x)](#phase-3-quality---v03x) - Testing and documentation
- [Phase 2 (v0.2.x)](#phase-2-protocols---v02x) - Service detection and OS fingerprinting
- [Phase 1 (v0.1.x)](#phase-1-foundation---v01x) - Core scanning engine
- [Pre-release (v0.0.x)](#pre-release---v00x) - Early development

---

## Phase 3: Quality - v0.3.x

Comprehensive testing, documentation, and code refinement. Sprint 3.1-3.7 (Jan 2025).

### [v0.3.9] - 2025-01-28

#### Added
- **Sprint 3.7:** Documentation completeness (4,500+ lines)
  - User Guide v1.0.0 (1,200 lines) with tutorials, examples, CLI reference
  - Technical Specification v1.0.0 (3,800 lines) covering architecture, protocols, performance
  - FAQ section (45 questions) addressing common user inquiries
  - Troubleshooting guide for all platforms (Linux, macOS, Windows, FreeBSD)
- GitHub Discussions board for community support
- mdBook integration for professional documentation site
- 150+ code examples in documentation

#### Changed
- README.md restructured for clarity (executive summary, quick start, features)
- Installation guide expanded with platform-specific troubleshooting
- Error messages improved with actionable suggestions (e.g., "Permission denied" → "Run `sudo setcap cap_net_raw,cap_net_admin=eip ./prtip`")

#### Fixed
- Documentation cross-references (0 broken links verified)
- Code snippets formatting in README and guides
- Inconsistent terminology (standardized on "SYN Scan" vs "Half-Open Scan")

### [v0.3.8] - 2025-01-24

#### Added
- **Sprint 3.6:** Code coverage improvements
  - Line coverage: 37% → 52% (+15pp)
  - Function coverage: 45% → 58% (+13pp)
  - 98 new tests (total: 391 → 489)
  - CI/CD integration with Codecov.io for automated tracking
- Debug-only test getters (`#[cfg(debug_assertions)]`) for testing private methods
- Scanner initialization pattern documented for all test files

#### Changed
- Test structure: Unit tests organized by module (one `tests.rs` per module)
- Integration tests: Scenarios grouped by feature (scan types, evasion, output formats)
- Tarpaulin configuration: Excludes network and scanner crates to focus on core logic

#### Fixed
- Scanner tests: Added missing `.initialize().await` calls (fixed 8 test failures on macOS)
- Race condition in service detection tests (added timeout guards)
- Flaky tests: Stabilized timing-dependent tests with retry logic

### [v0.3.7] - 2025-01-20

#### Added
- **Sprint 3.5:** Fuzz testing infrastructure
  - cargo-fuzz integration with 5 fuzz targets
  - IPv4 packet parser fuzzing (10M executions, 0 crashes)
  - IPv6 packet parser fuzzing (8M executions, 0 crashes)
  - TCP header parser fuzzing (12M executions, 0 crashes)
  - ICMP packet parser fuzzing (5M executions, 0 crashes)
  - Service probe response fuzzing (15M executions, 0 crashes)
- Structure-aware fuzzing using `arbitrary` crate for realistic test data
- Continuous fuzzing in CI/CD (10K executions per run)

#### Changed
- Input validation: Hardened against malformed packets (added bounds checking)
- Error handling: Graceful degradation for corrupted data (returns `None` instead of panic)

#### Fixed
- Buffer overflow potential in Ethernet frame parsing (discovered via fuzzing)
- Integer overflow in ICMP checksum calculation (caught by fuzzing, now uses `checked_add`)
- Panic on empty service banner (fuzzing found edge case)

### [v0.3.6] - 2025-01-16

#### Added
- **Sprint 3.4:** Performance benchmarking framework
  - Hyperfine integration for 10 benchmark scenarios
  - CI/CD automation with regression detection (5%/10% thresholds)
  - Historical tracking of performance metrics over time
  - 31-BENCHMARKING-GUIDE.md (1,044 lines) documenting methodology
- Baseline performance data for Phase 3:
  - SYN scan (65,535 ports): 259ms (253K pps)
  - Connect scan (1,000 ports): 2.1s (476 pps)
  - Service detection (100 ports): 8.4s (11.9 ports/sec)
  - OS fingerprinting (1 host): 412ms (16-probe sequence)

#### Changed
- Binary optimization: `lto = "fat"` and `codegen-units = 1` for 8% performance gain
- Benchmarks run on release builds only (debug builds excluded from CI)

#### Fixed
- Timing accuracy: Benchmarks now use `wall-clock` time (consistent with user experience)
- CI concurrency: Benchmarks run sequentially to avoid resource contention

### [v0.3.5] - 2025-01-12

#### Added
- **Sprint 3.3:** CI/CD pipeline enhancements
  - GitHub Actions workflows: Format Check, Clippy Lint, Test (Linux/macOS/Windows), Security Audit
  - MSRV (Minimum Supported Rust Version) verification: Rust 1.70.0
  - Multi-platform release builds: 8 targets (Linux x86_64/ARM64 GNU/musl, macOS Intel/ARM64, Windows x86_64, FreeBSD x86_64)
  - Automated GitHub Releases with binaries attached
- Pre-commit hooks with `husky` equivalent (`cargo-husky` integration)
- Dependency caching in CI (85% cache hit rate, 2min → 30s build time)

#### Changed
- Release workflow: Triggers on tags matching `v*.*.*` pattern
- Security audit: Runs daily (not just on PR/push) to catch new CVEs
- Test timeouts: Increased from 3s to 5s to accommodate CI variability

#### Fixed
- Windows CI failures: Npcap installation automated via PowerShell script
- macOS loopback test failures: Documented as expected Npcap behavior (4 SYN discovery tests ignored)
- Clippy warnings: All warnings resolved (target: 0 warnings in CI)

### [v0.3.4] - 2025-01-08

#### Added
- **Sprint 3.2:** Output format enhancements
  - Greppable output format (`-oG`) for automated parsing
  - PCAPNG packet capture export (`--pcap-file output.pcapng`)
  - Database schema upgrades: scan_metadata table for tracking scan parameters
  - Result streaming to disk (prevents OOM on large scans)
- Output validation: Ensures valid JSON/XML/Greppable syntax

#### Changed
- JSON output: Compact by default (`-oJ`), pretty-print with `--json-pretty`
- XML output: Nmap-compatible schema with DTD validation
- Database: WAL mode enabled by default for better concurrency

#### Fixed
- JSON escaping: Special characters in banners (quotes, newlines) properly escaped
- XML entities: `<>&"'` characters encoded to prevent parsing errors
- Database transactions: Batch inserts (10K records/tx) prevent lock contention

### [v0.3.3] - 2025-01-04

#### Added
- **Sprint 3.1:** Error handling overhaul
  - Custom error types with `thiserror` crate (ScanError, NetworkError, ParseError)
  - Error context propagation with `anyhow` for debugging
  - User-friendly error messages (e.g., "Cannot create raw socket: Operation not permitted" → "Root privileges required. Run with `sudo` or grant capabilities: `sudo setcap cap_net_raw,cap_net_admin=eip ./prtip`")
- `Result<T>` return types for all fallible operations (no panics in production code)
- Comprehensive error logging with `tracing` crate (debug, info, warn, error levels)

#### Changed
- Panic-free guarantee: All `unwrap()` calls replaced with `?` operator or `expect()` with justification comments
- Error propagation: Top-level errors bubble up with full context chain

#### Fixed
- Graceful handling of ICMP "port unreachable" responses (no longer logs as error)
- Timeout errors: Clear distinction between network timeout vs. application timeout
- File I/O errors: Provides specific error messages (e.g., "Permission denied writing to /var/log/prtip.log")

### [v0.3.2] - 2024-12-28

#### Added
- Logging infrastructure with `env_logger` and `log` crate
- Log levels: TRACE, DEBUG, INFO, WARN, ERROR configurable via `RUST_LOG` environment variable
- Structured logging for scan events (start, progress, completion)
- Log rotation support (max size 100MB, keep 5 files)

#### Changed
- Default log level: WARN (production), INFO (development)
- Timestamp format: ISO 8601 with milliseconds

#### Fixed
- Log spam: Reduced "packet dropped" messages from DEBUG to TRACE
- File handle exhaustion: Logs buffered before writing (1MB buffer)

### [v0.3.1] - 2024-12-24

#### Added
- Unit tests: 85 new tests (total: 306 → 391)
- Integration tests: 15 scenarios covering end-to-end workflows
- Test coverage: Baseline 37% line coverage (goal: 60% by Phase 4)

#### Changed
- Test organization: Moved from inline `#[cfg(test)]` to dedicated `tests/` modules
- Mock data: Shared fixtures in `tests/common/mod.rs`

#### Fixed
- Test flakiness: 3 timing-dependent tests stabilized with retry logic
- CI test failures: macOS loopback issues documented (4 tests ignored on macOS)

### [v0.3.0] - 2024-12-20

**Phase 3 Start:** Quality, Testing, and Documentation (Sprints 3.1-3.7)

#### Added
- Error handling framework (thiserror, anyhow)
- Logging infrastructure (env_logger, log)
- Test coverage measurement (tarpaulin)
- Benchmarking framework (criterion)
- CI/CD pipeline (GitHub Actions)

#### Changed
- Architecture: Refactored for testability (dependency injection, trait abstractions)
- Documentation: Comprehensive user guide and technical specification

#### Summary
Phase 3 focused on production-readiness through comprehensive testing (391 tests, 52% coverage), professional documentation (5,500+ lines), CI/CD automation (8 workflows), and performance validation (259ms for 65K ports). All code quality metrics met: 0 clippy warnings, 0 panics, 100% of tests passing on Linux/macOS/Windows/FreeBSD.

---

## Phase 2: Protocols - v0.2.x

Service detection, OS fingerprinting, and protocol-specific scanning. Sprints 2.1-2.8 (Oct-Dec 2024).

### [v0.2.8] - 2024-12-16

#### Added
- **Sprint 2.8:** OS fingerprinting finalization
  - Nmap OS fingerprinting database integration (2,600+ signatures)
  - TCP/IP stack fingerprinting (16-probe sequence: TCP options, window size, ICMP responses)
  - Accuracy: 85% exact match, 95% family match (e.g., "Linux 2.6.x" → "Linux kernel")
  - Fallback: Generic detection when exact match fails (e.g., "Linux kernel", "Windows", "BSD")
- OS detection confidence scoring (0-100%, based on probe responses)
- 25-OS-DETECTION-GUIDE.md (680 lines) documenting methodology

#### Changed
- OS fingerprinting: Requires both open and closed ports (validates TCP behavior)
- Probe timeouts: 500ms per probe (faster than Nmap's 1s default)

#### Fixed
- False positives: Stricter matching criteria (requires 10+ probe matches)
- IPv6 OS detection: Limited support (fewer signatures available)

### [v0.2.7] - 2024-12-12

#### Added
- **Sprint 2.7:** Service detection accuracy improvements
  - 187 service probes from nmap-service-probes database
  - Intensity levels 0-9 (0=fastest/least accurate, 9=slowest/most accurate)
  - SSL/TLS detection: Automatic HTTPS/SMTPS/FTPS service detection
  - Version extraction: HTTP Server headers, SSH banners, FTP welcome messages
- Probe matching: Regex-based pattern matching (500+ patterns)
- Service confidence scoring (0-100%, based on probe matches)

#### Changed
- Service detection: Parallel probing (50 concurrent probes per target)
- Timeout: 5s per probe (configurable via `--probe-timeout`)

#### Fixed
- Probe selection: Intensity 0 now correctly uses 1 probe (was using 3)
- Banner grabbing: UTF-8 validation prevents crash on binary data

### [v0.2.6] - 2024-12-08

#### Added
- **Sprint 2.6:** UDP scanning enhancements
  - Protocol-specific payloads (DNS query for port 53, SNMP GetRequest for port 161)
  - ICMP unreachable detection (port-unreachable, host-unreachable, network-unreachable)
  - UDP service detection: DNS, SNMP, NTP, NetBIOS probes
- 24-UDP-SCANNING-GUIDE.md (450 lines) with protocol details

#### Changed
- UDP scan speed: ~100 pps (limited by ICMP rate limiting on most systems)
- Port states: open, closed, open|filtered, filtered (4 states vs. TCP's 3)

#### Fixed
- False positives: UDP ports showing "open|filtered" when no response (now correctly labeled)
- ICMP handling: Correctly interprets ICMP type 3 code 3 (port unreachable)

### [v0.2.5] - 2024-12-04

#### Added
- **Sprint 2.5:** Stealth scan types (FIN, NULL, Xmas)
  - FIN scan (`-sF`): TCP FIN flag set (bypasses stateless firewalls)
  - NULL scan (`-sN`): No TCP flags set (exploits RFC 793 compliance)
  - Xmas scan (`-sX`): FIN+PSH+URG flags set ("lights up like a Christmas tree")
- Port states: open|filtered, closed, filtered (stealth scans cannot distinguish open from filtered)
- 23-STEALTH-SCANNING-GUIDE.md (520 lines) with evasion techniques

#### Changed
- Stealth scans: Require root/capabilities (send raw TCP packets)
- Platform support: FIN/NULL/Xmas fail on Windows and Cisco routers (known limitation)

#### Fixed
- Firewall detection: Stealth scans now correctly identify filtered ports
- RST handling: Correctly interprets RST response as "closed" (not "filtered")

### [v0.2.4] - 2024-11-30

#### Added
- **Sprint 2.4:** ACK scanning for firewall detection
  - ACK scan (`-sA`): Sends TCP ACK packets to detect stateful firewalls
  - Port states: filtered, unfiltered (ACK scan doesn't determine open/closed)
  - Firewall inference: unfiltered=no stateful firewall, filtered=stateful firewall present
- 22-ACK-SCANNING-GUIDE.md (380 lines) with firewall detection techniques

#### Changed
- ACK scan: Returns "unfiltered" for RST response, "filtered" for no response or ICMP unreachable

#### Fixed
- State reporting: ACK scan no longer reports "open" or "closed" (only filtered/unfiltered)

### [v0.2.3] - 2024-11-26

#### Added
- **Sprint 2.3:** Connect scan (`-sT`)
  - Full TCP three-way handshake (SYN → SYN-ACK → ACK)
  - No root/capabilities required (uses OS's `connect()` syscall)
  - Slower than SYN scan (1K-5K pps vs. 50K+ pps), but more reliable
- Timeout handling: Configurable connection timeout (default 1s)
- 21-CONNECT-SCANNING-GUIDE.md (320 lines) comparing SYN vs. Connect

#### Changed
- Default scan type: SYN (`-sS`) if root/capabilities available, Connect (`-sT`) otherwise
- Port states: open, closed, filtered (3 states, same as SYN scan)

#### Fixed
- Connection cleanup: Properly closes sockets after connect() to prevent resource exhaustion
- Error handling: Distinguishes ECONNREFUSED (closed) from ETIMEDOUT (filtered)

### [v0.2.2] - 2024-11-22

#### Added
- **Sprint 2.2:** Service detection infrastructure
  - Banner grabbing: Connects to open ports and captures initial server response
  - Protocol detection: Identifies HTTP, SSH, FTP, SMTP, Telnet by banner keywords
  - Service database: 50 common services pre-configured (HTTP/80, SSH/22, FTP/21, etc.)
- `--service-detection` flag (`-sV`) for enabling service detection
- Service output: Prints service name and version (e.g., "Apache/2.4.52 (Ubuntu)")

#### Changed
- Service detection: Adds 500ms-2s overhead per open port (configurable timeout)
- Output format: JSON includes `service` and `version` fields

#### Fixed
- Timeout issues: Service detection timeout now independent of SYN scan timeout
- Binary data: Handles non-UTF-8 banners gracefully (converts to hex or "binary data")

### [v0.2.1] - 2024-11-18

#### Added
- Service detection placeholder (banner grabbing)
- OS fingerprinting stub (TCP/IP stack analysis)
- Protocol-specific payloads for UDP (DNS, SNMP)

#### Changed
- Modular architecture: Separated scan types into dedicated modules

#### Fixed
- Packet construction errors for stealth scans (FIN/NULL/Xmas)

### [v0.2.0] - 2024-11-14

**Phase 2 Start:** Protocols and Detection (Sprints 2.1-2.8)

#### Added
- Service detection framework (nmap-service-probes integration)
- OS fingerprinting framework (Nmap database)
- Connect scan (`-sT`)
- Stealth scans (FIN, NULL, Xmas)
- ACK scan for firewall detection
- UDP scanning with ICMP unreachable detection

#### Summary
Phase 2 added advanced scanning techniques: 5 scan types (SYN, Connect, FIN, NULL, Xmas, ACK, UDP), service detection (187 probes, 85-90% accuracy), OS fingerprinting (2,600+ signatures, 85% accuracy). Total: 306 tests, 5,200+ lines of documentation, support for complex network environments.

---

## Phase 1: Foundation - v0.1.x

Core scanning engine, networking infrastructure, and basic functionality. Sprints 1.1-1.5 (Aug-Oct 2024).

### [v0.1.5] - 2024-10-28

#### Added
- **Sprint 1.5:** Cross-platform support finalization
  - Windows support with Npcap (raw packet capture on Windows)
  - macOS support with ChmodBPF (BPF device access)
  - FreeBSD support (libpcap compatibility)
- Platform-specific installation guides
- Privilege management: `setcap` on Linux, BPF group on macOS, Administrator on Windows

#### Changed
- Build system: Conditional compilation (`#[cfg(target_os)]`) for platform-specific code
- CI/CD: Added macOS and Windows runners to GitHub Actions

#### Fixed
- Windows: Npcap initialization takes 90s in old versions (documented workaround)
- macOS: ChmodBPF permissions must be granted before first run
- FreeBSD: BPF device not available by default (requires kernel module load)

### [v0.1.4] - 2024-10-24

#### Added
- **Sprint 1.4:** Rate limiting and performance optimization
  - Token bucket algorithm for rate limiting (max_rate, burst_size)
  - Adaptive rate limiting (adjusts based on packet loss)
  - `-T0` to `-T5` timing templates (paranoid to insane)
  - `--max-rate` flag for explicit pps limit (default 100K pps)
- Rate limiter overhead: <5% CPU at 100K pps
- Performance: SYN scan achieves 50K-100K pps on modern hardware

#### Changed
- Default timing: T3 (Normal, 1K-10K pps) balances speed and accuracy
- Burst size: 100 packets (allows short bursts without triggering rate limit)

#### Fixed
- Packet loss at high rates: Adaptive rate limiting reduces pps when loss detected
- CPU pinning: Rate limiter respects NUMA topology for better performance

### [v0.1.3] - 2024-10-20

#### Added
- **Sprint 1.3:** Output formats and result storage
  - JSON output (`-oJ output.json`)
  - XML output (`-oX output.xml`, Nmap-compatible schema)
  - SQLite database storage (`--database scans.db`)
  - Database schema: scans (metadata), scan_results (ports)
- Stream-to-disk: Results written incrementally (prevents OOM on large scans)

#### Changed
- Output buffering: 10K results buffered before writing (balances I/O and memory)
- Database: WAL mode for concurrent reads during scan

#### Fixed
- JSON escaping: Banners with quotes/newlines properly escaped
- XML validation: Schema compliant with Nmap DTD

### [v0.1.2] - 2024-10-16

#### Added
- **Sprint 1.2:** IPv4 and IPv6 support
  - Dual-stack scanning (IPv4 and IPv6 in single scan)
  - ICMPv6 for IPv6 host discovery (Neighbor Discovery Protocol)
  - IPv6 address parsing (standard, compressed, IPv4-mapped formats)
- CIDR notation support (e.g., `192.168.1.0/24`, `2001:db8::/32`)

#### Changed
- IPv6 overhead: +15% compared to IPv4 (larger headers, NDP vs. ARP)

#### Fixed
- IPv6 fragmentation: Correctly handles IPv6 Extension Headers
- Dual-stack: Resolves hostnames to both IPv4 and IPv6 addresses

### [v0.1.1] - 2024-10-12

#### Added
- **Sprint 1.1:** Core SYN scanning
  - Half-open TCP SYN scan (`-sS`, default)
  - Port range specification (`-p 1-65535`, `-p-`, `--top-ports 100`)
  - Host discovery (ICMP Echo, TCP SYN/ACK ping, ARP)
- Port states: open, closed, filtered
- Async I/O with Tokio multi-threaded runtime (10K+ concurrent connections per thread)

#### Changed
- Performance: 50K+ pps SYN scanning on commodity hardware

#### Fixed
- Packet parsing: Handles fragmented responses correctly
- State tracking: Concurrent HashMap prevents race conditions

### [v0.1.0] - 2024-10-08

**Phase 1 Start:** Foundation - Core Scanning Engine (Sprints 1.1-1.5)

#### Added
- Basic SYN scan implementation
- Raw socket creation (requires root/capabilities)
- Packet construction with pnet crate (Ethernet, IPv4, TCP)
- Tokio async runtime for concurrency
- CLI argument parsing with clap

#### Summary
Phase 1 established the core scanning engine: SYN scanning (50K+ pps), IPv4/IPv6 dual-stack, rate limiting (Token Bucket), output formats (JSON, XML, SQLite), cross-platform support (Linux, macOS, Windows, FreeBSD). Total: 168 tests, 2,800+ lines of documentation.

---

## Pre-release - v0.0.x

Early development and prototyping (Jul-Aug 2024).

### [v0.0.3] - 2024-08-30

#### Added
- Async packet transmission with Tokio
- Basic port scanning logic
- CLI framework with clap

#### Changed
- Architecture: Switched from blocking I/O to async/await

#### Fixed
- Memory leaks in packet buffer allocation

### [v0.0.2] - 2024-08-15

#### Added
- Raw socket creation
- Basic TCP packet construction
- IP address parsing

#### Changed
- Packet library: Replaced custom implementation with pnet crate

#### Fixed
- Checksum calculation errors in TCP headers

### [v0.0.1] - 2024-08-01

**Initial Prototype**

#### Added
- Project scaffolding (Cargo workspace, crate structure)
- Basic TCP connect() scanning (no raw sockets)
- Single-threaded execution

#### Summary
Proof-of-concept demonstrating feasibility of network scanner in Rust. 50 tests, basic documentation.

---

## Version Numbering Scheme

ProRT-IP follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

**MAJOR.MINOR.PATCH** (e.g., v0.5.2)

- **MAJOR:** Incompatible API changes (currently 0 = pre-1.0 development)
- **MINOR:** New features in a backward-compatible manner (Phase completion)
- **PATCH:** Backward-compatible bug fixes (Sprint completion)

**Phase to Version Mapping:**
- Phase 1 (Foundation) → v0.1.x
- Phase 2 (Protocols) → v0.2.x
- Phase 3 (Quality) → v0.3.x
- Phase 4 (Performance) → v0.4.x
- Phase 5 (Advanced Features) → v0.5.x
- Phase 6 (TUI Interface) → v0.6.x (In Progress)
- Phase 7 (Distributed Scanning) → v0.7.x (Planned)
- Phase 8 (Production Hardening) → v1.0.0 (Release)

---

## Related Documentation

- **[Current Changelog](https://github.com/doublegate/ProRT-IP/blob/main/CHANGELOG.md)** - v0.4.0 and later
- **[Release Notes](https://github.com/doublegate/ProRT-IP/releases)** - Detailed release information
- **[Roadmap](../project-management/phases.md)** - Phase planning and sprint breakdown
- **[Sprint Tracking](../project-management/sprints.md)** - Sprint completion status
- **[Technical Specification](../reference/tech-spec-v2.md)** - Complete technical reference

---

## Archive Notes

**Last Phase 3 Version:** v0.3.9 (2025-01-28)
**Total Phase 1-3 Commits:** 342 commits
**Total Phase 1-3 Lines Added:** ~45,000 lines of code, ~12,000 lines of documentation
**Total Phase 1-3 Tests:** 489 tests (52% coverage)

**Archive Date:** 2025-02-01
**Archived By:** ProRT-IP Documentation Team
**Archive Location:** `/docs/src/appendices/changelog-archive.md`

---

## Version Information

- **Document Version:** 1.0.0
- **Last Updated:** 2025-11-15
- **ProRT-IP Version:** v0.5.2+
- **Phase:** Phase 6 Sprint 6.2 COMPLETE

---

## See Also

- [Glossary](glossary.md) - Comprehensive term definitions
- [References](references.md) - External resources and standards
- [Roadmap](../project-management/phases.md) - Phase planning
- [Release Process](../development/release-process.md) - How releases are created
