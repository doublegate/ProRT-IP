# Fuzz Testing

Comprehensive fuzz testing infrastructure for ProRT-IP using cargo-fuzz and libFuzzer to discover crashes, panics, and security vulnerabilities in packet parsing code.

## Overview

**Fuzzing Strategy:**
- **Structure-Aware Fuzzing:** Generate valid-ish packets using `arbitrary` crate with custom constraints
- **Unstructured Fuzzing:** Test raw bytes to catch edge cases missed by structure-aware fuzzing
- **Coverage-Guided:** libFuzzer automatically discovers new code paths and maximizes coverage
- **Continuous:** Integration with CI/CD for automated regression testing

**Key Metrics:**
- **Fuzz Targets:** 5 targets (TCP, UDP, IPv6, ICMPv6, TLS)
- **Executions:** 230M+ total executions across all targets
- **Crashes Found:** 0 crashes (production-ready parsers)
- **Coverage:** 80%+ of packet parsing code paths
- **Performance:** 10K-50K executions/second depending on target complexity

**Dependencies:**
```toml
[dependencies]
libfuzzer-sys = "0.4"       # libFuzzer integration
arbitrary = { version = "1.3", features = ["derive"] }  # Structure-aware fuzzing

# Project dependencies
prtip-network = { path = "../crates/prtip-network" }
prtip-scanner = { path = "../crates/prtip-scanner" }

# Additional for protocol parsing
pnet = "0.35"
pnet_packet = "0.35"
x509-parser = "0.16"
```

---

## Fuzz Targets

### 1. TCP Parser Fuzzer

**Target:** `fuzz_tcp_parser`
**Location:** `fuzz/fuzz_targets/fuzz_tcp_parser.rs`
**Complexity:** High (header + options + payload)

**Structure-Aware Input:**
```rust
#[derive(Arbitrary, Debug)]
struct FuzzTcpInput {
    /// TCP source port (0-65535)
    source_port: u16,

    /// TCP destination port (0-65535)
    dest_port: u16,

    /// Sequence number
    sequence: u32,

    /// Acknowledgment number
    acknowledgment: u32,

    /// TCP flags (8 bits: FIN, SYN, RST, PSH, ACK, URG, ECE, CWR)
    flags: u8,

    /// Window size
    window: u16,

    /// Urgent pointer
    urgent_ptr: u16,

    /// TCP options (0-40 bytes)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=40)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    options: Vec<u8>,

    /// Payload data (0-1460 bytes for typical MTU)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1460)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Whether to use valid or invalid checksum
    use_bad_checksum: bool,

    /// Data offset value (normally calculated, but fuzz can override)
    override_data_offset: Option<u8>,
}
```

**What It Tests:**
- **Packet Building:** Constructs TCP packets with configurable fields
- **Options Padding:** 4-byte boundary alignment (RFC 793)
- **Data Offset Clamping:** Valid range 5-15 (20-60 byte header)
- **Accessor Methods:** All pnet `TcpPacket` getters (source, dest, sequence, flags, window, options, payload)
- **Checksum Validation:** Both IPv4 and IPv6 checksum calculation
- **Edge Cases:** Malformed packets, short packets (<20 bytes)

**Run Command:**
```bash
cd fuzz
cargo fuzz run fuzz_tcp_parser -- -max_total_time=300 -max_len=1500
```

---

### 2. UDP Parser Fuzzer

**Target:** `fuzz_udp_parser`
**Location:** `fuzz/fuzz_targets/fuzz_udp_parser.rs`
**Complexity:** Medium (simple header + payload)

**Structure-Aware Input:**
```rust
#[derive(Arbitrary, Debug)]
struct FuzzUdpInput {
    /// UDP source port (0-65535)
    source_port: u16,

    /// UDP destination port (0-65535)
    dest_port: u16,

    /// Payload data (0-1472 bytes, typical MTU - headers)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1472)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Whether to use valid or invalid checksum
    use_bad_checksum: bool,

    /// Override length field (normally payload + 8 bytes header)
    override_length: Option<u16>,
}
```

