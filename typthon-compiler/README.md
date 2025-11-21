# Typthon Compiler

A high-performance compiler for typed Python that generates native machine code across all architectures.

## Philosophy

**Fast Compilation**: Sub-second builds for rapid iteration
**Multi-Architecture**: x86-64, ARM64, RISC-V support out of the box
**Zero Dependencies**: Standalone binaries, no Python installation required
**Go-like Speed**: Compilation speed competitive with Go compiler
**Seamless Interop**: Call C, Rust, Zig, Swift, Go with minimal boilerplate

## Architecture

```
Python + Types → Parser → AST → Type Check → IR → SSA → Codegen → Native Binary
                                    ↑                               ↓
                                Typthon Core                    Standalone
```

## Project Structure

Organized semantically by compiler phase:

- `cmd/typthon/` - Compiler binary entry point
- `pkg/frontend/` - Parser, lexer, AST construction
- `pkg/ir/` - Intermediate representation
- `pkg/ssa/` - Static single assignment form
- `pkg/codegen/` - Architecture-specific code generation
  - `amd64/` - x86-64 backend
  - `arm64/` - ARM64 backend
  - `riscv64/` - RISC-V backend
- `pkg/linker/` - Object file generation and linking
- `pkg/interop/` - FFI and language interoperability
- `runtime/` - Minimal runtime (GC, allocator, builtins) in C
- `stdlib/` - Standard library in Typthon
- `internal/` - Internal utilities

## Design Principles

1. **Semantic Organization**: By compiler phase, not implementation detail
2. **One-Word Names**: Memorable, clear package names
3. **Minimal Files**: Each file does one thing exceptionally well
4. **Strong Typing**: Leverage Go's type system fully
5. **Zero Allocation**: Hot paths avoid allocations
6. **Parallel**: Concurrent compilation by default

## Building

```bash
# Build compiler
go build -o bin/typthon ./cmd/typthon

# Compile Python to native binary
bin/typthon compile program.py -o program
./program
```

## Integration

The compiler uses `typthon-core` for type checking:
- Via subprocess: Fast iteration during development
- Via FFI: Production integration for zero-copy
- Independent: Can evolve separately

## Status

Phase 1: Foundation - Proof of concept with simple functions → x86-64

