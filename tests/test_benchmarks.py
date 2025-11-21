"""Performance benchmarks for Typthon."""

import pytest
from typthon import type, infer, validate, check
from pathlib import Path


@pytest.fixture
def benchmark_code():
    """Generate code for benchmarking."""
    return """
def add(x: int, y: int) -> int:
    return x + y

def process(items: list[int]) -> list[int]:
    return [x * 2 for x in items if x > 0]
""" * 100  # Repeat for larger benchmark


def test_benchmark_validate_int(benchmark):
    """Benchmark integer validation."""
    result = benchmark(validate, 42, "int")
    assert result


def test_benchmark_validate_str(benchmark):
    """Benchmark string validation."""
    result = benchmark(validate, "hello", "str")
    assert result


def test_benchmark_validate_list(benchmark):
    """Benchmark list validation."""
    data = list(range(100))
    result = benchmark(validate, data, "list")
    assert result


def test_benchmark_type_decorator(benchmark):
    """Benchmark type decorator overhead."""

    @type("(int, int) -> int")
    def add(x, y):
        return x + y

    result = benchmark(add, 1, 2)
    assert result == 3


def test_benchmark_inference(benchmark):
    """Benchmark type inference."""

    def to_infer(items):
        return [x * 2 for x in items]

    result = benchmark(infer, to_infer)
    assert result is not None


def test_benchmark_check_file(benchmark, tmp_path: Path, benchmark_code):
    """Benchmark static checking on large file."""
    test_file = tmp_path / "benchmark.py"
    test_file.write_text(benchmark_code)

    result = benchmark(check, str(test_file))
    assert isinstance(result, list)


def test_benchmark_union_types(benchmark):
    """Benchmark union type validation."""

    @type("(int | str) -> str")
    def convert(value):
        return str(value)

    result = benchmark(convert, 42)
    assert result == "42"


def test_benchmark_nested_generics(benchmark):
    """Benchmark nested generic types."""

    @type("(dict[str, list[int]]) -> int")
    def sum_nested(data):
        return sum(sum(v) for v in data.values())

    data = {f"key_{i}": list(range(10)) for i in range(10)}
    result = benchmark(sum_nested, data)
    assert result == 450


def test_benchmark_complex_signature(benchmark):
    """Benchmark complex function signature."""

    @type("((int) -> int, list[int], int) -> list[int]")
    def map_filter(f, items, threshold):
        return [f(x) for x in items if x > threshold]

    result = benchmark(map_filter, lambda x: x * 2, list(range(100)), 50)
    assert len(result) == 49


# Comparative benchmarks (when mypy is available)
def test_compare_with_mypy(benchmark, tmp_path: Path, benchmark_code):
    """Compare performance with mypy (if available)."""
    pytest.importorskip("mypy")

    test_file = tmp_path / "mypy_test.py"
    test_file.write_text(benchmark_code)

    # Typthon check
    typthon_result = benchmark(check, str(test_file))
    
    # Note: In real benchmarks, would also run mypy for comparison
    assert isinstance(typthon_result, list)

