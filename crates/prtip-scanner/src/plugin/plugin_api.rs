//! Plugin API trait definitions.
//!
//! This module defines the trait hierarchy for ProRT-IP plugins. All plugins must
//! implement the base `Plugin` trait, and then one of the specialized traits:
//! - `ScanPlugin`: Lifecycle hooks (pre_scan, on_target, post_scan)
//! - `OutputPlugin`: Custom result formatting
//! - `DetectionPlugin`: Enhanced service detection

use super::lua_api::toml_to_lua;
use super::plugin_metadata::PluginType;
use mlua::Lua;
use parking_lot::Mutex;
use prtip_core::{Result as CoreResult, ScanResult, ScanTarget, ServiceInfo};
use std::any::Any;
use std::path::Path;
use std::sync::Arc;

#[cfg(test)]
use chrono::Utc;

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

        // Parse TOML config string and convert to Lua table
        let config_value = if config.is_empty() {
            // Empty config: pass empty table
            mlua::Value::Table(lua.create_table().map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to create empty config table: {}",
                    e
                )))
            })?)
        } else {
            // Parse TOML and convert to Lua table
            let toml_table: toml::Table = toml::from_str(config).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to parse plugin config TOML: {}",
                    e
                )))
            })?;

            let lua_table = toml_to_lua(&lua, &toml_table).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to convert config to Lua table: {}",
                    e
                )))
            })?;

            mlua::Value::Table(lua_table)
        };

        // Call Lua: on_load(config_table)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if on_load then
                return on_load(...)
            end
            return true
        "#,
            )
            .call(config_value);

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
    fn pre_scan(&mut self, targets: &mut Vec<ScanTarget>) -> CoreResult<()> {
        let lua = self.base.lua.lock();

        // Convert targets Vec to Lua array
        let targets_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create targets table: {}",
                e
            )))
        })?;

        for (i, target) in targets.iter().enumerate() {
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

            // Add network CIDR notation
            target_table
                .set("network", target.network.to_string())
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set network: {}",
                        e
                    )))
                })?;

            // Lua arrays are 1-indexed
            targets_table.set(i + 1, target_table).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to add target to array: {}",
                    e
                )))
            })?;
        }

        // Call Lua: on_pre_scan(targets)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if on_pre_scan then
                return on_pre_scan(...)
            end
            return nil
        "#,
            )
            .call(targets_table);

        match result {
            Ok(_) => Ok(()), // Success or nil (void return)
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Lua error in on_pre_scan: {}",
                e
            )))),
        }
    }

    fn on_target(&mut self, target: &ScanTarget, result: &mut ScanResult) -> CoreResult<()> {
        let lua = self.base.lua.lock();

        // Convert target to Lua table
        let target_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create target table: {}",
                e
            )))
        })?;

        let ip_addr = target.network.ip();
        target_table.set("ip", ip_addr.to_string()).map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!("Failed to set ip: {}", e)))
        })?;

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

        // Convert result to Lua table
        let result_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create result table: {}",
                e
            )))
        })?;

        result_table
            .set("target_ip", result.target_ip.to_string())
            .map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set target_ip: {}",
                    e
                )))
            })?;
        result_table.set("port", result.port).map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!("Failed to set port: {}", e)))
        })?;
        result_table
            .set("state", result.state.to_string())
            .map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set state: {}",
                    e
                )))
            })?;
        result_table
            .set("response_time_ms", result.response_time.as_millis() as u64)
            .map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set response_time_ms: {}",
                    e
                )))
            })?;

        if let Some(ref banner) = result.banner {
            result_table.set("banner", banner.clone()).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set banner: {}",
                    e
                )))
            })?;
        }

        if let Some(ref service) = result.service {
            result_table.set("service", service.clone()).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set service: {}",
                    e
                )))
            })?;
        }

        if let Some(ref version) = result.version {
            result_table.set("version", version.clone()).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set version: {}",
                    e
                )))
            })?;
        }

        // Call Lua: on_target(target, result)
        let lua_result: mlua::Result<mlua::Table> = lua
            .load(
                r#"
            if on_target then
                return on_target(...)
            end
            return nil
        "#,
            )
            .call((target_table, result_table));

        // Process returned table and update result if modified
        match lua_result {
            Ok(modified_result) => {
                // Update banner if modified
                if let Ok(Some(banner)) = modified_result.get::<Option<String>>("banner") {
                    result.banner = Some(banner);
                }

                // Update service if modified
                if let Ok(Some(service)) = modified_result.get::<Option<String>>("service") {
                    result.service = Some(service);
                }

                // Update version if modified
                if let Ok(Some(version)) = modified_result.get::<Option<String>>("version") {
                    result.version = Some(version);
                }

                Ok(())
            }
            Err(e) => {
                // If Lua returns nil or error doesn't happen, it's OK
                if e.to_string().contains("attempt to index a nil value") {
                    Ok(()) // on_target returned nil, which is fine
                } else {
                    Err(prtip_core::Error::from(std::io::Error::other(format!(
                        "Lua error in on_target: {}",
                        e
                    ))))
                }
            }
        }
    }

    fn post_scan(&mut self, results: &[ScanResult]) -> CoreResult<()> {
        let lua = self.base.lua.lock();

        // Convert results slice to Lua array
        let results_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create results table: {}",
                e
            )))
        })?;

        for (i, result) in results.iter().enumerate() {
            let result_table = lua.create_table().map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to create result table: {}",
                    e
                )))
            })?;

            result_table
                .set("target_ip", result.target_ip.to_string())
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set target_ip: {}",
                        e
                    )))
                })?;
            result_table.set("port", result.port).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!("Failed to set port: {}", e)))
            })?;
            result_table
                .set("state", result.state.to_string())
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set state: {}",
                        e
                    )))
                })?;
            result_table
                .set("response_time_ms", result.response_time.as_millis() as u64)
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set response_time_ms: {}",
                        e
                    )))
                })?;

            if let Some(ref banner) = result.banner {
                result_table.set("banner", banner.clone()).map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set banner: {}",
                        e
                    )))
                })?;
            }

            if let Some(ref service) = result.service {
                result_table.set("service", service.clone()).map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set service: {}",
                        e
                    )))
                })?;
            }

            if let Some(ref version) = result.version {
                result_table.set("version", version.clone()).map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set version: {}",
                        e
                    )))
                })?;
            }

            // Lua arrays are 1-indexed
            results_table.set(i + 1, result_table).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to add result to array: {}",
                    e
                )))
            })?;
        }

        // Call Lua: on_post_scan(results)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if on_post_scan then
                return on_post_scan(...)
            end
            return nil
        "#,
            )
            .call(results_table);

        match result {
            Ok(_) => Ok(()), // Success or nil (void return)
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Lua error in on_post_scan: {}",
                e
            )))),
        }
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
    fn format_result(&self, result: &ScanResult) -> CoreResult<String> {
        let lua = self.base.lua.lock();

        // Convert result to Lua table
        let result_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create result table: {}",
                e
            )))
        })?;

        result_table
            .set("target_ip", result.target_ip.to_string())
            .map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set target_ip: {}",
                    e
                )))
            })?;
        result_table.set("port", result.port).map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!("Failed to set port: {}", e)))
        })?;
        result_table
            .set("state", result.state.to_string())
            .map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set state: {}",
                    e
                )))
            })?;
        result_table
            .set("response_time_ms", result.response_time.as_millis() as u64)
            .map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set response_time_ms: {}",
                    e
                )))
            })?;

        if let Some(ref banner) = result.banner {
            result_table.set("banner", banner.clone()).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set banner: {}",
                    e
                )))
            })?;
        }

        if let Some(ref service) = result.service {
            result_table.set("service", service.clone()).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set service: {}",
                    e
                )))
            })?;
        }

        if let Some(ref version) = result.version {
            result_table.set("version", version.clone()).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to set version: {}",
                    e
                )))
            })?;
        }

        // Call Lua: format_result(result) -> string
        let lua_result: mlua::Result<String> = lua
            .load(
                r#"
            if format_result then
                return format_result(...)
            end
            return ""
        "#,
            )
            .call(result_table);

        match lua_result {
            Ok(formatted_string) => Ok(formatted_string),
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Lua error in format_result: {}",
                e
            )))),
        }
    }

    fn export(&self, results: &[ScanResult], path: &Path) -> CoreResult<()> {
        let lua = self.base.lua.lock();

        // Convert results slice to Lua array
        let results_table = lua.create_table().map_err(|e| {
            prtip_core::Error::from(std::io::Error::other(format!(
                "Failed to create results table: {}",
                e
            )))
        })?;

        for (i, result) in results.iter().enumerate() {
            let result_table = lua.create_table().map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to create result table: {}",
                    e
                )))
            })?;

            result_table
                .set("target_ip", result.target_ip.to_string())
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set target_ip: {}",
                        e
                    )))
                })?;
            result_table.set("port", result.port).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!("Failed to set port: {}", e)))
            })?;
            result_table
                .set("state", result.state.to_string())
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set state: {}",
                        e
                    )))
                })?;
            result_table
                .set("response_time_ms", result.response_time.as_millis() as u64)
                .map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set response_time_ms: {}",
                        e
                    )))
                })?;

            if let Some(ref banner) = result.banner {
                result_table.set("banner", banner.clone()).map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set banner: {}",
                        e
                    )))
                })?;
            }

            if let Some(ref service) = result.service {
                result_table.set("service", service.clone()).map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set service: {}",
                        e
                    )))
                })?;
            }

            if let Some(ref version) = result.version {
                result_table.set("version", version.clone()).map_err(|e| {
                    prtip_core::Error::from(std::io::Error::other(format!(
                        "Failed to set version: {}",
                        e
                    )))
                })?;
            }

            // Lua arrays are 1-indexed
            results_table.set(i + 1, result_table).map_err(|e| {
                prtip_core::Error::from(std::io::Error::other(format!(
                    "Failed to add result to array: {}",
                    e
                )))
            })?;
        }

        // Convert path to string
        let path_str = path.to_string_lossy().to_string();

        // Call Lua: export(results, path)
        let result: mlua::Result<mlua::Value> = lua
            .load(
                r#"
            if export then
                return export(...)
            end
            return nil
        "#,
            )
            .call((results_table, path_str));

        match result {
            Ok(_) => Ok(()), // Success or nil (void return)
            Err(e) => Err(prtip_core::Error::from(std::io::Error::other(format!(
                "Lua error in export: {}",
                e
            )))),
        }
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

    #[test]
    fn test_on_load_with_empty_config() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with on_load function
        lua.load(
            r#"
            function on_load(config)
                return true
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let mut plugin = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua_arc,
        );

        // Test empty config
        assert!(plugin.on_load("").is_ok());
    }

    #[test]
    fn test_on_load_with_toml_config() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code that validates config
        lua.load(
            r#"
            function on_load(config)
                if config.timeout and config.retries then
                    return true
                end
                return false
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let mut plugin = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua_arc,
        );

        // Test TOML config
        let config_toml = r#"
