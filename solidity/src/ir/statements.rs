use crate::ir::*;
use extlib::string::StringExt;
use meta::{Loc, Name};
use std::fmt::{self, Display};
use yul::ast as yast;

//-------------------------------------------------------------------------
// Data structures representing statements
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Stmt {
    Asm(AsmStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Emit(EmitStmt),
    Expr(ExprStmt),
    If(IfStmt),
    Return(ReturnStmt),
    Revert(RevertStmt),
    Throw(ThrowStmt),
    Try(TryStmt),
    VarDecl(VarDeclStmt),
    Loop(LoopStmt),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AsmStmt {
    pub is_evmasm: bool,
    pub flags: Vec<String>,
    pub yul_stmts: Vec<yast::Stmt>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BreakStmt {
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ContinueStmt {
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EmitStmt {
    pub event: Name,
    pub args: Vec<AtomicExpr>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub true_branch: Block,
    pub false_branch: Option<Block>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct LoopStmt {
    pub pre_loop: Option<Block>, // Pre-loop
    pub condition: Option<Expr>,
    pub post_loop: Option<Block>, // Post-loop
    pub body: Block,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RevertStmt {
    pub error: Option<Name>,
    pub args: Vec<AtomicExpr>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ThrowStmt {
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TryStmt {
    pub guarded_expr: Expr,
    pub returns: Vec<VarDecl>,
    pub body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CatchClause {
    pub error: Option<String>,
    pub params: Vec<VarDecl>,
    pub body: Block,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarDeclStmt {
    pub vars: Vec<Option<VarDecl>>,
    pub value: Option<Expr>,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementations for statements
//-------------------------------------------------------------------------

impl Stmt {
    pub fn loc(&self) -> Option<Loc> {
        match self {
            Stmt::Asm(s) => s.loc,
            Stmt::Break(s) => s.loc,
            Stmt::Continue(s) => s.loc,
            Stmt::Emit(s) => s.loc,
            Stmt::Expr(s) => s.loc,
            Stmt::If(s) => s.loc,
            Stmt::Return(s) => s.loc,
            Stmt::Revert(s) => s.loc,
            Stmt::Throw(s) => s.loc,
            Stmt::Try(s) => s.loc,
            Stmt::VarDecl(s) => s.loc,
            Stmt::Loop(s) => s.loc,
        }
    }
}

impl From<AsmStmt> for Stmt {
    fn from(stmt: AsmStmt) -> Self {
        Stmt::Asm(stmt)
    }
}

impl From<BreakStmt> for Stmt {
    fn from(stmt: BreakStmt) -> Self {
        Stmt::Break(stmt)
    }
}

impl From<ContinueStmt> for Stmt {
    fn from(stmt: ContinueStmt) -> Self {
        Stmt::Continue(stmt)
    }
}

impl From<IfStmt> for Stmt {
    fn from(stmt: IfStmt) -> Self {
        Stmt::If(stmt)
    }
}

impl From<LoopStmt> for Stmt {
    fn from(stmt: LoopStmt) -> Self {
        Stmt::Loop(stmt)
    }
}

impl From<EmitStmt> for Stmt {
    fn from(stmt: EmitStmt) -> Self {
        Stmt::Emit(stmt)
    }
}

impl From<ExprStmt> for Stmt {
    fn from(stmt: ExprStmt) -> Self {
        Stmt::Expr(stmt)
    }
}

impl From<ReturnStmt> for Stmt {
    fn from(stmt: ReturnStmt) -> Self {
        Stmt::Return(stmt)
    }
}

impl From<RevertStmt> for Stmt {
    fn from(stmt: RevertStmt) -> Self {
        Stmt::Revert(stmt)
    }
}

impl From<ThrowStmt> for Stmt {
    fn from(stmt: ThrowStmt) -> Self {
        Stmt::Throw(stmt)
    }
}

impl From<TryStmt> for Stmt {
    fn from(stmt: TryStmt) -> Self {
        Stmt::Try(stmt)
    }
}

impl From<VarDeclStmt> for Stmt {
    fn from(stmt: VarDeclStmt) -> Self {
        Stmt::VarDecl(stmt)
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Asm(s) => write!(f, "{s}"),
            Stmt::Break(s) => write!(f, "{s};"),
            Stmt::Continue(s) => write!(f, "{s};"),
            Stmt::Emit(s) => write!(f, "{s}"),
            Stmt::Expr(s) => write!(f, "{s}"),
            Stmt::Loop(s) => write!(f, "{s}"),
            Stmt::If(s) => write!(f, "{s}"),
            Stmt::Return(s) => write!(f, "{s}"),
            Stmt::Revert(s) => write!(f, "{s}"),
            Stmt::Throw(s) => write!(f, "{s};"),
            Stmt::Try(s) => write!(f, "{s}"),
            Stmt::VarDecl(v) => write!(f, "{v}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Assembly statement
//-------------------------------------------------------------------------

impl AsmStmt {
    pub fn new(
        evmasm: bool,
        flags: Vec<String>,
        stmts: Vec<yast::Stmt>,
        loc: Option<Loc>,
    ) -> Self {
        AsmStmt { is_evmasm: evmasm, flags, yul_stmts: stmts, loc }
    }
}

impl Display for AsmStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "assembly ").ok();
        if self.is_evmasm {
            write!(f, "\"evmasm\" ").ok();
        };
        if self.flags.is_empty() {
            writeln!(f, "{{").ok();
        } else {
            writeln!(f, "{} {{", self.flags.join(" ")).ok();
        }
        let stmts = self
            .yul_stmts
            .iter()
            .map(|stmt| format!("{stmt}").indent(4))
            .collect::<Vec<String>>()
            .join("\n");
        writeln!(f, "{stmts}\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementations for Break statement
//-------------------------------------------------------------------------

impl BreakStmt {
    pub fn new(loc: Option<Loc>) -> Self {
        BreakStmt { loc }
    }
}

impl Display for BreakStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "break")
    }
}

//-------------------------------------------------------------------------
// Implementations for Continue statement
//-------------------------------------------------------------------------

impl ContinueStmt {
    pub fn new(loc: Option<Loc>) -> Self {
        ContinueStmt { loc }
    }
}

impl Display for ContinueStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "continue")
    }
}

//-------------------------------------------------------------------------
// Implementations for Emit statement
//-------------------------------------------------------------------------

impl EmitStmt {
    pub fn new(event: Name, args: Vec<AtomicExpr>, loc: Option<Loc>) -> Self {
        EmitStmt { event, args, loc }
    }
}

impl Display for EmitStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "emit {}({});", self.event, args)
    }
}

//-------------------------------------------------------------------------
// Implementations for Expression statement
//-------------------------------------------------------------------------

impl ExprStmt {
    pub fn new(expr: Expr, loc: Option<Loc>) -> Self {
        ExprStmt { expr, loc }
    }
}

impl Display for ExprStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{};", self.expr)
    }
}

//-------------------------------------------------------------------------
// Implementations for If statement
//-------------------------------------------------------------------------

impl IfStmt {
    pub fn new(cond: Expr, true_br: Block, false_br: Option<Block>, loc: Option<Loc>) -> Self {
        IfStmt { condition: cond, true_branch: true_br, false_branch: false_br, loc }
    }
}

impl Display for IfStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) {}", self.condition, self.true_branch).ok();
        match &self.false_branch {
            None => Ok(()),
            Some(stmt) => write!(f, " else {stmt}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Loop statement
//-------------------------------------------------------------------------

impl LoopStmt {
    pub fn new(
        pre_stmt: Option<Block>,
        condition: Option<Expr>,
        post_stmt: Option<Block>,
        body: Block,
        loc: Option<Loc>,
    ) -> Self {
        LoopStmt { pre_loop: pre_stmt, condition, post_loop: post_stmt, body, loc }
    }
}

impl Display for LoopStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pre = match &self.pre_loop {
            Some(pre) => format!("{pre}"),
            None => "{ }".to_string(),
        };

        let cond = match &self.condition {
            Some(cond) => format!("({cond})"),
            None => "( )".to_string(),
        };

        let post = match &self.post_loop {
            Some(post) => format!("{post}"),
            None => "{ }".to_string(),
        };
        let post = match post.strip_suffix(";") {
            Some(output) => output.to_string(),
            None => post,
        };

        write!(f, "loop {} {} {} {}", pre, cond, post, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementations for Return statement
//-------------------------------------------------------------------------

impl ReturnStmt {
    pub fn new(expr: Option<Expr>, loc: Option<Loc>) -> Self {
        ReturnStmt { expr, loc }
    }
}

impl Display for ReturnStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.expr {
            Some(expr) => write!(f, "return {expr};"),
            None => write!(f, "return;"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Revert statement
//-------------------------------------------------------------------------

impl RevertStmt {
    pub fn new(error: Option<Name>, args: Vec<AtomicExpr>, loc: Option<Loc>) -> Self {
        RevertStmt { error, args, loc }
    }
}

impl Display for RevertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        match &self.error {
            Some(error) => write!(f, "revert {error}({args});"),
            None => write!(f, "revert({args});"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Throw statement
//-------------------------------------------------------------------------

impl ThrowStmt {
    pub fn new(loc: Option<Loc>) -> Self {
        ThrowStmt { loc }
    }
}

impl Display for ThrowStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "throw")
    }
}

//-------------------------------------------------------------------------
// Implementations for Try statement
//-------------------------------------------------------------------------

impl TryStmt {
    pub fn new(
        exp: Expr,
        returns: Vec<VarDecl>,
        body: Block,
        catch_clauses: Vec<CatchClause>,
        loc: Option<Loc>,
    ) -> Self {
        TryStmt { guarded_expr: exp, returns, body, catch_clauses, loc }
    }
}

impl Display for TryStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "try {}", self.guarded_expr).ok();

        if !self.returns.is_empty() {
            let returns = self
                .returns
                .iter()
                .map(|v| format!("{v}"))
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, " returns ({returns})").ok();
        }

        writeln!(f, "{}", self.body).ok();

        let catch_clauses = self
            .catch_clauses
            .iter()
            .map(|clause| format!("{clause}"))
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, " {catch_clauses}")
    }
}

//-------------------------------------------------------------------------
// Implementations for catch clause
//-------------------------------------------------------------------------

impl CatchClause {
    pub fn new(
        error: Option<String>,
        params: Vec<VarDecl>,
        body: Block,
        loc: Option<Loc>,
    ) -> Self {
        CatchClause { error, params, body, loc }
    }
}

impl Display for CatchClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "catch ").ok();
        let params = self
            .params
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<String>>()
            .join(", ");
        match &self.error {
            None => match self.params[..] {
                [] => write!(f, "{}", self.body),
                _ => write!(f, "({}) {}", params, self.body),
            },
            Some(error) => write!(f, "{}({}) {}", error, params, self.body),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for variable declaration statement
//-------------------------------------------------------------------------

impl VarDeclStmt {
    pub fn new(vars: Vec<Option<VarDecl>>, value: Option<Expr>, loc: Option<Loc>) -> Self {
        VarDeclStmt { vars, value, loc }
    }
}

impl Display for VarDeclStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.len() == 1 {
            match &self.vars[0] {
                None => write!(f, "",).ok(),
                Some(v) => write!(f, "{v}").ok(),
            }
        } else {
            let vars = self
                .vars
                .iter()
                .map(|v| match v {
                    None => "".to_string(),
                    Some(vdecl) => vdecl.to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "({vars})").ok()
        };
        match &self.value {
            None => write!(f, ";"),
            Some(exp) => write!(f, " = {exp};"),
        }
    }
}
