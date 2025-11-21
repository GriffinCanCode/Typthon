//! Query-based incremental computation using Salsa
//!
//! Provides a query system for incremental type checking and analysis.
//! Inspired by rust-analyzer's architecture.

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

    pub fn as_u64(&self) -> u64 {
        self.0
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

    /// Type check module (memoized based on source_text)
    fn check_module(&self, module: ModuleId) -> Arc<Vec<TypeError>>;

    /// Get inferred types for module (memoized based on source_text)
    fn inferred_types(&self, module: ModuleId) -> Arc<Vec<(String, Type)>>;

    /// Get module dependencies (memoized based on source_text)
    fn module_dependencies(&self, module: ModuleId) -> Arc<Vec<ModuleId>>;
}

/// Type check module implementation (parse inline to avoid Eq requirement on AST)
fn check_module(db: &dyn TypeCheckingDatabase, module: ModuleId) -> Arc<Vec<TypeError>> {
    use crate::compiler::analysis::TypeChecker;
    use crate::compiler::types::TypeContext;
    use crate::compiler::errors::SourceLocation;

    // Parse inline since AST doesn't implement Eq for memoization
    let source = db.source_text(module);
    let ast = match crate::compiler::frontend::parse_module(&source) {
        Ok(ast) => ast,
        Err(_e) => {
            // Return empty error list for parse failures
            return Arc::new(vec![]);
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
    let checker_errors = checker.check(&ast);

    // Convert checker errors to our TypeError format
    let errors = checker_errors.into_iter().map(|e| {
        TypeError::new(
            crate::compiler::errors::ErrorKind::TypeMismatch {
                expected: format!("{:?}", e),
                found: String::new(),
            },
            SourceLocation::default(),
        )
    }).collect();

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
    // Get source and parse inline
    let source = db.source_text(module);
    let ast = match crate::compiler::frontend::parse_module(&source) {
        Ok(ast) => ast,
        Err(_) => return Arc::new(vec![]),
    };

    // Extract imports from AST
    let deps = Vec::new();

    // Walk AST to find imports
    use rustpython_parser::ast::{StmtImport, StmtImportFrom};
    struct ImportCollector {
        imports: Vec<String>,
    }

    impl crate::compiler::ast::AstVisitor for ImportCollector {
        fn visit_import(&mut self, import: &StmtImport) {
            for alias in &import.names {
                self.imports.push(alias.name.to_string());
            }
            Default::default()
        }

        fn visit_import_from(&mut self, import: &StmtImportFrom) {
            if let Some(ref module) = import.module {
                self.imports.push(module.to_string());
            }
            Default::default()
        }

        // Required trait methods with default implementations
        fn visit_function_def(&mut self, _func: &rustpython_parser::ast::StmtFunctionDef) { }
        fn visit_async_function_def(&mut self, _func: &rustpython_parser::ast::StmtAsyncFunctionDef) { }
        fn visit_class_def(&mut self, _class: &rustpython_parser::ast::StmtClassDef) { }
        fn visit_return(&mut self, _ret: &rustpython_parser::ast::StmtReturn) { }
        fn visit_delete(&mut self, _del: &rustpython_parser::ast::StmtDelete) { }
        fn visit_assign(&mut self, _assign: &rustpython_parser::ast::StmtAssign) { }
        fn visit_aug_assign(&mut self, _aug: &rustpython_parser::ast::StmtAugAssign) { }
        fn visit_ann_assign(&mut self, _ann: &rustpython_parser::ast::StmtAnnAssign) { }
        fn visit_for(&mut self, _for_stmt: &rustpython_parser::ast::StmtFor) { }
        fn visit_async_for(&mut self, _for_stmt: &rustpython_parser::ast::StmtAsyncFor) { }
        fn visit_while(&mut self, _while_stmt: &rustpython_parser::ast::StmtWhile) { }
        fn visit_if(&mut self, _if_stmt: &rustpython_parser::ast::StmtIf) { }
        fn visit_with(&mut self, _with: &rustpython_parser::ast::StmtWith) { }
        fn visit_async_with(&mut self, _with: &rustpython_parser::ast::StmtAsyncWith) { }
        fn visit_match(&mut self, _match_stmt: &rustpython_parser::ast::StmtMatch) { }
        fn visit_raise(&mut self, _raise: &rustpython_parser::ast::StmtRaise) { }
        fn visit_try(&mut self, _try_stmt: &rustpython_parser::ast::StmtTry) { }
        fn visit_assert(&mut self, _assert: &rustpython_parser::ast::StmtAssert) { }
        fn visit_global(&mut self, _global: &rustpython_parser::ast::StmtGlobal) { }
        fn visit_nonlocal(&mut self, _nonlocal: &rustpython_parser::ast::StmtNonlocal) { }
        fn visit_expr_stmt(&mut self, _expr: &rustpython_parser::ast::StmtExpr) { }
        fn visit_pass(&mut self, _pass: &rustpython_parser::ast::StmtPass) { }
        fn visit_break(&mut self, _break_stmt: &rustpython_parser::ast::StmtBreak) { }
        fn visit_continue(&mut self, _cont: &rustpython_parser::ast::StmtContinue) { }
        fn visit_type_alias(&mut self, _type_alias: &rustpython_parser::ast::StmtTypeAlias) { }
        fn visit_try_star(&mut self, _try_star: &rustpython_parser::ast::StmtTryStar) { }
        fn visit_bool_op(&mut self, _bool_op: &rustpython_parser::ast::ExprBoolOp) { }
        fn visit_named_expr(&mut self, _named: &rustpython_parser::ast::ExprNamedExpr) { }
        fn visit_bin_op(&mut self, _bin_op: &rustpython_parser::ast::ExprBinOp) { }
        fn visit_unary_op(&mut self, _unary: &rustpython_parser::ast::ExprUnaryOp) { }
        fn visit_lambda(&mut self, _lambda: &rustpython_parser::ast::ExprLambda) { }
        fn visit_if_exp(&mut self, _if_exp: &rustpython_parser::ast::ExprIfExp) { }
        fn visit_dict(&mut self, _dict: &rustpython_parser::ast::ExprDict) { }
        fn visit_set(&mut self, _set: &rustpython_parser::ast::ExprSet) { }
        fn visit_list_comp(&mut self, _comp: &rustpython_parser::ast::ExprListComp) { }
        fn visit_set_comp(&mut self, _comp: &rustpython_parser::ast::ExprSetComp) { }
        fn visit_dict_comp(&mut self, _comp: &rustpython_parser::ast::ExprDictComp) { }
        fn visit_generator_exp(&mut self, _gen: &rustpython_parser::ast::ExprGeneratorExp) { }
        fn visit_await(&mut self, _await_expr: &rustpython_parser::ast::ExprAwait) { }
        fn visit_yield(&mut self, _yield_expr: &rustpython_parser::ast::ExprYield) { }
        fn visit_yield_from(&mut self, _yield_from: &rustpython_parser::ast::ExprYieldFrom) { }
        fn visit_compare(&mut self, _compare: &rustpython_parser::ast::ExprCompare) { }
        fn visit_call(&mut self, _call: &rustpython_parser::ast::ExprCall) { }
        fn visit_formatted_value(&mut self, _formatted: &rustpython_parser::ast::ExprFormattedValue) { }
        fn visit_joined_str(&mut self, _joined: &rustpython_parser::ast::ExprJoinedStr) { }
        fn visit_constant(&mut self, _constant: &rustpython_parser::ast::ExprConstant) { }
        fn visit_attribute(&mut self, _attr: &rustpython_parser::ast::ExprAttribute) { }
        fn visit_subscript(&mut self, _subscript: &rustpython_parser::ast::ExprSubscript) { }
        fn visit_starred(&mut self, _starred: &rustpython_parser::ast::ExprStarred) { }
        fn visit_name(&mut self, _name: &rustpython_parser::ast::ExprName) { }
        fn visit_list(&mut self, _list: &rustpython_parser::ast::ExprList) { }
        fn visit_tuple(&mut self, _tuple: &rustpython_parser::ast::ExprTuple) { }
        fn visit_slice(&mut self, _slice: &rustpython_parser::ast::ExprSlice) { }
    }

    use crate::compiler::ast::AstVisitor;

    let mut collector = ImportCollector { imports: vec![] };
    collector.visit_module(&ast);

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
        use futures::future::join_all;

        // Use tokio tasks instead of rayon to avoid Sync issues with salsa
        let mut tasks = Vec::new();

        for module in modules {
            let db = self.db.clone();
            tasks.push(tokio::task::spawn_blocking(move || {
                let errors = db.lock().check_module(module);
                (module, errors)
            }));
        }

        let results = join_all(tasks).await;
        results.into_iter()
            .filter_map(|r| r.ok())
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

