# Configuration

ProRT-IP supports multiple configuration methods: configuration files, environment variables, and scan templates. Command-line flags take precedence over all other configuration sources.

## Configuration Files

### Location Hierarchy

ProRT-IP searches for configuration files in the following order (highest to lowest priority):

1. `./prtip.toml` - Current directory (project-specific config)
2. `~/.config/prtip/config.toml` - User configuration
3. `/etc/prtip/config.toml` - System-wide configuration

**Creating User Configuration:**

```bash
mkdir -p ~/.config/prtip
nano ~/.config/prtip/config.toml
```

### Configuration Structure

**Complete Example:**

```toml
[scan]
default_scan_type = "syn"  # Default to TCP SYN scan (-sS)
default_ports = "1-1000"   # Scan top 1000 ports by default
timeout = 5000             # Connection timeout in milliseconds
max_retries = 3            # Maximum retry attempts per port

[timing]
template = "normal"        # Timing template (T3)
min_rate = 10             # Minimum packet rate (packets/sec)
max_rate = 1000           # Maximum packet rate (packets/sec)

[output]
default_format = "text"   # Default output format
colorize = true           # Enable colorized output
verbose = false           # Disable verbose mode by default

[performance]
numa = false              # NUMA optimization (Linux only)
batch_size = 1000         # Batch size for parallelism

[plugins]
enabled = true            # Enable plugin system
plugin_dir = "~/.prtip/plugins"  # Plugin directory location
```

### Configuration Sections

#### Scan Settings

Controls default scanning behavior:

```toml
[scan]
default_scan_type = "syn"      # syn|connect|udp|fin|null|xmas|ack|idle
default_ports = "1-1000"       # Port specification
timeout = 5000                 # Milliseconds
max_retries = 3                # Retry count
skip_host_discovery = false    # Skip ping (-Pn)
```

#### Timing Configuration

Controls scan speed and timing:

```toml
[timing]
template = "normal"            # paranoid|sneaky|polite|normal|aggressive|insane (T0-T5)
min_rate = 10                  # Minimum packets per second
max_rate = 1000                # Maximum packets per second
host_delay = 0                 # Milliseconds between probes
```

#### Output Settings

Controls output formatting and verbosity:

```toml
[output]
default_format = "text"        # text|json|xml|greppable
colorize = true                # Enable ANSI colors
verbose = false                # Verbose output (-v)
append_timestamp = true        # Append timestamp to filenames
```

#### Performance Tuning

Advanced performance options:

```toml
[performance]
numa = false                   # Enable NUMA optimization (Linux)
batch_size = 1000             # Parallelism batch size
max_concurrent = 10000        # Maximum concurrent connections
```

#### Plugin System

Plugin configuration:

```toml
[plugins]
enabled = true                 # Enable/disable plugins
plugin_dir = "~/.prtip/plugins"  # Plugin directory
auto_load = ["banner-grab", "http-headers"]  # Auto-load plugins
```

## Environment Variables

Environment variables provide runtime configuration without modifying files.

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `PRTIP_CONFIG` | Default configuration file | `~/.prtip/config.toml` | `/path/to/config.toml` |
| `PRTIP_DB` | Default database path | `~/.prtip/scans.db` | `/var/lib/prtip/scans.db` |
| `PRTIP_THREADS` | Number of worker threads | CPU cores | `8` |
| `PRTIP_LOG` | Log level | `info` | `debug`, `trace` |
| `PRTIP_DISABLE_HISTORY` | Disable command history | `false` | `true` |
| `PRTIP_PLUGIN_DIR` | Plugin directory | `~/.prtip/plugins` | `/usr/share/prtip/plugins` |
| `PRTIP_MAX_RATE` | Default max rate | from config | `1000` |

**Usage Examples:**

```bash
# Use custom configuration file
export PRTIP_CONFIG=~/my-config.toml
prtip -sS -p 80,443 192.168.1.1

# Enable debug logging
export PRTIP_LOG=debug
sudo -E prtip -sS -p 80,443 192.168.1.1

# Override thread count
export PRTIP_THREADS=4
prtip -sS -p 1-1000 192.168.1.0/24

# Disable command history (testing/automation)
export PRTIP_DISABLE_HISTORY=true
prtip -sS -F target.com
```

## Scan Templates

Scan templates provide pre-configured scanning scenarios with a single command.

### Built-in Templates

ProRT-IP includes 10 built-in templates:

**Web Server Scanning:**

