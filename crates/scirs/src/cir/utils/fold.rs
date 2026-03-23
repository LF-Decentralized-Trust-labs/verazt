//! Fold pattern for CIR — accumulating traversal.

use crate::cir::*;

/// Trait implementing the fold design pattern for CIR.
/// Traverses the CIR tree while accumulating a result of type `T`.
pub trait Fold<'a, T: Default> {
    // ── Module ──────────────────────────────────────
    fn fold_module(&mut self, module: &'a CanonModule) -> T {
        default::fold_module(self, module)
    }
    fn fold_decl(&mut self, decl: &'a CanonDecl) -> T {
        default::fold_decl(self, decl)
    }

    // ── Contract ────────────────────────────────────
    fn fold_contract_decl(&mut self, contract: &'a CanonContractDecl) -> T {
        default::fold_contract_decl(self, contract)
    }
    fn fold_member_decl(&mut self, member: &'a CanonMemberDecl) -> T {
        default::fold_member_decl(self, member)
    }

    // ── Storage & Function ──────────────────────────
    fn fold_storage_decl(&mut self, _storage: &'a CanonStorageDecl) -> T {
        T::default()
    }
    fn fold_function_decl(&mut self, func: &'a CanonFunctionDecl) -> T {
        default::fold_function_decl(self, func)
    }

    // ── Statements ──────────────────────────────────
    fn fold_stmts(&mut self, stmts: &'a [CanonStmt]) -> T {
        default::fold_stmts(self, stmts)
    }
    fn fold_stmt(&mut self, stmt: &'a CanonStmt) -> T {
        default::fold_stmt(self, stmt)
    }
    fn fold_local_var_stmt(&mut self, _stmt: &'a CanonLocalVarStmt) -> T {
        T::default()
    }
    fn fold_assign_stmt(&mut self, stmt: &'a CanonAssignStmt) -> T {
        default::fold_assign_stmt(self, stmt)
    }
    fn fold_aug_assign_stmt(&mut self, stmt: &'a CanonAugAssignStmt) -> T {
        default::fold_aug_assign_stmt(self, stmt)
    }
    fn fold_expr_stmt(&mut self, stmt: &'a CanonExprStmt) -> T {
        default::fold_expr_stmt(self, stmt)
    }
    fn fold_if_stmt(&mut self, stmt: &'a CanonIfStmt) -> T {
        default::fold_if_stmt(self, stmt)
    }
    fn fold_while_stmt(&mut self, stmt: &'a CanonWhileStmt) -> T {
        default::fold_while_stmt(self, stmt)
    }
    fn fold_for_stmt(&mut self, stmt: &'a CanonForStmt) -> T {
        default::fold_for_stmt(self, stmt)
    }
    fn fold_return_stmt(&mut self, stmt: &'a CanonReturnStmt) -> T {
        default::fold_return_stmt(self, stmt)
    }
    fn fold_revert_stmt(&mut self, stmt: &'a CanonRevertStmt) -> T {
        default::fold_revert_stmt(self, stmt)
    }
    fn fold_assert_stmt(&mut self, stmt: &'a CanonAssertStmt) -> T {
        default::fold_assert_stmt(self, stmt)
    }

    // ── Expressions ─────────────────────────────────
    fn fold_expr(&mut self, expr: &'a CanonExpr) -> T {
        default::fold_expr(self, expr)
    }
    fn fold_var_expr(&mut self, _var: &'a CanonVarExpr) -> T {
        T::default()
    }
    fn fold_binop_expr(&mut self, expr: &'a CanonBinOpExpr) -> T {
        default::fold_binop_expr(self, expr)
    }
    fn fold_unop_expr(&mut self, expr: &'a CanonUnOpExpr) -> T {
        default::fold_unop_expr(self, expr)
    }
    fn fold_index_access_expr(&mut self, expr: &'a CanonIndexAccessExpr) -> T {
        default::fold_index_access_expr(self, expr)
    }
    fn fold_field_access_expr(&mut self, expr: &'a CanonFieldAccessExpr) -> T {
        default::fold_field_access_expr(self, expr)
    }
    fn fold_call_expr(&mut self, expr: &'a CanonCallExpr) -> T {
        default::fold_call_expr(self, expr)
    }
    fn fold_type_cast_expr(&mut self, expr: &'a CanonTypeCastExpr) -> T {
        default::fold_type_cast_expr(self, expr)
    }

    /// Combine two folded values.
    fn combine(&mut self, a: T, _b: T) -> T {
        a
    }
}

