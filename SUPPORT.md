# Support

Welcome to ProRT-IP WarScan support! This document provides resources and guidance for getting help with the project.

## Table of Contents

- [Documentation](#documentation)
- [Getting Help](#getting-help)
- [Reporting Issues](#reporting-issues)
- [Community](#community)
- [FAQ](#faq)
- [Commercial Support](#commercial-support)

## Documentation

### Primary Documentation

ProRT-IP WarScan has comprehensive documentation in the `docs/` directory:

| Document | Purpose |
|----------|---------|
| [docs/README.md](docs/README.md) | Documentation navigation guide |
| [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md) | System architecture and design patterns |
| [docs/01-ROADMAP.md](docs/01-ROADMAP.md) | Development roadmap and timeline |
| [docs/02-TECHNICAL-SPECS.md](docs/02-TECHNICAL-SPECS.md) | Protocol specifications and packet formats |
| [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md) | Development environment setup |
| [docs/04-IMPLEMENTATION-GUIDE.md](docs/04-IMPLEMENTATION-GUIDE.md) | Implementation patterns and code examples |
| [docs/05-API-REFERENCE.md](docs/05-API-REFERENCE.md) | Complete API documentation |
| [docs/06-TESTING.md](docs/06-TESTING.md) | Testing strategy and guidelines |
| [docs/07-PERFORMANCE.md](docs/07-PERFORMANCE.md) | Performance optimization techniques |
| [docs/08-SECURITY.md](docs/08-SECURITY.md) | Security implementation and best practices |
| [docs/09-FAQ.md](docs/09-FAQ.md) | Frequently asked questions |
| [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md) | Current project status and tasks |

### Quick Start Guides

**New Users:**
1. Read [README.md](README.md) for project overview
2. Check [docs/09-FAQ.md](docs/09-FAQ.md) for common questions
3. Follow [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md) for installation

**Developers:**
1. Review [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md) for system design
2. Read [docs/04-IMPLEMENTATION-GUIDE.md](docs/04-IMPLEMENTATION-GUIDE.md) for code patterns
3. See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines

**Security Researchers:**
1. Read [SECURITY.md](SECURITY.md) for security policy
2. Review [docs/08-SECURITY.md](docs/08-SECURITY.md) for implementation details
3. Follow responsible disclosure practices

## Getting Help

### Before Asking for Help

1. **Search existing resources:**
   - Check [docs/09-FAQ.md](docs/09-FAQ.md) for common questions
   - Search [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
   - Review relevant documentation sections

2. **Verify your setup:**
   - Ensure system requirements are met ([docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md))
   - Check Rust version: `rustc --version` (1.70+ required)
   - Verify dependencies are installed

3. **Reproduce the issue:**
   - Try a minimal example
   - Note exact error messages
   - Document steps to reproduce

### Where to Get Help

#### GitHub Discussions (Recommended)

For questions, ideas, and general discussion:

**URL**: https://github.com/doublegate/ProRT-IP/discussions

**Categories:**
- **Q&A**: Ask questions and get answers
- **Ideas**: Propose new features or improvements
- **Show and Tell**: Share your projects and use cases
- **General**: General discussion about ProRT-IP

**When to use:**
- You have a question about usage
- You want to discuss features or design
- You're seeking advice on implementation
- You want to share your experience

#### GitHub Issues

For bug reports and feature requests:

**URL**: https://github.com/doublegate/ProRT-IP/issues

**When to use:**
- You've found a bug (not a security vulnerability)
- You have a specific feature request
- You've identified a documentation gap
- You want to track a specific problem

**Bug Report Template:**
```markdown
**Environment:**
- OS: [Linux/Windows/macOS version]
- Rust version: [output of `rustc --version`]
- ProRT-IP version: [version or commit hash]

**Description:**
Clear description of the issue

**Steps to Reproduce:**
1. Step one
2. Step two
3. ...

**Expected Behavior:**
What you expected to happen

**Actual Behavior:**
What actually happened

**Error Messages:**
```
Paste error messages here
```

**Additional Context:**
Any other relevant information
```

#### Email Support

For private inquiries:
- **General questions**: Consider GitHub Discussions instead
- **Security vulnerabilities**: See [SECURITY.md](SECURITY.md)
- **Partnership inquiries**: Contact maintainers via GitHub profile

## Reporting Issues

### Bug Reports

**Before reporting:**
- Search for existing issues
- Verify it's reproducible
- Collect necessary information

**Good bug reports include:**
- Clear, descriptive title
- Environment details (OS, Rust version, dependencies)
- Minimal reproducible example
- Expected vs actual behavior
- Complete error messages and stack traces
- Relevant log output (`RUST_LOG=debug`)

**Example:**
```
Title: TCP SYN scan fails on IPv6 targets

Environment:
- OS: Ubuntu 22.04 LTS
- Rust: 1.75.0
- ProRT-IP: commit abc1234

Description:
SYN scans fail when targeting IPv6 addresses, but work for IPv4.

Steps to Reproduce:
1. Run: prtip -sS -6 2001:db8::1
2. Observe error

Expected: Scan completes successfully
Actual: Error "Invalid checksum in IPv6 pseudo-header"

Error:
thread 'main' panicked at 'Invalid checksum', src/network/tcp.rs:123
```

### Feature Requests

**Good feature requests include:**
- Problem statement (what problem does this solve?)
- Proposed solution
- Alternative approaches considered
- Use cases and examples
- Impact on existing functionality

**Example:**
```
Title: Add support for SCTP scanning

Problem:
Many modern applications use SCTP, but ProRT-IP only supports TCP/UDP.

Proposed Solution:
Implement SCTP INIT scanning similar to TCP SYN scans.

Alternatives:
- Use external SCTP tools and integrate results
- Wait for community contribution

Use Cases:
- Telecom network assessments (SCTP is common in SS7/Diameter)
- WebRTC service discovery

Impact:
- New scan type option (-sY for SCTP)
- Additional packet parsing logic
- Platform-specific SCTP socket support
```

### Documentation Issues

**Found unclear or incorrect documentation?**
- Open an issue with the `documentation` label
- Specify which document and section
- Suggest improvements or corrections

## Community

### Project Status

**Current Phase**: Documentation complete, implementation starting

**Contribution Opportunities:**
- Review and provide feedback on documentation
- Help with initial implementation (Phase 1)
- Contribute test cases and benchmarks
- Improve cross-platform compatibility

### Ways to Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines:

- **Code**: Implement features, fix bugs, optimize performance
- **Documentation**: Improve guides, fix typos, add examples
- **Testing**: Write tests, report bugs, verify fixes
- **Design**: Propose architecture improvements, review PRs
- **Community**: Answer questions, help users, share knowledge

### Communication Channels

- **GitHub Discussions**: Questions, ideas, general discussion
- **GitHub Issues**: Bug reports, feature requests
- **Pull Requests**: Code contributions and reviews

**Planned:**
- Discord server (when community grows)
- Monthly development updates
- Community calls for major decisions

## FAQ

### General Questions

**Q: What is ProRT-IP WarScan?**
A: A modern network scanner combining Masscan's speed with Nmap's detection capabilities, written in Rust.

**Q: Is ProRT-IP ready for production use?**
A: Not yet. The project is in documentation and early development phase. See [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md) for current status.

**Q: What platforms are supported?**
A: Linux, Windows (with Npcap), and macOS (with BPF permissions). See [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md) for details.

**Q: How fast is ProRT-IP?**
A: Target performance is 1M+ packets/second in stateless mode. Actual performance depends on hardware and network conditions. See [docs/07-PERFORMANCE.md](docs/07-PERFORMANCE.md).

### Installation and Setup

**Q: How do I install ProRT-IP?**
A: Currently, build from source following [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md). Binary releases will be available with v1.0.

**Q: Why do I need elevated privileges?**
A: Raw packet operations require root/Administrator for socket creation. Privileges are dropped immediately after. See [docs/08-SECURITY.md](docs/08-SECURITY.md).

**Q: How do I set up Linux capabilities instead of root?**
A:
```bash
sudo setcap cap_net_raw=ep /path/to/prtip
```

### Usage and Features

**Q: How is ProRT-IP different from Nmap?**
A: ProRT-IP aims for higher performance (1M+ pps) using Rust's safety and modern async I/O, while maintaining Nmap-like detection accuracy.

**Q: Does ProRT-IP support IPv6?**
A: Planned for Phase 3. Currently focusing on IPv4. See [docs/01-ROADMAP.md](docs/01-ROADMAP.md).

**Q: Can I use ProRT-IP for bug bounties?**
A: Yes, if authorized by the program. Always verify authorization and comply with program rules.

### Development

**Q: Can I contribute?**
A: Yes! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines. Start with issues labeled `good-first-issue`.

**Q: What Rust version is required?**
A: Minimum Rust 1.70, latest stable recommended.

**Q: Where should I start contributing?**
A: Review [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md) for current tasks and [CONTRIBUTING.md](CONTRIBUTING.md) for process.

### More Questions

For comprehensive FAQ, see [docs/09-FAQ.md](docs/09-FAQ.md).

## Commercial Support

### Current Status

Commercial support is not currently available (pre-release phase).

### Future Plans

**Planned offerings:**
- **Enterprise support**: Priority bug fixes, dedicated assistance
- **Custom development**: Feature development for specific needs
- **Training and consulting**: Network security assessment training
- **Integration services**: Integration with existing security tools

**Interested in commercial support?**
Contact the maintainers to discuss your requirements and timeline.

## Response Times

**Expected response times (best effort):**

| Channel | First Response | Resolution |
|---------|---------------|------------|
| GitHub Discussions | 1-3 days | Varies |
| GitHub Issues (bugs) | 2-5 days | Varies by severity |
| GitHub Issues (features) | 3-7 days | Depends on roadmap |
| Pull Requests | 3-5 days | Varies by complexity |
| Security Reports | 1-2 days | 14-30 days (coordinated) |

**Note**: ProRT-IP is currently a community-driven project. Response times are not guaranteed and depend on maintainer availability.

## Additional Resources

### Official Links

- **GitHub Repository**: https://github.com/doublegate/ProRT-IP
- **Documentation**: [docs/README.md](docs/README.md)
- **Issue Tracker**: https://github.com/doublegate/ProRT-IP/issues
- **Discussions**: https://github.com/doublegate/ProRT-IP/discussions

### Related Projects

- **Nmap**: https://nmap.org/
- **Masscan**: https://github.com/robertdavidgraham/masscan
- **RustScan**: https://github.com/RustScan/RustScan
- **ZMap**: https://zmap.io/

### Learning Resources

- **Rust Programming**: https://doc.rust-lang.org/book/
- **Tokio Async Runtime**: https://tokio.rs/
- **Network Programming**: [docs/02-TECHNICAL-SPECS.md](docs/02-TECHNICAL-SPECS.md)
- **Packet Analysis**: Wireshark documentation

## Feedback

We welcome feedback on all aspects of the project:

- **Documentation**: Is it clear and helpful?
- **Features**: What would you like to see?
- **Usability**: How can we improve the user experience?
- **Community**: How can we better support users and contributors?

Share your feedback in [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions).

---

**Last Updated**: 2025-10-07

**Need more help?** Check [docs/09-FAQ.md](docs/09-FAQ.md) or ask in [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions).
