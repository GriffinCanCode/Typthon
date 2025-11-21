/*!
Document analyzer for LSP features.

Provides type checking, completion, and navigation features.
*/

use rustpython_parser::parse;
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
        match parse(content, "<string>") {
            Ok(_ast) => {
                // TODO: Integrate with typthon-core type checker
                // For now, just validate syntax
            }
            Err(err) => {
                errors.push(TypeError {
                    line: err.location.row().to_zero_indexed(),
                    col: err.location.column().to_zero_indexed(),
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
        let hover_text = match word {
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
        if line >= lines.len() || col == 0 {
            return completions;
        }

        let line_content = lines[line];
        if col > 0 && line_content.chars().nth(col - 1) == Some('.') {
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
        // TODO: Implement proper definition lookup using AST
        let lines: Vec<&str> = content.lines().collect();
        if line >= lines.len() {
            return None;
        }

        let line_content = lines[line];
        let word = extract_word_at_position(line_content, col);
        if word.is_empty() {
            return None;
        }

        // Search for definition (simple pattern matching for now)
        for (idx, line_text) in lines.iter().enumerate() {
            if line_text.contains(&format!("def {}(", word)) ||
               line_text.contains(&format!("class {}:", word)) {
                return Some(DefinitionLocation {
                    line: idx,
                    col: line_text.find(word).unwrap_or(0),
                    length: word.len(),
                });
            }
        }

        None
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
}

