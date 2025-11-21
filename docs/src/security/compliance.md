# Compliance

ProRT-IP is designed to support compliance with industry standards and regulatory requirements for security scanning activities.

## Industry Standards

### OWASP Guidelines

ProRT-IP aligns with OWASP testing methodology:

| OWASP Category | ProRT-IP Support |
|----------------|------------------|
| Information Gathering | Port scanning, service detection |
| Configuration Management | Service version enumeration |
| Authentication Testing | Port availability checks |
| Session Management | TCP connection testing |
| Input Validation | Protocol-specific probes |

**OWASP Testing Guide Integration:**

- **OTG-INFO-001**: Conduct search engine discovery - Network enumeration
- **OTG-INFO-002**: Fingerprint web server - Service detection
- **OTG-INFO-003**: Review webserver metafiles - Port/service mapping
- **OTG-CONFIG-001**: Test network infrastructure - Full network scanning

### NIST Cybersecurity Framework

ProRT-IP supports NIST CSF functions:

| Function | Activity | ProRT-IP Feature |
|----------|----------|------------------|
| **Identify** | Asset Management | Network discovery, port scanning |
| **Identify** | Risk Assessment | Vulnerability identification |
| **Protect** | Protective Technology | Firewall rule validation |
| **Detect** | Security Monitoring | Network change detection |
| **Respond** | Analysis | Incident investigation support |

**NIST SP 800-115 Alignment:**

- Section 4: Planning - Scan scope definition
- Section 5: Discovery - Network enumeration
- Section 6: Vulnerability Analysis - Service detection
- Section 7: Reporting - Multiple output formats

### CIS Benchmarks

ProRT-IP can verify CIS benchmark controls:

```bash
# Check for unnecessary services (CIS 2.1.x)
prtip -sS -p 1-65535 target --top-ports 1000

# Verify firewall configuration (CIS 3.x)
prtip -sA -p 1-1000 target  # ACK scan for firewall rules

# Check network services (CIS 5.x)
prtip -sV -p 22,80,443,3389 target
```

## Regulatory Requirements

### GDPR (General Data Protection Regulation)

When scanning EU systems:

| Article | Requirement | Implementation |
|---------|-------------|----------------|
| Art. 6 | Lawful basis | Document authorization |
| Art. 5 | Data minimization | Scan only necessary targets |
| Art. 32 | Security measures | Encrypt scan results |
| Art. 33 | Breach notification | Report within 72 hours |

### CCPA (California Consumer Privacy Act)

For California-related scanning:

- Document business purpose for scanning
- Implement reasonable security measures
- Maintain records of processing activities
- Honor data subject requests

### PCI DSS

For cardholder data environments:

| Requirement | ProRT-IP Support |
|-------------|------------------|
| 11.2 | Quarterly network scans |
| 11.3 | Penetration testing support |
| 11.4 | IDS/IPS testing |

```bash
# PCI DSS quarterly scan
prtip -sS -sV -p 1-65535 --top-ports 1000 pci-scope.txt \
    -oX pci-scan-$(date +%Y%m%d).xml
```

### HIPAA

For healthcare environments:

| Safeguard | Verification Method |
|-----------|---------------------|
| Access Control | Port/service inventory |
| Audit Controls | Scan logging |
| Integrity | Network change detection |
| Transmission Security | TLS certificate analysis |

### SOX (Sarbanes-Oxley)

For financial systems:

- Document all scanning activities
- Maintain audit trails
- Verify access controls
- Support change management

## Security Certifications

### ProRT-IP Security Status

| Aspect | Status | Details |
|--------|--------|---------|
| Code Audits | Regular | cargo audit, clippy |
| Memory Safety | Rust | No buffer overflows |
| Dependency Scanning | Automated | GitHub Dependabot |
| Fuzz Testing | 230M+ executions | 0 crashes |
| Test Coverage | 54.92% | 2,151+ tests |

## Compliance Documentation

### Audit Support

ProRT-IP provides audit-friendly features:

```bash
# XML output for compliance tools
prtip -sS -sV target -oX audit-scan.xml

# JSON for automated processing
prtip -sS -sV target -oJ audit-scan.json

# Greppable for quick analysis
prtip -sS target -oG audit-scan.gnmap
```

### Documentation Requirements

| Document | Retention | Purpose |
|----------|-----------|---------|
| Authorization | Duration of engagement | Legal protection |
| Scan results | Per retention policy | Audit evidence |
| Methodology | Indefinite | Process documentation |
| Findings | Per retention policy | Remediation tracking |

## See Also

- [Security Overview](./overview.md) - Security architecture
- [Responsible Use](./responsible-use.md) - Legal and ethical guidelines
- [Audit Checklist](./audit-checklist.md) - Security verification
