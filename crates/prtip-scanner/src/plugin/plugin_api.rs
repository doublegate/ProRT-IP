//! Plugin API trait definitions.
//!
//! This module defines the trait hierarchy for ProRT-IP plugins. All plugins must
//! implement the base `Plugin` trait, and then one of the specialized traits:
//! - `ScanPlugin`: Lifecycle hooks (pre_scan, on_target, post_scan)
//! - `OutputPlugin`: Custom result formatting
//! - `DetectionPlugin`: Enhanced service detection

use super::plugin_metadata::PluginType;
use mlua::Lua;
use parking_lot::Mutex;
use prtip_core::{Result as CoreResult, ScanResult, ScanTarget, ServiceInfo};
use std::any::Any;
use std::path::Path;
use std::sync::Arc;

/// Base plugin trait - all plugins must implement this
pub trait Plugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin version
    fn version(&self) -> &str;

    /// Get plugin type
    fn plugin_type(&self) -> PluginType;

    /// Initialize plugin with configuration
    ///
    /// Called once when the plugin is loaded. Return an error to prevent
    /// the plugin from being used.
    fn on_load(&mut self, config: &str) -> CoreResult<()>;

    /// Cleanup plugin resources
    ///
    /// Called when the plugin is unloaded. This is a best-effort cleanup;
    /// errors are logged but not propagated.
    fn on_unload(&mut self) -> CoreResult<()>;

    /// Downcast to concrete type (for trait upcasting)
    fn as_any(&self) -> &dyn Any;

    /// Downcast to concrete type (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Scan plugin - provides lifecycle hooks
pub trait ScanPlugin: Plugin {
    /// Called before scan starts
    ///
    /// Plugins can modify the target list (add/remove/reorder targets).
    /// Useful for port knocking, custom target selection, etc.
    fn pre_scan(&mut self, _targets: &mut Vec<ScanTarget>) -> CoreResult<()> {
        Ok(()) // Default: no-op
    }

    /// Called for each scanned target
    ///
    /// Plugins can add custom data to scan results via `result.custom_data`.
    /// Errors are logged but don't stop the scan.
    fn on_target(&mut self, _target: &ScanTarget, _result: &mut ScanResult) -> CoreResult<()> {
        Ok(()) // Default: no-op
    }

    /// Called after scan completes
    ///
    /// Useful for aggregate analysis, statistics, final reporting.
    fn post_scan(&mut self, _results: &[ScanResult]) -> CoreResult<()> {
        Ok(()) // Default: no-op
    }
}

/// Output plugin - provides custom result formatting
pub trait OutputPlugin: Plugin {
    /// Format single scan result
    ///
    /// Return a string representation of the result in the plugin's format.
    fn format_result(&self, result: &ScanResult) -> CoreResult<String>;

    /// Export all results to file
    ///
    /// Write all results to the specified path in the plugin's format.
    /// Requires Filesystem capability.
    fn export(&self, results: &[ScanResult], path: &Path) -> CoreResult<()>;
}

/// Detection plugin - provides enhanced service detection
pub trait DetectionPlugin: Plugin {
    /// Analyze service banner
    ///
    /// Parse banner string and return service information if recognized.
    /// Return None if the banner doesn't match this plugin's detection logic.
    fn analyze_banner(&self, banner: &str) -> CoreResult<Option<ServiceInfo>>;

    /// Actively probe service
    ///
    /// Send custom probes to the target and analyze responses.
    /// Requires Network capability.
    /// Return None if service is not detected.
    fn probe_service(&mut self, target: &ScanTarget) -> CoreResult<Option<ServiceInfo>>;
}

/// Lua plugin wrapper - implements Plugin trait by calling Lua functions
pub struct LuaPlugin {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub lua: Arc<Mutex<Lua>>,
}

impl LuaPlugin {
    pub fn new(
        name: String,
        version: String,
        plugin_type: PluginType,
        lua: Arc<Mutex<Lua>>,
    ) -> Self {
        Self {
            name,
            version,
            plugin_type,
            lua,
        }
    }
}

