# Typthon C++ Type System

High-performance C++ implementation of Typthon's core type system with SIMD-optimized set operations and FFI bindings for Rust interoperability.

## Architecture

```
cpp/
├── include/typthon/          # Public headers
│   ├── typthon.hpp           # Main header (include this)
│   ├── core/                 # Core type system
│   │   └── types.hpp         # TypeSet & TypeLattice
│   └── ffi/                  # Foreign Function Interface
│       └── ffi.hpp           # C ABI for Rust
└── src/                      # Implementation files
    ├── core/
    │   └── types.cpp         # Type system implementation
    └── ffi/
        └── ffi.cpp           # FFI implementation
```

## Modules

### Core (`core/`)

The core type system module provides efficient data structures and algorithms for type operations:

- **`TypeSet`**: SIMD-optimized bit-vector implementation for fast set operations
  - Supports up to 4096 unique types
  - Cache-aligned memory layout (64-byte alignment)
  - Architecture-specific optimizations:
    - x86/x64: AVX2 vectorization
    - ARM: NEON vectorization
    - Fallback: Scalar implementation
  - Operations: union (∪), intersection (∩), difference (\), subset (⊆), cardinality

- **`TypeLattice`**: Type hierarchy operations for subtyping relationships
  - `meet(a, b)`: Greatest lower bound (most specific common type)
  - `join(a, b)`: Least upper bound (most general common type)
  - `is_subtype(a, b)`: Check if type `a` is a subtype of `b` (a <: b)

### FFI (`ffi/`)

The FFI module provides C-compatible bindings for Rust interoperability:

- **`CTypeSet`**: Opaque C struct wrapping `TypeSet`
  - `typeset_new()`, `typeset_free()`: Lifecycle management
  - `typeset_insert()`, `typeset_contains()`: Element operations
  - `typeset_union()`, `typeset_intersection()`: Set operations
  - `typeset_is_subset()`, `typeset_cardinality()`: Queries

- **Type Lattice Functions**: Direct C bindings
  - `type_is_subtype()`, `type_meet()`, `type_join()`

## Usage

### C++ Usage

Include the main header to access all components:

```cpp
#include <typthon/typthon.hpp>

using namespace typthon;

// Create type sets
TypeSet set1, set2;
set1.insert(42);
set2.insert(42);
set2.insert(100);

// Perform set operations
TypeSet union_set = set1 | set2;
TypeSet intersect_set = set1 & set2;
bool is_subset = set1.is_subset_of(set2);

// Type lattice operations
TypeId meet = TypeLattice::meet(42, 100);
TypeId join = TypeLattice::join(42, 100);
bool subtype = TypeLattice::is_subtype(42, 100);
```

### Rust FFI Usage

The C bindings allow Rust to safely call into the C++ implementation:

```rust
extern "C" {
    fn typeset_new() -> *mut CTypeSet;
    fn typeset_free(set: *mut CTypeSet);
    fn typeset_insert(set: *mut CTypeSet, id: u64);
    fn type_is_subtype(a: u64, b: u64) -> bool;
}
```

## Performance Features

1. **SIMD Optimizations**: Automatic vectorization for set operations on supported architectures
2. **Cache-Friendly**: 64-byte aligned data structures for optimal cache line usage
3. **Zero-Copy FFI**: Opaque pointers avoid marshaling overhead
4. **Inline Functions**: Critical path operations are inlined for maximum performance

## Build Integration

This library is built as part of the Typthon project via the `build.rs` script. The C++ code is compiled and linked into the Rust binary through the `cxx` crate or direct FFI bindings.

### Compiler Requirements

- **C++17** or later
- **x86/x64**: AVX2 support recommended (automatic fallback to scalar)
- **ARM**: NEON support recommended (automatic fallback to scalar)

### Include Paths

When building, ensure the `include/` directory is in your include path:

```bash
# Example compile command
g++ -std=c++17 -I./include -O3 -march=native src/**/*.cpp
```

## Type System Design

### Type Identifiers

Types are represented as 64-bit unique identifiers (`TypeId`), allowing for:
- Fast comparison and hashing
- Compact representation
- Direct use in bit-vector operations

### Set Operations

The `TypeSet` class uses a bit-vector representation where each bit corresponds to a type ID. This enables:
- O(1) insertion, removal, and membership testing
- O(n/64) set operations with SIMD (n = max type IDs)
- Minimal memory footprint (512 bytes for 4096 types)

### Type Hierarchy

The `TypeLattice` maintains a directed graph of subtyping relationships, enabling:
- Efficient subtype checking via breadth-first search
- Meet and join operations for type inference
- Support for complex type hierarchies

## Contributing

When adding new functionality to the C++ type system:

1. Add interface declarations to appropriate header in `include/typthon/`
2. Implement in corresponding `.cpp` file in `src/`
3. Update `typthon.hpp` if adding new public APIs
4. Maintain C++17 compatibility
5. Add FFI bindings if Rust needs to call the new functionality
6. Follow the existing namespace structure (`typthon::`)

## License

See the LICENSE file in the project root.

