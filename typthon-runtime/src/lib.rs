//! Typthon Runtime - Minimal runtime for compiled Python
//!
//! This crate provides the core runtime support statically linked into
//! compiled Typthon programs.

#![allow(dead_code)]

pub mod logging;
pub mod allocator;
pub mod gc;
pub mod objects;
pub mod builtins;
pub mod interop;
pub mod ffi;

// Re-export core types
pub use allocator::Allocator;
pub use gc::RefCount;
pub use objects::{PyObject, ObjectType};
pub use builtins::*;

// Re-export logging for convenience
pub use logging::{
    init_runtime_logging, log_allocation, log_deallocation, log_gc_start,
    log_gc_complete, log_gc_mark, log_gc_sweep, log_ffi_call, log_ffi_return,
    log_ffi_error, log_builtin_call, log_type_conversion, log_runtime_error,
    log_runtime_warning, log_runtime_init, log_runtime_shutdown,
};

/// Runtime initialization
#[no_mangle]
pub extern "C" fn typthon_runtime_init() {
    // Initialize logging first
    init_runtime_logging();
    log_runtime_init();

    // Initialize subsystems would go here
    // allocator::init();
    // gc::init();
    // builtins::init();
    // interop::init();
}

/// Runtime cleanup
#[no_mangle]
pub extern "C" fn typthon_runtime_cleanup() {
    log_runtime_shutdown();
    // Cleanup subsystems would go here
    // builtins::cleanup();
    // gc::cleanup();
}

