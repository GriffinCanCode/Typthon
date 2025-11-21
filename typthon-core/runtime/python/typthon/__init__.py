"""Typthon: High-performance gradual type system for Python."""

# Import Rust extension bindings
try:
    from typthon.typthon import (
        TypeValidator,
        check_file_py,
        infer_types_py,
        analyze_effects_py,
        validate_refinement_py,
        init_runtime_py,
        get_runtime_stats,
        force_gc_py,
        clear_cache_py,
        get_metrics_py,
    )
except ImportError:
    TypeValidator = None
    check_file_py = None
    infer_types_py = None
    analyze_effects_py = None
    validate_refinement_py = None
    init_runtime_py = None
    get_runtime_stats = None
    force_gc_py = None
    clear_cache_py = None
    get_metrics_py = None

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
    "TypeValidator",
    # Rust functions
    "check_file_py",
    "infer_types_py",
    "analyze_effects_py",
    "validate_refinement_py",
    "init_runtime_py",
    "get_runtime_stats",
    "force_gc_py",
    "clear_cache_py",
    "get_metrics_py",
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
