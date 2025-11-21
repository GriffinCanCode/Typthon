//! Logging utilities for Typthon runtime
//!
//! Provides lightweight logging for runtime operations including GC, allocation,
//! and FFI interactions. Uses `tracing` for structured logging with minimal overhead.

// Re-export tracing macros for use throughout the runtime
pub use tracing::{debug, error, info, trace, warn, Level};

/// Initialize runtime logging with sensible defaults
///
/// This should be called early in the runtime initialization process.
/// For production builds, logs at INFO level and above are enabled.
/// For debug builds, DEBUG and TRACE levels are also enabled.
pub fn init_runtime_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            #[cfg(debug_assertions)]
            {
                EnvFilter::new("typthon_runtime=debug")
            }
            #[cfg(not(debug_assertions))]
            {
                EnvFilter::new("typthon_runtime=info")
            }
        });

    fmt()
        .with_env_filter(filter)
        .compact()
        .try_init()
        .ok(); // Ignore error if already initialized
}

/// Log an allocation event
#[inline]
pub fn log_allocation(size: usize, ptr: *const u8) {
    trace!(
        target: "allocator",
        size,
        ptr = ?ptr,
        "allocated memory"
    );
}

/// Log a deallocation event
#[inline]
pub fn log_deallocation(size: usize, ptr: *const u8) {
    trace!(
        target: "allocator",
        size,
        ptr = ?ptr,
        "deallocated memory"
    );
}

/// Log a GC cycle start
#[inline]
pub fn log_gc_start() {
    debug!(target: "gc", "starting garbage collection cycle");
}

/// Log a GC cycle completion
#[inline]
pub fn log_gc_complete(duration_us: u64, collected_bytes: usize, live_objects: usize) {
    info!(
        target: "gc",
        duration_us,
        collected_bytes,
        live_objects,
        "garbage collection complete"
    );
}

/// Log a GC mark phase
#[inline]
pub fn log_gc_mark(objects_marked: usize) {
    debug!(
        target: "gc",
        objects_marked,
        "mark phase complete"
    );
}

/// Log a GC sweep phase
#[inline]
pub fn log_gc_sweep(objects_swept: usize, bytes_reclaimed: usize) {
    debug!(
        target: "gc",
        objects_swept,
        bytes_reclaimed,
        "sweep phase complete"
    );
}

/// Log an FFI call
#[inline]
pub fn log_ffi_call(function_name: &str, args_count: usize) {
    trace!(
        target: "ffi",
        function = function_name,
        args_count,
        "FFI call"
    );
}

/// Log an FFI return
#[inline]
pub fn log_ffi_return(function_name: &str, success: bool) {
    trace!(
        target: "ffi",
        function = function_name,
        success,
        "FFI return"
    );
}

/// Log an FFI error
#[inline]
pub fn log_ffi_error(function_name: &str, error: &str) {
    error!(
        target: "ffi",
        function = function_name,
        error,
        "FFI error"
    );
}

/// Log a builtin function call
#[inline]
pub fn log_builtin_call(name: &str) {
    trace!(
        target: "builtins",
        name,
        "builtin function called"
    );
}

/// Log a type conversion
#[inline]
pub fn log_type_conversion(from: &str, to: &str) {
    trace!(
        target: "interop",
        from,
        to,
        "type conversion"
    );
}

/// Log a runtime error
#[inline]
pub fn log_runtime_error(error: &str, context: &str) {
    error!(
        target: "runtime",
        error,
        context,
        "runtime error"
    );
}

/// Log a runtime warning
#[inline]
pub fn log_runtime_warning(warning: &str, context: &str) {
    warn!(
        target: "runtime",
        warning,
        context,
        "runtime warning"
    );
}

/// Log runtime initialization
#[inline]
pub fn log_runtime_init() {
    info!(target: "runtime", "Typthon runtime initialized");
}

/// Log runtime shutdown
#[inline]
pub fn log_runtime_shutdown() {
    info!(target: "runtime", "Typthon runtime shutting down");
}

/// Macro for creating a traced function span
///
/// Usage:
/// ```ignore
/// #[traced_fn]
/// fn my_function() {
///     // function body
/// }
/// ```
#[macro_export]
macro_rules! traced_fn {
    ($name:expr) => {
        tracing::debug_span!($name).entered()
    };
}

/// Macro for timing a block of code
///
/// Usage:
/// ```ignore
/// time_block!("operation_name", {
///     // code to time
/// });
/// ```
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:block) => {{
        let _span = tracing::debug_span!($name).entered();
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::debug!(
            operation = $name,
            duration_us = duration.as_micros(),
            "operation complete"
        );
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_functions() {
        // These should not panic
        log_allocation(1024, std::ptr::null());
        log_deallocation(1024, std::ptr::null());
        log_gc_start();
        log_gc_complete(1000, 4096, 42);
        log_ffi_call("test_function", 3);
        log_ffi_return("test_function", true);
        log_builtin_call("len");
        log_type_conversion("int", "float");
        log_runtime_init();
        log_runtime_shutdown();
    }
}

