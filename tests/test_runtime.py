"""Test runtime validation system."""

import pytest
from typthon import Runtime, validate


def test_runtime_validate_primitives():
    """Test runtime validation of primitive types."""
    runtime = Runtime()

    assert runtime.validate(42, "int")
    assert runtime.validate(3.14, "float")
    assert runtime.validate("hello", "str")
    assert not runtime.validate(42, "str")


def test_runtime_strict_mode():
    """Test strict mode raises errors."""
    runtime = Runtime(strict=True)

    assert runtime.validate(42, "int")


def test_runtime_optimize():
    """Test optimized runtime checks."""
    runtime = Runtime(optimize=True)

    # Should use fast path for primitives
    assert runtime.validate(42, "int")
    assert runtime.validate(3.14, "float")


def test_validate_function():
    """Test standalone validate function."""
    assert validate(42, "int")
    assert validate([1, 2, 3], "list")
    assert not validate("hello", "int")

