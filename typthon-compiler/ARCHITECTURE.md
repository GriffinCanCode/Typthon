# Typthon Compiler Architecture

## Design Philosophy

The compiler is built on **first principles from compiler theory**, drawing inspiration from Go's fast compilation model and modern SSA-based optimizers.

### Core Principles

1. **Fast Compilation**: Sub-second builds prioritized over perfect optimization
2. **Direct Code Generation**: No LLVM - emit assembly directly for speed
3. **Multi-Architecture**: Cross-compilation built-in from day one
4. **Zero Dependencies**: Standalone compiler with no external tools required
5. **Semantic Organization**: Structured by compiler phase, not implementation detail

## Compilation Pipeline

```
Source → Lexer → Parser → AST → Type Check → IR → SSA → Optimize → Codegen → Link → Binary
         ↓                        ↑             ↓                     ↓
      Tokens              Typthon Core    Three-Address        Assembly
                                          Code + CFG          (x64/arm64/rv64)
```

### Phase 1: Frontend (Parsing & Type Checking)

**Input**: Python source with type annotations
**Output**: Typed AST

Components:
- Lexer: Tokenization
- Parser: Recursive descent for Python subset
- Type Checker: Integration with `typthon-core` for type validation

### Phase 2: IR Generation

**Input**: Typed AST
**Output**: Three-address code IR

IR Design:
- Explicit temporaries (SSA precursor)
- Typed instructions
- Control flow explicit (branches, jumps)
- Function-level granularity

Example:
```python
def add(a: int, b: int) -> int:
    return a + b
```

Becomes:
```
function add(a: int, b: int) -> int:
  block entry:
    %t0 = load a
    %t1 = load b
    %t2 = add %t0, %t1
    ret %t2
```

### Phase 3: SSA Construction

**Input**: IR with basic blocks
**Output**: SSA form with phi nodes

Algorithm: Cytron et al.'s minimal SSA construction
1. Compute dominance tree
2. Compute dominance frontiers
3. Insert φ nodes at frontiers
4. Rename variables for single assignment

### Phase 4: Optimization (Minimal for Phase 1)

**Input**: SSA
**Output**: Optimized SSA

Phase 1 optimizations (fast and simple):
- Dead code elimination
- Constant folding
- Algebraic simplification (x * 1 → x, x + 0 → x)

Future optimizations:
- Inlining
- Loop optimizations
- Escape analysis
- Devirtualization

### Phase 5: Code Generation

**Input**: SSA
**Output**: Assembly (per architecture)

Register Allocation:
- Linear scan (fast, good-enough)
- Future: Graph coloring for better code

Backends:
- `amd64/`: x86-64 (System V ABI for Unix/macOS, Windows x64 for Windows)
- `arm64/`: ARM64/AArch64 (Apple Silicon, ARM servers)
- `riscv64/`: RISC-V (future-proofing)

Calling Conventions:
- System V (Unix/macOS): args in rdi, rsi, rdx, rcx, r8, r9
- Windows x64: args in rcx, rdx, r8, r9
- ARM64: args in x0-x7
- RISC-V: args in a0-a7

### Phase 6: Linking

**Input**: Assembly per function
**Output**: Native executable

Process:
1. Assemble to object files (ELF/Mach-O/PE)
2. Link with runtime (`typthon-runtime`)
3. Static linking by default (standalone binary)
4. Optional dynamic linking for plugins

## Module Organization

Following Go patterns for clarity and speed:

```
pkg/
├── frontend/        # Lexer, parser, AST
├── ir/              # Intermediate representation
├── ssa/             # SSA construction and optimization
├── codegen/         # Code generation (multi-arch)
│   ├── amd64/       # x86-64 backend
│   ├── arm64/       # ARM64 backend
│   └── riscv64/     # RISC-V backend
├── linker/          # Object file generation and linking
└── interop/         # FFI and language interop
```

### Why Go for the Compiler?

1. **Fast Compilation**: Go compiles itself in <10s
2. **Cross-Compilation**: `GOOS=linux GOARCH=arm64 go build` - done
3. **Simplicity**: Clean language, easy to maintain
4. **Concurrency**: Parallel compilation trivial
5. **Self-Hosted**: Go compiler is written in Go (great reference)

### Why Not LLVM?

LLVM is powerful but **slow**:
- Rust uses LLVM → slow compilation
- Go uses custom backends → fast compilation

Trade-off: We sacrifice 10-20% runtime performance for 100x faster compilation.

## Runtime Integration

The compiler links against `typthon-runtime` (Rust, staticlib):

```
Compiled Code → Calls Runtime Functions → Runtime (GC, allocator, builtins)
```

Runtime is statically linked for standalone binaries.

## Performance Goals

### Compilation Speed
- 100ms for 1K LOC
- 1s for 10K LOC
- Linear scaling

### Generated Code Performance
- 10-50x faster than CPython
- 2-5x faster than PyPy
- Within 2x of C for numeric code

### Binary Size
- <2MB runtime overhead
- Strippable debug info

## Type System Integration

The compiler uses `typthon-core` for type checking:

**Option 1**: Subprocess (development)
```bash
typthon-core check source.py → types.json
typthon compile source.py --types types.json
```

**Option 2**: FFI (production)
```go
import "typthon-core-ffi"
types := typthoncore.Check(source)
```

**Option 3**: Rewrite (long-term)
- Rewrite type checker in Go for tighter integration
- Keep Rust version for Python API

## Code Quality

- **Formatting**: `gofmt` (enforced)
- **Linting**: `golangci-lint`
- **Testing**: Unit tests for each phase
- **Style**: Idiomatic Go, favor clarity over cleverness

## Comparison to Other Compilers

### vs Go Compiler
- **Similarity**: Architecture, SSA, fast compilation
- **Difference**: Python semantics vs Go semantics

### vs Codon
- **Similarity**: Compile typed Python to native
- **Difference**: Custom backends (no LLVM), faster compilation

### vs Nuitka/Cython
- **Similarity**: Compile Python
- **Difference**: Direct to assembly (not via C), type-driven optimizations

## Next Steps

Phase 1 deliverables:
1. Minimal parser (functions, integers, arithmetic)
2. IR generation
3. x86-64 code generator
4. Runtime integration
5. End-to-end: compile `def add(a: int, b: int) -> int: return a + b`

Success metric: Working binary in <1 week

