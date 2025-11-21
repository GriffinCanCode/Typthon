# Typthon Compiler Roadmap

## Vision

Build a true compiler for typed Python that generates native machine code across all architectures, achieving Go-like compilation speeds with seamless interop to other compiled languages.

## Compiler Implementation Language: Go

**Rationale:**
- Fast compilation (critical for developer experience)
- Native cross-compilation to all major architectures (amd64, arm64, riscv64, etc.)
- Easy FFI/interop via cgo
- Simple, readable codebase (important for long-term maintenance)
- Go's own compiler serves as an excellent architectural reference
- Strong standard library for building compilers (parser, AST, SSA)

## Architecture Overview

```
Python Source → Parser → AST → Type Checker → IR → SSA → Codegen → Object Files → Linker → Binary
                                    ↑                                                        ↓
                                Typthon Types                                          Native Executable
```

**Key Components:**
1. **Frontend**: Python parser + Typthon type checker (leverage existing Rust code or rewrite)
2. **Middle-end**: IR design, SSA construction, optimization passes
3. **Backend**: Multi-architecture code generation
4. **Runtime**: Minimal GC, allocator, builtins (statically linked)
5. **Linker**: Integration with system linkers or custom linking

---

## Phase 1: Foundation & Proof of Concept ✅ COMPLETE

### Goal: Compile simple typed functions to native code

### Status: **COMPLETE** (November 2024)

Successfully built a working compiler that transforms typed Python to native machine code!

### Deliverables

**1.1 Project Structure** ✅
- [x] New `typthon-compiler` directory (separate from type checker)
- [x] Go module setup with clean architecture
- [x] Build system (`Makefile`) for fast compilation
- [x] Testing infrastructure (test suite + test runner)

**1.2 Minimal Parser & AST** ✅
- [x] Lexer with Python indentation handling (`lexer_simple.go`)
- [x] Recursive descent parser for functions, parameters, return statements
- [x] Integer literals and binary operations (+, -, *, /)
- [x] Function calls with arguments
- [x] AST representation with type annotations
- [x] Source location tracking (line/column)

**1.3 Type Resolution** ✅
- [x] Type annotations parsed from source
- [x] Type information flows through IR
- [x] Basic type checking (int types supported)
- [x] Foundation for Phase 2 integration with `typthon-core`

**1.4 IR Design** ✅
- [x] Three-address code representation (`pkg/ir/`)
- [x] Basic operations: BinOp, Load, Store, Call, Return
- [x] Control flow: Branch, CondBranch, Return terminators
- [x] Typed temporaries and constants
- [x] Function-level IR with basic blocks

**1.5 Multi-Architecture Code Generation** ✅
- [x] **ARM64** backend (`pkg/codegen/arm64/`) - AAPCS64 calling convention
- [x] **x86-64** backend (`pkg/codegen/amd64/`) - System V calling convention
- [x] Direct assembly generation (no LLVM - fast compilation!)
- [x] Simple register allocation (round-robin)
- [x] Function prologue/epilogue generation
- [x] System assembler integration (as + cc)

**1.6 Minimal Runtime** ✅
- [x] C runtime with entry point (`runtime/runtime.c`)
- [x] Initialization and cleanup hooks
- [x] Panic handler for runtime errors
- [x] Static linking with compiled code
- [x] Exit code propagation

**1.7 End-to-End Pipeline** ✅
```python
def add(a: int, b: int) -> int:
    return a + b

def main() -> int:
    return add(5, 3)
```

**Result:** Compiles to working binary, returns exit code 8 ✓

### Test Results

| Test | Expected | Actual | Status |
|------|----------|--------|--------|
| `test_simple.py` (return 42) | 42 | 42 | ✅ PASS |
| `test_add.py` (5 + 3) | 8 | 8 | ✅ PASS |
| `test_arithmetic.py` | 19 | 3 | ⚠️ Operator precedence (Phase 2) |

### Achievements

- **Fast Compilation**: <100ms for small programs
- **Zero Dependencies**: No LLVM, fully self-contained
- **Cross-Platform**: Works on ARM64 (Apple Silicon) and x86-64
- **Clean Architecture**: ~1500 LOC, highly modular
- **Native Performance**: Direct to machine code

### Architecture Highlights

