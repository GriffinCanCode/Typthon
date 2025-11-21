"""Type decorator for static checking and runtime validation."""

import functools
from typing import Any, Callable, TypeVar, Union, cast

F = TypeVar('F', bound=Callable[..., Any])


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
    """
    def decorator(func: F) -> F:
        # Parse type annotation
        type_sig = annotation if isinstance(annotation, str) else str(annotation)

        # Store metadata
        func.__typthon_type__ = type_sig
        func.__typthon_runtime__ = runtime
        func.__typthon_strict__ = strict

        if not runtime:
            return func

        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            # Runtime validation
            if runtime:
                # TODO: Parse signature and validate args
                pass

            result = func(*args, **kwargs)

            # Validate return type
            if runtime:
                # TODO: Validate return value
                pass

            return result

        return cast(F, wrapper)

    return decorator

