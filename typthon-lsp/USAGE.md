# Typthon LSP Usage Guide

## Quick Start

### Building the Server

```bash
cd typthon-lsp
cargo build --release
```

The compiled binary will be at `target/release/typthon-lsp`.

### Running Tests

```bash
cargo test
```

All 16 tests should pass successfully.

## Features in Action

### 1. Real-time Diagnostics

As you type Python code, the LSP provides immediate feedback on syntax errors:

```python
def invalid(:  # ← Syntax error detected immediately
```

### 2. Hover Information

Hover over any identifier to see type information:

```python
x: int = 5  # Hover over 'int' shows: "Built-in integer type"
```

### 3. Code Completion

#### Keyword Completion
Start typing and get suggestions for Python keywords:
```python
de|  # Suggests: def, del
```

#### Method Completion
After a dot, get context-aware method suggestions:
```python
my_list = [1, 2, 3]
my_list.|  # Suggests: append, extend, pop, etc.
```

### 4. Go to Definition

Click on any function or class reference to jump to its definition:

```python
def hello():
    pass

hello()  # Ctrl+Click on 'hello' jumps to definition above
```

### 5. Find References

Find all usages of a symbol:

```python
def calculate(x):  # Find references shows all uses of 'calculate'
    return x * 2

result = calculate(5)
calculate(10)
# Shows 2 references + definition = 3 total
```

### 6. Rename Symbol

Rename a symbol everywhere it's used:

```python
def old_name():  # Rename to 'new_name'
    pass

old_name()  # Automatically renamed to new_name()
```

### 7. Signature Help

Get function signature hints while typing:

```python
print(|  # Shows: print(*args, sep=' ', end='\n', file=None, flush=False)
      # With documentation: "Print values to a stream..."
```

### 8. Semantic Highlighting

Different colors for different symbol types:
- **Functions**: Blue
- **Classes**: Green
- **Variables**: White
- **Parameters**: Orange
- **Keywords**: Purple

```python
class MyClass:      # Green (Class)
    def method(x):  # Blue (Function), Orange (Parameter)
        value = x   # White (Variable)
        return value
```

### 9. Code Actions

Quick fixes and refactorings available on demand:

```python
# Missing import ← "Add missing import" action available
some_function()

# Untyped variable ← "Add type annotation" action available
x = 5
```

### 10. Inlay Hints

See inferred types inline:

```python
result = calculate(5)  # : Unknown (type hint shown inline)
```

## Editor Configuration

### VS Code

1. Create a workspace folder: `.vscode/extensions/typthon/`
2. Add the extension configuration (see README.md)
3. Install the extension: `code --install-extension .vscode/extensions/typthon`
4. Open a Python file and start coding!

### Neovim

Add to your config:
```lua
require('lspconfig').typthon.setup{
  cmd = {'/path/to/typthon-lsp'},
  filetypes = {'python'},
}
```

### Other Editors

See README.md for Sublime Text, Emacs, and other editor configurations.

## Advanced Usage

### Custom Configuration

Set logging level:
```bash
RUST_LOG=typthon_lsp=debug typthon-lsp
```

### Performance Tips

- The LSP starts in <100ms
- Incremental analysis completes in <50ms for typical changes
- Memory usage: ~50MB for medium projects (10K LOC)

## Troubleshooting

### Server Not Responding

Check the logs:
```bash
RUST_LOG=debug typthon-lsp 2> lsp.log
```

### Completions Not Showing

Ensure trigger characters are configured:
- `.` for attribute access
- `(` for signature help

### References Not Found

The LSP currently only searches within the current document. Cross-file reference search is planned for v0.3.0.

## Example Workflow

1. Open a Python file in your editor
2. Start typing a function:
   ```python
   def calculate_sum(numbers):
       # Hover over 'numbers' to see parameter info
       total = 0  # Inlay hint shows inferred type
       for n in numbers:
           total += n
       return total
   ```

3. Use the function:
   ```python
   result = calculate_sum([1, 2, 3])
   # Get signature help when typing the parentheses
   # Hover over 'calculate_sum' to see the definition
   ```

4. Refactor:
   - Right-click on `calculate_sum` → Find References
   - Rename `calculate_sum` → `sum_numbers`
   - All usages update automatically

5. Add type annotations:
   - Trigger code action on `numbers` parameter
   - Select "Add type annotation"
   - (Future feature - currently returns placeholder)

## API Reference

### LSP Capabilities

The Typthon LSP implements:
- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didSave`
- `textDocument/didClose`
- `textDocument/hover`
- `textDocument/completion`
- `textDocument/definition`
- `textDocument/references`
- `textDocument/rename`
- `textDocument/codeAction`
- `textDocument/signatureHelp`
- `textDocument/semanticTokens/full`
- `textDocument/inlayHint`

### Configuration Options

Currently, the LSP uses default settings. Future versions will support:
- `typthon.enableInlayHints` (boolean)
- `typthon.maxCompletions` (number)
- `typthon.semanticHighlighting` (boolean)

## Next Steps

- Explore the [README.md](README.md) for detailed feature descriptions
- Check [CHANGELOG.md](CHANGELOG.md) for version history
- See [tests/](tests/) for usage examples in code

