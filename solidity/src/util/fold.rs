//! Module implementing the fold (left to right) pattern for Solidity AST.
//!
//! This map pattern will compute output of the same type when traversing all
//! data structures in the AST.

use core::metadata::DataLoc;

use crate::ast::*;

//------------------------------------------------------------------
// Trait providing the fold pattern.
//------------------------------------------------------------------

/// Trait implementing the map design pattern for Solidity AST.
pub trait Fold<'a, T> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn fold_source_units(&mut self, acc: T, source_units: &'a [SourceUnit]) -> T {
        default::fold_source_units(self, acc, source_units)
    }

    fn fold_source_unit(&mut self, acc: T, source_unit: &'a SourceUnit) -> T {
        default::fold_source_unit(self, acc, source_unit)
    }

    fn fold_source_unit_elem(&mut self, acc: T, elem: &'a SourceUnitElem) -> T {
        default::fold_source_unit_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    fn fold_pragma(&mut self, acc: T, pragma: &'a PragmaDir) -> T {
        default::fold_pragma(self, acc, pragma)
    }

    fn fold_import(&mut self, acc: T, import: &'a ImportDir) -> T {
        default::fold_import(self, acc, import)
    }

    //-------------------------------------------------
    // Using directives
    //-------------------------------------------------

    fn fold_using(&mut self, acc: T, using: &'a UsingDir) -> T {
        default::fold_using(self, acc, using)
    }

    fn fold_using_kind(&mut self, acc: T, kind: &'a UsingKind) -> T {
        default::fold_using_kind(self, acc, kind)
    }

    fn fold_using_func(&mut self, acc: T, func: &'a UsingFunc) -> T {
        default::fold_using_func(self, acc, func)
    }

    fn fold_using_lib(&mut self, acc: T, lib: &'a UsingLib) -> T {
        default::fold_using_lib(self, acc, lib)
    }

    //-------------------------------------------------
    // Error & event definitions.
    //-------------------------------------------------

    fn fold_error_def(&mut self, acc: T, error: &'a ErrorDef) -> T {
        default::fold_error_def(self, acc, error)
    }

    fn fold_event_def(&mut self, acc: T, event: &'a EventDef) -> T {
        default::fold_event_def(self, acc, event)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn fold_udv_type_def(&mut self, acc: T, typ: &'a UserTypeDef) -> T {
        default::fold_udv_type_def(self, acc, typ)
    }

    fn fold_struct_def(&mut self, acc: T, struct_: &'a StructDef) -> T {
        default::fold_struct_def(self, acc, struct_)
    }

    fn fold_struct_field(&mut self, acc: T, field: &'a StructField) -> T {
        default::fold_struct_field(self, acc, field)
    }

    fn fold_enum_def(&mut self, acc: T, enum_: &'a EnumDef) -> T {
        default::fold_enum_def(self, acc, enum_)
    }

    fn fold_enum_element(&mut self, acc: T, elem: &'a str) -> T {
        default::fold_enum_element(self, acc, elem)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn fold_contract_def(&mut self, acc: T, contract: &'a ContractDef) -> T {
        default::fold_contract_def(self, acc, contract)
    }

    fn fold_contract_elem(&mut self, acc: T, elem: &'a ContractElem) -> T {
        default::fold_contract_elem(self, acc, elem)
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    fn fold_func_def(&mut self, acc: T, func: &'a FunctionDef) -> T {
        default::fold_func_def(self, acc, func)
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    fn fold_block(&mut self, acc: T, block: &'a Block) -> T {
        default::fold_block(self, acc, block)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn fold_stmt(&mut self, acc: T, stmt: &'a Stmt) -> T {
        default::fold_stmt(self, acc, stmt)
    }

    fn fold_asm_stmt(&mut self, acc: T, stmt: &'a AsmStmt) -> T {
        default::fold_asm_stmt(self, acc, stmt)
    }

    fn fold_break_stmt(&mut self, acc: T, stmt: &'a BreakStmt) -> T {
        default::fold_break_stmt(self, acc, stmt)
    }

    fn fold_continue_stmt(&mut self, acc: T, stmt: &'a ContinueStmt) -> T {
        default::fold_continue_stmt(self, acc, stmt)
    }

    fn fold_expr_stmt(&mut self, acc: T, stmt: &'a ExprStmt) -> T {
        default::fold_expr_stmt(self, acc, stmt)
    }

    fn fold_if_stmt(&mut self, acc: T, stmt: &'a IfStmt) -> T {
        default::fold_if_stmt(self, acc, stmt)
    }

    fn fold_for_stmt(&mut self, acc: T, stmt: &'a ForStmt) -> T {
        default::fold_for_stmt(self, acc, stmt)
    }

    fn fold_while_stmt(&mut self, acc: T, stmt: &'a WhileStmt) -> T {
        default::fold_while_stmt(self, acc, stmt)
    }

    fn fold_do_while_stmt(&mut self, acc: T, stmt: &'a DoWhileStmt) -> T {
        default::fold_do_while_stmt(self, acc, stmt)
    }

    fn fold_throw_stmt(&mut self, acc: T, stmt: &'a ThrowStmt) -> T {
        default::fold_throw_stmt(self, acc, stmt)
    }

    fn fold_try_stmt(&mut self, acc: T, stmt: &'a TryStmt) -> T {
        default::fold_try_stmt(self, acc, stmt)
    }

    fn fold_catch_clause(&mut self, acc: T, cls: &'a CatchClause) -> T {
        default::fold_catch_clause(self, acc, cls)
    }

    fn fold_revert_stmt(&mut self, acc: T, stmt: &'a RevertStmt) -> T {
        default::fold_revert_stmt(self, acc, stmt)
    }

    fn fold_emit_stmt(&mut self, acc: T, stmt: &'a EmitStmt) -> T {
        default::fold_emit_stmt(self, acc, stmt)
    }

    fn fold_place_holder_stmt(&mut self, acc: T, stmt: &'a PlaceholderStmt) -> T {
        default::fold_place_holder_stmt(self, acc, stmt)
    }

    fn fold_return_stmt(&mut self, acc: T, stmt: &'a ReturnStmt) -> T {
        default::fold_return_stmt(self, acc, stmt)
    }

    fn fold_var_decl_stmt(&mut self, acc: T, stmt: &'a VarDeclStmt) -> T {
        default::fold_var_decl_stmt(self, acc, stmt)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn fold_var_decl(&mut self, acc: T, var: &'a VariableDecl) -> T {
        default::fold_var_decl(self, acc, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn fold_expr(&mut self, acc: T, expr: &'a Expr) -> T {
        default::fold_expr(self, acc, expr)
    }

    fn fold_unary_expr(&mut self, acc: T, expr: &'a UnaryExpr) -> T {
        default::fold_unary_expr(self, acc, expr)
    }

    fn fold_unary_op(&mut self, acc: T, op: &'a UnaryOp) -> T {
        default::fold_unary_op(self, acc, op)
    }

    fn fold_binary_expr(&mut self, acc: T, expr: &'a BinaryExpr) -> T {
        default::fold_binary_expr(self, acc, expr)
    }

    fn fold_binary_op(&mut self, acc: T, op: &'a BinOp) -> T {
        default::fold_binary_op(self, acc, op)
    }

    fn fold_assign_expr(&mut self, acc: T, expr: &'a AssignExpr) -> T {
        default::fold_assign_expr(self, acc, expr)
    }

    //-------------------------------------------------
    // Function call
    //-------------------------------------------------

    fn fold_call_expr(&mut self, acc: T, expr: &'a CallExpr) -> T {
        default::fold_call_expr(self, acc, expr)
    }

    fn fold_call_args(&mut self, acc: T, args: &'a CallArgs) -> T {
        default::fold_call_args(self, acc, args)
    }

    fn fold_named_arg(&mut self, acc: T, expr: &'a NamedArg) -> T {
        default::fold_named_arg(self, acc, expr)
    }

    //-------------------------------------------------
    // Function call options
    //-------------------------------------------------

    fn fold_call_opts_expr(&mut self, acc: T, expr: &'a CallOptsExpr) -> T {
        default::fold_call_opts_expr(self, acc, expr)
    }

    fn fold_call_opts(&mut self, acc: T, expr: &'a [CallOpt]) -> T {
        default::fold_call_opts(self, acc, expr)
    }

    fn fold_call_opt(&mut self, acc: T, expr: &'a CallOpt) -> T {
        default::fold_call_opt(self, acc, expr)
    }

    //-------------------------------------------------
    // Tuple
    //-------------------------------------------------

    fn fold_tuple_expr(&mut self, acc: T, expr: &'a TupleExpr) -> T {
        default::fold_tuple_expr(self, acc, expr)
    }

    fn fold_index_expr(&mut self, acc: T, expr: &'a IndexExpr) -> T {
        default::fold_index_expr(self, acc, expr)
    }

    fn fold_slice_expr(&mut self, acc: T, expr: &'a SliceExpr) -> T {
        default::fold_slice_expr(self, acc, expr)
    }

    fn fold_member_expr(&mut self, acc: T, expr: &'a MemberExpr) -> T {
        default::fold_member_expr(self, acc, expr)
    }

    fn fold_conditional_expr(&mut self, acc: T, expr: &'a ConditionalExpr) -> T {
        default::fold_conditional_expr(self, acc, expr)
    }

    fn fold_inline_array_expr(&mut self, acc: T, expr: &'a InlineArrayExpr) -> T {
        default::fold_inline_array_expr(self, acc, expr)
    }

    fn fold_new_expr(&mut self, acc: T, expr: &'a NewExpr) -> T {
        default::fold_new_expr(self, acc, expr)
    }

    fn fold_type_name_expr(&mut self, acc: T, expr: &'a TypeNameExpr) -> T {
        default::fold_type_name_expr(self, acc, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn fold_ident(&mut self, acc: T, id: &'a Identifier) -> T {
        default::fold_ident(self, acc, id)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn fold_lit(&mut self, acc: T, c: &'a Lit) -> T {
        default::fold_lit(self, acc, c)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn fold_name(&mut self, acc: T, name: &'a Name) -> T {
        default::fold_name(self, acc, name)
    }

    fn fold_name_opt(&mut self, acc: T, name: &'a Option<Name>) -> T {
        default::fold_name_opt(self, acc, name)
    }

    fn fold_name_path(&mut self, acc: T, name_path: &'a NamePath) -> T {
        default::fold_name_path(self, acc, name_path)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn fold_type(&mut self, acc: T, typ: &'a Type) -> T {
        default::fold_type(self, acc, typ)
    }

    fn fold_bytes_type(&mut self, acc: T, typ: &'a BytesType) -> T {
        default::fold_bytes_type(self, acc, typ)
    }

    fn fold_string_type(&mut self, acc: T, typ: &'a StringType) -> T {
        default::fold_string_type(self, acc, typ)
    }

    fn fold_array_type(&mut self, acc: T, typ: &'a ArrayType) -> T {
        default::fold_array_type(self, acc, typ)
    }

    fn fold_struct_type(&mut self, acc: T, typ: &'a StructType) -> T {
        default::fold_struct_type(self, acc, typ)
    }

    fn fold_enum_type(&mut self, acc: T, typ: &'a EnumType) -> T {
        default::fold_enum_type(self, acc, typ)
    }

    fn fold_slice_type(&mut self, acc: T, typ: &'a SliceType) -> T {
        default::fold_slice_type(self, acc, typ)
    }

    fn fold_tuple_type(&mut self, acc: T, typ: &'a TupleType) -> T {
        default::fold_tuple_type(self, acc, typ)
    }

    fn fold_mapping_type(&mut self, acc: T, typ: &'a MappingType) -> T {
        default::fold_mapping_type(self, acc, typ)
    }

    fn fold_func_type(&mut self, acc: T, typ: &'a FunctionType) -> T {
        default::fold_func_type(self, acc, typ)
    }

    fn fold_type_name(&mut self, acc: T, typ: &'a UserType) -> T {
        default::fold_type_name(self, acc, typ)
    }

    fn fold_contract_type(&mut self, acc: T, typ: &'a ContractType) -> T {
        default::fold_contract_type(self, acc, typ)
    }

    fn fold_magic_type(&mut self, acc: T, typ: &'a MagicType) -> T {
        default::fold_magic_type(self, acc, typ)
    }

    //-------------------------------------------------
    // Data location.
    //-------------------------------------------------

    fn fold_data_loc_opt(&mut self, acc: T, dloc: &'a Option<DataLoc>) -> T {
        default::fold_data_loc_opt(self, acc, dloc)
    }

    fn fold_data_loc(&mut self, acc: T, dloc: &'a DataLoc) -> T {
        default::fold_data_loc(self, acc, dloc)
    }
}

//------------------------------------------------------------------
// Default folding pattern
//------------------------------------------------------------------

/// Module contain default implementation of the folding pattern.
pub mod default {
    use core::metadata::DataLoc;

    use super::Fold;
    use crate::ast::*;

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
        folder: &mut F,
        acc: T,
        source_unit: &'a SourceUnit,
    ) -> T {
        source_unit
            .elems
            .iter()
            .fold(acc, |acc2, elem| folder.fold_source_unit_elem(acc2, elem))
    }

    pub fn fold_source_unit_elem<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        elem: &'a SourceUnitElem,
    ) -> T {
        match elem {
            SourceUnitElem::Pragma(p) => folder.fold_pragma(acc, p),
            SourceUnitElem::Import(i) => folder.fold_import(acc, i),
            SourceUnitElem::Using(u) => folder.fold_using(acc, u),
            SourceUnitElem::Error(e) => folder.fold_error_def(acc, e),
            SourceUnitElem::Var(v) => folder.fold_var_decl(acc, v),
            SourceUnitElem::UserType(t) => folder.fold_udv_type_def(acc, t),
            SourceUnitElem::Struct(s) => folder.fold_struct_def(acc, s),
            SourceUnitElem::Func(f) => folder.fold_func_def(acc, f),
            SourceUnitElem::Enum(e) => folder.fold_enum_def(acc, e),
            SourceUnitElem::Contract(c) => folder.fold_contract_def(acc, c),
        }
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    pub fn fold_pragma<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _pragma: &PragmaDir,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    pub fn fold_import<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _import: &ImportDir,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Using directives
    //-------------------------------------------------

    pub fn fold_using<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        using: &'a UsingDir,
    ) -> T {
        let nacc = folder.fold_using_kind(acc, &using.kind);
        match &using.target_type {
            None => nacc,
            Some(t) => folder.fold_type(nacc, t),
        }
    }

    pub fn fold_using_kind<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        kind: &'a UsingKind,
    ) -> T {
        match kind {
            UsingKind::UsingLib(ulib) => folder.fold_using_lib(acc, ulib),
            UsingKind::UsingFunc(ufuncs) => ufuncs
                .iter()
                .fold(acc, |acc2, ufunc| folder.fold_using_func(acc2, ufunc)),
        }
    }

    pub fn fold_using_lib<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        ulib: &'a UsingLib,
    ) -> T {
        let nacc = folder.fold_name(acc, &ulib.lib_name);
        folder.fold_name_path(nacc, &ulib.lib_path)
    }

    pub fn fold_using_func<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        ufunc: &'a UsingFunc,
    ) -> T {
        let nacc = folder.fold_name(acc, &ufunc.func_name);
        folder.fold_name_path(nacc, &ufunc.func_path)
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    pub fn fold_error_def<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _error: &ErrorDef,
    ) -> T {
        acc
    }

    pub fn fold_event_def<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _event: &'a EventDef,
    ) -> T {
        acc
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn fold_udv_type_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a UserTypeDef,
    ) -> T {
        let res = folder.fold_name(acc, &typ.name);
        folder.fold_type(res, &typ.base_type)
    }

    pub fn fold_struct_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        struct_: &'a StructDef,
    ) -> T {
        struct_
            .fields
            .iter()
            .fold(acc, |acc2, fld| folder.fold_struct_field(acc2, fld))
    }

    pub fn fold_struct_field<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        field: &'a StructField,
    ) -> T {
        folder.fold_type(acc, &field.typ)
    }

    pub fn fold_enum_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        enum_: &'a EnumDef,
    ) -> T {
        enum_
            .elems
            .iter()
            .fold(acc, |acc2, elem| folder.fold_enum_element(acc2, elem))
    }

    pub fn fold_enum_element<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
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
        folder: &mut F,
        acc: T,
        contract: &'a ContractDef,
    ) -> T {
        contract
            .body
            .iter()
            .fold(acc, |acc2, elem| folder.fold_contract_elem(acc2, elem))
    }

    pub fn fold_contract_elem<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        elem: &'a ContractElem,
    ) -> T {
        match elem {
            ContractElem::Using(u) => folder.fold_using(acc, u),
            ContractElem::EventDef(e) => folder.fold_event_def(acc, e),
            ContractElem::ErrorDef(e) => folder.fold_error_def(acc, e),
            ContractElem::StructDef(s) => folder.fold_struct_def(acc, s),
            ContractElem::EnumDef(e) => folder.fold_enum_def(acc, e),
            ContractElem::UserTypeDef(t) => folder.fold_udv_type_def(acc, t),
            ContractElem::VarDecl(v) => folder.fold_var_decl(acc, v),
            ContractElem::FuncDef(f) => folder.fold_func_def(acc, f),
        }
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    pub fn fold_func_def<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        func: &'a FunctionDef,
    ) -> T {
        let res = func
            .params
            .iter()
            .fold(acc, |acc2, p| folder.fold_var_decl(acc2, p));
        let res = func
            .modifier_invocs
            .iter()
            .fold(res, |acc2, m| folder.fold_call_expr(acc2, m));
        let res = func
            .returns
            .iter()
            .fold(res, |acc2, p| folder.fold_var_decl(acc2, p));
        match &func.body {
            None => res,
            Some(blk) => folder.fold_block(res, blk),
        }
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn fold_block<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        block: &'a Block,
    ) -> T {
        block
            .body
            .iter()
            .fold(acc, |acc2, stmt| folder.fold_stmt(acc2, stmt))
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn fold_stmt<'a, T, F: Fold<'a, T> + ?Sized>(folder: &mut F, acc: T, stmt: &'a Stmt) -> T {
        match stmt {
            Stmt::Asm(s) => folder.fold_asm_stmt(acc, s),
            Stmt::Block(b) => folder.fold_block(acc, b),
            Stmt::Break(s) => folder.fold_break_stmt(acc, s),
            Stmt::Continue(s) => folder.fold_continue_stmt(acc, s),
            Stmt::Placeholder(s) => folder.fold_place_holder_stmt(acc, s),
            Stmt::DoWhile(s) => folder.fold_do_while_stmt(acc, s),
            Stmt::Emit(s) => folder.fold_emit_stmt(acc, s),
            Stmt::Expr(s) => folder.fold_expr_stmt(acc, s),
            Stmt::For(s) => folder.fold_for_stmt(acc, s),
            Stmt::If(s) => folder.fold_if_stmt(acc, s),
            Stmt::Return(s) => folder.fold_return_stmt(acc, s),
            Stmt::Revert(s) => folder.fold_revert_stmt(acc, s),
            Stmt::Throw(s) => folder.fold_throw_stmt(acc, s),
            Stmt::Try(s) => folder.fold_try_stmt(acc, s),
            Stmt::VarDecl(s) => folder.fold_var_decl_stmt(acc, s),
            Stmt::While(s) => folder.fold_while_stmt(acc, s),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    pub fn fold_asm_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
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
        _folder: &mut F,
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
        _folder: &mut F,
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
        folder: &mut F,
        acc: T,
        stmt: &'a ExprStmt,
    ) -> T {
        folder.fold_expr(acc, &stmt.expr)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn fold_if_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a IfStmt,
    ) -> T {
        let res = folder.fold_expr(acc, &stmt.condition);
        let res = folder.fold_stmt(res, &stmt.true_branch);
        match &stmt.false_branch {
            None => res,
            Some(s) => folder.fold_stmt(res, s),
        }
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    pub fn fold_for_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a ForStmt,
    ) -> T {
        let res = match &stmt.pre_loop {
            Some(stmt) => folder.fold_stmt(acc, stmt),
            None => acc,
        };
        let res = match &stmt.condition {
            Some(exp) => folder.fold_expr(res, exp),
            None => res,
        };
        let res = match &stmt.post_loop {
            Some(stmt) => folder.fold_stmt(res, stmt),
            None => res,
        };
        folder.fold_stmt(res, &stmt.body)
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    pub fn fold_while_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a WhileStmt,
    ) -> T {
        let res = folder.fold_expr(acc, &stmt.condition);
        folder.fold_stmt(res, &stmt.body)
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    pub fn fold_do_while_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a DoWhileStmt,
    ) -> T {
        let res = folder.fold_expr(acc, &stmt.condition);
        folder.fold_stmt(res, &stmt.body)
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    pub fn fold_throw_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
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
        folder: &mut F,
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
        folder: &mut F,
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
        folder: &mut F,
        acc: T,
        stmt: &'a RevertStmt,
    ) -> T {
        let res = match &stmt.error {
            Some(exp) => folder.fold_expr(acc, exp),
            None => acc,
        };
        folder.fold_call_args(res, &stmt.args)
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn fold_emit_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a EmitStmt,
    ) -> T {
        folder.fold_call_args(acc, &stmt.args)
    }

    //-------------------------------------------------
    // Place holder statement
    //-------------------------------------------------

    pub fn fold_place_holder_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _stmt: &'a PlaceholderStmt,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn fold_return_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a ReturnStmt,
    ) -> T {
        match &stmt.expr {
            Some(expr) => folder.fold_expr(acc, expr),
            None => acc,
        }
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn fold_var_decl_stmt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        stmt: &'a VarDeclStmt,
    ) -> T {
        // Fold the assigned value first.
        let res = match &stmt.value {
            Some(exp) => folder.fold_expr(acc, exp),
            None => acc,
        };
        stmt.var_decls.iter().fold(res, |acc2, v| match v {
            Some(v) => folder.fold_var_decl(acc2, v),
            None => acc2,
        })
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    pub fn fold_var_decl<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        vdecl: &'a VariableDecl,
    ) -> T {
        match &vdecl.value {
            Some(exp) => folder.fold_expr(acc, exp),
            None => acc,
        }
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn fold_expr<'a, T, F: Fold<'a, T> + ?Sized>(folder: &mut F, acc: T, expr: &'a Expr) -> T {
        match expr {
            Expr::Ident(v) => folder.fold_ident(acc, v),
            Expr::Lit(c) => folder.fold_lit(acc, c),
            Expr::Unary(e) => folder.fold_unary_expr(acc, e),
            Expr::Binary(e) => folder.fold_binary_expr(acc, e),
            Expr::Assign(e) => folder.fold_assign_expr(acc, e),
            Expr::Call(e) => folder.fold_call_expr(acc, e),
            Expr::CallOpts(e) => folder.fold_call_opts_expr(acc, e),
            Expr::Tuple(e) => folder.fold_tuple_expr(acc, e),
            Expr::Index(e) => folder.fold_index_expr(acc, e),
            Expr::Slice(e) => folder.fold_slice_expr(acc, e),
            Expr::Member(e) => folder.fold_member_expr(acc, e),
            Expr::Conditional(e) => folder.fold_conditional_expr(acc, e),
            Expr::InlineArray(e) => folder.fold_inline_array_expr(acc, e),
            Expr::New(e) => folder.fold_new_expr(acc, e),
            Expr::TypeName(e) => folder.fold_type_name_expr(acc, e),
        }
    }

    //-------------------------------------------------
    // Unary expression.
    //-------------------------------------------------

    pub fn fold_unary_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a UnaryExpr,
    ) -> T {
        let res = folder.fold_unary_op(acc, &expr.op);
        folder.fold_expr(res, &expr.operand)
    }

    pub fn fold_unary_op<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _op: &'a UnaryOp,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Binary expression.
    //-------------------------------------------------

    pub fn fold_binary_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a BinaryExpr,
    ) -> T {
        let res = folder.fold_expr(acc, &expr.left);
        let res = folder.fold_binary_op(res, &expr.operator);
        folder.fold_expr(res, &expr.right)
    }

    pub fn fold_binary_op<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _op: &'a BinOp,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Assignment expression.
    //-------------------------------------------------

    pub fn fold_assign_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a AssignExpr,
    ) -> T {
        let res = folder.fold_expr(acc, &expr.left);
        folder.fold_expr(res, &expr.right)
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    pub fn fold_call_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a CallExpr,
    ) -> T {
        let res = folder.fold_expr(acc, &expr.callee);
        folder.fold_call_args(res, &expr.args)
    }

    pub fn fold_call_args<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a CallArgs,
    ) -> T {
        match expr {
            CallArgs::Unnamed(args) => args
                .iter()
                .fold(acc, |acc2, arg| folder.fold_expr(acc2, arg)),
            CallArgs::Named(args) => args
                .iter()
                .fold(acc, |acc2, arg| folder.fold_named_arg(acc2, arg)),
        }
    }

    pub fn fold_named_arg<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a NamedArg,
    ) -> T {
        folder.fold_expr(acc, &expr.value)
    }

    //-------------------------------------------------
    // Function call options expression.
    //-------------------------------------------------

    pub fn fold_call_opts_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a CallOptsExpr,
    ) -> T {
        let res = folder.fold_expr(acc, &expr.callee);
        folder.fold_call_opts(res, &expr.call_opts)
    }

    pub fn fold_call_opts<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        options: &'a [CallOpt],
    ) -> T {
        options
            .iter()
            .fold(acc, |acc2, arg| folder.fold_call_opt(acc2, arg))
    }

    pub fn fold_call_opt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        opt: &'a CallOpt,
    ) -> T {
        folder.fold_expr(acc, &opt.value)
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------

    pub fn fold_tuple_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a TupleExpr,
    ) -> T {
        expr.elems.iter().fold(acc, |acc2, exp| match exp {
            Some(exp) => folder.fold_expr(acc2, exp),
            None => acc2,
        })
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    pub fn fold_index_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a IndexExpr,
    ) -> T {
        let mut res = folder.fold_expr(acc, &expr.base_expr);
        if let Some(index) = &expr.index {
            res = folder.fold_expr(res, index);
        }
        res
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    pub fn fold_slice_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a SliceExpr,
    ) -> T {
        let mut res = folder.fold_expr(acc, &expr.base_expr);
        if let Some(start_index) = &expr.start_index {
            res = folder.fold_expr(res, start_index);
        }
        if let Some(end_index) = &expr.start_index {
            res = folder.fold_expr(res, end_index);
        }
        res
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    pub fn fold_member_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a MemberExpr,
    ) -> T {
        folder.fold_expr(acc, &expr.base)
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    pub fn fold_conditional_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a ConditionalExpr,
    ) -> T {
        let res = folder.fold_expr(acc, &expr.cond);
        let res = folder.fold_expr(res, &expr.true_br);
        folder.fold_expr(res, &expr.false_br)
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    pub fn fold_inline_array_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a InlineArrayExpr,
    ) -> T {
        expr.elems
            .iter()
            .fold(acc, |acc2, e| folder.fold_expr(acc2, e))
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    pub fn fold_new_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a NewExpr,
    ) -> T {
        folder.fold_type(acc, &expr.typ)
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    pub fn fold_type_name_expr<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        expr: &'a TypeNameExpr,
    ) -> T {
        folder.fold_type(acc, &expr.typ)
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn fold_ident<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _id: &'a Identifier,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn fold_lit<'a, T, F: Fold<'a, T> + ?Sized>(_folder: &mut F, acc: T, _lit: &'a Lit) -> T {
        // By default, return the accumulative value
        acc
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    pub fn fold_name<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _name: &'a Name,
    ) -> T {
        // By default, return the accumulative value
        acc
    }

    pub fn fold_name_opt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        name: &'a Option<Name>,
    ) -> T {
        match name {
            Some(n) => folder.fold_name(acc, n),
            None => acc,
        }
    }

    pub fn fold_name_path<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        name_path: &'a NamePath,
    ) -> T {
        name_path
            .names
            .iter()
            .fold(acc, |acc2, n| folder.fold_name(acc2, n))
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn fold_type<'a, T, F: Fold<'a, T> + ?Sized>(folder: &mut F, acc: T, typ: &'a Type) -> T {
        match typ {
            Type::Bool => acc,
            Type::Int(_) => acc,
            Type::Fixed(_) => acc,
            Type::Address(_) => acc,
            Type::Bytes(t) => folder.fold_bytes_type(acc, t),
            Type::String(t) => folder.fold_string_type(acc, t),
            Type::Array(t) => folder.fold_array_type(acc, t),
            Type::Slice(t) => folder.fold_slice_type(acc, t),
            Type::Struct(t) => folder.fold_struct_type(acc, t),
            Type::Enum(t) => folder.fold_enum_type(acc, t),
            Type::Module(_) => acc,
            Type::Tuple(t) => folder.fold_tuple_type(acc, t),
            Type::Func(t) => folder.fold_func_type(acc, t),
            Type::Mapping(t) => folder.fold_mapping_type(acc, t),
            Type::UserDefined(t) => folder.fold_type_name(acc, t),
            Type::Contract(t) => folder.fold_contract_type(acc, t),
            Type::Magic(t) => folder.fold_magic_type(acc, t),
        }
    }

    pub fn fold_bytes_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a BytesType,
    ) -> T {
        folder.fold_data_loc_opt(acc, &typ.data_loc)
    }

    pub fn fold_string_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a StringType,
    ) -> T {
        folder.fold_data_loc_opt(acc, &typ.data_loc)
    }

    pub fn fold_array_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a ArrayType,
    ) -> T {
        let res = folder.fold_type(acc, &typ.base);
        folder.fold_data_loc_opt(res, &typ.data_loc)
    }

    pub fn fold_slice_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a SliceType,
    ) -> T {
        folder.fold_type(acc, &typ.base)
    }

    pub fn fold_struct_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a StructType,
    ) -> T {
        let res = folder.fold_name(acc, &typ.name);
        let res = folder.fold_name_opt(res, &typ.scope);
        folder.fold_data_loc_opt(res, &typ.data_loc)
    }

    pub fn fold_enum_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a EnumType,
    ) -> T {
        let res = folder.fold_name(acc, &typ.name);
        folder.fold_name_opt(res, &typ.scope)
    }

    pub fn fold_tuple_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a TupleType,
    ) -> T {
        typ.elems.iter().fold(acc, |acc2, elem_opt| match elem_opt {
            Some(elem) => folder.fold_type(acc2, elem),
            None => acc2,
        })
    }

    pub fn fold_mapping_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a MappingType,
    ) -> T {
        let res = folder.fold_type(acc, &typ.key);
        let res = folder.fold_type(res, &typ.value);
        folder.fold_data_loc_opt(res, &typ.data_loc)
    }

    pub fn fold_func_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a FunctionType,
    ) -> T {
        let res = typ
            .params
            .iter()
            .fold(acc, |acc2, t| folder.fold_type(acc2, t));
        typ.returns
            .iter()
            .fold(res, |acc2, t| folder.fold_type(acc2, t))
    }

    pub fn fold_type_name<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a UserType,
    ) -> T {
        folder.fold_name(acc, &typ.name)
    }

    pub fn fold_contract_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a ContractType,
    ) -> T {
        folder.fold_name(acc, &typ.name)
    }

    pub fn fold_magic_type<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        typ: &'a MagicType,
    ) -> T {
        match typ {
            MagicType::BlockType => acc,
            MagicType::MessageType => acc,
            MagicType::TxnType => acc,
            MagicType::ABIType => acc,
            MagicType::MetaType(t) => folder.fold_type(acc, t),
        }
    }

    //-------------------------------------------------
    // Data location
    //-------------------------------------------------

    pub fn fold_data_loc_opt<'a, T, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        acc: T,
        dloc: &'a Option<DataLoc>,
    ) -> T {
        match dloc {
            Some(loc) => folder.fold_data_loc(acc, loc),
            None => acc,
        }
    }

    pub fn fold_data_loc<'a, T, F: Fold<'a, T> + ?Sized>(
        _folder: &mut F,
        acc: T,
        _dloc: &'a DataLoc,
    ) -> T {
        acc
    }
}
