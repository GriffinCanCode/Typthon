use crate::compiler::types::{Type, TypeContext};
use crate::compiler::analysis::{
    AdvancedTypeAnalyzer, EffectAnalyzer, RefinementAnalyzer,
    BiInfer, ConstraintSolver, VarianceAnalyzer, Constraint
};
use rustpython_parser::ast::{Mod, ModModule, Stmt, Expr, ExprConstant, Constant, Operator};
use num_traits::ToPrimitive;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

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
    advanced: AdvancedTypeAnalyzer,
    effects: EffectAnalyzer,
    refinements: RefinementAnalyzer,
    bi_infer: BiInfer,
    constraints: ConstraintSolver,
    variance: VarianceAnalyzer,
}

impl TypeChecker {
    pub fn new() -> Self {
        debug!("Creating new TypeChecker");
        let ctx = Arc::new(TypeContext::new());
        Self {
            effects: EffectAnalyzer::new(ctx.clone()),
            bi_infer: BiInfer::new(ctx.clone()),
            ctx,
            errors: Vec::new(),
            advanced: AdvancedTypeAnalyzer::new(),
            refinements: RefinementAnalyzer::new(),
            constraints: ConstraintSolver::new(),
            variance: VarianceAnalyzer::new(),
        }
    }

    pub fn with_context(ctx: Arc<TypeContext>) -> Self {
        Self {
            effects: EffectAnalyzer::new(ctx.clone()),
            bi_infer: BiInfer::new(ctx.clone()),
            ctx,
            errors: Vec::new(),
            advanced: AdvancedTypeAnalyzer::new(),
            refinements: RefinementAnalyzer::new(),
            constraints: ConstraintSolver::new(),
            variance: VarianceAnalyzer::new(),
        }
    }

    #[instrument(skip(self, module))]
    pub fn check(&mut self, module: &Mod) -> Vec<TypeError> {
        info!("Starting type checking");
        self.errors.clear();

        if let Mod::Module(ModModule { body, .. }) = module {
            // Phase 1: Analyze effects across the module (killer feature!)
            debug!("Phase 1: Analyzing effects");
            let effect_results = self.effects.analyze_module(module);
            info!(functions_analyzed = effect_results.len(), "Effect analysis complete");

            // Store effect analysis results for later use
            for (_func_name, _effects) in &effect_results {
                // Effect information is now tracked and available
            }

            // Phase 2: Check statements with all analyzers
            debug!(statements = body.len(), "Phase 2: Checking statements");
            for stmt in body {
                self.check_stmt(stmt);
            }

            // Phase 3: Solve constraints
            debug!("Phase 3: Solving constraints");
            if let Err(err) = self.constraints.solve() {
                error!(error = ?err, "Constraint solving failed");
                self.errors.push(TypeError {
                    message: format!("Constraint solving failed: {:?}", err),
                    line: 0,
                    col: 0,
                });
            } else {
                info!("Constraint solving complete");
            }
        }

        info!(error_count = self.errors.len(), "Type checking complete");
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

                // Create base function type
                let base_func_type = Type::Function(param_types.clone(), Box::new(return_type.clone()));

                // Check function body and infer effects
                for stmt in &func_def.body {
                    self.check_stmt(stmt);
                }

                // Annotate with inferred effects (killer feature!)
                let func_type = self.effects.annotate_function_type(&func_def.name, base_func_type);

                self.ctx.set_type(func_def.name.to_string(), func_type);
            }

            Stmt::Assign(assign) => {
                let value_type = self.infer_expr(&assign.value);

                for target in &assign.targets {
                    if let Expr::Name(name_expr) = target {
                        // Check if there's an annotation
                        if let Some(ann_type) = self.ctx.get_type(&name_expr.id) {
                            // Use bidirectional checking with expected type
                            if !self.bi_infer.check(&assign.value, &ann_type) {
                                self.errors.push(TypeError {
                                    message: format!("Type mismatch in assignment to {}", name_expr.id),
                                    line: 0,
                                    col: 0,
                                });
                            }
                            // Add constraint for solver (subtype constraint)
                            self.constraints.add_constraint(Constraint::Subtype(value_type.clone(), ann_type));
                        } else {
                            self.ctx.set_type(name_expr.id.to_string(), value_type.clone());
                        }
                    }
                }
            }

