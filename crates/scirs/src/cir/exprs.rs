//! CIR expression forms.
//!
//! Key differences from SIR expressions:
//! - `Ternary` is removed (lowered to `if` statement).
//! - `Tuple` is removed (unrolled).
//! - `FunctionCall` args must be atoms (Var or Lit) — no nested calls.

use crate::sir::dialect::DialectExpr;
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
    /// Function call — args must be atoms (Var or Lit).
    FunctionCall(CanonCallExpr),
    TypeCast(CanonTypeCastExpr),

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

    // ── Dialect extension ──────────────────────────────────────
    Dialect(DialectExpr),
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
            CanonExpr::Old(inner) => inner.typ(),
            CanonExpr::Result(_) => Type::None,
            CanonExpr::Forall { .. } | CanonExpr::Exists { .. } => Type::Bool,
            CanonExpr::Dialect(_) => Type::None,
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
            _ => None,
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
            CanonExpr::Old(inner) => write!(f, "old({inner})"),
            CanonExpr::Result(idx) => write!(f, "result({idx})"),
            CanonExpr::Forall { var, ty, body } => write!(f, "forall({var}: {ty}, {body})"),
            CanonExpr::Exists { var, ty, body } => write!(f, "exists({var}: {ty}, {body})"),
            CanonExpr::Dialect(d) => write!(f, "{d}"),
        }
    }
}
