# ProRT-IP Plugin System Guide

**Version**: 1.0.0 | **Last Updated**: 2024-11-06 | **Status**: Production

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Architecture](#architecture)
4. [Plugin Types](#plugin-types)
5. [Plugin Structure](#plugin-structure)
6. [API Reference](#api-reference)
7. [Security Model](#security-model)
8. [Development Guide](#development-guide)
9. [Example Plugins](#example-plugins)
10. [Testing Plugins](#testing-plugins)
11. [Deployment](#deployment)
12. [Troubleshooting](#troubleshooting)
13. [Best Practices](#best-practices)
14. [Future Extensions](#future-extensions)

---

## Overview

The ProRT-IP plugin system enables extensibility through Lua scripting, allowing users to customize scanning behavior, add detection capabilities, and create custom output formats without modifying core code.

### Key Features

- **Sandboxed Execution**: Lua plugins run in isolated environments with resource limits
- **Capabilities-Based Security**: Fine-grained permission model (Network, Filesystem, System, Database)
- **Three Plugin Types**: Scan lifecycle hooks, Output formatting, Service detection
- **Zero Native Dependencies**: Pure Lua implementation (no C libraries)
- **Hot Reloading**: Load/unload plugins without restarting ProRT-IP
- **Example Plugins**: `banner-analyzer` and `ssl-checker` included

### Design Goals

1. **Security First**: Deny-by-default capabilities, resource limits, sandboxing
2. **Simple API**: Easy to learn, hard to misuse
3. **Performance**: Minimal overhead, async-compatible
4. **Maintainability**: Clear interfaces, comprehensive documentation

---

## Quick Start

### Installation

1. Create plugin directory:
```bash
mkdir -p ~/.prtip/plugins/my-plugin
cd ~/.prtip/plugins/my-plugin
```

2. Create `plugin.toml`:
```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
author = "Your Name"
description = "My first ProRT-IP plugin"
plugin_type = "detection"
capabilities = []
```

3. Create `main.lua`:
```lua
function on_load(config)
    prtip.log("info", "Plugin loaded")
    return true
end

function on_unload()
    prtip.log("info", "Plugin unloaded")
end

function analyze_banner(banner)
    if string.match(banner, "HTTP") then
        return {
            service = "http",
            confidence = 0.8
        }
    end
    return nil
end
```

4. Test the plugin:
```bash
prtip --list-plugins
# Should show: my-plugin v1.0.0 (detection)
```

---

## Architecture

### System Components

```
┌─────────────────────────────────────┐
│       ProRT-IP Core Scanner         │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│        Plugin Manager               │
│  - Discovery                        │
│  - Loading/Unloading                │
│  - Lifecycle Management             │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│      Plugin API Layer               │
│  - ScanPlugin                       │
│  - OutputPlugin                     │
│  - DetectionPlugin                  │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│    Sandboxed Lua VM (mlua)          │
│  - Resource Limits                  │
│  - Capability Checks                │
│  - API Bindings (prtip table)       │
└─────────────────────────────────────┘
```

### Data Flow

1. **Discovery**: Scan `~/.prtip/plugins/` for `plugin.toml` files
2. **Metadata Parsing**: Validate plugin metadata (name, version, type, capabilities)
3. **VM Creation**: Create sandboxed Lua VM with resource limits
4. **API Registration**: Expose `prtip.*` functions to Lua
5. **Plugin Loading**: Load `main.lua` and call `on_load()`
6. **Execution**: Call plugin hooks during scanning (`analyze_banner`, `on_target`, etc.)
7. **Cleanup**: Call `on_unload()` and destroy VM

---

## Plugin Types

### 1. ScanPlugin

Provides lifecycle hooks for scan execution.

**Use Cases**:
- Pre-scan target manipulation (port knocking, custom filtering)
- Per-target custom data collection
- Post-scan aggregate analysis

**API Methods**:
```lua
function on_load(config)
function on_unload()
function pre_scan(targets)    -- Called before scan starts
function on_target(target, result)  -- Called for each target
function post_scan(results)   -- Called after scan completes
```

**Example**:
```lua
function pre_scan(targets)
    prtip.log("info", string.format("Scanning %d targets", #targets))
end

function on_target(target, result)
    if result.state == "open" then
        prtip.log("info", string.format("Found open port: %d", result.port))
    end
end

function post_scan(results)
    prtip.log("info", string.format("Scan complete: %d results", #results))
end
```

### 2. OutputPlugin

Custom result formatting and export.

**Use Cases**:
- Custom report formats (CSV, JSON, XML)
- Integration with external systems
- Data transformation

**API Methods**:
```lua
function on_load(config)
function on_unload()
function format_result(result)  -- Format single result
function export(results, path)  -- Export all results to file
```

**Example**:
```lua
function format_result(result)
    return string.format("%s:%d [%s]",
        result.target_ip,
        result.port,
        result.state)
end

function export(results, path)
    local file = io.open(path, "w")
    for _, result in ipairs(results) do
        file:write(format_result(result) .. "\n")
    end
    file:close()
end
```

### 3. DetectionPlugin

Enhanced service detection.

**Use Cases**:
- Banner analysis
- Active service probing
- Custom detection logic

**API Methods**:
```lua
function on_load(config)
function on_unload()
function analyze_banner(banner)     -- Passive analysis
function probe_service(target)      -- Active probing (requires Network capability)
```

**Return Format**:
```lua
return {
    service = "http",         -- Required: service name
    product = "Apache",       -- Optional: product name
    version = "2.4.41",       -- Optional: version string
    info = "Ubuntu",          -- Optional: additional info
    os_type = "Linux",        -- Optional: OS type
    confidence = 0.95         -- Optional: confidence (0.0-1.0, default 0.5)
}
```

---

## Plugin Structure

### Directory Layout

```
~/.prtip/plugins/my-plugin/
├── plugin.toml    # Required: Plugin metadata
├── main.lua       # Required: Plugin implementation
└── README.md      # Recommended: Documentation
```

### plugin.toml

Complete metadata specification:

```toml
[plugin]
name = "my-plugin"                # Required: Plugin identifier
version = "1.0.0"                 # Required: Semantic version
author = "Your Name"              # Required: Author name/email
description = "Plugin description" # Required: Short description
license = "GPL-3.0"               # Optional: License (default GPL-3.0)
plugin_type = "detection"         # Required: scan/output/detection
capabilities = ["network"]        # Optional: Required capabilities

[plugin.dependencies]
min_prtip_version = "0.4.0"       # Optional: Minimum ProRT-IP version
lua_version = "5.4"               # Optional: Lua version

[plugin.metadata]
tags = ["detection", "banner"]    # Optional: Search tags
category = "detection"            # Optional: Category
homepage = "https://example.com"  # Optional: Plugin homepage
repository = "https://github.com/..." # Optional: Source repository
```

### main.lua

Required functions:

```lua
-- Lifecycle (all plugin types)
function on_load(config)
    -- Initialize plugin
    -- Return true on success, false or error message on failure
    return true
end

function on_unload()
    -- Cleanup resources
    -- Errors are logged but not fatal
end

-- Type-specific functions (implement based on plugin_type)

-- ScanPlugin
function pre_scan(targets) end
function on_target(target, result) end
function post_scan(results) end

-- OutputPlugin
function format_result(result) return string end
function export(results, path) end

-- DetectionPlugin
function analyze_banner(banner) return service_info or nil end
function probe_service(target) return service_info or nil end
```

---

## API Reference

### Global `prtip` Table

All ProRT-IP functions are exposed through the `prtip` global table.

### Logging

```lua
prtip.log(level, message)
```

**Parameters**:
- `level` (string): "debug", "info", "warn", "error"
- `message` (string): Log message

**Example**:
```lua
prtip.log("info", "Plugin initialized successfully")
prtip.log("warn", "Unexpected banner format")
prtip.log("error", "Failed to connect to target")
```

### Target Information

```lua
target = prtip.get_target()
```

**Returns**:
- `target` (table): Target information
  - `ip` (string): IP address
  - `port` (number): Port number
  - `protocol` (string): "tcp" or "udp"

**Example**:
```lua
local target = prtip.get_target()
prtip.log("info", string.format("Scanning %s:%d", target.ip, target.port))
```

### Scan Configuration

```lua
config = prtip.scan_config
```

**Fields**:
- `scan_type` (string): Scan type ("syn", "connect", etc.)
- `rate` (number): Scan rate (packets/sec)
- `timing` (number): Timing template (0-5)
- `verbose` (boolean): Verbose output enabled

**Example**:
```lua
if prtip.scan_config.verbose then
    prtip.log("debug", "Verbose mode enabled")
end
```

### Network Operations

**Note**: Requires `network` capability.

#### Connect

```lua
socket_id = prtip.connect(ip, port, timeout)
```

**Parameters**:
- `ip` (string): Target IP address
- `port` (number): Target port (1-65535)
- `timeout` (number): Connection timeout in seconds (0-60)

**Returns**:
- `socket_id` (number): Socket identifier, or error

**Example**:
```lua
local socket_id = prtip.connect("192.168.1.1", 80, 5.0)
if socket_id then
    prtip.log("info", "Connected successfully")
end
```

#### Send

```lua
bytes_sent = prtip.send(socket_id, data)
```

**Parameters**:
- `socket_id` (number): Socket identifier from `prtip.connect()`
- `data` (string or table of bytes): Data to send

**Returns**:
- `bytes_sent` (number): Number of bytes sent

**Example**:
```lua
local bytes = prtip.send(socket_id, "GET / HTTP/1.0\r\n\r\n")
prtip.log("debug", string.format("Sent %d bytes", bytes))
```

#### Receive

```lua
data = prtip.receive(socket_id, max_bytes, timeout)
```

**Parameters**:
- `socket_id` (number): Socket identifier
- `max_bytes` (number): Maximum bytes to read (1-65536)
- `timeout` (number): Read timeout in seconds (0-60)

**Returns**:
- `data` (table of bytes): Received data

**Example**:
```lua
local data = prtip.receive(socket_id, 4096, 5.0)
local response = table.concat(data)
prtip.log("info", string.format("Received %d bytes", #data))
```

#### Close

```lua
prtip.close(socket_id)
```

**Parameters**:
- `socket_id` (number): Socket identifier

**Example**:
```lua
prtip.close(socket_id)
prtip.log("debug", "Socket closed")
```

### Result Manipulation

```lua
prtip.add_result(key, value)
```

**Parameters**:
- `key` (string): Result key
- `value` (any): Result value (string, number, boolean, table)

**Example**:
```lua
prtip.add_result("custom_field", "custom_value")
prtip.add_result("banner_length", #banner)
prtip.add_result("detected_features", {"ssl", "compression"})
```

---

## Security Model

### Capabilities

Fine-grained permission system based on deny-by-default principle.

#### Available Capabilities

| Capability | Description | Risk Level |
|------------|-------------|------------|
| `network` | Network connections | Medium |
| `filesystem` | File I/O operations | High |
| `system` | System commands | Critical |
| `database` | Database access | Medium |

#### Requesting Capabilities

In `plugin.toml`:
```toml
capabilities = ["network", "filesystem"]
```

#### Runtime Checks

Capabilities are checked before each privileged operation:

```lua
-- This will fail if 'network' capability not granted
local socket_id = prtip.connect(ip, port, timeout)
-- Error: "Plugin lacks 'network' capability"
```

### Resource Limits

Plugins are constrained by default limits to prevent DoS attacks.

#### Default Limits

| Resource | Limit | Configurable |
|----------|-------|--------------|
| Memory | 100 MB | Yes |
| CPU Time | 5 seconds | Yes |
| Instructions | 1,000,000 | Yes |

#### Enforcement

- **Memory**: Enforced by Lua VM
- **CPU Time**: Wall-clock timeout
- **Instructions**: Hook-based counting

**Example Violation**:
```lua
-- This will trigger instruction limit
while true do
    -- Infinite loop
end
-- Error: "Instruction limit of 1000000 exceeded"
```

### Sandboxing

Dangerous Lua libraries are removed from the VM environment.

#### Removed Libraries

- `io` - File I/O
- `os` - Operating system functions
- `debug` - Debug introspection
- `package.loadlib` - Native library loading

#### Safe Libraries

- `string` - String manipulation
- `table` - Table operations
- `math` - Mathematical functions
- `prtip` - ProRT-IP API

**Example**:
```lua
-- This will fail (io library removed)
local file = io.open("file.txt", "r")
-- Error: attempt to index nil value 'io'

-- This is allowed (string library present)
local upper = string.upper("hello")
```

---

## Development Guide

### Step 1: Plan Your Plugin

1. **Identify the Problem**: What functionality does ProRT-IP lack?
2. **Choose Plugin Type**: Scan, Output, or Detection?
3. **List Required Capabilities**: Network, Filesystem, etc.
4. **Design the API**: What functions will you implement?

### Step 2: Create Plugin Structure

```bash
mkdir -p ~/.prtip/plugins/my-plugin
cd ~/.prtip/plugins/my-plugin
touch plugin.toml main.lua README.md
```

### Step 3: Write plugin.toml

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
author = "Your Name <your.email@example.com>"
description = "One-line description"
plugin_type = "detection"
capabilities = []  # Add as needed

[plugin.dependencies]
min_prtip_version = "0.4.0"
lua_version = "5.4"

[plugin.metadata]
tags = ["detection", "custom"]
category = "detection"
```

### Step 4: Implement main.lua

Start with the lifecycle functions:

```lua
function on_load(config)
    prtip.log("info", "my-plugin loaded")
    -- Initialize state
    return true
end

function on_unload()
    prtip.log("info", "my-plugin unloaded")
    -- Cleanup state
end
```

Add type-specific functions based on your plugin type.

### Step 5: Test Your Plugin

```bash
# List plugins
prtip --list-plugins

# Test with real scan
prtip -sS -p 80 127.0.0.1 --plugin my-plugin

# Check logs
tail -f ~/.prtip/logs/prtip.log
```

### Step 6: Write README.md

Include:
- Overview
- Installation instructions
- Usage examples
- API reference
- Troubleshooting

### Step 7: Share Your Plugin

Consider submitting to the ProRT-IP plugin repository:
```bash
git clone https://github.com/doublegate/ProRT-IP/tree/main/examples/plugins
cd ProRT-IP-plugins
cp -r ~/.prtip/plugins/my-plugin plugins/
git add plugins/my-plugin
git commit -m "Add my-plugin"
git push
```

---

## Example Plugins

### Banner Analyzer

**Purpose**: Enhanced banner analysis for common services.

**Location**: `examples/plugins/banner-analyzer/`

**Key Features**:
- Detects HTTP, SSH, FTP, SMTP, MySQL, PostgreSQL, Redis, MongoDB
- Extracts product name, version, and OS type
- Confidence scoring (0.7-0.95)
- Zero capabilities required (passive analysis)

**Usage**:
```bash
prtip -sS -p 80,443,22 192.168.1.0/24 --plugin banner-analyzer
```

**Code Snippet**:
```lua
function analyze_http(banner)
    local lower = string.lower(banner)
    if string.match(lower, "apache") then
        local version = extract_version(banner, "Apache/([%d%.]+)")
        return {
            service = "http",
            product = "Apache",
            version = version,
            confidence = version and 0.95 or 0.85
        }
    end
    return nil
end
```

### SSL Checker

**Purpose**: SSL/TLS service detection and analysis.

**Location**: `examples/plugins/ssl-checker/`

**Key Features**:
- Identifies SSL/TLS ports (443, 465, 993, 995, etc.)
- Detects TLS protocol signatures
- Network capability utilization (active probing)
- Extensible for certificate analysis

**Usage**:
```bash
prtip -sS -p 443,8443 target.com --plugin ssl-checker
```

**Code Snippet**:
```lua
function analyze_banner(banner)
    local lower = string.lower(banner)
    if string.match(lower, "tls") or string.match(lower, "ssl") then
        return {
            service = "ssl",
            info = "TLS/SSL encrypted service",
            confidence = 0.7
        }
    end
    return nil
end
```

---

## Testing Plugins

### Unit Testing Lua Code

Create a test file `test_my_plugin.lua`:

```lua
package.path = package.path .. ";./?.lua"
local my_plugin = require("main")

function test_analyze_banner()
    local result = my_plugin.analyze_banner("HTTP/1.1 200 OK\r\nServer: Apache\r\n")
    assert(result ~= nil, "Should detect HTTP")
    assert(result.service == "http", "Should identify as HTTP")
    assert(result.confidence > 0.5, "Should have reasonable confidence")
    print("✓ test_analyze_banner passed")
end

test_analyze_banner()
print("All tests passed!")
```

Run with Lua:
```bash
lua test_my_plugin.lua
```

### Integration Testing

Use ProRT-IP's test framework:

```rust
#[test]
fn test_my_plugin_loading() {
    let temp_dir = TempDir::new().unwrap();
    copy_example_plugin(&temp_dir, "my-plugin").unwrap();

    let mut manager = PluginManager::new(temp_dir.path().to_path_buf());
    manager.discover_plugins().unwrap();

    let result = manager.load_plugin("my-plugin");
    assert!(result.is_ok(), "Plugin should load successfully");
}
```

### Manual Testing

1. **Load Test**: Verify plugin loads without errors
2. **Functionality Test**: Verify each function works correctly
3. **Error Handling Test**: Trigger error conditions
4. **Performance Test**: Measure execution time
5. **Security Test**: Verify capability enforcement

---

## Deployment

### Installation Methods

#### Method 1: Manual Copy

```bash
cp -r my-plugin ~/.prtip/plugins/
prtip --list-plugins  # Verify installation
```

#### Method 2: Git Clone

```bash
cd ~/.prtip/plugins
git clone https://github.com/username/my-plugin.git
prtip --list-plugins
```

#### Method 3: Package Manager (Future)

```bash
prtip plugin install my-plugin
prtip plugin update my-plugin
prtip plugin remove my-plugin
```

### System-Wide Deployment

For multi-user systems:

```bash
# System-wide location (requires root)
sudo cp -r my-plugin /opt/prtip/plugins/

# Update ProRT-IP config
sudo tee -a /etc/prtip/config.toml << EOF
[plugins]
system_path = "/opt/prtip/plugins"
user_path = "~/.prtip/plugins"
EOF
```

---

## Troubleshooting

### Plugin Not Loading

**Symptom**: Plugin doesn't appear in `--list-plugins`

**Diagnosis**:
1. Check file locations:
   ```bash
   ls -la ~/.prtip/plugins/my-plugin/
   # Should show: plugin.toml, main.lua
   ```
2. Verify `plugin.toml` is valid TOML:
   ```bash
   cat ~/.prtip/plugins/my-plugin/plugin.toml
   ```
3. Check ProRT-IP logs:
   ```bash
   prtip --log-level debug --list-plugins
   ```

**Solutions**:
- Fix TOML syntax errors
- Ensure required fields (name, version, author) are present
- Verify directory name matches plugin name

### Capability Errors

**Symptom**: "Plugin lacks 'network' capability"

**Diagnosis**:
Plugin requires capability not granted in `plugin.toml`.

**Solution**:
Add required capability:
```toml
capabilities = ["network"]
```

### Resource Limit Exceeded

**Symptom**: "Instruction limit exceeded" or "Memory limit exceeded"

**Diagnosis**:
Plugin is too resource-intensive.

**Solutions**:
1. Optimize Lua code (reduce loops, reuse tables)
2. Request increased limits (contact ProRT-IP maintainers)
3. Break processing into smaller chunks

### Lua Syntax Errors

**Symptom**: "Failed to execute Lua code"

**Diagnosis**:
Syntax error in `main.lua`.

**Solution**:
Test Lua syntax:
```bash
lua -l main.lua
```

Fix reported errors.

---

## Best Practices

### Security

1. **Minimize Capabilities**: Only request what you need
2. **Validate Input**: Never trust banner/target data
3. **Handle Errors**: Use `pcall()` for unsafe operations
4. **Avoid Secrets**: Don't hardcode credentials
5. **Log Securely**: Sanitize sensitive data in logs

### Performance

1. **Avoid Global State**: Use local variables
2. **Reuse Tables**: Don't create tables in loops
3. **Cache Results**: Store frequently accessed data
4. **Lazy Loading**: Defer expensive operations
5. **Profile Code**: Measure execution time

### Maintainability

1. **Document Functions**: Use comments liberally
2. **Follow Conventions**: Use ProRT-IP naming
3. **Version Carefully**: Use semantic versioning
4. **Test Thoroughly**: Cover edge cases
5. **Keep Simple**: KISS principle

### Example: Optimized Banner Analysis

**Bad**:
```lua
function analyze_banner(banner)
    for i = 1, #services do
        if string.match(banner, services[i].pattern) then
            return create_service_info(services[i])
        end
    end
    return nil
end
```

**Good**:
```lua
-- Cache pattern table (created once)
local patterns = build_pattern_table()

function analyze_banner(banner)
    local lower = string.lower(banner)
    -- Quick rejection for most cases
    if #lower < 3 then return nil end

    -- Ordered by frequency (HTTP most common)
    return analyze_http(lower)
        or analyze_ssh(lower)
        or analyze_ftp(lower)
end
```

---

## Future Extensions

### Planned API Additions

#### TLS/SSL API

```lua
-- TLS connection
socket_id = prtip.connect_tls(ip, port, timeout, {
    min_version = "TLS1.2",
    verify_cert = true,
    sni_hostname = "example.com"
})

-- Certificate extraction
cert_info = prtip.get_certificate(socket_id)
-- Returns: subject, issuer, validity, key size, etc.
```

#### HTTP API

```lua
-- HTTP request
response = prtip.http_request({
    url = "http://example.com",
    method = "GET",
    headers = {["User-Agent"] = "ProRT-IP"},
    timeout = 5.0
})
```

#### Database API

```lua
-- Query results database
results = prtip.query("SELECT * FROM results WHERE port = 80")
```

#### Filesystem API (with capability)

```lua
-- Read file
content = prtip.read_file("/path/to/file.txt")

-- Write file
prtip.write_file("/path/to/output.txt", content)
```

### Community Contributions

Want to contribute? Here's how:

1. **Plugin Ideas**: Share at https://github.com/doublegate/ProRT-IP/discussions
2. **API Requests**: Open feature request with use case
3. **Plugin Repository**: Submit to https://github.com/doublegate/ProRT-IP/tree/main/examples/plugins
4. **Documentation**: Improve this guide via PR

---

## Appendix

### Complete Example: HTTP Version Detector

**plugin.toml**:
```toml
[plugin]
name = "http-version-detector"
version = "1.0.0"
author = "Example Author"
description = "Detects HTTP server versions"
plugin_type = "detection"
capabilities = []
```

**main.lua**:
```lua
function on_load(config)
    prtip.log("info", "HTTP Version Detector loaded")
    return true
end

function on_unload()
    prtip.log("info", "HTTP Version Detector unloaded")
end

local function extract_version(text, pattern)
    return string.match(text, pattern)
end

function analyze_banner(banner)
    local lower = string.lower(banner)

    if string.match(lower, "^http/") then
        local http_version = extract_version(banner, "HTTP/([%d%.]+)")

        if string.match(lower, "apache") then
            local apache_version = extract_version(banner, "Apache/([%d%.]+)")
            return {
                service = "http",
                product = "Apache",
                version = apache_version,
                info = "HTTP/" .. (http_version or "1.1"),
                confidence = apache_version and 0.95 or 0.85
            }
        elseif string.match(lower, "nginx") then
            local nginx_version = extract_version(banner, "nginx/([%d%.]+)")
            return {
                service = "http",
                product = "nginx",
                version = nginx_version,
                info = "HTTP/" .. (http_version or "1.1"),
                confidence = nginx_version and 0.95 or 0.85
            }
        else
            return {
                service = "http",
                version = http_version,
                confidence = 0.7
            }
        end
    end

    return nil
end

function probe_service(target)
    -- Passive plugin, no active probing
    return nil
end
```

**README.md**:
```markdown
# HTTP Version Detector

Detects HTTP servers and versions from banners.

## Installation

cp -r http-version-detector ~/.prtip/plugins/

## Usage

prtip -sS -p 80,443,8080 target.com --plugin http-version-detector

## Supported Servers

- Apache
- nginx
- Generic HTTP servers

## License

GPL-3.0
```

---

## References

- **ProRT-IP Core**: https://github.com/doublegate/ProRT-IP
- **Lua 5.4 Manual**: https://www.lua.org/manual/5.4/
- **mlua Documentation**: https://docs.rs/mlua/latest/mlua/
- **Plugin Examples**: `examples/plugins/`
- **API Source**: `crates/prtip-scanner/src/plugin/`

---

**End of Plugin System Guide**

For support, see:
- **Issues**: https://github.com/doublegate/ProRT-IP/issues
- **Discussions**: https://github.com/doublegate/ProRT-IP/discussions
