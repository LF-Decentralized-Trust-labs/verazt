use crate::grep::core::{Match, MatchContext, Pattern};
use solidity::ast::{ContractElem, Expr, SourceUnit, SourceUnitElem, Stmt};
use std::collections::HashMap;

/// Pattern matcher that runs multiple patterns in one traversal
pub struct PatternMatcher {
    patterns: Vec<(String, Box<dyn Pattern>)>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    pub fn add_pattern(&mut self, name: impl Into<String>, pattern: impl Pattern + 'static) {
        self.patterns.push((name.into(), Box::new(pattern)));
    }

    /// Run all patterns on source units
    pub fn match_all(
        &self,
        units: &[SourceUnit],
        ctx: &MatchContext,
    ) -> HashMap<String, Vec<Match>> {
        let mut results: HashMap<String, Vec<Match>> = HashMap::new();

        for (name, _) in &self.patterns {
            results.insert(name.clone(), Vec::new());
        }

        let mut visitor =
            PatternVisitor { matcher: self, context: ctx.clone(), results: &mut results };

        for unit in units {
            visitor.visit_source_unit(unit);
        }

        results
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor for traversing AST and applying patterns
struct PatternVisitor<'a> {
    matcher: &'a PatternMatcher,
    context: MatchContext,
    results: &'a mut HashMap<String, Vec<Match>>,
}

impl<'a> PatternVisitor<'a> {
    fn visit_expr(&mut self, expr: &Expr) {
        // Try all patterns on this expression
        for (name, pattern) in &self.matcher.patterns {
            if let Some(m) = pattern.match_expr(expr, &self.context) {
                self.results.get_mut(name).unwrap().push(m);
            }
        }

        // Recurse into sub-expressions
        match expr {
            Expr::Binary(b) => {
                self.visit_expr(&b.left);
                self.visit_expr(&b.right);
            }
            Expr::Unary(u) => self.visit_expr(&u.body),
            Expr::Call(c) => {
                self.visit_expr(&c.callee);
                match &c.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.visit_expr(arg);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.visit_expr(&arg.value);
                        }
                    }
                }
            }
            Expr::Member(m) => self.visit_expr(&m.base),
            Expr::Index(i) => {
                self.visit_expr(&i.base_expr);
                if let Some(idx) = &i.index {
                    self.visit_expr(idx);
                }
            }
            Expr::Conditional(c) => {
                self.visit_expr(&c.cond);
                self.visit_expr(&c.true_br);
                self.visit_expr(&c.false_br);
            }
            Expr::Assign(a) => {
                self.visit_expr(&a.left);
                self.visit_expr(&a.right);
            }
            Expr::Tuple(t) => {
                for e in &t.elems {
                    if let Some(expr) = e {
                        self.visit_expr(expr);
                    }
                }
            }
            Expr::InlineArray(arr) => {
                for e in &arr.elems {
                    self.visit_expr(e);
                }
            }
            _ => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        // Try all patterns on this statement
        for (name, pattern) in &self.matcher.patterns {
            if let Some(m) = pattern.match_stmt(stmt, &self.context) {
                self.results.get_mut(name).unwrap().push(m);
            }
        }

        // Recurse and visit contained expressions
        match stmt {
            Stmt::Expr(e) => self.visit_expr(&e.expr),
            Stmt::If(i) => {
                self.visit_expr(&i.condition);
                self.visit_stmt(&i.true_branch);
                if let Some(fb) = &i.false_branch {
                    self.visit_stmt(fb);
                }
            }
            Stmt::Return(r) => {
                if let Some(e) = &r.expr {
                    self.visit_expr(e);
                }
            }
            Stmt::VarDecl(v) => {
                if let Some(val) = &v.value {
                    self.visit_expr(val);
                }
            }
            Stmt::While(w) => {
                self.visit_expr(&w.condition);
                self.visit_stmt(&w.body);
            }
            Stmt::DoWhile(d) => {
                self.visit_stmt(&d.body);
                self.visit_expr(&d.condition);
            }
            Stmt::For(f) => {
                if let Some(init) = &f.pre_loop {
                    self.visit_stmt(init);
                }
                if let Some(cond) = &f.condition {
                    self.visit_expr(cond);
                }
                if let Some(post) = &f.post_loop {
                    self.visit_stmt(post);
                }
                self.visit_stmt(&f.body);
            }
            Stmt::Block(b) => self.visit_block(b),
            Stmt::Try(t) => {
                self.visit_expr(&t.guarded_expr);
                for clause in &t.catch_clauses {
                    self.visit_block(&clause.body);
                }
            }
            Stmt::Emit(e) => {
                self.visit_expr(&e.event);
            }
            Stmt::Revert(r) => {
                if let Some(err) = &r.error {
                    self.visit_expr(err);
                }
            }
            _ => {}
        }
    }

    fn visit_block(&mut self, block: &solidity::ast::Block) {
        for stmt in &block.body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_source_unit(&mut self, unit: &SourceUnit) {
        for elem in &unit.elems {
            match elem {
                SourceUnitElem::Contract(c) => self.visit_contract(c),
                SourceUnitElem::Func(f) => self.visit_function(f),
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &solidity::ast::ContractDef) {
        let prev_contract = self.context.current_contract.clone();
        self.context.current_contract = Some(contract.name.base.clone());

        for elem in &contract.body {
            match elem {
                ContractElem::Func(f) => self.visit_function(f),
                ContractElem::Var(v) => {
                    if let Some(val) = &v.value {
                        self.visit_expr(val);
                    }
                }
                _ => {}
            }
        }

        self.context.current_contract = prev_contract;
    }

    fn visit_function(&mut self, func: &solidity::ast::FuncDef) {
        let prev_function = self.context.current_function.clone();
        self.context.current_function = Some(func.name.base.clone());

        if let Some(body) = &func.body {
            self.visit_block(body);
        }

        self.context.current_function = prev_function;
    }
}
