//! Cross-platform packet capture abstraction

use prtip_core::Result;

/// Cross-platform packet capture trait
pub trait PacketCapture: Send {
    /// Open the capture device with the specified interface
    ///
    /// # Arguments
    ///
    /// * `interface` - Optional interface name (None = auto-detect first non-loopback)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if opening fails
    fn open(&mut self, interface: Option<&str>) -> Result<()>;

    /// Send a raw packet
    ///
    /// # Arguments
    ///
    /// * `packet` - Raw packet bytes to send
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if sending fails
    fn send_packet(&mut self, packet: &[u8]) -> Result<()>;

    /// Receive a packet (blocking with timeout)
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Returns
    ///
    /// Returns Some(packet) if received, None on timeout, or an error
    fn receive_packet(&mut self, timeout_ms: u64) -> Result<Option<Vec<u8>>>;

    /// Close the capture device
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if closing fails
    fn close(&mut self) -> Result<()>;
}

/// Create a platform-specific packet capture instance
///
/// # Returns
///
/// Returns a boxed PacketCapture implementation for the current platform
///
/// # Examples
///
/// ```no_run
/// use prtip_network::capture::create_capture;
///
/// let mut capture = create_capture().unwrap();
/// capture.open(Some("eth0")).unwrap();
/// ```
pub fn create_capture() -> Result<Box<dyn PacketCapture>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxCapture::new()))
    }

    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsCapture::new()))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOSCapture::new()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(prtip_core::Error::Network(
            "Unsupported platform for packet capture".to_string(),
        ))
    }
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxCapture;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsCapture;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSCapture;
