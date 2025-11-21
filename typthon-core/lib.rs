//! Typthon Core - A high-performance gradual type system for Python
//!
//! This is the core library organized semantically by functionality rather than implementation language.

// Compiler modules
pub mod compiler {
    pub mod frontend;
    pub mod ast;
    pub mod analysis;
    pub mod types;
    pub mod errors;
}

// Runtime support
pub mod runtime {
    // Runtime components are language-specific but organized by target
}

// FFI and bindings layer
pub mod bindings;

// Infrastructure (performance, caching, etc.)
pub mod infrastructure;

// CLI (standalone binary, not a module)

// Re-export commonly used items for convenience
pub use compiler::{
    types::{Type, TypeContext},
    analysis::{TypeChecker, InferenceEngine, BiInfer, ConstraintSolver},
    ast::{AstVisitor, DefaultWalker},
    errors::{TypeError, ErrorKind, SourceLocation, ErrorCollector},
    frontend::{parse_module, Config},
};

pub use infrastructure::{
    IncrementalEngine, DependencyGraph, ResultCache, ParallelAnalyzer,
    PerformanceMetrics, LogConfig, LogFormat, LogOutput, init_logging,
    init_dev_logging, init_prod_logging
};

