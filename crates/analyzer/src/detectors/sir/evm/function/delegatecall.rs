//! Delegatecall Detector
//!
//! Detects dangerous usage of delegatecall.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FieldAccessExpr, FunctionDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::AccessControl,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[],
    description: "Detects potentially dangerous delegatecall usage on SIR.",
    id: DetectorId::Delegatecall,
    name: "Dangerous Delegatecall",
    recommendation: "Never delegatecall to user-supplied or untrusted addresses. If using \
         upgradeable proxies, use battle-tested patterns (OpenZeppelin \
         TransparentProxy or UUPS). Ensure storage layouts are identical \
         between proxy and implementation contracts.",
    references: &["https://swcregistry.io/docs/SWC-112"],
    risk_level: RiskLevel::High,
    swc_ids: &[112],
    target: Target::Evm,
};

/// Scan detector for delegatecall usage.
#[derive(Debug, Default)]
pub struct DelegatecallDetector;

impl DelegatecallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for DelegatecallDetector {
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
                if let DialectExpr::Evm(EvmExpr::Delegatecall(e)) = d {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Usage of delegatecall in '{}.{}'. \
                             Delegatecall to an untrusted address can lead \
                             to storage corruption and contract compromise.",
                            self.contract_name, self.func_name
                        )),
                        e.loc.clone(),
                    ));
                }
                visit::default::visit_dialect_expr(self, d);
            }

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                if fa.field == "delegatecall" {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Usage of delegatecall in '{}.{}'. \
                             Delegatecall to an untrusted address can lead \
                             to storage corruption and contract compromise.",
                            self.contract_name, self.func_name
                        )),
                        fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    ));
                }
                visit::default::visit_field_access_expr(self, fa);
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
    fn test_delegatecall_detector() {
        let detector = DelegatecallDetector::new();
        assert_eq!(detector.meta().id, DetectorId::Delegatecall);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
