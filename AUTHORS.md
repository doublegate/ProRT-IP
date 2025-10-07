# Authors and Acknowledgments

This document recognizes the contributors to ProRT-IP WarScan and acknowledges the projects and communities that have inspired and supported this work.

## Project Maintainers

### Core Development Team

**Lead Developer**
- **Role**: Project architecture, core implementation, security design
- **Contact**: Available via GitHub repository

**Contributors**
- Future contributors will be listed here

## Contributors

We welcome contributions from the community! Contributors will be recognized here based on their contributions.

### How to Be Listed

To have your contribution recognized:
1. Submit pull requests following [CONTRIBUTING.md](CONTRIBUTING.md)
2. Contribute documentation improvements
3. Report and help fix bugs
4. Assist other users in discussions
5. Participate in code reviews

Contributions of all sizes are valued and recognized.

### Contribution Categories

**Major Contributors** (10+ significant PRs or equivalent impact):
- Coming soon!

**Code Contributors** (merged pull requests):
- Coming soon!

**Documentation Contributors** (doc improvements, translations):
- Coming soon!

**Community Contributors** (issue reports, discussions, support):
- Coming soon!

**Security Researchers** (responsible vulnerability disclosure):
- Coming soon!

## Acknowledgments

ProRT-IP WarScan stands on the shoulders of giants. We acknowledge and thank the following projects and communities:

### Inspirational Projects

#### Nmap (Network Mapper)
- **Project**: https://nmap.org/
- **License**: Nmap Public Source License (NPSL) / GPLv2
- **Contribution**: Pioneer of network scanning, service detection, and OS fingerprinting
- **Recognition**: ProRT-IP's service detection and OS fingerprinting strategies are inspired by Nmap's proven methodologies. We use compatible probe database formats (nmap-service-probes, nmap-os-db) to leverage decades of collective security research.
- **Creator**: Gordon "Fyodor" Lyon and the Nmap development team

#### Masscan
- **Project**: https://github.com/robertdavidgraham/masscan
- **License**: AGPL-3.0
- **Contribution**: Demonstrated that stateless scanning can achieve 10M+ packets/second
- **Recognition**: ProRT-IP's stateless scanning architecture, including SYN cookie techniques and randomized packet ordering, is directly inspired by Masscan's innovations.
- **Creator**: Robert Graham

#### RustScan
- **Project**: https://github.com/RustScan/RustScan
- **License**: GPL-3.0
- **Contribution**: Proved viability of Rust for high-performance network scanning
- **Recognition**: RustScan's hybrid approach (fast port discovery → pipe to Nmap) validated the two-phase scanning model that ProRT-IP adopts. Their success in the Rust security tools ecosystem paved the way for projects like ours.
- **Creators**: RustScan community

#### ZMap
- **Project**: https://zmap.io/
- **License**: Apache 2.0
- **Contribution**: Internet-scale scanning capabilities and research
- **Recognition**: ZMap's academic research on scanning performance, hit rates, and network impact informed ProRT-IP's design decisions. Their publications on stateless scanning are foundational to our approach.
- **Creators**: University of Michigan research team

### Rust Ecosystem

ProRT-IP WarScan is built on excellent Rust libraries and frameworks:

#### Tokio
- **Project**: https://tokio.rs/
- **License**: MIT
- **Contribution**: Async runtime with work-stealing scheduler
- **Recognition**: Tokio's performance and ergonomics make high-speed async packet processing practical in Rust. Their scheduler design directly enables ProRT-IP's concurrency model.
- **Maintainers**: Tokio contributors and Carl Lerche

#### pnet (Packet Network)
- **Project**: https://github.com/libpnet/libpnet
- **License**: MIT/Apache-2.0
- **Contribution**: Cross-platform packet manipulation
- **Recognition**: pnet provides the foundation for ProRT-IP's packet crafting and parsing, abstracting platform differences (AF_PACKET, Npcap, BPF).
- **Maintainers**: pnet contributors

#### etherparse
- **Project**: https://github.com/JulianSchmid/etherparse
- **License**: BSD-3-Clause
- **Contribution**: Zero-allocation packet parsing
- **Recognition**: etherparse's safe, efficient parsing is crucial for ProRT-IP's performance at high packet rates.
- **Creator**: Julian Schmid

#### crossbeam
- **Project**: https://github.com/crossbeam-rs/crossbeam
- **License**: MIT/Apache-2.0
- **Contribution**: Lock-free data structures and concurrent algorithms
- **Recognition**: Crossbeam's channels and atomic utilities enable ProRT-IP's lock-free result collection critical for 1M+ pps performance.
- **Maintainers**: Crossbeam contributors

