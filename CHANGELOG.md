# Changelog

All notable changes to ProRT-IP WarScan will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - 2025-10-08

#### Enhancement Cycle 4: CLI & Scanner Integration (commit eec5169)

**Objective:** Integrate resource limits and interface detection modules into CLI and scanner workflows with RustScan-inspired patterns.

**CLI Enhancements** (`crates/prtip-cli/src/args.rs` - COMPLETE ✅):
- **New command-line flags**:
  * `--batch-size` / `-b SIZE` - Manual batch size control (overrides auto-calculation)
  * `--ulimit LIMIT` - Adjust file descriptor limits (RustScan pattern, Unix only)
  * `--interface-list` - Display available network interfaces with details and exit
  * Validation: batch size 1-100,000, ulimit >= 100
- **Argument validation**:
  * Zero batch size rejection
  * Excessive batch size warnings
  * Ulimit minimum enforcement
- **7 new CLI tests** - all passing (batch size, ulimit, interface list flags)

**Main CLI Integration** (`crates/prtip-cli/src/main.rs` - COMPLETE ✅):
- **Ulimit adjustment on startup**:
  * Calls `adjust_and_get_limit()` before scanner initialization
  * Success: info log with new limit
  * Failure: warning with manual command suggestion
- **Batch size calculation and warnings**:
  * Automatic batch size recommendation via `get_recommended_batch_size()`
  * Warning when requested batch exceeds safe limits
  * Auto-adjustment to safe values with user notification
  * Helpful error messages: "Use '-b X' or increase ulimit with '--ulimit Y'"
- **Interface list handler** (`handle_interface_list()` - 62 lines):
  * Formatted output with colored status (UP/DOWN)
  * Display: name, MAC, MTU, IPv4/IPv6 addresses
  * Loopback interface indication
  * Total interface count summary

**Scanner Integration** (`crates/prtip-scanner/src/connection_pool.rs` - COMPLETE ✅):
- **Ulimit-aware connection pooling**:
  * `check_ulimit_and_adjust()` private method (26 lines)
  * Automatic concurrency reduction when limits low
  * Warning messages with actionable fix commands
  * Graceful degradation on limit detection failure
- **Integration with resource limits module**:
  * Uses `get_recommended_batch_size()` for safety checks
  * Prevents "too many open files" errors
  * RustScan-inspired error messages
- **Enhanced documentation**:
  * Updated docstrings with ulimit awareness
  * Examples of automatic limit handling

**Configuration Updates** (`crates/prtip-core/src/config.rs` - COMPLETE ✅):
- **New PerformanceConfig fields**:
  * `batch_size: Option<usize>` - Manual batch size override
  * `requested_ulimit: Option<u64>` - User-requested ulimit value
  * Both fields use `#[serde(default)]` for backward compatibility
- **Default implementation updated**:
  * New fields initialize to None (auto-calculate)
- **All test configs updated** - 4 locations fixed

**Test Updates** (4 files modified, +7 tests):
- `crates/prtip-cli/src/args.rs`: +7 tests for new CLI arguments
- `crates/prtip-cli/src/output.rs`: PerformanceConfig struct initialization
- `crates/prtip-scanner/tests/integration_scanner.rs`: Test config updates
- `crates/prtip-scanner/src/scheduler.rs`: Test helper updates
- `crates/prtip-scanner/src/concurrent_scanner.rs`: Test config updates

**Summary Statistics**:
- **Files Modified:** 8 (args.rs, main.rs, config.rs, connection_pool.rs, + 4 test files)
- **Lines Added:** ~200 (CLI: 62, connection_pool: 26, config: 4, tests: 60, main: 50+)
- **Tests:** 345 → 352 (+7 new CLI argument tests)
- **Pass Rate:** 100%
- **Clippy:** Clean (0 warnings)
- **Code Quality:** All formatted with cargo fmt