/// Default implementations for the CIR Fold trait.
pub mod default {
    use super::Fold;
    use crate::cir::*;

    pub fn fold_module<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        module: &'a CanonModule,
    ) -> T {
        let mut result = T::default();
        for d in &module.decls {
            let r = folder.fold_decl(d);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_decl<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        decl: &'a CanonDecl,
    ) -> T {
        match decl {
            CanonDecl::Contract(c) => folder.fold_contract_decl(c),
            CanonDecl::Dialect(_) => T::default(),
        }
    }

    pub fn fold_contract_decl<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        contract: &'a CanonContractDecl,
    ) -> T {
        let mut result = T::default();
        for m in &contract.members {
            let r = folder.fold_member_decl(m);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_member_decl<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        member: &'a CanonMemberDecl,
    ) -> T {
        match member {
            CanonMemberDecl::Storage(s) => folder.fold_storage_decl(s),
            CanonMemberDecl::Function(f) => folder.fold_function_decl(f),
            CanonMemberDecl::TypeAlias(_) => T::default(),
            CanonMemberDecl::GlobalInvariant(inv) => folder.fold_expr(inv),
            CanonMemberDecl::Dialect(_) => T::default(),
        }
    }

    pub fn fold_function_decl<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        func: &'a CanonFunctionDecl,
    ) -> T {
        folder.fold_stmts(&func.body)
    }

    // ── Statements ─────────────────────────────────────────────

    pub fn fold_stmts<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmts: &'a [CanonStmt],
    ) -> T {
        let mut result = T::default();
        for s in stmts {
            let r = folder.fold_stmt(s);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonStmt,
    ) -> T {
        match stmt {
            CanonStmt::LocalVar(s) => folder.fold_local_var_stmt(s),
            CanonStmt::Assign(s) => folder.fold_assign_stmt(s),
            CanonStmt::AugAssign(s) => folder.fold_aug_assign_stmt(s),
            CanonStmt::Expr(s) => folder.fold_expr_stmt(s),
            CanonStmt::If(s) => folder.fold_if_stmt(s),
            CanonStmt::While(s) => folder.fold_while_stmt(s),
            CanonStmt::For(s) => folder.fold_for_stmt(s),
            CanonStmt::Return(s) => folder.fold_return_stmt(s),
            CanonStmt::Revert(s) => folder.fold_revert_stmt(s),
            CanonStmt::Assert(s) => folder.fold_assert_stmt(s),
            CanonStmt::Break | CanonStmt::Continue => T::default(),
            CanonStmt::Block(stmts) => folder.fold_stmts(stmts),
            CanonStmt::Dialect(_) => T::default(),
        }
    }

    pub fn fold_assign_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonAssignStmt,
    ) -> T {
        let a = folder.fold_expr(&stmt.lhs);
        let b = folder.fold_expr(&stmt.rhs);
        folder.combine(a, b)
    }

    pub fn fold_aug_assign_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonAugAssignStmt,
    ) -> T {
        let a = folder.fold_expr(&stmt.lhs);
        let b = folder.fold_expr(&stmt.rhs);
        folder.combine(a, b)
    }

    pub fn fold_expr_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonExprStmt,
    ) -> T {
        folder.fold_expr(&stmt.expr)
    }

    pub fn fold_if_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonIfStmt,
    ) -> T {
        let mut result = folder.fold_expr(&stmt.cond);
        let r = folder.fold_stmts(&stmt.then_body);
        result = folder.combine(result, r);
        if let Some(else_body) = &stmt.else_body {
            let r = folder.fold_stmts(else_body);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_while_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonWhileStmt,
    ) -> T {
        let mut result = folder.fold_expr(&stmt.cond);
        let r = folder.fold_stmts(&stmt.body);
        result = folder.combine(result, r);
        result
    }

    pub fn fold_for_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonForStmt,
    ) -> T {
        let mut result = T::default();
        if let Some(init) = &stmt.init {
            let r = folder.fold_stmt(init);
            result = folder.combine(result, r);
        }
        if let Some(cond) = &stmt.cond {
            let r = folder.fold_expr(cond);
            result = folder.combine(result, r);
        }
        if let Some(update) = &stmt.update {
            let r = folder.fold_stmt(update);
            result = folder.combine(result, r);
        }
        let r = folder.fold_stmts(&stmt.body);
        folder.combine(result, r)
    }

    pub fn fold_return_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonReturnStmt,
    ) -> T {
        match &stmt.value {
            Some(v) => folder.fold_expr(v),
            None => T::default(),
        }
    }

    pub fn fold_revert_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonRevertStmt,
    ) -> T {
        let mut result = T::default();
        for arg in &stmt.args {
            let r = folder.fold_expr(arg);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_assert_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a CanonAssertStmt,
    ) -> T {
        let mut result = folder.fold_expr(&stmt.cond);
        if let Some(msg) = &stmt.message {
            let r = folder.fold_expr(msg);
            result = folder.combine(result, r);
        }
        result
    }

    // ── Expressions ────────────────────────────────────────────

    pub fn fold_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonExpr,
    ) -> T {
        match expr {
            CanonExpr::Var(v) => folder.fold_var_expr(v),
            CanonExpr::Lit(_) => T::default(),
            CanonExpr::BinOp(e) => folder.fold_binop_expr(e),
            CanonExpr::UnOp(e) => folder.fold_unop_expr(e),
            CanonExpr::IndexAccess(e) => folder.fold_index_access_expr(e),
            CanonExpr::FieldAccess(e) => folder.fold_field_access_expr(e),
            CanonExpr::FunctionCall(e) => folder.fold_call_expr(e),
            CanonExpr::TypeCast(e) => folder.fold_type_cast_expr(e),
            CanonExpr::Old(inner) => folder.fold_expr(inner),
            CanonExpr::Result(_) => T::default(),
            CanonExpr::Forall { body, .. } => folder.fold_expr(body),
            CanonExpr::Exists { body, .. } => folder.fold_expr(body),
            CanonExpr::Dialect(_) => T::default(),
        }
    }

    pub fn fold_binop_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonBinOpExpr,
    ) -> T {
        let a = folder.fold_expr(&expr.lhs);
        let b = folder.fold_expr(&expr.rhs);
        folder.combine(a, b)
    }

    pub fn fold_unop_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonUnOpExpr,
    ) -> T {
        folder.fold_expr(&expr.operand)
    }

    pub fn fold_index_access_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonIndexAccessExpr,
    ) -> T {
        let mut result = folder.fold_expr(&expr.base);
        if let Some(idx) = &expr.index {
            let r = folder.fold_expr(idx);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_field_access_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonFieldAccessExpr,
    ) -> T {
        folder.fold_expr(&expr.base)
    }

    pub fn fold_call_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonCallExpr,
    ) -> T {
        let mut result = folder.fold_expr(&expr.callee);
        for arg in &expr.args {
            let r = folder.fold_expr(arg);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_type_cast_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CanonTypeCastExpr,
    ) -> T {
        folder.fold_expr(&expr.expr)
    }
}
