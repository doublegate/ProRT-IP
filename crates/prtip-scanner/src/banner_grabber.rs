//! Banner grabbing with protocol-specific handlers
//!
//! This module provides banner grabbing capabilities for common protocols
//! including HTTP, FTP, SSH, SMTP, POP3, and IMAP.
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::banner_grabber::BannerGrabber;
//! use std::net::SocketAddr;
//!
//! # async fn example() -> Result<(), prtip_core::Error> {
//! let grabber = BannerGrabber::new();
//! let addr = "192.168.1.1:80".parse().unwrap();
//!
//! let banner = grabber.grab_banner(addr).await?;
//! println!("Banner: {}", banner);
//! # Ok(())
//! # }
//! ```

use prtip_core::Error;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Banner grabber with protocol-specific handlers
pub struct BannerGrabber {
    /// Connection timeout
    timeout: Duration,
    /// Maximum banner size to read
    max_banner_size: usize,
}

impl BannerGrabber {
    /// Create new banner grabber with defaults
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_banner_size: 4096,
        }
    }

    /// Grab banner from target (auto-detect protocol by port)
    pub async fn grab_banner(&self, target: SocketAddr) -> Result<String, Error> {
        match target.port() {
            21 => self.grab_ftp_banner(target).await,
            22 => self.grab_ssh_banner(target).await,
            25 | 587 => self.grab_smtp_banner(target).await,
            80 | 8080 => self.grab_http_banner(target).await,
            110 => self.grab_pop3_banner(target).await,
            143 => self.grab_imap_banner(target).await,
            443 | 8443 => self.grab_https_banner(target).await,
            _ => self.grab_generic_banner(target).await,
        }
    }

    /// Grab generic TCP banner (wait for server to send data)
    pub async fn grab_generic_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        self.read_banner(stream).await
    }

    /// Grab HTTP banner
    pub async fn grab_http_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let mut stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // Send HTTP GET request
        let request = format!(
            "GET / HTTP/1.0\r\nHost: {}\r\nUser-Agent: ProRT-IP/1.0\r\n\r\n",
            target.ip()
        );

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| Error::Network(format!("Write failed: {}", e)))?;

        self.read_banner(stream).await
    }

    /// Grab HTTPS banner (would require TLS - placeholder)
    pub async fn grab_https_banner(&self, target: SocketAddr) -> Result<String, Error> {
        // TODO: Implement TLS handshake
        // For now, return placeholder
        Ok("HTTPS (TLS required)".to_string())
    }

    /// Grab FTP banner
    pub async fn grab_ftp_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // FTP sends banner immediately
        self.read_banner(stream).await
    }

    /// Grab SSH banner
    pub async fn grab_ssh_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // SSH sends version string immediately
        let mut reader = BufReader::new(stream);
        let mut banner = String::new();

        match timeout(self.timeout, reader.read_line(&mut banner)).await {
            Ok(Ok(_)) => Ok(banner.trim().to_string()),
            Ok(Err(e)) => Err(Error::Network(format!("Read error: {}", e))),
            Err(_) => Err(Error::Network("Read timeout".to_string())),
        }
    }

    /// Grab SMTP banner
    pub async fn grab_smtp_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let mut stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // SMTP sends 220 greeting
        let banner = self.read_banner_from(&mut stream).await?;

        // Send EHLO to get more info
        let ehlo = format!("EHLO {}\r\n", target.ip());
        stream
            .write_all(ehlo.as_bytes())
            .await
            .map_err(|e| Error::Network(format!("Write failed: {}", e)))?;

        let ehlo_response = self.read_banner_from(&mut stream).await?;

        Ok(format!("{}\n{}", banner, ehlo_response))
    }

    /// Grab POP3 banner
    pub async fn grab_pop3_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // POP3 sends +OK greeting
        self.read_banner(stream).await
    }

    /// Grab IMAP banner
    pub async fn grab_imap_banner(&self, target: SocketAddr) -> Result<String, Error> {
        let stream = timeout(self.timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // IMAP sends * OK greeting
        self.read_banner(stream).await
    }

    /// Read banner from stream (takes ownership)
    async fn read_banner(&self, mut stream: TcpStream) -> Result<String, Error> {
        self.read_banner_from(&mut stream).await
    }

    /// Read banner from stream reference
    async fn read_banner_from(&self, stream: &mut TcpStream) -> Result<String, Error> {
        let mut buffer = vec![0u8; self.max_banner_size];

        match timeout(self.timeout, stream.read(&mut buffer)).await {
            Ok(Ok(n)) if n > 0 => {
                let banner = String::from_utf8_lossy(&buffer[..n]);
                Ok(banner.trim().to_string())
            }
            Ok(Ok(_)) => Err(Error::Network("Empty response".to_string())),
            Ok(Err(e)) => Err(Error::Network(format!("Read error: {}", e))),
            Err(_) => Err(Error::Network("Read timeout".to_string())),
        }
    }

    /// Set connection timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Set maximum banner size
    pub fn set_max_banner_size(&mut self, size: usize) {
        self.max_banner_size = size;
    }
}

