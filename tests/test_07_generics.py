"""Test generic types and type variables.

This module tests generic programming features.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestBasicGenerics:
    """Test basic generic functionality."""

    def test_generic_function(self, validator):
        """Test generic function definition."""
        code = """
from typing import TypeVar

T = TypeVar('T')

def identity(x: T) -> T:
    return x
"""
        assert validator.validate(code)

    def test_generic_function_call(self, validator):
        """Test calling generic function."""
        code = """
from typing import TypeVar

T = TypeVar('T')

def identity(x: T) -> T:
    return x

result: int = identity(42)
"""
        assert validator.validate(code)

    def test_generic_with_multiple_params(self, validator):
        """Test generic with multiple type parameters."""
        code = """
from typing import TypeVar

T = TypeVar('T')
U = TypeVar('U')

def pair(first: T, second: U) -> tuple[T, U]:
    return (first, second)
"""
        assert validator.validate(code)

    def test_generic_list_function(self, validator):
        """Test generic function with list."""
        code = """
from typing import TypeVar

T = TypeVar('T')

def first(items: list[T]) -> T:
    return items[0]
"""
        assert validator.validate(code)

    def test_generic_mapping_function(self, validator):
        """Test generic function with mapping."""
        code = """
from typing import TypeVar

T = TypeVar('T')
U = TypeVar('U')

def map_list(f: callable, items: list[T]) -> list[U]:
    return [f(item) for item in items]
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestGenericClasses:
    """Test generic class definitions."""

    def test_generic_class(self, validator):
        """Test generic class definition."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class Box(Generic[T]):
    def __init__(self, value: T):
        self.value = value

    def get(self) -> T:
        return self.value
"""
        assert validator.validate(code)

    def test_generic_class_instantiation(self, validator):
        """Test generic class instantiation."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class Box(Generic[T]):
    def __init__(self, value: T):
        self.value = value

box: Box[int] = Box(42)
"""
        assert validator.validate(code)

    def test_generic_class_method_call(self, validator):
        """Test calling method on generic class."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class Box(Generic[T]):
    def __init__(self, value: T):
        self.value = value

    def get(self) -> T:
        return self.value

box = Box(42)
value: int = box.get()
"""
        assert validator.validate(code)

    def test_generic_class_multiple_params(self, validator):
        """Test generic class with multiple type parameters."""
        code = """
from typing import TypeVar, Generic

K = TypeVar('K')
V = TypeVar('V')

class Pair(Generic[K, V]):
    def __init__(self, key: K, value: V):
        self.key = key
        self.value = value
"""
        assert validator.validate(code)

    def test_generic_container(self, validator):
        """Test generic container class."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class Stack(Generic[T]):
    def __init__(self):
        self.items: list[T] = []

    def push(self, item: T) -> None:
        self.items.append(item)

    def pop(self) -> T:
        return self.items.pop()
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestBoundedGenerics:
    """Test bounded type variables."""

    def test_bounded_type_var(self, validator):
        """Test bounded type variable."""
        code = """
from typing import TypeVar

T = TypeVar('T', bound=int)

def double(x: T) -> T:
    return x * 2
"""
        assert validator.validate(code)

    def test_protocol_bound(self, validator):
        """Test type variable bound to protocol."""
        code = """
from typing import TypeVar, Protocol

class Comparable(Protocol):
    def __lt__(self, other) -> bool: ...

T = TypeVar('T', bound=Comparable)

def min_value(a: T, b: T) -> T:
    return a if a < b else b
"""
        assert validator.validate(code)

    def test_class_bound(self, validator):
        """Test type variable bound to class."""
        code = """
from typing import TypeVar

class Animal:
    def speak(self) -> str:
        return "sound"

T = TypeVar('T', bound=Animal)

def make_speak(animal: T) -> str:
    return animal.speak()
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestConstrainedGenerics:
    """Test constrained type variables."""

    def test_constrained_type_var(self, validator):
        """Test constrained type variable."""
        code = """
from typing import TypeVar

T = TypeVar('T', int, float)

def add(x: T, y: T) -> T:
    return x + y
"""
        assert validator.validate(code)

    def test_constrained_function_call(self, validator):
        """Test calling function with constrained type var."""
        code = """
from typing import TypeVar

T = TypeVar('T', int, str)

def process(x: T) -> T:
    return x

result1: int = process(42)
result2: str = process("hello")
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestVariance:
    """Test variance in generics."""

    def test_covariant_type_var(self, validator):
        """Test covariant type variable."""
        code = """
from typing import TypeVar, Generic

T_co = TypeVar('T_co', covariant=True)

