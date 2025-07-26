//! Module implementing the normalization design pattern for Solidity AST.
//!
//! This normalize pattern will transform an input data structure into a new
//! data structure of the same type and accumulate intermediate elements
//! generated during the normalization.

use crate::ast::*;

//------------------------------------------------------------------
// Trait for implementing the normalization utilities
//------------------------------------------------------------------

/// Trait implementing the normalization utilities for Solidity AST.
pub trait Normalize<'a, T> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn normalize_source_units(
        &mut self,
        acc: T,
        source_units: &'a [SourceUnit],
    ) -> (T, Vec<SourceUnit>) {
        default::normalize_source_units(self, acc, source_units)
    }

    fn normalize_source_unit(&mut self, acc: T, source_unit: &'a SourceUnit) -> (T, SourceUnit) {
        default::normalize_source_unit(self, acc, source_unit)
    }

    fn normalize_source_unit_elem(
        &mut self,
        acc: T,
        elem: &'a SourceUnitElem,
    ) -> (T, SourceUnitElem) {
        default::normalize_source_unit_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    fn normalize_pragma(&mut self, acc: T, pragma: &'a PragmaDir) -> (T, PragmaDir) {
        default::normalize_pragma(self, acc, pragma)
    }

    fn normalize_import(&mut self, acc: T, import: &'a ImportDir) -> (T, ImportDir) {
        default::normalize_import(self, acc, import)
    }

    fn normalize_using(&mut self, acc: T, using: &'a UsingDir) -> (T, UsingDir) {
        default::normalize_using(self, acc, using)
    }

    //-------------------------------------------------
    // Error & event definitions.
    //-------------------------------------------------

    fn normalize_error_def(&mut self, acc: T, error: &'a ErrorDef) -> (T, ErrorDef) {
        default::normalize_error_def(self, acc, error)
    }

    fn normalize_event_def(&mut self, acc: T, event: &'a EventDef) -> (T, EventDef) {
        default::normalize_event_def(self, acc, event)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn normalize_udv_type_def(
        &mut self,
        acc: T,
        typ: &'a UserTypeDef,
    ) -> (T, UserTypeDef) {
        default::normalize_udv_type_def(self, acc, typ)
    }

    fn normalize_struct_def(
        &mut self,
        acc: T,
        struct_: &'a StructDef,
    ) -> (T, StructDef) {
        default::normalize_struct_def(self, acc, struct_)
    }

    fn normalize_struct_field(&mut self, acc: T, field: &'a StructField) -> (T, StructField) {
        default::normalize_struct_field(self, acc, field)
    }

    fn normalize_enum_def(&mut self, acc: T, enum_: &'a EnumDef) -> (T, EnumDef) {
        default::normalize_enum_def(self, acc, enum_)
    }

    fn normalize_enum_elem(&mut self, acc: T, elem: &'a str) -> (T, String) {
        default::normalize_enum_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn normalize_contract_def(
        &mut self,
        acc: T,
        contract: &'a ContractDef,
    ) -> (T, ContractDef) {
        default::normalize_contract_def(self, acc, contract)
    }

    fn normalize_contract_elem(&mut self, acc: T, elem: &'a ContractElem) -> (T, ContractElem) {
        default::normalize_contract_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Function & block.
    //-------------------------------------------------

    fn normalize_func_def(
        &mut self,
        acc: T,
        func: &'a FunctionDef,
    ) -> (T, FunctionDef) {
        default::normalize_func_def(self, acc, func)
    }

    fn normalize_block(&mut self, acc: T, block: &'a Block) -> (T, Block) {
        default::normalize_block(self, acc, block)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn normalize_stmt(&mut self, acc: T, stmt: &'a Stmt) -> (T, Stmt) {
        default::normalize_stmt(self, acc, stmt)
    }

    fn normalize_asm_stmt(&mut self, acc: T, stmt: &'a AsmStmt) -> (T, AsmStmt) {
        default::normalize_asm_stmt(self, acc, stmt)
    }

    fn normalize_break_stmt(&mut self, acc: T, stmt: &'a BreakStmt) -> (T, BreakStmt) {
        default::normalize_break_stmt(self, acc, stmt)
    }

    fn normalize_continue_stmt(&mut self, acc: T, stmt: &'a ContinueStmt) -> (T, ContinueStmt) {
        default::normalize_continue_stmt(self, acc, stmt)
    }

    fn normalize_expr_stmt(&mut self, acc: T, stmt: &'a ExprStmt) -> (T, ExprStmt) {
        default::normalize_expr_stmt(self, acc, stmt)
    }

    fn normalize_if_stmt(&mut self, acc: T, stmt: &'a IfStmt) -> (T, IfStmt) {
        default::normalize_if_stmt(self, acc, stmt)
    }

    fn normalize_for_stmt(&mut self, acc: T, stmt: &'a ForStmt) -> (T, ForStmt) {
        default::normalize_for_stmt(self, acc, stmt)
    }

    fn normalize_while_stmt(&mut self, acc: T, stmt: &'a WhileStmt) -> (T, WhileStmt) {
        default::normalize_while_stmt(self, acc, stmt)
    }

    fn normalize_do_while_stmt(&mut self, acc: T, stmt: &'a DoWhileStmt) -> (T, DoWhileStmt) {
        default::normalize_do_while_stmt(self, acc, stmt)
    }

    fn normalize_throw_stmt(&mut self, acc: T, stmt: &'a ThrowStmt) -> (T, ThrowStmt) {
        default::normalize_throw_stmt(self, acc, stmt)
    }

    fn normalize_try_stmt(&mut self, acc: T, stmt: &'a TryStmt) -> (T, TryStmt) {
        default::normalize_try_stmt(self, acc, stmt)
    }

    fn normalize_catch_clause(&mut self, acc: T, cls: &'a CatchClause) -> (T, CatchClause) {
        default::normalize_catch_clause(self, acc, cls)
    }

    fn normalize_revert_stmt(&mut self, acc: T, stmt: &'a RevertStmt) -> (T, RevertStmt) {
        default::normalize_revert_stmt(self, acc, stmt)
    }

    fn normalize_emit_stmt(&mut self, acc: T, stmt: &'a EmitStmt) -> (T, EmitStmt) {
        default::normalize_emit_stmt(self, acc, stmt)
    }

    fn normalize_place_holder_stmt(
        &mut self,
        acc: T,
        stmt: &'a PlaceholderStmt,
    ) -> (T, PlaceholderStmt) {
        default::normalize_place_holder_stmt(self, acc, stmt)
    }

    fn normalize_return_stmt(&mut self, acc: T, stmt: &'a ReturnStmt) -> (T, ReturnStmt) {
        default::normalize_return_stmt(self, acc, stmt)
    }

    fn normalize_var_decl_stmt(&mut self, acc: T, stmt: &'a VarDeclStmt) -> (T, VarDeclStmt) {
        default::normalize_var_decl_stmt(self, acc, stmt)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn normalize_var_decl(
        &mut self,
        acc: T,
        vdecl: &'a VariableDecl,
    ) -> (T, VariableDecl) {
        default::normalize_var_decl(self, acc, vdecl)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn normalize_expr(&mut self, acc: T, expr: &'a Expr) -> (T, Expr) {
        default::normalize_expr(self, acc, expr)
    }

    fn normalize_unary_expr(&mut self, acc: T, expr: &'a UnaryExpr) -> (T, UnaryExpr) {
        default::normalize_unary_expr(self, acc, expr)
    }

    fn normalize_binary_expr(&mut self, acc: T, expr: &'a BinaryExpr) -> (T, BinaryExpr) {
        default::normalize_binary_expr(self, acc, expr)
    }

    fn normalize_assign_expr(&mut self, acc: T, expr: &'a AssignExpr) -> (T, AssignExpr) {
        default::normalize_assign_expr(self, acc, expr)
    }

    fn normalize_call_expr(&mut self, acc: T, expr: &'a CallExpr) -> (T, CallExpr) {
        default::normalize_call_expr(self, acc, expr)
    }

    fn normalize_call_args(&mut self, acc: T, expr: &'a CallArgs) -> (T, CallArgs) {
        default::normalize_call_args(self, acc, expr)
    }

    fn normalize_named_arg(&mut self, acc: T, expr: &'a NamedArg) -> (T, NamedArg) {
        default::normalize_named_arg(self, acc, expr)
    }

    fn normalize_call_opts_expr(&mut self, acc: T, expr: &'a CallOptsExpr) -> (T, CallOptsExpr) {
        default::normalize_call_opts_expr(self, acc, expr)
    }

    fn normalize_call_opt(&mut self, acc: T, call_opt: &'a CallOpt) -> (T, CallOpt) {
        default::normalize_call_opt(self, acc, call_opt)
    }

    fn normalize_tuple_expr(&mut self, acc: T, expr: &'a TupleExpr) -> (T, TupleExpr) {
        default::normalize_tuple_expr(self, acc, expr)
    }

    fn normalize_index_expr(&mut self, acc: T, expr: &'a IndexExpr) -> (T, IndexExpr) {
        default::normalize_index_expr(self, acc, expr)
    }

    fn normalize_slice_expr(&mut self, acc: T, expr: &'a SliceExpr) -> (T, SliceExpr) {
        default::normalize_slice_expr(self, acc, expr)
    }

    fn normalize_member_expr(&mut self, acc: T, expr: &'a MemberExpr) -> (T, MemberExpr) {
        default::normalize_member_expr(self, acc, expr)
    }

    fn normalize_conditional_expr(
        &mut self,
        acc: T,
        expr: &'a ConditionalExpr,
    ) -> (T, ConditionalExpr) {
        default::normalize_conditional_expr(self, acc, expr)
    }

    fn normalize_inline_array_expr(
        &mut self,
        acc: T,
        expr: &'a InlineArrayExpr,
    ) -> (T, InlineArrayExpr) {
        default::normalize_inline_array_expr(self, acc, expr)
    }

    fn normalize_new_expr(&mut self, acc: T, expr: &'a NewExpr) -> (T, NewExpr) {
        default::normalize_new_expr(self, acc, expr)
    }

    fn normalize_type_name_expr(&mut self, acc: T, expr: &'a TypeNameExpr) -> (T, TypeNameExpr) {
        default::normalize_type_name_expr(self, acc, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn normalize_ident(&mut self, acc: T, id: &'a Identifier) -> (T, Identifier) {
        default::normalize_ident(self, acc, id)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn normalize_type(&mut self, acc: T, typ: &'a Type) -> (T, Type) {
        default::normalize_type(self, acc, typ)
    }
}

//------------------------------------------------------------------
// Default normalization pattern
//------------------------------------------------------------------

/// Module contain default implementation of the AST normalization utilities.
pub mod default {
    use super::Normalize;
    use crate::ast::*;
    use std::borrow::Borrow;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn normalize_source_units<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        source_units: &'a [SourceUnit],
    ) -> (T, Vec<SourceUnit>) {
        let mut nsource_units = vec![];
        let nacc = source_units.iter().fold(acc, |acc2, source_unit| {
            let (nacc, nelem) = normalizer.normalize_source_unit(acc2, source_unit);
            nsource_units.push(nelem);
            nacc
        });
        (nacc, nsource_units)
    }

    pub fn normalize_source_unit<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        source_unit: &'a SourceUnit,
    ) -> (T, SourceUnit) {
        let mut nelems = vec![];
        let nacc = source_unit.elems.iter().fold(acc, |acc2, elem| {
            let (nacc, nelem) = normalizer.normalize_source_unit_elem(acc2, elem);
            nelems.push(nelem);
            nacc
        });
        let nsource_unit = SourceUnit { elems: nelems, ..source_unit.clone() };
        (nacc, nsource_unit)
    }

    pub fn normalize_source_unit_elem<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        elem: &'a SourceUnitElem,
    ) -> (T, SourceUnitElem) {
        match elem {
            SourceUnitElem::Pragma(pragma) => {
                let (nacc, npragma) = normalizer.normalize_pragma(acc, pragma);
                (nacc, npragma.into())
            }
            SourceUnitElem::Import(import) => {
                let (nacc, nimport) = normalizer.normalize_import(acc, import);
                (nacc, nimport.into())
            }
            SourceUnitElem::Using(using) => {
                let (nacc, nusing) = normalizer.normalize_using(acc, using);
                (nacc, nusing.into())
            }
            SourceUnitElem::Error(error) => {
                let (nacc, nerror) = normalizer.normalize_error_def(acc, error);
                (nacc, nerror.into())
            }
            SourceUnitElem::Var(var) => {
                let (nacc, nvar) = normalizer.normalize_var_decl(acc, var);
                (nacc, nvar.into())
            }
            SourceUnitElem::UserType(typ) => {
                let (nacc, ntyp) = normalizer.normalize_udv_type_def(acc, typ);
                (nacc, ntyp.into())
            }
            SourceUnitElem::Struct(struct_) => {
                let (nacc, nstruct) = normalizer.normalize_struct_def(acc, struct_);
                (nacc, nstruct.into())
            }
            SourceUnitElem::Func(func) => {
                let (nacc, nfunc) = normalizer.normalize_func_def(acc, func);
                (nacc, nfunc.into())
            }
            SourceUnitElem::Enum(enum_) => {
                let (nacc, nenum) = normalizer.normalize_enum_def(acc, enum_);
                (nacc, nenum.into())
            }
            SourceUnitElem::Contract(contract) => {
                let (nacc, ncontract) = normalizer.normalize_contract_def(acc, contract);
                (nacc, ncontract.into())
            }
        }
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    pub fn normalize_pragma<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        pragma: &'a PragmaDir,
    ) -> (T, PragmaDir) {
        (acc, pragma.clone())
    }

    pub fn normalize_import<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        import: &'a ImportDir,
    ) -> (T, ImportDir) {
        (acc, import.clone())
    }

    pub fn normalize_using<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        using: &'a UsingDir,
    ) -> (T, UsingDir) {
        (acc, using.clone())
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    pub fn normalize_error_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        error: &'a ErrorDef,
    ) -> (T, ErrorDef) {
        (acc, error.clone())
    }

    pub fn normalize_event_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        event: &'a EventDef,
    ) -> (T, EventDef) {
        (acc, event.clone())
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn normalize_udv_type_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        typ: &'a UserTypeDef,
    ) -> (T, UserTypeDef) {
        (acc, typ.clone())
    }

    pub fn normalize_struct_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        struct_: &'a StructDef,
    ) -> (T, StructDef) {
        let mut nfields = vec![];
        let nacc = struct_.fields.iter().fold(acc, |acc2, fld| {
            let (acc3, nfld) = normalizer.normalize_struct_field(acc2, fld);
            nfields.push(nfld);
            acc3
        });
        let nstruct = StructDef { fields: nfields, ..struct_.clone() };
        (nacc, nstruct)
    }

    pub fn normalize_struct_field<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        fld: &'a StructField,
    ) -> (T, StructField) {
        let (nacc, ntyp) = normalizer.normalize_type(acc, &fld.typ);
        let nfld = StructField { typ: ntyp, ..fld.clone() };
        (nacc, nfld)
    }

    pub fn normalize_enum_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        enum_: &'a EnumDef,
    ) -> (T, EnumDef) {
        let mut nelems = vec![];
        let nacc = enum_.elems.iter().fold(acc, |acc2, elem| {
            let (acc3, nelem) = normalizer.normalize_enum_elem(acc2, elem);
            nelems.push(nelem);
            acc3
        });
        let nenum = EnumDef { elems: nelems, ..enum_.clone() };
        (nacc, nenum)
    }

    pub fn normalize_enum_elem<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        elem: &'a str,
    ) -> (T, String) {
        (acc, elem.to_string())
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    pub fn normalize_contract_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        contract: &'a ContractDef,
    ) -> (T, ContractDef) {
        let mut nelems = vec![];
        let nacc = contract.body.iter().fold(acc, |acc2, elem| {
            let (acc3, nelem) = normalizer.normalize_contract_elem(acc2, elem);
            nelems.push(nelem);
            acc3
        });
        let ncontract = ContractDef { body: nelems, ..contract.clone() };
        (nacc, ncontract)
    }

    pub fn normalize_contract_elem<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        elem: &'a ContractElem,
    ) -> (T, ContractElem) {
        match elem {
            ContractElem::Using(u) => {
                let (nacc, nu) = normalizer.normalize_using(acc, u);
                (nacc, nu.into())
            }
            ContractElem::EventDef(e) => {
                let (nacc, ne) = normalizer.normalize_event_def(acc, e);
                (nacc, ne.into())
            }
            ContractElem::ErrorDef(e) => {
                let (nacc, ne) = normalizer.normalize_error_def(acc, e);
                (nacc, ne.into())
            }
            ContractElem::StructDef(s) => {
                let (nacc, ns) = normalizer.normalize_struct_def(acc, s);
                (nacc, ns.into())
            }
            ContractElem::EnumDef(e) => {
                let (nacc, ne) = normalizer.normalize_enum_def(acc, e);
                (nacc, ne.into())
            }
            ContractElem::UserTypeDef(t) => {
                let (nacc, nt) = normalizer.normalize_udv_type_def(acc, t);
                (nacc, nt.into())
            }
            ContractElem::VarDecl(v) => {
                let (nacc, nv) = normalizer.normalize_var_decl(acc, v);
                (nacc, nv.into())
            }
            ContractElem::FuncDef(f) => {
                let (acc, nf) = normalizer.normalize_func_def(acc, f);
                (acc, nf.into())
            }
        }
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    pub fn normalize_func_def<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        func: &'a FunctionDef,
    ) -> (T, FunctionDef) {
        let mut nparams = vec![];
        let nacc = func.params.iter().fold(acc, |acc2, p| {
            let (acc3, np) = normalizer.normalize_var_decl(acc2, p);
            nparams.push(np);
            acc3
        });
        let mut nmodifiers = vec![];
        let nacc = func.modifier_invocs.iter().fold(nacc, |acc2, m| {
            let (acc3, nm) = normalizer.normalize_call_expr(acc2, m);
            nmodifiers.push(nm);
            acc3
        });
        let mut nreturns = vec![];
        let nacc = func.returns.iter().fold(nacc, |acc2, p| {
            let (acc3, np) = normalizer.normalize_var_decl(acc2, p);
            nreturns.push(np);
            acc3
        });
        let (nacc, nbody) = match &func.body {
            Some(blk) => {
                let (acc2, nblk) = normalizer.normalize_block(nacc, blk);
                (acc2, Some(nblk))
            }
            None => (nacc, None),
        };
        let nfunc = FunctionDef {
            body: nbody,
            params: nparams,
            modifier_invocs: nmodifiers,
            returns: nreturns,
            ..func.clone()
        };
        (nacc, nfunc)
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn normalize_block<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        block: &'a Block,
    ) -> (T, Block) {
        let mut nstmts = vec![];
        let nacc = block.body.iter().fold(acc, |acc2, stmt| {
            let (acc3, nstmt) = normalizer.normalize_stmt(acc2, stmt);
            nstmts.push(nstmt);
            acc3
        });
        let nblock = Block { body: nstmts, ..block.clone() };
        (nacc, nblock)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn normalize_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a Stmt,
    ) -> (T, Stmt) {
        match stmt {
            Stmt::Asm(s) => {
                let (nacc, ns) = normalizer.normalize_asm_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Block(b) => {
                let (nacc, nb) = normalizer.normalize_block(acc, b);
                (nacc, nb.into())
            }
            Stmt::Break(s) => {
                let (nacc, ns) = normalizer.normalize_break_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Continue(s) => {
                let (nacc, ns) = normalizer.normalize_continue_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::DoWhile(s) => {
                let (nacc, ns) = normalizer.normalize_do_while_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Emit(s) => {
                let (nacc, ns) = normalizer.normalize_emit_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Expr(s) => {
                let (nacc, ns) = normalizer.normalize_expr_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::For(s) => {
                let (nacc, ns) = normalizer.normalize_for_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::If(s) => {
                let (nacc, ns) = normalizer.normalize_if_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Placeholder(s) => {
                let (nacc, ns) = normalizer.normalize_place_holder_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Return(s) => {
                let (nacc, ns) = normalizer.normalize_return_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Revert(s) => {
                let (nacc, ns) = normalizer.normalize_revert_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Throw(s) => {
                let (nacc, ns) = normalizer.normalize_throw_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::Try(s) => {
                let (nacc, ns) = normalizer.normalize_try_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::VarDecl(s) => {
                let (nacc, ns) = normalizer.normalize_var_decl_stmt(acc, s);
                (nacc, ns.into())
            }
            Stmt::While(s) => {
                let (nacc, ns) = normalizer.normalize_while_stmt(acc, s);
                (nacc, ns.into())
            }
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    pub fn normalize_asm_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        stmt: &'a AsmStmt,
    ) -> (T, AsmStmt) {
        (acc, stmt.clone())
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    pub fn normalize_break_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        stmt: &'a BreakStmt,
    ) -> (T, BreakStmt) {
        (acc, stmt.clone())
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    pub fn normalize_continue_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        stmt: &'a ContinueStmt,
    ) -> (T, ContinueStmt) {
        (acc, stmt.clone())
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    pub fn normalize_expr_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a ExprStmt,
    ) -> (T, ExprStmt) {
        let (nacc, expr) = normalizer.normalize_expr(acc, &stmt.expr);
        let nexpr = ExprStmt { expr, ..stmt.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn normalize_if_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a IfStmt,
    ) -> (T, IfStmt) {
        let (nacc, ncond) = normalizer.normalize_expr(acc, &stmt.condition);
        let (nacc, ntrue_br) = normalizer.normalize_stmt(nacc, stmt.true_branch.borrow());
        let (nacc, nfalse_br) = match &stmt.false_branch {
            Some(s) => {
                let (nacc, nfalse_br) = normalizer.normalize_stmt(nacc, s);
                (nacc, Some(nfalse_br))
            }
            None => (nacc, None),
        };
        let nstmt = IfStmt {
            condition: ncond,
            true_branch: Box::new(ntrue_br),
            false_branch: nfalse_br.map(Box::new),
            ..stmt.clone()
        };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    pub fn normalize_for_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a ForStmt,
    ) -> (T, ForStmt) {
        let (nacc, npre) = match stmt.pre_loop.as_ref() {
            Some(s) => {
                let (nacc, ns) = normalizer.normalize_stmt(acc, s);
                (nacc, Some(ns))
            }
            None => (acc, None),
        };
        let (nacc, npost) = match stmt.post_loop.as_ref() {
            Some(s) => {
                let (nacc, ns) = normalizer.normalize_stmt(nacc, s);
                (nacc, Some(ns))
            }
            None => (nacc, None),
        };
        let (nacc, ncond) = match stmt.condition.as_ref() {
            Some(e) => {
                let (nacc, ne) = normalizer.normalize_expr(nacc, e);
                (nacc, Some(ne))
            }
            None => (nacc, None),
        };
        // Turn body to a block first before performing normalization.
        let (nacc, nbody) = normalizer.normalize_stmt(nacc, stmt.body.borrow());
        let nstmt = ForStmt {
            pre_loop: npre.map(Box::new),
            post_loop: npost.map(Box::new),
            condition: ncond,
            body: Box::new(nbody),
            ..stmt.clone()
        };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    pub fn normalize_while_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a WhileStmt,
    ) -> (T, WhileStmt) {
        let (nacc, ncond) = normalizer.normalize_expr(acc, &stmt.condition);
        // Turn body to a block first before performing normalization.
        let (nacc, nbody) = normalizer.normalize_stmt(nacc, stmt.body.borrow());
        let nstmt = WhileStmt { condition: ncond, body: Box::new(nbody), ..stmt.clone() };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    pub fn normalize_do_while_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a DoWhileStmt,
    ) -> (T, DoWhileStmt) {
        let (nacc, ncond) = normalizer.normalize_expr(acc, &stmt.condition);
        // Turn body to a block first before performing normalization.
        let (nacc, nbody) = normalizer.normalize_stmt(nacc, stmt.body.borrow());
        let nstmt = DoWhileStmt { condition: ncond, body: Box::new(nbody), ..stmt.clone() };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    pub fn normalize_throw_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        stmt: &'a ThrowStmt,
    ) -> (T, ThrowStmt) {
        (acc, stmt.clone())
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    pub fn normalize_try_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a TryStmt,
    ) -> (T, TryStmt) {
        let (nacc, nexpr) = normalizer.normalize_expr(acc, &stmt.guarded_expr);
        let mut nreturn_params = vec![];
        let nacc = stmt.returns.iter().fold(nacc, |acc2, v| {
            let (acc3, nv) = normalizer.normalize_var_decl(acc2, v);
            nreturn_params.push(nv);
            acc3
        });
        let (nacc, nbody) = normalizer.normalize_block(nacc, &stmt.body);
        let mut ncatch_cls = vec![];
        let nacc = stmt.catch_clauses.iter().fold(nacc, |acc2, cls| {
            let (acc3, ncls) = normalizer.normalize_catch_clause(acc2, cls);
            ncatch_cls.push(ncls);
            acc3
        });
        let nstmt = TryStmt {
            guarded_expr: nexpr,
            returns: nreturn_params,
            body: nbody,
            catch_clauses: ncatch_cls,
            ..stmt.clone()
        };
        (nacc, nstmt)
    }

    pub fn normalize_catch_clause<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        cls: &'a CatchClause,
    ) -> (T, CatchClause) {
        let mut nparams = vec![];
        let nacc = cls.params.iter().fold(acc, |acc2, v| {
            let (acc3, nv) = normalizer.normalize_var_decl(acc2, v);
            nparams.push(nv);
            acc3
        });
        let (nacc, nbody) = normalizer.normalize_block(nacc, &cls.body);
        let ncls = CatchClause { params: nparams, body: nbody, ..cls.clone() };
        (nacc, ncls)
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    pub fn normalize_revert_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a RevertStmt,
    ) -> (T, RevertStmt) {
        let (nacc, arguments) = normalizer.normalize_call_args(acc, &stmt.args);
        let nstmt = RevertStmt { args: arguments, ..stmt.clone() };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn normalize_emit_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a EmitStmt,
    ) -> (T, EmitStmt) {
        let (nacc, arguments) = normalizer.normalize_call_args(acc, &stmt.args);
        let nstmt = EmitStmt { args: arguments, ..stmt.clone() };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn normalize_return_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a ReturnStmt,
    ) -> (T, ReturnStmt) {
        let (nacc, nexpr) = match &stmt.expr {
            Some(e) => {
                let (acc2, ne) = normalizer.normalize_expr(acc, e);
                (acc2, Some(ne))
            }
            None => (acc, None),
        };
        let nstmt = ReturnStmt { expr: nexpr, ..stmt.clone() };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // Place holder statement
    //-------------------------------------------------

    pub fn normalize_place_holder_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        stmt: &'a PlaceholderStmt,
    ) -> (T, PlaceholderStmt) {
        (acc, stmt.clone())
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn normalize_var_decl_stmt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        stmt: &'a VarDeclStmt,
    ) -> (T, VarDeclStmt) {
        // Normalize the assigned value first.
        let (nacc, nvalue) = match &stmt.value {
            Some(e) => {
                let (nacc, ne) = normalizer.normalize_expr(acc, e);
                (nacc, Some(ne))
            }
            None => (acc, None),
        };

        let mut nvdecl = vec![];
        let nacc = stmt.var_decls.iter().fold(nacc, |acc2, v| match v {
            Some(vd) => {
                let (acc3, nvd) = normalizer.normalize_var_decl(acc2, vd);
                nvdecl.push(Some(nvd));
                acc3
            }
            None => {
                nvdecl.push(None);
                acc2
            }
        });

        let nstmt = VarDeclStmt { var_decls: nvdecl, value: nvalue, ..stmt.clone() };
        (nacc, nstmt)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    pub fn normalize_var_decl<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        vdecl: &'a VariableDecl,
    ) -> (T, VariableDecl) {
        let (nacc, nvalue) = match &vdecl.value {
            Some(v) => {
                let (nacc, nv) = normalizer.normalize_expr(acc, v);
                (nacc, Some(nv))
            }
            None => (acc, None),
        };
        let nvdecl = VariableDecl { value: nvalue, ..vdecl.clone() };
        (nacc, nvdecl)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn normalize_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a Expr,
    ) -> (T, Expr) {
        use Expr::*;
        match expr {
            Ident(v) => {
                let (nacc, nv) = normalizer.normalize_ident(acc, v);
                (nacc, nv.into())
            }
            Lit(_) => (acc, expr.clone()),
            Unary(e) => {
                let (nacc, ne) = normalizer.normalize_unary_expr(acc, e);
                (nacc, ne.into())
            }
            Binary(e) => {
                let (nacc, ne) = normalizer.normalize_binary_expr(acc, e);
                (nacc, ne.into())
            }
            Assign(e) => {
                let (nacc, ne) = normalizer.normalize_assign_expr(acc, e);
                (nacc, ne.into())
            }
            Call(e) => {
                let (nacc, ne) = normalizer.normalize_call_expr(acc, e);
                (nacc, ne.into())
            }
            CallOpts(e) => {
                let (nacc, ne) = normalizer.normalize_call_opts_expr(acc, e);
                (nacc, ne.into())
            }
            Tuple(e) => {
                let (nacc, ne) = normalizer.normalize_tuple_expr(acc, e);
                (nacc, ne.into())
            }
            Index(e) => {
                let (nacc, ne) = normalizer.normalize_index_expr(acc, e);
                (nacc, ne.into())
            }
            Slice(e) => {
                let (nacc, ne) = normalizer.normalize_slice_expr(acc, e);
                (nacc, ne.into())
            }
            Member(e) => {
                let (nacc, ne) = normalizer.normalize_member_expr(acc, e);
                (nacc, ne.into())
            }
            Conditional(e) => {
                let (nacc, ne) = normalizer.normalize_conditional_expr(acc, e);
                (nacc, ne.into())
            }
            InlineArray(e) => {
                let (nacc, ne) = normalizer.normalize_inline_array_expr(acc, e);
                (nacc, ne.into())
            }
            New(e) => {
                let (nacc, ne) = normalizer.normalize_new_expr(acc, e);
                (nacc, ne.into())
            }
            TypeName(e) => {
                let (nacc, ne) = normalizer.normalize_type_name_expr(acc, e);
                (nacc, ne.into())
            }
        }
    }

    //-------------------------------------------------
    // Unary expression.
    //-------------------------------------------------

    pub fn normalize_unary_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a UnaryExpr,
    ) -> (T, UnaryExpr) {
        let (nacc, nopr) = normalizer.normalize_expr(acc, &expr.operand);
        let nexpr = UnaryExpr { operand: Box::new(nopr), ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Binary expression.
    //-------------------------------------------------

    pub fn normalize_binary_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a BinaryExpr,
    ) -> (T, BinaryExpr) {
        let (nacc, nlhs) = normalizer.normalize_expr(acc, &expr.left);
        let (nacc, nrhs) = normalizer.normalize_expr(nacc, &expr.right);
        let nexpr = BinaryExpr {
            left: Box::new(nlhs),
            right: Box::new(nrhs),
            ..expr.clone()
        };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Assignment expression.
    //-------------------------------------------------

    pub fn normalize_assign_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a AssignExpr,
    ) -> (T, AssignExpr) {
        let (nacc, nlhs) = normalizer.normalize_expr(acc, &expr.left);
        let (nacc, nrhs) = normalizer.normalize_expr(nacc, &expr.right);
        let nexpr = AssignExpr {
            left: Box::new(nlhs),
            right: Box::new(nrhs),
            ..expr.clone()
        };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    pub fn normalize_call_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a CallExpr,
    ) -> (T, CallExpr) {
        let (nacc, ncallee) = normalizer.normalize_expr(acc, &expr.callee);
        let (nacc, nargs) = normalizer.normalize_call_args(nacc, &expr.args);
        let nexpr = CallExpr { callee: Box::new(ncallee), args: nargs, ..expr.clone() };
        (nacc, nexpr)
    }

    pub fn normalize_call_args<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a CallArgs,
    ) -> (T, CallArgs) {
        match expr {
            CallArgs::Unnamed(args) => {
                let mut nargs = vec![];
                let nacc = args.iter().fold(acc, |acc2, arg| {
                    let (acc3, narg) = normalizer.normalize_expr(acc2, arg);
                    nargs.push(narg);
                    acc3
                });
                (nacc, CallArgs::Unnamed(nargs))
            }
            CallArgs::Named(args) => {
                let mut nargs = vec![];
                let nacc = args.iter().fold(acc, |acc2, arg| {
                    let (acc3, narg) = normalizer.normalize_named_arg(acc2, arg);
                    nargs.push(narg);
                    acc3
                });
                (nacc, CallArgs::Named(nargs))
            }
        }
    }

    pub fn normalize_named_arg<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a NamedArg,
    ) -> (T, NamedArg) {
        let (nacc, nvalue) = normalizer.normalize_expr(acc, &expr.value);
        let nexpr = NamedArg { value: nvalue, ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Function call options expression.
    //-------------------------------------------------

    pub fn normalize_call_opts_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a CallOptsExpr,
    ) -> (T, CallOptsExpr) {
        let (nacc, ncallee) = normalizer.normalize_expr(acc, &expr.callee);
        let mut nopts = vec![];
        let nacc = expr.call_opts.iter().fold(nacc, |acc2, opt| {
            let (acc3, nopt) = normalizer.normalize_call_opt(acc2, opt);
            nopts.push(nopt.clone());
            acc3
        });
        let nexpr = CallOptsExpr { callee: Box::new(ncallee), call_opts: nopts, ..expr.clone() };
        (nacc, nexpr)
    }

    pub fn normalize_call_opt<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        call_opt: &'a CallOpt,
    ) -> (T, CallOpt) {
        let (nacc, nvalue) = normalizer.normalize_expr(acc, &call_opt.value);
        let ncall_opt = CallOpt { value: nvalue, ..call_opt.clone() };
        (nacc, ncall_opt)
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------

    pub fn normalize_tuple_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a TupleExpr,
    ) -> (T, TupleExpr) {
        let mut nelems = vec![];
        let nacc = expr.elems.iter().fold(acc, |acc2, exp| match exp {
            Some(e) => {
                let (acc3, nexp) = normalizer.normalize_expr(acc2, e);
                nelems.push(Some(nexp));
                acc3
            }
            None => {
                nelems.push(None);
                acc2
            }
        });
        let nexpr = TupleExpr { elems: nelems, ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    pub fn normalize_index_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a IndexExpr,
    ) -> (T, IndexExpr) {
        let (nacc, nbase) = normalizer.normalize_expr(acc, &expr.base_expr);
        let (nacc, nindex) = match &expr.index {
            Some(e) => {
                let (acc2, ne) = normalizer.normalize_expr(nacc, e);
                (acc2, Some(Box::new(ne)))
            }
            None => (nacc, None),
        };
        let nexpr = IndexExpr { base_expr: Box::new(nbase), index: nindex, ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    pub fn normalize_slice_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a SliceExpr,
    ) -> (T, SliceExpr) {
        let (nacc, nbase) = normalizer.normalize_expr(acc, &expr.base_expr);
        let (nacc, nstart) = match &expr.start_index {
            Some(e) => {
                let (nacc, ne) = normalizer.normalize_expr(nacc, e);
                (nacc, Some(Box::new(ne)))
            }
            None => (nacc, None),
        };
        let (nacc, nend) = match &expr.end_index {
            Some(e) => {
                let (nacc, ne) = normalizer.normalize_expr(nacc, e);
                (nacc, Some(Box::new(ne)))
            }
            None => (nacc, None),
        };
        let nexpr = SliceExpr {
            base_expr: Box::new(nbase),
            start_index: nstart,
            end_index: nend,
            ..expr.clone()
        };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    pub fn normalize_member_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a MemberExpr,
    ) -> (T, MemberExpr) {
        let (nacc, nbase) = normalizer.normalize_expr(acc, &expr.base);
        let nexpr = MemberExpr { base: Box::new(nbase), ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    pub fn normalize_conditional_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a ConditionalExpr,
    ) -> (T, ConditionalExpr) {
        let (nacc, ncond) = normalizer.normalize_expr(acc, &expr.cond);
        let (nacc, ntrue_br) = normalizer.normalize_expr(nacc, &expr.true_br);
        let (nacc, nfalse_br) = normalizer.normalize_expr(nacc, &expr.false_br);
        let nexpr = ConditionalExpr {
            cond: Box::new(ncond),
            true_br: Box::new(ntrue_br),
            false_br: Box::new(nfalse_br),
            ..expr.clone()
        };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    pub fn normalize_inline_array_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a InlineArrayExpr,
    ) -> (T, InlineArrayExpr) {
        let mut nelems = vec![];
        let nacc = expr.elems.iter().fold(acc, |acc2, e| {
            let (acc3, ne) = normalizer.normalize_expr(acc2, e);
            nelems.push(ne);
            acc3
        });
        let nexpr = InlineArrayExpr { elems: nelems, ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    pub fn normalize_new_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a NewExpr,
    ) -> (T, NewExpr) {
        let (nacc, ntyp) = normalizer.normalize_type(acc, &expr.typ);
        let nexpr = NewExpr { typ: ntyp, ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    pub fn normalize_type_name_expr<'a, T, F: Normalize<'a, T> + ?Sized>(
        normalizer: &mut F,
        acc: T,
        expr: &'a TypeNameExpr,
    ) -> (T, TypeNameExpr) {
        let (nacc, ntyp) = normalizer.normalize_type(acc, &expr.typ);
        let nexpr = TypeNameExpr { typ: ntyp, ..expr.clone() };
        (nacc, nexpr)
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn normalize_ident<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        id: &'a Identifier,
    ) -> (T, Identifier) {
        (acc, id.clone())
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn normalize_type<'a, T, F: Normalize<'a, T> + ?Sized>(
        _normalizer: &mut F,
        acc: T,
        typ: &'a Type,
    ) -> (T, Type) {
        (acc, typ.clone())
    }
}
