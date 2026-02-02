//! Module implementing the map design pattern for IR.
//!
//! This map pattern will transform an input data structure into a new data
//! structure of the same type.

use crate::ir::*;

/// Trait for implementing the maping pattern
pub trait Map<'a> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn map_source_units(&mut self, source_units: &'a [SourceUnit]) -> Vec<SourceUnit> {
        default::map_source_units(self, source_units)
    }

    fn map_source_unit(&mut self, source_unit: &'a SourceUnit) -> SourceUnit {
        default::map_source_unit(self, source_unit)
    }

    fn map_source_unit_elem(&mut self, elem: &'a SourceUnitElem) -> SourceUnitElem {
        default::map_source_unit_elem(self, elem)
    }

    //-------------------------------------------------
    // Error & event definitions.
    //-------------------------------------------------

    fn map_error_def(&mut self, error: &'a ErrorDef) -> ErrorDef {
        default::map_error_def(self, error)
    }

    fn map_event_def(&mut self, event: &'a EventDef) -> EventDef {
        default::map_event_def(self, event)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn map_struct_def(&mut self, struct_: &'a StructDef) -> StructDef {
        default::map_struct_def(self, struct_)
    }

    fn map_struct_field(&mut self, field: &'a StructField) -> StructField {
        default::map_struct_field(self, field)
    }

    fn map_enum_def(&mut self, enum_: &'a EnumDef) -> EnumDef {
        default::map_enum_def(self, enum_)
    }

    fn map_enum_element(&mut self, elem: &'a str) -> String {
        default::map_enum_element(self, elem)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn map_contract_def(&mut self, contract: &'a ContractDef) -> ContractDef {
        default::map_contract_def(self, contract)
    }

    fn map_contract_element(&mut self, elem: &'a ContractElem) -> ContractElem {
        default::map_contract_element(self, elem)
    }

    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    fn map_func_def(&mut self, func: &'a FuncDef) -> FuncDef {
        default::map_func_def(self, func)
    }

    fn map_block(&mut self, block: &'a Block) -> Block {
        default::map_block(self, block)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn map_stmt(&mut self, stmt: &'a Stmt) -> Stmt {
        default::map_stmt(self, stmt)
    }

    fn map_asm_stmt(&mut self, stmt: &'a AsmStmt) -> AsmStmt {
        default::map_asm_stmt(self, stmt)
    }

    fn map_break_stmt(&mut self, stmt: &'a BreakStmt) -> BreakStmt {
        default::map_break_stmt(self, stmt)
    }

    fn map_continue_stmt(&mut self, stmt: &'a ContinueStmt) -> ContinueStmt {
        default::map_continue_stmt(self, stmt)
    }

    fn map_expr_stmt(&mut self, stmt: &'a ExprStmt) -> ExprStmt {
        default::map_expr_stmt(self, stmt)
    }

    fn map_if_stmt(&mut self, stmt: &'a IfStmt) -> IfStmt {
        default::map_if_stmt(self, stmt)
    }

    fn map_loop_stmt(&mut self, stmt: &'a LoopStmt) -> LoopStmt {
        default::map_loop_stmt(self, stmt)
    }

    fn map_throw_stmt(&mut self, stmt: &'a ThrowStmt) -> ThrowStmt {
        default::map_throw_stmt(self, stmt)
    }

    fn map_try_stmt(&mut self, stmt: &'a TryStmt) -> TryStmt {
        default::map_try_stmt(self, stmt)
    }

    fn map_catch_clause(&mut self, cls: &'a CatchClause) -> CatchClause {
        default::map_catch_clause(self, cls)
    }

    fn map_revert_stmt(&mut self, stmt: &'a RevertStmt) -> RevertStmt {
        default::map_revert_stmt(self, stmt)
    }

    fn map_emit_stmt(&mut self, stmt: &'a EmitStmt) -> EmitStmt {
        default::map_emit_stmt(self, stmt)
    }

    fn map_return_stmt(&mut self, stmt: &'a ReturnStmt) -> ReturnStmt {
        default::map_return_stmt(self, stmt)
    }

    fn map_var_decl_stmt(&mut self, stmt: &'a VarDeclStmt) -> VarDeclStmt {
        default::map_var_decl_stmt(self, stmt)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn map_var_decl(&mut self, var: &'a VarDecl) -> VarDecl {
        default::map_var_decl(self, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn map_expr(&mut self, expr: &'a Expr) -> Expr {
        default::map_expr(self, expr)
    }

    fn map_call_expr(&mut self, expr: &'a CallExpr) -> CallExpr {
        default::map_call_expr(self, expr)
    }

    fn map_callable_expr(&mut self, expr: &'a CalleeExpr) -> CalleeExpr {
        default::map_callable_expr(self, expr)
    }

    fn map_tuple_expr(&mut self, expr: &'a TupleExpr) -> TupleExpr {
        default::map_tuple_expr(self, expr)
    }

    fn map_index_expr(&mut self, expr: &'a IndexExpr) -> IndexExpr {
        default::map_index_expr(self, expr)
    }

    fn map_slice_expr(&mut self, expr: &'a SliceExpr) -> SliceExpr {
        default::map_slice_expr(self, expr)
    }

    fn map_member_expr(&mut self, expr: &'a MemberExpr) -> MemberExpr {
        default::map_member_expr(self, expr)
    }

    fn map_conditional_expr(&mut self, expr: &'a ConditionalExpr) -> ConditionalExpr {
        default::map_conditional_expr(self, expr)
    }

    fn map_inline_array_expr(&mut self, expr: &'a InlineArrayExpr) -> InlineArrayExpr {
        default::map_inline_array_expr(self, expr)
    }

    fn map_new_expr(&mut self, expr: &'a NewExpr) -> NewExpr {
        default::map_new_expr(self, expr)
    }

    fn map_type_expr(&mut self, expr: &'a TypeNameExpr) -> TypeNameExpr {
        default::map_type_expr(self, expr)
    }

    fn map_atomic_expr(&mut self, expr: &'a AtomicExpr) -> AtomicExpr {
        default::map_atomic_expr(self, expr)
    }

    //-------------------------------------------------
    // Variable
    //-------------------------------------------------

    fn map_variable(&mut self, var: &'a Variable) -> Variable {
        default::map_variable(self, var)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn map_type(&mut self, typ: &'a Type) -> Type {
        default::map_type(self, typ)
    }
}

/// Module contain default implementation of the maping pattern.
pub mod default {
    use super::Map;
    use crate::ir::*;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn map_source_units<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        source_units: &'a [SourceUnit],
    ) -> Vec<SourceUnit> {
        source_units
            .iter()
            .map(|source_unit| mapper.map_source_unit(source_unit))
            .collect()
    }

    pub fn map_source_unit<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        source_unit: &'a SourceUnit,
    ) -> SourceUnit {
        let nelems = source_unit
            .elems
            .iter()
            .map(|elem| mapper.map_source_unit_elem(elem))
            .collect();
        SourceUnit { elems: nelems, ..source_unit.clone() }
    }

    pub fn map_source_unit_elem<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        elem: &'a SourceUnitElem,
    ) -> SourceUnitElem {
        match elem {
            SourceUnitElem::ErrorDef(e) => mapper.map_error_def(e).into(),
            SourceUnitElem::VarDecl(v) => mapper.map_var_decl(v).into(),
            SourceUnitElem::StructDef(s) => mapper.map_struct_def(s).into(),
            SourceUnitElem::FuncDef(f) => mapper.map_func_def(f).into(),
            SourceUnitElem::EnumDef(e) => mapper.map_enum_def(e).into(),
            SourceUnitElem::ContractDef(c) => mapper.map_contract_def(c).into(),
        }
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    pub fn map_error_def<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        error: &'a ErrorDef,
    ) -> ErrorDef {
        error.clone()
    }

    pub fn map_event_def<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        event: &'a EventDef,
    ) -> EventDef {
        event.clone()
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn map_struct_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        struct_: &'a StructDef,
    ) -> StructDef {
        let nfields = struct_
            .fields
            .iter()
            .map(|fld| mapper.map_struct_field(fld))
            .collect();
        StructDef { fields: nfields, ..struct_.clone() }
    }

    pub fn map_struct_field<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        field: &'a StructField,
    ) -> StructField {
        let ntype = mapper.map_type(&field.typ);
        StructField { typ: ntype, ..field.clone() }
    }

    pub fn map_enum_def<'a, T: Map<'a> + ?Sized>(mapper: &mut T, enum_: &'a EnumDef) -> EnumDef {
        let nelems = enum_
            .elems
            .iter()
            .map(|elem| mapper.map_enum_element(elem))
            .collect();
        EnumDef { elems: nelems, ..enum_.clone() }
    }

    pub fn map_enum_element<'a, T: Map<'a> + ?Sized>(_mapper: &mut T, elem: &'a str) -> String {
        elem.to_string()
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    pub fn map_contract_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        contract: &'a ContractDef,
    ) -> ContractDef {
        let nelems = contract
            .elems
            .iter()
            .map(|elem| mapper.map_contract_element(elem))
            .collect();
        ContractDef { elems: nelems, ..contract.clone() }
    }

    pub fn map_contract_element<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        elem: &'a ContractElem,
    ) -> ContractElem {
        match elem {
            ContractElem::EventDef(e) => mapper.map_event_def(e).into(),
            ContractElem::ErrorDef(e) => mapper.map_error_def(e).into(),
            ContractElem::StructDef(s) => mapper.map_struct_def(s).into(),
            ContractElem::EnumDef(e) => mapper.map_enum_def(e).into(),
            ContractElem::VarDecl(v) => mapper.map_var_decl(v).into(),
            ContractElem::FuncDef(f) => mapper.map_func_def(f).into(),
        }
    }
    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    pub fn map_func_def<'a, T: Map<'a> + ?Sized>(mapper: &mut T, func: &'a FuncDef) -> FuncDef {
        let nparams = func.params.iter().map(|p| mapper.map_var_decl(p)).collect();
        let nreturns = func
            .returns
            .iter()
            .map(|p| mapper.map_var_decl(p))
            .collect();
        let nbody = func.body.as_ref().map(|blk| mapper.map_block(blk));
        FuncDef { params: nparams, returns: nreturns, body: nbody, ..func.clone() }
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn map_block<'a, T: Map<'a> + ?Sized>(mapper: &mut T, block: &'a Block) -> Block {
        let nstmts = block
            .stmts
            .iter()
            .map(|stmt| mapper.map_stmt(stmt))
            .collect();
        Block { stmts: nstmts, ..block.clone() }
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn map_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a Stmt) -> Stmt {
        match stmt {
            Stmt::Asm(s) => mapper.map_asm_stmt(s).into(),
            Stmt::Break(s) => mapper.map_break_stmt(s).into(),
            Stmt::Continue(s) => mapper.map_continue_stmt(s).into(),
            Stmt::Emit(s) => mapper.map_emit_stmt(s).into(),
            Stmt::Expr(s) => mapper.map_expr_stmt(s).into(),
            Stmt::Loop(s) => mapper.map_loop_stmt(s).into(),
            Stmt::If(s) => mapper.map_if_stmt(s).into(),
            Stmt::Return(s) => mapper.map_return_stmt(s).into(),
            Stmt::Revert(s) => mapper.map_revert_stmt(s).into(),
            Stmt::Throw(s) => mapper.map_throw_stmt(s).into(),
            Stmt::Try(s) => mapper.map_try_stmt(s).into(),
            Stmt::VarDecl(s) => mapper.map_var_decl_stmt(s).into(),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    pub fn map_asm_stmt<'a, T: Map<'a> + ?Sized>(_mapper: &mut T, stmt: &'a AsmStmt) -> AsmStmt {
        stmt.clone()
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    pub fn map_break_stmt<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        stmt: &'a BreakStmt,
    ) -> BreakStmt {
        stmt.clone()
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    pub fn map_continue_stmt<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        stmt: &'a ContinueStmt,
    ) -> ContinueStmt {
        stmt.clone()
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    pub fn map_expr_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a ExprStmt) -> ExprStmt {
        let expr = mapper.map_expr(&stmt.expr);
        ExprStmt { expr, ..stmt.clone() }
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn map_if_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a IfStmt) -> IfStmt {
        let ncond = mapper.map_expr(&stmt.condition);
        let ntrue_br = mapper.map_block(&stmt.true_branch);
        let nfalse_br = stmt.false_branch.as_ref().map(|blk| mapper.map_block(blk));
        IfStmt { condition: ncond, true_branch: ntrue_br, false_branch: nfalse_br, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Loop statement
    //-------------------------------------------------

    pub fn map_loop_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a LoopStmt) -> LoopStmt {
        let pre = stmt.pre_loop.as_ref().map(|blk| mapper.map_block(blk));
        let post = stmt.post_loop.as_ref().map(|blk| mapper.map_block(blk));
        let cond = stmt.condition.as_ref().map(|e| mapper.map_expr(e));
        let body = mapper.map_block(&stmt.body);
        LoopStmt { pre_loop: pre, post_loop: post, condition: cond, body, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    pub fn map_throw_stmt<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        stmt: &'a ThrowStmt,
    ) -> ThrowStmt {
        stmt.clone()
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    pub fn map_try_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a TryStmt) -> TryStmt {
        let nexpr = mapper.map_expr(&stmt.guarded_expr);
        let nreturns = stmt
            .returns
            .iter()
            .map(|v| mapper.map_var_decl(v))
            .collect();
        let nbody = mapper.map_block(&stmt.body);
        let ncatch_cls = stmt
            .catch_clauses
            .iter()
            .map(|cls| mapper.map_catch_clause(cls))
            .collect();
        TryStmt {
            guarded_expr: nexpr,
            returns: nreturns,
            body: nbody,
            catch_clauses: ncatch_cls,
            ..stmt.clone()
        }
    }

    pub fn map_catch_clause<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        cls: &'a CatchClause,
    ) -> CatchClause {
        let nparams = cls.params.iter().map(|v| mapper.map_var_decl(v)).collect();
        let nbody = mapper.map_block(&cls.body);
        CatchClause { params: nparams, body: nbody, ..cls.clone() }
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    pub fn map_revert_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a RevertStmt,
    ) -> RevertStmt {
        let args = stmt
            .args
            .iter()
            .map(|arg| mapper.map_atomic_expr(arg))
            .collect();
        RevertStmt { args, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn map_emit_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a EmitStmt) -> EmitStmt {
        let args = stmt
            .args
            .iter()
            .map(|arg| mapper.map_atomic_expr(arg))
            .collect();
        EmitStmt { args, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn map_return_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a ReturnStmt,
    ) -> ReturnStmt {
        let expr = stmt.expr.as_ref().map(|e| mapper.map_expr(e));
        ReturnStmt { expr, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn map_var_decl_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a VarDeclStmt,
    ) -> VarDeclStmt {
        // Map the assigned value first.
        let nvalue = stmt.value.as_ref().map(|e| mapper.map_expr(e));
        let nvdecl = stmt
            .vars
            .iter()
            .map(|v| v.as_ref().map(|u| mapper.map_var_decl(u)))
            .collect();
        VarDeclStmt { vars: nvdecl, value: nvalue, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    pub fn map_var_decl<'a, T: Map<'a> + ?Sized>(mapper: &mut T, vdecl: &'a VarDecl) -> VarDecl {
        let nvalue = vdecl.value.as_ref().map(|e| mapper.map_expr(e));
        VarDecl { value: nvalue, ..vdecl.clone() }
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn map_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a Expr) -> Expr {
        match expr {
            Expr::Var(v) => mapper.map_variable(v).into(),
            Expr::Lit(_) => expr.clone(),
            Expr::Call(e) => mapper.map_call_expr(e).into(),
            Expr::Tuple(e) => mapper.map_tuple_expr(e).into(),
            Expr::Index(e) => mapper.map_index_expr(e).into(),
            Expr::Slice(e) => mapper.map_slice_expr(e).into(),
            Expr::Member(e) => mapper.map_member_expr(e).into(),
            Expr::InlineArray(e) => mapper.map_inline_array_expr(e).into(),
            Expr::New(e) => mapper.map_new_expr(e).into(),
            Expr::TypeName(e) => mapper.map_type_expr(e).into(),
        }
    }

    pub fn map_call_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a CallExpr) -> CallExpr {
        let callee = mapper.map_callable_expr(&expr.callee);
        let arguments = expr
            .args
            .iter()
            .map(|arg| mapper.map_atomic_expr(arg))
            .collect();
        CallExpr { callee, args: arguments, ..expr.clone() }
    }

    pub fn map_callable_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a CalleeExpr,
    ) -> CalleeExpr {
        match expr {
            CalleeExpr::BuiltIn(_) => expr.clone(),
            CalleeExpr::FuncDef(f) => CalleeExpr::FuncDef(mapper.map_func_def(f)),
            CalleeExpr::ContractDef(c) => CalleeExpr::ContractDef(mapper.map_contract_def(c)),
            CalleeExpr::StructDef(s) => CalleeExpr::StructDef(mapper.map_struct_def(s)),
            CalleeExpr::MemberExpr(e) => CalleeExpr::MemberExpr(mapper.map_member_expr(e)),
            CalleeExpr::NewExpr(e) => CalleeExpr::NewExpr(mapper.map_new_expr(e)),
            CalleeExpr::TypeNameExpr(e) => CalleeExpr::TypeNameExpr(mapper.map_type_expr(e)),
        }
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------

    pub fn map_tuple_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a TupleExpr,
    ) -> TupleExpr {
        let nelems = expr
            .elems
            .iter()
            .map(|exp| exp.as_ref().map(|v| mapper.map_atomic_expr(v)))
            .collect();
        TupleExpr { elems: nelems, ..expr.clone() }
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    pub fn map_index_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a IndexExpr,
    ) -> IndexExpr {
        let base = mapper.map_variable(&expr.base);
        let index = expr
            .index
            .as_ref()
            .map(|idx| Box::new(mapper.map_atomic_expr(idx)));
        IndexExpr { base, index, ..expr.clone() }
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    pub fn map_slice_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a SliceExpr,
    ) -> SliceExpr {
        let base = mapper.map_variable(&expr.base);
        let start_idx = expr
            .start_index
            .as_ref()
            .map(|idx| Box::new(mapper.map_atomic_expr(idx)));
        let end_idx = expr
            .end_index
            .as_ref()
            .map(|idx| Box::new(mapper.map_atomic_expr(idx)));
        SliceExpr { base, start_index: start_idx, end_index: end_idx, ..expr.clone() }
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    pub fn map_member_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a MemberExpr,
    ) -> MemberExpr {
        let base = mapper.map_expr(&expr.base);
        MemberExpr { base: Box::new(base), ..expr.clone() }
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    pub fn map_conditional_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a ConditionalExpr,
    ) -> ConditionalExpr {
        let cond = mapper.map_atomic_expr(&expr.cond);
        let true_br = mapper.map_atomic_expr(&expr.true_br);
        let false_br = mapper.map_atomic_expr(&expr.false_br);
        ConditionalExpr {
            cond: Box::new(cond),
            true_br: Box::new(true_br),
            false_br: Box::new(false_br),
            ..expr.clone()
        }
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    pub fn map_inline_array_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a InlineArrayExpr,
    ) -> InlineArrayExpr {
        let nelems = expr
            .elems
            .iter()
            .map(|elem| mapper.map_atomic_expr(elem))
            .collect();
        InlineArrayExpr { elems: nelems, ..expr.clone() }
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    pub fn map_new_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a NewExpr) -> NewExpr {
        let ntyp = mapper.map_type(&expr.typ);
        NewExpr { typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    pub fn map_type_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a TypeNameExpr,
    ) -> TypeNameExpr {
        let ntyp = mapper.map_type(&expr.typ);
        TypeNameExpr { typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Atomic expression
    //-------------------------------------------------

    pub fn map_atomic_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a AtomicExpr,
    ) -> AtomicExpr {
        match expr {
            AtomicExpr::Lit(_) => expr.clone(),
            AtomicExpr::Var(v) => AtomicExpr::Var(mapper.map_variable(v)),
            AtomicExpr::Type(e) => AtomicExpr::Type(mapper.map_type_expr(e)),
        }
    }

    //-------------------------------------------------
    // Variable.
    //-------------------------------------------------

    pub fn map_variable<'a, T: Map<'a> + ?Sized>(_mapper: &mut T, var: &'a Variable) -> Variable {
        var.clone()
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn map_type<'a, T: Map<'a> + ?Sized>(_mapper: &mut T, typ: &'a Type) -> Type {
        typ.clone()
    }
}
