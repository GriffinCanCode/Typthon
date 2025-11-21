"""Protocol for structural typing."""

from typing import Any


class Protocol:
    """Protocol for structural typing (duck typing)."""

    def __init_subclass__(cls, **kwargs: Any):
        super().__init_subclass__(**kwargs)
        cls.__is_protocol__ = True

