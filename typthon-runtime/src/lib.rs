//! Typthon Runtime - Minimal runtime for compiled Python
//!
//! This crate provides the core runtime support statically linked into
//! compiled Typthon programs.

#![allow(dead_code)]

pub mod allocator;
pub mod gc;
pub mod builtins;
pub mod interop;
pub mod ffi;

// Re-export core types
pub use allocator::Allocator;
pub use gc::RefCount;
pub use builtins::*;

/// Runtime initialization
#[no_mangle]
pub extern "C" fn typthon_runtime_init() {
    // Initialize global state
    allocator::init();
    gc::init();
}

/// Runtime cleanup
#[no_mangle]
pub extern "C" fn typthon_runtime_cleanup() {
    gc::cleanup();
}