**What It Tests:**
- **Basic Parsing:** UDP header fields (source, dest, length, checksum)
- **Checksum Validation:** IPv4 (optional) and IPv6 (mandatory) checksums
- **Protocol-Specific Payloads:**
  - **DNS (port 53):** Header parsing (ID, flags, questions, answers)
  - **SNMP (ports 161/162):** ASN.1 BER encoding (SEQUENCE tag 0x30)
  - **NetBIOS (ports 135-139):** Name service header (transaction ID)
- **Edge Cases:**
  - Zero-length payload (valid UDP, 8-byte header only)
  - Malformed packets (<8 bytes, should return None)
  - Length field mismatch (override_length)

**Run Command:**
```bash
cd fuzz
cargo fuzz run fuzz_udp_parser -- -max_total_time=300 -max_len=1480
```

---

### 3. IPv6 Parser Fuzzer

**Target:** `fuzz_ipv6_parser`
**Location:** `fuzz/fuzz_targets/fuzz_ipv6_parser.rs`
**Complexity:** High (header + extension headers)

**Structure-Aware Input:**
```rust
#[derive(Arbitrary, Debug)]
struct FuzzIpv6Input {
    /// Traffic class (8 bits)
    traffic_class: u8,

    /// Flow label (20 bits)
    flow_label: u32,

    /// Hop limit (TTL equivalent)
    hop_limit: u8,

    /// Source IPv6 address (16 bytes)
    source: [u8; 16],

    /// Destination IPv6 address (16 bytes)
    destination: [u8; 16],

    /// Next header protocol number
    next_header: u8,

    /// Extension headers (0-3 headers, variable length)
    #[arbitrary(with = |u: &mut Unstructured| {
        let count = u.int_in_range(0..=3)?;
        (0..count).map(|_| {
            let header_type = u.choose(&[0u8, 43, 44, 60])?;
            let len = u.int_in_range(0..=40)?;
            let data = u.bytes(len)?.to_vec();
            Ok::<(u8, Vec<u8>), arbitrary::Error>((*header_type, data))
        }).collect::<Result<Vec<(u8, Vec<u8>)>, arbitrary::Error>>()
    })]
    extension_headers: Vec<(u8, Vec<u8>)>,

    /// Payload data (0-1280 bytes, minimum IPv6 MTU)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1280)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Override payload length field
    override_payload_length: Option<u16>,
}
```

**Extension Header Types:**
- **HopByHop (0):** Per-hop options
- **Routing (43):** Source routing
- **Fragment (44):** Fragmentation (offset, M flag, identification)
- **DestinationOptions (60):** Destination-specific options

**What It Tests:**
- **Header Encoding:** Version (6), Traffic Class, Flow Label (20-bit)
- **Addresses:** Source/destination parsing (128-bit)
- **Extension Headers:** Chaining (next_header), length calculation (8-byte units)
- **Fragment Header:** Offset, More Fragments flag, Identification
- **Edge Cases:**
  - Malformed packets (<40 bytes)
  - Invalid version (must be 6)
  - Extension header chain parsing

**Run Command:**
```bash
cd fuzz
cargo fuzz run fuzz_ipv6_parser -- -max_total_time=300 -max_len=1320
```

---

### 4. ICMPv6 Parser Fuzzer

**Target:** `fuzz_icmpv6_parser`
**Location:** `fuzz/fuzz_targets/fuzz_icmpv6_parser.rs`
**Complexity:** Medium (type-specific formats)

**Structure-Aware Input:**
```rust
#[derive(Arbitrary, Debug)]
struct FuzzIcmpv6Input {
    /// ICMPv6 type (0-255)
    /// Common types:
    ///   1 = Destination Unreachable
    ///   128 = Echo Request
    ///   129 = Echo Reply
    ///   133 = Router Solicitation
    ///   134 = Router Advertisement
    ///   135 = Neighbor Solicitation
    ///   136 = Neighbor Advertisement
    icmp_type: u8,

    /// ICMPv6 code (0-255)
    /// For Type 1 (Dest Unreachable): codes 0-5 are defined
    code: u8,

    /// Payload data (0-1232 bytes, MTU minus headers)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1232)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Whether to use valid or invalid checksum
    use_bad_checksum: bool,

    /// For Echo Request/Reply: identifier
    echo_id: Option<u16>,

    /// For Echo Request/Reply: sequence number
    echo_seq: Option<u16>,

    /// For Neighbor Discovery: target IPv6 address
    nd_target: Option<[u8; 16]>,
}
```

