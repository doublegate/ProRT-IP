#!/bin/bash
# ProRT-IP Fuzzing Corpus Generation Script
# Generates seed corpus files for all fuzz targets
#
# Usage: ./generate_corpus.sh
# Output: 460+ corpus files in fuzz/corpus/{target}/

set -euo pipefail

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORPUS_BASE="$SCRIPT_DIR/../corpus"

echo -e "${BLUE}=== ProRT-IP Fuzz Corpus Generator ===${NC}"
echo "Generating seed corpus files..."
echo

# Helper function to create hex dump from bytes
create_seed() {
    local target=$1
    local name=$2
    local data=$3
    local file="$CORPUS_BASE/$target/$name"

    echo -ne "$data" > "$file"
}

# =============================================================================
# TCP Corpus (100 seeds)
# =============================================================================
echo -e "${GREEN}[1/5] Generating TCP corpus (100 seeds)...${NC}"
TCP_DIR="$CORPUS_BASE/fuzz_tcp_parser"
mkdir -p "$TCP_DIR"

# Valid TCP packets (40 seeds)
# Minimal TCP packet (20 bytes header, no payload)
create_seed fuzz_tcp_parser "tcp_minimal" "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x02\x20\x00\x00\x00\x00\x00"

# SYN packet
create_seed fuzz_tcp_parser "tcp_syn" "\x04\x00\x00\x50\x00\x00\x00\x01\x00\x00\x00\x00\x50\x02\x20\x00\x00\x00\x00\x00"

# SYN-ACK packet
create_seed fuzz_tcp_parser "tcp_synack" "\x00\x50\x04\x00\x00\x00\x00\x01\x00\x00\x00\x02\x50\x12\x20\x00\x00\x00\x00\x00"

# ACK packet with payload
create_seed fuzz_tcp_parser "tcp_ack_payload" "\x00\x50\x00\x50\x00\x00\x00\x01\x00\x00\x00\x02\x50\x10\x20\x00\x00\x00\x00\x00GET / HTTP/1.1"

# FIN packet
create_seed fuzz_tcp_parser "tcp_fin" "\x00\x50\x00\x50\x00\x00\x00\x01\x00\x00\x00\x02\x50\x01\x20\x00\x00\x00\x00\x00"

# RST packet
create_seed fuzz_tcp_parser "tcp_rst" "\x00\x50\x00\x50\x00\x00\x00\x01\x00\x00\x00\x00\x50\x04\x20\x00\x00\x00\x00\x00"

# Generate more valid TCP variations (34 more)
for i in {1..34}; do
    port1=$(printf "%04x" $((1024 + i)))
    port2=$(printf "%04x" $((30000 + i)))
    seq=$(printf "%08x" $((i * 100)))
    # Random flag combination
    flags=$(printf "%02x" $((i % 64)))

    create_seed fuzz_tcp_parser "tcp_valid_$i" "$(echo -ne "\x${port1:0:2}\x${port1:2:2}\x${port2:0:2}\x${port2:2:2}\x${seq:0:2}\x${seq:2:2}\x${seq:4:2}\x${seq:6:2}\x00\x00\x00\x00\x50\x${flags}\x20\x00\x00\x00\x00\x00")"
done

# Invalid/malformed TCP packets (30 seeds)
# Truncated header (5 bytes)
create_seed fuzz_tcp_parser "tcp_truncated_5" "\x00\x50\x00\x50\x00"

# Truncated header (15 bytes)
create_seed fuzz_tcp_parser "tcp_truncated_15" "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x02\x20"

# Zero data offset
create_seed fuzz_tcp_parser "tcp_zero_offset" "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x20\x00\x00\x00\x00\x00"

# Invalid data offset (too large)
create_seed fuzz_tcp_parser "tcp_large_offset" "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\xF0\x02\x20\x00\x00\x00\x00\x00"

# All flags set
create_seed fuzz_tcp_parser "tcp_all_flags" "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\xFF\x20\x00\x00\x00\x00\x00"

# Generate more malformed packets (25 more)
for i in {1..25}; do
    # Random corrupted data
    seed_data=$(head -c 50 /dev/urandom | base64 | tr -d '\n' | cut -c1-50)
    echo -n "$seed_data" | base64 -d > "$TCP_DIR/tcp_malformed_$i" 2>/dev/null || echo -n "$seed_data" > "$TCP_DIR/tcp_malformed_$i"
done

# Edge cases (30 seeds)
# Minimum valid TCP (20 bytes)
create_seed fuzz_tcp_parser "tcp_edge_min" "\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x50\x00\x00\x00\x00\x00\x00\x00"

