use rustpython_parser::{parse, Mode};
use rustpython_parser::ast::{Mod, ModExpression, Expr};

pub type ParseError = String;

pub fn parse_module(source: &str) -> Result<Mod, ParseError> {
    parse(source, Mode::Module, "<string>")
        .map_err(|e| format!("Parse error: {}", e))
}

pub fn parse_expression(source: &str) -> Result<Expr, ParseError> {
    match parse(source, Mode::Expression, "<string>") {
        Ok(Mod::Expression(ModExpression { body, .. })) => Ok(*body),
        Ok(_) => Err("Expected expression".to_string()),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let source = "x = 1 + 2";
        assert!(parse_module(source).is_ok());
    }

    #[test]
    fn test_parse_function() {
        let source = r#"
def add(x: int, y: int) -> int:
    return x + y
"#;
        assert!(parse_module(source).is_ok());
    }
}
