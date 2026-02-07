use crate::astmatch::core::{Match, MatchContext, Pattern};
use solidity::ast::{ContractDef, ContractElem, Expr, FuncDef, SourceUnit, SourceUnitElem, Stmt};
use std::collections::HashMap;

/// Pattern matcher that runs multiple patterns in one traversal
pub struct PatternMatcher {
    patterns: Vec<(String, Box<dyn Pattern>)>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
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

        let mut visitor = PatternVisitor {
            matcher: self,
            context: ctx.clone(),
            results: &mut results,
        };

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
            Expr::Unary(u) => self.visit_expr(&u.operand),
            Expr::Call(c) => {
                self.visit_expr(&c.callee);
                for arg in c.args.args.iter() {
                    self.visit_expr(arg);
                }
            }
            Expr::Member(m) => self.visit_expr(&m.object),
            Expr::Index(i) => {
                self.visit_expr(&i.object);
                self.visit_expr(&i.index);
            }
            Expr::Conditional(c) => {
                self.visit_expr(&c.condition);
                self.visit_expr(&c.true_expr);
                self.visit_expr(&c.false_expr);
            }
            Expr::Assign(a) => {
                self.visit_expr(&a.lhs);
                self.visit_expr(&a.rhs);
            }
            Expr::Tuple(t) => {
                for e in &t.elems {
                    self.visit_expr(e);
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
                self.visit_block(&i.true_branch);
                if let Some(fb) = &i.false_branch {
                    self.visit_block(fb);
                }
            }
            Stmt::Return(r) => {
                if let Some(e) = &r.value {
                    self.visit_expr(e);
                }
            }
            Stmt::VarDecl(v) => {
                if let Some(val) = &v.value {
                    self.visit_expr(val);
                }
            }
            Stmt::While(w) => {
                let prev_in_loop = self.context.in_loop;
                self.context.in_loop = true;
                self.visit_expr(&w.condition);
                self.visit_block(&w.body);
                self.context.in_loop = prev_in_loop;
            }
            Stmt::DoWhile(d) => {
                let prev_in_loop = self.context.in_loop;
                self.context.in_loop = true;
                self.visit_block(&d.body);
                self.visit_expr(&d.condition);
                self.context.in_loop = prev_in_loop;
            }
            Stmt::For(f) => {
                let prev_in_loop = self.context.in_loop;
                self.context.in_loop = true;
                if let Some(init) = &f.init {
                    self.visit_stmt(init);
                }
                if let Some(cond) = &f.condition {
                    self.visit_expr(cond);
                }
                if let Some(post) = &f.post {
                    self.visit_expr(post);
                }
                self.visit_block(&f.body);
                self.context.in_loop = prev_in_loop;
            }
            Stmt::Block(b) => self.visit_block(b),
            Stmt::Try(t) => {
                self.visit_expr(&t.expr);
                for clause in &t.clauses {
                    self.visit_block(&clause.block);
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
        for stmt in &block.stmts {
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

    fn visit_contract(&mut self, contract: &ContractDef) {
        let prev_contract = self.context.current_contract.clone();
        self.context.current_contract = Some(contract.name.name.clone());

        for elem in &contract.elems {
            match elem {
                ContractElem::FuncDef(f) => self.visit_function(f),
                ContractElem::VarDecl(v) => {
                    if let Some(init) = &v.value {
                        self.visit_expr(init);
                    }
                }
                _ => {}
            }
        }

        self.context.current_contract = prev_contract;
    }

    fn visit_function(&mut self, func: &FuncDef) {
        let prev_function = self.context.current_function.clone();
        self.context.current_function = Some(func.name.name.clone());

        if let Some(body) = &func.body {
            self.visit_block(body);
        }

        self.context.current_function = prev_function;
    }
}
