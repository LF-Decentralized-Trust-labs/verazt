//! Module implementing the map pattern for Yul AST.
//!
//! This map pattern will transform an input data structure into a new data
//! structure of the same type.

use crate::ast::Name;

use crate::ast::yul::*;

/// Trait implementing the map design pattern for Yul AST.
pub trait YulMap {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn map_yul_source_unit(&mut self, source_unit: &YulSourceUnit) -> YulSourceUnit {
        yul_map_default::map_yul_source_unit(self, source_unit)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn map_yul_object(&mut self, object: &YulObject) -> YulObject {
        yul_map_default::map_yul_object(self, object)
    }

    fn map_yul_data(&mut self, data: &YulData) -> YulData {
        yul_map_default::map_yul_data(self, data)
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    fn map_yul_code(&mut self, code: &YulCode) -> YulCode {
        yul_map_default::map_yul_code(self, code)
    }

    fn map_yul_block(&mut self, block: &YulBlock) -> YulBlock {
        yul_map_default::map_yul_block(self, block)
    }

    fn map_yul_func_def(&mut self, func: &YulFuncDef) -> YulFuncDef {
        yul_map_default::map_yul_func_def(self, func)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn map_yul_stmt(&mut self, stmt: &YulStmt) -> YulStmt {
        yul_map_default::map_yul_stmt(self, stmt)
    }

    fn map_yul_assign_stmt(&mut self, stmt: &YulAssignStmt) -> YulAssignStmt {
        yul_map_default::map_yul_assign_stmt(self, stmt)
    }

    fn map_yul_if_stmt(&mut self, stmt: &YulIfStmt) -> YulIfStmt {
        yul_map_default::map_yul_if_stmt(self, stmt)
    }

    fn map_yul_for_stmt(&mut self, stmt: &YulForStmt) -> YulForStmt {
        yul_map_default::map_yul_for_stmt(self, stmt)
    }

    fn map_yul_switch_stmt(&mut self, stmt: &YulSwitchStmt) -> YulSwitchStmt {
        yul_map_default::map_yul_switch_stmt(self, stmt)
    }

    fn map_yul_switch_value(&mut self, case: &YulSwitchValue) -> YulSwitchValue {
        yul_map_default::map_yul_switch_value(self, case)
    }

    fn map_yul_switch_default(&mut self, case: &YulSwitchDefault) -> YulSwitchDefault {
        yul_map_default::map_yul_switch_default(self, case)
    }

    fn map_yul_var_decl(&mut self, var: &YulVarDecl) -> YulVarDecl {
        yul_map_default::map_yul_var_decl(self, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn map_yul_expr(&mut self, expr: &YulExpr) -> YulExpr {
        yul_map_default::map_yul_expr(self, expr)
    }

    fn map_yul_call_expr(&mut self, expr: &YulCallExpr) -> YulCallExpr {
        yul_map_default::map_yul_call_expr(self, expr)
    }

    fn map_yul_member_expr(&mut self, expr: &YulMemberExpr) -> YulMemberExpr {
        yul_map_default::map_yul_member_expr(self, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn map_yul_ident(&mut self, id: &YulIdentifier) -> YulIdentifier {
        yul_map_default::map_yul_ident(self, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn map_yul_name(&mut self, name: &Name) -> Name {
        yul_map_default::map_yul_name(self, name)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn map_yul_lit(&mut self, c: &YulLit) -> YulLit {
        yul_map_default::map_yul_lit(self, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn map_yul_type(&mut self, typ: &YulType) -> YulType {
        yul_map_default::map_yul_type(self, typ)
    }
}

//------------------------------------------------------------------
// Default mapping pattern for Yul AST
//------------------------------------------------------------------

/// Module containing default implementation of the mapping pattern for Yul AST.
pub mod yul_map_default {
    use super::YulMap;
    use crate::ast::Name;
    use crate::ast::yul::*;
    use either::Either;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn map_yul_source_unit<T: YulMap + ?Sized>(
        mapper: &mut T,
        source_unit: &YulSourceUnit,
    ) -> YulSourceUnit {
        let nobject = mapper.map_yul_object(&source_unit.main_object);
        YulSourceUnit::new(nobject)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    pub fn map_yul_object<T: YulMap + ?Sized>(mapper: &mut T, object: &YulObject) -> YulObject {
        let ncode = mapper.map_yul_code(&object.code);
        let nchildrend = object
            .children
            .iter()
            .map(|child| match child {
                Either::Left(obj) => Either::Left(mapper.map_yul_object(obj)),
                Either::Right(data) => Either::Right(mapper.map_yul_data(data)),
            })
            .collect();
        YulObject { code: ncode, children: nchildrend, ..object.clone() }
    }

    pub fn map_yul_data<T: YulMap + ?Sized>(_mapper: &mut T, data: &YulData) -> YulData {
        data.clone()
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    pub fn map_yul_code<T: YulMap + ?Sized>(mapper: &mut T, code: &YulCode) -> YulCode {
        let nbody = mapper.map_yul_block(&code.body);
        YulCode::new(nbody)
    }

    pub fn map_yul_block<T: YulMap + ?Sized>(mapper: &mut T, block: &YulBlock) -> YulBlock {
        let nbody = block
            .body
            .iter()
            .map(|stmt| mapper.map_yul_stmt(stmt))
            .collect();
        YulBlock::new(nbody)
    }

    pub fn map_yul_func_def<T: YulMap + ?Sized>(mapper: &mut T, func: &YulFuncDef) -> YulFuncDef {
        let nparams = func
            .params
            .iter()
            .map(|p| mapper.map_yul_ident(p))
            .collect();
        let nreturns = func
            .returns
            .iter()
            .map(|p| mapper.map_yul_ident(p))
            .collect();
        let nbody = mapper.map_yul_block(&func.body);
        YulFuncDef { params: nparams, returns: nreturns, body: nbody, ..func.clone() }
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn map_yul_stmt<T: YulMap + ?Sized>(mapper: &mut T, stmt: &YulStmt) -> YulStmt {
        use YulStmt::*;

        match stmt {
            Block(blk) => mapper.map_yul_block(blk).into(),
            FuncDef(func) => mapper.map_yul_func_def(func).into(),
            VarDecl(var) => mapper.map_yul_var_decl(var).into(),
            Assign(stmt) => mapper.map_yul_assign_stmt(stmt).into(),
            If(stmt) => mapper.map_yul_if_stmt(stmt).into(),
            For(stmt) => mapper.map_yul_for_stmt(stmt).into(),
            Switch(stmt) => mapper.map_yul_switch_stmt(stmt).into(),
            Break => stmt.clone(),
            Continue => stmt.clone(),
            Leave => stmt.clone(),
            Expr(expr) => mapper.map_yul_expr(expr).into(),
        }
    }

    pub fn map_yul_assign_stmt<T: YulMap + ?Sized>(
        mapper: &mut T,
        stmt: &YulAssignStmt,
    ) -> YulAssignStmt {
        let nvars = stmt
            .vars
            .iter()
            .map(|id| mapper.map_yul_ident(id))
            .collect();
        let nvalue = mapper.map_yul_expr(&stmt.value);
        YulAssignStmt::new(nvars, nvalue)
    }

    pub fn map_yul_if_stmt<T: YulMap + ?Sized>(mapper: &mut T, stmt: &YulIfStmt) -> YulIfStmt {
        let ncond = mapper.map_yul_expr(&stmt.cond);
        let nbody = mapper.map_yul_block(&stmt.body);
        YulIfStmt::new(ncond, nbody)
    }

    pub fn map_yul_for_stmt<T: YulMap + ?Sized>(mapper: &mut T, stmt: &YulForStmt) -> YulForStmt {
        let npre = mapper.map_yul_block(&stmt.pre_loop);
        let ncond = mapper.map_yul_expr(&stmt.condition);
        let npost = mapper.map_yul_block(&stmt.post_loop);
        let nblock = mapper.map_yul_block(&stmt.body);
        YulForStmt::new(npre, ncond, npost, nblock)
    }

    pub fn map_yul_switch_stmt<T: YulMap + ?Sized>(
        mapper: &mut T,
        stmt: &YulSwitchStmt,
    ) -> YulSwitchStmt {
        let nexpr = mapper.map_yul_expr(&stmt.expr);
        let nvalues = stmt
            .values
            .iter()
            .map(|case| mapper.map_yul_switch_value(case))
            .collect();
        let ndefault = stmt
            .default
            .as_ref()
            .map(|case| mapper.map_yul_switch_default(case));
        YulSwitchStmt::new(nexpr, nvalues, ndefault)
    }

    pub fn map_yul_switch_value<T: YulMap + ?Sized>(
        mapper: &mut T,
        case: &YulSwitchValue,
    ) -> YulSwitchValue {
        let nliteral = mapper.map_yul_lit(&case.literal);
        let nbody = mapper.map_yul_block(&case.body);
        YulSwitchValue::new(nliteral, nbody)
    }

    pub fn map_yul_switch_default<T: YulMap + ?Sized>(
        mapper: &mut T,
        case: &YulSwitchDefault,
    ) -> YulSwitchDefault {
        let nbody = mapper.map_yul_block(&case.body);
        YulSwitchDefault::new(nbody)
    }

    pub fn map_yul_var_decl<T: YulMap + ?Sized>(mapper: &mut T, vdecl: &YulVarDecl) -> YulVarDecl {
        // Map the assigned value first.
        let nvalue = vdecl.value.as_ref().map(|expr| mapper.map_yul_expr(expr));
        let nvdecl = vdecl
            .vars
            .iter()
            .map(|id| mapper.map_yul_ident(id))
            .collect();
        YulVarDecl::new(nvdecl, nvalue)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn map_yul_expr<T: YulMap + ?Sized>(mapper: &mut T, expr: &YulExpr) -> YulExpr {
        use YulExpr::*;
        match expr {
            Lit(lit) => mapper.map_yul_lit(lit).into(),
            Ident(id) => mapper.map_yul_ident(id).into(),
            Call(expr) => mapper.map_yul_call_expr(expr).into(),
            Member(expr) => mapper.map_yul_member_expr(expr).into(),
        }
    }

    pub fn map_yul_call_expr<T: YulMap + ?Sized>(
        mapper: &mut T,
        expr: &YulCallExpr,
    ) -> YulCallExpr {
        let ncallee = mapper.map_yul_ident(&expr.callee);
        let nargs = expr
            .args
            .iter()
            .map(|arg| mapper.map_yul_expr(arg))
            .collect();
        YulCallExpr::new(ncallee, nargs)
    }

    pub fn map_yul_member_expr<T: YulMap + ?Sized>(
        mapper: &mut T,
        expr: &YulMemberExpr,
    ) -> YulMemberExpr {
        let nbase = mapper.map_yul_name(&expr.base);
        let nmember = mapper.map_yul_name(&expr.member);
        YulMemberExpr { base: nbase, member: nmember, ..expr.clone() }
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn map_yul_ident<T: YulMap + ?Sized>(mapper: &mut T, id: &YulIdentifier) -> YulIdentifier {
        let nname = mapper.map_yul_name(&id.name);
        let ntyp = mapper.map_yul_type(&id.typ);
        YulIdentifier { name: nname, typ: ntyp, ..id.clone() }
    }

    //-------------------------------------------------
    // Name.
    //-------------------------------------------------

    pub fn map_yul_name<T: YulMap + ?Sized>(_mapper: &mut T, name: &Name) -> Name {
        name.clone()
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn map_yul_lit<T: YulMap + ?Sized>(_mapper: &mut T, c: &YulLit) -> YulLit {
        c.clone()
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn map_yul_type<T: YulMap + ?Sized>(_mapper: &mut T, typ: &YulType) -> YulType {
        typ.clone()
    }
}
