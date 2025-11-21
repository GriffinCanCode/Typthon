//! Typhon - Main Python Package API
//!
//! This is the main entry point for the Python package, exposing
//! a clean, high-level API for type checking, inference, and runtime.

// Import from crate root (which re-exports from typthon-core)
use crate::{
    TypeChecker, Type, TypeContext,
    parse_module,
    compiler::analysis::{EffectAnalyzer, RefinementAnalyzer, checker::TypeError as CheckerTypeError},
};

use std::path::Path as StdPath;

/// High-level API for type checking a Python file
pub fn check_file<P: AsRef<StdPath>>(path: P) -> Result<Vec<CheckerTypeError>, String> {
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

// Python bindings
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pyfunction]
fn check_file_py(path: String) -> PyResult<Vec<String>> {
    check_file(&path)
        .map(|errors| errors.iter().map(|e| e.to_string()).collect())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e))
}

#[cfg(feature = "python")]
#[pyfunction]
fn infer_types_py(source: String) -> PyResult<String> {
    infer_types(&source)
        .map(|ty| format!("{}", ty))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e))
}

#[cfg(feature = "python")]
#[pyfunction]
fn analyze_effects_py(source: String) -> PyResult<std::collections::HashMap<String, String>> {
    analyze_effects(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e))
}

#[cfg(feature = "python")]
#[pyfunction]
fn validate_refinement_py(value: String, predicate: String) -> PyResult<bool> {
    let analyzer = RefinementAnalyzer::new();

    let json_val: serde_json::Value = serde_json::from_str(&value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let pred = analyzer.parse_predicate(&predicate)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    Ok(analyzer.validate(&json_val, &pred))
}

#[cfg(feature = "python")]
#[pyclass]
pub struct TypeValidator {
    checker: TypeChecker,
}

#[cfg(feature = "python")]
#[pymethods]
impl TypeValidator {
    #[new]
    fn new() -> Self {
        Self { checker: TypeChecker::new() }
    }

    fn validate(&mut self, source: String) -> PyResult<bool> {
        let ast = parse_module(&source)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

        let errors = self.checker.check(&ast);
        Ok(errors.is_empty())
    }

    fn get_type(&mut self, expr: String) -> PyResult<String> {
        let ast = parse_module(&expr)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

        let ty = self.checker.infer(&ast);
        Ok(format!("{:?}", ty))
    }

    fn get_function_effects(&self, name: String) -> PyResult<Vec<String>> {
        if let Some(effects) = self.checker.get_function_effects(&name) {
            if effects.is_pure() {
                Ok(vec!["pure".to_string()])
            } else {
                Ok(vec![format!("{:?}", effects)])
            }
        } else {
            Ok(vec!["pure".to_string()])
        }
    }

    fn get_function_type(&self, name: String) -> PyResult<String> {
        self.checker.get_type(&name)
            .map(|ty| format!("{}", ty))
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Function '{}' not found", name)
            ))
    }
}

#[cfg(feature = "python")]
#[pyclass]
pub struct RuntimeStats {
    #[pyo3(get)]
    pub gc_collections: usize,
    #[pyo3(get)]
    pub heap_allocated: usize,
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_runtime_stats() -> RuntimeStats {
    // Runtime stats will be implemented later
    RuntimeStats {
        gc_collections: 0,
        heap_allocated: 0,
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn force_gc_py() {
    // GC forcing will be implemented later
}

#[cfg(feature = "python")]
#[pyfunction]
fn init_runtime_py() {
    // Runtime init will be implemented later
}

#[cfg(feature = "python")]
#[pymodule]
fn typhon(_py: Python, m: &PyModule) -> PyResult<()> {
    // Type checking and inference
    m.add_function(wrap_pyfunction!(check_file_py, m)?)?;
    m.add_function(wrap_pyfunction!(infer_types_py, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_effects_py, m)?)?;
    m.add_function(wrap_pyfunction!(validate_refinement_py, m)?)?;
    m.add_class::<TypeValidator>()?;

    // Runtime management
    m.add_function(wrap_pyfunction!(init_runtime_py, m)?)?;
    m.add_function(wrap_pyfunction!(get_runtime_stats, m)?)?;
    m.add_function(wrap_pyfunction!(force_gc_py, m)?)?;

    Ok(())
}