timeout = 5000
retries = 3
"#;
        assert!(plugin.on_load(config_toml).is_ok());
    }

    #[test]
    fn test_pre_scan_callback() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;
        use ipnetwork::IpNetwork;
        use std::str::FromStr;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with pre_scan function
        lua.load(
            r#"
            function on_load(config)
                return true
            end

            function on_pre_scan(targets)
                -- Lua function executed successfully
                return true
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let base = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua_arc,
        );
        let mut plugin = LuaScanPlugin::new(base);

        // Initialize plugin
        plugin.on_load("").unwrap();

        // Test pre_scan with some targets
        let mut targets = vec![
            ScanTarget {
                network: IpNetwork::from_str("192.168.1.1/32").unwrap(),
                hostname: None,
            },
            ScanTarget {
                network: IpNetwork::from_str("192.168.1.2/32").unwrap(),
                hostname: Some("test.local".to_string()),
            },
        ];

        assert!(plugin.pre_scan(&mut targets).is_ok());
    }

    #[test]
    fn test_on_target_callback() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;
        use ipnetwork::IpNetwork;
        use std::net::IpAddr;
        use std::str::FromStr;
        use std::time::Duration;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with on_target function that modifies banner
        lua.load(
            r#"
            function on_load(config)
                return true
            end

            function on_target(target, result)
                -- Modify banner
                result.banner = "Modified by plugin"
                return result
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let base = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua_arc,
        );
        let mut plugin = LuaScanPlugin::new(base);

        // Initialize plugin
        plugin.on_load("").unwrap();

        // Test on_target
        let target = ScanTarget {
            network: IpNetwork::from_str("192.168.1.1/32").unwrap(),
            hostname: None,
        };

        let mut result = ScanResult {
            target_ip: IpAddr::from_str("192.168.1.1").unwrap(),
            port: 80,
            state: prtip_core::PortState::Open,
            response_time: Duration::from_millis(10),
            timestamp: Utc::now(),
            banner: Some("Original banner".to_string()),
            service: None,
            version: None,
            raw_response: None,
        };

        assert!(plugin.on_target(&target, &mut result).is_ok());
        // Verify banner was modified by Lua plugin
        assert_eq!(result.banner, Some("Modified by plugin".to_string()));
    }

    #[test]
    fn test_post_scan_callback() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;
        use std::net::IpAddr;
        use std::str::FromStr;
        use std::time::Duration;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with post_scan function
        lua.load(
            r#"
            function on_load(config)
                return true
            end

            function on_post_scan(results)
                -- Lua function executed successfully
                return true
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let base = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua_arc,
        );
        let mut plugin = LuaScanPlugin::new(base);

        // Initialize plugin
        plugin.on_load("").unwrap();

        // Test post_scan with results
        let results = vec![
            ScanResult {
                target_ip: IpAddr::from_str("192.168.1.1").unwrap(),
                port: 80,
                state: prtip_core::PortState::Open,
                response_time: Duration::from_millis(10),
                timestamp: Utc::now(),
                banner: None,
                service: None,
                version: None,
                raw_response: None,
            },
            ScanResult {
                target_ip: IpAddr::from_str("192.168.1.1").unwrap(),
                port: 443,
                state: prtip_core::PortState::Open,
                response_time: Duration::from_millis(15),
                timestamp: Utc::now(),
                banner: None,
                service: None,
                version: None,
                raw_response: None,
            },
        ];

        assert!(plugin.post_scan(&results).is_ok());
    }

    #[test]
    fn test_format_result_callback() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;
        use std::net::IpAddr;
        use std::str::FromStr;
        use std::time::Duration;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with format_result function
        lua.load(
            r#"
            function on_load(config)
                return true
            end

            function format_result(result)
                return "Port " .. result.port .. " is " .. result.state
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let base = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Output,
            lua_arc,
        );
        let mut plugin = LuaOutputPlugin::new(base);

        // Initialize plugin
        plugin.on_load("").unwrap();

        // Test format_result
        let result = ScanResult {
            target_ip: IpAddr::from_str("192.168.1.1").unwrap(),
            port: 80,
            state: prtip_core::PortState::Open,
            response_time: Duration::from_millis(10),
            timestamp: Utc::now(),
            banner: None,
            service: None,
            version: None,
            raw_response: None,
        };

        let formatted = plugin.format_result(&result).unwrap();
        assert_eq!(formatted, "Port 80 is open");
    }

    #[test]
    fn test_export_callback() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;
        use std::net::IpAddr;
        use std::path::Path;
        use std::str::FromStr;
        use std::time::Duration;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with export function
        lua.load(
            r#"
            function on_load(config)
                return true
            end

            function export(results, path)
                -- Lua function executed successfully
                -- In real plugin, would write to file
                return true
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let base = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Output,
            lua_arc,
        );
        let mut plugin = LuaOutputPlugin::new(base);

        // Initialize plugin
        plugin.on_load("").unwrap();

        // Test export
        let results = vec![ScanResult {
            target_ip: IpAddr::from_str("192.168.1.1").unwrap(),
            port: 80,
            state: prtip_core::PortState::Open,
            response_time: Duration::from_millis(10),
            timestamp: Utc::now(),
            banner: None,
            service: None,
            version: None,
            raw_response: None,
        }];

        let path = Path::new("/tmp/test-export.txt");
        assert!(plugin.export(&results, path).is_ok());
    }

    #[test]
    fn test_lua_error_handling() {
        use super::super::lua_api::LuaContext;
        use super::super::lua_api::{create_sandboxed_vm, register_prtip_api, set_lua_context};
        use super::super::sandbox::PluginCapabilities;

        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        let context = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, context).unwrap();

        // Load Lua code with on_load that returns false (error)
        lua.load(
            r#"
            function on_load(config)
                return false  -- Signal failure
            end
        "#,
        )
        .exec()
        .unwrap();

        let lua_arc = Arc::new(Mutex::new(lua));
        let mut plugin = LuaPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::Scan,
            lua_arc,
        );

        // Test that on_load returns error when Lua returns false
        assert!(plugin.on_load("").is_err());
    }
}
