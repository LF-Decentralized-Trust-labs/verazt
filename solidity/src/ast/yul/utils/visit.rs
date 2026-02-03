//! Module implementing the visit (left to right) pattern for Yul AST.
//!
//! This visiting pattern will not transform any input data structure.

use meta::Name;

use crate::ast::yul::*;

/// Trait implementing the visit design pattern for Yul AST.
pub trait YulVisit {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn visit_yul_source_unit(&mut self, source_unit: &YulSourceUnit) {
        yul_visit_default::visit_yul_source_unit(self, source_unit)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn visit_yul_object(&mut self, object: &YulObject) {
        yul_visit_default::visit_yul_object(self, object)
    }

    fn visit_yul_data(&mut self, data: &YulData) {
        yul_visit_default::visit_yul_data(self, data)
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    fn visit_yul_code(&mut self, code: &YulCode) {
        yul_visit_default::visit_yul_code(self, code)
    }

    fn visit_yul_block(&mut self, block: &YulBlock) {
        yul_visit_default::visit_yul_block(self, block)
    }

    fn visit_yul_func_def(&mut self, func: &YulFuncDef) {
        yul_visit_default::visit_yul_func_def(self, func)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn visit_yul_stmt(&mut self, stmt: &YulStmt) {
        yul_visit_default::visit_yul_stmt(self, stmt)
    }

    fn visit_yul_assign_stmt(&mut self, stmt: &YulAssignStmt) {
        yul_visit_default::visit_yul_assign_stmt(self, stmt)
    }

    fn visit_yul_if_stmt(&mut self, stmt: &YulIfStmt) {
        yul_visit_default::visit_yul_if_stmt(self, stmt)
    }

    fn visit_yul_for_stmt(&mut self, stmt: &YulForStmt) {
        yul_visit_default::visit_yul_for_stmt(self, stmt)
    }

    fn visit_yul_switch_stmt(&mut self, stmt: &YulSwitchStmt) {
        yul_visit_default::visit_yul_switch_stmt(self, stmt)
    }

    fn visit_yul_switch_value(&mut self, case: &YulSwitchValue) {
        yul_visit_default::visit_yul_switch_value(self, case)
    }

    fn visit_yul_switch_default(&mut self, case: &YulSwitchDefault) {
        yul_visit_default::visit_yul_switch_default(self, case)
    }

    fn visit_yul_var_decl(&mut self, var: &YulVarDecl) {
        yul_visit_default::visit_yul_var_decl(self, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn visit_yul_expr(&mut self, expr: &YulExpr) {
        yul_visit_default::visit_yul_expr(self, expr)
    }

    fn visit_yul_call_expr(&mut self, expr: &YulCallExpr) {
        yul_visit_default::visit_yul_call_expr(self, expr)
    }

    fn visit_yul_member_expr(&mut self, expr: &YulMemberExpr) {
        yul_visit_default::visit_yul_member_expr(self, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn visit_yul_ident(&mut self, id: &YulIdentifier) {
        yul_visit_default::visit_yul_ident(self, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn visit_yul_name(&mut self, _name: &Name) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn visit_yul_lit(&mut self, c: &YulLit) {
        yul_visit_default::visit_yul_lit(self, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn visit_yul_type(&mut self, typ: &YulType) {
        yul_visit_default::visit_yul_type(self, typ)
    }
}

//------------------------------------------------------------------
// Default visiting pattern for Yul AST
//------------------------------------------------------------------

/// Module containing default implementation of the visiting pattern for Yul AST.
pub mod yul_visit_default {
    use either::Either;

    use super::YulVisit;
    use crate::ast::yul::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn visit_yul_source_unit<T: YulVisit + ?Sized>(visitor: &mut T, source_unit: &YulSourceUnit) {
        visitor.visit_yul_object(&source_unit.main_object)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    pub fn visit_yul_object<T: YulVisit + ?Sized>(visitor: &mut T, object: &YulObject) {
        visitor.visit_yul_code(&object.code);

        object.children.iter().for_each(|child| match child {
            Either::Left(obj) => visitor.visit_yul_object(obj),
            Either::Right(data) => visitor.visit_yul_data(data),
        })
    }

    pub fn visit_yul_data<T: YulVisit + ?Sized>(_visitor: &mut T, _data: &YulData) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    pub fn visit_yul_code<T: YulVisit + ?Sized>(visitor: &mut T, code: &YulCode) {
        visitor.visit_yul_block(&code.body)
    }

    pub fn visit_yul_block<T: YulVisit + ?Sized>(visitor: &mut T, block: &YulBlock) {
        block.body.iter().for_each(|stmt| visitor.visit_yul_stmt(stmt))
    }

    pub fn visit_yul_func_def<T: YulVisit + ?Sized>(visitor: &mut T, func: &YulFuncDef) {
        func.params.iter().for_each(|p| visitor.visit_yul_ident(p));

        func.returns.iter().for_each(|p| visitor.visit_yul_ident(p));

        visitor.visit_yul_block(&func.body)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn visit_yul_stmt<T: YulVisit + ?Sized>(visitor: &mut T, stmt: &YulStmt) {
        use YulStmt::*;

        match stmt {
            Block(blk) => visitor.visit_yul_block(blk),
            FuncDef(func) => visitor.visit_yul_func_def(func),
            VarDecl(var) => visitor.visit_yul_var_decl(var),
            Assign(stmt) => visitor.visit_yul_assign_stmt(stmt),
            If(stmt) => visitor.visit_yul_if_stmt(stmt),
            For(stmt) => visitor.visit_yul_for_stmt(stmt),
            Switch(stmt) => visitor.visit_yul_switch_stmt(stmt),
            Break => {}
            Continue => {}
            Leave => {}
            Expr(expr) => visitor.visit_yul_expr(expr),
        }
    }

    pub fn visit_yul_assign_stmt<T: YulVisit + ?Sized>(visitor: &mut T, stmt: &YulAssignStmt) {
        stmt.vars.iter().for_each(|id| visitor.visit_yul_ident(id));

        visitor.visit_yul_expr(&stmt.value)
    }

    pub fn visit_yul_if_stmt<T: YulVisit + ?Sized>(visitor: &mut T, stmt: &YulIfStmt) {
        visitor.visit_yul_expr(&stmt.cond);
        visitor.visit_yul_block(&stmt.body)
    }

    pub fn visit_yul_for_stmt<T: YulVisit + ?Sized>(visitor: &mut T, stmt: &YulForStmt) {
        visitor.visit_yul_block(&stmt.pre_loop);
        visitor.visit_yul_expr(&stmt.condition);
        visitor.visit_yul_block(&stmt.post_loop);
        visitor.visit_yul_block(&stmt.body)
    }

    pub fn visit_yul_switch_stmt<T: YulVisit + ?Sized>(visitor: &mut T, stmt: &YulSwitchStmt) {
        visitor.visit_yul_expr(&stmt.expr);

        stmt.values
            .iter()
            .for_each(|case| visitor.visit_yul_switch_value(case));

        if let Some(case) = &stmt.default {
            visitor.visit_yul_switch_default(case)
        }
    }

    pub fn visit_yul_switch_value<T: YulVisit + ?Sized>(visitor: &mut T, case: &YulSwitchValue) {
        visitor.visit_yul_lit(&case.literal);
        visitor.visit_yul_block(&case.body)
    }

    pub fn visit_yul_switch_default<T: YulVisit + ?Sized>(visitor: &mut T, case: &YulSwitchDefault) {
        visitor.visit_yul_block(&case.body)
    }

    pub fn visit_yul_var_decl<T: YulVisit + ?Sized>(visitor: &mut T, vdecl: &YulVarDecl) {
        // Visit the assigned value first.
        vdecl.value.iter().for_each(|e| visitor.visit_yul_expr(e));

        vdecl.vars.iter().for_each(|id| visitor.visit_yul_ident(id));
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn visit_yul_expr<T: YulVisit + ?Sized>(visitor: &mut T, expr: &YulExpr) {
        use YulExpr::*;
        match expr {
            Lit(lit) => visitor.visit_yul_lit(lit),
            Ident(ident) => visitor.visit_yul_ident(ident),
            Call(expr) => visitor.visit_yul_call_expr(expr),
            Member(expr) => visitor.visit_yul_member_expr(expr),
        }
    }

    pub fn visit_yul_call_expr<T: YulVisit + ?Sized>(visitor: &mut T, expr: &YulCallExpr) {
        visitor.visit_yul_ident(&expr.callee);
        expr.args.iter().for_each(|arg| visitor.visit_yul_expr(arg))
    }

    pub fn visit_yul_member_expr<T: YulVisit + ?Sized>(visitor: &mut T, expr: &YulMemberExpr) {
        visitor.visit_yul_name(&expr.base);
        visitor.visit_yul_name(&expr.member)
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn visit_yul_ident<T: YulVisit + ?Sized>(visitor: &mut T, id: &YulIdentifier) {
        visitor.visit_yul_type(&id.typ)
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn visit_yul_lit<T: YulVisit + ?Sized>(_visitor: &mut T, _lit: &YulLit) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn visit_yul_type<T: YulVisit + ?Sized>(_visitor: &mut T, _typ: &YulType) {
        // Do nothing by default
    }
}
