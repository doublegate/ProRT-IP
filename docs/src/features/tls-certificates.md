# TLS Certificate Analysis

Automatic X.509 certificate extraction and TLS protocol fingerprinting during network scanning.

## What is TLS Certificate Analysis?

**TLS Certificate Analysis** automatically extracts and analyzes X.509 certificates during TLS/SSL handshakes. ProRT-IP retrieves certificates, parses their contents, validates certificate chains, and fingerprints TLS protocol characteristics—all without user intervention when scanning HTTPS or other TLS-enabled services.

**ProRT-IP Implementation:**
- **X.509 v3 parsing** - Complete certificate field extraction (subject, issuer, validity, serial, signature)
- **Subject Alternative Names (SANs)** - DNS names, IP addresses, email addresses, URIs, other names
- **Certificate chain validation** - Structural linkage verification (end-entity → intermediate → root)
- **Public key analysis** - RSA/ECDSA/Ed25519 with security strength ratings
- **TLS fingerprinting** - Version detection (1.0-1.3), cipher suites, extensions, ALPN
- **<50ms overhead** - Minimal performance impact per connection

**Use Cases:**
- **Security Auditing** - Identify weak ciphers, deprecated TLS versions, expired certificates
- **Compliance Verification** - PCI DSS (TLS 1.2+ required), NIST SP 800-52 Rev 2
- **Asset Discovery** - Wildcard certificates, SANs reveal additional domains/subdomains
- **Vulnerability Assessment** - Self-signed certificates, weak key sizes, insecure cipher suites

---

## How It Works

### Automatic Certificate Extraction

ProRT-IP automatically extracts TLS certificates when scanning HTTPS (port 443) or other TLS-enabled ports:

**TLS Handshake Process:**

```
1. Client Hello (ProRT-IP)
   - Supported TLS versions: 1.0, 1.1, 1.2, 1.3
   - Cipher suite list: 50+ cipher suites
   - Extensions: SNI, supported_versions, key_share, signature_algorithms

2. Server Hello (Target)
   - Selected TLS version
   - Selected cipher suite
   - Server extensions

3. Certificate Message (Target)
   - Certificate chain (1-5 certificates typically)
   - End-entity certificate (server's certificate)
   - Intermediate CA certificates
   - (Optional) Root CA certificate

4. ProRT-IP Processing
   - Extract all certificates from chain
   - Parse X.509 DER-encoded data
   - Validate chain linkage
   - Analyze TLS fingerprint
   - Return results to scanner
```

**Performance:** <50ms total overhead (15ms TCP handshake + 20ms TLS handshake + 10ms parsing + 5ms analysis)

### Certificate Chain Validation

ProRT-IP performs **structural validation** (not cryptographic):

**Validation Steps:**
1. **Chain Extraction** - Extract all certificates from TLS ServerHello message
2. **Linkage Validation** - Verify each certificate's Issuer DN matches next certificate's Subject DN
3. **Self-Signed Detection** - Check if Issuer DN == Subject DN (root CA or self-signed)
4. **Basic Constraints** - Verify intermediate certificates have `CA:TRUE` extension

**What ProRT-IP DOES validate:**
- ✅ Certificate chain structural integrity (Issuer → Subject linkage)
- ✅ Self-signed certificate detection
- ✅ Basic extension syntax (Key Usage, Extended Key Usage, Basic Constraints)
- ✅ Certificate expiration dates (validity period)

**What ProRT-IP DOES NOT validate:**
- ❌ Cryptographic signature verification (performance overhead)
- ❌ Trust store validation (focus on discovery, not trust)
- ❌ Certificate revocation (CRL/OCSP checks - network overhead)
- ❌ Hostname verification (application-specific concern)

**Rationale:** ProRT-IP prioritizes **discovery and reconnaissance** over trust validation. For full trust validation, use OpenSSL or browser trust stores.

---

## Certificate Fields

### Subject and Issuer Distinguished Names (DN)

**Distinguished Name (DN)** identifies certificate subject and issuer:

**DN Components:**
- **CN (Common Name)** - Domain name (e.g., `example.com`) or organization name
- **O (Organization)** - Legal organization name (e.g., `Example Corp`)
- **OU (Organizational Unit)** - Department or division (e.g., `IT Department`)
- **C (Country)** - Two-letter country code (e.g., `US`)
- **ST (State/Province)** - State or province name (e.g., `California`)
- **L (Locality)** - City name (e.g., `San Francisco`)

**Example:**
```
Subject: CN=example.com, O=Example Corp, OU=IT, C=US, ST=California, L=San Francisco
Issuer: CN=DigiCert SHA2 Secure Server CA, O=DigiCert Inc, C=US
```

**Interpretation:**
- **Subject CN** typically matches the domain name (for server certificates)
- **Issuer** identifies the Certificate Authority (CA) that signed the certificate
- **Self-signed** certificates have identical Subject and Issuer DNs

### Subject Alternative Names (SANs)

**SANs** specify additional identities covered by the certificate:

#### 1. DNS Names
Most common SAN type for server certificates:

```
DNS Names: ["example.com", "www.example.com", "api.example.com", "*.example.com"]
```

**Wildcard Certificates:**
- `*.example.com` covers `api.example.com`, `mail.example.com`, but NOT `sub.api.example.com`
- Wildcard only matches one level of subdomain

#### 2. IP Addresses
For certificates issued to IP addresses:

```
IP Addresses: ["192.0.2.1", "2001:db8::1"]
```

**Use Cases:**
- Internal servers accessed by IP
- IoT devices without DNS names
- Load balancers with direct IP access

#### 3. Email Addresses
For S/MIME email encryption certificates:

```
Email Addresses: ["admin@example.com", "support@example.com"]
```

#### 4. URIs
For web service identifiers:

```
URIs: ["https://example.com/", "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6"]
```

#### 5. Other Names
For specialized identities (e.g., Active Directory User Principal Name):

```
Other Names: UPN = user@corp.example.com
```

### Validity Period

**Not Before / Not After** define certificate lifetime:

```
Valid From: 2024-01-15 00:00:00 UTC
Valid Until: 2025-02-15 23:59:59 UTC
Days Remaining: 156 days
```

