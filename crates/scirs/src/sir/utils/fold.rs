//! Fold pattern for SIR — accumulating traversal.

use crate::sir::*;

/// Trait implementing the fold design pattern for SIR.
/// Traverses the SIR tree while accumulating a result of type `T`.
pub trait Fold<'a, T: Default> {
    // ── Module ──────────────────────────────────────
    fn fold_modules(&mut self, modules: &'a [Module]) -> T {
        default::fold_modules(self, modules)
    }
    fn fold_module(&mut self, module: &'a Module) -> T {
        default::fold_module(self, module)
    }
    fn fold_decl(&mut self, decl: &'a Decl) -> T {
        default::fold_decl(self, decl)
    }

    // ── Contract ────────────────────────────────────
    fn fold_contract_decl(&mut self, contract: &'a ContractDecl) -> T {
        default::fold_contract_decl(self, contract)
    }
    fn fold_member_decl(&mut self, member: &'a MemberDecl) -> T {
        default::fold_member_decl(self, member)
    }

    // ── Storage & Function ──────────────────────────
    fn fold_storage_decl(&mut self, _storage: &'a StorageDecl) -> T {
        T::default()
    }
    fn fold_function_decl(&mut self, func: &'a FunctionDecl) -> T {
        default::fold_function_decl(self, func)
    }
    fn fold_param(&mut self, _param: &'a Param) -> T {
        T::default()
    }

    // ── Statements ──────────────────────────────────
    fn fold_stmts(&mut self, stmts: &'a [Stmt]) -> T {
        default::fold_stmts(self, stmts)
    }
    fn fold_stmt(&mut self, stmt: &'a Stmt) -> T {
        default::fold_stmt(self, stmt)
    }
    fn fold_local_var_stmt(&mut self, _stmt: &'a LocalVarStmt) -> T {
        T::default()
    }
    fn fold_assign_stmt(&mut self, stmt: &'a AssignStmt) -> T {
        default::fold_assign_stmt(self, stmt)
    }
    fn fold_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) -> T {
        default::fold_aug_assign_stmt(self, stmt)
    }
    fn fold_expr_stmt(&mut self, stmt: &'a ExprStmt) -> T {
        default::fold_expr_stmt(self, stmt)
    }
    fn fold_if_stmt(&mut self, stmt: &'a IfStmt) -> T {
        default::fold_if_stmt(self, stmt)
    }
    fn fold_while_stmt(&mut self, stmt: &'a WhileStmt) -> T {
        default::fold_while_stmt(self, stmt)
    }
    fn fold_for_stmt(&mut self, stmt: &'a ForStmt) -> T {
        default::fold_for_stmt(self, stmt)
    }
    fn fold_return_stmt(&mut self, stmt: &'a ReturnStmt) -> T {
        default::fold_return_stmt(self, stmt)
    }
    fn fold_revert_stmt(&mut self, stmt: &'a RevertStmt) -> T {
        default::fold_revert_stmt(self, stmt)
    }
    fn fold_assert_stmt(&mut self, stmt: &'a AssertStmt) -> T {
        default::fold_assert_stmt(self, stmt)
    }

    // ── Expressions ─────────────────────────────────
    fn fold_expr(&mut self, expr: &'a Expr) -> T {
        default::fold_expr(self, expr)
    }
    fn fold_var_expr(&mut self, _var: &'a VarExpr) -> T {
        T::default()
    }
    fn fold_binop_expr(&mut self, expr: &'a BinOpExpr) -> T {
        default::fold_binop_expr(self, expr)
    }
    fn fold_unop_expr(&mut self, expr: &'a UnOpExpr) -> T {
        default::fold_unop_expr(self, expr)
    }
    fn fold_index_access_expr(&mut self, expr: &'a IndexAccessExpr) -> T {
        default::fold_index_access_expr(self, expr)
    }
    fn fold_field_access_expr(&mut self, expr: &'a FieldAccessExpr) -> T {
        default::fold_field_access_expr(self, expr)
    }
    fn fold_call_expr(&mut self, expr: &'a CallExpr) -> T {
        default::fold_call_expr(self, expr)
    }
    fn fold_type_cast_expr(&mut self, expr: &'a TypeCastExpr) -> T {
        default::fold_type_cast_expr(self, expr)
    }
    fn fold_ternary_expr(&mut self, expr: &'a TernaryExpr) -> T {
        default::fold_ternary_expr(self, expr)
    }
    fn fold_tuple_expr(&mut self, expr: &'a TupleExpr) -> T {
        default::fold_tuple_expr(self, expr)
    }

    /// Combine two folded values. Override to change the accumulation strategy.
    fn combine(&mut self, a: T, _b: T) -> T {
        a
    }
}

/// Default implementations for the Fold trait.
pub mod default {
    use super::Fold;
    use crate::sir::*;

