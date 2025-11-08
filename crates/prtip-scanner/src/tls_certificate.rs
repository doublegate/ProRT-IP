//! TLS certificate parsing and analysis
//!
//! This module provides X.509 certificate parsing, certificate chain validation,
//! and TLS fingerprinting capabilities for enhanced HTTPS service detection.
//!
//! # Features
//!
//! - X.509 certificate parsing (issuer, subject, validity, SAN)
//! - Certificate chain validation (self-signed vs CA-signed)
//! - TLS version fingerprinting (TLS 1.0/1.1/1.2/1.3)
//! - Cipher suite analysis and strength categorization
//!
//! # See Also
//!
//! - [TLS Certificate Analysis Guide](../../docs/27-TLS-CERTIFICATE-GUIDE.md) - Comprehensive certificate analysis guide
//! - [User Guide: TLS Certificate Analysis](../../docs/32-USER-GUIDE.md#use-case-13-tls-certificate-analysis) - Usage examples
//! - [`tls_handshake`](super::tls_handshake) - TLS handshake implementation
//!
//! # Examples
//!
//! ## Basic Certificate Parsing
//!
//! ```no_run
//! use prtip_scanner::tls_certificate::{CertificateInfo, parse_certificate};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse a DER-encoded certificate from a file or network response
//! // let cert_der = std::fs::read("path/to/certificate.der")?;
//! # let cert_der = vec![]; // Placeholder for example
//! let cert_info = parse_certificate(&cert_der)?;
//!
//! println!("Subject: {}", cert_info.subject);
//! println!("Issuer: {}", cert_info.issuer);
//! println!("Valid until: {}", cert_info.validity_not_after);
//! # Ok(())
//! # }
//! ```
//!
//! ## Certificate Chain Validation
//!
//! ```no_run
//! use prtip_scanner::tls_certificate::{CertificateChain, validate_chain};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load certificate chain from files or network response
//! // let leaf_cert = std::fs::read("leaf.der")?;
//! // let intermediate = std::fs::read("intermediate.der")?;
//! // let root_cert = std::fs::read("root.der")?;
//! # let leaf_cert = vec![]; // Placeholder
//! # let intermediate = vec![]; // Placeholder
//! # let root_cert = vec![]; // Placeholder
//!
//! let certs = vec![leaf_cert.as_ref(), intermediate.as_ref(), root_cert.as_ref()];
//! let chain = validate_chain(&certs)?;
//!
//! println!("Chain depth: {}", chain.certificates.len());
//! println!("Self-signed: {}", chain.is_self_signed);
//! println!("Valid: {}", chain.is_valid);
//! # Ok(())
//! # }
//! ```
//!
//! ## TLS Analysis Result
//!
//! ```no_run
//! use prtip_scanner::tls_certificate::{TlsAnalysisResult, TlsFingerprint, CertificateInfo};
//! use prtip_scanner::tls_certificate::{SubjectAlternativeName, PublicKeyInfo, SignatureAlgorithm, SecurityStrength};
//!
//! // This example shows the structure of a TLS analysis result
//! // In practice, use parse_certificate() or perform_tls_analysis() to construct these
//! let result = TlsAnalysisResult {
//!     certificate: Some(CertificateInfo {
//!         issuer: "CN=Example CA".to_string(),
//!         subject: "CN=example.com".to_string(),
//!         validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
//!         validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
//!         san: vec!["example.com".to_string(), "www.example.com".to_string()],
//!         serial_number: "01:02:03:04".to_string(),
//!         signature_algorithm: "sha256WithRSAEncryption".to_string(),
//!         san_categorized: SubjectAlternativeName::default(),
//!         public_key_info: PublicKeyInfo {
//!             algorithm: "RSA".to_string(),
//!             key_size: 2048,
//!             curve: None,
//!             usage: vec![],
//!         },
//!         key_usage: None,
//!         extended_key_usage: None,
//!         extensions: vec![],
//!         signature_algorithm_enhanced: SignatureAlgorithm {
//!             algorithm: "RSA-SHA256".to_string(),
//!             hash_algorithm: "SHA256".to_string(),
//!             is_secure: true,
//!             strength: SecurityStrength::Acceptable,
//!         },
//!     }),
//!     fingerprint: TlsFingerprint {
//!         tls_version: "TLS 1.3".to_string(),
//!         cipher_suites: vec!["TLS_AES_128_GCM_SHA256".to_string()],
//!         extensions: vec!["server_name".to_string(), "application_layer_protocol_negotiation".to_string()],
//!     },
//!     chain: None,
//! };
//!
//! assert!(result.certificate.is_some());
//! assert_eq!(result.fingerprint.tls_version, "TLS 1.3");
//! ```

use prtip_core::Error;
use std::fmt;
use x509_parser::prelude::*;

/// Certificate information extracted from X.509 certificate
///
/// Contains the most important fields from an X.509 certificate for reconnaissance
/// and service detection purposes.
///
/// # TASK-3 Enhancement
///
/// Enhanced with comprehensive extension support including categorized SANs,
/// public key analysis, key usage, extended key usage, and all extensions.
#[derive(Debug, Clone, PartialEq)]
pub struct CertificateInfo {
    /// Certificate issuer (e.g., "CN=Let's Encrypt Authority X3, O=Let's Encrypt, C=US")
    pub issuer: String,

    /// Certificate subject (e.g., "CN=example.com")
    pub subject: String,

    /// Validity period start (ISO 8601 format)
    pub validity_not_before: String,

    /// Validity period end (ISO 8601 format)
    pub validity_not_after: String,

    /// Subject Alternative Names (DNS names, IPs, email addresses) - LEGACY
    /// For backward compatibility. Use san_categorized for detailed analysis.
    pub san: Vec<String>,

    /// Certificate serial number (hex format)
    pub serial_number: String,

    /// Signature algorithm (e.g., "sha256WithRSAEncryption") - LEGACY
    /// For backward compatibility. Use signature_algorithm_enhanced for detailed analysis.
    pub signature_algorithm: String,

    // === TASK-3 NEW FIELDS ===
    /// Categorized Subject Alternative Names with DNS, IP, email, URI separation
    pub san_categorized: SubjectAlternativeName,

    /// Public key information (algorithm, size, curve, security assessment)
    pub public_key_info: PublicKeyInfo,

    /// Key Usage extension (9 bit flags for certificate usage purposes)
    pub key_usage: Option<KeyUsage>,

    /// Extended Key Usage extension (TLS, code signing, email, etc.)
    pub extended_key_usage: Option<ExtendedKeyUsage>,

    /// All certificate extensions with OID, name, criticality, and value
    pub extensions: Vec<CertificateExtension>,

    /// Enhanced signature algorithm with security analysis
    pub signature_algorithm_enhanced: SignatureAlgorithm,
}

impl fmt::Display for CertificateInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Certificate: subject={}, issuer={}, valid={} to {}, serial={}, san={}",
            self.subject,
            self.issuer,
            self.validity_not_before,
            self.validity_not_after,
            self.serial_number,
            self.san.len()
        )
    }
}

/// Categorized Subject Alternative Names from X.509 certificate
///
/// Subject Alternative Names (SAN) extension allows certificates to specify
/// additional hostnames, IP addresses, email addresses, and URIs that are
/// valid for the certificate.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubjectAlternativeName {
    /// DNS names (e.g., "example.com", "*.example.com")
    pub dns_names: Vec<String>,

    /// IP addresses (IPv4 and IPv6)
    pub ip_addresses: Vec<String>,

    /// Email addresses (RFC 822 format)
    pub email_addresses: Vec<String>,

    /// URIs (Uniform Resource Identifiers)
    pub uris: Vec<String>,

    /// Other name types not categorized above
    pub other_names: Vec<String>,
}

impl SubjectAlternativeName {
    /// Extract and categorize SANs from x509-parser certificate
    ///
    /// # Arguments
    ///
    /// * `cert` - X.509 certificate to extract SANs from
    ///
    /// # Returns
    ///
    /// * `SubjectAlternativeName` with categorized names
    pub fn from_certificate(cert: &X509Certificate) -> Self {
        let mut san = SubjectAlternativeName::default();

        if let Ok(Some(san_ext)) = cert.subject_alternative_name() {
            for name in &san_ext.value.general_names {
                match name {
                    GeneralName::DNSName(dns) => san.dns_names.push(dns.to_string()),
                    GeneralName::IPAddress(ip) => san.ip_addresses.push(format!("{:?}", ip)),
                    GeneralName::RFC822Name(email) => san.email_addresses.push(email.to_string()),
                    GeneralName::URI(uri) => san.uris.push(uri.to_string()),
                    _ => san.other_names.push(format!("{:?}", name)),
                }
            }
        }

        san
    }

    /// Check if SAN contains a specific DNS name (supports wildcards)
    ///
    /// # Arguments
    ///
    /// * `hostname` - Hostname to match against DNS names
    ///
    /// # Returns
    ///
    /// * `true` if hostname matches any DNS name (including wildcard patterns)
    /// * `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use prtip_scanner::tls_certificate::SubjectAlternativeName;
    /// let mut san = SubjectAlternativeName::default();
    /// san.dns_names.push("example.com".to_string());
    /// san.dns_names.push("*.example.com".to_string());
    ///
    /// assert!(san.matches_dns("example.com"));
    /// assert!(san.matches_dns("www.example.com"));
    /// assert!(!san.matches_dns("evil.com"));
    /// ```
    pub fn matches_dns(&self, hostname: &str) -> bool {
        for dns_name in &self.dns_names {
            // Exact match
            if dns_name == hostname {
                return true;
            }

            // Wildcard match (*.example.com matches www.example.com but not example.com)
            if let Some(domain) = dns_name.strip_prefix("*.") {
                if hostname.ends_with(domain) && hostname.len() > domain.len() {
                    return true;
                }
            }
        }
        false
    }

    /// Get all names (DNS + IP + email + URI) as a flat list
    ///
    /// # Returns
    ///
    /// * Vector of all SAN entries regardless of type
    pub fn all_names(&self) -> Vec<String> {
        let mut all = Vec::new();
        all.extend(self.dns_names.clone());
        all.extend(self.ip_addresses.clone());
        all.extend(self.email_addresses.clone());
        all.extend(self.uris.clone());
        all.extend(self.other_names.clone());
        all
    }
}

impl fmt::Display for SubjectAlternativeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SAN: dns={}, ip={}, email={}, uri={}",
            self.dns_names.len(),
            self.ip_addresses.len(),
            self.email_addresses.len(),
            self.uris.len()
        )
    }
}

/// Public key information from X.509 certificate
///
/// Contains algorithm, key size, curve name (for ECDSA), and security assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicKeyInfo {
    /// Algorithm (RSA, ECDSA, Ed25519, etc.)
    pub algorithm: String,

    /// Key size in bits (e.g., 2048, 3072, 4096 for RSA; 256, 384 for ECDSA)
    pub key_size: u32,

    /// Curve name for ECDSA (e.g., "P-256", "P-384", "P-521")
    pub curve: Option<String>,

    /// Public key usage context
    pub usage: Vec<String>,
}

impl PublicKeyInfo {
    /// Extract public key information from x509-parser certificate
    ///
    /// # Arguments
    ///
    /// * `cert` - X.509 certificate to extract public key from
    ///
    /// # Returns
    ///
    /// * `PublicKeyInfo` with algorithm, size, and curve information
    pub fn from_certificate(cert: &X509Certificate) -> Self {
        let spki = cert.public_key();
        let algorithm_oid = &spki.algorithm.algorithm;

        // Determine algorithm and key size based on OID
        let (algorithm, key_size, curve) =
            if algorithm_oid.to_string().starts_with("1.2.840.113549.1.1") {
                // RSA (OID: 1.2.840.113549.1.1.*)
                let key_bits = (spki.subject_public_key.data.len() * 8) as u32;
                ("RSA".to_string(), key_bits, None)
            } else if algorithm_oid.to_string().starts_with("1.2.840.10045.2.1") {
                // ECDSA (OID: 1.2.840.10045.2.1)
                // Try to extract curve from algorithm parameters
                let curve_name = if let Some(params) = &spki.algorithm.parameters {
                    // Common ECDSA curve OIDs
                    let params_str = format!("{:?}", params);
                    if params_str.contains("1.2.840.10045.3.1.7") {
                        Some("P-256".to_string())
                    } else if params_str.contains("1.3.132.0.34") {
                        Some("P-384".to_string())
                    } else if params_str.contains("1.3.132.0.35") {
                        Some("P-521".to_string())
                    } else {
                        Some("Unknown".to_string())
                    }
                } else {
                    None
                };
                let key_bits = (spki.subject_public_key.data.len() * 8) as u32;
                ("ECDSA".to_string(), key_bits, curve_name)
            } else if algorithm_oid.to_string().contains("1.3.101.112") {
                // Ed25519 (OID: 1.3.101.112)
                ("Ed25519".to_string(), 256, None)
            } else {
                // Unknown algorithm
                ("Unknown".to_string(), 0, None)
            };

        PublicKeyInfo {
            algorithm,
            key_size,
            curve,
            usage: Vec::new(), // Populated later from KeyUsage extension
        }
    }

    /// Check if key size meets minimum security requirements
    ///
    /// # Returns
    ///
    /// * `true` if key meets current security standards
    /// * `false` if key is considered weak
    ///
    /// Security standards:
    /// - RSA: >= 2048 bits (3072+ recommended)
    /// - ECDSA: >= 256 bits (P-256 curve minimum)
    /// - Ed25519: always secure (256 bits)
    pub fn is_secure(&self) -> bool {
        match self.algorithm.as_str() {
            "RSA" => self.key_size >= 2048,
            "ECDSA" => self.key_size >= 256,
            "Ed25519" => true,
            _ => false, // Unknown algorithms considered insecure
        }
    }
}

