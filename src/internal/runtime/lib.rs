//! Runtime Module - Exposes runtime functionality for compiled Python
//!
//! This module re-exports the runtime functionality from typthon-runtime
//! that's needed for interop and FFI.

#[path = "../../../typthon-runtime/src/lib.rs"]
mod typthon_runtime;

// Re-export runtime core types
pub use typthon_runtime::{
    allocator::{Allocator, HeapStats, AllocationInfo},
    gc::{RefCount, GarbageCollector, GcStats},
    builtins::{print, len, range},
    interop::{PyObject, PyType, to_python, from_python},
    ffi::{CFunction, call_c_function},
};

// Re-export runtime initialization functions
pub use typthon_runtime::{
    typthon_runtime_init,
    typthon_runtime_cleanup,
};

/// Initialize the runtime with custom configuration
pub fn init_runtime() {
    typthon_runtime_init();
}

/// Cleanup runtime resources
pub fn cleanup_runtime() {
    typthon_runtime_cleanup();
}

/// Get current GC statistics
pub fn get_gc_stats() -> GcStats {
    typthon_runtime::gc::get_stats()
}

/// Get current allocator statistics
pub fn get_heap_stats() -> HeapStats {
    typthon_runtime::allocator::get_stats()
}

/// Force garbage collection
pub fn force_gc() {
    typthon_runtime::gc::collect();
}

