//! Visit pattern for CIR — read-only traversal.

use crate::cir::*;

/// Trait implementing the visit design pattern for CIR.
pub trait Visit<'a> {
    // ── Module ──────────────────────────────────────
    fn visit_module(&mut self, module: &'a CanonModule) {
        default::visit_module(self, module)
    }
    fn visit_decl(&mut self, decl: &'a CanonDecl) {
        default::visit_decl(self, decl)
    }

    // ── Contract ────────────────────────────────────
    fn visit_contract_decl(&mut self, contract: &'a CanonContractDecl) {
        default::visit_contract_decl(self, contract)
    }
    fn visit_member_decl(&mut self, member: &'a CanonMemberDecl) {
        default::visit_member_decl(self, member)
    }

    // ── Storage & Function ──────────────────────────
    fn visit_storage_decl(&mut self, storage: &'a CanonStorageDecl) {
        default::visit_storage_decl(self, storage)
    }
    fn visit_function_decl(&mut self, func: &'a CanonFunctionDecl) {
        default::visit_function_decl(self, func)
    }

    // ── Statements ──────────────────────────────────
    fn visit_stmts(&mut self, stmts: &'a [CanonStmt]) {
        default::visit_stmts(self, stmts)
    }
    fn visit_stmt(&mut self, stmt: &'a CanonStmt) {
        default::visit_stmt(self, stmt)
    }
    fn visit_local_var_stmt(&mut self, stmt: &'a CanonLocalVarStmt) {
        default::visit_local_var_stmt(self, stmt)
    }
    fn visit_assign_stmt(&mut self, stmt: &'a CanonAssignStmt) {
        default::visit_assign_stmt(self, stmt)
    }
    fn visit_aug_assign_stmt(&mut self, stmt: &'a CanonAugAssignStmt) {
        default::visit_aug_assign_stmt(self, stmt)
    }
    fn visit_expr_stmt(&mut self, stmt: &'a CanonExprStmt) {
        default::visit_expr_stmt(self, stmt)
    }
    fn visit_if_stmt(&mut self, stmt: &'a CanonIfStmt) {
        default::visit_if_stmt(self, stmt)
    }
    fn visit_while_stmt(&mut self, stmt: &'a CanonWhileStmt) {
        default::visit_while_stmt(self, stmt)
    }
    fn visit_for_stmt(&mut self, stmt: &'a CanonForStmt) {
        default::visit_for_stmt(self, stmt)
    }
    fn visit_return_stmt(&mut self, stmt: &'a CanonReturnStmt) {
        default::visit_return_stmt(self, stmt)
    }
    fn visit_revert_stmt(&mut self, stmt: &'a CanonRevertStmt) {
        default::visit_revert_stmt(self, stmt)
    }
    fn visit_assert_stmt(&mut self, stmt: &'a CanonAssertStmt) {
        default::visit_assert_stmt(self, stmt)
    }

    // ── Expressions ─────────────────────────────────
    fn visit_expr(&mut self, expr: &'a CanonExpr) {
        default::visit_expr(self, expr)
    }
    fn visit_var_expr(&mut self, _var: &'a CanonVarExpr) {}
    fn visit_binop_expr(&mut self, expr: &'a CanonBinOpExpr) {
        default::visit_binop_expr(self, expr)
    }
    fn visit_unop_expr(&mut self, expr: &'a CanonUnOpExpr) {
        default::visit_unop_expr(self, expr)
    }
    fn visit_index_access_expr(&mut self, expr: &'a CanonIndexAccessExpr) {
        default::visit_index_access_expr(self, expr)
    }
    fn visit_field_access_expr(&mut self, expr: &'a CanonFieldAccessExpr) {
        default::visit_field_access_expr(self, expr)
    }
    fn visit_call_expr(&mut self, expr: &'a CanonCallExpr) {
        default::visit_call_expr(self, expr)
    }
    fn visit_type_cast_expr(&mut self, expr: &'a CanonTypeCastExpr) {
        default::visit_type_cast_expr(self, expr)
    }
    fn visit_load_expr(&mut self, expr: &'a CanonLoadExpr) {
        default::visit_load_expr(self, expr)
    }
    fn visit_internal_call_expr(&mut self, expr: &'a CanonInternalCallExpr) {
        default::visit_internal_call_expr(self, expr)
    }
    fn visit_external_call_expr(&mut self, expr: &'a CanonExternalCallExpr) {
        default::visit_external_call_expr(self, expr)
    }
    fn visit_env_expr(&mut self, _expr: &'a CanonEnvExpr) {}

    // ── Shared chain statements ─────────────────────
    fn visit_store_stmt(&mut self, stmt: &'a CanonStoreStmt) {
        default::visit_store_stmt(self, stmt)
    }
    fn visit_emit_stmt(&mut self, stmt: &'a CanonEmitStmt) {
        default::visit_emit_stmt(self, stmt)
    }

    // ── Dialect ─────────────────────────────────────
    fn visit_dialect_expr(&mut self, expr: &'a CanonDialectExpr) {
        default::visit_dialect_expr(self, expr)
    }
    fn visit_dialect_stmt(&mut self, stmt: &'a CanonDialectStmt) {
        default::visit_dialect_stmt(self, stmt)
    }
    fn visit_dialect_member_decl(&mut self, _decl: &'a DialectMemberDecl) {}
}

/// Default implementations for the CIR Visit trait.
pub mod default {
    use super::Visit;
    use crate::cir::*;

    pub fn visit_module<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, module: &'a CanonModule) {
        for d in &module.decls {
            visitor.visit_decl(d)
        }
    }

    pub fn visit_decl<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, decl: &'a CanonDecl) {
        match decl {
            CanonDecl::Contract(c) => visitor.visit_contract_decl(c),
            CanonDecl::Dialect(_) => {}
        }
    }

    pub fn visit_contract_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        contract: &'a CanonContractDecl,
    ) {
        for m in &contract.members {
            visitor.visit_member_decl(m)
        }
    }

    pub fn visit_member_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        member: &'a CanonMemberDecl,
    ) {
        match member {
            CanonMemberDecl::Storage(s) => visitor.visit_storage_decl(s),
            CanonMemberDecl::Function(f) => visitor.visit_function_decl(f),
            CanonMemberDecl::TypeAlias(_) => {}
            CanonMemberDecl::GlobalInvariant(inv) => visitor.visit_expr(inv),
            CanonMemberDecl::Dialect(d) => visitor.visit_dialect_member_decl(d),
        }
    }

    pub fn visit_storage_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        storage: &'a CanonStorageDecl,
    ) {
        if let Some(init) = &storage.init {
            visitor.visit_expr(init);
        }
    }

    pub fn visit_function_decl<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        func: &'a CanonFunctionDecl,
    ) {
        visitor.visit_stmts(&func.body);
    }

    // ── Statements ─────────────────────────────────────────────

    pub fn visit_stmts<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmts: &'a [CanonStmt]) {
        for s in stmts {
            visitor.visit_stmt(s)
        }
    }

    pub fn visit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonStmt) {
        match stmt {
            CanonStmt::LocalVar(s) => visitor.visit_local_var_stmt(s),
            CanonStmt::Assign(s) => visitor.visit_assign_stmt(s),
            CanonStmt::AugAssign(s) => visitor.visit_aug_assign_stmt(s),
            CanonStmt::Expr(s) => visitor.visit_expr_stmt(s),
            CanonStmt::If(s) => visitor.visit_if_stmt(s),
            CanonStmt::While(s) => visitor.visit_while_stmt(s),
            CanonStmt::For(s) => visitor.visit_for_stmt(s),
            CanonStmt::Return(s) => visitor.visit_return_stmt(s),
            CanonStmt::Revert(s) => visitor.visit_revert_stmt(s),
            CanonStmt::Assert(s) => visitor.visit_assert_stmt(s),
            CanonStmt::Break | CanonStmt::Continue => {}
            CanonStmt::Block(stmts) => visitor.visit_stmts(stmts),
            CanonStmt::Store(s) => visitor.visit_store_stmt(s),
            CanonStmt::Emit(s) => visitor.visit_emit_stmt(s),
            CanonStmt::Dialect(s) => visitor.visit_dialect_stmt(s),
        }
    }

    pub fn visit_store_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonStoreStmt) {
        visit_resource(visitor, &stmt.resource);
        for key in &stmt.keys {
            visitor.visit_expr(key);
        }
        if let Some(value) = &stmt.value {
            visitor.visit_expr(value);
        }
    }

    pub fn visit_emit_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonEmitStmt) {
        for arg in &stmt.args {
            visitor.visit_expr(arg);
        }
    }

    pub fn visit_dialect_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonDialectStmt,
    ) {
        match stmt {
            CanonDialectStmt::Evm(CanonEvmStmt::Selfdestruct(s)) => visitor.visit_expr(&s.recipient),
            CanonDialectStmt::Evm(CanonEvmStmt::TryCatch(s)) => {
                visitor.visit_expr(&s.guarded);
                visitor.visit_stmts(&s.body);
                for clause in &s.catch_clauses {
                    visitor.visit_stmts(&clause.body);
                }
            }
            CanonDialectStmt::Move(CanonMoveStmt::Abort(s)) => visitor.visit_expr(&s.code),
            CanonDialectStmt::Move(CanonMoveStmt::SpecBlock(s)) => {
                for assertion in &s.assertions {
                    visitor.visit_expr(assertion);
                }
            }
        }
    }

    pub fn visit_local_var_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonLocalVarStmt,
    ) {
        if let Some(init) = &stmt.init {
            visitor.visit_expr(init)
        }
    }

    pub fn visit_assign_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonAssignStmt,
    ) {
        visitor.visit_expr(&stmt.lhs);
        visitor.visit_expr(&stmt.rhs);
    }

    pub fn visit_aug_assign_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonAugAssignStmt,
    ) {
        visitor.visit_expr(&stmt.lhs);
        visitor.visit_expr(&stmt.rhs);
    }

    pub fn visit_expr_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonExprStmt) {
        visitor.visit_expr(&stmt.expr);
    }

    pub fn visit_if_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonIfStmt) {
        visitor.visit_expr(&stmt.cond);
        visitor.visit_stmts(&stmt.then_body);
        if let Some(else_body) = &stmt.else_body {
            visitor.visit_stmts(else_body);
        }
    }

    pub fn visit_while_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonWhileStmt) {
        visitor.visit_expr(&stmt.cond);
        visitor.visit_stmts(&stmt.body);
        if let Some(inv) = &stmt.invariant {
            visitor.visit_expr(inv);
        }
    }

    pub fn visit_for_stmt<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, stmt: &'a CanonForStmt) {
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

    pub fn visit_return_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonReturnStmt,
    ) {
        if let Some(v) = &stmt.value {
            visitor.visit_expr(v);
        }
    }

    pub fn visit_revert_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonRevertStmt,
    ) {
        for arg in &stmt.args {
            visitor.visit_expr(arg);
        }
    }

    pub fn visit_assert_stmt<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        stmt: &'a CanonAssertStmt,
    ) {
        visitor.visit_expr(&stmt.cond);
        if let Some(msg) = &stmt.message {
            visitor.visit_expr(msg);
        }
    }

    // ── Expressions ────────────────────────────────────────────

    pub fn visit_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CanonExpr) {
        match expr {
            CanonExpr::Var(v) => visitor.visit_var_expr(v),
            CanonExpr::Lit(_) => {}
            CanonExpr::BinOp(e) => visitor.visit_binop_expr(e),
            CanonExpr::UnOp(e) => visitor.visit_unop_expr(e),
            CanonExpr::IndexAccess(e) => visitor.visit_index_access_expr(e),
            CanonExpr::FieldAccess(e) => visitor.visit_field_access_expr(e),
            CanonExpr::FunctionCall(e) => visitor.visit_call_expr(e),
            CanonExpr::TypeCast(e) => visitor.visit_type_cast_expr(e),
            CanonExpr::Load(e) => visitor.visit_load_expr(e),
            CanonExpr::InternalCall(e) => visitor.visit_internal_call_expr(e),
            CanonExpr::ExternalCall(e) => visitor.visit_external_call_expr(e),
            CanonExpr::Env(e) => visitor.visit_env_expr(e),
            CanonExpr::Old(inner) => visitor.visit_expr(inner),
            CanonExpr::Result(_) => {}
            CanonExpr::Forall { body, .. } => visitor.visit_expr(body),
            CanonExpr::Exists { body, .. } => visitor.visit_expr(body),
            CanonExpr::Dialect(d) => visitor.visit_dialect_expr(d),
        }
    }

    pub fn visit_binop_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CanonBinOpExpr) {
        visitor.visit_expr(&expr.lhs);
        visitor.visit_expr(&expr.rhs);
    }

    pub fn visit_unop_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CanonUnOpExpr) {
        visitor.visit_expr(&expr.operand);
    }

    pub fn visit_index_access_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CanonIndexAccessExpr,
    ) {
        visitor.visit_expr(&expr.base);
        if let Some(idx) = &expr.index {
            visitor.visit_expr(idx);
        }
    }

    pub fn visit_field_access_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CanonFieldAccessExpr,
    ) {
        visitor.visit_expr(&expr.base);
    }

    pub fn visit_call_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CanonCallExpr) {
        visitor.visit_expr(&expr.callee);
        for arg in &expr.args {
            visitor.visit_expr(arg);
        }
    }

    pub fn visit_type_cast_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CanonTypeCastExpr,
    ) {
        visitor.visit_expr(&expr.expr);
    }

    pub fn visit_load_expr<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, expr: &'a CanonLoadExpr) {
        visit_resource(visitor, &expr.resource);
        for key in &expr.keys {
            visitor.visit_expr(key);
        }
    }

    pub fn visit_internal_call_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CanonInternalCallExpr,
    ) {
        for arg in &expr.args {
            visitor.visit_expr(arg);
        }
    }

    pub fn visit_external_call_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CanonExternalCallExpr,
    ) {
        if let Some(address) = &expr.address {
            visitor.visit_expr(address);
        }
        for arg in &expr.args {
            visitor.visit_expr(arg);
        }
        if let Some(value) = &expr.value {
            visitor.visit_expr(value);
        }
    }

    pub fn visit_dialect_expr<'a, T: Visit<'a> + ?Sized>(
        visitor: &mut T,
        expr: &'a CanonDialectExpr,
    ) {
        for operand in expr.operands() {
            visitor.visit_expr(operand);
        }
    }

    fn visit_resource<'a, T: Visit<'a> + ?Sized>(visitor: &mut T, resource: &'a CanonResource) {
        match resource {
            CanonResource::AnchorAccount(account) => visitor.visit_expr(account),
            CanonResource::MoveGlobal(_) | CanonResource::StateVar(_) => {}
        }
    }
}
