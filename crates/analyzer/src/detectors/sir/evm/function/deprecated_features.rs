//! Deprecated Features Detector
//!
//! Detects usage of deprecated Solidity features.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{ContractDecl, FieldAccessExpr, FunctionDecl, Module, VarExpr};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::CodeQuality,
    bug_kind: BugKind::Refactoring,
    confidence: ConfidenceLevel::High,
    cwe_ids: &[],
    description: "Detects usage of deprecated Solidity constructs on SIR.",
    id: DetectorId::Deprecated,
    name: "Deprecated Features",
    recommendation: "Replace deprecated constructs with their modern equivalents: \
         `suicide()` → `selfdestruct()`, `throw` → `revert()`, \
         `sha3()` → `keccak256()`, `msg.gas` → `gasleft()`, \
         `constant` (on functions) → `view` or `pure`.",
    references: &["https://swcregistry.io/docs/SWC-111"],
    risk_level: RiskLevel::Low,
    swc_ids: &[111],
    target: Target::Evm,
};

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
            fn visit_var_expr(&mut self, v: &'a VarExpr) {
                for (deprecated, replacement) in DEPRECATED_IDENTS {
                    if v.name == *deprecated {
                        self.bugs.push(META.bug(
                            Some(&format!(
                                "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                deprecated, self.contract_name, self.func_name, replacement
                            )),
                            v.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        ));
                    }
                }
            }

            fn visit_field_access_expr(&mut self, fa: &'a FieldAccessExpr) {
                for (deprecated, replacement) in DEPRECATED_FIELDS {
                    if fa.field == *deprecated {
                        self.bugs.push(META.bug(
                            Some(&format!(
                                "Deprecated '{}' used in '{}.{}'. Use '{}' instead.",
                                deprecated, self.contract_name, self.func_name, replacement
                            )),
                            fa.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        ));
                    }
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
    fn test_deprecated_features_detector() {
        let detector = DeprecatedFeaturesDetector::new();
        assert_eq!(detector.meta().id, DetectorId::Deprecated);
        assert_eq!(detector.meta().risk_level, RiskLevel::Low);
    }
}
