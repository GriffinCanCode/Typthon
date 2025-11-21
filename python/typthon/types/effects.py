"""Advanced type factories for effects, refinements, and dependent types."""

from typing import Callable, TypeVar, Any

T = TypeVar('T')


class EffectType:
    """Effect type wrapper for tracking side effects."""

    def __init__(self, base_type: type, *effects: str):
        self.base_type = base_type
        self.effects = frozenset(effects)

    def __repr__(self) -> str:
        if not self.effects:
            return str(self.base_type)
        return f"{self.base_type} ! {{{', '.join(self.effects)}}}"

    def has_effect(self, effect: str) -> bool:
        return effect in self.effects

    def is_pure(self) -> bool:
        return len(self.effects) == 0 or self.effects == {"pure"}


class RefinementType:
    """Refinement type with value-level predicate."""

    def __init__(self, base_type: type, predicate: str):
        self.base_type = base_type
        self.predicate = predicate

    def __repr__(self) -> str:
        return f"{self.base_type}[{self.predicate}]"

    def validate(self, value: Any) -> bool:
        """Validate value against predicate (runtime check)."""
        # Import here to avoid circular dependency
        from typthon.core.validator import validate_refinement_type
        return validate_refinement_type(value, self.base_type, self.predicate)


class DependentType:
    """Dependent type with value constraints."""

    def __init__(self, base_type: type, constraint: str):
        self.base_type = base_type
        self.constraint = constraint

    def __repr__(self) -> str:
        return f"{self.base_type}[{self.constraint}]"


class NominalType:
    """Nominal type wrapper for structural escape hatch."""

    def __init__(self, name: str, base_type: type):
        self.name = name
        self.base_type = base_type
        self.__name__ = name

    def __repr__(self) -> str:
        return self.name

    def __call__(self, value: Any) -> Any:
        """Constructor that validates type."""
        # In production, would validate against base_type
        return value


class RecursiveType:
    """Recursive type for self-referential structures."""

    def __init__(self, name: str, body: Callable):
        self.name = name
        self.body = body
        self._resolved = None

    def resolve(self):
        """Resolve the recursive type by evaluating body."""
        if self._resolved is None:
            self._resolved = self.body(self)
        return self._resolved

    def __repr__(self) -> str:
        return f"rec {self.name}. {self.body}"


# Effect type factory
def effect(*effects: str) -> Callable[[type], EffectType]:
    """
    Create an effect type annotation.

    Effects:
        - 'pure': No side effects
        - 'io': File/console I/O
        - 'network': Network operations
        - 'mutation': State mutation
        - 'exception': Can throw exceptions
        - 'async': Async/await
        - 'random': Non-deterministic
        - 'time': Time-dependent

    Example:
        >>> IOInt = effect('io')(int)
        >>> @type("() -> IOInt")
        ... def read_input():
        ...     return int(input())
    """
    def wrapper(base_type: type) -> EffectType:
        return EffectType(base_type, *effects)
    return wrapper


# Refinement type factory
def refine(predicate: str) -> Callable[[type], RefinementType]:
    """
    Create a refinement type with value-level constraint.

    Predicates:
        - 'value > 0': Positive numbers
        - 'value < 0': Negative numbers
        - 'len(value) > 0': Non-empty collections
        - 'value % 2 == 0': Even numbers
        - Complex: 'value >= 0 and value <= 100'

    Example:
        >>> PositiveInt = refine('value > 0')(int)
        >>> NonEmptyStr = refine('len(value) > 0')(str)
        >>> @type("(PositiveInt) -> PositiveInt")
        ... def square(x):
        ...     return x * x
    """
    def wrapper(base_type: type) -> RefinementType:
        return RefinementType(base_type, predicate)
    return wrapper


# Dependent type factory
def dependent(constraint: str) -> Callable[[type], DependentType]:
    """
    Create a dependent type with compile-time constraint.

    Constraints:
        - 'len=5': Fixed length 5
        - '0<=len<=10': Length range
        - 'value=n': Value equals parameter n

    Example:
        >>> def fixed_array(n: int) -> type:
        ...     return dependent(f'len={n}')(list)
        >>> Array5 = fixed_array(5)
    """
    def wrapper(base_type: type) -> DependentType:
        return DependentType(base_type, constraint)
    return wrapper


# Nominal type factory
def newtype(name: str) -> Callable[[type], NominalType]:
    """
    Create a nominal type (branded type).

    Unlike structural types, nominal types are compared by name,
    not by structure. This provides type safety through identity.

    Example:
        >>> UserId = newtype('UserId')(int)
        >>> OrderId = newtype('OrderId')(int)
        >>> # UserId and OrderId are distinct types despite both being int
        >>> @type("(UserId) -> User")
        ... def get_user(id):
        ...     return users[id]
    """
    def wrapper(base_type: type) -> NominalType:
        return NominalType(name, base_type)
    return wrapper


# Recursive type factory
def recursive(name: str, body: Callable) -> RecursiveType:
    """
    Create a recursive type.

    Example:
        >>> # Linked list: List[T] = None | (T, List[T])
        >>> def LinkedList(T):
        ...     return recursive('List', lambda self:
        ...         Union[None, Tuple[T, self]])
        >>>
        >>> # JSON: JSON = None | bool | int | float | str | List[JSON] | Dict[str, JSON]
        >>> JSON = recursive('JSON', lambda self:
        ...     Union[None, bool, int, float, str, List[self], Dict[str, self]])
    """
    return RecursiveType(name, body)


# Common refinement types
class Positive(RefinementType):
    """Positive numbers: x > 0"""
    def __init__(self, base_type=int):
        super().__init__(base_type, 'value > 0')


class Negative(RefinementType):
    """Negative numbers: x < 0"""
    def __init__(self, base_type=int):
        super().__init__(base_type, 'value < 0')


class NonNegative(RefinementType):
    """Non-negative numbers: x >= 0"""
    def __init__(self, base_type=int):
        super().__init__(base_type, 'value >= 0')


class NonZero(RefinementType):
    """Non-zero numbers: x != 0"""
    def __init__(self, base_type=int):
        super().__init__(base_type, 'value != 0')


class NonEmpty(RefinementType):
    """Non-empty collections: len(x) > 0"""
    def __init__(self, base_type=str):
        super().__init__(base_type, 'len(value) > 0')


class Even(RefinementType):
    """Even numbers: x % 2 == 0"""
    def __init__(self):
        super().__init__(int, 'value % 2 == 0')


class Odd(RefinementType):
    """Odd numbers: x % 2 != 0"""
    def __init__(self):
        super().__init__(int, 'value % 2 != 0')


def Bounded(min_val: int, max_val: int, base_type=int) -> RefinementType:
    """Bounded numbers: min <= x <= max"""
    return RefinementType(base_type, f'value >= {min_val} and value <= {max_val}')


# Common effect types
IO = lambda t: EffectType(t, 'io')
Network = lambda t: EffectType(t, 'network')
Async = lambda t: EffectType(t, 'async')
Random = lambda t: EffectType(t, 'random')


__all__ = [
    'EffectType', 'RefinementType', 'DependentType', 'NominalType', 'RecursiveType',
    'effect', 'refine', 'dependent', 'newtype', 'recursive',
    'Positive', 'Negative', 'NonNegative', 'NonZero', 'NonEmpty',
    'Even', 'Odd', 'Bounded',
    'IO', 'Network', 'Async', 'Random',
]

