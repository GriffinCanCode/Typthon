"""Validation utilities."""

from typing import Any, Union
import json
from .runtime import _runtime

try:
    from typthon._core import validate_refinement, TypeValidator
    _has_core = True
except ImportError:
    _has_core = False
    TypeValidator = None
    validate_refinement = None


def validate(value: Any, type_annotation: Union[str, type]) -> bool:
    """
    Validate a value against a type annotation.

    Example:
        validate([1, 2, 3], "list[int]")  # True
        validate([1, "a"], "list[int]")   # False
    """
    type_str = type_annotation if isinstance(type_annotation, str) else str(type_annotation)
    return _runtime.validate(value, type_str)


def validate_effect_type(value: Any, base_type: type, effects: set[str]) -> bool:
    """
    Validate a value against an effect type.

    Args:
        value: Value to validate
        base_type: Base type (e.g., int, str)
        effects: Set of effects (e.g., {"io", "async"})

    Returns:
        True if value matches base type (effects checked at call site)
    """
    # Base type validation
    if not isinstance(value, base_type):
        return False
    return True


def validate_refinement_type(value: Any, base_type: type, predicate: str) -> bool:
    """
    Validate a value against a refinement type using Rust validator.

    Args:
        value: Value to validate
        base_type: Base type (e.g., int, str)
        predicate: Predicate string (e.g., "value > 0")

    Returns:
        True if value satisfies refinement
    """
    # First check base type
    if not isinstance(value, base_type):
        return False

    # Use Rust validator if available
    if _has_core and validate_refinement:
        try:
            json_value = json.dumps(value)
            return validate_refinement(json_value, predicate)
        except Exception:
            pass

    # Fallback to Python eval
    try:
        namespace = {'value': value, 'len': len, 'abs': abs}
        return eval(predicate, namespace)
    except Exception:
        return False


def validate_dependent_type(value: Any, base_type: type, constraint: str) -> bool:
    """
    Validate a value against a dependent type.

    Args:
        value: Value to validate
        base_type: Base type (e.g., list, str)
        constraint: Constraint string (e.g., "len=5", "0<=len<=10")

    Returns:
        True if value satisfies constraint
    """
    if not isinstance(value, base_type):
        return False

    # Parse constraint
    if constraint.startswith("len="):
        expected_len = int(constraint[4:])
        return len(value) == expected_len
    elif "<=len<=" in constraint:
        parts = constraint.split("<=len<=")
        min_len, max_len = int(parts[0]), int(parts[1])
        return min_len <= len(value) <= max_len

    return True