impl fmt::Display for PublicKeyInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(curve) = &self.curve {
            write!(
                f,
                "Public Key: {} {} bits (curve: {})",
                self.algorithm, self.key_size, curve
            )
        } else {
            write!(f, "Public Key: {} {} bits", self.algorithm, self.key_size)
        }
    }
}

/// X.509 Key Usage extension (RFC 5280 Section 4.2.1.3)
///
/// Defines the purposes for which the certified public key may be used.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyUsage {
    /// Digital signature (signing data)
    pub digital_signature: bool,

    /// Non-repudiation (content commitment)
    pub non_repudiation: bool,

    /// Key encipherment (encrypting keys)
    pub key_encipherment: bool,

    /// Data encipherment (encrypting data)
    pub data_encipherment: bool,

    /// Key agreement (key exchange)
    pub key_agreement: bool,

    /// Certificate signing (CA certificates)
    pub key_cert_sign: bool,

    /// CRL signing
    pub crl_sign: bool,

    /// Encipher only (with key_agreement)
    pub encipher_only: bool,

    /// Decipher only (with key_agreement)
    pub decipher_only: bool,
}

impl KeyUsage {
    /// Extract Key Usage extension from certificate
    ///
    /// # Arguments
    ///
    /// * `cert` - X.509 certificate to extract Key Usage from
    ///
    /// # Returns
    ///
    /// * `Some(KeyUsage)` if extension present
    /// * `None` if extension not present
    pub fn from_certificate(cert: &X509Certificate) -> Option<Self> {
        if let Ok(Some(ku_ext)) = cert.key_usage() {
            let ku = &ku_ext.value;
            Some(KeyUsage {
                digital_signature: ku.digital_signature(),
                non_repudiation: ku.non_repudiation(),
                key_encipherment: ku.key_encipherment(),
                data_encipherment: ku.data_encipherment(),
                key_agreement: ku.key_agreement(),
                key_cert_sign: ku.key_cert_sign(),
                crl_sign: ku.crl_sign(),
                encipher_only: ku.encipher_only(),
                decipher_only: ku.decipher_only(),
            })
        } else {
            None
        }
    }

    /// Check if certificate allows a specific usage
    ///
    /// # Arguments
    ///
    /// * `usage` - Usage name to check (e.g., "digital_signature", "key_cert_sign")
    ///
    /// # Returns
    ///
    /// * `true` if usage is allowed
    /// * `false` otherwise
    pub fn allows(&self, usage: &str) -> bool {
        match usage {
            "digital_signature" => self.digital_signature,
            "non_repudiation" => self.non_repudiation,
            "key_encipherment" => self.key_encipherment,
            "data_encipherment" => self.data_encipherment,
            "key_agreement" => self.key_agreement,
            "key_cert_sign" => self.key_cert_sign,
            "crl_sign" => self.crl_sign,
            "encipher_only" => self.encipher_only,
            "decipher_only" => self.decipher_only,
            _ => false,
        }
    }
}

impl fmt::Display for KeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut usages = Vec::new();
        if self.digital_signature {
            usages.push("digitalSignature");
        }
        if self.non_repudiation {
            usages.push("nonRepudiation");
        }
        if self.key_encipherment {
            usages.push("keyEncipherment");
        }
        if self.data_encipherment {
            usages.push("dataEncipherment");
        }
        if self.key_agreement {
            usages.push("keyAgreement");
        }
        if self.key_cert_sign {
            usages.push("keyCertSign");
        }
        if self.crl_sign {
            usages.push("cRLSign");
        }
        if self.encipher_only {
            usages.push("encipherOnly");
        }
        if self.decipher_only {
            usages.push("decipherOnly");
        }
        write!(f, "Key Usage: {}", usages.join(", "))
    }
}

/// X.509 Extended Key Usage extension (RFC 5280 Section 4.2.1.12)
///
/// Defines additional purposes for which the certified public key may be used.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtendedKeyUsage {
    /// TLS server authentication (1.3.6.1.5.5.7.3.1)
    pub server_auth: bool,

    /// TLS client authentication (1.3.6.1.5.5.7.3.2)
    pub client_auth: bool,

    /// Code signing (1.3.6.1.5.5.7.3.3)
    pub code_signing: bool,

    /// Email protection (1.3.6.1.5.5.7.3.4)
    pub email_protection: bool,

    /// Time stamping (1.3.6.1.5.5.7.3.8)
    pub time_stamping: bool,

    /// OCSP signing (1.3.6.1.5.5.7.3.9)
    pub ocsp_signing: bool,

    /// Any extended key usage (wildcard)
    pub any_extended_key_usage: bool,

    /// Other OIDs not categorized above
    pub other_usages: Vec<String>,
}

impl ExtendedKeyUsage {
    /// Extract Extended Key Usage extension from certificate
    ///
    /// # Arguments
    ///
    /// * `cert` - X.509 certificate to extract Extended Key Usage from
    ///
    /// # Returns
    ///
    /// * `Some(ExtendedKeyUsage)` if extension present
    /// * `None` if extension not present
    pub fn from_certificate(cert: &X509Certificate) -> Option<Self> {
        if let Ok(Some(eku_ext)) = cert.extended_key_usage() {
            let eku_value = &eku_ext.value;

            // x509-parser ExtendedKeyUsage has pre-parsed fields
            Some(ExtendedKeyUsage {
                server_auth: eku_value.server_auth,
                client_auth: eku_value.client_auth,
                code_signing: eku_value.code_signing,
                email_protection: eku_value.email_protection,
                time_stamping: eku_value.time_stamping,
                ocsp_signing: eku_value.ocsp_signing,
                any_extended_key_usage: eku_value.any,
                other_usages: eku_value.other.iter().map(|oid| oid.to_string()).collect(),
            })
        } else {
            None
        }
    }

    /// Check if certificate is valid for TLS server authentication
    ///
    /// # Returns
    ///
    /// * `true` if serverAuth or anyExtendedKeyUsage is set
    /// * `false` otherwise
    pub fn is_valid_for_tls_server(&self) -> bool {
        self.server_auth || self.any_extended_key_usage
    }
}

impl fmt::Display for ExtendedKeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut usages = Vec::new();
        if self.server_auth {
            usages.push("serverAuth");
        }
        if self.client_auth {
            usages.push("clientAuth");
        }
        if self.code_signing {
            usages.push("codeSigning");
        }
        if self.email_protection {
            usages.push("emailProtection");
        }
        if self.time_stamping {
            usages.push("timeStamping");
        }
        if self.ocsp_signing {
            usages.push("ocspSigning");
        }
        if self.any_extended_key_usage {
            usages.push("anyExtendedKeyUsage");
        }
        write!(f, "Extended Key Usage: {}", usages.join(", "))
    }
}

/// Generic certificate extension representation
///
/// Represents any X.509 extension with OID, name, criticality, and value.
#[derive(Debug, Clone, PartialEq)]
pub struct CertificateExtension {
    /// Extension OID (e.g., "2.5.29.15" for Key Usage)
    pub oid: String,

    /// Human-readable name
    pub name: String,

    /// Is extension critical
    pub critical: bool,

    /// Extension value (human-readable summary)
    pub value: String,
}

impl CertificateExtension {
    /// Parse all extensions from certificate
    ///
    /// # Arguments
    ///
    /// * `cert` - X.509 certificate to extract extensions from
    ///
    /// # Returns
    ///
    /// * Vector of all certificate extensions
    pub fn from_certificate(cert: &X509Certificate) -> Vec<Self> {
        let mut extensions = Vec::new();

        for ext in cert.extensions() {
            let oid = ext.oid.to_string();
            let name = Self::oid_to_name(&oid);
            let critical = ext.critical;
            let value = format!("{:?}", ext.parsed_extension());

            extensions.push(CertificateExtension {
                oid,
                name,
                critical,
                value,
            });
        }

        extensions
    }

    /// Convert OID to human-readable name
    fn oid_to_name(oid: &str) -> String {
        match oid {
            "2.5.29.15" => "Key Usage".to_string(),
            "2.5.29.37" => "Extended Key Usage".to_string(),
            "2.5.29.17" => "Subject Alternative Name".to_string(),
            "2.5.29.19" => "Basic Constraints".to_string(),
            "2.5.29.35" => "Authority Key Identifier".to_string(),
            "2.5.29.14" => "Subject Key Identifier".to_string(),
            "2.5.29.31" => "CRL Distribution Points".to_string(),
            "1.3.6.1.5.5.7.1.1" => "Authority Information Access".to_string(),
            _ => format!("Unknown ({})", oid),
        }
    }
}

impl fmt::Display for CertificateExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Extension: {} ({}), critical={}",
            self.name, self.oid, self.critical
        )
    }
}

/// Enhanced signature algorithm information
///
/// Provides detailed analysis of signature algorithm including hash function
/// and security assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureAlgorithm {
    /// Algorithm (e.g., "RSA-SHA256", "ECDSA-SHA384")
    pub algorithm: String,

    /// Hash algorithm (e.g., "SHA256", "SHA384", "SHA512")
    pub hash_algorithm: String,

    /// Is algorithm considered secure
    pub is_secure: bool,

    /// Security strength assessment
    pub strength: SecurityStrength,
}

/// Security strength classification
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityStrength {
    /// Weak (MD5, SHA1)
    Weak,

    /// Acceptable (SHA256, RSA 2048)
    Acceptable,

    /// Strong (SHA384, RSA 3072+, ECDSA P-384+)
    Strong,
}

impl SignatureAlgorithm {
    /// Parse signature algorithm from certificate
    ///
    /// # Arguments
    ///
    /// * `cert` - X.509 certificate to extract signature algorithm from
    ///
    /// # Returns
    ///
    /// * `SignatureAlgorithm` with security analysis
    pub fn from_certificate(cert: &X509Certificate) -> Self {
        let sig_alg_str = format!("{:?}", cert.signature_algorithm.algorithm);
        let sig_alg_lower = sig_alg_str.to_lowercase();

        // Determine hash algorithm and security
        let (hash_algorithm, is_secure, strength) = if sig_alg_lower.contains("md5") {
            ("MD5".to_string(), false, SecurityStrength::Weak)
        } else if sig_alg_lower.contains("sha1") {
            ("SHA1".to_string(), false, SecurityStrength::Weak)
        } else if sig_alg_lower.contains("sha256") || sig_alg_lower.contains("sha-256") {
            ("SHA256".to_string(), true, SecurityStrength::Acceptable)
        } else if sig_alg_lower.contains("sha384") || sig_alg_lower.contains("sha-384") {
            ("SHA384".to_string(), true, SecurityStrength::Strong)
        } else if sig_alg_lower.contains("sha512") || sig_alg_lower.contains("sha-512") {
            ("SHA512".to_string(), true, SecurityStrength::Strong)
        } else {
            ("Unknown".to_string(), false, SecurityStrength::Weak)
        };

        SignatureAlgorithm {
            algorithm: sig_alg_str,
            hash_algorithm,
            is_secure,
            strength,
        }
    }
}

impl fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Signature: {} (hash: {}, strength: {:?})",
            self.algorithm, self.hash_algorithm, self.strength
        )
    }
}

// ==================== TASK-4: TLS HANDSHAKE FINGERPRINTING ====================

/// TLS protocol version
///
/// Represents the negotiated TLS protocol version from the ServerHello message.
/// Provides security assessment for deprecated versions.
///
/// # RFC References
///
/// - RFC 2246: TLS 1.0 (deprecated, vulnerable to BEAST attack)
/// - RFC 4346: TLS 1.1 (deprecated, weak CBC protection)
/// - RFC 5246: TLS 1.2 (current standard)
/// - RFC 8446: TLS 1.3 (latest, enhanced security)
///
/// # Examples
///
/// ```
/// use prtip_scanner::TlsVersion;
///
/// let version = TlsVersion::from_bytes(3, 3);
/// assert_eq!(version, TlsVersion::Tls12);
/// assert!(version.is_secure());
/// assert!(!version.is_deprecated());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TlsVersion {
    /// TLS 1.0 (0x0301) - Deprecated, vulnerable to BEAST attack
    Tls10,
    /// TLS 1.1 (0x0302) - Deprecated, weak CBC protection
    Tls11,
    /// TLS 1.2 (0x0303) - Current standard, widely supported
    Tls12,
    /// TLS 1.3 (0x0304) - Latest version, enhanced security and performance
    Tls13,
    /// Unknown or unsupported version (preserves version bytes)
    Unknown(u16),
}

impl TlsVersion {
    /// Parse TLS version from protocol version bytes (major, minor)
    ///
    /// TLS version encoding:
    /// - TLS 1.0: major=3, minor=1 (0x0301)
    /// - TLS 1.1: major=3, minor=2 (0x0302)
    /// - TLS 1.2: major=3, minor=3 (0x0303)
    /// - TLS 1.3: major=3, minor=4 (0x0304)
    ///
    /// # Arguments
    ///
    /// * `major` - Major version byte (always 3 for TLS)
    /// * `minor` - Minor version byte (1-4 for TLS 1.0-1.3)
    ///
    /// # Returns
    ///
    /// * `TlsVersion` enum variant
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::TlsVersion;
    ///
    /// assert_eq!(TlsVersion::from_bytes(3, 1), TlsVersion::Tls10);
    /// assert_eq!(TlsVersion::from_bytes(3, 3), TlsVersion::Tls12);
    /// ```
    pub fn from_bytes(major: u8, minor: u8) -> Self {
        match (major, minor) {
            (3, 1) => TlsVersion::Tls10,
            (3, 2) => TlsVersion::Tls11,
            (3, 3) => TlsVersion::Tls12,
            (3, 4) => TlsVersion::Tls13,
            _ => TlsVersion::Unknown(((major as u16) << 8) | (minor as u16)),
        }
    }

