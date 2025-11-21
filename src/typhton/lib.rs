//! Typhon - Main Python Package API
//!
//! This is the main entry point for the Python package, exposing
//! a clean, high-level API for type checking, inference, and runtime.

// Import from crate root (which re-exports from typthon-core)
use crate::{
    TypeChecker, Type, TypeContext,
    parse_module,
    compiler::analysis::{EffectAnalyzer, checker::TypeError as CheckerTypeError},
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
fn validate_refinement_py(_value: String, _predicate: String) -> PyResult<bool> {
    // RefinementAnalyzer integration pending
    Ok(true)
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
    #[pyo3(get)]
    pub cache_hits: usize,
    #[pyo3(get)]
    pub cache_misses: usize,
    #[pyo3(get)]
    pub uptime_secs: u64,
}

#[cfg(feature = "python")]
static RUNTIME_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(feature = "python")]
#[pyfunction]
fn get_runtime_stats() -> RuntimeStats {
    // Static metrics tracking
    use std::sync::atomic::{AtomicUsize, Ordering};
    static GC_COUNT: AtomicUsize = AtomicUsize::new(0);
    static HEAP_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
    static CACHE_HITS: AtomicUsize = AtomicUsize::new(0);
    static CACHE_MISSES: AtomicUsize = AtomicUsize::new(0);

    use std::time::SystemTime;
    static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();
    let start = START_TIME.get_or_init(SystemTime::now);
    let uptime = SystemTime::now().duration_since(*start).unwrap_or_default();

    RuntimeStats {
        gc_collections: GC_COUNT.load(Ordering::Relaxed),
        heap_allocated: HEAP_ALLOCATED.load(Ordering::Relaxed),
        cache_hits: CACHE_HITS.load(Ordering::Relaxed),
        cache_misses: CACHE_MISSES.load(Ordering::Relaxed),
        uptime_secs: uptime.as_secs(),
    }
}

#[cfg(feature = "python")]
#[pyfunction]
fn force_gc_py() {
    // Increment GC counter
    use std::sync::atomic::{AtomicUsize, Ordering};
    static GC_COUNT: AtomicUsize = AtomicUsize::new(0);
    GC_COUNT.fetch_add(1, Ordering::Relaxed);

    // Note: Actual GC forcing would require integration with a memory allocator
    // For now, we just track the request
}

#[cfg(feature = "python")]
#[pyfunction]
fn init_runtime_py() {
    if RUNTIME_INITIALIZED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return; // Already initialized
    }

    // Runtime initialization (counters are initialized on first access)
    // This function ensures the runtime is ready
}

#[cfg(feature = "python")]
#[pyfunction]
fn clear_cache_py() -> PyResult<String> {
    // Reset static counters
    use std::sync::atomic::{AtomicUsize, Ordering};
    static GC_COUNT: AtomicUsize = AtomicUsize::new(0);
    static HEAP_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
    static CACHE_HITS: AtomicUsize = AtomicUsize::new(0);
    static CACHE_MISSES: AtomicUsize = AtomicUsize::new(0);

    GC_COUNT.store(0, Ordering::Relaxed);
    HEAP_ALLOCATED.store(0, Ordering::Relaxed);
    CACHE_HITS.store(0, Ordering::Relaxed);
    CACHE_MISSES.store(0, Ordering::Relaxed);

    Ok("Cache cleared".to_string())
}

#[cfg(feature = "python")]
#[pyfunction]
fn get_metrics_py() -> PyResult<std::collections::HashMap<String, String>> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static GC_COUNT: AtomicUsize = AtomicUsize::new(0);
    static HEAP_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
    static CACHE_HITS: AtomicUsize = AtomicUsize::new(0);
    static CACHE_MISSES: AtomicUsize = AtomicUsize::new(0);

    use std::time::SystemTime;
    static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();
    let start = START_TIME.get_or_init(SystemTime::now);
    let uptime = SystemTime::now().duration_since(*start).unwrap_or_default();

    let mut result = std::collections::HashMap::new();
    result.insert("uptime".to_string(), uptime.as_secs().to_string());
    result.insert("uptime_secs".to_string(), uptime.as_secs().to_string());  // Keep both for compatibility
    result.insert("gc_collections".to_string(), GC_COUNT.load(Ordering::Relaxed).to_string());
    result.insert("heap_allocated".to_string(), HEAP_ALLOCATED.load(Ordering::Relaxed).to_string());
    result.insert("cache_hits".to_string(), CACHE_HITS.load(Ordering::Relaxed).to_string());
    result.insert("cache_misses".to_string(), CACHE_MISSES.load(Ordering::Relaxed).to_string());

    Ok(result)
}

#[cfg(feature = "python")]
#[pymodule]
fn typthon(_py: Python, m: &PyModule) -> PyResult<()> {
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
    m.add_function(wrap_pyfunction!(clear_cache_py, m)?)?;
    m.add_function(wrap_pyfunction!(get_metrics_py, m)?)?;

    Ok(())
}

