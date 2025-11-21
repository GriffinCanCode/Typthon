//! Compiler Module - Exposes type checking, AST parsing, and analysis
//!
//! This module re-exports the compiler functionality from typthon-core
//! that's needed for the Python package.

use std::path::Path as StdPath;
#[path = "../../typthon-core/lib.rs"]
mod typthon_core;

// Re-export compiler modules
pub use typthon_core::compiler::{
    ast::{AstVisitor, DefaultWalker},
    analysis::{
        TypeChecker, InferenceEngine, BiInfer, ConstraintSolver,
        EffectAnalyzer, RefinementAnalyzer, ProtocolChecker
    },
    types::{Type, TypeContext},
    errors::{TypeError, ErrorKind, SourceLocation, ErrorCollector},
    frontend::{parse_module, Config},
};

// Re-export infrastructure for performance
pub use typthon_core::infrastructure::{
    IncrementalEngine, DependencyGraph, ResultCache,
    ParallelAnalyzer, PerformanceMetrics
};

/// High-level API for type checking a Python file
pub fn check_file<P: AsRef<StdPath>>(path: P) -> Result<Vec<TypeError>, String> {
    let source = std::fs::read_to_string(path.as_ref())
        .map_err(|e| e.to_string())?;

    let ast = parse_module(&source)
        .map_err(|e| e.to_string())?;

    let mut checker = TypeChecker::new();
    Ok(checker.check(&ast))
}

/// High-level API for type inference on source code
pub fn infer_types(source: &str) -> Result<Type, String> {
    let ast = parse_module(source)
        .map_err(|e| e.to_string())?;

    let mut checker = TypeChecker::new();
    Ok(checker.infer(&ast))
}

/// High-level API for effect analysis
pub fn analyze_effects(source: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let ast = parse_module(source)
        .map_err(|e| e.to_string())?;

    let ctx = std::sync::Arc::new(TypeContext::new());
    let mut analyzer = EffectAnalyzer::new(ctx);
    let effects = analyzer.analyze_module(&ast);

    Ok(effects.iter()
        .map(|(k, effect_set)| {
            let effect_str = if effect_set.is_pure() {
                "pure".to_string()
            } else {
                format!("{:?}", effect_set)
            };
            (k.clone(), effect_str)
        })
        .collect())
}

