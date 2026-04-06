//! Delegatecall Detector
//!
//! Detects dangerous usage of delegatecall.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, DialectExpr, FieldAccessExpr, FunctionDecl, Module};

/// Scan detector for delegatecall usage.
#[derive(Debug, Default)]
pub struct DelegatecallDetector;

impl DelegatecallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for DelegatecallDetector {
    fn id(&self) -> &'static str {
        "delegatecall"
    }

    fn name(&self) -> &'static str {
        "Dangerous Delegatecall"
    }

    fn description(&self) -> &'static str {
        "Detects potentially dangerous delegatecall usage on SIR."
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
        Confidence::Medium
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![112]
    }

    fn recommendation(&self) -> &'static str {
        "Never delegatecall to user-supplied or untrusted addresses. If using \
         upgradeable proxies, use battle-tested patterns (OpenZeppelin \
         TransparentProxy or UUPS). Ensure storage layouts are identical \
         between proxy and implementation contracts."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-112"]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            detector: &'b DelegatecallDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                if let DialectExpr::Evm(EvmExpr::Delegatecall(e)) = d {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Usage of delegatecall in '{}.{}'. \
                             Delegatecall to an untrusted address can lead \
                             to storage corruption and contract compromise.",
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

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                if fa.field == "delegatecall" {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Usage of delegatecall in '{}.{}'. \
                             Delegatecall to an untrusted address can lead \
                             to storage corruption and contract compromise.",
                            self.contract_name, self.func_name
                        )),
                        fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }
                visit::default::visit_field_access_expr(self, fa);
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
    fn test_delegatecall_detector() {
        let detector = DelegatecallDetector::new();
        assert_eq!(detector.id(), "delegatecall");
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
