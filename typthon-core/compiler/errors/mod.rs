use crate::compiler::types::Type;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl SourceLocation {
    pub fn new(line: usize, col: usize, end_line: usize, end_col: usize) -> Self {
        Self { line, col, end_line, end_col }
    }

    pub fn from_range(start: (usize, usize), end: (usize, usize)) -> Self {
        Self::new(start.0, start.1, end.0, end.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    TypeMismatch { expected: String, found: String },
    UndefinedVariable { name: String },
    UndefinedFunction { name: String },
    InvalidArgCount { expected: usize, found: usize },
    InvalidArgType { param: String, expected: String, found: String },
    InvalidReturnType { expected: String, found: String },
    NonCallable { ty: String },
    InvalidSubscript { container: String, key: String },
    InvalidAttribute { ty: String, attr: String },
    CircularDependency { chain: Vec<String> },
    ConstraintViolation { constraint: String, value: String },
    VarianceError { context: String },
    InfiniteType { var: String, ty: String },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            Self::UndefinedVariable { name } => {
                write!(f, "Undefined variable: {}", name)
            }
            Self::UndefinedFunction { name } => {
                write!(f, "Undefined function: {}", name)
            }
            Self::InvalidArgCount { expected, found } => {
                write!(f, "Invalid argument count: expected {}, found {}", expected, found)
            }
            Self::InvalidArgType { param, expected, found } => {
                write!(f, "Invalid type for parameter '{}': expected {}, found {}", param, expected, found)
            }
            Self::InvalidReturnType { expected, found } => {
                write!(f, "Invalid return type: expected {}, found {}", expected, found)
            }
            Self::NonCallable { ty } => {
                write!(f, "Type {} is not callable", ty)
            }
            Self::InvalidSubscript { container, key } => {
                write!(f, "Cannot subscript {} with {}", container, key)
            }
            Self::InvalidAttribute { ty, attr } => {
                write!(f, "Type {} has no attribute '{}'", ty, attr)
            }
            Self::CircularDependency { chain } => {
                write!(f, "Circular dependency: {}", chain.join(" -> "))
            }
            Self::ConstraintViolation { constraint, value } => {
                write!(f, "Constraint '{}' violated by value: {}", constraint, value)
            }
            Self::VarianceError { context } => {
                write!(f, "Variance error: {}", context)
            }
            Self::InfiniteType { var, ty } => {
                write!(f, "Infinite type: {} = {}", var, ty)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub kind: ErrorKind,
    pub location: SourceLocation,
    pub file: String,
    pub suggestions: Vec<String>,
}

impl TypeError {
    pub fn new(kind: ErrorKind, location: SourceLocation) -> Self {
        Self {
            kind,
            location,
            file: String::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = file;
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn type_mismatch(expected: Type, found: Type, location: SourceLocation) -> Self {
        let mut error = Self::new(
            ErrorKind::TypeMismatch {
                expected: expected.to_string(),
                found: found.to_string(),
            },
            location,
        );

        // Add intelligent suggestions
        if let (Type::Int, Type::Float) = (&expected, &found) {
            error = error.with_suggestion("Use int() to convert float to int".to_string());
        } else if let (Type::Str, Type::Int) = (&expected, &found) {
            error = error.with_suggestion("Use str() to convert int to string".to_string());
        } else if expected.is_subtype(&found) {
            error = error.with_suggestion(format!("Note: {} is a supertype of {}", found, expected));
        }

        error
    }

    pub fn undefined_variable(name: String, location: SourceLocation, similar: Vec<String>) -> Self {
        let mut error = Self::new(
            ErrorKind::UndefinedVariable { name: name.clone() },
            location,
        );

        if !similar.is_empty() {
            let suggestions = similar.iter()
                .take(3)
                .map(|s| format!("Did you mean '{}'?", s))
                .collect();
            error = error.with_suggestions(suggestions);
        }

        error
    }

    pub fn invalid_arg_count(expected: usize, found: usize, location: SourceLocation) -> Self {
        Self::new(ErrorKind::InvalidArgCount { expected, found }, location)
    }

    pub fn invalid_arg_type(param: String, expected: Type, found: Type, location: SourceLocation) -> Self {
        Self::new(
            ErrorKind::InvalidArgType {
                param,
                expected: expected.to_string(),
                found: found.to_string(),
            },
            location,
        )
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.file.is_empty() {
            write!(f, "{}:", self.file)?;
        }
        write!(
            f,
            "{}:{}: {}",
            self.location.line,
            self.location.col,
            self.kind
        )?;

        if !self.suggestions.is_empty() {
            write!(f, "\n")?;
            for suggestion in &self.suggestions {
                write!(f, "  hint: {}\n", suggestion)?;
            }
        }

        Ok(())
    }
}

/// Error collector for gathering multiple errors during type checking
pub struct ErrorCollector {
    errors: Vec<TypeError>,
    max_errors: usize,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            max_errors: 100,
        }
    }

    pub fn with_max(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
        }
    }

    pub fn add(&mut self, error: TypeError) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error);
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    pub fn into_errors(self) -> Vec<TypeError> {
        self.errors
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute Levenshtein distance for "did you mean" suggestions
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len { matrix[i][0] = i; }
    for j in 0..=b_len { matrix[0][j] = j; }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

/// Find similar names for "did you mean" suggestions
pub fn find_similar_names(target: &str, candidates: &[String], max_distance: usize) -> Vec<String> {
    let mut results: Vec<(String, usize)> = candidates
        .iter()
        .map(|c| (c.clone(), levenshtein_distance(target, c)))
        .filter(|(_, dist)| *dist <= max_distance && *dist > 0)
        .collect();

    results.sort_by_key(|(_, dist)| *dist);
    results.into_iter().map(|(name, _)| name).collect()
}
