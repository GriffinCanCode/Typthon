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

The project is organized **semantically by functionality** into independent but cooperating modules:

```
Typthon/
├── typthon/              # Thin wrapper (Python bindings)
├── typthon-core/         # Type checker and analysis engine
│   ├── compiler/         # Core compiler infrastructure
│   │   ├── frontend/     # Parsing, configuration
│   │   ├── ast/          # AST, visitors, walkers, location tracking
│   │   ├── analysis/     # Type checking, inference, effects, protocols
│   │   ├── types/        # Type system, type context, internment
│   │   └── errors/       # Error handling, reporting, suggestions
│   ├── runtime/          # Runtime support (language-agnostic)
│   │   ├── python/       # Python runtime API and validation
│   │   └── cpp/          # C++ FFI and performance optimizations
│   ├── bindings/         # FFI layer between Rust and other languages
│   ├── cli/              # Command-line interface
│   └── infrastructure/   # Performance optimizations
│       ├── arena.rs      # Memory arena allocation
│       ├── cache.rs      # Result caching
│       ├── incremental.rs # Incremental compilation
│       ├── metrics.rs    # Performance tracking
│       └── parallel.rs   # Parallel analysis
├── typthon-compiler/     # Native code compiler (Go)
│   ├── cmd/typthon/      # Compiler binary
│   ├── pkg/
│   │   ├── frontend/     # Parser and AST
│   │   ├── ir/           # Intermediate representation
│   │   ├── ssa/          # SSA construction
│   │   ├── codegen/      # Multi-architecture code generation
│   │   │   ├── amd64/    # x86-64 backend
│   │   │   ├── arm64/    # ARM64 backend
│   │   │   └── riscv64/  # RISC-V backend
│   │   ├── linker/       # Linking and object file generation
│   │   └── interop/      # FFI and language interop
│   └── runtime/          # C runtime (minimal)
└── typthon-runtime/      # Minimal runtime library (Rust → staticlib)
    └── src/
        ├── allocator.rs  # Memory allocator
        ├── gc.rs         # Garbage collector
        ├── builtins.rs   # Core builtins
        ├── interop.rs    # Interoperability
        └── ffi.rs        # C API
```

## Module Responsibilities

### typthon-core (Type Checker)
- **Purpose**: Type checking, inference, analysis
- **Language**: Rust
- **Key Files**: `typthon-core/compiler/**/*.rs`
- **Design**: Modular, cacheable, parallel-friendly
- **Innovation**: Bidirectional inference, effect tracking, protocol checking
- **Status**: Production (current implementation)

### typthon-compiler (Native Compiler)
- **Purpose**: Compile typed Python to native machine code
- **Language**: Go
- **Key Files**: `typthon-compiler/pkg/**/*.go`
- **Design**: Fast compilation, multi-architecture, zero dependencies
- **Innovation**: Go-like compilation speed, seamless interop
- **Status**: Phase 1 - Foundation

### typthon-runtime (Minimal Runtime)
- **Purpose**: Runtime support statically linked into binaries
- **Language**: Rust (compiled to staticlib)
- **Key Files**: `typthon-runtime/src/**/*.rs`
- **Design**: Minimal overhead, no dynamic dependencies
- **Innovation**: Zero-cost abstractions, predictable performance
- **Status**: Phase 1 - Foundation

### Relationship Between Modules

```
typthon-compiler           Uses for type checking
     ↓                    ───────────────────────→    typthon-core
     ↓
     ↓ Links statically
     ↓
     ↓
typthon-runtime
     ↓
     ↓ Embedded in
     ↓
Compiled Binary (standalone)
```

**Flow**:
1. `typthon-compiler` parses Python source
2. Calls `typthon-core` for type checking (subprocess or FFI)
3. Generates native code (IR → SSA → Assembly)
4. Links with `typthon-runtime` (staticlib)
5. Output: Standalone binary

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

## Compilation Model

### Interpreted Path (Current)