**Reference Inspirations**:
- RustScan `src/main.rs` (lines 225-287): ulimit adjustment and batch size inference
- RustScan `src/scanner/mod.rs` (line 86): batch size usage in FuturesUnordered
- naabu `pkg/runner/options.go`: CLI flag patterns for interface selection
- naabu `pkg/routing/router.go`: Interface detection and routing logic

**Integration Flow**:
1. CLI parses arguments including `--batch-size`, `--ulimit`, `--interface-list`
2. `--interface-list`: enumerate and display interfaces, exit early
3. `--ulimit`: attempt to adjust system limit before scanner creation
4. Config creation: pass batch_size and requested_ulimit to PerformanceConfig
5. Batch size validation: check against ulimit via `get_recommended_batch_size()`
6. Auto-adjustment: reduce batch size if exceeds safe limit
7. Warning messages: inform user of adjustments with fix commands
8. Connection pool: validates concurrency against ulimit on creation
9. Scanner: uses adjusted batch size for optimal performance

**User-Facing Improvements**:
- **Better error messages**: "Run 'ulimit -n 10000' to increase" instead of cryptic errors
- **Automatic safety**: System prevents resource exhaustion without user intervention
- **Visibility**: `--interface-list` shows network topology at a glance
- **Manual control**: Power users can override with `-b` and `--ulimit` flags
- **Helpful warnings**: Clear guidance when settings are constrained by limits

**Technical Highlights**:
- MSRV compatibility maintained (Rust 1.70+)
- Cross-platform support (Unix production, Windows stubs)
- Zero breaking changes to existing API
- Follows ProRT-IP architectural patterns
- Clean separation: CLI → Config → Scanner

---

#### Enhancement Cycle 3: Resource Limits & Interface Detection (commit 38b4f3e)

**Objective:** Implement production-critical resource management and network interface detection from RustScan/Naabu reference codebases.

**Resource Limits Module** (`crates/prtip-core/resource_limits.rs` - 363 lines, COMPLETE ✅):
- **Cross-platform ulimit detection**:
  * Uses `rlimit` crate (0.10.2) for Unix systems
  * Graceful Windows stub (conservative 2048 default)
  * Get/set file descriptor limits (RLIMIT_NOFILE)
  * MSRV compatible with Rust 1.70+
- **Intelligent batch size calculation** (RustScan pattern):
  * `calculate_optimal_batch_size()` - adapts to system limits
  * Low limits (<3000): use half of ulimit
  * Moderate limits (3000-8000): use ulimit - 100
  * High limits: use desired batch size
  * Prevents "too many open files" errors
- **Convenience APIs**:
  * `adjust_and_get_limit(requested_limit)` - set and return current limit
  * `get_recommended_batch_size(desired, requested_limit)` - one-shot calculation
  * Proper error handling with `ResourceLimitError`
- **11 comprehensive tests** - all passing

**Interface Detection Module** (`crates/prtip-network/interface.rs` - 406 lines, COMPLETE ✅):
- **Network interface enumeration** (naabu pattern):
  * Uses `pnet::datalink` for cross-platform support
  * Extract IPv4/IPv6 addresses per interface
  * MAC address, MTU, up/down status detection
  * Filter link-local IPv6 (fe80::/10) for routing
- **Smart routing logic**:
  * `find_interface_for_target(ip)` - select best interface
  * Prefer non-loopback interfaces
  * Match IPv4/IPv6 address families
  * Fallback to loopback if needed
- **Source IP selection**:
  * `get_source_ip_for_target(target)` - automatic source IP
  * `find_interface_by_name(name)` - manual interface selection
  * Proper address family matching (IPv4 to IPv4, IPv6 to IPv6)
- **13 comprehensive tests** - all passing (Unix-only tests)

**Dependencies Added:**
- `rlimit = "0.10.2"` - cross-platform resource limit management

**Test Coverage:**
- Total tests: **345 passing** (was 317 baseline, +28 new tests)
  * prtip-core: 66 tests (+11 for resource_limits)
  * prtip-network: 35 tests (+13 for interface)
  * All doc tests passing (+4 new doc tests)
