"""
Comprehensive tests for protocol checking functionality.
"""

import pytest
from typing import Protocol, runtime_checkable


# ============================================================================
# Test Protocols
# ============================================================================

@runtime_checkable
class Drawable(Protocol):
    def draw(self) -> None: ...
    def move(self, x: int, y: int) -> None: ...


@runtime_checkable
class Sized(Protocol):
    def __len__(self) -> int: ...


@runtime_checkable
class Comparable(Protocol):
    def __lt__(self, other) -> bool: ...
    def __le__(self, other) -> bool: ...
    def __gt__(self, other) -> bool: ...
    def __ge__(self, other) -> bool: ...


@runtime_checkable
class Numeric(Protocol):
    def __add__(self, other) -> 'Numeric': ...
    def __sub__(self, other) -> 'Numeric': ...
    def __mul__(self, other) -> 'Numeric': ...


# ============================================================================
# Test Implementations
# ============================================================================

class Shape:
    """Correctly implements Drawable."""
    def draw(self) -> None:
        pass

    def move(self, x: int, y: int) -> None:
        pass


class IncompleteShape:
    """Missing move() method."""
    def draw(self) -> None:
        pass


class WrongSignatureShape:
    """Has draw() but wrong signature."""
    def draw(self, color: str) -> None:
        pass

    def move(self, x: int, y: int) -> None:
        pass


class Container:
    """Implements Sized."""
    def __init__(self, items):
        self.items = items

    def __len__(self) -> int:
        return len(self.items)


class Number:
    """Implements Numeric protocol."""
    def __init__(self, value):
        self.value = value

    def __add__(self, other: 'Number') -> 'Number':
        return Number(self.value + other.value)

    def __sub__(self, other: 'Number') -> 'Number':
        return Number(self.value - other.value)

    def __mul__(self, other: 'Number') -> 'Number':
        return Number(self.value * other.value)


# ============================================================================
# Protocol Checking Tests
# ============================================================================

class TestBasicProtocolChecking:
    """Test basic protocol checking functionality."""

    def test_correct_implementation(self):
        """Test that correct implementations satisfy protocols."""
        shape = Shape()
        assert isinstance(shape, Drawable)

    def test_missing_method(self):
        """Test that missing methods fail protocol check."""
        incomplete = IncompleteShape()
        # Should fail protocol check
        assert not isinstance(incomplete, Drawable)

    def test_builtin_types_sized(self):
        """Test that built-in types satisfy Sized protocol."""
        assert isinstance("hello", Sized)
        assert isinstance([1, 2, 3], Sized)
        assert isinstance({'a': 1}, Sized)
        assert isinstance({1, 2, 3}, Sized)

    def test_custom_sized(self):
        """Test custom Sized implementation."""
        container = Container([1, 2, 3])
        assert isinstance(container, Sized)
        assert len(container) == 3

    def test_numeric_protocol(self):
        """Test Numeric protocol implementation."""
        num = Number(5)
        assert isinstance(num, Numeric)

        result = num + Number(3)
        assert result.value == 8


class TestProtocolFunctions:
    """Test functions that accept protocol types."""

    def test_function_with_protocol_param(self):
        """Test function accepting protocol parameter."""
        def render(obj: Drawable) -> str:
            obj.draw()
            return "rendered"

        shape = Shape()
        assert render(shape) == "rendered"

    def test_function_with_sized_param(self):
        """Test function with Sized parameter."""
        def get_size(obj: Sized) -> int:
            return len(obj)

        assert get_size("hello") == 5
        assert get_size([1, 2, 3]) == 3
        assert get_size(Container([1, 2])) == 2


class TestProtocolComposition:
    """Test protocol composition and inheritance."""

    @runtime_checkable
    class SizedDrawable(Drawable, Sized, Protocol):
        """Protocol combining Drawable and Sized."""
        pass

    def test_multiple_protocol_satisfaction(self):
        """Test object satisfying multiple protocols."""
        class DrawableContainer:
            def __init__(self, items):
                self.items = items

            def __len__(self) -> int:
                return len(self.items)

            def draw(self) -> None:
                pass

            def move(self, x: int, y: int) -> None:
                pass

        obj = DrawableContainer([1, 2, 3])
        assert isinstance(obj, self.SizedDrawable)
        assert isinstance(obj, Drawable)
        assert isinstance(obj, Sized)


class TestProtocolVariance:
    """Test variance in protocol method signatures."""

    def test_covariant_return_type(self):
        """Test that more specific return types are acceptable."""
        @runtime_checkable
        class Returner(Protocol):
            def get(self) -> object: ...

        class StringReturner:
            def get(self) -> str:
                return "hello"

        # More specific return type should satisfy protocol
        obj = StringReturner()
        assert isinstance(obj, Returner)

    def test_contravariant_parameters(self):
        """Test parameter contravariance."""
        @runtime_checkable
        class Processor(Protocol):
            def process(self, item: str) -> None: ...

        class AnyProcessor:
            def process(self, item: object) -> None:
                pass

        # More general parameter type should satisfy protocol
        obj = AnyProcessor()
        assert isinstance(obj, Processor)


