# Changelog

All notable changes to ProRT-IP WarScan will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- All 229+ tests passing (87 scanner + integration tests)

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
