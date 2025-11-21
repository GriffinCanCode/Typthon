//! C FFI - Stable ABI for generated code
//!
//! Design: Zero-overhead C bindings with:
//! 1. Object lifecycle (alloc, destroy)
//! 2. Reference counting (incref, decref)
//! 3. Type-safe conversions (Rust ↔ C)
//! 4. Error propagation via null pointers

mod object;
mod refcount;

pub use object::{typthon_object_new, typthon_object_destroy};
pub use refcount::{typthon_incref, typthon_decref, typthon_refcount};

use core::ptr::NonNull;
use crate::logging::{info, debug};

/// Initialize FFI subsystem (called once at program start)
#[no_mangle]
pub extern "C" fn typthon_ffi_init() {
    info!("FFI subsystem initializing");
    // Future: Register signal handlers, set up thread-local storage
    debug!("FFI ready for C interop");
}

/// Cleanup FFI subsystem (called at program exit)
#[no_mangle]
pub extern "C" fn typthon_ffi_cleanup() {
    debug!("Cleaning up FFI subsystem");
    // Future: Flush pending operations, validate refcounts
}

/// Get last error code (thread-local, for error propagation)
#[no_mangle]
pub extern "C" fn typthon_last_error() -> i32 {
    // Future: Implement thread-local error state
    0
}

/// Convert raw pointer to NonNull (internal helper)
#[inline(always)]
pub(crate) fn ptr_to_nonnull<T>(ptr: *mut T) -> Option<NonNull<T>> {
    NonNull::new(ptr)
}

/// Check pointer validity (internal helper)
#[inline(always)]
pub(crate) fn is_valid_ptr(ptr: *const u8) -> bool {
    !ptr.is_null() && ptr.is_aligned()
}

