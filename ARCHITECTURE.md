# Typthon Architecture

## Design Philosophy

Typthon is built on **first principles** from mathematics and computer science:

### Mathematical Foundations

1. **Set Theory**: Types are sets of values, subtyping is the subset relation (⊆)
2. **Lattice Theory**: Type hierarchies form complete lattices with meet (∧) and join (∨) operations
3. **Category Theory**: Functions are morphisms, composition preserves types
4. **Graph Theory**: Type inference is constraint propagation in a dependency graph

### Performance Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Python Layer (Elegant DSL)                             │
│  • Decorator syntax: @type, @infer                      │
│  • Runtime validation with zero-cost abstractions       │
│  • Developer-friendly error messages                    │
└──────────────────────┬──────────────────────────────────┘
                       │ PyO3 FFI (zero-copy)
┌──────────────────────▼──────────────────────────────────┐
│  Rust Core (Type System Engine)                         │
│  • AST parsing with rustpython-parser                   │
│  • Type inference via constraint solving                │
│  • Parallel analysis with rayon                         │
│  • Concurrent type context with dashmap                 │
└──────────────────────┬──────────────────────────────────┘
                       │ C FFI
┌──────────────────────▼──────────────────────────────────┐
│  C++ Performance Layer (Set Operations)                 │
│  • Cache-aligned bit vectors for type sets              │
│  • SIMD operations (AVX2) for union/intersection        │
│  • O(1) subset checks                                   │
│  • O(1) cardinality via popcount                        │
└─────────────────────────────────────────────────────────┘
```

## Project Structure

The project is organized **semantically by functionality** rather than by implementation language:

```
typthon-core/
├── compiler/           # Core compiler infrastructure
│   ├── frontend/       # Parsing, configuration, CLI argument handling
│   ├── ast/            # Abstract syntax tree, visitors, walkers, location tracking
│   ├── analysis/       # Type checking, inference, effects, protocols, constraints
│   ├── types/          # Core type system definitions, type context, internment
│   └── errors/         # Error handling, reporting, suggestions
├── runtime/            # Runtime support (language-agnostic organization)
│   ├── python/         # Python runtime API and validation
│   └── cpp/            # C++ FFI and performance optimizations
├── bindings/           # FFI layer between Rust and other languages
├── cli/                # Command-line interface (main.rs)
└── infrastructure/     # Performance optimizations
    ├── arena.rs        # Memory arena allocation
    ├── cache.rs        # Result caching
    ├── incremental.rs  # Incremental compilation
    ├── metrics.rs      # Performance tracking
    └── parallel.rs     # Parallel analysis
