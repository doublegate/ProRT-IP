# Changelog

All notable changes to ProRT-IP WarScan will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
