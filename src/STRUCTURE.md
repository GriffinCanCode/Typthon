# Typthon Source Structure

This document describes the organization of the `src/` directory for the Typthon Python package.

## Directory Structure

```
src/
├── lib.rs                  # Main crate entry point
├── internal/               # Internal module organization
│   ├── mod.rs             # Internal module declarations
│   ├── core/              # Core integration (stub)
│   │   └── mod.rs
│   ├── compiler/          # Compiler facade (stub)
│   │   └── mod.rs
│   └── runtime/           # Runtime facade (stub)
│       └── mod.rs
└── typhton/               # Main Python package API
    └── lib.rs             # Python bindings and high-level API
```

## Module Overview

### `src/lib.rs`
Main entry point that:
- Declares internal module structure
- Includes and re-exports `typthon-core` functionality
- Exposes the `typhon` module for Python bindings

### `src/internal/`
Organizational structure for internal modules:
- **core**: Integration point (currently a stub)
- **compiler**: Compiler functionality facade (currently a stub)
- **runtime**: Runtime functionality facade (currently a stub)

These are intentionally kept as stubs since the actual implementation comes from `typthon-core` and `typthon-runtime`.

### `src/typhton/lib.rs`
Main Python package API that provides:

#### High-Level Functions
- `check_file()` - Type check a Python file
- `infer_types()` - Infer types from Python source
- `analyze_effects()` - Analyze effects in Python code

#### Python Bindings (when `python` feature is enabled)
- `check_file_py()` - Python wrapper for file checking
- `infer_types_py()` - Python wrapper for type inference
- `analyze_effects_py()` - Python wrapper for effect analysis
- `validate_refinement_py()` - Validate refinement types
- `init_runtime_py()` - Initialize runtime
- `get_runtime_stats()` - Get runtime statistics
- `force_gc_py()` - Force garbage collection

#### Python Classes
- `TypeValidator` - Stateful type validator with methods:
  - `validate()` - Validate Python source
  - `get_type()` - Get type of expression
  - `get_function_effects()` - Get effects of a function
  - `get_function_type()` - Get type signature of a function

- `RuntimeStats` - Runtime statistics container with fields:
  - `gc_collections` - Number of GC collections
  - `heap_allocated` - Bytes allocated on heap

## Python Module Export

The Python module is exported as `typhon` (note the spelling) through the `#[pymodule]` attribute on the `typhon` function in `src/typhton/lib.rs`.

## Building

```bash
# Check compilation
cargo check --features python

# Build Python wheel
maturin build --release --features python
```

## Key Design Decisions

1. **Facade Pattern**: Internal modules (`internal/compiler`, `internal/runtime`, `internal/core`) are facades/stubs that keep the organization clean while the actual implementation comes from `typthon-core` and `typthon-runtime`.

2. **Single Entry Point**: All Python bindings are consolidated in `src/typhton/lib.rs` for easy maintenance.

3. **Clean API**: The `typhon` module exposes only what's needed for the Python package, hiding internal complexity.

4. **Feature Gating**: Python bindings are behind the `python` feature flag to allow building without Python dependencies.

## Exposed Functionality for Python Package

The following is intelligently exposed for Python users:

### Type Checking & Inference
- File-based type checking
- Source code type inference
- Effect analysis
- Refinement type validation

### Runtime Management
- Runtime initialization
- Garbage collection control
- Runtime statistics

### Advanced Features
- Stateful type validator for incremental checking
- Effect system integration
- Refinement type support

This structure ensures clean separation of concerns while providing a comprehensive API for Python users.

