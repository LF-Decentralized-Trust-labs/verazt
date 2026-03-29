//! SIR statement forms.

use crate::sir::dialect::DialectStmt;
use crate::sir::exprs::{BinOp, Expr};
use crate::sir::types::Type;
use common::loc::Loc;
use common::string::StringExt;
use std::fmt::{self, Display};

// ═══════════════════════════════════════════════════════════════════
// Core statement enum
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    LocalVar(LocalVarStmt),
    Assign(AssignStmt),
    AugAssign(AugAssignStmt),
    Expr(ExprStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Revert(RevertStmt),
    Assert(AssertStmt),
    Break,
    Continue,
    Block(Vec<Stmt>),
    Dialect(DialectStmt),
}

// ═══════════════════════════════════════════════════════════════════
// Statement sub-types
// ═══════════════════════════════════════════════════════════════════

/// Local variable declaration: `let x: T = init`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVarStmt {
    pub vars: Vec<Option<LocalVarDecl>>,
    pub init: Option<Expr>,
    pub span: Option<Loc>,
}

/// A single local variable declaration within a LocalVarStmt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVarDecl {
    pub name: String,
    pub ty: Type,
}

/// Assignment: `lhs = rhs`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignStmt {
    pub lhs: Expr,
    pub rhs: Expr,
    pub span: Option<Loc>,
}

/// Augmented assignment: `lhs += rhs`, `lhs -= rhs`, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AugAssignStmt {
    pub op: BinOp,
    pub lhs: Expr,
    pub rhs: Expr,
    pub span: Option<Loc>,
}

/// Expression statement: `expr;`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Option<Loc>,
}

/// If statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_body: Vec<Stmt>,
    pub else_body: Option<Vec<Stmt>>,
    pub span: Option<Loc>,
}

/// While loop with optional loop invariant annotation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Vec<Stmt>,
    pub invariant: Option<Expr>,
    pub span: Option<Loc>,
}

/// For loop
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStmt {
    pub init: Option<Box<Stmt>>,
    pub cond: Option<Expr>,
    pub update: Option<Box<Stmt>>,
    pub body: Vec<Stmt>,
    pub invariant: Option<Expr>,
    pub span: Option<Loc>,
}

/// Return statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Option<Loc>,
}

/// Abstract failure/revert
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevertStmt {
    pub error: Option<String>,
    pub args: Vec<Expr>,
    pub span: Option<Loc>,
}

/// Assertion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertStmt {
    pub cond: Expr,
    pub message: Option<Expr>,
    pub span: Option<Loc>,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl Stmt {
    pub fn span(&self) -> Option<&Loc> {
        match self {
            Stmt::LocalVar(s) => s.span.as_ref(),
            Stmt::Assign(s) => s.span.as_ref(),
            Stmt::AugAssign(s) => s.span.as_ref(),
            Stmt::Expr(s) => s.span.as_ref(),
            Stmt::If(s) => s.span.as_ref(),
            Stmt::While(s) => s.span.as_ref(),
            Stmt::For(s) => s.span.as_ref(),
            Stmt::Return(s) => s.span.as_ref(),
            Stmt::Revert(s) => s.span.as_ref(),
            Stmt::Assert(s) => s.span.as_ref(),
            Stmt::Break | Stmt::Continue | Stmt::Block(_) => None,
            Stmt::Dialect(_) => None,
        }
    }
}

impl From<LocalVarStmt> for Stmt {
    fn from(s: LocalVarStmt) -> Self {
        Stmt::LocalVar(s)
    }
}
impl From<AssignStmt> for Stmt {
    fn from(s: AssignStmt) -> Self {
        Stmt::Assign(s)
    }
}
impl From<AugAssignStmt> for Stmt {
    fn from(s: AugAssignStmt) -> Self {
        Stmt::AugAssign(s)
    }
}
impl From<ExprStmt> for Stmt {
    fn from(s: ExprStmt) -> Self {
        Stmt::Expr(s)
    }
}
impl From<IfStmt> for Stmt {
    fn from(s: IfStmt) -> Self {
        Stmt::If(s)
    }
}
impl From<WhileStmt> for Stmt {
    fn from(s: WhileStmt) -> Self {
        Stmt::While(s)
    }
}
impl From<ForStmt> for Stmt {
    fn from(s: ForStmt) -> Self {
        Stmt::For(s)
    }
}
impl From<ReturnStmt> for Stmt {
    fn from(s: ReturnStmt) -> Self {
        Stmt::Return(s)
    }
}
impl From<RevertStmt> for Stmt {
    fn from(s: RevertStmt) -> Self {
        Stmt::Revert(s)
    }
}
impl From<AssertStmt> for Stmt {
    fn from(s: AssertStmt) -> Self {
        Stmt::Assert(s)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display implementations
// ═══════════════════════════════════════════════════════════════════

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::LocalVar(s) => write!(f, "{s}"),
            Stmt::Assign(s) => write!(f, "{s}"),
            Stmt::AugAssign(s) => write!(f, "{s}"),
            Stmt::Expr(s) => write!(f, "{s}"),
            Stmt::If(s) => write!(f, "{s}"),
            Stmt::While(s) => write!(f, "{s}"),
            Stmt::For(s) => write!(f, "{s}"),
            Stmt::Return(s) => write!(f, "{s}"),
            Stmt::Revert(s) => write!(f, "{s}"),
            Stmt::Assert(s) => write!(f, "{s}"),
            Stmt::Break => write!(f, "break;"),
            Stmt::Continue => write!(f, "continue;"),
            Stmt::Block(stmts) => {
                writeln!(f, "{{")?;
                for s in stmts {
                    writeln!(f, "{}", format!("{s}").indent(2))?;
                }
                write!(f, "}}")
            }
            Stmt::Dialect(s) => write!(f, "{s}"),
        }
    }
}

impl Display for LocalVarStmt {
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

impl Display for AssignStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {};", self.lhs, self.rhs)
    }
}

impl Display for AugAssignStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}= {};", self.lhs, self.op, self.rhs)
    }
}

impl Display for ExprStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{};", self.expr)
    }
}

impl Display for IfStmt {
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

impl Display for WhileStmt {
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

impl Display for ForStmt {
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

impl Display for ReturnStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(e) => write!(f, "return {e};"),
            None => write!(f, "return;"),
        }
    }
}

impl Display for RevertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<_> = self.args.iter().map(|a| a.to_string()).collect();
        match &self.error {
            Some(error) => write!(f, "revert {error}({});", args.join(", ")),
            None => write!(f, "revert({});", args.join(", ")),
        }
    }
}

impl Display for AssertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "assert({})", self.cond)?;
        if let Some(msg) = &self.message {
            write!(f, ", {msg}")?;
        }
        write!(f, ";")
    }
}
