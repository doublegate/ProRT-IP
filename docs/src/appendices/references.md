# References

External resources, standards, specifications, and related tools for ProRT-IP development and network scanning.

---

## Quick Navigation

- [RFC Standards](#rfc-standards) - Protocol specifications (IPv4, IPv6, TCP, UDP, ICMP)
- [IEEE Standards](#ieee-standards) - Ethernet, 802.11, network layer standards
- [IETF Documents](#ietf-documents) - BCP, informational RFCs, drafts
- [Related Tools](#related-tools) - Nmap, Masscan, Wireshark, tcpdump
- [Security Resources](#security-resources) - OWASP, CWE, CVE, vulnerability databases
- [Compliance Frameworks](#compliance-frameworks) - GDPR, PCI DSS, NIST, ISO
- [Academic Papers](#academic-papers) - Network scanning research
- [Books](#books) - Network security, protocol analysis
- [Rust Ecosystem](#rust-ecosystem) - Libraries and frameworks
- [ProRT-IP Resources](#prort-ip-resources) - Official project links

---

## RFC Standards

Foundational protocol specifications from the Internet Engineering Task Force (IETF).

### IPv4 Protocol Suite

- **[RFC 791](https://www.rfc-editor.org/rfc/rfc791.html) - Internet Protocol (IPv4)**
  Published: September 1981
  Core IPv4 specification: header format, addressing, fragmentation, routing.

- **[RFC 792](https://www.rfc-editor.org/rfc/rfc792.html) - Internet Control Message Protocol (ICMP)**
  Published: September 1981
  Error reporting and diagnostic messages for IPv4 networks.

- **[RFC 793](https://www.rfc-editor.org/rfc/rfc793.html) - Transmission Control Protocol (TCP)**
  Published: September 1981
  Reliable, ordered, error-checked delivery of data streams.

- **[RFC 768](https://www.rfc-editor.org/rfc/rfc768.html) - User Datagram Protocol (UDP)**
  Published: August 1980
  Connectionless, unreliable datagram protocol for low-latency applications.

- **[RFC 826](https://www.rfc-editor.org/rfc/rfc826.html) - Address Resolution Protocol (ARP)**
  Published: November 1982
  Mapping IPv4 addresses to MAC addresses on local networks.

- **[RFC 950](https://www.rfc-editor.org/rfc/rfc950.html) - Internet Standard Subnetting Procedure**
  Published: August 1985
  CIDR notation and subnet mask calculations.

- **[RFC 1191](https://www.rfc-editor.org/rfc/rfc1191.html) - Path MTU Discovery**
  Published: November 1990
  Determining maximum transmission unit along network paths.

### IPv6 Protocol Suite

- **[RFC 8200](https://www.rfc-editor.org/rfc/rfc8200.html) - Internet Protocol, Version 6 (IPv6)**
  Published: July 2017
  128-bit addressing, simplified header format, extension headers.

- **[RFC 4443](https://www.rfc-editor.org/rfc/rfc4443.html) - Internet Control Message Protocol for IPv6 (ICMPv6)**
  Published: March 2006
  Error reporting, diagnostics, and Neighbor Discovery for IPv6.

- **[RFC 4861](https://www.rfc-editor.org/rfc/rfc4861.html) - Neighbor Discovery Protocol (NDP)**
  Published: September 2007
  Address autoconfiguration, duplicate detection, router discovery for IPv6.

- **[RFC 4291](https://www.rfc-editor.org/rfc/rfc4291.html) - IPv6 Addressing Architecture**
  Published: February 2006
  Address types (unicast, multicast, anycast), notation, scope.

- **[RFC 6724](https://www.rfc-editor.org/rfc/rfc6724.html) - Default Address Selection for IPv6**
  Published: September 2012
  Source and destination address selection algorithms.

### Transport and Application Layer

- **[RFC 1035](https://www.rfc-editor.org/rfc/rfc1035.html) - Domain Name System (DNS)**
  Published: November 1987
  Domain name to IP address resolution protocol.

- **[RFC 2616](https://www.rfc-editor.org/rfc/rfc2616.html) - Hypertext Transfer Protocol (HTTP/1.1)** *(Obsoleted by RFC 7230-7237)*
  Published: June 1999
  Web protocol for transferring hypermedia documents.

- **[RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) - HTTP Semantics**
  Published: June 2022
  Current HTTP specification (supersedes RFC 2616).

- **[RFC 8446](https://www.rfc-editor.org/rfc/rfc8446.html) - Transport Layer Security (TLS) 1.3**
  Published: August 2018
  Cryptographic protocol for secure communications.

- **[RFC 5246](https://www.rfc-editor.org/rfc/rfc5246.html) - TLS 1.2** *(Still widely used)*
  Published: August 2008
  Previous TLS version (90% of HTTPS still uses this).

- **[RFC 854](https://www.rfc-editor.org/rfc/rfc854.html) - Telnet Protocol**
  Published: May 1983
  Remote terminal access (insecure, replaced by SSH).

- **[RFC 959](https://www.rfc-editor.org/rfc/rfc959.html) - File Transfer Protocol (FTP)**
  Published: October 1985
  File transfer over TCP connections.

- **[RFC 1157](https://www.rfc-editor.org/rfc/rfc1157.html) - Simple Network Management Protocol (SNMP)**
  Published: May 1990
  Network device management and monitoring.

### Security and Privacy

- **[RFC 4251-4254](https://www.rfc-editor.org/rfc/rfc4251.html) - Secure Shell (SSH) Protocol**
  Published: January 2006
  Encrypted remote login and command execution.

- **[RFC 5280](https://www.rfc-editor.org/rfc/rfc5280.html) - X.509 Public Key Infrastructure Certificate**
  Published: May 2008
  Certificate format and validation (ProRT-IP parses X.509v3 certificates).

- **[RFC 6066](https://www.rfc-editor.org/rfc/rfc6066.html) - TLS Extensions**
  Published: January 2011
  Server Name Indication (SNI) used by ProRT-IP for virtual host TLS analysis.

- **[RFC 6797](https://www.rfc-editor.org/rfc/rfc6797.html) - HTTP Strict Transport Security (HSTS)**
  Published: November 2012
  Forcing HTTPS connections to prevent downgrade attacks.

---

## IEEE Standards

Network layer and physical layer specifications from the Institute of Electrical and Electronics Engineers.

### Ethernet and Local Area Networks

- **[IEEE 802.3](https://standards.ieee.org/ieee/802.3/7071/) - Ethernet**
  Published: Multiple revisions (latest 2022)
  MAC layer protocol, frame format (Dest MAC, Source MAC, EtherType, Payload, FCS).

- **[IEEE 802.1Q](https://standards.ieee.org/ieee/802.1Q/10323/) - Virtual LANs (VLANs)**
  Published: 2022 revision
  VLAN tagging and Quality of Service (QoS) prioritization.

- **[IEEE 802.1X](https://standards.ieee.org/ieee/802.1X/10344/) - Port-Based Network Access Control**
  Published: 2020 revision
  Authentication for wired and wireless LANs (EAP over LAN).

### Wireless Networks

- **[IEEE 802.11](https://standards.ieee.org/ieee/802.11/10536/) - Wireless LAN (Wi-Fi)**
  Published: Multiple amendments (802.11a/b/g/n/ac/ax)
  Wireless medium access control and physical layer specifications.

- **[IEEE 802.11i](https://standards.ieee.org/ieee/802.11i/3127/) - Wireless Security (WPA2)**
  Published: 2004 (incorporated into 802.11-2007)
  Enhanced security with AES encryption (replaced WEP).

---

## IETF Documents

Best Current Practices (BCP), informational RFCs, and Internet-Drafts.

### Best Current Practices

- **[BCP 38](https://www.rfc-editor.org/info/bcp38) - Network Ingress Filtering** *(RFC 2827)*
  Published: May 2000
  Preventing IP address spoofing at network ingress (challenges Idle scanning).

- **[BCP 84](https://www.rfc-editor.org/info/bcp84) - Ingress Filtering for Multihomed Networks** *(RFC 3704)*
  Published: March 2004
  Extending BCP 38 to multihomed networks.

- **[BCP 198](https://www.rfc-editor.org/info/bcp198) - IPv6 Address Allocation and Assignment** *(RFC 7608)*
  Published: July 2015
  IPv6 addressing best practices for operators.

### Informational and Experimental

- **[RFC 3514](https://www.rfc-editor.org/rfc/rfc3514.html) - The Security Flag in the IPv4 Header**
  Published: April 1, 2003 *(April Fools' RFC)*
  Humorous RFC proposing "Evil Bit" flag for malicious packets.

- **[RFC 7707](https://www.rfc-editor.org/rfc/rfc7707.html) - Network Reconnaissance in IPv6 Networks**
  Published: March 2016
  Analysis of IPv6 scanning challenges and address space size (2^64 addresses per subnet).

- **[RFC 5771](https://www.rfc-editor.org/rfc/rfc5771.html) - IANA Guidelines for IPv4 Multicast Address Assignments**
  Published: March 2010
  Multicast address ranges (224.0.0.0/4).

---

## Related Tools

Network scanning, packet analysis, and security tools referenced by ProRT-IP.

### Network Scanners

- **[Nmap](https://nmap.org/)** - Network Mapper
  Author: Gordon Lyon (Fyodor)
  The gold standard for network scanning. ProRT-IP is Nmap-compatible with 50+ flags (`-sS`, `-sV`, `-O`, `-p`, `-T0-T5`).
  - [Nmap Reference Guide](https://nmap.org/book/man.html)
  - [NSE Scripts](https://nmap.org/nsedoc/) (1,200+ scripts)
  - [OS Fingerprinting Database](https://nmap.org/book/osdetect.html) (2,600+ signatures, used by ProRT-IP)
  - [Service Probes](https://github.com/nmap/nmap/blob/master/nmap-service-probes) (ProRT-IP uses 187 of 1,000+ probes)

- **[Masscan](https://github.com/robertdavidgraham/masscan)** - Mass IP Port Scanner
  Author: Robert Graham
  Asynchronous TCP scanner achieving 10M+ pps with custom TCP/IP stack. ProRT-IP targets similar performance.
  - [Masscan Documentation](https://github.com/robertdavidgraham/masscan/blob/master/doc/masscan.8)
  - [Transmission Scheduler](https://github.com/robertdavidgraham/masscan/tree/master/doc)

- **[ZMap](https://zmap.io/)** - Fast Internet Scanner
  Authors: University of Michigan
  Single-packet probes for Internet-wide scanning (1.4B IPv4 addresses in 45 minutes).
  - [ZMap Research Paper](https://www.usenix.org/conference/usenixsecurity13/technical-sessions/paper/durumeric) (USENIX Security 2013)
  - [ZMap GitHub](https://github.com/zmap/zmap)

- **[RustScan](https://github.com/RustScan/RustScan)** - Modern Port Scanner
  Language: Rust
  Ports Nmap's capabilities to Rust with modern UX (ProRT-IP took inspiration for TUI design).
  - [RustScan Docs](https://github.com/RustScan/RustScan)

- **[Naabu](https://github.com/projectdiscovery/naabu)** - Fast Port Scanner
  Language: Go
  SYN scanning with 10K+ pps performance, inspired ProRT-IP's rate limiting design.

### Packet Capture and Analysis

- **[Wireshark](https://www.wireshark.org/)** - Network Protocol Analyzer
  GUI tool for deep packet inspection and dissection.
  - [Display Filters Reference](https://www.wireshark.org/docs/dfref/)
  - [PCAPNG Format Specification](https://github.com/pcapng/pcapng) (used by ProRT-IP)

- **[tcpdump](https://www.tcpdump.org/)** - Command-Line Packet Analyzer
  CLI tool for packet capture and filtering (BPF syntax).
  - [tcpdump Man Page](https://www.tcpdump.org/manpages/tcpdump.1.html)
  - [BPF Filter Syntax](https://www.tcpdump.org/manpages/pcap-filter.7.html)

- **[libpcap](https://www.tcpdump.org/)** - Packet Capture Library
  Cross-platform packet capture library (used by ProRT-IP).
  - [libpcap Documentation](https://www.tcpdump.org/manpages/)
  - [Npcap for Windows](https://npcap.com/) (required for ProRT-IP on Windows)

### Vulnerability Scanners

- **[OpenVAS](https://www.openvas.org/)** - Open Vulnerability Assessment Scanner
  Full-featured vulnerability scanner with 50K+ network vulnerability tests.

- **[Nikto](https://cirt.net/Nikto2)** - Web Server Scanner
  Scans web servers for 6,700+ potentially dangerous files and outdated software.

- **[Metasploit Framework](https://www.metasploit.com/)** - Penetration Testing Platform
  Exploitation framework with 2,000+ exploits and payloads.

---

## Security Resources

Vulnerability databases, security advisories, and best practices.

### OWASP (Open Web Application Security Project)

- **[OWASP Top 10](https://owasp.org/www-project-top-ten/)** - Most Critical Web Application Security Risks
  Updated: 2021
  1. Broken Access Control
  2. Cryptographic Failures
  3. Injection (SQL, Command, LDAP)
  4. Insecure Design
  5. Security Misconfiguration
  6. Vulnerable and Outdated Components
  7. Identification and Authentication Failures
  8. Software and Data Integrity Failures
  9. Security Logging and Monitoring Failures
  10. Server-Side Request Forgery (SSRF)

- **[OWASP Application Security Verification Standard (ASVS)](https://owasp.org/www-project-application-security-verification-standard/)**
  Comprehensive security requirements checklist for web applications.

- **[OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)**
  Methodology for testing web application security (ProRT-IP follows OWASP testing principles).

### CWE (Common Weakness Enumeration)

- **[CWE-20: Improper Input Validation](https://cwe.mitre.org/data/definitions/20.html)**
  ProRT-IP validates all inputs: IP addresses, CIDR ranges, ports (1-65535).

- **[CWE-78: OS Command Injection](https://cwe.mitre.org/data/definitions/78.html)**
  ProRT-IP never constructs shell commands from user input (Rust type safety).

- **[CWE-119: Improper Restriction of Operations within Memory Buffers](https://cwe.mitre.org/data/definitions/119.html)**
  Rust's memory safety prevents buffer overflows (compile-time guarantees).

- **[CWE-362: Concurrent Execution using Shared Resource (Race Condition)](https://cwe.mitre.org/data/definitions/362.html)**
  ProRT-IP uses Rust's ownership model and `Arc<RwLock<T>>` for safe concurrency.

### CVE (Common Vulnerabilities and Exposures)

- **[CVE Database](https://cve.mitre.org/)** - Official CVE database
  ProRT-IP's dependency audit scans for CVEs (zero high/critical CVEs in production).

- **[NVD (National Vulnerability Database)](https://nvd.nist.gov/)** - NIST's CVE enrichment
  CVSS scores, exploit availability, affected software versions.

- **[RustSec Advisory Database](https://rustsec.org/)** - Rust-specific vulnerabilities
  ProRT-IP runs `cargo audit` in CI/CD to detect vulnerable dependencies.

### Security Advisories

- **[US-CERT (CISA)](https://www.cisa.gov/uscert/)** - Cybersecurity & Infrastructure Security Agency
  Official U.S. government cybersecurity advisories.

- **[CERT/CC Vulnerability Notes](https://www.kb.cert.org/vuls/)** - Carnegie Mellon University
  In-depth vulnerability analysis and coordinated disclosure.

---

## Compliance Frameworks

Regulatory and industry standards for security and data protection.

### Data Protection

- **[GDPR (General Data Protection Regulation)](https://gdpr.eu/)** - European Union
  Effective: May 25, 2018
  ProRT-IP provides GDPR-compliant features:
  - Data minimization (configurable output fields)
  - Right to access (Article 15: `--export-scan-data`)
  - Right to erasure (Article 17: `--delete-scan-data`)
  - Data retention (configurable `retention_days`)

- **[CCPA (California Consumer Privacy Act)](https://oag.ca.gov/privacy/ccpa)** - California, USA
  Effective: January 1, 2020
  Consumer data privacy rights similar to GDPR.

### Payment Card Security

- **[PCI DSS (Payment Card Industry Data Security Standard)](https://www.pcisecuritystandards.org/)** - PCI Security Standards Council
  Current Version: 4.0 (March 2022)
  ProRT-IP supports PCI DSS compliance:
  - **Requirement 11.2:** Quarterly external vulnerability scans
    ```bash
    prtip -sS -sV -p- --scan-type quarterly-external target.com
    ```
  - **Requirement 11.3:** Annual penetration testing
    ```bash
    prtip -sS -sV -O -A -p- --scan-type annual-pentest target.com
    ```

### Government Standards

- **[NIST Cybersecurity Framework (CSF)](https://www.nist.gov/cyberframework)** - U.S. National Institute of Standards and Technology
  Version: 2.0 (February 2024)
  Five core functions: Identify, Protect, Detect, Respond, Recover.
  ProRT-IP supports **Identify** and **Protect** functions:
  - ID.AM-1: Physical devices and systems inventoried (asset discovery)
  - ID.RA-1: Vulnerabilities identified and documented
  - PR.AC-1: Identities and credentials managed (audit logging)
  - PR.AC-5: Network integrity protected (network segmentation verification)

- **[NIST SP 800-115](https://csrc.nist.gov/publications/detail/sp/800-115/final)** - Technical Guide to Information Security Testing and Assessment
  Published: September 2008
  Methodology for penetration testing (network scanning is Phase 1: Reconnaissance).

- **[NIST SP 800-53](https://csrc.nist.gov/publications/detail/sp/800-53/rev-5/final)** - Security and Privacy Controls
  Revision: 5 (September 2020)
  Comprehensive security controls for federal systems (ProRT-IP audit logging supports control AU-2).

### International Standards

- **[ISO/IEC 27001:2022](https://www.iso.org/standard/27001)** - Information Security Management
  International standard for ISMS (Information Security Management Systems).

- **[ISO/IEC 27002:2022](https://www.iso.org/standard/75652.html)** - Information Security Controls
  Code of practice for information security controls (114 controls across 4 themes).

---

## Academic Papers

Research publications on network scanning, performance optimization, and security.

### Network Scanning Research

- **ZMap: Fast Internet-Wide Scanning and Its Security Applications**
  Zakir Durumeric, Eric Wustrow, J. Alex Halderman
  USENIX Security Symposium, 2013
  [PDF](https://www.usenix.org/system/files/conference/usenixsecurity13/sec13-paper_durumeric.pdf)
  Internet-wide scanning (1.4B IPv4 addresses in 45 minutes), ZMap architecture.

- **A Search Engine Backed by Internet-Wide Scanning**
  Zakir Durumeric, David Adrian, Ariana Mirian, Michael Bailey, J. Alex Halderman
  ACM CCS, 2015
  [PDF](https://censys.io/)
  Censys.io research platform for Internet measurements.

- **The Matter of Heartbleed**
  Zakir Durumeric, Frank Li, James Kasten, Johanna Amann, Jethro Beekman, Mathias Payer, Nicolas Weaver, David Adrian, Vern Paxson, Michael Bailey, J. Alex Halderman
  ACM IMC, 2014
  [PDF](<!-- PDF link unavailable -->)
  Internet-wide scanning to measure Heartbleed vulnerability exposure.

### IPv6 Security

- **Network Reconnaissance in IPv6 Networks: A Comprehensive Survey**
  Fernando Gont, Tim Chown
  RFC 7707 (Informational), March 2016
  [RFC 7707](https://www.rfc-editor.org/rfc/rfc7707.html)
  Analysis of IPv6 address scanning challenges (2^64 addresses per subnet).

- **IPv6 Security: Attacks and Countermeasures in a Nutshell**
  Johanna Ullrich, Edgar Weippl
  USENIX ;login:, 2014
  [PDF](<!-- PDF link unavailable -->)
  IPv6-specific attack vectors (ICMPv6, NDP, Router Advertisements).

### Performance Optimization

- **The Click Modular Router**
  Eddie Kohler, Robert Morris, Benjie Chen, John Jannotti, M. Frans Kaashoek
  ACM Transactions on Computer Systems, 2000
  [PDF](https://pdos.csail.mit.edu/papers/click:tocs00/paper.pdf)
  Modular packet processing architecture (influenced ProRT-IP's zero-copy design).

- **Fast and Memory-Efficient Network Scanning using Masscan**
  Robert Graham
  DEF CON 22, 2014
  [Slides](<!-- PDF link unavailable -->)
  Masscan's transmission scheduler and asynchronous I/O design.

### Rust Async Performance

- **Tokio: A Runtime for Writing Reliable, Asynchronous, and Slim Applications**
  Carl Lerche
  Rust Conference, 2018
  [Video](https://www.youtube.com/watch?v=jNNz8ZJTbfQ)
  Tokio design principles (ProRT-IP uses Tokio multi-threaded runtime).

---

## Books

Authoritative books on network security, protocol analysis, and penetration testing.

### Network Scanning

- **Nmap Network Scanning: Official Nmap Project Guide to Network Discovery and Security Scanning**
  Gordon Lyon (Fyodor)
  Published: 2009
  ISBN: 978-0979958717
  468 pages, comprehensive Nmap reference (ProRT-IP compatibility guide).

- **Practical Packet Analysis: Using Wireshark to Solve Real-World Network Problems (3rd Edition)**
  Chris Sanders
  Published: 2017
  ISBN: 978-1593278021
  368 pages, packet analysis with Wireshark and tcpdump.

### Network Protocols

- **TCP/IP Illustrated, Volume 1: The Protocols (2nd Edition)**
  Kevin R. Fall, W. Richard Stevens
  Published: 2011
  ISBN: 978-0321336316
  1,017 pages, comprehensive TCP/IP protocol reference.

- **IPv6 Fundamentals: A Straightforward Approach to Understanding IPv6 (2nd Edition)**
  Rick Graziani
  Published: 2017
  ISBN: 978-1587144776
  704 pages, IPv6 addressing, routing, security.

### Penetration Testing

- **The Hacker Playbook 3: Practical Guide to Penetration Testing**
  Peter Kim
  Published: 2018
  ISBN: 978-1980901761
  392 pages, penetration testing methodology and tools.

- **Metasploit: The Penetration Tester's Guide**
  David Kennedy, Jim O'Gorman, Devon Kearns, Mati Aharoni
  Published: 2011
  ISBN: 978-1593272883
  328 pages, Metasploit framework for exploitation and post-exploitation.

### Rust Programming

- **The Rust Programming Language (2nd Edition)**
  Steve Klabnik, Carol Nichols
  Published: 2023
  ISBN: 978-1718503106
  560 pages, official Rust book ("The Book").

- **Programming Rust: Fast, Safe Systems Development (2nd Edition)**
  Jim Blandy, Jason Orendorff, Leonora F. S. Tindall
  Published: 2021
  ISBN: 978-1492052593
  738 pages, advanced Rust systems programming.

- **Asynchronous Programming in Rust**
  Maxwell Flitton
  Published: 2024
  ISBN: 978-1098149000
  350 pages, async/await, Tokio, Futures (ProRT-IP architecture guide).

---

## Rust Ecosystem

Libraries and frameworks used by ProRT-IP.

### Core Libraries

- **[tokio](https://tokio.rs/)** - Asynchronous Runtime
  Version: 1.35+
  Multi-threaded work-stealing scheduler, async I/O (ProRT-IP core dependency).
  - [Tokio Docs](https://docs.rs/tokio/latest/tokio/)
  - [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

- **[pnet](https://github.com/libpnet/libpnet)** - Packet Manipulation
  Version: 0.34+
  Cross-platform packet construction and parsing (TCP, UDP, ICMP, Ethernet, IPv4, IPv6).
  - [pnet Docs](https://docs.rs/pnet/latest/pnet/)

- **[clap](https://clap.rs/)** - Command-Line Argument Parsing
  Version: 4.4+
  Derive-based CLI parsing with 50+ Nmap-compatible flags.
  - [Clap Docs](https://docs.rs/clap/latest/clap/)

- **[sqlx](https://github.com/launchbadge/sqlx)** - Async SQL Toolkit
  Version: 0.7+
  Compile-time verified SQL queries, SQLite support for scan results.
  - [SQLx Docs](https://docs.rs/sqlx/latest/sqlx/)

### TUI and Rendering

- **[ratatui](https://ratatui.rs/)** - Terminal User Interface Library
  Version: 0.29+
  Immediate-mode rendering, 60 FPS performance (ProRT-IP TUI framework).
  - [ratatui Docs](https://docs.rs/ratatui/latest/ratatui/)
  - [ratatui Book](https://ratatui.rs/)

- **[crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal Manipulation
  Version: 0.28+
  Cross-platform terminal control (cursor, colors, input events).
  - [crossterm Docs](https://docs.rs/crossterm/latest/crossterm/)

### Security

- **[rustls](https://github.com/rustls/rustls)** - Modern TLS Library
  Version: 0.21+
  Memory-safe TLS implementation (ProRT-IP uses for HTTPS service detection).
  - [rustls Docs](https://docs.rs/rustls/latest/rustls/)

- **[x509-parser](https://github.com/rusticata/x509-parser)** - X.509 Certificate Parser
  Version: 0.16+
  ProRT-IP parses TLS certificates in 1.33μs (X.509v3 format).
  - [x509-parser Docs](https://docs.rs/x509-parser/latest/x509_parser/)

### Lua Integration

- **[mlua](https://github.com/mlua-rs/mlua)** - Lua Bindings
  Version: 0.11+
  Lua 5.4 integration for ProRT-IP plugin system (sandboxing, capabilities).
  - [mlua Docs](https://docs.rs/mlua/latest/mlua/)

### Testing

- **[proptest](https://github.com/proptest-rs/proptest)** - Property-Based Testing
  Version: 1.4+
  Generative testing with shrinking (ProRT-IP uses for CIDR parsing, port validation).
  - [proptest Docs](https://docs.rs/proptest/latest/proptest/)

- **[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)** - Fuzzing Tool
  Version: 0.11+
  libFuzzer-based fuzzing (ProRT-IP: 230M+ executions, 0 crashes).
  - [Rust Fuzz Book](https://rust-fuzz.github.io/book/)

- **[cargo-tarpaulin](https://github.com/xd009642/tarpaulin)** - Code Coverage
  Version: 0.31+
  Line coverage measurement (ProRT-IP: 54.92% coverage, 2,361 tests).
  - [tarpaulin Docs](https://github.com/xd009642/tarpaulin/blob/develop/README.md)

---

## ProRT-IP Resources

Official project links and community resources.

### Official Documentation

- **[GitHub Repository](https://github.com/doublegate/ProRT-IP)**
  Source code, issues, pull requests, releases.

- **[Release Notes](https://github.com/doublegate/ProRT-IP/releases)**
  Detailed changelog for each version (v0.1.0 - v0.5.2+).

- **[Issue Tracker](https://github.com/doublegate/ProRT-IP/issues)**
  Bug reports, feature requests, discussions.

- **[GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)**
  Community Q&A, announcements, ideas.

### Contributing

- **[Contributing Guide](https://github.com/doublegate/ProRT-IP/blob/main/CONTRIBUTING.md)**
  Development workflow, code style, testing requirements.

- **[Security Policy](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md)**
  Vulnerability disclosure process, supported versions.

- **[Code of Conduct](https://github.com/doublegate/ProRT-IP/blob/main/CODE_OF_CONDUCT.md)**
  Community standards and expectations.

### Documentation Sites

- **[User Guide](../user-guide/index.md)** - Getting started, CLI reference, tutorials
- **[Features](../features/index.md)** - Detailed feature documentation
- **[Advanced Topics](../advanced/index.md)** - Performance tuning, optimization
- **[Reference](../reference/index.md)** - Technical specification, API reference, FAQ
- **[Development](../development/index.md)** - Architecture, testing, CI/CD

### Community

- **[Discord Server](https://discord.gg/prort-ip)** *(Planned)*
  Real-time chat, support, development discussions.

- **[Reddit Community](https://reddit.com/r/prortip)** *(Planned)*
  News, tutorials, user stories.

---

## External Learning Resources

Additional resources for learning network scanning and security concepts.

### Online Courses

- **[Cybrary: Network Security](https://www.cybrary.it/catalog/cybersecurity/)**
  Free and paid courses on network security fundamentals.

- **[TryHackMe: Network Security](https://tryhackme.com/paths)**
  Hands-on labs for network scanning and penetration testing.

- **[HackTheBox Academy](https://academy.hackthebox.com/)**
  Penetration testing training with practical exercises.

### YouTube Channels

- **[LiveOverflow](https://www.youtube.com/c/LiveOverflow)**
  Binary exploitation, web security, CTF walkthroughs.

- **[IppSec](https://www.youtube.com/c/ippsec)**
  HackTheBox walkthroughs with detailed explanations.

- **[NetworkChuck](https://www.youtube.com/c/NetworkChuck)**
  Networking fundamentals, labs, certifications.

### Podcasts

- **[Darknet Diaries](https://darknetdiaries.com/)**
  True stories from the dark side of the Internet.

- **[Security Now](https://twit.tv/shows/security-now)**
  Weekly cybersecurity news and analysis (Steve Gibson).

---

## Version Information

- **Document Version:** 1.0.0
- **Last Updated:** 2025-11-15
- **ProRT-IP Version:** v0.5.2+
- **Phase:** Phase 6 Sprint 6.2 COMPLETE

---

## See Also

- [Glossary](glossary.md) - Comprehensive term definitions
- [Technical Specification](../reference/tech-spec-v2.md) - Complete technical reference
- [Comparison: Nmap](../reference/comparisons/nmap.md) - ProRT-IP vs Nmap
- [Comparison: Masscan](../reference/comparisons/masscan.md) - ProRT-IP vs Masscan
- [Security Model](../security/security-model.md) - Security architecture