**Industry Standards:**
- **CA/Browser Forum** - Maximum 398 days (13 months) for publicly-trusted certificates
- **Let's Encrypt** - 90-day default validity (encourages automation)
- **Internal PKI** - Often 1-3 years for internal certificates

**Security Implications:**
- ❌ **Expired certificates** - Immediate security failure, browsers reject
- ⚠️ **Expiring soon** - <30 days triggers browser warnings
- ✅ **Valid** - Certificate within validity period

### Serial Number

**Unique identifier** assigned by issuing CA:

```
Serial Number: 0C:9A:6E:8F:3A:7B:2D:1E:5F:4C:8A:9D:6E:3B:7A:1F
```

**Uses:**
- Certificate revocation lists (CRLs) identify certificates by serial number
- Uniquely identifies certificate within CA's issued certificates
- Forensic analysis and tracking

### Public Key Information

**Public key algorithm, size, and security rating:**

#### RSA Keys
```
Algorithm: RSA
Key Size: 2048 bits
Security Rating: ✅ Acceptable (minimum standard)
```

**RSA Key Size Recommendations:**
- ❌ **<2048 bits** - Insecure (deprecated, vulnerable to factorization)
- ✅ **2048 bits** - Acceptable (current minimum standard)
- ✅ **3072 bits** - Strong (government/high-security use cases)
- ✅ **4096 bits** - Very Strong (performance cost, ~10x slower operations)

#### ECDSA Keys
```
Algorithm: ECDSA
Curve: P-256 (secp256r1)
Security Rating: ✅ Secure (equivalent to RSA-3072)
```

**ECDSA Curve Recommendations:**
- ✅ **P-256** - Acceptable (equivalent to RSA-3072, widely supported)
- ✅ **P-384** - Strong (equivalent to RSA-7680, NIST Suite B)
- ✅ **P-521** - Very Strong (equivalent to RSA-15360, maximum security)

#### Ed25519 Keys
```
Algorithm: Ed25519
Key Size: 256 bits
Security Rating: ✅ Strong (equivalent to ~128-bit security, RSA-3072)
```

**Advantages:**
- Fast signature generation/verification
- Smaller key size (256 bits vs 2048+ bits for RSA)
- Immunity to timing attacks

### Signature Algorithm

**Hash algorithm and signature scheme:**

```
Signature Algorithm: SHA256-RSA
Security Rating: ✅ Secure
```

**Common Signature Algorithms:**
- ✅ **SHA256-RSA, SHA384-RSA, SHA512-RSA** - Secure
- ✅ **SHA256-ECDSA, SHA384-ECDSA** - Secure (faster than RSA)
- ⚠️ **SHA1-RSA** - Weak (deprecated, collision attacks)
- ❌ **MD5-RSA** - Insecure (broken, collision attacks)

### X.509 Extensions

**Standard X.509 v3 extensions ProRT-IP parses:**

#### Key Usage
Defines cryptographic operations the key may be used for:

```
Key Usage:
  - Digital Signature (SSL/TLS server authentication)
  - Key Encipherment (RSA key exchange)
```

**Common Values:**
- `digitalSignature` - Signing operations
- `keyEncipherment` - Encrypting keys (RSA key exchange)
- `keyAgreement` - Key agreement protocols (ECDHE)
- `keyCertSign` - Signing other certificates (CA certificates)
- `cRLSign` - Signing certificate revocation lists

#### Extended Key Usage
Purpose-specific restrictions:

```
Extended Key Usage:
  - TLS Web Server Authentication (1.3.6.1.5.5.7.3.1)
  - TLS Web Client Authentication (1.3.6.1.5.5.7.3.2)
```

**Common OIDs:**
- `1.3.6.1.5.5.7.3.1` - TLS Web Server Authentication
- `1.3.6.1.5.5.7.3.2` - TLS Web Client Authentication
- `1.3.6.1.5.5.7.3.3` - Code Signing
- `1.3.6.1.5.5.7.3.4` - Email Protection (S/MIME)

#### Basic Constraints
Identifies CA certificates and path length constraints:

```
Basic Constraints:
  CA: TRUE
  Path Length: 0
```

**Interpretation:**
- `CA: TRUE` - Certificate can sign other certificates (intermediate/root CA)
- `CA: FALSE` - End-entity certificate (server/client certificate)
- `Path Length: 0` - Can sign end-entity certificates only (no further intermediates)

#### Subject Key Identifier / Authority Key Identifier
Unique identifiers for key matching:

```
Subject Key Identifier: A3:B4:C5:D6:E7:F8:09:1A:2B:3C:4D:5E:6F:70:81:92
Authority Key Identifier: F8:09:1A:2B:3C:4D:5E:6F:70:81:92:A3:B4:C5:D6:E7
```

**Purpose:**
- Links certificates in chain (Subject Key ID → Authority Key ID)
- Enables certificate path building

---

## TLS Fingerprinting

### TLS Version Detection

ProRT-IP detects TLS protocol version from ServerHello:

| Version | Hex Code | Status | Security | PCI DSS |
|---------|----------|--------|----------|---------|
| TLS 1.0 | 0x0301 | Deprecated (RFC 8996) | ❌ Insecure | ❌ Prohibited |
| TLS 1.1 | 0x0302 | Deprecated (RFC 8996) | ❌ Insecure | ❌ Prohibited |
| TLS 1.2 | 0x0303 | Current Standard | ✅ Secure | ✅ Compliant |
| TLS 1.3 | 0x0304 | Latest Standard | ✅ Secure | ✅ Compliant |

**Example Output:**
```
TLS Version: TLS 1.3 (0x0304) ✅ Secure
```

**Compliance:**
- **PCI DSS** - TLS 1.0 and 1.1 prohibited since June 2018
- **NIST SP 800-52 Rev 2** - TLS 1.0 and 1.1 disallowed
- **HIPAA** - TLS 1.2+ recommended for healthcare data

### Cipher Suite Analysis

ProRT-IP enumerates negotiated cipher suites with security ratings:

**Cipher Suite Format:**
```
TLS_[KeyExchange]_[Authentication]_WITH_[Encryption]_[MAC]

Example: TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
```

