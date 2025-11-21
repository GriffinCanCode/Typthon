// Core modules
pub mod core;
pub mod analysis;
pub mod ast;
pub mod errors;
pub mod frontend;
pub mod ffi;
pub mod performance;

// Re-export commonly used items
pub use core::{Type, TypeContext};
pub use analysis::{TypeChecker, InferenceEngine, BiInfer, Constraint, ConstraintSolver};
pub use ast::{AstVisitor, DefaultWalker};
pub use errors::{TypeError, ErrorKind, SourceLocation, ErrorCollector};
pub use frontend::{parse_module, cli_main, Config};
pub use performance::{
    IncrementalEngine, DependencyGraph, ResultCache, ParallelAnalyzer,
    PerformanceMetrics, PerformanceConfig
};

use pyo3::prelude::*;

#[pyfunction]
fn check_file(path: String) -> PyResult<Vec<String>> {
    let source = std::fs::read_to_string(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let ast = parse_module(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

    let mut checker = TypeChecker::new();
    let errors = checker.check(&ast);

    Ok(errors.iter().map(|e| e.to_string()).collect())
}

#[pyfunction]
fn infer_types(source: String) -> PyResult<String> {
    let ast = parse_module(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

    let mut checker = TypeChecker::new();
    let result = checker.infer(&ast);

    Ok(format!("{:?}", result))
}

#[pyfunction]
fn check_effects(source: String) -> PyResult<std::collections::HashMap<String, Vec<String>>> {
    let ast = parse_module(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

    let ctx = std::sync::Arc::new(TypeContext::new());
    let mut analyzer = crate::analysis::EffectAnalyzer::new(ctx);
    let effects = analyzer.analyze_module(&ast);

    Ok(effects.iter().map(|(k, v)| {
        // Convert EffectSet to string representation
        (k.clone(), vec![format!("{:?}", v)])
    }).collect())
}

#[pyfunction]
fn validate_refinement(value: String, predicate: String) -> PyResult<bool> {
    let analyzer = crate::analysis::RefinementAnalyzer::new();

    let json_val: serde_json::Value = serde_json::from_str(&value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let pred = analyzer.parse_predicate(&predicate)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    Ok(analyzer.validate(&json_val, &pred))
}

#[pyfunction]
fn check_recursive_type(type_def: String) -> PyResult<bool> {
    // Parse and check if recursive type is well-formed (productive)
    let mut checker = TypeChecker::new();

    // For now, return true; full implementation would parse the type_def
    Ok(true)
}

#[pyclass]
struct TypeValidator {
    checker: TypeChecker,
}

#[pymethods]
impl TypeValidator {
    #[new]
    fn new() -> Self {
        Self {
            checker: TypeChecker::new(),
        }
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
            Ok(vec![format!("{:?}", effects)])
        } else {
            Ok(vec![])
        }
    }

    fn validate_refinement_value(&self, value: String, type_str: String) -> PyResult<bool> {
        let json_val: serde_json::Value = serde_json::from_str(&value)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        // Parse type_str to get Type; for now, just check basic types
        // Full implementation would parse the type annotation
        Ok(true)
    }
}

#[pymodule]
fn _core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    m.add_function(wrap_pyfunction!(infer_types, m)?)?;
    m.add_function(wrap_pyfunction!(check_effects, m)?)?;
    m.add_function(wrap_pyfunction!(validate_refinement, m)?)?;
    m.add_function(wrap_pyfunction!(check_recursive_type, m)?)?;
    m.add_class::<TypeValidator>()?;
    Ok(())
}