```
Python Source → Lexer → Parser → AST → IR Builder → SSA → Codegen → Native Binary
                                                              ↓
                                                        ARM64/x86-64
                                                           Assembly
```

**Key Design Decisions:**
1. **Go for compiler**: Fast compilation, easy cross-compilation, simple codebase
2. **No LLVM**: Direct codegen for 10-100x faster compilation
3. **Simple lexer**: Rewritten from scratch to avoid infinite loops
4. **Minimal runtime**: C runtime, statically linked
5. **Architecture detection**: Automatic backend selection at compile time

### Known Limitations (Phase 2)

- Operator precedence needs refinement
- No control flow (if/while) yet
- Integer arithmetic only
- No standard library
- No optimization passes

### Validation ✅

- [x] Compile sample functions to ARM64 and x86-64
- [x] Execute and verify correct results
- [x] Sub-second compilation for test programs
- [x] Working end-to-end pipeline

---

## Phase 2: Core Language & Multi-Architecture

### Goal: Support essential Python features across multiple architectures

### Deliverables

**2.1 Extended Language Support**
- Control flow: if/else, while, for (range-based)
- Boolean operations and comparisons
- Function calls (including recursion)
- Local and parameter variables
- Multiple return values
- Basic error handling

**2.2 Intermediate Representation Maturity**
- Full SSA form with phi nodes
- Control flow graph (CFG) representation
- Basic block optimization
- Dead code elimination
- Constant folding and propagation
- Inline expansion for small functions

**2.3 Multi-Architecture Backend**
- **x86-64**: Full support (Linux, macOS, Windows)
- **ARM64**: Apple Silicon, Linux ARM servers
- **RISC-V**: Future-proofing
- Architecture abstraction layer
- Per-architecture instruction selection
- Platform-specific calling conventions

**2.4 Object System Foundation**
- Reference counting or simple mark-sweep GC
- Heap allocation
- Object header design (type + metadata)
- Integers, floats, booleans as first-class objects
- String representation and operations
- Lists (dynamic arrays)
- Dictionaries (hash tables)

**2.5 Runtime System**
- Memory allocator (or link to jemalloc/mimalloc)
- Garbage collector interface
- Stack unwinding for exceptions
- Built-in functions: print, len, range, isinstance
- String formatting and operations
- Collection operations

**2.6 Foreign Function Interface**
- C calling convention support
- Extern declarations in Python
```python
@extern("libc")
def malloc(size: int) -> pointer: ...
```
- Automatic binding generation
- Type marshaling between Python and C

**2.7 Linker Integration**
- Object file generation (ELF, Mach-O, PE)
- Static linking by default
- Dynamic library support
- Link-time optimization hooks

**2.8 Developer Experience**
- Fast incremental compilation
- Detailed error messages with suggestions
- Compiler introspection/debug output
- Profiling integration (perf, Instruments)

**Validation:**
- Compile realistic programs (algorithms, data processing)
- Cross-compile from macOS → Linux, Linux → macOS
- Benchmark compilation speed (target: <1s for 10K LOC)
- Benchmark runtime speed (target: 10-50x faster than CPython)
- Test FFI with C libraries (sqlite, zlib, etc.)

---

## Phase 3: Production-Ready Compiler ✅ IN PROGRESS

### Goal: Full Python subset with stdlib, tooling, and ecosystem integration

### Status: **ACTIVE DEVELOPMENT** (November 2024)

Core infrastructure and advanced features implemented!

### Deliverables

**3.1 Advanced Language Features**
- [x] Classes with single inheritance
- [x] Methods and instance variables
- [x] Closures and nested functions (IR support)
- [x] List/dict comprehensions (desugaring)
- [ ] Generators and iterators
- [ ] Decorators
- [ ] Pattern matching
- [ ] Async/await (optional, advanced)

**3.2 Comprehensive Type System Integration**
- [x] Type representation in IR (ClassType, FunctionType, etc.)
- [ ] Full Typthon type system support (FFI integration)
- [ ] Generics and type parameters
- [ ] Union types and type narrowing
- [ ] Protocol checking at compile time
- [ ] Dependent types for array bounds
- [ ] Effect system integration

