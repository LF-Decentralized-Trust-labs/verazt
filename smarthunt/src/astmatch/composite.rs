use crate::astmatch::core::{Match, MatchContext, Pattern};
use solidity::ast::{Expr, Stmt};
use std::collections::HashMap;

/// Match if all patterns match
pub struct AndPattern {
    pub patterns: Vec<Box<dyn Pattern>>,
}

impl AndPattern {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
}

impl Pattern for AndPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        let mut combined_captures = HashMap::new();
        let mut loc = None;

        for pattern in &self.patterns {
            let m = pattern.match_expr(expr, ctx)?;
            if loc.is_none() {
                loc = Some(m.loc);
            }
            combined_captures.extend(m.captures);
        }

        Some(Match {
            loc: loc?,
            captures: combined_captures,
            context: ctx.clone(),
        })
    }

    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match> {
        let mut combined_captures = HashMap::new();
        let mut loc = None;

        for pattern in &self.patterns {
            let m = pattern.match_stmt(stmt, ctx)?;
            if loc.is_none() {
                loc = Some(m.loc);
            }
            combined_captures.extend(m.captures);
        }

        Some(Match {
            loc: loc?,
            captures: combined_captures,
            context: ctx.clone(),
        })
    }

    fn name(&self) -> &str {
        "AndPattern"
    }

    fn description(&self) -> &str {
        "Matches if all sub-patterns match"
    }
}

/// Match if any pattern matches
pub struct OrPattern {
    pub patterns: Vec<Box<dyn Pattern>>,
}

impl OrPattern {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
}

impl Pattern for OrPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        for pattern in &self.patterns {
            if let Some(m) = pattern.match_expr(expr, ctx) {
                return Some(m);
            }
        }
        None
    }

    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match> {
        for pattern in &self.patterns {
            if let Some(m) = pattern.match_stmt(stmt, ctx) {
                return Some(m);
            }
        }
        None
    }

    fn name(&self) -> &str {
        "OrPattern"
    }

    fn description(&self) -> &str {
        "Matches if any sub-pattern matches"
    }
}

/// Match if pattern does NOT match
pub struct NotPattern {
    pub inner: Box<dyn Pattern>,
}

impl NotPattern {
    pub fn new(inner: Box<dyn Pattern>) -> Self {
        Self { inner }
    }
}

impl Pattern for NotPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        if self.inner.match_expr(expr, ctx).is_none() {
            // Return a match with no captures
            Some(Match {
                loc: expr.loc(),
                captures: HashMap::new(),
                context: ctx.clone(),
            })
        } else {
            None
        }
    }

    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match> {
        if self.inner.match_stmt(stmt, ctx).is_none() {
            Some(Match {
                loc: stmt.loc(),
                captures: HashMap::new(),
                context: ctx.clone(),
            })
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "NotPattern"
    }

    fn description(&self) -> &str {
        "Matches if the sub-pattern does NOT match"
    }
}

/// Match with additional predicate
pub struct WherePattern {
    pub inner: Box<dyn Pattern>,
    pub predicate: Box<dyn Fn(&Match) -> bool + Send + Sync>,
}

impl WherePattern {
    pub fn new<F>(inner: Box<dyn Pattern>, predicate: F) -> Self
    where
        F: Fn(&Match) -> bool + Send + Sync + 'static,
    {
        Self {
            inner,
            predicate: Box::new(predicate),
        }
    }
}

impl Pattern for WherePattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        let m = self.inner.match_expr(expr, ctx)?;
        if (self.predicate)(&m) {
            Some(m)
        } else {
            None
        }
    }

    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match> {
        let m = self.inner.match_stmt(stmt, ctx)?;
        if (self.predicate)(&m) {
            Some(m)
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "WherePattern"
    }

    fn description(&self) -> &str {
        "Matches if the sub-pattern matches and predicate returns true"
    }
}

/// Match if expression contains a sub-pattern (recursive search)
pub struct ContainsPattern {
    pub inner: Box<dyn Pattern>,
}

impl ContainsPattern {
    pub fn new(inner: Box<dyn Pattern>) -> Self {
        Self { inner }
    }

    fn search_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        // Try direct match first
        if let Some(m) = self.inner.match_expr(expr, ctx) {
            return Some(m);
        }

        // Recursively search sub-expressions
        match expr {
            Expr::Binary(b) => self
                .search_expr(&b.left, ctx)
                .or_else(|| self.search_expr(&b.right, ctx)),
            Expr::Unary(u) => self.search_expr(&u.operand, ctx),
            Expr::Call(c) => {
                self.search_expr(&c.callee, ctx).or_else(|| {
                    c.args
                        .args
                        .iter()
                        .find_map(|arg| self.search_expr(arg, ctx))
                })
            }
            Expr::Member(m) => self.search_expr(&m.object, ctx),
            Expr::Index(i) => self
                .search_expr(&i.object, ctx)
                .or_else(|| self.search_expr(&i.index, ctx)),
            Expr::Conditional(c) => self
                .search_expr(&c.condition, ctx)
                .or_else(|| self.search_expr(&c.true_expr, ctx))
                .or_else(|| self.search_expr(&c.false_expr, ctx)),
            Expr::Assign(a) => self
                .search_expr(&a.lhs, ctx)
                .or_else(|| self.search_expr(&a.rhs, ctx)),
            _ => None,
        }
    }
}

impl Pattern for ContainsPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        self.search_expr(expr, ctx)
    }

    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match> {
        // Search in statement expressions
        match stmt {
            Stmt::Expr(e) => self.search_expr(&e.expr, ctx),
            Stmt::If(i) => self
                .search_expr(&i.condition, ctx)
                .or_else(|| {
                    i.true_branch
                        .stmts
                        .iter()
                        .find_map(|s| self.match_stmt(s, ctx))
                })
                .or_else(|| {
                    i.false_branch
                        .as_ref()
                        .and_then(|fb| fb.stmts.iter().find_map(|s| self.match_stmt(s, ctx)))
                }),
            Stmt::Return(r) => r.value.as_ref().and_then(|v| self.search_expr(v, ctx)),
            Stmt::VarDecl(v) => v.value.as_ref().and_then(|val| self.search_expr(val, ctx)),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        "ContainsPattern"
    }

    fn description(&self) -> &str {
        "Recursively searches for a sub-pattern within an expression or statement"
    }
}
