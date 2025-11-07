//! Integration tests for the plugin system
//!
//! These tests verify that the plugin system works end-to-end:
//! - Plugin discovery
//! - Plugin metadata parsing
//! - Plugin loading (Lua VM creation)
//! - Plugin execution (Lua function calls)
//! - Example plugins (banner-analyzer, ssl-checker)

use prtip_scanner::PluginManager;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to copy example plugin to temp directory
fn copy_example_plugin(temp_dir: &TempDir, plugin_name: &str) -> std::io::Result<()> {
    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("plugins")
        .join(plugin_name);

    let dest_dir = temp_dir.path().join(plugin_name);
    fs::create_dir_all(&dest_dir)?;

    // Copy plugin.toml
    let toml_src = source_dir.join("plugin.toml");
    let toml_dest = dest_dir.join("plugin.toml");
    if toml_src.exists() {
        fs::copy(&toml_src, &toml_dest)?;
    }

    // Copy main.lua
    let lua_src = source_dir.join("main.lua");
    let lua_dest = dest_dir.join("main.lua");
    if lua_src.exists() {
        fs::copy(&lua_src, &lua_dest)?;
    }

    Ok(())
}

#[test]
fn test_banner_analyzer_plugin_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Copy banner-analyzer plugin to temp directory
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();

    // Create plugin manager and discover plugins
    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    let count = manager.discover_plugins().unwrap();

    assert_eq!(count, 1, "Should discover 1 plugin");

    let plugins = manager.list_plugins();
    assert_eq!(plugins.len(), 1);

    let plugin_meta = plugins[0];
    assert_eq!(plugin_meta.plugin.name, "banner-analyzer");
    assert_eq!(plugin_meta.plugin.version, "1.0.0");
    assert_eq!(plugin_meta.plugin.plugin_type.to_string(), "detection");
}

#[test]
fn test_ssl_checker_plugin_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Copy ssl-checker plugin to temp directory
    copy_example_plugin(&temp_dir, "ssl-checker").unwrap();

    // Create plugin manager and discover plugins
    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    let count = manager.discover_plugins().unwrap();

    assert_eq!(count, 1, "Should discover 1 plugin");

    let plugins = manager.list_plugins();
    assert_eq!(plugins.len(), 1);

    let plugin_meta = plugins[0];
    assert_eq!(plugin_meta.plugin.name, "ssl-checker");
    assert_eq!(plugin_meta.plugin.version, "1.0.0");
    assert_eq!(plugin_meta.plugin.plugin_type.to_string(), "detection");
    assert_eq!(plugin_meta.plugin.capabilities, vec!["network"]);
}

#[test]
fn test_multiple_plugins_discovery() {
    let temp_dir = TempDir::new().unwrap();

    // Copy both example plugins
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();
    copy_example_plugin(&temp_dir, "ssl-checker").unwrap();

    // Create plugin manager and discover plugins
    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    let count = manager.discover_plugins().unwrap();

    assert_eq!(count, 2, "Should discover 2 plugins");

    let plugins = manager.list_plugins();
    assert_eq!(plugins.len(), 2);

    // Verify both plugins were discovered
    let plugin_names: Vec<&str> = plugins.iter().map(|p| p.plugin.name.as_str()).collect();
    assert!(plugin_names.contains(&"banner-analyzer"));
    assert!(plugin_names.contains(&"ssl-checker"));
}

#[test]
fn test_banner_analyzer_plugin_loading() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    // Load the plugin
    let result = manager.load_plugin("banner-analyzer");
    assert!(result.is_ok(), "Plugin should load successfully");

    // Verify plugin is in loaded list
    let loaded = manager.list_loaded();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], "banner-analyzer");
}

#[test]
fn test_ssl_checker_plugin_loading() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "ssl-checker").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    // Load the plugin
    let result = manager.load_plugin("ssl-checker");
    assert!(result.is_ok(), "Plugin should load successfully");

    // Verify plugin is in loaded list
    let loaded = manager.list_loaded();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], "ssl-checker");
}

#[test]
fn test_load_all_example_plugins() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();
    copy_example_plugin(&temp_dir, "ssl-checker").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    // Load all plugins
    let result = manager.load_all();
    assert!(result.is_ok(), "Should load all plugins successfully");

    // Verify both plugins are loaded
    let loaded = manager.list_loaded();
    assert_eq!(loaded.len(), 2);
    assert!(loaded.contains(&"banner-analyzer"));
    assert!(loaded.contains(&"ssl-checker"));
}

#[test]
fn test_plugin_unload() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();
    manager.load_plugin("banner-analyzer").unwrap();

    // Unload the plugin
    let result = manager.unload_plugin("banner-analyzer");
    assert!(result.is_ok(), "Plugin should unload successfully");

    // Verify plugin is not in loaded list
    let loaded = manager.list_loaded();
    assert_eq!(loaded.len(), 0);
}

#[test]
fn test_plugin_reload() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    // Load, unload, and reload
    manager.load_plugin("banner-analyzer").unwrap();
    manager.unload_plugin("banner-analyzer").unwrap();
    let result = manager.load_plugin("banner-analyzer");

    assert!(result.is_ok(), "Plugin should reload successfully");
    let loaded = manager.list_loaded();
    assert_eq!(loaded.len(), 1);
}

#[test]
fn test_invalid_plugin_load() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    // Try to load non-existent plugin
    let result = manager.load_plugin("non-existent-plugin");
    assert!(result.is_err(), "Should fail to load non-existent plugin");
}

#[test]
fn test_duplicate_plugin_load() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "banner-analyzer").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    // Load plugin twice
    manager.load_plugin("banner-analyzer").unwrap();
    let result = manager.load_plugin("banner-analyzer");

    // Should succeed (already loaded, no-op)
    assert!(
        result.is_ok(),
        "Loading already-loaded plugin should succeed"
    );

    let loaded = manager.list_loaded();
    assert_eq!(loaded.len(), 1, "Should still have only 1 loaded instance");
}
