# ProRT-IP Docker Images

This directory contains Docker configurations for running ProRT-IP WarScan in containers.

## Quick Start

### Pull from Docker Hub (when available)

```bash
docker pull doublegate/prtip:latest
docker run --rm --net=host --cap-add=NET_RAW doublegate/prtip -sS -p 80,443 192.168.1.1
```

### Build Locally

```bash
# Build from project root
cd /path/to/ProRT-IP
docker build -t prtip:latest -f docker/Dockerfile .

# Or use docker-compose
cd docker
docker-compose build
```

## Usage Examples

### Basic Scan

```bash
# SYN scan (requires NET_RAW capability)
docker run --rm --net=host --cap-add=NET_RAW prtip -sS -p 80,443 192.168.1.1

# TCP Connect scan (no special capabilities needed)
docker run --rm prtip -sT -p 80,443 host.docker.internal

# Fast scan
docker run --rm --net=host --cap-add=NET_RAW prtip -F 192.168.1.0/24
```

### Service Detection

```bash
docker run --rm --net=host --cap-add=NET_RAW prtip -sV -p 22,80,443 target.com
```

### TUI Dashboard

```bash
# Interactive TUI requires -it flags
docker run -it --rm --net=host --cap-add=NET_RAW prtip --tui -sS -p 1-1000 192.168.1.1
```

### Save Results

```bash
# Mount volume for persistent results
docker run --rm --net=host --cap-add=NET_RAW \
  -v $(pwd)/results:/home/prtip/results \
  prtip -sS -oX /home/prtip/results/scan.xml 192.168.1.1
```

### Using docker-compose

```bash
cd docker

# Run a scan
docker-compose run prtip -sS -p 80,443 192.168.1.1

# Interactive TUI
docker-compose run -it prtip --tui

# View logs
docker-compose logs -f prtip
```

## Image Variants

| Image | Base | Size | Use Case |
|-------|------|------|----------|
| `prtip:latest` | Debian Slim | ~50MB | Production (recommended) |
| `prtip:alpine` | Alpine 3.19 | ~25MB | Minimal footprint |

## Network Modes

### Host Network (Recommended for Scanning)

```bash
docker run --net=host --cap-add=NET_RAW prtip ...
```

- Direct access to host network interfaces
- Required for SYN and other raw packet scans
- Best performance

### Bridge Network

```bash
docker run --cap-add=NET_RAW prtip -sT ...
```

- Isolated network namespace
- Can scan other Docker containers
- Limited to TCP Connect scans

### Custom Network

```bash
docker network create scan-net
docker run --net=scan-net --cap-add=NET_RAW prtip ...
```

- Isolated testing environment
- Good for CI/CD

## Capabilities

ProRT-IP requires specific Linux capabilities for raw packet operations:

| Capability | Purpose |
|------------|---------|
| `NET_RAW` | Create raw sockets for SYN/UDP/ICMP scans |
| `NET_ADMIN` | Network configuration (optional, for advanced features) |

```bash
docker run --cap-add=NET_RAW --cap-add=NET_ADMIN prtip ...
```

## Security Considerations

1. **Run as non-root when possible**: The container runs as user `prtip` by default
2. **Use read-only filesystem**: `--read-only` flag for enhanced security
3. **Limit resources**: Use `--memory` and `--cpus` flags
4. **No new privileges**: `--security-opt=no-new-privileges:true`

### Secure Example

```bash
docker run --rm \
  --net=host \
  --cap-add=NET_RAW \
  --read-only \
  --memory=1g \
  --cpus=2 \
  --security-opt=no-new-privileges:true \
  prtip -sS -p 80,443 target.com
```

## Building for Different Architectures

```bash
# Build for ARM64
docker buildx build --platform linux/arm64 -t prtip:arm64 .

# Multi-arch build
docker buildx build --platform linux/amd64,linux/arm64 -t prtip:latest --push .
```

## Test Environment

The `test-environment/` directory contains a docker-compose setup with vulnerable services for testing:

```bash
cd test-environment
docker-compose up -d

# Scan the test environment
docker run --net=host --cap-add=NET_RAW prtip -A 172.20.0.0/24
```

## Troubleshooting

### Permission Denied

```bash
# Ensure capabilities are granted
docker run --cap-add=NET_RAW prtip ...

# Or run as root (not recommended)
docker run --user root prtip ...
```

### Network Issues

```bash
# Use host network mode for scanning external hosts
docker run --net=host ...

# Check network connectivity
docker run --rm prtip --iflist
```

### TUI Not Working

```bash
# Ensure interactive mode
docker run -it --rm ...

# Check terminal capabilities
docker run -it --rm prtip --tui 2>&1 | head -10
```

## CI/CD Integration

```yaml
# GitHub Actions example
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build image
        run: docker build -t prtip:test -f docker/Dockerfile .
      - name: Run scan
        run: docker run --cap-add=NET_RAW prtip:test -sT -p 80 localhost
```

## License

GPL-3.0 - See [LICENSE](../LICENSE)
