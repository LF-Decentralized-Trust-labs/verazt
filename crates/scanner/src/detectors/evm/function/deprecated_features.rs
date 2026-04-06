//! Deprecated Features Detector
//!
//! Detects usage of deprecated Solidity features.

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FieldAccessExpr, FunctionDecl, Module, VarExpr};

const DEPRECATED_IDENTS: &[(&str, &str)] = &[
    ("suicide", "selfdestruct"),
    ("sha3", "keccak256"),
    ("throw", "revert()"),
];

const DEPRECATED_FIELDS: &[(&str, &str)] = &[("callcode", "delegatecall")];

/// Scan detector for deprecated features.
#[derive(Debug, Default)]
pub struct DeprecatedFeaturesDetector;

impl DeprecatedFeaturesDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for DeprecatedFeaturesDetector {
    fn id(&self) -> &'static str {
        "deprecated-features"
    }

    fn name(&self) -> &'static str {
        "Deprecated Features"
    }

    fn description(&self) -> &'static str {
        "Detects usage of deprecated Solidity constructs on SIR."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::CodeQuality
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
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
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![111]
    }

    fn recommendation(&self) -> &'static str {
        "Replace deprecated constructs with their modern equivalents: \
         `suicide()` → `selfdestruct()`, `throw` → `revert()`, \
         `sha3()` → `keccak256()`, `msg.gas` → `gasleft()`, \
         `constant` (on functions) → `view` or `pure`."
    }

    fn references(&self) -> Vec<&'static str> {
        vec!["https://swcregistry.io/docs/SWC-111"]
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            detector: &'b DeprecatedFeaturesDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_var_expr(&mut self, v: &'a VarExpr) {
                for (deprecated, replacement) in DEPRECATED_IDENTS {
                    if v.name == *deprecated {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                deprecated, self.contract_name, self.func_name, replacement
                            )),
                            v.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
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

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                for (deprecated, replacement) in DEPRECATED_FIELDS {
                    if fa.field == *deprecated {
                        self.bugs.push(Bug::new(
                            self.detector.name(),
                            Some(&format!(
                                "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                deprecated, self.contract_name, self.func_name, replacement
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
    fn test_deprecated_features_detector() {
        let detector = DeprecatedFeaturesDetector::new();
        assert_eq!(detector.id(), "deprecated-features");
        assert_eq!(detector.risk_level(), RiskLevel::Low);
    }
}