# Maximum MTU payload (1500 - 20 IP - 20 TCP = 1460 bytes)
create_seed fuzz_tcp_parser "tcp_edge_max_mtu" "$(echo -ne "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x00\x00\x00\x00\x00\x00\x00"; head -c 1460 /dev/zero)"

# Generate more edge cases (28 more)
for i in {1..28}; do
    size=$((20 + i * 50))
    create_seed fuzz_tcp_parser "tcp_edge_$i" "$(echo -ne "\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x00\x00\x00\x00\x00\x00\x00"; head -c $size /dev/zero)"
done

echo "  ✓ Generated 100 TCP corpus seeds"

# =============================================================================
# UDP Corpus (80 seeds)
# =============================================================================
echo -e "${GREEN}[2/5] Generating UDP corpus (80 seeds)...${NC}"
UDP_DIR="$CORPUS_BASE/fuzz_udp_parser"
mkdir -p "$UDP_DIR"

# Valid UDP packets (30 seeds)
# Minimal UDP (8 bytes header, no payload)
create_seed fuzz_udp_parser "udp_minimal" "\x00\x35\x00\x35\x00\x08\x00\x00"

# DNS query (port 53)
create_seed fuzz_udp_parser "udp_dns_query" "\x00\x35\x00\x35\x00\x20\x00\x00\x00\x01\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01"

# SNMP request (port 161)
create_seed fuzz_udp_parser "udp_snmp" "\x00\xa1\x00\xa1\x00\x10\x00\x00\x30\x08\x02\x01\x00\x04\x00\xa0\x00"

# NetBIOS (port 137)
create_seed fuzz_udp_parser "udp_netbios" "\x00\x89\x00\x89\x00\x14\x00\x00\x00\x01\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00"

# Generate more valid UDP (26 more)
for i in {1..26}; do
    port=$(printf "%04x" $((5000 + i)))
    len=$(printf "%04x" $((8 + i)))
    create_seed fuzz_udp_parser "udp_valid_$i" "$(echo -ne "\x${port:0:2}\x${port:2:2}\x${port:0:2}\x${port:2:2}\x${len:0:2}\x${len:2:2}\x00\x00"; head -c $i /dev/zero)"
done

# Invalid/malformed UDP (25 seeds)
# Truncated header (4 bytes)
create_seed fuzz_udp_parser "udp_truncated_4" "\x00\x35\x00\x35"

# Zero length
create_seed fuzz_udp_parser "udp_zero_length" "\x00\x35\x00\x35\x00\x00\x00\x00"

# Length mismatch (claims 100 bytes, only 20)
create_seed fuzz_udp_parser "udp_length_mismatch" "\x00\x35\x00\x35\x00\x64\x00\x00\x00\x00\x00\x00"

# Generate more malformed (22 more)
for i in {1..22}; do
    seed_data=$(head -c 30 /dev/urandom | base64 | tr -d '\n' | cut -c1-30)
    echo -n "$seed_data" | base64 -d > "$UDP_DIR/udp_malformed_$i" 2>/dev/null || echo -n "$seed_data" > "$UDP_DIR/udp_malformed_$i"
done

# Edge cases (25 seeds)
# Maximum UDP payload (65535 - 8 = 65527, but limit to 1472 for MTU)
create_seed fuzz_udp_parser "udp_edge_max_mtu" "$(echo -ne "\x00\x35\x00\x35\x05\xc8\x00\x00"; head -c 1472 /dev/zero)"

# Generate more edge cases (24 more)
for i in {1..24}; do
    size=$((8 + i * 60))
    len=$(printf "%04x" $size)
    create_seed fuzz_udp_parser "udp_edge_$i" "$(echo -ne "\x00\x35\x00\x35\x${len:0:2}\x${len:2:2}\x00\x00"; head -c $((size - 8)) /dev/zero)"
done

echo "  ✓ Generated 80 UDP corpus seeds"

# =============================================================================
# IPv6 Corpus (100 seeds)
# =============================================================================
echo -e "${GREEN}[3/5] Generating IPv6 corpus (100 seeds)...${NC}"
IPV6_DIR="$CORPUS_BASE/fuzz_ipv6_parser"
mkdir -p "$IPV6_DIR"

# Valid IPv6 packets (40 seeds)
# Minimal IPv6 (40 bytes header, no payload)
create_seed fuzz_ipv6_parser "ipv6_minimal" "\x60\x00\x00\x00\x00\x00\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02"

# IPv6 with TCP (next header = 6)
create_seed fuzz_ipv6_parser "ipv6_tcp" "\x60\x00\x00\x00\x00\x14\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x50\x00\x50\x00\x00\x00\x00\x00\x00\x00\x00\x50\x02\x20\x00\x00\x00\x00\x00"

