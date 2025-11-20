#!/bin/bash
# Generate test target files for CDN benchmarks

set -e

# Output files
CDN_TEST="test-data/cdn-test-targets.txt"
CDN_MIXED="test-data/cdn-mixed-targets.txt"
CDN_LARGE="test-data/cdn-large-targets.txt"

# Clear existing files
> "$CDN_TEST"
> "$CDN_MIXED"
> "$CDN_LARGE"

echo "Generating test target files..."

# Function to generate IPs from CIDR range
generate_ips() {
    local base=$1
    local count=$2
    local prefix=$3

    for i in $(seq 1 $count); do
        # Generate random last octet(s)
        local rand=$((RANDOM % 255))
        echo "${base}.${rand}"
    done
}

# Generate CDN IPs for cdn-test-targets.txt (2,500 total, 50% CDN)
echo "# CDN Test Targets (2,500 IPs, 50% CDN)" >> "$CDN_TEST"

# Cloudflare: 104.16.0.0/12 (400 IPs)
echo "# Cloudflare CDN IPs" >> "$CDN_TEST"
for i in $(seq 16 35); do
    for j in $(seq 0 19); do
        echo "104.${i}.${j}.$((RANDOM % 255))" >> "$CDN_TEST"
    done
done

# AWS CloudFront: 3.0.0.0/8 (400 IPs)
echo "# AWS CloudFront CDN IPs" >> "$CDN_TEST"
for i in $(seq 0 19); do
    for j in $(seq 0 19); do
        echo "3.${i}.${j}.$((RANDOM % 255))" >> "$CDN_TEST"
    done
done

# Azure CDN: 13.64.0.0/11 (200 IPs)
echo "# Azure CDN IPs" >> "$CDN_TEST"
for i in $(seq 64 73); do
    for j in $(seq 0 19); do
        echo "13.${i}.${j}.$((RANDOM % 255))" >> "$CDN_TEST"
    done
done

# Akamai: 23.0.0.0/8 (200 IPs)
echo "# Akamai CDN IPs" >> "$CDN_TEST"
for i in $(seq 0 9); do
    for j in $(seq 0 19); do
        echo "23.${i}.${j}.$((RANDOM % 255))" >> "$CDN_TEST"
    done
done

# Fastly: 151.101.0.0/16 (200 IPs)
echo "# Fastly CDN IPs" >> "$CDN_TEST"
for i in $(seq 0 9); do
    for j in $(seq 0 19); do
        echo "151.101.${i}.$((RANDOM % 255))" >> "$CDN_TEST"
    done
done

# Google Cloud: 34.0.0.0/8 (200 IPs)
echo "# Google Cloud CDN IPs" >> "$CDN_TEST"
for i in $(seq 0 9); do
    for j in $(seq 0 19); do
        echo "34.${i}.${j}.$((RANDOM % 255))" >> "$CDN_TEST"
    done
done

# Regular (non-CDN) IPs (900 IPs)
echo "# Regular non-CDN IPs" >> "$CDN_TEST"
for i in $(seq 1 900); do
    # Use ranges unlikely to be CDN: 192.168.x.x, 10.x.x.x, 172.16-31.x.x
    choice=$((RANDOM % 3))
    case $choice in
        0) echo "192.168.$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_TEST" ;;
        1) echo "10.$((RANDOM % 255)).$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_TEST" ;;
        2) echo "172.$((16 + RANDOM % 16)).$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_TEST" ;;
    esac
done

# Generate mixed IPv4/IPv6 targets (1,000 total)
echo "# Mixed IPv4/IPv6 CDN Targets" >> "$CDN_MIXED"

# IPv4 CDN (400)
echo "# IPv4 CDN (Cloudflare)" >> "$CDN_MIXED"
for i in $(seq 16 25); do
    for j in $(seq 0 39); do
        echo "104.${i}.${j}.$((RANDOM % 255))" >> "$CDN_MIXED"
    done
done

# IPv6 CDN (100 Cloudflare 2606:4700::/32)
echo "# IPv6 CDN (Cloudflare)" >> "$CDN_MIXED"
for i in $(seq 0 99); do
    printf "2606:4700::%x\n" $((RANDOM % 65535)) >> "$CDN_MIXED"
done

# Regular IPv4 (300)
echo "# Regular IPv4" >> "$CDN_MIXED"
for i in $(seq 1 300); do
    echo "192.168.$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_MIXED"
done

# Regular IPv6 (200)
echo "# Regular IPv6" >> "$CDN_MIXED"
for i in $(seq 0 199); do
    printf "2001:db8::%x\n" $((RANDOM % 65535)) >> "$CDN_MIXED"
done

# Generate large CDN target list (10,000 IPs, 60% CDN)
echo "# Large CDN Test (10,000 IPs, 60% CDN)" >> "$CDN_LARGE"

# Cloudflare (2,000)
echo "# Cloudflare CDN IPs (2,000)" >> "$CDN_LARGE"
for i in $(seq 16 35); do
    for j in $(seq 0 99); do
        echo "104.${i}.${j}.$((RANDOM % 255))" >> "$CDN_LARGE"
    done
done

# AWS (2,000)
echo "# AWS CloudFront CDN IPs (2,000)" >> "$CDN_LARGE"
for i in $(seq 0 19); do
    for j in $(seq 0 99); do
        echo "3.${i}.${j}.$((RANDOM % 255))" >> "$CDN_LARGE"
    done
done

# Azure (800)
echo "# Azure CDN IPs (800)" >> "$CDN_LARGE"
for i in $(seq 64 71); do
    for j in $(seq 0 99); do
        echo "13.${i}.${j}.$((RANDOM % 255))" >> "$CDN_LARGE"
    done
done

# Akamai (600)
echo "# Akamai CDN IPs (600)" >> "$CDN_LARGE"
for i in $(seq 0 5); do
    for j in $(seq 0 99); do
        echo "23.${i}.${j}.$((RANDOM % 255))" >> "$CDN_LARGE"
    done
done

# Fastly (300)
echo "# Fastly CDN IPs (300)" >> "$CDN_LARGE"
for i in $(seq 0 2); do
    for j in $(seq 0 99); do
        echo "151.101.${i}.$((RANDOM % 255))" >> "$CDN_LARGE"
    done
done

# Google Cloud (300)
echo "# Google Cloud CDN IPs (300)" >> "$CDN_LARGE"
for i in $(seq 0 2); do
    for j in $(seq 0 99); do
        echo "34.${i}.${j}.$((RANDOM % 255))" >> "$CDN_LARGE"
    done
done

# Regular IPs (4,000)
echo "# Regular non-CDN IPs (4,000)" >> "$CDN_LARGE"
for i in $(seq 1 4000); do
    choice=$((RANDOM % 3))
    case $choice in
        0) echo "192.168.$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_LARGE" ;;
        1) echo "10.$((RANDOM % 255)).$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_LARGE" ;;
        2) echo "172.$((16 + RANDOM % 16)).$((RANDOM % 255)).$((RANDOM % 255))" >> "$CDN_LARGE" ;;
    esac
done

# Count and report
echo "Generated test target files:"
echo "  $CDN_TEST: $(wc -l < "$CDN_TEST") lines"
echo "  $CDN_MIXED: $(wc -l < "$CDN_MIXED") lines"
echo "  $CDN_LARGE: $(wc -l < "$CDN_LARGE") lines"