- Code quality: 100% clippy clean, formatted

**Reference Code Analysis:**
- `/home/parobek/Code/ProRT-IP/code_ref/RustScan/src/main.rs` - ulimit patterns (lines 225-287)
- `/home/parobek/Code/ProRT-IP/code_ref/naabu/pkg/routing/router.go` - interface routing
- `/home/parobek/Code/ProRT-IP/code_ref/naabu/pkg/runner/banners.go` - interface enumeration

---

#### Enhancement Cycle 2: Blackrock Completion & Port Filtering (commit f5be9c4)

**Objective:** Complete Blackrock algorithm with Masscan's proper domain splitting and implement comprehensive port exclusion/filtering inspired by RustScan/Naabu.

**Blackrock Algorithm - Full Masscan Implementation** (`crates/prtip-core/crypto.rs` - COMPLETE ✅):
- **Fixed domain splitting with (a × b) algorithm**:
  * Proper domain factorization: `a ≈ sqrt(range) - 2`, `b ≈ sqrt(range) + 3`
  * Ensures `a * b > range` for all input ranges
  * Hardcoded small-range values (0-8) for better statistical properties
  * Cycle-walking for format-preserving encryption
- **Full encrypt/decrypt implementation**:
  * Alternating modulo operations (odd rounds: mod a, even rounds: mod b)
  * Round-dependent F() function with seed mixing
  * Proper inverse operations for unshuffle
- **All tests passing**: 11/11 tests (was 9/11 in Cycle 1)
  * Bijectivity verified for ranges: 256, 1000, 1024
  * Power-of-2 and non-power-of-2 ranges
  * Deterministic shuffling validated
  * Unshuffle correctness confirmed

**Port Filtering System** (`crates/prtip-core/types.rs` - 167 lines, COMPLETE ✅):
- **Dual-mode filtering** (RustScan/Naabu pattern):
  * Whitelist mode: only allow specified ports
  * Blacklist mode: exclude specified ports
  * O(1) lookup performance via HashSet
- **Flexible port specification**:
  * Single ports: "80"
  * Ranges: "8000-8090"
  * Mixed: "80,443,8000-8090"
  * Reuses existing PortRange parser
- **API**:
  * `PortFilter::include(&["22", "80", "443"])` - whitelist
  * `PortFilter::exclude(&["80", "443"])` - blacklist
  * `filter.allows(port)` - O(1) check
  * `filter.filter_ports(vec)` - bulk filtering
- **10 comprehensive tests** - all passing

**Test Coverage:**
- Total tests: 131 passing (was 121 in Cycle 1, +10)
  * prtip-core: 55 unit tests (+10 port filter tests)
  * prtip-network: 29 tests
  * prtip-scanner: 93 tests
  * prtip-cli: 49 tests
  * integration: 14 tests
  * doctests: 37 tests
- Code quality: 100% clean (cargo fmt + clippy -D warnings)

#### Enhancement Cycle 1: Reference Codebase Integration (commit 5782aed)

**Objective:** Systematically incorporate high-value improvements from Masscan, RustScan, Naabu, and other reference implementations.

**Cryptographic Utilities** (`crates/prtip-core/crypto.rs` - 584 lines):
- **SipHash-2-4 Implementation** (COMPLETE ✅):
  * Fast cryptographic hash optimized for short inputs
  * Used for stateless sequence number generation
  * Passed all test vectors from SipHash specification
  * ~1 cycle/byte performance on 64-bit architectures
  * 9 comprehensive tests including avalanche effect validation

- **Blackrock Shuffling Algorithm** (PARTIAL - needs refinement for Phase 2):
  * Feistel cipher for bijective IP address randomization
  * Enables stateless scanning without tracking scanned IPs
  * Power-of-2 domain splitting implemented
  * Cycle-walking for format-preserving encryption
  * Note: Full Masscan algorithm uses (a * b > range) domain splitting
  * 7 tests passing (deterministic, different seeds, unshuffle, etc.)
  * 2 tests need refinement: full bijectivity for all ranges

