//! Parallel file analysis with integrated concurrency patterns
//!
//! This module orchestrates multiple concurrency models for optimal compiler performance:
//! - Rayon for data parallelism (analyzing independent modules)
//! - Actors for coordinating async I/O and incremental updates
//! - Query system for memoized, incremental type checking
//! - Structured concurrency for proper resource management

use crate::compiler::analysis::TypeChecker;
use crate::compiler::types::TypeContext;
use crate::compiler::errors::TypeError;
use crate::compiler::frontend::parse_module;
use crate::infrastructure::incremental::{IncrementalEngine, ModuleId};
use crate::infrastructure::cache::{ResultCache, CacheKey, CacheEntry, CachedError};
use crate::infrastructure::concurrency::{
    ActorSystem, QueryCoordinator, BatchFileReader, CompilerPipeline, CompilerStage,
};
use dashmap::DashMap;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Analysis task for a single module
#[derive(Debug, Clone)]
pub struct AnalysisTask {
    pub id: ModuleId,
    pub path: PathBuf,
    pub content: String,
}

/// Result of analyzing a module
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub id: ModuleId,
    pub errors: Vec<TypeError>,
    pub duration_ms: u64,
}

/// Parallel analyzer
pub struct ParallelAnalyzer {
    /// Type context (shared across threads)
    context: Arc<TypeContext>,
    /// Result cache
    cache: Arc<ResultCache>,
    /// Incremental engine
    incremental: Arc<IncrementalEngine>,
    /// Query coordinator for incremental computation
    query_coordinator: Arc<QueryCoordinator>,
    /// Async file reader
    file_reader: Arc<BatchFileReader>,
    /// Compilation pipeline configuration
    pipeline: CompilerPipeline,
    /// Number of worker threads
    workers: usize,
    /// Analysis results
    results: DashMap<ModuleId, AnalysisResult>,
}

impl ParallelAnalyzer {
    pub fn new(
        context: Arc<TypeContext>,
        cache: Arc<ResultCache>,
        incremental: Arc<IncrementalEngine>,
        workers: usize,
    ) -> Self {
        // Configure rayon thread pool
        if workers > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build_global()
                .ok();
        }

