# Scan Templates

**Status:** Planned content - comprehensive template system documentation coming soon.

## Overview

ProRT-IP's scan template system provides pre-configured scanning scenarios for common use cases. Templates combine port specifications, scan types, timing settings, and detection options into reusable configurations accessible via the `--template` flag.

## Current Implementation

For now, please refer to:
- **[Configuration Guide](./configuration.md#scan-templates)** - Scan template usage and examples
- **[CLI Reference](../reference/command-reference.md#scan-templates)** - Template command-line flags
- **[Basic Usage Guide](./basic-usage.md)** - Common scanning scenarios

## Built-in Templates

ProRT-IP includes 10 built-in templates:
- `web-servers` - Web service scanning (80, 443, 8080, 8443)
- `databases` - Database discovery (MySQL, PostgreSQL, MongoDB, Redis)
- `quick` - Fast top-100 port scan
- `thorough` - Comprehensive all-port scan
- `stealth` - Evasion-focused stealth scan
- `discovery` - Host discovery and enumeration
- `ssl-only` - SSL/TLS service analysis
- `admin-panels` - Admin interface discovery
- `mail-servers` - Email service scanning
- `file-shares` - SMB/NFS/FTP discovery

## Planned Content

This guide will include:
- Detailed explanation of each built-in template
- Template configuration format specification
- Creating custom templates (~/.prtip/templates.toml)
- Template inheritance and composition
- Template validation and testing
- Use case examples for each template
- Performance characteristics comparison
- Integration with CI/CD pipelines

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
