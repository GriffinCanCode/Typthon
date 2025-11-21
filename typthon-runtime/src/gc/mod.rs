//! Garbage collector - reference counting + cycle detection
//!
//! Design: Hybrid approach optimized for Python semantics:
//! 1. Reference counting (fast path, deterministic)
//! 2. Tricolor mark-sweep (rare, cycles only)
//! 3. Deferred collection (amortized cost)

mod refcount;
mod cycles;
mod roots;

#[cfg(test)]
mod tests;

pub use refcount::RefCount;
pub use cycles::{collect_cycles, register_potential_cycle};
pub use roots::{register_root, unregister_root, RootGuard};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use once_cell::sync::Lazy;
use crate::logging::{info, debug, warn, log_gc_complete};

/// Global GC state (lock-free counters + mutex for rare operations)
static GC_STATE: Lazy<GcState> = Lazy::new(GcState::new);

struct GcState {
    collection_threshold: AtomicUsize,
    objects_since_collection: AtomicUsize,
    collections_performed: AtomicUsize,
}

impl GcState {
    const INITIAL_THRESHOLD: usize = 700; // Python's default

    fn new() -> Self {
        Self {
            collection_threshold: AtomicUsize::new(Self::INITIAL_THRESHOLD),
            objects_since_collection: AtomicUsize::new(0),
            collections_performed: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn increment_objects(&self) -> bool {
        let count = self.objects_since_collection.fetch_add(1, Ordering::Relaxed);
        let threshold = self.collection_threshold.load(Ordering::Relaxed);
        count >= threshold
    }

    #[inline]
    fn reset_counter(&self) {
        self.objects_since_collection.store(0, Ordering::Relaxed);
        self.collections_performed.fetch_add(1, Ordering::Relaxed);
    }
}

/// Initialize GC subsystem
pub fn init() {
    info!("GC subsystem initializing");

    // Force initialization of lazy statics
    Lazy::force(&GC_STATE);
    roots::init_roots();
    cycles::init_collector();

    debug!(
        threshold = GC_STATE.collection_threshold.load(Ordering::Relaxed),
        "GC initialized with default threshold"
    );
}

/// Final cleanup and collection
pub fn cleanup() {
    info!("GC subsystem cleanup starting");

    let start = Instant::now();
    collect_cycles();
    roots::clear_roots();

    let stats = stats();
    log_gc_complete(start.elapsed().as_micros() as u64, 0, stats.total_objects);
}

/// Trigger GC if threshold exceeded (called after allocations)
#[inline]
pub fn maybe_collect() {
    if GC_STATE.increment_objects() {
        debug!(
            object_count = GC_STATE.objects_since_collection.load(Ordering::Relaxed),
            threshold = GC_STATE.collection_threshold.load(Ordering::Relaxed),
            "GC threshold exceeded, starting collection"
        );

        let start = Instant::now();
        collect_cycles();
        GC_STATE.reset_counter();

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 10 {
            warn!(
                duration_ms = elapsed.as_millis(),
                "GC collection took significant time"
            );
        }
    }
}

/// Force immediate collection (for testing/profiling)
pub fn force_collect() {
    info!("Forcing immediate GC collection");

    let start = Instant::now();
    collect_cycles();
    GC_STATE.reset_counter();

    let stats = stats();
    log_gc_complete(start.elapsed().as_micros() as u64, 0, stats.total_objects);
}

/// Get GC statistics
pub fn stats() -> GcStats {
    let mut base_stats = cycles::collector_stats();
    base_stats.collections_run = GC_STATE.collections_performed.load(Ordering::Relaxed);

    debug!(
        total_objects = base_stats.total_objects,
        reachable = base_stats.reachable_objects,
        cycles_collected = base_stats.cycles_collected,
        collections_run = base_stats.collections_run,
        "GC statistics retrieved"
    );

    base_stats
}

/// GC statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct GcStats {
    pub total_objects: usize,
    pub reachable_objects: usize,
    pub cycles_collected: usize,
    pub collections_run: usize,
}
