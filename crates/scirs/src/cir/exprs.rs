//! CIR expression forms.
//!
//! Key differences from SIR expressions:
//! - `Ternary` is removed (lowered to `if` statement).
//! - `Tuple` is removed (unrolled).
//! - `FunctionCall` args must be atoms (Var or Lit) — no nested calls.
//! - Chain semantics are explicit: contract state reads are `Load`, resolved
//!   calls are `InternalCall` / `ExternalCall`, environment reads are `Env`,
//!   and the remaining chain-specific forms are typed `Dialect` expressions. A
//!   `Var` always names a local variable or a non-value symbol.

use crate::cir::dialect::CanonDialectExpr;
use crate::semantics::{EnvVar, ExternalKind};
use crate::sir::exprs::{BinOp, OverflowSemantics, UnOp};
use crate::sir::lits::Lit;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Core expression enum
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonExpr {
    Var(CanonVarExpr),
    Lit(Lit),
    BinOp(CanonBinOpExpr),
    UnOp(CanonUnOpExpr),
    IndexAccess(CanonIndexAccessExpr),
    FieldAccess(CanonFieldAccessExpr),
    /// Unresolved call (builtin, library, or type constructor) — args must
    /// be atoms (Var or Lit).
    FunctionCall(CanonCallExpr),
    TypeCast(CanonTypeCastExpr),

    // ── Shared chain semantics ─────────────────────────────────
    /// Read of persistent state.
    Load(CanonLoadExpr),
    /// Call to a function of the same contract.
    InternalCall(CanonInternalCallExpr),
    /// Call leaving the current contract/program.
    ExternalCall(CanonExternalCallExpr),
    /// Read of an execution-environment value.
    Env(CanonEnvExpr),

    // NOTE: Ternary is removed — lowered to if statement.
    // NOTE: Tuple is removed — unrolled.

    // ── Spec-only expressions (valid in @requires/@ensures) ────
    Old(Box<CanonExpr>),
    Result(u32),
    Forall {
        var: String,
        ty: Type,
        body: Box<CanonExpr>,
    },
    Exists {
        var: String,
        ty: Type,
        body: Box<CanonExpr>,
    },

    // ── Chain-specific remainder ───────────────────────────────
    Dialect(CanonDialectExpr),
}

// ═══════════════════════════════════════════════════════════════════
// Expression sub-types
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonVarExpr {
    pub name: String,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonBinOpExpr {
    pub op: BinOp,
    pub lhs: Box<CanonExpr>,
    pub rhs: Box<CanonExpr>,
    pub overflow: OverflowSemantics,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonUnOpExpr {
    pub op: UnOp,
    pub operand: Box<CanonExpr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonIndexAccessExpr {
    pub base: Box<CanonExpr>,
    pub index: Option<Box<CanonExpr>>,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonFieldAccessExpr {
    pub base: Box<CanonExpr>,
    pub field: String,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonCallExpr {
    pub callee: Box<CanonExpr>,
    pub args: Vec<CanonExpr>,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonTypeCastExpr {
    pub ty: Type,
    pub expr: Box<CanonExpr>,
    pub span: Option<Loc>,
}

/// `resource[keys...]`, a read of persistent state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonLoadExpr {
    pub resource: CanonResource,
    /// Index operands (mapping keys, array indices, owner address).
    pub keys: Vec<CanonExpr>,
    pub ty: Type,
    pub span: Option<Loc>,
}

/// A persistent-state location family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonResource {
    /// A Solana/Anchor account, identified by the account expression.
    AnchorAccount(Box<CanonExpr>),
    /// A Move global resource of the given type, keyed by owner address.
    MoveGlobal(Type),
    /// A contract state variable, as a dotted path for struct fields
    /// (e.g. `balances`, `config.owner`).
    StateVar(String),
}

/// A call to function `func` of the same contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonInternalCallExpr {
    pub func: String,
    pub args: Vec<CanonExpr>,
    pub ty: Type,
    pub span: Option<Loc>,
}

/// A call leaving the current contract/program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonExternalCallExpr {
    pub kind: ExternalKind,
    /// The called address/program, when it is explicit.
    pub address: Option<Box<CanonExpr>>,
    pub args: Vec<CanonExpr>,
    /// Native value transferred with the call (ETH, lamports).
    pub value: Option<Box<CanonExpr>>,
    pub ty: Type,
    pub span: Option<Loc>,
}

/// A read of an execution-environment value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonEnvExpr {
    pub var: EnvVar,
    pub ty: Type,
    pub span: Option<Loc>,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl CanonExpr {
    pub fn typ(&self) -> Type {
        match self {
            CanonExpr::Var(v) => v.ty.clone(),
            CanonExpr::Lit(l) => l.typ(),
            CanonExpr::BinOp(e) => match e.op {
                BinOp::Eq
                | BinOp::Ne
                | BinOp::Lt
                | BinOp::Le
                | BinOp::Gt
                | BinOp::Ge
                | BinOp::And
                | BinOp::Or => Type::Bool,
                _ => e.lhs.typ(),
            },
            CanonExpr::UnOp(e) => match e.op {
                UnOp::Not => Type::Bool,
                _ => e.operand.typ(),
            },
            CanonExpr::IndexAccess(e) => e.ty.clone(),
            CanonExpr::FieldAccess(e) => e.ty.clone(),
            CanonExpr::FunctionCall(e) => e.ty.clone(),
            CanonExpr::TypeCast(e) => e.ty.clone(),
            CanonExpr::Load(e) => e.ty.clone(),
            CanonExpr::InternalCall(e) => e.ty.clone(),
            CanonExpr::ExternalCall(e) => e.ty.clone(),
            CanonExpr::Env(e) => e.ty.clone(),
            CanonExpr::Old(inner) => inner.typ(),
            CanonExpr::Result(_) => Type::None,
            CanonExpr::Forall { .. } | CanonExpr::Exists { .. } => Type::Bool,
            CanonExpr::Dialect(e) => e.ty.clone(),
        }
    }

    pub fn span(&self) -> Option<&Loc> {
        match self {
            CanonExpr::Var(v) => v.span.as_ref(),
            CanonExpr::Lit(l) => l.span(),
            CanonExpr::BinOp(e) => e.span.as_ref(),
            CanonExpr::UnOp(e) => e.span.as_ref(),
            CanonExpr::IndexAccess(e) => e.span.as_ref(),
            CanonExpr::FieldAccess(e) => e.span.as_ref(),
            CanonExpr::FunctionCall(e) => e.span.as_ref(),
            CanonExpr::TypeCast(e) => e.span.as_ref(),
            CanonExpr::Load(e) => e.span.as_ref(),
            CanonExpr::InternalCall(e) => e.span.as_ref(),
            CanonExpr::ExternalCall(e) => e.span.as_ref(),
            CanonExpr::Env(e) => e.span.as_ref(),
            CanonExpr::Dialect(e) => e.span.as_ref(),
            CanonExpr::Old(_)
            | CanonExpr::Result(_)
            | CanonExpr::Forall { .. }
            | CanonExpr::Exists { .. } => None,
        }
    }
}

impl CanonVarExpr {
    pub fn new(name: String, ty: Type, span: Option<Loc>) -> Self {
        CanonVarExpr { name, ty, span }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display implementations
// ═══════════════════════════════════════════════════════════════════

impl Display for CanonExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonExpr::Var(v) => write!(f, "{}", v.name),
            CanonExpr::Lit(l) => write!(f, "{l}"),
            CanonExpr::BinOp(e) => write!(f, "({} {} {})", e.lhs, e.op, e.rhs),
            CanonExpr::UnOp(e) => write!(f, "({}{})", e.op, e.operand),
            CanonExpr::IndexAccess(e) => match &e.index {
                Some(idx) => write!(f, "{}[{}]", e.base, idx),
                None => write!(f, "{}[]", e.base),
            },
            CanonExpr::FieldAccess(e) => write!(f, "{}.{}", e.base, e.field),
            CanonExpr::FunctionCall(e) => {
                let args: Vec<_> = e.args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", e.callee, args.join(", "))
            }
            CanonExpr::TypeCast(e) => write!(f, "{}({})", e.ty, e.expr),
            CanonExpr::Load(e) => {
                write!(f, "load {}", e.resource)?;
                for key in &e.keys {
                    write!(f, "[{key}]")?;
                }
                Ok(())
            }
            CanonExpr::InternalCall(e) => write!(f, "{}({})", e.func, join(&e.args)),
            CanonExpr::ExternalCall(e) => {
                write!(f, "external.{:?}", e.kind)?;
                if let Some(address) = &e.address {
                    write!(f, " {address}")?;
                }
                write!(f, "({})", join(&e.args))?;
                if let Some(value) = &e.value {
                    write!(f, " value {value}")?;
                }
                Ok(())
            }
            CanonExpr::Env(e) => write!(f, "env.{}", e.var),
            CanonExpr::Old(inner) => write!(f, "old({inner})"),
            CanonExpr::Result(idx) => write!(f, "result({idx})"),
            CanonExpr::Forall { var, ty, body } => write!(f, "forall({var}: {ty}, {body})"),
            CanonExpr::Exists { var, ty, body } => write!(f, "exists({var}: {ty}, {body})"),
            CanonExpr::Dialect(d) => write!(f, "{d}"),
        }
    }
}

impl Display for CanonResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonResource::AnchorAccount(account) => write!(f, "anchor.account<{account}>"),
            CanonResource::MoveGlobal(ty) => write!(f, "move.global<{ty}>"),
            CanonResource::StateVar(name) => write!(f, "@{name}"),
        }
    }
}

fn join(exprs: &[CanonExpr]) -> String {
    exprs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
