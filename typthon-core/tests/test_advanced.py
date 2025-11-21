"""Test advanced type system features."""

import pytest
from typthon.types import Union, Intersection, Literal, Protocol, effect, dependent


def test_union_type():
    """Test union type construction."""
    u = Union(int, str)
    assert len(u.types) == 2
    assert int in u.types
    assert str in u.types


def test_intersection_type():
    """Test intersection type construction."""
    i = Intersection(int, object)
    assert len(i.types) == 2


def test_literal_type():
    """Test literal type."""
    lit = Literal(1, 2, 3)
    assert 1 in lit.values
    assert 2 in lit.values
    assert 3 in lit.values


def test_protocol():
    """Test structural typing with protocols."""

    class Drawable(Protocol):
        def draw(self) -> None:
            ...

    assert hasattr(Drawable, "__is_protocol__")


def test_effect_type():
    """Test effect type annotations."""
    Effect = effect("io", "network")
    assert hasattr(Effect, "__effects__")
    assert "io" in Effect.__effects__
    assert "network" in Effect.__effects__


def test_dependent_type():
    """Test dependent type with constraints."""
    Positive = dependent("x > 0")
    assert hasattr(Positive, "__constraint__")
    assert Positive.__constraint__ == "x > 0"

