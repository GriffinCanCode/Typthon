use rustpython_parser::{parse, Mode};
use rustpython_parser::ast::{Mod, ModExpression, Expr};
use tracing::{debug, error, info, instrument};

pub type ParseError = String;

#[instrument(skip(source), fields(source_len = source.len()))]
pub fn parse_module(source: &str) -> Result<Mod, ParseError> {
    debug!("Parsing module");
    match parse(source, Mode::Module, "<string>") {
        Ok(ast) => {
            info!("Successfully parsed module");
            Ok(ast)
        }
        Err(e) => {
            error!(error = %e, "Failed to parse module");
            Err(format!("Parse error: {}", e))
        }
    }
}

#[instrument(skip(source), fields(source_len = source.len()))]
pub fn parse_expression(source: &str) -> Result<Expr, ParseError> {
    debug!("Parsing expression");
    match parse(source, Mode::Expression, "<string>") {
        Ok(Mod::Expression(ModExpression { body, .. })) => {
            info!("Successfully parsed expression");
            Ok(*body)
        }
        Ok(_) => {
            error!("Expected expression, got different AST node");
            Err("Expected expression".to_string())
        }
        Err(e) => {
            error!(error = %e, "Failed to parse expression");
            Err(format!("Parse error: {}", e))
        }
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
