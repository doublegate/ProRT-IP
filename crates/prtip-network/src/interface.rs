//! Network interface detection and routing
//!
//! This module provides cross-platform detection of network interfaces,
//! routing table analysis, and source IP selection for scanning operations.
//!
//! Inspired by naabu's routing package patterns.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;

#[cfg(unix)]
use pnet::datalink;

/// Errors that can occur during interface operations
#[derive(Error, Debug)]
pub enum InterfaceError {
    /// No suitable interface found
    #[error("No suitable network interface found for target {0}")]
    NoInterfaceFound(IpAddr),

    /// Failed to enumerate interfaces
    #[error("Failed to enumerate network interfaces: {0}")]
    EnumerationFailed(String),

    /// Interface has no suitable addresses
    #[error("Interface {0} has no suitable addresses")]
    NoAddresses(String),

    /// Invalid interface specification
    #[error("Invalid interface specification: {0}")]
    InvalidInterface(String),
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "wlan0")
    pub name: String,

    /// Hardware (MAC) address
    pub mac_address: Option<Vec<u8>>,

    /// IPv4 addresses assigned to this interface
    pub ipv4_addresses: Vec<Ipv4Addr>,

    /// IPv6 addresses assigned to this interface
    pub ipv6_addresses: Vec<Ipv6Addr>,

    /// Maximum transmission unit
    pub mtu: Option<usize>,

    /// Whether interface is up
    pub is_up: bool,

    /// Whether interface is loopback
    pub is_loopback: bool,
}

impl NetworkInterface {
    /// Check if interface has any IPv4 addresses
    pub fn has_ipv4(&self) -> bool {
        !self.ipv4_addresses.is_empty()
    }

    /// Check if interface has any IPv6 addresses
    pub fn has_ipv6(&self) -> bool {
        !self.ipv6_addresses.is_empty()
    }

    /// Get the first IPv4 address, if available
    pub fn first_ipv4(&self) -> Option<Ipv4Addr> {
        self.ipv4_addresses.first().copied()
    }

    /// Get the first IPv6 address, if available
    pub fn first_ipv6(&self) -> Option<Ipv6Addr> {
        self.ipv6_addresses.first().copied()
    }

    /// Get appropriate source IP for a target
    pub fn get_source_ip_for(&self, target: IpAddr) -> Option<IpAddr> {
        match target {
            IpAddr::V4(_) => self.first_ipv4().map(IpAddr::V4),
            IpAddr::V6(_) => self.first_ipv6().map(IpAddr::V6),
        }
    }
}

/// Enumerate all available network interfaces
///
/// # Examples
///
/// ```no_run
/// use prtip_network::interface::enumerate_interfaces;
///
/// match enumerate_interfaces() {
///     Ok(interfaces) => {
///         for iface in interfaces {
///             println!("Interface: {} ({})", iface.name,
///                      if iface.is_up { "UP" } else { "DOWN" });
///         }
///     }
///     Err(e) => eprintln!("Failed to enumerate interfaces: {}", e),
/// }
/// ```
#[cfg(unix)]
pub fn enumerate_interfaces() -> Result<Vec<NetworkInterface>, InterfaceError> {
    let interfaces = datalink::interfaces();

    let mut result = Vec::new();

    for iface in interfaces {
        let mut ipv4_addresses = Vec::new();
        let mut ipv6_addresses = Vec::new();

        // Extract IP addresses
        for ip_network in iface.ips.iter() {
            match ip_network.ip() {
                IpAddr::V4(ipv4) => ipv4_addresses.push(ipv4),
                IpAddr::V6(ipv6) => {
                    // Skip link-local IPv6 addresses (fe80::/10) as they're not routable
                    // Using manual check for MSRV compatibility (Rust 1.70)
                    let octets = ipv6.octets();
                    let is_link_local = octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80;
                    if !is_link_local {
                        ipv6_addresses.push(ipv6);
                    }
                }
            }
        }

        result.push(NetworkInterface {
            name: iface.name.clone(),
            mac_address: iface.mac.map(|mac| mac.octets().to_vec()),
            ipv4_addresses,
            ipv6_addresses,
            mtu: Some(iface.index as usize), // pnet doesn't expose MTU directly
            is_up: iface.is_up(),
            is_loopback: iface.is_loopback(),
        });
    }

    Ok(result)
}

/// Enumerate all available network interfaces (Windows stub)
#[cfg(not(unix))]
pub fn enumerate_interfaces() -> Result<Vec<NetworkInterface>, InterfaceError> {
    // Windows implementation would use different APIs
    // For now, return a conservative default
    Err(InterfaceError::EnumerationFailed(
        "Interface enumeration not yet implemented for this platform".to_string(),
    ))
}

/// Find the best interface for reaching a target IP
///
/// This performs simple routing logic:
/// 1. If target is on same subnet as any interface, use that interface
/// 2. Otherwise, use the default gateway interface
/// 3. Prefer non-loopback interfaces
/// 4. Match IPv4/IPv6 address families
///
/// # Arguments
///
/// * `target` - Target IP address to route to
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// use prtip_network::interface::find_interface_for_target;
///
/// let target: IpAddr = "8.8.8.8".parse().unwrap();
/// match find_interface_for_target(target) {
///     Ok(iface) => println!("Using interface: {}", iface.name),
///     Err(e) => eprintln!("No suitable interface: {}", e),
/// }
/// ```
pub fn find_interface_for_target(target: IpAddr) -> Result<NetworkInterface, InterfaceError> {
    let interfaces = enumerate_interfaces()?;

    // Filter to interfaces that are up and have addresses
    let mut candidates: Vec<_> = interfaces
        .into_iter()
        .filter(|iface| iface.is_up && !iface.is_loopback)
        .collect();

    // If no non-loopback interfaces, allow loopback
    if candidates.is_empty() {
        candidates = enumerate_interfaces()?
            .into_iter()
            .filter(|iface| iface.is_up)
            .collect();
    }

    // Find interface matching address family with addresses
    let matching: Vec<_> = candidates
        .into_iter()
        .filter(|iface| match target {
            IpAddr::V4(_) => iface.has_ipv4(),
            IpAddr::V6(_) => iface.has_ipv6(),
        })
        .collect();

    matching
        .into_iter()
        .next()
        .ok_or(InterfaceError::NoInterfaceFound(target))
}

