//! tx.origin detector.
//!
//! Detects dangerous usage of tx.origin for authentication.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{Expr, Loc};
use solidity::ast::utils::Visit;

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
        let mut visitor = TxOriginVisitor::new(self);
        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }
        visitor.bugs
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

/// Visitor to collect tx.origin usage bugs.
struct TxOriginVisitor<'a, 'b> {
    detector: &'a TxOriginDetector,
    bugs: Vec<Bug>,
    _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> TxOriginVisitor<'a, 'b> {
    fn new(detector: &'a TxOriginDetector) -> Self {
        Self {
            detector,
            bugs: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b> Visit<'b> for TxOriginVisitor<'a, 'b> {
    fn visit_expr(&mut self, expr: &'b Expr) {
        if let Some(loc) = self.detector.find_tx_origin(expr) {
            let bug = create_bug(
                self.detector,
                Some("tx.origin used for authentication. Consider using msg.sender instead."),
                loc,
            );
            self.bugs.push(bug);
        }
        // Continue visiting sub-expressions using default visitor
        solidity::ast::utils::visit::default::visit_expr(self, expr);
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
