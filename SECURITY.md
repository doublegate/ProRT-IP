# Security Policy

## Overview

ProRT-IP WarScan is a **defensive security tool** designed for authorized penetration testing and network security assessments. This document outlines our security policy, vulnerability reporting procedures, and best practices for users.

## Supported Versions

| Version | Status | Security Updates |
|---------|--------|------------------|
| 0.0.x   | Pre-release | Documentation only |
| 1.0.x   | Planned | Full support (when released) |

**Current Status:** Documentation and planning phase. No production releases yet.

## Reporting Security Vulnerabilities

We take security vulnerabilities seriously. If you discover a security issue in ProRT-IP WarScan, please follow responsible disclosure practices.

### Reporting Process

**DO NOT** create public GitHub issues for security vulnerabilities.

Instead, report security issues privately:

1. **Email**: Send details to the project maintainers
   - Subject: `[SECURITY] Brief description of vulnerability`
   - Include detailed information (see below)

2. **GitHub Security Advisory**: Use GitHub's private vulnerability reporting
   - Navigate to: `https://github.com/doublegate/ProRT-IP/security/advisories`
   - Click "Report a vulnerability"

### Information to Include

Please provide as much information as possible:

- **Vulnerability Type**: Buffer overflow, privilege escalation, DoS, etc.
- **Affected Component**: Scanner engine, packet parser, CLI, etc.
- **Affected Versions**: Which versions are vulnerable?
- **Attack Vector**: How can the vulnerability be exploited?
- **Impact Assessment**: What's the potential damage?
- **Proof of Concept**: Steps to reproduce (if safe to share)
- **Suggested Fix**: If you have recommendations
- **CVE Request**: Whether you plan to request a CVE identifier

### Example Report

```
Subject: [SECURITY] Buffer overflow in TCP packet parser

Component: prtip-network/tcp_parser.rs
Versions: All current development versions
Severity: High (Remote code execution potential)

Description:
The TCP packet parser does not properly validate the data offset field,
allowing malformed packets to trigger out-of-bounds reads.

Attack Vector:
Send crafted TCP packet with data offset > packet length while scanning
malicious targets.

Impact:
- Potential crash (DoS)
- Possible information disclosure via heap overflow
- Remote code execution (unconfirmed)

Reproduction:
1. Set up malicious server that sends packets with data_offset=15
2. Run prtip scan against the server
3. Observe crash in tcp_parser.rs:123

Suggested Fix:
Add bounds checking before accessing packet.data[offset..]
```

## Security Disclosure Timeline

We follow coordinated disclosure practices:

1. **Day 0**: Vulnerability reported privately
2. **Day 1-3**: Acknowledge receipt, begin investigation
3. **Day 3-7**: Assess severity, develop fix
4. **Day 7-14**: Test and validate fix
5. **Day 14-30**: Prepare security advisory and patched release
6. **Day 30+**: Public disclosure (coordinated with reporter)

**Exceptions:**

- **Critical vulnerabilities**: Expedited 7-14 day timeline
- **Already public**: Immediate disclosure and patching
- **Extended embargo**: By mutual agreement for complex issues

## Security Best Practices for Users

### Legal and Ethical Use

⚠️ **IMPORTANT**: ProRT-IP WarScan is a powerful network scanning tool. **Use only on networks you own or have explicit written permission to test.**

**Prohibited Uses:**

- Scanning networks without authorization
- Unauthorized penetration testing
- Network reconnaissance for malicious purposes
- Any illegal activity

**Authorized Uses:**

- Internal network security assessments
- Penetration testing with client authorization
- Security research on owned infrastructure
- Educational purposes in controlled environments

### Operational Security

1. **Privilege Management**
   - Run with minimum required privileges
   - Use Linux capabilities instead of root where possible
   - Never run as root unnecessarily

2. **Rate Limiting**
   - Use appropriate timing templates (-T0 through -T5)
   - Respect network bandwidth constraints
   - Avoid unintentional DoS conditions

3. **Data Protection**
   - Secure scan results (may contain sensitive network information)
   - Encrypt result databases if storing long-term
   - Follow data retention policies
   - Comply with applicable regulations (GDPR, CCPA, etc.)

4. **Audit Logging**
   - Enable audit logging for all scans
   - Retain logs for accountability
   - Monitor for unauthorized usage

### Network Safety

1. **Test in Isolated Environments**
   - Use Docker test networks for development
   - Validate on non-production networks first
   - Understand scan impact before production use

2. **Gradual Rollout**
   - Start with conservative timing (-T2 Polite)
   - Test single targets before /24 scans
   - Monitor network impact

3. **Firewall Considerations**
   - Expect IDS/IPS alerts from stealth scans
   - Coordinate with network security teams
   - Whitelist scanning systems if needed

## Implementation Security

For developers contributing to ProRT-IP WarScan, see **[docs/08-SECURITY.md](docs/08-SECURITY.md)** for comprehensive security implementation guidelines:

- Input validation requirements
- Privilege dropping patterns
- Safe packet parsing techniques
- DoS prevention strategies
- Security audit checklist (50+ items)

### Key Security Principles

