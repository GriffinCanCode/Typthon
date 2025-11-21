"""Advanced type constructs for Typthon."""

from .constructs import Union, Intersection, Optional, Literal
from .protocols import Protocol
from .variables import T, U, V, TypeVar, Generic
from .effects import (
    EffectType, RefinementType, DependentType, NominalType, RecursiveType,
    effect, refine, dependent, newtype, recursive,
    Positive, Negative, NonNegative, NonZero, NonEmpty, Even, Odd, Bounded,
    IO, Network, Async, Random,
)

__all__ = [
    # Basic constructs
    "Union",
    "Intersection",
    "Optional",
    "Literal",
    # Protocols
    "Protocol",
    # Type variables
    "T", "U", "V",
    "TypeVar",
    "Generic",
    # Advanced type classes
    "EffectType",
    "RefinementType",
    "DependentType",
    "NominalType",
    "RecursiveType",
    # Type constructors
    "effect",
    "refine",
    "dependent",
    "newtype",
    "recursive",
    # Refinement type shortcuts
    "Positive",
    "Negative",
    "NonNegative",
    "NonZero",
    "NonEmpty",
    "Even",
    "Odd",
    "Bounded",
    # Effect shortcuts
    "IO",
    "Network",
    "Async",
    "Random",
]