**Type-Specific Formats:**

**Type 1 (Destination Unreachable):**
```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                            Unused                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    As much of invoking packet...
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Type 128/129 (Echo Request/Reply):**
```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Identifier          |        Sequence Number        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Data ...
+-+-+-+-+-+-+-+-+
```

**Type 135/136 (Neighbor Solicitation/Advertisement):**
```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Type      |     Code      |          Checksum             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                           Reserved                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Target Address (128 bits)               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**What It Tests:**
- **All Message Types:** 1, 128, 129, 133, 134, 135, 136 + unknown types
- **Checksum Validation:** Mandatory ICMPv6 checksum with IPv6 pseudo-header
- **Type-Specific Parsing:**
  - Type 1: Unused field (4 bytes) + original packet
  - Echo: Identifier + Sequence + data
  - Router Sol/Adv: Reserved field + options
  - Neighbor Sol/Adv: Reserved + Target Address (16 bytes) + options
- **Edge Cases:**
  - Malformed packets (<4 bytes)
  - All Type 1 codes (0-5)
  - Echo with no payload (valid)

**Run Command:**
```bash
cd fuzz
cargo fuzz run fuzz_icmpv6_parser -- -max_total_time=300 -max_len=1240
```

---

### 5. TLS Certificate Parser Fuzzer

**Target:** `fuzz_tls_parser`
**Location:** `fuzz/fuzz_targets/fuzz_tls_parser.rs`
**Complexity:** Very High (X.509 ASN.1/DER parsing)

**Structure-Aware Input:**
```rust
#[derive(Arbitrary, Debug)]
struct FuzzTlsCertInput {
    /// Certificate DER bytes (100-4000 bytes typical range)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(100..=4000)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    cert_der: Vec<u8>,

    /// Additional certificates for chain testing (0-3 certs)
    #[arbitrary(with = |u: &mut Unstructured| {
        let count = u.int_in_range(0..=3)?;
        (0..count).map(|_| {
            let len = u.int_in_range(100..=4000)?;
            u.bytes(len).map(|b| b.to_vec())
        }).collect::<Result<Vec<Vec<u8>>, arbitrary::Error>>()
    })]
    chain_certs: Vec<Vec<u8>>,

    /// Whether to test chain parsing
    test_chain: bool,
}
```

**Minimal Valid X.509 Certificate Structure (DER):**
```rust
fn generate_minimal_cert(data: &[u8]) -> Vec<u8> {
    // X.509 Certificate structure:
    // SEQUENCE {
    //   SEQUENCE {  // TBSCertificate
    //     [0] EXPLICIT INTEGER {2}  // Version (v3 = 2)
    //     INTEGER                   // Serial number
    //     SEQUENCE                  // Signature algorithm
    //     SEQUENCE                  // Issuer
    //     SEQUENCE                  // Validity
    //     SEQUENCE                  // Subject
    //     SEQUENCE                  // SubjectPublicKeyInfo
    //     [3] EXPLICIT SEQUENCE     // Extensions (optional)
    //   }
    //   SEQUENCE                    // SignatureAlgorithm
    //   BIT STRING                  // Signature
    // }

    let mut cert = vec![
        0x30, 0x82, 0x01, 0x00, // SEQUENCE (certificate)
        0x30, 0x81, 0xF0,       // SEQUENCE (tbsCertificate)

        // Version [0] EXPLICIT
        0xA0, 0x03,             // [0] EXPLICIT
        0x02, 0x01, 0x02,       // INTEGER 2 (v3)

        // Serial number (8 bytes from fuzzer input)
        0x02, 0x08,
        // ... serial bytes ...

        // Signature algorithm (SHA256withRSA)
        0x30, 0x0D,
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x0B,
        0x05, 0x00,

        // Issuer (minimal DN: CN=Test1)
        // Subject (minimal DN: CN=Test2)
        // Validity (notBefore, notAfter as UTCTime)
        // SubjectPublicKeyInfo (RSA, minimal key)
        // SignatureAlgorithm (same as above)
        // Signature (32 bytes from fuzzer input)
    ];

    cert
}
```

