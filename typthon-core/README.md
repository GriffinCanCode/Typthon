# Typthon Source Code Architecture

This document describes the modular organization of the Typthon type checker source code.

## Module Structure

The codebase is organized into six main modules, each with a specific responsibility:

### 📦 `core/` - Core Type System

The fundamental type definitions and type context used throughout Typthon.

**Files:**
- `types.rs` - Type definitions, type context, and type operations

**Exports:**
- `Type` - Enum representing all Python types
- `TypeContext` - Context for managing type variables and scopes

**Purpose:** Provides the foundational type system that all other modules build upon.

---

### 🔍 `analysis/` - Type Analysis & Checking

All type analysis components including type checking, inference, and constraint solving.

**Files:**
- `checker.rs` - Main type checker implementation
- `inference.rs` - Type inference engine with unification
- `bidirectional.rs` - Bidirectional type checking (synthesis + checking)
- `constraints.rs` - Constraint solver and generic type parameters
- `variance.rs` - Variance analysis for generic types

**Exports:**
- `TypeChecker` - Main type checker interface
- `InferenceEngine` - Constraint-based type inference
- `BiInfer` - Bidirectional type inference
- `Constraint` - Constraint types for solving
- `ConstraintSolver` - Solves type constraints
- `TypeParameter` - Generic type parameter with bounds
- `GenericType` - Generic type definitions
- `Variance` - Variance annotations (covariant, contravariant, etc.)
- `VarianceAnalyzer` - Analyzes variance relationships

**Purpose:** Implements the core type checking and inference algorithms that power Typthon.

---

### 🌳 `ast/` - AST Traversal Utilities

Visitor pattern and walker utilities for traversing Python abstract syntax trees.

**Files:**
- `visitor.rs` - Visitor trait for extensible AST traversal
- `walker.rs` - Default walker implementation

**Exports:**
- `AstVisitor` - Trait for implementing custom AST visitors
- `DefaultWalker` - Default implementation that traverses entire tree

**Purpose:** Provides reusable patterns for traversing and analyzing Python ASTs.

---

### ⚠️ `errors/` - Error Handling

Comprehensive error types and error collection utilities.

**Files:**
- `mod.rs` - Error types, error collector, and helper functions

**Exports:**
- `TypeError` - Type error with location and suggestions
- `ErrorKind` - Enumeration of all error types
- `SourceLocation` - Source code location information
- `ErrorCollector` - Collects multiple errors during checking
- `levenshtein_distance` - String similarity for suggestions
- `find_similar_names` - "Did you mean?" suggestions

**Purpose:** Provides rich error reporting with helpful suggestions and context.

---

### 🎯 `frontend/` - User Interface

Parser, CLI, and configuration components that form the user-facing interface.

**Files:**
- `parser.rs` - Python source code parsing using RustPython parser
- `cli.rs` - Command-line interface implementation
- `config.rs` - Configuration file parsing and management

**Exports:**
- `parse_module` - Parse Python source code to AST
- `cli_main` - Main CLI entry point
- `Config` - Configuration structure

**Purpose:** Handles all user interaction including CLI, configuration, and parsing.

---

### 🔗 `ffi/` - Foreign Function Interface

C++ FFI bindings for interoperability with native components.

**Files:**
- `cpp_ffi.rs` - C++ type system bindings

**Exports:**
- `TypeSet` - Rust wrapper for C++ type sets

**Purpose:** Enables integration with C++ type systems and native libraries.

---

## Top-Level Files

### `lib.rs`

Main library entry point that:
- Declares all modules
- Re-exports commonly used items
- Provides PyO3 Python bindings

### `bin/typthon.rs`

Command-line binary entry point that invokes the CLI.

---

## Module Dependencies

```
┌─────────────┐
│   lib.rs    │  (Re-exports, PyO3 bindings)
└─────────────┘
      │
      ├─► core/         (No internal dependencies)
      │
      ├─► errors/       (Depends on: core)
      │
      ├─► ast/          (No internal dependencies)
      │
      ├─► analysis/     (Depends on: core, errors)
      │   ├─► checker.rs
      │   ├─► inference.rs
      │   ├─► bidirectional.rs
      │   ├─► constraints.rs
      │   └─► variance.rs
      │
      ├─► frontend/     (Depends on: core, analysis)
      │   ├─► parser.rs
      │   ├─► cli.rs
      │   └─► config.rs
      │
      └─► ffi/          (No internal dependencies)
          └─► cpp_ffi.rs
```

---

## Design Principles

### 1. **Separation of Concerns**
Each module has a single, well-defined responsibility.

### 2. **Minimal Dependencies**
Modules depend only on what they need, keeping the dependency graph clean.

### 3. **Barrel Exports**
Each module has a `mod.rs` that re-exports public items for convenient importing.

### 4. **Core Independence**
The `core` module has no dependencies on other internal modules, making it reusable.

### 5. **Layered Architecture**
- **Layer 1:** `core`, `ast`, `ffi` (foundational)
- **Layer 2:** `errors` (uses core)
- **Layer 3:** `analysis` (uses core + errors)
- **Layer 4:** `frontend` (uses everything)

---

## Adding New Features

When adding new functionality, consider:

1. **Which module does it belong to?**
   - Type system features → `core`
   - Analysis algorithms → `analysis`
   - AST utilities → `ast`
   - Error types → `errors`
   - User-facing features → `frontend`
   - Native integrations → `ffi`

2. **Update the module's `mod.rs`**
   - Add the new submodule declaration
   - Re-export public items

3. **Maintain dependency direction**
   - Lower layers should not depend on higher layers
   - Keep the dependency graph acyclic

4. **Document exports**
   - Add doc comments to public items
   - Update this README if adding new modules

---

## Testing

Each module should have its own tests. Run tests with:

```bash
# All tests
cargo test

# Specific module
cargo test --lib core
cargo test --lib analysis
```

---

## Building

```bash
# Check compilation
cargo check

# Build debug
cargo build

# Build release
cargo build --release

# Build Python wheel
maturin build --release
```

---

## Further Reading

- [ARCHITECTURE.md](../ARCHITECTURE.md) - Overall system architecture
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [ROADMAP.md](../ROADMAP.md) - Future development plans

