# Typthon Python Package

High-performance gradual type system for Python with Rust-powered type checking.

## Module Structure

```
typthon/
├── core/           # Runtime validation engine
│   ├── runtime.py      # Runtime class and global instance
│   └── validator.py    # Validation utilities
│
├── decorators/     # Function decorators
│   ├── type.py         # @type decorator for static/runtime checking
│   └── infer.py        # @infer decorator for automatic inference
│
├── checker/        # Static type checking
│   └── check.py        # check() function for file/directory checking
│
├── types/          # Type constructs
│   ├── constructs.py   # Union, Intersection, Optional, Literal
│   ├── protocols.py    # Protocol for structural typing
│   ├── variables.py    # TypeVar, Generic, T, U, V
│   └── effects.py      # effect() and dependent() factories
│
├── _core.abi3.so   # Rust extension (compiled)
└── py.typed        # PEP 561 marker for type checkers
```

## Quick Start

```python
from typthon import type, check, infer, validate

# Type-checked function
@type("(int, int) -> int")
def add(x, y):
    return x + y

# Automatic type inference
@infer
def process(data):
    return [x * 2 for x in data]

# Static type checking
errors = check("my_module.py")

# Runtime validation
validate([1, 2, 3], "list[int]")  # True
```

## Module Descriptions

### `core/` - Runtime Engine
- **runtime.py**: `Runtime` class for validation with caching and optimization
- **validator.py**: `validate()` function for checking values against types

### `decorators/` - Function Decorators
- **type.py**: `@type` decorator with static checking and optional runtime validation
- **infer.py**: `@infer` decorator for automatic type inference from function body

### `checker/` - Static Analysis
- **check.py**: `check()` function for analyzing Python files and directories

### `types/` - Type System
- **constructs.py**: Basic type constructs (Union, Intersection, Optional, Literal)
- **protocols.py**: Structural typing with Protocol
- **variables.py**: Type variables (T, U, V) and Generic base
- **effects.py**: Advanced features (effect tracking, dependent types)

## Architecture

Each module is self-contained with barrel exports through `__init__.py` files. The root `__init__.py` provides a clean public API by re-exporting key functionality.

## Development

All modules follow these principles:
- **Separation of concerns**: Each module has a single, well-defined purpose
- **Minimal dependencies**: Modules import only what they need
- **Fallback behavior**: Graceful degradation when Rust extension isn't available
- **Type safety**: Full type hints throughout (PEP 484)