**Concurrent Scanner** (`crates/prtip-scanner/concurrent_scanner.rs` - 380 lines):
- **FuturesUnordered Pattern** (COMPLETE ✅ - RustScan technique):
  * High-performance concurrent scanning with streaming results
  * Fixed-size task pool with automatic work stealing
  * Constant memory usage regardless of target count
  * Intelligent error handling with retry logic
  * "Too many open files" panic with helpful error message
  * Connection refused detection (closed ports)
  * Timeout handling (filtered ports)
  * 6 comprehensive tests all passing

**Test Coverage:**
- Total tests: 121 passing (49 core + 29 network + 93 scanner)
- Blackrock refinement: 2 tests need Phase 2 work
- SipHash: 100% passing (9/9 tests)
- Concurrent scanner: 100% passing (6/6 tests)
- All code passes `cargo fmt` and `cargo clippy -D warnings`

**Code Quality:**
- Comprehensive inline documentation with examples
- Doc comments for all public APIs
- Error handling with detailed messages
- No clippy warnings
- Consistent formatting

**Reference Inspiration:**
- SipHash: Masscan crypto-siphash24.c
- Blackrock: Masscan crypto-blackrock.c (partial adaptation)
- FuturesUnordered: RustScan src/scanner/mod.rs
- Error handling patterns: RustScan error recovery
- Port state determination: Naabu pkg/port/port.go

**Performance Improvements:**
- Concurrent scanner maintains constant `parallelism` concurrent tasks
- SipHash provides O(1) sequence number generation
- Blackrock enables stateless IP randomization (when fully implemented)
- FuturesUnordered provides optimal work distribution via futures runtime

### Added - 2025-10-08

#### Phase 2: Advanced Scanning (COMPLETE ✅ - commit 296838a)

**Total Implementation:** 2,646 lines added across 16 files

**Packet Building Infrastructure** (`crates/prtip-network/`):
- **packet_builder.rs** (790 lines): Complete TCP/UDP packet construction
  - `TcpPacketBuilder`: TCP header construction with all flags (SYN, FIN, ACK, RST, PSH, URG)
  - `UdpPacketBuilder`: UDP header construction with checksum calculation
  - IPv4 header construction with TTL, protocol, fragmentation support
  - Ethernet frame building for Layer 2 transmission
  - Checksum calculation including IPv4 pseudo-header for TCP/UDP
  - TCP options support: MSS, Window Scale, SACK, Timestamp, NOP, EOL
  - Comprehensive unit tests for all packet types and options

- **protocol_payloads.rs** (199 lines): Protocol-specific UDP payloads
  - DNS query (port 53): Standard query for root domain
  - NTP request (port 123): NTPv3 client request (48 bytes)
  - NetBIOS name query (port 137): Query for *<00><00>
  - SNMP GetRequest (port 161): SNMPv1 with community "public"
  - Sun RPC NULL call (port 111): Portmapper query
  - IKE handshake (port 500): IPSec Main Mode SA payload
  - SSDP discover (port 1900): UPnP M-SEARCH discovery
  - mDNS query (port 5353): Multicast DNS for _services._dns-sd._udp.local
  - Full unit tests for all protocol payloads

**TCP SYN Scanner** (`crates/prtip-scanner/syn_scanner.rs` - 437 lines):
- Half-open scanning with SYN packets (stealth technique)
- Connection state tracking with HashMap
- Sequence number generation and validation
- Response interpretation:
  * SYN/ACK → Open port (send RST to complete stealth)
  * RST → Closed port
  * No response → Filtered port (timeout)
- Concurrent scanning with semaphore-based parallelism
- Retry mechanism with exponential backoff
- Integration with timing templates for rate control
- Comprehensive tests including state tracking and response handling

