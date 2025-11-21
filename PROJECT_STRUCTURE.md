# Typthon Project Structure

## Overview

Typthon consists of three independent but cooperating modules:

```
Typthon/
├── typthon-core/         ✓ Production - Type checker and analysis
├── typthon-compiler/     ⚡ Phase 1   - Native code compiler
└── typthon-runtime/      ⚡ Phase 1   - Minimal runtime library
```

## Module Architecture

### 1. typthon-core/ (Type Checker)

**Language**: Rust
**Purpose**: Type checking, inference, and analysis engine
**Status**: ✓ Production

```
typthon-core/
├── compiler/
│   ├── analysis/      # Type checking, inference, effects, protocols
│   │   ├── checker.rs
│   │   ├── inference.rs
│   │   ├── bidirectional.rs
│   │   ├── effects.rs
│   │   ├── protocols.rs
│   │   ├── refinement.rs
│   │   └── constraints.rs
│   ├── ast/           # AST, visitors, location tracking
│   │   ├── location.rs
│   │   ├── visitor.rs
│   │   └── walker.rs
│   ├── types/         # Core type system
│   │   ├── types.rs
│   │   └── intern.rs
│   ├── frontend/      # Parser, config, CLI
│   │   ├── parser.rs
│   │   ├── config.rs
│   │   └── cli.rs
│   └── errors/        # Error handling
│       └── mod.rs
├── infrastructure/    # Performance optimizations
│   ├── arena.rs       # Memory arenas
│   ├── cache.rs       # Result caching
│   ├── incremental.rs # Incremental compilation
│   ├── metrics.rs     # Performance tracking
│   └── parallel.rs    # Parallel analysis
├── runtime/           # Runtime APIs (language-specific)
│   ├── python/        # Python API
│   └── cpp/           # C++ performance layer
├── bindings/          # FFI layer
│   ├── python_ffi.rs
│   └── cpp_ffi.rs
└── cli/               # CLI binary
    └── main.rs
```

**Key Features**:
- Bidirectional type inference
- Effect system tracking
- Protocol checking
- Refinement types
- SIMD-optimized set operations

### 2. typthon-compiler/ (Native Compiler)

**Language**: Go
**Purpose**: Compile typed Python to native machine code
**Status**: ⚡ Phase 1 - Foundation

```
typthon-compiler/
├── cmd/
│   └── typthon/       # Compiler binary entry point
│       └── main.go
├── pkg/
│   ├── frontend/      # Parser and AST construction
│   │   └── frontend.go
│   ├── ir/            # Intermediate representation (three-address code)
│   │   └── ir.go
│   ├── ssa/           # SSA construction and optimization
│   │   └── ssa.go
│   ├── codegen/       # Multi-architecture code generation
│   │   ├── amd64/     # x86-64 backend
│   │   │   └── amd64.go
│   │   ├── arm64/     # ARM64 backend
│   │   │   └── arm64.go
│   │   └── riscv64/   # RISC-V backend
│   │       └── riscv64.go
│   ├── linker/        # Object file generation and linking
│   │   └── linker.go
│   └── interop/       # FFI and language interoperability
│       └── interop.go
├── runtime/           # C runtime (minimal, will be moved to typthon-runtime)
├── stdlib/            # Standard library in Typthon
└── internal/          # Internal utilities
    ├── util/
    └── debug/
```

**Key Features**:
- Fast compilation (Go-like speed)
- Multi-architecture support (x86-64, ARM64, RISC-V)
- Direct assembly generation (no LLVM)
- Seamless FFI to C/Rust/etc.

**Design Philosophy**:
- Optimize for compilation speed over runtime perfection
- Minimal dependencies (self-contained)
- Parallel compilation by default

### 3. typthon-runtime/ (Minimal Runtime)

**Language**: Rust (compiled to staticlib)
**Purpose**: Runtime support statically linked into compiled binaries
**Status**: ⚡ Phase 1 - Foundation

```
typthon-runtime/
├── src/
│   ├── lib.rs         # Main entry point
│   ├── allocator.rs   # Memory allocator (bump + arena)
│   ├── gc.rs          # Garbage collector (refcount + cycle detection)
│   ├── builtins.rs    # Core built-in functions (print, len, range)
│   ├── interop.rs     # Language interoperability
│   └── ffi.rs         # C API for generated code
├── benches/           # Performance benchmarks
│   ├── allocator.rs
│   └── gc.rs
└── examples/          # Usage examples
    └── minimal.rs
```

