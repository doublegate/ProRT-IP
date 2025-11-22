//! Lua API integration for ProRT-IP plugins.
//!
//! This module exposes ProRT-IP functionality to Lua plugins through the `prtip` table.
//! All operations are subject to capability checks to ensure security.

use super::sandbox::{Capability, PluginCapabilities, ResourceLimits};
use mlua::{Lua, Result as LuaResult, Table, Value};
use parking_lot::Mutex;
use std::sync::Arc;
use toml;
use tracing::{debug, error, info, warn};

/// Create a sandboxed Lua VM with security restrictions
pub fn create_sandboxed_vm() -> LuaResult<Lua> {
    let lua = Lua::new();

    // Remove dangerous libraries for security
    lua.globals().set("io", Value::Nil)?;
    lua.globals().set("os", Value::Nil)?;
    lua.globals().set("debug", Value::Nil)?;

    // Remove loadlib from package (prevents loading native libraries)
    if let Ok(package) = lua.globals().get::<Table>("package") {
        package.set("loadlib", Value::Nil)?;
    }

    Ok(lua)
}

/// Set resource limits on Lua VM
pub fn set_resource_limits(lua: &Lua, limits: &ResourceLimits) -> LuaResult<()> {
    // Set memory limit
    lua.set_memory_limit(limits.max_memory_bytes)?;

    // Set instruction count hook (prevents infinite loops)
    let max_instructions = limits.max_instructions;
    let _ = lua.set_hook(
        mlua::HookTriggers {
            every_nth_instruction: Some(max_instructions as u32),
            ..Default::default()
        },
        move |_lua, _debug| {
            Err(mlua::Error::RuntimeError(format!(
                "Instruction limit of {} exceeded",
                max_instructions
            )))
        },
    );

    Ok(())
}

/// Lua context stored in registry for capability checking
#[derive(Clone)]
pub struct LuaContext {
    pub capabilities: Arc<Mutex<PluginCapabilities>>,
}

impl LuaContext {
    pub fn new(capabilities: PluginCapabilities) -> Self {
        Self {
            capabilities: Arc::new(Mutex::new(capabilities)),
        }
    }
}

// UserData implementation for LuaContext to store in Lua registry
impl mlua::UserData for LuaContext {}

/// Store LuaContext in Lua registry
pub fn set_lua_context(lua: &Lua, ctx: LuaContext) -> LuaResult<()> {
    lua.set_named_registry_value("prtip_context", lua.create_userdata(ctx)?)?;
    Ok(())
}

/// Retrieve LuaContext from Lua registry
pub fn get_lua_context(lua: &Lua) -> LuaResult<LuaContext> {
    let ctx_ud = lua.named_registry_value::<mlua::AnyUserData>("prtip_context")?;
    ctx_ud.borrow::<LuaContext>().map(|ctx| ctx.clone())
}

/// Check if plugin has required capability
pub fn check_capability(lua: &Lua, required: Capability) -> LuaResult<()> {
    let ctx = get_lua_context(lua)?;
    let caps = ctx.capabilities.lock();

    if !caps.has(required) {
        return Err(mlua::Error::RuntimeError(format!(
            "Plugin lacks '{}' capability",
            required
        )));
    }

    Ok(())
}

/// Register ProRT-IP API in Lua global scope
pub fn register_prtip_api(lua: &Lua) -> LuaResult<()> {
    let prtip_table = lua.create_table()?;

    // Logging functions
    prtip_table.set("log", lua.create_function(lua_log)?)?;

    // Target information (read-only, no capability required)
    prtip_table.set("get_target", lua.create_function(lua_get_target)?)?;

    // Scan configuration (read-only, no capability required)
    let scan_config = lua.create_table()?;
    scan_config.set("scan_type", "unknown")?; // Will be updated during execution
    scan_config.set("rate", 0)?;
    scan_config.set("timing", 3)?;
    scan_config.set("verbose", false)?;
    prtip_table.set("scan_config", scan_config)?;

    // Network operations (require Network capability)
    prtip_table.set("connect", lua.create_function(lua_connect)?)?;
    prtip_table.set("send", lua.create_function(lua_send)?)?;
    prtip_table.set("receive", lua.create_function(lua_receive)?)?;
    prtip_table.set("close", lua.create_function(lua_close)?)?;

    // Result manipulation (no capability required)
    prtip_table.set("add_result", lua.create_function(lua_add_result)?)?;

    // Register in global scope
    lua.globals().set("prtip", prtip_table)?;

    Ok(())
}

