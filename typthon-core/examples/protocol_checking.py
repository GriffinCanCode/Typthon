"""
Protocol Checking Examples

Demonstrates structural typing with protocols in Typthon.
"""

from typing import Protocol, runtime_checkable


# ============================================================================
# Basic Protocol Definitions
# ============================================================================

@runtime_checkable
class Sized(Protocol):
    """Protocol for objects with a length."""
    def __len__(self) -> int: ...


@runtime_checkable
class Comparable(Protocol):
    """Protocol for comparable objects."""
    def __lt__(self, other) -> bool: ...
    def __le__(self, other) -> bool: ...
    def __gt__(self, other) -> bool: ...
    def __ge__(self, other) -> bool: ...
    def __eq__(self, other) -> bool: ...
    def __ne__(self, other) -> bool: ...


@runtime_checkable
class Numeric(Protocol):
    """Protocol for numeric types."""
    def __add__(self, other) -> 'Numeric': ...
    def __sub__(self, other) -> 'Numeric': ...
    def __mul__(self, other) -> 'Numeric': ...
    def __truediv__(self, other) -> 'Numeric': ...


@runtime_checkable
class Drawable(Protocol):
    """Protocol for drawable objects."""
    def draw(self) -> None: ...
    def move(self, x: int, y: int) -> None: ...
    def get_bounds(self) -> tuple[int, int, int, int]: ...


# ============================================================================
# Concrete Implementations
# ============================================================================

class Rectangle:
    """Rectangle implements Drawable protocol."""
    def __init__(self, x: int, y: int, width: int, height: int):
        self.x = x
        self.y = y
        self.width = width
        self.height = height

    def draw(self) -> None:
        print(f"Drawing rectangle at ({self.x}, {self.y})")

    def move(self, x: int, y: int) -> None:
        self.x += x
        self.y += y

    def get_bounds(self) -> tuple[int, int, int, int]:
        return (self.x, self.y, self.width, self.height)


class Circle:
    """Circle implements Drawable protocol."""
    def __init__(self, x: int, y: int, radius: int):
        self.x = x
        self.y = y
        self.radius = radius

    def draw(self) -> None:
        print(f"Drawing circle at ({self.x}, {self.y}) with radius {self.radius}")

    def move(self, x: int, y: int) -> None:
        self.x += x
        self.y += y

    def get_bounds(self) -> tuple[int, int, int, int]:
        return (self.x - self.radius, self.y - self.radius,
                self.radius * 2, self.radius * 2)


class Point:
    """Point DOES NOT implement full Drawable protocol (missing get_bounds)."""
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def draw(self) -> None:
        print(f"Drawing point at ({self.x}, {self.y})")

    def move(self, x: int, y: int) -> None:
        self.x += x
        self.y += y


# ============================================================================
# Protocol Functions (Generic over Protocol)
# ============================================================================

def render_shape(shape: Drawable) -> None:
    """Render any drawable shape."""
    shape.draw()
    bounds = shape.get_bounds()
    print(f"  Bounds: {bounds}")


def render_all(shapes: list[Drawable]) -> None:
    """Render a list of drawable shapes."""
    for shape in shapes:
        render_shape(shape)


def get_size(obj: Sized) -> int:
    """Get size of any Sized object."""
    return len(obj)


def compare_items(a: Comparable, b: Comparable) -> str:
    """Compare two comparable items."""
    if a < b:
        return "first is less"
    elif a > b:
        return "first is greater"
    else:
        return "equal"


def add_numeric(a: Numeric, b: Numeric) -> Numeric:
    """Add two numeric values."""
    return a + b


# ============================================================================
# Protocol Composition
# ============================================================================

@runtime_checkable
class ComparableContainer(Sized, Comparable, Protocol):
    """Protocol combining Sized and Comparable."""
    pass


class SortedList:
    """Custom list that maintains sorted order."""
    def __init__(self, items=None):
        self._items = sorted(items) if items else []

    def __len__(self) -> int:
        return len(self._items)

    def __lt__(self, other: 'SortedList') -> bool:
        return len(self) < len(other)

    def __le__(self, other: 'SortedList') -> bool:
        return len(self) <= len(other)

    def __gt__(self, other: 'SortedList') -> bool:
        return len(self) > len(other)

    def __ge__(self, other: 'SortedList') -> bool:
        return len(self) >= len(other)

    def __eq__(self, other) -> bool:
        return len(self) == len(other)

    def __ne__(self, other) -> bool:
        return len(self) != len(other)

    def add(self, item):
        self._items.append(item)
        self._items.sort()


# ============================================================================
# Advanced: Generic Protocols with Type Parameters
# ============================================================================

from typing import TypeVar, Generic

T = TypeVar('T')


@runtime_checkable
class Container(Protocol[T]):
    """Generic container protocol."""
    def __contains__(self, item: T) -> bool: ...
    def __len__(self) -> int: ...
    def add(self, item: T) -> None: ...


