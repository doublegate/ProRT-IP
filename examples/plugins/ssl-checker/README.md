# SSL/TLS Certificate Checker Plugin

Detection plugin for ProRT-IP that identifies and analyzes SSL/TLS encrypted services and certificates.

## Overview

This plugin demonstrates the ProRT-IP plugin architecture for SSL/TLS service detection. It identifies encrypted services based on:
- Standard SSL/TLS ports (443, 8443, 465, 993, 995, etc.)
- TLS protocol signatures in banners
- Certificate information (when exposed via ProRT-IP API)

**Note**: This is a demonstration plugin showing plugin system integration patterns. Full certificate analysis would require extended ProRT-IP TLS API support.

## Features

### Current Implementation

- ✓ Standard SSL/TLS port identification
- ✓ TLS protocol signature detection in banners
- ✓ Service type identification (HTTPS, SMTPS, IMAPS, POP3S, etc.)
- ✓ Network capability utilization

### Future Enhancements

When ProRT-IP TLS API is extended, this plugin could provide:
- Certificate chain validation
- Certificate details extraction (subject, issuer, SAN)
- Security assessment (expiration, key strength, self-signed detection)
- TLS protocol analysis (versions, cipher suites)
- Vulnerability checks (Heartbleed, POODLE, FREAK)

## Supported SSL/TLS Services

| Port | Service | Description |
|------|---------|-------------|
| 443 | https | HTTPS web servers |
| 465 | smtps | SMTP over SSL |
| 587 | submission | Mail submission (STARTTLS) |
| 636 | ldaps | LDAP over SSL |
| 989 | ftps-data | FTP-DATA over SSL |
| 990 | ftps | FTP over SSL |
| 992 | telnets | Telnet over SSL |
| 993 | imaps | IMAP over SSL |
| 994 | ircs | IRC over SSL |
| 995 | pop3s | POP3 over SSL |
| 3389 | rdp | Remote Desktop Protocol over TLS |
| 5223 | xmpps | XMPP over SSL |
| 5671 | amqps | AMQP over TLS |
| 6514 | syslog-tls | Syslog over TLS |
| 8000 | https-alt | Alternative HTTPS port |
| 8080 | http-proxy | HTTP proxy (may support TLS) |
| 8443 | https-alt | Alternative HTTPS port |

## Installation

### Manual Installation

```bash
mkdir -p ~/.prtip/plugins/ssl-checker
cp plugin.toml main.lua README.md ~/.prtip/plugins/ssl-checker/
```

### Verify Installation

```bash
prtip --list-plugins
# Should show: ssl-checker v1.0.0 (detection)
```

## Usage

The plugin is automatically loaded when ProRT-IP starts. It analyzes services during scanning to detect SSL/TLS encrypted connections.

### Example Output

```
Target: 192.168.1.1:443
Service: https
Info: TLS/SSL encrypted service
Confidence: 0.8
```

## Technical Details

### Capabilities Required

- **network**: Required for active SSL/TLS probing (future implementation)

### API Functions

#### analyze_banner(banner)

Analyzes a service banner for SSL/TLS indicators.

**Parameters:**
- `banner` (string): The service banner text

**Returns:**
- ServiceInfo table with SSL/TLS detection, or `nil`

**Detection Logic:**
1. Check for "TLS" or "SSL" keywords in banner
2. Look for TLS protocol signatures (0x15, 0x16 bytes)
3. Identify HTTPS-specific patterns
4. Return service info with confidence score

#### probe_service(target)

Active SSL/TLS service probing (placeholder for future implementation).

**Parameters:**
- `target` (table): Target information with `ip` and optional `hostname`

**Returns:**
- `nil` (full implementation pending TLS API extension)

**Future Implementation:**
```lua
function probe_service(target)
    local socket_id = prtip.connect_tls(target.ip, 443, 5.0)
    if not socket_id then
        return nil
    end

    local cert_info = prtip.get_certificate(socket_id)
    prtip.close(socket_id)

    if cert_info then
        return {
            service = "https",
            product = cert_info.subject_cn,
            version = cert_info.tls_version,
            info = string.format("Expires: %s", cert_info.not_after),
            confidence = 0.95
        }
    end

    return nil
end
```

## Future Enhancement Roadmap

### 1. Certificate Chain Validation

- Verify certificate chain completeness
- Check intermediate certificates
- Validate root CA trust
- Detect chain ordering issues

### 2. Certificate Details Analysis

- Parse Subject and Issuer Distinguished Names
- Extract Subject Alternative Names (SAN)
- Analyze key usage and extended key usage flags
- Parse certificate policies
- Extract validity dates

### 3. Security Assessment

- Certificate expiration checking (warn if < 30 days)
- Weak key detection (< 2048-bit RSA, < 256-bit ECC)
- Self-signed certificate identification
- Certificate transparency log checking
- Revocation status (OCSP, CRL)

### 4. TLS Protocol Analysis

- Supported TLS versions (TLS 1.0, 1.1, 1.2, 1.3)
- Cipher suite enumeration
- Perfect Forward Secrecy (PFS) support
- Weak cipher detection
- Compression support (CRIME vulnerability)
- Renegotiation security

### 5. Vulnerability Checks

