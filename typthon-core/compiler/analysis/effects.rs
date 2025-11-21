use crate::compiler::types::{Type, Effect, EffectSet, TypeContext};
use rustpython_parser::ast::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Effect analyzer tracks side effects through the program
pub struct EffectAnalyzer {
    ctx: Arc<TypeContext>,
    function_effects: HashMap<String, EffectSet>,
    builtin_effects: HashMap<String, EffectSet>,
}

impl EffectAnalyzer {
    pub fn new(ctx: Arc<TypeContext>) -> Self {
        let mut analyzer = Self {
            ctx,
            function_effects: HashMap::new(),
            builtin_effects: HashMap::new(),
        };
        analyzer.init_builtins();
        analyzer
    }

    fn init_builtins(&mut self) {
        // Pure functions
        for name in &["len", "abs", "min", "max", "sum", "all", "any",
                      "sorted", "reversed", "enumerate", "zip", "map", "filter"] {
            self.builtin_effects.insert(name.to_string(), EffectSet::pure());
        }

        // IO effects
        for name in &["print", "input", "open", "read", "write"] {
            self.builtin_effects.insert(name.to_string(), EffectSet::single(Effect::IO));
        }

        // Random effects
        for name in &["random", "randint", "choice", "shuffle"] {
            self.builtin_effects.insert(name.to_string(), EffectSet::single(Effect::Random));
        }

        // Time effects
        for name in &["time", "sleep"] {
            self.builtin_effects.insert(name.to_string(), EffectSet::single(Effect::Time));
        }
    }

    /// Analyze effects in a module
    pub fn analyze_module(&mut self, module: &Mod) -> HashMap<String, EffectSet> {
        if let Mod::Module(mod_module) = module {
            for stmt in &mod_module.body {
                self.analyze_stmt(stmt);
            }
        }
        self.function_effects.clone()
    }

