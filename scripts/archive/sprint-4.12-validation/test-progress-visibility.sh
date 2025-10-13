#!/bin/bash
set -e

echo "==================================="
echo "Progress Bar Visibility Test Suite"
echo "==================================="
echo ""

PRTIP="/home/parobek/Code/ProRT-IP/target/release/prtip"

# Test 1: Localhost with progress (fast - may not see bar)
echo "Test 1: Localhost 1000 ports with progress"
echo "Expected: Scan completes fast, bar may not be visible"
$PRTIP --scan-type connect -p 1-1000 --progress 127.0.0.1
echo ""
sleep 2

# Test 2: Remote host with progress (should see bar)
echo "Test 2: Remote host 1000 ports with progress"
echo "Expected: Progress bar visible during scan"
$PRTIP --scan-type connect -p 1-1000 --progress scanme.nmap.org
echo ""
sleep 2

# Test 3: Large range with progress (definitely should see bar)
echo "Test 3: Remote host 5000 ports with progress"
echo "Expected: Progress bar clearly visible"
$PRTIP --scan-type connect -p 1-5000 --progress scanme.nmap.org
echo ""
sleep 2

# Test 4: With service detection (adds time)
echo "Test 4: Remote host with service detection + progress"
echo "Expected: Progress bar shows 'Port scanning...' then 'Service detection...'"
$PRTIP --scan-type connect -p 1-500 --sV --progress scanme.nmap.org
echo ""
sleep 2

# Test 5: Slow timing (T0 - paranoid)
echo "Test 5: T0 timing with progress (very slow)"
echo "Expected: Progress bar easily visible"
$PRTIP --scan-type connect -p 1-100 -T0 --progress scanme.nmap.org
echo ""

echo "==================================="
echo "Test Suite Complete"
echo "==================================="