            Stmt::Return(ret) => {
                if let Some(val) = &ret.value {
                    let inferred = self.infer_expr(val);
                    // If we know the expected return type, check against it
                    // For now, just infer; full implementation would track current function context
                    let _ = inferred;
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
        // Use standard inference (BiInfer is used for checking, not inference)
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
                        let similar = crate::compiler::errors::find_similar_names(&attr_expr.attr, &available, 2);

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
                // Check for common refinement types
                "Positive" => RefinementAnalyzer::positive_int(),
                "Negative" => RefinementAnalyzer::negative_int(),
                "NonEmpty" => RefinementAnalyzer::non_empty_str(),
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
                        // Advanced type annotations
                        "EffectType" => {
                            // Parse effect type annotation
                            let base = self.type_from_annotation(&subscript.slice);
                            base // For now, return base type; effects added via decorator analysis
                        }
                        "RefinementType" => {
                            // Parse refinement type annotation
                            self.type_from_annotation(&subscript.slice)
                        }
                        "RecursiveType" => {
                            // Handle recursive type annotation
                            if let Expr::Constant(c) = &*subscript.slice {
                                if let Constant::Str(name) = &c.value {
                                    // Register as recursive type placeholder
                                    Type::Class(name.to_string())
                                } else {
                                    Type::Any
                                }
                            } else {
                                Type::Any
                            }
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

            Expr::Call(call) => {
                // Handle type constructor calls like Bounded(0, 100)
                if let Expr::Name(name) = &*call.func {
                    match name.id.as_str() {
                        "Bounded" => {
                            if call.args.len() == 2 {
                                if let (Expr::Constant(ExprConstant { value: Constant::Int(min), .. }),
                                        Expr::Constant(ExprConstant { value: Constant::Int(max), .. })) =
                                    (&call.args[0], &call.args[1]) {
                                    // Convert BigInt to i64
                                    if let (Some(min_i64), Some(max_i64)) = (min.to_i64(), max.to_i64()) {
                                        return RefinementAnalyzer::bounded_int(min_i64, max_i64);
                                    }
                                }
                            }
                            Type::Int
                        }
                        "effect" | "refine" | "dependent" | "newtype" | "recursive" => {
                            // These are constructor calls; parse the result
                            Type::Any
                        }
                        _ => Type::Any,
                    }
                } else {
                    Type::Any
                }
            }

            _ => Type::Any,
        }
    }

    /// Get effects for a function
    pub fn get_function_effects(&self, name: &str) -> Option<crate::compiler::types::types::EffectSet> {
        self.effects.get_function_effects(name).cloned()
    }

    /// Get type for a name
    pub fn get_type(&self, name: &str) -> Option<Type> {
        self.ctx.get_type(name)
    }

    /// Check if a recursive type is well-formed
    pub fn check_recursive_type(&mut self, ty: &Type) -> bool {
        self.advanced.is_productive(ty)
    }

    /// Validate a value against a refinement type
    pub fn validate_refinement(&self, value: &serde_json::Value, ty: &Type) -> bool {
        if let Type::Refinement(_, pred) = ty {
            self.refinements.validate(value, pred)
        } else {
            true
        }
    }

    /// Register a recursive type definition
    pub fn define_recursive(&mut self, name: String, body: Type) -> Type {
        self.advanced.define_recursive(name, body)
    }

    /// Use bidirectional type checking for an expression
    pub fn bi_check(&mut self, expr: &Expr, expected: &Type) -> bool {
        self.bi_infer.check(expr, expected)
    }

    /// Add a type constraint for solving
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.add_constraint(constraint);
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