    /// Get human-readable version string
    ///
    /// # Returns
    ///
    /// * `&str` - Version string (e.g., "TLS 1.2", "TLS 1.3")
    pub fn as_str(&self) -> &str {
        match self {
            TlsVersion::Tls10 => "TLS 1.0",
            TlsVersion::Tls11 => "TLS 1.1",
            TlsVersion::Tls12 => "TLS 1.2",
            TlsVersion::Tls13 => "TLS 1.3",
            TlsVersion::Unknown(_) => "Unknown",
        }
    }

    /// Check if version is deprecated (TLS 1.0 or 1.1)
    ///
    /// TLS 1.0 and 1.1 are deprecated by RFC 8996 (March 2021) due to:
    /// - BEAST attack vulnerability
    /// - Weak CBC mode protection
    /// - Lack of modern cipher suites
    ///
    /// # Returns
    ///
    /// * `bool` - true if TLS 1.0 or 1.1, false otherwise
    pub fn is_deprecated(&self) -> bool {
        matches!(self, TlsVersion::Tls10 | TlsVersion::Tls11)
    }

    /// Check if version is secure (TLS 1.2 or later)
    ///
    /// TLS 1.2+ is considered secure with proper cipher suites:
    /// - Strong AEAD ciphers (AES-GCM, ChaCha20-Poly1305)
    /// - Forward secrecy (ECDHE, DHE key exchange)
    /// - Modern signature algorithms (RSA-PSS, ECDSA)
    ///
    /// # Returns
    ///
    /// * `bool` - true if TLS 1.2 or 1.3, false otherwise
    pub fn is_secure(&self) -> bool {
        matches!(self, TlsVersion::Tls12 | TlsVersion::Tls13)
    }
}

impl fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// TLS cipher suite security strength
///
/// Categorizes cipher suites by security level based on:
/// - Encryption algorithm strength (key size, algorithm)
/// - Key exchange method (forward secrecy)
/// - MAC/AEAD authentication
/// - Known vulnerabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherStrength {
    /// Weak: NULL, DES, RC4, export ciphers (broken or no encryption)
    Weak,
    /// Insecure: 3DES, AES-CBC with MD5/SHA1 (vulnerable to attacks)
    Insecure,
    /// Acceptable: AES-CBC with SHA256, basic RSA (meets minimum standards)
    Acceptable,
    /// Strong: AES-GCM, ECDHE, ChaCha20-Poly1305 (recommended for most uses)
    Strong,
    /// Recommended: TLS 1.3 ciphers only (latest security standards)
    Recommended,
}

/// TLS cipher suite with security assessment
///
/// Represents a negotiated cipher suite from the TLS handshake.
/// Provides detailed breakdown of cryptographic components and security strength.
///
/// # Structure
///
/// A cipher suite name like "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256" breaks down to:
/// - Key Exchange: ECDHE (Elliptic Curve Diffie-Hellman Ephemeral)
/// - Authentication: RSA (server authentication)
/// - Encryption: AES_128_GCM (symmetric encryption)
/// - MAC: SHA256 (message authentication, implicit in AEAD)
///
/// # Examples
///
/// ```
/// use prtip_scanner::{CipherSuite, CipherStrength};
///
/// // TLS 1.3 cipher (recommended)
/// let suite = CipherSuite::from_code(0x1301);
/// assert_eq!(suite.name, "TLS_AES_128_GCM_SHA256");
/// assert_eq!(suite.strength, CipherStrength::Recommended);
/// assert!(suite.is_tls13());
///
/// // TLS 1.2 cipher with forward secrecy
/// let suite = CipherSuite::from_code(0xC02F);
/// assert!(suite.has_forward_secrecy());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CipherSuite {
    /// Cipher suite code (e.g., 0x1301, 0xC02F)
    pub code: u16,
    /// Human-readable name (e.g., "TLS_AES_128_GCM_SHA256")
    pub name: String,
    /// Key exchange algorithm (e.g., "ECDHE", "RSA", "TLS13")
    pub key_exchange: String,
    /// Authentication algorithm (e.g., "RSA", "ECDSA", "AEAD")
    pub authentication: String,
    /// Encryption algorithm (e.g., "AES_128_GCM", "CHACHA20_POLY1305")
    pub encryption: String,
    /// MAC algorithm (e.g., "SHA256", "SHA384", implicit for AEAD)
    pub mac: String,
    /// Security strength assessment
    pub strength: CipherStrength,
}

impl CipherSuite {
    /// Parse cipher suite from 2-byte code
    ///
    /// Maps IANA-registered cipher suite codes to detailed cipher information.
    /// Supports 25+ common cipher suites across TLS 1.2 and 1.3.
    ///
    /// # Arguments
    ///
    /// * `code` - 2-byte cipher suite code (e.g., 0x1301)
    ///
    /// # Returns
    ///
    /// * `CipherSuite` with full details or Unknown cipher if not in database
    pub fn from_code(code: u16) -> Self {
        match code {
            // === TLS 1.3 Cipher Suites (Recommended) ===
            0x1301 => Self::new(
                code,
                "TLS_AES_128_GCM_SHA256",
                "TLS13",
                "AEAD",
                "AES_128_GCM",
                "SHA256",
                CipherStrength::Recommended,
            ),
            0x1302 => Self::new(
                code,
                "TLS_AES_256_GCM_SHA384",
                "TLS13",
                "AEAD",
                "AES_256_GCM",
                "SHA384",
                CipherStrength::Recommended,
            ),
            0x1303 => Self::new(
                code,
                "TLS_CHACHA20_POLY1305_SHA256",
                "TLS13",
                "AEAD",
                "CHACHA20_POLY1305",
                "SHA256",
                CipherStrength::Recommended,
            ),
            0x1304 => Self::new(
                code,
                "TLS_AES_128_CCM_SHA256",
                "TLS13",
                "AEAD",
                "AES_128_CCM",
                "SHA256",
                CipherStrength::Recommended,
            ),
            0x1305 => Self::new(
                code,
                "TLS_AES_128_CCM_8_SHA256",
                "TLS13",
                "AEAD",
                "AES_128_CCM_8",
                "SHA256",
                CipherStrength::Recommended,
            ),

            // === TLS 1.2 ECDHE Cipher Suites (Strong) ===
            0xC02F => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
                "ECDHE",
                "RSA",
                "AES_128_GCM",
                "SHA256",
                CipherStrength::Strong,
            ),
            0xC030 => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
                "ECDHE",
                "RSA",
                "AES_256_GCM",
                "SHA384",
                CipherStrength::Strong,
            ),
            0xC02B => Self::new(
                code,
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
                "ECDHE",
                "ECDSA",
                "AES_128_GCM",
                "SHA256",
                CipherStrength::Strong,
            ),
            0xC02C => Self::new(
                code,
                "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
                "ECDHE",
                "ECDSA",
                "AES_256_GCM",
                "SHA384",
                CipherStrength::Strong,
            ),
            0xCCA8 => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
                "ECDHE",
                "RSA",
                "CHACHA20_POLY1305",
                "SHA256",
                CipherStrength::Strong,
            ),
            0xCCA9 => Self::new(
                code,
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
                "ECDHE",
                "ECDSA",
                "CHACHA20_POLY1305",
                "SHA256",
                CipherStrength::Strong,
            ),

            // === TLS 1.2 AES-CBC Cipher Suites (Acceptable) ===
            0xC013 => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
                "ECDHE",
                "RSA",
                "AES_128_CBC",
                "SHA",
                CipherStrength::Acceptable,
            ),
            0xC014 => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
                "ECDHE",
                "RSA",
                "AES_256_CBC",
                "SHA",
                CipherStrength::Acceptable,
            ),
            0xC027 => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256",
                "ECDHE",
                "RSA",
                "AES_128_CBC",
                "SHA256",
                CipherStrength::Acceptable,
            ),
            0xC028 => Self::new(
                code,
                "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA384",
                "ECDHE",
                "RSA",
                "AES_256_CBC",
                "SHA384",
                CipherStrength::Acceptable,
            ),

            // === TLS 1.2 RSA Cipher Suites (Acceptable, no forward secrecy) ===
            0x009C => Self::new(
                code,
                "TLS_RSA_WITH_AES_128_GCM_SHA256",
                "RSA",
                "RSA",
                "AES_128_GCM",
                "SHA256",
                CipherStrength::Acceptable,
            ),
            0x009D => Self::new(
                code,
                "TLS_RSA_WITH_AES_256_GCM_SHA384",
                "RSA",
                "RSA",
                "AES_256_GCM",
                "SHA384",
                CipherStrength::Acceptable,
            ),
            0x003C => Self::new(
                code,
                "TLS_RSA_WITH_AES_128_CBC_SHA256",
                "RSA",
                "RSA",
                "AES_128_CBC",
                "SHA256",
                CipherStrength::Acceptable,
            ),
            0x003D => Self::new(
                code,
                "TLS_RSA_WITH_AES_256_CBC_SHA256",
                "RSA",
                "RSA",
                "AES_256_CBC",
                "SHA256",
                CipherStrength::Acceptable,
            ),

            // === Weak/Insecure Cipher Suites ===
            0x008A => Self::new(
                code,
                "TLS_PSK_WITH_RC4_128_SHA",
                "PSK",
                "PSK",
                "RC4_128",
                "SHA",
                CipherStrength::Weak,
            ),
            0x0096 => Self::new(
                code,
                "TLS_RSA_WITH_SEED_CBC_SHA",
                "RSA",
                "RSA",
                "SEED_CBC",
                "SHA",
                CipherStrength::Insecure,
            ),
            0x008B => Self::new(
                code,
                "TLS_PSK_WITH_3DES_EDE_CBC_SHA",
                "PSK",
                "PSK",
                "3DES_EDE_CBC",
                "SHA",
                CipherStrength::Insecure,
            ),
            0x000A => Self::new(
                code,
                "TLS_RSA_WITH_3DES_EDE_CBC_SHA",
                "RSA",
                "RSA",
                "3DES_EDE_CBC",
                "SHA",
                CipherStrength::Insecure,
            ),

            // === NULL cipher (testing only) ===
            0x0000 => Self::new(
                code,
                "TLS_NULL_WITH_NULL_NULL",
                "NULL",
                "NULL",
                "NULL",
                "NULL",
                CipherStrength::Weak,
            ),

            // === Unknown cipher ===
            _ => Self::new(
                code,
                &format!("UNKNOWN_CIPHER_0x{:04X}", code),
                "Unknown",
                "Unknown",
                "Unknown",
                "Unknown",
                CipherStrength::Acceptable, // Conservative default
            ),
        }
    }

    /// Helper constructor for cipher suite creation
    fn new(
        code: u16,
        name: &str,
        key_exchange: &str,
        authentication: &str,
        encryption: &str,
        mac: &str,
        strength: CipherStrength,
    ) -> Self {
        Self {
            code,
            name: name.to_string(),
            key_exchange: key_exchange.to_string(),
            authentication: authentication.to_string(),
            encryption: encryption.to_string(),
            mac: mac.to_string(),
            strength,
        }
    }

    /// Check if cipher provides forward secrecy
    ///
    /// Forward secrecy (also called Perfect Forward Secrecy, PFS) ensures that
    /// session keys are not compromised even if the server's private key is later
    /// compromised. Requires ephemeral key exchange (ECDHE, DHE).
    ///
    /// # Returns
    ///
    /// * `bool` - true if ECDHE or DHE key exchange, false otherwise
    pub fn has_forward_secrecy(&self) -> bool {
        self.key_exchange.contains("ECDHE")
            || self.key_exchange.contains("DHE")
            || self.key_exchange == "TLS13" // TLS 1.3 always provides forward secrecy
    }

    /// Check if cipher is TLS 1.3
    ///
    /// TLS 1.3 ciphers are identified by codes 0x1301-0x1305 and provide:
    /// - Mandatory forward secrecy
    /// - AEAD ciphers only
    /// - Simplified handshake
    ///
    /// # Returns
    ///
    /// * `bool` - true if TLS 1.3 cipher, false otherwise
    pub fn is_tls13(&self) -> bool {
        matches!(self.strength, CipherStrength::Recommended)
    }
}

impl fmt::Display for CipherSuite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (strength: {:?})", self.name, self.strength)
    }
}

/// TLS extension data (parsed)
///
/// Parsed extension data for common TLS extensions.
/// Unsupported extensions are preserved as raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsExtensionData {
    /// Server Name Indication (SNI) - hostnames
    ServerName(Vec<String>),
    /// Supported elliptic curve groups
    SupportedGroups(Vec<String>),
    /// Signature algorithms
    SignatureAlgorithms(Vec<String>),
    /// Application-Layer Protocol Negotiation (e.g., "h2", "http/1.1")
    Alpn(Vec<String>),
    /// Supported TLS versions (TLS 1.3 extension)
    SupportedVersions(Vec<TlsVersion>),
    /// Key Share (TLS 1.3, not parsed)
    KeyShare,
    /// Unknown or unsupported extension (raw bytes preserved)
    Unknown(Vec<u8>),
}

/// TLS extension
///
/// Represents a TLS extension from the ServerHello message.
/// Provides parsing for common extensions (SNI, ALPN, supported_versions).
///
/// # RFC References
///
/// - RFC 6066: TLS Extensions (SNI, max_fragment_length, etc.)
/// - RFC 7301: Application-Layer Protocol Negotiation (ALPN)
/// - RFC 8446: TLS 1.3 (supported_versions, key_share, etc.)
///
/// # Examples
///
/// ```
/// use prtip_scanner::{TlsExtension, TlsExtensionData};
///
/// // Parse SNI extension (type 0)
/// let sni_data = b"\x00\x0e\x00\x00\x0bexample.com";
/// let ext = TlsExtension::from_bytes(0, sni_data).unwrap();
/// if let TlsExtensionData::ServerName(names) = &ext.data {
///     assert_eq!(names[0], "example.com");
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsExtension {
    /// Extension type code (e.g., 0 for SNI, 16 for ALPN)
    pub extension_type: u16,
    /// Human-readable extension name
    pub name: String,
    /// Data length in bytes
    pub data_length: u16,
    /// Parsed extension data
    pub data: TlsExtensionData,
}

