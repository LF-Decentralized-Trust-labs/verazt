//! tx.origin Detector
//!
//! Detects usage of `tx.origin` for authentication.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::Visit;
use scirs::sir::{ContractDecl, DialectExpr, FunctionDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::AccessControl,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[345],
    description: "Using tx.origin for authentication is vulnerable to phishing attacks.",
    id: DetectorId::TxOrigin,
    name: "Dangerous use of tx.origin",
    recommendation: "Replace `tx.origin` with `msg.sender` for authentication. `tx.origin` \
         returns the original external account, making the contract vulnerable \
         to phishing attacks where a malicious contract relays the call.",
    references: &[
        "https://swcregistry.io/docs/SWC-115",
        "https://consensys.github.io/smart-contract-best-practices/development-recommendations/solidity-specific/tx-origin/",
    ],
    risk_level: RiskLevel::High,
    swc_ids: &[115],
    target: Target::Evm,
};

/// Scan detector for tx.origin usage.
#[derive(Debug, Default)]
pub struct TxOriginDetector;

impl TxOriginDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for TxOriginDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                if let DialectExpr::Evm(EvmExpr::TxOrigin(e)) = d {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "tx.origin used in '{}.{}'. \
                             Consider using msg.sender instead.",
                            self.contract_name, self.func_name
                        )),
                        e.loc.clone(),
                    ));
                }
            }
        }

        let mut visitor = Visitor {
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
        assert_eq!(detector.meta().id, DetectorId::TxOrigin);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
