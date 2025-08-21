//! Module implementing the visit (left to right) pattern for Yulc AST.
//!
//! This visiting pattern will not transform any input data structure.

use meta::Name;

use crate::ast::*;

/// Trait implementing the map design pattern for Solidity AST.
pub trait Visit {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) {
        default::visit_source_unit(self, source_unit)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn visit_object(&mut self, object: &Object) {
        default::visit_object(self, object)
    }

    fn visit_data(&mut self, data: &Data) {
        default::visit_data(self, data)
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    fn visit_code(&mut self, code: &Code) {
        default::visit_code(self, code)
    }

    fn visit_block(&mut self, block: &Block) {
        default::visit_block(self, block)
    }

    fn visit_func_def(&mut self, func: &FuncDef) {
        default::visit_func_def(self, func)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn visit_stmt(&mut self, stmt: &Stmt) {
        default::visit_stmt(self, stmt)
    }

    fn visit_assign_stmt(&mut self, stmt: &AssignStmt) {
        default::visit_assign_stmt(self, stmt)
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) {
        default::visit_if_stmt(self, stmt)
    }

    fn visit_for_stmt(&mut self, stmt: &ForStmt) {
        default::visit_for_stmt(self, stmt)
    }

    fn visit_switch_stmt(&mut self, stmt: &SwitchStmt) {
        default::visit_switch_stmt(self, stmt)
    }

    fn visit_switch_value(&mut self, case: &SwitchValue) {
        default::visit_switch_value(self, case)
    }

    fn visit_switch_default(&mut self, case: &SwitchDefault) {
        default::visit_switch_default(self, case)
    }

    fn visit_var_decl(&mut self, var: &VarDecl) {
        default::visit_var_decl(self, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn visit_expr(&mut self, expr: &Expr) {
        default::visit_expr(self, expr)
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) {
        default::visit_call_expr(self, expr)
    }

    fn visit_member_expr(&mut self, expr: &MemberExpr) {
        default::visit_member_expr(self, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn visit_ident(&mut self, id: &Identifier) {
        default::visit_ident(self, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn visit_name(&mut self, _name: &Name) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn visit_lit(&mut self, c: &Lit) {
        default::visit_lit(self, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn visit_type(&mut self, typ: &Type) {
        default::visit_type(self, typ)
    }
}

//------------------------------------------------------------------
// Default visiting pattern
//------------------------------------------------------------------

/// Module contain default implementation of the visiting pattern.
pub mod default {
    use either::Either;

    use super::Visit;
    use crate::ast::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn visit_source_unit<T: Visit + ?Sized>(visitor: &mut T, source_unit: &SourceUnit) {
        visitor.visit_object(&source_unit.main_object)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    pub fn visit_object<T: Visit + ?Sized>(visitor: &mut T, object: &Object) {
        visitor.visit_code(&object.code);

        object.children.iter().for_each(|child| match child {
            Either::Left(obj) => visitor.visit_object(obj),
            Either::Right(data) => visitor.visit_data(data),
        })
    }

    pub fn visit_data<T: Visit + ?Sized>(_visitor: &mut T, _data: &Data) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    pub fn visit_code<T: Visit + ?Sized>(visitor: &mut T, code: &Code) {
        visitor.visit_block(&code.body)
    }

    pub fn visit_block<T: Visit + ?Sized>(visitor: &mut T, block: &Block) {
        block.body.iter().for_each(|stmt| visitor.visit_stmt(stmt))
    }

    pub fn visit_func_def<T: Visit + ?Sized>(visitor: &mut T, func: &FuncDef) {
        func.params.iter().for_each(|p| visitor.visit_ident(p));

        func.returns.iter().for_each(|p| visitor.visit_ident(p));

        visitor.visit_block(&func.body)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn visit_stmt<T: Visit + ?Sized>(visitor: &mut T, stmt: &Stmt) {
        use Stmt::*;

        match stmt {
            Block(blk) => visitor.visit_block(blk),
            FuncDef(func) => visitor.visit_func_def(func),
            VarDecl(var) => visitor.visit_var_decl(var),
            Assign(stmt) => visitor.visit_assign_stmt(stmt),
            If(stmt) => visitor.visit_if_stmt(stmt),
            For(stmt) => visitor.visit_for_stmt(stmt),
            Switch(stmt) => visitor.visit_switch_stmt(stmt),
            Break => {}
            Continue => {}
            Leave => {}
            Expr(expr) => visitor.visit_expr(expr),
        }
    }

    pub fn visit_assign_stmt<T: Visit + ?Sized>(visitor: &mut T, stmt: &AssignStmt) {
        stmt.vars.iter().for_each(|id| visitor.visit_ident(id));

        visitor.visit_expr(&stmt.value)
    }

    pub fn visit_if_stmt<T: Visit + ?Sized>(visitor: &mut T, stmt: &IfStmt) {
        visitor.visit_expr(&stmt.cond);
        visitor.visit_block(&stmt.body)
    }

    pub fn visit_for_stmt<T: Visit + ?Sized>(visitor: &mut T, stmt: &ForStmt) {
        visitor.visit_block(&stmt.pre_loop);
        visitor.visit_expr(&stmt.condition);
        visitor.visit_block(&stmt.post_loop);
        visitor.visit_block(&stmt.body)
    }

    pub fn visit_switch_stmt<T: Visit + ?Sized>(visitor: &mut T, stmt: &SwitchStmt) {
        visitor.visit_expr(&stmt.expr);

        stmt.values
            .iter()
            .for_each(|case| visitor.visit_switch_value(case));

        if let Some(case) = &stmt.default {
            visitor.visit_switch_default(case)
        }
    }

    pub fn visit_switch_value<T: Visit + ?Sized>(visitor: &mut T, case: &SwitchValue) {
        visitor.visit_lit(&case.literal);
        visitor.visit_block(&case.body)
    }

    pub fn visit_switch_default<T: Visit + ?Sized>(visitor: &mut T, case: &SwitchDefault) {
        visitor.visit_block(&case.body)
    }

    pub fn visit_var_decl<T: Visit + ?Sized>(visitor: &mut T, vdecl: &VarDecl) {
        // Visit the assigned value first.
        vdecl.value.iter().for_each(|e| visitor.visit_expr(e));

        vdecl.vars.iter().for_each(|id| visitor.visit_ident(id));
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn visit_expr<T: Visit + ?Sized>(visitor: &mut T, expr: &Expr) {
        use Expr::*;
        match expr {
            Lit(lit) => visitor.visit_lit(lit),
            Ident(ident) => visitor.visit_ident(ident),
            Call(expr) => visitor.visit_call_expr(expr),
            Member(expr) => visitor.visit_member_expr(expr),
        }
    }

    pub fn visit_call_expr<T: Visit + ?Sized>(visitor: &mut T, expr: &CallExpr) {
        visitor.visit_ident(&expr.callee);
        expr.args.iter().for_each(|arg| visitor.visit_expr(arg))
    }

    pub fn visit_member_expr<T: Visit + ?Sized>(visitor: &mut T, expr: &MemberExpr) {
        visitor.visit_name(&expr.base);
        visitor.visit_name(&expr.member)
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn visit_ident<T: Visit + ?Sized>(visitor: &mut T, id: &Identifier) {
        visitor.visit_type(&id.typ)
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn visit_lit<T: Visit + ?Sized>(_visitor: &mut T, _lit: &Lit) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn visit_type<T: Visit + ?Sized>(_visitor: &mut T, _typ: &Type) {
        // Do nothing by default
    }
}
