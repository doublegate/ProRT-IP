//! Security sandboxing for plugin execution.
//!
//! This module provides capabilities-based access control and Lua VM sandboxing
//! to ensure plugins cannot perform unauthorized operations.

use std::collections::HashSet;
use thiserror::Error;

/// Security errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Permission denied: plugin lacks '{0}' capability")]
    PermissionDenied(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
}

/// Plugin capability types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Network operations (socket connect, send, receive)
    Network,
    /// Filesystem operations (read, write files)
    Filesystem,
    /// System operations (future, currently denied)
    System,
    /// Database operations (future)
    Database,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Network => write!(f, "network"),
            Capability::Filesystem => write!(f, "filesystem"),
            Capability::System => write!(f, "system"),
            Capability::Database => write!(f, "database"),
        }
    }
}

impl std::str::FromStr for Capability {
    type Err = SecurityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "network" => Ok(Capability::Network),
            "filesystem" => Ok(Capability::Filesystem),
            "system" => Ok(Capability::System),
            "database" => Ok(Capability::Database),
            _ => Err(SecurityError::SandboxViolation(format!(
                "Invalid capability: {}",
                s
            ))),
        }
    }
}

/// Plugin capabilities container (deny-by-default)
#[derive(Debug, Clone)]
pub struct PluginCapabilities {
    allowed: HashSet<Capability>,
}

impl PluginCapabilities {
    /// Create new capabilities set (empty by default)
    pub fn new() -> Self {
        Self {
            allowed: HashSet::new(),
        }
    }

    /// Create capabilities from string list
    pub fn from_strings(caps: &[String]) -> Result<Self, SecurityError> {
        let mut capabilities = Self::new();
        for cap_str in caps {
            let cap: Capability = cap_str.parse()?;
            capabilities.allowed.insert(cap);
        }
        Ok(capabilities)
    }

    /// Add capability
    pub fn add(&mut self, cap: Capability) {
        self.allowed.insert(cap);
    }

    /// Check if capability is allowed
    pub fn has(&self, cap: Capability) -> bool {
        self.allowed.contains(&cap)
    }

    /// Check capability and return error if missing
    pub fn require(&self, cap: Capability) -> Result<(), SecurityError> {
        if self.has(cap) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(cap.to_string()))
        }
    }

    /// Get all allowed capabilities
    pub fn list(&self) -> Vec<Capability> {
        self.allowed.iter().copied().collect()
    }
}

impl Default for PluginCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource limits for plugin execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (default: 100 MB)
    pub max_memory_bytes: usize,

    /// Maximum CPU time in seconds (default: 5 seconds)
    pub max_cpu_seconds: u64,

    /// Maximum Lua instructions (default: 1 million)
    pub max_instructions: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_seconds: 5,                  // 5 seconds
            max_instructions: 1_000_000,         // 1M instructions
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum memory in MB
    pub fn with_max_memory_mb(mut self, mb: usize) -> Self {
        self.max_memory_bytes = mb * 1024 * 1024;
        self
    }

    /// Set maximum CPU time in seconds
    pub fn with_max_cpu_seconds(mut self, seconds: u64) -> Self {
        self.max_cpu_seconds = seconds;
        self
    }

    /// Set maximum instructions
    pub fn with_max_instructions(mut self, instructions: usize) -> Self {
        self.max_instructions = instructions;
        self
    }
}

/// Sandbox configuration combining capabilities and resource limits
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub capabilities: PluginCapabilities,
    pub limits: ResourceLimits,
}

impl SandboxConfig {
    /// Create new sandbox configuration
    pub fn new(capabilities: PluginCapabilities, limits: ResourceLimits) -> Self {
        Self {
            capabilities,
            limits,
        }
    }

    /// Create sandbox with default deny-all capabilities
    pub fn deny_all() -> Self {
        Self {
            capabilities: PluginCapabilities::new(),
            limits: ResourceLimits::default(),
        }
    }

    /// Create sandbox from capability strings
    pub fn from_capability_strings(caps: &[String]) -> Result<Self, SecurityError> {
        Ok(Self {
            capabilities: PluginCapabilities::from_strings(caps)?,
            limits: ResourceLimits::default(),
        })
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::deny_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_parsing() {
        assert_eq!(
            "network".parse::<Capability>().unwrap(),
            Capability::Network
        );
        assert_eq!(
            "filesystem".parse::<Capability>().unwrap(),
            Capability::Filesystem
        );
        assert!("invalid".parse::<Capability>().is_err());
    }

    #[test]
    fn test_capabilities_deny_by_default() {
        let caps = PluginCapabilities::new();
        assert!(!caps.has(Capability::Network));
        assert!(!caps.has(Capability::Filesystem));
    }

    #[test]
    fn test_capabilities_add_and_check() {
        let mut caps = PluginCapabilities::new();
        caps.add(Capability::Network);

        assert!(caps.has(Capability::Network));
        assert!(!caps.has(Capability::Filesystem));
    }

    #[test]
    fn test_capabilities_require() {
        let mut caps = PluginCapabilities::new();
        caps.add(Capability::Network);

        assert!(caps.require(Capability::Network).is_ok());
        assert!(caps.require(Capability::Filesystem).is_err());
    }

    #[test]
    fn test_capabilities_from_strings() {
        let cap_strings = vec!["network".to_string(), "filesystem".to_string()];
        let caps = PluginCapabilities::from_strings(&cap_strings).unwrap();

        assert!(caps.has(Capability::Network));
        assert!(caps.has(Capability::Filesystem));
        assert!(!caps.has(Capability::System));
    }

    #[test]
    fn test_resource_limits_defaults() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 100 * 1024 * 1024);
        assert_eq!(limits.max_cpu_seconds, 5);
        assert_eq!(limits.max_instructions, 1_000_000);
    }

    #[test]
    fn test_resource_limits_builder() {
        let limits = ResourceLimits::new()
            .with_max_memory_mb(50)
            .with_max_cpu_seconds(10)
            .with_max_instructions(500_000);

        assert_eq!(limits.max_memory_bytes, 50 * 1024 * 1024);
        assert_eq!(limits.max_cpu_seconds, 10);
        assert_eq!(limits.max_instructions, 500_000);
    }

    #[test]
    fn test_sandbox_config_deny_all() {
        let config = SandboxConfig::deny_all();
        assert!(!config.capabilities.has(Capability::Network));
        assert!(!config.capabilities.has(Capability::Filesystem));
    }

    #[test]
    fn test_sandbox_config_from_strings() {
        let cap_strings = vec!["network".to_string()];
        let config = SandboxConfig::from_capability_strings(&cap_strings).unwrap();

        assert!(config.capabilities.has(Capability::Network));
        assert!(!config.capabilities.has(Capability::Filesystem));
    }
}