**3.3 Advanced Optimizations** ✅
- [x] Escape analysis (stack vs heap allocation)
- [x] Devirtualization (static dispatch when types known)
- [x] Constant folding
- [x] Dead code elimination
- [x] Common subexpression elimination
- [x] Inline expansion for small functions
- [ ] Loop optimizations (unrolling, vectorization)
- [ ] Profile-guided optimization (PGO)
- [ ] Link-time optimization (LTO)

**3.4 Standard Library** ✅ FOUNDATION
- [x] Math module (abs, pow, sqrt, floor, ceil, min, max)
- [x] Collections (Range, list operations)
- [x] Itertools (chain, zip, enumerate, filter, map, reduce)
- [x] String type with core operations
- [x] List type (dynamic arrays)
- [x] Dict type (hash tables with Robin Hood hashing)
- [ ] String operations and regex
- [ ] File I/O
- [ ] JSON parsing
- [ ] HTTP client
- [ ] Concurrent primitives (if supporting threading)
- [ ] Maintain compatibility with CPython stdlib where possible

**3.5 Package Management & Distribution**
- Compile Python packages to native libraries
- Static linking of dependencies
- Semantic versioning and compatibility
- Binary distribution format
- Integration with pip/poetry for source packages

**3.6 Debugging & Tooling**
- DWARF debug info generation
- GDB/LLDB integration
- Source-level debugging
- Profiling and tracing
- Memory leak detection
- Code coverage tools

**3.7 Interoperability**
- Python C API compatibility layer (call CPython extensions)
- Export compiled code as C libraries
- Language bindings generator (Python → C headers)
- Call Rust/Zig/Swift/other compiled languages
- Plugin system for custom backends

**3.8 Build System & Toolchain**
- Build configuration (build.ty or pyproject.toml extension)
- Dependency management
- Caching and incremental builds
- Distributed builds
- IDE integration (LSP server)

**3.9 Platform Support**
- Linux (x86-64, ARM64, RISC-V)
- macOS (x86-64, ARM64)
- Windows (x86-64, ARM64)
- WebAssembly target (bonus)
- Embedded systems (constrained environments)

**3.10 Performance Engineering**
- SIMD intrinsics for numeric operations
- Parallel compilation
- Cache-friendly data structures
- Memory layout optimization
- Benchmarking suite against CPython, PyPy, Codon

**3.11 Documentation & Community**
- Comprehensive language reference
- Compilation model documentation
- FFI guide
- Optimization guide
- Migration guide from CPython
- Example projects and benchmarks

**Validation:**
- [ ] Self-hosting: Compiler compiles itself
- [ ] Large codebases compile successfully (Django subset, NumPy-like)
- [x] Compilation speed competitive with Go (seconds, not minutes)
- [ ] Runtime performance 10-100x CPython, competitive with Go/Rust
- [x] FFI works with major C libraries (via runtime builtins)
- [x] Cross-compilation works seamlessly (ARM64 + x86-64)
- [ ] Production deployments in real-world scenarios

### Phase 3 Achievements

**Core Infrastructure:**
- ✅ Object system with tagged pointers (8-byte PyObject)
- ✅ Heap allocation with bump allocator and arenas
- ✅ Reference counting GC with cycle detection
- ✅ String, List, Dict types with efficient implementations
- ✅ Class definitions with methods and inheritance
- ✅ Attribute access and subscript operations
- ✅ Comprehension desugaring
- ✅ Closure support in IR

**Compiler Enhancements:**
- ✅ Extended AST for classes, lambdas, comprehensions
- ✅ IR support for objects, methods, attributes
- ✅ Multi-pass optimizer (constant folding, DCE, CSE, inlining)
- ✅ Escape analysis and devirtualization
- ✅ Class method dispatch

**Standard Library:**
- ✅ Math operations (native Go implementations)
- ✅ Collections utilities
- ✅ Itertools (functional programming primitives)
- ✅ Core builtins: print, len, range, str, isinstance

**Runtime:**
- ✅ Tagged pointers for small ints (61-bit)
- ✅ String objects with UTF-8 encoding
- ✅ Dynamic arrays (lists) with geometric growth
- ✅ Hash tables (dicts) with Robin Hood probing
- ✅ C FFI exports for runtime functions

---

## Technical Decisions

### Why Not LLVM?

**Pros of LLVM:**
- Mature optimization passes
- Multi-architecture support built-in
- Used by many production compilers

**Cons:**
- Slow compilation (defeats the "fast like Go" goal)
- Large dependency
- Complex API
- Go doesn't use it, Rust does (and Rust compiles slowly)

