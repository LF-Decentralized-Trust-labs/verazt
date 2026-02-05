//! Timestamp Dependence detector.
//!
//! Detects dangerous usage of block.timestamp.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{Expr, Loc};
use solidity::ast::utils::Visit;

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
        let mut visitor = TimestampVisitor::new(self);
        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }
        visitor.bugs
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

/// Visitor to collect timestamp dependence bugs.
struct TimestampVisitor<'a, 'b> {
    detector: &'a TimestampDependenceDetector,
    bugs: Vec<Bug>,
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> TimestampVisitor<'a, 'b> {
    fn new(detector: &'a TimestampDependenceDetector) -> Self {
        Self {
            detector,
            bugs: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b> Visit<'b> for TimestampVisitor<'a, 'b> {
    fn visit_if_stmt(&mut self, stmt: &'b solidity::ast::IfStmt) {
        // Check condition for timestamp usage
        if let Some(loc) = self.detector.find_timestamp_usage(&stmt.condition, true) {
            let bug = create_bug(
                self.detector,
                Some("block.timestamp used in a condition, which can be manipulated by miners."),
                loc,
            );
            self.bugs.push(bug);
        }
        // Continue with default visiting
        solidity::ast::utils::visit::default::visit_if_stmt(self, stmt);
    }

    fn visit_while_stmt(&mut self, stmt: &'b solidity::ast::WhileStmt) {
        if let Some(loc) = self.detector.find_timestamp_usage(&stmt.condition, true) {
            let bug = create_bug(
                self.detector,
                Some("block.timestamp used in a loop condition."),
                loc,
            );
            self.bugs.push(bug);
        }
        solidity::ast::utils::visit::default::visit_while_stmt(self, stmt);
    }

    fn visit_for_stmt(&mut self, stmt: &'b solidity::ast::ForStmt) {
        if let Some(cond) = &stmt.condition {
            if let Some(loc) = self.detector.find_timestamp_usage(cond, true) {
                let bug = create_bug(
                    self.detector,
                    Some("block.timestamp used in a loop condition."),
                    loc,
                );
                self.bugs.push(bug);
            }
        }
        solidity::ast::utils::visit::default::visit_for_stmt(self, stmt);
    }

    fn visit_expr_stmt(&mut self, stmt: &'b solidity::ast::ExprStmt) {
        // Check for require/assert with timestamp
        if let Expr::Call(call) = &stmt.expr {
            if let Expr::Ident(ident) = call.callee.as_ref() {
                if matches!(ident.name.base.as_str(), "require" | "assert") {
                    if let solidity::ast::CallArgs::Unnamed(args) = &call.args {
                        if let Some(first_arg) = args.first() {
                            if let Some(loc) = self.detector.find_timestamp_usage(first_arg, true) {
                                let bug = create_bug(
                                    self.detector,
                                    Some("block.timestamp used in require/assert condition."),
                                    loc,
                                );
                                self.bugs.push(bug);
                            }
                        }
                    }
                }
            }
        }
        solidity::ast::utils::visit::default::visit_expr_stmt(self, stmt);
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