    /// Analyze statement for effects
    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FunctionDef(func) => self.analyze_function(func),
            Stmt::AsyncFunctionDef(func) => self.analyze_async_function(func),
            Stmt::For(for_stmt) => {
                self.infer_expr_effects(&for_stmt.iter);
                for_stmt.body.iter().for_each(|s| self.analyze_stmt(s));
            }
            Stmt::While(while_stmt) => {
                self.infer_expr_effects(&while_stmt.test);
                while_stmt.body.iter().for_each(|s| self.analyze_stmt(s));
            }
            Stmt::If(if_stmt) => {
                self.infer_expr_effects(&if_stmt.test);
                if_stmt.body.iter().for_each(|s| self.analyze_stmt(s));
                if_stmt.orelse.iter().for_each(|s| self.analyze_stmt(s));
            }
            Stmt::With(with_stmt) => {
                // Context managers might have effects
                for item in &with_stmt.items {
                    self.infer_expr_effects(&item.context_expr);
                }
                with_stmt.body.iter().for_each(|s| self.analyze_stmt(s));
            }
            Stmt::Try(try_stmt) => {
                // Exception handling adds Exception effect
                try_stmt.body.iter().for_each(|s| self.analyze_stmt(s));
                for handler in &try_stmt.handlers {
                    let ExceptHandler::ExceptHandler(h) = handler;
                    h.body.iter().for_each(|s| self.analyze_stmt(s));
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.infer_expr_effects(&expr_stmt.value);
            }
            Stmt::Assign(assign) => {
                self.infer_expr_effects(&assign.value);
            }
            _ => {}
        }
    }

    /// Analyze function definition
    fn analyze_function(&mut self, func: &StmtFunctionDef) {
        let mut effects = EffectSet::pure();

        // Analyze function body
        for stmt in &func.body {
            let stmt_effects = self.infer_stmt_effects(stmt);
            effects = effects.union(stmt_effects);
        }

        self.function_effects.insert(func.name.to_string(), effects);
    }

    /// Analyze async function (always has Async effect)
    fn analyze_async_function(&mut self, func: &StmtAsyncFunctionDef) {
        let mut effects = EffectSet::single(Effect::Async);

        // Analyze function body
        for stmt in &func.body {
            let stmt_effects = self.infer_stmt_effects(stmt);
            effects = effects.union(stmt_effects);
        }

        self.function_effects.insert(func.name.to_string(), effects);
    }

    /// Infer effects of a statement
    fn infer_stmt_effects(&mut self, stmt: &Stmt) -> EffectSet {
        match stmt {
            Stmt::Expr(expr_stmt) => self.infer_expr_effects(&expr_stmt.value),
            Stmt::Assign(assign) => {
                let mut effects = self.infer_expr_effects(&assign.value);
                // Assignment is mutation
                effects = effects.union(EffectSet::single(Effect::Mutation));
                effects
            }
            Stmt::AugAssign(aug) => {
                let mut effects = self.infer_expr_effects(&aug.value);
                effects = effects.union(EffectSet::single(Effect::Mutation));
                effects
            }
            Stmt::Raise(_) => EffectSet::single(Effect::Exception),
            Stmt::Return(ret) => {
                ret.value.as_ref()
                    .map(|v| self.infer_expr_effects(v))
                    .unwrap_or_else(EffectSet::pure)
            }
            Stmt::For(for_stmt) => {
                let mut effects = self.infer_expr_effects(&for_stmt.iter);
                for body_stmt in &for_stmt.body {
                    effects = effects.union(self.infer_stmt_effects(body_stmt));
                }
                effects
            }
            Stmt::While(while_stmt) => {
                let mut effects = self.infer_expr_effects(&while_stmt.test);
                for body_stmt in &while_stmt.body {
                    effects = effects.union(self.infer_stmt_effects(body_stmt));
                }
                effects
            }
            Stmt::If(if_stmt) => {
                let mut effects = self.infer_expr_effects(&if_stmt.test);
                for body_stmt in &if_stmt.body {
                    effects = effects.union(self.infer_stmt_effects(body_stmt));
                }
                for else_stmt in &if_stmt.orelse {
                    effects = effects.union(self.infer_stmt_effects(else_stmt));
                }
                effects
            }
            _ => EffectSet::pure(),
        }
    }

    /// Infer effects of an expression
    fn infer_expr_effects(&mut self, expr: &Expr) -> EffectSet {
        match expr {
            Expr::Call(call) => self.infer_call_effects(call),
            Expr::Await(_) => EffectSet::single(Effect::Async),
            Expr::Yield(_) | Expr::YieldFrom(_) => EffectSet::single(Effect::Async),
            Expr::BinOp(binop) => {
                let left = self.infer_expr_effects(&binop.left);
                let right = self.infer_expr_effects(&binop.right);
                left.union(right)
            }
            Expr::UnaryOp(unary) => self.infer_expr_effects(&unary.operand),
            Expr::Lambda(lambda) => self.infer_expr_effects(&lambda.body),
            Expr::IfExp(if_exp) => {
                let test = self.infer_expr_effects(&if_exp.test);
                let body = self.infer_expr_effects(&if_exp.body);
                let orelse = self.infer_expr_effects(&if_exp.orelse);
                test.union(body).union(orelse)
            }
            Expr::ListComp(comp) => {
                let mut effects = self.infer_expr_effects(&comp.elt);
                for gen in &comp.generators {
                    effects = effects.union(self.infer_expr_effects(&gen.iter));
                    for cond in &gen.ifs {
                        effects = effects.union(self.infer_expr_effects(cond));
                    }
                }
                effects
            }
            Expr::List(list) => {
                list.elts.iter()
                    .fold(EffectSet::pure(), |acc, e| acc.union(self.infer_expr_effects(e)))
            }
            Expr::Tuple(tuple) => {
                tuple.elts.iter()
                    .fold(EffectSet::pure(), |acc, e| acc.union(self.infer_expr_effects(e)))
            }
            Expr::Dict(dict) => {
                let mut effects = EffectSet::pure();
                for key in &dict.keys {
                    if let Some(k) = key {
                        effects = effects.union(self.infer_expr_effects(k));
                    }
                }
                for value in &dict.values {
                    effects = effects.union(self.infer_expr_effects(value));
                }
                effects
            }
            _ => EffectSet::pure(),
        }
    }

    /// Infer effects of a function call
    fn infer_call_effects(&mut self, call: &ExprCall) -> EffectSet {
        // Check if it's a builtin
        if let Expr::Name(name) = &*call.func {
            if let Some(effects) = self.builtin_effects.get(name.id.as_str()) {
                return effects.clone();
            }

            // Check if we've analyzed this function
            if let Some(effects) = self.function_effects.get(name.id.as_str()) {
                return effects.clone();
            }
        }

        // Check if function type has effects
        if let Some(func_ty) = self.get_function_type(&call.func) {
            if let Type::Effect(_, effects) = func_ty {
                return effects;
            }
        }

        // Analyze arguments
        let mut effects = EffectSet::pure();
        for arg in &call.args {
            effects = effects.union(self.infer_expr_effects(arg));
        }

        // Conservative: assume unknown functions might have effects
        effects
    }

    fn get_function_type(&self, expr: &Expr) -> Option<Type> {
        if let Expr::Name(name) = expr {
            self.ctx.get_type(&name.id)
        } else {
            None
        }
    }

    /// Get inferred effects for a function
    pub fn get_function_effects(&self, name: &str) -> Option<&EffectSet> {
        self.function_effects.get(name)
    }

    /// Check if an expression is pure
    pub fn is_pure_expr(&mut self, expr: &Expr) -> bool {
        self.infer_expr_effects(expr).is_pure()
    }

    /// Add effect annotation to function type
    pub fn annotate_function_type(&self, name: &str, base_type: Type) -> Type {
        if let Some(effects) = self.function_effects.get(name) {
            if !effects.is_pure() {
                return Type::Effect(Box::new(base_type), effects.clone());
            }
        }
        base_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_function() {
        let ctx = Arc::new(TypeContext::new());
        let mut analyzer = EffectAnalyzer::new(ctx);

        let source = "def add(x, y):\n    return x + y";
        let module = rustpython_parser::parse_program(source, "test.py").unwrap();

        analyzer.analyze_module(&module);

        let effects = analyzer.get_function_effects("add").unwrap();
        assert!(effects.is_pure());
    }

    #[test]
    fn test_io_function() {
        let ctx = Arc::new(TypeContext::new());
        let mut analyzer = EffectAnalyzer::new(ctx);

        let source = "def greet():\n    print('Hello')";
        let module = rustpython_parser::parse_program(source, "test.py").unwrap();

        analyzer.analyze_module(&module);

        let effects = analyzer.get_function_effects("greet").unwrap();
        assert!(effects.contains(&Effect::IO));
    }

    #[test]
    fn test_mutation_function() {
        let ctx = Arc::new(TypeContext::new());
        let mut analyzer = EffectAnalyzer::new(ctx);

        let source = "def modify(x):\n    x = 42\n    return x";
        let module = rustpython_parser::parse_program(source, "test.py").unwrap();

        analyzer.analyze_module(&module);

        let effects = analyzer.get_function_effects("modify").unwrap();
        assert!(effects.contains(&Effect::Mutation));
    }
}

