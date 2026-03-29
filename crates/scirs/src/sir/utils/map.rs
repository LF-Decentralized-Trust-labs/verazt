//! Map pattern for SIR — transforming traversal.
//!
//! Produces a new SIR tree with nodes potentially rewritten.

use crate::sir::*;

/// Trait implementing the map design pattern for SIR.
/// Produces a new (potentially rewritten) SIR tree.
pub trait Map<'a> {
    // ── Module ──────────────────────────────────────
    fn map_modules(&mut self, modules: &'a [Module]) -> Vec<Module> {
        default::map_modules(self, modules)
    }
    fn map_module(&mut self, module: &'a Module) -> Module {
        default::map_module(self, module)
    }
    fn map_decl(&mut self, decl: &'a Decl) -> Decl {
        default::map_decl(self, decl)
    }

    // ── Contract ────────────────────────────────────
    fn map_contract_decl(&mut self, contract: &'a ContractDecl) -> ContractDecl {
        default::map_contract_decl(self, contract)
    }
    fn map_member_decl(&mut self, member: &'a MemberDecl) -> MemberDecl {
        default::map_member_decl(self, member)
    }

    // ── Storage & Function ──────────────────────────
    fn map_storage_decl(&mut self, storage: &'a StorageDecl) -> StorageDecl {
        default::map_storage_decl(self, storage)
    }
    fn map_function_decl(&mut self, func: &'a FunctionDecl) -> FunctionDecl {
        default::map_function_decl(self, func)
    }
    fn map_param(&mut self, param: &'a Param) -> Param {
        param.clone()
    }

    // ── Statements ──────────────────────────────────
    fn map_stmts(&mut self, stmts: &'a [Stmt]) -> Vec<Stmt> {
        default::map_stmts(self, stmts)
    }
    fn map_stmt(&mut self, stmt: &'a Stmt) -> Stmt {
        default::map_stmt(self, stmt)
    }
    fn map_local_var_stmt(&mut self, stmt: &'a LocalVarStmt) -> LocalVarStmt {
        default::map_local_var_stmt(self, stmt)
    }
    fn map_assign_stmt(&mut self, stmt: &'a AssignStmt) -> AssignStmt {
        default::map_assign_stmt(self, stmt)
    }
    fn map_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) -> AugAssignStmt {
        default::map_aug_assign_stmt(self, stmt)
    }
    fn map_expr_stmt(&mut self, stmt: &'a ExprStmt) -> ExprStmt {
        default::map_expr_stmt(self, stmt)
    }
    fn map_if_stmt(&mut self, stmt: &'a IfStmt) -> IfStmt {
        default::map_if_stmt(self, stmt)
    }
    fn map_while_stmt(&mut self, stmt: &'a WhileStmt) -> WhileStmt {
        default::map_while_stmt(self, stmt)
    }
    fn map_for_stmt(&mut self, stmt: &'a ForStmt) -> ForStmt {
        default::map_for_stmt(self, stmt)
    }
    fn map_return_stmt(&mut self, stmt: &'a ReturnStmt) -> ReturnStmt {
        default::map_return_stmt(self, stmt)
    }
    fn map_revert_stmt(&mut self, stmt: &'a RevertStmt) -> RevertStmt {
        default::map_revert_stmt(self, stmt)
    }
    fn map_assert_stmt(&mut self, stmt: &'a AssertStmt) -> AssertStmt {
        default::map_assert_stmt(self, stmt)
    }

    // ── Expressions ─────────────────────────────────
    fn map_expr(&mut self, expr: &'a Expr) -> Expr {
        default::map_expr(self, expr)
    }
    fn map_var_expr(&mut self, var: &'a VarExpr) -> VarExpr {
        var.clone()
    }
    fn map_binop_expr(&mut self, expr: &'a BinOpExpr) -> BinOpExpr {
        default::map_binop_expr(self, expr)
    }
    fn map_unop_expr(&mut self, expr: &'a UnOpExpr) -> UnOpExpr {
        default::map_unop_expr(self, expr)
    }
    fn map_index_access_expr(&mut self, expr: &'a IndexAccessExpr) -> IndexAccessExpr {
        default::map_index_access_expr(self, expr)
    }
    fn map_field_access_expr(&mut self, expr: &'a FieldAccessExpr) -> FieldAccessExpr {
        default::map_field_access_expr(self, expr)
    }
    fn map_call_expr(&mut self, expr: &'a CallExpr) -> CallExpr {
        default::map_call_expr(self, expr)
    }
    fn map_type_cast_expr(&mut self, expr: &'a TypeCastExpr) -> TypeCastExpr {
        default::map_type_cast_expr(self, expr)
    }
    fn map_ternary_expr(&mut self, expr: &'a TernaryExpr) -> TernaryExpr {
        default::map_ternary_expr(self, expr)
    }
    fn map_tuple_expr(&mut self, expr: &'a TupleExpr) -> TupleExpr {
        default::map_tuple_expr(self, expr)
    }

    // ── Type ────────────────────────────────────────
    fn map_type(&mut self, ty: &'a Type) -> Type {
        ty.clone()
    }
}

