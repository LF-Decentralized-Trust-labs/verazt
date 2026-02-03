//! Module to check similarity of 2 AST.

use crate::ast::*;
use extlib::error::Result;
use crate::ast::{DataLoc, Name, NamePath};

//-------------------------------------------------
// Trait for comparison
//-------------------------------------------------

pub trait Compare<'a> {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn compare_source_unit(&mut self, s1: &'a SourceUnit, s2: &'a SourceUnit) -> Result<()> {
        default::compare_source_unit(self, s1, s2)
    }

    fn compare_source_unit_elem(
        &mut self,
        e1: &'a SourceUnitElem,
        e2: &'a SourceUnitElem,
    ) -> Result<()> {
        default::compare_source_unit_elem(self, e1, e2)
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    fn compare_pragma_directive(&mut self, p1: &'a PragmaDir, p2: &'a PragmaDir) -> Result<()> {
        default::compare_pragma_directive(self, p1, p2)
    }

    fn compare_import_directive(&mut self, i1: &'a ImportDir, i2: &'a ImportDir) -> Result<()> {
        default::compare_import_directive(self, i1, i2)
    }

    fn compare_using_directive(&mut self, u1: &'a UsingDir, u2: &'a UsingDir) -> Result<()> {
        default::compare_using_directive(self, u1, u2)
    }

    //-------------------------------------------------
    // Error & event definitions.
    //-------------------------------------------------

    fn compare_error_defitinion(&mut self, e1: &'a ErrorDef, e2: &'a ErrorDef) -> Result<()> {
        default::compare_error_defitinion(self, e1, e2)
    }

    fn compare_event_definition(&mut self, e1: &'a EventDef, e2: &'a EventDef) -> Result<()> {
        default::compare_event_definition(self, e1, e2)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn compare_user_defined_type_definition(
        &mut self,
        t1: &'a TypeDef,
        t2: &'a TypeDef,
    ) -> Result<()> {
        default::compare_user_defined_type_definition(self, t1, t2)
    }

    fn compare_struct_def(
        &mut self,
        struct1: &'a StructDef,
        struct2: &'a StructDef,
    ) -> Result<()> {
        default::compare_struct_def(self, struct1, struct2)
    }

    fn compare_struct_field(
        &mut self,
        field1: &'a StructField,
        field2: &'a StructField,
    ) -> Result<()> {
        default::compare_struct_field(self, field1, field2)
    }

    fn compare_enum_def(&mut self, enum1: &'a EnumDef, enum2: &'a EnumDef) -> Result<()> {
        default::compare_enum_def(self, enum1, enum2)
    }

    fn compare_enum_elem(&mut self, elem1: &'a str, elem2: &'a str) -> Result<()> {
        default::compare_enum_elem(self, elem1, elem2)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn compare_contract_def(
        &mut self,
        contract1: &'a ContractDef,
        contract2: &'a ContractDef,
    ) -> Result<()> {
        default::compare_contract_def(self, contract1, contract2)
    }

    fn compare_base_contract(
        &mut self,
        contract1: &'a BaseContract,
        contract2: &'a BaseContract,
    ) -> Result<()> {
        default::compare_base_contract(self, contract1, contract2)
    }

    fn compare_contract_elem(
        &mut self,
        elem1: &'a ContractElem,
        elem2: &'a ContractElem,
    ) -> Result<()> {
        default::compare_contract_elem(self, elem1, elem2)
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    fn compare_func_def(&mut self, func1: &'a FuncDef, func2: &'a FuncDef) -> Result<()> {
        default::compare_func_def(self, func1, func2)
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    fn compare_block(&mut self, block1: &'a Block, block2: &'a Block) -> Result<()> {
        default::compare_block(self, block1, block2)
    }

    fn compare_block_opt(
        &mut self,
        block1: &'a Option<Block>,
        block2: &'a Option<Block>,
    ) -> Result<()> {
        default::compare_block_opt(self, block1, block2)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn compare_stmt(&mut self, stmt1: &'a Stmt, stmt2: &'a Stmt) -> Result<()> {
        default::compare_stmt(self, stmt1, stmt2)
    }

    fn compare_stmt_opt(
        &mut self,
        stmt1: &'a Option<Box<Stmt>>,
        stmt2: &'a Option<Box<Stmt>>,
    ) -> Result<()> {
        default::compare_stmt_opt(self, stmt1, stmt2)
    }

    fn compare_asm_stmt(&mut self, stmt1: &'a AsmStmt, stmt2: &'a AsmStmt) -> Result<()> {
        default::compare_asm_stmt(self, stmt1, stmt2)
    }

    fn compare_break_stmt(&mut self, stmt1: &'a BreakStmt, stmt2: &'a BreakStmt) -> Result<()> {
        default::compare_break_stmt(self, stmt1, stmt2)
    }

    fn compare_continue_stmt(
        &mut self,
        stmt1: &'a ContinueStmt,
        stmt2: &'a ContinueStmt,
    ) -> Result<()> {
        default::compare_continue_stmt(self, stmt1, stmt2)
    }

    fn compare_expr_stmt(&mut self, stmt1: &'a ExprStmt, stmt2: &'a ExprStmt) -> Result<()> {
        default::compare_expr_stmt(self, stmt1, stmt2)
    }

    fn compare_if_stmt(&mut self, stmt1: &'a IfStmt, stmt2: &'a IfStmt) -> Result<()> {
        default::compare_if_stmt(self, stmt1, stmt2)
    }

    fn compare_for_stmt(&mut self, stmt1: &'a ForStmt, stmt2: &'a ForStmt) -> Result<()> {
        default::compare_for_stmt(self, stmt1, stmt2)
    }

    fn compare_while_stmt(&mut self, stmt1: &'a WhileStmt, stmt2: &'a WhileStmt) -> Result<()> {
        default::compare_while_stmt(self, stmt1, stmt2)
    }

    fn compare_do_while_stmt(
        &mut self,
        stmt1: &'a DoWhileStmt,
        stmt2: &'a DoWhileStmt,
    ) -> Result<()> {
        default::compare_do_while_stmt(self, stmt1, stmt2)
    }

    fn compare_throw_stmt(&mut self, stmt1: &'a ThrowStmt, stmt2: &'a ThrowStmt) -> Result<()> {
        default::compare_throw_stmt(self, stmt1, stmt2)
    }

    fn compare_try_stmt(&mut self, stmt1: &'a TryStmt, stmt2: &'a TryStmt) -> Result<()> {
        default::compare_try_stmt(self, stmt1, stmt2)
    }

    fn compare_catch_clause(
        &mut self,
        cls1: &'a CatchClause,
        cls2: &'a CatchClause,
    ) -> Result<()> {
        default::compare_catch_clause(self, cls1, cls2)
    }

    fn compare_revert_stmt(&mut self, stmt1: &'a RevertStmt, stmt2: &'a RevertStmt) -> Result<()> {
        default::compare_revert_stmt(self, stmt1, stmt2)
    }

    fn compare_emit_stmt(&mut self, stmt1: &'a EmitStmt, stmt2: &'a EmitStmt) -> Result<()> {
        default::compare_emit_stmt(self, stmt1, stmt2)
    }

    fn compare_place_holder_stmt(
        &mut self,
        stmt1: &'a PlaceholderStmt,
        stmt2: &'a PlaceholderStmt,
    ) -> Result<()> {
        default::compare_place_holder_stmt(self, stmt1, stmt2)
    }

    fn compare_return_stmt(&mut self, stmt1: &'a ReturnStmt, stmt2: &'a ReturnStmt) -> Result<()> {
        default::compare_return_stmt(self, stmt1, stmt2)
    }

    fn compare_var_decl_stmt(
        &mut self,
        stmt1: &'a VarDeclStmt,
        stmt2: &'a VarDeclStmt,
    ) -> Result<()> {
        default::compare_var_decl_stmt(self, stmt1, stmt2)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn compare_var_decl(&mut self, vdecl1: &'a VarDecl, vdecl2: &'a VarDecl) -> Result<()> {
        default::compare_var_decl(self, vdecl1, vdecl2)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn compare_expr(&mut self, exp1: &'a Expr, exp2: &'a Expr) -> Result<()> {
        default::compare_expr(self, exp1, exp2)
    }

    fn compare_expr_opt(&mut self, exp1: &'a Option<Expr>, exp2: &'a Option<Expr>) -> Result<()> {
        default::compare_expr_opt(self, exp1, exp2)
    }

    fn compare_expr_rc_opt(
        &mut self,
        exp1: &'a Option<Box<Expr>>,
        exp2: &'a Option<Box<Expr>>,
    ) -> Result<()> {
        default::compare_expr_rc_opt(self, exp1, exp2)
    }

    fn compare_unary_expr(&mut self, exp1: &'a UnaryExpr, exp2: &'a UnaryExpr) -> Result<()> {
        default::compare_unary_expr(self, exp1, exp2)
    }

    fn compare_binary_expr(&mut self, exp1: &'a BinaryExpr, exp2: &'a BinaryExpr) -> Result<()> {
        default::compare_binary_expr(self, exp1, exp2)
    }

    fn compare_assign_expr(&mut self, exp1: &'a AssignExpr, exp2: &'a AssignExpr) -> Result<()> {
        default::compare_assign_expr(self, exp1, exp2)
    }

    fn compare_call_expr(&mut self, exp1: &'a CallExpr, exp2: &'a CallExpr) -> Result<()> {
        default::compare_call_expr(self, exp1, exp2)
    }

    fn compare_call_args(&mut self, args1: &'a CallArgs, args2: &'a CallArgs) -> Result<()> {
        default::compare_call_args(self, args1, args2)
    }

    fn compare_call_opts_expr(
        &mut self,
        exp1: &'a CallOptsExpr,
        exp2: &'a CallOptsExpr,
    ) -> Result<()> {
        default::compare_call_opts_expr(self, exp1, exp2)
    }

    fn compare_call_opt(&mut self, opt1: &'a CallOpt, opt2: &'a CallOpt) -> Result<()> {
        default::compare_call_opt(self, opt1, opt2)
    }

    fn compare_tuple_expr(&mut self, exp1: &'a TupleExpr, exp2: &'a TupleExpr) -> Result<()> {
        default::compare_tuple_expr(self, exp1, exp2)
    }

    fn compare_index_expr(&mut self, exp1: &'a IndexExpr, exp2: &'a IndexExpr) -> Result<()> {
        default::compare_index_expr(self, exp1, exp2)
    }

    fn compare_slice_expr(&mut self, exp1: &'a SliceExpr, exp2: &'a SliceExpr) -> Result<()> {
        default::compare_slice_expr(self, exp1, exp2)
    }

    fn compare_member_expr(&mut self, exp1: &'a MemberExpr, exp2: &'a MemberExpr) -> Result<()> {
        default::compare_member_expr(self, exp1, exp2)
    }

    fn compare_conditional_expr(
        &mut self,
        exp1: &'a ConditionalExpr,
        exp2: &'a ConditionalExpr,
    ) -> Result<()> {
        default::compare_conditional_expr(self, exp1, exp2)
    }

    fn compare_inline_array_expr(
        &mut self,
        exp1: &'a InlineArrayExpr,
        exp2: &'a InlineArrayExpr,
    ) -> Result<()> {
        default::compare_inline_array_expr(self, exp1, exp2)
    }

    fn compare_new_expr(&mut self, exp1: &'a NewExpr, exp2: &'a NewExpr) -> Result<()> {
        default::compare_new_expr(self, exp1, exp2)
    }

    fn compare_type_name_expr(
        &mut self,
        exp1: &'a TypeNameExpr,
        exp2: &'a TypeNameExpr,
    ) -> Result<()> {
        default::compare_type_name_expr(self, exp1, exp2)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn compare_ident(&mut self, id1: &'a Identifier, id2: &'a Identifier) -> Result<()> {
        default::compare_ident(self, id1, id2)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn compare_name(&mut self, name1: &'a Name, name2: &'a Name) -> Result<()> {
        default::compare_name(self, name1, name2)
    }

    fn compare_name_opt(
        &mut self,
        name1: &'a Option<Name>,
        name2: &'a Option<Name>,
    ) -> Result<()> {
        default::compare_name_opt(self, name1, name2)
    }

    fn compare_names(&mut self, names1: &'a [Name], names2: &'a [Name]) -> Result<()> {
        default::compare_names(self, names1, names2)
    }

    fn compare_name_path(&mut self, path1: &'a NamePath, path2: &'a NamePath) -> Result<()> {
        default::compare_names(self, &path1.names, &path2.names)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn compare_lit(&mut self, l1: &'a Lit, l2: &'a Lit) -> Result<()> {
        default::compare_lit(self, l1, l2)
    }

    fn compare_bool_lit(&mut self, l1: &'a BoolLit, l2: &'a BoolLit) -> Result<()> {
        default::compare_bool_lit(self, l1, l2)
    }

    fn compare_num_lit(&mut self, l1: &'a NumLit, l2: &'a NumLit) -> Result<()> {
        default::compare_num_lit(self, l1, l2)
    }

    fn compare_string_lit(&mut self, l1: &'a StringLit, l2: &'a StringLit) -> Result<()> {
        default::compare_string_lit(self, l1, l2)
    }

    fn compare_hex_lit(&mut self, l1: &'a HexLit, l2: &'a HexLit) -> Result<()> {
        default::compare_hex_lit(self, l1, l2)
    }

    fn compare_unicode_lit(&mut self, l1: &'a UnicodeLit, l2: &'a UnicodeLit) -> Result<()> {
        default::compare_unicode_lit(self, l1, l2)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn compare_type(&mut self, t1: &'a Type, t2: &'a Type) -> Result<()> {
        default::compare_type(self, t1, t2)
    }

    fn compare_int_type(&mut self, t1: &'a IntType, t2: &'a IntType) -> Result<()> {
        default::compare_int_type(self, t1, t2)
    }

    fn compare_fixed_type(&mut self, t1: &'a FixedType, t2: &'a FixedType) -> Result<()> {
        default::compare_fixed_type(self, t1, t2)
    }

    fn compare_string_type(&mut self, t1: &'a StringType, t2: &'a StringType) -> Result<()> {
        default::compare_string_type(self, t1, t2)
    }

    fn compare_address_type(&mut self, t1: &'a AddressType, t2: &'a AddressType) -> Result<()> {
        default::compare_address_type(self, t1, t2)
    }

    fn compare_bytes_type(&mut self, t1: &'a BytesType, t2: &'a BytesType) -> Result<()> {
        default::compare_bytes_type(self, t1, t2)
    }

    fn compare_array_type(&mut self, t1: &'a ArrayType, t2: &'a ArrayType) -> Result<()> {
        default::compare_array_type(self, t1, t2)
    }

    fn compare_slice_type(&mut self, t1: &'a SliceType, t2: &'a SliceType) -> Result<()> {
        default::compare_slice_type(self, t1, t2)
    }

    fn compare_struct_type(&mut self, t1: &'a StructType, t2: &'a StructType) -> Result<()> {
        default::compare_struct_type(self, t1, t2)
    }

    fn compare_enum_type(&mut self, t1: &'a EnumType, t2: &'a EnumType) -> Result<()> {
        default::compare_enum_type(self, t1, t2)
    }

    fn compare_tuple_type(&mut self, t1: &'a TupleType, t2: &'a TupleType) -> Result<()> {
        default::compare_tuple_type(self, t1, t2)
    }

    fn compare_func_type(&mut self, t1: &'a FuncType, t2: &'a FuncType) -> Result<()> {
        default::compare_func_type(self, t1, t2)
    }

    fn compare_mapping_type(&mut self, t1: &'a MappingType, t2: &'a MappingType) -> Result<()> {
        default::compare_mapping_type(self, t1, t2)
    }

    fn compare_type_name(
        &mut self,
        t1: &'a UserDefinedType,
        t2: &'a UserDefinedType,
    ) -> Result<()> {
        default::compare_type_name(self, t1, t2)
    }

    fn compare_contract_type(&mut self, t1: &'a ContractType, t2: &'a ContractType) -> Result<()> {
        default::compare_contract_type(self, t1, t2)
    }

    fn compare_magic_type(&mut self, t1: &'a MagicType, t2: &'a MagicType) -> Result<()> {
        default::compare_magic_type(self, t1, t2)
    }

    //-------------------------------------------------
    // Data location.
    //-------------------------------------------------

    fn compare_data_loc(&mut self, dloc1: &'a DataLoc, dloc2: &'a DataLoc) -> Result<()> {
        default::compare_data_loc(self, dloc1, dloc2)
    }
}

//------------------------------------------------------------------
// Default comparison utilities
//------------------------------------------------------------------

/// Module contain default implementation of the AST comparison utilities.
pub mod default {
    use super::Compare;
    use crate::ast::*;
    use extlib::{
        error::{Report, Result},
        fail,
    };
    use crate::ast::{DataLoc, Name};
    use std::fmt::Display;

    //-------------------------------------------------
    // Helper functions
    //-------------------------------------------------

    fn error<T: Display>(msg: &str, t1: T, t2: T, err: Option<Report>) -> Result<()> {
        let msg = format!("{msg}:\n\n{t1}\n\n-- vs --\n\n{t2}\n");
        match err {
            Some(err) => fail!("{}\n\nError: {}", msg, err),
            None => fail!("{}", msg),
        }
    }

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn compare_source_unit<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        sunit1: &'a SourceUnit,
        sunit2: &'a SourceUnit,
    ) -> Result<()> {
        if sunit1.elems.len() != sunit2.elems.len() {
            return error("Different number of source unit elements", sunit1, sunit2, None);
        }
        for (e1, e2) in sunit1.elems.iter().zip(sunit2.elems.iter()) {
            comparer.compare_source_unit_elem(e1, e2)?
        }
        Ok(())
    }

    pub fn compare_source_unit_elem<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        elem1: &'a SourceUnitElem,
        elem2: &'a SourceUnitElem,
    ) -> Result<()> {
        use SourceUnitElem::*;
        match (elem1, elem2) {
            (Pragma(p1), Pragma(p2)) => comparer.compare_pragma_directive(p1, p2),
            (Import(i1), Import(i2)) => comparer.compare_import_directive(i1, i2),
            (Using(u1), Using(u2)) => comparer.compare_using_directive(u1, u2),
            (Error(e1), Error(e2)) => comparer.compare_error_defitinion(e1, e2),
            (Func(f1), Func(f2)) => comparer.compare_func_def(f1, f2),
            (Var(v1), Var(v2)) => comparer.compare_var_decl(v1, v2),
            (UserType(t1), UserType(t2)) => comparer.compare_user_defined_type_definition(t1, t2),
            (Struct(s1), Struct(s2)) => comparer.compare_struct_def(s1, s2),
            (Enum(e1), Enum(e2)) => comparer.compare_enum_def(e1, e2),
            (Contract(c1), Contract(c2)) => comparer.compare_contract_def(c1, c2),
            _ => error("Different source unit element", elem1, elem2, None),
        }
    }

    //-------------------------------------------------
    // Directives
    //-------------------------------------------------

    pub fn compare_pragma_directive<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        pragma1: &'a PragmaDir,
        pragma2: &'a PragmaDir,
    ) -> Result<()> {
        if pragma1.to_string() != pragma2.to_string() {
            return error("Different pragma", pragma1, pragma2, None);
        }
        Ok(())
    }

    pub fn compare_import_directive<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        import1: &'a ImportDir,
        import2: &'a ImportDir,
    ) -> Result<()> {
        if import1.to_string() != import2.to_string() {
            return error("Different import", import1, import2, None);
        }
        Ok(())
    }

    pub fn compare_using_directive<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        using1: &'a UsingDir,
        using2: &'a UsingDir,
    ) -> Result<()> {
        if using1.to_string() != using2.to_string() {
            return error("Different using", using1, using2, None);
        }
        Ok(())
    }

    //-------------------------------------------------
    // Error definition
    //-------------------------------------------------

    pub fn compare_error_defitinion<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        error1: &'a ErrorDef,
        error2: &'a ErrorDef,
    ) -> Result<()> {
        if error1.params.len() != error2.params.len() {
            return error("Different error definition", error1, error2, None);
        }
        if let Err(err) = comparer.compare_name(&error1.name, &error2.name) {
            return error("Different error definition", error1, error2, Some(err));
        }
        for (p1, p2) in error1.params.iter().zip(error2.params.iter()) {
            comparer.compare_var_decl(p1, p2)?
        }
        Ok(())
    }

    pub fn compare_event_definition<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        event1: &'a EventDef,
        event2: &'a EventDef,
    ) -> Result<()> {
        if comparer.compare_name(&event1.name, &event2.name).is_err()
            || !event1.is_anonymous.eq(&event2.is_anonymous)
            || event1.params.len() != event2.params.len()
        {
            return error("Different event definitions", event1, event2, None);
        }
        for (p1, p2) in event1.params.iter().zip(event2.params.iter()) {
            comparer.compare_var_decl(p1, p2)?
        }
        Ok(())
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    pub fn compare_user_defined_type_definition<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        typ1: &'a TypeDef,
        typ2: &'a TypeDef,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&typ1.name, &typ2.name) {
            return error("Different user-defined value type definition", typ1, typ2, Some(err));
        }
        comparer.compare_type(&typ1.base_typ, &typ2.base_typ)
    }

    pub fn compare_struct_def<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        struct1: &'a StructDef,
        struct2: &'a StructDef,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&struct1.name, &struct2.name) {
            return error("Different struct name", &struct1, &struct2, Some(err));
        }
        if struct1.fields.len() != struct2.fields.len() {
            return error("Different number of struct fields", &struct1, &struct2, None);
        }
        for (field1, field2) in struct1.fields.iter().zip(struct2.fields.iter()) {
            if let Err(err) = comparer.compare_struct_field(field1, field2) {
                return error("Different struct definition", &struct1, &struct2, Some(err));
            }
        }
        Ok(())
    }

    pub fn compare_struct_field<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        field1: &'a StructField,
        field2: &'a StructField,
    ) -> Result<()> {
        if !field1.name.eq(&field2.name) {
            return error("Different struct field", &field1, &field2, None);
        }
        comparer.compare_type(&field1.typ, &field2.typ)
    }

    pub fn compare_enum_def<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        enum1: &'a EnumDef,
        enum2: &'a EnumDef,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&enum1.name, &enum2.name) {
            return error("Different enum name", &enum1, &enum2, Some(err));
        }
        if enum1.elems.len() != enum2.elems.len() {
            return error("Different number of enum elements", &enum1, &enum2, None);
        }
        for (elem1, elem2) in enum1.elems.iter().zip(enum2.elems.iter()) {
            if let Err(err) = comparer.compare_enum_elem(elem1, elem2) {
                return error("Different enum definition", &enum1, &enum2, Some(err));
            }
        }
        Ok(())
    }

    pub fn compare_enum_elem<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        elem1: &'a str,
        elem2: &'a str,
    ) -> Result<()> {
        if !elem1.eq(elem2) {
            return error("Different enum element", &elem1, &elem2, None);
        }
        Ok(())
    }

    //-------------------------------------------------
    // Contract definition
    //-------------------------------------------------

    pub fn compare_contract_def<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        ct1: &'a ContractDef,
        ct2: &'a ContractDef,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&ct1.name, &ct2.name) {
            return error("Different contract name", &ct1, &ct2, Some(err));
        }
        if ct1.kind != ct2.kind {
            return error("Different contract kind", &ct1, &ct2, None);
        }
        if ct1.is_abstract != ct2.is_abstract {
            return error("Different contract abstract type", &ct1, &ct2, None);
        }
        if ct1.base_contracts.len() != ct2.base_contracts.len() {
            return error("Different contract bases", &ct1, &ct2, None);
        }
        if ct1.body.len() != ct2.body.len() {
            return error("Different number of contract elements", &ct1, &ct2, None);
        }
        for (b1, b2) in ct1.base_contracts.iter().zip(ct2.base_contracts.iter()) {
            comparer.compare_base_contract(b1, b2)?
        }
        for (elem1, elem2) in ct1.body.iter().zip(ct2.body.iter()) {
            comparer.compare_contract_elem(elem1, elem2)?
        }
        Ok(())
    }

    pub fn compare_base_contract<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        base1: &'a BaseContract,
        base2: &'a BaseContract,
    ) -> Result<()> {
        if !base1.name.eq(&base2.name) || base1.args.len() != base2.args.len() {
            return error("Different base contract", base1, base2, None);
        }
        for (arg1, arg2) in base1.args.iter().zip(base2.args.iter()) {
            comparer.compare_expr(arg1, arg2)?
        }
        Ok(())
    }

    pub fn compare_contract_elem<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        elem1: &'a ContractElem,
        elem2: &'a ContractElem,
    ) -> Result<()> {
        use ContractElem::*;
        match (elem1, elem2) {
            (Using(u1), Using(u2)) => comparer.compare_using_directive(u1, u2),
            (Event(e1), Event(e2)) => comparer.compare_event_definition(e1, e2),
            (Error(e1), Error(e2)) => comparer.compare_error_defitinion(e1, e2),
            (Struct(s1), Struct(s2)) => comparer.compare_struct_def(s1, s2),
            (Enum(e1), Enum(e2)) => comparer.compare_enum_def(e1, e2),
            (Type(type1), Type(type2)) => {
                comparer.compare_user_defined_type_definition(type1, type2)
            }
            (Var(var1), Var(var2)) => comparer.compare_var_decl(var1, var2),
            (Func(func1), Func(func2)) => comparer.compare_func_def(func1, func2),
            _ => error("Different contract elements", elem1, elem2, None),
        }
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    pub fn compare_func_def<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        f1: &'a FuncDef,
        f2: &'a FuncDef,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&f1.name, &f2.name) {
            return error("Different function ", &f1, &f2, Some(err));
        }
        if f1.kind != f2.kind {
            return error("Different function kind", &f1.name, &f2.name, None);
        }
        if f1.visibility != f2.visibility {
            return error("Different function visibility", &f1.name, &f2.name, None);
        }
        if f1.mutability != f2.mutability {
            return error("Different function mutability", &f1.name, &f2.name, None);
        }
        if f1.is_virtual != f2.is_virtual {
            return error("Different function virtualilty", &f1.name, &f2.name, None);
        }
        if f1.params.len() != f2.params.len() {
            return error("Different function parameters", &f1.name, &f2.name, None);
        }
        if f1.returns.len() != f2.returns.len() {
            return error("Different function returns", &f1.name, &f2.name, None);
        }
        if f1.modifier_invocs.len() != f2.modifier_invocs.len() {
            return error("Different function modifiers", f1, f2, None);
        }
        for (p1, p2) in f1.params.iter().zip(f2.params.iter()) {
            comparer.compare_var_decl(p1, p2)?
        }
        for (p1, p2) in f1.returns.iter().zip(f2.returns.iter()) {
            comparer.compare_var_decl(p1, p2)?
        }
        for (m1, m2) in f1.modifier_invocs.iter().zip(f2.modifier_invocs.iter()) {
            comparer.compare_call_expr(m1, m2)?
        }
        match (&f1.overriding, &f2.overriding) {
            (Overriding::Some(names1), Overriding::Some(names2)) => {
                comparer.compare_names(names1, names2)?
            }
            (Overriding::All, Overriding::All) => {}
            (Overriding::None, Overriding::None) => {}
            _ => return error("Different function overrides", f1, f2, None),
        }
        comparer.compare_block_opt(&f1.body, &f2.body)
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    pub fn compare_block<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        blk1: &'a Block,
        blk2: &'a Block,
    ) -> Result<()> {
        if blk1.unchecked != blk2.unchecked || blk1.body.len() != blk2.body.len() {
            return error("Different blocks", blk1, blk2, None);
        }
        for (s1, s2) in blk1.body.iter().zip(blk2.body.iter()) {
            comparer.compare_stmt(s1, s2)?
        }
        Ok(())
    }

    pub fn compare_block_opt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        blk1: &'a Option<Block>,
        blk2: &'a Option<Block>,
    ) -> Result<()> {
        match (blk1, blk2) {
            (Some(b1), Some(b2)) => comparer.compare_block(b1, b2),
            (None, None) => Ok(()),
            _ => fail!("Different block: {:?} -- {:?}", blk1, blk2),
        }
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn compare_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a Stmt,
        stmt2: &'a Stmt,
    ) -> Result<()> {
        use Stmt::*;
        match (stmt1, stmt2) {
            (Asm(s1), Asm(s2)) => comparer.compare_asm_stmt(s1, s2),
            (Block(blk1), Block(blk2)) => comparer.compare_block(blk1, blk2),
            (Break(s1), Break(s2)) => comparer.compare_break_stmt(s1, s2),
            (Continue(s1), Continue(s2)) => comparer.compare_continue_stmt(s1, s2),
            (DoWhile(s1), DoWhile(s2)) => comparer.compare_do_while_stmt(s1, s2),
            (Emit(s1), Emit(s2)) => comparer.compare_emit_stmt(s1, s2),
            (Expr(s1), Expr(s2)) => comparer.compare_expr_stmt(s1, s2),
            (For(s1), For(s2)) => comparer.compare_for_stmt(s1, s2),
            (If(s1), If(s2)) => comparer.compare_if_stmt(s1, s2),
            (Placeholder(s1), Placeholder(s2)) => comparer.compare_place_holder_stmt(s1, s2),
            (Return(s1), Return(s2)) => comparer.compare_return_stmt(s1, s2),
            (Revert(s1), Revert(s2)) => comparer.compare_revert_stmt(s1, s2),
            (Throw(s1), Throw(s2)) => comparer.compare_throw_stmt(s1, s2),
            (Try(s1), Try(s2)) => comparer.compare_try_stmt(s1, s2),
            (VarDecl(s1), VarDecl(s2)) => comparer.compare_var_decl_stmt(s1, s2),
            (While(s1), While(s2)) => comparer.compare_while_stmt(s1, s2),
            _ => error("Different statements", stmt1, stmt2, None),
        }
    }

    pub fn compare_stmt_opt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a Option<Box<Stmt>>,
        stmt2: &'a Option<Box<Stmt>>,
    ) -> Result<()> {
        match (stmt1, stmt2) {
            (Some(s1), Some(s2)) => comparer.compare_stmt(s1, s2),
            (None, None) => Ok(()),
            _ => panic!("Different statement: {:?} -- {:?}", stmt1, stmt2),
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    pub fn compare_asm_stmt<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        _stmt1: &'a AsmStmt,
        _stmt2: &'a AsmStmt,
    ) -> Result<()> {
        panic!("TODO: compare 2 assembly statements")
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    pub fn compare_break_stmt<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        _stmt1: &'a BreakStmt,
        _stmt2: &'a BreakStmt,
    ) -> Result<()> {
        Ok(())
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    pub fn compare_continue_stmt<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        _stmt1: &'a ContinueStmt,
        _stmt2: &'a ContinueStmt,
    ) -> Result<()> {
        Ok(())
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    pub fn compare_expr_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a ExprStmt,
        stmt2: &'a ExprStmt,
    ) -> Result<()> {
        comparer.compare_expr(&stmt1.expr, &stmt2.expr)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    pub fn compare_if_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a IfStmt,
        stmt2: &'a IfStmt,
    ) -> Result<()> {
        comparer.compare_expr(&stmt1.condition, &stmt2.condition)?;
        comparer.compare_stmt(&stmt1.true_branch, &stmt2.true_branch)?;
        comparer.compare_stmt_opt(&stmt1.false_branch, &stmt2.false_branch)?;
        Ok(())
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    pub fn compare_for_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        s1: &'a ForStmt,
        s2: &'a ForStmt,
    ) -> Result<()> {
        comparer.compare_expr_opt(&s1.condition, &s2.condition)?;
        comparer.compare_stmt_opt(&s1.pre_loop, &s2.pre_loop)?;
        comparer.compare_stmt_opt(&s1.post_loop, &s2.post_loop)?;
        comparer.compare_stmt(&s1.body, &s2.body)
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    pub fn compare_while_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a WhileStmt,
        stmt2: &'a WhileStmt,
    ) -> Result<()> {
        comparer.compare_expr(&stmt1.condition, &stmt2.condition)?;
        comparer.compare_stmt(&stmt1.body, &stmt2.body)
    }

    //-------------------------------------------------
    // Do-while statement
    //-------------------------------------------------

    pub fn compare_do_while_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a DoWhileStmt,
        stmt2: &'a DoWhileStmt,
    ) -> Result<()> {
        comparer.compare_expr(&stmt1.condition, &stmt2.condition)?;
        comparer.compare_stmt(&stmt1.body, &stmt2.body)
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    pub fn compare_throw_stmt<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        _stmt1: &'a ThrowStmt,
        _stmt2: &'a ThrowStmt,
    ) -> Result<()> {
        Ok(())
    }

    //-------------------------------------------------
    // Try statement
    //-------------------------------------------------

    pub fn compare_try_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a TryStmt,
        stmt2: &'a TryStmt,
    ) -> Result<()> {
        comparer.compare_expr(&stmt1.guarded_expr, &stmt2.guarded_expr)?;
        comparer.compare_block(&stmt1.body, &stmt2.body)?;
        if stmt1.returns.len() != stmt2.returns.len() {
            return error("Different parameters in try statement", stmt1, stmt2, None);
        }
        for (p1, p2) in stmt1.returns.iter().zip(stmt2.returns.iter()) {
            comparer.compare_var_decl(p1, p2)?
        }
        if stmt1.catch_clauses.len() != stmt2.catch_clauses.len() {
            return error("Different catch clauses in try statements", stmt1, stmt2, None);
        }
        for (c1, c2) in stmt1.catch_clauses.iter().zip(stmt2.catch_clauses.iter()) {
            comparer.compare_catch_clause(c1, c2)?;
        }
        Ok(())
    }

    pub fn compare_catch_clause<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        cls1: &'a CatchClause,
        cls2: &'a CatchClause,
    ) -> Result<()> {
        match (&cls1.error, &cls2.error) {
            (Some(error1), Some(error2)) if error1 != error2 => {
                return error("Different errors in catch clause", cls1, cls2, None);
            }
            (None, None) => {}
            _ => {
                return error("Different errors in catch clause", cls1, cls2, None);
            }
        }
        if cls1.params.len() != cls2.params.len() {
            return error("Different parameters in catch clause", cls1, cls2, None);
        }
        for (p1, p2) in cls1.params.iter().zip(cls2.params.iter()) {
            comparer.compare_var_decl(p1, p2)?;
        }
        comparer.compare_block(&cls1.body, &cls2.body)
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    pub fn compare_revert_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a RevertStmt,
        stmt2: &'a RevertStmt,
    ) -> Result<()> {
        comparer.compare_expr_opt(&stmt1.error, &stmt2.error)?;
        comparer.compare_call_args(&stmt1.args, &stmt2.args)
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    pub fn compare_emit_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a EmitStmt,
        stmt2: &'a EmitStmt,
    ) -> Result<()> {
        comparer.compare_expr(&stmt1.event, &stmt2.event)?;
        comparer.compare_call_args(&stmt1.args, &stmt2.args)
    }

    //-------------------------------------------------
    // Placeholder statement
    //-------------------------------------------------

    pub fn compare_place_holder_stmt<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        _stmt1: &'a PlaceholderStmt,
        _stmt2: &'a PlaceholderStmt,
    ) -> Result<()> {
        Ok(())
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    pub fn compare_return_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a ReturnStmt,
        stmt2: &'a ReturnStmt,
    ) -> Result<()> {
        comparer.compare_expr_opt(&stmt1.expr, &stmt2.expr)
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    pub fn compare_var_decl_stmt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        stmt1: &'a VarDeclStmt,
        stmt2: &'a VarDeclStmt,
    ) -> Result<()> {
        if stmt1.var_decls.len() != stmt2.var_decls.len() {
            return error("Different variable declaration statement", stmt1, stmt2, None);
        }
        for (v1, v2) in stmt1.var_decls.iter().zip(stmt2.var_decls.iter()) {
            match (v1, v2) {
                (Some(v1), Some(v2)) => comparer.compare_var_decl(v1, v2)?,
                (None, None) => {}
                _ => {
                    fail!("Different var decl: {:?} -- {:?}", v1, v2);
                }
            }
        }
        comparer.compare_expr_opt(&stmt1.value, &stmt2.value)
    }

    //-------------------------------------------------
    // Arguments
    //-------------------------------------------------

    pub fn compare_call_args<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        args1: &'a CallArgs,
        args2: &'a CallArgs,
    ) -> Result<()> {
        match (args1, args2) {
            (CallArgs::Unnamed(args1), CallArgs::Unnamed(args2)) => {
                if args1.len() != args2.len() {
                    fail!("Different number of arguments");
                }
                for (e1, e2) in args1.iter().zip(args2.iter()) {
                    comparer.compare_expr(e1, e2)?
                }
            }
            (CallArgs::Named(args1), CallArgs::Named(args2)) => {
                if args1.len() != args2.len() {
                    fail!("Different number of arguments");
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    if a1.name != a2.name {
                        fail!("Different argument names");
                    }
                    comparer.compare_expr(&a1.value, &a2.value)?
                }
            }
            _ => fail!("Different argument types"),
        }
        Ok(())
    }

    //-------------------------------------------------
    // Variable declaration
    //-------------------------------------------------

    pub fn compare_var_decl<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        vdecl1: &'a VarDecl,
        vdecl2: &'a VarDecl,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&vdecl1.name, &vdecl2.name) {
            return error("Different var decl", &vdecl1.name, &vdecl2.name, Some(err));
        }
        comparer.compare_type(&vdecl1.typ, &vdecl2.typ)?;
        comparer.compare_expr_opt(&vdecl1.value, &vdecl2.value)
    }

    //-------------------------------------------------
    // Expression
    //-------------------------------------------------

    pub fn compare_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a Expr,
        exp2: &'a Expr,
    ) -> Result<()> {
        use Expr::*;
        match (exp1, exp2) {
            (Ident(id1), Ident(id2)) => comparer.compare_ident(id1, id2),
            (Lit(l1), Lit(l2)) => comparer.compare_lit(l1, l2),
            (Unary(e1), Unary(e2)) => comparer.compare_unary_expr(e1, e2),
            (Binary(e1), Binary(e2)) => comparer.compare_binary_expr(e1, e2),
            (Assign(e1), Assign(e2)) => comparer.compare_assign_expr(e1, e2),
            (Call(e1), Call(e2)) => comparer.compare_call_expr(e1, e2),
            (CallOpts(e1), CallOpts(e2)) => comparer.compare_call_opts_expr(e1, e2),
            (Tuple(e1), Tuple(e2)) => comparer.compare_tuple_expr(e1, e2),
            (Index(e1), Index(e2)) => comparer.compare_index_expr(e1, e2),
            (Slice(e1), Slice(e2)) => comparer.compare_slice_expr(e1, e2),
            (Member(e1), Member(e2)) => comparer.compare_member_expr(e1, e2),
            (Conditional(e1), Conditional(e2)) => comparer.compare_conditional_expr(e1, e2),
            (InlineArray(e1), InlineArray(e2)) => comparer.compare_inline_array_expr(e1, e2),
            (New(e1), New(e2)) => comparer.compare_new_expr(e1, e2),
            (TypeName(e1), TypeName(e2)) => comparer.compare_type_name_expr(e1, e2),
            _ => error("Different expression types", exp1, exp2, None),
        }
    }

    pub fn compare_expr_opt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a Option<Expr>,
        exp2: &'a Option<Expr>,
    ) -> Result<()> {
        match (exp1, exp2) {
            (Some(e1), Some(e2)) => comparer.compare_expr(e1, e2),
            (None, None) => Ok(()),
            _ => fail!("Different expressions: {:?} -- {:?}", exp1, exp2,),
        }
    }

    pub fn compare_expr_rc_opt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a Option<Box<Expr>>,
        exp2: &'a Option<Box<Expr>>,
    ) -> Result<()> {
        match (exp1, exp2) {
            (Some(e1), Some(e2)) => comparer.compare_expr(e1, e2),
            (None, None) => Ok(()),
            _ => fail!("Different expressions: {:?} -- {:?}", exp1, exp2),
        }
    }

    pub fn compare_unary_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a UnaryExpr,
        exp2: &'a UnaryExpr,
    ) -> Result<()> {
        if exp1.op != exp2.op {
            return error("Different unary operators", exp1, exp2, None);
        }
        comparer.compare_expr(&exp1.body, &exp2.body)
    }

    pub fn compare_binary_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a BinaryExpr,
        exp2: &'a BinaryExpr,
    ) -> Result<()> {
        if exp1.operator != exp2.operator {
            return error("Different binary operators", exp1, exp2, None);
        }
        comparer.compare_expr(&exp1.left, &exp2.left)?;
        comparer.compare_expr(&exp1.right, &exp2.right)
    }

    pub fn compare_assign_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a AssignExpr,
        exp2: &'a AssignExpr,
    ) -> Result<()> {
        if exp1.operator != exp2.operator {
            return error("Different assignment operators", exp1, exp2, None);
        }
        comparer.compare_expr(&exp1.left, &exp2.left)?;
        comparer.compare_expr(&exp1.right, &exp2.right)
    }

    pub fn compare_call_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a CallExpr,
        exp2: &'a CallExpr,
    ) -> Result<()> {
        comparer.compare_expr(&exp1.callee, &exp2.callee)?;
        comparer.compare_call_args(&exp1.args, &exp2.args)
    }

    pub fn compare_call_opts_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a CallOptsExpr,
        exp2: &'a CallOptsExpr,
    ) -> Result<()> {
        comparer.compare_expr(&exp1.callee, &exp2.callee)?;
        if exp1.call_opts.len() != exp2.call_opts.len() {
            return error("Different number of call options", exp1, exp2, None);
        }
        for (o1, o2) in exp1.call_opts.iter().zip(exp2.call_opts.iter()) {
            comparer.compare_call_opt(o1, o2)?;
        }
        Ok(())
    }

    pub fn compare_call_opt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        opt1: &'a CallOpt,
        opt2: &'a CallOpt,
    ) -> Result<()> {
        if opt1.name != opt2.name {
            return error("Different call option names", opt1, opt2, None);
        }
        comparer.compare_expr(&opt1.value, &opt2.value)
    }

    pub fn compare_tuple_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a TupleExpr,
        exp2: &'a TupleExpr,
    ) -> Result<()> {
        if exp1.elems.len() != exp2.elems.len() {
            return error("Different number of tuple elements", exp1, exp2, None);
        }
        for (e1, e2) in exp1.elems.iter().zip(exp2.elems.iter()) {
            comparer.compare_expr_opt(e1, e2)?
        }
        Ok(())
    }

    pub fn compare_index_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a IndexExpr,
        exp2: &'a IndexExpr,
    ) -> Result<()> {
        comparer.compare_expr(&exp1.base_expr, &exp2.base_expr)?;
        comparer.compare_expr_rc_opt(&exp1.index, &exp2.index)
    }

    pub fn compare_slice_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a SliceExpr,
        exp2: &'a SliceExpr,
    ) -> Result<()> {
        comparer.compare_expr(&exp1.base_expr, &exp2.base_expr)?;
        comparer.compare_expr_rc_opt(&exp1.start_index, &exp2.start_index)?;
        comparer.compare_expr_rc_opt(&exp1.end_index, &exp2.end_index)
    }

    pub fn compare_member_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a MemberExpr,
        exp2: &'a MemberExpr,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&exp1.member, &exp2.member) {
            return error("Different member access expr", exp1, exp2, Some(err));
        }
        comparer.compare_expr(&exp1.base, &exp2.base)
    }

    pub fn compare_conditional_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a ConditionalExpr,
        exp2: &'a ConditionalExpr,
    ) -> Result<()> {
        comparer.compare_expr(&exp1.cond, &exp2.cond)?;
        comparer.compare_expr(&exp1.true_br, &exp2.true_br)?;
        comparer.compare_expr(&exp1.false_br, &exp2.false_br)?;
        Ok(())
    }

    pub fn compare_inline_array_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a InlineArrayExpr,
        exp2: &'a InlineArrayExpr,
    ) -> Result<()> {
        if exp1.elems.len() != exp2.elems.len() {
            return error("Different inline array", exp1, exp2, None);
        }
        for (e1, e2) in exp1.elems.iter().zip(exp2.elems.iter()) {
            comparer.compare_expr(e1, e2)?
        }
        Ok(())
    }

    pub fn compare_new_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a NewExpr,
        exp2: &'a NewExpr,
    ) -> Result<()> {
        comparer.compare_type(&exp1.typ, &exp2.typ)
    }

    pub fn compare_type_name_expr<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        exp1: &'a TypeNameExpr,
        exp2: &'a TypeNameExpr,
    ) -> Result<()> {
        comparer.compare_type(&exp1.typ, &exp2.typ)
    }

    //-------------------------------------------------
    // Identifiers
    //-------------------------------------------------

    pub fn compare_ident<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        id1: &'a Identifier,
        id2: &'a Identifier,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&id1.name, &id2.name) {
            return error("Different identifiers", id1, id2, Some(err));
        }
        Ok(())
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    pub fn compare_name_opt<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        name1: &'a Option<Name>,
        name2: &'a Option<Name>,
    ) -> Result<()> {
        match (name1, name2) {
            (Some(name1), Some(name2)) => comparer.compare_name(name1, name2),
            (None, None) => Ok(()),
            _ => fail!("Different name: {:?} -- {:?}", name1, name2),
        }
    }

    pub fn compare_names<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        names1: &'a [Name],
        names2: &'a [Name],
    ) -> Result<()> {
        if names1.len() != names2.len() {
            fail!("Different names: {:?} -- {:?}", names1, names2);
        }
        for (n1, n2) in names1.iter().zip(names2.iter()) {
            comparer.compare_name(n1, n2)?
        }
        Ok(())
    }

    pub fn compare_name<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        name1: &'a Name,
        name2: &'a Name,
    ) -> Result<()> {
        if name1.to_string() != name2.to_string() {
            return error("Different names", name1, name2, None);
        }
        Ok(())
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    pub fn compare_lit<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        l1: &'a Lit,
        l2: &'a Lit,
    ) -> Result<()> {
        use Lit::*;
        match (l1, l2) {
            (Bool(b1), Bool(b2)) => comparer.compare_bool_lit(b1, b2),
            (Num(n1), Num(n2)) => comparer.compare_num_lit(n1, n2),
            (String(s1), String(s2)) => comparer.compare_string_lit(s1, s2),
            (Hex(h1), Hex(h2)) => comparer.compare_hex_lit(h1, h2),
            (Unicode(u1), Unicode(u2)) => comparer.compare_unicode_lit(u1, u2),
            _ => error("Different literal types", l1, l2, None),
        }
    }

    pub fn compare_bool_lit<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        l1: &'a BoolLit,
        l2: &'a BoolLit,
    ) -> Result<()> {
        if l1.value != l2.value || comparer.compare_type(&l1.typ, &l2.typ).is_err() {
            return error("Different boolean literals", l1, l2, None);
        }
        Ok(())
    }

    pub fn compare_num_lit<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        l1: &'a NumLit,
        l2: &'a NumLit,
    ) -> Result<()> {
        if l1.value != l2.value || l1.unit != l2.unit {
            return error("Different number literals", l1, l2, None);
        }
        Ok(())
    }

    pub fn compare_string_lit<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        l1: &'a StringLit,
        l2: &'a StringLit,
    ) -> Result<()> {
        if l1.value != l2.value || comparer.compare_type(&l1.typ, &l2.typ).is_err() {
            return error("Different string literals", l1, l2, None);
        }
        Ok(())
    }

    pub fn compare_hex_lit<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        l1: &'a HexLit,
        l2: &'a HexLit,
    ) -> Result<()> {
        if l1.value != l2.value || comparer.compare_type(&l1.typ, &l2.typ).is_err() {
            return error("Different hex-string literals", l1, l2, None);
        }
        Ok(())
    }

    pub fn compare_unicode_lit<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        l1: &'a UnicodeLit,
        l2: &'a UnicodeLit,
    ) -> Result<()> {
        if l1.value != l2.value || comparer.compare_type(&l1.typ, &l2.typ).is_err() {
            return error("Different unicode-string literals", l1, l2, None);
        }
        Ok(())
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn compare_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a Type,
        t2: &'a Type,
    ) -> Result<()> {
        match (t1, t2) {
            (Type::Bool, Type::Bool) => Ok(()),
            (Type::Int(ty1), Type::Int(ty2)) => comparer.compare_int_type(ty1, ty2),
            (Type::Fixed(ty1), Type::Fixed(ty2)) => comparer.compare_fixed_type(ty1, ty2),
            (Type::String(ty1), Type::String(ty2)) => comparer.compare_string_type(ty1, ty2),
            (Type::Address(ty1), Type::Address(ty2)) => comparer.compare_address_type(ty1, ty2),
            (Type::Bytes(ty1), Type::Bytes(ty2)) => comparer.compare_bytes_type(ty1, ty2),
            (Type::Array(ty1), Type::Array(ty2)) => comparer.compare_array_type(ty1, ty2),
            (Type::Slice(ty1), Type::Slice(ty2)) => comparer.compare_slice_type(ty1, ty2),
            (Type::Struct(ty1), Type::Struct(ty2)) => comparer.compare_struct_type(ty1, ty2),
            (Type::Enum(ty1), Type::Enum(ty2)) => comparer.compare_enum_type(ty1, ty2),
            (Type::Module(m1), Type::Module(m2)) if m1 == m2 => Ok(()),
            (Type::Tuple(ty1), Type::Tuple(ty2)) => comparer.compare_tuple_type(ty1, ty2),
            (Type::Func(ty1), Type::Func(ty2)) => comparer.compare_func_type(ty1, ty2),
            (Type::Mapping(ty1), Type::Mapping(ty2)) => comparer.compare_mapping_type(ty1, ty2),
            (Type::UserDefined(ty1), Type::UserDefined(ty2)) => {
                comparer.compare_type_name(ty1, ty2)
            }
            (Type::Contract(ty1), Type::Contract(ty2)) => comparer.compare_contract_type(ty1, ty2),
            (Type::Magic(ty1), Type::Magic(ty2)) => comparer.compare_magic_type(ty1, ty2),
            _ => error("Different types", t1, t2, None),
        }
    }

    pub fn compare_int_type<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        t1: &'a IntType,
        t2: &'a IntType,
    ) -> Result<()> {
        if t1.is_signed != t2.is_signed {
            return error("Different int types", t1, t2, None);
        }
        match (t1.bitwidth, t2.bitwidth) {
            (None, None) => Ok(()),
            (None, Some(256)) => Ok(()),
            (Some(256), None) => Ok(()),
            (Some(bw1), Some(bw2)) if bw1 == bw2 => Ok(()),
            _ => error("Different int types", t1, t2, None),
        }
    }

    pub fn compare_fixed_type<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        t1: &'a FixedType,
        t2: &'a FixedType,
    ) -> Result<()> {
        if t1 != t2 {
            return error("Different fixed types", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_string_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a StringType,
        t2: &'a StringType,
    ) -> Result<()> {
        if comparer
            .compare_data_loc(&t1.data_loc, &t2.data_loc)
            .is_err()
            || t1.is_ptr != t2.is_ptr
        {
            return error("Different string type", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_address_type<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        t1: &'a AddressType,
        t2: &'a AddressType,
    ) -> Result<()> {
        if t1 != t2 {
            return error("Different address types", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_bytes_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a BytesType,
        t2: &'a BytesType,
    ) -> Result<()> {
        if comparer
            .compare_data_loc(&t1.data_loc, &t2.data_loc)
            .is_err()
            || t1.is_ptr != t2.is_ptr
            || t1.length != t2.length
        {
            return error("Different bytes types", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_array_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a ArrayType,
        t2: &'a ArrayType,
    ) -> Result<()> {
        if comparer
            .compare_data_loc(&t1.data_loc, &t2.data_loc)
            .is_err()
            || comparer.compare_type(&t1.base, &t2.base).is_err()
            || t1.is_ptr != t2.is_ptr
            || t1.length != t2.length
        {
            return error("Different array types", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_slice_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a SliceType,
        t2: &'a SliceType,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_type(&t1.base, &t2.base) {
            return error("Different slice types", t1, t2, Some(err));
        }
        Ok(())
    }

    pub fn compare_struct_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a StructType,
        t2: &'a StructType,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_data_loc(&t1.data_loc, &t2.data_loc) {
            return error("Different struct type", &t1, &t2, Some(err));
        }
        if let Err(err) = comparer.compare_name(&t1.name, &t2.name) {
            return error("Different struct type", &t1, &t2, Some(err));
        }
        if let Err(err) = comparer.compare_name_opt(&t1.scope, &t2.scope) {
            return error("Different struct type", &t1, &t2, Some(err));
        }
        if t1.is_ptr != t2.is_ptr {
            return error("Different struct pointer type", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_enum_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a EnumType,
        t2: &'a EnumType,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_name(&t1.name, &t2.name) {
            return error("Different enum type", &t1, &t2, Some(err));
        }
        if let Err(err) = comparer.compare_name_opt(&t1.scope, &t2.scope) {
            return error("Different enum type", &t1, &t2, Some(err));
        }
        Ok(())
    }

    pub fn compare_tuple_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a TupleType,
        t2: &'a TupleType,
    ) -> Result<()> {
        if t1.elems.len() != t2.elems.len() {
            return error("Different tuple types", t1, t2, None);
        }
        for (e1, e2) in t1.elems.iter().zip(t2.elems.iter()) {
            match (e1, e2) {
                (Some(e1), Some(e2)) => {
                    if let Err(err) = comparer.compare_type(e1, e2) {
                        return error("Different tuple type", t1, t2, Some(err));
                    }
                }
                (None, None) => {}
                _ => return error("Different tuple type", t1, t2, None),
            };
        }
        Ok(())
    }

    pub fn compare_func_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a FuncType,
        t2: &'a FuncType,
    ) -> Result<()> {
        if t1.params.len() != t2.params.len()
            || t1.returns.len() != t2.returns.len()
            || t1.visibility != t2.visibility
            || t1.mutability != t2.mutability
        {
            return error("Different function type", t1, t2, None);
        }
        for (p1, p2) in t1.params.iter().zip(t2.params.iter()) {
            if let Err(err) = comparer.compare_type(p1, p2) {
                return error("Different function param", p1, p2, Some(err));
            }
        }
        for (r1, r2) in t1.returns.iter().zip(t2.returns.iter()) {
            if let Err(err) = comparer.compare_type(r1, r2) {
                return error("Different function return", r1, r2, Some(err));
            }
        }
        Ok(())
    }

    pub fn compare_mapping_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a MappingType,
        t2: &'a MappingType,
    ) -> Result<()> {
        if let Err(err) = comparer.compare_type(&t1.key, &t2.key) {
            return error("Different mapping key type", &t1.key, &t2.key, Some(err));
        }
        if let Err(err) = comparer.compare_type(&t1.value, &t2.value) {
            return error("Different mapping value type", &t1.value, &t2.value, Some(err));
        }
        Ok(())
    }

    pub fn compare_type_name<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        t1: &'a UserDefinedType,
        t2: &'a UserDefinedType,
    ) -> Result<()> {
        if t1.name != t2.name || t1.scope != t2.scope {
            return error("Different type name", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_contract_type<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        t1: &'a ContractType,
        t2: &'a ContractType,
    ) -> Result<()> {
        if t1.name != t2.name || t1.is_lib != t2.is_lib || t1.scope != t2.scope {
            return error("Different contract type", t1, t2, None);
        }
        Ok(())
    }

    pub fn compare_magic_type<'a, T: Compare<'a> + ?Sized>(
        comparer: &mut T,
        t1: &'a MagicType,
        t2: &'a MagicType,
    ) -> Result<()> {
        match (t1, t2) {
            (MagicType::MetaType(m1), MagicType::MetaType(m2)) => {
                if let Err(err) = comparer.compare_type(m1, m2) {
                    return error("Different magic type", t1, t2, Some(err));
                }
            }
            _ => {
                if t1 != t2 {
                    return error("Different magic type", t1, t2, None);
                }
            }
        };
        Ok(())
    }

    //-------------------------------------------------
    // Data location
    //-------------------------------------------------

    pub fn compare_data_loc<'a, T: Compare<'a> + ?Sized>(
        _comparer: &mut T,
        loc1: &'a DataLoc,
        loc2: &'a DataLoc,
    ) -> Result<()> {
        if loc1 != loc2 {
            return error("Different data location", loc1, loc2, None);
        }
        Ok(())
    }
}
