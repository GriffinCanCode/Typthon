"""Test class and OOP features.

This module tests type checking for classes, inheritance, and OOP concepts.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestBasicClasses:
    """Test basic class type checking."""

    def test_simple_class(self, validator):
        """Test simple class definition."""
        code = """
class Point:
    x: int
    y: int
"""
        assert validator.validate(code)

    def test_class_with_init(self, validator):
        """Test class with __init__."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y
"""
        assert validator.validate(code)

    def test_class_with_method(self, validator):
        """Test class with method."""
        code = """
class Calculator:
    def add(self, x: int, y: int) -> int:
        return x + y
"""
        assert validator.validate(code)

    def test_class_instantiation(self, validator):
        """Test class instantiation."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

p = Point(3, 4)
"""
        assert validator.validate(code)

    def test_attribute_access(self, validator):
        """Test attribute access."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

p = Point(3, 4)
x_val: int = p.x
"""
        assert validator.validate(code)

    def test_method_call(self, validator):
        """Test method call."""
        code = """
class Calculator:
    def add(self, x: int, y: int) -> int:
        return x + y

calc = Calculator()
result: int = calc.add(1, 2)
"""
        assert validator.validate(code)

    def test_class_attribute_error(self, validator):
        """Test class attribute type error."""
        code = """
class Point:
    x: int
    y: int

p = Point()
p.x = "not an int"  # Type error
"""
        assert not validator.validate(code)

    def test_property_decorator(self, validator):
        """Test property decorator."""
        code = """
class Circle:
    def __init__(self, radius: float):
        self._radius = radius

    @property
    def area(self) -> float:
        return 3.14159 * self._radius ** 2
"""
        assert validator.validate(code)

    def test_setter_decorator(self, validator):
        """Test setter decorator."""
        code = """
class Temperature:
    def __init__(self, celsius: float):
        self._celsius = celsius

    @property
    def celsius(self) -> float:
        return self._celsius

    @celsius.setter
    def celsius(self, value: float) -> None:
        self._celsius = value
"""
        assert validator.validate(code)

    def test_class_variable(self, validator):
        """Test class variable."""
        code = """
class Counter:
    count: int = 0

    def increment(self) -> None:
        Counter.count += 1
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestInheritance:
    """Test class inheritance."""

    def test_simple_inheritance(self, validator):
        """Test simple inheritance."""
        code = """
class Animal:
    def speak(self) -> str:
        return "sound"

class Dog(Animal):
    pass
"""
        assert validator.validate(code)

    def test_method_override(self, validator):
        """Test method override."""
        code = """
class Animal:
    def speak(self) -> str:
        return "sound"

class Dog(Animal):
    def speak(self) -> str:
        return "woof"
"""
        assert validator.validate(code)

    def test_super_call(self, validator):
        """Test super() call."""
        code = """
class Animal:
    def __init__(self, name: str):
        self.name = name

class Dog(Animal):
    def __init__(self, name: str, breed: str):
        super().__init__(name)
        self.breed = breed
"""
        assert validator.validate(code)

    def test_multiple_inheritance(self, validator):
        """Test multiple inheritance."""
        code = """
class Flyable:
    def fly(self) -> str:
        return "flying"

class Swimmable:
    def swim(self) -> str:
        return "swimming"

class Duck(Flyable, Swimmable):
    pass
"""
        assert validator.validate(code)

    def test_inherited_method_call(self, validator):
        """Test calling inherited method."""
        code = """
class Animal:
    def eat(self) -> str:
        return "eating"

class Dog(Animal):
    pass

dog = Dog()
result: str = dog.eat()
"""
        assert validator.validate(code)

    def test_subclass_assignment(self, validator):
        """Test subclass can be assigned to base type."""
        code = """
class Animal:
    pass

class Dog(Animal):
    pass

animal: Animal = Dog()
"""
        assert validator.validate(code)

    def test_override_type_compatibility(self, validator):
        """Test override maintains type compatibility."""
        code = """
class Base:
    def method(self) -> int:
        return 0

class Derived(Base):
    def method(self) -> int:  # Same return type
        return 1
"""
        assert validator.validate(code)

    def test_abstract_base_class(self, validator):
        """Test abstract base class."""
        code = """
from abc import ABC, abstractmethod

class Shape(ABC):
    @abstractmethod
    def area(self) -> float:
        pass

class Circle(Shape):
    def __init__(self, radius: float):
        self.radius = radius

    def area(self) -> float:
        return 3.14159 * self.radius ** 2
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestDunderMethods:
    """Test special methods (dunder methods)."""

    def test_str_method(self, validator):
        """Test __str__ method."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def __str__(self) -> str:
        return f"({self.x}, {self.y})"
"""
        assert validator.validate(code)

    def test_repr_method(self, validator):
        """Test __repr__ method."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def __repr__(self) -> str:
        return f"Point({self.x}, {self.y})"
"""
        assert validator.validate(code)

    def test_eq_method(self, validator):
        """Test __eq__ method."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Point):
            return self.x == other.x and self.y == other.y
        return False
"""
        assert validator.validate(code)

    def test_lt_method(self, validator):
        """Test __lt__ method."""
        code = """
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def __lt__(self, other: Point) -> bool:
        return (self.x ** 2 + self.y ** 2) < (other.x ** 2 + other.y ** 2)
