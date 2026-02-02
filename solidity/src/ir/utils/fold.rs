//! Module implementing the fold (left to right) pattern for IR.
//!
//! This map pattern will compute output of the same type when traversing all
//! data structures in the AST.

use crate::ir::*;

/// Trait providing the fold pattern.
pub trait Fold<'a, T> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn fold_source_units(&mut self, acc: T, source_units: &'a [SourceUnit]) -> T {
        default::fold_source_units(self, acc, source_units)
    }

    fn fold_source_unit(&self, acc: T, source_unit: &'a SourceUnit) -> T {
        default::fold_source_unit(self, acc, source_unit)
    }

    fn fold_source_unit_elem(&self, acc: T, elem: &'a SourceUnitElem) -> T {
        default::fold_source_unit_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Error & event definitions.
    //-------------------------------------------------

    fn fold_error_def(&self, acc: T, error: &'a ErrorDef) -> T {
        default::fold_error_def(self, acc, error)
    }

    fn fold_event_def(&self, acc: T, event: &'a EventDef) -> T {
        default::fold_event_def(self, acc, event)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn fold_struct_def(&self, acc: T, struct_: &'a StructDef) -> T {
        default::fold_struct_def(self, acc, struct_)
    }

    fn fold_struct_field(&self, acc: T, field: &'a StructField) -> T {
        default::fold_struct_field(self, acc, field)
    }

    fn fold_enum_def(&self, acc: T, enum_: &'a EnumDef) -> T {
        default::fold_enum_def(self, acc, enum_)
    }

    fn fold_enum_element(&self, acc: T, elem: &'a str) -> T {
        default::fold_enum_element(self, acc, elem)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn fold_contract_def(&self, acc: T, contract: &'a ContractDef) -> T {
        default::fold_contract_def(self, acc, contract)
    }

    fn fold_contract_elem(&self, acc: T, elem: &'a ContractElem) -> T {
        default::fold_contract_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    fn fold_func_def(&self, acc: T, func: &'a FuncDef) -> T {
        default::fold_func_def(self, acc, func)
    }

    fn fold_block(&self, acc: T, block: &'a Block) -> T {
        default::fold_block(self, acc, block)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn fold_stmt(&self, acc: T, stmt: &'a Stmt) -> T {
        default::fold_stmt(self, acc, stmt)
    }

    fn fold_asm_stmt(&self, acc: T, stmt: &'a AsmStmt) -> T {
        default::fold_asm_stmt(self, acc, stmt)
    }

    fn fold_break_stmt(&self, acc: T, stmt: &'a BreakStmt) -> T {
        default::fold_break_stmt(self, acc, stmt)
    }

    fn fold_continue_stmt(&self, acc: T, stmt: &'a ContinueStmt) -> T {
        default::fold_continue_stmt(self, acc, stmt)
    }

    fn fold_expr_stmt(&self, acc: T, stmt: &'a ExprStmt) -> T {
        default::fold_expr_stmt(self, acc, stmt)
    }

    fn fold_if_stmt(&self, acc: T, stmt: &'a IfStmt) -> T {
        default::fold_if_stmt(self, acc, stmt)
    }

    fn fold_loop_stmt(&self, acc: T, stmt: &'a LoopStmt) -> T {
        default::fold_loop_stmt(self, acc, stmt)
    }

    fn fold_throw_stmt(&self, acc: T, stmt: &'a ThrowStmt) -> T {
        default::fold_throw_stmt(self, acc, stmt)
    }

    fn fold_try_stmt(&self, acc: T, stmt: &'a TryStmt) -> T {
        default::fold_try_stmt(self, acc, stmt)
    }

    fn fold_catch_clause(&self, acc: T, cls: &'a CatchClause) -> T {
        default::fold_catch_clause(self, acc, cls)
    }

    fn fold_revert_stmt(&self, acc: T, stmt: &'a RevertStmt) -> T {
        default::fold_revert_stmt(self, acc, stmt)
    }

    fn fold_emit_stmt(&self, acc: T, stmt: &'a EmitStmt) -> T {
        default::fold_emit_stmt(self, acc, stmt)
    }

    fn fold_return_stmt(&self, acc: T, stmt: &'a ReturnStmt) -> T {
        default::fold_return_stmt(self, acc, stmt)
    }

    fn fold_var_decl_stmt(&self, acc: T, stmt: &'a VarDeclStmt) -> T {
        default::fold_var_decl_stmt(self, acc, stmt)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn fold_var_decl(&self, acc: T, var: &'a VarDecl) -> T {
        default::fold_var_decl(self, acc, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn fold_expr(&self, acc: T, expr: &'a Expr) -> T {
        default::fold_expr(self, acc, expr)
    }

    fn fold_call_expr(&self, acc: T, expr: &'a CallExpr) -> T {
        default::fold_call_expr(self, acc, expr)
    }

    fn fold_callable_expr(&self, acc: T, expr: &'a CalleeExpr) -> T {
        default::fold_callable_expr(self, acc, expr)
    }

    fn fold_tuple_expr(&self, acc: T, expr: &'a TupleExpr) -> T {
        default::fold_tuple_expr(self, acc, expr)
    }

    fn fold_index_expr(&self, acc: T, expr: &'a IndexExpr) -> T {
        default::fold_index_expr(self, acc, expr)
    }

    fn fold_slice_expr(&self, acc: T, expr: &'a SliceExpr) -> T {
        default::fold_slice_expr(self, acc, expr)
    }

    fn fold_member_expr(&self, acc: T, expr: &'a MemberExpr) -> T {
        default::fold_member_expr(self, acc, expr)
    }

    fn fold_conditional_expr(&self, acc: T, expr: &'a ConditionalExpr) -> T {
        default::fold_conditional_expr(self, acc, expr)
    }

    fn fold_inline_array_expr(&self, acc: T, expr: &'a InlineArrayExpr) -> T {
        default::fold_inline_array_expr(self, acc, expr)
    }

    fn fold_new_expr(&self, acc: T, expr: &'a NewExpr) -> T {
        default::fold_new_expr(self, acc, expr)
    }

    fn fold_type_expr(&self, acc: T, expr: &'a TypeNameExpr) -> T {
        default::fold_type_expr(self, acc, expr)
    }

    fn fold_atomic_expr(&self, acc: T, expr: &'a AtomicExpr) -> T {
        default::fold_atomic_expr(self, acc, expr)
    }

    //-------------------------------------------------
    // Variable
    //-------------------------------------------------

    fn fold_variable(&self, acc: T, var: &'a Variable) -> T {
        default::fold_variable(self, acc, var)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn fold_lit(&self, acc: T, c: &'a Lit) -> T {
        default::fold_lit(self, acc, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn fold_type(&self, acc: T, typ: &'a Type) -> T {
        default::fold_type(self, acc, typ)
    }
}

/// Module contain default implementation of the folding pattern.
pub mod default {
    use super::Fold;
    use crate::ir::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn fold_source_units<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        source_units: &'a [SourceUnit],
    ) -> T {
        source_units
            .iter()
            .fold(acc, |acc2, source_unit| folder.fold_source_unit(acc2, source_unit))
    }

    pub fn fold_source_unit<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        source_unit: &'a SourceUnit,
    ) -> T {
        source_unit
            .elems
            .iter()
            .fold(acc, |acc2, elem| folder.fold_source_unit_elem(acc2, elem))
    }

    pub fn fold_source_unit_elem<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        elem: &'a SourceUnitElem,
    ) -> T {
        match elem {
            SourceUnitElem::ErrorDef(e) => folder.fold_error_def(acc, e),
            SourceUnitElem::VarDecl(v) => folder.fold_var_decl(acc, v),
            SourceUnitElem::StructDef(s) => folder.fold_struct_def(acc, s),
            SourceUnitElem::FuncDef(f) => folder.fold_func_def(acc, f),
            SourceUnitElem::EnumDef(e) => folder.fold_enum_def(acc, e),
            SourceUnitElem::ContractDef(c) => folder.fold_contract_def(acc, c),
        }
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    pub fn fold_error_def<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _error: &'a ErrorDef,
    ) -> T {
        acc
    }

    pub fn fold_event_def<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _event: &'a EventDef,
    ) -> T {
        acc
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn fold_struct_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        struct_: &'a StructDef,
    ) -> T {
        struct_
            .fields
            .iter()
            .fold(acc, |acc2, fld| folder.fold_struct_field(acc2, fld))
    }

    pub fn fold_struct_field<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        field: &'a StructField,
    ) -> T {
        folder.fold_type(acc, &field.typ)
    }

    pub fn fold_enum_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        enum_: &'a EnumDef,
    ) -> T {
        enum_
            .elems
            .iter()
            .fold(acc, |acc2, elem| folder.fold_enum_element(acc2, elem))
    }

    pub fn fold_enum_element<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _elem: &'a str,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    pub fn fold_contract_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        contract: &'a ContractDef,
    ) -> T {
        contract
            .elems
            .iter()
            .fold(acc, |acc2, elem| folder.fold_contract_elem(acc2, elem))
    }

    pub fn fold_contract_elem<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        elem: &'a ContractElem,
    ) -> T {
        match elem {
            ContractElem::EventDef(e) => folder.fold_event_def(acc, e),
            ContractElem::ErrorDef(e) => folder.fold_error_def(acc, e),
            ContractElem::StructDef(s) => folder.fold_struct_def(acc, s),
            ContractElem::EnumDef(e) => folder.fold_enum_def(acc, e),
            ContractElem::VarDecl(v) => folder.fold_var_decl(acc, v),
            ContractElem::FuncDef(f) => folder.fold_func_def(acc, f),
        }
    }

    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    pub fn fold_func_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        func: &'a FuncDef,
    ) -> T {
        let res = func
            .params
            .iter()
            .fold(acc, |acc2, p| folder.fold_var_decl(acc2, p));
        let res = func
            .returns
            .iter()
            .fold(res, |acc2, p| folder.fold_var_decl(acc2, p));
        match &func.body {
            None => res,
            Some(b) => folder.fold_block(res, b),
        }
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn fold_block<'a, T, F: Fold<'a, T> + ?Sized>(folder: &F, acc: T, block: &'a Block) -> T {
        block
            .stmts
            .iter()
            .fold(acc, |acc2, stmt| folder.fold_stmt(acc2, stmt))
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn fold_stmt<'a, T, F: Fold<'a, T> + ?Sized>(folder: &F, acc: T, stmt: &'a Stmt) -> T {
        match stmt {
            Stmt::Asm(s) => folder.fold_asm_stmt(acc, s),
            Stmt::Break(s) => folder.fold_break_stmt(acc, s),
            Stmt::Continue(s) => folder.fold_continue_stmt(acc, s),
            Stmt::Emit(s) => folder.fold_emit_stmt(acc, s),
            Stmt::Expr(s) => folder.fold_expr_stmt(acc, s),
            Stmt::Loop(s) => folder.fold_loop_stmt(acc, s),
            Stmt::If(s) => folder.fold_if_stmt(acc, s),
            Stmt::Return(s) => folder.fold_return_stmt(acc, s),
            Stmt::Revert(s) => folder.fold_revert_stmt(acc, s),
            Stmt::Throw(s) => folder.fold_throw_stmt(acc, s),
            Stmt::Try(s) => folder.fold_try_stmt(acc, s),
            Stmt::VarDecl(s) => folder.fold_var_decl_stmt(acc, s),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    pub fn fold_asm_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _stmt: &'a AsmStmt,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    pub fn fold_break_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _stmt: &'a BreakStmt,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    pub fn fold_continue_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _stmt: &'a ContinueStmt,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    pub fn fold_expr_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a ExprStmt,
    ) -> T {
        folder.fold_expr(acc, &stmt.expr)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn fold_if_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a IfStmt,
    ) -> T {
        let res = folder.fold_expr(acc, &stmt.condition);
        let res = folder.fold_block(res, &stmt.true_branch);
        match &stmt.false_branch {
            None => res,
            Some(blk) => folder.fold_block(res, blk),
        }
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    pub fn fold_throw_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _stmt: &'a ThrowStmt,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    pub fn fold_try_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a TryStmt,
    ) -> T {
        let res = folder.fold_expr(acc, &stmt.guarded_expr);
        let res = stmt
            .returns
            .iter()
            .fold(res, |acc2, v| folder.fold_var_decl(acc2, v));
        let res = folder.fold_block(res, &stmt.body);
        stmt.catch_clauses
            .iter()
            .fold(res, |acc2, cls| folder.fold_catch_clause(acc2, cls))
    }

    pub fn fold_catch_clause<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        cls: &'a CatchClause,
    ) -> T {
        let res = cls
            .params
            .iter()
            .fold(acc, |acc2, v| folder.fold_var_decl(acc2, v));
        folder.fold_block(res, &cls.body)
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    pub fn fold_revert_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a RevertStmt,
    ) -> T {
        stmt.args
            .iter()
            .fold(acc, |acc2, arg| folder.fold_atomic_expr(acc2, arg))
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn fold_emit_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a EmitStmt,
    ) -> T {
        stmt.args
            .iter()
            .fold(acc, |acc2, arg| folder.fold_atomic_expr(acc2, arg))
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn fold_return_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a ReturnStmt,
    ) -> T {
        match &stmt.expr {
            Some(expr) => folder.fold_expr(acc, expr),
            None => acc,
        }
    }

    //-------------------------------------------------
    // Loop statement
    //-------------------------------------------------

    pub fn fold_loop_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a LoopStmt,
    ) -> T {
        let res = match &stmt.pre_loop {
            Some(blk) => folder.fold_block(acc, blk),
            None => acc,
        };
        let res = match &stmt.condition {
            Some(exp) => folder.fold_expr(res, exp),
            None => res,
        };
        let res = match &stmt.post_loop {
            Some(blk) => folder.fold_block(res, blk),
            None => res,
        };
        folder.fold_block(res, &stmt.body)
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn fold_var_decl_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        stmt: &'a VarDeclStmt,
    ) -> T {
        // Fold the assigned value first.
        let res = match &stmt.value {
            Some(exp) => folder.fold_expr(acc, exp),
            None => acc,
        };
        stmt.vars.iter().fold(res, |acc2, v| match v {
            Some(v) => folder.fold_var_decl(acc2, v),
            None => acc2,
        })
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    pub fn fold_var_decl<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        vdecl: &'a VarDecl,
    ) -> T {
        match &vdecl.value {
            Some(exp) => folder.fold_expr(acc, exp),
            None => acc,
        }
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn fold_expr<'a, T, F: Fold<'a, T> + ?Sized>(folder: &F, acc: T, expr: &'a Expr) -> T {
        match expr {
            Expr::Var(v) => folder.fold_variable(acc, v),
            Expr::Lit(c) => folder.fold_lit(acc, c),
            Expr::Call(e) => folder.fold_call_expr(acc, e),
            Expr::Tuple(e) => folder.fold_tuple_expr(acc, e),
            Expr::Index(e) => folder.fold_index_expr(acc, e),
            Expr::Slice(e) => folder.fold_slice_expr(acc, e),
            Expr::Member(e) => folder.fold_member_expr(acc, e),
            Expr::InlineArray(e) => folder.fold_inline_array_expr(acc, e),
            Expr::New(e) => folder.fold_new_expr(acc, e),
            Expr::TypeName(e) => folder.fold_type_expr(acc, e),
        }
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    pub fn fold_call_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a CallExpr,
    ) -> T {
        let res = folder.fold_callable_expr(acc, &expr.callee);
        expr.args
            .iter()
            .fold(res, |acc2, arg| folder.fold_atomic_expr(acc2, arg))
    }

    pub fn fold_callable_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a CalleeExpr,
    ) -> T {
        match expr {
            CalleeExpr::BuiltIn(_) => acc,
            CalleeExpr::FuncDef(f) => folder.fold_func_def(acc, f),
            CalleeExpr::ContractDef(c) => folder.fold_contract_def(acc, c),
            CalleeExpr::StructDef(s) => folder.fold_struct_def(acc, s),
            CalleeExpr::MemberExpr(e) => folder.fold_member_expr(acc, e),
            CalleeExpr::NewExpr(e) => folder.fold_new_expr(acc, e),
            CalleeExpr::TypeNameExpr(e) => folder.fold_type_expr(acc, e),
        }
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------

    pub fn fold_tuple_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a TupleExpr,
    ) -> T {
        expr.elems.iter().fold(acc, |acc2, v| match v {
            Some(v) => folder.fold_atomic_expr(acc2, v),
            None => acc2,
        })
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    pub fn fold_index_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a IndexExpr,
    ) -> T {
        let mut res = folder.fold_variable(acc, &expr.base);
        if let Some(index) = &expr.index {
            res = folder.fold_atomic_expr(res, index);
        }
        res
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    pub fn fold_slice_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a SliceExpr,
    ) -> T {
        let mut res = folder.fold_variable(acc, &expr.base);
        if let Some(start_index) = &expr.start_index {
            res = folder.fold_atomic_expr(res, start_index);
        }
        if let Some(end_index) = &expr.start_index {
            res = folder.fold_atomic_expr(res, end_index);
        }
        res
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    pub fn fold_member_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a MemberExpr,
    ) -> T {
        folder.fold_expr(acc, &expr.base)
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    pub fn fold_conditional_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a ConditionalExpr,
    ) -> T {
        let res = folder.fold_atomic_expr(acc, &expr.cond);
        let res = folder.fold_atomic_expr(res, &expr.true_br);
        folder.fold_atomic_expr(res, &expr.false_br)
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    pub fn fold_inline_array_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a InlineArrayExpr,
    ) -> T {
        expr.elems
            .iter()
            .fold(acc, |acc2, e| folder.fold_atomic_expr(acc2, e))
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    pub fn fold_new_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a NewExpr,
    ) -> T {
        folder.fold_type(acc, &expr.typ)
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    pub fn fold_type_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a TypeNameExpr,
    ) -> T {
        folder.fold_type(acc, &expr.typ)
    }

    //-------------------------------------------------
    // Atomic expression
    //-------------------------------------------------

    pub fn fold_atomic_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &F,
        acc: T,
        expr: &'a AtomicExpr,
    ) -> T {
        match expr {
            AtomicExpr::Lit(l) => folder.fold_lit(acc, l),
            AtomicExpr::Var(v) => folder.fold_variable(acc, v),
            AtomicExpr::Type(e) => folder.fold_type_expr(acc, e),
        }
    }

    //-------------------------------------------------
    // Variable.
    //-------------------------------------------------

    pub fn fold_variable<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &F,
        acc: T,
        _var: &'a Variable,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn fold_lit<'a, T, F: Fold<'a, T> + ?Sized>(_folder: &F, acc: T, _lit: &'a Lit) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn fold_type<'a, T, F: Fold<'a, T> + ?Sized>(_folder: &F, acc: T, _typ: &'a Type) -> T {
        // By default, return the accumulative value
        acc
    }
}
