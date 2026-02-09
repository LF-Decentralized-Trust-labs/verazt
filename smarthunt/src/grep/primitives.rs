use crate::grep::core::{CapturedNode, Match, MatchContext, Pattern};
use solidity::ast::{Expr, Stmt};
use std::collections::HashMap;

/// Match a specific identifier
pub struct IdentPattern {
    pub name: String,
    pub capture_as: Option<String>,
}

impl IdentPattern {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            capture_as: None,
        }
    }

    pub fn capture(mut self, name: impl Into<String>) -> Self {
        self.capture_as = Some(name.into());
        self
    }
}

impl Pattern for IdentPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        if let solidity::ast::Expr::Ident(id) = expr {
            if id.name.base == self.name {
                let mut captures = HashMap::new();
                if let Some(key) = &self.capture_as {
                    captures.insert(key.clone(), CapturedNode::Ident(id.clone()));
                }
                return Some(Match {
                    loc: id.loc,
                    captures,
                    context: ctx.clone(),
                });
            }
        }
        None
    }

    fn match_stmt(&self, _stmt: &Stmt, _ctx: &MatchContext) -> Option<Match> {
        None // Identifiers are expressions, not statements
    }

    fn name(&self) -> &str {
        "IdentPattern"
    }

    fn description(&self) -> &str {
        "Matches a specific identifier by name"
    }
}

/// Match a member access: object.member
pub struct MemberAccessPattern {
    pub object: Box<dyn Pattern>,
    pub member: String,
    pub capture_object_as: Option<String>,
}

impl MemberAccessPattern {
    pub fn new(object: Box<dyn Pattern>, member: impl Into<String>) -> Self {
        Self {
            object,
            member: member.into(),
            capture_object_as: None,
        }
    }

    pub fn capture_object(mut self, name: impl Into<String>) -> Self {
        self.capture_object_as = Some(name.into());
        self
    }
}

impl Pattern for MemberAccessPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        if let solidity::ast::Expr::Member(m) = expr {
            if m.member.base == self.member {
                if let Some(obj_match) = self.object.match_expr(&m.base, ctx) {
                    let mut captures = obj_match.captures;
                    if let Some(key) = &self.capture_object_as {
                        captures.insert(key.clone(), CapturedNode::Expr(m.base.clone()));
                    }
                    return Some(Match {
                        loc: m.loc,
                        captures,
                        context: ctx.clone(),
                    });
                }
            }
        }
        None
    }

    fn match_stmt(&self, _stmt: &Stmt, _ctx: &MatchContext) -> Option<Match> {
        None
    }

    fn name(&self) -> &str {
        "MemberAccessPattern"
    }

    fn description(&self) -> &str {
        "Matches a member access expression (object.member)"
    }
}

/// Match a function call
pub struct CallPattern {
    pub callee: Box<dyn Pattern>,
    pub capture_as: Option<String>,
}

impl CallPattern {
    pub fn new(callee: Box<dyn Pattern>) -> Self {
        Self {
            callee,
            capture_as: None,
        }
    }

    pub fn capture(mut self, name: impl Into<String>) -> Self {
        self.capture_as = Some(name.into());
        self
    }
}

impl Pattern for CallPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        if let Expr::Call(call) = expr {
            if let Some(callee_match) = self.callee.match_expr(&call.callee, ctx) {
                let mut captures = callee_match.captures;
                if let Some(key) = &self.capture_as {
                    captures.insert(key.clone(), CapturedNode::Expr(Box::new(expr.clone())));
                }
                return Some(Match {
                    loc: call.loc,
                    captures,
                    context: ctx.clone(),
                });
            }
        }
        None
    }

    fn match_stmt(&self, _stmt: &Stmt, _ctx: &MatchContext) -> Option<Match> {
        None
    }

    fn name(&self) -> &str {
        "CallPattern"
    }

    fn description(&self) -> &str {
        "Matches a function call expression"
    }
}

/// Match any expression (wildcard)
pub struct AnyExpr {
    pub capture_as: Option<String>,
}

impl AnyExpr {
    pub fn new() -> Self {
        Self { capture_as: None }
    }

    pub fn capture(mut self, name: impl Into<String>) -> Self {
        self.capture_as = Some(name.into());
        self
    }
}

impl Default for AnyExpr {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for AnyExpr {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        let mut captures = HashMap::new();
        if let Some(key) = &self.capture_as {
            captures.insert(key.clone(), CapturedNode::Expr(Box::new(expr.clone())));
        }
        Some(Match {
            loc: expr.loc(),
            captures,
            context: ctx.clone(),
        })
    }

    fn match_stmt(&self, _stmt: &Stmt, _ctx: &MatchContext) -> Option<Match> {
        None
    }

    fn name(&self) -> &str {
        "AnyExpr"
    }

    fn description(&self) -> &str {
        "Matches any expression"
    }
}

/// Match any statement (wildcard)
pub struct AnyStmt {
    pub capture_as: Option<String>,
}

impl AnyStmt {
    pub fn new() -> Self {
        Self { capture_as: None }
    }

    pub fn capture(mut self, name: impl Into<String>) -> Self {
        self.capture_as = Some(name.into());
        self
    }
}

impl Default for AnyStmt {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for AnyStmt {
    fn match_expr(&self, _expr: &Expr, _ctx: &MatchContext) -> Option<Match> {
        None
    }

    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match> {
        let mut captures = HashMap::new();
        if let Some(key) = &self.capture_as {
            captures.insert(key.clone(), CapturedNode::Stmt(Box::new(stmt.clone())));
        }
        Some(Match {
            loc: stmt.loc(),
            captures,
            context: ctx.clone(),
        })
    }

    fn name(&self) -> &str {
        "AnyStmt"
    }

    fn description(&self) -> &str {
        "Matches any statement"
    }
}
