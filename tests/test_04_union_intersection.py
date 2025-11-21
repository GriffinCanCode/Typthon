"""Test union and intersection types.

This module tests advanced type system features including unions and intersections.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestUnionTypes:
    """Test union type checking."""

    def test_simple_union(self, validator):
        """Test simple union type."""
        code = """
from typing import Union
x: Union[int, str] = 42
"""
        assert validator.validate(code)

    def test_union_str_alternative(self, validator):
        """Test union with string value."""
        code = """
from typing import Union
x: Union[int, str] = "hello"
"""
        assert validator.validate(code)

    def test_union_type_error(self, validator):
        """Test union type mismatch."""
        code = """
from typing import Union
x: Union[int, str] = 3.14  # Type error
"""
        assert not validator.validate(code)

    def test_union_with_none(self, validator):
        """Test union with None (Optional)."""
        code = """
from typing import Union
x: Union[int, None] = None
"""
        assert validator.validate(code)

    def test_optional_type(self, validator):
        """Test Optional type."""
        code = """
from typing import Optional
x: Optional[int] = None
"""
        assert validator.validate(code)

    def test_optional_with_value(self, validator):
        """Test Optional with actual value."""
        code = """
from typing import Optional
x: Optional[int] = 42
"""
        assert validator.validate(code)

    def test_union_in_function_param(self, validator):
        """Test union in function parameter."""
        code = """
from typing import Union

def process(value: Union[int, str]) -> str:
    if isinstance(value, int):
        return str(value)
    return value
"""
        assert validator.validate(code)

    def test_union_in_function_return(self, validator):
        """Test union in function return type."""
        code = """
from typing import Union

def get_value(flag: bool) -> Union[int, str]:
    if flag:
        return 42
    return "hello"
"""
        assert validator.validate(code)

    def test_union_of_collections(self, validator):
        """Test union of collection types."""
        code = """
from typing import Union
x: Union[list[int], tuple[int, ...]] = [1, 2, 3]
"""
        assert validator.validate(code)

    def test_nested_unions(self, validator):
        """Test nested union types."""
        code = """
from typing import Union
x: Union[int, Union[str, float]] = 3.14
"""
        assert validator.validate(code)

    def test_union_three_types(self, validator):
        """Test union of three types."""
        code = """
from typing import Union
x: Union[int, str, float] = "hello"
"""
        assert validator.validate(code)

    def test_union_with_list(self, validator):
        """Test union with list type."""
        code = """
from typing import Union
x: Union[int, list[int]] = [1, 2, 3]
"""
        assert validator.validate(code)

    def test_union_type_narrowing(self, validator):
        """Test type narrowing with isinstance."""
        code = """
from typing import Union

def process(value: Union[int, str]) -> int:
    if isinstance(value, int):
        return value * 2
    else:
        return len(value)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestIntersectionTypes:
    """Test intersection type checking."""

    def test_protocol_intersection(self, validator):
        """Test intersection of protocols."""
        code = """
from typing import Protocol

class Readable(Protocol):
    def read(self) -> str: ...

class Writable(Protocol):
    def write(self, data: str) -> None: ...

def process(obj: Readable & Writable) -> None:
    data = obj.read()
    obj.write(data)
"""
        assert validator.validate(code)

    def test_intersection_with_concrete(self, validator):
        """Test intersection with concrete type."""
        code = """
from typing import Protocol

class Hashable(Protocol):
    def __hash__(self) -> int: ...

def process(value: int & Hashable) -> int:
    return hash(value)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestComplexUnions:
    """Test complex union scenarios."""

    def test_union_in_dict_values(self, validator):
        """Test union in dict value types."""
        code = """
from typing import Union
data: dict[str, Union[int, str]] = {
    'age': 25,
    'name': 'Alice'
}
"""
        assert validator.validate(code)

    def test_union_in_list(self, validator):
        """Test union in list element type."""
        code = """
from typing import Union
items: list[Union[int, str]] = [1, 'hello', 2, 'world']
"""
        assert validator.validate(code)

    def test_union_with_callable(self, validator):
        """Test union with callable type."""
        code = """
