//! Visit pattern for SIR — read-only traversal.

use crate::sir::*;

/// Trait implementing the visit design pattern for SIR.
pub trait Visit<'a> {
    // ── Module ──────────────────────────────────────
    fn visit_modules(&mut self, modules: &'a [Module]) {
        default::visit_modules(self, modules)
    }
    fn visit_module(&mut self, module: &'a Module) {
        default::visit_module(self, module)
    }
    fn visit_decl(&mut self, decl: &'a Decl) {
        default::visit_decl(self, decl)
    }

    // ── Contract ────────────────────────────────────
    fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
        default::visit_contract_decl(self, contract)
    }
    fn visit_member_decl(&mut self, member: &'a MemberDecl) {
        default::visit_member_decl(self, member)
    }

    // ── Storage & Function ──────────────────────────
    fn visit_storage_decl(&mut self, storage: &'a StorageDecl) {
        default::visit_storage_decl(self, storage)
    }
    fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
        default::visit_function_decl(self, func)
    }
    fn visit_param(&mut self, param: &'a Param) {
        default::visit_param(self, param)
    }
    fn visit_func_spec(&mut self, spec: &'a FuncSpec) {
        default::visit_func_spec(self, spec)
    }

    // ── Statements ──────────────────────────────────
    fn visit_stmts(&mut self, stmts: &'a [Stmt]) {
        default::visit_stmts(self, stmts)
    }
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        default::visit_stmt(self, stmt)
    }
    fn visit_local_var_stmt(&mut self, stmt: &'a LocalVarStmt) {
        default::visit_local_var_stmt(self, stmt)
    }
    fn visit_assign_stmt(&mut self, stmt: &'a AssignStmt) {
        default::visit_assign_stmt(self, stmt)
    }
    fn visit_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) {
        default::visit_aug_assign_stmt(self, stmt)
    }
    fn visit_expr_stmt(&mut self, stmt: &'a ExprStmt) {
        default::visit_expr_stmt(self, stmt)
    }
    fn visit_if_stmt(&mut self, stmt: &'a IfStmt) {
        default::visit_if_stmt(self, stmt)
    }
    fn visit_while_stmt(&mut self, stmt: &'a WhileStmt) {
        default::visit_while_stmt(self, stmt)
    }
    fn visit_for_stmt(&mut self, stmt: &'a ForStmt) {
        default::visit_for_stmt(self, stmt)
    }
    fn visit_return_stmt(&mut self, stmt: &'a ReturnStmt) {
        default::visit_return_stmt(self, stmt)
    }
    fn visit_revert_stmt(&mut self, stmt: &'a RevertStmt) {
        default::visit_revert_stmt(self, stmt)
    }
    fn visit_assert_stmt(&mut self, stmt: &'a AssertStmt) {
        default::visit_assert_stmt(self, stmt)
    }

    // ── Expressions ─────────────────────────────────
    fn visit_expr(&mut self, expr: &'a Expr) {
        default::visit_expr(self, expr)
    }
    fn visit_var_expr(&mut self, var: &'a VarExpr) {
        default::visit_var_expr(self, var)
    }
    fn visit_binop_expr(&mut self, expr: &'a BinOpExpr) {
        default::visit_binop_expr(self, expr)
    }
    fn visit_unop_expr(&mut self, expr: &'a UnOpExpr) {
        default::visit_unop_expr(self, expr)
    }
    fn visit_index_access_expr(&mut self, expr: &'a IndexAccessExpr) {
        default::visit_index_access_expr(self, expr)
    }
    fn visit_field_access_expr(&mut self, expr: &'a FieldAccessExpr) {
        default::visit_field_access_expr(self, expr)
    }
    fn visit_call_expr(&mut self, expr: &'a CallExpr) {
        default::visit_call_expr(self, expr)
    }
    fn visit_type_cast_expr(&mut self, expr: &'a TypeCastExpr) {
        default::visit_type_cast_expr(self, expr)
    }
    fn visit_ternary_expr(&mut self, expr: &'a TernaryExpr) {
        default::visit_ternary_expr(self, expr)
    }
    fn visit_tuple_expr(&mut self, expr: &'a TupleExpr) {
        default::visit_tuple_expr(self, expr)
    }

    // ── Type ────────────────────────────────────────
    fn visit_type(&mut self, ty: &'a Type) {
        default::visit_type(self, ty)
    }

    // ── Dialect ─────────────────────────────────────
    fn visit_dialect_expr(&mut self, _expr: &'a DialectExpr) {}
    fn visit_dialect_stmt(&mut self, _stmt: &'a DialectStmt) {}
    fn visit_dialect_member_decl(&mut self, _decl: &'a DialectMemberDecl) {}
    fn visit_dialect_type(&mut self, _ty: &'a DialectType) {}
}

/// Default implementations for the Visit trait.
pub mod default {
    use super::Visit;
    use crate::sir::*;

    pub fn visit_modules<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, modules: &'a [Module]) {
        for m in modules {
            visitor.visit_module(m)
        }
    }

    pub fn visit_module<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, module: &'a Module) {
        for d in &module.decls {
            visitor.visit_decl(d)
        }
    }

    pub fn visit_decl<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, decl: &'a Decl) {
        match decl {
            Decl::Contract(c) => visitor.visit_contract_decl(c),
            Decl::Dialect(_) => {}
        }
    }

    pub fn visit_contract_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        contract: &'a ContractDecl,
    ) {
        for m in &contract.members {
            visitor.visit_member_decl(m)
        }
    }

    pub fn visit_member_decl<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, member: &'a MemberDecl) {
        match member {
            MemberDecl::Storage(s) => visitor.visit_storage_decl(s),
            MemberDecl::Function(f) => visitor.visit_function_decl(f),
            MemberDecl::TypeAlias(_) => {}
            MemberDecl::GlobalInvariant(inv) => visitor.visit_expr(inv),
            MemberDecl::Dialect(d) => visitor.visit_dialect_member_decl(d),
        }
    }

    pub fn visit_storage_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        storage: &'a StorageDecl,
    ) {
        visitor.visit_type(&storage.ty);
        if let Some(init) = &storage.init {
            visitor.visit_expr(init);
        }
    }

    pub fn visit_function_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        func: &'a FunctionDecl,
    ) {
        for p in &func.params {
            visitor.visit_param(p)
        }
        for r in &func.returns {
            visitor.visit_type(r)
        }
        if let Some(spec) = &func.spec {
            visitor.visit_func_spec(spec)
        }
        if let Some(body) = &func.body {
            visitor.visit_stmts(body)
        }
    }

    pub fn visit_param<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, param: &'a Param) {
        visitor.visit_type(&param.ty)
    }

    pub fn visit_func_spec<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, spec: &'a FuncSpec) {
        for r in &spec.requires {
            visitor.visit_expr(r)
        }
        for e in &spec.ensures {
            visitor.visit_expr(e)
        }
        if let Some(d) = &spec.decreases {
            visitor.visit_expr(d)
        }
    }

    // ── Statements ─────────────────────────────────────────────

    pub fn visit_stmts<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmts: &'a [Stmt]) {
        for s in stmts {
            visitor.visit_stmt(s)
        }
    }

    pub fn visit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a Stmt) {
        match stmt {
            Stmt::LocalVar(s) => visitor.visit_local_var_stmt(s),
            Stmt::Assign(s) => visitor.visit_assign_stmt(s),
            Stmt::AugAssign(s) => visitor.visit_aug_assign_stmt(s),
            Stmt::Expr(s) => visitor.visit_expr_stmt(s),
            Stmt::If(s) => visitor.visit_if_stmt(s),
            Stmt::While(s) => visitor.visit_while_stmt(s),
            Stmt::For(s) => visitor.visit_for_stmt(s),
            Stmt::Return(s) => visitor.visit_return_stmt(s),
            Stmt::Revert(s) => visitor.visit_revert_stmt(s),
            Stmt::Assert(s) => visitor.visit_assert_stmt(s),
            Stmt::Break | Stmt::Continue => {}
            Stmt::Block(stmts) => visitor.visit_stmts(stmts),
            Stmt::Dialect(s) => visitor.visit_dialect_stmt(s),
        }
    }

    pub fn visit_local_var_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a LocalVarStmt,
    ) {
        if let Some(init) = &stmt.init {
            visitor.visit_expr(init)
        }
    }

    pub fn visit_assign_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a AssignStmt) {
        visitor.visit_expr(&stmt.lhs);
        visitor.visit_expr(&stmt.rhs);
    }

    pub fn visit_aug_assign_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a AugAssignStmt,
    ) {
        visitor.visit_expr(&stmt.lhs);
        visitor.visit_expr(&stmt.rhs);
    }

    pub fn visit_expr_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a ExprStmt) {
        visitor.visit_expr(&stmt.expr);
    }

    pub fn visit_if_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a IfStmt) {
        visitor.visit_expr(&stmt.cond);
        visitor.visit_stmts(&stmt.then_body);
        if let Some(else_body) = &stmt.else_body {
            visitor.visit_stmts(else_body);
        }
    }

    pub fn visit_while_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a WhileStmt) {
        visitor.visit_expr(&stmt.cond);
        visitor.visit_stmts(&stmt.body);
        if let Some(inv) = &stmt.invariant {
            visitor.visit_expr(inv);
        }
    }

    pub fn visit_for_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a ForStmt) {
        if let Some(init) = &stmt.init {
            visitor.visit_stmt(init);
        }
        if let Some(cond) = &stmt.cond {
            visitor.visit_expr(cond);
        }
        if let Some(update) = &stmt.update {
            visitor.visit_stmt(update);
        }
        visitor.visit_stmts(&stmt.body);
        if let Some(inv) = &stmt.invariant {
            visitor.visit_expr(inv);
        }
    }

    pub fn visit_return_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a ReturnStmt) {
        if let Some(v) = &stmt.value {
            visitor.visit_expr(v);
        }
    }

    pub fn visit_revert_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a RevertStmt) {
        for arg in &stmt.args {
            visitor.visit_expr(arg);
        }
    }

    pub fn visit_assert_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a AssertStmt) {
        visitor.visit_expr(&stmt.cond);
        if let Some(msg) = &stmt.message {
            visitor.visit_expr(msg);
        }
    }

    // ── Expressions ────────────────────────────────────────────

    pub fn visit_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a Expr) {
        match expr {
            Expr::Var(v) => visitor.visit_var_expr(v),
            Expr::Lit(_) => {}
            Expr::BinOp(e) => visitor.visit_binop_expr(e),
            Expr::UnOp(e) => visitor.visit_unop_expr(e),
            Expr::IndexAccess(e) => visitor.visit_index_access_expr(e),
            Expr::FieldAccess(e) => visitor.visit_field_access_expr(e),
            Expr::FunctionCall(e) => visitor.visit_call_expr(e),
            Expr::TypeCast(e) => visitor.visit_type_cast_expr(e),
            Expr::Ternary(e) => visitor.visit_ternary_expr(e),
            Expr::Tuple(e) => visitor.visit_tuple_expr(e),
            Expr::Old(inner) => visitor.visit_expr(inner),
            Expr::Result(_) => {}
            Expr::Forall { body, .. } => visitor.visit_expr(body),
            Expr::Exists { body, .. } => visitor.visit_expr(body),
            Expr::Dialect(d) => visitor.visit_dialect_expr(d),
        }
    }

    pub fn visit_var_expr<'a, T: Visit<'a> + ?Sized>(_visitor: &mut T, _var: &'a VarExpr) {}

    pub fn visit_binop_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a BinOpExpr) {
        visitor.visit_expr(&expr.lhs);
        visitor.visit_expr(&expr.rhs);
    }

    pub fn visit_unop_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a UnOpExpr) {
        visitor.visit_expr(&expr.operand);
    }

    pub fn visit_index_access_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a IndexAccessExpr,
    ) {
        visitor.visit_expr(&expr.base);
        if let Some(idx) = &expr.index {
            visitor.visit_expr(idx);
        }
    }

    pub fn visit_field_access_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a FieldAccessExpr,
    ) {
        visitor.visit_expr(&expr.base);
    }

    pub fn visit_call_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CallExpr) {
        visitor.visit_expr(&expr.callee);
        for arg in &expr.args {
            visitor.visit_expr(arg);
        }
    }

    pub fn visit_type_cast_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a TypeCastExpr,
    ) {
        visitor.visit_type(&expr.ty);
        visitor.visit_expr(&expr.expr);
    }

    pub fn visit_ternary_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a TernaryExpr) {
        visitor.visit_expr(&expr.cond);
        visitor.visit_expr(&expr.then_expr);
        visitor.visit_expr(&expr.else_expr);
    }

    pub fn visit_tuple_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a TupleExpr) {
        for e in &expr.elems {
            if let Some(ex) = e {
                visitor.visit_expr(ex);
            }
        }
    }

    // ── Type ───────────────────────────────────────────────────

    pub fn visit_type<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, ty: &'a Type) {
        match ty {
            Type::Dialect(dt) => visitor.visit_dialect_type(dt),
            _ => {}
        }
    }
}
