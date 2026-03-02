//! Module implementing the visiting design pattern for Solidity AST.
//!
//! This visiting pattern will not transform any input data structure.

use crate::ast::*;
use crate::ast::{DataLoc, Name, NamePath};

//------------------------------------------------------------------
// Trait for implementing the visiting pattern
//------------------------------------------------------------------

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
    // Directives
    //-------------------------------------------------

    fn visit_pragma(&mut self, pragma: &'a PragmaDir) {
        default::visit_pragma(self, pragma)
    }

    fn visit_import(&mut self, import: &'a ImportDir) {
        default::visit_import(self, import)
    }

    //-------------------------------------------------
    // Using directives
    //-------------------------------------------------

    fn visit_using(&mut self, using: &'a UsingDir) {
        default::visit_using(self, using)
    }

    fn visit_using_kind(&mut self, kind: &'a UsingKind) {
        default::visit_using_kind(self, kind)
    }

    fn visit_using_lib(&mut self, ulib: &'a UsingLib) {
        default::visit_using_lib(self, ulib)
    }

    fn visit_using_func(&mut self, ufunc: &'a UsingFunc) {
        default::visit_using_func(self, ufunc)
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

    fn visit_udv_type_def(&mut self, typ: &'a TypeDef) {
        default::visit_udv_type_def(self, typ)
    }

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
    // Function
    //-------------------------------------------------

    fn visit_func_def(&mut self, func: &'a FuncDef) {
        default::visit_func_def(self, func)
    }

    //-------------------------------------------------
    // Block.
    //-------------------------------------------------

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

    fn visit_for_stmt(&mut self, stmt: &'a ForStmt) {
        default::visit_for_stmt(self, stmt)
    }

    fn visit_while_stmt(&mut self, stmt: &'a WhileStmt) {
        default::visit_while_stmt(self, stmt)
    }

    fn visit_do_while_stmt(&mut self, stmt: &'a DoWhileStmt) {
        default::visit_do_while_stmt(self, stmt)
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

    fn visit_place_holder_stmt(&mut self, stmt: &'a PlaceholderStmt) {
        default::visit_place_holder_stmt(self, stmt)
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

    fn visit_unary_expr(&mut self, expr: &'a UnaryExpr) {
        default::visit_unary_expr(self, expr)
    }

    fn visit_binary_expr(&mut self, expr: &'a BinaryExpr) {
        default::visit_binary_expr(self, expr)
    }

    fn visit_assign_expr(&mut self, expr: &'a AssignExpr) {
        default::visit_assign_expr(self, expr)
    }

    fn visit_call_expr(&mut self, expr: &'a CallExpr) {
        default::visit_call_expr(self, expr)
    }

    fn visit_call_opts_expr(&mut self, expr: &'a CallOptsExpr) {
        default::visit_call_opts_expr(self, expr)
    }

    fn visit_call_opt(&mut self, opt: &'a CallOpt) {
        default::visit_call_opt(self, opt)
    }

    fn visit_call_args(&mut self, args: &'a CallArgs) {
        default::visit_call_args(self, args)
    }

    fn visit_named_arg(&mut self, arg: &'a NamedArg) {
        default::visit_named_arg(self, arg)
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

    fn visit_type_name_expr(&mut self, expr: &'a TypeNameExpr) {
        default::visit_type_name_expr(self, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn visit_ident(&mut self, id: &'a Identifier) {
        default::visit_ident(self, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn visit_name(&mut self, name: &'a Name) {
        default::visit_name(self, name)
    }

    fn visit_name_opt(&mut self, name: &'a Option<Name>) {
        default::visit_name_opt(self, name)
    }

    fn visit_name_path(&mut self, name_path: &'a NamePath) {
        default::visit_name_path(self, name_path)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn visit_type(&mut self, typ: &'a Type) {
        default::visit_type(self, typ)
    }

    fn visit_bytes_type(&mut self, typ: &'a BytesType) {
        default::visit_bytes_type(self, typ)
    }

    fn visit_string_type(&mut self, typ: &'a StringType) {
        default::visit_string_type(self, typ)
    }

    fn visit_array_type(&mut self, typ: &'a ArrayType) {
        default::visit_array_type(self, typ)
    }

    fn visit_struct_type(&mut self, typ: &'a StructType) {
        default::visit_struct_type(self, typ)
    }

    fn visit_enum_type(&mut self, typ: &'a EnumType) {
        default::visit_enum_type(self, typ)
    }

    fn visit_slice_type(&mut self, typ: &'a SliceType) {
        default::visit_slice_type(self, typ)
    }

    fn visit_tuple_type(&mut self, typ: &'a TupleType) {
        default::visit_tuple_type(self, typ)
    }

    fn visit_func_type(&mut self, typ: &'a FuncType) {
        default::visit_func_type(self, typ)
    }

    fn visit_mapping_type(&mut self, typ: &'a MappingType) {
        default::visit_mapping_type(self, typ)
    }

    fn visit_contract_type(&mut self, typ: &'a ContractType) {
        default::visit_contract_type(self, typ)
    }

    fn visit_type_name(&mut self, typ: &'a UserDefinedType) {
        default::visit_type_name(self, typ)
    }

    fn visit_magic_type(&mut self, typ: &'a MagicType) {
        default::visit_magic_type(self, typ)
    }

    //-------------------------------------------------
    // Data location.
    //-------------------------------------------------

    fn visit_data_loc(&mut self, dloc: &'a DataLoc) {
        default::visit_data_loc(self, dloc)
    }
}

//------------------------------------------------------------------
// Default visiting pattern
//------------------------------------------------------------------

pub mod default {
    use super::Visit;
    use crate::ast::*;
    use crate::ast::{DataLoc, Name, NamePath};

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

    pub fn visit_source_unit<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        source_unit: &'a SourceUnit,
    ) {
        source_unit
            .elems
            .iter()
            .for_each(|elem| visitor.visit_source_unit_elem(elem))
    }

    pub fn visit_source_unit_elem<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        elem: &'a SourceUnitElem,
    ) {
        match elem {
            SourceUnitElem::Pragma(pragma) => visitor.visit_pragma(pragma),
            SourceUnitElem::Import(import) => visitor.visit_import(import),
            SourceUnitElem::Using(using) => visitor.visit_using(using),
            SourceUnitElem::Error(error) => visitor.visit_error_def(error),
            SourceUnitElem::Var(var) => visitor.visit_var_decl(var),
            SourceUnitElem::UserType(typ) => visitor.visit_udv_type_def(typ),
            SourceUnitElem::Struct(struct_) => visitor.visit_struct_def(struct_),
            SourceUnitElem::Func(func) => visitor.visit_func_def(func),
            SourceUnitElem::Enum(enum_) => visitor.visit_enum_def(enum_),
            SourceUnitElem::Contract(contract) => visitor.visit_contract_def(contract),
        }
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    pub fn visit_pragma<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _pragma: &'a PragmaDir) {
        // Do nothing by default
    }

    pub fn visit_import<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _import: &'a ImportDir) {
        // Do nothing by default
    }

    //-------------------------------------------------
    // Using directives
    //-------------------------------------------------

    pub fn visit_using<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, using: &'a UsingDir) {
        visitor.visit_using_kind(&using.kind);
        if let Some(t) = &using.target_type {
            visitor.visit_type(t)
        }
    }

    pub fn visit_using_kind<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, kind: &'a UsingKind) {
        match kind {
            UsingKind::UsingLib(ulib) => visitor.visit_using_lib(ulib),
            UsingKind::UsingFunc(ufuncs) => ufuncs
                .iter()
                .for_each(|ufunc| visitor.visit_using_func(ufunc)),
        }
    }

    pub fn visit_using_lib<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, ulib: &'a UsingLib) {
        visitor.visit_name(&ulib.lib_name);
        visitor.visit_name_path(&ulib.lib_path)
    }

    pub fn visit_using_func<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, ufunc: &'a UsingFunc) {
        visitor.visit_name(&ufunc.func_name);
        visitor.visit_name_path(&ufunc.func_path)
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

    pub fn visit_udv_type_def<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a TypeDef) {
        visitor.visit_name(&typ.name);
        visitor.visit_type(&typ.base_typ)
    }

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
            .body
            .iter()
            .for_each(|elem| visitor.visit_contract_elem(elem))
    }

    pub fn visit_contract_elem<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        contract: &'a ContractElem,
    ) {
        match contract {
            ContractElem::Using(using) => visitor.visit_using(using),
            ContractElem::Event(event) => visitor.visit_event_def(event),
            ContractElem::Error(error) => visitor.visit_error_def(error),
            ContractElem::Struct(struct_) => visitor.visit_struct_def(struct_),
            ContractElem::Enum(enum_) => visitor.visit_enum_def(enum_),
            ContractElem::Type(typ) => visitor.visit_udv_type_def(typ),
            ContractElem::Var(var) => visitor.visit_var_decl(var),
            ContractElem::Func(func) => visitor.visit_func_def(func),
        }
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    pub fn visit_func_def<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, func: &'a FuncDef) {
        func.params.iter().for_each(|p| visitor.visit_var_decl(p));
        func.modifier_invocs
            .iter()
            .for_each(|m| visitor.visit_call_expr(m));
        func.returns.iter().for_each(|p| visitor.visit_var_decl(p));
        func.body.iter().for_each(|blk| visitor.visit_block(blk))
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn visit_block<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, block: &'a Block) {
        block.body.iter().for_each(|stmt| visitor.visit_stmt(stmt))
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn visit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a Stmt) {
        match stmt {
            Stmt::Asm(s) => visitor.visit_asm_stmt(s),
            Stmt::Block(b) => visitor.visit_block(b),
            Stmt::Break(s) => visitor.visit_break_stmt(s),
            Stmt::Continue(s) => visitor.visit_continue_stmt(s),
            Stmt::While(s) => visitor.visit_while_stmt(s),
            Stmt::DoWhile(s) => visitor.visit_do_while_stmt(s),
            Stmt::Emit(s) => visitor.visit_emit_stmt(s),
            Stmt::Expr(s) => visitor.visit_expr_stmt(s),
            Stmt::For(s) => visitor.visit_for_stmt(s),
            Stmt::If(s) => visitor.visit_if_stmt(s),
            Stmt::Return(s) => visitor.visit_return_stmt(s),
            Stmt::Revert(s) => visitor.visit_revert_stmt(s),
            Stmt::Throw(s) => visitor.visit_throw_stmt(s),
            Stmt::Try(s) => visitor.visit_try_stmt(s),
            Stmt::Placeholder(s) => visitor.visit_place_holder_stmt(s),
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
        visitor.visit_stmt(&stmt.true_branch);
        stmt.false_branch
            .iter()
            .for_each(|blk| visitor.visit_stmt(blk))
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    pub fn visit_for_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a ForStmt) {
        stmt.pre_loop.iter().for_each(|e| visitor.visit_stmt(e));
        stmt.post_loop.iter().for_each(|e| visitor.visit_stmt(e));
        stmt.condition.iter().for_each(|e| visitor.visit_expr(e));
        visitor.visit_stmt(&stmt.body)
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    pub fn visit_while_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a WhileStmt) {
        visitor.visit_expr(&stmt.condition);
        visitor.visit_stmt(&stmt.body)
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    pub fn visit_do_while_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a DoWhileStmt) {
        visitor.visit_expr(&stmt.condition);
        visitor.visit_stmt(&stmt.body)
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
        if let Some(expr) = &stmt.error {
            visitor.visit_expr(expr)
        }
        visitor.visit_call_args(&stmt.args)
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn visit_emit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a EmitStmt) {
        visitor.visit_call_args(&stmt.args)
    }

    //-------------------------------------------------
    // Place holder statement
    //-------------------------------------------------

    pub fn visit_place_holder_stmt<'a, T: Visit<'a> + ?Sized>(
        _visitor: &mut T,
        _stmt: &'a PlaceholderStmt,
    ) {
        // Do nothing by default
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
        stmt.var_decls.iter().for_each(|v| {
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
            Expr::Ident(v) => visitor.visit_ident(v),
            Expr::Unary(e) => visitor.visit_unary_expr(e),
            Expr::Binary(e) => visitor.visit_binary_expr(e),
            Expr::Assign(e) => visitor.visit_assign_expr(e),
            Expr::Call(e) => visitor.visit_call_expr(e),
            Expr::CallOpts(e) => visitor.visit_call_opts_expr(e),
            Expr::Tuple(e) => visitor.visit_tuple_expr(e),
            Expr::Index(e) => visitor.visit_index_expr(e),
            Expr::Slice(e) => visitor.visit_slice_expr(e),
            Expr::Member(e) => visitor.visit_member_expr(e),
            Expr::Conditional(e) => visitor.visit_conditional_expr(e),
            Expr::InlineArray(e) => visitor.visit_inline_array_expr(e),
            Expr::New(e) => visitor.visit_new_expr(e),
            Expr::TypeName(e) => visitor.visit_type_name_expr(e),
        }
    }

    //-------------------------------------------------
    // Unary expression.
    //-------------------------------------------------

    pub fn visit_unary_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a UnaryExpr) {
        visitor.visit_expr(&expr.body)
    }

    //-------------------------------------------------
    // Binary expression.
    //-------------------------------------------------

    pub fn visit_binary_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a BinaryExpr) {
        visitor.visit_expr(&expr.left);
        visitor.visit_expr(&expr.right)
    }

    //-------------------------------------------------
    // Assignment expression.
    //-------------------------------------------------

    pub fn visit_assign_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a AssignExpr) {
        visitor.visit_expr(&expr.left);
        visitor.visit_expr(&expr.right)
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    pub fn visit_call_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CallExpr) {
        visitor.visit_expr(&expr.callee);
        visitor.visit_call_args(&expr.args)
    }

    pub fn visit_call_args<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CallArgs) {
        match expr {
            CallArgs::Unnamed(args) => args.iter().for_each(|arg| visitor.visit_expr(arg)),
            CallArgs::Named(args) => args.iter().for_each(|arg| visitor.visit_named_arg(arg)),
        }
    }

    pub fn visit_named_arg<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a NamedArg) {
        visitor.visit_expr(&expr.value)
    }

    //-------------------------------------------------
    // Function call options expression.
    //-------------------------------------------------

    pub fn visit_call_opts_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CallOptsExpr,
    ) {
        visitor.visit_expr(&expr.callee);
        expr.call_opts
            .iter()
            .for_each(|opt| visitor.visit_call_opt(opt))
    }

    pub fn visit_call_opt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, opt: &'a CallOpt) {
        visitor.visit_expr(&opt.value);
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------
    pub fn visit_tuple_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a TupleExpr) {
        expr.elems.iter().for_each(|exp| match exp {
            Some(exp) => visitor.visit_expr(exp),
            None => (),
        })
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    pub fn visit_index_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a IndexExpr) {
        visitor.visit_expr(&expr.base_expr);
        if let Some(index) = &expr.index {
            visitor.visit_expr(index)
        }
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    pub fn visit_slice_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a SliceExpr) {
        visitor.visit_expr(&expr.base_expr);
        if let Some(start_index) = &expr.start_index {
            visitor.visit_expr(start_index);
        }
        if let Some(end_index) = &expr.end_index {
            visitor.visit_expr(end_index);
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
        visitor.visit_expr(&expr.cond);
        visitor.visit_expr(&expr.true_br);
        visitor.visit_expr(&expr.false_br)
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    pub fn visit_inline_array_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a InlineArrayExpr,
    ) {
        for e in &expr.elems {
            visitor.visit_expr(e);
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

    pub fn visit_type_name_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a TypeNameExpr,
    ) {
        visitor.visit_type(&expr.typ);
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    pub fn visit_ident<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, id: &'a Identifier) {
        visitor.visit_name(&id.name)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    pub fn visit_name<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _name: &'a Name) {
        // Do nothing by default
    }

    pub fn visit_name_opt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, name: &'a Option<Name>) {
        if let Some(n) = name {
            visitor.visit_name(n)
        }
    }

    pub fn visit_name_path<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, name_path: &'a NamePath) {
        name_path.names.iter().for_each(|n| visitor.visit_name(n))
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn visit_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a Type) {
        match typ {
            Type::Bool => (),
            Type::Int(_) => (),
            Type::Fixed(_) => (),
            Type::Address(_) => (),
            Type::Bytes(t) => visitor.visit_bytes_type(t),
            Type::String(t) => visitor.visit_string_type(t),
            Type::Array(t) => visitor.visit_array_type(t),
            Type::Slice(t) => visitor.visit_slice_type(t),
            Type::Struct(t) => visitor.visit_struct_type(t),
            Type::Enum(t) => visitor.visit_enum_type(t),
            Type::Module(_) => (),
            Type::Tuple(t) => visitor.visit_tuple_type(t),
            Type::Func(t) => visitor.visit_func_type(t),
            Type::Mapping(t) => visitor.visit_mapping_type(t),
            Type::UserDefined(t) => visitor.visit_type_name(t),
            Type::Contract(t) => visitor.visit_contract_type(t),
            Type::Magic(t) => visitor.visit_magic_type(t),
        }
    }

    pub fn visit_bytes_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a BytesType) {
        visitor.visit_data_loc(&typ.data_loc)
    }

    pub fn visit_string_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a StringType) {
        visitor.visit_data_loc(&typ.data_loc)
    }

    pub fn visit_array_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a ArrayType) {
        visitor.visit_data_loc(&typ.data_loc);
        visitor.visit_type(&typ.base)
    }

    pub fn visit_slice_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a SliceType) {
        visitor.visit_type(&typ.base)
    }

    pub fn visit_struct_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a StructType) {
        visitor.visit_name(&typ.name);
        visitor.visit_name_opt(&typ.scope);
        visitor.visit_data_loc(&typ.data_loc)
    }

    pub fn visit_enum_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a EnumType) {
        visitor.visit_name(&typ.name);
        visitor.visit_name_opt(&typ.scope)
    }

    pub fn visit_tuple_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a TupleType) {
        typ.elems.iter().for_each(|elem_typ| {
            if let Some(t) = elem_typ {
                visitor.visit_type(t)
            }
        })
    }

    pub fn visit_func_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a FuncType) {
        typ.params.iter().for_each(|t| visitor.visit_type(t));
        typ.returns.iter().for_each(|t| visitor.visit_type(t))
    }

    pub fn visit_mapping_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a MappingType) {
        visitor.visit_data_loc(&typ.data_loc);
        visitor.visit_type(&typ.key);
        visitor.visit_type(&typ.value)
    }

    pub fn visit_type_name<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a UserDefinedType) {
        visitor.visit_name(&typ.name)
    }

    pub fn visit_contract_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a ContractType) {
        visitor.visit_name(&typ.name);
    }

    pub fn visit_magic_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, typ: &'a MagicType) {
        match typ {
            MagicType::BlockType => (),
            MagicType::MessageType => (),
            MagicType::TxnType => (),
            MagicType::ABIType => (),
            MagicType::MetaType(t) => visitor.visit_type(t),
        }
    }

    //-------------------------------------------------
    // Data location
    //-------------------------------------------------

    pub fn visit_data_loc<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _dloc: &'a DataLoc) {
        // Do nothing by default
    }
}