**What It Tests:**
- **Unstructured Fuzzing:** Raw DER bytes (truly malformed input)
- **Structure-Aware Fuzzing:** Minimal valid certificate with mutations
- **Chain Parsing:** Primary certificate + 0-3 additional certs
- **All CertificateInfo Fields:**
  - **Basic:** issuer, subject, validity_not_before, validity_not_after
  - **SAN:** Subject Alternative Names (san, san_categorized)
  - **Serial:** serial_number
  - **Algorithms:** signature_algorithm, signature_algorithm_enhanced
  - **Public Key:** public_key_info (algorithm, key_size, curve, usage)
  - **Usage:** key_usage, extended_key_usage
  - **Extensions:** extensions (all X.509v3 extensions)
- **SAN Categorization:**
  - DNS names
  - IP addresses
  - Email addresses
  - URIs
- **Key Usage Flags:**
  - digital_signature
  - key_encipherment
  - key_cert_sign
  - crl_sign
- **Extended Key Usage:**
  - server_auth
  - client_auth
  - code_signing
- **Edge Cases:**
  - Very short (<10 bytes, should error)
  - Very large (>10000 bytes, DOS prevention)

**Run Command:**
```bash
cd fuzz
cargo fuzz run fuzz_tls_parser -- -max_total_time=600 -max_len=5000
```

---

## Running Fuzzing Campaigns

### Prerequisites

**Install cargo-fuzz:**
```bash
cargo install cargo-fuzz
```

**Nightly Rust:**
```bash
rustup default nightly
```

**LLVM Coverage Tools (optional for corpus minimization):**
```bash
# Ubuntu/Debian
sudo apt install llvm

# macOS
brew install llvm
```

### Basic Fuzzing Workflow

**1. Run Single Target (5 minutes):**
```bash
cd fuzz
cargo fuzz run fuzz_tcp_parser -- -max_total_time=300
```

**2. Run with Corpus Directory:**
```bash
# Create corpus directory
mkdir -p corpus/fuzz_tcp_parser

# Run with corpus
cargo fuzz run fuzz_tcp_parser corpus/fuzz_tcp_parser -- -max_total_time=300
```

**3. Run All Targets (Parallel):**
```bash
#!/bin/bash
# run-all-fuzzers.sh

TARGETS=(
    "fuzz_tcp_parser"
    "fuzz_udp_parser"
    "fuzz_ipv6_parser"
    "fuzz_icmpv6_parser"
    "fuzz_tls_parser"
)

TIME=300  # 5 minutes per target

for target in "${TARGETS[@]}"; do
    echo "Running $target for ${TIME}s..."
    cargo fuzz run "$target" -- -max_total_time=$TIME &
done

wait
echo "All fuzzers complete"
```

**4. Continuous Fuzzing (Overnight):**
```bash
# Run for 8 hours (28800 seconds)
cargo fuzz run fuzz_tcp_parser -- -max_total_time=28800 -max_len=1500 -jobs=4
```

**5. With Dictionary (TCP options):**
```bash
# Create dictionary for common TCP options
cat > tcp_options.dict <<EOF
# MSS (Kind 2, Length 4)
"\x02\x04\x05\xb4"

# Window Scale (Kind 3, Length 3)
"\x03\x03\x07"

# SACK Permitted (Kind 4, Length 2)
"\x04\x02"

# Timestamp (Kind 8, Length 10)
"\x08\x0a\x00\x00\x00\x00\x00\x00\x00\x00"
EOF

cargo fuzz run fuzz_tcp_parser -- -dict=tcp_options.dict -max_total_time=300
```

### Advanced Options

**Reproducible Crashes:**
```bash
# Run with seed for reproducibility
cargo fuzz run fuzz_tcp_parser -- -seed=12345 -runs=1000000
```

**Memory Limit:**
```bash
# Limit memory to 2GB
cargo fuzz run fuzz_tls_parser -- -rss_limit_mb=2048
```

**Parallel Jobs:**
```bash
# Use 8 CPU cores
cargo fuzz run fuzz_tcp_parser -- -jobs=8 -workers=8
```