#### clap (Command Line Argument Parser)
- **Project**: https://github.com/clap-rs/clap
- **License**: MIT/Apache-2.0
- **Contribution**: Ergonomic CLI with derive macros
- **Recognition**: clap makes ProRT-IP's command-line interface powerful and user-friendly, following Unix conventions.
- **Maintainers**: clap-rs team

#### serde
- **Project**: https://serde.rs/
- **License**: MIT/Apache-2.0
- **Contribution**: Serialization framework
- **Recognition**: serde's zero-cost abstractions enable ProRT-IP's multiple output formats (JSON, XML, TOML) without performance penalty.
- **Creators**: David Tolnay and Erick Tryzelaar

### Other Notable Libraries

- **sqlx**: Async SQL with compile-time query checking
- **tracing**: Structured, composable logging
- **governor**: Token bucket rate limiting
- **mlua**: Lua scripting integration (planned)
- **openssl**: SSL/TLS support for encrypted service detection
- **ring**: Cryptographic operations
- **pcap**: Packet capture library bindings

### Development Tools

- **cargo**: Rust package manager and build system
- **rustfmt**: Code formatting
- **clippy**: Linting and best practices
- **criterion**: Benchmarking framework
- **tarpaulin**: Code coverage analysis
- **cargo-audit**: Security vulnerability scanning
- **cargo-fuzz**: Fuzzing infrastructure

### Security Community

We acknowledge the broader security research community:

- **IETF**: Protocol specifications (RFC 793, 791, 792, etc.)
- **MITRE**: CVE database and security frameworks
- **OWASP**: Web application security guidance
- **SANS Institute**: Security training and research
- **Academic researchers**: Continuous security improvements

### Documentation and Design

- **Markdown**: Documentation format
- **GitHub**: Repository hosting and collaboration
- **CommonMark**: Markdown specification
- **The Rust Book**: Rust learning resources
- **Shields.io**: README badges

## Special Thanks

### Individual Acknowledgments

- **Gordon "Fyodor" Lyon**: For creating Nmap and defining modern network scanning
- **Robert Graham**: For pushing the boundaries of scanning performance with Masscan
- **The Rust Core Team**: For creating a language that makes safe, fast systems programming accessible
- **All open-source contributors**: For building the ecosystem that makes projects like ProRT-IP possible

### Community Support

- **Rust Community**: For welcoming new projects and providing excellent documentation
- **Security Research Community**: For sharing knowledge and advancing the field
- **GitHub**: For providing free hosting and tools for open-source projects

## Recognition Policy

### Contributor Recognition

All contributors are recognized based on:
- **Code contributions**: Merged pull requests
- **Documentation**: Improvements to guides and references
- **Bug reports**: Well-documented issues that led to fixes
- **Security reports**: Responsible vulnerability disclosure
- **Community support**: Helping users and reviewing code

### Recognition Levels

- **1+ merged PR**: Listed in Contributors section
- **5+ merged PRs**: Listed with contribution area
- **10+ merged PRs**: Listed as Major Contributor
- **Significant feature**: Special recognition in CHANGELOG
- **Security research**: Credited in security advisories

### How to Update

To update your information in AUTHORS.md:
1. Submit a pull request with your changes
2. Include your preferred name and contact method (optional)
3. Maintainers will review and merge

## License Agreement

By contributing to ProRT-IP WarScan, you agree that your contributions will be licensed under the **GNU General Public License v3.0 (GPLv3)**, the same license as the project.

See [LICENSE](LICENSE) for full license text.

## Contact

- **Project Repository**: https://github.com/doublegate/ProRT-IP
- **Discussions**: https://github.com/doublegate/ProRT-IP/discussions
- **Issues**: https://github.com/doublegate/ProRT-IP/issues

---

**Last Updated**: 2025-10-07

**Want to be listed here?** See [CONTRIBUTING.md](CONTRIBUTING.md) to get started!

---

## Appendix: Full Dependency Credits

For a complete list of all dependencies and their licenses, run:
```bash
cargo tree --prefix none --edges normal | sort -u
cargo license
```

Key dependencies and their licenses:

| Crate | License | Purpose |
|-------|---------|---------|
| tokio | MIT | Async runtime |
| pnet | MIT/Apache-2.0 | Packet manipulation |
| etherparse | BSD-3-Clause | Packet parsing |
| clap | MIT/Apache-2.0 | CLI parsing |
| serde | MIT/Apache-2.0 | Serialization |
| sqlx | MIT/Apache-2.0 | Database |
| crossbeam | MIT/Apache-2.0 | Concurrency |
| tracing | MIT | Logging |
| governor | MIT | Rate limiting |
| openssl | Apache-2.0 | Cryptography |

*Full dependency tree available in `Cargo.lock` once project is implemented.*
