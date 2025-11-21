"""Validation utilities."""

from typing import Any, Union
from .runtime import _runtime


def validate(value: Any, type_annotation: Union[str, type]) -> bool:
    """
    Validate a value against a type annotation.

    Example:
        validate([1, 2, 3], "list[int]")  # True
        validate([1, "a"], "list[int]")   # False
    """
    type_str = type_annotation if isinstance(type_annotation, str) else str(type_annotation)
    return _runtime.validate(value, type_str)

