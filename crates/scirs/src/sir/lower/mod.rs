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
//! - Makes chain semantics explicit: reads and writes of contract state
//!   become `Load` / `Store` (locals shadow state variables per block
//!   scope), calls to the contract's own functions become `InternalCall`,
//!   member calls on values become `ExternalCall`, and dialect constructs
//!   are mapped by `lower_dialect`.

mod elim_modifiers;
mod elim_named_args;
mod elim_using;
mod flatten_expr;
mod lower_dialect;
mod resolve_inheritance;

use crate::cir::defs::*;
use crate::cir::exprs::*;
use crate::cir::module::*;
use crate::cir::stmts::*;
use crate::semantics::ExternalKind;
use crate::sir;
use crate::sir::dialect::move_lang::MoveExpr;
use crate::sir::dialect::DialectExpr;
use std::collections::{HashMap, HashSet};
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

/// Prefix of temporaries introduced by lowering.
const TMP_PREFIX: &str = "__cir_tmp";

/// A contract state location: a path (struct fields joined by `.`) and its
/// index expressions in order.
#[derive(Clone)]
struct StoragePath {
    keys: Vec<sir::Expr>,
    path: String,
}

/// Internal state for the SIR → CIR lowering.
struct CirLowerer {
    /// Names of all contracts, interfaces, and libraries of the module.
    contracts: HashSet<String>,
    /// Names of the current contract's functions (internal call targets).
    functions: HashSet<String>,
    /// Local variable scopes of the current function, innermost last.
    scopes: Vec<HashSet<String>>,
    /// Locals of the current function that are storage pointers (e.g.
    /// `var acc = accounts[msg.sender]`), with the location they refer to.
    /// Their keys are re-evaluated at each use.
    storage_aliases: HashMap<String, StoragePath>,
    /// Names of the current contract's state variables.
    storage_vars: HashSet<String>,
    /// Counter for temporaries introduced by lowering.
    tmp_index: usize,
}

impl CirLowerer {
    fn new() -> Self {
        CirLowerer {
            contracts: HashSet::new(),
            functions: HashSet::new(),
            scopes: Vec::new(),
            storage_aliases: HashMap::new(),
            storage_vars: HashSet::new(),
            tmp_index: 0,
        }
    }

    // ─── Name classification ─────────────────────────────────────

    fn is_local(&self, name: &str) -> bool {
        self.scopes.iter().any(|scope| scope.contains(name))
    }

    fn is_state_var(&self, name: &str) -> bool {
        self.storage_vars.contains(name) && !self.is_local(name)
    }

    fn declare_local(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    /// If `expr` denotes contract state, directly or through a storage
    /// pointer local, return the location.
    fn storage_path(&self, expr: &sir::Expr) -> Option<StoragePath> {
        match expr {
            sir::Expr::Var(var) if self.is_state_var(&var.name) => {
                Some(StoragePath { keys: vec![], path: var.name.clone() })
            }
            sir::Expr::Var(var) => self.storage_aliases.get(&var.name).cloned(),
            sir::Expr::IndexAccess(access) => {
                let mut location = self.storage_path(&access.base)?;
                location.keys.extend(access.index.as_deref().cloned());
                Some(location)
            }
            sir::Expr::FieldAccess(access) => {
                let mut location = self.storage_path(&access.base)?;
                location.path = format!("{}.{}", location.path, access.field);
                Some(location)
            }
            _ => None,
        }
    }

    /// Record whether the local `name` declared with type `ty` and
    /// initializer `init` is a storage pointer.
    fn track_storage_alias(&mut self, name: &str, ty: &crate::sir::Type, init: Option<&sir::Expr>) {
        let location = init.filter(|_| is_reference_type(ty)).and_then(|e| self.storage_path(e));
        match location {
            Some(location) => self.storage_aliases.insert(name.to_string(), location),
            None => self.storage_aliases.remove(name),
        };
    }

    /// Returns `true` if `expr` evaluates to a value (as opposed to naming a
    /// contract, library, or type), so a member call on it is external.
    fn is_value_receiver(&self, expr: &sir::Expr) -> bool {
        match expr {
            sir::Expr::Var(var) => self.is_local(&var.name) || self.is_state_var(&var.name),
            _ => true,
        }
    }

    /// Returns `true` if `expr` has a type whose members are called
    /// externally: a contract (or interface) or an address.
    fn is_external_receiver(&self, expr: &sir::Expr) -> bool {
        match expr.typ() {
            crate::sir::Type::TypeRef(name) => self.contracts.contains(&name),
            crate::sir::Type::Dialect(crate::sir::DialectType::Evm(evm_type)) => matches!(
                evm_type,
                crate::sir::evm::EvmType::Address | crate::sir::evm::EvmType::AddressPayable
            ),
            _ => false,
        }
    }

    fn fresh_tmp(&mut self) -> String {
        self.tmp_index += 1;
        format!("{TMP_PREFIX}{}", self.tmp_index)
    }

    fn lower_module(&mut self, module: &sir::Module) -> Result<CanonModule, CirLowerError> {
        self.contracts = module
            .decls
            .iter()
            .filter_map(|decl| match decl {
                sir::Decl::Contract(c) => Some(c.name.clone()),
                sir::Decl::Dialect(_) => None,
            })
            .collect();
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
        self.functions.clear();
        self.storage_vars.clear();
        for member in &contract.members {
            match member {
                sir::MemberDecl::Function(f) => {
                    self.functions.insert(f.name.clone());
                }
                sir::MemberDecl::Storage(s) => {
                    self.storage_vars.insert(s.name.clone());
                }
                sir::MemberDecl::TypeAlias(_)
                | sir::MemberDecl::GlobalInvariant(_)
                | sir::MemberDecl::Dialect(_)
                | sir::MemberDecl::UsingFor(_) => {}
            }
        }

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

        let mut canon =
            CanonContractDecl::new(contract.name.clone(), members, contract.span.clone());
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
        let mut canon = CanonStorageDecl::new(
            storage.name.clone(),
            storage.ty.clone(),
            init,
            storage.span.clone(),
        );
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

        self.storage_aliases.clear();
        let param_names = func.params.iter().map(|p| p.name.clone());
        let body = match &func.body {
            Some(stmts) => self.lower_scoped(param_names, stmts)?,
            None => vec![],
        };

        let mut canon = CanonFunctionDecl::new(
            func.name.clone(),
            params,
            func.returns.clone(),
            body,
            func.span.clone(),
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

    /// Lower `stmts` in a new local scope that starts with `declared`.
    fn lower_scoped(
        &mut self,
        declared: impl IntoIterator<Item = String>,
        stmts: &[sir::Stmt],
    ) -> Result<Vec<CanonStmt>, CirLowerError> {
        self.scopes.push(declared.into_iter().collect());
        let lowered = self.lower_stmts(stmts);
        self.scopes.pop();
        lowered
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
                // The initializer is evaluated before the variables are in scope.
                let init = match &s.init {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                // A single reference-typed local initialized from state is a
                // storage pointer; any other declaration ends an alias.
                let alias_init = match s.vars.as_slice() {
                    [Some(_)] => s.init.as_ref(),
                    _ => None,
                };
                for decl in s.vars.iter().flatten() {
                    self.declare_local(&decl.name);
                    self.track_storage_alias(&decl.name, &decl.ty, alias_init);
                }
                Ok(CanonStmt::LocalVar(CanonLocalVarStmt { vars, init, span: s.span.clone() }))
            }
            sir::Stmt::Assign(s) => match self.storage_path(&s.lhs) {
                Some(location) => {
                    let keys = self.lower_exprs(&location.keys)?;
                    Ok(CanonStmt::Store(CanonStoreStmt {
                        resource: CanonResource::StateVar(location.path),
                        keys,
                        value: Some(self.lower_expr(&s.rhs)?),
                        span: s.span.clone(),
                    }))
                }
                None => Ok(CanonStmt::Assign(CanonAssignStmt {
                    lhs: self.lower_expr(&s.lhs)?,
                    rhs: self.lower_expr(&s.rhs)?,
                    span: s.span.clone(),
                })),
            },
            sir::Stmt::AugAssign(s) => match self.storage_path(&s.lhs) {
                Some(location) => self.lower_state_aug_assign(s, location),
                None => Ok(CanonStmt::AugAssign(CanonAugAssignStmt {
                    op: s.op,
                    lhs: self.lower_expr(&s.lhs)?,
                    rhs: self.lower_expr(&s.rhs)?,
                    span: s.span.clone(),
                })),
            },
            sir::Stmt::Expr(s) => match &s.expr {
                sir::Expr::Dialect(DialectExpr::Move(MoveExpr::MoveTo(move_to))) => {
                    self.lower_move_to(move_to)
                }
                // `delete state[keys]` resets contract state.
                sir::Expr::UnOp(unop) if unop.op == sir::UnOp::Delete => {
                    match self.storage_path(&unop.operand) {
                        Some(location) => Ok(CanonStmt::Store(CanonStoreStmt {
                            resource: CanonResource::StateVar(location.path),
                            keys: self.lower_exprs(&location.keys)?,
                            value: None,
                            span: s.span.clone(),
                        })),
                        None => Ok(CanonStmt::Expr(CanonExprStmt {
                            expr: self.lower_expr(&s.expr)?,
                            span: s.span.clone(),
                        })),
                    }
                }
                expr => Ok(CanonStmt::Expr(CanonExprStmt {
                    expr: self.lower_expr(expr)?,
                    span: s.span.clone(),
                })),
            },
            sir::Stmt::If(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let then_body = self.lower_scoped([], &s.then_body)?;
                let else_body = match &s.else_body {
                    Some(stmts) => Some(self.lower_scoped([], stmts)?),
                    None => None,
                };
                Ok(CanonStmt::If(CanonIfStmt { cond, then_body, else_body, span: s.span.clone() }))
            }
            sir::Stmt::While(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let body = self.lower_scoped([], &s.body)?;
                let invariant = match &s.invariant {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::While(CanonWhileStmt {
                    cond,
                    body,
                    invariant,
                    span: s.span.clone(),
                }))
            }
            sir::Stmt::For(s) => {
                // Variables declared in the init clause are scoped to the loop.
                self.scopes.push(HashSet::new());
                let lowered = self.lower_for(s);
                self.scopes.pop();
                lowered
            }
            sir::Stmt::Return(s) => {
                let value = match &s.value {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::Return(CanonReturnStmt { value, span: s.span.clone() }))
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
                    span: s.span.clone(),
                }))
            }
            sir::Stmt::Assert(s) => {
                let cond = self.lower_expr(&s.cond)?;
                let message = match &s.message {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                Ok(CanonStmt::Assert(CanonAssertStmt { cond, message, span: s.span.clone() }))
            }
            sir::Stmt::Break => Ok(CanonStmt::Break),
            sir::Stmt::Continue => Ok(CanonStmt::Continue),
            sir::Stmt::Block(stmts) => Ok(CanonStmt::Block(self.lower_scoped([], stmts)?)),
            sir::Stmt::Dialect(s) => self.lower_dialect_stmt(s),
        }
    }

