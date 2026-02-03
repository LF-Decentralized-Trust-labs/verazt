//! Module handling Yul IR expressions.

use super::*;
use meta::Name;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all expressions
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum YulIRExpr {
    Lit(YulIRLit),
    Ident(YulIRIdentifier),
    Member(YulIRMemberExpr),
    Call(YulIRCallExpr),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRCallExpr {
    pub callee: YulIRIdentifier,
    pub args: Vec<YulIRExpr>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct YulIRMemberExpr {
    pub base: Name,
    pub member: Name,
}

//-------------------------------------------------------------------------
// Implementations for Expressions
//-------------------------------------------------------------------------

impl From<YulIRLit> for YulIRExpr {
    fn from(lit: YulIRLit) -> Self {
        YulIRExpr::Lit(lit)
    }
}

impl From<YulIRIdentifier> for YulIRExpr {
    fn from(ident: YulIRIdentifier) -> Self {
        YulIRExpr::Ident(ident)
    }
}

impl From<YulIRCallExpr> for YulIRExpr {
    fn from(expr: YulIRCallExpr) -> Self {
        YulIRExpr::Call(expr)
    }
}

impl From<YulIRMemberExpr> for YulIRExpr {
    fn from(expr: YulIRMemberExpr) -> Self {
        YulIRExpr::Member(expr)
    }
}

impl Display for YulIRExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulIRExpr::Lit(lit) => write!(f, "{lit}"),
            YulIRExpr::Ident(ident) => write!(f, "{ident}"),
            YulIRExpr::Call(expr) => write!(f, "{expr}"),
            YulIRExpr::Member(expr) => write!(f, "{expr}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Call expression
//-------------------------------------------------------------------------

impl YulIRCallExpr {
    pub fn new(function: YulIRIdentifier, arguments: Vec<YulIRExpr>) -> YulIRCallExpr {
        YulIRCallExpr { callee: function, args: arguments }
    }
}

impl Display for YulIRCallExpr {
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

impl YulIRMemberExpr {
    pub fn new(base: Name, member: Name) -> Self {
        YulIRMemberExpr { base, member }
    }
}

impl Display for YulIRMemberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.base, self.member)
    }
}
