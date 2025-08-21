use crate::ast::*;
use meta::{Loc, Name};
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all expressions
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Lit(Lit),
    Ident(Identifier),
    Member(MemberExpr),
    Call(CallExpr),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallExpr {
    pub callee: Identifier,
    pub args: Vec<Expr>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MemberExpr {
    pub base: Name,
    pub member: Name,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementations for Expressions
//-------------------------------------------------------------------------

impl From<Lit> for Expr {
    fn from(lit: Lit) -> Self {
        Expr::Lit(lit)
    }
}

impl From<Identifier> for Expr {
    fn from(ident: Identifier) -> Self {
        Expr::Ident(ident)
    }
}

impl From<CallExpr> for Expr {
    fn from(expr: CallExpr) -> Self {
        Expr::Call(expr)
    }
}

impl From<MemberExpr> for Expr {
    fn from(expr: MemberExpr) -> Self {
        Expr::Member(expr)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Lit(lit) => write!(f, "{lit}"),
            Expr::Ident(ident) => write!(f, "{ident}"),
            Expr::Call(expr) => write!(f, "{expr}"),
            Expr::Member(expr) => write!(f, "{expr}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Call expression
//-------------------------------------------------------------------------

impl CallExpr {
    pub fn new(function: Identifier, arguments: Vec<Expr>) -> CallExpr {
        CallExpr { callee: function, args: arguments }
    }
}

impl Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}({})", self.callee, args)
    }
}

//-------------------------------------------------------------------------
// Implementations for Member access expression
//-------------------------------------------------------------------------

impl MemberExpr {
    pub fn new(base: Name, member: Name, loc: Option<Loc>) -> Self {
        MemberExpr { base, member, loc }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.member.index = index
    }
}

impl Display for MemberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.base, self.member)
    }
}
