//! Vyper AST expression nodes.

use super::loc::Loc;
use super::types::Type;
use std::fmt::{self, Display};

/// Identifier (variable name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub loc: Option<Loc>,
}

/// Expression nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    // ── Basic ───────────────────────────────────────
    Ident(Identifier),
    Lit(Lit),
    Attribute(AttributeExpr),
    Subscript(SubscriptExpr),
    Call(CallExpr),

    // ── Operators ───────────────────────────────────
    BinOp(BinOpExpr),
    BoolOp(BoolOpExpr),
    Compare(CompareExpr),
    UnaryOp(UnaryOpExpr),

    // ── Vyper builtins ──────────────────────────────
    Convert {
        expr: Box<Expr>,
        to: Type,
        loc: Option<Loc>,
    },
    Empty(Type, Option<Loc>),
    Len(Box<Expr>, Option<Loc>),
    Concat(Vec<Expr>, Option<Loc>),
    Slice {
        expr: Box<Expr>,
        start: Box<Expr>,
        length: Box<Expr>,
        loc: Option<Loc>,
    },
    RawCall {
        target: Box<Expr>,
        data: Box<Expr>,
        value: Option<Box<Expr>>,
        gas: Option<Box<Expr>>,
        loc: Option<Loc>,
    },
    Send {
        target: Box<Expr>,
        value: Box<Expr>,
        loc: Option<Loc>,
    },
    Keccak256(Box<Expr>, Option<Loc>),
    Sha256(Box<Expr>, Option<Loc>),

    // ── Tuple ───────────────────────────────────────
    Tuple(Vec<Expr>, Option<Loc>),

    // ── Ternary (if-expression) ─────────────────────
    IfExp(IfExpExpr),
}

/// A literal value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lit {
    pub kind: LitKind,
    pub loc: Option<Loc>,
}

/// Literal kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LitKind {
    Bool(bool),
    Int(i128),
    Str(String),
    Hex(String),
    Bytes(Vec<u8>),
}

/// Attribute access: `obj.field` (e.g., `self.x`, `msg.sender`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeExpr {
    pub value: Box<Expr>,
    pub attr: String,
    pub loc: Option<Loc>,
}

/// Subscript access: `arr[i]`, `map[k]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptExpr {
    pub value: Box<Expr>,
    pub index: Box<Expr>,
    pub loc: Option<Loc>,
}

/// Function call: `f(args)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallExpr {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
    pub keywords: Vec<Keyword>,
    pub loc: Option<Loc>,
}

/// Keyword argument in a function call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyword {
    pub name: String,
    pub value: Expr,
}

/// Binary operation: `left op right`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOpExpr {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
    pub loc: Option<Loc>,
}

/// Boolean operation: `and` / `or`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolOpExpr {
    pub op: BoolOp,
    pub values: Vec<Expr>,
    pub loc: Option<Loc>,
}

/// Comparison: `left op right`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareExpr {
    pub left: Box<Expr>,
    pub ops: Vec<CmpOp>,
    pub comparators: Vec<Expr>,
    pub loc: Option<Loc>,
}

/// Unary operation: `not x`, `-x`, `~x`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub loc: Option<Loc>,
}

/// If expression (ternary): `x if cond else y`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfExpExpr {
    pub test: Box<Expr>,
    pub body: Box<Expr>,
    pub orelse: Box<Expr>,
    pub loc: Option<Loc>,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
    Invert,
}

/// Boolean operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOp {
    And,
    Or,
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    NotEq,
    Lt,
    LtE,
    Gt,
    GtE,
    In,
    NotIn,
}

