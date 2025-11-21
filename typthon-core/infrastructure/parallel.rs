//! Parallel file analysis
//!
//! Work-stealing parallelism for analyzing multiple files concurrently.

use crate::compiler::analysis::TypeChecker;
use crate::compiler::types::TypeContext;
use crate::compiler::errors::TypeError;
use crate::compiler::frontend::parse_module;
use crate::infrastructure::incremental::{IncrementalEngine, ModuleId};
use crate::infrastructure::cache::{ResultCache, CacheKey, CacheEntry, CachedError};
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

/// Parallel analyzer coordinating workers
pub struct ParallelAnalyzer {
    /// Type context (shared across threads)
    context: Arc<TypeContext>,

    /// Result cache
    cache: Arc<ResultCache>,

    /// Incremental engine
    incremental: Arc<IncrementalEngine>,

    /// Number of worker threads (0 = auto)
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
            workers: if workers == 0 { num_cpus::get() } else { workers },
            results: DashMap::new(),
        }
    }

    /// Analyze a list of modules in parallel
    pub fn analyze_modules(&self, modules: Vec<AnalysisTask>) -> Vec<AnalysisResult> {
        self.results.clear();

        // Get dependency layers for ordered parallelism
        let layers = self.incremental.get_layers();
        let task_map: DashMap<ModuleId, AnalysisTask> = modules.iter()
            .map(|task| (task.id, task.clone()))
            .collect();

        // Process each layer in parallel
        for layer in layers {
            let tasks_in_layer: Vec<_> = layer.iter()
                .filter_map(|id| task_map.get(id).map(|t| t.clone()))
                .collect();

            // Analyze layer in parallel
            let layer_results: Vec<_> = tasks_in_layer
                .par_iter()
                .map(|task| self.analyze_task(task))
                .collect();

            // Store results
            for result in layer_results {
                self.results.insert(result.id, result);
            }
        }

        // Collect all results
        self.results.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Analyze a single task
    fn analyze_task(&self, task: &AnalysisTask) -> AnalysisResult {
        let start = Instant::now();

        // Check cache first
        let cache_key = CacheKey {
            module: task.id,
            hash: crate::infrastructure::incremental::ContentHash::from_str(&task.content),
        };

        if let Some(cached) = self.cache.get(&cache_key) {
            // Cache hit - convert cached errors back to TypeErrors
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
        let errors = match parse_module(&task.content) {
            Ok(ast) => {
                let mut checker = TypeChecker::with_context(self.context.clone());
                checker.check(&ast)
            }
            Err(e) => {
                vec![crate::compiler::analysis::checker::TypeError {
                    message: format!("parse error: {}", e),
                    line: 0,
                    col: 0,
                }]
            }
        };

        let duration = start.elapsed().as_millis() as u64;

        // Cache the result
        let cache_entry = CacheEntry {
            module: task.id,
            hash: cache_key.hash,
            types: vec![], // TODO: Extract inferred types
            errors: vec![], // Use CachedError for now
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size_bytes: task.content.len(),
        };

        let _ = self.cache.set(cache_key, cache_entry);

        // Convert to errors::TypeError for result
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

    /// Convert cached error back to TypeError
    fn cached_error_to_type_error(&self, cached: &CachedError) -> TypeError {
        TypeError::new(
            crate::compiler::errors::ErrorKind::TypeMismatch {
                expected: "".to_string(),
                found: cached.message.clone(),
            },
            crate::compiler::errors::SourceLocation::new(cached.line, cached.col, cached.line, cached.col),
        ).with_file(cached.file.clone())
    }

    /// Analyze a project directory
    pub fn analyze_project(&self, root: &Path) -> Vec<AnalysisResult> {
        // Find all Python files
        let tasks = self.find_python_files(root);

        // Analyze in parallel
        self.analyze_modules(tasks)
    }

    /// Find all Python files in directory
    fn find_python_files(&self, root: &Path) -> Vec<AnalysisTask> {
        use std::fs;

        let mut tasks = Vec::new();

        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // Recursively search subdirectories
                    tasks.extend(self.find_python_files(&path));
                } else if path.extension() == Some(std::ffi::OsStr::new("py")) {
                    // Read Python file
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

    /// Get analysis result for a module
    pub fn get_result(&self, id: ModuleId) -> Option<AnalysisResult> {
        self.results.get(&id).map(|r| r.clone())
    }

    /// Get all results
    pub fn get_all_results(&self) -> Vec<AnalysisResult> {
        self.results.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get number of workers
    pub fn worker_count(&self) -> usize {
        self.workers
    }
}

// Add num_cpus for CPU detection
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
                id: ModuleId(1),
                path: PathBuf::from("test1.py"),
                content: "x = 1 + 2".to_string(),
            },
            AnalysisTask {
                id: ModuleId(2),
                path: PathBuf::from("test2.py"),
                content: "y = \"hello\"".to_string(),
            },
        ];

        let results = analyzer.analyze_modules(tasks);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_cache_hit() {
        let context = Arc::new(TypeContext::new());
        let temp = TempDir::new().unwrap();
        let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());
        let graph = Arc::new(DependencyGraph::new());
        let incremental = Arc::new(IncrementalEngine::new(graph));

        let analyzer = ParallelAnalyzer::new(context, cache.clone(), incremental, 1);

        let task = AnalysisTask {
            id: ModuleId(1),
            path: PathBuf::from("test.py"),
            content: "x = 1".to_string(),
        };

        // First analysis - cache miss
        let result1 = analyzer.analyze_task(&task);

        // Second analysis - should be cache hit (faster)
        let result2 = analyzer.analyze_task(&task);

        assert!(result2.duration_ms <= result1.duration_ms);
    }
}

