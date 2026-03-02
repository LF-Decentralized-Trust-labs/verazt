//! Module implementing the fold (left to right) pattern for Yul AST.
//!
//! This map pattern will compute output of the same type when traversing all
//! data structures in the AST.

use crate::ast::Name;

use crate::ast::yul::*;

/// Trait implementing the fold design pattern for Yul AST.
pub trait YulFold<T> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn fold_yul_source_unit(&self, acc: T, source_unit: &YulSourceUnit) -> T {
        yul_fold_default::fold_yul_source_unit(self, acc, source_unit)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn fold_yul_object(&self, acc: T, object: &YulObject) -> T {
        yul_fold_default::fold_yul_object(self, acc, object)
    }

    fn fold_yul_data(&self, acc: T, data: &YulData) -> T {
        yul_fold_default::fold_yul_data(self, acc, data)
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    fn fold_yul_code(&self, acc: T, code: &YulCode) -> T {
        yul_fold_default::fold_yul_code(self, acc, code)
    }

    fn fold_yul_block(&self, acc: T, block: &YulBlock) -> T {
        yul_fold_default::fold_yul_block(self, acc, block)
    }

    fn fold_yul_func_def(&self, acc: T, func: &YulFuncDef) -> T {
        yul_fold_default::fold_yul_func_def(self, acc, func)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn fold_yul_stmt(&self, acc: T, stmt: &YulStmt) -> T {
        yul_fold_default::fold_yul_stmt(self, acc, stmt)
    }

    fn fold_yul_assign_stmt(&self, acc: T, stmt: &YulAssignStmt) -> T {
        yul_fold_default::fold_yul_assign_stmt(self, acc, stmt)
    }

    fn fold_yul_if_stmt(&self, acc: T, stmt: &YulIfStmt) -> T {
        yul_fold_default::fold_yul_if_stmt(self, acc, stmt)
    }

    fn fold_yul_for_stmt(&self, acc: T, stmt: &YulForStmt) -> T {
        yul_fold_default::fold_yul_for_stmt(self, acc, stmt)
    }

    fn fold_yul_switch_stmt(&self, acc: T, stmt: &YulSwitchStmt) -> T {
        yul_fold_default::fold_yul_switch_stmt(self, acc, stmt)
    }

    fn fold_yul_switch_value(&self, acc: T, case: &YulSwitchValue) -> T {
        yul_fold_default::fold_yul_switch_value(self, acc, case)
    }

    fn fold_yul_switch_default(&self, acc: T, case: &YulSwitchDefault) -> T {
        yul_fold_default::fold_yul_switch_default(self, acc, case)
    }

    fn fold_yul_var_decl(&self, acc: T, var: &YulVarDecl) -> T {
        yul_fold_default::fold_yul_var_decl(self, acc, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn fold_yul_expr(&self, acc: T, expr: &YulExpr) -> T {
        yul_fold_default::fold_yul_expr(self, acc, expr)
    }

    fn fold_yul_call_expr(&self, acc: T, expr: &YulCallExpr) -> T {
        yul_fold_default::fold_yul_call_expr(self, acc, expr)
    }

    fn fold_yul_member_expr(&self, acc: T, expr: &YulMemberExpr) -> T {
        yul_fold_default::fold_yul_member_expr(self, acc, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn fold_yul_ident(&self, acc: T, id: &YulIdentifier) -> T {
        yul_fold_default::fold_yul_ident(self, acc, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn fold_yul_name(&self, acc: T, name: &Name) -> T {
        yul_fold_default::fold_yul_name(self, acc, name)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn fold_yul_lit(&self, acc: T, c: &YulLit) -> T {
        yul_fold_default::fold_yul_lit(self, acc, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn fold_yul_type(&self, acc: T, typ: &YulType) -> T {
        yul_fold_default::fold_yul_type(self, acc, typ)
    }
}

/// Module containing default implementation of the folding pattern for Yul AST.
pub mod yul_fold_default {
    use crate::ast::Name;
    use either::Either;

    use super::YulFold;
    use crate::ast::yul::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn fold_yul_source_unit<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        source_unit: &YulSourceUnit,
    ) -> T {
        folder.fold_yul_object(acc, &source_unit.main_object)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    pub fn fold_yul_object<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        object: &YulObject,
    ) -> T {
        let res = folder.fold_yul_code(acc, &object.code);
        object.children.iter().fold(res, |acc2, child| match child {
            Either::Left(obj) => folder.fold_yul_object(acc2, obj),
            Either::Right(data) => folder.fold_yul_data(acc2, data),
        })
    }

    pub fn fold_yul_data<T, F: YulFold<T> + ?Sized>(_folder: &F, acc: T, _data: &YulData) -> T {
        acc
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    pub fn fold_yul_code<T, F: YulFold<T> + ?Sized>(folder: &F, acc: T, code: &YulCode) -> T {
        folder.fold_yul_block(acc, &code.body)
    }

    pub fn fold_yul_block<T, F: YulFold<T> + ?Sized>(folder: &F, acc: T, block: &YulBlock) -> T {
        block
            .body
            .iter()
            .fold(acc, |acc2, stmt| folder.fold_yul_stmt(acc2, stmt))
    }

    pub fn fold_yul_func_def<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        func: &YulFuncDef,
    ) -> T {
        let res = func
            .params
            .iter()
            .fold(acc, |acc2, p| folder.fold_yul_ident(acc2, p));

        let res = func
            .returns
            .iter()
            .fold(res, |acc2, p| folder.fold_yul_ident(acc2, p));

        folder.fold_yul_block(res, &func.body)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn fold_yul_stmt<T, F: YulFold<T> + ?Sized>(folder: &F, acc: T, stmt: &YulStmt) -> T {
        use YulStmt::*;

        match stmt {
            Block(blk) => folder.fold_yul_block(acc, blk),
            FuncDef(func) => folder.fold_yul_func_def(acc, func),
            VarDecl(var) => folder.fold_yul_var_decl(acc, var),
            Assign(stmt) => folder.fold_yul_assign_stmt(acc, stmt),
            If(stmt) => folder.fold_yul_if_stmt(acc, stmt),
            For(stmt) => folder.fold_yul_for_stmt(acc, stmt),
            Switch(stmt) => folder.fold_yul_switch_stmt(acc, stmt),
            Break => acc,
            Continue => acc,
            Leave => acc,
            Expr(expr) => folder.fold_yul_expr(acc, expr),
        }
    }

    pub fn fold_yul_assign_stmt<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &YulAssignStmt,
    ) -> T {
        let res = stmt
            .vars
            .iter()
            .fold(acc, |acc2, id| folder.fold_yul_ident(acc2, id));

        folder.fold_yul_expr(res, &stmt.value)
    }

    pub fn fold_yul_if_stmt<T, F: YulFold<T> + ?Sized>(folder: &F, acc: T, stmt: &YulIfStmt) -> T {
        let res = folder.fold_yul_expr(acc, &stmt.cond);
        folder.fold_yul_block(res, &stmt.body)
    }

    pub fn fold_yul_for_stmt<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &YulForStmt,
    ) -> T {
        let res = folder.fold_yul_block(acc, &stmt.pre_loop);
        let res = folder.fold_yul_expr(res, &stmt.condition);
        let res = folder.fold_yul_block(res, &stmt.post_loop);
        folder.fold_yul_block(res, &stmt.body)
    }

    pub fn fold_yul_switch_stmt<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &YulSwitchStmt,
    ) -> T {
        let res = folder.fold_yul_expr(acc, &stmt.expr);

        let res = stmt
            .values
            .iter()
            .fold(res, |acc2, case| folder.fold_yul_switch_value(acc2, case));

        match &stmt.default {
            Some(case) => folder.fold_yul_switch_default(res, case),
            None => res,
        }
    }

    pub fn fold_yul_switch_value<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        case: &YulSwitchValue,
    ) -> T {
        let res = folder.fold_yul_lit(acc, &case.literal);
        folder.fold_yul_block(res, &case.body)
    }

    pub fn fold_yul_switch_default<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        case: &YulSwitchDefault,
    ) -> T {
        folder.fold_yul_block(acc, &case.body)
    }

    pub fn fold_yul_var_decl<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        vdecl: &YulVarDecl,
    ) -> T {
        // Fold the assigned value first.
        let res = match &vdecl.value {
            Some(expr) => folder.fold_yul_expr(acc, expr),
            None => acc,
        };

        vdecl
            .vars
            .iter()
            .fold(res, |acc2, id| folder.fold_yul_ident(acc2, id))
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn fold_yul_expr<T, F: YulFold<T> + ?Sized>(folder: &F, acc: T, expr: &YulExpr) -> T {
        match expr {
            YulExpr::Lit(lit) => folder.fold_yul_lit(acc, lit),
            YulExpr::Ident(ident) => folder.fold_yul_ident(acc, ident),
            YulExpr::Call(expr) => folder.fold_yul_call_expr(acc, expr),
            YulExpr::Member(expr) => folder.fold_yul_member_expr(acc, expr),
        }
    }

    pub fn fold_yul_call_expr<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &YulCallExpr,
    ) -> T {
        let res = folder.fold_yul_ident(acc, &expr.callee);
        expr.args
            .iter()
            .fold(res, |acc2, arg| folder.fold_yul_expr(acc2, arg))
    }

    pub fn fold_yul_member_expr<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &YulMemberExpr,
    ) -> T {
        let res = folder.fold_yul_name(acc, &expr.base);
        folder.fold_yul_name(res, &expr.member)
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn fold_yul_ident<T, F: YulFold<T> + ?Sized>(
        folder: &F,
        acc: T,
        ident: &YulIdentifier,
    ) -> T {
        let res = folder.fold_yul_name(acc, &ident.name);
        folder.fold_yul_type(res, &ident.typ)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    pub fn fold_yul_name<T, F: YulFold<T> + ?Sized>(_folder: &F, acc: T, _name: &Name) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn fold_yul_lit<T, F: YulFold<T> + ?Sized>(_folder: &F, acc: T, _lit: &YulLit) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn fold_yul_type<T, F: YulFold<T> + ?Sized>(_folder: &F, acc: T, _typ: &YulType) -> T {
        // By default, return the accumulative value
        acc
    }
}
