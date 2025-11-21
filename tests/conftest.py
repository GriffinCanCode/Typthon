"""Global pytest configuration for Typthon test suite.

This configuration provides fixtures and utilities for testing Typthon
from a user's perspective (via Python bindings).
"""

import sys
import tempfile
from pathlib import Path
from typing import Generator

import pytest

# Try to import typthon - gracefully handle if not installed
try:
    import typhon
    TYPHON_AVAILABLE = True
except ImportError:
    TYPHON_AVAILABLE = False
    typhon = None


def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line("markers", "slow: mark test as slow")
    config.addinivalue_line("markers", "integration: mark test as integration test")
    config.addinivalue_line("markers", "unit: mark test as unit test")
    config.addinivalue_line("markers", "requires_typhon: test requires typhon to be installed")
    config.addinivalue_line("markers", "type_checking: tests type checking functionality")
    config.addinivalue_line("markers", "inference: tests type inference")
    config.addinivalue_line("markers", "effects: tests effect system")
    config.addinivalue_line("markers", "refinement: tests refinement types")
    config.addinivalue_line("markers", "protocols: tests protocol checking")
    config.addinivalue_line("markers", "cache: tests caching functionality")
    config.addinivalue_line("markers", "performance: tests performance characteristics")


def pytest_collection_modifyitems(config, items):
    """Automatically skip tests requiring typhon if not available."""
    skip_typhon = pytest.mark.skip(reason="typhon not installed")
    for item in items:
        if "requires_typhon" in item.keywords and not TYPHON_AVAILABLE:
            item.add_marker(skip_typhon)


@pytest.fixture
def temp_file(tmp_path: Path) -> Generator[Path, None, None]:
    """Create a temporary Python file for testing."""
    file_path = tmp_path / "test_module.py"
    yield file_path
    if file_path.exists():
        file_path.unlink()


@pytest.fixture
def temp_dir(tmp_path: Path) -> Path:
    """Provide a temporary directory for testing."""
    return tmp_path


@pytest.fixture
def write_file(temp_file: Path):
    """Factory fixture for writing test Python code to a file."""
    def _write(code: str) -> Path:
        temp_file.write_text(code)
        return temp_file
    return _write


@pytest.fixture
def validator():
    """Provide a TypeValidator instance."""
    if not TYPHON_AVAILABLE:
        pytest.skip("typhon not installed")
    return typhon.TypeValidator()


@pytest.fixture
def simple_valid_code() -> str:
    """Simple valid Python code."""
    return """
def add(x: int, y: int) -> int:
    return x + y

result: int = add(1, 2)
"""


@pytest.fixture
def simple_invalid_code() -> str:
    """Simple invalid Python code with type error."""
    return """
def add(x: int, y: int) -> int:
    return x + y

result: int = add("hello", "world")  # Type error
"""


@pytest.fixture
def class_code() -> str:
    """Python code with class definition."""
    return """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def distance(self) -> float:
        return (self.x ** 2 + self.y ** 2) ** 0.5

p = Point(3, 4)
dist: float = p.distance()
"""


@pytest.fixture
def function_with_effects() -> str:
    """Python code with side effects."""
    return """
def read_file(path: str) -> str:
    with open(path) as f:
        return f.read()

def process(data: str) -> int:
    return len(data)
"""


@pytest.fixture
def union_type_code() -> str:
    """Python code with union types."""
    return """
from typing import Union

def process(value: Union[int, str]) -> str:
    if isinstance(value, int):
        return str(value)
    return value
"""


@pytest.fixture
def generic_code() -> str:
    """Python code with generic types."""
    return """
from typing import List, Dict, TypeVar

T = TypeVar('T')

def first(items: List[T]) -> T:
    return items[0]

def count_by_type(items: List[str]) -> Dict[str, int]:
    result: Dict[str, int] = {}
    for item in items:
        result[item] = result.get(item, 0) + 1
    return result
"""


# Test categories for organization
TEST_CATEGORIES = [
    "basic",
    "types",
    "inference",
    "effects",
    "refinement",
    "protocols",
    "generics",
    "unions",
    "classes",
    "functions",
    "errors",
    "performance",
    "integration",
]