# IPv6 with ICMPv6 (next header = 58)
create_seed fuzz_ipv6_parser "ipv6_icmpv6" "\x60\x00\x00\x00\x00\x08\x3a\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x80\x00\x00\x00\x00\x00\x00\x00"

# Generate more valid IPv6 (37 more)
for i in {1..37}; do
    payload_len=$(printf "%04x" $((i * 10)))
    hop_limit=$(printf "%02x" $((64 - i % 32)))
    create_seed fuzz_ipv6_parser "ipv6_valid_$i" "$(echo -ne "\x60\x00\x00\x00\x${payload_len:0:2}\x${payload_len:2:2}\x06\x${hop_limit}\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02"; head -c $((i * 10)) /dev/zero)"
done

# Invalid/malformed IPv6 (30 seeds)
# Truncated header (20 bytes)
create_seed fuzz_ipv6_parser "ipv6_truncated_20" "\x60\x00\x00\x00\x00\x00\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"

# Wrong version (version = 4)
create_seed fuzz_ipv6_parser "ipv6_wrong_version" "\x40\x00\x00\x00\x00\x00\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02"

# Payload length mismatch
create_seed fuzz_ipv6_parser "ipv6_length_mismatch" "\x60\x00\x00\x00\x00\x64\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x00"

# Generate more malformed (27 more)
for i in {1..27}; do
    seed_data=$(head -c 60 /dev/urandom | base64 | tr -d '\n' | cut -c1-60)
    echo -n "$seed_data" | base64 -d > "$IPV6_DIR/ipv6_malformed_$i" 2>/dev/null || echo -n "$seed_data" > "$IPV6_DIR/ipv6_malformed_$i"
done

# Edge cases (30 seeds)
# Maximum MTU (1280 bytes minimum IPv6 MTU)
create_seed fuzz_ipv6_parser "ipv6_edge_max_mtu" "$(echo -ne "\x60\x00\x00\x00\x04\xd8\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02"; head -c 1240 /dev/zero)"

# Generate more edge cases (29 more)
for i in {1..29}; do
    size=$((i * 40))
    payload_len=$(printf "%04x" $size)
    create_seed fuzz_ipv6_parser "ipv6_edge_$i" "$(echo -ne "\x60\x00\x00\x00\x${payload_len:0:2}\x${payload_len:2:2}\x06\x40\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02"; head -c $size /dev/zero)"
done

echo "  ✓ Generated 100 IPv6 corpus seeds"

# =============================================================================
# ICMPv6 Corpus (80 seeds)
# =============================================================================
echo -e "${GREEN}[4/5] Generating ICMPv6 corpus (80 seeds)...${NC}"
ICMPV6_DIR="$CORPUS_BASE/fuzz_icmpv6_parser"
mkdir -p "$ICMPV6_DIR"

# Valid ICMPv6 packets (30 seeds)
# Echo Request (type 128)
create_seed fuzz_icmpv6_parser "icmpv6_echo_request" "\x80\x00\x00\x00\x00\x01\x00\x01test"

# Echo Reply (type 129)
create_seed fuzz_icmpv6_parser "icmpv6_echo_reply" "\x81\x00\x00\x00\x00\x01\x00\x01test"

# Destination Unreachable (type 1)
create_seed fuzz_icmpv6_parser "icmpv6_dest_unreach" "\x01\x00\x00\x00\x00\x00\x00\x00\x60\x00\x00\x00\x00\x14\x06\x40"

# Router Solicitation (type 133)
create_seed fuzz_icmpv6_parser "icmpv6_router_sol" "\x85\x00\x00\x00\x00\x00\x00\x00"

# Neighbor Solicitation (type 135)
create_seed fuzz_icmpv6_parser "icmpv6_neighbor_sol" "\x87\x00\x00\x00\x00\x00\x00\x00\xfe\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01"

# Generate more valid ICMPv6 (25 more)
for i in {1..25}; do
    type=$(printf "%02x" $((128 + i % 16)))
    id=$(printf "%04x" $i)
    seq=$(printf "%04x" $((i * 2)))
    create_seed fuzz_icmpv6_parser "icmpv6_valid_$i" "$(echo -ne "\x${type}\x00\x00\x00\x${id:0:2}\x${id:2:2}\x${seq:0:2}\x${seq:2:2}"; head -c $((i * 10)) /dev/zero)"
done

# Invalid/malformed ICMPv6 (25 seeds)
# Truncated header (2 bytes)
create_seed fuzz_icmpv6_parser "icmpv6_truncated_2" "\x80\x00"

# Invalid type (255)
create_seed fuzz_icmpv6_parser "icmpv6_invalid_type" "\xff\x00\x00\x00\x00\x00\x00\x00"

