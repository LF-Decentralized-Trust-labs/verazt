//! Module implementing the visiting design pattern for IR.
//!
//! This visiting pattern will not transform any input data structure.

use crate::ir::*;

/// Trait implementing the visit design pattern for Solidity AST.
pub trait Visit<'a> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn visit_source_units(&mut self, source_units: &'a [SourceUnit]) {
        default::visit_source_units(self, source_units)
    }

    fn visit_source_unit(&mut self, source_unit: &'a SourceUnit) {
        default::visit_source_unit(self, source_unit)
    }

    fn visit_source_unit_elem(&mut self, elem: &'a SourceUnitElem) {
        default::visit_source_unit_elem(self, elem)
    }

    //-------------------------------------------------
    // Error & event definitions.
    //-------------------------------------------------

    fn visit_error_def(&mut self, error: &'a ErrorDef) {
        default::visit_error_def(self, error)
    }

    fn visit_event_def(&mut self, event: &'a EventDef) {
        default::visit_event_def(self, event)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn visit_struct_def(&mut self, struct_: &'a StructDef) {
        default::visit_struct_def(self, struct_)
    }

    fn visit_struct_field(&mut self, field: &'a StructField) {
        default::visit_struct_field(self, field)
    }

    fn visit_enum_def(&mut self, enum_: &'a EnumDef) {
        default::visit_enum_def(self, enum_)
    }

    fn visit_enum_element(&mut self, elem: &'a str) {
        default::visit_enum_element(self, elem)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn visit_contract_def(&mut self, contract: &'a ContractDef) {
        default::visit_contract_def(self, contract)
    }

    fn visit_contract_elem(&mut self, elem: &'a ContractElem) {
        default::visit_contract_elem(self, elem)
    }

    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    fn visit_func_def(&mut self, func: &'a FuncDef) {
        default::visit_func_def(self, func)
    }

    fn visit_block(&mut self, block: &'a Block) {
        default::visit_block(self, block)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        default::visit_stmt(self, stmt)
    }

    fn visit_asm_stmt(&mut self, stmt: &'a AsmStmt) {
        default::visit_asm_stmt(self, stmt)
    }

    fn visit_break_stmt(&mut self, stmt: &'a BreakStmt) {
        default::visit_break_stmt(self, stmt)
    }

    fn visit_continue_stmt(&mut self, stmt: &'a ContinueStmt) {
        default::visit_continue_stmt(self, stmt)
    }

    fn visit_expr_stmt(&mut self, stmt: &'a ExprStmt) {
        default::visit_expr_stmt(self, stmt)
    }

    fn visit_if_stmt(&mut self, stmt: &'a IfStmt) {
        default::visit_if_stmt(self, stmt)
    }

    fn visit_loop_stmt(&mut self, stmt: &'a LoopStmt) {
        default::visit_loop_stmt(self, stmt)
    }

    fn visit_throw_stmt(&mut self, stmt: &'a ThrowStmt) {
        default::visit_throw_stmt(self, stmt)
    }

    fn visit_try_stmt(&mut self, stmt: &'a TryStmt) {
        default::visit_try_stmt(self, stmt)
    }

    fn visit_catch_clause(&mut self, cls: &'a CatchClause) {
        default::visit_catch_clause(self, cls)
    }

    fn visit_revert_stmt(&mut self, stmt: &'a RevertStmt) {
        default::visit_revert_stmt(self, stmt)
    }

    fn visit_emit_stmt(&mut self, stmt: &'a EmitStmt) {
        default::visit_emit_stmt(self, stmt)
    }

    fn visit_return_stmt(&mut self, stmt: &'a ReturnStmt) {
        default::visit_return_stmt(self, stmt)
    }

    fn visit_var_decl_stmt(&mut self, stmt: &'a VarDeclStmt) {
        default::visit_var_decl_stmt(self, stmt)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn visit_var_decl(&mut self, var: &'a VarDecl) {
        default::visit_var_decl(self, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn visit_expr(&mut self, expr: &'a Expr) {
        default::visit_expr(self, expr)
    }

    fn visit_call_expr(&mut self, expr: &'a CallExpr) {
        default::visit_call_expr(self, expr)
    }

    fn visit_callable_expr(&mut self, expr: &'a CalleeExpr) {
        default::visit_callable_expr(self, expr)
    }

    fn visit_tuple_expr(&mut self, expr: &'a TupleExpr) {
        default::visit_tuple_expr(self, expr)
    }

    fn visit_index_expr(&mut self, expr: &'a IndexExpr) {
        default::visit_index_expr(self, expr)
    }

    fn visit_slice_expr(&mut self, expr: &'a SliceExpr) {
        default::visit_slice_expr(self, expr)
    }

    fn visit_member_expr(&mut self, expr: &'a MemberExpr) {
        default::visit_member_expr(self, expr)
    }

    fn visit_conditional_expr(&mut self, expr: &'a ConditionalExpr) {
        default::visit_conditional_expr(self, expr)
    }

    fn visit_inline_array_expr(&mut self, expr: &'a InlineArrayExpr) {
        default::visit_inline_array_expr(self, expr)
    }

    fn visit_new_expr(&mut self, expr: &'a NewExpr) {
        default::visit_new_expr(self, expr)
    }

    fn visit_type_expr(&mut self, expr: &'a TypeNameExpr) {
        default::visit_type_expr(self, expr)
    }

    fn visit_atomic_expr(&mut self, expr: &'a AtomicExpr) {
        default::visit_atomic_expr(self, expr)
    }

    //-------------------------------------------------
    // Variable
    //-------------------------------------------------

    fn visit_var(&mut self, v: &'a Variable) {
        default::visit_var(self, v)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn visit_type(&mut self, typ: &'a Type) {
        default::visit_type(self, typ)
    }
}

/// Module implement  the visit design pattern for Solidity AST.
pub mod default {
    use super::Visit;
    use crate::ir::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn visit_source_units<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        source_units: &'a [SourceUnit],
    ) {
        source_units
            .iter()
            .for_each(|source_unit| visitor.visit_source_unit(source_unit))
    }

    pub fn visit_source_unit<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, source: &'a SourceUnit) {
        source
            .elems
            .iter()
            .for_each(|elem| visitor.visit_source_unit_elem(elem))
    }

    pub fn visit_source_unit_elem<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        elem: &'a SourceUnitElem,
    ) {
        match elem {
            SourceUnitElem::ErrorDef(e) => visitor.visit_error_def(e),
            SourceUnitElem::VarDecl(v) => visitor.visit_var_decl(v),
            SourceUnitElem::StructDef(s) => visitor.visit_struct_def(s),
            SourceUnitElem::FuncDef(f) => visitor.visit_func_def(f),
            SourceUnitElem::EnumDef(e) => visitor.visit_enum_def(e),
            SourceUnitElem::ContractDef(c) => visitor.visit_contract_def(c),
        }
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    pub fn visit_error_def<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _error: &'a ErrorDef) {
        // Do nothing by default
    }

    pub fn visit_event_def<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _event: &'a EventDef) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn visit_struct_def<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, struct_: &'a StructDef) {
        struct_
            .fields
            .iter()
            .for_each(|field| visitor.visit_struct_field(field))
    }

    pub fn visit_struct_field<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, field: &'a StructField) {
        visitor.visit_type(&field.typ)
    }

    pub fn visit_enum_def<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, enum_: &'a EnumDef) {
        enum_
            .elems
            .iter()
            .for_each(|elem| visitor.visit_enum_element(elem))
    }

    pub fn visit_enum_element<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _elem: &'a str) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    pub fn visit_contract_def<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        contract: &'a ContractDef,
    ) {
        contract
            .elems
            .iter()
            .for_each(|elem| visitor.visit_contract_elem(elem))
    }

    pub fn visit_contract_elem<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        elem: &'a ContractElem,
    ) {
        match elem {
            ContractElem::EventDef(e) => visitor.visit_event_def(e),
            ContractElem::ErrorDef(e) => visitor.visit_error_def(e),
            ContractElem::StructDef(s) => visitor.visit_struct_def(s),
            ContractElem::EnumDef(e) => visitor.visit_enum_def(e),
            ContractElem::VarDecl(v) => visitor.visit_var_decl(v),
            ContractElem::FuncDef(f) => visitor.visit_func_def(f),
        }
    }

    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    pub fn visit_func_def<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, func: &'a FuncDef) {
        func.params.iter().for_each(|p| visitor.visit_var_decl(p));
        func.returns.iter().for_each(|p| visitor.visit_var_decl(p));
        func.body.iter().for_each(|b| visitor.visit_block(b))
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn visit_block<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, block: &'a Block) {
        block.stmts.iter().for_each(|stmt| visitor.visit_stmt(stmt))
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn visit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a Stmt) {
        match stmt {
            Stmt::Asm(s) => visitor.visit_asm_stmt(s),
            Stmt::Break(s) => visitor.visit_break_stmt(s),
            Stmt::Continue(s) => visitor.visit_continue_stmt(s),
            Stmt::Loop(s) => visitor.visit_loop_stmt(s),
            Stmt::Emit(s) => visitor.visit_emit_stmt(s),
            Stmt::Expr(s) => visitor.visit_expr_stmt(s),
            Stmt::If(s) => visitor.visit_if_stmt(s),
            Stmt::Return(s) => visitor.visit_return_stmt(s),
            Stmt::Revert(s) => visitor.visit_revert_stmt(s),
            Stmt::Throw(s) => visitor.visit_throw_stmt(s),
            Stmt::Try(s) => visitor.visit_try_stmt(s),
            Stmt::VarDecl(s) => visitor.visit_var_decl_stmt(s),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    pub fn visit_asm_stmt<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _stmt: &'a AsmStmt) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    pub fn visit_break_stmt<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _stmt: &'a BreakStmt) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    pub fn visit_continue_stmt<'a, T: Visit<'a> + ?Sized>(
        _visitor: &mut T,
        _stmt: &'a ContinueStmt,
    ) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    pub fn visit_expr_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a ExprStmt) {
        visitor.visit_expr(&stmt.expr)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn visit_if_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a IfStmt) {
        visitor.visit_expr(&stmt.condition);
        visitor.visit_block(&stmt.true_branch);
        stmt.false_branch
            .iter()
            .for_each(|blk| visitor.visit_block(blk))
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    pub fn visit_loop_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a LoopStmt) {
        stmt.pre_loop
            .iter()
            .for_each(|blk| visitor.visit_block(blk));
        stmt.post_loop
            .iter()
            .for_each(|blk| visitor.visit_block(blk));
        stmt.condition
            .iter()
            .for_each(|expr| visitor.visit_expr(expr));
        visitor.visit_block(&stmt.body)
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    pub fn visit_throw_stmt<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _stmt: &'a ThrowStmt) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    pub fn visit_try_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a TryStmt) {
        visitor.visit_expr(&stmt.guarded_expr);
        stmt.returns.iter().for_each(|v| visitor.visit_var_decl(v));
        visitor.visit_block(&stmt.body);
        stmt.catch_clauses
            .iter()
            .for_each(|cls| visitor.visit_catch_clause(cls));
    }

    pub fn visit_catch_clause<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, cls: &'a CatchClause) {
        cls.params.iter().for_each(|v| visitor.visit_var_decl(v));
        visitor.visit_block(&cls.body)
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    pub fn visit_revert_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a RevertStmt) {
        stmt.args
            .iter()
            .for_each(|arg| visitor.visit_atomic_expr(arg))
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn visit_emit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a EmitStmt) {
        stmt.args
            .iter()
            .for_each(|arg| visitor.visit_atomic_expr(arg))
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn visit_return_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a ReturnStmt) {
        if let Some(expr) = &stmt.expr {
            visitor.visit_expr(expr)
        }
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn visit_var_decl_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a VarDeclStmt) {
        // Visit the assigned value first.
        stmt.value.iter().for_each(|e| visitor.visit_expr(e));
        stmt.vars.iter().for_each(|v| {
            if let Some(v) = v {
                visitor.visit_var_decl(v)
            }
        })
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    pub fn visit_var_decl<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, vdecl: &'a VarDecl) {
        vdecl.value.iter().for_each(|e| visitor.visit_expr(e))
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn visit_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a Expr) {
        match expr {
            Expr::Lit(_) => (),
            Expr::Var(v) => visitor.visit_var(v),
            Expr::Call(e) => visitor.visit_call_expr(e),
            Expr::Tuple(e) => visitor.visit_tuple_expr(e),
            Expr::Index(e) => visitor.visit_index_expr(e),
            Expr::Slice(e) => visitor.visit_slice_expr(e),
            Expr::Member(e) => visitor.visit_member_expr(e),
            Expr::InlineArray(e) => visitor.visit_inline_array_expr(e),
            Expr::New(e) => visitor.visit_new_expr(e),
            Expr::TypeName(e) => visitor.visit_type_expr(e),
        }
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    pub fn visit_call_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CallExpr) {
        visitor.visit_callable_expr(&expr.callee);
        expr.args
            .iter()
            .for_each(|arg| visitor.visit_atomic_expr(arg))
    }

    pub fn visit_callable_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CalleeExpr) {
        match expr {
            CalleeExpr::BuiltIn(_) => {}
            CalleeExpr::FuncDef(f) => visitor.visit_func_def(f),
            CalleeExpr::ContractDef(c) => visitor.visit_contract_def(c),
            CalleeExpr::StructDef(s) => visitor.visit_struct_def(s),
            CalleeExpr::MemberExpr(e) => visitor.visit_member_expr(e),
            CalleeExpr::NewExpr(e) => visitor.visit_new_expr(e),
            CalleeExpr::TypeNameExpr(e) => visitor.visit_type_expr(e),
        }
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------
    pub fn visit_tuple_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a TupleExpr) {
        expr.elems.iter().for_each(|v| {
            if let Some(v) = v {
                visitor.visit_atomic_expr(v)
            }
        })
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------
    pub fn visit_index_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a IndexExpr) {
        visitor.visit_var(&expr.base);
        if let Some(idx) = &expr.index {
            visitor.visit_atomic_expr(idx)
        }
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------
    pub fn visit_slice_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a SliceExpr) {
        visitor.visit_var(&expr.base);
        if let Some(idx) = &expr.start_index {
            visitor.visit_atomic_expr(idx);
        }
        if let Some(idx) = &expr.end_index {
            visitor.visit_atomic_expr(idx);
        }
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------
    pub fn visit_member_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a MemberExpr) {
        visitor.visit_expr(&expr.base)
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    pub fn visit_conditional_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a ConditionalExpr,
    ) {
        visitor.visit_atomic_expr(&expr.cond);
        visitor.visit_atomic_expr(&expr.true_br);
        visitor.visit_atomic_expr(&expr.false_br)
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    pub fn visit_inline_array_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a InlineArrayExpr,
    ) {
        for elem in &expr.elems {
            visitor.visit_atomic_expr(elem);
        }
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    pub fn visit_new_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a NewExpr) {
        visitor.visit_type(&expr.typ);
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    pub fn visit_type_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a TypeNameExpr) {
        visitor.visit_type(&expr.typ);
    }

    //-------------------------------------------------
    // Atomic expression
    //-------------------------------------------------

    pub fn visit_atomic_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a AtomicExpr) {
        match expr {
            AtomicExpr::Lit(_) => (),
            AtomicExpr::Var(v) => visitor.visit_var(v),
            AtomicExpr::Type(e) => visitor.visit_type_expr(e),
        }
    }

    //-------------------------------------------------
    // Variable
    //-------------------------------------------------

    pub fn visit_var<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _var: &'a Variable) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn visit_type<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _typ: &'a Type) {
        // Do nothing by default
    }
}
