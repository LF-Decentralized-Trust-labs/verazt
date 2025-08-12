use crate::ast::*;
use base::string::StringExt;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all statements
//-------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt {
    Assign(AssignStmt),
    Block(Block),
    Break,
    Continue,
    Expr(Expr),
    For(ForStmt),
    FuncDef(FuncDef),
    If(IfStmt),
    Leave,
    Switch(SwitchStmt),
    VarDecl(VarDecl),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AssignStmt {
    pub vars: Vec<Identifier>,
    pub value: Expr,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ForStmt {
    pub pre_loop: Block,
    pub condition: Expr,
    pub post_loop: Block,
    pub body: Block,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IfStmt {
    pub cond: Expr,
    pub body: Block,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwitchStmt {
    pub expr: Expr,
    pub values: Vec<SwitchValue>,
    pub default: Option<SwitchDefault>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwitchValue {
    pub literal: Lit,
    pub body: Block,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwitchDefault {
    pub body: Block,
}

//-------------------------------------------------------------------------
// Implementations for statements
//-------------------------------------------------------------------------

impl From<Block> for Stmt {
    fn from(blk: Block) -> Self {
        Stmt::Block(blk)
    }
}

impl From<FuncDef> for Stmt {
    fn from(func: FuncDef) -> Self {
        Stmt::FuncDef(func)
    }
}

impl From<VarDecl> for Stmt {
    fn from(var: VarDecl) -> Self {
        Stmt::VarDecl(var)
    }
}

impl From<AssignStmt> for Stmt {
    fn from(stmt: AssignStmt) -> Self {
        Stmt::Assign(stmt)
    }
}

impl From<IfStmt> for Stmt {
    fn from(stmt: IfStmt) -> Self {
        Stmt::If(stmt)
    }
}

impl From<ForStmt> for Stmt {
    fn from(stmt: ForStmt) -> Self {
        Stmt::For(stmt)
    }
}

impl From<SwitchStmt> for Stmt {
    fn from(stmt: SwitchStmt) -> Self {
        Stmt::Switch(stmt)
    }
}

impl From<Expr> for Stmt {
    fn from(expr: Expr) -> Self {
        Stmt::Expr(expr)
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Block(b) => write!(f, "{b}"),
            Stmt::FuncDef(fd) => write!(f, "{fd}"),
            Stmt::VarDecl(v) => write!(f, "{v}"),
            Stmt::Assign(s) => write!(f, "{s}"),
            Stmt::Expr(s) => write!(f, "{s}"),
            Stmt::If(s) => write!(f, "{s}"),
            Stmt::Switch(s) => write!(f, "{s}"),
            Stmt::For(s) => write!(f, "{s}"),
            Stmt::Break => write!(f, "break"),
            Stmt::Continue => write!(f, "continue"),
            Stmt::Leave => write!(f, "leave"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Assignment statement
//-------------------------------------------------------------------------

impl AssignStmt {
    pub fn new(variables: Vec<Identifier>, value: Expr) -> AssignStmt {
        AssignStmt { vars: variables, value }
    }
}

impl Display for AssignStmt {
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

impl ForStmt {
    pub fn new(pre_loop: Block, condition: Expr, post_loop: Block, body: Block) -> ForStmt {
        ForStmt { pre_loop, condition, post_loop, body }
    }
}

impl Display for ForStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "for {} {} {} {}", self.pre_loop, self.condition, self.post_loop, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for If statement
//-------------------------------------------------------------------------

impl IfStmt {
    pub fn new(condition: Expr, body: Block) -> IfStmt {
        IfStmt { cond: condition, body }
    }
}

impl Display for IfStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "if {} {}", self.cond, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch statement
//-------------------------------------------------------------------------

impl SwitchStmt {
    pub fn new(
        expression: Expr,
        values: Vec<SwitchValue>,
        default: Option<SwitchDefault>,
    ) -> SwitchStmt {
        SwitchStmt { expr: expression, values, default }
    }
}

impl Display for SwitchStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "switch {}", self.expr).ok();
        for case in &self.values {
            writeln!(f, "{}", format!("{case}").indent(4)).ok();
        }

        if let Some(case) = &self.default {
            writeln!(f, "{}", format!("{case}").indent(4)).ok();
        }

        write!(f, "")
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch value
//-------------------------------------------------------------------------

impl SwitchValue {
    pub fn new(literal: Lit, body: Block) -> SwitchValue {
        SwitchValue { literal, body }
    }
}

impl Display for SwitchValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "case {} {}", self.literal, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Switch default
//-------------------------------------------------------------------------

impl SwitchDefault {
    pub fn new(body: Block) -> SwitchDefault {
        SwitchDefault { body }
    }
}

impl Display for SwitchDefault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "default {}", self.body)
    }
}
