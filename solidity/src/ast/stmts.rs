use crate::ast::*;
use crate::ast::yul as yast;
use extlib::string::StringExt;
use crate::ast::Loc;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structure representing all statements
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone)]
#[remain::sorted]
pub enum Stmt {
    Asm(AsmStmt),
    Block(Block),
    Break(BreakStmt),
    Continue(ContinueStmt),
    DoWhile(DoWhileStmt),
    Emit(EmitStmt),
    Expr(ExprStmt),
    For(ForStmt),
    If(IfStmt),
    Placeholder(PlaceholderStmt),
    Return(ReturnStmt),
    Revert(RevertStmt),
    Throw(ThrowStmt),
    Try(TryStmt),
    VarDecl(VarDeclStmt),
    While(WhileStmt),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AsmStmt {
    pub id: Option<isize>,
    pub is_evmasm: bool,
    pub asm_flags: Vec<String>,
    pub body: Vec<yast::YulStmt>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BreakStmt {
    pub id: Option<isize>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ContinueStmt {
    pub id: Option<isize>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DoWhileStmt {
    pub id: Option<isize>,
    pub condition: Expr,
    pub body: Box<Stmt>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EmitStmt {
    pub id: Option<isize>,
    pub event: Expr,
    pub args: CallArgs,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ExprStmt {
    pub id: Option<isize>,
    pub expr: Expr,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ForStmt {
    pub id: Option<isize>,
    pub pre_loop: Option<Box<Stmt>>, // Pre-loop
    pub condition: Option<Expr>,
    pub post_loop: Option<Box<Stmt>>, // Post-loop
    pub body: Box<Stmt>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IfStmt {
    pub id: Option<isize>,
    pub condition: Expr,
    pub true_branch: Box<Stmt>,
    pub false_branch: Option<Box<Stmt>>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PlaceholderStmt {
    pub id: Option<isize>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ReturnStmt {
    pub id: Option<isize>,
    pub expr: Option<Expr>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RevertStmt {
    pub id: Option<isize>,
    pub error: Option<Expr>,
    pub args: CallArgs,
    pub loc: Option<Loc>,
}

// Deprecated since Solidity 0.5.0.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ThrowStmt {
    pub id: Option<isize>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TryStmt {
    pub id: Option<isize>,
    pub guarded_expr: Expr,
    pub returns: Vec<VarDecl>,
    pub body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CatchClause {
    pub id: Option<isize>,
    pub error: Option<String>,
    pub params: Vec<VarDecl>,
    pub body: Block,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarDeclStmt {
    pub id: Option<isize>,
    pub var_decls: Vec<Option<VarDecl>>,
    pub value: Option<Expr>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct WhileStmt {
    pub id: Option<isize>,
    pub condition: Expr,
    pub body: Box<Stmt>,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementations for statements
//-------------------------------------------------------------------------

impl Stmt {
    pub fn is_unchecked_block(&self) -> bool {
        match self {
            Stmt::Block(b) => b.unchecked,
            _ => false,
        }
    }

    pub fn loc(&self) -> Option<Loc> {
        match self {
            Stmt::Asm(s) => s.loc,
            Stmt::Block(b) => b.loc,
            Stmt::Break(s) => s.loc,
            Stmt::Continue(s) => s.loc,
            Stmt::DoWhile(s) => s.loc,
            Stmt::Emit(s) => s.loc,
            Stmt::Expr(s) => s.loc,
            Stmt::For(s) => s.loc,
            Stmt::If(s) => s.loc,
            Stmt::Placeholder(s) => s.loc,
            Stmt::Return(s) => s.loc,
            Stmt::Revert(s) => s.loc,
            Stmt::Throw(s) => s.loc,
            Stmt::Try(s) => s.loc,
            Stmt::VarDecl(s) => s.loc,
            Stmt::While(s) => s.loc,
        }
    }

    pub fn id(&self) -> Option<isize> {
        match self {
            Stmt::Asm(s) => s.id,
            Stmt::Block(b) => b.id,
            Stmt::Break(s) => s.id,
            Stmt::Continue(s) => s.id,
            Stmt::DoWhile(s) => s.id,
            Stmt::Emit(s) => s.id,
            Stmt::Expr(s) => s.id,
            Stmt::For(s) => s.id,
            Stmt::If(s) => s.id,
            Stmt::Placeholder(s) => s.id,
            Stmt::Return(s) => s.id,
            Stmt::Revert(s) => s.id,
            Stmt::Throw(s) => s.id,
            Stmt::Try(s) => s.id,
            Stmt::VarDecl(s) => s.id,
            Stmt::While(s) => s.id,
        }
    }
}

impl From<AsmStmt> for Stmt {
    fn from(stmt: AsmStmt) -> Self {
        Stmt::Asm(stmt)
    }
}

impl From<Block> for Stmt {
    fn from(blk: Block) -> Self {
        Stmt::Block(blk)
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

impl From<ForStmt> for Stmt {
    fn from(stmt: ForStmt) -> Self {
        Stmt::For(stmt)
    }
}

impl From<WhileStmt> for Stmt {
    fn from(stmt: WhileStmt) -> Self {
        Stmt::While(stmt)
    }
}

impl From<DoWhileStmt> for Stmt {
    fn from(stmt: DoWhileStmt) -> Self {
        Stmt::DoWhile(stmt)
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

impl From<PlaceholderStmt> for Stmt {
    fn from(stmt: PlaceholderStmt) -> Self {
        Stmt::Placeholder(stmt)
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
            Stmt::Block(b) => write!(f, "{b}"),
            Stmt::Break(s) => write!(f, "{s};"),
            Stmt::Continue(s) => write!(f, "{s};"),
            Stmt::DoWhile(s) => write!(f, "{s}"),
            Stmt::Emit(s) => write!(f, "{s}"),
            Stmt::Expr(e) => write!(f, "{e};"),
            Stmt::For(s) => write!(f, "{s}"),
            Stmt::If(s) => write!(f, "{s}"),
            Stmt::Placeholder(s) => write!(f, "{s};"),
            Stmt::Return(s) => write!(f, "{s}"),
            Stmt::Revert(s) => write!(f, "{s}"),
            Stmt::Throw(s) => write!(f, "{s};"),
            Stmt::Try(s) => write!(f, "{s}"),
            Stmt::VarDecl(v) => write!(f, "{v}"),
            Stmt::While(s) => write!(f, "{s}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Assembly statement
//-------------------------------------------------------------------------

impl AsmStmt {
    pub fn new(
        id: Option<isize>,
        evmasm: bool,
        flags: Vec<String>,
        body: Vec<yast::YulStmt>,
        loc: Option<Loc>,
    ) -> Self {
        AsmStmt { id, is_evmasm: evmasm, asm_flags: flags, body, loc }
    }
}

impl Display for AsmStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "assembly ").ok();
        if self.is_evmasm {
            write!(f, "\"evmasm\" ").ok();
        };
        if self.asm_flags.is_empty() {
            writeln!(f, "{{").ok();
        } else {
            writeln!(f, "{} {{", self.asm_flags.join(" ")).ok();
        }
        let stmts = self
            .body
            .iter()
            .map(|stmt| format!("{stmt}").indent(4))
            .collect::<Vec<String>>()
            .join("\n");
        writeln!(f, "{stmts}\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementation for Break statement
//-------------------------------------------------------------------------

impl BreakStmt {
    pub fn new(id: Option<isize>, loc: Option<Loc>) -> Self {
        BreakStmt { id, loc }
    }
}

impl Display for BreakStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "break")
    }
}

//-------------------------------------------------------------------------
// Implementation for Continue statement
//-------------------------------------------------------------------------

impl ContinueStmt {
    pub fn new(id: Option<isize>, loc: Option<Loc>) -> Self {
        ContinueStmt { id, loc }
    }
}

impl Display for ContinueStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "continue")
    }
}

//-------------------------------------------------------------------------
// Implementation for Do-while statement
//-------------------------------------------------------------------------

impl DoWhileStmt {
    pub fn new(id: Option<isize>, cond: Expr, body: Stmt, loc: Option<Loc>) -> Self {
        DoWhileStmt { id, condition: cond, body: Box::new(body), loc }
    }
}

impl Display for DoWhileStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "do {}\nwhile ({});", self.body, self.condition)
    }
}

//-------------------------------------------------------------------------
// Implementation for Emit statement
//-------------------------------------------------------------------------

impl EmitStmt {
    pub fn new(id: Option<isize>, event: Expr, args: CallArgs, loc: Option<Loc>) -> Self {
        EmitStmt { id, event, args, loc }
    }
}

impl Display for EmitStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "emit {}({});", self.event, self.args)
    }
}

//-------------------------------------------------------------------------
// Implementation for Expression statement
//-------------------------------------------------------------------------

impl ExprStmt {
    pub fn new(id: Option<isize>, expr: Expr, loc: Option<Loc>) -> Self {
        ExprStmt { id, expr, loc }
    }
}

impl Display for ExprStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expr)
    }
}

//-------------------------------------------------------------------------
// Implementation for For statement
//-------------------------------------------------------------------------

impl ForStmt {
    pub fn new(
        id: Option<isize>,
        pre: Option<Stmt>,
        cond: Option<Expr>,
        post: Option<Stmt>,
        body: Stmt,
        loc: Option<Loc>,
    ) -> Self {
        ForStmt {
            id,
            pre_loop: pre.map(Box::new),
            condition: cond,
            post_loop: post.map(Box::new),
            body: Box::new(body),
            loc,
        }
    }
}

impl Display for ForStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pre = match &self.pre_loop {
            Some(pre) => format!("{pre}"),
            None => ";".to_string(),
        };

        let cond = match &self.condition {
            Some(cond) => format!("{cond};"),
            None => ";".to_string(),
        };

        let post = match &self.post_loop {
            Some(post) => format!("{post}"),
            None => "".to_string(),
        };
        let post = match post.strip_suffix(";") {
            Some(output) => output.to_string(),
            None => post,
        };

        write!(f, "for ({} {} {}) {}", pre, cond, post, self.body)
    }
}

//-------------------------------------------------------------------------
// Implementation for If statement
//-------------------------------------------------------------------------

impl IfStmt {
    pub fn new(
        id: Option<isize>,
        cond: Expr,
        true_br: Stmt,
        false_br: Option<Stmt>,
        loc: Option<Loc>,
    ) -> Self {
        IfStmt {
            id,
            condition: cond,
            true_branch: Box::new(true_br),
            false_branch: false_br.map(Box::new),
            loc,
        }
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
// Implementation for Placeholder statement
//-------------------------------------------------------------------------

impl PlaceholderStmt {
    pub fn new(id: Option<isize>, loc: Option<Loc>) -> Self {
        PlaceholderStmt { id, loc }
    }
}

impl Display for PlaceholderStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}

//-------------------------------------------------------------------------
// Implementation for Return statement
//-------------------------------------------------------------------------

impl ReturnStmt {
    pub fn new(id: Option<isize>, expr: Option<Expr>, loc: Option<Loc>) -> Self {
        ReturnStmt { id, expr, loc }
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
// Implementation for Revert statement
//-------------------------------------------------------------------------

impl RevertStmt {
    pub fn new(
        id: Option<isize>,
        error: Option<Expr>,
        arguments: CallArgs,
        loc: Option<Loc>,
    ) -> Self {
        RevertStmt { id, error, args: arguments, loc }
    }
}

impl Display for RevertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            Some(error) => write!(f, "revert {}({});", error, self.args),
            None => write!(f, "revert({});", self.args),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Throw statement
//-------------------------------------------------------------------------

impl ThrowStmt {
    pub fn new(id: Option<isize>, loc: Option<Loc>) -> Self {
        ThrowStmt { id, loc }
    }
}

impl Display for ThrowStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "throw")
    }
}

//-------------------------------------------------------------------------
// Implementation for Try statement
//-------------------------------------------------------------------------

impl TryStmt {
    pub fn new(
        id: Option<isize>,
        exp: Expr,
        returns: Vec<VarDecl>,
        body: Block,
        catches: Vec<CatchClause>,
        loc: Option<Loc>,
    ) -> Self {
        TryStmt { id, guarded_expr: exp, returns, body, catch_clauses: catches, loc }
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
// Implementation for Catch clause
//-------------------------------------------------------------------------

impl CatchClause {
    pub fn new(
        id: Option<isize>,
        error: Option<&str>,
        params: Vec<VarDecl>,
        body: Block,
        loc: Option<Loc>,
    ) -> Self {
        CatchClause { id, error: error.map(|v| v.to_string()), params, body, loc }
    }
}

impl Display for CatchClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "catch ").ok();
        let params = {
            self.params
                .iter()
                .map(|v| format!("{v}"))
                .collect::<Vec<String>>()
                .join(", ")
        };
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
// Implementation for Variable declaration statement
//-------------------------------------------------------------------------

impl VarDeclStmt {
    pub fn new(
        id: Option<isize>,
        vars: Vec<Option<VarDecl>>,
        value: Option<Expr>,
        loc: Option<Loc>,
    ) -> Self {
        VarDeclStmt { id, var_decls: vars, value, loc }
    }
}

impl From<VarDecl> for VarDeclStmt {
    fn from(vdecl: VarDecl) -> Self {
        let loc = vdecl.loc;
        Self::new(None, vec![Some(vdecl)], None, loc)
    }
}

impl Display for VarDeclStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.var_decls.len() == 1 {
            match &self.var_decls[0] {
                None => write!(f, "",).ok(),
                Some(v) => write!(f, "{v}").ok(),
            }
        } else {
            let var_decls = self
                .var_decls
                .iter()
                .map(|v| match v {
                    None => "".to_string(),
                    Some(vdecl) => vdecl.to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "({var_decls})").ok()
        };

        match &self.value {
            None => write!(f, ";"),
            Some(exp) => write!(f, " = {exp};"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for While statement
//-------------------------------------------------------------------------

impl WhileStmt {
    pub fn new(id: Option<isize>, cond: Expr, body: Stmt, loc: Option<Loc>) -> Self {
        WhileStmt { id, condition: cond, body: Box::new(body), loc }
    }
}

impl Display for WhileStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while ({}) {} ", self.condition, self.body)
    }
}
