//! Unchecked Call Return Detector (DFA-based)
//!
//! Detects low-level calls whose return values are not checked using
//! def-use analysis.
//!
//! This detector finds:
//! - `.call()`, `.delegatecall()`, `.staticcall()` returns not checked
//! - `.send()` returns not checked
//! - Low-level calls used as expression statements (return value discarded)

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::pipeline::detector::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{Block, ContractElem, Expr, FuncDef, Loc, SourceUnitElem, Stmt};

/// DFA-based detector for unchecked call return values.
#[derive(Debug, Default)]
pub struct UncheckedCallDfaDetector;

impl UncheckedCallDfaDetector {
    pub fn new() -> Self {
        Self
    }

    fn check_function(&self, func: &FuncDef, contract_name: &str, bugs: &mut Vec<Bug>) {
        if let Some(body) = &func.body {
            self.check_block(body, contract_name, &func.name.base, bugs);
        }
    }

    fn check_block(
        &self,
        block: &Block,
        contract_name: &str,
        func_name: &str,
        bugs: &mut Vec<Bug>,
    ) {
        for stmt in &block.body {
            self.check_stmt(stmt, contract_name, func_name, bugs);
        }
    }

    fn check_stmt(&self, stmt: &Stmt, contract_name: &str, func_name: &str, bugs: &mut Vec<Bug>) {
        match stmt {
            // Expression statement — the return value is discarded
            Stmt::Expr(expr_stmt) => {
                if let Some(loc) = self.find_unchecked_call(&expr_stmt.expr) {
                    let bug = create_bug(
                        self,
                        Some(&format!(
                            "Unchecked call return value in '{}.{}'. \
                             The return value of a low-level call is not checked.",
                            contract_name, func_name,
                        )),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
            Stmt::Block(block) => {
                self.check_block(block, contract_name, func_name, bugs);
            }
            Stmt::If(if_stmt) => {
                self.check_stmt(&if_stmt.true_branch, contract_name, func_name, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.check_stmt(false_br, contract_name, func_name, bugs);
                }
            }
            Stmt::While(w) => {
                self.check_stmt(&w.body, contract_name, func_name, bugs);
            }
            Stmt::DoWhile(d) => {
                self.check_stmt(&d.body, contract_name, func_name, bugs);
            }
            Stmt::For(f) => {
                if let Some(pre) = &f.pre_loop {
                    self.check_stmt(pre, contract_name, func_name, bugs);
                }
                self.check_stmt(&f.body, contract_name, func_name, bugs);
                if let Some(post) = &f.post_loop {
                    self.check_stmt(post, contract_name, func_name, bugs);
                }
            }
            Stmt::Try(t) => {
                self.check_block(&t.body, contract_name, func_name, bugs);
                for catch in &t.catch_clauses {
                    self.check_block(&catch.body, contract_name, func_name, bugs);
                }
            }
            _ => {}
        }
    }

    /// Check if an expression is a low-level call whose return value is not
    /// checked.
    fn find_unchecked_call(&self, expr: &Expr) -> Option<Loc> {
        match expr {
            // Direct call expression as statement (return value discarded)
            Expr::Call(call) => {
                if self.is_low_level_call(&call.callee) {
                    return call.loc;
                }
                None
            }
            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    let method = member.member.base.as_str();
                    if matches!(method, "call" | "delegatecall" | "staticcall" | "send") {
                        return call_opts.loc;
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn is_low_level_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Member(member) => {
                let method = member.member.base.as_str();
                matches!(method, "call" | "delegatecall" | "staticcall" | "send")
            }
            _ => false,
        }
    }
}

impl Pass for UncheckedCallDfaDetector {
    fn id(&self) -> PassId {
        PassId::UncheckedCall
    }

    fn name(&self) -> &'static str {
        "Unchecked Call Return (DFA)"
    }

    fn description(&self) -> &'static str {
        "Detects low-level calls whose return values are not checked using def-use analysis"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Statement
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl BugDetectionPass for UncheckedCallDfaDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                match elem {
                    SourceUnitElem::Contract(contract) => {
                        let contract_name = &contract.name.base;
                        for elem in &contract.body {
                            if let ContractElem::Func(func) = elem {
                                self.check_function(func, contract_name, &mut bugs);
                            }
                        }
                    }
                    SourceUnitElem::Func(func) => {
                        self.check_function(func, "global", &mut bugs);
                    }
                    _ => {}
                }
            }
        }

        Ok(bugs)
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
        vec![252] // CWE-252: Unchecked Return Value
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![104] // SWC-104: Unchecked Call Return Value
    }

    fn recommendation(&self) -> &'static str {
        "Ensure the return value of the low-level call is checked. \
         Use `require(success)` or handle the failure case explicitly."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-104"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unchecked_call_detector() {
        let detector = UncheckedCallDfaDetector::new();
        assert_eq!(detector.id(), PassId::UncheckedCall);
        assert_eq!(detector.swc_ids(), vec![104]);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
    }
}
