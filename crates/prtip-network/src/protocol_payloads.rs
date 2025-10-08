//! Protocol-specific payloads for UDP scanning
//!
//! This module provides well-formed payloads for various UDP protocols
//! to improve detection rates during UDP scans.

/// Get protocol-specific UDP payload for a given port
pub fn get_udp_payload(port: u16) -> Option<Vec<u8>> {
    match port {
        53 => Some(dns_query()),
        123 => Some(ntp_request()),
        137 => Some(netbios_name_query()),
        161 => Some(snmp_get_request()),
        111 => Some(rpc_null_call()),
        500 => Some(ike_handshake()),
        1900 => Some(ssdp_discover()),
        5353 => Some(mdns_query()),
        _ => None,
    }
}

/// DNS standard query for root domain
fn dns_query() -> Vec<u8> {
    vec![
        0x12, 0x34, // Transaction ID
        0x01, 0x00, // Flags: standard query
        0x00, 0x01, // Questions: 1
        0x00, 0x00, // Answer RRs: 0
        0x00, 0x00, // Authority RRs: 0
        0x00, 0x00, // Additional RRs: 0
        0x00, // Name: root (empty label)
        0x00, 0x01, // Type: A
        0x00, 0x01, // Class: IN
    ]
}

/// NTP version 3 client request
fn ntp_request() -> Vec<u8> {
    let mut payload = vec![0x1B]; // LI=0, VN=3, Mode=3 (client)
    payload.resize(48, 0); // NTP packets are 48 bytes
    payload
}

/// NetBIOS Name Service query for *<00><00>
fn netbios_name_query() -> Vec<u8> {
    vec![
        0xAB, 0xCD, // Transaction ID
        0x01, 0x00, // Flags: query
        0x00, 0x01, // Questions: 1
        0x00, 0x00, // Answer RRs: 0
        0x00, 0x00, // Authority RRs: 0
        0x00, 0x00, // Additional RRs: 0
        // Query for *<00><00>
        0x20, // Length: 32 (encoded)
        0x43, 0x4B, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x00, // Null terminator
        0x00, 0x21, // Type: NB (NetBIOS general name service)
        0x00, 0x01, // Class: IN
    ]
}

/// SNMP GetRequest for sysDescr.0 with community "public"
fn snmp_get_request() -> Vec<u8> {
    vec![
        0x30, 0x26, // SEQUENCE, length 38
        0x02, 0x01, 0x00, // INTEGER version (0 = SNMPv1)
        0x04, 0x06, 0x70, 0x75, 0x62, 0x6C, 0x69, 0x63, // OCTET STRING "public"
        0xA0, 0x19, // GetRequest PDU
        0x02, 0x01, 0x00, // Request ID: 0
        0x02, 0x01, 0x00, // Error status: 0
        0x02, 0x01, 0x00, // Error index: 0
        0x30, 0x0E, // Variable bindings
        0x30, 0x0C, // Variable binding
        0x06, 0x08, 0x2B, 0x06, 0x01, 0x02, 0x01, 0x01, 0x01, 0x00, // OID: 1.3.6.1.2.1.1.1.0
        // (sysDescr.0)
        0x05, 0x00, // NULL value
    ]
}

/// Sun RPC NULL call (portmapper query)
fn rpc_null_call() -> Vec<u8> {
    vec![
        0x00, 0x00, 0x00, 0x01, // XID
        0x00, 0x00, 0x00, 0x00, // Message type: Call
        0x00, 0x00, 0x00, 0x02, // RPC version: 2
        0x00, 0x00, 0x00, 0x64, // Program: portmapper (100)
        0x00, 0x00, 0x00, 0x02, // Program version: 2
        0x00, 0x00, 0x00, 0x00, // Procedure: NULL
        0x00, 0x00, 0x00, 0x00, // Credentials: NULL
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Verifier: NULL
        0x00, 0x00, 0x00, 0x00,
    ]
}

/// IKE (IPSec) Main Mode SA payload
fn ike_handshake() -> Vec<u8> {
    vec![
        // IKE Header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Initiator cookie (would be random)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Responder cookie (0)
        0x01, // Next payload: SA
        0x10, // Version: 1.0
        0x02, // Exchange type: Identity Protection (Main Mode)
        0x00, // Flags
        0x00, 0x00, 0x00, 0x00, // Message ID
        0x00, 0x00, 0x00, 0x78, // Length: 120
        // SA payload (simplified)
        0x00, 0x00, 0x00, 0x5C, // Payload length
        0x00, 0x00, 0x00, 0x01, // DOI: IPsec
        0x00, 0x00, 0x00, 0x01, // Situation: Identity Only
    ]
}

/// SSDP M-SEARCH discovery
fn ssdp_discover() -> Vec<u8> {
    b"M-SEARCH * HTTP/1.1\r\n\
      HOST: 239.255.255.250:1900\r\n\
      MAN: \"ssdp:discover\"\r\n\
      MX: 3\r\n\
      ST: ssdp:all\r\n\
      \r\n"
        .to_vec()
}

/// mDNS (Multicast DNS) query
fn mdns_query() -> Vec<u8> {
    vec![
        0x00, 0x00, // Transaction ID: 0
        0x00, 0x00, // Flags: standard query
        0x00, 0x01, // Questions: 1
        0x00, 0x00, // Answer RRs: 0
        0x00, 0x00, // Authority RRs: 0
        0x00, 0x00, // Additional RRs: 0
        // Query for _services._dns-sd._udp.local
        0x09, 0x5F, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, // "_services"
        0x07, 0x5F, 0x64, 0x6E, 0x73, 0x2D, 0x73, 0x64, // "_dns-sd"
        0x04, 0x5F, 0x75, 0x64, 0x70, // "_udp"
        0x05, 0x6C, 0x6F, 0x63, 0x61, 0x6C, // "local"
        0x00, // Null terminator
        0x00, 0x0C, // Type: PTR
        0x00, 0x01, // Class: IN
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_query_format() {
        let payload = dns_query();
        assert!(payload.len() >= 12); // Minimum DNS header
        assert_eq!(payload[2], 0x01); // Standard query flag
    }

    #[test]
    fn test_ntp_request_size() {
        let payload = ntp_request();
        assert_eq!(payload.len(), 48); // NTP packets are always 48 bytes
        assert_eq!(payload[0], 0x1B); // LI=0, VN=3, Mode=3
    }

    #[test]
    fn test_netbios_query() {
        let payload = netbios_name_query();
        assert!(payload.len() > 12); // Has NetBIOS name encoded
    }

    #[test]
    fn test_snmp_get_request() {
        let payload = snmp_get_request();
        assert_eq!(payload[0], 0x30); // SEQUENCE tag
        assert_eq!(payload[2], 0x02); // INTEGER tag for version
    }

    #[test]
    fn test_get_udp_payload() {
        assert!(get_udp_payload(53).is_some()); // DNS
        assert!(get_udp_payload(123).is_some()); // NTP
        assert!(get_udp_payload(161).is_some()); // SNMP
        assert!(get_udp_payload(9999).is_none()); // Unknown port
    }

    #[test]
    fn test_rpc_null_call() {
        let payload = rpc_null_call();
        assert_eq!(payload.len(), 40); // RPC NULL call size
                                       // Check RPC version
        assert_eq!(&payload[8..12], &[0x00, 0x00, 0x00, 0x02]);
    }

    #[test]
    fn test_ssdp_discover() {
        let payload = ssdp_discover();
        let text = String::from_utf8_lossy(&payload);
        assert!(text.contains("M-SEARCH"));
        assert!(text.contains("ssdp:discover"));
    }
}