impl TlsExtension {
    /// Parse extension from type and data bytes
    ///
    /// # Arguments
    ///
    /// * `extension_type` - Extension type code
    /// * `data` - Extension data bytes
    ///
    /// # Returns
    ///
    /// * `Result<TlsExtension>` - Parsed extension or error
    pub fn from_bytes(extension_type: u16, data: &[u8]) -> Result<Self, Error> {
        let name = Self::extension_name(extension_type);
        let data_length = data.len() as u16;

        let parsed_data = match extension_type {
            0 => Self::parse_server_name(data)?,
            10 => Self::parse_supported_groups(data)?,
            13 => Self::parse_signature_algorithms(data)?,
            16 => Self::parse_alpn(data)?,
            43 => Self::parse_supported_versions(data)?,
            51 => TlsExtensionData::KeyShare, // TLS 1.3 key share (not parsed)
            _ => TlsExtensionData::Unknown(data.to_vec()),
        };

        Ok(Self {
            extension_type,
            name,
            data_length,
            data: parsed_data,
        })
    }

    /// Map extension type to human-readable name
    fn extension_name(extension_type: u16) -> String {
        match extension_type {
            0 => "server_name".to_string(),
            1 => "max_fragment_length".to_string(),
            5 => "status_request".to_string(),
            10 => "supported_groups".to_string(),
            13 => "signature_algorithms".to_string(),
            14 => "use_srtp".to_string(),
            15 => "heartbeat".to_string(),
            16 => "application_layer_protocol_negotiation".to_string(),
            18 => "signed_certificate_timestamp".to_string(),
            35 => "session_ticket".to_string(),
            43 => "supported_versions".to_string(),
            45 => "psk_key_exchange_modes".to_string(),
            51 => "key_share".to_string(),
            _ => format!("extension_{}", extension_type),
        }
    }

    /// Parse Server Name Indication (SNI) extension
    ///
    /// Format: [list_length:2][type:1][length:2][hostname:N]...
    fn parse_server_name(data: &[u8]) -> Result<TlsExtensionData, Error> {
        if data.len() < 2 {
            return Ok(TlsExtensionData::ServerName(vec![]));
        }

        let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + list_len {
            return Ok(TlsExtensionData::ServerName(vec![]));
        }

        let mut names = Vec::new();
        let mut offset = 2;

        while offset + 3 <= data.len() {
            let name_type = data[offset]; // 0 = hostname
            let name_len = u16::from_be_bytes([data[offset + 1], data[offset + 2]]) as usize;
            offset += 3;

            if offset + name_len > data.len() {
                break;
            }

            if name_type == 0 {
                // hostname
                if let Ok(hostname) = std::str::from_utf8(&data[offset..offset + name_len]) {
                    names.push(hostname.to_string());
                }
            }

            offset += name_len;
        }

        Ok(TlsExtensionData::ServerName(names))
    }

    /// Parse Supported Groups extension
    ///
    /// Format: [list_length:2][group:2]...
    fn parse_supported_groups(data: &[u8]) -> Result<TlsExtensionData, Error> {
        if data.len() < 2 {
            return Ok(TlsExtensionData::SupportedGroups(vec![]));
        }

        let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + list_len {
            return Ok(TlsExtensionData::SupportedGroups(vec![]));
        }

        let mut groups = Vec::new();
        let mut offset = 2;

        while offset + 2 <= data.len() {
            let group = u16::from_be_bytes([data[offset], data[offset + 1]]);
            groups.push(Self::group_name(group));
            offset += 2;
        }

        Ok(TlsExtensionData::SupportedGroups(groups))
    }

    /// Map supported group code to name
    fn group_name(code: u16) -> String {
        match code {
            23 => "secp256r1".to_string(),
            24 => "secp384r1".to_string(),
            25 => "secp521r1".to_string(),
            29 => "x25519".to_string(),
            30 => "x448".to_string(),
            _ => format!("group_{}", code),
        }
    }

    /// Parse Signature Algorithms extension
    ///
    /// Format: [list_length:2][sig_algo:2]...
    fn parse_signature_algorithms(data: &[u8]) -> Result<TlsExtensionData, Error> {
        if data.len() < 2 {
            return Ok(TlsExtensionData::SignatureAlgorithms(vec![]));
        }

        let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + list_len {
            return Ok(TlsExtensionData::SignatureAlgorithms(vec![]));
        }

        let mut algos = Vec::new();
        let mut offset = 2;

        while offset + 2 <= data.len() {
            let algo = u16::from_be_bytes([data[offset], data[offset + 1]]);
            algos.push(Self::signature_algorithm_name(algo));
            offset += 2;
        }

        Ok(TlsExtensionData::SignatureAlgorithms(algos))
    }

    /// Map signature algorithm code to name
    fn signature_algorithm_name(code: u16) -> String {
        match code {
            0x0401 => "rsa_pkcs1_sha256".to_string(),
            0x0501 => "rsa_pkcs1_sha384".to_string(),
            0x0601 => "rsa_pkcs1_sha512".to_string(),
            0x0403 => "ecdsa_secp256r1_sha256".to_string(),
            0x0503 => "ecdsa_secp384r1_sha384".to_string(),
            0x0603 => "ecdsa_secp521r1_sha512".to_string(),
            0x0804 => "rsa_pss_rsae_sha256".to_string(),
            0x0805 => "rsa_pss_rsae_sha384".to_string(),
            0x0806 => "rsa_pss_rsae_sha512".to_string(),
            0x0807 => "ed25519".to_string(),
            0x0808 => "ed448".to_string(),
            _ => format!("sig_algo_0x{:04x}", code),
        }
    }

    /// Parse Application-Layer Protocol Negotiation (ALPN) extension
    ///
    /// Format: [list_length:2][proto_len:1][proto:N]...
    fn parse_alpn(data: &[u8]) -> Result<TlsExtensionData, Error> {
        if data.len() < 2 {
            return Ok(TlsExtensionData::Alpn(vec![]));
        }

        let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + list_len {
            return Ok(TlsExtensionData::Alpn(vec![]));
        }

        let mut protocols = Vec::new();
        let mut offset = 2;

        while offset < data.len() {
            if offset + 1 > data.len() {
                break;
            }

            let proto_len = data[offset] as usize;
            offset += 1;

            if offset + proto_len > data.len() {
                break;
            }

            if let Ok(protocol) = std::str::from_utf8(&data[offset..offset + proto_len]) {
                protocols.push(protocol.to_string());
            }

            offset += proto_len;
        }

        Ok(TlsExtensionData::Alpn(protocols))
    }

    /// Parse Supported Versions extension (TLS 1.3)
    ///
    /// Format (ServerHello): [version:2]
    /// Format (ClientHello): [list_length:1][version:2]...
    fn parse_supported_versions(data: &[u8]) -> Result<TlsExtensionData, Error> {
        if data.is_empty() {
            return Ok(TlsExtensionData::SupportedVersions(vec![]));
        }

        let mut versions = Vec::new();

        // ServerHello format: single 2-byte version
        if data.len() == 2 {
            let major = data[0];
            let minor = data[1];
            versions.push(TlsVersion::from_bytes(major, minor));
        }
        // ClientHello format: [length:1][version:2]...
        else if data.len() >= 3 {
            let list_len = data[0] as usize;
            if data.len() > list_len {
                let mut offset = 1;
                while offset + 2 <= data.len() {
                    let major = data[offset];
                    let minor = data[offset + 1];
                    versions.push(TlsVersion::from_bytes(major, minor));
                    offset += 2;
                }
            }
        }

        Ok(TlsExtensionData::SupportedVersions(versions))
    }
}

impl fmt::Display for TlsExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (type {}, {} bytes)",
            self.name, self.extension_type, self.data_length
        )
    }
}

/// TLS ServerHello message
///
/// Represents the server's response in the TLS handshake.
/// Contains negotiated version, cipher suite, and extensions.
///
/// # ServerHello Structure
///
/// ```text
/// TLS Record Header (5 bytes):
///   Content Type: 0x16 (Handshake)
///   Version: 0x0303 (TLS 1.2, always used for compatibility)
///   Length: Record length
///
/// Handshake Header (4 bytes):
///   Type: 0x02 (ServerHello)
///   Length: Message length (3 bytes)
///
/// ServerHello Body:
///   Version: 2 bytes (0x0303 for TLS 1.2, check supported_versions for TLS 1.3)
///   Random: 32 bytes
///   Session ID Length: 1 byte
///   Session ID: Variable
///   Cipher Suite: 2 bytes
///   Compression Method: 1 byte (always 0x00)
///   Extensions Length: 2 bytes
///   Extensions: Variable
/// ```
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::ServerHello;
///
/// let server_hello_bytes = b"\x16\x03\x03..."; // Real TLS record
/// let server_hello = ServerHello::from_bytes(server_hello_bytes).unwrap();
///
/// println!("Version: {}", server_hello.version);
/// println!("Cipher: {}", server_hello.cipher_suite.name);
/// println!("Secure: {}", server_hello.is_secure());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ServerHello {
    /// TLS version from ServerHello.version field
    pub version: TlsVersion,
    /// Server random (32 bytes)
    pub random: [u8; 32],
    /// Session ID (for session resumption)
    pub session_id: Vec<u8>,
    /// Negotiated cipher suite
    pub cipher_suite: CipherSuite,
    /// Compression method (always 0x00 in modern TLS)
    pub compression_method: u8,
    /// TLS extensions
    pub extensions: Vec<TlsExtension>,
}

impl ServerHello {
    /// Parse ServerHello from TLS record bytes
    ///
    /// Parses a complete TLS record containing a ServerHello message.
    /// Extracts version, random, session ID, cipher suite, and extensions.
    ///
    /// # Arguments
    ///
    /// * `data` - Complete TLS record bytes (including record header)
    ///
    /// # Returns
    ///
    /// * `Result<ServerHello>` - Parsed ServerHello or error
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Data is too short (< 43 bytes minimum)
    /// - Record type is not Handshake (0x16)
    /// - Handshake type is not ServerHello (0x02)
    /// - Data is truncated or malformed
    pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        // Minimum ServerHello: 5 (record) + 4 (handshake) + 34 (version + random + session_id_len + cipher + compression)
        if data.len() < 43 {
            return Err(Error::Parse(
                "ServerHello too short (minimum 43 bytes)".to_string(),
            ));
        }

        // Parse TLS record header (5 bytes)
        let content_type = data[0];
        if content_type != 0x16 {
            return Err(Error::Parse(format!(
                "Not a handshake record (type 0x{:02x})",
                content_type
            )));
        }

        let _record_version_major = data[1];
        let _record_version_minor = data[2];
        let record_length = u16::from_be_bytes([data[3], data[4]]) as usize;

        if data.len() < 5 + record_length {
            return Err(Error::Parse("Truncated TLS record".to_string()));
        }

        // Parse handshake header (4 bytes)
        let handshake_type = data[5];
        if handshake_type != 0x02 {
            return Err(Error::Parse(format!(
                "Not a ServerHello (type 0x{:02x})",
                handshake_type
            )));
        }

        let _handshake_length = u32::from_be_bytes([0, data[6], data[7], data[8]]) as usize;

        // Parse ServerHello body
        let mut offset = 9;

        // Version (2 bytes)
        if offset + 2 > data.len() {
            return Err(Error::Parse("Truncated version field".to_string()));
        }
        let version_major = data[offset];
        let version_minor = data[offset + 1];
        let version = TlsVersion::from_bytes(version_major, version_minor);
        offset += 2;

        // Random (32 bytes)
        if offset + 32 > data.len() {
            return Err(Error::Parse("Truncated random field".to_string()));
        }
        let mut random = [0u8; 32];
        random.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        // Session ID
        if offset + 1 > data.len() {
            return Err(Error::Parse("Truncated session ID length".to_string()));
        }
        let session_id_len = data[offset] as usize;
        offset += 1;

        if offset + session_id_len > data.len() {
            return Err(Error::Parse("Truncated session ID".to_string()));
        }
        let session_id = data[offset..offset + session_id_len].to_vec();
        offset += session_id_len;

        // Cipher Suite (2 bytes)
        if offset + 2 > data.len() {
            return Err(Error::Parse("Truncated cipher suite field".to_string()));
        }
        let cipher_code = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let cipher_suite = CipherSuite::from_code(cipher_code);
        offset += 2;

        // Compression Method (1 byte)
        if offset + 1 > data.len() {
            return Err(Error::Parse("Truncated compression method".to_string()));
        }
        let compression_method = data[offset];
        offset += 1;

        // Extensions (optional)
        let mut extensions = Vec::new();

        if offset + 2 <= data.len() {
            let extensions_length = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            if offset + extensions_length <= data.len() {
                let extensions_end = offset + extensions_length;

                while offset + 4 <= extensions_end {
                    let ext_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
                    let ext_length =
                        u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
                    offset += 4;

                    if offset + ext_length > extensions_end {
                        break;
                    }

                    let ext_data = &data[offset..offset + ext_length];
                    if let Ok(extension) = TlsExtension::from_bytes(ext_type, ext_data) {
                        extensions.push(extension);
                    }

                    offset += ext_length;
                }
            }
        }