class Bag(Generic[T]):
    """Simple bag/multiset implementation."""
    def __init__(self):
        self._items: list[T] = []

    def __contains__(self, item: T) -> bool:
        return item in self._items

    def __len__(self) -> int:
        return len(self._items)

    def add(self, item: T) -> None:
        self._items.append(item)

    def count(self, item: T) -> int:
        return self._items.count(item)


def container_stats(container: Container[T]) -> dict[str, int]:
    """Get statistics for any container."""
    return {
        'size': len(container),
        'is_empty': len(container) == 0
    }


# ============================================================================
# Async Protocols
# ============================================================================

from typing import AsyncIterator


@runtime_checkable
class AsyncReadable(Protocol):
    """Protocol for async readable streams."""
    async def read(self, size: int = -1) -> bytes: ...
    async def close(self) -> None: ...


class AsyncFileReader:
    """Async file reader implementing AsyncReadable."""
    def __init__(self, filename: str):
        self.filename = filename
        self.file = None

    async def open(self):
        # Simulate async file opening
        print(f"Opening {self.filename}")
        self.file = open(self.filename, 'rb')

    async def read(self, size: int = -1) -> bytes:
        if self.file is None:
            raise ValueError("File not opened")
        return self.file.read(size)

    async def close(self) -> None:
        if self.file:
            self.file.close()
            print(f"Closed {self.filename}")


async def process_stream(stream: AsyncReadable) -> int:
    """Process an async readable stream."""
    total_bytes = 0
    chunk = await stream.read(1024)
    while chunk:
        total_bytes += len(chunk)
        chunk = await stream.read(1024)
    await stream.close()
    return total_bytes


# ============================================================================
# Context Manager Protocol
# ============================================================================

@runtime_checkable
class ResourceManager(Protocol):
    """Protocol for resource managers."""
    def __enter__(self): ...
    def __exit__(self, exc_type, exc_val, exc_tb): ...


class DatabaseConnection:
    """Database connection with context manager support."""
    def __init__(self, db_name: str):
        self.db_name = db_name
        self.connected = False

    def __enter__(self):
        print(f"Connecting to {self.db_name}")
        self.connected = True
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print(f"Closing connection to {self.db_name}")
        self.connected = False
        return False

    def query(self, sql: str):
        if not self.connected:
            raise RuntimeError("Not connected")
        print(f"Executing: {sql}")


def use_resource(resource: ResourceManager):
    """Use any resource manager."""
    with resource:
        print("Resource is active")


# ============================================================================
# Examples Usage
# ============================================================================

def main():
    print("=" * 70)
    print("Protocol Checking Examples")
    print("=" * 70)

    # Example 1: Drawable protocol
    print("\n1. Drawable Protocol:")
    print("-" * 70)
    rect = Rectangle(10, 20, 100, 50)
    circle = Circle(50, 50, 25)

    render_shape(rect)
    render_shape(circle)

    shapes: list[Drawable] = [rect, circle]
    print("\nRendering all shapes:")
    render_all(shapes)

    # Example 2: Sized protocol
    print("\n2. Sized Protocol:")
    print("-" * 70)
    print(f"String size: {get_size('hello')}")
    print(f"List size: {get_size([1, 2, 3, 4, 5])}")
    print(f"Dict size: {get_size({'a': 1, 'b': 2})}")

    # Example 3: Comparable protocol
    print("\n3. Comparable Protocol:")
    print("-" * 70)
    print(f"Comparing 5 and 10: {compare_items(5, 10)}")
    print(f"Comparing 'abc' and 'xyz': {compare_items('abc', 'xyz')}")

    # Example 4: Numeric protocol
    print("\n4. Numeric Protocol:")
    print("-" * 70)
    print(f"Adding integers: {add_numeric(5, 3)}")
    print(f"Adding floats: {add_numeric(2.5, 3.7)}")

    # Example 5: Protocol composition
    print("\n5. Protocol Composition:")
    print("-" * 70)
    list1 = SortedList([1, 3, 5])
    list2 = SortedList([2, 4, 6, 8])
    print(f"List1 length: {len(list1)}")
    print(f"List2 length: {len(list2)}")
    print(f"List1 < List2: {list1 < list2}")

    # Example 6: Generic protocols
    print("\n6. Generic Protocols:")
    print("-" * 70)
    bag: Container[str] = Bag()
    bag.add("apple")
    bag.add("banana")
    bag.add("apple")
    print(f"Bag stats: {container_stats(bag)}")
    print(f"Contains 'apple': {'apple' in bag}")

    # Example 7: Context manager protocol
    print("\n7. Context Manager Protocol:")
    print("-" * 70)
    db = DatabaseConnection("mydb")
    use_resource(db)

    # Example 8: Type checking errors (these should fail)
    print("\n8. Protocol Violations (Expected Errors):")
    print("-" * 70)
    point = Point(10, 20)
    # This should fail: Point doesn't implement full Drawable protocol
    # render_shape(point)  # Uncomment to see error
    print("Note: Point doesn't implement get_bounds(), so it fails Drawable")

    print("\n" + "=" * 70)
    print("Protocol checking demonstration complete!")
    print("=" * 70)


if __name__ == "__main__":
    main()