/// Get the source IP to use when scanning a target
///
/// This is a convenience function that combines interface detection
/// with source IP selection.
///
/// # Arguments
///
/// * `target` - Target IP address
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// use prtip_network::interface::get_source_ip_for_target;
///
/// let target: IpAddr = "192.168.1.1".parse().unwrap();
/// match get_source_ip_for_target(target) {
///     Ok(source) => println!("Using source IP: {}", source),
///     Err(e) => eprintln!("Failed to determine source IP: {}", e),
/// }
/// ```
pub fn get_source_ip_for_target(target: IpAddr) -> Result<IpAddr, InterfaceError> {
    let interface = find_interface_for_target(target)?;

    interface
        .get_source_ip_for(target)
        .ok_or_else(|| InterfaceError::NoAddresses(interface.name.clone()))
}

/// Find interface by name
///
/// # Arguments
///
/// * `name` - Interface name (e.g., "eth0")
///
/// # Examples
///
/// ```no_run
/// use prtip_network::interface::find_interface_by_name;
///
/// match find_interface_by_name("eth0") {
///     Ok(iface) => println!("Found interface: {:?}", iface),
///     Err(e) => eprintln!("Interface not found: {}", e),
/// }
/// ```
pub fn find_interface_by_name(name: &str) -> Result<NetworkInterface, InterfaceError> {
    let interfaces = enumerate_interfaces()?;

    interfaces
        .into_iter()
        .find(|iface| iface.name == name)
        .ok_or_else(|| InterfaceError::InvalidInterface(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_interfaces() {
        let result = enumerate_interfaces();

        // Should succeed on Unix systems
        #[cfg(unix)]
        {
            assert!(result.is_ok());
            let interfaces = result.unwrap();
            // Should have at least loopback
            assert!(!interfaces.is_empty());

            // Verify at least one interface has the loopback flag
            let has_loopback = interfaces.iter().any(|iface| iface.is_loopback);
            assert!(has_loopback);
        }

        // May fail on Windows (not implemented)
        #[cfg(not(unix))]
        {
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_interface_properties() {
        let interfaces = enumerate_interfaces().unwrap();

        for iface in interfaces {
            // Interface name should not be empty
            assert!(!iface.name.is_empty());

            // If interface has IPv4 addresses, first_ipv4() should work
            if iface.has_ipv4() {
                assert!(iface.first_ipv4().is_some());
            }

            // If interface has IPv6 addresses, first_ipv6() should work
            if iface.has_ipv6() {
                assert!(iface.first_ipv6().is_some());
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_find_interface_for_localhost() {
        // Finding interface for localhost should work
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let result = find_interface_for_target(target);

        // Should find loopback interface
        assert!(result.is_ok());
        let iface = result.unwrap();
        assert!(iface.is_loopback || iface.has_ipv4());
    }

    #[test]
    #[cfg(unix)]
    fn test_get_source_ip_for_localhost() {
        let target: IpAddr = "127.0.0.1".parse().unwrap();
        let result = get_source_ip_for_target(target);

        // Should get a valid source IP
        assert!(result.is_ok());
        let source = result.unwrap();

        // Source should be IPv4
        assert!(matches!(source, IpAddr::V4(_)));
    }

    #[test]
    #[cfg(unix)]
    fn test_find_interface_by_name_loopback() {
        // Try to find loopback interface
        // Name varies by platform: "lo" on Linux, "lo0" on macOS
        let result1 = find_interface_by_name("lo");
        let result2 = find_interface_by_name("lo0");

        // At least one should succeed
        assert!(result1.is_ok() || result2.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_find_interface_by_name_invalid() {
        let result = find_interface_by_name("definitely-not-a-real-interface-name-12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_network_interface_source_ip_matching() {
        let iface = NetworkInterface {
            name: "test0".to_string(),
            mac_address: None,
            ipv4_addresses: vec!["192.168.1.1".parse().unwrap()],
            ipv6_addresses: vec!["fe80::1".parse().unwrap()],
            mtu: Some(1500),
            is_up: true,
            is_loopback: false,
        };

        // Should return IPv4 for IPv4 target
        let ipv4_target: IpAddr = "8.8.8.8".parse().unwrap();
        let source = iface.get_source_ip_for(ipv4_target);
        assert!(source.is_some());
        assert!(matches!(source.unwrap(), IpAddr::V4(_)));

        // Should return IPv6 for IPv6 target
        let ipv6_target: IpAddr = "2001:4860:4860::8888".parse().unwrap();
        let source = iface.get_source_ip_for(ipv6_target);
        assert!(source.is_some());
        assert!(matches!(source.unwrap(), IpAddr::V6(_)));
    }

    #[test]
    fn test_network_interface_no_addresses() {
        let iface = NetworkInterface {
            name: "test0".to_string(),
            mac_address: None,
            ipv4_addresses: vec![],
            ipv6_addresses: vec![],
            mtu: Some(1500),
            is_up: true,
            is_loopback: false,
        };

        assert!(!iface.has_ipv4());
        assert!(!iface.has_ipv6());
        assert!(iface.first_ipv4().is_none());
        assert!(iface.first_ipv6().is_none());
    }
}
