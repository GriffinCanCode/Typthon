use rustpython_parser::ast::*;

/// Visitor trait for traversing Python AST with extensible callbacks
pub trait AstVisitor<T = ()>
where
    T: Default,
{
    fn visit_module(&mut self, module: &Mod) -> T {
        self.walk_module(module)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        self.walk_stmt(stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> T {
        self.walk_expr(expr)
    }

    fn visit_pattern(&mut self, pattern: &Pattern) -> T {
        self.walk_pattern(pattern)
    }

    // Specific statement visitors
    fn visit_function_def(&mut self, func: &StmtFunctionDef) -> T;
    fn visit_async_function_def(&mut self, func: &StmtAsyncFunctionDef) -> T;
    fn visit_class_def(&mut self, class: &StmtClassDef) -> T;
    fn visit_return(&mut self, ret: &StmtReturn) -> T;
    fn visit_delete(&mut self, del: &StmtDelete) -> T;
    fn visit_assign(&mut self, assign: &StmtAssign) -> T;
    fn visit_aug_assign(&mut self, aug: &StmtAugAssign) -> T;
    fn visit_ann_assign(&mut self, ann: &StmtAnnAssign) -> T;
    fn visit_for(&mut self, for_stmt: &StmtFor) -> T;
    fn visit_async_for(&mut self, for_stmt: &StmtAsyncFor) -> T;
    fn visit_while(&mut self, while_stmt: &StmtWhile) -> T;
    fn visit_if(&mut self, if_stmt: &StmtIf) -> T;
    fn visit_with(&mut self, with: &StmtWith) -> T;
    fn visit_async_with(&mut self, with: &StmtAsyncWith) -> T;
    fn visit_match(&mut self, match_stmt: &StmtMatch) -> T;
    fn visit_raise(&mut self, raise: &StmtRaise) -> T;
    fn visit_try(&mut self, try_stmt: &StmtTry) -> T;
    fn visit_assert(&mut self, assert: &StmtAssert) -> T;
    fn visit_import(&mut self, import: &StmtImport) -> T;
    fn visit_import_from(&mut self, import: &StmtImportFrom) -> T;
    fn visit_global(&mut self, global: &StmtGlobal) -> T;
    fn visit_nonlocal(&mut self, nonlocal: &StmtNonlocal) -> T;
    fn visit_expr_stmt(&mut self, expr: &StmtExpr) -> T;
    fn visit_pass(&mut self, pass: &StmtPass) -> T;
    fn visit_break(&mut self, break_stmt: &StmtBreak) -> T;
    fn visit_continue(&mut self, cont: &StmtContinue) -> T;
    fn visit_type_alias(&mut self, type_alias: &StmtTypeAlias) -> T;
    fn visit_try_star(&mut self, try_star: &StmtTryStar) -> T;

    // Specific expression visitors
    fn visit_bool_op(&mut self, bool_op: &ExprBoolOp) -> T;
    fn visit_named_expr(&mut self, named: &ExprNamedExpr) -> T;
    fn visit_bin_op(&mut self, bin_op: &ExprBinOp) -> T;
    fn visit_unary_op(&mut self, unary: &ExprUnaryOp) -> T;
    fn visit_lambda(&mut self, lambda: &ExprLambda) -> T;
    fn visit_if_exp(&mut self, if_exp: &ExprIfExp) -> T;
    fn visit_dict(&mut self, dict: &ExprDict) -> T;
    fn visit_set(&mut self, set: &ExprSet) -> T;
    fn visit_list_comp(&mut self, comp: &ExprListComp) -> T;
    fn visit_set_comp(&mut self, comp: &ExprSetComp) -> T;
    fn visit_dict_comp(&mut self, comp: &ExprDictComp) -> T;
    fn visit_generator_exp(&mut self, gen: &ExprGeneratorExp) -> T;
    fn visit_await(&mut self, await_expr: &ExprAwait) -> T;
    fn visit_yield(&mut self, yield_expr: &ExprYield) -> T;
    fn visit_yield_from(&mut self, yield_from: &ExprYieldFrom) -> T;
    fn visit_compare(&mut self, compare: &ExprCompare) -> T;
    fn visit_call(&mut self, call: &ExprCall) -> T;
    fn visit_formatted_value(&mut self, formatted: &ExprFormattedValue) -> T;
    fn visit_joined_str(&mut self, joined: &ExprJoinedStr) -> T;
    fn visit_constant(&mut self, constant: &ExprConstant) -> T;
    fn visit_attribute(&mut self, attr: &ExprAttribute) -> T;
    fn visit_subscript(&mut self, subscript: &ExprSubscript) -> T;
    fn visit_starred(&mut self, starred: &ExprStarred) -> T;
    fn visit_name(&mut self, name: &ExprName) -> T;
    fn visit_list(&mut self, list: &ExprList) -> T;
    fn visit_tuple(&mut self, tuple: &ExprTuple) -> T;
    fn visit_slice(&mut self, slice: &ExprSlice) -> T;

    // Default walk implementations
    fn walk_module(&mut self, module: &Mod) -> T {
        match module {
            Mod::Module(ModModule { body, .. }) => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Mod::Interactive(ModInteractive { body, .. }) => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Mod::Expression(ModExpression { body, .. }) => {
                self.visit_expr(body);
            }
            Mod::FunctionType(ModFunctionType { argtypes, returns, .. }) => {
                for arg in argtypes {
                    self.visit_expr(arg);
                }
                self.visit_expr(returns);
            }
        }
        T::default()
    }

    fn walk_stmt(&mut self, stmt: &Stmt) -> T {
        match stmt {
            Stmt::FunctionDef(f) => self.visit_function_def(f),
            Stmt::AsyncFunctionDef(f) => self.visit_async_function_def(f),
            Stmt::ClassDef(c) => self.visit_class_def(c),
            Stmt::Return(r) => self.visit_return(r),
            Stmt::Delete(d) => self.visit_delete(d),
            Stmt::Assign(a) => self.visit_assign(a),
            Stmt::AugAssign(a) => self.visit_aug_assign(a),
            Stmt::AnnAssign(a) => self.visit_ann_assign(a),
            Stmt::For(f) => self.visit_for(f),
            Stmt::AsyncFor(f) => self.visit_async_for(f),
            Stmt::While(w) => self.visit_while(w),
            Stmt::If(i) => self.visit_if(i),
            Stmt::With(w) => self.visit_with(w),
            Stmt::AsyncWith(w) => self.visit_async_with(w),
            Stmt::Match(m) => self.visit_match(m),
            Stmt::Raise(r) => self.visit_raise(r),
            Stmt::Try(t) => self.visit_try(t),
            Stmt::Assert(a) => self.visit_assert(a),
            Stmt::Import(i) => self.visit_import(i),
            Stmt::ImportFrom(i) => self.visit_import_from(i),
            Stmt::Global(g) => self.visit_global(g),
            Stmt::Nonlocal(n) => self.visit_nonlocal(n),
            Stmt::Expr(e) => self.visit_expr_stmt(e),
            Stmt::Pass(p) => self.visit_pass(p),
            Stmt::Break(b) => self.visit_break(b),
            Stmt::Continue(c) => self.visit_continue(c),
            Stmt::TypeAlias(t) => self.visit_type_alias(t),
            Stmt::TryStar(t) => self.visit_try_star(t),
        }
    }

    fn walk_expr(&mut self, expr: &Expr) -> T {
        match expr {
            Expr::BoolOp(b) => self.visit_bool_op(b),
            Expr::NamedExpr(n) => self.visit_named_expr(n),
            Expr::BinOp(b) => self.visit_bin_op(b),
            Expr::UnaryOp(u) => self.visit_unary_op(u),
            Expr::Lambda(l) => self.visit_lambda(l),
            Expr::IfExp(i) => self.visit_if_exp(i),
            Expr::Dict(d) => self.visit_dict(d),
            Expr::Set(s) => self.visit_set(s),
            Expr::ListComp(l) => self.visit_list_comp(l),
            Expr::SetComp(s) => self.visit_set_comp(s),
            Expr::DictComp(d) => self.visit_dict_comp(d),
            Expr::GeneratorExp(g) => self.visit_generator_exp(g),
            Expr::Await(a) => self.visit_await(a),
            Expr::Yield(y) => self.visit_yield(y),
            Expr::YieldFrom(y) => self.visit_yield_from(y),
            Expr::Compare(c) => self.visit_compare(c),
            Expr::Call(c) => self.visit_call(c),
            Expr::FormattedValue(f) => self.visit_formatted_value(f),
            Expr::JoinedStr(j) => self.visit_joined_str(j),
            Expr::Constant(c) => self.visit_constant(c),
            Expr::Attribute(a) => self.visit_attribute(a),
            Expr::Subscript(s) => self.visit_subscript(s),
            Expr::Starred(s) => self.visit_starred(s),
            Expr::Name(n) => self.visit_name(n),
            Expr::List(l) => self.visit_list(l),
            Expr::Tuple(t) => self.visit_tuple(t),
            Expr::Slice(s) => self.visit_slice(s),
        }
    }

    fn walk_pattern(&mut self, _pattern: &Pattern) -> T {
        T::default()
    }
}

