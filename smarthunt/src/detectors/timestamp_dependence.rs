//! Timestamp Dependence detector.
//!
//! Detects dangerous usage of block.timestamp.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for timestamp dependence.
pub struct TimestampDependenceDetector;

impl TimestampDependenceDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimestampDependenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for TimestampDependenceDetector {
    fn id(&self) -> &'static str {
        "timestamp-dependence"
    }

    fn name(&self) -> &'static str {
        "Timestamp Dependence"
    }

    fn description(&self) -> &'static str {
        "Block.timestamp can be manipulated by miners to a small degree. \
         Using it for critical logic (especially time-based conditions) \
         may be vulnerable to manipulation."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![829] // CWE-829: Inclusion of Functionality from Untrusted Control Sphere
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![116] // SWC-116: Block values as a proxy for time
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, &mut bugs);
        }

        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Avoid using block.timestamp for strict equality checks or as a source \
         of randomness. If time-based logic is needed, consider using block.number \
         with known block times or external time oracles."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-116",
        ]
    }
}

impl TimestampDependenceDetector {
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
            self.visit_block(body, bugs, false);
        }
    }

    fn visit_block(&self, block: &Block, bugs: &mut Vec<Bug>, in_condition: bool) {
        for stmt in &block.body {
            self.visit_stmt(stmt, bugs, in_condition);
        }
    }

    fn visit_stmt(&self, stmt: &Stmt, bugs: &mut Vec<Bug>, in_condition: bool) {
        match stmt {
            Stmt::Block(block) => self.visit_block(block, bugs, in_condition),

            Stmt::If(if_stmt) => {
                // Check condition for timestamp usage
                if let Some(loc) = self.find_timestamp_usage(&if_stmt.condition, true) {
                    let bug = create_bug(
                        self,
                        Some("block.timestamp used in a condition, which can be manipulated by miners."),
                        loc,
                    );
                    bugs.push(bug);
                }

                self.visit_stmt(&if_stmt.true_branch, bugs, false);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.visit_stmt(false_br, bugs, false);
                }
            }

            Stmt::While(while_stmt) => {
                if let Some(loc) = self.find_timestamp_usage(&while_stmt.condition, true) {
                    let bug = create_bug(
                        self,
                        Some("block.timestamp used in a loop condition."),
                        loc,
                    );
                    bugs.push(bug);
                }
                self.visit_stmt(&while_stmt.body, bugs, false);
            }

            Stmt::For(for_stmt) => {
                if let Some(cond) = &for_stmt.condition {
                    if let Some(loc) = self.find_timestamp_usage(cond, true) {
                        let bug = create_bug(
                            self,
                            Some("block.timestamp used in a loop condition."),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                self.visit_stmt(&for_stmt.body, bugs, false);
            }

            Stmt::Expr(expr_stmt) => {
                // Check for require/assert with timestamp
                if let Expr::Call(call) = &expr_stmt.expr {
                    if let Expr::Ident(ident) = call.callee.as_ref() {
                        if matches!(ident.name.base.as_str(), "require" | "assert") {
                            if let solidity::ast::CallArgs::Unnamed(args) = &call.args {
                                if let Some(first_arg) = args.first() {
                                    if let Some(loc) = self.find_timestamp_usage(first_arg, true) {
                                        let bug = create_bug(
                                            self,
                                            Some("block.timestamp used in require/assert condition."),
                                            loc,
                                        );
                                        bugs.push(bug);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn find_timestamp_usage(&self, expr: &Expr, in_comparison: bool) -> Option<Loc> {
        match expr {
            Expr::Member(member) => {
                if let Expr::Ident(base) = member.base.as_ref() {
                    if base.name.base.as_str() == "block"
                        && matches!(member.member.base.as_str(), "timestamp" | "number")
                    {
                        if in_comparison {
                            return member.loc;
                        }
                    }
                }
                self.find_timestamp_usage(&member.base, in_comparison)
            }

            Expr::Binary(binary) => {
                // Comparisons with timestamp are risky
                let is_comparison = matches!(
                    binary.operator,
                    solidity::ast::BinOp::Lt | solidity::ast::BinOp::Gt |
                    solidity::ast::BinOp::Le | solidity::ast::BinOp::Ge |
                    solidity::ast::BinOp::Eq | solidity::ast::BinOp::Ne
                );

                if let Some(loc) = self.find_timestamp_usage(&binary.left, is_comparison) {
                    return Some(loc);
                }
                self.find_timestamp_usage(&binary.right, is_comparison)
            }

            Expr::Unary(unary) => self.find_timestamp_usage(&unary.body, in_comparison),

            Expr::Call(call) => {
                if let Some(loc) = self.find_timestamp_usage(&call.callee, in_comparison) {
                    return Some(loc);
                }
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            if let Some(loc) = self.find_timestamp_usage(arg, in_comparison) {
                                return Some(loc);
                            }
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            if let Some(loc) = self.find_timestamp_usage(&arg.value, in_comparison) {
                                return Some(loc);
                            }
                        }
                    }
                }
                None
            }

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_dependence_detector_new() {
        let detector = TimestampDependenceDetector::new();
        assert_eq!(detector.id(), "timestamp-dependence");
    }
}
