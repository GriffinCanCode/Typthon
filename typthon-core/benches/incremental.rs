//! Incremental checking benchmarks
//!
//! Measures performance of incremental vs full checking.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use typthon::{
    DependencyGraph, IncrementalEngine, ResultCache, ParallelAnalyzer,
    TypeContext, PerformanceConfig,
};
use typthon::performance::{AnalysisTask, ModuleId};

fn generate_module_content(id: usize, imports: &[usize]) -> String {
    let mut content = String::new();

    // Add imports
    for import in imports {
        content.push_str(&format!("import module_{}\n", import));
    }

    // Add some code
    content.push_str(&format!(r#"
def function_{}(x: int, y: int) -> int:
    return x + y + {}

class Class_{}:
    def method(self, value: str) -> str:
        return value + "_processed"

data_{} = [1, 2, 3, 4, 5]
result_{} = function_{}(10, 20)
"#, id, id, id, id, id, id));

    content
}

fn setup_project(num_modules: usize, deps_per_module: usize) -> (Vec<AnalysisTask>, Arc<IncrementalEngine>) {
    let graph = Arc::new(DependencyGraph::new());
    let incremental = Arc::new(IncrementalEngine::new(graph.clone()));

    let mut tasks = Vec::new();

    for i in 0..num_modules {
        let deps: Vec<usize> = if i > 0 {
            (0..deps_per_module.min(i))
                .map(|_| i.saturating_sub(1))
                .collect()
        } else {
            vec![]
        };

        let content = generate_module_content(i, &deps);
        let path = PathBuf::from(format!("module_{}.py", i));
        let id = ModuleId::from_path(&path);

        // Register with incremental engine
        incremental.register_module(path.clone(), &content, vec![]);

        tasks.push(AnalysisTask {
            id,
            path,
            content,
        });
    }

    (tasks, incremental)
}

fn bench_full_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_analysis");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (tasks, incremental) = setup_project(size, 2);
            let temp = TempDir::new().unwrap();
            let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());
            let context = Arc::new(TypeContext::new());
            let analyzer = ParallelAnalyzer::new(context, cache, incremental, 4);

            b.iter(|| {
                black_box(analyzer.analyze_modules(tasks.clone()))
            });
        });
    }

    group.finish();
}

fn bench_incremental_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_update");

    for size in [50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (mut tasks, incremental) = setup_project(size, 2);
            let temp = TempDir::new().unwrap();
            let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());
            let context = Arc::new(TypeContext::new());
            let analyzer = ParallelAnalyzer::new(context, cache, incremental.clone(), 4);

            // Initial full analysis
            analyzer.analyze_modules(tasks.clone());

            b.iter(|| {
                // Change one module
                if let Some(task) = tasks.first_mut() {
                    task.content.push_str("\n# Comment added\n");
                    incremental.mark_changed(task.id);
                }

                // Reanalyze (should be faster due to incremental)
                black_box(analyzer.analyze_modules(
                    incremental.get_invalid_modules()
                        .into_iter()
                        .filter_map(|id| tasks.iter().find(|t| t.id == id).cloned())
                        .collect()
                ))
            });
        });
    }

    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    let temp = TempDir::new().unwrap();
    let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());

    group.bench_function("cold_miss", |b| {
        let (tasks, _) = setup_project(10, 1);
        let context = Arc::new(TypeContext::new());
        let graph = Arc::new(DependencyGraph::new());
        let incremental = Arc::new(IncrementalEngine::new(graph));
        let analyzer = ParallelAnalyzer::new(context, cache.clone(), incremental, 1);

        b.iter(|| {
            black_box(analyzer.analyze_modules(tasks.clone()))
        });
    });

    group.bench_function("warm_hit", |b| {
        let (tasks, _) = setup_project(10, 1);
        let context = Arc::new(TypeContext::new());
        let graph = Arc::new(DependencyGraph::new());
        let incremental = Arc::new(IncrementalEngine::new(graph));
        let analyzer = ParallelAnalyzer::new(context, cache.clone(), incremental, 1);

        // Populate cache
        analyzer.analyze_modules(tasks.clone());

        b.iter(|| {
            black_box(analyzer.analyze_modules(tasks.clone()))
        });
    });

    group.finish();
}

fn bench_parallel_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_scaling");

    let (tasks, _) = setup_project(100, 3);

    for workers in [1, 2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(workers), workers, |b, &workers| {
            let temp = TempDir::new().unwrap();
            let cache = Arc::new(ResultCache::new(temp.path().to_path_buf(), 100).unwrap());
            let context = Arc::new(TypeContext::new());
            let graph = Arc::new(DependencyGraph::new());
            let incremental = Arc::new(IncrementalEngine::new(graph));
            let analyzer = ParallelAnalyzer::new(context, cache, incremental, workers);

            b.iter(|| {
                black_box(analyzer.analyze_modules(tasks.clone()))
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_full_analysis,
    bench_incremental_update,
    bench_cache_performance,
    bench_parallel_scaling,
);
criterion_main!(benches);