```

## Layer Responsibilities

### Compiler Layer
- **Purpose**: Core type checking engine, inference, analysis
- **Key Files**: `typthon-core/compiler/**/*.rs`
- **Design**: Modular, cacheable, parallel-friendly
- **Innovation**: Bidirectional inference, effect tracking, protocol checking

### Runtime Layer
- **Purpose**: Language-specific runtime support and APIs
- **Python**: `typthon-core/runtime/python/` - User-facing API, runtime validation
- **C++**: `typthon-core/runtime/cpp/` - Performance-critical operations
- **Design**: Minimal overhead, graceful degradation
- **Key Files**: `src/*.rs`
- **Design**: Fearless concurrency, zero-cost abstractions
- **Innovation**: Flow-sensitive analysis, smart union simplification

### C++ Performance Layer
- **Purpose**: Ultra-fast set operations on type sets
- **Key Files**: `cpp/*.{hpp,cpp}`
- **Design**: Cache-friendly, SIMD-optimized
- **Innovation**: Bit vector representation for 4096+ types

## Type System Design

### Type Representation

```rust
enum Type {
    // Primitives
    Int, Float, Str, Bool, Bytes, None, Any, Never,

    // Containers (structural subtyping)
    List(Box<Type>),
    Tuple(Vec<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),

    // Functions (contravariant params, covariant return)
    Function(Vec<Type>, Box<Type>),

    // Advanced
    Union(Vec<Type>),        // A | B
    Intersection(Vec<Type>), // A & B
    Generic(String, Vec<Type>),
    Class(String),
    Var(u64),               // Type variables for inference
}
```

### Subtyping Rules

1. **Structural**: Containers use structural subtyping
2. **Contravariance**: Function parameters are contravariant
3. **Covariance**: Return types and container elements are covariant
4. **Union Handling**: `A <: B | C` if `A <: B` or `A <: C`
5. **Intersection**: `A & B <: C` if `A <: C` or `B <: C`

### Type Inference Algorithm

1. **Constraint Generation**: Walk AST, generate type constraints
2. **Constraint Solving**: Unification with occurs check
3. **Substitution**: Apply solved constraints
4. **Simplification**: Reduce unions/intersections to normal form

## Performance Optimizations

### Zero-Copy Design
- Python → Rust via PyO3 (no serialization)
- Rust → C++ via raw pointers (no allocation)
- Shared memory for type contexts

### Caching Strategy
- Memoize type inference results
- Cache subtyping queries
- Reuse parsed ASTs

### Parallel Analysis
- Multiple files analyzed concurrently (rayon)
- Thread-safe type context (dashmap)
- Lock-free where possible

### SIMD Optimizations
- AVX2 for bit vector operations
- Vectorized union/intersection
- Fast cardinality via popcount

## Build System

### Multi-Language Compilation
1. **C++ Layer**: Compiled via `build.rs` using `cc` crate
2. **Rust Core**: Compiled by cargo with LTO and optimization
3. **Python Bindings**: Generated by PyO3, packaged by maturin
4. **Distribution**: Platform-specific wheels for PyPI

### Platform Support
- **Linux**: gcc/clang with -march=native
- **macOS**: clang with Apple Silicon optimizations
- **Windows**: MSVC with Windows-specific SIMD

## Testing Strategy

### Unit Tests
- Rust: `cargo test` for core logic
- Python: `pytest` for API and integration
- C++: Inline tests in source

### Property Tests
- Type inference soundness
- Subtyping transitivity
- Union/intersection laws

### Benchmarks
- Compare against mypy, pyright
- Measure SIMD speedup
- Profile hot paths

## Future Enhancements

1. **Incremental Checking**: Cache results, only recheck changed code
2. **Language Server**: LSP integration for IDE support
3. **Effect System**: Track IO, network, mutations
4. **Refinement Types**: Value-level constraints (x > 0)
5. **Gradual Typing**: Learn from runtime to improve static analysis

## Why This Design?

### Compared to mypy
- **Faster**: Rust + C++ vs pure Python
- **More features**: Effects, intersections, dependent types
- **Better inference**: Bidirectional type inference

### Compared to pyright
- **More control**: Custom type rules, not bound to TypeScript
- **Runtime integration**: Validate at runtime, not just static
- **Extensible**: Plugin system for custom types

### Compared to pydantic
- **Static analysis**: Catch errors before runtime
- **Gradual**: Works with existing untyped code
- **Performance**: SIMD-optimized validation

## Code Quality

### Rust
- Clippy lints: deny warnings
- Format: rustfmt
- Style: Idiomatic Rust, zero unsafe when possible

### C++
- Standard: C++17
- Style: Modern C++, RAII, no raw new/delete
- Optimization: -O3, -march=native, LTO

### Python
- Format: ruff
- Type hints: Full coverage with mypy
- Style: Pythonic, PEP 8

## Elegance Principles

1. **One Word Names**: Files and modules have memorable single-word names
2. **Short Functions**: Each function does one thing well
3. **Strong Typing**: Maximum type safety in all layers
4. **Zero Duplication**: DRY principle strictly enforced
5. **First Principles**: Every design decision justified from fundamentals

