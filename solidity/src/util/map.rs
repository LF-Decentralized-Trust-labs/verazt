//! Module implementing the map design pattern for Solidity AST.
//!
//! This map pattern will transform an input data structure into a new data
//! structure of the same type.

use crate::ast::*;
use meta::DataLoc;

//------------------------------------------------------------------
// Trait for implementing the mapping utilities
//------------------------------------------------------------------

/// Trait implementing the mapping utilities for Solidity AST.
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
    // Directives
    //-------------------------------------------------

    fn map_pragma(&mut self, pragma: &'a PragmaDir) -> PragmaDir {
        default::map_pragma(self, pragma)
    }

    fn map_import(&mut self, import: &'a ImportDir) -> ImportDir {
        default::map_import(self, import)
    }

    //-------------------------------------------------
    // Using directives
    //-------------------------------------------------

    fn map_using(&mut self, using: &'a UsingDir) -> UsingDir {
        default::map_using(self, using)
    }

    fn map_using_kind(&mut self, kind: &'a UsingKind) -> UsingKind {
        default::map_using_kind(self, kind)
    }

    fn map_using_lib(&mut self, ulib: &'a UsingLib) -> UsingLib {
        default::map_using_lib(self, ulib)
    }

    fn map_using_func(&mut self, ufund: &'a UsingFunc) -> UsingFunc {
        default::map_using_func(self, ufund)
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

    fn map_udv_type_def(&mut self, typ: &'a TypeDef) -> TypeDef {
        default::map_udv_type_def(self, typ)
    }

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

    fn map_base_contract(&mut self, base: &'a BaseContract) -> BaseContract {
        default::map_base_contract(self, base)
    }

    fn map_contract_elem(&mut self, elem: &'a ContractElem) -> ContractElem {
        default::map_contract_element(self, elem)
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    fn map_func_def(&mut self, func: &'a FuncDef) -> FuncDef {
        default::map_func_def(self, func)
    }

    //-------------------------------------------------
    // Block.
    //-------------------------------------------------

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

    fn map_for_stmt(&mut self, stmt: &'a ForStmt) -> ForStmt {
        default::map_for_stmt(self, stmt)
    }

    fn map_while_stmt(&mut self, stmt: &'a WhileStmt) -> WhileStmt {
        default::map_while_stmt(self, stmt)
    }

    fn map_do_while_stmt(&mut self, stmt: &'a DoWhileStmt) -> DoWhileStmt {
        default::map_do_while_stmt(self, stmt)
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

    fn map_place_holder_stmt(&mut self, stmt: &'a PlaceholderStmt) -> PlaceholderStmt {
        default::map_place_holder_stmt(self, stmt)
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

    fn map_var_decl(&mut self, vdecl: &'a VarDecl) -> VarDecl {
        default::map_var_decl(self, vdecl)
    }

    fn map_var_decls(&mut self, vdecls: &'a [VarDecl]) -> Vec<VarDecl> {
        default::map_var_decls(self, vdecls)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn map_expr(&mut self, expr: &'a Expr) -> Expr {
        default::map_expr(self, expr)
    }

    fn map_unary_expr(&mut self, expr: &'a UnaryExpr) -> UnaryExpr {
        default::map_unary_expr(self, expr)
    }

    fn map_binary_expr(&mut self, expr: &'a BinaryExpr) -> BinaryExpr {
        default::map_binary_expr(self, expr)
    }

    fn map_assign_expr(&mut self, expr: &'a AssignExpr) -> AssignExpr {
        default::map_assign_expr(self, expr)
    }

    fn map_call_expr(&mut self, expr: &'a CallExpr) -> CallExpr {
        default::map_call_expr(self, expr)
    }

    fn map_call_args(&mut self, expr: &'a CallArgs) -> CallArgs {
        default::map_call_args(self, expr)
    }

    fn map_named_arg(&mut self, expr: &'a NamedArg) -> NamedArg {
        default::map_named_arg(self, expr)
    }

    fn map_call_opts_expr(&mut self, expr: &'a CallOptsExpr) -> CallOptsExpr {
        default::map_call_opts_expr(self, expr)
    }

    fn map_call_opt(&mut self, call_opt: &'a CallOpt) -> CallOpt {
        default::map_call_opt(self, call_opt)
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

    fn map_type_name_expr(&mut self, expr: &'a TypeNameExpr) -> TypeNameExpr {
        default::map_type_name_expr(self, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn map_ident(&mut self, id: &'a Identifier) -> Identifier {
        default::map_ident(self, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn map_name(&mut self, name: &'a Name) -> Name {
        default::map_name(self, name)
    }

    fn map_name_opt(&mut self, name: &'a Option<Name>) -> Option<Name> {
        default::map_name_opt(self, name)
    }

    fn map_names(&mut self, names: &'a [Name]) -> Vec<Name> {
        default::map_names(self, names)
    }

    fn map_name_path(&mut self, name_path: &'a NamePath) -> NamePath {
        default::map_path(self, name_path)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn map_type(&mut self, typ: &'a Type) -> Type {
        default::map_type(self, typ)
    }

    fn map_bytes_type(&mut self, typ: &'a BytesType) -> BytesType {
        default::map_bytes_type(self, typ)
    }

    fn map_string_type(&mut self, typ: &'a StringType) -> StringType {
        default::map_string_type(self, typ)
    }

    fn map_array_type(&mut self, typ: &'a ArrayType) -> ArrayType {
        default::map_array_type(self, typ)
    }

    fn map_struct_type(&mut self, typ: &'a StructType) -> StructType {
        default::map_struct_type(self, typ)
    }

    fn map_enum_type(&mut self, typ: &'a EnumType) -> EnumType {
        default::map_enum_type(self, typ)
    }

    fn map_slice_type(&mut self, typ: &'a SliceType) -> SliceType {
        default::map_slice_type(self, typ)
    }

    fn map_tuple_type(&mut self, typ: &'a TupleType) -> TupleType {
        default::map_tuple_type(self, typ)
    }

    fn map_func_type(&mut self, typ: &'a FuncType) -> FuncType {
        default::map_function_type(self, typ)
    }

    fn map_mapping_type(&mut self, typ: &'a MappingType) -> MappingType {
        default::map_mapping_type(self, typ)
    }

    fn map_contract_type(&mut self, typ: &'a ContractType) -> ContractType {
        default::map_contract_type(self, typ)
    }

    fn map_type_name(&mut self, typ: &'a UserDefinedType) -> UserDefinedType {
        default::map_type_name(self, typ)
    }

    fn map_magic_type(&mut self, typ: &'a MagicType) -> MagicType {
        default::map_magic_type(self, typ)
    }

    //-------------------------------------------------
    // Overriding
    //-------------------------------------------------

    fn map_overriding(&mut self, overriding: &'a Overriding) -> Overriding {
        default::map_overriding(self, overriding)
    }

    //-------------------------------------------------
    // Data location.
    //-------------------------------------------------

    fn map_data_loc(&mut self, dloc: &'a DataLoc) -> DataLoc {
        default::map_data_loc(self, dloc)
    }
}

//------------------------------------------------------------------
// Default mapping pattern
//------------------------------------------------------------------

pub mod default {
    use super::Map;
    use crate::ast::*;
    use meta::DataLoc;

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
        let elems = source_unit
            .elems
            .iter()
            .map(|elem| mapper.map_source_unit_elem(elem))
            .collect();
        SourceUnit { elems, ..source_unit.clone() }
    }

    pub fn map_source_unit_elem<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        elem: &'a SourceUnitElem,
    ) -> SourceUnitElem {
        match elem {
            SourceUnitElem::Pragma(p) => mapper.map_pragma(p).into(),
            SourceUnitElem::Import(i) => mapper.map_import(i).into(),
            SourceUnitElem::Using(u) => mapper.map_using(u).into(),
            SourceUnitElem::Error(e) => mapper.map_error_def(e).into(),
            SourceUnitElem::Var(v) => mapper.map_var_decl(v).into(),
            SourceUnitElem::UserType(t) => mapper.map_udv_type_def(t).into(),
            SourceUnitElem::Struct(s) => mapper.map_struct_def(s).into(),
            SourceUnitElem::Enum(e) => mapper.map_enum_def(e).into(),
            SourceUnitElem::Func(f) => mapper.map_func_def(f).into(),
            SourceUnitElem::Contract(c) => mapper.map_contract_def(c).into(),
        }
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    pub fn map_pragma<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        pragma: &'a PragmaDir,
    ) -> PragmaDir {
        pragma.clone()
    }

    pub fn map_import<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        import: &'a ImportDir,
    ) -> ImportDir {
        import.clone()
    }

    //-------------------------------------------------
    // Using directives
    //-------------------------------------------------

    pub fn map_using<'a, T: Map<'a> + ?Sized>(mapper: &mut T, using: &'a UsingDir) -> UsingDir {
        let nkind = mapper.map_using_kind(&using.kind);
        let ntyp = using.target_type.as_ref().map(|t| mapper.map_type(t));
        UsingDir { kind: nkind, target_type: ntyp, ..using.clone() }
    }

    pub fn map_using_kind<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        kind: &'a UsingKind,
    ) -> UsingKind {
        match kind {
            UsingKind::UsingLib(ulib) => {
                let nulib = mapper.map_using_lib(ulib);
                UsingKind::UsingLib(nulib)
            }
            UsingKind::UsingFunc(usings) => {
                let nufuncs = usings
                    .iter()
                    .map(|ufunc| mapper.map_using_func(ufunc))
                    .collect();
                UsingKind::UsingFunc(nufuncs)
            }
        }
    }

    pub fn map_using_lib<'a, T: Map<'a> + ?Sized>(mapper: &mut T, ulib: &'a UsingLib) -> UsingLib {
        let nname = mapper.map_name(&ulib.lib_name);
        let nscope = mapper.map_name_path(&ulib.lib_path);
        UsingLib { lib_name: nname, lib_path: nscope, ..ulib.clone() }
    }

    pub fn map_using_func<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        ufunc: &'a UsingFunc,
    ) -> UsingFunc {
        let nname = mapper.map_name(&ufunc.func_name);
        let nscope = mapper.map_name_path(&ufunc.func_path);
        UsingFunc { func_name: nname, func_path: nscope, ..ufunc.clone() }
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    pub fn map_error_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        error: &'a ErrorDef,
    ) -> ErrorDef {
        let nname = mapper.map_name(&error.name);
        let nparams = mapper.map_var_decls(&error.params);
        ErrorDef { name: nname, params: nparams, ..error.clone() }
    }

    pub fn map_event_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        event: &'a EventDef,
    ) -> EventDef {
        let nname = mapper.map_name(&event.name);
        let nparams = mapper.map_var_decls(&event.params);
        EventDef { name: nname, params: nparams, ..event.clone() }
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn map_udv_type_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a TypeDef,
    ) -> TypeDef {
        let nname = mapper.map_name(&typ.name);
        let nbase = mapper.map_type(&typ.base_typ);
        TypeDef { name: nname, base_typ: nbase, ..typ.clone() }
    }

    pub fn map_struct_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        struct_: &'a StructDef,
    ) -> StructDef {
        let nname = mapper.map_name(&struct_.name);
        let nfields = struct_
            .fields
            .iter()
            .map(|fld| mapper.map_struct_field(fld))
            .collect();
        StructDef { name: nname, fields: nfields, ..struct_.clone() }
    }

    pub fn map_struct_field<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        field: &'a StructField,
    ) -> StructField {
        let ntyp = mapper.map_type(&field.typ);
        StructField { typ: ntyp, ..field.clone() }
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
        let nname = mapper.map_name(&contract.name);
        let nbases: Vec<BaseContract> = contract
            .base_contracts
            .iter()
            .map(|base| mapper.map_base_contract(base))
            .collect();
        let nbody = contract
            .body
            .iter()
            .map(|elem| mapper.map_contract_elem(elem))
            .collect();
        ContractDef { name: nname, base_contracts: nbases, body: nbody, ..contract.clone() }
    }

    pub fn map_base_contract<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        base: &'a BaseContract,
    ) -> BaseContract {
        let nname = mapper.map_name(&base.name);
        let nargs = base.args.iter().map(|arg| mapper.map_expr(arg)).collect();
        BaseContract { name: nname, args: nargs, ..base.clone() }
    }

    pub fn map_contract_element<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        elem: &'a ContractElem,
    ) -> ContractElem {
        match elem {
            ContractElem::Using(p) => mapper.map_using(p).into(),
            ContractElem::Event(e) => mapper.map_event_def(e).into(),
            ContractElem::Error(e) => mapper.map_error_def(e).into(),
            ContractElem::Struct(s) => mapper.map_struct_def(s).into(),
            ContractElem::Enum(e) => mapper.map_enum_def(e).into(),
            ContractElem::Type(t) => mapper.map_udv_type_def(t).into(),
            ContractElem::Var(v) => mapper.map_var_decl(v).into(),
            ContractElem::Func(f) => mapper.map_func_def(f).into(),
        }
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    pub fn map_func_def<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        func: &'a FuncDef,
    ) -> FuncDef {
        let name = mapper.map_name(&func.name);
        let params = mapper.map_var_decls(&func.params);
        let modifier_invocs = func
            .modifier_invocs
            .iter()
            .map(|m| mapper.map_call_expr(m))
            .collect();
        let returns = mapper.map_var_decls(&func.returns);
        let overriding = mapper.map_overriding(&func.overriding);
        let body = func.body.as_ref().map(|blk| mapper.map_block(blk));
        FuncDef { name, body, params, modifier_invocs, returns, overriding, ..func.clone() }
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn map_block<'a, T: Map<'a> + ?Sized>(mapper: &mut T, block: &'a Block) -> Block {
        let nstmts = block
            .body
            .iter()
            .map(|stmt| mapper.map_stmt(stmt))
            .collect();
        Block { body: nstmts, ..block.clone() }
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn map_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a Stmt) -> Stmt {
        match stmt {
            Stmt::Asm(s) => mapper.map_asm_stmt(s).into(),
            Stmt::Block(b) => mapper.map_block(b).into(),
            Stmt::Break(s) => mapper.map_break_stmt(s).into(),
            Stmt::Continue(s) => mapper.map_continue_stmt(s).into(),
            Stmt::DoWhile(s) => mapper.map_do_while_stmt(s).into(),
            Stmt::Emit(s) => mapper.map_emit_stmt(s).into(),
            Stmt::Expr(s) => mapper.map_expr_stmt(s).into(),
            Stmt::For(s) => mapper.map_for_stmt(s).into(),
            Stmt::If(s) => mapper.map_if_stmt(s).into(),
            Stmt::Placeholder(s) => mapper.map_place_holder_stmt(s).into(),
            Stmt::Return(s) => mapper.map_return_stmt(s).into(),
            Stmt::Revert(s) => mapper.map_revert_stmt(s).into(),
            Stmt::Throw(s) => mapper.map_throw_stmt(s).into(),
            Stmt::Try(s) => mapper.map_try_stmt(s).into(),
            Stmt::VarDecl(s) => mapper.map_var_decl_stmt(s).into(),
            Stmt::While(s) => mapper.map_while_stmt(s).into(),
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
        let nbody = mapper.map_expr(&stmt.expr);
        ExprStmt { expr: nbody, ..stmt.clone() }
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn map_if_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a IfStmt) -> IfStmt {
        let ncond = mapper.map_expr(&stmt.condition);
        let ntrue_br = mapper.map_stmt(&stmt.true_branch);
        let nfalse_br = stmt.false_branch.as_ref().map(|stmt| mapper.map_stmt(stmt));
        IfStmt {
            condition: ncond,
            true_branch: Box::new(ntrue_br),
            false_branch: nfalse_br.map(Box::new),
            ..stmt.clone()
        }
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    pub fn map_for_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a ForStmt) -> ForStmt {
        let npre = stmt.pre_loop.as_ref().map(|e| mapper.map_stmt(e));
        let npost = stmt.post_loop.as_ref().map(|e| mapper.map_stmt(e));
        let ncond = stmt.condition.as_ref().map(|e| mapper.map_expr(e));
        let nbody = mapper.map_stmt(&stmt.body);
        ForStmt {
            pre_loop: npre.map(Box::new),
            post_loop: npost.map(Box::new),
            condition: ncond,
            body: Box::new(nbody),
            ..stmt.clone()
        }
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    pub fn map_while_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a WhileStmt,
    ) -> WhileStmt {
        let ncond = mapper.map_expr(&stmt.condition);
        let nbody = mapper.map_stmt(&stmt.body);
        WhileStmt { condition: ncond, body: Box::new(nbody), ..stmt.clone() }
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    pub fn map_do_while_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a DoWhileStmt,
    ) -> DoWhileStmt {
        let ncond = mapper.map_expr(&stmt.condition);
        let nbody = mapper.map_stmt(&stmt.body);
        DoWhileStmt { condition: ncond, body: Box::new(nbody), ..stmt.clone() }
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
        let nreturns = mapper.map_var_decls(&stmt.returns);
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
        let nparams = mapper.map_var_decls(&cls.params);
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
        let nerror = stmt.error.as_ref().map(|expr| mapper.map_expr(expr));
        let nargs = mapper.map_call_args(&stmt.args);
        RevertStmt { error: nerror, args: nargs, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn map_emit_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a EmitStmt) -> EmitStmt {
        let arguments = mapper.map_call_args(&stmt.args);
        EmitStmt { args: arguments, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn map_return_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a ReturnStmt,
    ) -> ReturnStmt {
        let nexpr = stmt.expr.as_ref().map(|expr| mapper.map_expr(expr));
        ReturnStmt { expr: nexpr, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Place holder statement
    //-------------------------------------------------

    pub fn map_place_holder_stmt<'a, T: Map<'a> + ?Sized>(
        _mapper: &mut T,
        stmt: &'a PlaceholderStmt,
    ) -> PlaceholderStmt {
        stmt.clone()
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn map_var_decl_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a VarDeclStmt,
    ) -> VarDeclStmt {
        // Map the assigned value first because it'll affect variable declarations.
        let value = stmt.value.as_ref().map(|e| mapper.map_expr(e));
        let var_decls = stmt
            .var_decls
            .iter()
            .map(|v| v.as_ref().map(|u| mapper.map_var_decl(u)))
            .collect();
        VarDeclStmt { var_decls, value, ..stmt.clone() }
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    pub fn map_var_decl<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        vdecl: &'a VarDecl,
    ) -> VarDecl {
        let name = mapper.map_name(&vdecl.name);
        let typ = mapper.map_type(&vdecl.typ);
        let value = vdecl.value.as_ref().map(|e| mapper.map_expr(e));
        let overriding = match &vdecl.overriding {
            Overriding::None | Overriding::All => vdecl.overriding.clone(),
            Overriding::Some(names) => {
                let mapped_names = mapper.map_names(&names);
                Overriding::Some(mapped_names)
            }
        };
        VarDecl { name, typ, value, overriding, ..vdecl.clone() }
    }

    pub fn map_var_decls<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        vdecls: &'a [VarDecl],
    ) -> Vec<VarDecl> {
        vdecls
            .iter()
            .map(|p| mapper.map_var_decl(p))
            .collect::<Vec<VarDecl>>()
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn map_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a Expr) -> Expr {
        match expr {
            Expr::Ident(v) => mapper.map_ident(v).into(),
            Expr::Lit(_) => expr.clone(),
            Expr::Unary(e) => mapper.map_unary_expr(e).into(),
            Expr::Binary(e) => mapper.map_binary_expr(e).into(),
            Expr::Assign(e) => mapper.map_assign_expr(e).into(),
            Expr::Call(e) => mapper.map_call_expr(e).into(),
            Expr::CallOpts(e) => mapper.map_call_opts_expr(e).into(),
            Expr::Tuple(e) => mapper.map_tuple_expr(e).into(),
            Expr::Index(e) => mapper.map_index_expr(e).into(),
            Expr::Slice(e) => mapper.map_slice_expr(e).into(),
            Expr::Member(e) => mapper.map_member_expr(e).into(),
            Expr::Conditional(e) => mapper.map_conditional_expr(e).into(),
            Expr::InlineArray(e) => mapper.map_inline_array_expr(e).into(),
            Expr::New(e) => mapper.map_new_expr(e).into(),
            Expr::TypeName(e) => mapper.map_type_name_expr(e).into(),
        }
    }

    //-------------------------------------------------
    // Unary expression.
    //-------------------------------------------------

    pub fn map_unary_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a UnaryExpr,
    ) -> UnaryExpr {
        let noperand = mapper.map_expr(&expr.body);
        let ntyp = mapper.map_type(&expr.typ);
        UnaryExpr { body: Box::new(noperand), typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Binary expression.
    //-------------------------------------------------

    pub fn map_binary_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a BinaryExpr,
    ) -> BinaryExpr {
        let nlhs = mapper.map_expr(&expr.left);
        let nrhs = mapper.map_expr(&expr.right);
        let ntyp = mapper.map_type(&expr.typ);
        BinaryExpr { left: Box::new(nlhs), right: Box::new(nrhs), typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Assignment expression.
    //-------------------------------------------------

    pub fn map_assign_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a AssignExpr,
    ) -> AssignExpr {
        let nlhs = mapper.map_expr(&expr.left);
        let nrhs = mapper.map_expr(&expr.right);
        let ntyp = mapper.map_type(&expr.typ);
        AssignExpr { left: Box::new(nlhs), right: Box::new(nrhs), typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    pub fn map_call_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a CallExpr) -> CallExpr {
        let ncallee = mapper.map_expr(&expr.callee);
        let nargs = mapper.map_call_args(&expr.args);
        let ntyp = mapper.map_type(&expr.typ);
        CallExpr { callee: Box::new(ncallee), args: nargs, typ: ntyp, ..expr.clone() }
    }

    pub fn map_call_args<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a CallArgs) -> CallArgs {
        match expr {
            CallArgs::Unnamed(args) => {
                let nargs: Vec<Expr> = args.iter().map(|arg| mapper.map_expr(arg)).collect();
                CallArgs::Unnamed(nargs)
            }
            CallArgs::Named(args) => {
                let nargs = args.iter().map(|arg| mapper.map_named_arg(arg)).collect();
                CallArgs::Named(nargs)
            }
        }
    }

    pub fn map_named_arg<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a NamedArg) -> NamedArg {
        let nvalue = mapper.map_expr(&expr.value);
        NamedArg { value: nvalue, ..expr.clone() }
    }

    //-------------------------------------------------
    // Function call options expression.
    //-------------------------------------------------

    pub fn map_call_opts_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a CallOptsExpr,
    ) -> CallOptsExpr {
        let ncallee = mapper.map_expr(&expr.callee);
        let nopts = expr
            .call_opts
            .iter()
            .map(|opt| mapper.map_call_opt(opt))
            .collect();
        let ntyp = mapper.map_type(&expr.typ);
        CallOptsExpr { callee: Box::new(ncallee), call_opts: nopts, typ: ntyp, ..expr.clone() }
    }

    pub fn map_call_opt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, opt: &'a CallOpt) -> CallOpt {
        let nvalue = mapper.map_expr(&opt.value);
        CallOpt { value: nvalue, ..opt.clone() }
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
            .map(|exp| exp.as_ref().map(|exp| mapper.map_expr(exp)))
            .collect();
        let ntyp = mapper.map_type(&expr.typ);
        TupleExpr { elems: nelems, typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    pub fn map_index_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a IndexExpr,
    ) -> IndexExpr {
        let nbase = mapper.map_expr(&expr.base_expr);
        let nindex = expr
            .index
            .as_ref()
            .map(|index| Box::new(mapper.map_expr(index)));
        let ntyp = mapper.map_type(&expr.typ);
        IndexExpr { base_expr: Box::new(nbase), index: nindex, typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    pub fn map_slice_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a SliceExpr,
    ) -> SliceExpr {
        let nbase = mapper.map_expr(&expr.base_expr);
        let nstart = expr
            .start_index
            .as_ref()
            .map(|index| Box::new(mapper.map_expr(index)));
        let nend = expr
            .end_index
            .as_ref()
            .map(|index| Box::new(mapper.map_expr(index)));
        let ntyp = mapper.map_type(&expr.typ);
        SliceExpr {
            base_expr: Box::new(nbase),
            start_index: nstart,
            end_index: nend,
            typ: ntyp,
            ..expr.clone()
        }
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    pub fn map_member_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a MemberExpr,
    ) -> MemberExpr {
        let nbase = mapper.map_expr(&expr.base);
        let nmember = mapper.map_name(&expr.member);
        let ntyp = mapper.map_type(&expr.typ);
        MemberExpr { base: Box::new(nbase), member: nmember, typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    pub fn map_conditional_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a ConditionalExpr,
    ) -> ConditionalExpr {
        let ncond = mapper.map_expr(&expr.cond);
        let ntrue_br = mapper.map_expr(&expr.true_br);
        let nbfalse_br = mapper.map_expr(&expr.false_br);
        let ntyp = mapper.map_type(&expr.typ);
        ConditionalExpr {
            cond: Box::new(ncond),
            true_br: Box::new(ntrue_br),
            false_br: Box::new(nbfalse_br),
            typ: ntyp,
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
        let nelems = expr.elems.iter().map(|e| mapper.map_expr(e)).collect();
        let ntyp = mapper.map_type(&expr.typ);
        InlineArrayExpr { elems: nelems, typ: ntyp, ..expr.clone() }
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

    pub fn map_type_name_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a TypeNameExpr,
    ) -> TypeNameExpr {
        let ntyp = mapper.map_type(&expr.typ);
        TypeNameExpr { typ: ntyp, ..expr.clone() }
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn map_ident<'a, T: Map<'a> + ?Sized>(mapper: &mut T, id: &'a Identifier) -> Identifier {
        let nname = mapper.map_name(&id.name);
        Identifier { name: nname, ..id.clone() }
    }

    //-------------------------------------------------
    // Name.
    //-------------------------------------------------

    pub fn map_name<'a, T: Map<'a> + ?Sized>(_mapper: &mut T, name: &'a Name) -> Name {
        name.clone()
    }

    pub fn map_name_opt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        name: &'a Option<Name>,
    ) -> Option<Name> {
        name.as_ref().map(|n| mapper.map_name(n))
    }

    pub fn map_names<'a, T: Map<'a> + ?Sized>(mapper: &mut T, names: &'a [Name]) -> Vec<Name> {
        names.iter().map(|n| mapper.map_name(n)).collect()
    }

    pub fn map_path<'a, T: Map<'a> + ?Sized>(mapper: &mut T, scope: &'a NamePath) -> NamePath {
        let nnames = scope.names.iter().map(|n| mapper.map_name(n)).collect();
        NamePath { names: nnames }
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn map_type<'a, T: Map<'a> + ?Sized>(mapper: &mut T, typ: &'a Type) -> Type {
        match typ {
            Type::Bool => typ.clone(),
            Type::Int(_) => typ.clone(),
            Type::Fixed(_) => typ.clone(),
            Type::Address(_) => typ.clone(),
            Type::Bytes(t) => mapper.map_bytes_type(t).into(),
            Type::String(t) => mapper.map_string_type(t).into(),
            Type::Array(t) => mapper.map_array_type(t).into(),
            Type::Slice(t) => mapper.map_slice_type(t).into(),
            Type::Struct(t) => mapper.map_struct_type(t).into(),
            Type::Enum(t) => mapper.map_enum_type(t).into(),
            Type::Module(_) => typ.clone(),
            Type::Tuple(t) => mapper.map_tuple_type(t).into(),
            Type::Func(t) => mapper.map_func_type(t).into(),
            Type::Mapping(t) => mapper.map_mapping_type(t).into(),
            Type::UserDefined(t) => mapper.map_type_name(t).into(),
            Type::Contract(t) => mapper.map_contract_type(t).into(),
            Type::Magic(t) => mapper.map_magic_type(t).into(),
        }
    }

    pub fn map_bytes_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a BytesType,
    ) -> BytesType {
        let ndata_loc = mapper.map_data_loc(&typ.data_loc);
        BytesType { data_loc: ndata_loc, ..typ.clone() }
    }

    pub fn map_string_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a StringType,
    ) -> StringType {
        let ndata_loc = mapper.map_data_loc(&typ.data_loc);
        StringType { data_loc: ndata_loc, ..typ.clone() }
    }

    pub fn map_array_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a ArrayType,
    ) -> ArrayType {
        let nbase = mapper.map_type(&typ.base);
        let ndata_loc = mapper.map_data_loc(&typ.data_loc);
        ArrayType { base: Box::new(nbase), data_loc: ndata_loc, ..typ.clone() }
    }

    pub fn map_slice_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a SliceType,
    ) -> SliceType {
        let nbase = mapper.map_type(&typ.base);
        SliceType::new(nbase)
    }

    pub fn map_struct_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a StructType,
    ) -> StructType {
        let nname = mapper.map_name(&typ.name);
        let nscope = mapper.map_name_opt(&typ.scope);
        let ndloc = mapper.map_data_loc(&typ.data_loc);
        StructType { name: nname, scope: nscope, data_loc: ndloc, ..typ.clone() }
    }

    pub fn map_enum_type<'a, T: Map<'a> + ?Sized>(mapper: &mut T, typ: &'a EnumType) -> EnumType {
        let nname = mapper.map_name(&typ.name);
        let nscope = mapper.map_name_opt(&typ.scope);
        EnumType { name: nname, scope: nscope }
    }

    pub fn map_tuple_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a TupleType,
    ) -> TupleType {
        let nelems = typ
            .elems
            .iter()
            .map(|elem| elem.as_ref().map(|t| Box::new(mapper.map_type(t))))
            .collect();
        TupleType { elems: nelems }
    }

    pub fn map_function_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a FuncType,
    ) -> FuncType {
        let nparams = typ
            .params
            .iter()
            .map(|t| Box::new(mapper.map_type(t)))
            .collect();
        let nreturns = typ
            .returns
            .iter()
            .map(|t| Box::new(mapper.map_type(t)))
            .collect();
        FuncType { params: nparams, returns: nreturns, ..typ.clone() }
    }

    pub fn map_mapping_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a MappingType,
    ) -> MappingType {
        let nkey = Box::new(mapper.map_type(&typ.key));
        let nvalue = Box::new(mapper.map_type(&typ.value));
        let ndloc = mapper.map_data_loc(&typ.data_loc);
        MappingType { key: nkey, value: nvalue, data_loc: ndloc }
    }

    pub fn map_type_name<'a, T: Map<'a> + ?Sized>(mapper: &mut T, typ: &'a UserDefinedType) -> UserDefinedType {
        let nname = mapper.map_name(&typ.name);
        let nscope = mapper.map_name_opt(&typ.scope);
        UserDefinedType { name: nname, scope: nscope }
    }

    pub fn map_contract_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a ContractType,
    ) -> ContractType {
        let nname = mapper.map_name(&typ.name);
        ContractType { name: nname, ..typ.clone() }
    }

    pub fn map_magic_type<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        typ: &'a MagicType,
    ) -> MagicType {
        match typ {
            MagicType::BlockType => typ.clone(),
            MagicType::MessageType => typ.clone(),
            MagicType::TxnType => typ.clone(),
            MagicType::ABIType => typ.clone(),
            MagicType::MetaType(t) => MagicType::MetaType(Box::new(mapper.map_type(t))),
        }
    }

    //-------------------------------------------------
    // Overriding
    //-------------------------------------------------

    pub fn map_overriding<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        overriding: &'a Overriding,
    ) -> Overriding {
        match overriding {
            Overriding::None | Overriding::All => overriding.clone(),
            Overriding::Some(names) => Overriding::Some(mapper.map_names(names)),
        }
    }

    //-------------------------------------------------
    // Data location
    //-------------------------------------------------

    pub fn map_data_loc<'a, T: Map<'a> + ?Sized>(_mapper: &mut T, dloc: &'a DataLoc) -> DataLoc {
        *dloc
    }
}
