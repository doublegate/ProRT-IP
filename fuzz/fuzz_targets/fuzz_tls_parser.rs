#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use prtip_scanner::tls_certificate::{parse_certificate, parse_certificate_chain};

/// Structure-aware fuzzing input for TLS certificates
///
/// This fuzzer targets X.509 certificate parsing, which is a complex ASN.1/DER format.
/// We'll test both structured (valid-ish) and unstructured (random) inputs.
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

/// Generate a minimal valid X.509 certificate structure (DER-encoded)
/// This is structure-aware fuzzing - start with a valid structure and mutate it
fn generate_minimal_cert(data: &[u8]) -> Vec<u8> {
    // X.509 Certificate structure (simplified):
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

    // Start with minimal valid DER structure
    let mut cert = vec![
        0x30, 0x82, 0x01, 0x00, // SEQUENCE (certificate, length placeholder)
        0x30, 0x81, 0xF0,       // SEQUENCE (tbsCertificate, length placeholder)

        // Version [0] EXPLICIT
        0xA0, 0x03,             // [0] EXPLICIT
        0x02, 0x01, 0x02,       // INTEGER 2 (v3)

        // Serial number
        0x02, 0x08,             // INTEGER (8 bytes)
    ];

    // Add 8 bytes of serial from fuzzer input
    if data.len() >= 8 {
        cert.extend_from_slice(&data[0..8]);
    } else {
        cert.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    // Signature algorithm (SHA256withRSA)
    cert.extend_from_slice(&[
        0x30, 0x0D,                                     // SEQUENCE
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7,      // OID 1.2.840.113549.1.1.11
        0x0D, 0x01, 0x01, 0x0B,
        0x05, 0x00,                                     // NULL
    ]);

    // Issuer (minimal DN)
    cert.extend_from_slice(&[
        0x30, 0x10,             // SEQUENCE
        0x31, 0x0E,             // SET
        0x30, 0x0C,             // SEQUENCE
        0x06, 0x03, 0x55, 0x04, 0x03,  // OID commonName
        0x13, 0x05,             // UTF8String (5 bytes)
        b'T', b'e', b's', b't', b'1',  // "Test1"
    ]);

    // Validity (notBefore + notAfter)
    cert.extend_from_slice(&[
        0x30, 0x1E,             // SEQUENCE
        0x17, 0x0D,             // UTCTime (13 bytes)
        b'2', b'0', b'0', b'1', b'0', b'1', b'0', b'0', b'0', b'0', b'0', b'0', b'Z',
        0x17, 0x0D,             // UTCTime (13 bytes)
        b'2', b'9', b'1', b'2', b'3', b'1', b'2', b'3', b'5', b'9', b'5', b'9', b'Z',
    ]);

    // Subject (minimal DN)
    cert.extend_from_slice(&[
        0x30, 0x10,             // SEQUENCE
        0x31, 0x0E,             // SET
        0x30, 0x0C,             // SEQUENCE
        0x06, 0x03, 0x55, 0x04, 0x03,  // OID commonName
        0x13, 0x05,             // UTF8String (5 bytes)
        b'T', b'e', b's', b't', b'2',  // "Test2"
    ]);

    // SubjectPublicKeyInfo (RSA public key, minimal)
    cert.extend_from_slice(&[
        0x30, 0x22,             // SEQUENCE
        0x30, 0x0D,             // SEQUENCE (algorithm)
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7,  // OID rsaEncryption
        0x0D, 0x01, 0x01, 0x01,
        0x05, 0x00,             // NULL
        0x03, 0x11, 0x00,       // BIT STRING (16 bytes + padding bit)
        0x30, 0x0E,             // SEQUENCE
        0x02, 0x07,             // INTEGER (modulus, 7 bytes)
        0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
        0x02, 0x03,             // INTEGER (exponent, 3 bytes)
        0x01, 0x00, 0x01,       // 65537
    ]);

    // SignatureAlgorithm (same as above)
    cert.extend_from_slice(&[
        0x30, 0x0D,                                     // SEQUENCE
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7,      // OID 1.2.840.113549.1.1.11
        0x0D, 0x01, 0x01, 0x0B,
        0x05, 0x00,                                     // NULL
    ]);

    // Signature (BIT STRING, 32 bytes)
    cert.extend_from_slice(&[
        0x03, 0x21, 0x00,       // BIT STRING (32 bytes + padding)
    ]);

    // Add signature bytes from fuzzer input or zeros
    if data.len() >= 40 {
        cert.extend_from_slice(&data[8..40]);
    } else {
        cert.extend_from_slice(&[0u8; 32]);
    }

    // Fix lengths
    let total_len = cert.len();
    cert[2] = ((total_len - 4) >> 8) as u8;
    cert[3] = (total_len - 4) as u8;

    cert
}

fuzz_target!(|input: FuzzTlsCertInput| {
    // Test 1: Parse arbitrary DER bytes (unstructured fuzzing)
    // This catches crashes in X.509 parser with truly malformed input
    let _ = parse_certificate(&input.cert_der);

    // Test 2: Parse with minimal structure (structure-aware fuzzing)
    let structured_cert = generate_minimal_cert(&input.cert_der);
    let _ = parse_certificate(&structured_cert);

    // Test 3: Parse certificate chain
    if input.test_chain && !input.chain_certs.is_empty() {
        // Build chain with primary cert + additional certs
        let mut chain_refs: Vec<&[u8]> = vec![&input.cert_der];
        for cert in &input.chain_certs {
            chain_refs.push(cert);
        }

        // Parse chain - should not panic
        let _ = parse_certificate_chain(chain_refs);
    }

    // Test 4: Parse single certificate and exercise all fields
    if let Ok(cert_info) = parse_certificate(&input.cert_der) {
        // Access all fields to ensure no panic in accessor methods
        let _ = &cert_info.issuer;
        let _ = &cert_info.subject;
        let _ = &cert_info.validity_not_before;
        let _ = &cert_info.validity_not_after;
        let _ = &cert_info.san;
        let _ = &cert_info.serial_number;
        let _ = &cert_info.signature_algorithm;
        let _ = &cert_info.san_categorized;
        let _ = &cert_info.public_key_info;
        let _ = &cert_info.key_usage;
        let _ = &cert_info.extended_key_usage;
        let _ = &cert_info.extensions;
        let _ = &cert_info.signature_algorithm_enhanced;

        // Test SAN categorization
        let _ = cert_info.san_categorized.dns_names.len();
        let _ = cert_info.san_categorized.ip_addresses.len();
        let _ = cert_info.san_categorized.email_addresses.len();
        let _ = cert_info.san_categorized.uris.len();

        // Test public key info
        let _ = cert_info.public_key_info.algorithm.clone();
        let _ = cert_info.public_key_info.key_size;
        let _ = cert_info.public_key_info.curve.clone();
        let _ = cert_info.public_key_info.usage.clone();

        // Test key usage
        if let Some(ku) = &cert_info.key_usage {
            let _ = ku.digital_signature;
            let _ = ku.key_encipherment;
            let _ = ku.key_cert_sign;
            let _ = ku.crl_sign;
        }

        // Test extended key usage
        if let Some(eku) = &cert_info.extended_key_usage {
            let _ = eku.server_auth;
            let _ = eku.client_auth;
            let _ = eku.code_signing;
        }
    }

    // Test 5: Edge cases - very short inputs
    if input.cert_der.len() < 10 {
        let result = parse_certificate(&input.cert_der);
        // Should return error, not panic
        assert!(result.is_err(), "Should reject very short certificate data");
    }

    // Test 6: Edge cases - very large inputs (DOS prevention)
    if input.cert_der.len() > 10000 {
        // Should handle gracefully, not OOM
        let _ = parse_certificate(&input.cert_der);
    }
});
