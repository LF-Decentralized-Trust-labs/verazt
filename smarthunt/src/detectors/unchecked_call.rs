//! Unchecked call detector.
//!
//! Detects unchecked return values from low-level calls.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for unchecked low-level calls.
pub struct UncheckedCallDetector;

impl UncheckedCallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UncheckedCallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for UncheckedCallDetector {
    fn id(&self) -> &'static str {
        "unchecked-call"
    }

    fn name(&self) -> &'static str {
        "Unchecked Low-Level Call"
    }

    fn description(&self) -> &'static str {
        "Low-level calls (call, delegatecall, staticcall, send) return a boolean \
         indicating success or failure. If this return value is not checked, \
         the contract may continue execution even when the call failed."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]
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
        vec![252] // CWE-252: Unchecked Return Value
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![104] // SWC-104: Unchecked Call Return Value
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, &mut bugs);
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Always check the return value of low-level calls. Use require() to revert \
         on failure, or handle the failure case appropriately."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-104",
        ]
    }
}

impl UncheckedCallDetector {
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
            
            Stmt::Expr(expr_stmt) => {
                // Check if this is an unchecked low-level call
                if let Some(loc) = self.find_unchecked_call(&expr_stmt.expr) {
                    let bug = create_bug(
                        self,
                        Some("Return value of low-level call not checked."),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
            
            Stmt::If(if_stmt) => {
                self.visit_stmt(&if_stmt.true_branch, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.visit_stmt(false_br, bugs);
                }
            }
            
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.visit_stmt(pre, bugs);
                }
                self.visit_stmt(&for_stmt.body, bugs);
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post, bugs);
                }
            }
            
            Stmt::While(while_stmt) => {
                self.visit_stmt(&while_stmt.body, bugs);
            }
            
            Stmt::DoWhile(do_while) => {
                self.visit_stmt(&do_while.body, bugs);
            }
            
            Stmt::Try(try_stmt) => {
                self.visit_block(&try_stmt.body, bugs);
                for clause in &try_stmt.catch_clauses {
                    self.visit_block(&clause.body, bugs);
                }
            }
            
            _ => {}
        }
    }

    fn find_unchecked_call(&self, expr: &Expr) -> Option<Loc> {
        // Only check for direct call expressions (not assignments or conditions)
        match expr {
            Expr::Call(call) => {
                if self.is_low_level_call(call) {
                    return call.loc;
                }
            }
            // CallOpts like addr.call{value: x}("") are also unchecked if not assigned
            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    let method = member.member.base.as_str();
                    if matches!(method, "call" | "delegatecall" | "staticcall" | "send") {
                        return call_opts.loc;
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn is_low_level_call(&self, call: &solidity::ast::CallExpr) -> bool {
        if let Expr::Member(member) = call.callee.as_ref() {
            let method = member.member.base.as_str();
            // Note: transfer is not here because it reverts on failure
            return matches!(method, "call" | "delegatecall" | "staticcall" | "send");
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unchecked_call_detector() {
        let detector = UncheckedCallDetector::new();
        assert_eq!(detector.id(), "unchecked-call");
        assert_eq!(detector.swc_ids(), vec![104]);
    }
}
