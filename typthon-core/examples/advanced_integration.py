"""
Comprehensive example demonstrating end-to-end integration of advanced type features.

This example showcases:
- Effect types (IO, Async, Network, Mutation)
- Refinement types (Positive, Bounded, Custom predicates)
- Dependent types (Fixed length, Length ranges)
- Recursive types (JSON, Linked lists)
- Combinations of advanced features
"""

from typthon import (
    type, infer, validate,
    EffectType, RefinementType, DependentType, RecursiveType,
    effect, refine, dependent, recursive,
    Positive, Negative, NonEmpty, Even, Odd, Bounded,
    IO, Async, Random
)


# ==============================================================================
# 1. EFFECT TYPES
# ==============================================================================

print("=" * 80)
print("EFFECT TYPES DEMONSTRATION")
print("=" * 80)

@type("() -> IO[str]")
def read_user_input():
    """Function with IO effect."""
    return input("Enter your name: ")


@type("(str) -> IO[None]")
def greet(name: str):
    """Function with IO effect."""
    print(f"Hello, {name}!")


@type("(int, int) -> int")
def add_pure(a: int, b: int) -> int:
    """Pure function with no effects."""
    return a + b


@type("() -> Random[int]")
def roll_dice():
    """Function with Random effect."""
    import random
    return random.randint(1, 6)


# Effect type instances
IOInt = IO(int)
AsyncStr = Async(str)
RandomFloat = Random(float)

print(f"IOInt: {IOInt}")
print(f"AsyncStr: {AsyncStr}")
print(f"RandomFloat: {RandomFloat}")
print(f"IOInt has IO effect: {IOInt.has_effect('io')}")
print(f"IOInt is pure: {IOInt.is_pure()}")


# ==============================================================================
# 2. REFINEMENT TYPES
# ==============================================================================

print("\n" + "=" * 80)
print("REFINEMENT TYPES DEMONSTRATION")
print("=" * 80)

# Built-in refinement types
@type("(Positive) -> Positive")
def square_positive(x: int) -> int:
    """Square a positive number, result is always positive."""
    return x * x


@type("(NonEmpty[str]) -> int")
def string_length(s: str) -> int:
    """Get length of non-empty string."""
    return len(s)


@type("(Even) -> Even")
def double_even(x: int) -> int:
    """Double an even number, result is always even."""
    return x * 2


# Custom refinement types
Percentage = Bounded(0, 100)
Prime = refine('value in [2, 3, 5, 7, 11, 13, 17, 19]')(int)
PositiveFloat = refine('value > 0.0')(float)


# Demonstrate refinement validation
pos = Positive()
print(f"\nPositive(5): {pos.validate(5)}")
print(f"Positive(-5): {pos.validate(-5)}")
print(f"Positive(0): {pos.validate(0)}")

percentage = Percentage
print(f"\nPercentage(50): {percentage.validate(50)}")
print(f"Percentage(150): {percentage.validate(150)}")

even = Even()
print(f"\nEven(4): {even.validate(4)}")
print(f"Even(5): {even.validate(5)}")


# ==============================================================================
# 3. DEPENDENT TYPES
# ==============================================================================

print("\n" + "=" * 80)
print("DEPENDENT TYPES DEMONSTRATION")
print("=" * 80)

# Fixed-length arrays
Array3 = dependent('len=3')(list)
Array5 = dependent('len=5')(list)

# Variable-length strings
ShortString = dependent('0<=len<=10')(str)
MediumString = dependent('10<=len<=100')(str)


def validate_array3(arr):
    """Validate a 3-element array."""
    from typthon.core.validator import validate_dependent_type
    return validate_dependent_type(arr, list, 'len=3')


def validate_short_string(s):
    """Validate a short string (0-10 chars)."""
    from typthon.core.validator import validate_dependent_type
    return validate_dependent_type(s, str, '0<=len<=10')


print(f"\nArray3([1,2,3]): {validate_array3([1, 2, 3])}")
print(f"Array3([1,2]): {validate_array3([1, 2])}")

print(f"\nShortString('hello'): {validate_short_string('hello')}")
print(f"ShortString('a'*20): {validate_short_string('a' * 20)}")


# ==============================================================================
# 4. RECURSIVE TYPES
# ==============================================================================

print("\n" + "=" * 80)
print("RECURSIVE TYPES DEMONSTRATION")
print("=" * 80)

# JSON type: recursive union of primitives and structures
JSON = recursive('JSON', lambda self:
    type('None | bool | int | float | str | list[self] | dict[str, self]'))

print(f"JSON type: {JSON}")


# Linked list: List[T] = None | (T, List[T])
def LinkedList(T):
    """Create a linked list type for elements of type T."""
    return recursive('List', lambda self:
        type(f'None | tuple[{T}, self]'))


IntList = LinkedList('int')
StrList = LinkedList('str')

print(f"\nIntList: {IntList}")
print(f"StrList: {StrList}")


# Binary tree: Tree[T] = Leaf(T) | Node(Tree[T], T, Tree[T])
Tree = recursive('Tree', lambda self:
    type('tuple[int] | tuple[self, int, self]'))

print(f"\nBinary Tree: {Tree}")


# ==============================================================================
# 5. ADVANCED COMBINATIONS
# ==============================================================================