    fn lower_for(&mut self, s: &sir::ForStmt) -> Result<CanonStmt, CirLowerError> {
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
        let body = self.lower_scoped([], &s.body)?;
        let invariant = match &s.invariant {
            Some(e) => Some(self.lower_expr(e)?),
            None => None,
        };
        Ok(CanonStmt::For(CanonForStmt { init, cond, update, body, invariant, span: s.span.clone() }))
    }

    /// Lower `state[keys] op= rhs` to `state[keys] = state[keys] op rhs`.
    /// Keys that are not side-effect-free atoms are bound to temporaries
    /// first, so they are evaluated once.
    fn lower_state_aug_assign(
        &mut self,
        s: &sir::AugAssignStmt,
        location: StoragePath,
    ) -> Result<CanonStmt, CirLowerError> {
        let StoragePath { keys: key_exprs, path } = location;
        let mut stmts = Vec::new();
        let mut keys = Vec::new();
        for key_expr in &key_exprs {
            let key = self.lower_expr(key_expr)?;
            if is_pure_atom(&key) {
                keys.push(key);
                continue;
            }
            let name = self.fresh_tmp();
            let ty = key.typ();
            stmts.push(CanonStmt::LocalVar(CanonLocalVarStmt {
                vars: vec![Some(CanonLocalVarDecl { name: name.clone(), ty: ty.clone() })],
                init: Some(key),
                span: s.span.clone(),
            }));
            self.declare_local(&name);
            keys.push(CanonExpr::Var(CanonVarExpr::new(name, ty, s.span.clone())));
        }

        let resource = CanonResource::StateVar(path);
        let current = CanonExpr::Load(CanonLoadExpr {
            resource: resource.clone(),
            keys: keys.clone(),
            ty: s.lhs.typ(),
            span: s.span.clone(),
        });
        let value = CanonExpr::BinOp(CanonBinOpExpr {
            op: s.op,
            lhs: Box::new(current),
            rhs: Box::new(self.lower_expr(&s.rhs)?),
            overflow: crate::sir::OverflowSemantics::Checked,
            span: s.span.clone(),
        });
        let store =
            CanonStmt::Store(CanonStoreStmt { resource, keys, value: Some(value), span: s.span.clone() });
        if stmts.is_empty() {
            return Ok(store);
        }
        stmts.push(store);
        Ok(CanonStmt::Block(stmts))
    }

