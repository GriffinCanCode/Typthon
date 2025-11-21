# Changelog

All notable changes to the Typthon LSP will be documented in this file.

## [0.2.0] - 2025-11-21

### Added

#### Core LSP Features
- **Find References**: Complete implementation of finding all symbol references in a document
  - Accurate word boundary detection
  - Support for functions, classes, variables, and parameters
  - Returns precise location information for each reference

- **Rename Symbol**: Full workspace edit support for renaming symbols
  - Finds all references to a symbol
  - Generates text edits for all occurrences
  - Preserves document structure and formatting

- **Code Actions**: Quick fix and refactoring suggestions
  - "Add missing import" action
  - "Add type annotation" action
  - Extensible architecture for adding more actions

- **Signature Help**: Function signature information while typing
  - Triggered by `(` and `,` characters
  - Shows parameter information and documentation
  - Built-in support for common Python functions (print, len, etc.)

- **Semantic Tokens**: Advanced syntax highlighting
  - Distinguishes between functions, classes, variables, parameters
  - Provides token type information to editors
  - Supports incremental updates

- **Inlay Hints**: Inline type information display
  - Shows inferred types for variables without annotations
  - Configurable display options
  - Helps with code comprehension

#### Analysis & Infrastructure
- **Enhanced Symbol Extraction**: Improved AST traversal
  - Accurate position tracking using byte offsets
  - Support for nested symbols (class methods, nested functions)
  - Parameter extraction from function signatures

- **Improved Position Mapping**: Robust offset-to-line/column conversion
  - Handles multi-byte UTF-8 characters correctly
  - Efficient position calculation
  - Accurate range information

#### Testing
- **Comprehensive Test Suite**: 16 tests covering all major features
  - Unit tests for symbol extraction, completions, hover, references
  - Integration tests for document analysis
  - Position mapping tests
  - All tests passing with 100% success rate

### Changed
- Upgraded `tracing-subscriber` with `env-filter` feature for better logging
- Improved completion trigger logic to handle edge cases
- Enhanced error handling in document analysis

### Fixed
- Fixed AST traversal to work with rustpython-parser 0.3 API
- Corrected byte offset to line/column conversion
- Fixed semantic token encoding to use proper SemanticToken structure
- Resolved parameter extraction from function arguments

## [0.1.0] - Initial Release

### Added
- Basic LSP server implementation
- Document synchronization (open, change, save, close)
- Simple diagnostics (syntax errors)
- Basic hover information
- Simple completions
- Go to definition
- Integration with tower-lsp framework

---

## Future Roadmap

### Planned for 0.3.0
- Full integration with typthon-core type checker
- Cross-file type checking and analysis
- Advanced type inference
- Workspace-wide symbol search
- Performance optimizations

### Planned for 0.4.0
- Code lens (show references count)
- Call hierarchy
- Document symbols (outline view)
- Folding ranges
- Document formatting