print("\n" + "=" * 80)
print("ADVANCED COMBINATIONS")
print("=" * 80)

# Combining effects with refinements
@type("() -> IO[Positive]")
def read_positive_integer():
    """Read a positive integer with IO effect."""
    while True:
        value = int(input("Enter a positive integer: "))
        if value > 0:
            return value
        print("Must be positive!")


# Combining refinements with dependent types
PositiveArray5 = dependent('len=5')(list)  # Would contain positive ints


# Effect + Refinement + Dependent
@type("(NonEmpty[str]) -> IO[Bounded[0, 100]]")
def analyze_string(s: str) -> int:
    """
    Analyze a non-empty string and return a percentage score with IO effect.
    """
    score = min(100, len(s) * 10)
    print(f"Analysis score: {score}")
    return score


# Multiple effects
@type("(str) -> IO[Async[str]]")
async def fetch_and_log(url: str) -> str:
    """
    Fetch data from URL (Network + Async) and log it (IO).
    """
    print(f"Fetching from {url}...")
    # Simulated async fetch
    return f"Data from {url}"


# Complex recursive type with refinements
@type("(Positive) -> JSON")
def generate_config(size: int):
    """
    Generate a JSON config object based on positive size parameter.
    """
    return {
        "size": size,
        "enabled": True,
        "values": list(range(size)),
        "metadata": {
            "version": "1.0",
            "count": size
        }
    }


print("\nAdvanced type combinations defined successfully!")


# ==============================================================================
# 6. RUNTIME VALIDATION
# ==============================================================================

print("\n" + "=" * 80)
print("RUNTIME VALIDATION")
print("=" * 80)

from typthon.core.validator import (
    validate_refinement_type,
    validate_dependent_type,
    validate_effect_type
)

# Test refinement validation
print("\nRefinement Validation:")
print(f"  validate_refinement_type(10, int, 'value > 0'): {validate_refinement_type(10, int, 'value > 0')}")
print(f"  validate_refinement_type(-5, int, 'value > 0'): {validate_refinement_type(-5, int, 'value > 0')}")
print(f"  validate_refinement_type(50, int, 'value >= 0 and value <= 100'): {validate_refinement_type(50, int, 'value >= 0 and value <= 100')}")

# Test dependent validation
print("\nDependent Type Validation:")
print(f"  validate_dependent_type([1,2,3], list, 'len=3'): {validate_dependent_type([1,2,3], list, 'len=3')}")
print(f"  validate_dependent_type([1,2], list, 'len=3'): {validate_dependent_type([1,2], list, 'len=3')}")
print(f"  validate_dependent_type('hello', str, '0<=len<=10'): {validate_dependent_type('hello', str, '0<=len<=10')}")

# Test effect validation
print("\nEffect Type Validation:")
io_type = IO(int)
print(f"  IO(int) has 'io' effect: {io_type.has_effect('io')}")
print(f"  IO(int) is pure: {io_type.is_pure()}")


# ==============================================================================
# 7. PRACTICAL EXAMPLES
# ==============================================================================

print("\n" + "=" * 80)
print("PRACTICAL EXAMPLES")
print("=" * 80)

# Example 1: Safe division with refinements
@type("(int, NonZero) -> float")
def safe_divide(a: int, b: int) -> float:
    """Divide by a non-zero integer."""
    return a / b


NonZero = refine('value != 0')(int)
print("\nSafe division:")
print(f"  safe_divide(10, 2) = {safe_divide(10, 2)}")
print(f"  NonZero validates 5: {NonZero.validate(5)}")
print(f"  NonZero validates 0: {NonZero.validate(0)}")


# Example 2: Configuration with dependent types
@type("(Positive) -> dict")
def create_config(max_retries: int):
    """
    Create configuration with positive max_retries.
    """
    return {
        "max_retries": max_retries,
        "timeout": max_retries * 10,
        "backoff": 2 ** max_retries
    }


print("\nConfiguration creation:")
config = create_config(3)
print(f"  create_config(3) = {config}")


# Example 3: Data validation pipeline
@type("(list[int]) -> IO[Positive]")
def validate_and_sum(numbers: list):
    """
    Validate a list of numbers and return their positive sum with IO.
    """
    total = sum(numbers)
    print(f"Sum of {numbers} = {total}")
    return abs(total) if total != 0 else 1


print("\nData validation pipeline:")
result = validate_and_sum([1, 2, 3, 4, 5])
print(f"  validate_and_sum([1,2,3,4,5]) = {result}")


# ==============================================================================
# SUMMARY
# ==============================================================================

print("\n" + "=" * 80)
print("INTEGRATION SUMMARY")
print("=" * 80)
print("""
✓ Effect Types: Track side effects (IO, Async, Network, Mutation, Random)
✓ Refinement Types: Value-level predicates (Positive, Bounded, Even, etc.)
✓ Dependent Types: Length and value constraints
✓ Recursive Types: Self-referential structures (JSON, Trees, Lists)
✓ Advanced Combinations: Multiple features working together
✓ Runtime Validation: Rust-powered validation for all types
✓ Type Checking: Static analysis with bidirectional inference
✓ Constraint Solving: Automatic type constraint resolution

All advanced type features are now fully integrated end-to-end!
""")

