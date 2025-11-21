use crate::compiler::types::{Type, TypeContext};
use crate::compiler::errors::{TypeError, ErrorCollector, SourceLocation};
use crate::compiler::analysis::inference::InferenceEngine;
use crate::compiler::ast::{LineIndex, SourceLocationExt};
use rustpython_parser::ast::{Expr, Constant, Operator, Comprehension};
use std::sync::Arc;
use num_traits::ToPrimitive;

/// Bidirectional type inference: combines bottom-up (synthesis) and top-down (checking)
pub struct BiInfer {
    ctx: Arc<TypeContext>,
    engine: InferenceEngine,
    errors: ErrorCollector,
    line_index: Arc<LineIndex>,
}

impl BiInfer {
    pub fn new(ctx: Arc<TypeContext>) -> Self {
        Self::with_source(ctx, "")
    }

    pub fn with_source(ctx: Arc<TypeContext>, source: &str) -> Self {
        Self {
            ctx,
            engine: InferenceEngine::new(),
            errors: ErrorCollector::new(),
            line_index: Arc::new(LineIndex::new(source)),
        }
    }

    pub fn errors(&self) -> &[TypeError] {
        self.errors.errors()
    }

    pub fn into_errors(self) -> Vec<TypeError> {
        self.errors.into_errors()
    }

