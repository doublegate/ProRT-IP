# Ethical Scanning Notice

**Target Lists Generated:** 2024-11-17
**Purpose:** Internet-scale validation for ProRT-IP Sprint 6.3

## Responsible Disclosure Policy

### Scan Limitations
- **Single Port Only:** All internet-scale scans limited to port 80 or 443 (HTTP/HTTPS)
- **Rate Limiting:** Maximum 10,000 packets per second (10K pps)
- **No Exploitation:** SYN scan only, no connection attempts or service probing
- **Duration:** Scans complete in 5-15 seconds per target list

### Legal Compliance
- **Authorization:** Self-generated IP lists from public CDN ranges (no unauthorized target systems)
- **Scope:** Discovery scans only (equivalent to ping/traceroute)
- **Intent:** Performance validation for security tool development
- **Data Retention:** No scan data stored beyond benchmark validation

### Target IP Sources
1. **CDN Ranges (60-70%):** Publicly documented CIDR blocks
   - Cloudflare: https://www.cloudflare.com/ips/
   - Fastly: https://api.fastly.com/public-ip-list
   - Akamai: Public ASN lookups (AS12222, AS20940)
   - AWS CloudFront: https://ip-ranges.amazonaws.com/ip-ranges.json

2. **Public Ranges (30-40%):** Unallocated or well-known infrastructure
   - Google Public DNS (8.8.0.0/16)
   - RIPE NCC allocations (185.0.0.0/8)
   - APNIC ranges (1.1.0.0/16)

### Mitigation Measures
- **Firewall Friendly:** SYN scans appear as normal connection attempts (no fragmentation, decoys, or evasion)
- **Logging:** All scans logged with timestamps, target counts, and configurations
- **Abort Capability:** Scans can be terminated immediately (Ctrl+C)
- **Reporting:** Results aggregated at CIDR level (no individual IP reporting)

### Contact Information
**Project:** ProRT-IP Network Scanner
**Repository:** https://github.com/doublegate/ProRT-IP
**License:** GPL-3.0 (Open Source Security Tool)
**Purpose:** Validate batch I/O optimizations (sendmmsg/recvmmsg) and CDN filtering accuracy

### Acknowledgment
By using these target lists, you acknowledge:
1. Scans are limited to discovery (no exploitation or vulnerability testing)
2. Rate limiting prevents denial-of-service conditions
3. Results used solely for performance benchmarking
4. Compliance with local laws and regulations (consult legal counsel if uncertain)

### Recommendations
- **Production Scans:** Use explicit authorization from target network owners
- **Whitelisting:** Contact CDN providers for permission (optional but recommended)
- **Alternative Validation:** Consider using owned infrastructure or cloud sandbox environments

**Note:** This notice applies to internet-scale validation only. Production use of ProRT-IP requires proper authorization and compliance with applicable laws.