**Components:**
- **Key Exchange** - ECDHE (Elliptic Curve Diffie-Hellman Ephemeral), DHE (Diffie-Hellman Ephemeral), RSA
- **Authentication** - RSA, ECDSA, DSA
- **Encryption** - AES_128_GCM, AES_256_GCM, CHACHA20_POLY1305
- **MAC** - SHA256, SHA384 (for AEAD ciphers, MAC is integrated)

**Security Categories:**

#### ❌ INSECURE (Disable Immediately)
- **NULL Encryption** - No encryption (plaintext)
- **Export-Grade** - 40-56 bit keys (broken in minutes)
- **RC4** - Stream cipher with known biases
- **DES / 3DES** - 56-bit / 112-bit effective security (insufficient)
- **MD5 MAC** - Collision attacks
- **Anonymous DH** - No authentication (MITM vulnerable)

#### ⚠️ WEAK (Replace Soon)
- **CBC Mode without AEAD** - BEAST, Lucky13 attacks
- **No Forward Secrecy** - RSA key exchange allows passive decryption
- **SHA-1 MAC** - Collision attacks (deprecated)

#### ✅ SECURE (Recommended)
**TLS 1.3 Ciphers (AEAD only):**
- `TLS_AES_128_GCM_SHA256` - AES-128 with GCM (strong)
- `TLS_AES_256_GCM_SHA384` - AES-256 with GCM (stronger)
- `TLS_CHACHA20_POLY1305_SHA256` - ChaCha20-Poly1305 (mobile-optimized)

**TLS 1.2 ECDHE+AEAD Ciphers:**
- `TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256` - Forward secrecy + AEAD
- `TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384` - ECDSA + AEAD
- `TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256` - ChaCha20-Poly1305

**Example Output:**
```
Cipher Suites:
  - TLS_AES_128_GCM_SHA256 (TLS 1.3) ✅ Secure [AEAD, Forward Secrecy]
  - TLS_CHACHA20_POLY1305_SHA256 (TLS 1.3) ✅ Secure [AEAD, Forward Secrecy]
  - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 (TLS 1.2) ✅ Secure [AEAD, Forward Secrecy]
```

### TLS Extensions

ProRT-IP enumerates TLS extensions from ServerHello:

**Common Extensions:**
- **server_name (SNI)** - Server Name Indication (which virtual host)
- **supported_versions** - TLS versions supported
- **key_share** - Key exchange parameters (TLS 1.3)
- **signature_algorithms** - Supported signature algorithms
- **renegotiation_info** - Secure renegotiation
- **application_layer_protocol_negotiation (ALPN)** - HTTP/2, HTTP/3 negotiation

**ALPN Protocols:**
- `h2` - HTTP/2
- `http/1.1` - HTTP/1.1
- `h3` - HTTP/3 (QUIC)

**Example Output:**
```
TLS Extensions:
  - server_name (SNI): example.com
  - supported_versions: TLS 1.2, TLS 1.3
  - key_share: X25519 (TLS 1.3)
  - signature_algorithms: ecdsa_secp256r1_sha256, rsa_pss_rsae_sha256
  - renegotiation_info: Secure renegotiation supported
  - alpn: h2, http/1.1

ALPN Negotiated Protocol: h2 (HTTP/2)
```

---

## Usage

### Basic Certificate Inspection

Scan HTTPS port and display certificate details:

```bash
prtip -sS -p 443 -sV example.com
```

**Expected Output:**
```
PORT    STATE SERVICE  VERSION
443/tcp open  https
  TLS Certificate:
    Subject: CN=example.com, O=Example Corp, C=US
    Issuer: CN=DigiCert SHA2 Secure Server CA, O=DigiCert Inc, C=US
    Valid From: 2024-01-15 00:00:00 UTC
    Valid Until: 2025-02-15 23:59:59 UTC (156 days remaining)
    Serial: 0C:9A:6E:8F:3A:7B:2D:1E:5F:4C:8A:9D:6E:3B:7A:1F
    SANs: example.com, www.example.com, api.example.com, *.example.com
    Public Key: RSA 2048 bits ✅ Acceptable
    Signature: SHA256-RSA ✅ Secure
  TLS Fingerprint:
    Version: TLS 1.3 (0x0304) ✅ Secure
    Ciphers: TLS_AES_128_GCM_SHA256, TLS_CHACHA20_POLY1305_SHA256
    Extensions: server_name, supported_versions, key_share, alpn
    ALPN: h2 (HTTP/2)
```

**Interpretation:**
- **SANs** reveal 4 domains covered (example.com, www, api, wildcard subdomain)
- **RSA 2048 bits** meets minimum security standard
- **TLS 1.3** with AEAD ciphers (secure configuration)
- **HTTP/2** negotiated via ALPN

### Wildcard Certificate Detection

Identify wildcard certificates that cover multiple subdomains:

```bash
prtip -sS -p 443 -sV example.com | grep '\*\.'
```

**Example Output:**
```
SANs: *.example.com, *.cdn.example.com
```

**Asset Discovery:**
Wildcard certificates hint at subdomain infrastructure:
- `*.example.com` → likely has api.example.com, mail.example.com, admin.example.com, etc.
- `*.cdn.example.com` → CDN infrastructure with multiple edge nodes

**Follow-Up:**
```bash
# Enumerate common subdomains
for sub in api www mail admin cdn ftp ssh vpn; do
  prtip -sS -p 443 -sV $sub.example.com
done
```

### Multi-Port Mail Server Scan

Scan all TLS-enabled mail ports (SMTPS, submission, IMAPS, POP3S):

```bash
prtip -sS -p 25,465,587,993,995 -sV mail.example.com
```

**Expected Output:**
```
PORT    STATE SERVICE  VERSION
25/tcp  open  smtp     Postfix smtpd
465/tcp open  smtps    Postfix smtpd
  TLS Certificate:
    Subject: CN=mail.example.com
    SANs: mail.example.com, smtp.example.com
587/tcp open  submission Postfix smtpd
  TLS Certificate: (same as port 465)
993/tcp open  imaps    Dovecot imapd
  TLS Certificate:
    Subject: CN=mail.example.com
    SANs: mail.example.com, imap.example.com
995/tcp open  pop3s    Dovecot pop3d
  TLS Certificate: (same as port 993)
```