# Generate more malformed (23 more)
for i in {1..23}; do
    seed_data=$(head -c 40 /dev/urandom | base64 | tr -d '\n' | cut -c1-40)
    echo -n "$seed_data" | base64 -d > "$ICMPV6_DIR/icmpv6_malformed_$i" 2>/dev/null || echo -n "$seed_data" > "$ICMPV6_DIR/icmpv6_malformed_$i"
done

# Edge cases (25 seeds)
# Maximum payload (1232 bytes for ICMPv6)
create_seed fuzz_icmpv6_parser "icmpv6_edge_max" "$(echo -ne "\x80\x00\x00\x00\x00\x01\x00\x01"; head -c 1232 /dev/zero)"

# Generate more edge cases (24 more)
for i in {1..24}; do
    size=$((i * 50))
    create_seed fuzz_icmpv6_parser "icmpv6_edge_$i" "$(echo -ne "\x80\x00\x00\x00\x00\x01\x00\x01"; head -c $size /dev/zero)"
done

echo "  ✓ Generated 80 ICMPv6 corpus seeds"

# =============================================================================
# TLS Certificate Corpus (100 seeds)
# =============================================================================
echo -e "${GREEN}[5/5] Generating TLS certificate corpus (100 seeds)...${NC}"
TLS_DIR="$CORPUS_BASE/fuzz_tls_parser"
mkdir -p "$TLS_DIR"

# Note: Real X.509 certificates are complex DER-encoded ASN.1 structures
# We'll generate simplified structures and some random data

# Valid-ish certificate structures (40 seeds)
# Minimal valid DER structure
create_seed fuzz_tls_parser "tls_minimal_der" "\x30\x82\x01\x00\x30\x81\xF0\xA0\x03\x02\x01\x02"

# Generate minimal certificate structures (39 more)
for i in {1..39}; do
    # SEQUENCE tag + length + version + serial
    size=$((200 + i * 50))
    len=$(printf "%04x" $size)
    create_seed fuzz_tls_parser "tls_valid_$i" "$(echo -ne "\x30\x82\x${len:0:2}\x${len:2:2}\x30\x81\xF0\xA0\x03\x02\x01\x02\x02\x08"; head -c 8 /dev/urandom; head -c $((size - 20)) /dev/zero)"
done

# Invalid/malformed certificates (30 seeds)
# Wrong tag (not SEQUENCE)
create_seed fuzz_tls_parser "tls_wrong_tag" "\x04\x82\x01\x00\x30\x81\xF0"

# Truncated DER
create_seed fuzz_tls_parser "tls_truncated" "\x30\x82\x01\x00"

# Invalid length encoding
create_seed fuzz_tls_parser "tls_invalid_length" "\x30\xFF\xFF\xFF\xFF"

# Generate more malformed (27 more)
for i in {1..27}; do
    seed_data=$(head -c 500 /dev/urandom | base64 | tr -d '\n' | cut -c1-500)
    echo -n "$seed_data" | base64 -d > "$TLS_DIR/tls_malformed_$i" 2>/dev/null || echo -n "$seed_data" > "$TLS_DIR/tls_malformed_$i"
done

# Edge cases (30 seeds)
# Very small certificate (100 bytes)
create_seed fuzz_tls_parser "tls_edge_small" "$(echo -ne "\x30\x82\x00\x64"; head -c 96 /dev/urandom)"

# Large certificate (4000 bytes)
create_seed fuzz_tls_parser "tls_edge_large" "$(echo -ne "\x30\x82\x0f\xa0"; head -c 3996 /dev/urandom)"

# Generate more edge cases (28 more)
for i in {1..28}; do
    size=$((500 + i * 100))
    len=$(printf "%04x" $size)
    create_seed fuzz_tls_parser "tls_edge_$i" "$(echo -ne "\x30\x82\x${len:0:2}\x${len:2:2}"; head -c $((size - 4)) /dev/urandom)"
done

echo "  ✓ Generated 100 TLS certificate corpus seeds"

# =============================================================================
# Summary
# =============================================================================
echo
echo -e "${BLUE}=== Corpus Generation Complete ===${NC}"
echo "Total seeds generated:"
echo "  - TCP:     100 seeds in $TCP_DIR"
echo "  - UDP:     80 seeds in $UDP_DIR"
echo "  - IPv6:    100 seeds in $IPV6_DIR"
echo "  - ICMPv6:  80 seeds in $ICMPV6_DIR"
echo "  - TLS:     100 seeds in $TLS_DIR"
echo "  -----------"
echo "  TOTAL:     460 seeds"
echo
echo "To run fuzzers with corpus:"
echo "  cargo +nightly fuzz run <target>"
echo
echo "Example:"
echo "  cargo +nightly fuzz run fuzz_tcp_parser"