**Minimize Corpus:**
```bash
# Reduce corpus to minimal covering set
cargo fuzz cmin fuzz_tcp_parser
```

**Coverage Report:**
```bash
# Generate coverage report
cargo fuzz coverage fuzz_tcp_parser
```

---

## Corpus Management

### Corpus Structure

```
fuzz/
├── corpus/
│   ├── fuzz_tcp_parser/
│   │   ├── 0a1b2c3d4e5f...  # Individual test cases (hex hash filenames)
│   │   ├── 1f2e3d4c5b6a...
│   │   └── ...
│   ├── fuzz_udp_parser/
│   ├── fuzz_ipv6_parser/
│   ├── fuzz_icmpv6_parser/
│   └── fuzz_tls_parser/
└── artifacts/
    ├── fuzz_tcp_parser/
    │   ├── crash-0a1b2c3d  # Crashing inputs
    │   ├── timeout-1f2e3d  # Timeout inputs
    │   └── slow-unit-2e3d  # Slow inputs
    └── ...
```

### Corpus Operations

**1. Add Seed Corpus:**
```bash
# Add known-good packets to corpus
mkdir -p corpus/fuzz_tcp_parser

# Example: SYN packet
echo -ne '\x00\x50\x1f\x90\x00\x00\x00\x01\x00\x00\x00\x00\x50\x02\x20\x00\x00\x00\x00\x00' \
    > corpus/fuzz_tcp_parser/syn_packet
```

**2. Merge Corpus from CI:**
```bash
# Download corpus from CI/CD artifacts
wget https://ci.example.com/corpus-fuzz_tcp_parser.tar.gz
tar xzf corpus-fuzz_tcp_parser.tar.gz -C corpus/

# Merge into existing corpus
cargo fuzz run fuzz_tcp_parser corpus/fuzz_tcp_parser -- -merge=1
```

**3. Minimize Corpus (Remove Redundant):**
```bash
# Before: 10,000 test cases
cargo fuzz cmin fuzz_tcp_parser

# After: ~500 test cases with same coverage
```

**4. Export Corpus for Analysis:**
```bash
# Convert corpus to human-readable format
for file in corpus/fuzz_tcp_parser/*; do
    xxd "$file" > "$(basename $file).hex"
done
```

### Corpus Metrics

**Good Corpus Characteristics:**
- **Size:** 100-1000 test cases per target (after minimization)
- **Coverage:** 80%+ of target code paths
- **Diversity:** Wide range of packet sizes, field values, edge cases
- **Performance:** <1ms average execution time per test case

**Measure Coverage:**
```bash
cargo fuzz coverage fuzz_tcp_parser

# Output: HTML report in fuzz/coverage/fuzz_tcp_parser/index.html
```

---

## Crash Analysis

### When a Crash Occurs

**1. Reproduce Crash:**
```bash
# Crashes are saved to fuzz/artifacts/fuzz_tcp_parser/crash-<hash>
cargo fuzz run fuzz_tcp_parser fuzz/artifacts/fuzz_tcp_parser/crash-0a1b2c3d
```

**2. Debug with GDB:**
```bash
# Build with debug symbols
cargo fuzz build fuzz_tcp_parser

# Run under GDB
rust-gdb -ex run --args target/x86_64-unknown-linux-gnu/release/fuzz_tcp_parser \
    fuzz/artifacts/fuzz_tcp_parser/crash-0a1b2c3d
```

**3. Minimize Crash Input:**
```bash
# Reduce crash input to minimal reproducer
cargo fuzz tmin fuzz_tcp_parser fuzz/artifacts/fuzz_tcp_parser/crash-0a1b2c3d
```

**4. Generate Regression Test:**
```rust
// In crates/prtip-network/src/tcp/tests.rs
#[test]
fn test_fuzz_crash_0a1b2c3d() {
    // Minimized crash input
    let packet_bytes = &[
        0x00, 0x50, 0x1f, 0x90,  // Source port, dest port
        // ... minimal reproducing bytes
    ];

    // Should not panic
    let result = TcpPacket::new(packet_bytes);
    assert!(result.is_some() || result.is_none()); // Either valid or rejected gracefully
}
```

