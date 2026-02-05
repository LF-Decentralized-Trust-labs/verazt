//! Unchecked call detector.
//!
//! Detects unchecked return values from low-level calls.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{Expr, Loc};
use solidity::ast::utils::Visit;

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
        let mut visitor = UncheckedCallVisitor::new(self);
        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }
        visitor.bugs
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

/// Visitor to collect unchecked call bugs.
struct UncheckedCallVisitor<'a, 'b> {
    detector: &'a UncheckedCallDetector,
    bugs: Vec<Bug>,
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> UncheckedCallVisitor<'a, 'b> {
    fn new(detector: &'a UncheckedCallDetector) -> Self {
        Self {
            detector,
            bugs: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b> Visit<'b> for UncheckedCallVisitor<'a, 'b> {
    fn visit_expr_stmt(&mut self, stmt: &'b solidity::ast::ExprStmt) {
        // Check if this is an unchecked low-level call
        if let Some(loc) = self.detector.find_unchecked_call(&stmt.expr) {
            let bug = create_bug(
                self.detector,
                Some("Return value of low-level call not checked."),
                loc,
            );
            self.bugs.push(bug);
        }
        solidity::ast::utils::visit::default::visit_expr_stmt(self, stmt);
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
