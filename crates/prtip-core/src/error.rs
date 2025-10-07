//! Error types for ProRT-IP core library

use std::io;
use std::net::AddrParseError;
use thiserror::Error;

/// Result type alias for ProRT-IP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error types for scanning operations
#[derive(Error, Debug)]
pub enum Error {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// I/O errors with automatic conversion
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Parsing errors for IP addresses, CIDR, ports
    #[error("Parse error: {0}")]
    Parse(String),

    /// Permission/capability errors
    #[error("Insufficient privileges: {0}")]
    Privilege(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Storage/database errors
    #[error("Storage error: {0}")]
    Storage(String),

    /// Invalid target specification
    #[error("Invalid target: {0}")]
    InvalidTarget(String),

    /// Invalid port range
    #[error("Invalid port range: {0}")]
    InvalidPortRange(String),

    /// Invalid CIDR notation
    #[error("Invalid CIDR: {0}")]
    InvalidCidr(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Error::Parse(format!("Invalid IP address: {}", err))
    }
}

impl From<ipnetwork::IpNetworkError> for Error {
    fn from(err: ipnetwork::IpNetworkError) -> Self {
        Error::Parse(format!("Invalid IP network: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(format!("JSON error: {}", err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Config(format!("TOML parse error: {}", err))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Config(format!("TOML serialization error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let err = Error::Network("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_parse_error_conversion() {
        let parse_err = "invalid".parse::<std::net::IpAddr>().unwrap_err();
        let err: Error = parse_err.into();
        assert!(matches!(err, Error::Parse(_)));
        assert!(err.to_string().contains("Invalid IP address"));
    }

    #[test]
    fn test_network_error_conversion() {
        use ipnetwork::IpNetwork;
        let network_err = "999.0.0.0/24".parse::<IpNetwork>().unwrap_err();
        let err: Error = network_err.into();
        assert!(matches!(err, Error::Parse(_)));
        assert!(err.to_string().contains("Invalid IP network"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<Vec<i32>>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn test_error_result_type() {
        fn returns_result() -> Result<i32> {
            Err(Error::Timeout)
        }

        let result = returns_result();
        assert!(result.is_err());
        if let Err(Error::Timeout) = result {
            // Success
        } else {
            panic!("Expected Timeout error");
        }
    }
}