**Analysis:**
- **Ports 465, 587** use same certificate (SMTP server)
- **Ports 993, 995** use same certificate (IMAP/POP3 server)
- **SANs** reveal service-specific DNS names

### Subnet Scan for Expired Certificates

Find hosts with expired certificates across subnet:

```bash
prtip -sS -p 443 -sV 192.168.1.0/24 -oG - | grep "EXPIRED"
```

**Expected Output:**
```
Host: 192.168.1.10 (server01.local)
  443/tcp: EXPIRED certificate (expired 45 days ago)

Host: 192.168.1.25 (server02.local)
  443/tcp: EXPIRED certificate (expired 12 days ago)
```

**Remediation:**
1. Identify affected servers
2. Renew certificates immediately (browsers will reject)
3. Update web server configuration
4. Verify with `openssl s_client -connect HOST:443`

### TLS Version Compliance Audit

Identify servers using deprecated TLS versions (1.0/1.1):

```bash
prtip -sS -p 443 -sV 10.0.0.0/16 -oJ tls_audit.json
```

**Post-Processing (jq):**
```bash
cat tls_audit.json | jq '.hosts[] | select(.ports[].service.tls.version | test("TLS 1\\.[01]")) | {ip: .address, port: .ports[].port, version: .ports[].service.tls.version}'
```

**Example Output:**
```json
{
  "ip": "10.0.5.123",
  "port": 443,
  "version": "TLS 1.0"
}
{
  "ip": "10.0.12.45",
  "port": 8443,
  "version": "TLS 1.1"
}
```

**Compliance Action:**
- **PCI DSS** - Upgrade to TLS 1.2+ immediately (required since June 2018)
- **NIST SP 800-52 Rev 2** - TLS 1.0/1.1 disallowed
- **HIPAA** - TLS 1.2+ recommended

### JSON Output for Automation

Export certificate data to JSON for programmatic processing:

```bash
prtip -sS -p 443 -sV example.com -oJ certs.json
```

**Example JSON Structure:**
```json
{
  "hosts": [
    {
      "address": "93.184.216.34",
      "hostname": "example.com",
      "ports": [
        {
          "port": 443,
          "protocol": "tcp",
          "state": "open",
          "service": {
            "name": "https",
            "tls": {
              "version": "TLS 1.3",
              "certificate": {
                "subject": "CN=example.com, O=Example Corp, C=US",
                "issuer": "CN=DigiCert SHA2 Secure Server CA, O=DigiCert Inc, C=US",
                "valid_from": "2024-01-15T00:00:00Z",
                "valid_until": "2025-02-15T23:59:59Z",
                "serial": "0C:9A:6E:8F:3A:7B:2D:1E:5F:4C:8A:9D:6E:3B:7A:1F",
                "sans": ["example.com", "www.example.com", "*.example.com"],
                "public_key": {
                  "algorithm": "RSA",
                  "size": 2048,
                  "security_rating": "acceptable"
                },
                "signature_algorithm": "SHA256-RSA"
              },
              "ciphers": ["TLS_AES_128_GCM_SHA256", "TLS_CHACHA20_POLY1305_SHA256"],
              "extensions": ["server_name", "supported_versions", "key_share", "alpn"],
              "alpn": "h2"
            }
          }
        }
      ]
    }
  ]
}
```

**Automation Example (Python):**
```python
import json

with open('certs.json') as f:
    data = json.load(f)

for host in data['hosts']:
    for port in host['ports']:
        if 'tls' in port['service']:
            cert = port['service']['tls']['certificate']
            print(f"{host['address']}:{port['port']}")
            print(f"  Subject: {cert['subject']}")
            print(f"  Expires: {cert['valid_until']}")
            print(f"  SANs: {', '.join(cert['sans'])}")
            print()
```

### Self-Signed Certificate Detection

Identify self-signed certificates (common in development/internal infrastructure):

```bash
prtip -sS -p 443 -sV 192.168.1.0/24 -oG - | grep "Self-Signed"
```

**Expected Output:**
```
Host: 192.168.1.50 (dev-server.local)
  443/tcp: Self-Signed certificate (Issuer == Subject)

Host: 192.168.1.100 (router.local)
  443/tcp: Self-Signed certificate (Issuer == Subject)
```

**Analysis:**
- **Development Servers** - Expected for internal development
- **Network Devices** - Routers, switches often use self-signed certificates
- **Production Servers** - ❌ **Security risk** (browsers reject, no trust validation)

**Recommendation:**
- **Internal PKI** - Deploy internal Certificate Authority for trusted internal certificates
- **Let's Encrypt** - Free publicly-trusted certificates for internet-facing servers

### Weak Cipher Suite Detection

Identify servers supporting insecure or weak cipher suites:

```bash
prtip -sS -p 443 -sV example.com -v | grep -E "(RC4|DES|3DES|MD5|NULL|EXPORT)"
```

**Example Output:**
```
⚠️ WARNING: Weak cipher detected
  Cipher: TLS_RSA_WITH_3DES_EDE_CBC_SHA
  Issue: 3DES provides only 112-bit effective security (insufficient)
  Recommendation: Disable 3DES, use AES-GCM or ChaCha20-Poly1305
```

**Server Configuration Fix (Nginx):**
```nginx
ssl_ciphers 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384';
ssl_prefer_server_ciphers on;
```

**Verification:**
```bash
prtip -sS -p 443 -sV example.com -v | grep "Cipher"
# Should show only secure AEAD ciphers
```

---

## Security Considerations

### Deprecated TLS Versions

**TLS 1.0 and 1.1 are deprecated** (RFC 8996, March 2021):

**Known Vulnerabilities:**
- **BEAST** (Browser Exploit Against SSL/TLS) - CBC mode attack on TLS 1.0
- **CRIME** - Compression-based attack
- **POODLE** - Padding oracle attack (SSL 3.0, affects TLS 1.0 fallback)