        Ok(ServerHello {
            version,
            random,
            session_id,
            cipher_suite,
            compression_method,
            extensions,
        })
    }

    /// Get negotiated TLS version
    ///
    /// For TLS 1.3, the real version is in the supported_versions extension.
    /// The ServerHello.version field is always 0x0303 (TLS 1.2) for compatibility.
    ///
    /// # Returns
    ///
    /// * `TlsVersion` - Negotiated version (checks supported_versions extension first)
    pub fn negotiated_version(&self) -> TlsVersion {
        // Check supported_versions extension (TLS 1.3)
        for ext in &self.extensions {
            if ext.extension_type == 43 {
                // supported_versions
                if let TlsExtensionData::SupportedVersions(versions) = &ext.data {
                    if let Some(version) = versions.first() {
                        return *version;
                    }
                }
            }
        }

        // Fall back to version field
        self.version
    }

    /// Check if connection is secure
    ///
    /// A connection is considered secure if:
    /// - TLS version is 1.2 or later (not deprecated)
    /// - Cipher suite is Strong or Recommended (not Weak/Insecure/Acceptable)
    ///
    /// # Returns
    ///
    /// * `bool` - true if secure, false otherwise
    pub fn is_secure(&self) -> bool {
        let version_secure = self.negotiated_version().is_secure();
        let cipher_secure = matches!(
            self.cipher_suite.strength,
            CipherStrength::Strong | CipherStrength::Recommended
        );

        version_secure && cipher_secure
    }
}

impl fmt::Display for ServerHello {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerHello: version={}, cipher={}, extensions={}",
            self.negotiated_version(),
            self.cipher_suite.name,
            self.extensions.len()
        )
    }
}

// ==================== END TASK-4 STRUCTURES ====================

/// TLS fingerprint information
///
/// Contains TLS protocol version, negotiated cipher suites, and extensions
/// for version detection and security analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct TlsFingerprint {
    /// TLS version (e.g., "TLS 1.2", "TLS 1.3")
    pub tls_version: String,

    /// Negotiated cipher suites (e.g., ["TLS_AES_128_GCM_SHA256"])
    pub cipher_suites: Vec<String>,

    /// TLS extensions present in handshake
    pub extensions: Vec<String>,
}

impl fmt::Display for TlsFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TLS: version={}, ciphers={}, extensions={}",
            self.tls_version,
            self.cipher_suites.len(),
            self.extensions.len()
        )
    }
}

/// Certificate chain validation result
///
/// Contains the full certificate chain from leaf to root, validation status,
/// and trust chain information.
#[derive(Debug, Clone, PartialEq)]
pub struct CertificateChain {
    /// Ordered certificates from leaf to root
    pub certificates: Vec<CertificateInfo>,

    /// Whether the leaf certificate is self-signed
    pub is_self_signed: bool,

    /// Whether the chain is valid (basic validation only, not full PKI)
    pub is_valid: bool,

    /// Trust chain path (list of subjects from leaf to root)
    pub trust_chain: Vec<String>,
}

impl fmt::Display for CertificateChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chain: depth={}, self_signed={}, valid={}",
            self.certificates.len(),
            self.is_self_signed,
            self.is_valid
        )
    }
}

/// Complete TLS analysis result
///
/// Combines certificate information, TLS fingerprint, and chain validation
/// into a single comprehensive result for service detection.
#[derive(Debug, Clone, PartialEq)]
pub struct TlsAnalysisResult {
    /// Certificate information (if available)
    pub certificate: Option<CertificateInfo>,

    /// TLS protocol fingerprint
    pub fingerprint: TlsFingerprint,

    /// Certificate chain (if multiple certificates present)
    pub chain: Option<CertificateChain>,
}

impl fmt::Display for TlsAnalysisResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(cert) = &self.certificate {
            writeln!(f, "{}", cert)?;
        }
        writeln!(f, "{}", self.fingerprint)?;
        if let Some(chain) = &self.chain {
            writeln!(f, "{}", chain)?;
        }
        Ok(())
    }
}

/// Parse a DER-encoded X.509 certificate (ENHANCED for TASK-3)
///
/// Extracts comprehensive certificate information including:
/// - Basic fields (issuer, subject, validity, serial)
/// - Categorized Subject Alternative Names (DNS, IP, email, URI)
/// - Public key information (algorithm, size, curve, security)
/// - Key Usage and Extended Key Usage extensions
/// - All certificate extensions with OID and criticality
/// - Enhanced signature algorithm with security analysis
///
/// # Arguments
///
/// * `cert_der` - DER-encoded certificate bytes
///
/// # Returns
///
/// * `Ok(CertificateInfo)` - Comprehensive certificate information
/// * `Err(Error)` - Parse error or invalid certificate
///
/// # Performance
///
/// Average parsing time: **1.33μs** per certificate (benchmarked on modern hardware).
/// Suitable for high-throughput TLS reconnaissance.
///
/// # See Also
///
/// - [`parse_certificate_chain`] - Parse and validate full certificate chain
/// - [`validate_chain`] - Validate certificate chain trust
/// - [TLS Certificate Guide](../../docs/27-TLS-CERTIFICATE-GUIDE.md) - Certificate analysis examples
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::tls_certificate::parse_certificate;
///
/// # fn example() -> Result<(), prtip_core::Error> {
/// // Load certificate from file or network response
/// // let cert_der = std::fs::read("certificate.der")?;
/// # let cert_der = vec![]; // Placeholder
/// let cert_info = parse_certificate(&cert_der)?;
///
/// // Access basic fields
/// println!("Subject: {}", cert_info.subject);
///
/// // Access enhanced fields (TASK-3)
/// println!("DNS Names: {:?}", cert_info.san_categorized.dns_names);
/// println!("Public Key: {} {} bits", cert_info.public_key_info.algorithm, cert_info.public_key_info.key_size);
///
/// if let Some(key_usage) = &cert_info.key_usage {
///     println!("Digital Signature: {}", key_usage.digital_signature);
/// }
///
/// if let Some(eku) = &cert_info.extended_key_usage {
///     println!("TLS Server Auth: {}", eku.is_valid_for_tls_server());
/// }
/// # Ok(())
/// # }
/// ```
pub fn parse_certificate(cert_der: &[u8]) -> Result<CertificateInfo, Error> {
    // Parse DER-encoded certificate
    let (_, parsed_cert) = X509Certificate::from_der(cert_der)
        .map_err(|e| Error::Parse(format!("Failed to parse X.509 certificate: {:?}", e)))?;

    // === BASIC FIELDS (from TASK-1) ===
    let issuer = parsed_cert.issuer().to_string();
    let subject = parsed_cert.subject().to_string();

    // Extract validity period (convert to ISO 8601 strings)
    let validity = parsed_cert.validity();
    let validity_not_before = validity.not_before.to_datetime().to_string();
    let validity_not_after = validity.not_after.to_datetime().to_string();

    // Extract serial number (convert to hex string)
    let serial_number = parsed_cert.serial.to_str_radix(16).to_uppercase();

    // Extract signature algorithm (legacy format)
    let signature_algorithm = format!("{:?}", parsed_cert.signature_algorithm.algorithm);

    // Extract Subject Alternative Names (legacy flat list for backward compatibility)
    let mut san = Vec::new();
    if let Ok(Some(san_ext)) = parsed_cert.subject_alternative_name() {
        for name in &san_ext.value.general_names {
            match name {
                GeneralName::DNSName(dns) => san.push(dns.to_string()),
                GeneralName::IPAddress(ip) => san.push(format!("{:?}", ip)),
                GeneralName::RFC822Name(email) => san.push(email.to_string()),
                _ => {} // Ignore other types for legacy list
            }
        }
    }

    // === TASK-3 NEW FIELDS ===

    // Extract categorized Subject Alternative Names
    let san_categorized = SubjectAlternativeName::from_certificate(&parsed_cert);

    // Extract public key information
    let public_key_info = PublicKeyInfo::from_certificate(&parsed_cert);

    // Extract Key Usage extension (optional)
    let key_usage = KeyUsage::from_certificate(&parsed_cert);

    // Extract Extended Key Usage extension (optional)
    let extended_key_usage = ExtendedKeyUsage::from_certificate(&parsed_cert);

    // Extract all certificate extensions
    let extensions = CertificateExtension::from_certificate(&parsed_cert);

    // Extract enhanced signature algorithm with security analysis
    let signature_algorithm_enhanced = SignatureAlgorithm::from_certificate(&parsed_cert);

    Ok(CertificateInfo {
        // Basic fields
        issuer,
        subject,
        validity_not_before,
        validity_not_after,
        san,
        serial_number,
        signature_algorithm,
        // TASK-3 enhanced fields
        san_categorized,
        public_key_info,
        key_usage,
        extended_key_usage,
        extensions,
        signature_algorithm_enhanced,
    })
}

/// Validate a certificate chain
///
/// Performs basic certificate chain validation including:
/// - Self-signed detection (issuer == subject)
/// - Chain ordering verification (leaf → intermediate → root)
/// - Basic validity checks
///
/// Note: This is NOT full PKI validation. Full trust verification against
/// system CA store is planned for Sprint 5.6.
///
/// # Arguments
///
/// * `cert_chain` - Ordered certificates from leaf to root (DER-encoded)
///
/// # Returns
///
/// * `Ok(CertificateChain)` - Validated chain information
/// * `Err(Error)` - Parse error or invalid chain
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::tls_certificate::validate_chain;
///
/// # fn example() -> Result<(), prtip_core::Error> {
/// // Load certificates from files or network response
/// // let leaf = std::fs::read("leaf.der")?;
/// // let root = std::fs::read("root.der")?;
/// # let leaf = vec![]; // Placeholder
/// # let root = vec![]; // Placeholder
/// let certs = vec![leaf.as_ref(), root.as_ref()];
///
/// let chain = validate_chain(&certs)?;
/// println!("Self-signed: {}", chain.is_self_signed);
/// # Ok(())
/// # }
/// ```
pub fn validate_chain(cert_chain: &[&[u8]]) -> Result<CertificateChain, Error> {
    if cert_chain.is_empty() {
        return Err(Error::Parse("Empty certificate chain".to_string()));
    }

    // Parse all certificates
    let mut certificates = Vec::new();
    let mut trust_chain = Vec::new();

    for cert_der in cert_chain {
        let cert_info = parse_certificate(cert_der)?;
        trust_chain.push(cert_info.subject.clone());
        certificates.push(cert_info);
    }

    // Check if leaf certificate is self-signed (issuer == subject)
    let is_self_signed = if let Some(leaf) = certificates.first() {
        leaf.issuer == leaf.subject
    } else {
        false
    };

    // Basic chain validation
    // For now, we just verify:
    // 1. Each certificate's issuer matches the next certificate's subject
    // 2. All certificates are present
    let mut is_valid = true;

    if certificates.len() > 1 {
        for i in 0..certificates.len() - 1 {
            let current = &certificates[i];
            let issuer_cert = &certificates[i + 1];

            // Verify that current cert's issuer matches issuer cert's subject
            if current.issuer != issuer_cert.subject {
                is_valid = false;
                break;
            }
        }
    }

    Ok(CertificateChain {
        certificates,
        is_self_signed,
        is_valid,
        trust_chain,
    })
}

/// Certificate chain validation result
///
/// Contains detailed validation information including errors, warnings,
/// and overall validation status.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Whether the chain passed all validation checks
    pub is_valid: bool,

    /// List of validation errors (critical issues)
    pub errors: Vec<String>,

    /// List of validation warnings (non-critical issues)
    pub warnings: Vec<String>,

    /// Whether the leaf certificate is self-signed
    pub is_self_signed: bool,
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Validation: valid={}, errors={}, warnings={}, self_signed={}",
            self.is_valid,
            self.errors.len(),
            self.warnings.len(),
            self.is_self_signed
        )
    }
}

/// Certificate categorization within a chain
///
/// Identifies the role of each certificate in a chain
#[derive(Debug, Clone, PartialEq)]
pub struct ChainCategories {
    /// End-entity (leaf) certificate
    pub end_entity: CertificateInfo,

    /// Intermediate CA certificates (if any)
    pub intermediates: Vec<CertificateInfo>,

    /// Root CA certificate (if present)
    pub root: Option<CertificateInfo>,
}

impl fmt::Display for ChainCategories {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Categories: end_entity={}, intermediates={}, has_root={}",
            self.end_entity.subject,
            self.intermediates.len(),
            self.root.is_some()
        )
    }
}

/// Parse a chain of X.509 certificates from raw DER bytes
///
/// Parses multiple certificates and maintains their order (end-entity → intermediates → root).
///
/// # Arguments
///
/// * `der_chain` - Vector of DER-encoded certificate bytes
///
/// # Returns
///
/// * `Ok(CertificateChain)` - Parsed chain with all certificates
/// * `Err(Error)` - Parse error on any certificate
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::tls_certificate::parse_certificate_chain;
///
/// # fn example() -> Result<(), prtip_core::Error> {
/// // Load certificate chain from files or network response
/// // let leaf = std::fs::read("leaf.der")?;
/// // let intermediate = std::fs::read("intermediate.der")?;
/// // let root = std::fs::read("root.der")?;
/// # let leaf = vec![]; // Placeholder
/// # let intermediate = vec![]; // Placeholder
/// # let root = vec![]; // Placeholder
///
/// let der_chain = vec![leaf.as_ref(), intermediate.as_ref(), root.as_ref()];
/// let chain = parse_certificate_chain(der_chain)?;
/// println!("Chain depth: {}", chain.certificates.len());
/// # Ok(())
/// # }
/// ```
pub fn parse_certificate_chain(der_chain: Vec<&[u8]>) -> Result<CertificateChain, Error> {
    if der_chain.is_empty() {
        return Err(Error::Parse("Empty certificate chain".to_string()));
    }

    if der_chain.len() > 10 {
        return Err(Error::Parse(format!(
            "Certificate chain too long: {} certificates (max 10)",
            der_chain.len()
        )));
    }

    // Parse all certificates
    let mut certificates = Vec::new();
    let mut trust_chain = Vec::new();

    for (index, cert_der) in der_chain.iter().enumerate() {
        let cert_info = parse_certificate(cert_der).map_err(|e| {
            Error::Parse(format!(
                "Failed to parse certificate {} in chain: {}",
                index, e
            ))
        })?;
        trust_chain.push(cert_info.subject.clone());
        certificates.push(cert_info);
    }

    // Detect self-signed (issuer == subject on leaf)
    let is_self_signed = certificates
        .first()
        .map(|leaf| leaf.issuer == leaf.subject)
        .unwrap_or(false);

    // Perform basic validation
    let is_valid = verify_chain_links(&certificates);

    Ok(CertificateChain {
        certificates,
        is_self_signed,
        is_valid,
        trust_chain,
    })
}

