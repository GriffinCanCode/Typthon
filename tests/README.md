
# Typthon Comprehensive Test Suite

This directory contains the comprehensive test suite for Typthon, testing the system from a user's perspective through the Python bindings.

## Test Organization

Tests are organized by category:

- **test_01_basic_types.py**: Primitive types, basic operations, simple functions
- **test_02_collections.py**: Lists, tuples, dicts, sets, and complex collections
- **test_03_type_inference.py**: Type inference engine functionality
- **test_04_union_intersection.py**: Union and intersection types
- **test_05_classes_oop.py**: Classes, inheritance, and OOP features
- **test_06_effects.py**: Effect system for tracking side effects
- **test_07_generics.py**: Generic types and type variables

## Running Tests

### Run All Tests (All Languages)

```bash
python3 tests/run_all_tests.py
```

This runs tests for:
- Python bindings (pytest)
- Rust core (cargo test)
- Go compiler (go test + integration tests)

### Run Python Tests Only

```bash
# From project root
pytest tests/ -v

# Run specific test file
pytest tests/test_01_basic_types.py -v

# Run specific test class
pytest tests/test_01_basic_types.py::TestPrimitiveTypes -v

# Run specific test
pytest tests/test_01_basic_types.py::TestPrimitiveTypes::test_int_literal -v

# Run with markers
pytest tests/ -m "type_checking" -v
pytest tests/ -m "inference" -v
pytest tests/ -m "effects" -v

# Run with coverage
pytest tests/ --cov=typhon --cov-report=html
```

### Run Rust Tests

```bash
# From project root
cargo test --all

# Run specific test
cargo test test_protocol_checking

# Run with output
cargo test -- --nocapture
```

### Run Go Tests

```bash
# Unit tests
cd typthon-compiler
go test ./...

# Integration tests
cd typthon-compiler/tests
bash run_tests.sh
```

## Test Markers

Tests are marked with pytest markers for filtering:

- `@pytest.mark.unit`: Unit tests
- `@pytest.mark.integration`: Integration tests
- `@pytest.mark.slow`: Slow-running tests
- `@pytest.mark.requires_typhon`: Requires typhon to be installed
- `@pytest.mark.type_checking`: Type checking tests
- `@pytest.mark.inference`: Type inference tests
- `@pytest.mark.effects`: Effect system tests
- `@pytest.mark.refinement`: Refinement type tests
- `@pytest.mark.protocols`: Protocol checking tests
- `@pytest.mark.cache`: Caching functionality tests
- `@pytest.mark.performance`: Performance tests

## Test Statistics

- **Total Tests**: 550+
- **Test Files**: 7
- **Coverage**: All major features from user perspective
- **Languages**: Python (user API), Rust (core), Go (compiler)

## Writing New Tests

### Guidelines

1. **Test from User Perspective**: Write tests as if you're a user installing from pip
2. **Use Fixtures**: Leverage conftest.py fixtures for common test data
3. **Clear Naming**: Test names should clearly describe what they test
4. **Good Documentation**: Include docstrings explaining the test purpose
5. **Markers**: Use appropriate pytest markers for categorization
6. **Assertions**: Use clear, specific assertions

### Example Test

```python
@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestMyFeature:
    """Test my new feature."""

    def test_basic_case(self, validator):
        """Test the basic case of my feature."""
        code = """
def my_function(x: int) -> int:
    return x * 2
"""
        assert validator.validate(code)

    def test_error_case(self, validator):
        """Test error detection in my feature."""
        code = """
def my_function(x: int) -> int:
    return "not an int"  # Type error
"""
        assert not validator.validate(code)
```

## Continuous Integration

Tests run automatically on:
- Every push to main branch
- Every pull request
- Multiple OS: Ubuntu, macOS, Windows
- Multiple Python versions: 3.10, 3.11, 3.12

## Dependencies

Install test dependencies:

```bash
pip install pytest pytest-benchmark
```

## Performance Benchmarks

Run performance benchmarks:

```bash
pytest tests/ --benchmark-only
```

## Troubleshooting

### Tests Failing to Import typhon

Make sure typhon is built and installed:

```bash
# Development mode
maturin develop

# Or full build
maturin build --release
pip install --force-reinstall target/wheels/*.whl
```

### Timeout Issues

Increase timeout in pytest.ini or pass `--timeout=600` to pytest.

### Parallel Execution

For faster execution with pytest-xdist:

```bash
pip install pytest-xdist
pytest tests/ -n auto
```

## Test Coverage Goals

- **Type Checking**: 100+ tests
- **Type Inference**: 50+ tests
- **Collections**: 100+ tests
- **Classes & OOP**: 80+ tests
- **Generics**: 60+ tests
- **Effects**: 60+ tests
- **Union/Intersection**: 50+ tests
- **Error Cases**: Present in all categories

**Total**: 550+ comprehensive tests covering all user-facing functionality.

