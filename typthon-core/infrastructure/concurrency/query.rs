//! Query-based incremental computation using Salsa
//!
//! Provides a query system for incremental type checking and analysis.
//! Inspired by rust-analyzer's architecture.

use salsa::{Database, ParallelDatabase};
use std::sync::Arc;
use parking_lot::Mutex;
use std::path::PathBuf;
use crate::compiler::types::Type;
use crate::compiler::errors::TypeError;

/// Module identifier for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u64);

impl ModuleId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn from_path(path: &std::path::Path) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// Query database for type checking
#[salsa::query_group(TypeCheckingDatabaseStorage)]
pub trait TypeCheckingDatabase: salsa::Database {
    /// Get source text for a module
    #[salsa::input]
    fn source_text(&self, module: ModuleId) -> Arc<String>;

    /// Get module path
    #[salsa::input]
    fn module_path(&self, module: ModuleId) -> PathBuf;

    /// Parse module to AST (memoized)
    fn parse_module(&self, module: ModuleId) -> Result<Arc<crate::compiler::ast::Module>, String>;

    /// Type check module (memoized)
    fn check_module(&self, module: ModuleId) -> Arc<Vec<TypeError>>;

    /// Get inferred types for module (memoized)
    fn inferred_types(&self, module: ModuleId) -> Arc<Vec<(String, Type)>>;

    /// Get module dependencies
    fn module_dependencies(&self, module: ModuleId) -> Arc<Vec<ModuleId>>;
}

/// Parse module implementation
fn parse_module(db: &dyn TypeCheckingDatabase, module: ModuleId) -> Result<Arc<crate::compiler::ast::Module>, String> {
    let source = db.source_text(module);
    crate::compiler::frontend::parse_module(&source)
        .map(Arc::new)
        .map_err(|e| format!("{:?}", e))
}

/// Type check module implementation
fn check_module(db: &dyn TypeCheckingDatabase, module: ModuleId) -> Arc<Vec<TypeError>> {
    use crate::compiler::analysis::TypeChecker;
    use crate::compiler::types::TypeContext;

    let ast = match db.parse_module(module) {
        Ok(ast) => ast,
        Err(e) => {
            return Arc::new(vec![TypeError::new(
                crate::compiler::errors::ErrorKind::ParseError(e),
                crate::compiler::errors::SourceLocation::default(),
            )]);
        }
    };

    // Check dependencies first
    let deps = db.module_dependencies(module);
    for &dep in deps.iter() {
        let _dep_errors = db.check_module(dep);
        // Propagate dependency types to context
    }

    let context = Arc::new(TypeContext::new());
    let mut checker = TypeChecker::with_context(context);
    let errors = checker.check(&ast);

    Arc::new(errors)
}

/// Get inferred types implementation
fn inferred_types(db: &dyn TypeCheckingDatabase, module: ModuleId) -> Arc<Vec<(String, Type)>> {
    // Ensure module is checked first
    let _errors = db.check_module(module);

    // Extract inferred types from checking
    // In a full implementation, this would be stored in the context during checking
    Arc::new(vec![])
}

/// Get module dependencies implementation
fn module_dependencies(db: &dyn TypeCheckingDatabase, module: ModuleId) -> Arc<Vec<ModuleId>> {
    let ast = match db.parse_module(module) {
        Ok(ast) => ast,
        Err(_) => return Arc::new(vec![]),
    };

    // Extract imports from AST
    let mut deps = Vec::new();

    // Walk AST to find imports
    use crate::compiler::ast::{AstVisitor, DefaultWalker};
    struct ImportCollector {
        imports: Vec<String>,
    }

    impl AstVisitor for ImportCollector {
        fn visit_import(&mut self, name: &str) {
            self.imports.push(name.to_string());
        }
    }

    let mut collector = ImportCollector { imports: vec![] };
    DefaultWalker::walk_module(&mut collector, &ast);

    // Convert import names to module IDs
    // In a full implementation, this would use a module resolver
    for _import in collector.imports {
        // deps.push(resolve_import(import));
    }

    Arc::new(deps)
}

