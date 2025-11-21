# Security Audit Checklist

Comprehensive security verification checklist for ProRT-IP deployments and code reviews.

## Pre-Deployment Checklist

### Binary Verification

- [ ] Binary built from trusted source
- [ ] Release signatures verified (if available)
- [ ] Binary permissions restricted (no setuid unless required)
- [ ] Dependencies audited (`cargo audit`)
- [ ] No known CVEs in dependencies

### System Configuration

- [ ] Dedicated user account created (not root)
- [ ] Linux capabilities set (`CAP_NET_RAW`) instead of setuid root
- [ ] File permissions restricted (700 for config, 755 for binary)
- [ ] Working directory secured
- [ ] Log directory permissions set (700)

### Network Configuration

- [ ] Firewall rules reviewed
- [ ] Outbound traffic monitoring configured
- [ ] Rate limiting enabled
- [ ] Source IP binding configured (if multi-homed)

## Operational Security Checklist

### Before Each Scan

- [ ] Authorization documentation verified
- [ ] Scope boundaries confirmed
- [ ] Emergency contacts available
- [ ] Timing template appropriate for target
- [ ] Rate limiting configured
- [ ] Output file permissions pre-set

### During Scans

- [ ] Resource usage monitored
- [ ] Network impact observed
- [ ] Logs being captured
- [ ] No scope creep occurring
- [ ] Stop conditions understood

### After Each Scan

- [ ] Results secured (encrypted if sensitive)
- [ ] Logs retained appropriately
- [ ] Temporary files cleaned
- [ ] Scan data access logged
- [ ] Results shared only with authorized parties

## Post-Scan Review Checklist

### Data Handling

- [ ] Scan results encrypted at rest
- [ ] Access logs reviewed
- [ ] Data retention policy followed
- [ ] Personal data handled per GDPR/CCPA
- [ ] Sharing limited to need-to-know

### Reporting

- [ ] Findings documented professionally
- [ ] Severity ratings appropriate
- [ ] Remediation recommendations provided
- [ ] Sensitive details redacted for distribution
- [ ] Report delivered securely

### Cleanup

- [ ] Temporary files deleted
- [ ] Working directories cleaned
- [ ] Cache files removed
- [ ] Memory cleared (restart if needed)
- [ ] Session tokens invalidated

## Code Security Checklist

For developers and code reviewers.

### Input Validation

- [ ] All IP addresses validated (`IpAddr::parse()`)
- [ ] CIDR notation validated (`ipnetwork` crate)
- [ ] Port numbers range-checked (1-65535)
- [ ] File paths sanitized
- [ ] User input never used directly in shell commands

### Memory Safety

- [ ] No `unsafe` blocks without documented justification
- [ ] All array accesses bounds-checked
- [ ] No panics in production code paths
- [ ] `Result` types handled (no `unwrap()` in production)
- [ ] Buffer sizes validated before allocation

### Privilege Management

- [ ] Privileges dropped after socket creation
- [ ] Privilege drop verified (cannot regain root)
- [ ] No unnecessary capabilities retained
- [ ] Supplementary groups cleared

### Error Handling

- [ ] Errors logged appropriately (not to user)
- [ ] No sensitive data in error messages
- [ ] Graceful degradation on failures
- [ ] Resource cleanup in error paths

### Cryptography

- [ ] No custom crypto implementations
- [ ] TLS 1.2+ required for connections
- [ ] Certificate validation not disabled
- [ ] Secure random number generation

### Logging and Audit

- [ ] Security events logged
- [ ] Logs don't contain sensitive data
- [ ] Log injection prevented
- [ ] Audit trail maintained

## Deployment Security Checklist

### Container Deployments

- [ ] Minimal base image used
- [ ] Non-root user configured
- [ ] Read-only filesystem where possible
- [ ] Capabilities dropped (`--cap-drop=ALL --cap-add=NET_RAW`)
- [ ] Network isolation configured
- [ ] Resource limits set

### Bare Metal Deployments

- [ ] Dedicated service account
- [ ] SELinux/AppArmor profile applied
- [ ] Systemd service hardening enabled
- [ ] File system permissions restricted
- [ ] Network segmentation in place

### Cloud Deployments

- [ ] Instance hardening applied
- [ ] Security groups configured
- [ ] VPC/network isolation
- [ ] Logging to central SIEM
- [ ] Access controls (IAM) configured

## Vulnerability Response Checklist

When vulnerabilities are discovered:

- [ ] Stop affected scans immediately
- [ ] Assess scope of exposure
- [ ] Notify affected parties
- [ ] Apply patches when available
- [ ] Verify fix effectiveness
- [ ] Document incident and response
- [ ] Update monitoring for similar issues

## Compliance Verification

### OWASP Guidelines

- [ ] A01:2021 - Broken Access Control reviewed
- [ ] A02:2021 - Cryptographic Failures addressed
- [ ] A03:2021 - Injection prevented
- [ ] A04:2021 - Insecure Design reviewed
- [ ] A05:2021 - Security Misconfiguration checked

### Industry Standards

- [ ] NIST Cybersecurity Framework alignment
- [ ] CIS Benchmarks for deployment platform
- [ ] PCI DSS if handling payment data
- [ ] HIPAA if handling health data
- [ ] SOC 2 controls if applicable

## Quick Reference Commands

```bash
# Verify dependencies
cargo audit

# Check for unsafe code
cargo geiger

# Security-focused clippy
cargo clippy -- -D warnings -W clippy::pedantic

# Test with address sanitizer
RUSTFLAGS='-Zsanitizer=address' cargo +nightly test

# Set capabilities (instead of setuid root)
sudo setcap cap_net_raw=ep /usr/local/bin/prtip

# Verify capabilities
getcap /usr/local/bin/prtip
```

## See Also

- [Security Overview](./overview.md) - Security architecture
- [Responsible Use](./responsible-use.md) - Legal and ethical guidelines
- [Compliance](./compliance.md) - Regulatory requirements