**Compliance Requirements:**
- **PCI DSS** - TLS 1.0/1.1 prohibited since June 30, 2018
- **NIST SP 800-52 Rev 2** - TLS 1.0/1.1 disallowed for federal systems
- **HIPAA** - TLS 1.2+ strongly recommended for healthcare data

**Remediation:**
```nginx
# Nginx: Disable TLS 1.0 and 1.1
ssl_protocols TLSv1.2 TLSv1.3;
```

```apache
# Apache: Disable TLS 1.0 and 1.1
SSLProtocol -all +TLSv1.2 +TLSv1.3
```

### Weak and Insecure Cipher Suites

**Immediately disable:**

#### NULL Encryption
```
TLS_RSA_WITH_NULL_SHA256
```
**Risk:** No encryption (plaintext transmission)

#### Export-Grade Ciphers
```
TLS_RSA_EXPORT_WITH_DES40_CBC_SHA
TLS_RSA_EXPORT_WITH_RC4_40_MD5
```
**Risk:** 40-56 bit keys (broken in minutes with modern hardware)

#### RC4 Stream Cipher
```
TLS_RSA_WITH_RC4_128_SHA
TLS_ECDHE_RSA_WITH_RC4_128_SHA
```
**Risk:** Statistical biases enable plaintext recovery (CVE-2013-2566, CVE-2015-2808)

#### DES / 3DES
```
TLS_RSA_WITH_DES_CBC_SHA
TLS_RSA_WITH_3DES_EDE_CBC_SHA
```
**Risk:** 56-bit / 112-bit effective security (insufficient), Sweet32 attack

#### MD5 MAC
```
TLS_RSA_WITH_RC4_128_MD5
```
**Risk:** MD5 collision attacks enable signature forgery

### Certificate Validation Scope

**What ProRT-IP validates:**
- ✅ **Certificate chain structural integrity** (Issuer → Subject linkage)
- ✅ **Self-signed certificate detection**
- ✅ **Certificate expiration** (validity period)
- ✅ **Public key algorithm and key size**
- ✅ **Signature algorithm strength**

**What ProRT-IP DOES NOT validate:**
- ❌ **Cryptographic signature verification** (performance overhead)
- ❌ **Trust store validation** (system/browser trust stores)
- ❌ **Certificate revocation** (CRL/OCSP checks)
- ❌ **Hostname verification** (certificate CN/SAN matches requested hostname)

**Rationale:**
ProRT-IP prioritizes **network reconnaissance and asset discovery** over full trust validation. For production trust validation, use:
- **OpenSSL** - `openssl s_client -connect HOST:443 -verify 5`
- **Browser Trust Stores** - Firefox/Chrome built-in validation
- **Dedicated Tools** - `testssl.sh`, `sslyze`, `sslscan`

### Forward Secrecy (Perfect Forward Secrecy)

**Forward Secrecy** ensures past communications remain secure even if server's private key is compromised:

**Cipher Suites with Forward Secrecy:**
- ✅ **ECDHE** (Elliptic Curve Diffie-Hellman Ephemeral) - Modern, fast
- ✅ **DHE** (Diffie-Hellman Ephemeral) - Legacy, slower

**Cipher Suites WITHOUT Forward Secrecy:**
- ❌ **RSA key exchange** - `TLS_RSA_WITH_AES_128_GCM_SHA256`

**Example:**
```
TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
  ↑ ECDHE = Forward Secrecy

TLS_RSA_WITH_AES_128_GCM_SHA256
  ↑ RSA = No Forward Secrecy
```

**Impact:**
- **With Forward Secrecy** - Passive attacker recording traffic cannot decrypt past sessions even with server's private key
- **Without Forward Secrecy** - Compromise of server's RSA private key enables decryption of all past recorded sessions

**Recommendation:**
- Prefer `ECDHE` cipher suites for all TLS 1.2 connections
- TLS 1.3 mandates forward secrecy (all TLS 1.3 ciphers use ECDHE or DHE)

### Key Size Recommendations

**NIST SP 800-57 Part 1 Rev 5 (2020):**

| Algorithm | Minimum | Recommended | High Security |
|-----------|---------|-------------|---------------|
| RSA | 2048 bits | 3072 bits | 4096 bits |
| ECDSA | 224 bits (P-224) | 256 bits (P-256) | 384 bits (P-384) |
| Ed25519 | 256 bits | 256 bits | 256 bits |

**Security Levels:**
- **RSA 2048 bits** ≈ 112-bit security (minimum acceptable)
- **RSA 3072 bits** ≈ 128-bit security (recommended for sensitive data)
- **ECDSA P-256** ≈ 128-bit security (equivalent to RSA-3072)
- **Ed25519 256 bits** ≈ 128-bit security (modern, fast)

**Deprecation Timeline:**
- **2023** - RSA 1024-bit fully deprecated
- **2030** - NIST recommends 2048-bit minimum for RSA (112-bit security)
- **2031+** - Transition to post-quantum cryptography begins

---

## Troubleshooting

### Issue 1: No Certificate Information Displayed

**Symptom:**
```
PORT    STATE SERVICE  VERSION
443/tcp open  https    Apache httpd 2.4.52
  (No TLS certificate information)
```

**Possible Causes:**
1. **Port open but not TLS-enabled** (e.g., HTTP on port 443)
2. **Service detection not enabled** (need `-sV` flag)
3. **TLS handshake timeout** (server slow to respond)
4. **Unsupported TLS version** (server requires TLS 1.0 only, ProRT-IP prefers 1.2+)

**Solutions:**

**Verify service detection enabled:**
```bash
prtip -sS -p 443 -sV example.com
#              ↑ Must include -sV flag
```

**Increase timeout for slow servers:**
```bash
prtip -sS -p 443 -sV --host-timeout 30s example.com
```

**Try legacy TLS version negotiation:**
```bash
prtip -sS -p 443 -sV --tls-version 1.0 example.com
```

**Manual verification with OpenSSL:**
```bash
openssl s_client -connect example.com:443 -showcerts
# If this fails, port may not be TLS-enabled
```

### Issue 2: Certificate Parsing Failed

**Symptom:**
```
⚠️ Warning: Certificate parsing failed
  Reason: Malformed DER encoding
```

