//! tx.origin Detector
//!
//! Detects dangerous usage of tx.origin for authentication.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use solidity::analysis::pass::Pass;
use solidity::analysis::pass_id::PassId;
use solidity::analysis::pass_level::PassLevel;
use solidity::analysis::pass_representation::PassRepresentation;
use solidity::analysis::context::AnalysisContext;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for tx.origin usage.
///
/// Using tx.origin for authentication is vulnerable to phishing attacks.
/// An attacker can trick a user into calling a malicious contract that then
/// calls the vulnerable contract, and tx.origin will be the user's address.
pub struct TxOriginDetector;

impl TxOriginDetector {
    /// Create a new tx.origin detector.
    pub fn new() -> Self {
        Self
    }

    /// Check if an expression is tx.origin.
    fn is_tx_origin(&self, expr: &Expr) -> bool {
        if let Expr::Member(member) = expr {
            if let Expr::Ident(base) = member.base.as_ref() {
                return base.name.base.as_str() == "tx"
                    && member.member.base.as_str() == "origin";
            }
        }
        false
    }

    /// Visit an expression and collect bugs.
    fn visit_expr(&self, expr: &Expr, bugs: &mut Vec<Bug>) {
        // Check if this expression is tx.origin
        if self.is_tx_origin(expr) {
            if let Expr::Member(member) = expr {
                if let Some(loc) = member.loc {
                    let bug = create_bug(
                        self,
                        Some("tx.origin used for authentication. Consider using msg.sender instead."),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
        }

        // Recursively visit sub-expressions
        match expr {
            Expr::Call(call) => {
                self.visit_expr(&call.callee, bugs);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.visit_expr(arg, bugs);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.visit_expr(&arg.value, bugs);
                        }
                    }
                }
            }
            Expr::Binary(b) => {
                self.visit_expr(&b.left, bugs);
                self.visit_expr(&b.right, bugs);
            }
            Expr::Unary(u) => {
                self.visit_expr(&u.body, bugs);
            }
            Expr::Member(m) => {
                self.visit_expr(&m.base, bugs);
            }
            Expr::Index(i) => {
                self.visit_expr(&i.base_expr, bugs);
                if let Some(ref idx) = i.index {
                    self.visit_expr(idx, bugs);
                }
            }
            Expr::Tuple(t) => {
                for e in t.elems.iter().flatten() {
                    self.visit_expr(e, bugs);
                }
            }
            Expr::Conditional(c) => {
                self.visit_expr(&c.cond, bugs);
                self.visit_expr(&c.true_br, bugs);
                self.visit_expr(&c.false_br, bugs);
            }
            Expr::Assign(a) => {
                self.visit_expr(&a.left, bugs);
                self.visit_expr(&a.right, bugs);
            }
            _ => {}
        }
    }

    /// Visit a statement and collect bugs.
    fn visit_stmt(&self, stmt: &Stmt, bugs: &mut Vec<Bug>) {
        match stmt {
            Stmt::Expr(e) => self.visit_expr(&e.expr, bugs),
            Stmt::Block(b) => self.visit_block(b, bugs),
            Stmt::If(i) => {
                self.visit_expr(&i.condition, bugs);
                self.visit_stmt(&i.true_branch, bugs);
                if let Some(ref else_stmt) = i.false_branch {
                    self.visit_stmt(else_stmt, bugs);
                }
            }
            Stmt::While(w) => {
                self.visit_expr(&w.condition, bugs);
                self.visit_stmt(&w.body, bugs);
            }
            Stmt::DoWhile(d) => {
                self.visit_expr(&d.condition, bugs);
                self.visit_stmt(&d.body, bugs);
            }
            Stmt::For(f) => {
                if let Some(ref pre) = f.pre_loop {
                    self.visit_stmt(pre, bugs);
                }
                if let Some(ref cond) = f.condition {
                    self.visit_expr(cond, bugs);
                }
                if let Some(ref post) = f.post_loop {
                    self.visit_stmt(post, bugs);
                }
                self.visit_stmt(&f.body, bugs);
            }
            Stmt::Return(r) => {
                if let Some(ref expr) = r.expr {
                    self.visit_expr(expr, bugs);
                }
            }
            Stmt::VarDecl(v) => {
                if let Some(ref value) = v.value {
                    self.visit_expr(value, bugs);
                }
            }
            Stmt::Emit(e) => {
                self.visit_expr(&e.event, bugs);
                match &e.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.visit_expr(arg, bugs);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.visit_expr(&arg.value, bugs);
                        }
                    }
                }
            }
            Stmt::Revert(r) => {
                if let Some(ref error) = r.error {
                    self.visit_expr(error, bugs);
                }
                match &r.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.visit_expr(arg, bugs);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.visit_expr(&arg.value, bugs);
                        }
                    }
                }
            }
            Stmt::Try(t) => {
                self.visit_expr(&t.guarded_expr, bugs);
                self.visit_block(&t.body, bugs);
                for catch in &t.catch_clauses {
                    self.visit_block(&catch.body, bugs);
                }
            }
            _ => {}
        }
    }

    /// Visit a block and collect bugs.
    fn visit_block(&self, block: &Block, bugs: &mut Vec<Bug>) {
        for stmt in &block.body {
            self.visit_stmt(stmt, bugs);
        }
    }

    /// Visit a function and collect bugs.
    fn visit_func(&self, func: &FuncDef, bugs: &mut Vec<Bug>) {
        if let Some(ref body) = func.body {
            self.visit_block(body, bugs);
        }
    }

    /// Visit a contract and collect bugs.
    fn visit_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_func(func, bugs);
            }
        }
    }

    /// Visit a source unit and collect bugs.
    fn visit_source_unit(&self, source_unit: &SourceUnit, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.visit_contract(contract, bugs);
                }
                SourceUnitElem::Func(func) => {
                    self.visit_func(func, bugs);
                }
                _ => {}
            }
        }
    }
}

impl Default for TxOriginDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for TxOriginDetector {
    fn id(&self) -> PassId {
        PassId::TxOrigin
    }

    fn name(&self) -> &'static str {
        "Dangerous use of tx.origin"
    }

    fn description(&self) -> &'static str {
        "Using tx.origin for authentication is vulnerable to phishing attacks. \
         An attacker can trick a user into calling a malicious contract that then \
         calls the vulnerable contract, and tx.origin will be the user's address."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![] // Pattern-based, no passes required
    }
}

impl BugDetectionPass for TxOriginDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, &mut bugs);
        }

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![345] // CWE-345: Insufficient Verification of Data Authenticity
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![115] // SWC-115: Authorization through tx.origin
    }

    fn recommendation(&self) -> &'static str {
        "Use msg.sender instead of tx.origin for authentication."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-115",
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/solidity-specific/tx-origin/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_origin_detector() {
        let detector = TxOriginDetector::new();
        assert_eq!(detector.id(), PassId::TxOrigin);
        assert_eq!(detector.swc_ids(), vec![115]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
