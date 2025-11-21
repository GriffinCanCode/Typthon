use crate::compiler::types::{Type, Predicate, PredicateExpr, CompareOp, BinOp};
use rustpython_parser::ast::*;
use std::collections::HashMap;

/// Refinement type analyzer for extracting and validating predicates
pub struct RefinementAnalyzer {
    predicates: HashMap<String, Predicate>,
}

impl RefinementAnalyzer {
    pub fn new() -> Self {
        Self {
            predicates: HashMap::new(),
        }
    }

    /// Parse predicate from string expression
    pub fn parse_predicate(&self, expr_str: &str) -> Result<Predicate, String> {
        // Simple predicate parser
        // Supports: value > 0, value < 100, value == 42, etc.

        let parts: Vec<&str> = expr_str.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!("Invalid predicate format: {}", expr_str));
        }

        let left = self.parse_pred_expr(parts[0])?;
        let op = self.parse_compare_op(parts[1])?;
        let right = self.parse_pred_expr(parts[2])?;

        Ok(Predicate::Compare { op, left, right })
    }

    fn parse_pred_expr(&self, s: &str) -> Result<PredicateExpr, String> {
        if s == "value" {
            Ok(PredicateExpr::Value)
        } else if let Ok(n) = s.parse::<i64>() {
            Ok(PredicateExpr::Literal(n))
        } else if s.starts_with("len(") && s.ends_with(')') {
            Ok(PredicateExpr::Property("len".to_string()))
        } else {
            Ok(PredicateExpr::Property(s.to_string()))
        }
    }

    fn parse_compare_op(&self, s: &str) -> Result<CompareOp, String> {
        match s {
            "==" => Ok(CompareOp::Eq),
            "!=" => Ok(CompareOp::Ne),
            "<" => Ok(CompareOp::Lt),
            "<=" => Ok(CompareOp::Le),
            ">" => Ok(CompareOp::Gt),
            ">=" => Ok(CompareOp::Ge),
            _ => Err(format!("Unknown comparison operator: {}", s)),
        }
    }

    /// Extract predicates from function annotations
    pub fn extract_from_annotation(&mut self, func: &StmtFunctionDef) -> HashMap<String, Predicate> {
        let mut predicates = HashMap::new();

        // Look for special decorators or annotations
        for decorator in &func.decorator_list {
            if let Expr::Call(call) = decorator {
                if let Expr::Name(name) = &*call.func {
                    if name.id.as_str() == "refine" && !call.args.is_empty() {
                        if let Expr::Constant(ExprConstant { value: Constant::Str(pred_str), .. }) = &call.args[0] {
                            if let Ok(pred) = self.parse_predicate(pred_str) {
                                predicates.insert(func.name.to_string(), pred);
                            }
                        }
                    }
                }
            }
        }

        predicates
    }

    /// Validate value against predicate at runtime
    pub fn validate(&self, value: &serde_json::Value, predicate: &Predicate) -> bool {
        match predicate {
            Predicate::True => true,

            Predicate::Compare { op, left, right } => {
                let left_val = self.eval_pred_expr(value, left);
                let right_val = self.eval_pred_expr(value, right);

                if let (Some(l), Some(r)) = (left_val, right_val) {
                    self.compare(l, r, op)
                } else {
                    false
                }
            }

            Predicate::And(preds) => preds.iter().all(|p| self.validate(value, p)),
            Predicate::Or(preds) => preds.iter().any(|p| self.validate(value, p)),
            Predicate::Not(pred) => !self.validate(value, pred),
            Predicate::Custom(_) => false, // Cannot validate custom predicates
        }
    }

    fn eval_pred_expr(&self, value: &serde_json::Value, expr: &PredicateExpr) -> Option<i64> {
        match expr {
            PredicateExpr::Value => {
                if let serde_json::Value::Number(n) = value {
                    n.as_i64()
                } else {
                    None
                }
            }
            PredicateExpr::Literal(n) => Some(*n),
            PredicateExpr::Property(prop) => {
                match prop.as_str() {
                    "len" => {
                        if let serde_json::Value::String(s) = value {
                            Some(s.len() as i64)
                        } else if let serde_json::Value::Array(a) = value {
                            Some(a.len() as i64)
                        } else {
                            None
                        }
                    }
                    "abs" => {
                        if let serde_json::Value::Number(n) = value {
                            n.as_i64().map(|x| x.abs())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            PredicateExpr::BinOp(left, op, right) => {
                let l = self.eval_pred_expr(value, left)?;
                let r = self.eval_pred_expr(value, right)?;
                Some(match op {
                    BinOp::Add => l + r,
                    BinOp::Sub => l - r,
                    BinOp::Mul => l * r,
                    BinOp::Div => if r != 0 { l / r } else { return None },
                    BinOp::Mod => if r != 0 { l % r } else { return None },
                })
            }
        }
    }

    fn compare(&self, left: i64, right: i64, op: &CompareOp) -> bool {
        match op {
            CompareOp::Eq => left == right,
            CompareOp::Ne => left != right,
            CompareOp::Lt => left < right,
            CompareOp::Le => left <= right,
            CompareOp::Gt => left > right,
            CompareOp::Ge => left >= right,
        }
    }

    /// Create common refinement types
    pub fn positive_int() -> Type {
        Type::Int.refine(Predicate::Compare {
            op: CompareOp::Gt,
            left: PredicateExpr::Value,
            right: PredicateExpr::Literal(0),
        })
    }

    pub fn negative_int() -> Type {
        Type::Int.refine(Predicate::Compare {
            op: CompareOp::Lt,
            left: PredicateExpr::Value,
            right: PredicateExpr::Literal(0),
        })
    }

    pub fn non_empty_str() -> Type {
        Type::Str.refine(Predicate::Compare {
            op: CompareOp::Gt,
            left: PredicateExpr::Property("len".to_string()),
            right: PredicateExpr::Literal(0),
        })
    }

    pub fn bounded_int(min: i64, max: i64) -> Type {
        let lower = Predicate::Compare {
            op: CompareOp::Ge,
            left: PredicateExpr::Value,
            right: PredicateExpr::Literal(min),
        };
        let upper = Predicate::Compare {
            op: CompareOp::Le,
            left: PredicateExpr::Value,
            right: PredicateExpr::Literal(max),
        };
        Type::Int.refine(lower.and(upper))
    }
}

impl Default for RefinementAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Common refinement type constructors
pub mod refinements {
    use super::*;

    /// Positive integers: x > 0
    pub fn positive() -> Type {
        RefinementAnalyzer::positive_int()
    }

    /// Negative integers: x < 0
    pub fn negative() -> Type {
        RefinementAnalyzer::negative_int()
    }

    /// Non-negative integers: x >= 0
    pub fn non_negative() -> Type {
        RefinementAnalyzer::bounded_int(0, i64::MAX)
    }

    /// Natural numbers: x >= 1
    pub fn natural() -> Type {
        RefinementAnalyzer::bounded_int(1, i64::MAX)
    }

    /// Non-empty string
    pub fn non_empty_str() -> Type {
        RefinementAnalyzer::non_empty_str()
    }

    /// Bounded integer range
    pub fn range(min: i64, max: i64) -> Type {
        RefinementAnalyzer::bounded_int(min, max)
    }

    /// Even number: x % 2 == 0
    pub fn even() -> Type {
        Type::Int.refine(Predicate::Compare {
            op: CompareOp::Eq,
            left: PredicateExpr::BinOp(
                Box::new(PredicateExpr::Value),
                BinOp::Mod,
                Box::new(PredicateExpr::Literal(2)),
            ),
            right: PredicateExpr::Literal(0),
        })
    }

    /// Odd number: x % 2 != 0
    pub fn odd() -> Type {
        Type::Int.refine(Predicate::Compare {
            op: CompareOp::Ne,
            left: PredicateExpr::BinOp(
                Box::new(PredicateExpr::Value),
                BinOp::Mod,
                Box::new(PredicateExpr::Literal(2)),
            ),
            right: PredicateExpr::Literal(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_predicate() {
        let analyzer = RefinementAnalyzer::new();
        let pred = analyzer.parse_predicate("value > 0").unwrap();

        match pred {
            Predicate::Compare { op, .. } => assert_eq!(op, CompareOp::Gt),
            _ => panic!("Expected Compare predicate"),
        }
    }

    #[test]
    fn test_validate_positive() {
        let analyzer = RefinementAnalyzer::new();
        let pred = Predicate::Compare {
            op: CompareOp::Gt,
            left: PredicateExpr::Value,
            right: PredicateExpr::Literal(0),
        };

        assert!(analyzer.validate(&serde_json::json!(5), &pred));
        assert!(!analyzer.validate(&serde_json::json!(-5), &pred));
        assert!(!analyzer.validate(&serde_json::json!(0), &pred));
    }

    #[test]
    fn test_bounded_int() {
        let ty = refinements::range(0, 100);
        match ty {
            Type::Refinement(base, _) => {
                assert_eq!(*base, Type::Int);
            }
            _ => panic!("Expected Refinement type"),
        }
    }
}

