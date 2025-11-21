"""Test type inference engine."""

import pytest
from typthon import infer


def test_infer_simple_function():
    """Test inference on simple function."""

    @infer
    def identity(x):
        return x

    assert hasattr(identity, "__typthon_inferred__")


def test_infer_list_comprehension():
    """Test inference on list comprehension."""

    @infer
    def doubles(items):
        return [x * 2 for x in items]

    # Should infer (list[T]) -> list[T]
    assert doubles([1, 2, 3]) == [2, 4, 6]


def test_infer_dict_operations():
    """Test inference with dict operations."""

    @infer
    def swap_dict(d):
        return {v: k for k, v in d.items()}

    result = swap_dict({"a": 1, "b": 2})
    assert result == {1: "a", 2: "b"}


def test_infer_nested_functions():
    """Test inference with nested functions."""

    @infer
    def outer(x):
        def inner(y):
            return x + y
        return inner(10)

    assert outer(5) == 15