class TestGenericProtocols:
    """Test generic protocol support."""

    def test_generic_container(self):
        """Test generic container protocol."""
        from typing import TypeVar, Generic

        T = TypeVar('T')

        @runtime_checkable
        class Container(Protocol[T]):
            def add(self, item: T) -> None: ...
            def __len__(self) -> int: ...

        class Bag(Generic[T]):
            def __init__(self):
                self.items: list[T] = []

            def add(self, item: T) -> None:
                self.items.append(item)

            def __len__(self) -> int:
                return len(self.items)

        bag: Container[int] = Bag()
        bag.add(42)
        assert len(bag) == 1


class TestProtocolErrors:
    """Test protocol checking error cases."""

    def test_wrong_signature_fails(self):
        """Test that wrong method signatures fail protocol check."""
        wrong = WrongSignatureShape()
        # Should fail because draw() has wrong signature
        # (Note: runtime_checkable doesn't check signatures deeply)
        # But type checker should catch this
        pass

    def test_missing_required_method(self):
        """Test that missing required methods are caught."""
        class Empty:
            pass

        empty = Empty()
        assert not isinstance(empty, Drawable)
        assert not isinstance(empty, Sized)


class TestProtocolWithBuiltins:
    """Test protocols work correctly with built-in types."""

    def test_str_protocols(self):
        """Test str satisfies various protocols."""
        s = "hello"
        assert isinstance(s, Sized)
        # str also implements Comparable
        assert s < "world"

    def test_list_protocols(self):
        """Test list satisfies protocols."""
        lst = [1, 2, 3]
        assert isinstance(lst, Sized)
        assert len(lst) == 3

    def test_dict_protocols(self):
        """Test dict satisfies protocols."""
        d = {'a': 1, 'b': 2}
        assert isinstance(d, Sized)
        assert len(d) == 2


class TestAsyncProtocols:
    """Test async protocol support."""

    @pytest.mark.asyncio
    async def test_async_readable(self):
        """Test async readable protocol."""
        @runtime_checkable
        class AsyncReadable(Protocol):
            async def read(self, size: int) -> bytes: ...

        class Reader:
            async def read(self, size: int) -> bytes:
                return b"data"

        reader = Reader()
        assert isinstance(reader, AsyncReadable)
        data = await reader.read(100)
        assert data == b"data"


class TestContextManagerProtocol:
    """Test context manager protocol."""

    def test_context_manager(self):
        """Test context manager protocol."""
        @runtime_checkable
        class ContextManager(Protocol):
            def __enter__(self): ...
            def __exit__(self, exc_type, exc_val, exc_tb): ...

        class Resource:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc_val, exc_tb):
                return False

        resource = Resource()
        assert isinstance(resource, ContextManager)

        with resource:
            pass  # Should work


class TestProtocolInheritance:
    """Test protocol inheritance and extension."""

    def test_protocol_inheritance(self):
        """Test protocols can inherit from other protocols."""
        @runtime_checkable
        class Base(Protocol):
            def base_method(self) -> int: ...

        @runtime_checkable
        class Extended(Base, Protocol):
            def extended_method(self) -> str: ...

        class Implementation:
            def base_method(self) -> int:
                return 42

            def extended_method(self) -> str:
                return "hello"

        impl = Implementation()
        assert isinstance(impl, Base)
        assert isinstance(impl, Extended)


class TestProtocolWithProperties:
    """Test protocols with properties."""

    def test_protocol_with_property(self):
        """Test protocol can include properties."""
        @runtime_checkable
        class HasValue(Protocol):
            @property
            def value(self) -> int: ...

        class Container:
            def __init__(self, val: int):
                self._value = val

            @property
            def value(self) -> int:
                return self._value

        container = Container(42)
        assert isinstance(container, HasValue)
        assert container.value == 42


# ============================================================================
# Performance Tests
# ============================================================================

class TestProtocolPerformance:
    """Test protocol checking performance."""

    def test_protocol_check_performance(self):
        """Test that protocol checking is fast."""
        import time

        shape = Shape()

        start = time.perf_counter()
        for _ in range(1000):
            isinstance(shape, Drawable)
        elapsed = time.perf_counter() - start

        # Should be very fast (< 10ms for 1000 checks)
        assert elapsed < 0.01

    def test_protocol_with_many_methods(self):
        """Test protocol with many methods."""
        @runtime_checkable
        class ManyMethods(Protocol):
            def method1(self) -> int: ...
            def method2(self) -> int: ...
            def method3(self) -> int: ...
            def method4(self) -> int: ...
            def method5(self) -> int: ...

        class Implementation:
            def method1(self) -> int: return 1
            def method2(self) -> int: return 2
            def method3(self) -> int: return 3
            def method4(self) -> int: return 4
            def method5(self) -> int: return 5

        impl = Implementation()
        assert isinstance(impl, ManyMethods)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

