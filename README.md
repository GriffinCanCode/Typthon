# Typthon

A high-performance gradual type system for Python with blazing-fast static analysis and elegant runtime validation.

## Architecture

**Multi-Layer Performance Design:**

```
┌─────────────────────────────────────────┐
│  Python API Layer (DSL + Validation)    │
│  • Elegant decorator syntax             │
│  • Runtime type validation              │
│  • Developer-friendly errors            │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│  Rust Core (Type Checker Engine)        │
│  • AST parsing & analysis               │
│  • Type inference engine                │
│  • Constraint solving                   │
│  • PyO3 bindings                        │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│  C++ Performance Layer (Set Operations) │
│  • Bit vector operations                │
│  • Cache-optimized algorithms           │
│  • SIMD where applicable                │
└─────────────────────────────────────────┘
```

## Design Philosophy

1. **First Principles**: Built on set theory and lattice theory
2. **Zero-Cost Abstractions**: Pay only for what you use
3. **Gradual Typing**: Mix static and dynamic seamlessly
4. **Structural by Default**: Duck typing that scales
5. **Effect Tracking**: Know what your code does

## Installation

### Python Package
```bash
pip install typthon
```

### CLI Tool
```bash
cargo build --release --bin typthon
# Binary will be at target/release/typthon
```

## CLI Usage

Check Python files from the command line:

```bash
# Check a single file
typthon script.py

# Check multiple files
typthon src/**/*.py

# With options
typthon --strict --no-color myproject/
```

For full CLI documentation, see [CLI_README.md](CLI_README.md).

## Python API Usage

```python
from typthon import type, check, infer

@type("(int, int) -> int")
def add(x, y):
    return x + y

@infer  # Automatic type inference
def process(data):
    return [x * 2 for x in data]

# Static analysis
check("my_module.py")
```

## Features

- 🚀 **Blazing Fast**: Rust + C++ core, faster than mypy
- 🎯 **Precise**: Flow-sensitive type narrowing
- 🔄 **Gradual**: Static analysis + runtime validation
- 🎨 **Elegant**: Clean, pythonic API
- 🔧 **Extensible**: Plugin system for custom types
- ⚡ **Zero Overhead**: Optional runtime checks

## Innovations

- **Union/Intersection** types with O(1) operations via bit vectors
- **Effect types** for tracking side effects
- **Dependent types** lite for validation
- **Smart inference** that learns from runtime behavior
- **Flow-sensitive** analysis for better precision

## Benchmarks

Coming soon. Expected: 10-100x faster than mypy on large codebases.

## License

MIT