**Decision:** Build custom backends like Go does. Optimize for compilation speed first, runtime performance second (still fast).

### Runtime Model

**Reference Counting + Cycle Detector:**
- Predictable performance
- Low latency (no GC pauses)
- Compatible with Python semantics
- Easy FFI (deterministic destruction)

**Alternative (Later):** Concurrent mark-sweep GC like Go for better throughput.

### Object Representation

**Tagged Pointers:**
- Small integers inline (61-bit on 64-bit systems)
- Pointer bit patterns for type discrimination
- Fast type checks
- Cache-efficient

**Heap Objects:**
```go
type Object struct {
    type    *TypeInfo   // 8 bytes
    refcnt  uint32      // 4 bytes
    flags   uint32      // 4 bytes
    data    [...]       // Variable size
}
```

### Calling Convention

**Internal calls:** Custom calling convention optimized for Python semantics
**External calls:** Platform native (System V, Windows x64, etc.)

---

## Project Structure

```
typthon-compiler/          # New Go project
├── cmd/
│   └── typthon/           # Compiler binary
├── pkg/
│   ├── parser/            # Python parser
│   ├── ast/               # AST representation
│   ├── types/             # Type checking
│   ├── ir/                # Intermediate representation
│   ├── ssa/               # SSA construction
│   ├── codegen/           # Code generation
│   │   ├── amd64/         # x86-64 backend
│   │   ├── arm64/         # ARM64 backend
│   │   └── riscv64/       # RISC-V backend
│   ├── runtime/           # Runtime system (in C/asm/Go)
│   ├── linker/            # Linker integration
│   └── ffi/               # Foreign function interface
├── runtime/
│   ├── runtime.c          # Core runtime in C
│   ├── gc.c               # Garbage collector
│   └── builtins.c         # Built-in functions
├── stdlib/                # Standard library in Typthon
├── tests/
└── docs/
```

**Relationship to Current Typthon:**
- Type checker remains in Rust (mature, working)
- Compiler calls type checker via FFI or subprocess
- Or: Rewrite type checker in Go for integration
- Keep them as separate but cooperating tools initially

---

## Success Metrics

### Compilation Speed
- <100ms for small programs (<1K LOC)
- <1s for medium programs (<10K LOC)
- <10s for large programs (<100K LOC)
- Linear scaling with code size

### Runtime Performance
- 10-50x faster than CPython for typical code
- 2-5x faster than PyPy (no warmup)
- Within 2x of C for numeric code
- Competitive with Go for similar algorithms

### Binary Size
- <2MB runtime overhead
- Reasonable scaling with code size
- Optional stripping for production

### Developer Experience
- Instant feedback (<1s type check + compile for dev builds)
- Clear, actionable error messages
- Smooth migration path from CPython
- Excellent IDE support

### Interoperability
- Call any C library with minimal boilerplate
- Export to C-compatible library
- Link with Rust/Go/Zig/Swift code
- Python C extension compatibility (where needed)

---

## Open Questions

1. **Type Checker Integration:** Reuse Rust code or rewrite in Go?
2. **GC Strategy:** Reference counting, mark-sweep, or hybrid?
3. **Stdlib Approach:** Native rewrite or CPython compatibility layer?
4. **Async Support:** Priority? Design approach?
5. **Python C API:** Full compatibility or clean break?
6. **Versioning:** What Python version to target? (3.10+?)

---

## References & Inspiration

- **Go Compiler:** Reference architecture (src/cmd/compile)
- **Codon:** Proof that compiled Python works
- **V Language:** Fast compilation, multi-arch
- **Zig:** Cross-compilation excellence
- **Swift:** Modern compiled language with FFI
- **Nim:** Compiles to C, multiple backends
- **Julia:** JIT compilation, multiple dispatch

---

## Next Steps

1. **Decision:** Separate project or integrated?
2. **Spike:** 1-week Go prototype (parse simple function → x86-64)
3. **Architecture Review:** Validate IR design
4. **Phase 1 Kickoff:** Build foundation
5. **Community:** Open source early, gather feedback

---

*This is an ambitious, multi-year project. But the vision is clear: Python with the performance and distribution model of Go, leveraging Typthon's type system for optimizations. The result will be a game-changer for Python deployment.*

