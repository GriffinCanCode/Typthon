/*!
Diagnostic collection and reporting for LSP.
*/

use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub source: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.source,
            self.line + 1,
            self.col + 1,
            match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
                Severity::Hint => "hint",
            },
            self.message
        )
    }
}

/// Collects diagnostics during analysis
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn error(&mut self, line: usize, col: usize, message: String) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            line,
            col,
            message,
            source: "typthon".to_string(),
        });
    }

    pub fn warning(&mut self, line: usize, col: usize, message: String) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            line,
            col,
            message,
            source: "typthon".to_string(),
        });
    }

    pub fn info(&mut self, line: usize, col: usize, message: String) {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Info,
            line,
            col,
            message,
            source: "typthon".to_string(),
        });
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