"""
        assert validator.validate(code)

    def test_add_method(self, validator):
        """Test __add__ method."""
        code = """
class Vector:
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def __add__(self, other: Vector) -> Vector:
        return Vector(self.x + other.x, self.y + other.y)
"""
        assert validator.validate(code)

    def test_len_method(self, validator):
        """Test __len__ method."""
        code = """
class Collection:
    def __init__(self, items: list[int]):
        self.items = items

    def __len__(self) -> int:
        return len(self.items)
"""
        assert validator.validate(code)

    def test_getitem_method(self, validator):
        """Test __getitem__ method."""
        code = """
class Array:
    def __init__(self, data: list[int]):
        self.data = data

    def __getitem__(self, index: int) -> int:
        return self.data[index]
"""
        assert validator.validate(code)

    def test_setitem_method(self, validator):
        """Test __setitem__ method."""
        code = """
class Array:
    def __init__(self, data: list[int]):
        self.data = data

    def __setitem__(self, index: int, value: int) -> None:
        self.data[index] = value
"""
        assert validator.validate(code)

    def test_iter_method(self, validator):
        """Test __iter__ method."""
        code = """
class Range:
    def __init__(self, start: int, end: int):
        self.start = start
        self.end = end

    def __iter__(self):
        return RangeIterator(self.start, self.end)

class RangeIterator:
    def __init__(self, start: int, end: int):
        self.current = start
        self.end = end

    def __next__(self) -> int:
        if self.current >= self.end:
            raise StopIteration
        value = self.current
        self.current += 1
        return value
"""
        assert validator.validate(code)

    def test_call_method(self, validator):
        """Test __call__ method."""
        code = """
class Multiplier:
    def __init__(self, factor: int):
        self.factor = factor

    def __call__(self, x: int) -> int:
        return x * self.factor
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestDataClasses:
    """Test dataclass functionality."""

    def test_basic_dataclass(self, validator):
        """Test basic dataclass."""
        code = """
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int
"""
        assert validator.validate(code)

    def test_dataclass_with_defaults(self, validator):
        """Test dataclass with default values."""
        code = """
from dataclasses import dataclass

@dataclass
class Point:
    x: int = 0
    y: int = 0
"""
        assert validator.validate(code)

    def test_dataclass_instantiation(self, validator):
        """Test dataclass instantiation."""
        code = """
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int

p = Point(3, 4)
"""
        assert validator.validate(code)

    def test_frozen_dataclass(self, validator):
        """Test frozen dataclass."""
        code = """
from dataclasses import dataclass

@dataclass(frozen=True)
class Point:
    x: int
    y: int
"""
        assert validator.validate(code)

    def test_dataclass_post_init(self, validator):
        """Test dataclass __post_init__."""
        code = """
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int
    distance: float = 0.0

    def __post_init__(self):
        self.distance = (self.x ** 2 + self.y ** 2) ** 0.5
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestClassComposition:
    """Test class composition patterns."""

    def test_has_a_relationship(self, validator):
        """Test has-a relationship."""
        code = """
class Engine:
    def start(self) -> str:
        return "Engine started"

class Car:
    def __init__(self):
        self.engine = Engine()

    def start(self) -> str:
        return self.engine.start()
"""
        assert validator.validate(code)

    def test_delegation(self, validator):
        """Test delegation pattern."""
        code = """
class Logger:
    def log(self, message: str) -> None:
        print(message)

class Service:
    def __init__(self, logger: Logger):
        self.logger = logger

    def process(self, data: str) -> None:
        self.logger.log(f"Processing {data}")
"""
        assert validator.validate(code)

    def test_mixin(self, validator):
        """Test mixin pattern."""
        code = """
class JSONMixin:
    def to_json(self) -> str:
        return "{}"

class LogMixin:
    def log(self, message: str) -> None:
        print(message)

class Model(JSONMixin, LogMixin):
    pass
"""
        assert validator.validate(code)

    def test_strategy_pattern(self, validator):
        """Test strategy pattern."""
        code = """
from typing import Protocol

class SortStrategy(Protocol):
    def sort(self, data: list[int]) -> list[int]: ...

class BubbleSort:
    def sort(self, data: list[int]) -> list[int]:
        return sorted(data)

class Sorter:
    def __init__(self, strategy: SortStrategy):
        self.strategy = strategy

    def sort(self, data: list[int]) -> list[int]:
        return self.strategy.sort(data)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestNestedClasses:
    """Test nested class definitions."""

    def test_nested_class(self, validator):
        """Test nested class definition."""
        code = """
class Outer:
    class Inner:
        def method(self) -> int:
            return 42
"""
        assert validator.validate(code)

    def test_nested_class_access(self, validator):
        """Test accessing nested class."""
        code = """
class Outer:
    class Inner:
        def method(self) -> int:
            return 42

inner = Outer.Inner()
result: int = inner.method()
"""
        assert validator.validate(code)

    def test_nested_class_with_outer_reference(self, validator):
        """Test nested class with reference to outer."""
        code = """
class Outer:
    value: int = 42

    class Inner:
        def get_outer_value(self, outer: Outer) -> int:
            return outer.value
"""
        assert validator.validate(code)

