"""Type decorator for static checking and runtime validation."""

import functools
import warnings
from typing import Any, Callable, TypeVar, Union, cast

from ..core.signature_parser import parse_signature, ParsedType, TypeSignature
from ..core.validator import (
    validate_refinement_type,
    validate_effect_type,
    validate_dependent_type
)
from ..types.effects import EffectType, RefinementType, DependentType

F = TypeVar('F', bound=Callable[..., Any])


class TypeValidationError(TypeError):
    """Raised when runtime type validation fails."""
    pass


def type(annotation: Union[str, type], *, runtime: bool = True, strict: bool = False) -> Callable[[F], F]:
    """
    Decorator for type-annotating functions with static checking and optional runtime validation.

    Args:
        annotation: Type signature like "(int, int) -> int" or a Python type
        runtime: Enable runtime validation
        strict: Raise on type errors (vs warn)

    Example:
        @type("(int, int) -> int")
        def add(x, y):
            return x + y

        @type("(int) -> int ! {io}")
        def read_number():
            return int(input())

        @type("(int[value > 0]) -> int[value > 0]")
        def square_positive(x):
            return x * x
    """
    def decorator(func: F) -> F:
        # Parse type annotation
        type_sig_str = annotation if isinstance(annotation, str) else str(annotation)

        # Parse signature
        try:
            sig = parse_signature(type_sig_str)
        except Exception as e:
            warnings.warn(f"Failed to parse type signature '{type_sig_str}': {e}")
            return func

        # Store metadata
        func.__typthon_type__ = type_sig_str
        func.__typthon_runtime__ = runtime
        func.__typthon_strict__ = strict
        func.__typthon_signature__ = sig

        if not runtime:
            return func

        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            # Validate arguments
            if runtime:
                try:
                    validate_args(sig.params, args)
                except TypeValidationError as e:
                    if strict:
                        raise
                    warnings.warn(f"Type error in {func.__name__}: {e}")

            # Call function
            result = func(*args, **kwargs)

            # Validate return type
            if runtime:
                try:
                    validate_return(sig.return_type, result)
                except TypeValidationError as e:
                    if strict:
                        raise
                    warnings.warn(f"Return type error in {func.__name__}: {e}")

            return result

        # Preserve signature info
        wrapper.__typthon_type__ = type_sig_str
        wrapper.__typthon_signature__ = sig

        return cast(F, wrapper)

    return decorator


def validate_args(param_types: list[ParsedType], args: tuple) -> None:
    """Validate function arguments against parameter types."""
    if len(args) != len(param_types):
        raise TypeValidationError(
            f"Expected {len(param_types)} arguments, got {len(args)}"
        )

    for i, (param_type, arg) in enumerate(zip(param_types, args)):
        try:
            validate_value(arg, param_type, f"argument {i}")
        except TypeValidationError as e:
            raise TypeValidationError(f"Parameter {i}: {e}") from None


def validate_return(return_type: ParsedType, value: Any) -> None:
    """Validate return value against return type."""
    validate_value(value, return_type, "return value")


def validate_value(value: Any, parsed_type: ParsedType, context: str = "value") -> None:
    """Validate a value against a parsed type."""
    # Handle refinement types
    if parsed_type.is_refinement and parsed_type.predicate:
        base_type = get_python_type(parsed_type.base_type)
        if not validate_refinement_type(value, base_type, parsed_type.predicate):
            raise TypeValidationError(
                f"{context} failed refinement: {parsed_type}"
            )
        return

    # Handle effect types
    if parsed_type.is_effect:
        base_type = get_python_type(parsed_type.base_type)
        if not validate_effect_type(value, base_type, set(parsed_type.effects)):
            raise TypeValidationError(
                f"{context} type mismatch for effect type: expected {parsed_type.base_type}, got {type(value).__name__}"
            )
        return

    # Handle Union types
    if parsed_type.base_type == 'Union':
        for arg_type in parsed_type.args:
            try:
                validate_value(value, arg_type, context)
                return  # Valid for at least one type in union
            except TypeValidationError:
                continue
        raise TypeValidationError(
            f"{context} does not match any type in union: {parsed_type}"
        )

    # Handle generic types
    if parsed_type.args:
        if parsed_type.base_type in ('list', 'List'):
            if not isinstance(value, list):
                raise TypeValidationError(
                    f"{context} must be list, got {type(value).__name__}"
                )
            if parsed_type.args and value:
                elem_type = parsed_type.args[0]
                for i, elem in enumerate(value):
                    validate_value(elem, elem_type, f"{context}[{i}]")
            return

        elif parsed_type.base_type in ('dict', 'Dict'):
            if not isinstance(value, dict):
                raise TypeValidationError(
                    f"{context} must be dict, got {type(value).__name__}"
                )
            if len(parsed_type.args) >= 2 and value:
                key_type, val_type = parsed_type.args[0], parsed_type.args[1]
                for k, v in value.items():
                    validate_value(k, key_type, f"{context} key")
                    validate_value(v, val_type, f"{context}[{k}]")
            return

        elif parsed_type.base_type in ('tuple', 'Tuple'):
            if not isinstance(value, tuple):
                raise TypeValidationError(
                    f"{context} must be tuple, got {type(value).__name__}"
                )
            if parsed_type.args and len(value) == len(parsed_type.args):
                for i, (elem, elem_type) in enumerate(zip(value, parsed_type.args)):
                    validate_value(elem, elem_type, f"{context}[{i}]")
            return

        elif parsed_type.base_type in ('set', 'Set'):
            if not isinstance(value, set):
                raise TypeValidationError(
                    f"{context} must be set, got {type(value).__name__}"
                )
            if parsed_type.args and value:
                elem_type = parsed_type.args[0]
                for elem in value:
                    validate_value(elem, elem_type, f"{context} element")
            return

    # Handle basic types
    expected_type = get_python_type(parsed_type.base_type)
    if expected_type == Any:
        return  # Any type accepts everything

    if not isinstance(value, expected_type):
        raise TypeValidationError(
            f"{context} type mismatch: expected {parsed_type.base_type}, got {type(value).__name__}"
        )


def get_python_type(type_name: str) -> type:
    """Convert type name string to Python type."""
    type_map = {
        'int': int,
        'float': float,
        'str': str,
        'bool': bool,
        'bytes': bytes,
        'None': type(None),
        'list': list,
        'List': list,
        'tuple': tuple,
        'Tuple': tuple,
        'dict': dict,
        'Dict': dict,
        'set': set,
        'Set': set,
        'frozenset': frozenset,
        'Any': Any,
    }
    return type_map.get(type_name, Any)

