//! Plugin manager for discovery, loading, and lifecycle management.
//!
//! This module provides the `PluginManager` which handles:
//! - Plugin discovery (scanning ~/.prtip/plugins/)
//! - Plugin loading (creating Lua VMs, loading scripts)
//! - Plugin lifecycle (on_load, on_unload)
//! - Plugin execution coordination
//!
//! # See Also
//!
//! - [Plugin System Guide](../../../docs/30-PLUGIN-SYSTEM-GUIDE.md) - Complete plugin development guide
//! - [User Guide: Plugins](../../../docs/32-USER-GUIDE.md#use-case-14-plugin-development-and-extension) - Plugin usage examples
//! - [`lua_api`](super::lua_api) - Lua API and sandboxing details

use super::lua_api::{
    create_sandboxed_vm, register_prtip_api, set_lua_context, set_resource_limits, LuaContext,
};
use super::plugin_api::{LuaDetectionPlugin, LuaOutputPlugin, LuaPlugin, LuaScanPlugin, Plugin};
use super::plugin_metadata::{PluginMetadata, PluginType};
use super::sandbox::{PluginCapabilities, ResourceLimits};
use parking_lot::Mutex;
use prtip_core::Result as CoreResult;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Plugin manager - central orchestrator for plugin system
pub struct PluginManager {
    /// Plugin metadata cache (lightweight)
    metadata_cache: HashMap<String, PluginMetadata>,

    /// Loaded plugins (heavy - Lua VMs)
    loaded_plugins: HashMap<String, Box<dyn Plugin>>,

    /// Plugins directory path
    plugins_dir: PathBuf,

