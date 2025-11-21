//! C API for FFI with Go compiler
//!
//! Design: Minimal C-compatible interface for type checking from Go

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::compiler::frontend::parse_module;
use crate::compiler::analysis::TypeChecker;
use crate::infrastructure::init_logging;

/// Initialize the type checker (called once from Go)
#[no_mangle]
pub extern "C" fn typthon_init_checker() {
    init_logging();
}

/// Cleanup type checker resources
#[no_mangle]
pub extern "C" fn typthon_cleanup_checker() {
    // Cleanup if needed
}

/// Type check a file by path
/// Returns 0 on success, non-zero on error
#[no_mangle]
pub extern "C" fn typthon_check_file(filename: *const c_char) -> i32 {
    if filename.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(filename) };
    let filename_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    // Read file and check
    let source = match std::fs::read_to_string(filename_str) {
        Ok(s) => s,
        Err(_) => return -2,
    };

    typthon_check_source_impl(&source)
}

/// Type check source code directly
/// Returns 0 on success, non-zero on error
#[no_mangle]
pub extern "C" fn typthon_check_source(source: *const c_char, len: i32) -> i32 {
    if source.is_null() || len < 0 {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(source) };
    let source_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    typthon_check_source_impl(source_str)
}

fn typthon_check_source_impl(source: &str) -> i32 {
    // Parse and type check
    let ast = match parse_module(source, "input") {
        Ok(ast) => ast,
        Err(_) => return -3,
    };

    let mut checker = TypeChecker::new();
    match checker.check_module(&ast) {
        Ok(_) => 0,
        Err(_) => -4,
    }
}

/// Get type information for a symbol (placeholder for future)
#[no_mangle]
pub extern "C" fn typthon_get_type_info(
    symbol: *const c_char,
) -> *const c_char {
    if symbol.is_null() {
        return ptr::null();
    }

    // TODO: Implement actual type lookup
    // For now, return a placeholder
    let type_str = CString::new("int").unwrap();
    type_str.into_raw()
}

/// Free a string returned by the type checker
#[no_mangle]
pub extern "C" fn typthon_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

