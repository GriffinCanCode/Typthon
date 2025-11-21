//! Location extraction from rustpython AST nodes
//!
//! Provides utilities to extract precise source locations from all AST node types.
//! This enables accurate error reporting and source mapping.
//!
//! ## Design
//!
//! rustpython-parser 0.3 provides byte offsets via the `Ranged` trait.
//! We convert these to line:column positions by maintaining line boundaries
//! from the source text.

use rustpython_parser::ast::{Expr, Stmt, Pattern, Mod, Ranged, ModModule};
use crate::compiler::errors::SourceLocation;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Line index for fast byte offset to line/column conversion
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offsets where each line starts
    line_starts: Vec<usize>,
}

impl LineIndex {
    /// Create line index from source text
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    /// Convert byte offset to (line, column)
    pub fn offset_to_position(&self, offset: usize) -> (usize, usize) {
        // Binary search for the line
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };

        let line_start = self.line_starts[line];
        let column = offset.saturating_sub(line_start);

        (line + 1, column) // 1-indexed line numbers
    }
}

thread_local! {
    /// Thread-local cache of line indices keyed by source hash
    static LINE_CACHE: Arc<RwLock<HashMap<u64, Arc<LineIndex>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Extension trait for extracting source locations from AST nodes
pub trait SourceLocationExt {
    fn source_location(&self, index: &LineIndex) -> SourceLocation;
}

impl SourceLocationExt for Expr {
    fn source_location(&self, index: &LineIndex) -> SourceLocation {
        let range = self.range();
        let (start_line, start_col) = index.offset_to_position(range.start().to_usize());
        let (end_line, end_col) = index.offset_to_position(range.end().to_usize());
        SourceLocation::new(start_line, start_col, end_line, end_col)
    }
}

impl SourceLocationExt for Stmt {
    fn source_location(&self, index: &LineIndex) -> SourceLocation {
        let range = self.range();
        let (start_line, start_col) = index.offset_to_position(range.start().to_usize());
        let (end_line, end_col) = index.offset_to_position(range.end().to_usize());
        SourceLocation::new(start_line, start_col, end_line, end_col)
    }
}

impl SourceLocationExt for Pattern {
    fn source_location(&self, index: &LineIndex) -> SourceLocation {
        let range = self.range();
        let (start_line, start_col) = index.offset_to_position(range.start().to_usize());
        let (end_line, end_col) = index.offset_to_position(range.end().to_usize());
        SourceLocation::new(start_line, start_col, end_line, end_col)
    }
}

impl SourceLocationExt for Mod {
    fn source_location(&self, index: &LineIndex) -> SourceLocation {
        // Mod doesn't have a direct range, so we use the first/last statement
        match self {
            Mod::Module(ModModule { body, .. }) if !body.is_empty() => {
                let first = &body[0];
                let last = &body[body.len() - 1];
                let first_range = first.range();
                let last_range = last.range();

                let (start_line, start_col) = index.offset_to_position(first_range.start().to_usize());
                let (end_line, end_col) = index.offset_to_position(last_range.end().to_usize());
                SourceLocation::new(start_line, start_col, end_line, end_col)
            }
            _ => SourceLocation::new(1, 0, 1, 0), // Empty module
        }
    }
}

/// Fallback: create location from range when no source available
pub fn location_from_range<T: Ranged>(node: &T) -> SourceLocation {
    let range = node.range();
    let start = range.start().to_usize();
    let end = range.end().to_usize();

    // Rough approximation: assume ~80 chars per line
    let start_line = start / 80 + 1;
    let start_col = start % 80;
    let end_line = end / 80 + 1;
    let end_col = end % 80;

    SourceLocation::new(start_line, start_col, end_line, end_col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::frontend::parse_module;
    use rustpython_parser::ast::ModModule;

    #[test]
    fn test_line_index() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);

        assert_eq!(index.offset_to_position(0), (1, 0));  // Start of line 1
        assert_eq!(index.offset_to_position(6), (2, 0));  // Start of line 2
        assert_eq!(index.offset_to_position(12), (3, 0)); // Start of line 3
    }

    #[test]
    fn test_expr_location_extraction() {
        let source = "x = 1 + 2";
        let index = LineIndex::new(source);
        let ast = parse_module(source).unwrap();

        if let Mod::Module(ModModule { body, .. }) = &ast {
            assert!(!body.is_empty());
            let loc = body[0].source_location(&index);
            assert_eq!(loc.line, 1);
            assert!(loc.col == 0);
        }
    }

    #[test]
    fn test_multiline_location() {
        let source = "def foo():\n    return 42";
        let index = LineIndex::new(source);
        let ast = parse_module(source).unwrap();

        if let Mod::Module(ModModule { body, .. }) = &ast {
            assert!(!body.is_empty());
            let loc = body[0].source_location(&index);
            assert_eq!(loc.line, 1);
            assert!(loc.end_line >= 2); // Should span multiple lines
        }
    }

    #[test]
    fn test_fallback_location() {
        let source = "x = 1";
        let ast = parse_module(source).unwrap();

        if let Mod::Module(ModModule { body, .. }) = &ast {
            let loc = location_from_range(&body[0]);
            assert!(loc.line > 0);
        }
    }
}