    pub fn fold_modules<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        modules: &'a [Module],
    ) -> T {
        let mut result = T::default();
        for m in modules {
            let r = folder.fold_module(m);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_module<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        module: &'a Module,
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
        decl: &'a Decl,
    ) -> T {
        match decl {
            Decl::Contract(c) => folder.fold_contract_decl(c),
            Decl::Dialect(_) => T::default(),
        }
    }

    pub fn fold_contract_decl<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        contract: &'a ContractDecl,
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
        member: &'a MemberDecl,
    ) -> T {
        match member {
            MemberDecl::Storage(s) => folder.fold_storage_decl(s),
            MemberDecl::Function(f) => folder.fold_function_decl(f),
            MemberDecl::TypeAlias(_) => T::default(),
            MemberDecl::GlobalInvariant(inv) => folder.fold_expr(inv),
            MemberDecl::Dialect(_) => T::default(),
            MemberDecl::UsingFor(_) => T::default(),
        }
    }

    pub fn fold_function_decl<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        func: &'a FunctionDecl,
    ) -> T {
        let mut result = T::default();
        for p in &func.params {
            let r = folder.fold_param(p);
            result = folder.combine(result, r);
        }
        if let Some(body) = &func.body {
            let r = folder.fold_stmts(body);
            result = folder.combine(result, r);
        }
        result
    }

    // ── Statements ─────────────────────────────────────────────

    pub fn fold_stmts<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmts: &'a [Stmt],
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
        stmt: &'a Stmt,
    ) -> T {
        match stmt {
            Stmt::LocalVar(s) => folder.fold_local_var_stmt(s),
            Stmt::Assign(s) => folder.fold_assign_stmt(s),
            Stmt::AugAssign(s) => folder.fold_aug_assign_stmt(s),
            Stmt::Expr(s) => folder.fold_expr_stmt(s),
            Stmt::If(s) => folder.fold_if_stmt(s),
            Stmt::While(s) => folder.fold_while_stmt(s),
            Stmt::For(s) => folder.fold_for_stmt(s),
            Stmt::Return(s) => folder.fold_return_stmt(s),
            Stmt::Revert(s) => folder.fold_revert_stmt(s),
            Stmt::Assert(s) => folder.fold_assert_stmt(s),
            Stmt::Break | Stmt::Continue => T::default(),
            Stmt::Block(stmts) => folder.fold_stmts(stmts),
            Stmt::Dialect(_) => T::default(),
        }
    }

    pub fn fold_assign_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a AssignStmt,
    ) -> T {
        let a = folder.fold_expr(&stmt.lhs);
        let b = folder.fold_expr(&stmt.rhs);
        folder.combine(a, b)
    }

    pub fn fold_aug_assign_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a AugAssignStmt,
    ) -> T {
        let a = folder.fold_expr(&stmt.lhs);
        let b = folder.fold_expr(&stmt.rhs);
        folder.combine(a, b)
    }

    pub fn fold_expr_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a ExprStmt,
    ) -> T {
        folder.fold_expr(&stmt.expr)
    }

    pub fn fold_if_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a IfStmt,
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
        stmt: &'a WhileStmt,
    ) -> T {
        let mut result = folder.fold_expr(&stmt.cond);
        let r = folder.fold_stmts(&stmt.body);
        result = folder.combine(result, r);
        result
    }

    pub fn fold_for_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a ForStmt,
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
        stmt: &'a ReturnStmt,
    ) -> T {
        match &stmt.value {
            Some(v) => folder.fold_expr(v),
            None => T::default(),
        }
    }

    pub fn fold_revert_stmt<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        stmt: &'a RevertStmt,
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
        stmt: &'a AssertStmt,
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
        expr: &'a Expr,
    ) -> T {
        match expr {
            Expr::Var(v) => folder.fold_var_expr(v),
            Expr::Lit(_) => T::default(),
            Expr::BinOp(e) => folder.fold_binop_expr(e),
            Expr::UnOp(e) => folder.fold_unop_expr(e),
            Expr::IndexAccess(e) => folder.fold_index_access_expr(e),
            Expr::FieldAccess(e) => folder.fold_field_access_expr(e),
            Expr::FunctionCall(e) => folder.fold_call_expr(e),
            Expr::TypeCast(e) => folder.fold_type_cast_expr(e),
            Expr::Ternary(e) => folder.fold_ternary_expr(e),
            Expr::Tuple(e) => folder.fold_tuple_expr(e),
            Expr::Old(inner) => folder.fold_expr(inner),
            Expr::Result(_) => T::default(),
            Expr::Forall { body, .. } => folder.fold_expr(body),
            Expr::Exists { body, .. } => folder.fold_expr(body),
            Expr::Dialect(_) => T::default(),
        }
    }

    pub fn fold_binop_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a BinOpExpr,
    ) -> T {
        let a = folder.fold_expr(&expr.lhs);
        let b = folder.fold_expr(&expr.rhs);
        folder.combine(a, b)
    }

    pub fn fold_unop_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a UnOpExpr,
    ) -> T {
        folder.fold_expr(&expr.operand)
    }

    pub fn fold_index_access_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a IndexAccessExpr,
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
        expr: &'a FieldAccessExpr,
    ) -> T {
        folder.fold_expr(&expr.base)
    }

    pub fn fold_call_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a CallExpr,
    ) -> T {
        let mut result = folder.fold_expr(&expr.callee);
        let args = expr.args.exprs();
        for arg in args {
            let r = folder.fold_expr(arg);
            result = folder.combine(result, r);
        }
        result
    }

    pub fn fold_type_cast_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a TypeCastExpr,
    ) -> T {
        folder.fold_expr(&expr.expr)
    }

    pub fn fold_ternary_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a TernaryExpr,
    ) -> T {
        let a = folder.fold_expr(&expr.cond);
        let b = folder.fold_expr(&expr.then_expr);
        let c = folder.fold_expr(&expr.else_expr);
        let ab = folder.combine(a, b);
        folder.combine(ab, c)
    }

    pub fn fold_tuple_expr<'a, T: Default, F: Fold<'a, T> + ?Sized>(
        folder: &mut F,
        expr: &'a TupleExpr,
    ) -> T {
        let mut result = T::default();
        for e in &expr.elems {
            if let Some(ex) = e {
                let r = folder.fold_expr(ex);
                result = folder.combine(result, r);
            }
        }
        result
    }
}
