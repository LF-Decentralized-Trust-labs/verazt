//! Module implementing the fold (left to right) pattern for Yul AST.
//!
//! This map pattern will compute output of the same type when traversing all
//! data structures in the AST.

use meta::Name;

use crate::ast::*;

/// Trait implementing the map design pattern for Solidity AST.
pub trait Fold<T> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn fold_source_unit(&self, acc: T, source_unit: &SourceUnit) -> T {
        default::fold_source_unit(self, acc, source_unit)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn fold_object(&self, acc: T, object: &Object) -> T {
        default::fold_object(self, acc, object)
    }

    fn fold_data(&self, acc: T, data: &Data) -> T {
        default::fold_data(self, acc, data)
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    fn fold_code(&self, acc: T, code: &Code) -> T {
        default::fold_code(self, acc, code)
    }

    fn fold_block(&self, acc: T, block: &Block) -> T {
        default::fold_block(self, acc, block)
    }

    fn fold_func_def(&self, acc: T, func: &FuncDef) -> T {
        default::fold_func_def(self, acc, func)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn fold_stmt(&self, acc: T, stmt: &Stmt) -> T {
        default::fold_stmt(self, acc, stmt)
    }

    fn fold_assign_stmt(&self, acc: T, stmt: &AssignStmt) -> T {
        default::fold_assign_stmt(self, acc, stmt)
    }

    fn fold_if_stmt(&self, acc: T, stmt: &IfStmt) -> T {
        default::fold_if_stmt(self, acc, stmt)
    }

    fn fold_for_stmt(&self, acc: T, stmt: &ForStmt) -> T {
        default::fold_for_stmt(self, acc, stmt)
    }

    fn fold_switch_stmt(&self, acc: T, stmt: &SwitchStmt) -> T {
        default::fold_switch_stmt(self, acc, stmt)
    }

    fn fold_switch_value(&self, acc: T, case: &SwitchValue) -> T {
        default::fold_switch_value(self, acc, case)
    }

    fn fold_switch_default(&self, acc: T, case: &SwitchDefault) -> T {
        default::fold_switch_default(self, acc, case)
    }

    fn fold_var_decl(&self, acc: T, var: &VarDecl) -> T {
        default::fold_var_decl(self, acc, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn fold_expr(&self, acc: T, expr: &Expr) -> T {
        default::fold_expr(self, acc, expr)
    }

    fn fold_call_expr(&self, acc: T, expr: &CallExpr) -> T {
        default::fold_call_expr(self, acc, expr)
    }

    fn fold_member_expr(&self, acc: T, expr: &MemberExpr) -> T {
        default::fold_member_expr(self, acc, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn fold_ident(&self, acc: T, id: &Identifier) -> T {
        default::fold_ident(self, acc, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn fold_name(&self, acc: T, name: &Name) -> T {
        default::fold_name(self, acc, name)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn fold_lit(&self, acc: T, c: &Lit) -> T {
        default::fold_lit(self, acc, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn fold_type(&self, acc: T, typ: &Type) -> T {
        default::fold_type(self, acc, typ)
    }
}

/// Module contain default implementation of the folding pattern.
pub mod default {
    use either::Either;
    use meta::Name;

    use super::Fold;
    use crate::ast::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn fold_source_unit<T, F: Fold<T> + ?Sized>(
        folder: &F,
        acc: T,
        source_unit: &SourceUnit,
    ) -> T {
        folder.fold_object(acc, &source_unit.main_object)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    pub fn fold_object<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, object: &Object) -> T {
        let res = folder.fold_code(acc, &object.code);
        object.children.iter().fold(res, |acc2, child| match child {
            Either::Left(obj) => folder.fold_object(acc2, obj),
            Either::Right(data) => folder.fold_data(acc2, data),
        })
    }

    pub fn fold_data<T, F: Fold<T> + ?Sized>(_folder: &F, acc: T, _data: &Data) -> T {
        acc
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    pub fn fold_code<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, code: &Code) -> T {
        folder.fold_block(acc, &code.body)
    }

    pub fn fold_block<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, block: &Block) -> T {
        block
            .body
            .iter()
            .fold(acc, |acc2, stmt| folder.fold_stmt(acc2, stmt))
    }

    pub fn fold_func_def<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, func: &FuncDef) -> T {
        let res = func
            .params
            .iter()
            .fold(acc, |acc2, p| folder.fold_ident(acc2, p));

        let res = func
            .returns
            .iter()
            .fold(res, |acc2, p| folder.fold_ident(acc2, p));

        folder.fold_block(res, &func.body)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn fold_stmt<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, stmt: &Stmt) -> T {
        use Stmt::*;

        match stmt {
            Block(blk) => folder.fold_block(acc, blk),
            FuncDef(func) => folder.fold_func_def(acc, func),
            VarDecl(var) => folder.fold_var_decl(acc, var),
            Assign(stmt) => folder.fold_assign_stmt(acc, stmt),
            If(stmt) => folder.fold_if_stmt(acc, stmt),
            For(stmt) => folder.fold_for_stmt(acc, stmt),
            Switch(stmt) => folder.fold_switch_stmt(acc, stmt),
            Break => acc,
            Continue => acc,
            Leave => acc,
            Expr(expr) => folder.fold_expr(acc, expr),
        }
    }

    pub fn fold_assign_stmt<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, stmt: &AssignStmt) -> T {
        let res = stmt
            .vars
            .iter()
            .fold(acc, |acc2, id| folder.fold_ident(acc2, id));

        folder.fold_expr(res, &stmt.value)
    }

    pub fn fold_if_stmt<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, stmt: &IfStmt) -> T {
        let res = folder.fold_expr(acc, &stmt.cond);
        folder.fold_block(res, &stmt.body)
    }

    pub fn fold_for_stmt<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, stmt: &ForStmt) -> T {
        let res = folder.fold_block(acc, &stmt.pre_loop);
        let res = folder.fold_expr(res, &stmt.condition);
        let res = folder.fold_block(res, &stmt.post_loop);
        folder.fold_block(res, &stmt.body)
    }

    pub fn fold_switch_stmt<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, stmt: &SwitchStmt) -> T {
        let res = folder.fold_expr(acc, &stmt.expr);

        let res = stmt
            .values
            .iter()
            .fold(res, |acc2, case| folder.fold_switch_value(acc2, case));

        match &stmt.default {
            Some(case) => folder.fold_switch_default(res, case),
            None => res,
        }
    }

    pub fn fold_switch_value<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, case: &SwitchValue) -> T {
        let res = folder.fold_lit(acc, &case.literal);
        folder.fold_block(res, &case.body)
    }

    pub fn fold_switch_default<T, F: Fold<T> + ?Sized>(
        folder: &F,
        acc: T,
        case: &SwitchDefault,
    ) -> T {
        folder.fold_block(acc, &case.body)
    }

    pub fn fold_var_decl<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, vdecl: &VarDecl) -> T {
        // Fold the assigned value first.
        let res = match &vdecl.value {
            Some(expr) => folder.fold_expr(acc, expr),
            None => acc,
        };

        vdecl
            .vars
            .iter()
            .fold(res, |acc2, id| folder.fold_ident(acc2, id))
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn fold_expr<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, expr: &Expr) -> T {
        match expr {
            Expr::Lit(lit) => folder.fold_lit(acc, lit),
            Expr::Ident(ident) => folder.fold_ident(acc, ident),
            Expr::Call(expr) => folder.fold_call_expr(acc, expr),
            Expr::Member(_) => todo!(),
        }
    }

    pub fn fold_call_expr<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, expr: &CallExpr) -> T {
        let res = folder.fold_ident(acc, &expr.callee);
        expr.args
            .iter()
            .fold(res, |acc2, arg| folder.fold_expr(acc2, arg))
    }

    pub fn fold_member_expr<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, expr: &MemberExpr) -> T {
        let res = folder.fold_name(acc, &expr.base);
        folder.fold_name(res, &expr.member)
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn fold_ident<T, F: Fold<T> + ?Sized>(folder: &F, acc: T, ident: &Identifier) -> T {
        let res = folder.fold_name(acc, &ident.name);
        folder.fold_type(res, &ident.typ)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    pub fn fold_name<T, F: Fold<T> + ?Sized>(_folder: &F, acc: T, _name: &Name) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn fold_lit<T, F: Fold<T> + ?Sized>(_folder: &F, acc: T, _lit: &Lit) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn fold_type<T, F: Fold<T> + ?Sized>(_folder: &F, acc: T, _typ: &Type) -> T {
        // By default, return the accumulative value
        acc
    }
}
