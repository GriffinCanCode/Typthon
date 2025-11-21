use crate::compiler::ast::visitor::AstVisitor;
use rustpython_parser::ast::*;
use rustpython_parser::ast::ExceptHandler;

/// Default AST walker that traverses the entire tree
pub struct DefaultWalker;

impl AstVisitor for DefaultWalker {
    fn visit_function_def(&mut self, func: &StmtFunctionDef) -> () {
        for arg in &func.args.args {
            if let Some(annotation) = &arg.def.annotation {
                self.visit_expr(annotation);
            }
        }
        if let Some(returns) = &func.returns {
            self.visit_expr(returns);
        }
        for stmt in &func.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_async_function_def(&mut self, func: &StmtAsyncFunctionDef) -> () {
        for arg in &func.args.args {
            if let Some(annotation) = &arg.def.annotation {
                self.visit_expr(annotation);
            }
        }
        if let Some(returns) = &func.returns {
            self.visit_expr(returns);
        }
        for stmt in &func.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_class_def(&mut self, class: &StmtClassDef) -> () {
        for base in &class.bases {
            self.visit_expr(base);
        }
        for stmt in &class.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_return(&mut self, ret: &StmtReturn) -> () {
        if let Some(value) = &ret.value {
            self.visit_expr(value);
        }
    }

    fn visit_delete(&mut self, del: &StmtDelete) -> () {
        for target in &del.targets {
            self.visit_expr(target);
        }
    }

    fn visit_assign(&mut self, assign: &StmtAssign) -> () {
        for target in &assign.targets {
            self.visit_expr(target);
        }
        self.visit_expr(&assign.value);
    }

    fn visit_aug_assign(&mut self, aug: &StmtAugAssign) -> () {
        self.visit_expr(&aug.target);
        self.visit_expr(&aug.value);
    }

    fn visit_ann_assign(&mut self, ann: &StmtAnnAssign) -> () {
        self.visit_expr(&ann.target);
        self.visit_expr(&ann.annotation);
        if let Some(value) = &ann.value {
            self.visit_expr(value);
        }
    }

    fn visit_for(&mut self, for_stmt: &StmtFor) -> () {
        self.visit_expr(&for_stmt.target);
        self.visit_expr(&for_stmt.iter);
        for stmt in &for_stmt.body {
            self.visit_stmt(stmt);
        }
        for stmt in &for_stmt.orelse {
            self.visit_stmt(stmt);
        }
    }

    fn visit_async_for(&mut self, for_stmt: &StmtAsyncFor) -> () {
        self.visit_expr(&for_stmt.target);
        self.visit_expr(&for_stmt.iter);
        for stmt in &for_stmt.body {
            self.visit_stmt(stmt);
        }
        for stmt in &for_stmt.orelse {
            self.visit_stmt(stmt);
        }
    }

    fn visit_while(&mut self, while_stmt: &StmtWhile) -> () {
        self.visit_expr(&while_stmt.test);
        for stmt in &while_stmt.body {
            self.visit_stmt(stmt);
        }
        for stmt in &while_stmt.orelse {
            self.visit_stmt(stmt);
        }
    }

    fn visit_if(&mut self, if_stmt: &StmtIf) -> () {
        self.visit_expr(&if_stmt.test);
        for stmt in &if_stmt.body {
            self.visit_stmt(stmt);
        }
        for stmt in &if_stmt.orelse {
            self.visit_stmt(stmt);
        }
    }

    fn visit_with(&mut self, with: &StmtWith) -> () {
        for item in &with.items {
            self.visit_expr(&item.context_expr);
            if let Some(optional_vars) = &item.optional_vars {
                self.visit_expr(optional_vars);
            }
        }
        for stmt in &with.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_async_with(&mut self, with: &StmtAsyncWith) -> () {
        for item in &with.items {
            self.visit_expr(&item.context_expr);
            if let Some(optional_vars) = &item.optional_vars {
                self.visit_expr(optional_vars);
            }
        }
        for stmt in &with.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_match(&mut self, match_stmt: &StmtMatch) -> () {
        self.visit_expr(&match_stmt.subject);
        for case in &match_stmt.cases {
            self.visit_pattern(&case.pattern);
            if let Some(guard) = &case.guard {
                self.visit_expr(guard);
            }
            for stmt in &case.body {
                self.visit_stmt(stmt);
            }
        }
    }

    fn visit_raise(&mut self, raise: &StmtRaise) -> () {
        if let Some(exc) = &raise.exc {
            self.visit_expr(exc);
        }
        if let Some(cause) = &raise.cause {
            self.visit_expr(cause);
        }
    }

    fn visit_try(&mut self, try_stmt: &StmtTry) -> () {
        for stmt in &try_stmt.body {
            self.visit_stmt(stmt);
        }
        for handler in &try_stmt.handlers {
            let ExceptHandler::ExceptHandler(h) = handler;
            if let Some(ty) = &h.type_ {
                self.visit_expr(ty);
            }
            for stmt in &h.body {
                self.visit_stmt(stmt);
            }
        }
        for stmt in &try_stmt.orelse {
            self.visit_stmt(stmt);
        }
        for stmt in &try_stmt.finalbody {
            self.visit_stmt(stmt);
        }
    }

    fn visit_assert(&mut self, assert: &StmtAssert) -> () {
        self.visit_expr(&assert.test);
        if let Some(msg) = &assert.msg {
            self.visit_expr(msg);
        }
    }

    fn visit_import(&mut self, _import: &StmtImport) -> () {}

    fn visit_import_from(&mut self, _import: &StmtImportFrom) -> () {}

    fn visit_global(&mut self, _global: &StmtGlobal) -> () {}

    fn visit_nonlocal(&mut self, _nonlocal: &StmtNonlocal) -> () {}

    fn visit_expr_stmt(&mut self, expr: &StmtExpr) -> () {
        self.visit_expr(&expr.value);
    }

    fn visit_pass(&mut self, _pass: &StmtPass) -> () {}

    fn visit_break(&mut self, _break_stmt: &StmtBreak) -> () {}

    fn visit_continue(&mut self, _cont: &StmtContinue) -> () {}

    fn visit_type_alias(&mut self, type_alias: &StmtTypeAlias) -> () {
        self.visit_expr(&type_alias.value);
    }

    fn visit_try_star(&mut self, try_star: &StmtTryStar) -> () {
        for stmt in &try_star.body {
            self.visit_stmt(stmt);
        }
        for handler in &try_star.handlers {
            let ExceptHandler::ExceptHandler(h) = handler;
            if let Some(ty) = &h.type_ {
                self.visit_expr(ty);
            }
            for stmt in &h.body {
                self.visit_stmt(stmt);
            }
        }
        for stmt in &try_star.orelse {
            self.visit_stmt(stmt);
        }
        for stmt in &try_star.finalbody {
            self.visit_stmt(stmt);
        }
    }

    fn visit_bool_op(&mut self, bool_op: &ExprBoolOp) -> () {
        for value in &bool_op.values {
            self.visit_expr(value);
        }
    }

    fn visit_named_expr(&mut self, named: &ExprNamedExpr) -> () {
        self.visit_expr(&named.target);
        self.visit_expr(&named.value);
    }

    fn visit_bin_op(&mut self, bin_op: &ExprBinOp) -> () {
        self.visit_expr(&bin_op.left);
        self.visit_expr(&bin_op.right);
    }

    fn visit_unary_op(&mut self, unary: &ExprUnaryOp) -> () {
        self.visit_expr(&unary.operand);
    }

    fn visit_lambda(&mut self, lambda: &ExprLambda) -> () {
        self.visit_expr(&lambda.body);
    }

    fn visit_if_exp(&mut self, if_exp: &ExprIfExp) -> () {
        self.visit_expr(&if_exp.test);
        self.visit_expr(&if_exp.body);
        self.visit_expr(&if_exp.orelse);
    }

    fn visit_dict(&mut self, dict: &ExprDict) -> () {
        for key in &dict.keys {
            if let Some(k) = key {
                self.visit_expr(k);
            }
        }
        for value in &dict.values {
            self.visit_expr(value);
        }
    }

    fn visit_set(&mut self, set: &ExprSet) -> () {
        for elt in &set.elts {
            self.visit_expr(elt);
        }
    }

    fn visit_list_comp(&mut self, comp: &ExprListComp) -> () {
        self.visit_expr(&comp.elt);
        for gen in &comp.generators {
            self.visit_expr(&gen.target);
            self.visit_expr(&gen.iter);
            for cond in &gen.ifs {
                self.visit_expr(cond);
            }
        }
    }

    fn visit_set_comp(&mut self, comp: &ExprSetComp) -> () {
        self.visit_expr(&comp.elt);
        for gen in &comp.generators {
            self.visit_expr(&gen.target);
            self.visit_expr(&gen.iter);
            for cond in &gen.ifs {
                self.visit_expr(cond);
            }
        }
    }

    fn visit_dict_comp(&mut self, comp: &ExprDictComp) -> () {
        self.visit_expr(&comp.key);
        self.visit_expr(&comp.value);
        for gen in &comp.generators {
            self.visit_expr(&gen.target);
            self.visit_expr(&gen.iter);
            for cond in &gen.ifs {
                self.visit_expr(cond);
            }
        }
    }

    fn visit_generator_exp(&mut self, gen: &ExprGeneratorExp) -> () {
        self.visit_expr(&gen.elt);
        for generator in &gen.generators {
            self.visit_expr(&generator.target);
            self.visit_expr(&generator.iter);
            for cond in &generator.ifs {
                self.visit_expr(cond);
            }
        }
    }

    fn visit_await(&mut self, await_expr: &ExprAwait) -> () {
        self.visit_expr(&await_expr.value);
    }

    fn visit_yield(&mut self, yield_expr: &ExprYield) -> () {
        if let Some(value) = &yield_expr.value {
            self.visit_expr(value);
        }
    }

    fn visit_yield_from(&mut self, yield_from: &ExprYieldFrom) -> () {
        self.visit_expr(&yield_from.value);
    }

    fn visit_compare(&mut self, compare: &ExprCompare) -> () {
        self.visit_expr(&compare.left);
        for comparator in &compare.comparators {
            self.visit_expr(comparator);
        }
    }

    fn visit_call(&mut self, call: &ExprCall) -> () {
        self.visit_expr(&call.func);
        for arg in &call.args {
            self.visit_expr(arg);
        }
    }

    fn visit_formatted_value(&mut self, formatted: &ExprFormattedValue) -> () {
        self.visit_expr(&formatted.value);
    }

    fn visit_joined_str(&mut self, joined: &ExprJoinedStr) -> () {
        for value in &joined.values {
            self.visit_expr(value);
        }
    }

    fn visit_constant(&mut self, _constant: &ExprConstant) -> () {}

    fn visit_attribute(&mut self, attr: &ExprAttribute) -> () {
        self.visit_expr(&attr.value);
    }

    fn visit_subscript(&mut self, subscript: &ExprSubscript) -> () {
        self.visit_expr(&subscript.value);
        self.visit_expr(&subscript.slice);
    }

    fn visit_starred(&mut self, starred: &ExprStarred) -> () {
        self.visit_expr(&starred.value);
    }

    fn visit_name(&mut self, _name: &ExprName) -> () {}

    fn visit_list(&mut self, list: &ExprList) -> () {
        for elt in &list.elts {
            self.visit_expr(elt);
        }
    }

    fn visit_tuple(&mut self, tuple: &ExprTuple) -> () {
        for elt in &tuple.elts {
            self.visit_expr(elt);
        }
    }

    fn visit_slice(&mut self, slice: &ExprSlice) -> () {
        if let Some(lower) = &slice.lower {
            self.visit_expr(lower);
        }
        if let Some(upper) = &slice.upper {
            self.visit_expr(upper);
        }
        if let Some(step) = &slice.step {
            self.visit_expr(step);
        }
    }
}

