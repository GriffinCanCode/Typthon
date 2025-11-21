# Typthon Runtime

Minimal, high-performance runtime for compiled Typthon programs.

## Philosophy

**Minimal**: Only what's absolutely necessary
**Fast**: Hand-optimized memory management
**Portable**: Works across all architectures
**Static**: Statically linked into binaries (no dynamic dependencies)
**Zero-Cost**: Overhead only when features are used

## Components

- `allocator/` - Fast memory allocator (bump allocation + arena)
- `gc/` - Reference counting + cycle detector
- `builtins/` - Core built-in functions (print, len, etc.)
- `interop/` - FFI support for calling C/Rust/etc.
- `ffi/` - C API for integration

## Design

### Memory Model

**Tagged Pointers**: Small integers inline, objects on heap
**Reference Counting**: Predictable, low-latency deallocation
**Cycle Detection**: Mark-sweep for circular references (rare)
**Arena Allocation**: Fast bulk allocation/deallocation

### Object Layout

```
Object (16 bytes header):
  - TypeInfo* (8 bytes)
  - RefCount (4 bytes)
  - Flags (4 bytes)
  - Data (variable)
```

### Types

```rust
// Core types
int, float, bool, str, bytes, None

// Collections
list, tuple, dict, set

// Custom classes
class MyClass: ...
```

## Building

```bash
# Build static library
cargo build --release

# Output: target/release/libtypthon_runtime.a
# Link this into compiled Python programs
```

## Integration

The compiler automatically links this runtime into generated binaries.

## Performance

- Allocation: ~10ns per small object (bump allocator)
- Refcount inc/dec: 2 instructions (no branches)
- GC pause: <1ms for cycle detection
- Memory overhead: 16 bytes per object

## Status

Phase 1: Foundation - Core allocator and reference counting

