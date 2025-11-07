//! Plugin metadata parsing and validation.
//!
//! This module provides functionality for parsing plugin.toml files and validating
//! plugin metadata according to the plugin system specification.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use toml;

/// Plugin metadata errors
#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Failed to read metadata file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid plugin type: {0}")]
    InvalidPluginType(String),

    #[error("Invalid capability: {0}")]
    InvalidCapability(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
}

/// Plugin type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// Scan lifecycle hooks (pre_scan, on_target, post_scan)
    Scan,
    /// Custom output formatting
    Output,
    /// Enhanced service detection
    Detection,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Scan => write!(f, "scan"),
            PluginType::Output => write!(f, "output"),
            PluginType::Detection => write!(f, "detection"),
        }
    }
}

impl std::str::FromStr for PluginType {
    type Err = MetadataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "scan" => Ok(PluginType::Scan),
            "output" => Ok(PluginType::Output),
            "detection" => Ok(PluginType::Detection),
            _ => Err(MetadataError::InvalidPluginType(s.to_string())),
        }
    }
}

/// Plugin metadata from plugin.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin section
    pub plugin: PluginInfo,
}

/// Core plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name (required)
    pub name: String,

    /// Semantic version (required)
    pub version: String,

    /// Author information (required)
    pub author: String,

    /// Short description (required)
    pub description: String,

    /// License (optional, defaults to GPL-3.0)
    #[serde(default = "default_license")]
    pub license: String,

    /// Plugin type (required)
    pub plugin_type: PluginType,

    /// Capabilities required (optional)
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Plugin dependencies (optional)
    #[serde(default)]
    pub dependencies: PluginDependencies,

    /// Additional metadata (optional)
    #[serde(default)]
    pub metadata: PluginMetadataExtra,
}

fn default_license() -> String {
    "GPL-3.0".to_string()
}

/// Plugin dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginDependencies {
    /// Minimum ProRT-IP version required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_prtip_version: Option<String>,

    /// Lua version requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lua_version: Option<String>,
}

/// Additional plugin metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginMetadataExtra {
    /// Plugin tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Plugin category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

impl PluginMetadata {
    /// Parse plugin metadata from TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self, MetadataError> {
        let contents = std::fs::read_to_string(path)?;
        Self::parse(&contents)
    }

    /// Parse plugin metadata from TOML string
    pub fn parse(toml: &str) -> Result<Self, MetadataError> {
        let metadata: PluginMetadata = toml::from_str(toml)?;

        // Validate required fields
        if metadata.plugin.name.is_empty() {
            return Err(MetadataError::MissingField("name".to_string()));
        }

        if metadata.plugin.version.is_empty() {
            return Err(MetadataError::MissingField("version".to_string()));
        }

        if metadata.plugin.author.is_empty() {
            return Err(MetadataError::MissingField("author".to_string()));
        }

        // Validate version format (simple semantic versioning check)
        if !Self::is_valid_version(&metadata.plugin.version) {
            return Err(MetadataError::InvalidVersion(
                metadata.plugin.version.clone(),
            ));
        }

        Ok(metadata)
    }

    /// Validate version string (basic semver check)
    fn is_valid_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return false;
        }

        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    /// Get plugin directory (parent of plugin.toml)
    pub fn directory(&self, toml_path: &Path) -> PathBuf {
        toml_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_metadata() {
        let toml = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
author = "Test Author"
description = "Test plugin"
plugin_type = "detection"
capabilities = ["network"]
"#;

        let metadata = PluginMetadata::parse(toml).unwrap();
        assert_eq!(metadata.plugin.name, "test-plugin");
        assert_eq!(metadata.plugin.version, "1.0.0");
        assert_eq!(metadata.plugin.plugin_type, PluginType::Detection);
        assert_eq!(metadata.plugin.capabilities, vec!["network"]);
    }

    #[test]
    fn test_missing_required_field() {
        let toml = r#"
[plugin]
version = "1.0.0"
author = "Test Author"
description = "Test plugin"
plugin_type = "detection"
"#;

        let result = PluginMetadata::parse(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_version() {
        let toml = r#"
[plugin]
name = "test-plugin"
version = "invalid"
author = "Test Author"
description = "Test plugin"
plugin_type = "detection"
"#;

        let result = PluginMetadata::parse(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_type_parsing() {
        assert_eq!("scan".parse::<PluginType>().unwrap(), PluginType::Scan);
        assert_eq!("output".parse::<PluginType>().unwrap(), PluginType::Output);
        assert_eq!(
            "detection".parse::<PluginType>().unwrap(),
            PluginType::Detection
        );
        assert!("invalid".parse::<PluginType>().is_err());
    }

    #[test]
    fn test_version_validation() {
        assert!(PluginMetadata::is_valid_version("1.0.0"));
        assert!(PluginMetadata::is_valid_version("0.1.0"));
        assert!(PluginMetadata::is_valid_version("10.20.30"));
        assert!(!PluginMetadata::is_valid_version("invalid"));
        assert!(!PluginMetadata::is_valid_version("1"));
        assert!(!PluginMetadata::is_valid_version("1.2.3.4"));
    }
}