**UDP Scanner** (`crates/prtip-scanner/udp_scanner.rs` - 258 lines):
- Protocol-specific payload selection (8 protocols)
- ICMP port unreachable interpretation for closed ports
- Open|Filtered state handling (UDP characteristic)
- Timeout-based filtering detection
- Integration with protocol_payloads module
- Concurrent scanning with rate limiting
- Comprehensive tests for payload selection and ICMP handling

**Stealth Scanner** (`crates/prtip-scanner/stealth_scanner.rs` - 388 lines):
- **FIN scan**: Single FIN flag (RFC 793 exploit)
- **NULL scan**: No flags set (RFC 793 exploit)
- **Xmas scan**: FIN + PSH + URG flags (packet "lit up")
- **ACK scan**: ACK flag for firewall state detection
- Response interpretation:
  * No response → Open|Filtered (FIN/NULL/Xmas)
  * RST → Closed (FIN/NULL/Xmas)
  * RST → Unfiltered (ACK scan)
  * No response → Filtered (ACK scan)
- Platform limitations documented (Windows, Cisco devices send RST regardless)
- Comprehensive tests for all stealth scan types

**Timing Templates** (`crates/prtip-scanner/timing.rs` - 441 lines):
- **T0 (Paranoid)**: 5-minute probe delays, serial scanning, IDS evasion
- **T1 (Sneaky)**: 15-second delays, serial scanning
- **T2 (Polite)**: 0.4-second delays, bandwidth reduction
- **T3 (Normal)**: Default balanced behavior (1-second timeout)
- **T4 (Aggressive)**: Fast/reliable networks (200ms timeout, parallel)
- **T5 (Insane)**: Maximum speed (50ms timeout, sacrifices accuracy)
- RTT (Round-Trip Time) estimation with sliding window
- AIMD (Additive Increase Multiplicative Decrease) congestion control
- Adaptive timeout calculation based on measured RTT
- Probe timing with configurable delays
- Comprehensive tests for all timing templates and RTT estimation

### Added - 2025-10-08

#### Performance Enhancements (Reference Implementation-Inspired)

**Adaptive Rate Limiter** (Masscan-inspired):
- New `AdaptiveRateLimiterV2` with dynamic batch sizing
- Circular buffer tracking (256 buckets) for recent packet rates
- Adaptive batch size: increases by 0.5% when below target, decreases by 0.1% when above
- Handles system suspend/resume gracefully (avoids burst after pause)
- Optimized for high-speed scanning (>100K pps with reduced syscall overhead)
- Comprehensive tests including rate enforcement and batch adaptation

**Connection Pool** (RustScan-inspired):
- New `ConnectionPool` using `FuturesUnordered` for efficient concurrent scanning
- Constant memory usage with bounded concurrency
- Better CPU utilization through work-stealing scheduler
- Configurable timeout and retry logic
- Performance benefits over simple semaphore approach

**Dependencies**:
- Added `futures = "0.3"` for FuturesUnordered support

**Code Quality**:
- Fixed clippy warnings: unnecessary lazy evaluations in packet_builder
- Added `is_empty()` method to TcpOption enum (clippy requirement)
- Fixed unused import warnings
- All 278 tests passing (49 core + 29 network + 114 scanner + 49 cli + 37 integration)

**Dependencies Added**:
- `pnet_packet` for packet manipulation
- `rand` for randomization
- `futures` for FuturesUnordered support

**Configuration Updates** (`crates/prtip-core/`):
- Added `ScanType` enum variants: Syn, Fin, Null, Xmas, Ack, Udp
- Added timing template configuration options
- Added scan delay and retry configuration

**Summary Statistics**:
- **Phase 2 Implementation:** 2,646 lines (6 core scanning modules)
- **Performance Enhancements:** 905 lines (2 optimization modules)
- **Total Added:** 3,551 lines of production code
- **Test Coverage:** 278 tests across all modules
- **Scan Types:** 7 (Connect, SYN, UDP, FIN, NULL, Xmas, ACK)
- **Protocol Payloads:** 8 (DNS, NTP, NetBIOS, SNMP, RPC, IKE, SSDP, mDNS)
- **Timing Templates:** 6 (T0-T5)
- **Performance Modules:** 2 (adaptive rate limiter, connection pool)