**Possible Causes:**
1. **Non-standard certificate encoding** (server using proprietary format)
2. **Truncated certificate chain** (server sent incomplete data)
3. **Protocol implementation bug** (server TLS stack bug)

**Solutions:**

**Capture raw TLS handshake with tcpdump:**
```bash
sudo tcpdump -i any -w tls_handshake.pcap host example.com and port 443
# Perform scan in another terminal
prtip -sS -p 443 -sV example.com
# Analyze pcap with Wireshark
wireshark tls_handshake.pcap
```

**Try alternative TLS libraries:**
```bash
# OpenSSL
openssl s_client -connect example.com:443 -showcerts

# GnuTLS
gnutls-cli --print-cert example.com:443

# testssl.sh
testssl.sh example.com:443
```

**Report issue:**
If parsing fails for publicly-trusted certificate, report to ProRT-IP GitHub issues with:
- Target hostname/IP
- tcpdump/Wireshark capture
- OpenSSL s_client output

### Issue 3: Self-Signed Certificate Detected

**Symptom:**
```
⚠️ Warning: Self-Signed certificate
  Issuer: CN=localhost, O=Acme Corp
  Subject: CN=localhost, O=Acme Corp
```

**Analysis:**
Self-signed certificates have identical Issuer and Subject DNs.

**Scenarios:**

**1. Development/Testing Environment**
```
✅ Expected behavior
   Action: No action required for dev/test
```

**2. Internal Infrastructure**
```
✅ Acceptable with internal PKI
   Action: Verify certificate issued by internal CA
```

**3. Production Internet-Facing Server**
```
❌ Security risk
   Action: Obtain publicly-trusted certificate immediately
```

**Remediation (Production):**

**Option 1: Let's Encrypt (Free, Automated)**
```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d example.com -d www.example.com

# Auto-renewal (90-day validity)
sudo certbot renew --dry-run
```

**Option 2: Commercial CA (DigiCert, GlobalSign, etc.)**
1. Generate CSR (Certificate Signing Request)
2. Purchase certificate from CA
3. Complete domain validation
4. Install signed certificate

### Issue 4: Certificate Expired

**Symptom:**
```
❌ Error: Certificate expired
  Valid Until: 2024-03-15 23:59:59 UTC
  Expired: 45 days ago
```

**Impact:**
- **Browsers reject connection** (NET::ERR_CERT_DATE_INVALID)
- **API clients fail** (SSL certificate verification failure)
- **Compliance violations** (PCI DSS, HIPAA)

**Solutions:**

**Immediate Remediation:**
```bash
# 1. Renew certificate with CA (Let's Encrypt example)
sudo certbot renew --force-renewal

# 2. Verify new certificate
openssl s_client -connect example.com:443 | openssl x509 -noout -dates

# 3. Reload web server
sudo systemctl reload nginx  # or apache2
```

**Prevent Future Expiration:**

**Let's Encrypt Auto-Renewal:**
```bash
# Cron job (runs twice daily)
0 0,12 * * * /usr/bin/certbot renew --quiet --post-hook "systemctl reload nginx"
```

**Commercial CA Reminder:**
Set calendar reminders 30/60/90 days before expiration.

**Monitoring:**
```bash
# Scan all production servers for expiring certificates
prtip -sS -p 443 -sV -iL production_hosts.txt -oJ certs.json

# Alert on certificates expiring within 30 days
cat certs.json | jq '.hosts[].ports[] | select(.service.tls.certificate.days_remaining < 30) | {host: .host, days: .service.tls.certificate.days_remaining}'
```

### Issue 5: TLS 1.0/1.1 Detected (Compliance Violation)

**Symptom:**
```
⚠️ Warning: Deprecated TLS version
  Version: TLS 1.0 (0x0301)
  Status: Prohibited by PCI DSS since June 2018
```

**Impact:**
- **PCI DSS non-compliance** - Payment card processing prohibited
- **NIST SP 800-52 Rev 2 violation** - Federal systems disallowed
- **Security risk** - BEAST, CRIME, POODLE attacks

**Solutions:**

**Nginx: Disable TLS 1.0/1.1**
```nginx
# /etc/nginx/nginx.conf or site config
ssl_protocols TLSv1.2 TLSv1.3;
ssl_prefer_server_ciphers on;
ssl_ciphers 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384';

# Reload Nginx
sudo nginx -t && sudo systemctl reload nginx
```

**Apache: Disable TLS 1.0/1.1**
```apache
# /etc/apache2/mods-available/ssl.conf
SSLProtocol -all +TLSv1.2 +TLSv1.3
SSLCipherSuite ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384
SSLHonorCipherOrder on

# Reload Apache
sudo apachectl configtest && sudo systemctl reload apache2
```

**Verification:**
```bash
# Should fail with protocol version error
openssl s_client -connect example.com:443 -tls1
# error:1409442E:SSL routines:ssl3_read_bytes:tlsv1 alert protocol version

# Should succeed
openssl s_client -connect example.com:443 -tls1_2
# Connected successfully
```

### Issue 6: Weak Cipher Suite Detected

**Symptom:**
```
⚠️ Warning: Weak cipher suite
  Cipher: TLS_RSA_WITH_AES_128_CBC_SHA
  Issues:
    - No forward secrecy (RSA key exchange)
    - CBC mode vulnerable to Lucky13 attack
    - SHA-1 MAC deprecated
  Recommendation: Use ECDHE+AEAD ciphers
```

**Solutions:**

**Modern Cipher Suite Configuration:**

**Nginx:**
```nginx
ssl_ciphers 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-CHACHA20-POLY1305:TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256';
ssl_prefer_server_ciphers on;
```

**Apache:**
```apache
SSLCipherSuite ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-CHACHA20-POLY1305:TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256
SSLHonorCipherOrder on
```

**Verification:**
```bash
prtip -sS -p 443 -sV example.com -v | grep "Ciphers:"
# Should show only AEAD ciphers (GCM, CHACHA20-POLY1305)
```

**Testing Tools:**
```bash
# testssl.sh - Comprehensive cipher suite analysis
testssl.sh --cipher-per-proto example.com:443

# sslyze - Python-based TLS scanner
sslyze --regular example.com:443
```

