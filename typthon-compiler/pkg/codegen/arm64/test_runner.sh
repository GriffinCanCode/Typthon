#!/bin/bash
# Comprehensive test runner for ARM64 code generation

set -e

echo "======================================"
echo "  ARM64 Code Generation Test Suite"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Function to run a test
run_test() {
    local test_name=$1
    TESTS_TOTAL=$((TESTS_TOTAL + 1))

    echo -n "Running: $test_name ... "

    if go test -run "^${test_name}$" -v > /tmp/test_output.txt 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        cat /tmp/test_output.txt
        echo ""
    fi
}

echo "1. Unit Tests"
echo "-------------"

# Run all unit tests
run_test "TestArithmeticOperations"
run_test "TestComparisonOperations"
run_test "TestBooleanOperations"
run_test "TestFunctionCall"
run_test "TestMemoryOperations"
run_test "TestRegisterAllocation"
run_test "TestCallingConvention"
run_test "TestStackOperations"
run_test "TestBranchOperations"

echo ""
echo "2. Validator Tests"
echo "------------------"

run_test "TestValidatorValidCode"
run_test "TestValidatorInvalidRegister"
run_test "TestValidatorStackBalance"
run_test "TestValidatorCalleeSavedRegisters"
run_test "TestValidatorInvalidMemoryAddressing"
run_test "TestValidatorWithGeneratedCode"
run_test "TestValidatorComplexFunction"
run_test "TestValidatorRedundantMoves"
run_test "TestValidatorConditionalOperations"
run_test "TestValidatorBranchOperations"
run_test "TestValidatorFramePointerHandling"

echo ""
echo "3. Benchmarks"
echo "-------------"

echo -n "Running benchmarks ... "
if go test -bench=. -benchtime=100ms > /tmp/bench_output.txt 2>&1; then
    echo -e "${GREEN}DONE${NC}"
    echo ""
    grep "Benchmark" /tmp/bench_output.txt
else
    echo -e "${YELLOW}SKIPPED${NC}"
fi

echo ""
echo "======================================"
echo "  Test Summary"
echo "======================================"
echo -e "Total:  $TESTS_TOTAL"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi

