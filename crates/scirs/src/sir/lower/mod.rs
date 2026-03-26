//! SIR → CIR lowering.
//!
//! Orchestrates the conversion from `sir::Module` to `cir::CanonModule`.
//!
//! The pipeline runs 5 semantic normalization passes on the SIR module before
//! performing the structural conversion to CIR:
//!
//! 1. `elim_named_args`  — convert named call arguments to positional form.
//! 2. `elim_using`       — strip `UsingFor` member declarations.
//! 3. `resolve_inheritance` — flatten inheritance, merge parent members.
//! 4. `elim_modifiers`   — inline modifier bodies into function bodies.
//! 5. `flatten_expr`     — introduce temporaries so call args are atoms.
//!
//! After normalization the structural conversion:
//! - Strips the now-empty `parents` field.
//! - Drops `ModifierDef` member declarations (already inlined).
//! - Converts `sir::Expr` → `cir::CanonExpr`.
//! - Converts `sir::Stmt` → `cir::CanonStmt`.

mod elim_modifiers;
mod elim_named_args;
mod elim_using;
mod flatten_expr;
mod resolve_inheritance;

use crate::cir::defs::*;
use crate::cir::exprs::*;
use crate::cir::module::*;
use crate::cir::stmts::*;
use crate::sir;
use thiserror::Error;

/// Errors that can occur during SIR → CIR lowering.
#[derive(Debug, Error)]
pub enum CirLowerError {
    #[error("CIR lowering error: {0}")]
    General(String),
}

/// Lower a SIR Module into a CIR CanonModule.
///
/// This is the main entry point for SIR → CIR conversion.
pub fn lower_module(sir_module: &sir::Module) -> Result<CanonModule, CirLowerError> {
    // Phase 1: Semantic normalization (SIR → SIR)
    let module = elim_named_args::run(sir_module)?;
    let module = elim_using::run(&module)?;
    let module = resolve_inheritance::run(&module)?;
    let module = elim_modifiers::run(&module)?;
    let module = flatten_expr::run(&module)?;

    // Phase 2: Structural conversion (SIR → CIR)
    let mut lowerer = CirLowerer::new();
    lowerer.lower_module(&module)
}

/// Internal state for the SIR → CIR lowering.
struct CirLowerer {
    _tmp_var_index: usize,
}

impl CirLowerer {
    fn new() -> Self {
        CirLowerer { _tmp_var_index: 0 }
    }

    fn lower_module(&mut self, module: &sir::Module) -> Result<CanonModule, CirLowerError> {
        let mut decls = Vec::new();

        for decl in &module.decls {
            match decl {
                sir::Decl::Contract(c) => {
                    decls.push(CanonDecl::Contract(self.lower_contract(c)?));
                }
                sir::Decl::Dialect(d) => {
                    decls.push(CanonDecl::Dialect(d.clone()));
                }
            }
        }

        let mut canon_module = CanonModule::new(&module.id, decls);
        canon_module.attrs = module.attrs.clone();
        Ok(canon_module)
    }

    fn lower_contract(
        &mut self,
        contract: &sir::ContractDecl,
    ) -> Result<CanonContractDecl, CirLowerError> {
        let mut members = Vec::new();

        for member in &contract.members {
            match member {
                sir::MemberDecl::Storage(s) => {
                    members.push(CanonMemberDecl::Storage(self.lower_storage(s)?));
                }
                sir::MemberDecl::Function(f) => {
                    members.push(CanonMemberDecl::Function(self.lower_function(f)?));
                }
                sir::MemberDecl::TypeAlias(ta) => {
                    members.push(CanonMemberDecl::TypeAlias(CanonTypeAlias {
                        name: ta.name.clone(),
                        ty: ta.ty.clone(),
                    }));
                }
                sir::MemberDecl::GlobalInvariant(inv) => {
                    members.push(CanonMemberDecl::GlobalInvariant(self.lower_expr(inv)?));
                }
                sir::MemberDecl::Dialect(d) => {
                    // Filter out modifier definitions (already inlined).
                    if Self::is_modifier_def(d) {
                        continue;
                    }
                    members.push(CanonMemberDecl::Dialect(d.clone()));
                }
                sir::MemberDecl::UsingFor(_) => {
                    // UsingFor declarations must be eliminated by the
                    // elim_using pass before reaching here.
                    continue;
                }
            }
        }

        let mut canon = CanonContractDecl::new(contract.name.clone(), members, contract.span);
        canon.attrs = contract.attrs.clone();
        Ok(canon)
    }

    /// Check if a dialect member declaration is a modifier definition.
    fn is_modifier_def(d: &sir::DialectMemberDecl) -> bool {
        matches!(d, sir::DialectMemberDecl::Evm(sir::dialect::evm::EvmMemberDecl::ModifierDef(_)))
    }

    fn lower_storage(
        &mut self,
        storage: &sir::StorageDecl,
    ) -> Result<CanonStorageDecl, CirLowerError> {
        let init = match &storage.init {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        let mut canon =
            CanonStorageDecl::new(storage.name.clone(), storage.ty.clone(), init, storage.span);
        canon.attrs = storage.attrs.clone();
        Ok(canon)
    }

    fn lower_function(
        &mut self,
        func: &sir::FunctionDecl,
    ) -> Result<CanonFunctionDecl, CirLowerError> {
        let params: Vec<CanonParam> = func
            .params
            .iter()
            .map(|p| CanonParam::new(p.name.clone(), p.ty.clone()))
            .collect();

        let body = match &func.body {
            Some(stmts) => self.lower_stmts(stmts)?,
            None => vec![],
        };

        let mut canon = CanonFunctionDecl::new(
            func.name.clone(),
            params,
            func.returns.clone(),
            body,
            func.span,
        );
        canon.attrs = func.attrs.clone();
        canon.spec = func.spec.clone();
        canon.type_params = func
            .type_params
            .iter()
            .map(|tp| CanonTypeParam { name: tp.name.clone() })
            .collect();

        Ok(canon)
    }

    // ─── Statement lowering ──────────────────────────────────────

    fn lower_stmts(&mut self, stmts: &[sir::Stmt]) -> Result<Vec<CanonStmt>, CirLowerError> {
        stmts.iter().map(|s| self.lower_stmt(s)).collect()
    }

    fn lower_stmt(&mut self, stmt: &sir::Stmt) -> Result<CanonStmt, CirLowerError> {
        match stmt {
            sir::Stmt::LocalVar(s) => {
                let vars = s
                    .vars
                    .iter()
                    .map(|v| {
                        v.as_ref()
                            .map(|d| CanonLocalVarDecl { name: d.name.clone(), ty: d.ty.clone() })
                    })
                    .collect();
                let init = match &s.init {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::LocalVar(CanonLocalVarStmt { vars, init, span: s.span }))
            }
            sir::Stmt::Assign(s) => Ok(CanonStmt::Assign(CanonAssignStmt {
                lhs: self.lower_expr(&s.lhs)?,
                rhs: self.lower_expr(&s.rhs)?,
                span: s.span,
            })),
            sir::Stmt::AugAssign(s) => Ok(CanonStmt::AugAssign(CanonAugAssignStmt {
                op: s.op,
                lhs: self.lower_expr(&s.lhs)?,
                rhs: self.lower_expr(&s.rhs)?,
                span: s.span,
            })),
            sir::Stmt::Expr(s) => Ok(CanonStmt::Expr(CanonExprStmt {
                expr: self.lower_expr(&s.expr)?,
                span: s.span,
            })),
            sir::Stmt::If(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let then_body = self.lower_stmts(&s.then_body)?;
                let else_body = match &s.else_body {
                    Some(stmts) => Some(self.lower_stmts(stmts)?),
                    None => None,
                };
                Ok(CanonStmt::If(CanonIfStmt { cond, then_body, else_body, span: s.span }))
            }
            sir::Stmt::While(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let body = self.lower_stmts(&s.body)?;
                let invariant = match &s.invariant {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::While(CanonWhileStmt { cond, body, invariant, span: s.span }))
            }
            sir::Stmt::For(s) => {
                let init = match &s.init {
                    Some(stmt) => Some(Box::new(self.lower_stmt(stmt)?)),
                    None => None,
                };
                let cond = match &s.cond {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                let update = match &s.update {
                    Some(stmt) => Some(Box::new(self.lower_stmt(stmt)?)),
                    None => None,
                };
                let body = self.lower_stmts(&s.body)?;
                let invariant = match &s.invariant {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::For(CanonForStmt {
                    init,
                    cond,
                    update,
                    body,
                    invariant,
                    span: s.span,
                }))
            }
            sir::Stmt::Return(s) => {
                let value = match &s.value {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::Return(CanonReturnStmt { value, span: s.span }))
            }
            sir::Stmt::Revert(s) => {
                let args = s
                    .args
                    .iter()
                    .map(|e| self.lower_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CanonStmt::Revert(CanonRevertStmt {
                    error: s.error.clone(),
                    args,
                    span: s.span,
                }))
            }
            sir::Stmt::Assert(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let message = match &s.message {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::Assert(CanonAssertStmt { cond, message, span: s.span }))
            }
            sir::Stmt::Break => Ok(CanonStmt::Break),
            sir::Stmt::Continue => Ok(CanonStmt::Continue),
            sir::Stmt::Block(stmts) => Ok(CanonStmt::Block(self.lower_stmts(stmts)?)),
            sir::Stmt::Dialect(s) => Ok(CanonStmt::Dialect(s.clone())),
        }
    }

    // ─── Expression lowering ─────────────────────────────────────

    fn lower_expr(&mut self, expr: &sir::Expr) -> Result<CanonExpr, CirLowerError> {
        match expr {
            sir::Expr::Var(v) => Ok(CanonExpr::Var(CanonVarExpr {
                name: v.name.clone(),
                ty: v.ty.clone(),
                span: v.span,
            })),
            sir::Expr::Lit(l) => Ok(CanonExpr::Lit(l.clone())),
            sir::Expr::BinOp(e) => Ok(CanonExpr::BinOp(CanonBinOpExpr {
                op: e.op,
                lhs: Box::new(self.lower_expr(&e.lhs)?),
                rhs: Box::new(self.lower_expr(&e.rhs)?),
                overflow: e.overflow,
                span: e.span,
            })),
            sir::Expr::UnOp(e) => Ok(CanonExpr::UnOp(CanonUnOpExpr {
                op: e.op,
                operand: Box::new(self.lower_expr(&e.operand)?),
                span: e.span,
            })),
            sir::Expr::IndexAccess(e) => {
                let index = match &e.index {
                    Some(idx) => Some(Box::new(self.lower_expr(idx)?)),
                    None => None,
                };
                Ok(CanonExpr::IndexAccess(CanonIndexAccessExpr {
                    base: Box::new(self.lower_expr(&e.base)?),
                    index,
                    ty: e.ty.clone(),
                    span: e.span,
                }))
            }
            sir::Expr::FieldAccess(e) => Ok(CanonExpr::FieldAccess(CanonFieldAccessExpr {
                base: Box::new(self.lower_expr(&e.base)?),
                field: e.field.clone(),
                ty: e.ty.clone(),
                span: e.span,
            })),
            sir::Expr::FunctionCall(e) => {
                let args = match &e.args {
                    sir::CallArgs::Positional(args) => args
                        .iter()
                        .map(|a| self.lower_expr(a))
                        .collect::<Result<Vec<_>, _>>()?,
                    sir::CallArgs::Named(_) => {
                        return Err(CirLowerError::General(
                            "Named arguments must be eliminated before CIR lowering".into(),
                        ));
                    }
                };
                Ok(CanonExpr::FunctionCall(CanonCallExpr {
                    callee: Box::new(self.lower_expr(&e.callee)?),
                    args,
                    ty: e.ty.clone(),
                    span: e.span,
                }))
            }
            sir::Expr::TypeCast(e) => Ok(CanonExpr::TypeCast(CanonTypeCastExpr {
                ty: e.ty.clone(),
                expr: Box::new(self.lower_expr(&e.expr)?),
                span: e.span,
            })),
            sir::Expr::Ternary(e) => {
                // Ternary is still allowed at this stage — the AST-level
                // normalization may not have eliminated all of them yet.
                // We lower it as a BinOp placeholder for now; in the future
                // this should be lowered to an if-statement at the CIR level.
                // For now, preserve it as a function call pattern.
                let cond = self.lower_expr(&e.cond)?;
                let then_expr = self.lower_expr(&e.then_expr)?;
                let else_expr = self.lower_expr(&e.else_expr)?;
                // Represent ternary as: __ternary__(cond, then, else)
                let callee = CanonExpr::Var(CanonVarExpr::new(
                    "__ternary__".to_string(),
                    then_expr.typ(),
                    e.span,
                ));
                Ok(CanonExpr::FunctionCall(CanonCallExpr {
                    callee: Box::new(callee),
                    args: vec![cond, then_expr, else_expr],
                    ty: expr.typ(),
                    span: e.span,
                }))
            }
            sir::Expr::Tuple(e) => {
                // Tuples should be unrolled by AST normalization.
                // If one still appears, convert it to a function call pattern.
                let elems = e
                    .elems
                    .iter()
                    .filter_map(|elem| elem.as_ref())
                    .map(|elem| self.lower_expr(elem))
                    .collect::<Result<Vec<_>, _>>()?;
                let callee = CanonExpr::Var(CanonVarExpr::new(
                    "__tuple__".to_string(),
                    e.ty.clone(),
                    e.span,
                ));
                Ok(CanonExpr::FunctionCall(CanonCallExpr {
                    callee: Box::new(callee),
                    args: elems,
                    ty: e.ty.clone(),
                    span: e.span,
                }))
            }
            sir::Expr::Old(inner) => Ok(CanonExpr::Old(Box::new(self.lower_expr(inner)?))),
            sir::Expr::Result(idx) => Ok(CanonExpr::Result(*idx)),
            sir::Expr::Forall { var, ty, body } => Ok(CanonExpr::Forall {
                var: var.clone(),
                ty: ty.clone(),
                body: Box::new(self.lower_expr(body)?),
            }),
            sir::Expr::Exists { var, ty, body } => Ok(CanonExpr::Exists {
                var: var.clone(),
                ty: ty.clone(),
                body: Box::new(self.lower_expr(body)?),
            }),
            sir::Expr::Dialect(d) => Ok(CanonExpr::Dialect(d.clone())),
        }
    }
}
