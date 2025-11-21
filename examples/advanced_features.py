"""Advanced features demonstration."""

from typthon import type, Protocol
from typthon.types import Union, Intersection, Literal, effect, dependent


# Example 1: Protocol (structural typing)
class Drawable(Protocol):
    """Protocol for drawable objects."""

    def draw(self) -> None:
        """Draw the object."""
        ...


class Circle:
    """Circle implements Drawable protocol implicitly."""

    def draw(self) -> None:
        print("Drawing circle")


# Example 2: Literal types
@type("(Literal[1, 2, 3]) -> str")
def number_name(n):
    """Get name for specific numbers."""
    names = {1: "one", 2: "two", 3: "three"}
    return names[n]


# Example 3: Complex union types
@type("(int | float | str | None) -> str")
def describe_value(value):
    """Describe any value."""
    match value:
        case None:
            return "nothing"
        case int():
            return f"integer: {value}"
        case float():
            return f"floating: {value}"
        case str():
            return f"text: {value}"
        case _:
            return "unknown"


# Example 4: Intersection types
@type("(HasName & HasAge) -> str")
def format_person(obj):
    """Format object with name and age."""
    return f"{obj.name} ({obj.age})"


# Example 5: Generic types with constraints
@type("(T, list[T]) -> list[T]")
def prepend(item, items):
    """Prepend item to list."""
    return [item] + items


# Example 6: Effect types
@type("(str) -> dict", effects=["io", "network"])
def fetch_data(url):
    """Fetch data from URL (has side effects)."""
    # In real code, would make HTTP request
    return {"status": "ok", "data": "..."}


# Example 7: Dependent types
@type("(int) -> int")
@dependent("x > 0")
def positive_sqrt(x):
    """Square root of positive number."""
    return int(x ** 0.5)


# Example 8: Complex function signatures
@type("((T) -> U, list[T]) -> list[U]")
def map_generic(f, items):
    """Generic map function."""
    return [f(x) for x in items]


@type("((T, T) -> T, list[T]) -> T")
def reduce_generic(f, items):
    """Generic reduce function."""
    result = items[0]
    for item in items[1:]:
        result = f(result, item)
    return result


# Example 9: Nested generic types
@type("(dict[str, list[int]]) -> list[tuple[str, int]]")
def flatten_dict(data):
    """Flatten dictionary of lists."""
    result = []
    for key, values in data.items():
        for value in values:
            result.append((key, value))
    return result


# Example 10: Conditional types (approximation)
@type("(list[T]) -> T | None")
def first_or_none(items):
    """Get first item or None."""
    return items[0] if items else None


# Example 11: Multiple return types based on input
@type("(bool, T, U) -> T | U")
def conditional_return(condition, true_val, false_val):
    """Return different types based on condition."""
    return true_val if condition else false_val


# Example 12: Recursive types (approximation)
@type("(list[int | list]) -> int")
def sum_nested(items):
    """Sum nested list structure."""
    total = 0
    for item in items:
        if isinstance(item, int):
            total += item
        elif isinstance(item, list):
            total += sum_nested(item)
    return total


if __name__ == "__main__":
    # Test protocols
    circle = Circle()
    circle.draw()

    # Test literals
    print(f"number_name(1) = {number_name(1)}")

    # Test unions
    print(f"describe_value(42) = {describe_value(42)}")
    print(f"describe_value(3.14) = {describe_value(3.14)}")
    print(f"describe_value('hello') = {describe_value('hello')}")
    print(f"describe_value(None) = {describe_value(None)}")

    # Test generics
    print(f"prepend(0, [1, 2, 3]) = {prepend(0, [1, 2, 3])}")

    # Test map/reduce
    print(f"map_generic(lambda x: x*2, [1,2,3]) = {map_generic(lambda x: x*2, [1,2,3])}")
    print(f"reduce_generic(lambda x,y: x+y, [1,2,3]) = {reduce_generic(lambda x, y: x+y, [1,2,3])}")

    # Test nested types
    data = {"a": [1, 2], "b": [3, 4]}
    print(f"flatten_dict({data}) = {flatten_dict(data)}")

    # Test optional returns
    print(f"first_or_none([1,2,3]) = {first_or_none([1, 2, 3])}")
    print(f"first_or_none([]) = {first_or_none([])}")

    # Test recursive types
    nested = [1, [2, [3, 4]], 5]
    print(f"sum_nested({nested}) = {sum_nested(nested)}")

