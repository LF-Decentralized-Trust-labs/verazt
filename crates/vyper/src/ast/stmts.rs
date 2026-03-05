//! Vyper AST statement nodes.

use super::exprs::Expr;
use super::loc::Loc;

/// Statement nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Assign(AssignStmt),
    AugAssign(AugAssignStmt),
    AnnAssign(AnnAssignStmt),
    If(IfStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Assert(AssertStmt),
    Raise(RaiseStmt),
    Log(LogStmt),
    Pass(Option<Loc>),
    Break(Option<Loc>),
    Continue(Option<Loc>),
    Expr(ExprStmt),
}

/// Assignment: `x = expr`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignStmt {
    pub target: Expr,
    pub value: Expr,
    pub loc: Option<Loc>,
}

/// Augmented assignment: `x += expr`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AugAssignStmt {
    pub target: Expr,
    pub op: super::exprs::BinOp,
    pub value: Expr,
    pub loc: Option<Loc>,
}

/// Annotated assignment (local var decl): `x: T = expr`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnAssignStmt {
    pub target: Expr,
    pub annotation: super::types::Type,
    pub value: Option<Expr>,
    pub loc: Option<Loc>,
}

/// If statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_body: Vec<Stmt>,
    pub else_body: Vec<Stmt>,
    pub loc: Option<Loc>,
}

/// For loop
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStmt {
    pub target: Expr,
    pub iter: ForIter,
    pub body: Vec<Stmt>,
    pub loc: Option<Loc>,
}

/// For loop iterator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForIter {
    Range(RangeIter),
    Iterable(Box<Expr>),
}

/// Range iterator: `range(start, stop)` or `range(stop)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeIter {
    pub start: Option<Box<Expr>>,
    pub stop: Box<Expr>,
}

/// Return statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub loc: Option<Loc>,
}

/// Assert statement: `assert cond, "msg"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertStmt {
    pub test: Expr,
    pub msg: Option<Expr>,
    pub loc: Option<Loc>,
}

/// Raise statement (revert)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaiseStmt {
    pub exc: Option<Expr>,
    pub loc: Option<Loc>,
}

/// Log statement: `log EventName(args)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogStmt {
    pub event: Expr,
    pub loc: Option<Loc>,
}

/// Expression statement: `expr`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprStmt {
    pub value: Expr,
    pub loc: Option<Loc>,
}

impl Stmt {
    /// Get the source location of this statement.
    pub fn loc(&self) -> Option<&Loc> {
        match self {
            Stmt::Assign(s) => s.loc.as_ref(),
            Stmt::AugAssign(s) => s.loc.as_ref(),
            Stmt::AnnAssign(s) => s.loc.as_ref(),
            Stmt::If(s) => s.loc.as_ref(),
            Stmt::For(s) => s.loc.as_ref(),
            Stmt::Return(s) => s.loc.as_ref(),
            Stmt::Assert(s) => s.loc.as_ref(),
            Stmt::Raise(s) => s.loc.as_ref(),
            Stmt::Log(s) => s.loc.as_ref(),
            Stmt::Pass(loc) => loc.as_ref(),
            Stmt::Break(loc) => loc.as_ref(),
            Stmt::Continue(loc) => loc.as_ref(),
            Stmt::Expr(s) => s.loc.as_ref(),
        }
    }
}