```
Python + Types → typthon-core → Type Errors
      ↓
   CPython Interpreter → Execution
```

### Compiled Path (New)

```
Python + Types → typthon-core → Type Errors (or success)
      ↓
typthon-compiler → IR → SSA → Codegen → Link
      ↓
Native Binary (no Python required)
```

### Hybrid Model (Future)

```
Typed functions → Compiled to native
Untyped code    → Interpreted (CPython)
                → Seamless interop
```

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

### Compilation Speed
- Direct assembly generation (no LLVM)
- Parallel compilation of functions
- Minimal optimization passes
- Incremental compilation

## Build System

### Multi-Language Compilation
1. **C++ Layer**: Compiled via `build.rs` using `cc` crate
2. **Rust Core**: Compiled by cargo with LTO and optimization
3. **Go Compiler**: Compiled by Go toolchain
4. **Python Bindings**: Generated by PyO3, packaged by maturin
5. **Distribution**: Platform-specific wheels for PyPI + standalone compiler

### Platform Support
- **Linux**: gcc/clang (type checker), Go (compiler)
- **macOS**: clang with Apple Silicon optimizations
- **Windows**: MSVC (type checker), Go (compiler)

## Testing Strategy

### Unit Tests
- Rust: `cargo test` for type checker
- Go: `go test` for compiler
- Python: `pytest` for API and integration

### Property Tests
- Type inference soundness
- Subtyping transitivity
- Union/intersection laws

### Benchmarks
- Type checker: compare against mypy, pyright
- Compiler: compilation speed vs Go, Rust
- Runtime: execution speed vs CPython, PyPy
- SIMD speedup for set operations

## Future Enhancements

1. **Self-Hosting**: Compiler compiles itself
2. **Language Server**: LSP integration for IDE support
3. **JIT Mode**: Hot functions compiled on-the-fly
4. **Gradual Compilation**: Transparent speedup as types added
5. **Standard Library**: Core modules compiled to native

## Why This Design?

### Compared to mypy
- **Faster**: Rust + C++ vs pure Python
- **More features**: Effects, intersections, dependent types
- **Better inference**: Bidirectional type inference
- **Native compilation**: Not just type checking

### Compared to pyright
- **More control**: Custom type rules
- **Runtime integration**: Validate at runtime
- **Extensible**: Plugin system
- **Native compilation**: Generate binaries

### Compared to Codon
- **Faster compilation**: No LLVM
- **Better interop**: Seamless FFI
- **Gradual**: Works with existing code
- **Modular**: Type checker separate from compiler

### Compared to Go
- **Python syntax**: Familiar to Python devs
- **Gradual typing**: Optional types
- **Rich ecosystem**: Python libraries
- **Similar performance**: Native binaries

## Code Quality

### Rust (Type Checker & Runtime)
- Clippy lints: deny warnings
- Format: rustfmt
- Style: Idiomatic Rust, minimal unsafe

### Go (Compiler)
- Format: gofmt
- Style: Idiomatic Go, favor clarity
- Linting: golangci-lint

### C++
- Standard: C++17
- Style: Modern C++, RAII
- Optimization: -O3, -march=native

### Python
- Format: ruff
- Type hints: Full coverage
- Style: Pythonic, PEP 8

## Elegance Principles

1. **One Word Names**: Files and modules have memorable single-word names
2. **Short Functions**: Each function does one thing well
3. **Strong Typing**: Maximum type safety in all layers
4. **Zero Duplication**: DRY principle strictly enforced
5. **First Principles**: Every design decision justified from fundamentals
6. **Semantic Organization**: By functionality, not language
7. **Minimal Dependencies**: Only essential libraries

## Status

- **typthon-core**: ✓ Production (type checker)
- **typthon-compiler**: ⚡ Phase 1 (foundation)
- **typthon-runtime**: ⚡ Phase 1 (foundation)

See `ROADMAP.md` for detailed development plan.
