"""Pytest configuration."""

import pytest


@pytest.fixture
def sample_code():
    """Provide sample code for testing."""
    return """
def add(x: int, y: int) -> int:
    return x + y

def greet(name: str) -> str:
    return f"Hello, {name}!"
"""


@pytest.fixture
def complex_code():
    """Provide complex code for testing."""
    return """
from typing import List, Dict, Optional

def process_data(items: List[int]) -> Dict[str, int]:
    result = {}
    for i, item in enumerate(items):
        result[f"item_{i}"] = item * 2
    return result

def find_max(items: List[int]) -> Optional[int]:
    if not items:
        return None
    return max(items)
"""

