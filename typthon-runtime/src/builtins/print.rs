//! Output operations - print functions for primitive types
//!
//! Design: Minimal overhead with optional buffering for performance.
//! Supports multiple output targets (stdout, stderr, custom).

use crate::logging::{debug, trace};

/// Output target abstraction
pub trait Output {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ()>;

    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), ()> {
        self.write_bytes(s.as_bytes())
    }
}

/// Global output state (future: thread-local buffers)
static mut STDOUT: Option<&'static mut dyn Output> = None;

/// Initialize print subsystem
pub(crate) fn init() {
    debug!("Print subsystem initialized");
    // Future: Initialize output buffers
}

/// Cleanup print subsystem
pub(crate) fn cleanup() {
    trace!("Cleaning up print subsystem");
    unsafe { STDOUT = None; }
}

/// Print integer (C FFI export)
#[no_mangle]
pub extern "C" fn typthon_print_int(val: i64) {
    print_int(val);
}

/// Print string (C FFI export)
#[no_mangle]
pub extern "C" fn typthon_print_str(ptr: *const u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    if let Ok(s) = core::str::from_utf8(slice) {
        print_str(s);
    }
}

/// Print float (C FFI export)
#[no_mangle]
pub extern "C" fn typthon_print_float(val: f64) {
    print_float(val);
}

/// Safe Rust API - print integer
#[inline]
pub fn print_int(val: i64) {
    // Future: Use buffered output
    println!("{}", val);
}

/// Safe Rust API - print string
#[inline]
pub fn print_str(s: &str) {
    println!("{}", s);
}

/// Safe Rust API - print float
#[inline]
pub fn print_float(val: f64) {
    println!("{}", val);
}

/// Formatted output buffer (future: zero-alloc printing)
pub struct PrintBuffer {
    cursor: usize,
    capacity: usize,
}

impl PrintBuffer {
    const CAPACITY: usize = 4096;

    #[inline]
    pub const fn new() -> Self {
        Self {
            cursor: 0,
            capacity: Self::CAPACITY,
        }
    }

    /// Flush buffered output
    pub fn flush(&mut self) {
        // Future implementation
    }
}
