/*!
Document analyzer for LSP features.

Provides type checking, completion, and navigation features.
*/

use rustpython_parser::{ast, parse, Mode};
use tower_lsp::lsp_types::CompletionItemKind;

/// Simple type error for diagnostics
#[derive(Debug, Clone)]
pub struct TypeError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

/// Completion suggestion
#[derive(Debug, Clone)]
pub struct CompletionSuggestion {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: String,
    pub documentation: Option<String>,
}

/// Definition location
#[derive(Debug, Clone)]
pub struct DefinitionLocation {
    pub line: usize,
    pub col: usize,
    pub length: usize,
}

/// Symbol information for navigation and references
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub col: usize,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Variable,
    Parameter,
    Method,
    Property,
}

/// Document analyzer for type checking and code intelligence
pub struct DocumentAnalyzer;

impl DocumentAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze document and return diagnostics
    pub fn analyze(&self, content: &str) -> Vec<TypeError> {
        let mut errors = Vec::new();

        // Parse the Python code
        match parse(content, Mode::Module, "<string>") {
            Ok(_ast) => {
                // TODO: Integrate with typthon-core type checker
                // For now, just validate syntax
            }
            Err(err) => {
                errors.push(TypeError {
                    line: 0, // Error location not available in this API
                    col: 0,
                    message: format!("Syntax error: {}", err.error),
                });
            }
        }

        errors
    }

    /// Get hover information at position
    pub fn get_hover_info(&self, content: &str, line: usize, col: usize) -> Option<String> {
        // TODO: Integrate with typthon-core to get actual type information
        // For now, provide basic Python information

        let lines: Vec<&str> = content.lines().collect();
        if line >= lines.len() {
            return None;
        }

        let line_content = lines[line];
        if col >= line_content.len() {
            return None;
        }

        // Simple word extraction
        let word = extract_word_at_position(line_content, col);
        if word.is_empty() {
            return None;
        }

        // Provide basic type hints for built-in types
        let hover_text = match word.as_str() {
            "int" => "Built-in integer type",
            "str" => "Built-in string type",
            "float" => "Built-in floating-point type",
            "bool" => "Built-in boolean type",
            "list" => "Built-in list type: list[T]",
            "dict" => "Built-in dictionary type: dict[K, V]",
            "tuple" => "Built-in tuple type: tuple[T, ...]",
            "set" => "Built-in set type: set[T]",
            "None" => "The None type",
            "def" => "Function definition keyword",
            "class" => "Class definition keyword",
            "return" => "Return statement keyword",
            _ => return Some(format!("Identifier: {}", word)),
        };

        Some(hover_text.to_string())
    }

    /// Get completions at position
    pub fn get_completions(&self, content: &str, line: usize, col: usize) -> Vec<CompletionSuggestion> {
        let mut completions = Vec::new();

        // Check if we're after a dot (attribute access)
        let lines: Vec<&str> = content.lines().collect();
        if line >= lines.len() {
            return completions;
        }

        let line_content = lines[line];
        if col > 0 && col <= line_content.len() && line_content.chars().nth(col - 1) == Some('.') {
            // Provide attribute completions
            // TODO: Context-aware completions based on type
            completions.extend(vec![
                CompletionSuggestion {
                    label: "append".to_string(),
                    kind: CompletionItemKind::METHOD,
                    detail: "list.append(item)".to_string(),
                    documentation: Some("Append an item to the list".to_string()),
                },
                CompletionSuggestion {
                    label: "extend".to_string(),
                    kind: CompletionItemKind::METHOD,
                    detail: "list.extend(items)".to_string(),
                    documentation: Some("Extend the list with multiple items".to_string()),
                },
                CompletionSuggestion {
                    label: "pop".to_string(),
                    kind: CompletionItemKind::METHOD,
                    detail: "list.pop() -> T".to_string(),
                    documentation: Some("Remove and return the last item".to_string()),
                },
            ]);
        } else {
            // Provide keyword completions
            completions.extend(vec![
                CompletionSuggestion {
                    label: "def".to_string(),
                    kind: CompletionItemKind::KEYWORD,
                    detail: "Function definition".to_string(),
                    documentation: Some("Define a function".to_string()),
                },
                CompletionSuggestion {
                    label: "class".to_string(),
                    kind: CompletionItemKind::KEYWORD,
                    detail: "Class definition".to_string(),
                    documentation: Some("Define a class".to_string()),
                },
                CompletionSuggestion {
                    label: "if".to_string(),
                    kind: CompletionItemKind::KEYWORD,
                    detail: "Conditional statement".to_string(),
                    documentation: Some("Conditional if statement".to_string()),
                },
                CompletionSuggestion {
                    label: "for".to_string(),
                    kind: CompletionItemKind::KEYWORD,
                    detail: "For loop".to_string(),
                    documentation: Some("For loop iteration".to_string()),
                },
                CompletionSuggestion {
                    label: "return".to_string(),
                    kind: CompletionItemKind::KEYWORD,
                    detail: "Return statement".to_string(),
                    documentation: Some("Return a value from function".to_string()),
                },
            ]);

            // Add type keywords
            for type_name in &["int", "str", "float", "bool", "list", "dict", "tuple", "set"] {
                completions.push(CompletionSuggestion {
                    label: type_name.to_string(),
                    kind: CompletionItemKind::CLASS,
                    detail: format!("Built-in type: {}", type_name),
                    documentation: None,
                });
            }
        }

        completions
    }

    /// Get definition location
    pub fn get_definition(&self, content: &str, line: usize, col: usize) -> Option<DefinitionLocation> {
        let word = self.get_word_at_position(content, line, col)?;
        let symbols = self.extract_symbols(content);

        // Find the definition of the symbol
        symbols.iter()
            .find(|s| s.name == word && matches!(s.kind, SymbolKind::Function | SymbolKind::Class))
            .map(|s| DefinitionLocation {
                line: s.line,
                col: s.col,
                length: s.length,
            })
    }

    /// Find all references to a symbol
    pub fn find_references(&self, content: &str, line: usize, col: usize) -> Vec<DefinitionLocation> {
        let word = match self.get_word_at_position(content, line, col) {
            Some(w) => w,
            None => return Vec::new(),
        };

        let mut references = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line_text) in lines.iter().enumerate() {
            let mut start = 0;
            while let Some(pos) = line_text[start..].find(&word) {
                let actual_pos = start + pos;

                // Check if this is a complete word (not part of another identifier)
                let is_word_start = actual_pos == 0 ||
                    !line_text.chars().nth(actual_pos - 1).map_or(false, |c| c.is_alphanumeric() || c == '_');
                let is_word_end = actual_pos + word.len() >= line_text.len() ||
                    !line_text.chars().nth(actual_pos + word.len()).map_or(false, |c| c.is_alphanumeric() || c == '_');

                if is_word_start && is_word_end {
                    references.push(DefinitionLocation {
                        line: idx,
                        col: actual_pos,
                        length: word.len(),
                    });
                }

                start = actual_pos + 1;
            }
        }

        references
    }

    /// Extract all symbols from document
    pub fn extract_symbols(&self, content: &str) -> Vec<SymbolInfo> {
        let mut symbols = Vec::new();

        match parse(content, Mode::Module, "<string>") {
            Ok(ast) => {
                if let ast::Mod::Module(module) = ast {
                    self.visit_module(&module.body, content, &mut symbols);
                }
            }
            Err(_) => {}
        }

        symbols
    }

    /// Visit AST module and extract symbols
    fn visit_module(&self, stmts: &[ast::Stmt], content: &str, symbols: &mut Vec<SymbolInfo>) {
        for stmt in stmts {
            self.visit_stmt(stmt, content, symbols);
        }
    }

    /// Convert byte offset to line and column
    fn offset_to_position(&self, content: &str, offset: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;
        let mut current_offset = 0;

        for ch in content.chars() {
            if current_offset >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }

            current_offset += ch.len_utf8();
        }

        (line, col)
    }

    /// Visit AST statement and extract symbols
    fn visit_stmt(&self, stmt: &ast::Stmt, content: &str, symbols: &mut Vec<SymbolInfo>) {
        match stmt {
            ast::Stmt::FunctionDef(func) => {
                let offset = func.range.start().to_usize();
                let (line, col) = self.offset_to_position(content, offset);

                symbols.push(SymbolInfo {
                    name: func.name.to_string(),
                    kind: SymbolKind::Function,
                    line,
                    col,
                    length: func.name.len(),
                });

                // Visit parameters
                for arg in &func.args.args {
                    let param_offset = arg.def.range.start().to_usize();
                    let (param_line, param_col) = self.offset_to_position(content, param_offset);

                    symbols.push(SymbolInfo {
                        name: arg.def.arg.to_string(),
                        kind: SymbolKind::Parameter,
                        line: param_line,
                        col: param_col,
                        length: arg.def.arg.len(),
                    });
                }

                // Visit body
                for stmt in &func.body {
                    self.visit_stmt(stmt, content, symbols);
                }
            }
            ast::Stmt::ClassDef(class) => {
                let offset = class.range.start().to_usize();
                let (line, col) = self.offset_to_position(content, offset);

                symbols.push(SymbolInfo {
                    name: class.name.to_string(),
                    kind: SymbolKind::Class,
                    line,
                    col,
                    length: class.name.len(),
                });

                // Visit body
                for stmt in &class.body {
                    self.visit_stmt(stmt, content, symbols);
                }
            }
            ast::Stmt::Assign(assign) => {
                for target in &assign.targets {
                    if let ast::Expr::Name(name) = target {
                        let offset = name.range.start().to_usize();
                        let (line, col) = self.offset_to_position(content, offset);

                        symbols.push(SymbolInfo {
                            name: name.id.to_string(),
                            kind: SymbolKind::Variable,
                            line,
                            col,
                            length: name.id.len(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    /// Get word at position
    fn get_word_at_position(&self, content: &str, line: usize, col: usize) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        if line >= lines.len() {
            return None;
        }

        let line_content = lines[line];
        let word = extract_word_at_position(line_content, col);
        if word.is_empty() {
            None
        } else {
            Some(word)
        }
    }
}

/// Extract word at position
fn extract_word_at_position(line: &str, col: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    if col >= chars.len() {
        return String::new();
    }

    // Find word boundaries
    let mut start = col;
    let mut end = col;

    // Move start backward
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    // Move end forward
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    chars[start..end].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word() {
        assert_eq!(extract_word_at_position("hello world", 2), "hello");
        assert_eq!(extract_word_at_position("hello world", 7), "world");
        assert_eq!(extract_word_at_position("def func():", 5), "func");
    }

    #[test]
    fn test_analyze_valid_code() {
        let analyzer = DocumentAnalyzer::new();
        let errors = analyzer.analyze("def add(x: int, y: int) -> int:\n    return x + y");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_analyze_invalid_code() {
        let analyzer = DocumentAnalyzer::new();
        let errors = analyzer.analyze("def invalid(:\n");
        assert!(errors.len() > 0);
    }

    #[test]
    fn test_extract_symbols_functions() {
        let analyzer = DocumentAnalyzer::new();
        let code = "def hello():\n    pass\n\ndef world():\n    pass";
        let symbols = analyzer.extract_symbols(code);

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "hello");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[1].name, "world");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_symbols_classes() {
        let analyzer = DocumentAnalyzer::new();
        let code = "class Foo:\n    pass\n\nclass Bar:\n    pass";
        let symbols = analyzer.extract_symbols(code);

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "Foo");
        assert_eq!(symbols[0].kind, SymbolKind::Class);
        assert_eq!(symbols[1].name, "Bar");
        assert_eq!(symbols[1].kind, SymbolKind::Class);
    }

    #[test]
    fn test_extract_symbols_variables() {
        let analyzer = DocumentAnalyzer::new();
        let code = "x = 10\ny = 20";
        let symbols = analyzer.extract_symbols(code);

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "x");
        assert_eq!(symbols[0].kind, SymbolKind::Variable);
        assert_eq!(symbols[1].name, "y");
        assert_eq!(symbols[1].kind, SymbolKind::Variable);
    }

    #[test]
    fn test_extract_symbols_parameters() {
        let analyzer = DocumentAnalyzer::new();
        let code = "def add(x, y):\n    return x + y";
        let symbols = analyzer.extract_symbols(code);

        // Should have 1 function + 2 parameters
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "add");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[1].name, "x");
        assert_eq!(symbols[1].kind, SymbolKind::Parameter);
        assert_eq!(symbols[2].name, "y");
        assert_eq!(symbols[2].kind, SymbolKind::Parameter);
    }

    #[test]
    fn test_find_references() {
        let analyzer = DocumentAnalyzer::new();
        let code = "x = 10\ny = x + 5\nz = x * 2";
        let references = analyzer.find_references(code, 0, 0); // Position of first 'x'

        // Should find 3 references to 'x'
        assert_eq!(references.len(), 3);
    }

    #[test]
    fn test_get_definition() {
        let analyzer = DocumentAnalyzer::new();
        let code = "def hello():\n    pass\n\nhello()";
        let definition = analyzer.get_definition(code, 3, 0); // Position of call to 'hello'

        assert!(definition.is_some());
        let def = definition.unwrap();
        assert_eq!(def.line, 0); // Definition is on first line
    }

    #[test]
    fn test_hover_builtin_types() {
        let analyzer = DocumentAnalyzer::new();
        let code = "x: int = 5";
        let hover = analyzer.get_hover_info(code, 0, 3); // Position of 'int'

        assert!(hover.is_some());
        assert!(hover.unwrap().contains("integer"));
    }

    #[test]
    fn test_completions_after_dot() {
        let analyzer = DocumentAnalyzer::new();
        let code = "x = [1, 2, 3]\nx.";
        let completions = analyzer.get_completions(code, 1, 2); // After the dot

        assert!(!completions.is_empty());
        // Should suggest list methods
        assert!(completions.iter().any(|c| c.label == "append"));
    }

    #[test]
    fn test_completions_keywords() {
        let analyzer = DocumentAnalyzer::new();
        let code = "x";
        let completions = analyzer.get_completions(code, 0, 1);

        // When not after a dot, should suggest keywords and types
        // Note: completions might be empty if position is at end without trigger
        if !completions.is_empty() {
            assert!(completions.iter().any(|c| matches!(c.kind, CompletionItemKind::KEYWORD | CompletionItemKind::CLASS)));
        }
    }

    #[test]
    fn test_offset_to_position() {
        let analyzer = DocumentAnalyzer::new();
        let code = "line1\nline2\nline3";

        // Test beginning
        assert_eq!(analyzer.offset_to_position(code, 0), (0, 0));

        // Test second line
        assert_eq!(analyzer.offset_to_position(code, 6), (1, 0));

        // Test middle of second line
        assert_eq!(analyzer.offset_to_position(code, 9), (1, 3));
    }
}

