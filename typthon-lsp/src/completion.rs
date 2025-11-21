/*!
Code completion support for LSP.
*/

use tower_lsp::lsp_types::CompletionItemKind;

/// Built-in Python keywords
pub const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break",
    "class", "continue", "def", "del", "elif", "else", "except", "finally",
    "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal",
    "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
];

/// Built-in Python types
pub const PYTHON_TYPES: &[&str] = &[
    "int", "str", "float", "bool", "list", "dict", "tuple", "set", "frozenset",
    "bytes", "bytearray", "complex", "object", "type", "None",
];

/// Built-in Python functions
pub const PYTHON_BUILTINS: &[(&str, &str)] = &[
    ("abs", "abs(x) -> number"),
    ("all", "all(iterable) -> bool"),
    ("any", "any(iterable) -> bool"),
    ("enumerate", "enumerate(iterable, start=0) -> iterator"),
    ("filter", "filter(function, iterable) -> iterator"),
    ("isinstance", "isinstance(object, classinfo) -> bool"),
    ("len", "len(obj) -> int"),
    ("map", "map(function, iterable) -> iterator"),
    ("max", "max(iterable) -> value"),
    ("min", "min(iterable) -> value"),
    ("print", "print(*args, **kwargs) -> None"),
    ("range", "range(stop) -> range object"),
    ("reversed", "reversed(sequence) -> iterator"),
    ("sorted", "sorted(iterable) -> list"),
    ("sum", "sum(iterable, start=0) -> number"),
    ("zip", "zip(*iterables) -> iterator"),
];

/// Get completion kind for identifier
pub fn get_completion_kind(name: &str) -> CompletionItemKind {
    if PYTHON_KEYWORDS.contains(&name) {
        CompletionItemKind::KEYWORD
    } else if PYTHON_TYPES.contains(&name) {
        CompletionItemKind::CLASS
    } else if PYTHON_BUILTINS.iter().any(|(builtin, _)| *builtin == name) {
        CompletionItemKind::FUNCTION
    } else {
        CompletionItemKind::VARIABLE
    }
}

