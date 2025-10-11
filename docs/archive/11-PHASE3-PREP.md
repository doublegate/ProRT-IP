# Phase 3: Detection Systems - Preparation Guide

**Status:** Ready to Begin
**Prerequisites:** ✅ All Complete (Phase 1, Phase 2, Enhancement Cycles 1-5)
**Target Start:** 2025-10-08
**Target Completion:** Week 10 (approximately 4 weeks)

---

## Prerequisites Checklist

- ✅ Phase 1: Core Infrastructure complete
- ✅ Phase 2: Advanced Scanning complete
- ✅ TCP SYN scanning operational
- ✅ UDP scanning operational
- ✅ Packet construction infrastructure ready
- ✅ Network interface detection ready
- ✅ Progress tracking ready
- ✅ Error handling ready
- ✅ 391 tests passing (100% success rate)

---

## Phase 3 Overview

Phase 3 focuses on implementing detection systems for OS fingerprinting, service identification, and banner grabbing.

### Sprint 3.1: OS Fingerprinting Foundation (Week 7)

**Deliverables:**

- OS fingerprint database schema
- nmap-os-db parser
- 16-probe sequence implementation
- ISN analysis algorithms
- TCP characteristics analysis

**Estimated Lines:** ~800-1000
**Estimated Tests:** 20-25

### Sprint 3.2: Service Detection Framework (Week 8)

**Deliverables:**

- nmap-service-probes parser
- Service detection engine
- Probe matching logic
- Intensity level support (0-9)
- NULL probe and GetRequest handling

**Estimated Lines:** ~600-800
**Estimated Tests:** 15-20

### Sprint 3.3: Banner Grabbing & Integration (Week 9)

**Deliverables:**

- Banner grabbing module
- Protocol-specific handlers (HTTP, FTP, SSH, SMTP)
- SSL/TLS support
- Integration with scanners
- CLI flags for detection features

**Estimated Lines:** ~400-600
**Estimated Tests:** 12-18

### Sprint 3.4: Testing & Refinement (Week 10)

**Deliverables:**

- Integration testing
- Database updates
- Performance tuning
- Documentation
- Bug fixes

**Estimated Lines:** ~200-400 (tests, docs)
**Estimated Tests:** 10-15

---

## Technical Foundation

### Available Infrastructure

**From Phase 2:**

- TCP SYN scanner (syn_scanner.rs)
- UDP scanner (udp_scanner.rs)
- Packet builder (packet_builder.rs)
- Protocol payloads (protocol_payloads.rs)
- Timing templates (timing.rs)

**From Enhancements:**

- SipHash for checksums (crypto.rs)
- Progress tracking (progress.rs)
- Error categorization (errors.rs)
- Resource management (resource_limits.rs)
- Interface detection (interface.rs)

### Reference Materials

**In code_ref/:**

- nmap: OS fingerprinting implementation (FPEngine.*, os-db files)
- nmap: Service detection (service-probes, ServiceProbe.*)
- nmap: Banner grabbing patterns
- RustScan: Service detection integration
- naabu: Banner grabbing implementation

**Documentation:**

- docs/02-TECHNICAL-SPECS.md: Protocol specifications
- docs/04-IMPLEMENTATION-GUIDE.md: Code patterns
- ref-docs/: Original technical specifications

---

## Implementation Tasks

### Sprint 3.1: OS Fingerprinting Foundation

**Database Schema (Week 7, Day 1-2):**

- [ ] Design fingerprint storage structure
- [ ] Parse nmap-os-db format
- [ ] Create in-memory fingerprint database
- [ ] Implement fingerprint lookup functions

**16-Probe Sequence (Week 7, Day 3-4):**

- [ ] 6 TCP SYN probes to open port
  - [ ] SEQ probe (6 packets, different sequences)
  - [ ] OPS probe (TCP options variation)
  - [ ] WIN probe (window size variation)
  - [ ] T2-T7 probes (closed port tests)
- [ ] 2 ICMP echo requests
  - [ ] IE1 (ICMP echo with specific TOS/DF)
  - [ ] IE2 (ICMP echo variation)
- [ ] 1 ECN probe (ECN support test)
- [ ] 1 UDP probe to closed port

**Analysis Algorithms (Week 7, Day 5):**

- [ ] ISN (Initial Sequence Number) analysis
  - [ ] GCD calculation
  - [ ] ISR (ISN rate) detection
  - [ ] TI/CI/II patterns
- [ ] TCP timestamp parsing
- [ ] TCP option ordering extraction
- [ ] Window size analysis
- [ ] IP ID generation pattern detection

### Sprint 3.2: Service Detection Framework

**Service Probe Database (Week 8, Day 1-2):**

- [ ] Parse nmap-service-probes format
- [ ] Extract probe definitions
- [ ] Extract match patterns (regex)
- [ ] Build probe selection logic

**Detection Engine (Week 8, Day 3-4):**

