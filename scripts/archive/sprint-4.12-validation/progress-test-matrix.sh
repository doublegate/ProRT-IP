#!/bin/bash

echo "Progress Bar Test Matrix"
echo "========================"
echo ""

PRTIP="/home/parobek/Code/ProRT-IP/target/release/prtip"
results=()

# Test 1: Localhost fast
echo -n "Test 1 (localhost fast): "
if timeout 5 $PRTIP -p 1-1000 --progress 127.0.0.1 2>&1 | grep -q "ports"; then
    echo "VISIBLE"
    results+=(0)
else
    echo "TOO FAST (expected)"
    results+=(1)
fi

# Test 2: Remote 1K
echo -n "Test 2 (remote 1K): "
if timeout 30 $PRTIP -p 1-1000 --progress scanme.nmap.org 2>&1 | grep -q "ports"; then
    echo "VISIBLE"
    results+=(0)
else
    echo "NOT VISIBLE"
    results+=(1)
fi

# Test 3: Remote 5K
echo -n "Test 3 (remote 5K): "
if timeout 60 $PRTIP -p 1-5000 --progress scanme.nmap.org 2>&1 | grep -q "ports"; then
    echo "VISIBLE"
    results+=(0)
else
    echo "NOT VISIBLE"
    results+=(1)
fi

# Test 4: With service detection
echo -n "Test 4 (with --sV): "
if timeout 60 $PRTIP -p 1-100 --sV --progress scanme.nmap.org 2>&1 | grep -q "scanning"; then
    echo "VISIBLE"
    results+=(0)
else
    echo "NOT VISIBLE"
    results+=(1)
fi

# Test 5: Slow timing
echo -n "Test 5 (T0 timing): "
if timeout 120 $PRTIP -p 1-50 -T0 --progress scanme.nmap.org 2>&1 | grep -q "ports"; then
    echo "VISIBLE"
    results+=(0)
else
    echo "NOT VISIBLE"
    results+=(1)
fi

echo ""
echo "========================"
passed=0
for result in "${results[@]}"; do
    if [ $result -eq 0 ]; then
        ((passed++))
    fi
done
echo "Tests passed: $passed/${#results[@]}"
echo "========================"
