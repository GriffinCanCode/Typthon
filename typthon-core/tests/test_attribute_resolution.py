"""
Test attribute resolution for various types.
"""
import pytest


def test_str_attributes():
    """Test string method attribute access."""
    s = "hello"
    upper = s.upper()  # Should resolve to str
    lower = s.lower()  # Should resolve to str
    split = s.split(",")  # Should resolve to list[str]
    assert isinstance(upper, str)
    assert isinstance(lower, str)
    assert isinstance(split, list)


def test_list_attributes():
    """Test list method attribute access."""
    items: list[int] = [1, 2, 3]
    items.append(4)  # Should resolve to None
    items.extend([5, 6])  # Should resolve to None
    popped = items.pop()  # Should resolve to int
    assert popped == 6


def test_dict_attributes():
    """Test dict method attribute access."""
    data: dict[str, int] = {"a": 1, "b": 2}
    keys = data.keys()  # Should resolve to list
    values = data.values()  # Should resolve to list
    items = data.items()  # Should resolve to list[tuple]
    value = data.get("a")  # Should resolve to int | None
    assert len(keys) == 2


def test_set_attributes():
    """Test set method attribute access."""
    nums: set[int] = {1, 2, 3}
    nums.add(4)  # Should resolve to None
    nums.remove(1)  # Should resolve to None
    union_result = nums.union({5, 6})  # Should resolve to set[int]
    assert 4 in nums


def test_custom_class_attributes():
    """Test custom class attribute access."""
    class Person:
        def __init__(self, name: str, age: int):
            self.name = name
            self.age = age

        def greet(self) -> str:
            return f"Hello, I'm {self.name}"

    p = Person("Alice", 30)
    name = p.name  # Should resolve to str
    age = p.age  # Should resolve to int
    greeting = p.greet()  # Should resolve to str
    assert greeting == "Hello, I'm Alice"


def test_union_attributes():
    """Test attribute access on union types."""
    from typing import Union

    def process(value: Union[str, list[str]]) -> int:
        # Both str and list have __len__, so this should work
        return len(value)

    assert process("hello") == 5
    assert process(["a", "b"]) == 2


def test_invalid_attribute():
    """Test that invalid attribute access is caught."""
    s = "hello"
    # This should generate a type error:
    # s.invalid_method()  # Error: Type 'str' has no attribute 'invalid_method'


def test_attribute_typo_suggestions():
    """Test that typos in attributes provide suggestions."""
    items = [1, 2, 3]
    # This should suggest 'append':
    # items.appnd(4)  # Error: Did you mean 'append'?


def test_chained_attributes():
    """Test chained attribute access."""
    text = "  hello world  "
    result = text.strip().upper().split()  # Should all resolve correctly
    assert result == ["HELLO", "WORLD"]


def test_method_call_return_attributes():
    """Test accessing attributes on method return values."""
    text = "hello"
    length = text.upper().strip().__len__()  # Chain of attribute accesses
    assert length == 5


def test_generic_type_attributes():
    """Test attribute resolution with generic types."""
    from typing import List

    def process_items(items: List[str]) -> None:
        items.append("new")  # Should resolve with correct type
        items.extend(["a", "b"])  # Should resolve with correct type
        result = items.pop()  # Should return str
        assert isinstance(result, str)

    process_items(["x", "y"])


def test_inheritance_attributes():
    """Test attribute resolution with inheritance."""
    class Base:
        def base_method(self) -> str:
            return "base"

    class Derived(Base):
        def derived_method(self) -> int:
            return 42

    obj = Derived()
    base_result = obj.base_method()  # Should resolve from base class
    derived_result = obj.derived_method()  # Should resolve from derived class
    assert base_result == "base"
    assert derived_result == 42


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