from typing import Union, Callable

def process(value: Union[int, Callable[[], int]]) -> int:
    if callable(value):
        return value()
    return value
"""
        assert validator.validate(code)

    def test_discriminated_union(self, validator):
        """Test discriminated union pattern."""
        code = """
from typing import Union, Literal

class Success:
    tag: Literal['success'] = 'success'
    value: int

class Failure:
    tag: Literal['failure'] = 'failure'
    error: str

Result = Union[Success, Failure]
"""
        assert validator.validate(code)

    def test_union_exhaustiveness(self, validator):
        """Test exhaustive union handling."""
        code = """
from typing import Union

def handle(value: Union[int, str, bool]) -> str:
    if isinstance(value, int):
        return f"int: {value}"
    elif isinstance(value, str):
        return f"str: {value}"
    elif isinstance(value, bool):
        return f"bool: {value}"
    else:
        raise ValueError("Unexpected type")
"""
        assert validator.validate(code)

    def test_union_with_generic(self, validator):
        """Test union with generic types."""
        code = """
from typing import Union, TypeVar, Generic

T = TypeVar('T')

class Container(Generic[T]):
    value: T

def process(x: Union[Container[int], Container[str]]) -> None:
    pass
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestUnionNarrowing:
    """Test type narrowing for unions."""

    def test_isinstance_narrowing(self, validator):
        """Test isinstance narrows union."""
        code = """
from typing import Union

def process(value: Union[int, str]) -> str:
    if isinstance(value, int):
        # value is int here
        doubled = value * 2
        return str(doubled)
    # value is str here
    return value.upper()
"""
        assert validator.validate(code)

    def test_none_check_narrowing(self, validator):
        """Test None check narrows Optional."""
        code = """
from typing import Optional

def process(value: Optional[int]) -> int:
    if value is None:
        return 0
    # value is int here
    return value * 2
"""
        assert validator.validate(code)

    def test_truthiness_narrowing(self, validator):
        """Test truthiness check narrows Optional."""
        code = """
from typing import Optional

def process(value: Optional[str]) -> str:
    if value:
        # value is str here
        return value.upper()
    return ""
"""
        assert validator.validate(code)

    def test_hasattr_narrowing(self, validator):
        """Test hasattr narrows union."""
        code = """
from typing import Union

class A:
    x: int

class B:
    y: str

def process(obj: Union[A, B]) -> None:
    if hasattr(obj, 'x'):
        # obj is A here
        value = obj.x
"""
        assert validator.validate(code)

    def test_comparison_narrowing(self, validator):
        """Test comparison narrows union."""
        code = """
from typing import Union

def process(value: Union[int, str]) -> int:
    if value == 0:
        # value is int here
        return value
    elif isinstance(value, str):
        return len(value)
    else:
        return value
"""
        assert validator.validate(code)

    def test_match_statement_narrowing(self, validator):
        """Test match statement narrows union."""
        code = """
from typing import Union

def process(value: Union[int, str, list]) -> str:
    match value:
        case int(x):
            return f"int: {x}"
        case str(s):
            return f"str: {s}"
        case list(items):
            return f"list of {len(items)} items"
        case _:
            return "unknown"
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestUnionEquivalence:
    """Test union type equivalence."""

    def test_union_order_irrelevant(self, validator):
        """Test that union order doesn't matter."""
        code = """
from typing import Union
x: Union[int, str] = 42
y: Union[str, int] = 42
"""
        assert validator.validate(code)

    def test_union_deduplication(self, validator):
        """Test that duplicate types in union are deduplicated."""
        code = """
from typing import Union
x: Union[int, int, str] = 42
"""
        assert validator.validate(code)

    def test_nested_union_flattening(self, validator):
        """Test that nested unions are flattened."""
        code = """
from typing import Union
x: Union[int, Union[str, float]] = "hello"
"""
        assert validator.validate(code)

    def test_never_in_union(self, validator):
        """Test that Never in union is eliminated."""
        code = """
from typing import Union, Never
x: Union[int, Never] = 42
"""
        assert validator.validate(code)

