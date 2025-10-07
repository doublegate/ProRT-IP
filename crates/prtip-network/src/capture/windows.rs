//! Windows packet capture implementation using Npcap

use super::PacketCapture;
use pnet_datalink::{self, Channel, Config, DataLinkReceiver, DataLinkSender, NetworkInterface};
use prtip_core::{Error, Result};
use std::time::Duration;

/// Windows-specific packet capture using Npcap
pub struct WindowsCapture {
    interface: Option<NetworkInterface>,
    tx: Option<Box<dyn DataLinkSender>>,
    rx: Option<Box<dyn DataLinkReceiver>>,
}

impl WindowsCapture {
    /// Create a new Windows packet capture instance
    pub fn new() -> Self {
        Self {
            interface: None,
            tx: None,
            rx: None,
        }
    }

    /// Get list of available network interfaces
    fn get_interfaces() -> Vec<NetworkInterface> {
        pnet_datalink::interfaces()
    }

    /// Find interface by name or select first non-loopback
    fn find_interface(name: Option<&str>) -> Result<NetworkInterface> {
        let interfaces = Self::get_interfaces();

        if let Some(iface_name) = name {
            interfaces
                .into_iter()
                .find(|i| i.name.contains(iface_name) || i.description.contains(iface_name))
                .ok_or_else(|| Error::Network(format!("Interface not found: {}", iface_name)))
        } else {
            // Select first non-loopback interface
            interfaces
                .into_iter()
                .find(|i| !i.is_loopback() && i.is_up() && !i.ips.is_empty())
                .ok_or_else(|| Error::Network("No suitable interface found".to_string()))
        }
    }
}

impl Default for WindowsCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketCapture for WindowsCapture {
    fn open(&mut self, interface: Option<&str>) -> Result<()> {
        // Find the interface
        let iface = Self::find_interface(interface)?;

        tracing::debug!(
            "Opening packet capture on interface: {} ({})",
            iface.name,
            iface.description
        );

        // Check for Npcap availability
        tracing::info!(
            "Attempting to open Npcap on Windows. Ensure Npcap is installed and you have Administrator privileges."
        );

        // Configure channel with longer timeout for Windows
        let config = Config {
            read_timeout: Some(Duration::from_millis(100)),
            write_buffer_size: 4096,
            read_buffer_size: 4096,
            ..Default::default()
        };

        // Create Ethernet channel
        let (tx, rx) = match pnet_datalink::channel(&iface, config) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => {
                return Err(Error::Network(
                    "Unsupported channel type (expected Ethernet)".to_string(),
                ))
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to create channel on {}: {}. \
                     Ensure Npcap is installed and you have Administrator privileges.",
                    iface.name, e
                );
                return Err(Error::Network(err_msg));
            }
        };

        self.interface = Some(iface);
        self.tx = Some(tx);
        self.rx = Some(rx);

        tracing::info!("Packet capture opened successfully on Windows");
        Ok(())
    }

    fn send_packet(&mut self, packet: &[u8]) -> Result<()> {
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| Error::Network("Capture not open (call open() first)".to_string()))?;

        if packet.is_empty() {
            return Err(Error::Network("Cannot send empty packet".to_string()));
        }

        if packet.len() > 65535 {
            return Err(Error::Network(format!(
                "Packet too large: {} bytes (max 65535)",
                packet.len()
            )));
        }

        let _ = tx.send_to(packet, None).ok_or_else(|| {
            Error::Network("Failed to send packet".to_string())
        })?;
        Ok(())
    }

    fn receive_packet(&mut self, timeout_ms: u64) -> Result<Option<Vec<u8>>> {
        let rx = self
            .rx
            .as_mut()
            .ok_or_else(|| Error::Network("Capture not open (call open() first)".to_string()))?;

        // Calculate timeout iterations
        let iterations = (timeout_ms + 99) / 100; // Round up

        for _ in 0..iterations {
            match rx.next() {
                Ok(packet) => {
                    return Ok(Some(packet.to_vec()));
                }
                Err(_e) => {
                    // pnet errors don't expose inner types, just continue on error
                    // Most errors will be timeouts
                    continue;
                }
            }
        }

        Ok(None) // Timeout
    }

    fn close(&mut self) -> Result<()> {
        // Drop the handles to close connections
        self.tx = None;
        self.rx = None;
        self.interface = None;

        tracing::debug!("Packet capture closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_capture_creation() {
        let capture = WindowsCapture::new();
        assert!(capture.interface.is_none());
        assert!(capture.tx.is_none());
        assert!(capture.rx.is_none());
    }

    #[test]
    fn test_get_interfaces() {
        let interfaces = WindowsCapture::get_interfaces();
        // Should have at least some interfaces
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn test_send_packet_not_open() {
        let mut capture = WindowsCapture::new();
        let result = capture.send_packet(&[0u8; 64]);
        assert!(result.is_err());
    }

    #[test]
    fn test_send_packet_empty() {
        let mut capture = WindowsCapture::new();
        let result = capture.send_packet(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_close_unopened() {
        let mut capture = WindowsCapture::new();
        let result = capture.close();
        assert!(result.is_ok());
    }
}
