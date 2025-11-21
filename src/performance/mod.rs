//! Performance optimizations for Typthon
//!
//! This module implements Phase 4 optimizations including:
//! - Incremental type checking with dependency tracking
//! - Persistent caching of analysis results
//! - Parallel file analysis
//! - Memory pool allocation for AST nodes
//! - Profile-guided optimization support

pub mod incremental;
pub mod cache;
pub mod parallel;
pub mod arena;
pub mod metrics;

pub use incremental::{IncrementalEngine, DependencyGraph, ModuleId, ContentHash};
pub use cache::{ResultCache, CacheKey, CacheEntry};
pub use parallel::{ParallelAnalyzer, AnalysisTask};
pub use arena::AstArena;
pub use metrics::PerformanceMetrics;

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable incremental checking
    pub incremental: bool,

    /// Enable persistent cache
    pub cache_enabled: bool,

    /// Cache directory
    pub cache_dir: std::path::PathBuf,

    /// Maximum cache size in MB
    pub cache_size_mb: usize,

    /// Number of parallel workers (0 = auto)
    pub workers: usize,

    /// Enable memory pooling
    pub memory_pool: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            incremental: true,
            cache_enabled: true,
            cache_dir: std::env::temp_dir().join("typthon_cache"),
            cache_size_mb: 1024, // 1GB default
            workers: 0, // Auto-detect
            memory_pool: true,
        }
    }
}

