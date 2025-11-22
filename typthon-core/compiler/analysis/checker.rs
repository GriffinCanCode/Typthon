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
    class_attributes: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    current_class: Option<String>,
    current_function_return_type: Option<Type>,
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
            class_attributes: std::collections::HashMap::new(),
            current_class: None,
            current_function_return_type: None,
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
            class_attributes: std::collections::HashMap::new(),
            current_class: None,
            current_function_return_type: None,
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

                // Infer return type (only check if explicitly annotated)
                let (return_type, has_return_annotation) = if let Some(ret) = &func_def.returns {
                    (self.type_from_annotation(ret), true)
                } else {
                    (self.ctx.fresh_var(), false)
                };

                // Create base function type
                let base_func_type = Type::Function(param_types.clone(), Box::new(return_type.clone()));

                // Set parameters in context for function body
                for (arg, param_ty) in func_def.args.args.iter().zip(param_types.iter()) {
                    self.ctx.set_type(arg.def.arg.to_string(), param_ty.clone());
                }

                // Track current function return type for validation (only if annotated)
                let prev_return_type = self.current_function_return_type.take();
                if has_return_annotation {
                    self.current_function_return_type = Some(return_type.clone());
                }

                // Check function body and infer effects
                for stmt in &func_def.body {
                    self.check_stmt(stmt);
                }

                // Restore previous return type
                self.current_function_return_type = prev_return_type;

                // Annotate with inferred effects (killer feature!)
                let func_type = self.effects.annotate_function_type(&func_def.name, base_func_type);

                self.ctx.set_type(func_def.name.to_string(), func_type);
            }

            Stmt::Assign(assign) => {
                let value_type = self.infer_expr(&assign.value);

                for target in &assign.targets {
                    match target {
                        Expr::Name(name_expr) => {
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
                        Expr::Attribute(attr) => {
                            // Track class attribute assignments (self.x = value)
                            if let Expr::Name(base) = &*attr.value {
                                if base.id.as_str() == "self" {
                                    if let Some(class_name) = &self.current_class {
                                        if let Some(attrs) = self.class_attributes.get_mut(class_name) {
                                            attrs.insert(attr.attr.to_string(), value_type.clone());
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            Stmt::AnnAssign(ann_assign) => {
                // Handle annotated assignments: x: int = value
                let ann_type = self.type_from_annotation(&ann_assign.annotation);

                if let Some(value) = &ann_assign.value {
                    let value_type = self.infer_expr(value);

                    // Check type compatibility
                    if !self.is_compatible(&value_type, &ann_type) {
                        if let Expr::Name(name_expr) = &*ann_assign.target {
                            self.errors.push(TypeError {
                                message: format!(
                                    "Type mismatch: cannot assign {} to variable '{}' of type {}",
                                    value_type, name_expr.id, ann_type
                                ),
                                line: 0,
                                col: 0,
                            });
                        } else {
                            self.errors.push(TypeError {
                                message: format!(
                                    "Type mismatch: cannot assign {} to type {}",
                                    value_type, ann_type
                                ),
                                line: 0,
                                col: 0,
                            });
                        }
                    }

                    // Add constraint
                    self.constraints.add_constraint(Constraint::Subtype(value_type, ann_type.clone()));
                }

                // Register the variable with its annotation type
                if let Expr::Name(name_expr) = &*ann_assign.target {
                    self.ctx.set_type(name_expr.id.to_string(), ann_type);
                }
            }

            Stmt::Return(ret) => {
                if let Some(val) = &ret.value {
                    let inferred = self.infer_expr(val);
                    // Check against expected return type
                    if let Some(expected) = &self.current_function_return_type {
                        if !inferred.is_subtype(expected) {
                            self.errors.push(TypeError {
                                message: format!(
                                    "Return type mismatch: expected {:?}, got {:?}",
                                    expected, inferred
                                ),
                                line: 0,
                                col: 0,
                            });
                        }
                    }
                } else if let Some(expected) = &self.current_function_return_type {
                    // Empty return, check if function expects None
                    if !matches!(expected, Type::None) {
                        self.errors.push(TypeError {
                            message: format!("Expected return value of type {:?}, got None", expected),
                            line: 0,
                            col: 0,
                        });
                    }
                }
            }

            Stmt::Expr(expr_stmt) => {
                self.infer_expr(&expr_stmt.value);
            }

            Stmt::Import(_) | Stmt::ImportFrom(_) => {
                // Import statements are valid, no type checking needed
                // Types from typing module are handled in type_from_annotation
            }

            Stmt::ClassDef(class_def) => {
                // Register class type
                let class_type = Type::Class(class_def.name.to_string());
                self.ctx.set_type(class_def.name.to_string(), class_type);

                // Track current class for attribute resolution
                let prev_class = self.current_class.clone();
                self.current_class = Some(class_def.name.to_string());
                self.class_attributes.insert(class_def.name.to_string(), std::collections::HashMap::new());

                // Check class body
                for stmt in &class_def.body {
                    self.check_stmt(stmt);
                }

                // Restore previous class context
                self.current_class = prev_class;
            }

            Stmt::For(for_stmt) => {
                // Infer the type of the iterable
                let iterable_ty = self.infer_expr(&for_stmt.iter);

                // Get the element type from the iterable
                let elem_ty = match iterable_ty {
                    Type::List(elem) => *elem,
                    Type::Set(elem) => *elem,
                    Type::Tuple(elems) if !elems.is_empty() => {
                        // For tuple, use union of all element types
                        Type::union(elems)
                    }
                    Type::Dict(key, _) => *key, // Iterating over dict gives keys
                    Type::Str => Type::Str, // String iteration gives strings
                    _ => self.ctx.fresh_var(),
                };

                // Set the loop variable type
                if let Expr::Name(name_expr) = &*for_stmt.target {
                    self.ctx.set_type(name_expr.id.to_string(), elem_ty);
                }

                // Check the loop body
                for stmt in &for_stmt.body {
                    self.check_stmt(stmt);
                }

                // Check orelse clause if present
                for stmt in &for_stmt.orelse {
                    self.check_stmt(stmt);
                }
            }

            Stmt::While(while_stmt) => {
                // Check the condition
                let _cond_ty = self.infer_expr(&while_stmt.test);

                // Check the body
                for stmt in &while_stmt.body {
                    self.check_stmt(stmt);
                }

                // Check orelse clause if present
                for stmt in &while_stmt.orelse {
                    self.check_stmt(stmt);
                }
            }

            Stmt::If(if_stmt) => {
                // Check the condition
                let _cond_ty = self.infer_expr(&if_stmt.test);

                // Check the if body
                for stmt in &if_stmt.body {
                    self.check_stmt(stmt);
                }

                // Check elif/else clauses
                for stmt in &if_stmt.orelse {
                    self.check_stmt(stmt);
                }
            }

            Stmt::With(_) => {
                // Context manager - basic traversal for now
                // Full implementation would track resource types
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

                use rustpython_parser::ast::Operator as Op;
                match binop.op {
                    // Addition
                    Op::Add => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Type::Int
                        } else if matches!(left_ty, Type::Int | Type::Float)
                            && matches!(right_ty, Type::Int | Type::Float) {
                            Type::Float
                        } else if left_ty == Type::Str && right_ty == Type::Str {
                            Type::Str  // String concatenation
                        } else if matches!(left_ty, Type::List(_)) && matches!(right_ty, Type::List(_)) {
                            left_ty  // List concatenation
                        } else {
                            Type::Any
                        }
                    }
                    // Multiplication
                    Op::Mult => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Type::Int
                        } else if matches!(left_ty, Type::Int | Type::Float)
                            && matches!(right_ty, Type::Int | Type::Float) {
                            Type::Float
                        } else if (left_ty == Type::Str && right_ty == Type::Int) ||
                                  (left_ty == Type::Int && right_ty == Type::Str) {
                            Type::Str  // String repetition
                        } else {
                            Type::Any
                        }
                    }
                    // Subtraction, Modulo, Power
                    Op::Sub | Op::Mod | Op::Pow => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Type::Int
                        } else if matches!(left_ty, Type::Int | Type::Float)
                            && matches!(right_ty, Type::Int | Type::Float) {
                            Type::Float
                        } else {
                            Type::Any
                        }
                    }
                    // Division always returns float
                    Op::Div => Type::Float,
                    // Floor division returns int
                    Op::FloorDiv => Type::Int,
                    _ => Type::Any,
                }
            }

            Expr::Compare(_compare) => {
                // All comparisons return bool (==, !=, <, >, <=, >=, in, not in, is, is not)
                Type::Bool
            }

            Expr::UnaryOp(unary) => {
                use rustpython_parser::ast::UnaryOp as UOp;
                match unary.op {
                    UOp::Not => Type::Bool,
                    UOp::UAdd | UOp::USub => {
                        let operand_ty = self.infer_expr(&unary.operand);
                        operand_ty  // +x and -x preserve type
                    }
                    UOp::Invert => Type::Int,  // ~x for bitwise inversion
                }
            }

            Expr::BoolOp(_bool_op) => {
                // and, or operations return bool
                Type::Bool
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

            Expr::ListComp(list_comp) => {
                // Handle generators to set loop variable types
                for generator in &list_comp.generators {
                    let iterable_ty = self.infer_expr(&generator.iter);

                    // Get element type from iterable
                    let elem_ty = match iterable_ty {
                        Type::List(elem) => *elem,
                        Type::Set(elem) => *elem,
                        Type::Tuple(elems) if !elems.is_empty() => Type::union(elems),
                        Type::Dict(key, _) => *key,
                        Type::Str => Type::Str,
                        _ => self.ctx.fresh_var(),
                    };

                    // Set loop variable type
                    if let Expr::Name(name_expr) = &generator.target {
                        self.ctx.set_type(name_expr.id.to_string(), elem_ty);
                    }
                }

                // Infer type of list comprehension from element expression
                let elem_type = self.infer_expr(&list_comp.elt);
                Type::List(Box::new(elem_type))
            }

            Expr::DictComp(dict_comp) => {
                // Handle generators to set loop variable types
                for generator in &dict_comp.generators {
                    let iterable_ty = self.infer_expr(&generator.iter);

                    let elem_ty = match iterable_ty {
                        Type::List(elem) => *elem,
                        Type::Set(elem) => *elem,
                        Type::Tuple(elems) if !elems.is_empty() => Type::union(elems),
                        Type::Dict(key, _) => *key,
                        Type::Str => Type::Str,
                        _ => self.ctx.fresh_var(),
                    };

                    if let Expr::Name(name_expr) = &generator.target {
                        self.ctx.set_type(name_expr.id.to_string(), elem_ty);
                    }
                }

                // Infer type of dict comprehension
                let key_type = self.infer_expr(&dict_comp.key);
                let value_type = self.infer_expr(&dict_comp.value);
                Type::Dict(Box::new(key_type), Box::new(value_type))
            }

            Expr::SetComp(set_comp) => {
                // Handle generators to set loop variable types
                for generator in &set_comp.generators {
                    let iterable_ty = self.infer_expr(&generator.iter);

                    let elem_ty = match iterable_ty {
                        Type::List(elem) => *elem,
                        Type::Set(elem) => *elem,
                        Type::Tuple(elems) if !elems.is_empty() => Type::union(elems),
                        Type::Dict(key, _) => *key,
                        Type::Str => Type::Str,
                        _ => self.ctx.fresh_var(),
                    };

                    if let Expr::Name(name_expr) = &generator.target {
                        self.ctx.set_type(name_expr.id.to_string(), elem_ty);
                    }
                }

                // Infer type of set comprehension
                let elem_type = self.infer_expr(&set_comp.elt);
                Type::Set(Box::new(elem_type))
            }

            Expr::Set(set_expr) => {
                if set_expr.elts.is_empty() {
                    Type::Set(Box::new(self.ctx.fresh_var()))
                } else {
                    let elem_types: Vec<Type> = set_expr.elts.iter().map(|e| self.infer_expr(e)).collect();
                    let unified = Type::union(elem_types);
                    Type::Set(Box::new(unified))
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

                match func_ty {
                    Type::Function(params, ret) => {
                        // Check argument count
                        if call_expr.args.len() != params.len() {
                            self.errors.push(TypeError {
                                message: format!(
                                    "Function call argument count mismatch: expected {}, got {}",
                                    params.len(),
                                    call_expr.args.len()
                                ),
                                line: 0,
                                col: 0,
                            });
                        }

                        // Check argument types
                        for (i, (arg, param_ty)) in call_expr.args.iter().zip(params.iter()).enumerate() {
                            let arg_ty = self.infer_expr(arg);
                            if !arg_ty.is_subtype(param_ty) {
                                self.errors.push(TypeError {
                                    message: format!(
                                        "Argument {} type mismatch: expected {:?}, got {:?}",
                                        i, param_ty, arg_ty
                                    ),
                                    line: 0,
                                    col: 0,
                                });
                            }
                        }

                        *ret
                    }
                    _ => self.ctx.fresh_var()
                }
            }

            Expr::Subscript(subscript_expr) => {
                // Handle indexing: list[i], dict[key], tuple[i]
                let value_ty = self.infer_expr(&subscript_expr.value);

                match value_ty {
                    Type::List(elem_ty) => *elem_ty,
                    Type::Dict(_, val_ty) => *val_ty,
                    Type::Tuple(types) => {
                        // For tuple indexing, if we can determine the index statically, return that type
                        // Otherwise, return union of all types
                        if types.is_empty() {
                            Type::Any
                        } else if types.len() == 1 {
                            types[0].clone()
                        } else {
                            Type::union(types)
                        }
                    }
                    Type::Str => Type::Str,  // String indexing returns str
                    _ => self.ctx.fresh_var(),
                }
            }

            Expr::Attribute(attr_expr) => {
                let value_ty = self.infer_expr(&attr_expr.value);

                // For class types, look up in class_attributes
                if let Type::Class(class_name) = &value_ty {
                    if let Some(attrs) = self.class_attributes.get(class_name) {
                        if let Some(attr_ty) = attrs.get(attr_expr.attr.as_str()) {
                            return attr_ty.clone();
                        }
                    }
                }

                // Otherwise, lookup attribute from context
                self.ctx.has_attribute(&value_ty, &attr_expr.attr)
                    .unwrap_or_else(|| {
                        // Don't generate error for class types - attributes might be set dynamically
                        if matches!(value_ty, Type::Class(_)) {
                            self.ctx.fresh_var()
                        } else {
                            // Generate error with suggestions for non-class types
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
                        }
                    })
            }

            _ => Type::Any,
        }
    }

    fn is_compatible(&self, actual: &Type, expected: &Type) -> bool {
        // Check if actual type is compatible with expected type
        match (actual, expected) {
            // Exact matches
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Str, Type::Str) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Bytes, Type::Bytes) => true,
            (Type::None, Type::None) => true,
            (Type::Any, _) | (_, Type::Any) => true,

            // Int is compatible with Float (subtyping)
            (Type::Int, Type::Float) => true,

            // Collection types
            (Type::List(a), Type::List(b)) => self.is_compatible(a, b),
            (Type::Set(a), Type::Set(b)) => self.is_compatible(a, b),
            (Type::Dict(ka, va), Type::Dict(kb, vb)) => {
                self.is_compatible(ka, kb) && self.is_compatible(va, vb)
            }
            (Type::Tuple(ta), Type::Tuple(tb)) => {
                ta.len() == tb.len() &&
                ta.iter().zip(tb.iter()).all(|(a, b)| self.is_compatible(a, b))
            }

            // Function types
            (Type::Function(pa, ra), Type::Function(pb, rb)) => {
                pa.len() == pb.len() &&
                pa.iter().zip(pb.iter()).all(|(a, b)| self.is_compatible(a, b)) &&
                self.is_compatible(ra, rb)
            }

            // Union types - actual must be one of the expected union members
            (actual, Type::Union(expected_types)) => {
                expected_types.iter().any(|t| self.is_compatible(actual, t))
            }

            // Class types
            (Type::Class(a), Type::Class(b)) => a == b,

            // Generic types
            (Type::Generic(na, ta), Type::Generic(nb, tb)) => {
                na == nb && ta.len() == tb.len() &&
                ta.iter().zip(tb.iter()).all(|(a, b)| self.is_compatible(a, b))
            }

            // Type variables are always compatible (will be resolved by constraint solver)
            (Type::Var(_), _) | (_, Type::Var(_)) => true,

            // Default: incompatible
            _ => false,
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
                        "tuple" | "Tuple" => {
                            // Handle tuple type annotations
                            if let Expr::Tuple(tuple_expr) = &*subscript.slice {
                                // Empty tuple: tuple[()] or tuple with elements
                                if tuple_expr.elts.is_empty() {
                                    Type::Tuple(vec![])
                                } else {
                                    let types = tuple_expr.elts.iter()
                                        .map(|e| self.type_from_annotation(e))
                                        .collect();
                                    Type::Tuple(types)
                                }
                            } else {
                                // Single element tuple or error - treat as single element
                                Type::Tuple(vec![self.type_from_annotation(&subscript.slice)])
                            }
                        }
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
                        "Union" => {
                            // Handle Union[T1, T2, ...] from typing
                            if let Expr::Tuple(tuple_expr) = &*subscript.slice {
                                let types = tuple_expr.elts.iter()
                                    .map(|e| self.type_from_annotation(e))
                                    .collect();
                                Type::Union(types)
                            } else {
                                // Single type in Union
                                Type::Union(vec![self.type_from_annotation(&subscript.slice)])
                            }
                        }
                        "Optional" => {
                            // Optional[T] is Union[T, None]
                            let inner_type = self.type_from_annotation(&subscript.slice);
                            Type::Union(vec![inner_type, Type::None])
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
