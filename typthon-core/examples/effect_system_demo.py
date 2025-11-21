"""
End-to-End Effect System and Runtime Validation Demo

This example demonstrates Typthon's killer features:
1. Automatic effect inference in the Rust type checker
2. Effect-annotated function types
3. Runtime validation with the @type decorator
4. Refinement types with predicates
5. Complete integration of all advanced features
"""

from typthon.decorators.type import type
from typthon.types.effects import effect, refine, Positive, NonEmpty


# ============================================================================
# Basic Runtime Validation
# ============================================================================

@type("(int, int) -> int")
def add(x, y):
    """Pure function with runtime validation."""
    return x + y


@type("(str, str) -> str")
def concat(a, b):
    """String concatenation with type checking."""
    return a + b


# ============================================================================
# Effect Types with Runtime Validation
# ============================================================================

@type("() -> int ! {io}", runtime=True, strict=True)
def read_number():
    """Function with IO effect - tracked at compile time."""
    return int(input("Enter a number: "))


@type("(str) -> None ! {io}")
def print_message(msg):
    """Function with IO effect for printing."""
    print(msg)


@type("() -> int ! {io, exception}")
def risky_read():
    """Function with multiple effects."""
    try:
        return int(input("Enter a number: "))
    except ValueError:
        raise ValueError("Invalid input!")


# ============================================================================
# Refinement Types with Runtime Validation
# ============================================================================

@type("(int[value > 0]) -> int[value > 0]", runtime=True, strict=True)
def square_positive(x):
    """Square a positive number, returns positive."""
    return x * x


@type("(str[len(value) > 0]) -> int", runtime=True)
def count_chars(s):
    """Count characters in non-empty string."""
    return len(s)


@type("(int[value > 0], int[value > 0]) -> int[value > 0]", runtime=True)
def multiply_positive(a, b):
    """Multiply two positive numbers."""
    return a * b


# ============================================================================
# Complex Types with Runtime Validation
# ============================================================================

@type("(list[int]) -> int")
def sum_list(numbers):
    """Sum a list of integers."""
    return sum(numbers)


@type("(dict[str, int]) -> list[str]")
def get_keys(d):
    """Get keys from a dictionary."""
    return list(d.keys())


@type("(list[str[len(value) > 0]]) -> str")
def join_non_empty(strings):
    """Join non-empty strings."""
    return ", ".join(strings)


# ============================================================================
# Union Types with Runtime Validation
# ============================================================================

@type("(int | str) -> str")
def to_string(value):
    """Convert int or str to string."""
    return str(value)


@type("(list[int | float]) -> float")
def average(numbers):
    """Calculate average of numbers."""
    return sum(numbers) / len(numbers) if numbers else 0.0


# ============================================================================
# Mutation Tracking
# ============================================================================

@type("(list[int], int) -> None ! {mutation}")
def append_number(lst, num):
    """Mutating function - effect tracked."""
    lst.append(num)


# ============================================================================
# Demonstrations
# ============================================================================

def demo_basic_validation():
    """Demonstrate basic runtime validation."""
    print("\n=== Basic Runtime Validation ===")

    # Valid calls
    result = add(5, 3)
    print(f"add(5, 3) = {result}")

    result = concat("Hello", " World")
    print(f"concat('Hello', ' World') = {result}")

    # Invalid call (will warn or raise depending on strict mode)
    try:
        result = add("5", 3)  # Type error: expected int, got str
        print(f"add('5', 3) = {result}")
    except TypeError as e:
        print(f"Caught type error: {e}")


def demo_refinement_validation():
    """Demonstrate refinement type validation."""
    print("\n=== Refinement Type Validation ===")

    # Valid calls
    result = square_positive(5)
    print(f"square_positive(5) = {result}")

    result = count_chars("hello")
    print(f"count_chars('hello') = {result}")

    # Invalid calls
    try:
        result = square_positive(-5)  # Refinement error: value not > 0
        print(f"square_positive(-5) = {result}")
    except TypeError as e:
        print(f"Caught refinement error: {e}")

    try:
        result = count_chars("")  # Refinement error: empty string
        print(f"count_chars('') = {result}")
    except TypeError as e:
        print(f"Caught refinement error: {e}")


def demo_collection_validation():
    """Demonstrate collection type validation."""
    print("\n=== Collection Type Validation ===")

    # Valid calls
    result = sum_list([1, 2, 3, 4, 5])
    print(f"sum_list([1, 2, 3, 4, 5]) = {result}")

    result = get_keys({"a": 1, "b": 2, "c": 3})
    print(f"get_keys({{'a': 1, 'b': 2, 'c': 3}}) = {result}")

    # Invalid call
    try:
        result = sum_list([1, "2", 3])  # Type error: list element not int
        print(f"sum_list([1, '2', 3]) = {result}")
    except TypeError as e:
        print(f"Caught type error: {e}")


def demo_union_validation():
    """Demonstrate union type validation."""
    print("\n=== Union Type Validation ===")

    # Valid calls with different types
    result = to_string(42)
    print(f"to_string(42) = {result}")

    result = to_string("hello")
    print(f"to_string('hello') = {result}")

    result = average([1, 2.5, 3, 4.5])
    print(f"average([1, 2.5, 3, 4.5]) = {result}")


def demo_effect_system():
    """Demonstrate effect system integration."""
    print("\n=== Effect System Demo ===")
    print("Effect tracking happens at compile time via the Rust checker.")
    print("The @type decorator also validates types at runtime.")
    print()
    print("Functions with effects:")
    print("  - read_number() has effects: {io, exception}")
    print("  - print_message(msg) has effects: {io}")
    print("  - append_number(lst, num) has effects: {mutation}")


def demo_signature_parsing():
    """Demonstrate signature parsing."""
    print("\n=== Signature Parsing ===")

    from typthon.core import parse_signature

    signatures = [
        "(int, int) -> int",
        "(str) -> list[int]",
        "(int) -> int ! {io}",
        "(int[value > 0]) -> int[value > 0]",
        "(list[int | str]) -> str",
        "() -> dict[str, int] ! {io, exception}",
    ]

    for sig_str in signatures:
        sig = parse_signature(sig_str)
        print(f"Parsed: {sig_str}")
        print(f"  Params: {sig.params}")
        print(f"  Return: {sig.return_type}")
        print(f"  Effects: {sig.effects}")
        print()


if __name__ == "__main__":
    print("=" * 70)
    print("Typthon Effect System and Runtime Validation Demo")
    print("=" * 70)

    demo_basic_validation()
    demo_refinement_validation()
    demo_collection_validation()
    demo_union_validation()
    demo_effect_system()
    demo_signature_parsing()

    print("\n" + "=" * 70)
    print("Demo complete! The effect system is fully integrated.")
    print("=" * 70)

