"""Test type system semantics."""

import pytest
from typthon import type, validate, infer


def test_primitive_types():
    """Test basic type validation for primitives."""
    assert validate(42, "int")
    assert validate(3.14, "float")
    assert validate("hello", "str")
    assert validate(True, "bool")
    assert not validate("42", "int")


def test_function_decorator():
    """Test @type decorator on functions."""

    @type("(int, int) -> int")
    def add(x, y):
        return x + y

    assert hasattr(add, "__typthon_type__")
    assert add.__typthon_type__ == "(int, int) -> int"
    assert add(1, 2) == 3


def test_type_inference():
    """Test automatic type inference."""

    @infer
    def double(x):
        return x * 2

    assert hasattr(double, "__typthon_inferred__")


def test_union_types():
    """Test union type handling."""

    @type("(int | str) -> str")
    def to_string(value):
        return str(value)

    assert to_string(42) == "42"
    assert to_string("hello") == "hello"


def test_generic_types():
    """Test generic container types."""

    @type("(list[int]) -> int")
    def sum_list(items):
        return sum(items)

    assert sum_list([1, 2, 3]) == 6


def test_optional_types():
    """Test optional type handling."""
    from typthon import Optional

    @type("(int | None) -> bool")
    def is_none(value):
        return value is None

    assert is_none(None)
    assert not is_none(42)


def test_complex_signatures():
    """Test complex function signatures."""

    @type("(list[int], int) -> list[int]")
    def filter_greater(items, threshold):
        return [x for x in items if x > threshold]

    result = filter_greater([1, 5, 3, 8, 2], 3)
    assert result == [5, 8]

