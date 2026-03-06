//! SSA Ops for ANIR.
//!
//! Each SCIR statement/expression lowers to one or more `Op` values.

use crate::cfg::BlockId;
use crate::interfaces::{
    AliasGroupId, CallRisk, CallTarget, SinkCategory, StorageRef, TaintLabel,
};
use scir::{Attr, BinOp, Lit, OverflowSemantics, Span, Type, UnOp};
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// ID types
// ═══════════════════════════════════════════════════════════════════

/// A unique identifier for an Op within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpId(pub usize);

/// A reference to an Op result (used as operand).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpRef(pub OpId);

/// An SSA name: `{original_name}_{version}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SsaName {
    pub base: String,
    pub version: u32,
}

/// A parameter index in a function signature.
pub type ParamIndex = usize;

/// A return value index.
pub type ReturnIndex = usize;

impl Display for OpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl Display for OpRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SsaName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.base, self.version)
    }
}

impl SsaName {
    pub fn new(base: &str, version: u32) -> Self {
        SsaName { base: base.to_string(), version }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Op
// ═══════════════════════════════════════════════════════════════════

/// An SSA operation (value node) in ANIR.
#[derive(Debug, Clone)]
pub struct Op {
    pub id: OpId,
    pub kind: OpKind,
    pub result: Option<(SsaName, Type)>,
    pub attrs: Vec<Attr>,
    pub span: Option<Span>,
}

impl Op {
    pub fn new(id: OpId, kind: OpKind) -> Self {
        Op { id, kind, result: None, attrs: vec![], span: None }
    }

    pub fn with_result(mut self, name: SsaName, ty: Type) -> Self {
        self.result = Some((name, ty));
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_attrs(mut self, attrs: Vec<Attr>) -> Self {
        self.attrs = attrs;
        self
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((name, ty)) = &self.result {
            write!(f, "{}: {ty} = ", name)?;
        }
        write!(f, "{}", self.kind)
    }
}

// ═══════════════════════════════════════════════════════════════════
// OpKind
// ═══════════════════════════════════════════════════════════════════

/// The kind of SSA operation.
#[derive(Debug, Clone)]
pub enum OpKind {
    // ── Core ops ──────────────────────────────────────────────
    /// A constant literal value.
    Const(Lit),
    /// Binary operation.
    BinOp {
        op: BinOp,
        lhs: OpRef,
        rhs: OpRef,
        overflow: OverflowSemantics,
    },
    /// Unary operation.
    UnOp { op: UnOp, operand: OpRef },
    /// Phi function (block arguments).
    Phi(Vec<(BlockId, OpRef)>),
    /// Assertion.
    Assert { cond: OpRef },
    /// Return from function.
    Return(Vec<OpRef>),
    /// Function parameter.
    Param { index: ParamIndex },
    /// Expression statement (side effects only).
    ExprStmt { expr: OpRef },

    // ── Dialect ops (lowered; implement one of the four interfaces) ──
    /// A storage operation (read or write).
    Storage(StorageDialectOp),
    /// A call operation (internal or external).
    Call(CallDialectOp),
    /// A taint source pseudo-value.
    TaintSrc(TaintSourceOp),
    /// A taint sink operation.
    TaintSnk(TaintSinkOp),

    // ── SSA pseudo-values ────────────────────────────────────
    /// A pseudo-value with a label and taint info.
    PseudoValue { label: String, taint: TaintLabel },

    /// Opaque (dialect op that has been lowered but retained).
    Opaque { description: String },
}

impl Display for OpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpKind::Const(lit) => write!(f, "const {lit}"),
            OpKind::BinOp { op, lhs, rhs, overflow } => {
                write!(f, "binop {op} {lhs}, {rhs} [{overflow:?}]")
            }
            OpKind::UnOp { op, operand } => write!(f, "unop {op} {operand}"),
            OpKind::Phi(args) => {
                let parts: Vec<_> = args.iter().map(|(bb, r)| format!("{bb}: {r}")).collect();
                write!(f, "phi [{}]", parts.join(", "))
            }
            OpKind::Assert { cond } => write!(f, "assert {cond}"),
            OpKind::Return(vals) => {
                let parts: Vec<_> = vals.iter().map(|r| r.to_string()).collect();
                write!(f, "return {}", parts.join(", "))
            }
            OpKind::Param { index } => write!(f, "param {index}"),
            OpKind::ExprStmt { expr } => write!(f, "expr_stmt {expr}"),
            OpKind::Storage(s) => write!(f, "{s}"),
            OpKind::Call(c) => write!(f, "{c}"),
            OpKind::TaintSrc(t) => write!(f, "{t}"),
            OpKind::TaintSnk(t) => write!(f, "{t}"),
            OpKind::PseudoValue { label, taint } => {
                write!(f, "pseudo_value \"{label}\" [{taint}]")
            }
            OpKind::Opaque { description } => write!(f, "opaque({description})"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Concrete dialect ops (enum-based, not Box<dyn Trait>)
// ═══════════════════════════════════════════════════════════════════

/// A concrete storage dialect operation.
#[derive(Debug, Clone)]
pub struct StorageDialectOp {
    pub storage_ref: StorageRef,
    pub is_write: bool,
    pub alias_group_id: AliasGroupId,
    pub key_operand: Option<OpRef>,
    pub value_operand: Option<OpRef>,
    pub dialect_name: String,
    pub op_name: String,
}

impl Display for StorageDialectOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rw = if self.is_write { "write" } else { "read" };
        write!(f, "{}.{} [{rw}] {}", self.dialect_name, self.op_name, self.storage_ref)
    }
}

/// A concrete call dialect operation.
#[derive(Debug, Clone)]
pub struct CallDialectOp {
    pub callee: CallTarget,
    pub call_risk: CallRisk,
    pub args: Vec<OpRef>,
    pub dialect_name: String,
    pub op_name: String,
}

impl Display for CallDialectOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{} callee={}", self.dialect_name, self.op_name, self.callee)
    }
}

/// A concrete taint source operation.
#[derive(Debug, Clone)]
pub struct TaintSourceOp {
    pub label: TaintLabel,
    pub dialect_name: String,
    pub op_name: String,
}

impl Display for TaintSourceOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{} [taint: {}]", self.dialect_name, self.op_name, self.label)
    }
}

/// A concrete taint sink operation.
#[derive(Debug, Clone)]
pub struct TaintSinkOp {
    pub category: SinkCategory,
    pub dialect_name: String,
    pub op_name: String,
}

impl Display for TaintSinkOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{} [sink: {:?}]", self.dialect_name, self.op_name, self.category)
    }
}