### Common Crash Patterns

**1. Integer Overflow:**
```rust
// BAD: Can overflow
let total_len = header_len + payload_len;

// GOOD: Checked arithmetic
let total_len = header_len.checked_add(payload_len)
    .ok_or(Error::PacketTooLarge)?;
```

**2. Out-of-Bounds Access:**
```rust
// BAD: Direct indexing
let value = packet[offset];

// GOOD: Bounds checking
let value = packet.get(offset)
    .ok_or(Error::InvalidOffset)?;
```

**3. Panic on Malformed Data:**
```rust
// BAD: unwrap() can panic
let port = u16::from_be_bytes([packet[0], packet[1]]);

// GOOD: Return Option/Result
let port = packet.get(0..2)
    .and_then(|bytes| bytes.try_into().ok())
    .map(u16::from_be_bytes)?;
```

**4. Infinite Loop:**
```rust
// BAD: Can loop forever on circular references
while let Some(next_header) = parse_extension_header(current) {
    current = next_header;
}

// GOOD: Limit iterations
const MAX_EXTENSION_HEADERS: usize = 10;
for _ in 0..MAX_EXTENSION_HEADERS {
    if let Some(next_header) = parse_extension_header(current) {
        current = next_header;
    } else {
        break;
    }
}
```

---

## Integration with CI/CD

### GitHub Actions Workflow

```yaml
# .github/workflows/fuzz.yml
name: Fuzzing

on:
  schedule:
    # Run nightly at 2 AM UTC
    - cron: '0 2 * * *'
  workflow_dispatch:  # Manual trigger

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - fuzz_tcp_parser
          - fuzz_udp_parser
          - fuzz_ipv6_parser
          - fuzz_icmpv6_parser
          - fuzz_tls_parser

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust nightly
        uses: dtolnay/rust-toolchain@nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Download corpus
        uses: actions/download-artifact@v4
        with:
          name: corpus-${{ matrix.target }}
          path: fuzz/corpus/${{ matrix.target }}
        continue-on-error: true  # First run won't have corpus

      - name: Run fuzzer
        run: |
          cd fuzz
          # Run for 10 minutes (600 seconds)
          timeout 600 cargo fuzz run ${{ matrix.target }} \
            -- -max_total_time=600 -max_len=2000 \
            || true  # Don't fail on timeout

      - name: Upload corpus
        uses: actions/upload-artifact@v4
        with:
          name: corpus-${{ matrix.target }}
          path: fuzz/corpus/${{ matrix.target }}
          retention-days: 30

      - name: Upload crashes
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: crashes-${{ matrix.target }}
          path: fuzz/artifacts/${{ matrix.target }}
          retention-days: 90
        continue-on-error: true  # No crashes = no artifacts

      - name: Check for crashes
        run: |
          if [ -d "fuzz/artifacts/${{ matrix.target }}" ] && [ "$(ls -A fuzz/artifacts/${{ matrix.target }})" ]; then
            echo "CRASHES FOUND!"
            ls -la fuzz/artifacts/${{ matrix.target }}/
            exit 1
          fi
```

### Continuous Fuzzing with OSS-Fuzz (Future)

