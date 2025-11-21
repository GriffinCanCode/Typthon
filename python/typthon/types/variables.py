"""Type variables and generic types."""

from typing import TypeVar as _TypeVar, Generic as _Generic

# Type variables for generic types
T = _TypeVar('T')
U = _TypeVar('U')
V = _TypeVar('V')

# Re-export typing constructs
TypeVar = _TypeVar
Generic = _Generic