- [ ] NULL probe implementation (self-announcing)
- [ ] GetRequest probe for HTTP-like services
- [ ] Protocol-specific probe sending
- [ ] Response matching with regex
- [ ] Intensity level support (0-9)
- [ ] Fallback chain logic

**Service Identification (Week 8, Day 5):**

- [ ] Version string extraction
- [ ] CPE (Common Platform Enumeration) output
- [ ] Confidence scoring
- [ ] Soft matching for partial matches

### Sprint 3.3: Banner Grabbing & Integration

**Banner Grabbing (Week 9, Day 1-2):**

- [ ] TCP banner grabber
- [ ] Timeout handling
- [ ] Buffer management
- [ ] Protocol-specific handlers:
  - [ ] HTTP/HTTPS (GET request)
  - [ ] FTP (wait for banner)
  - [ ] SSH (version string)
  - [ ] SMTP (HELO command)
  - [ ] POP3/IMAP (wait for banner)

**SSL/TLS Support (Week 9, Day 3):**

- [ ] TLS handshake implementation
- [ ] Certificate parsing
- [ ] SNI (Server Name Indication)
- [ ] Protocol version detection

**Scanner Integration (Week 9, Day 4-5):**

- [ ] Integrate with SYN scanner
- [ ] Integrate with Connect scanner
- [ ] CLI flags:
  - [ ] `-sV` (service detection)
  - [ ] `-O` (OS detection)
  - [ ] `--version-intensity 0-9`
  - [ ] `--osscan-limit` (only scan with open ports)
- [ ] Output formatting for detection results

### Sprint 3.4: Testing & Refinement

**Integration Testing (Week 10, Day 1-2):**

- [ ] Test OS detection against known systems
- [ ] Test service detection on common services
- [ ] Test banner grabbing with SSL
- [ ] Cross-platform testing

**Documentation (Week 10, Day 3):**

- [ ] Update API documentation
- [ ] Add usage examples
- [ ] Document database formats
- [ ] Update CHANGELOG

**Performance & Bug Fixes (Week 10, Day 4-5):**

- [ ] Profile detection performance
- [ ] Optimize database lookups
- [ ] Fix reported bugs
- [ ] Code review and cleanup

---

## Success Criteria

Phase 3 will be considered complete when:

- ✅ OS fingerprinting produces matches from nmap-os-db
- ✅ Service detection identifies common services (HTTP, SSH, FTP, etc.)
- ✅ Banner grabbing works for text-based protocols
- ✅ Integration with existing scanners seamless
- ✅ CLI supports detection features with proper flags
- ✅ All tests pass (target: 450+ total tests)
- ✅ Documentation updated comprehensively

---

## Key Design Decisions

### OS Fingerprinting Approach

- **16-probe sequence:** Follows Nmap's proven methodology
- **Weighted scoring:** Allows fuzzy matching for accuracy
- **Database format:** Compatible with nmap-os-db for easy updates
- **Probe optimization:** Send only necessary probes based on initial results

### Service Detection Strategy

- **NULL probe first:** Many services self-announce (SMTP, FTP, SSH)
- **Intensity levels:** Balance accuracy vs speed
- **Probe fallbacks:** Use multiple probes for ambiguous services
- **Regex matching:** Flexible version string extraction

### Banner Grabbing Implementation

- **Non-blocking I/O:** Async with timeouts
- **Protocol awareness:** Different strategies per protocol
- **SSL/TLS support:** Handle encrypted services
- **Buffer limits:** Prevent memory exhaustion

---

## Testing Strategy

### Unit Tests

- Fingerprint parsing and matching
- Probe generation
- Response interpretation
- Banner parsing

### Integration Tests

- End-to-end OS detection
- Service detection on mock servers
- SSL banner grabbing
- Cross-scanner integration

### System Tests

- Detection against real systems
- Performance benchmarks
- Accuracy validation
- Edge case handling

---

## Dependencies

**Existing:**

- tokio (async runtime)
- pnet_packet (packet manipulation)
- sqlx (database)
- clap (CLI)

**New (likely):**

- regex (pattern matching)
- openssl or rustls (SSL/TLS)
- x509-parser (certificate parsing)

---

## Risk Mitigation

**Potential Risks:**

1. **Database size:** nmap-os-db is large
   - *Mitigation:* Use efficient data structures, consider compression
2. **Probe timing:** Too fast may trigger IDS
   - *Mitigation:* Respect timing templates, add delays
3. **Accuracy:** Fingerprint matching may be fuzzy
   - *Mitigation:* Implement confidence scores, multiple probes
4. **SSL complexity:** TLS handshake can be tricky
   - *Mitigation:* Use well-tested libraries (rustls/openssl)

---

## Next Steps

1. Review Phase 3 sprint plans in docs/01-ROADMAP.md
2. Familiarize with nmap-os-db format
3. Familiarize with nmap-service-probes format
4. Begin Sprint 3.1: OS Fingerprinting Foundation

**Ready to begin Phase 3 implementation!**
