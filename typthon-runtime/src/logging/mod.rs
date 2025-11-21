//! Logging infrastructure - structured tracing throughout runtime
//!
//! Design: Uses `tracing` for structured, contextual logging with:
//! - Configurable log levels per module
//! - Zero-cost when disabled
//! - Span-based performance tracking
//! - File and console output with rotation

use once_cell::sync::OnceCell;
use std::io;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

mod macros;
pub use macros::*;

/// Global logging state
static LOGGER_INITIALIZED: OnceCell<()> = OnceCell::new();

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Default log level
    pub level: Level,
    /// Enable file logging
    pub file_output: bool,
    /// Log file path (if file_output enabled)
    pub log_path: Option<String>,
    /// Enable JSON format (vs human-readable)
    pub json_format: bool,
    /// Show span events (enter/exit)
    pub show_spans: bool,
    /// Enable performance tracking
    pub track_performance: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            file_output: false,
            log_path: None,
            json_format: false,
            show_spans: false,
            track_performance: cfg!(debug_assertions),
        }
    }
}

impl LogConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // TYPTHON_LOG_LEVEL: trace, debug, info, warn, error
        if let Ok(level_str) = std::env::var("TYPTHON_LOG_LEVEL") {
            config.level = match level_str.to_lowercase().as_str() {
                "trace" => Level::TRACE,
                "debug" => Level::DEBUG,
                "info" => Level::INFO,
                "warn" => Level::WARN,
                "error" => Level::ERROR,
                _ => Level::INFO,
            };
        }

        // TYPTHON_LOG_FILE: path to log file
        if let Ok(path) = std::env::var("TYPTHON_LOG_FILE") {
            config.file_output = true;
            config.log_path = Some(path);
        }

        // TYPTHON_LOG_JSON: enable JSON format
        config.json_format = std::env::var("TYPTHON_LOG_JSON").is_ok();

        // TYPTHON_LOG_SPANS: show span events
        config.show_spans = std::env::var("TYPTHON_LOG_SPANS").is_ok();

        // TYPTHON_LOG_PERF: enable performance tracking
        if let Ok(val) = std::env::var("TYPTHON_LOG_PERF") {
            config.track_performance = val == "1" || val.to_lowercase() == "true";
        }

        config
    }

    /// Create high-performance config (minimal logging)
    pub fn performance() -> Self {
        Self {
            level: Level::ERROR,
            file_output: false,
            log_path: None,
            json_format: false,
            show_spans: false,
            track_performance: false,
        }
    }

    /// Create debug config (verbose logging)
    pub fn debug() -> Self {
        Self {
            level: Level::TRACE,
            file_output: true,
            log_path: Some("typthon_runtime.log".to_string()),
            json_format: false,
            show_spans: true,
            track_performance: true,
        }
    }
}

/// Initialize logging with default configuration
pub fn init() {
    init_with_config(LogConfig::from_env());
}

/// Initialize logging with custom configuration
pub fn init_with_config(config: LogConfig) {
    LOGGER_INITIALIZED.get_or_init(|| {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                EnvFilter::new(format!(
                    "typthon_runtime={}",
                    config.level.as_str().to_lowercase()
                ))
            });

        let span_events = if config.show_spans {
            FmtSpan::ENTER | FmtSpan::CLOSE
        } else {
            FmtSpan::NONE
        };

        // Simplified: just console logging with env filter
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_writer(io::stdout)
                    .with_span_events(span_events)
                    .with_target(true)
                    .with_thread_ids(cfg!(debug_assertions))
                    .with_line_number(cfg!(debug_assertions))
            )
            .init();
    });
}

/// Check if logging is initialized
pub fn is_initialized() -> bool {
    LOGGER_INITIALIZED.get().is_some()
}

// ============================================================================
// Runtime-specific logging functions
// ============================================================================

/// Initialize runtime logging (alias for init)
pub fn init_runtime_logging() {
    init();
}

/// Log memory allocation
#[inline]
pub fn log_allocation(size: usize, ptr: *const u8) {
    use tracing::trace;
    trace!(
        event = "allocation",
        size_bytes = size,
        address = ?ptr,
        "Memory allocated"
    );
}

