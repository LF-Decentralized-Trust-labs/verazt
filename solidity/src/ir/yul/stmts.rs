//! Module handling Yul IR statements.

use super::*;
use extlib::string::StringExt;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all statements
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum YulIRStmt {
    Assign(YulIRAssignStmt),
    Block(YulIRBlock),
    Break,
    Continue,
    Expr(YulIRExpr),
    For(YulIRForStmt),
    FuncDef(YulIRFuncDef),
    If(YulIRIfStmt),
    Leave,
    Switch(YulIRSwitchStmt),
    VarDecl(YulIRVarDecl),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRAssignStmt {
    pub vars: Vec<YulIRIdentifier>,
    pub value: YulIRExpr,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRForStmt {
    pub pre_loop: YulIRBlock,
    pub condition: YulIRExpr,
    pub post_loop: YulIRBlock,
    pub body: YulIRBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRIfStmt {
    pub cond: YulIRExpr,
    pub body: YulIRBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRSwitchStmt {
    pub expr: YulIRExpr,
    pub values: Vec<YulIRSwitchValue>,
    pub default: Option<YulIRSwitchDefault>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRSwitchValue {
    pub literal: YulIRLit,
    pub body: YulIRBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct YulIRSwitchDefault {
    pub body: YulIRBlock,
}

//-------------------------------------------------------------------------
// Implementations for statements
//-------------------------------------------------------------------------

impl From<YulIRBlock> for YulIRStmt {
    fn from(blk: YulIRBlock) -> Self {
        YulIRStmt::Block(blk)
    }
}

impl From<YulIRFuncDef> for YulIRStmt {
    fn from(func: YulIRFuncDef) -> Self {
        YulIRStmt::FuncDef(func)
    }
}

impl From<YulIRVarDecl> for YulIRStmt {
    fn from(var: YulIRVarDecl) -> Self {
        YulIRStmt::VarDecl(var)
    }
}

impl From<YulIRAssignStmt> for YulIRStmt {
    fn from(stmt: YulIRAssignStmt) -> Self {
        YulIRStmt::Assign(stmt)
    }
}

impl From<YulIRIfStmt> for YulIRStmt {
    fn from(stmt: YulIRIfStmt) -> Self {
        YulIRStmt::If(stmt)
    }
}

impl From<YulIRForStmt> for YulIRStmt {
    fn from(stmt: YulIRForStmt) -> Self {
        YulIRStmt::For(stmt)
    }
}

impl From<YulIRSwitchStmt> for YulIRStmt {
    fn from(stmt: YulIRSwitchStmt) -> Self {
        YulIRStmt::Switch(stmt)
    }
}

impl From<YulIRExpr> for YulIRStmt {
    fn from(expr: YulIRExpr) -> Self {
        YulIRStmt::Expr(expr)
    }
}

impl Display for YulIRStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YulIRStmt::Block(b) => write!(f, "{b}"),
            YulIRStmt::FuncDef(fd) => write!(f, "{fd}"),
            YulIRStmt::VarDecl(v) => write!(f, "{v}"),
            YulIRStmt::Assign(s) => write!(f, "{s}"),
            YulIRStmt::Expr(s) => write!(f, "{s}"),
            YulIRStmt::If(s) => write!(f, "{s}"),
            YulIRStmt::Switch(s) => write!(f, "{s}"),
            YulIRStmt::For(s) => write!(f, "{s}"),
            YulIRStmt::Break => write!(f, "break"),
            YulIRStmt::Continue => write!(f, "continue"),
            YulIRStmt::Leave => write!(f, "leave"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Assignment statement
//-------------------------------------------------------------------------

impl YulIRAssignStmt {
    pub fn new(variables: Vec<YulIRIdentifier>, value: YulIRExpr) -> YulIRAssignStmt {
        YulIRAssignStmt { vars: variables, value }
    }
}

impl Display for YulIRAssignStmt {
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

impl YulIRForStmt {
    pub fn new(pre_loop: YulIRBlock, condition: YulIRExpr, post_loop: YulIRBlock, body: YulIRBlock) -> YulIRForStmt {
        YulIRForStmt { pre_loop, condition, post_loop, body }
    }
}

impl Display for YulIRForStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "for {} {} {} {}", self.pre_loop, self.condition, self.post_loop, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for If statement
//-------------------------------------------------------------------------

impl YulIRIfStmt {
    pub fn new(condition: YulIRExpr, body: YulIRBlock) -> YulIRIfStmt {
        YulIRIfStmt { cond: condition, body }
    }
}

impl Display for YulIRIfStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if {} {}", self.cond, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch statement
//-------------------------------------------------------------------------

impl YulIRSwitchStmt {
    pub fn new(
        expression: YulIRExpr,
        values: Vec<YulIRSwitchValue>,
        default: Option<YulIRSwitchDefault>,
    ) -> YulIRSwitchStmt {
        YulIRSwitchStmt { expr: expression, values, default }
    }
}

impl Display for YulIRSwitchStmt {
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

impl YulIRSwitchValue {
    pub fn new(literal: YulIRLit, body: YulIRBlock) -> YulIRSwitchValue {
        YulIRSwitchValue { literal, body }
    }
}

impl Display for YulIRSwitchValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "case {} ", self.literal).ok();
        write!(f, "{}", format!("{}", self.body).indent(4))
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch default
//-------------------------------------------------------------------------

impl YulIRSwitchDefault {
    pub fn new(body: YulIRBlock) -> YulIRSwitchDefault {
        YulIRSwitchDefault { body }
    }
}

impl Display for YulIRSwitchDefault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "default ").ok();
        write!(f, "{}", format!("{}", self.body).indent(4))
    }
}