### Changed - 2025-10-08

**Reference Code Analysis**:
- Analyzed 7+ reference implementations (Masscan, RustScan, Naabu, Nmap, etc.)
- Identified 3,271 source files across reference codebases
- Extracted key optimization patterns:
  * Masscan's adaptive throttler with circular buffer
  * RustScan's FuturesUnordered concurrent scanning pattern
  * SipHash-based randomization for stateless scanning
  * Batch processing to reduce per-packet overhead

**Documentation**:
- Enhanced adaptive rate limiter with extensive inline documentation
- Added connection pool module with performance rationale
- Updated module exports in prtip-scanner lib.rs

### Fixed - 2025-10-07

#### Security
- **Upgraded sqlx from 0.7.4 to 0.8.6** - Fixes RUSTSEC-2024-0363 (Binary Protocol Misinterpretation)
- Configured governor rate limiter with `burst=1` for strict linear rate limiting
- Fixed 7 test failures after sqlx upgrade:
  - Rate limiter tests: Burst capacity configuration issue
  - Discovery tests: Network-agnostic test improvements

#### Test Suite
- All 215 tests passing across workspace
- Updated discovery tests to handle varying network configurations
- Made tests more robust for different routing setups

### Added - 2025-10-07

#### Phase 1: Core Infrastructure (COMPLETE ✅)

**prtip-core crate**:
- Core types: `ScanTarget`, `ScanResult`, `PortState`, `PortRange`
- Configuration: `Config`, `ScanConfig`, `NetworkConfig`, `OutputConfig`, `PerformanceConfig`
- Enums: `ScanType`, `TimingTemplate`, `OutputFormat`
- CIDR notation parsing with `ipnetwork` crate
- Port range parsing (single: `80`, list: `80,443`, range: `1-1000`)
- 49 unit tests with comprehensive coverage

**prtip-network crate**:
- Cross-platform packet capture abstraction
- Platform-specific implementations (Linux/Windows/macOS)
- Privilege checking: `check_privileges()`, `drop_privileges()`
- Capability detection (Linux CAP_NET_RAW)
- 29 unit tests

**prtip-scanner crate**:
- TCP connect scanner with full 3-way handshake
- Rate limiting with governor (token bucket algorithm)
- Host discovery engine (TCP SYN ping)
- Scan scheduler with async orchestration
- SQLite result storage with indexing
- Concurrent scanning with semaphore-based parallelism
- Retry mechanism with exponential backoff
- 62 unit tests + 14 integration tests

**prtip-cli crate**:
- Complete CLI with clap argument parsing
- Output formatters: Text (colorized), JSON, XML
- Progress reporting with colored terminal output
- Database integration for result storage
- Scan summary with statistics
- 49 tests including args validation and output formatting

### Changed - 2025-10-07

#### Dependencies
- **sqlx**: 0.7.4 → 0.8.6 (security fix)
- **Cargo.lock**: Updated with 322 dependencies
- **Rate limiter**: Configured with strict burst=1 for predictable timing

### Added - 2025-10-07

#### Root-Level Documentation
- **CONTRIBUTING.md** (10 KB): Comprehensive contribution guidelines
  - Code of conduct reference
  - Development setup and workflow
  - Coding standards (rustfmt, clippy)
  - Testing requirements (>80% coverage)
  - Security guidelines and best practices
  - Pull request process and checklist
  - Commit message conventions (Conventional Commits)
  - Branch naming conventions
  - Code review criteria
  - 11 detailed sections with examples

- **SECURITY.md** (9 KB): Security policy and vulnerability reporting
  - Supported versions table
  - Private vulnerability reporting process
  - Security disclosure timeline (coordinated 14-30 day)
  - Responsible use guidelines (authorized testing only)
  - Operational security best practices
  - Network safety recommendations
  - Implementation security reference
  - Security hardening recommendations (Docker, AppArmor, capabilities)
  - Compliance and certification roadmap
  - Legal disclaimer about authorized use

