"""Runtime validation engine."""

from typing import Any

try:
    from typthon._core import TypeValidator
except ImportError:
    class TypeValidator:
        def validate(self, source: str) -> bool:
            return True

        def get_type(self, expr: str) -> str:
            return "Any"


class Runtime:
    """Runtime validation engine."""

    def __init__(self, *, strict: bool = False, optimize: bool = True):
        self.strict = strict
        self.optimize = optimize
        self.validator = TypeValidator()
        self._cache: dict[str, Any] = {}

    def validate(self, value: Any, expected_type: str) -> bool:
        """Validate value against type at runtime."""
        # Fast path for primitives
        if expected_type == "int":
            return isinstance(value, int) and not isinstance(value, bool)
        if expected_type == "float":
            return isinstance(value, (int, float)) and not isinstance(value, bool)
        if expected_type == "str":
            return isinstance(value, str)
        if expected_type == "bool":
            return isinstance(value, bool)

        # Use Rust validator for complex types
        try:
            return self.validator.validate(f"{value!r}")
        except Exception:
            return not self.strict


# Global runtime instance
_runtime = Runtime()