    // ─── Expression lowering ─────────────────────────────────────

    fn lower_exprs(&mut self, exprs: &[sir::Expr]) -> Result<Vec<CanonExpr>, CirLowerError> {
        exprs.iter().map(|e| self.lower_expr(e)).collect()
    }

    /// `state[keys...]` as a `Load`.
    fn lower_state_load(
        &mut self,
        location: StoragePath,
        ty: &crate::sir::Type,
        span: Option<&crate::sir::Loc>,
    ) -> Result<CanonExpr, CirLowerError> {
        Ok(CanonExpr::Load(CanonLoadExpr {
            resource: CanonResource::StateVar(location.path),
            keys: self.lower_exprs(&location.keys)?,
            ty: ty.clone(),
            span: span.cloned(),
        }))
    }

    fn lower_expr(&mut self, expr: &sir::Expr) -> Result<CanonExpr, CirLowerError> {
        if let Some(location) = self.storage_path(expr) {
            return self.lower_state_load(location, &expr.typ(), expr.span());
        }
        match expr {
            sir::Expr::Var(v) => Ok(CanonExpr::Var(CanonVarExpr {
                name: v.name.clone(),
                ty: v.ty.clone(),
                span: v.span.clone(),
            })),
            sir::Expr::Lit(l) => Ok(CanonExpr::Lit(l.clone())),
            sir::Expr::BinOp(e) => Ok(CanonExpr::BinOp(CanonBinOpExpr {
                op: e.op,
                lhs: Box::new(self.lower_expr(&e.lhs)?),
                rhs: Box::new(self.lower_expr(&e.rhs)?),
                overflow: e.overflow,
                span: e.span.clone(),
            })),
            sir::Expr::UnOp(e) => Ok(CanonExpr::UnOp(CanonUnOpExpr {
                op: e.op,
                operand: Box::new(self.lower_expr(&e.operand)?),
                span: e.span.clone(),
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
                    span: e.span.clone(),
                }))
            }
            sir::Expr::FieldAccess(e) => Ok(CanonExpr::FieldAccess(CanonFieldAccessExpr {
                base: Box::new(self.lower_expr(&e.base)?),
                field: e.field.clone(),
                ty: e.ty.clone(),
                span: e.span.clone(),
            })),
            sir::Expr::FunctionCall(e) => {
                if let Some(call) = self.lower_resolved_call(e)? {
                    return Ok(call);
                }
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
                    span: e.span.clone(),
                }))
            }
            sir::Expr::TypeCast(e) => Ok(CanonExpr::TypeCast(CanonTypeCastExpr {
                ty: e.ty.clone(),
                expr: Box::new(self.lower_expr(&e.expr)?),
                span: e.span.clone(),
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
                    e.span.clone(),
                ));
                Ok(CanonExpr::FunctionCall(CanonCallExpr {
                    callee: Box::new(callee),
                    args: vec![cond, then_expr, else_expr],
                    ty: expr.typ(),
                    span: e.span.clone(),
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
                    e.span.clone(),
                ));
                Ok(CanonExpr::FunctionCall(CanonCallExpr {
                    callee: Box::new(callee),
                    args: elems,
                    ty: e.ty.clone(),
                    span: e.span.clone(),
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
            sir::Expr::Dialect(d) => self.lower_dialect_expr(d),
        }
    }

    /// Lower a call whose target is known: a function of this contract, or
    /// a member call on a value (an external call). Returns `None` for
    /// unresolved calls (builtins, libraries, type constructors).
    fn lower_resolved_call(
        &mut self,
        call: &sir::CallExpr,
    ) -> Result<Option<CanonExpr>, CirLowerError> {
        let sir::CallArgs::Positional(arg_exprs) = &call.args else {
            return Ok(None);
        };
        // Legacy option syntax: `addr.call.value(v).gas(g)(data)`.
        if let Some((method, value)) = peel_legacy_call_options(&call.callee) {
            let kind = match method.field.as_str() {
                "delegatecall" => ExternalKind::DelegateCall,
                _ => ExternalKind::Call,
            };
            let address = self.lower_expr(&method.base)?;
            let args = arg_exprs.iter().map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
            let value = match value {
                Some(value) => Some(Box::new(self.lower_expr(value)?)),
                None => None,
            };
            return Ok(Some(CanonExpr::ExternalCall(CanonExternalCallExpr {
                kind,
                address: Some(Box::new(address)),
                args,
                value,
                ty: call.ty.clone(),
                span: call.span.clone(),
            })));
        }
        match &*call.callee {
            sir::Expr::Var(var) if self.functions.contains(&var.name) && !self.is_local(&var.name) => {
                let args = arg_exprs.iter().map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
                Ok(Some(CanonExpr::InternalCall(CanonInternalCallExpr {
                    func: var.name.clone(),
                    args,
                    ty: call.ty.clone(),
                    span: call.span.clone(),
                })))
            }
            sir::Expr::FieldAccess(access) if self.is_value_receiver(&access.base) => {
                let has_one_arg = arg_exprs.len() == 1;
                let kind = match access.field.as_str() {
                    "call" => ExternalKind::Call,
                    "delegatecall" => ExternalKind::DelegateCall,
                    "staticcall" => ExternalKind::StaticCall,
                    "transfer" if has_one_arg => ExternalKind::Transfer,
                    "send" if has_one_arg => ExternalKind::Send,
                    _ => ExternalKind::HighLevel,
                };
                // A member call on a value that is not a contract or an
                // address is a library call (`using L for T`), not external.
                if kind == ExternalKind::HighLevel && !self.is_external_receiver(&access.base) {
                    return Ok(None);
                }
                let address = self.lower_expr(&access.base)?;
                let mut args: Vec<CanonExpr> =
                    arg_exprs.iter().map(|a| self.lower_expr(a)).collect::<Result<_, _>>()?;
                let value = match kind {
                    ExternalKind::Transfer | ExternalKind::Send => args.pop(),
                    ExternalKind::Call
                    | ExternalKind::Cpi
                    | ExternalKind::DelegateCall
                    | ExternalKind::HighLevel
                    | ExternalKind::StaticCall
                    | ExternalKind::SystemTransfer
                    | ExternalKind::TokenTransfer => None,
                };
                Ok(Some(CanonExpr::ExternalCall(CanonExternalCallExpr {
                    kind,
                    address: Some(Box::new(address)),
                    args,
                    value: value.map(Box::new),
                    ty: call.ty.clone(),
                    span: call.span.clone(),
                })))
            }
            _ => Ok(None),
        }
    }
}

/// Returns `true` for types whose locals refer to (rather than copy) a state
/// location when initialized from one: structs, arrays, and mappings.
fn is_reference_type(ty: &crate::sir::Type) -> bool {
    matches!(
        ty,
        crate::sir::Type::TypeRef(_)
            | crate::sir::Type::Array(_)
            | crate::sir::Type::FixedArray(..)
            | crate::sir::Type::Map(..)
    )
}

/// Peel the legacy call options `.value(v)` / `.gas(g)` (Solidity < 0.7) off
/// a callee such as `addr.call.value(v).gas(g)`, returning the underlying
/// `addr.call` / `addr.delegatecall` access and the value option.
fn peel_legacy_call_options(
    callee: &sir::Expr,
) -> Option<(&sir::FieldAccessExpr, Option<&sir::Expr>)> {
    let sir::Expr::FunctionCall(option_call) = callee else { return None };
    let sir::Expr::FieldAccess(option) = &*option_call.callee else { return None };
    let sir::CallArgs::Positional(option_args) = &option_call.args else { return None };
    let [option_arg] = option_args.as_slice() else { return None };
    let (method, value) = match &*option.base {
        sir::Expr::FieldAccess(method) if matches!(method.field.as_str(), "call" | "delegatecall") => {
            (method, None)
        }
        base => peel_legacy_call_options(base)?,
    };
    match option.field.as_str() {
        "value" => Some((method, Some(option_arg))),
        "gas" => Some((method, value)),
        _ => None,
    }
}

/// Returns `true` if evaluating `expr` twice is equivalent to evaluating it
/// once: locals, literals, environment reads, and state reads with such
/// keys.
fn is_pure_atom(expr: &CanonExpr) -> bool {
    match expr {
        CanonExpr::Var(_) | CanonExpr::Lit(_) | CanonExpr::Env(_) => true,
        CanonExpr::Load(load) => load.keys.iter().all(is_pure_atom),
        CanonExpr::BinOp(_)
        | CanonExpr::UnOp(_)
        | CanonExpr::IndexAccess(_)
        | CanonExpr::FieldAccess(_)
        | CanonExpr::FunctionCall(_)
        | CanonExpr::TypeCast(_)
        | CanonExpr::InternalCall(_)
        | CanonExpr::ExternalCall(_)
        | CanonExpr::Old(_)
        | CanonExpr::Result(_)
        | CanonExpr::Forall { .. }
        | CanonExpr::Exists { .. }
        | CanonExpr::Dialect(_) => false,
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::utils::visit::{self, Visit};
    use crate::sir::{
        AssignStmt, AugAssignStmt, BinOp, BoolLit, CallArgs, CallExpr, ContractDecl, Decl, Expr,
        FunctionDecl, IfStmt, IndexAccessExpr, Lit, LocalVarDecl, LocalVarStmt, MemberDecl,
        StorageDecl, Stmt, Type, VarExpr,
    };

    fn var(name: &str) -> Expr {
        Expr::Var(VarExpr::new(name.to_string(), Type::I256, None))
    }

    fn lit() -> Expr {
        Expr::Lit(Lit::Bool(BoolLit::new(true, None)))
    }

    /// Lower contract `C { uint balances; function f(c) { body } }` and
    /// return the CIR body of `f`.
    fn lower_body(body: Vec<Stmt>) -> Vec<CanonStmt> {
        let storage = StorageDecl::new("balances".to_string(), Type::I256, None, None);
        let params = vec![sir::Param::new("c".to_string(), Type::Bool)];
        let func = FunctionDecl::new("f".to_string(), params, vec![], Some(body), None);
        let members = vec![MemberDecl::Storage(storage), MemberDecl::Function(func)];
        let contract = ContractDecl::new("C".to_string(), members, None);
        let module = lower_module(&sir::Module::new("m", vec![Decl::Contract(contract)])).unwrap();
        let CanonDecl::Contract(contract) = &module.decls[0] else { panic!("expected contract") };
        contract
            .members
            .iter()
            .find_map(|m| match m {
                CanonMemberDecl::Function(f) => Some(f.body.clone()),
                _ => None,
            })
            .unwrap()
    }

    /// Counts stores and unresolved calls anywhere in a body.
    #[derive(Default)]
    struct Counter {
        calls: usize,
        stores: usize,
    }

    impl<'a> Visit<'a> for Counter {
        fn visit_store_stmt(&mut self, stmt: &'a CanonStoreStmt) {
            self.stores += 1;
            visit::default::visit_store_stmt(self, stmt);
        }

        fn visit_call_expr(&mut self, expr: &'a CanonCallExpr) {
            self.calls += 1;
            visit::default::visit_call_expr(self, expr);
        }
    }

    fn count(stmts: &[CanonStmt]) -> Counter {
        let mut counter = Counter::default();
        counter.visit_stmts(stmts);
        counter
    }

    #[test]
    fn test_local_shadows_state_only_inside_its_block() {
        // if (c) { uint balances = true; balances = true; } balances = true;
        let local = Stmt::LocalVar(LocalVarStmt {
            vars: vec![Some(LocalVarDecl { name: "balances".to_string(), ty: Type::I256, is_storage_ref: false })],
            init: Some(lit()),
            span: None,
        });
        let write = || Stmt::Assign(AssignStmt { lhs: var("balances"), rhs: lit(), span: None });
        let branch = Stmt::If(IfStmt {
            cond: var("c"),
            then_body: vec![local, write()],
            else_body: None,
            span: None,
        });
        let body = lower_body(vec![branch, write()]);

        // Only the write after the block reaches contract state.
        let CanonStmt::If(branch) = &body[0] else { panic!("expected if") };
        assert_eq!(count(&branch.then_body).stores, 0);
        assert!(matches!(&body[1], CanonStmt::Store(s)
            if s.resource == CanonResource::StateVar("balances".to_string())));
    }

    #[test]
    fn test_state_aug_assign_evaluates_impure_key_once() {
        // balances[next()] += true;
        let key = Expr::FunctionCall(CallExpr {
            callee: Box::new(var("next")),
            args: CallArgs::Positional(vec![]),
            ty: Type::I256,
            span: None,
        });
        let lhs = Expr::IndexAccess(IndexAccessExpr {
            base: Box::new(var("balances")),
            index: Some(Box::new(key)),
            ty: Type::I256,
            span: None,
        });
        let aug = Stmt::AugAssign(AugAssignStmt { op: BinOp::Add, lhs, rhs: lit(), span: None });
        let body = lower_body(vec![aug]);

        let counter = count(&body);
        assert_eq!(counter.stores, 1);
        assert_eq!(counter.calls, 1, "the key must be evaluated exactly once");
    }

    #[test]
    fn test_legacy_call_value_syntax_is_external_call() {
        // c.call.value(true)("") with Solidity < 0.7 option syntax.
        let call_member = Expr::FieldAccess(sir::FieldAccessExpr {
            base: Box::new(var("c")),
            field: "call".to_string(),
            ty: Type::None,
            span: None,
        });
        let value_member = Expr::FieldAccess(sir::FieldAccessExpr {
            base: Box::new(call_member),
            field: "value".to_string(),
            ty: Type::None,
            span: None,
        });
        let with_value = Expr::FunctionCall(CallExpr {
            callee: Box::new(value_member),
            args: CallArgs::Positional(vec![lit()]),
            ty: Type::None,
            span: None,
        });
        let call = Expr::FunctionCall(CallExpr {
            callee: Box::new(with_value),
            args: CallArgs::Positional(vec![]),
            ty: Type::Bool,
            span: None,
        });
        let body = lower_body(vec![Stmt::Expr(sir::ExprStmt { expr: call, span: None })]);

        let CanonStmt::Expr(stmt) = &body[0] else { panic!("expected expression statement") };
        let CanonExpr::ExternalCall(call) = &stmt.expr else { panic!("expected external call") };
        assert_eq!(call.kind, ExternalKind::Call);
        assert!(matches!(call.address.as_deref(), Some(CanonExpr::Var(v)) if v.name == "c"));
        assert!(call.value.is_some());
    }
}
