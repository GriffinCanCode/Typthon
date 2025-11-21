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
}

#[pymodule]
fn _core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_file, m)?)?;
    m.add_function(wrap_pyfunction!(infer_types, m)?)?;
    m.add_class::<TypeValidator>()?;
    Ok(())
}

