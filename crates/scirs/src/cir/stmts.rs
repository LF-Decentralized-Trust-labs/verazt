//! CIR statement forms.
//!
//! Structurally identical to SIR statements, but using `CanonExpr` instead of
//! `sir::Expr` and `CanonStmt` recursively.

use crate::cir::exprs::CanonExpr;
use crate::sir::dialect::DialectStmt;
use crate::sir::exprs::BinOp;
use crate::sir::types::Type;
use common::loc::Loc;
use common::string::StringExt;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Core statement enum
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonStmt {
    LocalVar(CanonLocalVarStmt),
    Assign(CanonAssignStmt),
    AugAssign(CanonAugAssignStmt),
    Expr(CanonExprStmt),
    If(CanonIfStmt),
    While(CanonWhileStmt),
    For(CanonForStmt),
    Return(CanonReturnStmt),
    Revert(CanonRevertStmt),
    Assert(CanonAssertStmt),
    Break,
    Continue,
    Block(Vec<CanonStmt>),
    Dialect(DialectStmt),
}

// ═══════════════════════════════════════════════════════════════════
// Statement sub-types
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonLocalVarStmt {
    pub vars: Vec<Option<CanonLocalVarDecl>>,
    pub init: Option<CanonExpr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonLocalVarDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonAssignStmt {
    pub lhs: CanonExpr,
    pub rhs: CanonExpr,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonAugAssignStmt {
    pub op: BinOp,
    pub lhs: CanonExpr,
    pub rhs: CanonExpr,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonExprStmt {
    pub expr: CanonExpr,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonIfStmt {
    pub cond: CanonExpr,
    pub then_body: Vec<CanonStmt>,
    pub else_body: Option<Vec<CanonStmt>>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonWhileStmt {
    pub cond: CanonExpr,
    pub body: Vec<CanonStmt>,
    pub invariant: Option<CanonExpr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonForStmt {
    pub init: Option<Box<CanonStmt>>,
    pub cond: Option<CanonExpr>,
    pub update: Option<Box<CanonStmt>>,
    pub body: Vec<CanonStmt>,
    pub invariant: Option<CanonExpr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonReturnStmt {
    pub value: Option<CanonExpr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonRevertStmt {
    pub error: Option<String>,
    pub args: Vec<CanonExpr>,
    pub span: Option<Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonAssertStmt {
    pub cond: CanonExpr,
    pub message: Option<CanonExpr>,
    pub span: Option<Loc>,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl CanonStmt {
    pub fn span(&self) -> Option<&Loc> {
        match self {
            CanonStmt::LocalVar(s) => s.span.as_ref(),
            CanonStmt::Assign(s) => s.span.as_ref(),
            CanonStmt::AugAssign(s) => s.span.as_ref(),
            CanonStmt::Expr(s) => s.span.as_ref(),
            CanonStmt::If(s) => s.span.as_ref(),
            CanonStmt::While(s) => s.span.as_ref(),
            CanonStmt::For(s) => s.span.as_ref(),
            CanonStmt::Return(s) => s.span.as_ref(),
            CanonStmt::Revert(s) => s.span.as_ref(),
            CanonStmt::Assert(s) => s.span.as_ref(),
            CanonStmt::Break | CanonStmt::Continue | CanonStmt::Block(_) => None,
            CanonStmt::Dialect(_) => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display implementations
// ═══════════════════════════════════════════════════════════════════

impl Display for CanonStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonStmt::LocalVar(s) => write!(f, "{s}"),
            CanonStmt::Assign(s) => write!(f, "{s}"),
            CanonStmt::AugAssign(s) => write!(f, "{s}"),
            CanonStmt::Expr(s) => write!(f, "{s}"),
            CanonStmt::If(s) => write!(f, "{s}"),
            CanonStmt::While(s) => write!(f, "{s}"),
            CanonStmt::For(s) => write!(f, "{s}"),
            CanonStmt::Return(s) => write!(f, "{s}"),
            CanonStmt::Revert(s) => write!(f, "{s}"),
            CanonStmt::Assert(s) => write!(f, "{s}"),
            CanonStmt::Break => write!(f, "break;"),
            CanonStmt::Continue => write!(f, "continue;"),
            CanonStmt::Block(stmts) => {
                writeln!(f, "{{")?;
                for s in stmts {
                    writeln!(f, "{}", format!("{s}").indent(2))?;
                }
                write!(f, "}}")
            }
            CanonStmt::Dialect(s) => write!(f, "{s}"),
        }
    }
}

impl Display for CanonLocalVarStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.len() == 1 {
            if let Some(v) = &self.vars[0] {
                write!(f, "{} {}", v.ty, v.name)?;
            }
        } else {
            let vs: Vec<_> = self
                .vars
                .iter()
                .map(|v| match v {
                    Some(d) => format!("{} {}", d.ty, d.name),
                    None => String::new(),
                })
                .collect();
            write!(f, "({})", vs.join(", "))?;
        }
        match &self.init {
            Some(e) => write!(f, " = {e};"),
            None => write!(f, ";"),
        }
    }
}

impl Display for CanonAssignStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {};", self.lhs, self.rhs)
    }
}

impl Display for CanonAugAssignStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}= {};", self.lhs, self.op, self.rhs)
    }
}

impl Display for CanonExprStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{};", self.expr)
    }
}

impl Display for CanonIfStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) ", self.cond)?;
        write!(f, "{{")?;
        if !self.then_body.is_empty() {
            writeln!(f)?;
            for s in &self.then_body {
                writeln!(f, "{}", format!("{s}").indent(2))?;
            }
        }
        write!(f, "}}")?;
        if let Some(else_body) = &self.else_body {
            write!(f, " else {{")?;
            if !else_body.is_empty() {
                writeln!(f)?;
                for s in else_body {
                    writeln!(f, "{}", format!("{s}").indent(2))?;
                }
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl Display for CanonWhileStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while ({}) {{", self.cond)?;
        if !self.body.is_empty() {
            writeln!(f)?;
            for s in &self.body {
                writeln!(f, "{}", format!("{s}").indent(2))?;
            }
        }
        write!(f, "}}")
    }
}

impl Display for CanonForStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "for (")?;
        if let Some(init) = &self.init {
            write!(f, "{init}")?;
        }
        write!(f, " ")?;
        if let Some(cond) = &self.cond {
            write!(f, "{cond}")?;
        }
        write!(f, "; ")?;
        if let Some(update) = &self.update {
            write!(f, "{update}")?;
        }
        write!(f, ") {{")?;
        if !self.body.is_empty() {
            writeln!(f)?;
            for s in &self.body {
                writeln!(f, "{}", format!("{s}").indent(2))?;
            }
        }
        write!(f, "}}")
    }
}

impl Display for CanonReturnStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(e) => write!(f, "return {e};"),
            None => write!(f, "return;"),
        }
    }
}

impl Display for CanonRevertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<_> = self.args.iter().map(|a| a.to_string()).collect();
        match &self.error {
            Some(error) => write!(f, "revert {error}({});", args.join(", ")),
            None => write!(f, "revert({});", args.join(", ")),
        }
    }
}

impl Display for CanonAssertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "assert({})", self.cond)?;
        if let Some(msg) = &self.message {
            write!(f, ", {msg}")?;
        }
        write!(f, ";")
    }
}