impl Expr {
    /// Get the source location of this expression.
    pub fn loc(&self) -> Option<&Loc> {
        match self {
            Expr::Ident(id) => id.loc.as_ref(),
            Expr::Lit(lit) => lit.loc.as_ref(),
            Expr::Attribute(e) => e.loc.as_ref(),
            Expr::Subscript(e) => e.loc.as_ref(),
            Expr::Call(e) => e.loc.as_ref(),
            Expr::BinOp(e) => e.loc.as_ref(),
            Expr::BoolOp(e) => e.loc.as_ref(),
            Expr::Compare(e) => e.loc.as_ref(),
            Expr::UnaryOp(e) => e.loc.as_ref(),
            Expr::Convert { loc, .. } => loc.as_ref(),
            Expr::Empty(_, loc) => loc.as_ref(),
            Expr::Len(_, loc) => loc.as_ref(),
            Expr::Concat(_, loc) => loc.as_ref(),
            Expr::Slice { loc, .. } => loc.as_ref(),
            Expr::RawCall { loc, .. } => loc.as_ref(),
            Expr::Send { loc, .. } => loc.as_ref(),
            Expr::Keccak256(_, loc) => loc.as_ref(),
            Expr::Sha256(_, loc) => loc.as_ref(),
            Expr::Tuple(_, loc) => loc.as_ref(),
            Expr::IfExp(e) => e.loc.as_ref(),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::FloorDiv => write!(f, "//"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Pow => write!(f, "**"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
        }
    }
}

impl Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CmpOp::Eq => write!(f, "=="),
            CmpOp::NotEq => write!(f, "!="),
            CmpOp::Lt => write!(f, "<"),
            CmpOp::LtE => write!(f, "<="),
            CmpOp::Gt => write!(f, ">"),
            CmpOp::GtE => write!(f, ">="),
            CmpOp::In => write!(f, "in"),
            CmpOp::NotIn => write!(f, "not in"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Ident(id) => write!(f, "{}", id.name),
            Expr::Lit(lit) => write!(f, "{}", lit.kind),
            Expr::Attribute(e) => write!(f, "{}.{}", e.value, e.attr),
            Expr::Subscript(e) => write!(f, "{}[{}]", e.value, e.index),
            Expr::Call(e) => {
                let args: Vec<_> = e.args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", e.func, args.join(", "))
            }
            Expr::BinOp(e) => write!(f, "({} {} {})", e.left, e.op, e.right),
            Expr::BoolOp(e) => {
                let op = match e.op {
                    BoolOp::And => "and",
                    BoolOp::Or => "or",
                };
                let vals: Vec<_> = e.values.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", vals.join(&format!(" {op} ")))
            }
            Expr::Compare(e) => {
                write!(f, "({}", e.left)?;
                for (op, cmp) in e.ops.iter().zip(e.comparators.iter()) {
                    write!(f, " {op} {cmp}")?;
                }
                write!(f, ")")
            }
            Expr::UnaryOp(e) => {
                let op = match e.op {
                    UnaryOp::Not => "not ",
                    UnaryOp::Neg => "-",
                    UnaryOp::Invert => "~",
                };
                write!(f, "{op}{}", e.operand)
            }
            Expr::Convert { expr, to, .. } => write!(f, "convert({expr}, {to})"),
            Expr::Empty(ty, _) => write!(f, "empty({ty})"),
            Expr::Len(e, _) => write!(f, "len({e})"),
            Expr::Concat(parts, _) => {
                let ps: Vec<_> = parts.iter().map(|p| p.to_string()).collect();
                write!(f, "concat({})", ps.join(", "))
            }
            Expr::Slice { expr, start, length, .. } => {
                write!(f, "slice({expr}, {start}, {length})")
            }
            Expr::RawCall { target, data, .. } => write!(f, "raw_call({target}, {data})"),
            Expr::Send { target, value, .. } => write!(f, "send({target}, {value})"),
            Expr::Keccak256(e, _) => write!(f, "keccak256({e})"),
            Expr::Sha256(e, _) => write!(f, "sha256({e})"),
            Expr::Tuple(elems, _) => {
                let es: Vec<_> = elems.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", es.join(", "))
            }
            Expr::IfExp(e) => write!(f, "{} if {} else {}", e.body, e.test, e.orelse),
        }
    }
}

impl Display for LitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LitKind::Bool(b) => write!(f, "{b}"),
            LitKind::Int(i) => write!(f, "{i}"),
            LitKind::Str(s) => write!(f, "\"{s}\""),
            LitKind::Hex(h) => write!(f, "0x{h}"),
            LitKind::Bytes(b) => write!(f, "b\"{}\"", hex::encode(b)),
        }
    }
}

/// Hex encoding helper (we avoid adding hex crate just for display).
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
