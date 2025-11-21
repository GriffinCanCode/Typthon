"""
Example: Real Source Location Extraction from AST Nodes

This example demonstrates the new source location extraction system in Typthon.
All error messages now include precise line and column information extracted
from the rustpython AST.

## Key Features:

1. **LineIndex**: Fast byte-offset to line:column conversion
2. **SourceLocationExt**: Extension trait for extracting locations from AST nodes
3. **Real locations**: No more placeholder (0, 0, 0, 0) locations
4. **Accurate error reporting**: Point users to exact error locations

## Usage Pattern:

```python
from typthon import check_file

# Type check a file - errors now have real locations
errors = check_file("myfile.py")
for error in errors:
    # Each error has accurate line:col information
    print(f"{error.file}:{error.line}:{error.col}: {error.message}")
```

## Architecture:

The system works in three layers:

1. **Parsing**: rustpython-parser provides byte offsets via the `Ranged` trait
2. **Indexing**: LineIndex converts byte offsets to (line, column) positions
3. **Extraction**: SourceLocationExt trait provides `.source_location(index)`

## Performance:

- LineIndex creation: O(n) where n = source length
- Location lookup: O(log m) where m = number of lines (binary search)
- Memory: O(m) to store line start offsets

## Example Type Errors with Real Locations:

"""

# Example 1: Undefined variable
def example_undefined():
    x = 10
    y = x + z  # Error at line 49, col 12: Undefined variable: z

# Example 2: Type mismatch
def example_mismatch(x: int) -> str:
    return x + 1  # Error at line 53, col 11: Type mismatch: expected str, found int

# Example 3: Invalid attribute access
class Point:
    x: int
    y: int

def example_attribute():
    p = Point()
    p.z = 10  # Error at line 62, col 4: Type Point has no attribute 'z'

# Example 4: Multi-line error
def example_multiline(
    x: int,
    y: str
):
    # Error spans from line 67 to line 71
    return x + y  # Error: Cannot add int and str

# Example 5: Comprehension error
def example_comprehension():
    # Error at specific location in comprehension
    result = [x * 2 for x in range(10) if x > "5"]  # Error: Cannot compare int and str

# Example 6: Lambda error
def example_lambda():
    # Error in lambda body
    f = lambda x: x + "hello"  # Error when x is int
    return f(42)

"""
## Implementation Notes:

### From rustpython AST to SourceLocation:

```rust
use crate::ast::{LineIndex, SourceLocationExt};

// Create index from source
let index = LineIndex::new(source);

// Extract location from any AST node
let loc = expr.source_location(&index);
let loc = stmt.source_location(&index);
let loc = pattern.source_location(&index);
```

### Design Rationale:

Why extension trait instead of From<&Expr>?
- Rust orphan rules: Can't implement external traits for external types
- Flexibility: Extension traits allow passing context (LineIndex)
- Performance: Reuse LineIndex across multiple extractions

Why LineIndex instead of direct parsing?
- Separation of concerns: Parsing vs location mapping
- Caching: Reuse index for entire file analysis
- Accuracy: Binary search ensures O(log m) lookups

### Integration with Error System:

The error system already supported SourceLocation:

```rust
pub struct TypeError {
    pub kind: ErrorKind,
    pub location: SourceLocation,  // Now populated with real data!
    pub file: String,
    pub suggestions: Vec<String>,
}
```

Now instead of:
```rust
SourceLocation::new(0, 0, 0, 0)  // Placeholder
```

We use:
```rust
expr.source_location(&line_index)  // Real location!
```

## Future Enhancements:

1. **Source caching**: Cache LineIndex for frequently-analyzed files
2. **Span highlighting**: Extract full source text for error spans
3. **IDE integration**: Provide locations for LSP goto-definition
4. **Incremental updates**: Update LineIndex when source changes
5. **Multi-file support**: Track locations across imports

## Testing:

See `src/ast/location.rs` for comprehensive tests:
- Line boundary detection
- Multi-line expressions
- Edge cases (empty files, single line, etc.)
- Performance benchmarks
"""

if __name__ == "__main__":
    # These examples will produce errors with real line:column info
    example_undefined()
    example_mismatch(10)
    example_attribute()
    example_multiline(1, "test")
    example_comprehension()
    example_lambda()