- **SUPPORT.md** (9 KB): Support resources and community help
  - Complete documentation index with descriptions
  - Quick start guides (users, developers, security researchers)
  - GitHub Discussions and Issues guidance
  - Bug report and feature request templates
  - FAQ cross-reference
  - Response time expectations
  - Commercial support plans (future)
  - External resource links

- **AUTHORS.md** (8 KB): Contributors and acknowledgments
  - Contribution recognition policy
  - Acknowledgments to Nmap, Masscan, RustScan, ZMap
  - Rust ecosystem contributors (Tokio, pnet, etherparse, clap, etc.)
  - Individual recognition (Fyodor Lyon, Robert Graham, Rust team)
  - Contribution categories and levels
  - Full dependency credits table
  - License agreement statement

- **ROADMAP.md** (8 KB): High-level development roadmap
  - Project vision and goals
  - Current status (Genesis phase complete)
  - 8-phase overview with timelines
  - Performance targets table
  - Feature comparison vs Nmap/Masscan/RustScan
  - Technology stack summary
  - Release strategy (0.x → 1.0 → 2.0+)
  - Community goals (short/mid/long-term)
  - Risk management
  - Success metrics
  - Timeline summary

#### Enhanced Root README
- **README.md** updated with comprehensive sections:
  - Table of Contents with all major sections
  - Root documentation table (6 files)
  - Technical documentation table (12 files in docs/)
  - Quick Start guides (users, developers, security researchers)
  - Enhanced roadmap overview with phase table
  - Expanded Contributing section with guidelines
  - New Support section with resources
  - New Security section with vulnerability reporting
  - New Authors & Acknowledgments section
  - Updated project statistics (478 KB total docs)
  - Links section with GitHub URLs
  - Current status badges and last updated date

### Changed - 2025-10-07

#### Repository Metadata
- **Total documentation**: Now 478 KB (237 KB docs/ + 241 KB ref-docs/)
- **Root documents**: 6 files (ROADMAP, CONTRIBUTING, SECURITY, SUPPORT, AUTHORS, CHANGELOG)
- **GitHub repository**: Complete with all standard community health files
- **Repository structure**: Professional open-source project layout

---

### Phase 1: Core Infrastructure (Target: Weeks 1-3)
- Workspace setup and crate organization
- Packet capture abstraction layer (Linux/Windows/macOS)
- Basic TCP connect scanning
- CLI argument parsing with clap
- Privilege management and capability detection
- Result storage with SQLite

### Phase 2: Advanced Scanning (Target: Weeks 4-6)
- TCP SYN scanning with raw sockets
- UDP scanning with protocol-specific probes
- Stealth scan variants (FIN, NULL, Xmas, ACK)
- Timing templates (T0-T5)
- Rate limiting with token bucket algorithm

### Phase 3: Detection Systems (Target: Weeks 7-10)
- OS fingerprinting (16-probe sequence)
- Service version detection engine
- Banner grabbing with SSL/TLS support
- nmap-service-probes database parser

### Phase 4: Performance Optimization (Target: Weeks 11-13)
- Lock-free data structures
- Stateless scanning mode (1M+ pps target)
- NUMA-aware thread placement
- Batched syscalls (sendmmsg/recvmmsg)

### Phase 5: Advanced Features (Target: Weeks 14-16)
- Idle (zombie) scanning
- Packet fragmentation and decoy scanning
- Lua plugin system with mlua
- Audit logging and error recovery

### Phase 6-7: UI and Release (Target: Weeks 17-20)
- TUI interface with real-time progress
- Documentation completion
- v1.0 release preparation

---

## [0.0.1] - 2025-10-07

### Added - Genesis Phase

