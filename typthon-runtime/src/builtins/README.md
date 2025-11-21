# Builtins Module

Minimal, zero-overhead implementation of core Python built-in functions for compiled code.

## Design Philosophy

1. **Dual APIs**: Both C FFI exports (for compiled Python) and safe Rust APIs
2. **Zero-cost abstractions**: Use Rust's type system for safety without runtime overhead
3. **Modular structure**: Each builtin category in its own focused module
4. **Extensibility**: Traits allow custom types to integrate seamlessly

## Architecture

```
builtins/
├── mod.rs        # Module entry point, public API
├── print.rs      # Output operations (print)
├── len.rs        # Length queries (len)
├── iter.rs       # Iterators (range)
├── tests.rs      # Comprehensive test suite
└── README.md     # This file
```

## Modules

### Print (`print.rs`)
Output operations with optional buffering:
- `print_int(i64)` - Print integers
- `print_str(&str)` - Print strings
- `print_float(f64)` - Print floats
- `Output` trait for custom output targets

### Len (`len.rs`)
Fast length queries via object header:
- `len(obj)` - Generic length computation
- `Sized` trait for custom types
- Direct header reads for minimal overhead

### Iter (`iter.rs`)
Iterator support with full Rust integration:
- `Range` - Python's range() with Iterator/DoubleEndedIterator/ExactSizeIterator
- `range(start, end, step)` - Constructor function
- Full compatibility with Rust's iterator ecosystem

## FFI Interface

All builtins expose C-compatible functions with `typthon_` prefix:

```c
void typthon_print_int(int64_t val);
void typthon_print_str(const uint8_t* ptr, size_t len);
void typthon_print_float(double val);
size_t typthon_len(const uint8_t* obj);
Range typthon_range(int64_t start, int64_t end, int64_t step);
int64_t typthon_range_next(Range* range);
```

## Usage Examples

### Rust
```rust
use typthon_runtime::builtins::{print_int, range, len};

// Printing
print_int(42);

// Iteration
for i in range(0, 10, 2) {
    print_int(i); // 0, 2, 4, 6, 8
}

// Length
let arr = [1, 2, 3];
assert_eq!(len(&arr[..]), 3);
```

### C FFI
```c
#include "typthon_runtime.h"

// Printing
typthon_print_int(42);

// Iteration
Range r = typthon_range(0, 10, 2);
int64_t val;
while ((val = typthon_range_next(&r)) != INT64_MIN) {
    typthon_print_int(val);
}
```

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| `print_*` | O(n) for string length | Buffering planned |
| `len` | O(1) | Direct header read |
| `range.next()` | O(1) | No allocations |
| `range.len()` | O(1) | Pre-computed |

## Future Extensions

1. **More builtins**: `isinstance`, `type`, `str`, `int`, `float`
2. **Buffered I/O**: Thread-local output buffers for batch printing
3. **Custom allocators**: Integration with arena allocator for string formatting
4. **SIMD optimizations**: Vectorized string operations
5. **no_std support**: Complete freestanding implementation

## Implementation Notes

- All FFI functions use `#[no_mangle]` and `extern "C"` for stable ABI
- Range uses `#[repr(C)]` for guaranteed memory layout
- Print operations currently require std - no_std planned
- Length queries assume object header layout (future: unify with allocator)

## Testing

Run comprehensive test suite:
```bash
cargo test --package typthon-runtime --lib builtins::tests
```

All tests validate both Rust APIs and FFI exports for consistency.

