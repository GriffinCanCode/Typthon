"""Unified test runner for Typthon across all languages.

This script runs tests for:
- Python bindings (pytest)
- Rust core (cargo test)
- Go compiler (go test)
"""

import subprocess
import sys
import time
from pathlib import Path
from typing import NamedTuple


class TestResult(NamedTuple):
    """Test result container."""
    name: str
    passed: bool
    duration: float
    output: str


def run_python_tests() -> TestResult:
    """Run Python tests via pytest."""
    print("=" * 70)
    print("Running Python Tests (pytest)")
    print("=" * 70)

    start = time.time()
    try:
        result = subprocess.run(
            ["pytest", "tests/", "-v", "--tb=short"],
            capture_output=True,
            text=True,
            timeout=300
        )
        duration = time.time() - start
        passed = result.returncode == 0
        output = result.stdout + result.stderr
        print(output)
        return TestResult("Python (pytest)", passed, duration, output)
    except subprocess.TimeoutExpired:
        duration = time.time() - start
        return TestResult("Python (pytest)", False, duration, "TIMEOUT")
    except FileNotFoundError:
        return TestResult("Python (pytest)", False, 0, "pytest not found")


def run_rust_tests() -> TestResult:
    """Run Rust tests via cargo test."""
    print("\n" + "=" * 70)
    print("Running Rust Tests (cargo test)")
    print("=" * 70)

    start = time.time()
    try:
        result = subprocess.run(
            ["cargo", "test", "--all"],
            capture_output=True,
            text=True,
            timeout=600
        )
        duration = time.time() - start
        passed = result.returncode == 0
        output = result.stdout + result.stderr
        print(output)
        return TestResult("Rust (cargo)", passed, duration, output)
    except subprocess.TimeoutExpired:
        duration = time.time() - start
        return TestResult("Rust (cargo)", False, duration, "TIMEOUT")
    except FileNotFoundError:
        return TestResult("Rust (cargo)", False, 0, "cargo not found")


def run_go_tests() -> TestResult:
    """Run Go compiler tests."""
    print("\n" + "=" * 70)
    print("Running Go Compiler Tests (go test)")
    print("=" * 70)

    compiler_dir = Path(__file__).parent.parent / "typthon-compiler"
    if not compiler_dir.exists():
        return TestResult("Go (compiler)", False, 0, "Compiler directory not found")

    start = time.time()
    try:
        result = subprocess.run(
            ["go", "test", "./..."],
            cwd=compiler_dir,
            capture_output=True,
            text=True,
            timeout=300
        )
        duration = time.time() - start
        passed = result.returncode == 0
        output = result.stdout + result.stderr
        print(output)
        return TestResult("Go (compiler)", passed, duration, output)
    except subprocess.TimeoutExpired:
        duration = time.time() - start
        return TestResult("Go (compiler)", False, duration, "TIMEOUT")
    except FileNotFoundError:
        return TestResult("Go (compiler)", False, 0, "go not found")


def run_go_integration_tests() -> TestResult:
    """Run Go compiler integration tests."""
    print("\n" + "=" * 70)
    print("Running Go Integration Tests (bash)")
    print("=" * 70)

    test_script = Path(__file__).parent.parent / "typthon-compiler" / "tests" / "run_tests.sh"
    if not test_script.exists():
        return TestResult("Go (integration)", False, 0, "Test script not found")

    start = time.time()
    try:
        result = subprocess.run(
            ["bash", str(test_script)],
            capture_output=True,
            text=True,
            timeout=180
        )
        duration = time.time() - start
        passed = result.returncode == 0
        output = result.stdout + result.stderr
        print(output)
        return TestResult("Go (integration)", passed, duration, output)
    except subprocess.TimeoutExpired:
        duration = time.time() - start
        return TestResult("Go (integration)", False, duration, "TIMEOUT")


def print_summary(results: list[TestResult]) -> None:
    """Print test summary."""
    print("\n" + "=" * 70)
    print("TEST SUMMARY")
    print("=" * 70)

    total_duration = sum(r.duration for r in results)
    passed_count = sum(1 for r in results if r.passed)
    total_count = len(results)

    print(f"\nTotal Tests: {total_count}")
    print(f"Passed: {passed_count}")
    print(f"Failed: {total_count - passed_count}")
    print(f"Total Duration: {total_duration:.2f}s\n")

    for result in results:
        status = "✓ PASS" if result.passed else "✗ FAIL"
        print(f"  {status:8} {result.name:30} ({result.duration:.2f}s)")

    print("\n" + "=" * 70)

    if passed_count == total_count:
        print("🎉 ALL TESTS PASSED!")
    else:
        print(f"❌ {total_count - passed_count} TEST SUITE(S) FAILED")
    print("=" * 70)


def main() -> int:
    """Run all tests and return exit code."""
    print("""
╔═══════════════════════════════════════════════════════════════════════╗
║                    TYPTHON COMPREHENSIVE TEST SUITE                   ║
║                                                                       ║
║  Testing across multiple languages:                                  ║
║    - Python bindings (user-facing API)                              ║
║    - Rust core (type checker & runtime)                             ║
║    - Go compiler (native code generation)                           ║
╚═══════════════════════════════════════════════════════════════════════╝
""")

    results = []

    # Run Python tests (most important - user-facing)
    results.append(run_python_tests())

    # Run Rust tests (core functionality)
    results.append(run_rust_tests())

    # Run Go tests (compiler)
    results.append(run_go_tests())
    results.append(run_go_integration_tests())

    # Print summary
    print_summary(results)

    # Return non-zero if any tests failed
    return 0 if all(r.passed for r in results) else 1


if __name__ == "__main__":
    sys.exit(main())