**Key Features**:
- Minimal overhead (16 bytes per object)
- Fast allocation (~10ns per object)
- Reference counting + cycle detection
- Zero dynamic dependencies
- Statically linked into binaries

**Object Layout**:
```
Object Header (16 bytes):
  - TypeInfo*  (8 bytes)
  - RefCount   (4 bytes)
  - Flags      (4 bytes)
  - Data       (variable)
```

## Module Relationships

```
┌─────────────────────┐
│  typthon-compiler   │
│      (Go)           │
└──────────┬──────────┘
           │
           │ Uses for type checking
           ├────────────────────────→ ┌─────────────────────┐
           │                          │   typthon-core      │
           │                          │     (Rust)          │
           │                          └─────────────────────┘
           │
           │ Links statically
           ├────────────────────────→ ┌─────────────────────┐
           │                          │  typthon-runtime    │
           │                          │  (Rust→staticlib)   │
           │                          └─────────────────────┘
           ↓
    ┌──────────────┐
    │   Binary     │
    │ (standalone) │
    └──────────────┘
```

## Compilation Flow

### Current (Type Checking Only)

```
Python + Types → typthon-core → Type Errors
      ↓
   CPython → Execution
```

### Future (Native Compilation)

```
Python + Types
      ↓
typthon-core (type check)
      ↓
typthon-compiler:
  ├─→ Parse
  ├─→ Generate IR
  ├─→ SSA Construction
  ├─→ Optimize
  ├─→ Codegen (x64/arm64/rv64)
  └─→ Link with typthon-runtime
      ↓
  Native Binary (standalone, no Python needed)
```

## Build Commands

### Type Checker
```bash
cargo build --release                    # Build library
cargo build --release --bin typthon      # Build CLI
cargo test                               # Run tests
```

### Compiler
```bash
cd typthon-compiler
make build                               # Build compiler
./bin/typthon version                    # Test
```

### Runtime
```bash
cd typthon-runtime
cargo build --release                    # Build staticlib
# Output: target/release/libtypthon_runtime.a
```

## Design Patterns

All three modules follow consistent patterns:

1. **Semantic Organization**: By functionality, not language
2. **One-Word Names**: Memorable, clear names (checker, inference, codegen, allocator)
3. **Short Files**: Focused, single-purpose files
4. **Strong Typing**: Leverage language type systems fully
5. **Minimal Dependencies**: Only essential libraries
6. **Zero Duplication**: DRY principle enforced

## Why This Structure?

### Separation of Concerns
- **Type Checker**: Mature, production-ready, language-agnostic analysis
- **Compiler**: Experimental, fast iteration, focused on codegen
- **Runtime**: Minimal, stable, versioned independently

### Independent Evolution
- Type checker can improve without breaking compiler
- Compiler can experiment without touching type checker
- Runtime is versioned (can support multiple compiler versions)

### Language Choice
- **Rust** for type checker: Zero-cost abstractions, fearless concurrency
- **Go** for compiler: Fast compilation, excellent cross-compilation
- **Rust** for runtime: Memory safety, no GC, FFI-friendly

### Reusability
- Type checker: Used by compiler, LSP server, linters
- Compiler: Can use different type checkers
- Runtime: Can be used by other Python compilers

## Future Integration

### LSP Server (Language Server Protocol)
```
typthon-lsp/          # New module
├── Uses typthon-core for analysis
└── Provides IDE integration
```

### Package Manager
```
typthon-pkg/          # New module
├── Dependency resolution
├── Binary distribution
└── Compatible with pip/poetry
```

### Standard Library
```
typthon-stdlib/       # New module
├── Core modules compiled to native
└── Python-compatible API
```

## Status Summary

| Module | Language | Status | Purpose |
|--------|----------|--------|---------|
| typthon-core | Rust | ✓ Production | Type checking & analysis |
| typthon-compiler | Go | ⚡ Phase 1 | Native code generation |
| typthon-runtime | Rust | ⚡ Phase 1 | Minimal runtime library |

See `ROADMAP.md` for detailed development plan.