1. **Defense in Depth**
   - Multiple layers of validation
   - Fail securely on errors
   - Principle of least privilege

2. **Secure by Default**
   - Conservative default settings
   - Require explicit opt-in for aggressive scans
   - Warn users of potential impact

3. **Input Validation**
   - Validate all user input (IP addresses, CIDR, ports, files)
   - Use safe parsing libraries (`IpAddr::parse()`, not regex)
   - Reject invalid input early

4. **Resource Limits**
   - Bounded memory usage
   - Connection limits
   - Timeout enforcement
   - File descriptor limits

5. **Safe Packet Parsing**
   - Never `panic!` on malformed packets
   - Use bounds-checked parsing (pnet, etherparse)
   - Return `Result` types, not `unwrap()`
   - Validate offsets before indexing

## Known Security Considerations

### Privilege Requirements

ProRT-IP WarScan requires elevated privileges for raw packet operations:

- **Linux**: `CAP_NET_RAW` capability or root
- **Windows**: Administrator privileges (Npcap requirement)
- **macOS**: Root or ChmodBPF permissions

**Mitigation**: Privileges are dropped immediately after creating raw sockets. Scanning logic runs unprivileged.

### Packet Crafting Risks

Raw packet crafting can potentially:

- Trigger firewall rules
- Generate IDS/IPS alerts
- Cause network instability if misconfigured

**Mitigation**:

- Comprehensive testing before release
- Rate limiting by default
- Clear documentation of risks

### Stateless Scanning Risks

High-speed stateless scanning (1M+ pps) can:

- Overwhelm network infrastructure
- Trigger DoS protections
- Consume excessive bandwidth

**Mitigation**:

- Adaptive rate limiting
- User confirmation for high-rate scans
- Conservative defaults (T3 Normal)

## Security Vulnerability History

No security vulnerabilities reported yet (pre-release phase).

Future security advisories will be listed here with:

- CVE identifiers
- Affected versions
- Severity ratings
- Mitigation steps
- Credit to reporters

## Security Hardening Recommendations

### For Deployment

1. **Containerization**

   ```bash
   # Run in Docker with restricted capabilities
   docker run --rm --cap-drop=ALL --cap-add=NET_RAW prtip:latest
   ```

2. **AppArmor/SELinux Profiles**
   - Restrict file system access
   - Limit network operations
   - Enforce privilege boundaries

3. **Separate User Account**

   ```bash
   # Create dedicated user
   sudo useradd -r -s /bin/false prtip-scanner

   # Set capabilities on binary
   sudo setcap cap_net_raw=ep /usr/local/bin/prtip
   ```

4. **Audit Logging**

   ```bash
   # Log all scans
   prtip --audit-log /var/log/prtip/audit.log ...
   ```

### For Development

1. **Dependency Auditing**

   ```bash
   # Check for vulnerable dependencies
   cargo audit

   # Update dependencies regularly
   cargo update
   ```

2. **Fuzzing**

   ```bash
   # Fuzz packet parsers
   cargo fuzz run tcp_parser
   cargo fuzz run udp_parser
   ```

3. **Static Analysis**

   ```bash
   # Clippy with security lints
   cargo clippy -- -D warnings

   # Additional security checks
   cargo geiger  # Unsafe code detection
   ```

4. **Memory Safety**

   ```bash
   # Test with Miri
   cargo +nightly miri test

   # Test with Valgrind
   valgrind --leak-check=full ./target/debug/prtip
   ```

## Compliance and Certifications

### Current Status

- No formal security certifications yet (pre-release)
- Security review planned before v1.0 release

### Planned Certifications

- External security audit before v1.0
- OWASP ASVS compliance review
- Common Vulnerability Scoring System (CVSS) adoption

## Security Contact

For security-related questions or concerns:

- **Security Issues**: Use private reporting (see above)
- **Security Questions**: Create public GitHub Discussion
- **General Security**: See [docs/08-SECURITY.md](docs/08-SECURITY.md)

## Acknowledgments

We appreciate security researchers who:

- Follow responsible disclosure practices
- Provide detailed vulnerability reports
- Work with us on coordinated disclosure
- Help improve security for all users

Security researchers will be credited in:

- CHANGELOG.md security fix entries
- Security advisories
- AUTHORS.md acknowledgments section

## Legal Notice

**ProRT-IP WarScan is provided "as is" without warranty of any kind.** Users are responsible for:

- Obtaining proper authorization before scanning
- Compliance with applicable laws and regulations
- Understanding and accepting risks
- Proper configuration and usage

**The developers are not liable for:**

- Unauthorized or illegal use
- Network damage or service disruption
- Legal consequences of misuse
- Any damages resulting from use

## Additional Resources

- **Implementation Security**: [docs/08-SECURITY.md](docs/08-SECURITY.md)
- **Testing Security**: [docs/06-TESTING.md](docs/06-TESTING.md)
- **Architecture Security**: [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md#security-architecture)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md#security-guidelines)
- **Support**: [SUPPORT.md](SUPPORT.md)

---

**Last Updated**: 2025-11-16
**Security Policy Version**: 1.0
