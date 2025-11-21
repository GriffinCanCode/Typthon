//! Built-in functions - minimal implementation of core Python builtins
//!
//! Design: Zero-overhead abstractions for compiled Python, exposing both
//! C FFI exports and safe Rust APIs. Each builtin is in a focused module.

mod print;
mod len;
mod iter;

#[cfg(test)]
mod tests;

pub use print::{print_int, print_str, print_float, Output};
pub use len::{len, HasLen};
pub use iter::{Range, range};

use crate::logging::{info, debug};

/// Initialize builtins subsystem
///
/// Pre-allocates resources and caches for builtin operations.
pub fn init() {
    info!("Builtins subsystem initializing");
    print::init();
    debug!("Builtins initialized (print, len, iter)");
}

/// Cleanup builtins resources
pub fn cleanup() {
    debug!("Cleaning up builtins subsystem");
    print::cleanup();
}
