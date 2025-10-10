#!/usr/bin/env bash
#
# Network Latency Simulation Script
#
# This script uses Linux tc (traffic control) with netem (network emulator)
# to add artificial latency to network interfaces for realistic testing.
#
# Usage:
#   ./network-latency.sh add <interface> <latency_ms>  # Add latency
#   ./network-latency.sh remove <interface>            # Remove latency
#   ./network-latency.sh show <interface>              # Show current settings
#   ./network-latency.sh docker <latency_ms>           # Add latency to docker0
#
# Examples:
#   ./network-latency.sh add docker0 50ms              # 50ms RTT (25ms each way)
#   ./network-latency.sh docker 10ms                   # 10ms RTT on docker0
#   ./network-latency.sh remove docker0                # Remove latency
#   ./network-latency.sh show docker0                  # Show current qdisc
#
# Phase 4 Performance Testing:
#   - Use with Metasploitable2 Docker container for realistic network testing
#   - Simulate various network conditions (LAN: 10ms, WAN: 50ms, Internet: 100ms)
#   - Validate timing template behavior (T0-T5) with real latency
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script metadata
SCRIPT_NAME="$(basename "$0")"
VERSION="1.0.0"

# Functions
usage() {
    cat <<EOF
${GREEN}ProRT-IP Network Latency Simulator v${VERSION}${NC}

${BLUE}Usage:${NC}
  ${SCRIPT_NAME} add <interface> <latency>     Add network latency
  ${SCRIPT_NAME} remove <interface>            Remove network latency
  ${SCRIPT_NAME} show <interface>              Show current qdisc configuration
  ${SCRIPT_NAME} docker <latency>              Add latency to docker0 interface

${BLUE}Arguments:${NC}
  interface    Network interface (e.g., eth0, docker0, wlan0)
  latency      Latency in milliseconds (e.g., 10ms, 50ms, 100ms)

${BLUE}Examples:${NC}
  # Add 50ms latency to docker0 (simulates 100ms RTT)
  sudo ${SCRIPT_NAME} add docker0 50ms

  # Quick docker latency setup
  sudo ${SCRIPT_NAME} docker 25ms

  # Remove latency from docker0
  sudo ${SCRIPT_NAME} remove docker0

  # Show current configuration
  ${SCRIPT_NAME} show docker0

${BLUE}Common Scenarios:${NC}
  LAN testing:      10ms  (20ms RTT)
  WAN testing:      50ms  (100ms RTT)
  Internet testing: 100ms (200ms RTT)
  Satellite link:   300ms (600ms RTT)

${BLUE}Requirements:${NC}
  - Linux kernel with tc (traffic control) support
  - Root privileges (sudo)
  - iproute2 package installed

${YELLOW}Note:${NC} This script modifies kernel network settings. Use with caution.
EOF
    exit 0
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}Error: This script must be run as root (use sudo)${NC}" >&2
        exit 1
    fi
}

# Check if interface exists
check_interface() {
    local iface="$1"
    if ! ip link show "$iface" &> /dev/null; then
        echo -e "${RED}Error: Interface '$iface' does not exist${NC}" >&2
        echo -e "${YELLOW}Available interfaces:${NC}" >&2
        ip -br link show | awk '{print "  - " $1}' >&2
        exit 1
    fi
}

# Add latency to interface
add_latency() {
    local iface="$1"
    local latency="$2"

    check_root
    check_interface "$iface"

    # Remove existing qdisc if present
    tc qdisc del dev "$iface" root &> /dev/null || true

    # Add netem qdisc with delay
    echo -e "${BLUE}Adding ${latency} latency to ${iface}...${NC}"
    if tc qdisc add dev "$iface" root netem delay "$latency"; then
        echo -e "${GREEN}✓ Successfully added ${latency} latency to ${iface}${NC}"
        echo -e "${YELLOW}Note: Total RTT will be ${latency} × 2 (${latency} each way)${NC}"
        show_qdisc "$iface"
    else
        echo -e "${RED}✗ Failed to add latency${NC}" >&2
        exit 1
    fi
}

# Remove latency from interface
remove_latency() {
    local iface="$1"

    check_root
    check_interface "$iface"

    echo -e "${BLUE}Removing latency from ${iface}...${NC}"
    if tc qdisc del dev "$iface" root &> /dev/null; then
        echo -e "${GREEN}✓ Successfully removed latency from ${iface}${NC}"
    else
        echo -e "${YELLOW}No latency configuration found on ${iface}${NC}"
    fi
}

# Show current qdisc configuration
show_qdisc() {
    local iface="$1"

    check_interface "$iface"

    echo -e "${BLUE}Current qdisc configuration for ${iface}:${NC}"
    tc qdisc show dev "$iface"

    # Check if netem is configured
    if tc qdisc show dev "$iface" | grep -q "netem"; then
        echo -e "${GREEN}✓ Network emulation (netem) is active${NC}"
    else
        echo -e "${YELLOW}⚠ No network emulation configured (using default qdisc)${NC}"
    fi
}

# Quick docker setup
docker_latency() {
    local latency="$1"
    add_latency "docker0" "$latency"
}

# Main command dispatcher
main() {
    if [[ $# -eq 0 ]]; then
        usage
    fi

    local command="$1"

    case "$command" in
        add)
            if [[ $# -ne 3 ]]; then
                echo -e "${RED}Error: 'add' requires interface and latency${NC}" >&2
                echo "Usage: $SCRIPT_NAME add <interface> <latency>" >&2
                exit 1
            fi
            add_latency "$2" "$3"
            ;;
        remove)
            if [[ $# -ne 2 ]]; then
                echo -e "${RED}Error: 'remove' requires interface${NC}" >&2
                echo "Usage: $SCRIPT_NAME remove <interface>" >&2
                exit 1
            fi
            remove_latency "$2"
            ;;
        show)
            if [[ $# -ne 2 ]]; then
                echo -e "${RED}Error: 'show' requires interface${NC}" >&2
                echo "Usage: $SCRIPT_NAME show <interface>" >&2
                exit 1
            fi
            show_qdisc "$2"
            ;;
        docker)
            if [[ $# -ne 2 ]]; then
                echo -e "${RED}Error: 'docker' requires latency${NC}" >&2
                echo "Usage: $SCRIPT_NAME docker <latency>" >&2
                exit 1
            fi
            docker_latency "$2"
            ;;
        -h|--help|help)
            usage
            ;;
        -v|--version)
            echo "v${VERSION}"
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown command '$command'${NC}" >&2
            echo "Run '$SCRIPT_NAME --help' for usage information" >&2
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