#### Documentation
- **Comprehensive documentation suite** (237 KB across 12 documents)
  - `00-ARCHITECTURE.md` (23 KB): System architecture and design patterns
  - `01-ROADMAP.md` (18 KB): 8 phases, 20 weeks, 122+ tracked tasks
  - `02-TECHNICAL-SPECS.md` (22 KB): Protocol specifications and packet formats
  - `03-DEV-SETUP.md` (14 KB): Development environment setup
  - `04-IMPLEMENTATION-GUIDE.md` (24 KB): Code structure and 500+ lines of examples
  - `05-API-REFERENCE.md` (20 KB): 50+ documented APIs
  - `06-TESTING.md` (17 KB): Testing strategy with 5 test levels
  - `07-PERFORMANCE.md` (17 KB): Performance benchmarks and optimization techniques
  - `08-SECURITY.md` (20 KB): Security implementation and audit checklist
  - `09-FAQ.md` (12 KB): 30+ FAQs and troubleshooting
  - `10-PROJECT-STATUS.md` (19 KB): Task tracking with checkboxes
  - `docs/README.md` (14 KB): Documentation navigation guide
  - `docs/00-INDEX.md`: Complete documentation index

#### Repository Setup
- **Git repository initialized** with main branch
- **GitHub repository created**: https://github.com/doublegate/ProRT-IP
- **Project README** with badges, features, and build instructions
- **CLAUDE.md**: Project memory for Claude Code instances
- **CLAUDE.local.md**: Local development session tracking
- **CHANGELOG.md**: This changelog following Keep a Changelog format
- **.gitignore**: Comprehensive ignore rules for Rust projects

#### Reference Documentation
- `ref-docs/ProRT-IP_Overview.md`: High-level project vision
- `ref-docs/ProRT-IP_WarScan_Technical_Specification.md` (190 KB): Complete technical details
- `ref-docs/ProRT-IP_WarScan_Technical_Specification-v2.md` (36 KB): Condensed guide

#### Project Planning
- **8-phase development roadmap** (20 weeks total)
- **122+ tracked implementation tasks** across 14 sprints
- **6 major milestones** with success criteria
- **Performance targets**: 1M+ pps stateless, 50K+ pps stateful
- **Coverage goals**: >80% overall, >90% core modules

#### Architecture Decisions
- **Hybrid stateless/stateful architecture** for speed and depth
- **Tokio async runtime** with multi-threaded work-stealing scheduler
- **Cross-platform packet capture** abstraction (Linux/Windows/macOS)
- **Lock-free coordination** for high-performance scanning
- **Privilege dropping** pattern for security
- **Plugin system** with Lua scripting (planned Phase 5)

#### Security Framework
- **50+ item security audit checklist**
- Input validation patterns for IP/CIDR/ports
- Privilege management patterns (capabilities, setuid)
- DoS prevention strategies (rate limiting, resource bounds)
- Packet parsing safety guidelines

#### Testing Infrastructure
- Unit test strategy (>90% coverage target for core)
- Integration test approach with Docker test networks
- System test scenarios for end-to-end validation
- Performance test baselines with Criterion
- Fuzz testing strategy for input validation

### Repository Statistics
- **Total Documentation**: 478 KB (237 KB docs + 241 KB ref-docs)
- **Files Tracked**: 19 files
- **Lines of Documentation**: 16,509 insertions
- **Code Examples**: 500+ lines in implementation guide
- **API Documentation**: 50+ documented interfaces
- **Tracked Tasks**: 122+ implementation tasks

---

## Version History Legend

### Types of Changes
- `Added` - New features
- `Changed` - Changes in existing functionality
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Fixed` - Bug fixes
- `Security` - Vulnerability fixes

### Version Numbering
- **Major** (X.0.0): Incompatible API changes
- **Minor** (0.X.0): Backwards-compatible functionality
- **Patch** (0.0.X): Backwards-compatible bug fixes

---

**Current Status**: Documentation Complete | Implementation Starting Soon

For detailed project status, see [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md)

[Unreleased]: https://github.com/doublegate/ProRT-IP/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/doublegate/ProRT-IP/releases/tag/v0.0.1
