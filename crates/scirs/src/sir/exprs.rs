//! SIR expression forms.

use crate::sir::dialect::DialectExpr;
use crate::sir::lits::Lit;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Core expression enum
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Var(VarExpr),
    Lit(Lit),
    BinOp(BinOpExpr),
    UnOp(UnOpExpr),
    IndexAccess(IndexAccessExpr),
    FieldAccess(FieldAccessExpr),
    FunctionCall(CallExpr),
    TypeCast(TypeCastExpr),
    Ternary(TernaryExpr),
    Tuple(TupleExpr),

    // ── Spec-only expressions (valid in @requires/@ensures) ────
    Old(Box<Expr>),
    Result(u32),
    Forall {
        var: String,
        ty: Type,
        body: Box<Expr>,
    },
    Exists {
        var: String,
        ty: Type,
        body: Box<Expr>,
    },

    // ── Dialect extension ──────────────────────────────────────
    Dialect(DialectExpr),
}

// ═══════════════════════════════════════════════════════════════════
// Expression sub-types
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarExpr {
    pub name: String,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOpExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub overflow: OverflowSemantics,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnOpExpr {
    pub op: UnOp,
    pub operand: Box<Expr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexAccessExpr {
    pub base: Box<Expr>,
    pub index: Option<Box<Expr>>,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldAccessExpr {
    pub base: Box<Expr>,
    pub field: String,
    pub ty: Type,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: CallArgs,
    pub ty: Type,
    pub span: Option<Loc>,
}

/// Call arguments: either positional (evaluated in order) or named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArgs {
    Positional(Vec<Expr>),
    Named(Vec<NamedArg>),
}

/// A single named argument in a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedArg {
    pub name: String,
    pub value: Expr,
}

impl CallArgs {
    pub fn positional(args: Vec<Expr>) -> Self {
        CallArgs::Positional(args)
    }

    pub fn exprs(&self) -> Vec<&Expr> {
        match self {
            CallArgs::Positional(args) => args.iter().collect(),
            CallArgs::Named(named) => named.iter().map(|n| &n.value).collect(),
        }
    }

    pub fn into_positional(self) -> Vec<Expr> {
        match self {
            CallArgs::Positional(args) => args,
            CallArgs::Named(named) => named.into_iter().map(|n| n.value).collect(),
        }
    }

    pub fn is_named(&self) -> bool {
        matches!(self, CallArgs::Named(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCastExpr {
    pub ty: Type,
    pub expr: Box<Expr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TernaryExpr {
    pub cond: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Box<Expr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleExpr {
    pub elems: Vec<Option<Expr>>,
    pub ty: Type,
    pub span: Option<Loc>,
}

// ═══════════════════════════════════════════════════════════════════
// Binary and Unary operators
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    /// Arithmetic negation: `-x`
    Neg,
    /// Logical not: `!x`
    Not,
    /// Bitwise not: `~x`
    BitNot,
    /// Delete operator (Solidity-specific, zeroes storage)
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowSemantics {
    /// Panic/revert on overflow (Solidity ≥0.8, Vyper 0.3.9+)
    Checked,
    /// Silent wrap (Solidity <0.8, Move u64, Rust release)
    Wrapping,
    /// Clamp at max/min
    Saturating,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl Expr {
    pub fn typ(&self) -> Type {
        match self {
            Expr::Var(v) => v.ty.clone(),
            Expr::Lit(l) => l.typ(),
            Expr::BinOp(e) => {
                // For comparison/logical ops, result is bool; otherwise inherit from lhs
                match e.op {
                    BinOp::Eq
                    | BinOp::Ne
                    | BinOp::Lt
                    | BinOp::Le
                    | BinOp::Gt
                    | BinOp::Ge
                    | BinOp::And
                    | BinOp::Or => Type::Bool,
                    _ => e.lhs.typ(),
                }
            }
            Expr::UnOp(e) => match e.op {
                UnOp::Not => Type::Bool,
                _ => e.operand.typ(),
            },
            Expr::IndexAccess(e) => e.ty.clone(),
            Expr::FieldAccess(e) => e.ty.clone(),
            Expr::FunctionCall(e) => e.ty.clone(),
            Expr::TypeCast(e) => e.ty.clone(),
            Expr::Ternary(e) => e.then_expr.typ(),
            Expr::Tuple(e) => e.ty.clone(),
            Expr::Old(inner) => inner.typ(),
            Expr::Result(_) => Type::None, // determined by function signature
            Expr::Forall { .. } | Expr::Exists { .. } => Type::Bool,
            Expr::Dialect(d) => d.typ(),
        }
    }

    pub fn span(&self) -> Option<Loc> {
        match self {
            Expr::Var(v) => v.span,
            Expr::Lit(l) => l.span(),
            Expr::BinOp(e) => e.span,
            Expr::UnOp(e) => e.span,
            Expr::IndexAccess(e) => e.span,
            Expr::FieldAccess(e) => e.span,
            Expr::FunctionCall(e) => e.span,
            Expr::TypeCast(e) => e.span,
            Expr::Ternary(e) => e.span,
            Expr::Tuple(e) => e.span,
            _ => None,
        }
    }
}

impl From<Lit> for Expr {
    fn from(l: Lit) -> Self {
        Expr::Lit(l)
    }
}
impl From<VarExpr> for Expr {
    fn from(v: VarExpr) -> Self {
        Expr::Var(v)
    }
}
impl From<BinOpExpr> for Expr {
    fn from(e: BinOpExpr) -> Self {
        Expr::BinOp(e)
    }
}
impl From<UnOpExpr> for Expr {
    fn from(e: UnOpExpr) -> Self {
        Expr::UnOp(e)
    }
}
impl From<IndexAccessExpr> for Expr {
    fn from(e: IndexAccessExpr) -> Self {
        Expr::IndexAccess(e)
    }
}
impl From<FieldAccessExpr> for Expr {
    fn from(e: FieldAccessExpr) -> Self {
        Expr::FieldAccess(e)
    }
}
impl From<CallExpr> for Expr {
    fn from(e: CallExpr) -> Self {
        Expr::FunctionCall(e)
    }
}
impl From<TypeCastExpr> for Expr {
    fn from(e: TypeCastExpr) -> Self {
        Expr::TypeCast(e)
    }
}
impl From<TernaryExpr> for Expr {
    fn from(e: TernaryExpr) -> Self {
        Expr::Ternary(e)
    }
}
impl From<TupleExpr> for Expr {
    fn from(e: TupleExpr) -> Self {
        Expr::Tuple(e)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(v) => write!(f, "{v}"),
            Expr::Lit(l) => write!(f, "{l}"),
            Expr::BinOp(e) => write!(f, "{e}"),
            Expr::UnOp(e) => write!(f, "{e}"),
            Expr::IndexAccess(e) => write!(f, "{e}"),
            Expr::FieldAccess(e) => write!(f, "{e}"),
            Expr::FunctionCall(e) => write!(f, "{e}"),
            Expr::TypeCast(e) => write!(f, "{e}"),
            Expr::Ternary(e) => write!(f, "{e}"),
            Expr::Tuple(e) => write!(f, "{e}"),
            Expr::Old(inner) => write!(f, "old({inner})"),
            Expr::Result(idx) => write!(f, "result({idx})"),
            Expr::Forall { var, ty, body } => write!(f, "forall({var}: {ty}, {body})"),
            Expr::Exists { var, ty, body } => write!(f, "exists({var}: {ty}, {body})"),
            Expr::Dialect(d) => write!(f, "{d}"),
        }
    }
}

// ─── Sub-type Display implementations ─────────────────────────────

impl VarExpr {
    pub fn new(name: String, ty: Type, span: Option<Loc>) -> Self {
        VarExpr { name, ty, span }
    }
}

impl Display for VarExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for BinOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {} {})", self.lhs, self.op, self.rhs)
    }
}

impl Display for UnOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{})", self.op, self.operand)
    }
}

impl Display for IndexAccessExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.index {
            Some(idx) => write!(f, "{}[{}]", self.base, idx),
            None => write!(f, "{}[]", self.base),
        }
    }
}

impl Display for FieldAccessExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.base, self.field)
    }
}

impl Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<_> = match &self.args {
            CallArgs::Positional(args) => args.iter().map(|a| a.to_string()).collect(),
            CallArgs::Named(named) => named
                .iter()
                .map(|n| format!("{}: {}", n.name, n.value))
                .collect(),
        };
        write!(f, "{}({})", self.callee, args.join(", "))
    }
}

impl Display for TypeCastExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.ty, self.expr)
    }
}

impl Display for TernaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ? {} : {}", self.cond, self.then_expr, self.else_expr)
    }
}

impl Display for TupleExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elems: Vec<_> = self
            .elems
            .iter()
            .map(|e| match e {
                Some(expr) => expr.to_string(),
                None => String::new(),
            })
            .collect();
        write!(f, "({})", elems.join(", "))
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        };
        write!(f, "{s}")
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnOp::Neg => "-",
            UnOp::Not => "!",
            UnOp::BitNot => "~",
            UnOp::Delete => "delete ",
        };
        write!(f, "{s}")
    }
}