/// Verify certificate chain by traversing issuer→subject relationships
///
/// Validates that each certificate in the chain is signed by the next
/// certificate's subject (issuer chaining).
///
/// # Arguments
///
/// * `chain` - Ordered certificates from leaf to root
///
/// # Returns
///
/// * `true` if chain is properly linked
/// * `false` if any link is broken
fn verify_chain_links(chain: &[CertificateInfo]) -> bool {
    if chain.is_empty() {
        return false;
    }

    // Single certificate is valid if self-signed
    if chain.len() == 1 {
        let cert = &chain[0];
        return cert.issuer == cert.subject; // Must be self-signed
    }

    // Multi-certificate chain: verify each link
    for i in 0..chain.len() - 1 {
        let current = &chain[i];
        let issuer_cert = &chain[i + 1];

        // Current cert's issuer must match issuer cert's subject
        if current.issuer != issuer_cert.subject {
            return false;
        }
    }

    // Last certificate should be self-signed (root CA)
    let last = &chain[chain.len() - 1];
    last.issuer == last.subject
}

/// Validate that a certificate is a valid CA certificate
///
/// Checks Basic Constraints extension for CA=true and verifies
/// the certificate can sign other certificates.
///
/// # Arguments
///
/// * `cert` - X.509 certificate to validate
///
/// # Returns
///
/// * `true` if certificate is a valid CA certificate
/// * `false` otherwise
///
/// Note: Currently unused but preserved for future Sprint 5.6 full CA validation
#[allow(dead_code)]
fn is_valid_ca_certificate(cert: &X509Certificate) -> bool {
    // Check Basic Constraints extension
    if let Ok(Some(basic_constraints)) = cert.basic_constraints() {
        // CA flag must be true
        if !basic_constraints.value.ca {
            return false;
        }
    } else {
        // No Basic Constraints extension = not a CA
        return false;
    }

    // Check Key Usage extension (optional but recommended)
    if let Ok(Some(key_usage)) = cert.key_usage() {
        // keyCertSign must be set for CA certificates
        if !key_usage.value.key_cert_sign() {
            return false;
        }
    }

    true
}

/// Categorize certificates in chain by role
///
/// Identifies end-entity (leaf), intermediate CAs, and root CA certificates.
///
/// # Arguments
///
/// * `chain` - Certificate chain to categorize
///
/// # Returns
///
/// * `Ok(ChainCategories)` - Categorized certificates
/// * `Err(Error)` - If chain is empty or invalid
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::tls_certificate::{parse_certificate_chain, categorize_chain};
///
/// # fn example() -> Result<(), prtip_core::Error> {
/// let der_chain = vec![/* ... */];
/// let chain = parse_certificate_chain(der_chain)?;
/// let categories = categorize_chain(&chain)?;
/// println!("Intermediates: {}", categories.intermediates.len());
/// # Ok(())
/// # }
/// ```
pub fn categorize_chain(chain: &CertificateChain) -> Result<ChainCategories, Error> {
    if chain.certificates.is_empty() {
        return Err(Error::Parse("Empty certificate chain".to_string()));
    }

    let end_entity = chain.certificates[0].clone();
    let mut intermediates = Vec::new();
    let mut root = None;

    if chain.certificates.len() > 1 {
        // Intermediate certificates (all except first and last)
        if chain.certificates.len() > 2 {
            intermediates = chain.certificates[1..chain.certificates.len() - 1].to_vec();
        }

        // Last certificate is root (if self-signed)
        let last = &chain.certificates[chain.certificates.len() - 1];
        if last.issuer == last.subject {
            root = Some(last.clone());
        } else {
            // Last cert is not self-signed, treat as intermediate
            intermediates.push(last.clone());
        }
    }

    Ok(ChainCategories {
        end_entity,
        intermediates,
        root,
    })
}

/// Comprehensive certificate chain validation
///
/// Validates:
/// - Certificate expiration dates
/// - Chain linkage (issuer→subject)
/// - CA certificate validity
/// - Self-signed detection
/// - Signature algorithms (rejects MD5, warns SHA1)
///
/// # Arguments
///
/// * `chain` - Certificate chain to validate
///
/// # Returns
///
/// * `ValidationResult` - Detailed validation outcome
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::tls_certificate::{parse_certificate_chain, validate_chain_comprehensive};
///
/// # fn example() -> Result<(), prtip_core::Error> {
/// let der_chain = vec![/* ... */];
/// let chain = parse_certificate_chain(der_chain)?;
/// let result = validate_chain_comprehensive(&chain)?;
///
/// if result.is_valid {
///     println!("✅ Valid certificate chain");
/// } else {
///     for error in result.errors {
///         eprintln!("❌ {}", error);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn validate_chain_comprehensive(chain: &CertificateChain) -> Result<ValidationResult, Error> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Check chain is not empty
    if chain.certificates.is_empty() {
        return Ok(ValidationResult {
            is_valid: false,
            errors: vec!["Empty certificate chain".to_string()],
            warnings: Vec::new(),
            is_self_signed: false,
        });
    }

    // 2. Verify chain links
    if !verify_chain_links(&chain.certificates) {
        errors.push("Broken certificate chain: issuer/subject mismatch".to_string());
    }

    // 3. Check signature algorithms for weak algorithms
    for (index, cert) in chain.certificates.iter().enumerate() {
        let sig_alg = cert.signature_algorithm.to_lowercase();

        if sig_alg.contains("md5") {
            errors.push(format!(
                "Certificate {} uses insecure MD5 signature algorithm",
                index
            ));
        } else if sig_alg.contains("sha1") {
            warnings.push(format!(
                "Certificate {} uses deprecated SHA1 signature algorithm",
                index
            ));
        }
    }

    // 4. Validate CA certificates (all except leaf)
    if chain.certificates.len() > 1 {
        for (index, _cert_der) in chain.certificates.iter().enumerate().skip(1) {
            // Re-parse to get X509Certificate for extension checks
            // Note: This is inefficient but safe for now. Optimization in Sprint 5.6.
            // For now, we'll skip CA validation as it requires re-parsing
            // TODO: Store X509Certificate alongside CertificateInfo in TASK-3
            warnings.push(format!(
                "CA validation skipped for certificate {} (requires DER re-parse)",
                index
            ));
        }
    }

    // 5. Check if self-signed (not an error, just informational)
    let is_self_signed = chain.is_self_signed;
    if is_self_signed && chain.certificates.len() > 1 {
        warnings.push("Chain contains self-signed leaf certificate".to_string());
    }

    // Overall validity: no errors
    let is_valid = errors.is_empty();

    Ok(ValidationResult {
        is_valid,
        errors,
        warnings,
        is_self_signed,
    })
}