```bash
prtip --template web-servers 192.168.1.0/24
# Equivalent to: -p 80,443,8080,8443 -sV --script http-*
```

**Database Discovery:**

```bash
prtip --template databases 192.168.1.1
# Equivalent to: -p 3306,5432,27017,6379,1521 -sV
```

**Quick Scan (Top 100 Ports):**

```bash
prtip --template quick 192.168.1.0/24
# Equivalent to: -F -T4 -sS
```

**Comprehensive Scan:**

```bash
prtip --template thorough 192.168.1.1
# Equivalent to: -p- -T3 -sV -O
```

**Stealth Scan (Evasion):**

```bash
prtip --template stealth 192.168.1.1
# Equivalent to: -sF -T0 -f -D RND:5
```

**SSL/TLS Analysis:**

```bash
prtip --template ssl-only 192.168.1.1
# Equivalent to: -p 443,8443,993,995,465,636,3389 -sV --tls-analysis
```

**Additional Templates:**

- `discovery` - Network discovery and host enumeration
- `admin-panels` - Common admin interface ports
- `mail-servers` - Email service discovery
- `file-shares` - SMB/NFS/FTP scanning

### Template Management

**List Available Templates:**

```bash
prtip --list-templates
```

**Show Template Details:**

```bash
prtip --show-template web-servers
```

**Override Template Values:**

```bash
# Override port range
prtip --template quick -p 1-10000

# Override timing
prtip --template stealth -T3

# Add additional flags
prtip --template web-servers -sV --version-intensity 9
```

### Custom Templates

Create custom templates in `~/.prtip/templates.toml`:

```toml
[templates.staging-web]
description = "Scan staging environment web servers"
ports = "80,443,3000,8080,8443"
scan_type = "syn"
timing = "T4"
service_detection = true

[templates.internal-audit]
description = "Internal network security audit"
ports = "1-65535"
scan_type = "syn"
timing = "T3"
service_detection = true
os_detection = true
evasion = ["fragment", "ttl=64"]

[templates.dns-servers]
description = "DNS server discovery and testing"
ports = "53,853,5353"
scan_type = "udp"
service_detection = true
```

**Using Custom Templates:**

```bash
prtip --template staging-web 10.0.0.0/24
prtip --template internal-audit 192.168.1.0/24
```

## Configuration Precedence

Configuration sources are applied in the following order (highest to lowest priority):

1. **Command-line flags** (highest priority)
2. **Environment variables**
3. **Project configuration** (`./prtip.toml`)
4. **User configuration** (`~/.config/prtip/config.toml`)
5. **System configuration** (`/etc/prtip/config.toml`)
6. **Built-in defaults** (lowest priority)

**Example Precedence:**

```toml
# System config (/etc/prtip/config.toml)
[scan]
default_ports = "1-1000"

# User config (~/.config/prtip/config.toml)
[scan]
default_ports = "1-10000"

# Command-line
prtip -p 80,443 target.com
# Result: Scans ports 80 and 443 (command-line wins)
```

## Platform-Specific Configuration

### Linux Configuration

**NUMA Optimization:**

```toml
[performance]
numa = true                    # Enable NUMA optimization
numa_nodes = [0, 1]           # Specific NUMA nodes
```

**Capabilities (No sudo):**

```bash
# Set capabilities (one-time setup)
sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip

# Configure for non-root use
[scan]
drop_privileges = true
user = "scanner"
group = "scanner"
```

### Windows Configuration

**Npcap Settings:**

```toml
[windows]
npcap_path = "C:\\Program Files\\Npcap"
loopback_support = true
```

### macOS Configuration

**BPF Device Access:**

```toml
[macos]
bpf_buffer_size = 4194304     # 4MB buffer
bpf_devices = ["/dev/bpf0", "/dev/bpf1"]
```

## Configuration Validation

**Validate Configuration:**

```bash
prtip --validate-config
prtip --validate-config --config ~/custom.toml
```

**Example Output:**

```
✓ Configuration valid
✓ All plugins loadable
✓ Database path writable
⚠ Warning: NUMA enabled but system has only 1 NUMA node
✓ Timing values within acceptable ranges
```

## See Also

- [Basic Usage](./basic-usage.md) - Getting started with ProRT-IP
- [CLI Reference](./cli-reference.md) - Complete command-line reference
- [Scan Templates](./scan-templates.md) - Template system documentation
- [Performance Tuning](../advanced/performance-tuning.md) - Optimization guide
- [Environment Variables](./cli-reference.md#environment-variables) - Complete variable reference