        Self {
            context,
            cache,
            incremental,
            query_coordinator: Arc::new(QueryCoordinator::new()),
            file_reader: Arc::new(BatchFileReader::new(1000, workers)),
            pipeline: CompilerPipeline::check_only(),
            workers: if workers == 0 { num_cpus::get() } else { workers },
            results: DashMap::new(),
        }
    }

    /// Create analyzer with custom pipeline
    pub fn with_pipeline(mut self, pipeline: CompilerPipeline) -> Self {
        self.pipeline = pipeline;
        self
    }

    /// Analyze modules using query-based incremental computation
    pub async fn analyze_incremental(&self, modules: Vec<AnalysisTask>) -> Vec<AnalysisResult> {
        // Update query database with new sources
        for task in &modules {
            self.query_coordinator.update_source(
                crate::infrastructure::concurrency::QueryModuleId::new(task.id.0),
                Arc::new(task.content.clone())
            );
            self.query_coordinator.set_path(
                crate::infrastructure::concurrency::QueryModuleId::new(task.id.0),
                task.path.clone()
            );
        }

        // Use query system for parallel incremental checking
        let query_modules: Vec<_> = modules.iter()
            .map(|t| crate::infrastructure::concurrency::QueryModuleId::new(t.id.0))
            .collect();

        let query_results = self.query_coordinator.check_parallel(query_modules).await;

        // Convert query results to analysis results
        query_results.into_iter().map(|(qid, errors)| {
            AnalysisResult {
                id: ModuleId::new(qid.0),
                errors: (*errors).clone(),
                duration_ms: 0,
            }
        }).collect()
    }

    /// Analyze modules in parallel with dependency ordering
    pub fn analyze_modules(&self, modules: Vec<AnalysisTask>) -> Vec<AnalysisResult> {
        self.results.clear();

        // Get dependency layers for ordered parallelism
        let layers = self.incremental.get_layers();
        let task_map: DashMap<ModuleId, AnalysisTask> = modules.iter()
            .map(|task| (task.id, task.clone()))
            .collect();

        if layers.is_empty() && !modules.is_empty() {
            // No dependencies - full parallelism
            let layer_results: Vec<_> = modules
                .par_iter()
                .map(|task| self.analyze_task(task))
                .collect();

            for result in layer_results {
                self.results.insert(result.id, result);
            }
        } else {
            // Process dependency layers in parallel
            for layer in layers {
                let tasks_in_layer: Vec<_> = layer.iter()
                    .filter_map(|id| task_map.get(id).map(|t| t.clone()))
                    .collect();

                let layer_results: Vec<_> = tasks_in_layer
                    .par_iter()
                    .map(|task| self.analyze_task(task))
                    .collect();

                for result in layer_results {
                    self.results.insert(result.id, result);
                }
            }
        }

        self.results.iter().map(|e| e.value().clone()).collect()
    }

    /// Analyze a single task with caching
    fn analyze_task(&self, task: &AnalysisTask) -> AnalysisResult {
        let start = Instant::now();

        // Check cache
        let cache_key = CacheKey {
            module: task.id,
            hash: crate::infrastructure::incremental::ContentHash::from_str(&task.content),
        };

        if let Some(cached) = self.cache.get(&cache_key) {
            let errors = cached.errors.iter()
                .map(|e| self.cached_error_to_type_error(e))
                .collect();

            return AnalysisResult {
                id: task.id,
                errors,
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Cache miss - perform analysis
        let (errors, inferred_types) = match parse_module(&task.content) {
            Ok(ast) => {
                let mut checker = TypeChecker::with_context(self.context.clone());
                let check_errors = checker.check(&ast);
                let types = self.extract_types_from_context(&task.id);
                (check_errors, types)
            }
            Err(e) => {
                (vec![crate::compiler::analysis::checker::TypeError {
                    message: format!("parse error: {}", e),
                    line: 0,
                    col: 0,
                }], vec![])
            }
        };

        let duration = start.elapsed().as_millis() as u64;

        // Cache result
        let cached_errors: Vec<CachedError> = errors.iter().map(|e| CachedError {
            message: e.message.clone(),
            line: e.line,
            col: e.col,
            file: task.path.to_string_lossy().to_string(),
        }).collect();

        let cache_entry = CacheEntry {
            module: task.id,
            hash: cache_key.hash,
            types: inferred_types,
            errors: cached_errors,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size_bytes: task.content.len(),
        };

        let _ = self.cache.set(cache_key, cache_entry);

        // Convert to result
        let result_errors: Vec<crate::compiler::errors::TypeError> = errors.iter().map(|e| {
            crate::compiler::errors::TypeError::new(
                crate::compiler::errors::ErrorKind::TypeMismatch {
                    expected: "".to_string(),
                    found: e.message.clone(),
                },
                crate::compiler::errors::SourceLocation::new(e.line, e.col, e.line, e.col),
            )
        }).collect();

        AnalysisResult {
            id: task.id,
            errors: result_errors,
            duration_ms: duration,
        }
    }

    fn cached_error_to_type_error(&self, cached: &CachedError) -> TypeError {
        TypeError::new(
            crate::compiler::errors::ErrorKind::TypeMismatch {
                expected: "".to_string(),
                found: cached.message.clone(),
            },
            crate::compiler::errors::SourceLocation::new(cached.line, cached.col, cached.line, cached.col),
        ).with_file(cached.file.clone())
    }

    /// Analyze project directory with async I/O
    pub async fn analyze_project_async(&self, root: &Path) -> Vec<AnalysisResult> {
        // Use async file reader for efficient I/O
        let files = match self.file_reader.read_directory(root).await {
            Ok(files) => files,
            Err(_) => return vec![],
        };

        let tasks: Vec<_> = files.into_iter()
            .map(|(path, content)| AnalysisTask {
                id: ModuleId::from_path(&path),
                path,
                content: (*content).clone(),
            })
            .collect();

        // Use incremental query system
        self.analyze_incremental(tasks).await
    }

    /// Analyze project directory (sync version)
    pub fn analyze_project(&self, root: &Path) -> Vec<AnalysisResult> {
        let tasks = self.find_python_files(root);
        self.analyze_modules(tasks)
    }

    fn find_python_files(&self, root: &Path) -> Vec<AnalysisTask> {
        use std::fs;

        let mut tasks = Vec::new();

        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    tasks.extend(self.find_python_files(&path));
                } else if path.extension() == Some(std::ffi::OsStr::new("py")) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let id = ModuleId::from_path(&path);
                        tasks.push(AnalysisTask {
                            id,
                            path,
                            content,
                        });
                    }
                }
            }
        }

        tasks
    }

    pub fn get_result(&self, id: ModuleId) -> Option<AnalysisResult> {
        self.results.get(&id).map(|r| r.clone())
    }

    pub fn get_all_results(&self) -> Vec<AnalysisResult> {
        self.results.iter().map(|e| e.value().clone()).collect()
    }

    pub fn worker_count(&self) -> usize {
        self.workers
    }

    pub fn pipeline(&self) -> &CompilerPipeline {
        &self.pipeline
    }

    pub fn query_coordinator(&self) -> &Arc<QueryCoordinator> {
        &self.query_coordinator
    }

    fn extract_types_from_context(&self, _module_id: &ModuleId) -> Vec<(String, crate::compiler::types::Type)> {
        vec![]
    }
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::DependencyGraph;
    use tempfile::TempDir;

    #[test]
    fn test_parallel_analysis() {
        let context = Arc::new(TypeContext::new());
        let temp = TempDir::new().unwrap();
        let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());
        let graph = Arc::new(DependencyGraph::new());
        let incremental = Arc::new(IncrementalEngine::new(graph));

        let analyzer = ParallelAnalyzer::new(context, cache, incremental, 2);

        let tasks = vec![
            AnalysisTask {
                id: ModuleId::new(1),
                path: PathBuf::from("test1.py"),
                content: "x = 1 + 2".to_string(),
            },
            AnalysisTask {
                id: ModuleId::new(2),
                path: PathBuf::from("test2.py"),
                content: "y = \"hello\"".to_string(),
            },
        ];

        let results = analyzer.analyze_modules(tasks);
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_incremental_analysis() {
        let context = Arc::new(TypeContext::new());
        let temp = TempDir::new().unwrap();
        let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());
        let graph = Arc::new(DependencyGraph::new());
        let incremental = Arc::new(IncrementalEngine::new(graph));

        let analyzer = ParallelAnalyzer::new(context, cache, incremental, 2);

        let tasks = vec![
            AnalysisTask {
                id: ModuleId::new(1),
                path: PathBuf::from("test1.py"),
                content: "x = 1".to_string(),
            },
        ];

        let results = analyzer.analyze_incremental(tasks).await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_async_project_analysis() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.py"), "x = 1").unwrap();

        let context = Arc::new(TypeContext::new());
        let cache_dir = temp.path().join("cache");
        std::fs::create_dir(&cache_dir).unwrap();
        let cache = Arc::new(ResultCache::new(cache_dir, 100).unwrap());
        let graph = Arc::new(DependencyGraph::new());
        let incremental = Arc::new(IncrementalEngine::new(graph));

        let analyzer = ParallelAnalyzer::new(context, cache, incremental, 2);
        let results = analyzer.analyze_project_async(temp.path()).await;

        assert!(!results.is_empty());
    }
}
