//! Interoperability - Call external functions and languages
//!
//! Design: Zero-overhead FFI with automatic type marshaling
//!
//! Architecture:
//! - `types.rs` - FFI type system (FfiType, FfiValue, TypedArg)
//! - `marshal.rs` - Python ↔ C type conversions
//! - `call.rs` - Dynamic function calling with inline assembly
//! - `abi.rs` - Calling convention support (System V, Win64, ARM)
//! - `library.rs` - Dynamic library loading (dlopen/LoadLibrary)

mod types;
mod marshal;
mod call;
mod abi;
mod library;

pub use types::{FfiType, FfiValue, TypedArg};
pub use marshal::{to_c, from_c, marshal_args, can_zero_copy, python_type_name};
pub use call::{call_extern, FunctionCall, CallError};
pub use abi::{CallingConvention, RegisterAllocator};
pub use library::{Library, LoadError, SymbolError};

use crate::logging::{info, debug};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Global statistics counters
static CALLS_MADE: AtomicUsize = AtomicUsize::new(0);
static MARSHALING_ERRORS: AtomicUsize = AtomicUsize::new(0);
static LIBRARIES_LOADED: AtomicUsize = AtomicUsize::new(0);

/// Initialize interop subsystem
pub fn init() {
    info!("Interop subsystem initializing");
    CALLS_MADE.store(0, Ordering::Relaxed);
    MARSHALING_ERRORS.store(0, Ordering::Relaxed);
    LIBRARIES_LOADED.store(0, Ordering::Relaxed);
    debug!("FFI and library loading capabilities ready");
}

/// Increment FFI call counter
pub(crate) fn increment_calls() {
    CALLS_MADE.fetch_add(1, Ordering::Relaxed);
}

/// Increment marshaling error counter
pub(crate) fn increment_marshaling_errors() {
    MARSHALING_ERRORS.fetch_add(1, Ordering::Relaxed);
}

/// Increment libraries loaded counter
pub(crate) fn increment_libraries_loaded() {
    LIBRARIES_LOADED.fetch_add(1, Ordering::Relaxed);
}

/// Get interop statistics
pub fn stats() -> InteropStats {
    let stats = InteropStats {
        calls_made: CALLS_MADE.load(Ordering::Relaxed),
        marshaling_errors: MARSHALING_ERRORS.load(Ordering::Relaxed),
        libraries_loaded: LIBRARIES_LOADED.load(Ordering::Relaxed),
    };

    debug!(
        ffi_calls = stats.calls_made,
        errors = stats.marshaling_errors,
        libraries = stats.libraries_loaded,
        "Interop statistics retrieved"
    );

    stats
}

/// Interop statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct InteropStats {
    pub calls_made: usize,
    pub marshaling_errors: usize,
    pub libraries_loaded: usize,
}

#[cfg(test)]
mod tests;
