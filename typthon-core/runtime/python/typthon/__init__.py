"""Typthon: High-performance gradual type system for Python."""

from typthon.decorators import type, infer
from typthon.checker import check
from typthon.core import Runtime, validate
from typthon.types import (
    # Type variables
    T, U, V, TypeVar, Generic,
    # Basic constructs
    Union, Intersection, Optional, Literal, Protocol,
    # Advanced type classes
    EffectType, RefinementType, DependentType, NominalType, RecursiveType,
    # Type constructors
    effect, refine, dependent, newtype, recursive,
    # Refinement shortcuts
    Positive, Negative, NonNegative, NonZero, NonEmpty, Even, Odd, Bounded,
    # Effect shortcuts
    IO, Network, Async, Random,
)

__version__ = "0.3.0"

__all__ = [
    # Core decorators
    "type",
    "check",
    "infer",
    "validate",
    # Runtime
    "Runtime",
    # Type variables
    "T", "U", "V",
    "TypeVar",
    "Generic",
    # Basic constructs
    "Union",
    "Intersection",
    "Optional",
    "Literal",
    "Protocol",
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
    # Refinement shortcuts
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