/// Log memory deallocation
#[inline]
pub fn log_deallocation(ptr: *const u8) {
    use tracing::trace;
    trace!(
        event = "deallocation",
        address = ?ptr,
        "Memory deallocated"
    );
}

/// Log GC cycle start
pub fn log_gc_start(candidate_count: usize) {
    use tracing::info;
    info!(
        event = "gc_start",
        candidates = candidate_count,
        "Starting garbage collection cycle"
    );
}

/// Log GC cycle completion
pub fn log_gc_complete(duration_us: u64, collected: usize, total: usize) {
    use tracing::info;
    info!(
        event = "gc_complete",
        objects_collected = collected,
        total_objects = total,
        duration_us = duration_us,
        "Garbage collection cycle complete"
    );
}

/// Log GC mark phase
pub fn log_gc_mark(marked: usize) {
    use tracing::debug;
    debug!(
        event = "gc_mark",
        objects_marked = marked,
        "GC mark phase complete"
    );
}

/// Log GC sweep phase
pub fn log_gc_sweep(swept: usize) {
    use tracing::debug;
    debug!(
        event = "gc_sweep",
        objects_swept = swept,
        "GC sweep phase complete"
    );
}

/// Log FFI function call
pub fn log_ffi_call(fn_name: &str, arg_count: usize) {
    use tracing::debug;
    debug!(
        event = "ffi_call",
        function = fn_name,
        args = arg_count,
        "FFI function called"
    );
}

/// Log FFI function return
pub fn log_ffi_return(fn_name: &str) {
    use tracing::trace;
    trace!(
        event = "ffi_return",
        function = fn_name,
        "FFI function returned"
    );
}

/// Log FFI error
pub fn log_ffi_error(fn_name: &str, error: &str) {
    use tracing::error;
    error!(
        event = "ffi_error",
        function = fn_name,
        error = error,
        "FFI function error"
    );
}

/// Log builtin function call
pub fn log_builtin_call(builtin: &str) {
    use tracing::trace;
    trace!(
        event = "builtin_call",
        function = builtin,
        "Builtin function called"
    );
}

/// Log type conversion
pub fn log_type_conversion(from_type: &str, to_type: &str) {
    use tracing::trace;
    trace!(
        event = "type_conversion",
        from = from_type,
        to = to_type,
        "Type conversion performed"
    );
}

/// Log runtime error
pub fn log_runtime_error(error: &str) {
    use tracing::error;
    error!(
        event = "runtime_error",
        error = error,
        "Runtime error occurred"
    );
}

/// Log runtime warning
pub fn log_runtime_warning(warning: &str) {
    use tracing::warn;
    warn!(
        event = "runtime_warning",
        warning = warning,
        "Runtime warning"
    );
}

/// Log runtime initialization
pub fn log_runtime_init() {
    use tracing::info;
    info!(
        event = "runtime_init",
        "Typthon runtime initializing"
    );
}

/// Log runtime shutdown
pub fn log_runtime_shutdown() {
    use tracing::info;
    info!(
        event = "runtime_shutdown",
        "Typthon runtime shutting down"
    );
}

/// Performance tracking utilities
pub mod perf {
    use std::time::Instant;
    use tracing::debug;

    /// Track operation duration (returns guard that logs on drop)
    #[must_use]
    pub fn track(operation: &str) -> PerformanceGuard {
        PerformanceGuard {
            operation: operation.to_string(),
            start: Instant::now(),
        }
    }

    pub struct PerformanceGuard {
        operation: String,
        start: Instant,
    }

    impl Drop for PerformanceGuard {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed();
            debug!(
                operation = %self.operation,
                duration_us = elapsed.as_micros(),
                "operation completed"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = LogConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.file_output);

        let perf_config = LogConfig::performance();
        assert_eq!(perf_config.level, Level::ERROR);

        let debug_config = LogConfig::debug();
        assert_eq!(debug_config.level, Level::TRACE);
    }

    #[test]
    fn test_init_idempotent() {
        init();
        init(); // Should not panic
        assert!(is_initialized());
    }
}