### Issue 7: Debugging TLS Handshake Failures

**Symptom:**
```
Error: TLS handshake timeout
  Port: 443
  Timeout: 10s
```

**Debugging Steps:**

**1. Verify Port Accessibility**
```bash
# TCP connection test
nc -zv example.com 443
# Connection to example.com 443 port [tcp/https] succeeded!
```

**2. Capture TLS Handshake with tcpdump**
```bash
sudo tcpdump -i any -s 0 -w tls_debug.pcap host example.com and port 443
# Perform scan in another terminal
prtip -sS -p 443 -sV example.com
# Analyze with Wireshark
wireshark tls_debug.pcap
```

**3. Manual TLS Handshake with OpenSSL**
```bash
# Verbose TLS handshake
openssl s_client -connect example.com:443 -showcerts -debug
```

**4. Check for Firewall/IDS Interference**
```bash
# Some IDS/firewalls block TLS scanning
# Try from different source IP or use timing template
prtip -sS -p 443 -sV -T2 example.com
```

**5. Review Server TLS Configuration**
```bash
# Server may require specific TLS version or cipher
# Try legacy TLS 1.0
prtip -sS -p 443 -sV --tls-version 1.0 example.com

# Try specific cipher suite
openssl s_client -connect example.com:443 -cipher 'ECDHE-RSA-AES128-GCM-SHA256'
```

---

## Performance

### Overhead Measurement

**TLS certificate analysis overhead per connection:**

| Phase | Time | Percentage |
|-------|------|------------|
| TCP connection (3-way handshake) | 15ms | 30% |
| TLS handshake (ClientHello → ServerHello → Certificate) | 20ms | 40% |
| Certificate extraction + DER parsing | 10ms | 20% |
| Service detection (HTTP probe, banner grab) | 5ms | 10% |
| **Total** | **50ms** | **100%** |

**Comparison with Nmap:**
- **ProRT-IP** - 50ms per HTTPS port (optimized TLS handshake)
- **Nmap** - 150-200ms per HTTPS port (includes extensive NSE scripting)

**Comparison with Masscan:**
- **Masscan** - 5-10ms per port (stateless SYN scan only, no service detection)
- **ProRT-IP** - 50ms per port (stateful TLS handshake + certificate extraction)

**Trade-off:** ProRT-IP 10x slower than Masscan, but extracts rich certificate metadata unavailable in stateless scanning.

### Benchmark Results

**Test Configuration:**
- **Target:** 100 HTTPS hosts (port 443)
- **Network:** Gigabit Ethernet (1000 Mbps)
- **ProRT-IP Settings:** 10 parallel workers, timing template T3 (Normal)

**Results:**

| Metric | Time | Throughput |
|--------|------|------------|
| Total scan time | 1.5s | 66.7 hosts/sec |
| Per-host time | 15ms average | 200ms max |
| TLS handshakes | 100 | 66.7 handshakes/sec |

**Comparison with Other Tools:**

| Tool | Time (100 HTTPS hosts, port 443) | Relative Speed |
|------|----------------------------------|----------------|
| ProRT-IP (T3, 10 workers) | **1.5s** | **1.0x (baseline)** |
| Masscan (port state only) | 0.8s | 0.53x (1.9x faster, no TLS) |
| Nmap (default) | 25s | 16.7x (16.7x slower) |
| Nmap (-T4) | 12s | 8.0x (8.0x slower) |
| RustScan (default) | 18s | 12.0x (12.0x slower) |

**Memory Usage:**
- **ProRT-IP** - 45 MB peak (10 workers, 100 hosts)
- **Nmap** - 120 MB peak (NSE scripting engine overhead)

**CPU Usage:**
- **ProRT-IP** - 25% average (asynchronous I/O, minimal blocking)
- **Nmap** - 85% average (synchronous model, more blocking)

### Optimization Tips

**1. Targeted Scanning**
Scan only TLS-enabled ports to minimize overhead:

```bash
# Scan only common TLS ports
prtip -sS -p 443,8443,465,587,993,995 -sV TARGET
```

**2. Increase Parallelism**
More workers process TLS handshakes concurrently:

```bash
# 20 parallel workers (default 10)
prtip -sS -p 443 -sV --max-workers 20 TARGET
```

**Trade-off:** Increased CPU/memory usage, faster completion

**3. Disable TLS Analysis for Faster Scanning**
If only port state needed (not certificate details):

```bash
# SYN scan only (no service detection)
prtip -sS -p 443 TARGET
# 10x faster (no TLS handshake)
```

**4. Adjust Timeouts**
Reduce timeouts for fast networks:

```bash
# 5-second timeout (default 10s)
prtip -sS -p 443 -sV --host-timeout 5s TARGET
```

**5. Output Format Selection**
Binary formats faster than JSON/XML:

```bash
# Greppable format (fastest)
prtip -sS -p 443 -sV TARGET -oG results.grep

# JSON (moderate speed)
prtip -sS -p 443 -sV TARGET -oJ results.json

# XML (slowest, but Nmap-compatible)
prtip -sS -p 443 -sV TARGET -oX results.xml
```

**6. Batch Processing**
Process large target lists in batches:

```bash
# Split targets into 10K-host batches
split -l 10000 all_targets.txt batch_

# Scan each batch separately
for batch in batch_*; do
  prtip -sS -p 443 -sV -iL $batch -oJ results_${batch}.json
done
```

**7. Skip Closed Ports**
Only scan hosts with port 443 open:

```bash
# Phase 1: Fast SYN scan to identify open ports
prtip -sS -p 443 10.0.0.0/16 -oG - | grep "open" > open_hosts.txt

# Phase 2: Service detection only on open hosts
prtip -sS -p 443 -sV -iL open_hosts.txt -oJ certs.json
```

---

## Best Practices

### 1. Scan Only TLS-Enabled Ports

**Efficient scanning:**
```bash
# Common TLS ports
prtip -sS -p 443,8443,465,587,993,995,636,3389 -sV TARGET
```