impl Default for BannerGrabber {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol-specific banner parser
pub struct BannerParser;

impl BannerParser {
    /// Parse HTTP banner to extract server info
    pub fn parse_http_banner(banner: &str) -> Option<String> {
        for line in banner.lines() {
            if line.starts_with("Server:") {
                return Some(
                    line.strip_prefix("Server:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
            }
        }
        None
    }

    /// Parse FTP banner to extract server info
    pub fn parse_ftp_banner(banner: &str) -> Option<String> {
        if banner.starts_with("220") {
            return Some(banner.strip_prefix("220").unwrap_or("").trim().to_string());
        }
        None
    }

    /// Parse SSH banner to extract version
    pub fn parse_ssh_banner(banner: &str) -> Option<String> {
        if banner.starts_with("SSH-") {
            return Some(banner.to_string());
        }
        None
    }

    /// Parse SMTP banner to extract server info
    pub fn parse_smtp_banner(banner: &str) -> Option<String> {
        for line in banner.lines() {
            if line.starts_with("220") {
                return Some(line.strip_prefix("220").unwrap_or("").trim().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_banner() {
        let banner = "HTTP/1.1 200 OK\r\nServer: nginx/1.18.0\r\nContent-Type: text/html\r\n";
        let server = BannerParser::parse_http_banner(banner);
        assert_eq!(server, Some("nginx/1.18.0".to_string()));
    }

    #[test]
    fn test_parse_ftp_banner() {
        let banner = "220 ProFTPD 1.3.5 Server ready.";
        let server = BannerParser::parse_ftp_banner(banner);
        assert_eq!(server, Some("ProFTPD 1.3.5 Server ready.".to_string()));
    }

    #[test]
    fn test_parse_ssh_banner() {
        let banner = "SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.5";
        let version = BannerParser::parse_ssh_banner(banner);
        assert_eq!(
            version,
            Some("SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.5".to_string())
        );
    }

    #[test]
    fn test_parse_smtp_banner() {
        let banner = "220 mail.example.com ESMTP Postfix";
        let server = BannerParser::parse_smtp_banner(banner);
        assert_eq!(server, Some("mail.example.com ESMTP Postfix".to_string()));
    }

    #[test]
    fn test_banner_grabber_creation() {
        let grabber = BannerGrabber::new();
        assert_eq!(grabber.timeout, Duration::from_secs(5));
        assert_eq!(grabber.max_banner_size, 4096);
    }

    #[test]
    fn test_set_timeout() {
        let mut grabber = BannerGrabber::new();
        grabber.set_timeout(Duration::from_secs(10));
        assert_eq!(grabber.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_set_max_banner_size() {
        let mut grabber = BannerGrabber::new();
        grabber.set_max_banner_size(8192);
        assert_eq!(grabber.max_banner_size, 8192);
    }
}