/// Main database implementation
#[salsa::database(TypeCheckingDatabaseStorage)]
pub struct CompilerDatabase {
    storage: salsa::Storage<Self>,
}

impl CompilerDatabase {
    pub fn new() -> Self {
        Self {
            storage: salsa::Storage::default(),
        }
    }

    /// Set source text for a module
    pub fn set_source_text(&mut self, module: ModuleId, text: Arc<String>) {
        self.set_source_text_impl(module, text);
    }

    /// Set module path
    pub fn set_module_path(&mut self, module: ModuleId, path: PathBuf) {
        self.set_module_path_impl(module, path);
    }

    /// Create snapshot for parallel access
    pub fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::ParallelDatabase::snapshot(self)
    }
}

impl salsa::Database for CompilerDatabase {}

impl salsa::ParallelDatabase for CompilerDatabase {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
        })
    }
}

impl Default for CompilerDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Query coordinator for parallel queries
pub struct QueryCoordinator {
    db: Arc<Mutex<CompilerDatabase>>,
}

impl QueryCoordinator {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(CompilerDatabase::new())),
        }
    }

    /// Get database snapshot for parallel access
    pub fn snapshot(&self) -> salsa::Snapshot<CompilerDatabase> {
        self.db.lock().snapshot()
    }

    /// Update module source
    pub fn update_source(&self, module: ModuleId, text: Arc<String>) {
        self.db.lock().set_source_text(module, text);
    }

    /// Set module path
    pub fn set_path(&self, module: ModuleId, path: PathBuf) {
        self.db.lock().set_module_path(module, path);
    }

    /// Check module (uses memoization)
    pub fn check(&self, module: ModuleId) -> Arc<Vec<TypeError>> {
        let db = self.db.lock();
        db.check_module(module)
    }

    /// Get inferred types (uses memoization)
    pub fn types(&self, module: ModuleId) -> Arc<Vec<(String, Type)>> {
        let db = self.db.lock();
        db.inferred_types(module)
    }

    /// Parallel query execution
    pub async fn check_parallel(&self, modules: Vec<ModuleId>) -> Vec<(ModuleId, Arc<Vec<TypeError>>)> {
        use rayon::prelude::*;

        let snapshot = self.snapshot();

        modules.par_iter()
            .map(|&module| {
                let errors = snapshot.check_module(module);
                (module, errors)
            })
            .collect()
    }
}

impl Default for QueryCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Query caching statistics
#[derive(Debug, Default)]
pub struct QueryStats {
    pub total_queries: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl QueryStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_queries as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_memoization() {
        let mut db = CompilerDatabase::new();
        let module = ModuleId::new(1);

        db.set_source_text(module, Arc::new("x = 1".to_string()));
        db.set_module_path(module, PathBuf::from("test.py"));

        // First check - computes
        let errors1 = db.check_module(module);

        // Second check - should be memoized
        let errors2 = db.check_module(module);

        assert!(Arc::ptr_eq(&errors1, &errors2));
    }

    #[test]
    fn test_query_invalidation() {
        let mut db = CompilerDatabase::new();
        let module = ModuleId::new(1);

        db.set_source_text(module, Arc::new("x = 1".to_string()));
        db.set_module_path(module, PathBuf::from("test.py"));

        let errors1 = db.check_module(module);

        // Update source - invalidates query
        db.set_source_text(module, Arc::new("x = 2".to_string()));

        let errors2 = db.check_module(module);

        // Should not be same Arc (was recomputed)
        assert!(!Arc::ptr_eq(&errors1, &errors2));
    }

    #[tokio::test]
    async fn test_parallel_queries() {
        let coordinator = QueryCoordinator::new();

        let modules: Vec<_> = (0..10).map(|i| {
            let module = ModuleId::new(i);
            coordinator.update_source(module, Arc::new(format!("x{} = {}", i, i)));
            coordinator.set_path(module, PathBuf::from(format!("test{}.py", i)));
            module
        }).collect();

        let results = coordinator.check_parallel(modules).await;
        assert_eq!(results.len(), 10);
    }
}