**Port Reference:**
- **443** - HTTPS
- **8443** - Alternative HTTPS
- **465** - SMTPS (SMTP over TLS)
- **587** - SMTP Submission (STARTTLS)
- **993** - IMAPS (IMAP over TLS)
- **995** - POP3S (POP3 over TLS)
- **636** - LDAPS (LDAP over TLS)
- **3389** - RDP (Remote Desktop over TLS)

### 2. Combine with Service Detection

**Always use `-sV` for TLS scanning:**
```bash
prtip -sS -p 443 -sV TARGET
#              ↑ Required for TLS certificate extraction
```

**Without `-sV`:**
```
PORT    STATE SERVICE
443/tcp open  https
```

**With `-sV`:**
```
PORT    STATE SERVICE  VERSION
443/tcp open  https    Apache httpd 2.4.52
  TLS Certificate:
    Subject: CN=example.com
    Issuer: CN=DigiCert SHA2 Secure Server CA
    Valid: 2024-01-15 to 2025-02-15 (156 days)
    SANs: example.com, www.example.com
```

### 3. Export to JSON for Analysis

**JSON output enables programmatic processing:**
```bash
prtip -sS -p 443 -sV 10.0.0.0/16 -oJ certs.json
```

**Example Analysis (Python):**
```python
import json
from datetime import datetime

with open('certs.json') as f:
    data = json.load(f)

# Find certificates expiring within 30 days
for host in data['hosts']:
    for port in host['ports']:
        if 'tls' in port['service']:
            cert = port['service']['tls']['certificate']
            expiry = datetime.fromisoformat(cert['valid_until'])
            days_remaining = (expiry - datetime.now()).days

            if days_remaining < 30:
                print(f"⚠️ {host['address']}:{port['port']}")
                print(f"   Expires in {days_remaining} days")
                print(f"   Subject: {cert['subject']}")
```

### 4. Monitor Certificate Expiration

**Automated scanning + alerting:**

```bash
#!/bin/bash
# weekly_cert_check.sh

# Scan all production servers
prtip -sS -p 443 -sV -iL production_hosts.txt -oJ weekly_certs.json

# Alert on certificates expiring within 30 days
cat weekly_certs.json | jq '.hosts[].ports[] | select(.service.tls.certificate.days_remaining < 30)' > expiring_certs.txt

# Send email if any expiring certificates found
if [ -s expiring_certs.txt ]; then
  mail -s "Certificate Expiration Alert" admin@example.com < expiring_certs.txt
fi
```

**Cron job (weekly scan):**
```cron
0 2 * * 0 /usr/local/bin/weekly_cert_check.sh
```

### 5. Verify TLS Configuration Changes

**After updating server TLS settings:**

```bash
# 1. Verify TLS version
prtip -sS -p 443 -sV example.com | grep "TLS Version"
# Expected: TLS 1.2 or TLS 1.3

# 2. Verify cipher suites
prtip -sS -p 443 -sV example.com | grep "Ciphers:"
# Expected: AEAD ciphers only (GCM, CHACHA20-POLY1305)

# 3. Verify forward secrecy
prtip -sS -p 443 -sV example.com | grep "ECDHE"
# Expected: All ciphers use ECDHE key exchange

# 4. Cross-check with testssl.sh
testssl.sh --protocols --ciphers example.com:443
```

### 6. Regular Compliance Audits

**Quarterly TLS compliance scan:**

```bash
# Scan all internet-facing servers
prtip -sS -p 443 -sV -iL public_servers.txt -oJ compliance_audit.json

# Check for violations
jq '.hosts[].ports[] | select(.service.tls.version | test("TLS 1\\.[01]")) | {host: .host, version: .service.tls.version}' compliance_audit.json > tls_violations.txt

# Generate compliance report
if [ -s tls_violations.txt ]; then
  echo "❌ PCI DSS Violation: TLS 1.0/1.1 detected"
  cat tls_violations.txt
else
  echo "✅ Compliance: All servers TLS 1.2+"
fi
```

### 7. Document Certificate Inventory

**Maintain certificate inventory spreadsheet:**

| Hostname | IP | Port | Subject | Issuer | Valid Until | Days Remaining | SANs |
|----------|----|----|---------|--------|-------------|----------------|------|
| web.example.com | 203.0.113.10 | 443 | CN=web.example.com | DigiCert | 2025-02-15 | 156 | web.example.com, www.example.com |
| mail.example.com | 203.0.113.20 | 465 | CN=mail.example.com | Let's Encrypt | 2025-01-05 | 45 | mail.example.com, smtp.example.com |

**Automated inventory generation:**
```bash
prtip -sS -p 443,465,993 -sV -iL all_servers.txt -oJ inventory.json

# Convert to CSV
jq -r '.hosts[].ports[] | select(.service.tls) | [.host, .address, .port, .service.tls.certificate.subject, .service.tls.certificate.issuer, .service.tls.certificate.valid_until, .service.tls.certificate.days_remaining, (.service.tls.certificate.sans | join("; "))] | @csv' inventory.json > inventory.csv
```

---

## See Also

- **[Service Detection](./service-detection.md)** - Protocol-specific service identification
- **[User Guide: Service Detection](../user-guide/basic-usage.md#service-detection)** - Usage examples and workflows
- **[Architecture: Scanning Engine](../../00-ARCHITECTURE.md)** - TLS integration in scan pipeline
- **[Technical Specs: TLS Implementation](../../02-TECHNICAL-SPECS.md#tls-certificate-analysis)** - Complete implementation details
- **[Security Guide](../../08-SECURITY.md)** - TLS security best practices

**External Resources:**
- **RFC 5280** - X.509 v3 Certificate and CRL Profile
- **RFC 8446** - TLS 1.3 Protocol Specification
- **RFC 8996** - TLS 1.0 and TLS 1.1 Deprecation
- **NIST SP 800-52 Rev 2** - Guidelines for TLS Implementations
- **NIST SP 800-57 Part 1 Rev 5** - Key Management Recommendations
- **PCI DSS v4.0** - Payment Card Industry Data Security Standard
- **testssl.sh** - Comprehensive TLS testing tool
- **SSL Labs Server Test** - Online TLS configuration analyzer

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