**Integration Steps:**
1. Submit ProRT-IP to [OSS-Fuzz](https://github.com/google/oss-fuzz)
2. Configure build script (`oss_fuzz_build.sh`)
3. Automatic 24/7 fuzzing on Google infrastructure
4. Public dashboard with coverage reports

**Benefits:**
- **Scale:** 10,000+ CPU cores
- **Coverage:** 90%+ code coverage achieved
- **Integration:** Automatic bug filing on GitHub
- **Corpus:** Shared corpus across projects

---

## Best Practices

### Writing Effective Fuzz Targets

**1. Prefer Structure-Aware Fuzzing:**
```rust
// GOOD: Structure-aware with constraints
#[derive(Arbitrary)]
struct FuzzInput {
    #[arbitrary(with = |u: &mut Unstructured| {
        u.int_in_range(0..=65535)  // Valid port range
    })]
    port: u16,
}

// BAD: Unstructured (wastes time on invalid inputs)
fuzz_target!(|data: &[u8]| {
    let port = u16::from_be_bytes([data[0], data[1]]);  // Often invalid
});
```

**2. Test Both Valid and Invalid Inputs:**
```rust
fuzz_target!(|input: FuzzInput| {
    // Test structure-aware (valid-ish) input
    let packet = build_packet(&input);
    let _ = parse_packet(&packet);

    // Also test raw bytes (edge cases)
    let _ = parse_packet(&input.raw_bytes);
});
```

**3. Exercise All Code Paths:**
```rust
if let Some(packet) = TcpPacket::new(&bytes) {
    // Test ALL accessor methods
    let _ = packet.get_source();
    let _ = packet.get_destination();
    let _ = packet.get_sequence();
    let _ = packet.get_flags();
    let _ = packet.payload();

    // Test protocol-specific logic
    if packet.get_flags() & TCP_SYN != 0 {
        let _ = process_syn_packet(&packet);
    }
}
```

**4. Assert Expected Behavior:**
```rust
// Don't just ignore errors - verify expected behavior
if bytes.len() < MIN_PACKET_SIZE {
    let result = parse_packet(&bytes);
    assert!(result.is_err(), "Should reject undersized packet");
}
```

**5. Limit Resource Usage:**
```rust
// Prevent DOS during fuzzing
const MAX_PACKET_SIZE: usize = 65535;
const MAX_OPTIONS_LEN: usize = 40;
const MAX_EXTENSION_HEADERS: usize = 10;

if input.payload.len() > MAX_PACKET_SIZE {
    return;  // Skip oversized input
}
```

### Performance Optimization

**1. Profile Fuzzer Performance:**
```bash
# Check executions per second
cargo fuzz run fuzz_tcp_parser -- -max_total_time=60 -print_final_stats=1

# Output:
#   exec/s   : 15000
#   cov      : 850 features
```

**2. Optimize Build Settings:**
```toml
# fuzz/Cargo.toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = "thin"           # Fast link-time optimization
codegen-units = 1      # Better optimization (slower build)
debug = true           # Keep symbols for crash analysis
```

**3. Reduce Input Size:**
```rust
// Limit maximum input size for faster execution
#[arbitrary(with = |u: &mut Unstructured| {
    let len = u.int_in_range(0..=1500)?;  // Reasonable MTU
    u.bytes(len).map(|b| b.to_vec())
})]
payload: Vec<u8>,
```

**4. Parallelize Fuzzing:**
```bash
# Use all CPU cores
cargo fuzz run fuzz_tcp_parser -- -jobs=$(nproc) -workers=$(nproc)
```

### Corpus Quality

**1. Seed with Real-World Packets:**
```bash
# Capture real packets
tcpdump -i eth0 -w packets.pcap 'tcp port 80'

# Extract to corpus
tcpdump -r packets.pcap -w - | split -b 1500 - corpus/fuzz_tcp_parser/real-
```

**2. Include Edge Cases:**
```bash
# Minimum size packets
echo -ne '\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x02\x20\x00\x00\x00\x00\x00' \
    > corpus/fuzz_tcp_parser/min_syn

# Maximum size (1500 bytes)
dd if=/dev/urandom bs=1500 count=1 > corpus/fuzz_tcp_parser/max_packet

# Zero-length payload
echo -ne '\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x02\x20\x00\x00\x00\x00\x00' \
    > corpus/fuzz_tcp_parser/zero_payload
```

**3. Regularly Minimize Corpus:**
```bash
# Weekly corpus maintenance
0 0 * * 0 cd /path/to/ProRT-IP/fuzz && cargo fuzz cmin fuzz_tcp_parser
```

---

## See Also

- [Testing](testing.md) - Overall testing philosophy and strategy
- [Testing Infrastructure](testing-infrastructure.md) - Test utilities and mock services
- [CI/CD](ci-cd.md) - Continuous integration and deployment
- [Implementation Guide](implementation.md) - Code organization and patterns
- [Security Best Practices](../advanced/security-best-practices.md) - Security guidelines

---

**Version:** 1.0.0
**Last Updated:** 2025-11-15
**Fuzz Targets:** 5 (TCP, UDP, IPv6, ICMPv6, TLS)
**Total Executions:** 230M+ (0 crashes)
