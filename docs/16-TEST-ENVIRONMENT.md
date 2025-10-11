# ProRT-IP Test Environment Setup

**Version:** 1.0
**Last Updated:** 2025-10-10
**Purpose:** Phase 4 Performance Benchmarking and Service Detection Validation

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Test Services](#test-services)
5. [Network Latency Simulation](#network-latency-simulation)
6. [Benchmark Scenarios](#benchmark-scenarios)
7. [Validation Tests](#validation-tests)
8. [Troubleshooting](#troubleshooting)

---

## Overview

This test environment provides a comprehensive suite of services for validating ProRT-IP's performance and accuracy against realistic network conditions. It replaces localhost testing with a Docker-based environment that includes:

- **10+ services** spanning HTTP, SSH, FTP, SMTP, DNS, databases, and more
- **Network latency simulation** using Linux tc (traffic control)
- **Isolated Docker network** (172.20.0.0/24) for controlled testing
- **Health checks** to ensure services are ready for testing

### Why This Environment?

The Phase 3 baseline (../benchmarks/1-BASELINE-RESULTS.md) revealed that localhost testing is **91-2000x faster** than expected network scans due to zero latency. This environment solves that problem by:

1. **Realistic RTT**: Add 10-100ms latency to simulate LAN/WAN/Internet conditions
2. **Diverse Services**: Test service detection accuracy against real implementations
3. **Repeatable Tests**: Consistent environment across development machines
4. **Isolation**: No impact on production systems or external networks

---

## Prerequisites

### Required Software

```bash
# Docker and Docker Compose
sudo pacman -S docker docker-compose     # Arch Linux
sudo apt install docker.io docker-compose  # Ubuntu/Debian
sudo dnf install docker docker-compose   # Fedora

# iproute2 (for tc traffic control)
sudo pacman -S iproute2     # Arch Linux
sudo apt install iproute2   # Ubuntu/Debian

# Optional: Network diagnostic tools
sudo pacman -S bind-tools nmap net-tools  # Arch Linux
```

### System Configuration

```bash
# Enable and start Docker service
sudo systemctl enable docker
sudo systemctl start docker

# Add current user to docker group (avoid sudo for docker commands)
sudo usermod -aG docker $USER
newgrp docker  # Activate group membership

# Verify Docker installation
docker --version
docker-compose --version
```

### Firewall Configuration

If you have a firewall enabled, allow Docker network traffic:

```bash
# UFW (Ubuntu)
sudo ufw allow from 172.20.0.0/24

# Firewalld (Fedora)
sudo firewall-cmd --permanent --zone=trusted --add-source=172.20.0.0/24
sudo firewall-cmd --reload

# iptables (manual)
sudo iptables -A INPUT -s 172.20.0.0/24 -j ACCEPT
```

---

## Quick Start

### 1. Start Test Environment

```bash
cd /home/parobek/Code/ProRT-IP/docker/test-environment

# Start all services
docker-compose up -d

# Verify services are running
docker-compose ps

# Check health status
docker-compose ps | grep "healthy"
```

**Expected Output:**

```
NAME                    IMAGE                       STATUS              PORTS
prtip-bind9             ubuntu/bind9:latest         Up (healthy)        53/tcp, 53/udp
prtip-memcached         memcached:latest            Up                  11211/tcp
prtip-metasploitable2   tleemcjr/metasploitable2    Up (healthy)        21/tcp, 22/tcp, 23/tcp, ...
prtip-mysql             mysql:8.0                   Up (healthy)        3306/tcp
prtip-nginx             nginx:latest                Up (healthy)        80/tcp, 443/tcp
prtip-openssh           linuxserver/openssh-server  Up (healthy)        2222/tcp
prtip-postgres          postgres:15                 Up (healthy)        5432/tcp
prtip-redis             redis:7-alpine              Up (healthy)        6379/tcp
prtip-snmpd             polinux/snmpd               Up                  161/udp
prtip-vsftpd            fauria/vsftpd               Up                  20/tcp, 21/tcp, ...
```

### 2. Get Container IP Addresses

```bash
# Quick reference
docker inspect -f '{{.Name}} - {{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' \
  $(docker-compose ps -q) | sed 's/\///g'

# Or use the helper command
docker network inspect prtip-test-environment_prtip_test | \
  grep -A 3 "IPv4Address"
```

**Expected IP Addresses:**

- Metasploitable2: `172.20.0.10`
- Nginx: `172.20.0.20`
- OpenSSH: `172.20.0.21`
- vsftpd: `172.20.0.22`
- MySQL: `172.20.0.30`
- PostgreSQL: `172.20.0.31`
- Redis: `172.20.0.32`
- Memcached: `172.20.0.33`
- BIND9: `172.20.0.40`
- SNMPD: `172.20.0.41`

### 3. Verify Connectivity

```bash
# Test basic connectivity
ping -c 3 172.20.0.10  # Metasploitable2

# Test HTTP service
curl -I http://172.20.0.20  # Nginx

# Test SSH banner
nc 172.20.0.21 2222 | head -n 1

# Test DNS
dig @172.20.0.40 google.com
```

### 4. Add Network Latency (Optional)

```bash
# Add 50ms latency to docker0 interface (simulates 100ms RTT)
sudo ../../../scripts/network-latency.sh docker 50ms

# Verify latency
ping -c 5 172.20.0.20

# Expected: ~100ms RTT (50ms each way)
```

### 5. Run ProRT-IP Tests

```bash
cd /home/parobek/Code/ProRT-IP

# Build release binary
cargo build --release

# Quick connectivity test
./target/release/prtip --scan-type connect -p 80 172.20.0.20

# Full service detection
./target/release/prtip --scan-type connect --sV -p 1-1000 172.20.0.10

# Performance benchmark
time ./target/release/prtip --scan-type connect -p 1-10000 172.20.0.10 --timing 4
```

### 6. Stop Test Environment

```bash
# Stop all containers
docker-compose down

# Remove latency (if added)
sudo ../../../scripts/network-latency.sh remove docker0

# Clean up volumes (optional, removes databases)
docker-compose down -v
```

---

## Test Services

### Service Matrix

| Service | Container IP | Ports | Purpose | Status |
|---------|--------------|-------|---------|--------|
| **Metasploitable2** | 172.20.0.10 | 21, 22, 23, 25, 53, 80, 139, 445, 3306, 5432, 8180 | Comprehensive vulnerable VM | ✅ |
| **Nginx** | 172.20.0.20 | 80, 443 | Modern HTTP/HTTPS server | ✅ |
| **OpenSSH** | 172.20.0.21 | 2222 | SSH server (custom port) | ✅ |
| **vsftpd** | 172.20.0.22 | 20, 21, 21100-21110 | FTP server | ✅ |
| **MySQL** | 172.20.0.30 | 3306 | MySQL database | ✅ |
| **PostgreSQL** | 172.20.0.31 | 5432 | PostgreSQL database | ✅ |
| **Redis** | 172.20.0.32 | 6379 | Redis key-value store | ✅ |
| **Memcached** | 172.20.0.33 | 11211 | Memcached server | ✅ |
| **BIND9** | 172.20.0.40 | 53 (TCP/UDP) | DNS server | ✅ |
| **SNMPD** | 172.20.0.41 | 161 (UDP) | SNMP agent | ✅ |

### Service Details

#### 1. Metasploitable2 (172.20.0.10)

**Description:** Intentionally vulnerable Linux VM with 20+ services

**Services:**

- FTP (21): vsftpd 2.3.4 (with backdoor)
- SSH (22): OpenSSH 4.7p1
- Telnet (23): Linux telnetd
- SMTP (25): Postfix
- DNS (53): ISC BIND 9.x
- HTTP (80): Apache 2.2.8
- NetBIOS (139): Samba 3.x
- SMB (445): Samba 3.x
- MySQL (3306): MySQL 5.0
- PostgreSQL (5432): PostgreSQL 8.3

**Test Scenarios:**

```bash
# Service version detection
prtip --scan-type connect --sV -p 21-80 172.20.0.10

# Banner grabbing
prtip --scan-type connect --banner-grab -p 22,25,80 172.20.0.10

# Full port scan
prtip --scan-type connect -p- 172.20.0.10 --timing 4
```

#### 2. Nginx (172.20.0.20)

**Description:** Modern HTTP server with custom headers for testing

**Custom Headers:**

- `X-Server-Type: ProRT-IP-Test-Nginx`
- `X-Test-Environment: Phase-4-Benchmarking`

**Test Scenarios:**

```bash
# HTTP banner grabbing
prtip --scan-type connect --sV -p 80 172.20.0.20

# Verify custom headers detected
curl -I http://172.20.0.20
```

#### 3. OpenSSH (172.20.0.21)

**Description:** SSH server on custom port 2222

**Credentials:**

- Username: `testuser`
- Password: `testpassword`

**Test Scenarios:**

```bash
# SSH banner grabbing
prtip --scan-type connect --sV -p 2222 172.20.0.21

# Verify banner
nc 172.20.0.21 2222
```

#### 4. MySQL & PostgreSQL

**Description:** Database servers for testing MySQL/PostgreSQL detection

**MySQL (172.20.0.30:3306):**

- Root password: `rootpassword`
- Database: `testdb`
- User: `testuser` / `testpass`

**PostgreSQL (172.20.0.31:5432):**

- User: `testuser`
- Password: `testpass`
- Database: `testdb`

**Test Scenarios:**

```bash
# Database service detection
prtip --scan-type connect --sV -p 3306,5432 172.20.0.30-31
```

#### 5. Redis & Memcached

**Description:** Key-value stores for caching

**Test Scenarios:**

```bash
# Redis detection
prtip --scan-type connect --sV -p 6379 172.20.0.32

# Memcached detection
prtip --scan-type connect --sV -p 11211 172.20.0.33
```

#### 6. BIND9 DNS (172.20.0.40)

**Description:** DNS server for UDP testing

**Test Scenarios:**

```bash
# UDP DNS scan
prtip --scan-type udp -p 53 172.20.0.40

# Verify DNS response
dig @172.20.0.40 google.com
```

#### 7. SNMPD (172.20.0.41)

**Description:** SNMP agent for UDP testing

**Community String:** `public`

**Test Scenarios:**

```bash
# UDP SNMP scan
prtip --scan-type udp -p 161 172.20.0.41

# Verify SNMP response
snmpwalk -v2c -c public 172.20.0.41
```

---

## Network Latency Simulation

### Overview

The `network-latency.sh` script uses Linux `tc` (traffic control) with `netem` (network emulator) to add artificial latency to network interfaces. This simulates realistic network conditions for performance testing.

### Usage

```bash
# Basic syntax
sudo scripts/network-latency.sh <command> <interface> [latency]

# Quick docker setup
sudo scripts/network-latency.sh docker <latency>

# Manual interface setup
sudo scripts/network-latency.sh add <interface> <latency>
sudo scripts/network-latency.sh remove <interface>
sudo scripts/network-latency.sh show <interface>
```

### Common Scenarios

#### LAN Testing (10ms RTT)

```bash
# Add 5ms latency (5ms × 2 = 10ms RTT)
sudo scripts/network-latency.sh docker 5ms

# Verify
ping -c 5 172.20.0.10
# Expected RTT: ~10ms
```

#### WAN Testing (100ms RTT)

```bash
# Add 50ms latency (50ms × 2 = 100ms RTT)
sudo scripts/network-latency.sh docker 50ms

# Verify
ping -c 5 172.20.0.10
# Expected RTT: ~100ms
```

#### Internet Testing (200ms RTT)

```bash
# Add 100ms latency (100ms × 2 = 200ms RTT)
sudo scripts/network-latency.sh docker 100ms

# Verify
ping -c 5 172.20.0.10
# Expected RTT: ~200ms
```

#### Satellite Link (600ms RTT)

```bash
# Add 300ms latency (300ms × 2 = 600ms RTT)
sudo scripts/network-latency.sh docker 300ms

# Verify
ping -c 5 172.20.0.10
# Expected RTT: ~600ms
```

### Remove Latency

```bash
# Remove latency from docker0
sudo scripts/network-latency.sh remove docker0

# Verify
ping -c 5 172.20.0.10
# Expected RTT: <1ms (loopback speed)
```

### Show Current Configuration

```bash
# View current qdisc configuration
sudo scripts/network-latency.sh show docker0

# Expected output (with latency):
# Current qdisc configuration for docker0:
# qdisc netem 8001: root refcnt 2 limit 1000 delay 50ms
# ✓ Network emulation (netem) is active

# Expected output (without latency):
# Current qdisc configuration for docker0:
# qdisc noqueue 0: root refcnt 2
# ⚠ No network emulation configured (using default qdisc)
```

---

## Benchmark Scenarios

### Scenario 1: Timing Template Validation

**Purpose:** Validate timing templates (T0-T5) with realistic network latency

**Test Setup:**

```bash
# Add 50ms latency (100ms RTT)
sudo scripts/network-latency.sh docker 50ms
```

**Test Commands:**

```bash
# T0 - Paranoid (5-minute probe delays)
time prtip --scan-type connect -p 1-100 172.20.0.10 --timing 0

# T2 - Polite (400ms delays)
time prtip --scan-type connect -p 1-100 172.20.0.10 --timing 2

# T3 - Normal (default)
time prtip --scan-type connect -p 1-100 172.20.0.10 --timing 3

# T4 - Aggressive (fast)
time prtip --scan-type connect -p 1-100 172.20.0.10 --timing 4

# T5 - Insane (maximum speed)
time prtip --scan-type connect -p 1-100 172.20.0.10 --timing 5
```

**Expected Results:**

| Template | Expected Duration | Relative Speed |
|----------|-------------------|----------------|
| T0 | 500-600 seconds | 1.0x (baseline) |
| T2 | 10-20 seconds | 25-60x faster |
| T3 | 5-10 seconds | 50-120x faster |
| T4 | 2-5 seconds | 100-300x faster |
| T5 | 1-2 seconds | 250-600x faster |

### Scenario 2: Service Detection Accuracy

**Purpose:** Validate service detection against diverse services

**Test Setup:**

```bash
# No latency needed for accuracy testing
sudo scripts/network-latency.sh remove docker0
```

**Test Commands:**

```bash
# Full service scan on Metasploitable2
prtip --scan-type connect --sV -p 21,22,25,80,3306,5432 172.20.0.10 \
  --version-intensity 7 --output json > metasploitable-services.json

# Compare with Nmap baseline
nmap -sV -p 21,22,25,80,3306,5432 172.20.0.10 -oX nmap-baseline.xml

# Manual comparison (check for >95% accuracy)
```

**Expected Services:**

| Port | Service | Version |
|------|---------|---------|
| 21 | FTP | vsftpd 2.3.4 |
| 22 | SSH | OpenSSH 4.7p1 |
| 25 | SMTP | Postfix |
| 80 | HTTP | Apache 2.2.8 |
| 3306 | MySQL | MySQL 5.0 |
| 5432 | PostgreSQL | PostgreSQL 8.3 |

### Scenario 3: Performance Benchmarking

**Purpose:** Measure ProRT-IP performance vs baselines with network latency

**Test Setup:**

```bash
# Add 50ms latency (100ms RTT)
sudo scripts/network-latency.sh docker 50ms
```

**Test Commands:**

```bash
# ProRT-IP (10K ports, T4)
time prtip --scan-type connect -p 1-10000 172.20.0.10 --timing 4 \
  --no-progress --output json > prtip-10k.json

# Nmap (10K ports, T4)
time nmap -sT -p 1-10000 172.20.0.10 -T4 -oX nmap-10k.xml

# RustScan (10K ports)
time rustscan -a 172.20.0.10 --range 1-10000 --batch-size 5000 \
  > rustscan-10k.txt

# Masscan (10K ports, 10K pps)
time masscan -p 1-10000 172.20.0.10 --rate 10000 -oJ masscan-10k.json
```

**Expected Performance (100ms RTT):**

| Tool | Duration | Ports/Second | Notes |
|------|----------|--------------|-------|
| ProRT-IP (T4) | 15-30s | 330-660 pps | Stateful with connection tracking |
| Nmap (T4) | 20-40s | 250-500 pps | Baseline for accuracy comparison |
| RustScan | 10-20s | 500-1000 pps | Rust async I/O |
| Masscan | 1-2s | 5000-10000 pps | Stateless (less accurate) |

**Performance Targets:**

- ProRT-IP should be **1.5-2x faster** than Nmap
- ProRT-IP should be **5-10x slower** than Masscan (acceptable for stateful tracking)
- ProRT-IP should be **comparable** to RustScan (both use Tokio + FuturesUnordered)

### Scenario 4: Full Port Range Optimization

**Purpose:** Validate Sprint 4.4 full port range optimization (65K ports in <10s)

**Test Setup:**

```bash
# Remove latency for localhost-speed testing
sudo scripts/network-latency.sh remove docker0
```

**Test Commands:**

```bash
# Full port range scan
time prtip --scan-type connect -p- 172.20.0.10 --timing 4 \
  --no-progress --output json > prtip-full-range.json

# Expected: <10 seconds on localhost, <60 seconds with 100ms latency
```

**Expected Results:**

| Environment | Duration | Ports/Second | Status |
|-------------|----------|--------------|--------|
| Localhost (no latency) | <10s | 6,500+ pps | ✅ Sprint 4.4 target |
| LAN (10ms RTT) | <30s | 2,200+ pps | Acceptable |
| WAN (100ms RTT) | <60s | 1,100+ pps | Acceptable |

---

## Validation Tests

### Test Suite: Service Detection Accuracy

**Objective:** Validate >95% accuracy vs Nmap service detection

**Test 1: HTTP Server Detection**

```bash
# ProRT-IP
prtip --scan-type connect --sV -p 80 172.20.0.20

# Nmap baseline
nmap -sV -p 80 172.20.0.20

# Expected:
# - Port 80: open
# - Service: http
# - Product: nginx
# - Version: detected
# - Custom headers: X-Server-Type, X-Test-Environment
```

**Test 2: SSH Banner Grabbing**

```bash
# ProRT-IP
prtip --scan-type connect --sV -p 2222 172.20.0.21

# Nmap baseline
nmap -sV -p 2222 172.20.0.21

# Expected:
# - Port 2222: open
# - Service: ssh
# - Banner: OpenSSH_X.X
```

**Test 3: Database Detection**

```bash
# ProRT-IP
prtip --scan-type connect --sV -p 3306,5432 172.20.0.30-31

# Nmap baseline
nmap -sV -p 3306,5432 172.20.0.30-31

# Expected:
# - Port 3306: MySQL
# - Port 5432: PostgreSQL
```

**Test 4: UDP Service Detection**

```bash
# ProRT-IP
prtip --scan-type udp --sV -p 53,161 172.20.0.40-41

# Nmap baseline
nmap -sU -sV -p 53,161 172.20.0.40-41

# Expected:
# - Port 53: domain (DNS)
# - Port 161: snmp
```

**Success Criteria:**

- ✅ >95% service identification match with Nmap
- ✅ Correct version detection for major services
- ✅ <10% speed penalty vs port scanning alone

---

## Troubleshooting

### Container Startup Issues

**Problem:** Containers fail to start or immediately exit

**Solutions:**

```bash
# Check Docker daemon status
sudo systemctl status docker

# View container logs
docker-compose logs <service-name>

# Restart Docker service
sudo systemctl restart docker

# Clean up and restart
docker-compose down
docker-compose up -d
```

### Network Connectivity Issues

**Problem:** Cannot reach container IP addresses

**Solutions:**

```bash
# Verify Docker network
docker network ls
docker network inspect prtip-test-environment_prtip_test

# Check routing
ip route | grep 172.20.0.0

# Restart Docker network
docker-compose down
docker network prune -f
docker-compose up -d
```

### Latency Simulation Not Working

**Problem:** `tc` command fails or latency not applied

**Solutions:**

```bash
# Check if tc is installed
which tc

# Install iproute2
sudo pacman -S iproute2  # Arch
sudo apt install iproute2  # Ubuntu

# Verify kernel modules
lsmod | grep sch_netem
# If not loaded:
sudo modprobe sch_netem

# Check current qdisc
tc qdisc show dev docker0
```

### Permission Denied Errors

**Problem:** ProRT-IP fails with permission errors

**Solutions:**

```bash
# Check if user is in docker group
groups | grep docker

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# For raw packet scanning, use sudo or capabilities
sudo prtip --scan-type syn -p 80 172.20.0.10

# Or set capabilities
sudo setcap cap_net_raw+ep ./target/release/prtip
```

### Service Health Check Failures

**Problem:** Services show "unhealthy" status

**Solutions:**

```bash
# Check service logs
docker-compose logs <service-name>

# Restart specific service
docker-compose restart <service-name>

# Wait for services to initialize (some take 30-60 seconds)
docker-compose ps
# Repeat until all show "healthy"
```

---

## Next Steps

1. **Sprint 4.1 Complete:** ✅ Test environment setup, latency simulation
2. **Sprint 4.2:** Lock-free data structures for reduced contention
3. **Sprint 4.3:** Batched syscalls (sendmmsg/recvmmsg) for 1M+ pps
4. **Sprint 4.4:** Full port range optimization (65K ports in <10s)
5. **Sprint 4.6:** Service detection validation (use this environment)

**Related Documentation:**

- [Phase 4 Performance Plan](../.cursor/plans/phase-4-performance-plan.plan.md)
- [Benchmark Methodology](14-BENCHMARKS.md)
- [Performance Baselines](../benchmarks/README.md)
- [Performance Optimization Guide](07-PERFORMANCE.md)

---

**Document Metadata:**

- **Created:** 2025-10-10
- **Author:** Claude Code (Automated Setup)
- **Version:** 1.0
- **Sprint:** Phase 4 Sprint 4.1 (Network Testing Infrastructure)
