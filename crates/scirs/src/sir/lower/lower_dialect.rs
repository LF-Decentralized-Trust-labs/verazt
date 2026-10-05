//! SIR dialect → CIR lowering.
//!
//! Maps chain-specific SIR constructs to the shared CIR forms (`Load`,
//! `Store`, `ExternalCall`, `Env`, `Emit`) where they generalize, and to the
//! typed CIR dialect remainder otherwise. This is the only place that knows
//! how each chain's constructs relate to the shared semantics.

use super::{CirLowerError, CirLowerer};
use crate::cir::dialect::*;
use crate::cir::exprs::*;
use crate::cir::stmts::*;
use crate::semantics::{EnvVar, EvmBuiltin, ExternalKind};
use crate::sir::dialect::anchor::{AnchorExpr, AnchorStmt};
use crate::sir::dialect::evm::{EvmExpr, EvmStmt, EvmTryCatch};
use crate::sir::dialect::move_lang::{MoveExpr, MoveMoveTo, MoveStmt};
use crate::sir::dialect::{DialectExpr, DialectStmt};
use crate::sir::{Expr, Loc, Type};

type Lowered<T> = Result<T, CirLowerError>;

impl CirLowerer {
    // ─── Expressions ─────────────────────────────────────────────

    pub(super) fn lower_dialect_expr(&mut self, expr: &DialectExpr) -> Lowered<CanonExpr> {
        let ty = expr.typ();
        match expr {
            DialectExpr::Evm(e) => self.lower_evm_expr(e, ty),
            DialectExpr::Move(e) => self.lower_move_expr(e, ty),
            DialectExpr::Anchor(e) => self.lower_anchor_expr(e, ty),
        }
    }

    fn lower_evm_expr(&mut self, expr: &EvmExpr, ty: Type) -> Lowered<CanonExpr> {
        match expr {
            // ── Environment reads ──────────────────────────
            EvmExpr::BlockBasefee(e) => Ok(env(EnvVar::BlockBasefee, ty, &e.loc)),
            EvmExpr::BlockChainid(e) => Ok(env(EnvVar::BlockChainid, ty, &e.loc)),
            EvmExpr::BlockCoinbase(e) => Ok(env(EnvVar::BlockCoinbase, ty, &e.loc)),
            EvmExpr::BlockDifficulty(e) => Ok(env(EnvVar::BlockDifficulty, ty, &e.loc)),
            EvmExpr::BlockGaslimit(e) => Ok(env(EnvVar::BlockGaslimit, ty, &e.loc)),
            EvmExpr::BlockNumber(e) => Ok(env(EnvVar::BlockNumber, ty, &e.loc)),
            EvmExpr::Gasleft(e) => Ok(env(EnvVar::GasLeft, ty, &e.loc)),
            EvmExpr::MsgData(e) => Ok(env(EnvVar::CallData, ty, &e.loc)),
            EvmExpr::MsgSender(e) => Ok(env(EnvVar::Caller, ty, &e.loc)),
            EvmExpr::MsgSig(e) => Ok(env(EnvVar::Selector, ty, &e.loc)),
            EvmExpr::MsgValue(e) => Ok(env(EnvVar::CallValue, ty, &e.loc)),
            EvmExpr::SelfBalance(e) => Ok(env(EnvVar::SelfBalance, ty, &e.loc)),
            EvmExpr::This(e) => Ok(env(EnvVar::SelfAddress, ty, &e.loc)),
            EvmExpr::Timestamp(e) => Ok(env(EnvVar::Timestamp, ty, &e.loc)),
            EvmExpr::TxOrigin(e) => Ok(env(EnvVar::Origin, ty, &e.loc)),

            // ── External calls ─────────────────────────────
            EvmExpr::Delegatecall(e) => {
                let call = ExternalCallParts::new(ExternalKind::DelegateCall, &e.loc)
                    .address(&e.target)
                    .args(vec![&e.data]);
                self.lower_external_call(call, ty)
            }
            EvmExpr::LowLevelCall(e) => {
                let call = ExternalCallParts::new(ExternalKind::Call, &e.loc)
                    .address(&e.target)
                    .args(vec![&e.data])
                    .value(e.value.as_deref());
                self.lower_external_call(call, ty)
            }
            EvmExpr::RawCall(e) => {
                let call = ExternalCallParts::new(ExternalKind::Call, &e.loc)
                    .address(&e.target)
                    .args(vec![&e.data])
                    .value(e.value.as_deref());
                self.lower_external_call(call, ty)
            }
            EvmExpr::Send(e) => {
                let call = ExternalCallParts::new(ExternalKind::Send, &e.loc)
                    .address(&e.target)
                    .value(Some(&e.value));
                self.lower_external_call(call, ty)
            }
            EvmExpr::Transfer(e) => {
                let call = ExternalCallParts::new(ExternalKind::Transfer, &e.loc)
                    .address(&e.target)
                    .value(Some(&e.amount));
                self.lower_external_call(call, ty)
            }

            // ── Builtins ───────────────────────────────────
            EvmExpr::AbiDecode(e) => {
                self.evm_builtin(EvmBuiltin::AbiDecode, vec![&e.data], ty, &e.loc)
            }
            EvmExpr::AbiEncode(e) => {
                self.evm_builtin(EvmBuiltin::AbiEncode, e.args.iter().collect(), ty, &e.loc)
            }
            EvmExpr::AbiEncodeCall(e) => {
                let args = std::iter::once(&*e.func).chain(&e.args).collect();
                self.evm_builtin(EvmBuiltin::AbiEncodeCall, args, ty, &e.loc)
            }
            EvmExpr::AbiEncodePacked(e) => {
                self.evm_builtin(EvmBuiltin::AbiEncodePacked, e.args.iter().collect(), ty, &e.loc)
            }
            EvmExpr::AbiEncodeWithSelector(e) => {
                let args = std::iter::once(&*e.selector).chain(&e.args).collect();
                self.evm_builtin(EvmBuiltin::AbiEncodeWithSelector, args, ty, &e.loc)
            }
            EvmExpr::AbiEncodeWithSignature(e) => {
                let args = std::iter::once(&*e.signature).chain(&e.args).collect();
                self.evm_builtin(EvmBuiltin::AbiEncodeWithSignature, args, ty, &e.loc)
            }
            EvmExpr::Addmod(e) => {
                self.evm_builtin(EvmBuiltin::Addmod, vec![&e.x, &e.y, &e.k], ty, &e.loc)
            }
            EvmExpr::Blockhash(e) => {
                self.evm_builtin(EvmBuiltin::Blockhash, vec![&e.expr], ty, &e.loc)
            }
            EvmExpr::Concat(e) => {
                self.evm_builtin(EvmBuiltin::Concat, e.exprs.iter().collect(), ty, &e.loc)
            }
            EvmExpr::Convert(e) => {
                self.evm_builtin(EvmBuiltin::Convert, vec![&e.expr], ty, &e.loc)
            }
            EvmExpr::Ecrecover(e) => self.evm_builtin(
                EvmBuiltin::Ecrecover,
                vec![&e.hash, &e.v, &e.r, &e.s],
                ty,
                &e.loc,
            ),
            EvmExpr::Empty(e) => self.evm_builtin(EvmBuiltin::Empty, vec![], ty, &e.loc),
            EvmExpr::Keccak256(e) => {
                self.evm_builtin(EvmBuiltin::Keccak256, vec![&e.expr], ty, &e.loc)
            }
            EvmExpr::Len(e) => self.evm_builtin(EvmBuiltin::Len, vec![&e.expr], ty, &e.loc),
            EvmExpr::Mulmod(e) => {
                self.evm_builtin(EvmBuiltin::Mulmod, vec![&e.x, &e.y, &e.k], ty, &e.loc)
            }
            EvmExpr::Ripemd160(e) => {
                self.evm_builtin(EvmBuiltin::Ripemd160, vec![&e.expr], ty, &e.loc)
            }
            EvmExpr::Sha256(e) => self.evm_builtin(EvmBuiltin::Sha256, vec![&e.expr], ty, &e.loc),
            EvmExpr::Slice(e) => {
                self.evm_builtin(EvmBuiltin::Slice, vec![&e.expr, &e.start, &e.length], ty, &e.loc)
            }

            // ── Other ──────────────────────────────────────
            EvmExpr::InlineAsm(e) => {
                let kind = CanonDialectKind::Evm(CanonEvmExpr::InlineAsm(e.asm_text.clone()));
                Ok(dialect(kind, ty, &e.loc))
            }
            // `super` only appears as a callee; inheritance is resolved
            // before CIR, so it is a plain symbol here.
            EvmExpr::Super(e) => {
                Ok(CanonExpr::Var(CanonVarExpr::new("super".to_string(), ty, Some(e.loc.clone()))))
            }
        }
    }

    fn lower_move_expr(&mut self, expr: &MoveExpr, ty: Type) -> Lowered<CanonExpr> {
        let kind = match expr {
            MoveExpr::BorrowGlobal(e) => {
                return Ok(CanonExpr::Load(CanonLoadExpr {
                    resource: CanonResource::MoveGlobal(e.ty.clone()),
                    keys: vec![self.lower_expr(&e.addr)?],
                    ty,
                    span: Some(e.loc.clone()),
                }));
            }
            // Specification variables are plain symbols.
            MoveExpr::GhostVar(e) => {
                return Ok(CanonExpr::Var(CanonVarExpr::new(
                    e.name.clone(),
                    ty,
                    Some(e.loc.clone()),
                )));
            }
            // `move_to` in expression position (it is normally a statement,
            // lowered to `Store` by `lower_move_to`): keep it as an
            // unresolved call so its operands are still evaluated.
            MoveExpr::MoveTo(e) => {
                let callee =
                    CanonExpr::Var(CanonVarExpr::new("move_to".to_string(), Type::None, None));
                return Ok(CanonExpr::FunctionCall(CanonCallExpr {
                    callee: Box::new(callee),
                    args: vec![self.lower_expr(&e.resource)?, self.lower_expr(&e.signer)?],
                    ty,
                    span: Some(e.loc.clone()),
                }));
            }
            MoveExpr::BorrowGlobalMut(e) => {
                (CanonMoveExpr::BorrowGlobalMut(self.move_global(&e.addr, &e.ty)?), &e.loc)
            }
            MoveExpr::Exists(e) => {
                (CanonMoveExpr::Exists(self.move_global(&e.addr, &e.ty)?), &e.loc)
            }
            MoveExpr::MoveFrom(e) => {
                (CanonMoveExpr::MoveFrom(self.move_global(&e.addr, &e.ty)?), &e.loc)
            }
            MoveExpr::SignerAddress(e) => {
                (CanonMoveExpr::SignerAddress(Box::new(self.lower_expr(&e.expr)?)), &e.loc)
            }
            MoveExpr::WriteRef(e) => (
                CanonMoveExpr::WriteRef(CanonMoveWriteRefExpr {
                    reference: Box::new(self.lower_expr(&e.reference)?),
                    value: Box::new(self.lower_expr(&e.value)?),
                }),
                &e.loc,
            ),
        };
        let (kind, loc) = kind;
        Ok(dialect(CanonDialectKind::Move(kind), ty, loc))
    }

    fn lower_anchor_expr(&mut self, expr: &AnchorExpr, ty: Type) -> Lowered<CanonExpr> {
        match expr {
            AnchorExpr::AccountLoad(e) => Ok(CanonExpr::Load(CanonLoadExpr {
                resource: CanonResource::AnchorAccount(Box::new(self.lower_expr(&e.expr)?)),
                keys: vec![],
                ty,
                span: Some(e.loc.clone()),
            })),
            AnchorExpr::AccountLoadMut(e) => {
                let account = Box::new(self.lower_expr(&e.expr)?);
                let kind = CanonDialectKind::Anchor(CanonAnchorExpr::AccountLoadMut(account));
                Ok(dialect(kind, ty, &e.loc))
            }
            AnchorExpr::Cpi(e) => {
                let args = e.accounts.iter().chain(std::iter::once(&*e.data)).collect();
                let call = ExternalCallParts::new(ExternalKind::Cpi, &e.loc)
                    .address(&e.program)
                    .args(args);
                self.lower_external_call(call, ty)
            }
            AnchorExpr::FindProgramAddress(e) => {
                let pda = CanonAnchorPdaExpr {
                    program_id: Box::new(self.lower_expr(&e.program_id)?),
                    seeds: e
                        .seeds
                        .iter()
                        .map(|s| self.lower_expr(s))
                        .collect::<Lowered<_>>()?,
                };
                let kind = CanonDialectKind::Anchor(CanonAnchorExpr::FindProgramAddress(pda));
                Ok(dialect(kind, ty, &e.loc))
            }
            // `Ok(e)` is a transparent result wrapper.
            AnchorExpr::Ok(e) => self.lower_expr(&e.expr),
            AnchorExpr::SignerKey(e) => {
                let account = Box::new(self.lower_expr(&e.expr)?);
                Ok(dialect(
                    CanonDialectKind::Anchor(CanonAnchorExpr::SignerKey(account)),
                    ty,
                    &e.loc,
                ))
            }
            AnchorExpr::SystemTransfer(e) => {
                let call = ExternalCallParts::new(ExternalKind::SystemTransfer, &e.loc)
                    .args(vec![&e.from, &e.to])
                    .value(Some(&e.lamports));
                self.lower_external_call(call, ty)
            }
            AnchorExpr::TokenTransfer(e) => {
                let call = ExternalCallParts::new(ExternalKind::TokenTransfer, &e.loc).args(vec![
                    &e.from,
                    &e.to,
                    &e.authority,
                    &e.amount,
                ]);
                self.lower_external_call(call, ty)
            }
        }
    }

    // ─── Statements ──────────────────────────────────────────────

    pub(super) fn lower_dialect_stmt(&mut self, stmt: &DialectStmt) -> Lowered<CanonStmt> {
        match stmt {
            DialectStmt::Evm(EvmStmt::EmitEvent(e)) => Ok(CanonStmt::Emit(CanonEmitStmt {
                event: e.event.clone(),
                args: e
                    .args
                    .iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<Lowered<_>>()?,
                span: Some(e.loc.clone()),
            })),
            DialectStmt::Evm(EvmStmt::TryCatch(t)) => self.lower_try_catch(t),
            // Modifier bodies are inlined before CIR; a stray placeholder
            // has no effect.
            DialectStmt::Evm(EvmStmt::Placeholder(_)) => Ok(CanonStmt::Block(vec![])),
            DialectStmt::Evm(EvmStmt::Selfdestruct(s)) => {
                let stmt = CanonSelfdestructStmt {
                    recipient: self.lower_expr(&s.recipient)?,
                    span: Some(s.loc.clone()),
                };
                Ok(CanonStmt::Dialect(CanonDialectStmt::Evm(CanonEvmStmt::Selfdestruct(stmt))))
            }
            DialectStmt::Move(MoveStmt::Abort(a)) => {
                let stmt =
                    CanonAbortStmt { code: self.lower_expr(&a.expr)?, span: Some(a.loc.clone()) };
                Ok(CanonStmt::Dialect(CanonDialectStmt::Move(CanonMoveStmt::Abort(stmt))))
            }
            DialectStmt::Move(MoveStmt::SpecBlock(s)) => {
                let stmt = CanonSpecBlockStmt {
                    assertions: s
                        .assertions
                        .iter()
                        .map(|a| self.lower_expr(a))
                        .collect::<Lowered<_>>()?,
                    span: Some(s.loc.clone()),
                };
                Ok(CanonStmt::Dialect(CanonDialectStmt::Move(CanonMoveStmt::SpecBlock(stmt))))
            }
            DialectStmt::Anchor(AnchorStmt::EmitEvent(e)) => Ok(CanonStmt::Emit(CanonEmitStmt {
                event: e.event.clone(),
                args: e
                    .fields
                    .iter()
                    .map(|(_, a)| self.lower_expr(a))
                    .collect::<Lowered<_>>()?,
                span: Some(e.loc.clone()),
            })),
        }
    }

    /// `move_to(signer, resource)` stores `resource` under the signer.
    pub(super) fn lower_move_to(&mut self, move_to: &MoveMoveTo) -> Lowered<CanonStmt> {
        Ok(CanonStmt::Store(CanonStoreStmt {
            resource: CanonResource::MoveGlobal(move_to.resource.typ()),
            keys: vec![self.lower_expr(&move_to.signer)?],
            value: Some(self.lower_expr(&move_to.resource)?),
            span: Some(move_to.loc.clone()),
        }))
    }

    fn lower_try_catch(&mut self, t: &EvmTryCatch) -> Lowered<CanonStmt> {
        let guarded = self.lower_expr(&t.guarded_expr)?;
        let body = self.lower_scoped(t.returns.iter().map(|(name, _)| name.clone()), &t.body)?;
        let mut catch_clauses = Vec::new();
        for clause in &t.catch_clauses {
            let params = clause.params.iter().map(|(name, _)| name.clone());
            catch_clauses.push(CanonCatchClause {
                error: clause.error.clone(),
                params: clause.params.clone(),
                body: self.lower_scoped(params, &clause.body)?,
                span: Some(clause.loc.clone()),
            });
        }
        let stmt = CanonTryCatchStmt {
            guarded,
            returns: t.returns.clone(),
            body,
            catch_clauses,
            span: Some(t.loc.clone()),
        };
        Ok(CanonStmt::Dialect(CanonDialectStmt::Evm(CanonEvmStmt::TryCatch(stmt))))
    }

    // ─── Helpers ─────────────────────────────────────────────────

    fn lower_external_call(&mut self, call: ExternalCallParts, ty: Type) -> Lowered<CanonExpr> {
        let address = match call.address {
            Some(address) => Some(Box::new(self.lower_expr(address)?)),
            None => None,
        };
        let args = call
            .args
            .iter()
            .map(|a| self.lower_expr(a))
            .collect::<Lowered<_>>()?;
        let value = match call.value {
            Some(value) => Some(Box::new(self.lower_expr(value)?)),
            None => None,
        };
        Ok(CanonExpr::ExternalCall(CanonExternalCallExpr {
            kind: call.kind,
            address,
            args,
            value,
            ty,
            span: Some(call.loc.clone()),
        }))
    }

    fn evm_builtin(
        &mut self,
        builtin: EvmBuiltin,
        args: Vec<&Expr>,
        ty: Type,
        loc: &Loc,
    ) -> Lowered<CanonExpr> {
        let args = args
            .into_iter()
            .map(|a| self.lower_expr(a))
            .collect::<Lowered<_>>()?;
        let kind =
            CanonDialectKind::Evm(CanonEvmExpr::Builtin(CanonEvmBuiltinExpr { builtin, args }));
        Ok(dialect(kind, ty, loc))
    }

    fn move_global(&mut self, addr: &Expr, ty: &Type) -> Lowered<CanonMoveGlobalExpr> {
        Ok(CanonMoveGlobalExpr { addr: Box::new(self.lower_expr(addr)?), ty: ty.clone() })
    }
}

/// The SIR operands of an external call, before lowering.
struct ExternalCallParts<'e> {
    address: Option<&'e Expr>,
    args: Vec<&'e Expr>,
    kind: ExternalKind,
    loc: &'e Loc,
    value: Option<&'e Expr>,
}

impl<'e> ExternalCallParts<'e> {
    fn new(kind: ExternalKind, loc: &'e Loc) -> Self {
        ExternalCallParts { address: None, args: vec![], kind, loc, value: None }
    }

    fn address(mut self, address: &'e Expr) -> Self {
        self.address = Some(address);
        self
    }

    fn args(mut self, args: Vec<&'e Expr>) -> Self {
        self.args = args;
        self
    }

    fn value(mut self, value: Option<&'e Expr>) -> Self {
        self.value = value;
        self
    }
}

fn env(var: EnvVar, ty: Type, loc: &Loc) -> CanonExpr {
    CanonExpr::Env(CanonEnvExpr { var, ty, span: Some(loc.clone()) })
}

fn dialect(kind: CanonDialectKind, ty: Type, loc: &Loc) -> CanonExpr {
    CanonExpr::Dialect(CanonDialectExpr { kind, ty, span: Some(loc.clone()) })
}
