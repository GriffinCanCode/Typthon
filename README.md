# Typthon

A high-performance gradual type system for Python with blazing-fast static analysis and elegant runtime validation.

## Architecture

### Project Structure

The project is organized semantically by functionality rather than implementation language:

```
typthon-core/
├── compiler/           # Core compiler components
│   ├── frontend/       # Parsing, configuration, CLI argument handling
│   ├── ast/            # Abstract syntax tree, visitors, walkers
│   ├── analysis/       # Type checking, inference, effects, protocols
│   ├── types/          # Core type system definitions
│   └── errors/         # Error handling and reporting
├── runtime/            # Runtime support (all languages)
│   ├── python/         # Python runtime and API
│   └── cpp/            # C++ FFI and optimizations
├── bindings/           # FFI layer between languages
├── cli/                # Command-line interface
└── infrastructure/     # Performance, caching, parallelization

Additional:
├── examples/           # Usage examples
├── tests/              # Test suite
├── benches/            # Performance benchmarks
└── docs/               # Documentation
```

**Multi-Layer Performance Design:**

```
┌─────────────────────────────────────────┐
│  Python API Layer (DSL + Validation)    │
│  typthon-core/runtime/python/           │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│  Rust Compiler (Type Checker Engine)    │
│  typthon-core/compiler/                 │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│  C++ Performance Layer (Optimizations)  │
│  typthon-core/runtime/cpp/              │
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

