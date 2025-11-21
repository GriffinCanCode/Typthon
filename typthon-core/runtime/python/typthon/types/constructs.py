"""Basic type constructs."""

from typing import Any


class Union:
    """Union type: A | B means value can be A or B."""

    def __init__(self, *types: type):
        self.types = types

    def __repr__(self) -> str:
        return " | ".join(str(t) for t in self.types)

    def __or__(self, other: type) -> "Union":
        return Union(*self.types, other)

    @classmethod
    def __class_getitem__(cls, params):
        """Make Union subscriptable: Union[int, str]"""
        if not isinstance(params, tuple):
            params = (params,)
        return cls(*params)


class Intersection:
    """Intersection type: A & B means value must satisfy both A and B."""

    def __init__(self, *types: type):
        self.types = types

    def __repr__(self) -> str:
        return " & ".join(str(t) for t in self.types)

    def __and__(self, other: type) -> "Intersection":
        return Intersection(*self.types, other)

    @classmethod
    def __class_getitem__(cls, params):
        """Make Intersection subscriptable: Intersection[int, str]"""
        if not isinstance(params, tuple):
            params = (params,)
        return cls(*params)


class Optional:
    """Optional type: Optional[T] = T | None."""

    def __init__(self, inner_type: type):
        self.inner = inner_type

    def __repr__(self) -> str:
        return f"{self.inner} | None"

    @classmethod
    def __class_getitem__(cls, param):
        """Make Optional subscriptable: Optional[int]"""
        return cls(param)


class Literal:
    """Literal type: Literal[1, 2, 3] means value must be exactly 1, 2, or 3."""

    def __init__(self, *values: Any):
        self.values = values

    def __repr__(self) -> str:
        return f"Literal[{', '.join(repr(v) for v in self.values)}]"

    @classmethod
    def __class_getitem__(cls, params):
        """Make Literal subscriptable: Literal[1, 2, 3]"""
        if not isinstance(params, tuple):
            params = (params,)
        return cls(*params)