// Lua API function implementations

/// prtip.log(level, message)
fn lua_log(_lua: &Lua, (level, message): (String, String)) -> LuaResult<()> {
    match level.to_lowercase().as_str() {
        "debug" => debug!("[Plugin] {}", message),
        "info" => info!("[Plugin] {}", message),
        "warn" => warn!("[Plugin] {}", message),
        "error" => error!("[Plugin] {}", message),
        _ => info!("[Plugin] {}", message),
    }
    Ok(())
}

/// prtip.get_target() -> {ip, port, protocol}
fn lua_get_target(lua: &Lua, _args: ()) -> LuaResult<Table> {
    // Create empty target table (will be populated by plugin manager during execution)
    let target = lua.create_table()?;
    target.set("ip", "0.0.0.0")?;
    target.set("port", 0)?;
    target.set("protocol", "tcp")?;
    Ok(target)
}

/// prtip.connect(ip, port, timeout) -> socket_id
fn lua_connect(lua: &Lua, (ip, port, timeout): (String, u16, f64)) -> LuaResult<u64> {
    // Check capability
    check_capability(lua, Capability::Network)?;

    // Validate inputs
    if ip.is_empty() {
        return Err(mlua::Error::RuntimeError(
            "IP address cannot be empty".to_string(),
        ));
    }

    if timeout <= 0.0 || timeout > 60.0 {
        return Err(mlua::Error::RuntimeError(
            "Timeout must be between 0 and 60 seconds".to_string(),
        ));
    }

    // TODO: Implement actual network connection
    // For now, return a stub socket ID
    info!(
        "[Plugin] Connect to {}:{} with timeout {}s",
        ip, port, timeout
    );
    Ok(1) // Stub socket ID
}

/// prtip.send(socket_id, data) -> bytes_sent
fn lua_send(lua: &Lua, (socket_id, data): (u64, Vec<u8>)) -> LuaResult<usize> {
    // Check capability
    check_capability(lua, Capability::Network)?;

    // Validate inputs
    if data.is_empty() {
        return Err(mlua::Error::RuntimeError(
            "Data cannot be empty".to_string(),
        ));
    }

    // TODO: Implement actual network send
    info!("[Plugin] Send {} bytes to socket {}", data.len(), socket_id);
    Ok(data.len()) // Stub
}

/// prtip.receive(socket_id, max_bytes, timeout) -> data
fn lua_receive(
    lua: &Lua,
    (socket_id, max_bytes, timeout): (u64, usize, f64),
) -> LuaResult<Vec<u8>> {
    // Check capability
    check_capability(lua, Capability::Network)?;

    // Validate inputs
    if max_bytes == 0 || max_bytes > 65536 {
        return Err(mlua::Error::RuntimeError(
            "max_bytes must be between 1 and 65536".to_string(),
        ));
    }

    if timeout <= 0.0 || timeout > 60.0 {
        return Err(mlua::Error::RuntimeError(
            "Timeout must be between 0 and 60 seconds".to_string(),
        ));
    }

    // TODO: Implement actual network receive
    info!(
        "[Plugin] Receive up to {} bytes from socket {} with timeout {}s",
        max_bytes, socket_id, timeout
    );
    Ok(Vec::new()) // Stub
}

/// prtip.close(socket_id)
fn lua_close(lua: &Lua, socket_id: u64) -> LuaResult<()> {
    // Check capability
    check_capability(lua, Capability::Network)?;

    // TODO: Implement actual socket close
    info!("[Plugin] Close socket {}", socket_id);
    Ok(())
}

/// prtip.add_result(key, value)
fn lua_add_result(_lua: &Lua, (key, value): (String, Value)) -> LuaResult<()> {
    // No capability required for adding custom results

    // Validate key
    if key.is_empty() {
        return Err(mlua::Error::RuntimeError("Key cannot be empty".to_string()));
    }

    // TODO: Store result in scan context
    info!("[Plugin] Add result: {} = {:?}", key, value);
    Ok(())
}

/// Convert TOML table to Lua table
pub fn toml_to_lua(lua: &Lua, toml_table: &toml::Table) -> LuaResult<Table> {
    let lua_table = lua.create_table()?;

    for (key, value) in toml_table {
        let lua_value = toml_value_to_lua(lua, value)?;
        lua_table.set(key.clone(), lua_value)?;
    }

    Ok(lua_table)
}