- Heartbleed (CVE-2014-0160)
- POODLE (CVE-2014-3566)
- FREAK (CVE-2015-0204)
- LOGJAM (CVE-2015-4000)
- BEAST (CVE-2011-3389)
- CRIME (CVE-2012-4929)
- DROWN (CVE-2016-0800)

## Required ProRT-IP TLS API Extensions

To enable full SSL/TLS analysis capabilities, ProRT-IP would need to expose:

### Connection API

```lua
-- Connect with TLS/SSL
socket_id = prtip.connect_tls(ip, port, timeout, tls_options)

-- TLS options table
tls_options = {
    min_version = "TLS1.2",  -- Minimum TLS version
    max_version = "TLS1.3",  -- Maximum TLS version
    verify_cert = true,      -- Verify certificate chain
    ciphers = nil,           -- Specific ciphers (nil = default)
    sni_hostname = "example.com"  -- SNI hostname
}
```

### Certificate API

```lua
-- Get certificate information
cert_info = prtip.get_certificate(socket_id)

-- Returns table:
{
    subject_cn = "example.com",
    subject_dn = "CN=example.com,O=Example Inc,C=US",
    issuer_cn = "Example CA",
    issuer_dn = "CN=Example CA,O=Example CA,C=US",
    not_before = "2024-01-01T00:00:00Z",
    not_after = "2025-01-01T00:00:00Z",
    serial_number = "01:23:45:67:89:AB:CD:EF",
    sans = {"example.com", "www.example.com", "*.example.com"},
    public_key = {
        algorithm = "RSA",
        size = 2048
    },
    signature_algorithm = "SHA256-RSA",
    tls_version = "TLS1.3",
    cipher_suite = "TLS_AES_256_GCM_SHA384",
    is_self_signed = false,
    chain_valid = true,
    chain = { cert1, cert2, cert3 }  -- Full chain
}
```

### Protocol API

```lua
-- Get TLS protocol information
proto_info = prtip.get_tls_info(socket_id)

-- Returns table:
{
    version = "TLS1.3",
    cipher_suite = "TLS_AES_256_GCM_SHA384",
    has_pfs = true,
    compression = "none",
    renegotiation_secure = true
}
```

## Security Considerations

- **Network Access**: Requires `network` capability for active probing
- **Rate Limiting**: Respects ProRT-IP rate limits
- **Timeout Handling**: Uses configured timeouts
- **Error Handling**: Gracefully handles connection failures
- **Memory Efficient**: Minimal memory footprint
- **No Credential Storage**: Does not store sensitive data

## Performance

- **Fast Detection**: Banner analysis is O(n) where n is banner length
- **Low Overhead**: Minimal CPU usage
- **Non-Blocking**: Uses async operations when available
- **Stateless**: Each analysis is independent

## Troubleshooting

### Plugin Not Loading

```bash
# Check files exist
ls -la ~/.prtip/plugins/ssl-checker/
# Verify capabilities are allowed in configuration
```

### No SSL/TLS Detections

1. Verify banner capture is enabled
2. Check target has SSL/TLS services
3. Enable debug logging: `--log-level debug`
4. Review plugin logs

### Connection Failures

- Check network connectivity
- Verify target accepts connections
- Increase timeout values
- Check firewall rules

## Example Scenarios

### Scan for HTTPS Services

```bash
prtip -sS -p 443,8443 -sV target-network
# SSL checker will analyze detected services
```

### Full SSL/TLS Assessment

```bash
prtip -sS -p 443,465,993,995 -sV --plugin ssl-checker target-host
# Comprehensive SSL/TLS port scan with analysis
```

## Integration Examples

### Custom Certificate Validation

When TLS API is available, custom validation could be added:

```lua
function validate_custom(cert_info)
    -- Check certificate expiration
    local expires_soon = check_expiration(cert_info.not_after, 30)

    -- Check key strength
    local weak_key = cert_info.public_key.size < 2048

    -- Check self-signed
    local self_signed = cert_info.is_self_signed

    return {
        expires_soon = expires_soon,
        weak_key = weak_key,
        self_signed = self_signed,
        overall_secure = not (expires_soon or weak_key or self_signed)
    }
end
```

## License

GPL-3.0 - Same as ProRT-IP

## Contributing

Contributions welcome:
1. Add SSL/TLS port mappings
2. Improve banner detection patterns
3. Design TLS API extensions
4. Submit pull request to ProRT-IP repository

## Version History

- **1.0.0** (2024): Initial demonstration release

## Author

ProRT-IP Contributors

## See Also

- ProRT-IP Plugin System Guide: `docs/30-PLUGIN-SYSTEM-GUIDE.md`
- TLS Certificate Analysis: `crates/prtip-scanner/src/tls_certificate.rs`
- Plugin API Reference: `crates/prtip-scanner/src/plugin/`
- Banner Analyzer Plugin: `examples/plugins/banner-analyzer/`

## References

- [RFC 5280](https://tools.ietf.org/html/rfc5280) - X.509 Public Key Infrastructure
- [RFC 8446](https://tools.ietf.org/html/rfc8446) - TLS 1.3
- [RFC 6125](https://tools.ietf.org/html/rfc6125) - Certificate Validation
- [OWASP TLS Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Security_Cheat_Sheet.html)
