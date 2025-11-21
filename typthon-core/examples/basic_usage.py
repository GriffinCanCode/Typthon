"""Basic usage examples for Typthon."""

from typthon import type, infer, check, validate


# Example 1: Simple function typing
@type("(int, int) -> int")
def add(x, y):
    """Add two integers."""
    return x + y


# Example 2: Union types
@type("(int | str) -> str")
def to_string(value):
    """Convert value to string."""
    return str(value)


# Example 3: Generic container types
@type("(list[int]) -> int")
def sum_list(items):
    """Sum all integers in a list."""
    return sum(items)


# Example 4: Optional types
@type("(str | None) -> int")
def string_length(s):
    """Get string length, 0 if None."""
    return len(s) if s else 0


# Example 5: Complex nested types
@type("(dict[str, list[int]]) -> dict[str, int]")
def sum_dict_values(data):
    """Sum all lists in dictionary values."""
    return {k: sum(v) for k, v in data.items()}


# Example 6: Automatic type inference
@infer
def process_data(items):
    """Process items (type will be inferred)."""
    return [x * 2 for x in items if x > 0]


# Example 7: Function composition
@type("(list[int]) -> list[int]")
def filter_positive(items):
    return [x for x in items if x > 0]


@type("(list[int]) -> list[int]")
def double_all(items):
    return [x * 2 for x in items]


def composed_operation(items):
    """Compose multiple typed functions."""
    return double_all(filter_positive(items))


# Example 8: Higher-order functions
@type("((int) -> int, list[int]) -> list[int]")
def map_fn(f, items):
    """Map function over list."""
    return [f(x) for x in items]


# Example 9: Runtime validation
def validate_examples():
    """Demonstrate runtime validation."""
    print(f"42 is int: {validate(42, 'int')}")
    print(f"'hello' is int: {validate('hello', 'int')}")
    print(f"[1, 2, 3] is list: {validate([1, 2, 3], 'list')}")


# Example 10: Static checking
def check_file_example():
    """Check this file for type errors."""
    errors = check(__file__)
    if errors:
        print("Type errors found:")
        for error in errors:
            print(f"  {error}")
    else:
        print("No type errors!")


if __name__ == "__main__":
    # Test basic operations
    print(f"add(1, 2) = {add(1, 2)}")
    print(f"to_string(42) = {to_string(42)}")
    print(f"sum_list([1, 2, 3]) = {sum_list([1, 2, 3])}")
    print(f"string_length('hello') = {string_length('hello')}")
    print(f"string_length(None) = {string_length(None)}")

    # Test complex types
    data = {"a": [1, 2, 3], "b": [4, 5, 6]}
    print(f"sum_dict_values({data}) = {sum_dict_values(data)}")

    # Test inference
    print(f"process_data([1, -2, 3, -4, 5]) = {process_data([1, -2, 3, -4, 5])}")

    # Test composition
    print(f"composed([1, -2, 3, -4, 5]) = {composed_operation([1, -2, 3, -4, 5])}")

    # Test validation
    print("\nRuntime validation:")
    validate_examples()

    # Static checking
    print("\nStatic checking:")
    check_file_example()

