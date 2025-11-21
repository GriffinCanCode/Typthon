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

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
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

#[cfg(feature = "python")]
#[pyfunction]
fn infer_types(source: String) -> PyResult<String> {
    let ast = parse_module(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

    let mut checker = TypeChecker::new();
    let result = checker.infer(&ast);

    Ok(format!("{:?}", result))
}

#[cfg(feature = "python")]
#[pyfunction]
fn check_effects(source: String) -> PyResult<std::collections::HashMap<String, Vec<String>>> {
    let ast = parse_module(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

    let ctx = std::sync::Arc::new(TypeContext::new());
    let mut analyzer = crate::compiler::analysis::EffectAnalyzer::new(ctx);
    let effects = analyzer.analyze_module(&ast);

    // Convert EffectSet to detailed effect strings
    Ok(effects.iter().map(|(k, effect_set)| {
        let effect_strings: Vec<String> = if effect_set.is_pure() {
            vec!["pure".to_string()]
        } else {
            // Extract individual effects from EffectSet
            let effects_str = format!("{:?}", effect_set);
            vec![effects_str]
        };
        (k.clone(), effect_strings)
    }).collect())
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_function_type_with_effects(source: String, func_name: String) -> PyResult<String> {
    let ast = parse_module(&source)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(e.to_string()))?;

    let mut checker = TypeChecker::new();
    checker.check(&ast);

    if let Some(ty) = checker.get_type(&func_name) {
        Ok(format!("{}", ty))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Function '{}' not found", func_name)
        ))
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn validate_refinement(value: String, predicate: String) -> PyResult<bool> {
    let analyzer = crate::compiler::analysis::RefinementAnalyzer::new();

    let json_val: serde_json::Value = serde_json::from_str(&value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let pred = analyzer.parse_predicate(&predicate)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    Ok(analyzer.validate(&json_val, &pred))
}

#[cfg(feature = "python")]
#[pyfunction]
fn check_recursive_type(_type_def: String) -> PyResult<bool> {
    // Parse and check if recursive type is well-formed (productive)
    // For now, return true; full implementation would parse the type_def
    Ok(true)
}

#[cfg(feature = "python")]
#[pyclass]
struct TypeValidator {
    checker: TypeChecker,
}

#[cfg(feature = "python")]
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
            if effects.is_pure() {
                Ok(vec!["pure".to_string()])
            } else {
                // Parse the debug format to extract individual effects
                let effects_str = format!("{:?}", effects);
                Ok(vec![effects_str])
            }
        } else {
            Ok(vec!["pure".to_string()])
        }
    }

    fn get_function_type(&self, name: String) -> PyResult<String> {
        if let Some(ty) = self.checker.get_type(&name) {
            Ok(format!("{}", ty))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Function '{}' not found", name)
            ))
        }
    }

    fn validate_refinement_value(&self, _value: String, _type_str: String) -> PyResult<bool> {
        // Parse type_str to get Type; for now, just check basic types
        // Full implementation would parse the type annotation
        Ok(true)
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn _core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    m.add_function(wrap_pyfunction!(infer_types, m)?)?;
    m.add_function(wrap_pyfunction!(check_effects, m)?)?;
    m.add_function(wrap_pyfunction!(get_function_type_with_effects, m)?)?;
    m.add_function(wrap_pyfunction!(validate_refinement, m)?)?;
    m.add_function(wrap_pyfunction!(check_recursive_type, m)?)?;
    m.add_class::<TypeValidator>()?;
    Ok(())
}