class ReadOnlyBox(Generic[T_co]):
    def __init__(self, value: T_co):
        self._value = value

    def get(self) -> T_co:
        return self._value
"""
        assert validator.validate(code)

    def test_contravariant_type_var(self, validator):
        """Test contravariant type variable."""
        code = """
from typing import TypeVar, Generic

T_contra = TypeVar('T_contra', contravariant=True)

class WriteOnlyBox(Generic[T_contra]):
    def put(self, value: T_contra) -> None:
        pass
"""
        assert validator.validate(code)

    def test_invariant_type_var(self, validator):
        """Test invariant type variable (default)."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class MutableBox(Generic[T]):
    def __init__(self, value: T):
        self._value = value

    def get(self) -> T:
        return self._value

    def set(self, value: T) -> None:
        self._value = value
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestGenericProtocols:
    """Test generic protocols."""

    def test_generic_protocol(self, validator):
        """Test generic protocol definition."""
        code = """
from typing import TypeVar, Protocol

T = TypeVar('T')

class Container(Protocol[T]):
    def add(self, item: T) -> None: ...
    def get(self) -> T: ...
"""
        assert validator.validate(code)

    def test_generic_protocol_implementation(self, validator):
        """Test implementing generic protocol."""
        code = """
from typing import TypeVar, Protocol

T = TypeVar('T')

class Container(Protocol[T]):
    def add(self, item: T) -> None: ...

class ListContainer:
    def __init__(self):
        self.items: list[int] = []

    def add(self, item: int) -> None:
        self.items.append(item)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestGenericInheritance:
    """Test generic class inheritance."""

    def test_inherit_generic(self, validator):
        """Test inheriting from generic class."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class Base(Generic[T]):
    def method(self, x: T) -> T:
        return x

class Derived(Base[int]):
    pass
"""
        assert validator.validate(code)

    def test_inherit_and_specialize(self, validator):
        """Test inheriting and specializing generic."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')

class Container(Generic[T]):
    def add(self, item: T) -> None:
        pass

class IntContainer(Container[int]):
    def add_multiple(self, items: list[int]) -> None:
        for item in items:
            self.add(item)
"""
        assert validator.validate(code)

    def test_multiple_generic_inheritance(self, validator):
        """Test inheriting from multiple generics."""
        code = """
from typing import TypeVar, Generic

T = TypeVar('T')
U = TypeVar('U')

class Readable(Generic[T]):
    def read(self) -> T: ...

class Writable(Generic[U]):
    def write(self, value: U) -> None: ...

class ReadWrite(Readable[T], Writable[T], Generic[T]):
    pass
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestGenericAliases:
    """Test generic type aliases."""

    def test_simple_alias(self, validator):
        """Test simple generic type alias."""
        code = """
from typing import TypeVar

T = TypeVar('T')
ListOfT = list[T]

def first(items: ListOfT) -> T:
    return items[0]
"""
        assert validator.validate(code)

    def test_complex_alias(self, validator):
        """Test complex generic type alias."""
        code = """
from typing import TypeVar

K = TypeVar('K')
V = TypeVar('V')
Mapping = dict[K, list[V]]

def get_values(m: Mapping[str, int], key: str) -> list[int]:
    return m.get(key, [])
"""
        assert validator.validate(code)

    def test_nested_alias(self, validator):
        """Test nested generic type alias."""
        code = """
Matrix = list[list[int]]

def sum_matrix(m: Matrix) -> int:
    return sum(sum(row) for row in m)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestGenericCallables:
    """Test generic callable types."""

    def test_generic_callable(self, validator):
        """Test generic callable type."""
        code = """
from typing import TypeVar, Callable

T = TypeVar('T')
U = TypeVar('U')

def apply(f: Callable[[T], U], x: T) -> U:
    return f(x)
"""
        assert validator.validate(code)

    def test_generic_higher_order(self, validator):
        """Test generic higher-order function."""
        code = """
from typing import TypeVar, Callable

T = TypeVar('T')

def compose(f: Callable[[T], T], g: Callable[[T], T]) -> Callable[[T], T]:
    def composed(x: T) -> T:
        return f(g(x))
    return composed
"""
        assert validator.validate(code)

    def test_partial_application(self, validator):
        """Test partial application with generics."""
        code = """
from typing import TypeVar, Callable

T = TypeVar('T')
U = TypeVar('U')
V = TypeVar('V')

def curry(f: Callable[[T, U], V]) -> Callable[[T], Callable[[U], V]]:
    def curried(x: T) -> Callable[[U], V]:
        def inner(y: U) -> V:
            return f(x, y)
        return inner
    return curried
"""
        assert validator.validate(code)