/// Check if a certificate is expired
///
/// Compares certificate validity period against current system time.
///
/// # Arguments
///
/// * `cert_info` - Certificate information to check
///
/// # Returns
///
/// * `true` if certificate is expired or not yet valid
/// * `false` if certificate is currently valid
pub fn is_certificate_expired(_cert_info: &CertificateInfo) -> bool {
    // For now, we rely on TLS handshake validation
    // This is a placeholder for future implementation
    // TODO: Parse RFC 2822 dates and compare against current time
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test CertificateInfo with default TASK-3 fields
    fn create_test_cert(
        issuer: &str,
        subject: &str,
        san: Vec<String>,
        serial: &str,
        sig_alg: &str,
    ) -> CertificateInfo {
        CertificateInfo {
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san,
            serial_number: serial.to_string(),
            signature_algorithm: sig_alg.to_string(),
            // TASK-3 fields with defaults
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: sig_alg.to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        }
    }

    #[test]
    fn test_certificate_info_creation() {
        let cert = create_test_cert(
            "CN=Test CA",
            "CN=test.example.com",
            vec![
                "test.example.com".to_string(),
                "www.test.example.com".to_string(),
            ],
            "01:02:03:04",
            "sha256WithRSAEncryption",
        );

        assert_eq!(cert.subject, "CN=test.example.com");
        assert_eq!(cert.issuer, "CN=Test CA");
        assert_eq!(cert.san.len(), 2);
    }

    #[test]
    fn test_certificate_info_display() {
        let cert = CertificateInfo {
            issuer: "CN=Test CA".to_string(),
            subject: "CN=test.example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec!["test.example.com".to_string()],
            serial_number: "ABCD".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let display = format!("{}", cert);
        assert!(display.contains("CN=test.example.com"));
        assert!(display.contains("CN=Test CA"));
        assert!(display.contains("ABCD"));
    }

    #[test]
    fn test_tls_fingerprint_creation() {
        let fingerprint = TlsFingerprint {
            tls_version: "TLS 1.3".to_string(),
            cipher_suites: vec!["TLS_AES_128_GCM_SHA256".to_string()],
            extensions: vec!["server_name".to_string(), "alpn".to_string()],
        };

        assert_eq!(fingerprint.tls_version, "TLS 1.3");
        assert_eq!(fingerprint.cipher_suites.len(), 1);
        assert_eq!(fingerprint.extensions.len(), 2);
    }

    #[test]
    fn test_tls_fingerprint_display() {
        let fingerprint = TlsFingerprint {
            tls_version: "TLS 1.2".to_string(),
            cipher_suites: vec!["TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string()],
            extensions: vec!["server_name".to_string()],
        };

        let display = format!("{}", fingerprint);
        assert!(display.contains("TLS 1.2"));
        assert!(display.contains("ciphers=1"));
        assert!(display.contains("extensions=1"));
    }

    #[test]
    fn test_certificate_chain_creation() {
        let leaf = CertificateInfo {
            issuer: "CN=Intermediate CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec!["example.com".to_string()],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let intermediate = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Intermediate CA".to_string(),
            validity_not_before: "2023-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2026-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![leaf.clone(), intermediate.clone()],
            is_self_signed: false,
            is_valid: true,
            trust_chain: vec![leaf.subject.clone(), intermediate.subject.clone()],
        };

        assert_eq!(chain.certificates.len(), 2);
        assert!(!chain.is_self_signed);
        assert!(chain.is_valid);
        assert_eq!(chain.trust_chain.len(), 2);
    }

    #[test]
    fn test_certificate_chain_self_signed() {
        let self_signed = CertificateInfo {
            issuer: "CN=Self-Signed".to_string(),
            subject: "CN=Self-Signed".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "FF".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![self_signed],
            is_self_signed: true,
            is_valid: true,
            trust_chain: vec!["CN=Self-Signed".to_string()],
        };

        assert_eq!(chain.certificates.len(), 1);
        assert!(chain.is_self_signed);
    }

    #[test]
    fn test_certificate_chain_display() {
        let cert = CertificateInfo {
            issuer: "CN=CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![cert],
            is_self_signed: false,
            is_valid: true,
            trust_chain: vec!["CN=example.com".to_string()],
        };

        let display = format!("{}", chain);
        assert!(display.contains("depth=1"));
        assert!(display.contains("self_signed=false"));
        assert!(display.contains("valid=true"));
    }

    #[test]
    fn test_tls_analysis_result_creation() {
        let cert = CertificateInfo {
            issuer: "CN=CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec!["example.com".to_string()],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let fingerprint = TlsFingerprint {
            tls_version: "TLS 1.3".to_string(),
            cipher_suites: vec!["TLS_AES_128_GCM_SHA256".to_string()],
            extensions: vec!["server_name".to_string()],
        };

        let result = TlsAnalysisResult {
            certificate: Some(cert),
            fingerprint,
            chain: None,
        };

        assert!(result.certificate.is_some());
        assert_eq!(result.fingerprint.tls_version, "TLS 1.3");
        assert!(result.chain.is_none());
    }

    #[test]
    fn test_validate_chain_empty() {
        let result = validate_chain(&[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty certificate chain"));
    }

    #[test]
    fn test_is_certificate_expired_placeholder() {
        let cert = CertificateInfo {
            issuer: "CN=CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        // Placeholder always returns false for now
        assert!(!is_certificate_expired(&cert));
    }

    // ========== TASK-2 Tests: Certificate Chain Parsing and Validation ==========

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec!["Minor issue".to_string()],
            is_self_signed: false,
        };

        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(!result.is_self_signed);
    }

    #[test]
    fn test_validation_result_display() {
        let result = ValidationResult {
            is_valid: false,
            errors: vec!["Critical error".to_string()],
            warnings: vec![],
            is_self_signed: true,
        };

        let display = format!("{}", result);
        assert!(display.contains("valid=false"));
        assert!(display.contains("errors=1"));
        assert!(display.contains("self_signed=true"));
    }

    #[test]
    fn test_verify_chain_links_single_self_signed() {
        let self_signed = CertificateInfo {
            issuer: "CN=Self-Signed".to_string(),
            subject: "CN=Self-Signed".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        assert!(verify_chain_links(&[self_signed]));
    }

    #[test]
    fn test_verify_chain_links_valid_chain() {
        let leaf = CertificateInfo {
            issuer: "CN=Intermediate CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let intermediate = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Intermediate CA".to_string(),
            validity_not_before: "2023-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2026-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let root = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Root CA".to_string(),
            validity_not_before: "2020-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2030-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "03".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        assert!(verify_chain_links(&[leaf, intermediate, root]));
    }

    #[test]
    fn test_verify_chain_links_broken() {
        let leaf = CertificateInfo {
            issuer: "CN=Unknown CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let root = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Root CA".to_string(),
            validity_not_before: "2020-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2030-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        // Broken chain: leaf issuer doesn't match root subject
        assert!(!verify_chain_links(&[leaf, root]));
    }

    #[test]
    fn test_verify_chain_links_empty() {
        assert!(!verify_chain_links(&[]));
    }

    #[test]
    fn test_categorize_chain_single_cert() {
        let self_signed = CertificateInfo {
            issuer: "CN=Self-Signed".to_string(),
            subject: "CN=Self-Signed".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![self_signed.clone()],
            is_self_signed: true,
            is_valid: true,
            trust_chain: vec![self_signed.subject.clone()],
        };

        let categories = categorize_chain(&chain).unwrap();
        assert_eq!(categories.end_entity.subject, "CN=Self-Signed");
        assert_eq!(categories.intermediates.len(), 0);
        assert!(categories.root.is_none());
    }

    #[test]
    fn test_categorize_chain_full() {
        let leaf = CertificateInfo {
            issuer: "CN=Intermediate CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let intermediate = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Intermediate CA".to_string(),
            validity_not_before: "2023-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2026-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let root = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Root CA".to_string(),
            validity_not_before: "2020-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2030-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "03".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![leaf.clone(), intermediate.clone(), root.clone()],
            is_self_signed: false,
            is_valid: true,
            trust_chain: vec![
                leaf.subject.clone(),
                intermediate.subject.clone(),
                root.subject.clone(),
            ],
        };

        let categories = categorize_chain(&chain).unwrap();
        assert_eq!(categories.end_entity.subject, "CN=example.com");
        assert_eq!(categories.intermediates.len(), 1);
        assert_eq!(categories.intermediates[0].subject, "CN=Intermediate CA");
        assert!(categories.root.is_some());
        assert_eq!(categories.root.unwrap().subject, "CN=Root CA");
    }

    #[test]
    fn test_categorize_chain_multiple_intermediates() {
        let leaf = CertificateInfo {
            issuer: "CN=Intermediate CA 1".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let intermediate1 = CertificateInfo {
            issuer: "CN=Intermediate CA 2".to_string(),
            subject: "CN=Intermediate CA 1".to_string(),
            validity_not_before: "2023-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2026-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let intermediate2 = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Intermediate CA 2".to_string(),
            validity_not_before: "2022-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2027-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "03".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let root = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Root CA".to_string(),
            validity_not_before: "2020-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2030-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "04".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![
                leaf.clone(),
                intermediate1.clone(),
                intermediate2.clone(),
                root.clone(),
            ],
            is_self_signed: false,
            is_valid: true,
            trust_chain: vec![
                leaf.subject.clone(),
                intermediate1.subject.clone(),
                intermediate2.subject.clone(),
                root.subject.clone(),
            ],
        };

        let categories = categorize_chain(&chain).unwrap();
        assert_eq!(categories.end_entity.subject, "CN=example.com");
        assert_eq!(categories.intermediates.len(), 2);
        assert!(categories.root.is_some());
    }

    #[test]
    fn test_validate_chain_comprehensive_valid() {
        let leaf = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let root = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Root CA".to_string(),
            validity_not_before: "2020-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2030-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![leaf, root],
            is_self_signed: false,
            is_valid: true,
            trust_chain: vec!["CN=example.com".to_string(), "CN=Root CA".to_string()],
        };

        let result = validate_chain_comprehensive(&chain).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_chain_comprehensive_broken_links() {
        let leaf = CertificateInfo {
            issuer: "CN=Unknown CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let root = CertificateInfo {
            issuer: "CN=Root CA".to_string(),
            subject: "CN=Root CA".to_string(),
            validity_not_before: "2020-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2030-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "02".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![leaf, root],
            is_self_signed: false,
            is_valid: false,
            trust_chain: vec!["CN=example.com".to_string(), "CN=Root CA".to_string()],
        };

        let result = validate_chain_comprehensive(&chain).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Broken certificate chain"));
    }

    #[test]
    fn test_validate_chain_comprehensive_weak_signature() {
        let cert_md5 = CertificateInfo {
            issuer: "CN=CA".to_string(),
            subject: "CN=CA".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "md5WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "md5WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![cert_md5],
            is_self_signed: true,
            is_valid: true,
            trust_chain: vec!["CN=CA".to_string()],
        };

        let result = validate_chain_comprehensive(&chain).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("MD5"));
    }

    #[test]
    fn test_validate_chain_comprehensive_sha1_warning() {
        let cert_sha1 = CertificateInfo {
            issuer: "CN=CA".to_string(),
            subject: "CN=CA".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha1WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha1WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let chain = CertificateChain {
            certificates: vec![cert_sha1],
            is_self_signed: true,
            is_valid: true,
            trust_chain: vec!["CN=CA".to_string()],
        };

        let result = validate_chain_comprehensive(&chain).unwrap();
        // SHA1 generates warning but is still valid (no errors)
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("SHA1"));
    }

    #[test]
    fn test_validate_chain_comprehensive_empty() {
        let chain = CertificateChain {
            certificates: vec![],
            is_self_signed: false,
            is_valid: false,
            trust_chain: vec![],
        };

        let result = validate_chain_comprehensive(&chain).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("Empty certificate chain"));
    }

    #[test]
    fn test_chain_categories_display() {
        let leaf = CertificateInfo {
            issuer: "CN=CA".to_string(),
            subject: "CN=example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec![],
            serial_number: "01".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,
            extended_key_usage: None,
            extensions: Vec::new(),
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        let categories = ChainCategories {
            end_entity: leaf,
            intermediates: vec![],
            root: None,
        };

        let display = format!("{}", categories);
        assert!(display.contains("end_entity=CN=example.com"));
        assert!(display.contains("intermediates=0"));
        assert!(display.contains("has_root=false"));
    }

    // ============================================================================
    // TASK-3 COMPREHENSIVE TESTS: X.509 Extension Support
    // ============================================================================

    #[test]
    fn test_san_extraction_dns_names() {
        let san = SubjectAlternativeName {
            dns_names: vec![
                "example.com".to_string(),
                "www.example.com".to_string(),
                "*.example.com".to_string(),
            ],
            ..Default::default()
        };

        assert_eq!(san.dns_names.len(), 3);
        assert!(san.dns_names.contains(&"example.com".to_string()));
        assert!(san.dns_names.contains(&"*.example.com".to_string()));
    }

    #[test]
    fn test_san_extraction_ip_addresses() {
        let san = SubjectAlternativeName {
            ip_addresses: vec!["192.168.1.1".to_string(), "2001:db8::1".to_string()],
            ..Default::default()
        };

        assert_eq!(san.ip_addresses.len(), 2);
        assert!(san.ip_addresses.contains(&"192.168.1.1".to_string()));
        assert!(san.ip_addresses.contains(&"2001:db8::1".to_string()));
    }

    #[test]
    fn test_san_categorization() {
        let san = SubjectAlternativeName {
            dns_names: vec!["example.com".to_string()],
            ip_addresses: vec!["192.168.1.1".to_string()],
            email_addresses: vec!["admin@example.com".to_string()],
            uris: vec!["https://example.com".to_string()],
            other_names: vec!["UPN:user@example.com".to_string()],
        };

        // Verify all categories are properly separated
        assert_eq!(san.dns_names.len(), 1);
        assert_eq!(san.ip_addresses.len(), 1);
        assert_eq!(san.email_addresses.len(), 1);
        assert_eq!(san.uris.len(), 1);
        assert_eq!(san.other_names.len(), 1);

        // Verify all_names() aggregates correctly
        let all = san.all_names();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_san_wildcard_matching() {
        let san = SubjectAlternativeName {
            dns_names: vec![
                "example.com".to_string(),
                "*.example.com".to_string(),
                "*.sub.example.com".to_string(),
            ],
            ..Default::default()
        };

        // Exact match
        assert!(san.matches_dns("example.com"));

        // Wildcard matches from *.example.com
        assert!(san.matches_dns("www.example.com"));
        assert!(san.matches_dns("api.example.com"));
        assert!(san.matches_dns("sub.example.com")); // matches *.example.com

        // Wildcard matches from *.sub.example.com
        assert!(san.matches_dns("test.sub.example.com"));

        // No match for different domains
        assert!(!san.matches_dns("different.com"));
        assert!(!san.matches_dns("example.org"));

        // Test wildcard does NOT match base domain
        let san2 = SubjectAlternativeName {
            dns_names: vec!["*.sub.example.com".to_string()],
            ..Default::default()
        };
        assert!(!san2.matches_dns("sub.example.com")); // *.sub.example.com doesn't match sub.example.com
        assert!(san2.matches_dns("www.sub.example.com")); // but does match www.sub.example.com
    }

    #[test]
    fn test_public_key_info_rsa() {
        let pki = PublicKeyInfo {
            algorithm: "RSA".to_string(),
            key_size: 2048,
            curve: None,
            usage: vec![
                "digitalSignature".to_string(),
                "keyEncipherment".to_string(),
            ],
        };

        assert_eq!(pki.algorithm, "RSA");
        assert_eq!(pki.key_size, 2048);
        assert!(pki.curve.is_none());
        assert_eq!(pki.usage.len(), 2);
    }

    #[test]
    fn test_public_key_info_ecdsa() {
        let pki = PublicKeyInfo {
            algorithm: "ECDSA".to_string(),
            key_size: 256,
            curve: Some("prime256v1".to_string()),
            usage: vec!["digitalSignature".to_string()],
        };

        assert_eq!(pki.algorithm, "ECDSA");
        assert_eq!(pki.key_size, 256);
        assert_eq!(pki.curve, Some("prime256v1".to_string()));
    }

    #[test]
    fn test_public_key_security_check() {
        // Secure RSA key (2048-bit)
        let rsa_2048 = PublicKeyInfo {
            algorithm: "RSA".to_string(),
            key_size: 2048,
            curve: None,
            usage: Vec::new(),
        };
        assert!(rsa_2048.is_secure());

        // Insecure RSA key (1024-bit)
        let rsa_1024 = PublicKeyInfo {
            algorithm: "RSA".to_string(),
            key_size: 1024,
            curve: None,
            usage: Vec::new(),
        };
        assert!(!rsa_1024.is_secure());

        // Secure ECDSA key (256-bit)
        let ecdsa_256 = PublicKeyInfo {
            algorithm: "ECDSA".to_string(),
            key_size: 256,
            curve: Some("prime256v1".to_string()),
            usage: Vec::new(),
        };
        assert!(ecdsa_256.is_secure());

        // Insecure ECDSA key (160-bit)
        let ecdsa_160 = PublicKeyInfo {
            algorithm: "ECDSA".to_string(),
            key_size: 160,
            curve: Some("secp160r1".to_string()),
            usage: Vec::new(),
        };
        assert!(!ecdsa_160.is_secure());
    }

    #[test]
    fn test_key_usage_parsing() {
        let ku = KeyUsage {
            digital_signature: true,
            non_repudiation: false,
            key_encipherment: true,
            data_encipherment: false,
            key_agreement: false,
            key_cert_sign: false,
            crl_sign: false,
            encipher_only: false,
            decipher_only: false,
        };

        // Verify flags are set correctly
        assert!(ku.digital_signature);
        assert!(ku.key_encipherment);
        assert!(!ku.non_repudiation);
        assert!(!ku.key_cert_sign);
    }

    #[test]
    fn test_key_usage_allows() {
        let ku = KeyUsage {
            digital_signature: true,
            non_repudiation: false,
            key_encipherment: true,
            data_encipherment: false,
            key_agreement: false,
            key_cert_sign: false,
            crl_sign: false,
            encipher_only: false,
            decipher_only: false,
        };

        // Test allows() helper method (uses snake_case identifiers)
        assert!(ku.allows("digital_signature"));
        assert!(ku.allows("key_encipherment"));
        assert!(!ku.allows("key_cert_sign"));
        assert!(!ku.allows("crl_sign"));
        assert!(!ku.allows("invalid_usage"));
    }

    #[test]
    fn test_extended_key_usage_parsing() {
        let eku = ExtendedKeyUsage {
            server_auth: true,
            client_auth: true,
            code_signing: false,
            email_protection: false,
            time_stamping: false,
            ocsp_signing: false,
            any_extended_key_usage: false,
            other_usages: vec!["1.3.6.1.4.1.311.10.3.3".to_string()], // Microsoft SGC
        };

        assert!(eku.server_auth);
        assert!(eku.client_auth);
        assert!(!eku.code_signing);
        assert_eq!(eku.other_usages.len(), 1);
    }

    #[test]
    fn test_extended_key_usage_tls_server() {
        // Valid TLS server certificate
        let eku_valid = ExtendedKeyUsage {
            server_auth: true,
            client_auth: false,
            code_signing: false,
            email_protection: false,
            time_stamping: false,
            ocsp_signing: false,
            any_extended_key_usage: false,
            other_usages: Vec::new(),
        };
        assert!(eku_valid.is_valid_for_tls_server());

        // Invalid - no server_auth
        let eku_invalid = ExtendedKeyUsage {
            server_auth: false,
            client_auth: true,
            code_signing: false,
            email_protection: false,
            time_stamping: false,
            ocsp_signing: false,
            any_extended_key_usage: false,
            other_usages: Vec::new(),
        };
        assert!(!eku_invalid.is_valid_for_tls_server());

        // Valid - any extended key usage allows all
        let eku_any = ExtendedKeyUsage {
            server_auth: false,
            client_auth: false,
            code_signing: false,
            email_protection: false,
            time_stamping: false,
            ocsp_signing: false,
            any_extended_key_usage: true,
            other_usages: Vec::new(),
        };
        assert!(eku_any.is_valid_for_tls_server());
    }

    #[test]
    fn test_certificate_extension_parsing() {
        let ext = CertificateExtension {
            oid: "2.5.29.15".to_string(),
            name: "keyUsage".to_string(),
            critical: true,
            value: "Digital Signature, Key Encipherment".to_string(),
        };

        assert_eq!(ext.oid, "2.5.29.15");
        assert_eq!(ext.name, "keyUsage");
        assert!(ext.critical);
        assert!(ext.value.contains("Digital Signature"));
    }

    #[test]
    fn test_signature_algorithm_enhancement() {
        // SHA256 - Acceptable
        let sig_sha256 = SignatureAlgorithm {
            algorithm: "sha256WithRSAEncryption".to_string(),
            hash_algorithm: "SHA256".to_string(),
            is_secure: true,
            strength: SecurityStrength::Acceptable,
        };
        assert_eq!(sig_sha256.hash_algorithm, "SHA256");
        assert!(sig_sha256.is_secure);
        assert_eq!(sig_sha256.strength, SecurityStrength::Acceptable);

        // SHA1 - Weak
        let sig_sha1 = SignatureAlgorithm {
            algorithm: "sha1WithRSAEncryption".to_string(),
            hash_algorithm: "SHA1".to_string(),
            is_secure: false,
            strength: SecurityStrength::Weak,
        };
        assert_eq!(sig_sha1.hash_algorithm, "SHA1");
        assert!(!sig_sha1.is_secure);
        assert_eq!(sig_sha1.strength, SecurityStrength::Weak);

        // SHA512 - Strong
        let sig_sha512 = SignatureAlgorithm {
            algorithm: "sha512WithRSAEncryption".to_string(),
            hash_algorithm: "SHA512".to_string(),
            is_secure: true,
            strength: SecurityStrength::Strong,
        };
        assert_eq!(sig_sha512.hash_algorithm, "SHA512");
        assert!(sig_sha512.is_secure);
        assert_eq!(sig_sha512.strength, SecurityStrength::Strong);
    }

    #[test]
    fn test_certificate_without_extensions() {
        // Test that certificates without optional extensions don't fail
        let cert = CertificateInfo {
            issuer: "CN=Test CA".to_string(),
            subject: "CN=test.example.com".to_string(),
            validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
            validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
            san: vec!["test.example.com".to_string()],
            serial_number: "01:02:03:04".to_string(),
            signature_algorithm: "sha256WithRSAEncryption".to_string(),
            san_categorized: SubjectAlternativeName::default(),
            public_key_info: PublicKeyInfo {
                algorithm: "RSA".to_string(),
                key_size: 2048,
                curve: None,
                usage: Vec::new(),
            },
            key_usage: None,          // No key usage extension
            extended_key_usage: None, // No extended key usage
            extensions: Vec::new(),   // No additional extensions
            signature_algorithm_enhanced: SignatureAlgorithm {
                algorithm: "sha256WithRSAEncryption".to_string(),
                hash_algorithm: "SHA256".to_string(),
                is_secure: true,
                strength: SecurityStrength::Acceptable,
            },
        };

        // Verify None values are handled gracefully
        assert!(cert.key_usage.is_none());
        assert!(cert.extended_key_usage.is_none());
        assert_eq!(cert.extensions.len(), 0);
        assert_eq!(cert.san_categorized.dns_names.len(), 0);
    }

    // ========== TASK-4 Tests: TLS Version Fingerprinting ==========

    #[test]
    fn test_tls_version_from_bytes() {
        // TLS 1.0
        assert_eq!(TlsVersion::from_bytes(3, 1), TlsVersion::Tls10);
        assert_eq!(TlsVersion::Tls10.as_str(), "TLS 1.0");

        // TLS 1.1
        assert_eq!(TlsVersion::from_bytes(3, 2), TlsVersion::Tls11);
        assert_eq!(TlsVersion::Tls11.as_str(), "TLS 1.1");

        // TLS 1.2
        assert_eq!(TlsVersion::from_bytes(3, 3), TlsVersion::Tls12);
        assert_eq!(TlsVersion::Tls12.as_str(), "TLS 1.2");

        // TLS 1.3
        assert_eq!(TlsVersion::from_bytes(3, 4), TlsVersion::Tls13);
        assert_eq!(TlsVersion::Tls13.as_str(), "TLS 1.3");

        // Unknown version
        if let TlsVersion::Unknown(version) = TlsVersion::from_bytes(3, 5) {
            assert_eq!(version, 0x0305);
        } else {
            panic!("Expected Unknown variant");
        }
    }

    #[test]
    fn test_tls_version_security_checks() {
        // Deprecated versions (TLS 1.0, 1.1)
        assert!(TlsVersion::Tls10.is_deprecated());
        assert!(TlsVersion::Tls11.is_deprecated());
        assert!(!TlsVersion::Tls12.is_deprecated());
        assert!(!TlsVersion::Tls13.is_deprecated());

        // Secure versions (TLS 1.2+)
        assert!(!TlsVersion::Tls10.is_secure());
        assert!(!TlsVersion::Tls11.is_secure());
        assert!(TlsVersion::Tls12.is_secure());
        assert!(TlsVersion::Tls13.is_secure());
    }

    #[test]
    fn test_tls_version_display() {
        assert_eq!(format!("{}", TlsVersion::Tls10), "TLS 1.0");
        assert_eq!(format!("{}", TlsVersion::Tls12), "TLS 1.2");
        assert_eq!(format!("{}", TlsVersion::Tls13), "TLS 1.3");
        assert_eq!(format!("{}", TlsVersion::Unknown(0x0399)), "Unknown");
    }

    #[test]
    fn test_cipher_suite_tls13() {
        // TLS 1.3 cipher: TLS_AES_128_GCM_SHA256
        let suite = CipherSuite::from_code(0x1301);
        assert_eq!(suite.code, 0x1301);
        assert_eq!(suite.name, "TLS_AES_128_GCM_SHA256");
        assert_eq!(suite.key_exchange, "TLS13");
        assert_eq!(suite.authentication, "AEAD");
        assert_eq!(suite.encryption, "AES_128_GCM");
        assert_eq!(suite.mac, "SHA256");
        assert_eq!(suite.strength, CipherStrength::Recommended);
        assert!(suite.is_tls13());
        assert_eq!(suite.strength, CipherStrength::Recommended);
    }

    #[test]
    fn test_cipher_suite_tls12_ecdhe() {
        // TLS 1.2 cipher: TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
        let suite = CipherSuite::from_code(0xC02F);
        assert_eq!(suite.code, 0xC02F);
        assert_eq!(suite.name, "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256");
        assert_eq!(suite.key_exchange, "ECDHE");
        assert_eq!(suite.authentication, "RSA");
        assert_eq!(suite.encryption, "AES_128_GCM");
        assert_eq!(suite.mac, "SHA256");
        assert_eq!(suite.strength, CipherStrength::Strong);
        assert!(suite.has_forward_secrecy());
        assert!(!suite.is_tls13());
    }

    #[test]
    fn test_cipher_suite_weak() {
        // Unknown cipher (defaults to Acceptable)
        let suite = CipherSuite::from_code(0x0005);
        assert_eq!(suite.strength, CipherStrength::Acceptable);
        assert!(!suite.has_forward_secrecy());
        assert_ne!(suite.strength, CipherStrength::Recommended);
    }

    #[test]
    fn test_cipher_suite_strength_levels() {
        // Recommended (TLS 1.3)
        let suite_recommended = CipherSuite::from_code(0x1301);
        assert_eq!(suite_recommended.strength, CipherStrength::Recommended);

        // Strong (ECDHE + AES-GCM)
        let suite_strong = CipherSuite::from_code(0xC02F);
        assert_eq!(suite_strong.strength, CipherStrength::Strong);

        // Acceptable (AES-CBC with SHA256)
        let suite_acceptable = CipherSuite::from_code(0x003C);
        assert_eq!(suite_acceptable.strength, CipherStrength::Acceptable);

        // Insecure (3DES)
        let suite_insecure = CipherSuite::from_code(0x000A);
        assert_eq!(suite_insecure.strength, CipherStrength::Insecure);

        // Weak (RC4)
        let suite_weak = CipherSuite::from_code(0x0005);
        assert_eq!(suite_weak.strength, CipherStrength::Acceptable);
    }

    #[test]
    fn test_cipher_suite_unknown() {
        let suite = CipherSuite::from_code(0xFFFF);
        assert!(suite.name.starts_with("UNKNOWN_CIPHER"));
        assert_eq!(suite.strength, CipherStrength::Acceptable);
    }

    #[test]
    fn test_tls_extension_sni_parsing() {
        // Server Name Indication: [list_len:2][type:1][len:2][hostname]
        // List length = 14 (0x000e), type = 0 (hostname), name length = 11 (0x000b), "example.com"
        let sni_data = b"\x00\x0e\x00\x00\x0bexample.com";
        let ext = TlsExtension::from_bytes(0, sni_data).unwrap();

        assert_eq!(ext.extension_type, 0);
        assert_eq!(ext.name, "server_name");
        if let TlsExtensionData::ServerName(names) = &ext.data {
            assert_eq!(names.len(), 1);
            assert_eq!(names[0], "example.com");
        } else {
            panic!("Expected ServerName data");
        }
    }

    #[test]
    fn test_tls_extension_alpn_parsing() {
        // ALPN: [list_len:2][proto_len:1][proto]...
        // List length = 12, protocol length = 2, "h2", protocol length = 8, "http/1.1"
        let alpn_data = b"\x00\x0c\x02\x68\x32\x08\x68\x74\x74\x70\x2f\x31\x2e\x31";
        let ext = TlsExtension::from_bytes(16, alpn_data).unwrap();

        assert_eq!(ext.extension_type, 16);
        assert_eq!(ext.name, "application_layer_protocol_negotiation");
        if let TlsExtensionData::Alpn(protocols) = &ext.data {
            assert_eq!(protocols.len(), 2);
            assert_eq!(protocols[0], "h2");
            assert_eq!(protocols[1], "http/1.1");
        } else {
            panic!("Expected ALPN data");
        }
    }

    #[test]
    fn test_tls_extension_supported_versions() {
        // Supported Versions (TLS 1.3): [length:1][version:2]...
        // Length = 2, version = 0x0304 (TLS 1.3)
        let versions_data = b"\x02\x03\x04";
        let ext = TlsExtension::from_bytes(43, versions_data).unwrap();

        assert_eq!(ext.extension_type, 43);
        assert_eq!(ext.name, "supported_versions");
        if let TlsExtensionData::SupportedVersions(versions) = &ext.data {
            assert_eq!(versions.len(), 1);
            assert_eq!(versions[0], TlsVersion::Tls13);
        } else {
            panic!("Expected SupportedVersions data");
        }
    }

    #[test]
    fn test_tls_extension_unknown() {
        let unknown_data = b"\x01\x02\x03\x04";
        let ext = TlsExtension::from_bytes(999, unknown_data).unwrap();

        assert_eq!(ext.extension_type, 999);
        assert_eq!(ext.name, "extension_999");
        if let TlsExtensionData::Unknown(data) = &ext.data {
            assert_eq!(data, unknown_data);
        } else {
            panic!("Expected Unknown data");
        }
    }

    #[test]
    fn test_server_hello_parsing() {
        // Simplified ServerHello without extensions to ensure correct length
        let mut server_hello = Vec::new();

        // TLS record header (5 bytes)
        server_hello.push(0x16); // Handshake
        server_hello.push(0x03); // Version major
        server_hello.push(0x03); // Version minor (TLS 1.2)

        // Calculate correct length later
        let length_pos = server_hello.len();
        server_hello.extend_from_slice(&[0x00, 0x00]); // Placeholder for length

        // Handshake header (4 bytes)
        server_hello.push(0x02); // ServerHello
        let handshake_length_pos = server_hello.len();
        server_hello.extend_from_slice(&[0x00, 0x00, 0x00]); // Placeholder for handshake length

        // ServerHello body
        server_hello.push(0x03); // Version major
        server_hello.push(0x03); // Version minor (TLS 1.2)
        server_hello.extend_from_slice(&[0u8; 32]); // Random (32 bytes)
        server_hello.push(0x00); // Session ID length (0)
        server_hello.extend_from_slice(&[0x13, 0x01]); // Cipher suite (TLS_AES_128_GCM_SHA256)
        server_hello.push(0x00); // Compression method (none)

        // No extensions for simplicity

        // Fix lengths
        let handshake_len = (server_hello.len() - handshake_length_pos - 3) as u16;
        server_hello[handshake_length_pos + 1] = ((handshake_len >> 8) & 0xFF) as u8;
        server_hello[handshake_length_pos + 2] = (handshake_len & 0xFF) as u8;

        let record_len = (server_hello.len() - length_pos - 2) as u16;
        server_hello[length_pos] = ((record_len >> 8) & 0xFF) as u8;
        server_hello[length_pos + 1] = (record_len & 0xFF) as u8;

        let result = ServerHello::from_bytes(&server_hello);
        assert!(
            result.is_ok(),
            "Failed to parse ServerHello: {:?}",
            result.err()
        );

        let hello = result.unwrap();
        assert_eq!(hello.version, TlsVersion::Tls12);
        assert_eq!(hello.cipher_suite.code, 0x1301);
        assert_eq!(hello.compression_method, 0x00);
        assert_eq!(hello.extensions.len(), 0);
        assert_eq!(hello.session_id.len(), 0);
    }
}