/// Default implementations for the Map trait.
pub mod default {
    use super::Map;
    use crate::sir::*;

    pub fn map_modules<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        modules: &'a [Module],
    ) -> Vec<Module> {
        modules.iter().map(|m| mapper.map_module(m)).collect()
    }

    pub fn map_module<'a, T: Map<'a> + ?Sized>(mapper: &mut T, module: &'a Module) -> Module {
        Module {
            id: module.id.clone(),
            attrs: module.attrs.clone(),
            decls: module.decls.iter().map(|d| mapper.map_decl(d)).collect(),
        }
    }

    pub fn map_decl<'a, T: Map<'a> + ?Sized>(mapper: &mut T, decl: &'a Decl) -> Decl {
        match decl {
            Decl::Contract(c) => Decl::Contract(mapper.map_contract_decl(c)),
            Decl::Dialect(d) => Decl::Dialect(d.clone()),
        }
    }

    pub fn map_contract_decl<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        contract: &'a ContractDecl,
    ) -> ContractDecl {
        ContractDecl {
            name: contract.name.clone(),
            parents: contract.parents.clone(),
            attrs: contract.attrs.clone(),
            members: contract
                .members
                .iter()
                .map(|m| mapper.map_member_decl(m))
                .collect(),
            span: contract.span.clone(),
        }
    }

    pub fn map_member_decl<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        member: &'a MemberDecl,
    ) -> MemberDecl {
        match member {
            MemberDecl::Storage(s) => MemberDecl::Storage(mapper.map_storage_decl(s)),
            MemberDecl::Function(f) => MemberDecl::Function(mapper.map_function_decl(f)),
            MemberDecl::TypeAlias(ta) => MemberDecl::TypeAlias(ta.clone()),
            MemberDecl::GlobalInvariant(inv) => MemberDecl::GlobalInvariant(mapper.map_expr(inv)),
            MemberDecl::Dialect(d) => MemberDecl::Dialect(d.clone()),
            MemberDecl::UsingFor(u) => MemberDecl::UsingFor(u.clone()),
        }
    }

    pub fn map_storage_decl<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        storage: &'a StorageDecl,
    ) -> StorageDecl {
        StorageDecl {
            name: storage.name.clone(),
            ty: mapper.map_type(&storage.ty),
            init: storage.init.as_ref().map(|e| mapper.map_expr(e)),
            attrs: storage.attrs.clone(),
            span: storage.span.clone(),
        }
    }

    pub fn map_function_decl<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        func: &'a FunctionDecl,
    ) -> FunctionDecl {
        FunctionDecl {
            name: func.name.clone(),
            type_params: func.type_params.clone(),
            params: func.params.iter().map(|p| mapper.map_param(p)).collect(),
            returns: func.returns.iter().map(|t| mapper.map_type(t)).collect(),
            attrs: func.attrs.clone(),
            spec: func.spec.clone(),
            body: func.body.as_ref().map(|b| mapper.map_stmts(b)),
            modifier_invocs: func
                .modifier_invocs
                .iter()
                .map(|m| ModifierInvoc {
                    name: m.name.clone(),
                    args: m.args.iter().map(|a| mapper.map_expr(a)).collect(),
                    span: m.span.clone(),
                })
                .collect(),
            span: func.span.clone(),
        }
    }

    // ── Statements ─────────────────────────────────────────────

    pub fn map_stmts<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmts: &'a [Stmt]) -> Vec<Stmt> {
        stmts.iter().map(|s| mapper.map_stmt(s)).collect()
    }

    pub fn map_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a Stmt) -> Stmt {
        match stmt {
            Stmt::LocalVar(s) => Stmt::LocalVar(mapper.map_local_var_stmt(s)),
            Stmt::Assign(s) => Stmt::Assign(mapper.map_assign_stmt(s)),
            Stmt::AugAssign(s) => Stmt::AugAssign(mapper.map_aug_assign_stmt(s)),
            Stmt::Expr(s) => Stmt::Expr(mapper.map_expr_stmt(s)),
            Stmt::If(s) => Stmt::If(mapper.map_if_stmt(s)),
            Stmt::While(s) => Stmt::While(mapper.map_while_stmt(s)),
            Stmt::For(s) => Stmt::For(mapper.map_for_stmt(s)),
            Stmt::Return(s) => Stmt::Return(mapper.map_return_stmt(s)),
            Stmt::Revert(s) => Stmt::Revert(mapper.map_revert_stmt(s)),
            Stmt::Assert(s) => Stmt::Assert(mapper.map_assert_stmt(s)),
            Stmt::Break => Stmt::Break,
            Stmt::Continue => Stmt::Continue,
            Stmt::Block(stmts) => Stmt::Block(mapper.map_stmts(stmts)),
            Stmt::Dialect(d) => Stmt::Dialect(d.clone()),
        }
    }

    pub fn map_local_var_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a LocalVarStmt,
    ) -> LocalVarStmt {
        LocalVarStmt {
            vars: stmt.vars.clone(),
            init: stmt.init.as_ref().map(|e| mapper.map_expr(e)),
            span: stmt.span.clone(),
        }
    }

    pub fn map_assign_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a AssignStmt,
    ) -> AssignStmt {
        AssignStmt {
            lhs: mapper.map_expr(&stmt.lhs),
            rhs: mapper.map_expr(&stmt.rhs),
            span: stmt.span.clone(),
        }
    }

    pub fn map_aug_assign_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a AugAssignStmt,
    ) -> AugAssignStmt {
        AugAssignStmt {
            op: stmt.op,
            lhs: mapper.map_expr(&stmt.lhs),
            rhs: mapper.map_expr(&stmt.rhs),
            span: stmt.span.clone(),
        }
    }

    pub fn map_expr_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a ExprStmt) -> ExprStmt {
        ExprStmt { expr: mapper.map_expr(&stmt.expr), span: stmt.span.clone() }
    }

    pub fn map_if_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a IfStmt) -> IfStmt {
        IfStmt {
            cond: mapper.map_expr(&stmt.cond),
            then_body: mapper.map_stmts(&stmt.then_body),
            else_body: stmt.else_body.as_ref().map(|b| mapper.map_stmts(b)),
            span: stmt.span.clone(),
        }
    }

    pub fn map_while_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a WhileStmt,
    ) -> WhileStmt {
        WhileStmt {
            cond: mapper.map_expr(&stmt.cond),
            body: mapper.map_stmts(&stmt.body),
            invariant: stmt.invariant.as_ref().map(|e| mapper.map_expr(e)),
            span: stmt.span.clone(),
        }
    }

    pub fn map_for_stmt<'a, T: Map<'a> + ?Sized>(mapper: &mut T, stmt: &'a ForStmt) -> ForStmt {
        ForStmt {
            init: stmt.init.as_ref().map(|s| Box::new(mapper.map_stmt(s))),
            cond: stmt.cond.as_ref().map(|e| mapper.map_expr(e)),
            update: stmt.update.as_ref().map(|s| Box::new(mapper.map_stmt(s))),
            body: mapper.map_stmts(&stmt.body),
            invariant: stmt.invariant.as_ref().map(|e| mapper.map_expr(e)),
            span: stmt.span.clone(),
        }
    }

    pub fn map_return_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a ReturnStmt,
    ) -> ReturnStmt {
        ReturnStmt {
            value: stmt.value.as_ref().map(|e| mapper.map_expr(e)),
            span: stmt.span.clone(),
        }
    }

    pub fn map_revert_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a RevertStmt,
    ) -> RevertStmt {
        RevertStmt {
            error: stmt.error.clone(),
            args: stmt.args.iter().map(|e| mapper.map_expr(e)).collect(),
            span: stmt.span.clone(),
        }
    }

    pub fn map_assert_stmt<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        stmt: &'a AssertStmt,
    ) -> AssertStmt {
        AssertStmt {
            cond: mapper.map_expr(&stmt.cond),
            message: stmt.message.as_ref().map(|e| mapper.map_expr(e)),
            span: stmt.span.clone(),
        }
    }

    // ── Expressions ────────────────────────────────────────────

    pub fn map_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a Expr) -> Expr {
        match expr {
            Expr::Var(v) => Expr::Var(mapper.map_var_expr(v)),
            Expr::Lit(l) => Expr::Lit(l.clone()),
            Expr::BinOp(e) => Expr::BinOp(mapper.map_binop_expr(e)),
            Expr::UnOp(e) => Expr::UnOp(mapper.map_unop_expr(e)),
            Expr::IndexAccess(e) => Expr::IndexAccess(mapper.map_index_access_expr(e)),
            Expr::FieldAccess(e) => Expr::FieldAccess(mapper.map_field_access_expr(e)),
            Expr::FunctionCall(e) => Expr::FunctionCall(mapper.map_call_expr(e)),
            Expr::TypeCast(e) => Expr::TypeCast(mapper.map_type_cast_expr(e)),
            Expr::Ternary(e) => Expr::Ternary(mapper.map_ternary_expr(e)),
            Expr::Tuple(e) => Expr::Tuple(mapper.map_tuple_expr(e)),
            Expr::Old(inner) => Expr::Old(Box::new(mapper.map_expr(inner))),
            Expr::Result(idx) => Expr::Result(*idx),
            Expr::Forall { var, ty, body } => Expr::Forall {
                var: var.clone(),
                ty: mapper.map_type(ty),
                body: Box::new(mapper.map_expr(body)),
            },
            Expr::Exists { var, ty, body } => Expr::Exists {
                var: var.clone(),
                ty: mapper.map_type(ty),
                body: Box::new(mapper.map_expr(body)),
            },
            Expr::Dialect(d) => Expr::Dialect(d.clone()),
        }
    }

    pub fn map_binop_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a BinOpExpr,
    ) -> BinOpExpr {
        BinOpExpr {
            op: expr.op,
            lhs: Box::new(mapper.map_expr(&expr.lhs)),
            rhs: Box::new(mapper.map_expr(&expr.rhs)),
            overflow: expr.overflow,
            span: expr.span.clone(),
        }
    }

    pub fn map_unop_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a UnOpExpr) -> UnOpExpr {
        UnOpExpr {
            op: expr.op,
            operand: Box::new(mapper.map_expr(&expr.operand)),
            span: expr.span.clone(),
        }
    }

    pub fn map_index_access_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a IndexAccessExpr,
    ) -> IndexAccessExpr {
        IndexAccessExpr {
            base: Box::new(mapper.map_expr(&expr.base)),
            index: expr.index.as_ref().map(|e| Box::new(mapper.map_expr(e))),
            ty: mapper.map_type(&expr.ty),
            span: expr.span.clone(),
        }
    }

    pub fn map_field_access_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a FieldAccessExpr,
    ) -> FieldAccessExpr {
        FieldAccessExpr {
            base: Box::new(mapper.map_expr(&expr.base)),
            field: expr.field.clone(),
            ty: mapper.map_type(&expr.ty),
            span: expr.span.clone(),
        }
    }

    pub fn map_call_expr<'a, T: Map<'a> + ?Sized>(mapper: &mut T, expr: &'a CallExpr) -> CallExpr {
        let args = match &expr.args {
            CallArgs::Positional(args) => {
                CallArgs::Positional(args.iter().map(|a| mapper.map_expr(a)).collect())
            }
            CallArgs::Named(named) => CallArgs::Named(
                named
                    .iter()
                    .map(|n| NamedArg { name: n.name.clone(), value: mapper.map_expr(&n.value) })
                    .collect(),
            ),
        };
        CallExpr {
            callee: Box::new(mapper.map_expr(&expr.callee)),
            args,
            ty: mapper.map_type(&expr.ty),
            span: expr.span.clone(),
        }
    }

    pub fn map_type_cast_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a TypeCastExpr,
    ) -> TypeCastExpr {
        TypeCastExpr {
            ty: mapper.map_type(&expr.ty),
            expr: Box::new(mapper.map_expr(&expr.expr)),
            span: expr.span.clone(),
        }
    }

    pub fn map_ternary_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a TernaryExpr,
    ) -> TernaryExpr {
        TernaryExpr {
            cond: Box::new(mapper.map_expr(&expr.cond)),
            then_expr: Box::new(mapper.map_expr(&expr.then_expr)),
            else_expr: Box::new(mapper.map_expr(&expr.else_expr)),
            span: expr.span.clone(),
        }
    }

    pub fn map_tuple_expr<'a, T: Map<'a> + ?Sized>(
        mapper: &mut T,
        expr: &'a TupleExpr,
    ) -> TupleExpr {
        TupleExpr {
            elems: expr
                .elems
                .iter()
                .map(|e| e.as_ref().map(|ex| mapper.map_expr(ex)))
                .collect(),
            ty: mapper.map_type(&expr.ty),
            span: expr.span.clone(),
        }
    }
}
