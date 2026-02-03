//! Module handling Yul statements.

use super::*;
use extlib::string::StringExt;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all statements
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum YulStmt {
    Assign(YulAssignStmt),
    Block(YulBlock),
    Break,
    Continue,
    Expr(YulExpr),
    For(YulForStmt),
    FuncDef(YulFuncDef),
    If(YulIfStmt),
    Leave,
    Switch(YulSwitchStmt),
    VarDecl(YulVarDecl),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulAssignStmt {
    pub vars: Vec<YulIdentifier>,
    pub value: YulExpr,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulForStmt {
    pub pre_loop: YulBlock,
    pub condition: YulExpr,
    pub post_loop: YulBlock,
    pub body: YulBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIfStmt {
    pub cond: YulExpr,
    pub body: YulBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulSwitchStmt {
    pub expr: YulExpr,
    pub values: Vec<YulSwitchValue>,
    pub default: Option<YulSwitchDefault>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulSwitchValue {
    pub literal: YulLit,
    pub body: YulBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulSwitchDefault {
    pub body: YulBlock,
}

//-------------------------------------------------------------------------
// Implementations for statements
//-------------------------------------------------------------------------

impl From<YulBlock> for YulStmt {
    fn from(blk: YulBlock) -> Self {
        YulStmt::Block(blk)
    }
}

impl From<YulFuncDef> for YulStmt {
    fn from(func: YulFuncDef) -> Self {
        YulStmt::FuncDef(func)
    }
}

impl From<YulVarDecl> for YulStmt {
    fn from(var: YulVarDecl) -> Self {
        YulStmt::VarDecl(var)
    }
}

impl From<YulAssignStmt> for YulStmt {
    fn from(stmt: YulAssignStmt) -> Self {
        YulStmt::Assign(stmt)
    }
}

impl From<YulIfStmt> for YulStmt {
    fn from(stmt: YulIfStmt) -> Self {
        YulStmt::If(stmt)
    }
}

impl From<YulForStmt> for YulStmt {
    fn from(stmt: YulForStmt) -> Self {
        YulStmt::For(stmt)
    }
}

impl From<YulSwitchStmt> for YulStmt {
    fn from(stmt: YulSwitchStmt) -> Self {
        YulStmt::Switch(stmt)
    }
}

impl From<YulExpr> for YulStmt {
    fn from(expr: YulExpr) -> Self {
        YulStmt::Expr(expr)
    }
}

impl Display for YulStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulStmt::Block(b) => write!(f, "{b}"),
            YulStmt::FuncDef(fd) => write!(f, "{fd}"),
            YulStmt::VarDecl(v) => write!(f, "{v}"),
            YulStmt::Assign(s) => write!(f, "{s}"),
            YulStmt::Expr(s) => write!(f, "{s}"),
            YulStmt::If(s) => write!(f, "{s}"),
            YulStmt::Switch(s) => write!(f, "{s}"),
            YulStmt::For(s) => write!(f, "{s}"),
            YulStmt::Break => write!(f, "break"),
            YulStmt::Continue => write!(f, "continue"),
            YulStmt::Leave => write!(f, "leave"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Assignment statement
//-------------------------------------------------------------------------

impl YulAssignStmt {
    pub fn new(variables: Vec<YulIdentifier>, value: YulExpr) -> YulAssignStmt {
        YulAssignStmt { vars: variables, value }
    }
}

impl Display for YulAssignStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.vars.is_empty() {
            panic!("Assignment: LHS is empty");
        }
        let vars = self
            .vars
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{} := {}", vars, self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for For statement
//-------------------------------------------------------------------------

impl YulForStmt {
    pub fn new(pre_loop: YulBlock, condition: YulExpr, post_loop: YulBlock, body: YulBlock) -> YulForStmt {
        YulForStmt { pre_loop, condition, post_loop, body }
    }
}

impl Display for YulForStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "for {} {} {} {}", self.pre_loop, self.condition, self.post_loop, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for If statement
//-------------------------------------------------------------------------

impl YulIfStmt {
    pub fn new(condition: YulExpr, body: YulBlock) -> YulIfStmt {
        YulIfStmt { cond: condition, body }
    }
}

impl Display for YulIfStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if {} {}", self.cond, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch statement
//-------------------------------------------------------------------------

impl YulSwitchStmt {
    pub fn new(
        expression: YulExpr,
        values: Vec<YulSwitchValue>,
        default: Option<YulSwitchDefault>,
    ) -> YulSwitchStmt {
        YulSwitchStmt { expr: expression, values, default }
    }
}

impl Display for YulSwitchStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "switch {} ", self.expr).ok();
        for case in &self.values {
            writeln!(f).ok();
            write!(f, "{case}").ok();
        }
        if let Some(case) = &self.default {
            writeln!(f).ok();
            write!(f, "{case}").ok();
        }
        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch value
//-------------------------------------------------------------------------

impl YulSwitchValue {
    pub fn new(literal: YulLit, body: YulBlock) -> YulSwitchValue {
        YulSwitchValue { literal, body }
    }
}

impl Display for YulSwitchValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "case {} ", self.literal).ok();
        write!(f, "{}", format!("{}", self.body).indent(4))
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch default
//-------------------------------------------------------------------------

impl YulSwitchDefault {
    pub fn new(body: YulBlock) -> YulSwitchDefault {
        YulSwitchDefault { body }
    }
}

impl Display for YulSwitchDefault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "default ").ok();
        write!(f, "{}", format!("{}", self.body).indent(4))
    }
}
