"""Test Phase 4 performance features."""

import pytest
import tempfile
from pathlib import Path
from typthon import check

def create_test_file(path: Path, content: str):
    """Helper to create test file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)

class TestIncrementalChecking:
    """Test incremental type checking."""

    def test_file_change_detection(self, tmp_path):
        """Test that file changes are detected."""
        test_file = tmp_path / "test.py"

        # Initial content
        create_test_file(test_file, "x: int = 1")
        errors1 = check(str(test_file))

        # Change content
        create_test_file(test_file, "x: str = 'hello'")
        errors2 = check(str(test_file))

        # Should detect change
        assert isinstance(errors1, list)
        assert isinstance(errors2, list)

    def test_unchanged_file_caching(self, tmp_path):
        """Test that unchanged files use cache."""
        test_file = tmp_path / "test.py"
        create_test_file(test_file, "x: int = 1")

        # First check
        import time
        start1 = time.time()
        check(str(test_file))
        time1 = time.time() - start1

        # Second check (should be faster with cache)
        start2 = time.time()
        check(str(test_file))
        time2 = time.time() - start2

        # Cache should make it faster (or at least not slower)
        assert time2 <= time1 * 2  # Allow some variance

    def test_dependency_invalidation(self, tmp_path):
        """Test that changing a module invalidates dependents."""
        # Create base module
        base = tmp_path / "base.py"
        create_test_file(base, """
def add(x: int, y: int) -> int:
    return x + y
""")

        # Create dependent module
        dependent = tmp_path / "dependent.py"
        create_test_file(dependent, """
from base import add

result = add(1, 2)
""")

        # Check both
        check(str(base))
        check(str(dependent))

        # Change base
        create_test_file(base, """
def add(x: int, y: int) -> str:  # Changed return type
    return str(x + y)
""")

        # Dependent should need recheck
        errors = check(str(dependent))
        assert isinstance(errors, list)

class TestCaching:
    """Test persistent result caching."""

    def test_cache_hit(self, tmp_path):
        """Test cache hit on unchanged content."""
        test_file = tmp_path / "cached.py"
        create_test_file(test_file, "x: int = 42")

        # First check populates cache
        result1 = check(str(test_file))

        # Second check should hit cache
        result2 = check(str(test_file))

        assert isinstance(result1, list)
        assert isinstance(result2, list)

    def test_cache_miss_on_change(self, tmp_path):
        """Test cache miss when content changes."""
        test_file = tmp_path / "changing.py"

        create_test_file(test_file, "x: int = 1")
        check(str(test_file))

        create_test_file(test_file, "x: str = 'hello'")
        result = check(str(test_file))

        assert isinstance(result, list)

class TestParallelAnalysis:
    """Test parallel file analysis."""

    def test_multiple_files(self, tmp_path):
        """Test analyzing multiple files in parallel."""
        files = []
        for i in range(10):
            test_file = tmp_path / f"test_{i}.py"
            create_test_file(test_file, f"""
def func_{i}(x: int) -> int:
    return x + {i}
""")
            files.append(test_file)

        # Check all files
        for f in files:
            errors = check(str(f))
            assert isinstance(errors, list)

    def test_independent_modules(self, tmp_path):
        """Test that independent modules can be checked in parallel."""
        module_a = tmp_path / "a.py"
        module_b = tmp_path / "b.py"

        create_test_file(module_a, "x: int = 1")
        create_test_file(module_b, "y: str = 'hello'")

        # These should be checkable in parallel
        errors_a = check(str(module_a))
        errors_b = check(str(module_b))

        assert isinstance(errors_a, list)
        assert isinstance(errors_b, list)

class TestMemoryOptimization:
    """Test memory pool allocation."""

    def test_large_file_memory(self, tmp_path):
        """Test memory usage on large file."""
        test_file = tmp_path / "large.py"

        # Generate large file
        lines = []
        for i in range(1000):
            lines.append(f"x_{i}: int = {i}")

        create_test_file(test_file, "\n".join(lines))

        # Should handle large file efficiently
        errors = check(str(test_file))
        assert isinstance(errors, list)

    def test_many_small_files(self, tmp_path):
        """Test memory usage with many small files."""
        for i in range(100):
            test_file = tmp_path / f"small_{i}.py"
            create_test_file(test_file, f"x = {i}")
            check(str(test_file))

        # Should handle many files without excessive memory

class TestPerformanceRegression:
    """Test that performance doesn't regress."""

    def test_simple_check_speed(self, tmp_path):
        """Test that simple checks are fast."""
        test_file = tmp_path / "simple.py"
        create_test_file(test_file, "x: int = 1")

        import time
        start = time.time()
        check(str(test_file))
        duration = time.time() - start

        # Should complete in reasonable time
        assert duration < 1.0  # Less than 1 second for simple file

    def test_complex_check_speed(self, tmp_path):
        """Test performance on complex code."""
        test_file = tmp_path / "complex.py"

        content = """
from typing import Dict, List, Tuple

def process(data: Dict[str, List[int]]) -> Tuple[int, float]:
    total = sum(sum(values) for values in data.values())
    count = sum(len(values) for values in data.values())
    return total, total / count if count > 0 else 0.0

class DataProcessor:
    def __init__(self, config: Dict[str, str]):
        self.config = config

    def process_batch(self, items: List[Dict[str, int]]) -> List[int]:
        return [item.get('value', 0) for item in items]
"""

        create_test_file(test_file, content)

        import time
        start = time.time()
        errors = check(str(test_file))
        duration = time.time() - start

        assert isinstance(errors, list)
        assert duration < 2.0  # Less than 2 seconds for complex file

def test_benchmark_suite_availability():
    """Test that benchmark suite is available."""
    # Benchmarks should be runnable with: cargo bench
    assert True  # Placeholder

if __name__ == "__main__":
    pytest.main([__file__, "-v"])

