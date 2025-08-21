//! Module implementing the map pattern for Yulc AST.
//!
//! This map pattern will transform an input data structure into a new data
//! structure of the same type.

use meta::Name;

use crate::ast::*;

/// Trait implementing the map design pattern for Solidity AST.
pub trait Map {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn map_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        default::map_source_unit(self, source_unit)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn map_object(&mut self, object: &Object) -> Object {
        default::map_object(self, object)
    }

    fn map_data(&mut self, data: &Data) -> Data {
        default::map_data(self, data)
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    fn map_code(&mut self, code: &Code) -> Code {
        default::map_code(self, code)
    }

    fn map_block(&mut self, block: &Block) -> Block {
        default::map_block(self, block)
    }

    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        default::map_func_def(self, func)
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn map_stmt(&mut self, stmt: &Stmt) -> Stmt {
        default::map_stmt(self, stmt)
    }

    fn map_assign_stmt(&mut self, stmt: &AssignStmt) -> AssignStmt {
        default::map_assign_stmt(self, stmt)
    }

    fn map_if_stmt(&mut self, stmt: &IfStmt) -> IfStmt {
        default::map_if_stmt(self, stmt)
    }

    fn map_for_stmt(&mut self, stmt: &ForStmt) -> ForStmt {
        default::map_for_stmt(self, stmt)
    }

    fn map_switch_stmt(&mut self, stmt: &SwitchStmt) -> SwitchStmt {
        default::map_switch_stmt(self, stmt)
    }

    fn map_switch_value(&mut self, case: &SwitchValue) -> SwitchValue {
        default::map_switch_value(self, case)
    }

    fn map_switch_default(&mut self, case: &SwitchDefault) -> SwitchDefault {
        default::map_switch_default(self, case)
    }

    fn map_var_decl(&mut self, var: &VarDecl) -> VarDecl {
        default::map_var_decl(self, var)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    fn map_expr(&mut self, expr: &Expr) -> Expr {
        default::map_expr(self, expr)
    }

    fn map_call_expr(&mut self, expr: &CallExpr) -> CallExpr {
        default::map_call_expr(self, expr)
    }

    fn map_member_expr(&mut self, expr: &MemberExpr) -> MemberExpr {
        default::map_member_expr(self, expr)
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn map_ident(&mut self, id: &Identifier) -> Identifier {
        default::map_ident(self, id)
    }

    //-------------------------------------------------
    // Name
    //-------------------------------------------------

    fn map_name(&mut self, name: &Name) -> Name {
        default::map_name(self, name)
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn map_lit(&mut self, c: &Lit) -> Lit {
        default::map_lit(self, c)
    }

    //-------------------------------------------------
    // Type.
    //-------------------------------------------------

    fn map_type(&mut self, typ: &Type) -> Type {
        default::map_type(self, typ)
    }
}

//------------------------------------------------------------------
// Default maping pattern
//------------------------------------------------------------------

/// Module contain default implementation of the maping pattern.
pub mod default {
    use super::Map;
    use crate::ast::*;
    use either::Either;
    use meta::Name;

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    pub fn map_source_unit<T: Map + ?Sized>(
        mapper: &mut T,
        source_unit: &SourceUnit,
    ) -> SourceUnit {
        let nobject = mapper.map_object(&source_unit.main_object);
        SourceUnit::new(nobject)
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    pub fn map_object<T: Map + ?Sized>(mapper: &mut T, object: &Object) -> Object {
        let ncode = mapper.map_code(&object.code);
        let nchildrend = object
            .children
            .iter()
            .map(|child| match child {
                Either::Left(obj) => Either::Left(mapper.map_object(obj)),
                Either::Right(data) => Either::Right(mapper.map_data(data)),
            })
            .collect();
        Object { code: ncode, children: nchildrend, ..object.clone() }
    }

    pub fn map_data<T: Map + ?Sized>(_mapper: &mut T, data: &Data) -> Data {
        data.clone()
    }

    //-------------------------------------------------
    // Code
    //-------------------------------------------------

    pub fn map_code<T: Map + ?Sized>(mapper: &mut T, code: &Code) -> Code {
        let nbody = mapper.map_block(&code.body);
        Code::new(nbody)
    }

    pub fn map_block<T: Map + ?Sized>(mapper: &mut T, block: &Block) -> Block {
        let nbody = block
            .body
            .iter()
            .map(|stmt| mapper.map_stmt(stmt))
            .collect();
        Block::new(nbody)
    }

    pub fn map_func_def<T: Map + ?Sized>(mapper: &mut T, func: &FuncDef) -> FuncDef {
        let nparams = func.params.iter().map(|p| mapper.map_ident(p)).collect();
        let nreturns = func.returns.iter().map(|p| mapper.map_ident(p)).collect();
        let nbody = mapper.map_block(&func.body);
        FuncDef { params: nparams, returns: nreturns, body: nbody, ..func.clone() }
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    pub fn map_stmt<T: Map + ?Sized>(mapper: &mut T, stmt: &Stmt) -> Stmt {
        use Stmt::*;

        match stmt {
            Block(blk) => mapper.map_block(blk).into(),
            FuncDef(func) => mapper.map_func_def(func).into(),
            VarDecl(var) => mapper.map_var_decl(var).into(),
            Assign(stmt) => mapper.map_assign_stmt(stmt).into(),
            If(stmt) => mapper.map_if_stmt(stmt).into(),
            For(stmt) => mapper.map_for_stmt(stmt).into(),
            Switch(stmt) => mapper.map_switch_stmt(stmt).into(),
            Break => stmt.clone(),
            Continue => stmt.clone(),
            Leave => stmt.clone(),
            Expr(expr) => mapper.map_expr(expr).into(),
        }
    }

    pub fn map_assign_stmt<T: Map + ?Sized>(mapper: &mut T, stmt: &AssignStmt) -> AssignStmt {
        let nvars = stmt.vars.iter().map(|id| mapper.map_ident(id)).collect();
        let nvalue = mapper.map_expr(&stmt.value);
        AssignStmt::new(nvars, nvalue)
    }

    pub fn map_if_stmt<T: Map + ?Sized>(mapper: &mut T, stmt: &IfStmt) -> IfStmt {
        let ncond = mapper.map_expr(&stmt.cond);
        let nbody = mapper.map_block(&stmt.body);
        IfStmt::new(ncond, nbody)
    }

    pub fn map_for_stmt<T: Map + ?Sized>(mapper: &mut T, stmt: &ForStmt) -> ForStmt {
        let npre = mapper.map_block(&stmt.pre_loop);
        let ncond = mapper.map_expr(&stmt.condition);
        let npost = mapper.map_block(&stmt.post_loop);
        let nblock = mapper.map_block(&stmt.body);
        ForStmt::new(npre, ncond, npost, nblock)
    }

    pub fn map_switch_stmt<T: Map + ?Sized>(mapper: &mut T, stmt: &SwitchStmt) -> SwitchStmt {
        let nexpr = mapper.map_expr(&stmt.expr);
        let nvalues = stmt
            .values
            .iter()
            .map(|case| mapper.map_switch_value(case))
            .collect();
        let ndefault = stmt
            .default
            .as_ref()
            .map(|case| mapper.map_switch_default(case));
        SwitchStmt::new(nexpr, nvalues, ndefault)
    }

    pub fn map_switch_value<T: Map + ?Sized>(mapper: &mut T, case: &SwitchValue) -> SwitchValue {
        let nliteral = mapper.map_lit(&case.literal);
        let nbody = mapper.map_block(&case.body);
        SwitchValue::new(nliteral, nbody)
    }

    pub fn map_switch_default<T: Map + ?Sized>(
        mapper: &mut T,
        case: &SwitchDefault,
    ) -> SwitchDefault {
        let nbody = mapper.map_block(&case.body);
        SwitchDefault::new(nbody)
    }

    pub fn map_var_decl<T: Map + ?Sized>(mapper: &mut T, vdecl: &VarDecl) -> VarDecl {
        // Map the assigned value first.
        let nvalue = vdecl.value.as_ref().map(|expr| mapper.map_expr(expr));
        let nvdecl = vdecl.vars.iter().map(|id| mapper.map_ident(id)).collect();
        VarDecl::new(nvdecl, nvalue)
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    pub fn map_expr<T: Map + ?Sized>(mapper: &mut T, expr: &Expr) -> Expr {
        use Expr::*;
        match expr {
            Lit(lit) => mapper.map_lit(lit).into(),
            Ident(id) => mapper.map_ident(id).into(),
            Call(expr) => mapper.map_call_expr(expr).into(),
            Member(expr) => mapper.map_member_expr(expr).into(),
        }
    }

    pub fn map_call_expr<T: Map + ?Sized>(mapper: &mut T, expr: &CallExpr) -> CallExpr {
        let ncallee = mapper.map_ident(&expr.callee);
        let nargs = expr.args.iter().map(|arg| mapper.map_expr(arg)).collect();
        CallExpr::new(ncallee, nargs)
    }

    pub fn map_member_expr<T: Map + ?Sized>(mapper: &mut T, expr: &MemberExpr) -> MemberExpr {
        let nbase = mapper.map_name(&expr.base);
        let nmember = mapper.map_name(&expr.member);
        MemberExpr { base: nbase, member: nmember, ..expr.clone() }
    }

    //-------------------------------------------------
    // Identifier.
    //-------------------------------------------------

    pub fn map_ident<T: Map + ?Sized>(mapper: &mut T, id: &Identifier) -> Identifier {
        let nname = mapper.map_name(&id.name);
        let ntyp = mapper.map_type(&id.typ);
        Identifier { name: nname, typ: ntyp, ..id.clone() }
    }

    //-------------------------------------------------
    // Name.
    //-------------------------------------------------

    pub fn map_name<T: Map + ?Sized>(_mapper: &mut T, name: &Name) -> Name {
        name.clone()
    }

    //-------------------------------------------------
    // Literal.
    //-------------------------------------------------

    pub fn map_lit<T: Map + ?Sized>(_mapper: &mut T, c: &Lit) -> Lit {
        c.clone()
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    pub fn map_type<T: Map + ?Sized>(_mapper: &mut T, typ: &Type) -> Type {
        typ.clone()
    }
}
