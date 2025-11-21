//! Standardized logging infrastructure for Typthon
//!
//! This module provides a consistent logging interface using the `tracing` crate,
//! with support for structured logging, multiple output formats, and flexible configuration.

use std::path::Path;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable format with timestamps
    Pretty,
    /// Compact format for production
    Compact,
    /// JSON format for structured logging
    Json,
}

/// Log output destination
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogOutput {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// File with rotation (daily)
    File { directory: String, prefix: String },
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Minimum log level
    pub level: Level,
    /// Output format
    pub format: LogFormat,
    /// Output destination
    pub output: LogOutput,
    /// Whether to include span events
    pub span_events: bool,
    /// Custom filter directives (e.g., "typthon=debug,hyper=info")
    pub filter: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Pretty,
            output: LogOutput::Stderr,
            span_events: false,
            filter: None,
        }
    }
}

impl LogConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_output(mut self, output: LogOutput) -> Self {
        self.output = output;
        self
    }

    pub fn with_span_events(mut self, enabled: bool) -> Self {
        self.span_events = enabled;
        self
    }

    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

/// Initialize the global logging system
///
/// Returns a `WorkerGuard` that must be kept alive for the duration of the program
/// to ensure all logs are flushed. Drop it before the program exits.
pub fn init_logging(config: LogConfig) -> Option<WorkerGuard> {
    let filter = build_filter(&config);

    match config.output {
        LogOutput::Stdout | LogOutput::Stderr => {
            let (writer, guard) = if matches!(config.output, LogOutput::Stdout) {
                let (w, g) = tracing_appender::non_blocking(std::io::stdout());
                (w, Some(g))
            } else {
                let (w, g) = tracing_appender::non_blocking(std::io::stderr());
                (w, Some(g))
            };

            match config.format {
                LogFormat::Pretty => {
                    let layer = fmt::layer()
                        .with_writer(writer)
                        .pretty()
                        .with_span_events(span_events_config(config.span_events))
                        .with_filter(filter);

                    tracing_subscriber::registry().with(layer).init();
                }
                LogFormat::Compact => {
                    let layer = fmt::layer()
                        .with_writer(writer)
                        .compact()
                        .with_span_events(span_events_config(config.span_events))
                        .with_filter(filter);

                    tracing_subscriber::registry().with(layer).init();
                }
                LogFormat::Json => {
                    let layer = fmt::layer()
                        .with_writer(writer)
                        .json()
                        .with_span_events(span_events_config(config.span_events))
                        .with_filter(filter);

                    tracing_subscriber::registry().with(layer).init();
                }
            }
            guard
        }
        LogOutput::File { directory, prefix } => {
            let file_appender = rolling::daily(&directory, &prefix);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            match config.format {
                LogFormat::Pretty => {
                    let layer = fmt::layer()
                        .with_writer(non_blocking)
                        .pretty()
                        .with_span_events(span_events_config(config.span_events))
                        .with_filter(filter);

                    tracing_subscriber::registry().with(layer).init();
                }
                LogFormat::Compact => {
                    let layer = fmt::layer()
                        .with_writer(non_blocking)
                        .compact()
                        .with_span_events(span_events_config(config.span_events))
                        .with_filter(filter);

                    tracing_subscriber::registry().with(layer).init();
                }
                LogFormat::Json => {
                    let layer = fmt::layer()
                        .with_writer(non_blocking)
                        .json()
                        .with_span_events(span_events_config(config.span_events))
                        .with_filter(filter);

                    tracing_subscriber::registry().with(layer).init();
                }
            }
            Some(guard)
        }
    }
}

fn build_filter(config: &LogConfig) -> EnvFilter {
    let base_filter = EnvFilter::from_default_env()
        .add_directive(config.level.into());

    match &config.filter {
        Some(filter_str) => {
            filter_str.split(',')
                .fold(base_filter, |filter, directive| {
                    filter.add_directive(directive.parse().unwrap_or_else(|_| {
                        tracing::warn!("Invalid filter directive: {}", directive);
                        config.level.into()
                    }))
                })
        }
        None => base_filter,
    }
}

fn span_events_config(enabled: bool) -> FmtSpan {
    if enabled {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    }
}

/// Initialize logging with defaults for development
pub fn init_dev_logging() -> Option<WorkerGuard> {
    init_logging(LogConfig {
        level: Level::DEBUG,
        format: LogFormat::Pretty,
        output: LogOutput::Stderr,
        span_events: true,
        filter: Some("typthon=debug".to_string()),
    })
}

/// Initialize logging with defaults for production
pub fn init_prod_logging(log_dir: impl AsRef<Path>) -> Option<WorkerGuard> {
    init_logging(LogConfig {
        level: Level::INFO,
        format: LogFormat::Json,
        output: LogOutput::File {
            directory: log_dir.as_ref().to_string_lossy().to_string(),
            prefix: "typthon".to_string(),
        },
        span_events: false,
        filter: Some("typthon=info,warn".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = LogConfig::new()
            .with_level(Level::DEBUG)
            .with_format(LogFormat::Json)
            .with_span_events(true)
            .with_filter("typthon=trace");

        assert_eq!(config.level, Level::DEBUG);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.span_events, true);
        assert_eq!(config.filter, Some("typthon=trace".to_string()));
    }
}