/// Convert TOML value to Lua value recursively
fn toml_value_to_lua(lua: &Lua, value: &toml::Value) -> LuaResult<Value> {
    match value {
        toml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        toml::Value::Integer(i) => Ok(Value::Integer(*i)),
        toml::Value::Float(f) => Ok(Value::Number(*f)),
        toml::Value::Boolean(b) => Ok(Value::Boolean(*b)),
        toml::Value::Array(arr) => {
            let lua_table = lua.create_table()?;
            for (i, item) in arr.iter().enumerate() {
                let lua_value = toml_value_to_lua(lua, item)?;
                lua_table.set(i + 1, lua_value)?; // Lua arrays are 1-indexed
            }
            Ok(Value::Table(lua_table))
        }
        toml::Value::Table(table) => {
            let lua_table = lua.create_table()?;
            for (key, val) in table {
                let lua_value = toml_value_to_lua(lua, val)?;
                lua_table.set(key.clone(), lua_value)?;
            }
            Ok(Value::Table(lua_table))
        }
        toml::Value::Datetime(dt) => Ok(Value::String(lua.create_string(dt.to_string())?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sandboxed_vm() {
        let lua = create_sandboxed_vm().unwrap();

        // Verify dangerous libraries are removed
        assert!(lua.globals().get::<Value>("io").unwrap().is_nil());
        assert!(lua.globals().get::<Value>("os").unwrap().is_nil());
        assert!(lua.globals().get::<Value>("debug").unwrap().is_nil());

        // Verify safe libraries are present
        assert!(!lua.globals().get::<Value>("string").unwrap().is_nil());
        assert!(!lua.globals().get::<Value>("table").unwrap().is_nil());
        assert!(!lua.globals().get::<Value>("math").unwrap().is_nil());
    }

    #[test]
    fn test_register_prtip_api() {
        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        // Verify prtip table exists
        let prtip: Table = lua.globals().get("prtip").unwrap();

        // Verify functions exist
        assert!(!prtip.get::<Value>("log").unwrap().is_nil());
        assert!(!prtip.get::<Value>("get_target").unwrap().is_nil());
        assert!(!prtip.get::<Value>("connect").unwrap().is_nil());
        assert!(!prtip.get::<Value>("send").unwrap().is_nil());
        assert!(!prtip.get::<Value>("receive").unwrap().is_nil());
        assert!(!prtip.get::<Value>("close").unwrap().is_nil());

        // Verify scan_config table exists
        let scan_config: Table = prtip.get("scan_config").unwrap();
        assert!(!scan_config.is_empty());
    }

    #[test]
    fn test_lua_log() {
        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        // Test logging (should not panic)
        let result: LuaResult<()> = lua
            .load(
                r#"
            prtip.log("info", "Test message")
            prtip.log("debug", "Debug message")
            prtip.log("warn", "Warning message")
            prtip.log("error", "Error message")
        "#,
            )
            .exec();

        assert!(result.is_ok());
    }

    #[test]
    fn test_capability_checking() {
        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        // Set context with NO capabilities
        let ctx = LuaContext::new(PluginCapabilities::new());
        set_lua_context(&lua, ctx).unwrap();

        // Try to connect (should fail without Network capability)
        let result: LuaResult<()> = lua
            .load(
                r#"
            prtip.connect("192.168.1.1", 80, 1.0)
        "#,
            )
            .exec();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("lacks 'network' capability"));
    }

    #[test]
    fn test_capability_with_permission() {
        let lua = create_sandboxed_vm().unwrap();
        register_prtip_api(&lua).unwrap();

        // Set context WITH Network capability
        let mut caps = PluginCapabilities::new();
        caps.add(Capability::Network);
        let ctx = LuaContext::new(caps);
        set_lua_context(&lua, ctx).unwrap();

        // Try to connect (should succeed with Network capability)
        let result: LuaResult<()> = lua
            .load(
                r#"
            local socket_id = prtip.connect("192.168.1.1", 80, 1.0)
        "#,
            )
            .exec();

        assert!(result.is_ok());
    }

    #[test]
    fn test_set_resource_limits() {
        let lua = create_sandboxed_vm().unwrap();
        let limits = ResourceLimits::new()
            .with_max_memory_mb(50)
            .with_max_instructions(100_000);

        set_resource_limits(&lua, &limits).unwrap();

        // Memory limit is set (no way to verify directly via mlua API)
        // Instruction limit will trigger on infinite loop
        let result: LuaResult<()> = lua
            .load(
                r#"
            local i = 0
            while i < 1000000 do
                i = i + 1
            end
        "#,
            )
            .exec();

        // Should fail due to instruction limit
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Instruction limit"));
    }
}
