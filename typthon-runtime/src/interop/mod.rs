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

/// Initialize interop subsystem
pub fn init() {
    // Future: Initialize FFI runtime, symbol caches, etc.
}

/// Get interop statistics
pub fn stats() -> InteropStats {
    InteropStats {
        calls_made: 0, // TODO: Add counters
        marshaling_errors: 0,
        libraries_loaded: 0,
    }
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
