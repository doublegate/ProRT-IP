//! ProRT-IP Plugin System
//!
//! This module provides a Lua-based plugin system for extending ProRT-IP functionality.
//! Plugins can hook into the scan lifecycle, provide custom output formats, or enhance
//! service detection.
//!
//! # Security
//!
//! Plugins run in sandboxed Lua VMs with:
//! - Dangerous libraries removed (io, os, debug)
//! - Resource limits (memory, CPU, instructions)
//! - Capabilities-based access control (deny-by-default)
//!
//! # Plugin Types
//!
//! - **ScanPlugin**: Pre-scan, per-target, and post-scan hooks
//! - **OutputPlugin**: Custom result formatting and export
//! - **DetectionPlugin**: Enhanced service detection via banner analysis or active probing
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::plugin::{PluginManager, PluginType};
//!
//! # fn main() -> prtip_core::Result<()> {
//! // Create plugin manager
//! let mut manager = PluginManager::with_default_dir()?;
//!
//! // Discover plugins in ~/.prtip/plugins/
//! manager.discover_plugins()?;
//!
//! // Load specific plugin
//! manager.load_plugin("banner-analyzer")?;
//!
//! // List loaded plugins
//! for name in manager.list_loaded() {
//!     println!("Loaded: {}", name);
//! }
//! # Ok(())
//! # }
//! ```

mod lua_api;
mod plugin_api;
mod plugin_manager;
mod plugin_metadata;
mod sandbox;

// Re-export public API
pub use lua_api::{create_sandboxed_vm, register_prtip_api, LuaContext};
pub use plugin_api::{
    DetectionPlugin, LuaDetectionPlugin, LuaOutputPlugin, LuaPlugin, LuaScanPlugin, OutputPlugin,
    Plugin, ScanPlugin,
};
pub use plugin_manager::PluginManager;
pub use plugin_metadata::{PluginDependencies, PluginInfo, PluginMetadata, PluginType};
pub use sandbox::{Capability, PluginCapabilities, ResourceLimits, SandboxConfig, SecurityError};
