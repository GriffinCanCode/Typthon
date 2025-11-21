"""
Comprehensive examples of attribute resolution in Typthon.

This demonstrates the attribute resolution system across:
- Built-in types (str, list, dict, set)
- Custom classes
- Union types
- Inheritance
- Error detection with suggestions
"""


def string_operations():
    """String attribute access examples."""
    text: str = "hello world"

    # Method calls - all resolve correctly
    upper = text.upper()  # Returns: str
    lower = text.lower()  # Returns: str
    stripped = text.strip()  # Returns: str

    # Method with arguments
    words = text.split(" ")  # Returns: list[str]
    replaced = text.replace("world", "python")  # Returns: str

    # Boolean methods
    starts = text.startswith("hello")  # Returns: bool
    ends = text.endswith("world")  # Returns: bool

    # Chained calls
    result = text.strip().upper().split()  # All methods resolve correctly

    return result


def list_operations():
    """List attribute access examples."""
    numbers: list[int] = [1, 2, 3, 4, 5]

    # Mutation methods
    numbers.append(6)  # Returns: None
    numbers.extend([7, 8])  # Returns: None

    # Query/modify
    last = numbers.pop()  # Returns: int
    numbers.remove(4)  # Returns: None

    # Utility methods
    numbers.sort()  # Returns: None
    numbers.reverse()  # Returns: None
    copy = numbers.copy()  # Returns: list[int]

    return copy


def dict_operations():
    """Dictionary attribute access examples."""
    data: dict[str, int] = {"a": 1, "b": 2, "c": 3}

    # View methods
    keys = data.keys()  # Returns: list (keys view)
    values = data.values()  # Returns: list (values view)
    items = data.items()  # Returns: list[tuple[str, int]]

    # Access methods
    value = data.get("a")  # Returns: int | None
    popped = data.pop("b")  # Returns: int

    # Mutation
    data.clear()  # Returns: None
    data.update({"x": 10, "y": 20})  # Returns: None

    return data


def set_operations():
    """Set attribute access examples."""
    numbers: set[int] = {1, 2, 3, 4, 5}

    # Mutation methods
    numbers.add(6)  # Returns: None
    numbers.remove(1)  # Returns: None
    numbers.discard(10)  # Returns: None (no error if absent)

    # Set operations
    union = numbers.union({7, 8})  # Returns: set[int]
    intersection = numbers.intersection({2, 3, 4})  # Returns: set[int]

    # Clear
    numbers.clear()  # Returns: None

    return union


def custom_class_example():
    """Custom class with attribute resolution."""

    class Point:
        """2D point with methods."""

        def __init__(self, x: float, y: float):
            self.x = x
            self.y = y

        def distance_from_origin(self) -> float:
            """Calculate distance from origin."""
            return (self.x ** 2 + self.y ** 2) ** 0.5

        def translate(self, dx: float, dy: float) -> None:
            """Move point by delta."""
            self.x += dx
            self.y += dy

        def to_string(self) -> str:
            """String representation."""
            return f"Point({self.x}, {self.y})"

    # Create instance and access attributes
    p = Point(3.0, 4.0)

    # Property access - resolves to float
    x_coord = p.x
    y_coord = p.y

    # Method calls - resolve correctly
    distance = p.distance_from_origin()  # Returns: float
    p.translate(1.0, 1.0)  # Returns: None
    description = p.to_string()  # Returns: str

    return description


def union_type_example():
    """Attribute resolution on union types."""
    from typing import Union

    def process_value(value: Union[str, list[str]]) -> int:
        """Process either a string or list of strings.

        Both types support len(), so this is valid.
        """
        # This works because both str and list have __len__
        return len(value)

    # Both calls are valid
    str_length = process_value("hello")  # Returns: 5
    list_length = process_value(["a", "b", "c"])  # Returns: 3

    return str_length + list_length


def inheritance_example():
    """Attribute resolution with inheritance."""

    class Animal:
        """Base class."""

        def __init__(self, name: str):
            self.name = name

        def speak(self) -> str:
            """Base speak method."""
            return "Some sound"

    class Dog(Animal):
        """Derived class."""

        def __init__(self, name: str, breed: str):
            super().__init__(name)
            self.breed = breed

        def speak(self) -> str:
            """Override speak method."""
            return "Woof!"

        def fetch(self) -> str:
            """Dog-specific method."""
            return f"{self.name} is fetching!"

    # Create derived instance
    dog = Dog("Buddy", "Labrador")

    # Access base class attributes
    name = dog.name  # Resolved from Animal
    sound = dog.speak()  # Resolved from Dog (overridden)

    # Access derived class attributes
    breed = dog.breed  # Resolved from Dog
    fetch_action = dog.fetch()  # Resolved from Dog

    return fetch_action


def error_detection_examples():
    """Examples of error detection with helpful suggestions."""

    # Example 1: Invalid attribute
    text = "hello"
    # text.invalid()  # ERROR: Type 'str' has no attribute 'invalid'

    # Example 2: Typo with suggestion
    numbers = [1, 2, 3]
    # numbers.appnd(4)  # ERROR: Did you mean 'append'?
    # numbers.exten([4, 5])  # ERROR: Did you mean 'extend'?

    # Example 3: Wrong type
    value: int = 42
    # value.upper()  # ERROR: Type 'int' has no attribute 'upper'

    pass  # Avoid actual errors in example


def advanced_chaining():
    """Advanced attribute chaining and composition."""

    # Complex chaining
    text = "  Hello, World!  "
    result = (
        text
        .strip()        # str -> str
        .lower()        # str -> str
        .replace(",", "")  # str -> str
        .split()        # str -> list[str]
    )
    # Result type: list[str]

    # Chaining on method returns
    words = ["hello", "world"]
    processed = (
        " ".join(words)  # list[str] -> str (via str.join)
        .upper()         # str -> str
        .split()         # str -> list[str]
    )

    return processed


def generic_constraints():
    """Attribute resolution with generic constraints."""
    from typing import TypeVar, Generic

    T = TypeVar('T')

    class Container(Generic[T]):
        """Generic container with methods."""

        def __init__(self, value: T):
            self.value = value

        def get(self) -> T:
            """Get the contained value."""
            return self.value

        def set(self, value: T) -> None:
            """Set a new value."""
            self.value = value

    # String container
    str_container = Container[str]("hello")
    text = str_container.get()  # Returns: str
    text.upper()  # Attribute resolved on returned str

    # List container
    list_container = Container[list[int]]([1, 2, 3])
    items = list_container.get()  # Returns: list[int]
    items.append(4)  # Attribute resolved on returned list

    return items


if __name__ == "__main__":
    print("=== String Operations ===")
    print(string_operations())

    print("\n=== List Operations ===")
    print(list_operations())

    print("\n=== Dict Operations ===")
    print(dict_operations())

    print("\n=== Set Operations ===")
    print(set_operations())

    print("\n=== Custom Class ===")
    print(custom_class_example())

    print("\n=== Union Types ===")
    print(union_type_example())

    print("\n=== Inheritance ===")
    print(inheritance_example())

    print("\n=== Advanced Chaining ===")
    print(advanced_chaining())

    print("\n=== Generic Constraints ===")
    print(generic_constraints())

    print("\n✓ All examples completed successfully!")