    /// Default resource limits
    default_limits: ResourceLimits,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            metadata_cache: HashMap::new(),
            loaded_plugins: HashMap::new(),
            plugins_dir,
            default_limits: ResourceLimits::default(),
        }
    }

    /// Create plugin manager with default directory (~/.prtip/plugins)
    pub fn with_default_dir() -> CoreResult<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("HOME or USERPROFILE not found: {}", e),
                )
            })?;
        let plugins_dir = PathBuf::from(home).join(".prtip").join("plugins");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&plugins_dir)?;

        Ok(Self::new(plugins_dir))
    }

    /// Discover plugins in the plugins directory
    ///
    /// Scans for plugin.toml files and caches metadata.
    /// Returns count of discovered plugins.
    pub fn discover_plugins(&mut self) -> CoreResult<usize> {
        info!("Discovering plugins in {:?}", self.plugins_dir);

        self.metadata_cache.clear();
        let mut count = 0;

        if !self.plugins_dir.exists() {
            warn!("Plugins directory does not exist: {:?}", self.plugins_dir);
            return Ok(0);
        }

        // Scan directory for plugin.toml files
        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let toml_path = path.join("plugin.toml");
                if toml_path.exists() {
                    match PluginMetadata::from_file(&toml_path) {
                        Ok(metadata) => {
                            let plugin_name = metadata.plugin.name.clone();
                            info!(
                                "Discovered plugin: {} v{}",
                                plugin_name, metadata.plugin.version
                            );
                            self.metadata_cache.insert(plugin_name, metadata);
                            count += 1;
                        }
                        Err(e) => {
                            warn!("Failed to parse {}: {}", toml_path.display(), e);
                        }
                    }
                }
            }
        }

        info!("Discovered {} plugins", count);
        Ok(count)
    }

    /// Load plugin by name
    ///
    /// Creates Lua VM, loads script, registers API, and calls on_load().
    /// Plugin is cached for subsequent use.
    ///
    /// # Arguments
    ///
    /// * `name` - Plugin name (must match directory name in plugins_dir)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Plugin loaded successfully
    /// * `Err` - Plugin not found, load error, or on_load() failure
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Plugin not discovered (call `discover_plugins()` first)
    /// - `main.lua` not found in plugin directory
    /// - Lua syntax error in plugin code
    /// - Plugin's `on_load()` function returns false or errors
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_scanner::plugin::PluginManager;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create manager and discover plugins
    /// let mut manager = PluginManager::with_default_dir()?;
    /// manager.discover_plugins()?;
    ///
    /// // Load specific plugin
    /// manager.load_plugin("port-banner")?;
    ///
    /// // Plugin is now ready to use
    /// if let Some(plugin) = manager.get_plugin("port-banner") {
    ///     println!("Loaded: {} v{}", plugin.name(), plugin.version());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`discover_plugins`](Self::discover_plugins) - Scan plugins directory
    /// - [`load_all`](Self::load_all) - Load all discovered plugins
    /// - [Plugin System Guide](../../../docs/30-PLUGIN-SYSTEM-GUIDE.md) - Plugin development guide
    pub fn load_plugin(&mut self, name: &str) -> CoreResult<()> {
        // Check if already loaded
        if self.loaded_plugins.contains_key(name) {
            debug!("Plugin {} already loaded", name);
            return Ok(());
        }

        // Get metadata
        let metadata = self
            .metadata_cache
            .get(name)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Plugin {} not found", name),
                )
            })?
            .clone();

        info!("Loading plugin: {} v{}", name, metadata.plugin.version);

        // Get plugin directory
        let plugin_dir = self.plugins_dir.join(name);
        let lua_path = plugin_dir.join("main.lua");

        if !lua_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("main.lua not found in {}", plugin_dir.display()),
            )
            .into());
        }

        // Read Lua script
        let lua_code = std::fs::read_to_string(&lua_path)?;

        // Create sandboxed Lua VM
        let lua = create_sandboxed_vm()
            .map_err(|e| std::io::Error::other(format!("Failed to create Lua VM: {}", e)))?;

        // Set resource limits
        set_resource_limits(&lua, &self.default_limits)
            .map_err(|e| std::io::Error::other(format!("Failed to set resource limits: {}", e)))?;

        // Register ProRT-IP API
        register_prtip_api(&lua)
            .map_err(|e| std::io::Error::other(format!("Failed to register API: {}", e)))?;

        // Create sandbox configuration from capabilities
        let capabilities = PluginCapabilities::from_strings(&metadata.plugin.capabilities)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid capabilities: {}", e),
                )
            })?;
        let context = LuaContext::new(capabilities);
        set_lua_context(&lua, context)
            .map_err(|e| std::io::Error::other(format!("Failed to set context: {}", e)))?;

        // Execute Lua code (loads functions into global scope)
        lua.load(&lua_code).exec().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to execute Lua code: {}", e),
            )
        })?;

        // Create plugin wrapper based on type
        let lua_arc = Arc::new(Mutex::new(lua));
        let base = LuaPlugin::new(
            metadata.plugin.name.clone(),
            metadata.plugin.version.clone(),
            metadata.plugin.plugin_type,
            lua_arc,
        );

        let mut plugin: Box<dyn Plugin> = match metadata.plugin.plugin_type {
            PluginType::Scan => Box::new(LuaScanPlugin::new(base)),
            PluginType::Output => Box::new(LuaOutputPlugin::new(base)),
            PluginType::Detection => Box::new(LuaDetectionPlugin::new(base)),
        };

        // Call on_load with plugin configuration
        // Serialize TOML config table to string for backward compatibility
        let config_str = if metadata.config.is_empty() {
            String::new()
        } else {
            toml::to_string(&metadata.config).map_err(|e| {
                std::io::Error::other(format!("Failed to serialize plugin config: {}", e))
            })?
        };
        plugin.on_load(&config_str)?;

        info!("Plugin {} loaded successfully", name);
        self.loaded_plugins.insert(name.to_string(), plugin);

        Ok(())
    }

    /// Unload plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> CoreResult<()> {
        if let Some(mut plugin) = self.loaded_plugins.remove(name) {
            info!("Unloading plugin: {}", name);
            plugin.on_unload()?;
            info!("Plugin {} unloaded successfully", name);
        } else {
            warn!("Plugin {} not loaded", name);
        }
        Ok(())
    }

    /// Get loaded plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.loaded_plugins.get(name).map(|p| &**p)
    }

    /// Get loaded plugin by name (mutable)
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut Box<dyn Plugin>> {
        self.loaded_plugins.get_mut(name)
    }

    /// List all discovered plugins
    pub fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.metadata_cache.values().collect()
    }

    /// List all loaded plugins
    pub fn list_loaded(&self) -> Vec<&str> {
        self.loaded_plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Load all discovered plugins
    pub fn load_all(&mut self) -> CoreResult<()> {
        let plugin_names: Vec<String> = self.metadata_cache.keys().cloned().collect();

        for name in plugin_names {
            if let Err(e) = self.load_plugin(&name) {
                error!("Failed to load plugin {}: {}", name, e);
                // Continue loading other plugins
            }
        }

        Ok(())
    }

    /// Unload all plugins
    pub fn unload_all(&mut self) -> CoreResult<()> {
        let plugin_names: Vec<String> = self.loaded_plugins.keys().cloned().collect();

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name) {
                error!("Failed to unload plugin {}: {}", name, e);
                // Continue unloading other plugins
            }
        }

        Ok(())
    }

    /// Get plugins directory path
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Set default resource limits for new plugins
    pub fn set_default_limits(&mut self, limits: ResourceLimits) {
        self.default_limits = limits;
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        // Unload all plugins on drop (best effort)
        let _ = self.unload_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_plugin(dir: &Path, name: &str, plugin_type: &str) -> std::io::Result<()> {
        let plugin_dir = dir.join(name);
        fs::create_dir_all(&plugin_dir)?;

        // Create plugin.toml
        let toml_content = format!(
            r#"
[plugin]
name = "{}"
version = "1.0.0"
author = "Test Author"
description = "Test plugin"
plugin_type = "{}"
"#,
            name, plugin_type
        );
        fs::write(plugin_dir.join("plugin.toml"), toml_content)?;

        // Create main.lua
        let lua_content = r#"
function on_load(config)
    prtip.log("info", "Test plugin loaded")
    return true
end

function on_unload()
    prtip.log("info", "Test plugin unloaded")
end
"#;
        fs::write(plugin_dir.join("main.lua"), lua_content)?;

        Ok(())
    }

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_discovery() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin", "detection").unwrap();

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        let count = manager.discover_plugins().unwrap();

        assert_eq!(count, 1);
        assert_eq!(manager.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_loading() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin", "detection").unwrap();

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().unwrap();

        assert!(manager.load_plugin("test-plugin").is_ok());
        assert_eq!(manager.list_loaded().len(), 1);
    }

    #[test]
    fn test_plugin_unloading() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin", "detection").unwrap();

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().unwrap();
        manager.load_plugin("test-plugin").unwrap();

        assert!(manager.unload_plugin("test-plugin").is_ok());
        assert_eq!(manager.list_loaded().len(), 0);
    }

    #[test]
    fn test_load_all_plugins() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "plugin1", "detection").unwrap();
        create_test_plugin(temp_dir.path(), "plugin2", "scan").unwrap();

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
        manager.discover_plugins().unwrap();
        manager.load_all().unwrap();

        assert_eq!(manager.list_loaded().len(), 2);
    }
}
