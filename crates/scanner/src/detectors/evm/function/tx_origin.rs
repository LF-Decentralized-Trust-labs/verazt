//! tx.origin Detector
//!
//! Detects usage of `tx.origin` for authentication.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::Visit;
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl, Module};

/// Scan detector for tx.origin usage.
#[derive(Debug, Default)]
pub struct TxOriginDetector;

impl TxOriginDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for TxOriginDetector {
    fn id(&self) -> &'static str {
        "tx-origin"
    }

    fn name(&self) -> &'static str {
        "Dangerous use of tx.origin"
    }

    fn description(&self) -> &'static str {
        "Using tx.origin for authentication is vulnerable to phishing attacks."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::AccessControl
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> Confidence {
        Confidence::High
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![345]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![115]
    }

    fn recommendation(&self) -> &'static str {
        "Replace `tx.origin` with `msg.sender` for authentication. `tx.origin` \
         returns the original external account, making the contract vulnerable \
         to phishing attacks where a malicious contract relays the call."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-115",
            "https://consensys.github.io/smart-contract-best-practices/development-recommendations/solidity-specific/tx-origin/",
        ]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            detector: &'b TxOriginDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                if let DialectExpr::Evm(EvmExpr::TxOrigin(e)) = d {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "tx.origin used in '{}.{}'. \
                             Consider using msg.sender instead.",
                            self.contract_name, self.func_name
                        )),
                        e.loc.clone(),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: contract.name.clone(),
            func_name: func.name.clone(),
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_origin_detector() {
        let detector = TxOriginDetector::new();
        assert_eq!(detector.id(), "tx-origin");
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
