//! Module handling Yul expressions.

use super::*;
use meta::{Loc, Name};
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all expressions
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum YulExpr {
    Lit(YulLit),
    Ident(YulIdentifier),
    Member(YulMemberExpr),
    Call(YulCallExpr),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulCallExpr {
    pub callee: YulIdentifier,
    pub args: Vec<YulExpr>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct YulMemberExpr {
    pub base: Name,
    pub member: Name,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementations for Expressions
//-------------------------------------------------------------------------

impl From<YulLit> for YulExpr {
    fn from(lit: YulLit) -> Self {
        YulExpr::Lit(lit)
    }
}

impl From<YulIdentifier> for YulExpr {
    fn from(ident: YulIdentifier) -> Self {
        YulExpr::Ident(ident)
    }
}

impl From<YulCallExpr> for YulExpr {
    fn from(expr: YulCallExpr) -> Self {
        YulExpr::Call(expr)
    }
}

impl From<YulMemberExpr> for YulExpr {
    fn from(expr: YulMemberExpr) -> Self {
        YulExpr::Member(expr)
    }
}

impl Display for YulExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulExpr::Lit(lit) => write!(f, "{lit}"),
            YulExpr::Ident(ident) => write!(f, "{ident}"),
            YulExpr::Call(expr) => write!(f, "{expr}"),
            YulExpr::Member(expr) => write!(f, "{expr}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Call expression
//-------------------------------------------------------------------------

impl YulCallExpr {
    pub fn new(function: YulIdentifier, arguments: Vec<YulExpr>) -> YulCallExpr {
        YulCallExpr { callee: function, args: arguments }
    }
}

impl Display for YulCallExpr {
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

impl YulMemberExpr {
    pub fn new(base: Name, member: Name, loc: Option<Loc>) -> Self {
        YulMemberExpr { base, member, loc }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.member.index = index
    }
}

impl Display for YulMemberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.base, self.member)
    }
}
