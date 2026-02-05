//! Low-level call detector.
//!
//! Detects usage of low-level calls like call, delegatecall, staticcall.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractElem, Expr, Loc, Stmt};

/// Detector for low-level calls.
pub struct LowLevelCallDetector;

impl LowLevelCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LowLevelCallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for LowLevelCallDetector {
    fn id(&self) -> &'static str {
        "low-level-call"
    }

    fn name(&self) -> &'static str {
        "Low Level Call"
    }

    fn description(&self) -> &'static str {
        "Low-level calls (call, delegatecall, staticcall) bypass type checking \
         and can lead to unexpected behavior if not handled properly."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![749] // CWE-749: Exposed Dangerous Method or Function
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![] // No specific SWC for this
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let solidity::ast::SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(&contract, &mut bugs);
                }
            }
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Prefer high-level function calls when possible. If low-level calls are \
         necessary, ensure proper validation of return values and use reentrancy guards."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://docs.soliditylang.org/en/latest/units-and-global-variables.html#members-of-address-types",
        ]
    }
}

impl LowLevelCallDetector {
    fn check_contract(&self, contract: &solidity::ast::ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if let Some(body) = &func.body {
                    self.check_block(body, bugs);
                }
            }
        }
    }

    fn check_block(&self, block: &solidity::ast::Block, bugs: &mut Vec<Bug>) {
        for s in &block.body {
            self.check_statement(s, bugs);
        }
    }

    fn check_statement(&self, stmt: &Stmt, bugs: &mut Vec<Bug>) {
        match stmt {
            Stmt::Block(block) => {
                self.check_block(block, bugs);
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr, bugs);
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_expr(expr, bugs);
                }
            }
            Stmt::If(if_stmt) => {
                self.check_expr(&if_stmt.condition, bugs);
                self.check_statement(&if_stmt.true_branch, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.check_statement(false_br, bugs);
                }
            }
            Stmt::While(while_stmt) => {
                self.check_expr(&while_stmt.condition, bugs);
                self.check_statement(&while_stmt.body, bugs);
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.check_statement(pre, bugs);
                }
                if let Some(cond) = &for_stmt.condition {
                    self.check_expr(cond, bugs);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.check_statement(post, bugs);
                }
                self.check_statement(&for_stmt.body, bugs);
            }
            Stmt::VarDecl(var_decl) => {
                if let Some(expr) = &var_decl.value {
                    self.check_expr(expr, bugs);
                }
            }
            Stmt::Try(try_stmt) => {
                self.check_expr(&try_stmt.guarded_expr, bugs);
                self.check_block(&try_stmt.body, bugs);
            }
            Stmt::DoWhile(do_while) => {
                self.check_statement(&do_while.body, bugs);
                self.check_expr(&do_while.condition, bugs);
            }
            Stmt::Asm(_) | Stmt::Emit(_) | Stmt::Revert(_) | Stmt::Break(_) 
            | Stmt::Continue(_) | Stmt::Placeholder(_) | Stmt::Throw(_) => {}
        }
    }

    fn check_expr(&self, expr: &Expr, bugs: &mut Vec<Bug>) {
        match expr {
            Expr::Call(call) => {
                // Check if this is a low-level call
                if let Expr::Member(member) = &*call.callee {
                    let method = member.member.base.as_str();
                    if matches!(method, "call" | "delegatecall" | "staticcall") {
                        let loc = call.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Low-level {} used. Consider using high-level function calls.",
                                method
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                
                // Check callee and arguments
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
                self.check_expr(&member.base, bugs);
            }
            Expr::Index(index) => {
                self.check_expr(&index.base_expr, bugs);
                if let Some(idx) = &index.index {
                    self.check_expr(idx, bugs);
                }
            }
            Expr::Unary(unary) => {
                self.check_expr(&unary.body, bugs);
            }
            Expr::Binary(binary) => {
                self.check_expr(&binary.left, bugs);
                self.check_expr(&binary.right, bugs);
            }
            Expr::Conditional(cond_expr) => {
                self.check_expr(&cond_expr.cond, bugs);
                self.check_expr(&cond_expr.true_br, bugs);
                self.check_expr(&cond_expr.false_br, bugs);
            }
            Expr::Assign(assign) => {
                self.check_expr(&assign.left, bugs);
                self.check_expr(&assign.right, bugs);
            }
            Expr::Tuple(tuple) => {
                for elem in &tuple.elems {
                    if let Some(e) = elem {
                        self.check_expr(e, bugs);
                    }
                }
            }
            Expr::New(_new_expr) => {
                // NewExpr just has a type, no arguments to check
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_level_call_detector() {
        let detector = LowLevelCallDetector::new();
        assert_eq!(detector.id(), "low-level-call");
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
