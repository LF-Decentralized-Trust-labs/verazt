//! tx.origin detector.
//!
//! Detects dangerous usage of tx.origin for authentication.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for tx.origin usage.
pub struct TxOriginDetector;

impl TxOriginDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TxOriginDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for TxOriginDetector {
    fn id(&self) -> &'static str {
        "tx-origin"
    }

    fn name(&self) -> &'static str {
        "Dangerous use of tx.origin"
    }

    fn description(&self) -> &'static str {
        "Using tx.origin for authentication is vulnerable to phishing attacks. \
         An attacker can trick a user into calling a malicious contract that then \
         calls the vulnerable contract, and tx.origin will be the user's address."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![] // Pattern-based, no passes required
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

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, &mut bugs);
        }
        
        bugs
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

impl TxOriginDetector {
    fn visit_source_unit(&self, source_unit: &SourceUnit, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.visit_contract(contract, bugs);
                }
                SourceUnitElem::Func(func) => {
                    self.visit_function(func, bugs);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_function(func, bugs);
            }
        }
    }

    fn visit_function(&self, func: &FuncDef, bugs: &mut Vec<Bug>) {
        if let Some(body) = &func.body {
            self.visit_block(body, bugs);
        }
    }

    fn visit_block(&self, block: &Block, bugs: &mut Vec<Bug>) {
        for stmt in &block.body {
            self.visit_stmt(stmt, bugs);
        }
    }

    fn visit_stmt(&self, stmt: &Stmt, bugs: &mut Vec<Bug>) {
        match stmt {
            Stmt::Block(block) => self.visit_block(block, bugs),
            Stmt::If(if_stmt) => {
                self.check_expr(&if_stmt.condition, bugs);
                self.visit_stmt(&if_stmt.true_branch, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.visit_stmt(false_br, bugs);
                }
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.visit_stmt(pre, bugs);
                }
                if let Some(cond) = &for_stmt.condition {
                    self.check_expr(cond, bugs);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post, bugs);
                }
                self.visit_stmt(&for_stmt.body, bugs);
            }
            Stmt::While(while_stmt) => {
                self.check_expr(&while_stmt.condition, bugs);
                self.visit_stmt(&while_stmt.body, bugs);
            }
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body, bugs);
                self.check_expr(&do_while.condition, bugs);
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr, bugs);
            }
            Stmt::VarDecl(var_decl) => {
                if let Some(value) = &var_decl.value {
                    self.check_expr(value, bugs);
                }
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_expr(expr, bugs);
                }
            }
            Stmt::Try(try_stmt) => {
                self.check_expr(&try_stmt.guarded_expr, bugs);
                self.visit_block(&try_stmt.body, bugs);
                for clause in &try_stmt.catch_clauses {
                    self.visit_block(&clause.body, bugs);
                }
            }
            _ => {}
        }
    }

    fn check_expr(&self, expr: &Expr, bugs: &mut Vec<Bug>) {
        if let Some(loc) = self.find_tx_origin(expr) {
            let bug = create_bug(
                self,
                Some("tx.origin used for authentication. Consider using msg.sender instead."),
                loc,
            );
            bugs.push(bug);
        }
        
        // Recursively check sub-expressions
        match expr {
            Expr::Binary(bin) => {
                self.check_expr(&bin.left, bugs);
                self.check_expr(&bin.right, bugs);
            }
            Expr::Unary(unary) => {
                self.check_expr(&unary.body, bugs);
            }
            Expr::Call(call) => {
                self.check_expr(&call.callee, bugs);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.check_expr(arg, bugs);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.check_expr(&arg.value, bugs);
                        }
                    }
                }
            }
            Expr::Member(member) => {
                // Don't double-report if this is the tx.origin we found
                if !self.is_tx_origin(expr) {
                    self.check_expr(&member.base, bugs);
                }
            }
            Expr::Index(index) => {
                self.check_expr(&index.base_expr, bugs);
                if let Some(idx) = &index.index {
                    self.check_expr(idx, bugs);
                }
            }
            Expr::Conditional(cond) => {
                self.check_expr(&cond.cond, bugs);
                self.check_expr(&cond.true_br, bugs);
                self.check_expr(&cond.false_br, bugs);
            }
            Expr::Tuple(tuple) => {
                for elem in tuple.elems.iter().flatten() {
                    self.check_expr(elem, bugs);
                }
            }
            Expr::Assign(assign) => {
                self.check_expr(&assign.left, bugs);
                self.check_expr(&assign.right, bugs);
            }
            _ => {}
        }
    }

    fn find_tx_origin(&self, expr: &Expr) -> Option<Loc> {
        if self.is_tx_origin(expr) {
            if let Expr::Member(member) = expr {
                return member.loc;
            }
        }
        None
    }

    fn is_tx_origin(&self, expr: &Expr) -> bool {
        if let Expr::Member(member) = expr {
            if let Expr::Ident(base) = member.base.as_ref() {
                return base.name.base.as_str() == "tx" 
                    && member.member.base.as_str() == "origin";
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_origin_detector() {
        let detector = TxOriginDetector::new();
        assert_eq!(detector.id(), "tx-origin");
        assert_eq!(detector.swc_ids(), vec![115]);
    }
}
