use crate::core::types::{Type, TypeContext};
use rustpython_parser::ast::{Mod, ModModule, Stmt, Expr, Constant, Operator};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}, Col {}: {}", self.line, self.col, self.message)
    }
}

pub struct TypeChecker {
    ctx: Arc<TypeContext>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            ctx: Arc::new(TypeContext::new()),
            errors: Vec::new(),
        }
    }

    pub fn with_context(ctx: Arc<TypeContext>) -> Self {
        Self {
            ctx,
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, module: &Mod) -> Vec<TypeError> {
        self.errors.clear();

        if let Mod::Module(ModModule { body, .. }) = module {
            for stmt in body {
                self.check_stmt(stmt);
            }
        }

        self.errors.clone()
    }

    pub fn infer(&mut self, module: &Mod) -> Type {
        if let Mod::Module(ModModule { body, .. }) = module {
            if let Some(last) = body.last() {
                return self.infer_stmt(last);
            }
        }
        Type::None
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FunctionDef(func_def) => {
                // Infer parameter types
                let param_types: Vec<Type> = func_def.args.args.iter()
                    .map(|arg| {
                        if let Some(ann) = &arg.def.annotation {
                            self.type_from_annotation(ann)
                        } else {
                            self.ctx.fresh_var()
                        }
                    })
                    .collect();

                // Infer return type
                let return_type = if let Some(ret) = &func_def.returns {
                    self.type_from_annotation(ret)
                } else {
                    self.ctx.fresh_var()
                };

                let func_type = Type::Function(param_types.clone(), Box::new(return_type.clone()));
                self.ctx.set_type(func_def.name.to_string(), func_type);

                // Check function body
                for stmt in &func_def.body {
                    self.check_stmt(stmt);
                }
            }

            Stmt::Assign(assign) => {
                let value_type = self.infer_expr(&assign.value);

                for target in &assign.targets {
                    if let Expr::Name(name_expr) = target {
                        self.ctx.set_type(name_expr.id.to_string(), value_type.clone());
                    }
                }
            }

            Stmt::Return(ret) => {
                if let Some(val) = &ret.value {
                    self.infer_expr(val);
                }
            }

            Stmt::Expr(expr_stmt) => {
                self.infer_expr(&expr_stmt.value);
            }

            _ => {}
        }
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Type {
        self.check_stmt(stmt);
        Type::None
    }

    fn infer_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Constant(const_expr) => {
                match &const_expr.value {
                    Constant::None => Type::None,
                    Constant::Bool(_) => Type::Bool,
                    Constant::Int(_) => Type::Int,
                    Constant::Float(_) => Type::Float,
                    Constant::Str(_) => Type::Str,
                    Constant::Bytes(_) => Type::Bytes,
                    _ => Type::Any,
                }
            }

            Expr::Name(name_expr) => {
                self.ctx.get_type(&name_expr.id).unwrap_or_else(|| self.ctx.fresh_var())
            }

            Expr::BinOp(binop) => {
                let left_ty = self.infer_expr(&binop.left);
                let right_ty = self.infer_expr(&binop.right);

                // Simplified: assume numeric operations
                if left_ty == Type::Int && right_ty == Type::Int {
                    Type::Int
                } else if matches!(left_ty, Type::Int | Type::Float)
                    && matches!(right_ty, Type::Int | Type::Float) {
                    Type::Float
                } else {
                    Type::Any
                }
            }

            Expr::List(list_expr) => {
                if list_expr.elts.is_empty() {
                    Type::List(Box::new(self.ctx.fresh_var()))
                } else {
                    let elem_types: Vec<Type> = list_expr.elts.iter().map(|e| self.infer_expr(e)).collect();
                    let unified = Type::union(elem_types);
                    Type::List(Box::new(unified))
                }
            }

            Expr::Tuple(tuple_expr) => {
                let types: Vec<Type> = tuple_expr.elts.iter().map(|e| self.infer_expr(e)).collect();
                Type::Tuple(types)
            }

            Expr::Dict(dict_expr) => {
                let key_types: Vec<Type> = dict_expr.keys.iter()
                    .filter_map(|k| k.as_ref().map(|e| self.infer_expr(e)))
                    .collect();
                let value_types: Vec<Type> = dict_expr.values.iter().map(|v| self.infer_expr(v)).collect();

                let key_type = if key_types.is_empty() {
                    self.ctx.fresh_var()
                } else {
                    Type::union(key_types)
                };

                let value_type = if value_types.is_empty() {
                    self.ctx.fresh_var()
                } else {
                    Type::union(value_types)
                };

                Type::Dict(Box::new(key_type), Box::new(value_type))
            }

            Expr::Call(call_expr) => {
                let func_ty = self.infer_expr(&call_expr.func);

                if let Type::Function(_, ret) = func_ty {
                    *ret
                } else {
                    self.ctx.fresh_var()
                }
            }

            Expr::Attribute(attr_expr) => {
                let value_ty = self.infer_expr(&attr_expr.value);

                // Lookup attribute
                self.ctx.has_attribute(&value_ty, &attr_expr.attr)
                    .unwrap_or_else(|| {
                        // Generate error with suggestions
                        let available = self.ctx.get_attributes(&value_ty);
                        let similar = crate::errors::find_similar_names(&attr_expr.attr, &available, 2);

                        let mut msg = format!(
                            "Type '{}' has no attribute '{}'",
                            value_ty, attr_expr.attr
                        );
                        if !similar.is_empty() {
                            msg.push_str(&format!(". Did you mean: {}?", similar.join(", ")));
                        }

                        self.errors.push(TypeError {
                            message: msg,
                            line: 0,
                            col: 0,
                        });

                        self.ctx.fresh_var()
                    })
            }

            _ => Type::Any,
        }
    }

    fn type_from_annotation(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Name(name_expr) => match name_expr.id.as_str() {
                "int" => Type::Int,
                "float" => Type::Float,
                "str" => Type::Str,
                "bool" => Type::Bool,
                "bytes" => Type::Bytes,
                "None" => Type::None,
                "Any" => Type::Any,
                _ => Type::Class(name_expr.id.to_string()),
            },

            Expr::Subscript(subscript) => {
                if let Expr::Name(name_expr) = &*subscript.value {
                    match name_expr.id.as_str() {
                        "list" | "List" => Type::List(Box::new(self.type_from_annotation(&subscript.slice))),
                        "set" | "Set" => Type::Set(Box::new(self.type_from_annotation(&subscript.slice))),
                        "dict" | "Dict" => {
                            if let Expr::Tuple(tuple_expr) = &*subscript.slice {
                                if tuple_expr.elts.len() == 2 {
                                    return Type::Dict(
                                        Box::new(self.type_from_annotation(&tuple_expr.elts[0])),
                                        Box::new(self.type_from_annotation(&tuple_expr.elts[1])),
                                    );
                                }
                            }
                            Type::Dict(Box::new(Type::Any), Box::new(Type::Any))
                        }
                        _ => Type::Generic(name_expr.id.to_string(), vec![self.type_from_annotation(&subscript.slice)]),
                    }
                } else {
                    Type::Any
                }
            }

            Expr::BinOp(binop) => {
                if matches!(binop.op, Operator::BitOr) {
                    let left_ty = self.type_from_annotation(&binop.left);
                    let right_ty = self.type_from_annotation(&binop.right);
                    Type::Union(vec![left_ty, right_ty])
                } else {
                    Type::Any
                }
            }

            _ => Type::Any,
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
