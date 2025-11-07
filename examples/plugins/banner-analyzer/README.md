# Banner Analyzer Plugin

Enhanced banner analysis plugin for ProRT-IP that identifies services, products, versions, and additional information from service banners.

## Overview

This detection plugin analyzes banners captured during port scanning to identify:
- Service type (HTTP, SSH, FTP, SMTP, MySQL, PostgreSQL, Redis, MongoDB)
- Product name and version (Apache, nginx, OpenSSH, etc.)
- Operating system type (Linux, Windows, FreeBSD)
- Confidence level (0.0-1.0)

## Supported Services

| Service | Products Detected | Version Extraction | OS Detection |
|---------|------------------|-------------------|--------------|
| **HTTP** | Apache, nginx, Microsoft IIS | ✓ | ✓ |
| **SSH** | OpenSSH | ✓ | ✓ |
| **FTP** | ProFTPD, vsftpd, Microsoft FTP | ✓ | ✓ |
| **SMTP** | Postfix, Sendmail, Microsoft Exchange | ✓ | ✓ |
| **MySQL** | MySQL | ✓ | - |
| **PostgreSQL** | PostgreSQL | ✓ | - |
| **Redis** | Redis | ✓ | - |
| **MongoDB** | MongoDB | ✓ | - |

## Installation

1. Copy the plugin directory to `~/.prtip/plugins/banner-analyzer/`
2. Verify files are present:
   - `plugin.toml` - Plugin metadata
   - `main.lua` - Plugin implementation
   - `README.md` - This documentation

## Usage

The plugin is automatically loaded when ProRT-IP starts. It analyzes banners captured during scanning and enhances service detection results.

### Manual Installation

```bash
mkdir -p ~/.prtip/plugins/banner-analyzer
cp plugin.toml main.lua README.md ~/.prtip/plugins/banner-analyzer/
```

### Verify Installation

```bash
prtip --list-plugins
# Should show: banner-analyzer v1.0.0 (detection)
```

## Examples

### HTTP Detection

```
Banner: "HTTP/1.1 200 OK\r\nServer: Apache/2.4.41 (Ubuntu)\r\n"
Result:
  service: http
  product: Apache
  version: 2.4.41
  os_type: Linux
  confidence: 0.95
```

### SSH Detection

```
Banner: "SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3"
Result:
  service: ssh
  product: OpenSSH
  version: 8.2p1
  os_type: Linux
  confidence: 0.95
```

### MySQL Detection

```
Banner: "5.7.33-0ubuntu0.18.04.1-log"
Result:
  service: mysql
  product: MySQL
  version: 5.7.33
  confidence: 0.9
```

## Technical Details

### Banner Analysis Process

1. **Normalization**: Convert banner to lowercase for case-insensitive matching
2. **Service Identification**: Match against known service patterns
3. **Product Detection**: Identify specific products (Apache, nginx, etc.)
4. **Version Extraction**: Extract version numbers using Lua patterns
5. **OS Detection**: Identify operating system from banner hints
6. **Confidence Scoring**: Assign confidence based on match quality

### Confidence Levels

- **0.95**: Exact product and version match
- **0.90**: Product match with partial information
- **0.85**: Product match without version
- **0.75**: Service type only
- **0.70**: Generic service identification

### API Functions

#### analyze_banner(banner)

Analyzes a service banner and returns ServiceInfo or nil.

**Parameters:**
- `banner` (string): The service banner text

**Returns:**
- ServiceInfo table with fields:
  - `service` (string, required): Service name
  - `product` (string, optional): Product name
  - `version` (string, optional): Version string
  - `info` (string, optional): Additional information
  - `os_type` (string, optional): Operating system
  - `confidence` (number): Confidence level (0.0-1.0)
- `nil` if service not recognized

#### probe_service(target)

Active service probing (not implemented in this plugin).

**Parameters:**
- `target` (table): Target information with `ip` and optional `hostname`

**Returns:**
- `nil` (this plugin uses passive analysis only)

## Limitations

- **Passive Analysis Only**: Does not actively probe services
- **Banner Dependent**: Requires service to send a banner
- **Pattern Matching**: May not detect all service variants
- **Version Accuracy**: Depends on banner format consistency

## Extending the Plugin

To add detection for new services:

1. Create an analyzer function:
```lua
local function analyze_myservice(banner)
    local lower = string.lower(banner)
    if string.match(lower, "myservice") then
        return {
            service = "myservice",
            product = "MyProduct",
            confidence = 0.9
        }
    end
    return nil
end
```

2. Add to the analyzer chain in `analyze_banner()`:
```lua
local result = analyze_http(banner)
    or analyze_ssh(banner)
    or analyze_myservice(banner)  -- Add here
    or ...
```

## Security Considerations

- **No Network Access**: Plugin requires no capabilities (passive analysis)
- **No File System**: Does not read/write files
- **No External Resources**: Self-contained logic only
- **Safe Lua Patterns**: Uses bounded regex patterns
- **Memory Efficient**: Minimal memory footprint

## Performance

- **Fast**: Pattern matching is O(n) where n is banner length
- **Low Memory**: ~1KB per banner analyzed
- **No I/O**: Pure computation, no blocking operations
- **Stateless**: Each banner analyzed independently

## Troubleshooting

### Plugin Not Loading

Check plugin directory path:
```bash
ls -la ~/.prtip/plugins/banner-analyzer/
# Should show: plugin.toml, main.lua, README.md
```

### No Detections

1. Verify banner capture is enabled
2. Check banner format matches expected patterns
3. Enable debug logging: `--log-level debug`
4. Review plugin logs in ProRT-IP output

### Low Confidence

- Banner may be truncated
- Service version format may differ
- Consider updating patterns in main.lua

## License

GPL-3.0 - Same as ProRT-IP

## Contributing

To contribute improvements:
1. Add new service patterns
2. Improve version extraction
3. Enhance OS detection
4. Submit pull request to ProRT-IP repository

## Version History

- **1.0.0** (2024): Initial release with 8 service types

## Author

ProRT-IP Contributors

## See Also

- ProRT-IP Plugin System Guide: `docs/30-PLUGIN-SYSTEM-GUIDE.md`
- Plugin API Reference: `crates/prtip-scanner/src/plugin/`
- Service Detection: `crates/prtip-scanner/src/service_detector.rs`
