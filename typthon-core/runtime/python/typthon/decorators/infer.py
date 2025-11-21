"""Type inference decorator."""

import inspect
import textwrap
from typing import Any, Callable, TypeVar

try:
    from typthon._core import infer_types
except ImportError:
    def infer_types(source: str) -> str:
        return "[Dev Mode] Type inference not available"

F = TypeVar('F', bound=Callable[..., Any])


def infer(func: F) -> F:
    """
    Decorator for automatic type inference.

    Example:
        @infer
        def process(data):
            return [x * 2 for x in data]  # Infers: (list[T]) -> list[T]
    """
    source = textwrap.dedent(inspect.getsource(func))
    inferred = infer_types(source)
    func.__typthon_inferred__ = inferred
    return func

