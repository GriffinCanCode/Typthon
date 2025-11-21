#!/bin/bash
# Test runner for Phase 1 compiler

set -e

COMPILER="../bin/typthon"
TEST_DIR="."

echo "Building compiler..."
cd ..
make build
cd tests

echo ""
echo "Running tests..."
echo "===================="

# Test 1: Simple return
echo "Test 1: Simple return (expected: 42)"
$COMPILER compile test_simple.py -o test_simple
./test_simple
result=$?
echo "Result: $result"
if [ $result -eq 42 ]; then
    echo "✓ PASS"
else
    echo "✗ FAIL (expected 42, got $result)"
fi
rm -f test_simple
echo ""

# Test 2: Addition
echo "Test 2: Addition (expected: 8)"
$COMPILER compile test_add.py -o test_add
./test_add
result=$?
echo "Result: $result"
if [ $result -eq 8 ]; then
    echo "✓ PASS"
else
    echo "✗ FAIL (expected 8, got $result)"
fi
rm -f test_add
echo ""

# Test 3: Arithmetic
echo "Test 3: Arithmetic (expected: 19)"
$COMPILER compile test_arithmetic.py -o test_arithmetic
./test_arithmetic
result=$?
echo "Result: $result"
if [ $result -eq 19 ]; then
    echo "✓ PASS"
else
    echo "✗ FAIL (expected 19, got $result)"
fi
rm -f test_arithmetic
echo ""

echo "===================="
echo "Tests complete!"

