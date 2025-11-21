//! Built-in functions - minimal implementation of core Python builtins
//!
//! Design: Zero-overhead abstractions for compiled Python, exposing both
//! C FFI exports and safe Rust APIs. Each builtin is in a focused module.

mod print;
mod len;
mod iter;
mod string;
mod list;
mod dict;

#[cfg(test)]
mod tests;

pub use print::{print_int, print_str, print_float, Output};
pub use len::{len, HasLen};
pub use iter::{Range, range};
pub use string::{py_string_new, py_string_len, py_string_concat, py_string_eq, py_string_as_str};
pub use list::{py_list_new, py_list_len, py_list_get, py_list_set, py_list_append, py_list_from_slice};
pub use dict::{py_dict_new, py_dict_len, py_dict_get, py_dict_set, py_dict_contains};

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