    /// Synthesize type (bottom-up): infer type from expression
    pub fn synthesize(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Constant(c) => self.synth_constant(c),
            Expr::Name(n) => self.synth_name(n),
            Expr::BinOp(b) => self.synth_binop(b),
            Expr::UnaryOp(u) => self.synth_unaryop(u),
            Expr::BoolOp(b) => self.synth_boolop(b),
            Expr::Compare(c) => self.synth_compare(c),
            Expr::Call(c) => self.synth_call(c),
            Expr::List(l) => self.synth_list(l),
            Expr::Tuple(t) => self.synth_tuple(t),
            Expr::Dict(d) => self.synth_dict(d),
            Expr::Set(s) => self.synth_set(s),
            Expr::ListComp(lc) => self.synth_list_comp(lc),
            Expr::DictComp(dc) => self.synth_dict_comp(dc),
            Expr::SetComp(sc) => self.synth_set_comp(sc),
            Expr::GeneratorExp(g) => self.synth_generator(g),
            Expr::Lambda(l) => self.synth_lambda(l),
            Expr::IfExp(i) => self.synth_if_expr(i),
            Expr::Subscript(s) => self.synth_subscript(s),
            Expr::Attribute(a) => self.synth_attribute(a),
            Expr::Slice(s) => self.synth_slice(s),
            Expr::NamedExpr(n) => self.synthesize(&n.value),
            Expr::Starred(s) => self.synthesize(&s.value),
            Expr::Await(a) => self.synthesize(&a.value),
            Expr::Yield(y) => y.value.as_ref().map_or(Type::None, |v| self.synthesize(v)),
            Expr::YieldFrom(y) => self.synthesize(&y.value),
            Expr::FormattedValue(f) => {
                self.synthesize(&f.value);
                Type::Str
            }
            Expr::JoinedStr(_) => Type::Str,
        }
    }

    /// Check type (top-down): verify expression has expected type
    pub fn check(&mut self, expr: &Expr, expected: &Type) -> bool {
        let synthesized = self.synthesize(expr);
        if !synthesized.is_subtype(expected) {
            self.errors.add(TypeError::type_mismatch(
                expected.clone(),
                synthesized,
                expr.source_location(&self.line_index),
            ));
            false
        } else {
            true
        }
    }

    fn synth_constant(&mut self, constant: &rustpython_parser::ast::ExprConstant) -> Type {
        let constant_type = |c: &Constant| match c {
            Constant::None => Type::None,
            Constant::Bool(_) => Type::Bool,
            Constant::Int(_) => Type::Int,
            Constant::Float(_) => Type::Float,
            Constant::Str(_) => Type::Str,
            Constant::Bytes(_) => Type::Bytes,
            _ => Type::Any,
        };

        match &constant.value {
            Constant::Tuple(items) => Type::Tuple(items.iter().map(constant_type).collect()),
            c => constant_type(c),
        }
    }

    fn synth_name(&mut self, name: &rustpython_parser::ast::ExprName) -> Type {
        self.ctx.get_type(&name.id).unwrap_or_else(|| {
            let loc = Expr::Name(name.clone()).source_location(&self.line_index);
            self.errors.add(TypeError::undefined_variable(
                name.id.to_string(),
                loc,
                vec![],
            ));
            self.ctx.fresh_var()
        })
    }

    fn synth_binop(&mut self, binop: &rustpython_parser::ast::ExprBinOp) -> Type {
        let (left, right) = (self.synthesize(&binop.left), self.synthesize(&binop.right));

        use Operator::*;
        match (binop.op, &left, &right) {
            // Numeric operations
            (Add | Sub | Mult | Div | Mod | Pow | FloorDiv, Type::Int, Type::Int) => Type::Int,
            (Add | Sub | Mult | Div | Mod | Pow | FloorDiv, Type::Float, _) |
            (Add | Sub | Mult | Div | Mod | Pow | FloorDiv, _, Type::Float) => Type::Float,
            (Add, Type::Str, Type::Str) => Type::Str,
            (Add, Type::List(a), Type::List(b)) => Type::List(Box::new(Type::union(vec![*a.clone(), *b.clone()]))),
            // Bitwise operations
            (BitOr | BitXor | BitAnd | LShift | RShift, Type::Int, Type::Int) => Type::Int,
            _ => Type::Any,
        }
    }

    fn synth_unaryop(&mut self, unary: &rustpython_parser::ast::ExprUnaryOp) -> Type {
        use rustpython_parser::ast::UnaryOp::*;
        let operand = self.synthesize(&unary.operand);
        match (unary.op, &operand) {
            (Not, _) => Type::Bool,
            (UAdd | USub, _) => operand,
            (Invert, Type::Int) => Type::Int,
            (Invert, _) => Type::Any,
        }
    }

    fn synth_boolop(&mut self, boolop: &rustpython_parser::ast::ExprBoolOp) -> Type {
        let types: Vec<Type> = boolop.values.iter().map(|e| self.synthesize(e)).collect();
        Type::union(types)
    }

    fn synth_compare(&mut self, _compare: &rustpython_parser::ast::ExprCompare) -> Type {
        Type::Bool
    }

    fn synth_call(&mut self, call: &rustpython_parser::ast::ExprCall) -> Type {
        let func_ty = self.synthesize(&call.func);

        match func_ty {
            Type::Function(params, ret) => {
                // Check argument count
                if params.len() != call.args.len() {
                    self.errors.add(TypeError::invalid_arg_count(
                        params.len(),
                        call.args.len(),
                        SourceLocation::new(0, 0, 0, 0),
                    ));
                }

                // Check argument types
                for (i, (arg, param_ty)) in call.args.iter().zip(params.iter()).enumerate() {
                    let arg_ty = self.synthesize(arg);
                    if !arg_ty.is_subtype(param_ty) {
                        self.errors.add(TypeError::invalid_arg_type(
                            format!("arg{}", i),
                            param_ty.clone(),
                            arg_ty,
                            SourceLocation::new(0, 0, 0, 0),
                        ));
                    }
                }

                *ret
            }
            Type::Class(name) => Type::Class(name), // Constructor call
            _ => {
                // Try to infer from builtins
                if let Expr::Name(n) = &*call.func {
                    self.infer_builtin_call(&n.id, &call.args)
                } else {
                    self.ctx.fresh_var()
                }
            }
        }
    }

    fn infer_builtin_call(&mut self, name: &str, args: &[Expr]) -> Type {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "str" => Type::Str,
            "bool" => Type::Bool,
            "list" => Type::List(Box::new(
                args.first().map(|a| self.synthesize(a)).unwrap_or_else(|| self.ctx.fresh_var())
            )),
            "dict" => Type::Dict(Box::new(self.ctx.fresh_var()), Box::new(self.ctx.fresh_var())),
            "set" => Type::Set(Box::new(self.ctx.fresh_var())),
            "tuple" => Type::Tuple(args.iter().map(|a| self.synthesize(a)).collect()),
            "len" => Type::Int,
            "range" | "enumerate" | "zip" | "map" | "filter" => Type::Class(name.to_string()),
            _ => self.ctx.fresh_var(),
        }
    }

    fn synth_list(&mut self, list: &rustpython_parser::ast::ExprList) -> Type {
        Type::List(Box::new(if list.elts.is_empty() {
            self.ctx.fresh_var()
        } else {
            Type::union(list.elts.iter().map(|e| self.synthesize(e)).collect())
        }))
    }

    fn synth_tuple(&mut self, tuple: &rustpython_parser::ast::ExprTuple) -> Type {
        Type::Tuple(tuple.elts.iter().map(|e| self.synthesize(e)).collect())
    }

    fn synth_dict(&mut self, dict: &rustpython_parser::ast::ExprDict) -> Type {
        let key_types: Vec<_> = dict.keys.iter().filter_map(|k| k.as_ref().map(|e| self.synthesize(e))).collect();
        let value_types: Vec<_> = dict.values.iter().map(|v| self.synthesize(v)).collect();

        let key_ty = if key_types.is_empty() { self.ctx.fresh_var() } else { Type::union(key_types) };
        let val_ty = if value_types.is_empty() { self.ctx.fresh_var() } else { Type::union(value_types) };

        Type::Dict(Box::new(key_ty), Box::new(val_ty))
    }

    fn synth_set(&mut self, set: &rustpython_parser::ast::ExprSet) -> Type {
        Type::Set(Box::new(Type::union(set.elts.iter().map(|e| self.synthesize(e)).collect())))
    }

    fn synth_list_comp(&mut self, comp: &rustpython_parser::ast::ExprListComp) -> Type {
        self.synth_comprehension(&comp.generators);
        Type::List(Box::new(self.synthesize(&comp.elt)))
    }

    fn synth_dict_comp(&mut self, comp: &rustpython_parser::ast::ExprDictComp) -> Type {
        self.synth_comprehension(&comp.generators);
        Type::Dict(Box::new(self.synthesize(&comp.key)), Box::new(self.synthesize(&comp.value)))
    }

    fn synth_set_comp(&mut self, comp: &rustpython_parser::ast::ExprSetComp) -> Type {
        self.synth_comprehension(&comp.generators);
        Type::Set(Box::new(self.synthesize(&comp.elt)))
    }

    fn synth_generator(&mut self, gen: &rustpython_parser::ast::ExprGeneratorExp) -> Type {
        self.synth_comprehension(&gen.generators);
        Type::Generic("Generator".to_string(), vec![self.synthesize(&gen.elt)])
    }

    fn synth_comprehension(&mut self, generators: &[Comprehension]) {
        for gen in generators {
            let _ = self.synthesize(&gen.iter); // TODO: Bind target to element type
            gen.ifs.iter().for_each(|cond| { self.check(cond, &Type::Bool); });
        }
    }

    fn synth_lambda(&mut self, lambda: &rustpython_parser::ast::ExprLambda) -> Type {
        let param_types = lambda.args.args.iter().map(|_| self.ctx.fresh_var()).collect();
        Type::Function(param_types, Box::new(self.synthesize(&lambda.body)))
    }

    fn synth_if_expr(&mut self, if_expr: &rustpython_parser::ast::ExprIfExp) -> Type {
        self.check(&if_expr.test, &Type::Bool);
        Type::union(vec![self.synthesize(&if_expr.body), self.synthesize(&if_expr.orelse)])
    }

    fn synth_subscript(&mut self, subscript: &rustpython_parser::ast::ExprSubscript) -> Type {
        let value_ty = self.synthesize(&subscript.value);
        let _ = self.synthesize(&subscript.slice);

        match value_ty {
            Type::List(elem) => *elem,
            Type::Tuple(elems) => {
                if let Expr::Constant(c) = &*subscript.slice { // Try to extract constant index
                    if let Constant::Int(idx) = &c.value {
                        return elems.get(idx.to_u32().unwrap_or(0) as usize).cloned().unwrap_or(Type::Any);
                    }
                }
                Type::union(elems)
            }
            Type::Dict(_, val) => *val,
            Type::Str => Type::Str,
            _ => self.ctx.fresh_var(),
        }
    }

    fn synth_attribute(&mut self, attr: &rustpython_parser::ast::ExprAttribute) -> Type {
        let value_ty = self.synthesize(&attr.value);

        self.ctx.has_attribute(&value_ty, &attr.attr).unwrap_or_else(|| {
            let available = self.ctx.get_attributes(&value_ty);
            let similar = crate::compiler::errors::find_similar_names(&attr.attr, &available, 2);

            let error = TypeError::new(
                crate::compiler::errors::ErrorKind::InvalidAttribute {
                    ty: value_ty.to_string(),
                    attr: attr.attr.to_string(),
                },
                SourceLocation::new(0, 0, 0, 0),
            );

            self.errors.add(if similar.is_empty() {
                error
            } else {
                error.with_suggestions(similar.iter().take(3).map(|s| format!("Did you mean '{}'?", s)).collect())
            });

            self.ctx.fresh_var()
        })
    }

    fn synth_slice(&mut self, _slice: &rustpython_parser::ast::ExprSlice) -> Type {
        Type::Class("slice".to_string())
    }
}