impl Plugin for LuaPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn plugin_type(&self) -> PluginType {
        self.plugin_type
    }

    fn on_load(&mut self, config: &str) -> CoreResult<()> {
        let lua = self.lua.lock();

        // Call Lua: on_load(config)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if on_load then
                return on_load(...)
            end
            return true
        "#,
            )
            .call(config);

        match result {
            Ok(mlua::Value::Boolean(true)) => Ok(()),
            Ok(mlua::Value::Boolean(false)) => Err(prtip_core::Error::from(std::io::Error::other(
                format!("Plugin {} initialization failed", self.name),
            ))),
            Ok(mlua::Value::String(err)) => {
                let err_str = err.to_string_lossy();
                Err(prtip_core::Error::from(std::io::Error::other(format!(
                    "Plugin {} error: {}",
                    self.name, err_str
                ))))
            }
            Ok(_) => Ok(()), // Other values treated as success
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Plugin {} Lua error: {}",
                self.name, e
            )))),
        }
    }

    fn on_unload(&mut self) -> CoreResult<()> {
        let lua = self.lua.lock();

        // Call Lua: on_unload() if it exists
        let _result: mlua::Result<()> = lua
            .load(
                r#"
            if on_unload then
                on_unload()
            end
        "#,
            )
            .exec();

        // Ignore errors in cleanup
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Lua scan plugin wrapper
pub struct LuaScanPlugin {
    pub base: LuaPlugin,
}

impl LuaScanPlugin {
    pub fn new(base: LuaPlugin) -> Self {
        Self { base }
    }
}

impl Plugin for LuaScanPlugin {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn version(&self) -> &str {
        self.base.version()
    }

    fn plugin_type(&self) -> PluginType {
        self.base.plugin_type()
    }

    fn on_load(&mut self, config: &str) -> CoreResult<()> {
        self.base.on_load(config)
    }

    fn on_unload(&mut self) -> CoreResult<()> {
        self.base.on_unload()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ScanPlugin for LuaScanPlugin {
    fn pre_scan(&mut self, _targets: &mut Vec<ScanTarget>) -> CoreResult<()> {
        // TODO: Call Lua on_pre_scan function
        Ok(())
    }

    fn on_target(&mut self, _target: &ScanTarget, _result: &mut ScanResult) -> CoreResult<()> {
        // TODO: Call Lua on_target function
        Ok(())
    }

    fn post_scan(&mut self, _results: &[ScanResult]) -> CoreResult<()> {
        // TODO: Call Lua on_post_scan function
        Ok(())
    }
}

/// Lua output plugin wrapper
pub struct LuaOutputPlugin {
    pub base: LuaPlugin,
}

impl LuaOutputPlugin {
    pub fn new(base: LuaPlugin) -> Self {
        Self { base }
    }
}

impl Plugin for LuaOutputPlugin {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn version(&self) -> &str {
        self.base.version()
    }

    fn plugin_type(&self) -> PluginType {
        self.base.plugin_type()
    }

    fn on_load(&mut self, config: &str) -> CoreResult<()> {
        self.base.on_load(config)
    }

    fn on_unload(&mut self) -> CoreResult<()> {
        self.base.on_unload()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl OutputPlugin for LuaOutputPlugin {
    fn format_result(&self, _result: &ScanResult) -> CoreResult<String> {
        // TODO: Call Lua format_result function
        Ok(String::new())
    }

    fn export(&self, _results: &[ScanResult], _path: &Path) -> CoreResult<()> {
        // TODO: Call Lua export function
        Ok(())
    }
}

/// Lua detection plugin wrapper
pub struct LuaDetectionPlugin {
    pub base: LuaPlugin,
}

impl LuaDetectionPlugin {
    pub fn new(base: LuaPlugin) -> Self {
        Self { base }
    }
}

impl Plugin for LuaDetectionPlugin {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn version(&self) -> &str {
        self.base.version()
    }

    fn plugin_type(&self) -> PluginType {
        self.base.plugin_type()
    }

    fn on_load(&mut self, config: &str) -> CoreResult<()> {
        self.base.on_load(config)
    }

    fn on_unload(&mut self) -> CoreResult<()> {
        self.base.on_unload()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DetectionPlugin for LuaDetectionPlugin {
    fn analyze_banner(&self, banner: &str) -> CoreResult<Option<ServiceInfo>> {
        let lua = self.base.lua.lock();

        // Call Lua: analyze_banner(banner)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if analyze_banner then
                return analyze_banner(...)
            end
            return nil
        "#,
            )
            .call(banner);

        match result {
            Ok(mlua::Value::Nil) => Ok(None),
            Ok(mlua::Value::Table(table)) => {
                // Parse Lua table into ServiceInfo
                let service: String = table.get("service").map_err(|e| {
                    prtip_core::Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Missing 'service' field: {}", e),
                    ))
                })?;

                let product: Option<String> = table.get("product").ok();
                let version: Option<String> = table.get("version").ok();
                let info: Option<String> = table.get("info").ok();
                let os_type: Option<String> = table.get("os_type").ok();
                let confidence: f32 = table.get("confidence").unwrap_or(0.5);

                Ok(Some(ServiceInfo {
                    service,
                    product,
                    version,
                    info,
                    os_type,
                    confidence,
                }))
            }
            Ok(_) => Ok(None), // Other return types treated as no detection
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Lua error in analyze_banner: {}",
                e
            )))),
        }
    }

    fn probe_service(&mut self, target: &ScanTarget) -> CoreResult<Option<ServiceInfo>> {
        let lua = self.base.lua.lock();

        // Create target table for Lua
        let target_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create target table: {}",
                e
            )))
        })?;

        // Get IP address from network
        let ip_addr = target.network.ip();
        target_table.set("ip", ip_addr.to_string()).map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!("Failed to set ip: {}", e)))
        })?;

        // Add hostname if available
        if let Some(ref hostname) = target.hostname {
            target_table
                .set("hostname", hostname.clone())
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set hostname: {}",
                        e
                    )))
                })?;
        }

        // Call Lua: probe_service(target)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if probe_service then
                return probe_service(...)
            end
            return nil
        "#,
            )
            .call(target_table);

        match result {
            Ok(mlua::Value::Nil) => Ok(None),
            Ok(mlua::Value::Table(table)) => {
                // Parse Lua table into ServiceInfo
                let service: String = table.get("service").map_err(|e| {
                    prtip_core::Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Missing 'service' field: {}", e),
                    ))
                })?;

                let product: Option<String> = table.get("product").ok();
                let version: Option<String> = table.get("version").ok();
                let info: Option<String> = table.get("info").ok();
                let os_type: Option<String> = table.get("os_type").ok();
                let confidence: f32 = table.get("confidence").unwrap_or(0.5);

                Ok(Some(ServiceInfo {
                    service,
                    product,
                    version,
                    info,
                    os_type,
                    confidence,
                }))
            }
            Ok(_) => Ok(None), // Other return types treated as no detection
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Lua error in probe_service: {}",
                e
            )))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_plugin_creation() {
        let lua = Arc::new(Mutex::new(Lua::new()));
        let plugin = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Detection,
            lua,
        );

        assert_eq!(plugin.name(), "test-plugin");
        assert_eq!(plugin.version(), "1.0.0");
        assert_eq!(plugin.plugin_type(), PluginType::Detection);
    }

    #[test]
    fn test_lua_scan_plugin() {
        let lua = Arc::new(Mutex::new(Lua::new()));
        let base = LuaPlugin::new(
            "scan-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua,
        );
        let mut plugin = LuaScanPlugin::new(base);

        // Test default implementations (should not panic)
        let mut targets = vec![];
        assert!(plugin.pre_scan(&mut targets).is_ok());

        let results = vec![];
        assert!(plugin.post_scan(&results).is_ok());
    }
}
