#!/bin/bash
# Profile-Guided Optimization script
#
# Usage: ./scripts/profile.sh

set -e

echo "=== Typthon Profile-Guided Optimization ==="
echo

# Step 1: Build with instrumentation
echo "Step 1: Building with instrumentation..."
export RUSTFLAGS="-Cprofile-generate=/tmp/typthon-pgo"
cargo build --release
echo "✓ Instrumented build complete"
echo

# Step 2: Run representative workloads
echo "Step 2: Running representative workloads..."

# Create test project
mkdir -p /tmp/typthon-test-project
for i in {1..100}; do
    cat > "/tmp/typthon-test-project/module_$i.py" << EOF
"""Module $i for profiling."""

def function_$i(x: int, y: int) -> int:
    """Add two numbers."""
    return x + y + $i

class Class_$i:
    """Class $i."""

    def method(self, value: str) -> str:
        """Process value."""
        return value + "_processed_$i"

data_$i = [1, 2, 3, 4, 5]
result_$i = function_$i(10, 20)
EOF
done

# Run type checker on test project
echo "Running type checker..."
./target/release/typthon check /tmp/typthon-test-project/

# Run pytest benchmarks
echo "Running benchmarks..."
python3 -m pytest tests/test_benchmarks.py -v || true

echo "✓ Workload execution complete"
echo

# Step 3: Merge profile data
echo "Step 3: Merging profile data..."
if command -v llvm-profdata &> /dev/null; then
    llvm-profdata merge -o /tmp/typthon.profdata /tmp/typthon-pgo
    echo "✓ Profile data merged"
else
    echo "⚠ llvm-profdata not found, skipping merge"
    echo "Install with: brew install llvm (macOS) or apt install llvm (Linux)"
fi
echo

# Step 4: Rebuild with profile data
if [ -f "/tmp/typthon.profdata" ]; then
    echo "Step 4: Rebuilding with profile guidance..."
    unset RUSTFLAGS
    export RUSTFLAGS="-Cprofile-use=/tmp/typthon.profdata"
    cargo build --release
    echo "✓ Optimized build complete"
    echo

    echo "=== PGO Complete ==="
    echo "Optimized binary: ./target/release/typthon"
else
    echo "⚠ Skipping optimized build (no profile data)"
fi

# Cleanup
rm -rf /tmp/typthon-test-project
echo
echo "Cleaned up temporary files"

